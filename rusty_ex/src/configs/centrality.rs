use std::collections::HashMap;

use crate::types::{FeatureIndex, FeaturesGraph};

#[derive(Default)]
pub struct Centrality {
    pub measures: Measures,
    pub feat_graph_indices: Vec<FeatureIndex>,
}

#[derive(Default)]
pub struct Measures {
    pub katz: Option<Vec<f64>>,
    pub closeness: Vec<Option<f64>>,
    pub eigenvector: Option<Vec<f64>>,
}

impl Centrality {
    pub fn new(feat_graph: &FeaturesGraph) -> Self {
        let feat_graph_indices = feat_graph
            .graph
            .node_indices()
            .collect::<Vec<FeatureIndex>>();
        let measures = Measures {
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
        let mut measures = Measures::default();
        if let Some(katz) = &self.measures.katz {
            measures.katz = Some(
                katz.iter()
                    .enumerate()
                    .map(|(index, katz)| katz * refiner_hm[&self.feat_graph_indices[index]])
                    .collect(),
            );
        }
        if let Some(eigenvector) = &self.measures.eigenvector {
            measures.eigenvector = Some(
                eigenvector
                    .iter()
                    .enumerate()
                    .map(|(index, eigenvector)| {
                        eigenvector * refiner_hm[&self.feat_graph_indices[index]]
                    })
                    .collect(),
            );
        }
        measures.closeness = self
            .measures
            .closeness
            .iter()
            .enumerate()
            .map(|(index, closeness)| match closeness {
                Some(closeness) => Some(closeness * refiner_hm[&self.feat_graph_indices[index]]),
                None => None,
            })
            .collect();

        Centrality {
            measures,
            feat_graph_indices: self.feat_graph_indices.clone(),
        }
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
