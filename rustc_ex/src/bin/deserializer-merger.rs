#![feature(rustc_private)]

use rustc_ex::types::*;
use rustc_ex::{GLOBAL_FEATURE_NAME, GLOBAL_NODE_ID, GLOBAL_NODE_INDEX};
use rustworkx_core::petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use std::fs::File;

pub struct SuperCollector {
    /// Relationships between all nodes of the AST (both annotated or not)
    ast_graph: AstGraph<SuperAstKey>,
    /// Multigraph storing relationships between features
    features_graph: FeaturesGraph,
    /// Graph storing only annotated artifacts (AST nodes with features)
    artifacts_graph: ArtifactsGraph<SuperArtifactKey>,
}

impl SuperCollector {
    /// Initialize the global scope (AST node, feature node, artifact node)
    fn init_global_scope(&mut self) {
        let ident = Some(GLOBAL_FEATURE_NAME.to_string());
        let node_id = GLOBAL_NODE_ID;
        let feature = Feature {
            name: GLOBAL_FEATURE_NAME.to_string(),
            not: false,
        };
        let features = ComplexFeature::Feature(feature.clone());
        let artifact = SimpleArtifactKey(node_id);

        let index = self.ast_graph.create_node(
            SuperAstKey {
                node_id: SimpleAstKey(node_id),
                krate: GLOBAL_FEATURE_NAME.to_string(),
            },
            ident.clone(),
            features.clone(),
            NodeWeightKind::Block("Global".to_string()),
            NodeWeight::ToBeCalculated,
        );
        assert_eq!(
            index,
            AstIndex::new(GLOBAL_NODE_INDEX),
            "Error: global AST node has an index != 0"
        );

        let index = self.features_graph.create_node(FeatureKey(feature), None);
        assert_eq!(
            index,
            FeatureIndex::new(GLOBAL_NODE_INDEX),
            "Error: global feature node has an index != 0"
        );

        let index = self.artifacts_graph.create_node(
            SuperArtifactKey {
                artifact,
                krate: GLOBAL_FEATURE_NAME.to_string(),
            },
            ident,
            vec![0.into()],
            NodeWeight::ToBeCalculated,
        );
        assert_eq!(
            index,
            ArtifactIndex::new(GLOBAL_NODE_INDEX),
            "Error: global artifact node has an index != 0"
        );
    }

    /// Import an AST graph into self SuperCollector
    fn import_ast_graph(
        &mut self,
        ast_graph: DiGraph<AstNode<SimpleAstKey>, Edge>,
        crate_name: String,
    ) {
        let mut index_map = HashMap::new();
        index_map.insert(
            NodeIndex::new(GLOBAL_NODE_INDEX),
            NodeIndex::new(GLOBAL_NODE_INDEX),
        );

        // create nodes in new graph
        for old_node_index in ast_graph.node_indices() {
            if old_node_index == NodeIndex::new(GLOBAL_NODE_INDEX) {
                continue;
            }
            let old_node = ast_graph
                .node_weight(old_node_index)
                .expect("Node not found importing graph");

            let new_node_index = self.ast_graph.create_node(
                SuperAstKey {
                    node_id: old_node.node_id.clone(),
                    krate: crate_name.clone(),
                },
                old_node.ident.clone(),
                old_node.features.clone(),
                old_node.weight_kind.clone(),
                old_node.weight.clone(),
            );
            index_map.insert(old_node_index, new_node_index);
        }

        // create edges in new graph
        for old_edge_index in ast_graph.edge_indices() {
            let (source, target) = ast_graph
                .edge_endpoints(old_edge_index)
                .expect("Edge not found importing graph");

            let new_source = index_map
                .get(&source)
                .expect("Source node not found in index map");
            let new_target = index_map
                .get(&target)
                .expect("Target node not found in index map");

            let edge_weight = ast_graph
                .edge_weight(old_edge_index)
                .expect("Edge weight not found importing graph")
                .clone();

            self.ast_graph
                .graph
                .add_edge(*new_source, *new_target, edge_weight);
        }
    }

    /// Import a features graph into self SuperCollector
    fn import_features_graph(&mut self, features_graph: DiGraph<FeatureNode, Edge>) {
        let mut index_map: HashMap<FeatureKey, NodeIndex> = HashMap::new();

        // create nodes (only features not already created) in new graph
        for node_index in features_graph.node_indices() {
            let node = features_graph
                .node_weight(node_index)
                .expect("Node not found importing graph");

            if index_map.contains_key(&node.feature) {
                continue;
            }

            let new_node_index = self
                .features_graph
                .create_node(node.feature.clone(), node.weight);
            index_map.insert(node.feature.clone(), new_node_index);
        }

        // create edges in new graph
        for old_edge_index in features_graph.edge_indices() {
            let (source_index, target_index) = features_graph
                .edge_endpoints(old_edge_index)
                .expect("Edge not found importing graph");

            let source = features_graph
                .node_weight(source_index)
                .expect("Source node not found");
            let target = features_graph
                .node_weight(target_index)
                .expect("Target node not found");

            let edge_weight = features_graph
                .edge_weight(old_edge_index)
                .expect("Edge weight not found importing graph")
                .clone();

            eprintln!("{:?} -> {:?}", source.feature, target.feature);

            self.features_graph.graph.add_edge(
                *index_map.get(&source.feature).expect("Feature not found"),
                *index_map.get(&target.feature).expect("Feature not found"),
                edge_weight,
            );
        }
    }

    /// Import an artifacts graph into self SuperCollector
    fn import_artifacts_graph(
        &mut self,
        artifacts_graph: DiGraph<ArtifactNode<SimpleArtifactKey>, Edge>,
        crate_name: String,
    ) {
        let mut index_map = HashMap::new();
        index_map.insert(
            NodeIndex::new(GLOBAL_NODE_INDEX),
            NodeIndex::new(GLOBAL_NODE_INDEX),
        );

        // create nodes in new graph
        for old_node_index in artifacts_graph.node_indices() {
            if old_node_index == NodeIndex::new(GLOBAL_NODE_INDEX) {
                continue;
            }
            let old_node = artifacts_graph
                .node_weight(old_node_index)
                .expect("Node not found importing graph");

            let new_node_index = self.artifacts_graph.create_node(
                SuperArtifactKey {
                    artifact: old_node.artifact.clone(),
                    krate: crate_name.clone(),
                },
                old_node.ident.clone(),
                old_node.features.clone(),
                old_node.weight.clone(),
            );
            index_map.insert(old_node_index, new_node_index);
        }

        // create edges in new graph
        for old_edge_index in artifacts_graph.edge_indices() {
            let (source, target) = artifacts_graph
                .edge_endpoints(old_edge_index)
                .expect("Edge not found importing graph");

            let new_source = index_map
                .get(&source)
                .expect("Source node not found in index map");
            let new_target = index_map
                .get(&target)
                .expect("Target node not found in index map");

            let edge_weight = artifacts_graph
                .edge_weight(old_edge_index)
                .expect("Edge weight not found importing graph")
                .clone();

            self.artifacts_graph
                .graph
                .add_edge(*new_source, *new_target, edge_weight);
        }
    }
}

fn main() {
    // TODO: prendere in maniera decente i file da unire
    let results = vec!["collector1.json", "collector2.json"];

    let mut super_collector = SuperCollector {
        ast_graph: AstGraph::new(),
        features_graph: FeaturesGraph::new(),
        artifacts_graph: ArtifactsGraph::new(),
    };

    super_collector.init_global_scope();

    for file_path in results {
        let file = File::open(file_path).unwrap();
        let collector: SimpleSerialization = serde_json::from_reader(file).unwrap();
        super_collector.import_ast_graph(collector.ast_graph.graph, file_path.to_string());
        super_collector.import_features_graph(collector.features_graph.graph);
        super_collector
            .import_artifacts_graph(collector.artifacts_graph.graph, file_path.to_string());
    }

    super_collector.features_graph.print_dot();
}
