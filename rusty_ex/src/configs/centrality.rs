use std::{collections::HashMap, fmt::Debug};

use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};

use crate::{
    types::{FeatureIndex, FeaturesGraph},
    GLOBAL_DUMMY_INDEX, GLOBAL_NODE_INDEX,
};

/// The method to be selected for the centrality computation.
/// It is meant to be used user-side.
pub enum CentralityMethod {
    Katz,
    Closeness,
    Eigenvector,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum CentralityKind {
    #[default]
    All,
    Katz,
    Closeness,
    Eigenvector,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Centrality<T>
where
    T: Clone,
{
    pub measures: CentralityMeasures,
    // pub feat_graph_indices: Vec<FeatureIndex>,
    pub indices: Vec<T>,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct CentralityMeasures {
    pub katz: Option<Vec<f64>>,
    pub closeness: Vec<Option<f64>>,
    pub eigenvector: Option<Vec<f64>>,
}

impl<T> Centrality<T>
where
    T: Clone,
{
    /// Create a new Centrality struct with the centrality measures computed for the
    /// FeaturesGraph. If remove_dummy is true, the dummy node is removed from the
    /// centrality measures.
    ///
    /// NOTE: The dummy node is the node with the index GLOBAL_DUMMY_INDEX.
    /// If `remove_dummy_and_global` is false, you have to be sure when calling
    /// `refine` that the dummy node and the global node are not present in the
    /// refiner hashmap.
    pub fn new(
        feat_graph: &FeaturesGraph,
        refiner_hm: &HashMap<FeatureIndex, f64>,
        cnf_mapping: &HashMap<String, T>,
        remove_dummy_and_global: bool,
    ) -> Self {
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
            katz: Centrality::<FeatureIndex>::compute_katz(feat_graph),
            closeness: Centrality::<FeatureIndex>::compute_closeness(feat_graph),
            eigenvector: Centrality::<FeatureIndex>::compute_eigenvector(feat_graph),
        };

        let refined_centrality = Centrality::<FeatureIndex>::refine_with_art_map(
            &measures,
            &feat_graph_indices,
            refiner_hm.clone(),
        );
        Centrality::align_indices_with_cnf_map(refined_centrality, feat_graph, cnf_mapping)
    }

    fn align_indices_with_cnf_map(
        centrality: Centrality<FeatureIndex>,
        feat_graph: &FeaturesGraph,
        mapping: &HashMap<String, T>,
    ) -> Centrality<T> {
        let indices = centrality
            .indices
            .iter()
            .map(|index| {
                let feature_name = feat_graph.graph[*index].feature.0.name.clone();
                mapping.get(&feature_name).unwrap().clone()
            })
            .collect();

        Centrality {
            measures: centrality.measures,
            indices,
        }
    }

    fn refine_with_art_map(
        calc_measures: &CentralityMeasures,
        feat_graph_indices: &[FeatureIndex],
        refiner_hm: HashMap<FeatureIndex, f64>,
    ) -> Centrality<FeatureIndex> {
        let mut measures = CentralityMeasures::default();

        // They are ordered in the same way as the feat_graph_indices.
        let refined_values: Vec<&f64> = feat_graph_indices
            .iter()
            .map(|feature_index| refiner_hm.get(feature_index).unwrap())
            .collect();

        if let Some(katz) = &calc_measures.katz {
            measures.katz = Some(
                katz.iter()
                    .zip(refined_values.iter())
                    .map(|(katz, refined_value)| katz * *refined_value)
                    .collect(),
            );
        }

        measures.closeness = calc_measures
            .closeness
            .iter()
            .zip(refined_values.iter())
            .map(|(closeness, refined_value)| {
                closeness
                    .as_ref()
                    .map(|closeness| closeness * *refined_value)
            })
            .collect();

        if let Some(eigenvector) = &calc_measures.eigenvector {
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
            indices: feat_graph_indices.to_vec(),
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
        // The `closeness_centrality` does not work as expected when the network is weighted.
        // rustworkx_core::centrality::closeness_centrality(&feat_graph.graph, true)
        rustworkx_core::centrality::newman_weighted_closeness_centrality(
            &feat_graph.graph,
            true,
            |e| e.weight().weight,
        )
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
}
