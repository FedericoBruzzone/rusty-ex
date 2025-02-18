use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    types::{FeatureIndex, FeaturesGraph},
    GLOBAL_DUMMY_INDEX, GLOBAL_NODE_INDEX,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum CentralityKind {
    #[default]
    All,
    Katz,
    Closeness,
    Eigenvector,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Centrality {
    pub measures: CentralityMeasures,
    pub feat_graph_indices: Vec<FeatureIndex>,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct CentralityMeasures {
    pub katz: Option<Vec<f64>>,
    pub closeness: Vec<Option<f64>>,
    pub eigenvector: Option<Vec<f64>>,
}

impl Centrality {
    /// Create a new Centrality struct with the centrality measures computed for the
    /// FeaturesGraph. If remove_dummy is true, the dummy node is removed from the
    /// centrality measures.
    ///
    /// NOTE: The dummy node is the node with the index GLOBAL_DUMMY_INDEX.
    /// If `remove_dummy_and_global` is false, you have to be sure when calling
    /// `refine` that the dummy node and the global node are not present in the
    /// refiner hashmap.
    pub fn new(feat_graph: &FeaturesGraph, remove_dummy_and_global: bool) -> Self {
        let node_indices = feat_graph.graph.node_indices();
        let feat_graph_indices = if remove_dummy_and_global {
            node_indices
                .filter(|node| {
                    *node != FeatureIndex::new(GLOBAL_DUMMY_INDEX)
                        && *node != FeatureIndex::new(GLOBAL_NODE_INDEX)
                })
                .collect::<Vec<FeatureIndex>>()
        } else {
            node_indices.collect::<Vec<FeatureIndex>>()
        };

        let measures = CentralityMeasures {
            katz: Centrality::compute_katz(feat_graph),
            closeness: Centrality::compute_closeness(feat_graph),
            eigenvector: Centrality::compute_eigenvector(feat_graph),
        };

        Centrality {
            measures,
            feat_graph_indices,
        }
    }

    pub fn refine(&self, refiner_hm: HashMap<FeatureIndex, f64>) -> Self {
        let mut measures = CentralityMeasures::default();

        // They are ordered in the same way as the feat_graph_indices.
        let refined_values: Vec<&f64> = self
            .feat_graph_indices
            .iter()
            .map(|feature_index| refiner_hm.get(feature_index).unwrap())
            .collect();

        if let Some(katz) = &self.measures.katz {
            measures.katz = Some(
                katz.iter()
                    .zip(refined_values.iter())
                    .map(|(katz, refined_value)| katz * *refined_value)
                    .collect(),
            );
        }

        measures.closeness = self
            .measures
            .closeness
            .iter()
            .zip(refined_values.iter())
            .map(|(closeness, refined_value)| {
                closeness
                    .as_ref()
                    .map(|closeness| closeness * *refined_value)
            })
            .collect();

        if let Some(eigenvector) = &self.measures.eigenvector {
            measures.eigenvector = Some(
                eigenvector
                    .iter()
                    .zip(refined_values.iter())
                    .map(|(eigenvector, refined_value)| eigenvector * *refined_value)
                    .collect(),
            );
        }

        Centrality {
            measures,
            feat_graph_indices: self.feat_graph_indices.clone(),
        }
    }

    pub fn katz(&self) -> Option<&Vec<f64>> {
        self.measures.katz.as_ref()
    }

    pub fn closeness(&self) -> &Vec<Option<f64>> {
        &self.measures.closeness
    }

    pub fn eigenvector(&self) -> Option<&Vec<f64>> {
        self.measures.eigenvector.as_ref()
    }

    fn compute_katz(feat_graph: &FeaturesGraph) -> Option<Vec<f64>> {
        let katz: rustworkx_core::Result<Option<Vec<f64>>> =
            rustworkx_core::centrality::katz_centrality(
                &feat_graph.graph,
                |e| Ok(e.weight().weight),
                None,
                None,
                None,
                None,
                None,
            );

        match katz {
            Ok(katz) => katz,
            Err(e) => {
                // TODO: Probabily we should return handle this error gracefully
                panic!("Error computing katz centrality: {:?}", e);
            }
        }
    }

    // The Option around the f64 is because the closeness centrality can fail
    // if the graph is not connected (i.e. there is a node that is not reachable
    // from all other nodes)
    fn compute_closeness(feat_graph: &FeaturesGraph) -> Vec<Option<f64>> {
        rustworkx_core::centrality::closeness_centrality(&feat_graph.graph, true)
    }

    fn compute_eigenvector(feat_graph: &FeaturesGraph) -> Option<Vec<f64>> {
        let eigenvector: rustworkx_core::Result<Option<Vec<f64>>> =
            rustworkx_core::centrality::eigenvector_centrality(
                &feat_graph.graph,
                |e| Ok(e.weight().weight),
                None,
                Some(1e-2),
            );

        match eigenvector {
            Ok(eigenvector) => eigenvector,
            Err(e) => {
                // TODO: Probabily we should return handle this error gracefully
                panic!("Error computing eigenvector centrality: {:?}", e);
            }
        }
    }

    // NOTE: To avoid to keep a reference to the FeaturesGraph, we pass it as an argument
    // to the pretty_print method instead of storing it in the struct. It means that the
    // feature graph passed to the pretty_print method should be the same used to createù
    // the Centrality struct.
    pub fn pretty_print(&self, feat_graph: &FeaturesGraph) {
        let katz_zip = match self.measures.katz.as_ref() {
            Some(katz) => Ok(katz.iter().zip(self.feat_graph_indices.iter())),
            None => Err("Katz centrality not computed"),
        };
        let closeness_zip = self
            .measures
            .closeness
            .iter()
            .zip(self.feat_graph_indices.iter());
        let eigenvector_zip = match self.measures.eigenvector.as_ref() {
            Some(eigenvector) => Ok(eigenvector.iter().zip(self.feat_graph_indices.iter())),
            None => Err("Eigenvector centrality not computed"),
        };

        println!("Centrality measures:");
        println!("Katz centrality:");
        match katz_zip {
            Ok(katz) => {
                for (katz, node) in katz {
                    let feature_node = feat_graph.graph.node_weight(*node).unwrap();
                    println!(
                        "Node: {:?}, Feature: ({:?}, {:?}), centrality: {:.4}",
                        node.index(),
                        feature_node.feature.0.name,
                        feature_node.feature.0.not,
                        katz
                    );
                }
            }
            Err(e) => {
                println!("{}", e);
            }
        }

        println!("Closeness centrality:");
        for (closeness, node) in closeness_zip {
            let feature_node = feat_graph.graph.node_weight(*node).unwrap();
            match closeness {
                Some(closeness) => {
                    println!(
                        "Node: {:?}, Feature: ({:?}, {:?}), centrality: {:.4}",
                        node.index(),
                        feature_node.feature.0.name,
                        feature_node.feature.0.not,
                        closeness
                    );
                }
                None => {
                    println!(
                        "Node: {:?}, Feature: ({:?}, {:?}), centrality: Not connected",
                        node.index(),
                        feature_node.feature.0.name,
                        feature_node.feature.0.not
                    );
                }
            }
        }

        println!("Eigenvector centrality:");
        match eigenvector_zip {
            Ok(eigenvector) => {
                for (eigenvector, node) in eigenvector {
                    let feature_node = feat_graph.graph.node_weight(*node).unwrap();
                    println!(
                        "Node: {:?}, Feature: ({:?}, {:?}), centrality: {:.4}",
                        node.index(),
                        feature_node.feature.0.name,
                        feature_node.feature.0.not,
                        eigenvector
                    );
                }
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}
