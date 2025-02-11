#![feature(rustc_private)]

use clap::Parser;
use rustworkx_core::petgraph::graph::{DiGraph, NodeIndex};
use rusty_ex::types::*;
use rusty_ex::{
    GLOBAL_DUMMY_INDEX, GLOBAL_DUMMY_NAME, GLOBAL_FEATURE_NAME, GLOBAL_NODE_ID, GLOBAL_NODE_INDEX,
};
use std::collections::{HashMap, HashSet};
use std::fs::File;

pub struct SuperCollector {
    /// Relationships between all terms (all pieces of code annotated or not)
    terms_tree: TermsTree<SuperTermKey>,
    /// Multigraph storing relationships between features
    features_graph: FeaturesGraph,
    /// Relationships between all artifacts (terms nodes with features)
    artifacts_tree: ArtifactsTree<SuperArtifactKey>,
}

impl SuperCollector {
    /// Initialize the global scope (term node, feature node, artifact node)
    fn init_global_scope(&mut self) {
        let ident = Some(GLOBAL_FEATURE_NAME.to_string());
        let node_id = GLOBAL_NODE_ID;
        let feature = Feature {
            name: GLOBAL_FEATURE_NAME.to_string(),
            not: false,
        };
        let features = ComplexFeature::Feature(feature.clone());
        let artifact = SimpleArtifactKey(node_id);

        let index = self.terms_tree.create_node(
            SuperTermKey {
                node_id: SimpleTermKey(node_id),
                krate: GLOBAL_FEATURE_NAME.to_string(),
            },
            ident.clone(),
            features.clone(),
            TermWeightKind::Children("Global".to_string()),
            TermWeight::ToBeCalculated,
        );
        assert_eq!(
            index,
            TermIndex::new(GLOBAL_NODE_INDEX),
            "Error: global term node has an index != 0"
        );

        let mut complex_feature = HashSet::new();
        assert_eq!(complex_feature.insert(features.clone()), true);
        let index = self.features_graph.create_node(
            FeatureKey(feature.clone()),
            Some(1.0),
            complex_feature,
        );
        assert_eq!(
            index,
            FeatureIndex::new(GLOBAL_NODE_INDEX),
            "Error: global feature node has an index != 0"
        );

        let index = self.artifacts_tree.create_node(
            SuperArtifactKey {
                artifact,
                krate: GLOBAL_FEATURE_NAME.to_string(),
            },
            ident,
            ComplexFeature::Feature(feature.clone()),
            vec![0.into()],
            TermWeight::ToBeCalculated,
        );
        assert_eq!(
            index,
            ArtifactIndex::new(GLOBAL_NODE_INDEX),
            "Error: global artifact node has an index != 0"
        );

        // create dummy node in features graph for centrality
        let dummy_feature = Feature {
            name: GLOBAL_DUMMY_NAME.to_string(),
            not: false,
        };
        let mut complex_feature = HashSet::new();
        assert_eq!(
            complex_feature.insert(ComplexFeature::Feature(dummy_feature.clone())),
            true
        );
        self.features_graph.create_node(
            FeatureKey(dummy_feature.clone()),
            Some(1.0),
            complex_feature,
        );
        assert_eq!(
            index,
            FeatureIndex::new(GLOBAL_NODE_INDEX),
            "Error: global term node has an index != 0"
        );

        self.features_graph.graph.add_edge(
            FeatureIndex::new(GLOBAL_NODE_INDEX),
            FeatureIndex::new(GLOBAL_DUMMY_INDEX),
            Edge { weight: 1.0 },
        );
    }

    /// Import a Terms Tree into self SuperCollector
    fn import_terms_tree(
        &mut self,
        terms_tree: DiGraph<TermNode<SimpleTermKey>, Edge>,
        crate_name: String,
    ) {
        let mut index_map = HashMap::new();
        index_map.insert(
            NodeIndex::new(GLOBAL_NODE_INDEX),
            NodeIndex::new(GLOBAL_NODE_INDEX),
        );

        // create nodes in new graph
        for old_node_index in terms_tree.node_indices() {
            if old_node_index == NodeIndex::new(GLOBAL_NODE_INDEX) {
                continue;
            }
            let old_node = terms_tree
                .node_weight(old_node_index)
                .expect("Error: node not found importing terms tree");

            let new_node_index = self.terms_tree.create_node(
                SuperTermKey {
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
        for old_edge_index in terms_tree.edge_indices() {
            let (source, target) = terms_tree
                .edge_endpoints(old_edge_index)
                .expect("Error: edge not found importing terms tree");

            let new_source = index_map
                .get(&source)
                .expect("Error: source node not found in terms index map");
            let new_target = index_map
                .get(&target)
                .expect("Error: target node not found in terms index map");

            let edge_weight = terms_tree
                .edge_weight(old_edge_index)
                .expect("Error: edge weight not found importing terms tree")
                .clone();

            self.terms_tree
                .graph
                .add_edge(*new_source, *new_target, edge_weight);
        }
    }

    /// Import a features graph into self SuperCollector
    fn import_features_graph(&mut self, features_graph: DiGraph<FeatureNode, Edge>) {
        // FIXME: stiamo perdendo informazione sulla composizione delle feature
        // viene perso il compelx feature delle feature che già esistono

        // create nodes (only features not already created) in new graph
        let nodes_to_add: Vec<_> = features_graph
            .node_indices()
            .map(|node_index| {
                features_graph
                    .node_weight(node_index)
                    .expect("Error: node not found importing features graph")
            })
            .filter(|node| {
                node.feature.0.name != GLOBAL_DUMMY_NAME
                    && node.feature.0.name != GLOBAL_FEATURE_NAME
            })
            .filter(|node| !self.features_graph.nodes.contains_key(&node.feature))
            .collect();

        for node in nodes_to_add {
            let new_node_index = self.features_graph.create_node(
                node.feature.clone(),
                node.weight,
                node.complex_feature.clone(),
            );

            self.features_graph.graph.add_edge(
                NodeIndex::new(1),
                new_node_index,
                Edge { weight: 1.0 },
            );
        }

        // create edges in new graph
        for old_edge_index in features_graph.edge_indices() {
            let (source_index, target_index) = features_graph
                .edge_endpoints(old_edge_index)
                .expect("Error: edge not found importing features graph");

            let source = features_graph
                .node_weight(source_index)
                .expect("Error: source node not found in features index map");
            let target = features_graph
                .node_weight(target_index)
                .expect("Error: target node not found in features index map");

            if source.feature.0.name == GLOBAL_DUMMY_NAME
                || target.feature.0.name == GLOBAL_DUMMY_NAME
            {
                continue;
            }

            let edge_weight = features_graph
                .edge_weight(old_edge_index)
                .expect("Error: edge weight not found importing features graph")
                .clone();

            self.features_graph.graph.add_edge(
                *self
                    .features_graph
                    .nodes
                    .get(&source.feature)
                    .expect("Error: feature not found importing features graph"),
                *self
                    .features_graph
                    .nodes
                    .get(&target.feature)
                    .expect("Error: feature not found importing features graph"),
                edge_weight,
            );
        }
    }

    /// Import an artifacts tree into self SuperCollector
    fn import_artifacts_tree(
        &mut self,
        artifacts_tree: DiGraph<ArtifactNode<SimpleArtifactKey>, Edge>,
        crate_name: String,
    ) {
        let mut index_map = HashMap::new();
        index_map.insert(
            NodeIndex::new(GLOBAL_NODE_INDEX),
            NodeIndex::new(GLOBAL_NODE_INDEX),
        );

        // create nodes in new graph
        for old_node_index in artifacts_tree.node_indices() {
            if old_node_index == NodeIndex::new(GLOBAL_NODE_INDEX) {
                continue;
            }
            let old_node = artifacts_tree
                .node_weight(old_node_index)
                .expect("Error: node not found importing artifacts tree");

            let new_node_index = self.artifacts_tree.create_node(
                SuperArtifactKey {
                    artifact: old_node.artifact.clone(),
                    krate: crate_name.clone(),
                },
                old_node.ident.clone(),
                old_node.complex_feature.clone(),
                old_node.features_indexes.clone(),
                old_node.weight.clone(),
            );
            index_map.insert(old_node_index, new_node_index);
        }

        // create edges in new graph
        for old_edge_index in artifacts_tree.edge_indices() {
            let (source, target) = artifacts_tree
                .edge_endpoints(old_edge_index)
                .expect("Error: edge not found importing artifacts tree");

            let new_source = index_map
                .get(&source)
                .expect("Error: source node not found in artifacts index map");
            let new_target = index_map
                .get(&target)
                .expect("Error: target node not found in artifacts index map");

            let edge_weight = artifacts_tree
                .edge_weight(old_edge_index)
                .expect("Error: edge weight not found importing artifacts tree")
                .clone();

            self.artifacts_tree
                .graph
                .add_edge(*new_source, *new_target, edge_weight);
        }
    }
}

// To parse CLI arguments, we use Clap for this example. But that
// detail is up to you.
#[derive(Parser)]
pub struct Args {
    /// Files to be deserialized
    #[clap(short, long)]
    files: Vec<String>,

    // Pass --print-terms-tree to print the Terms Tree in DOT format
    #[clap(long)]
    print_terms_tree: bool,

    /// Pass --print-features-graph to print the Features Graph in DOT format
    #[clap(long)]
    print_features_graph: bool,

    /// Pass --print-artifacts-tree to print the Artifacts Tree in DOT format
    #[clap(long)]
    print_artifacts_tree: bool,

    /// Pass --print-centrality to print some centrality measures on Features Graph
    #[clap(long)]
    print_centrality: bool,

    /// Pass --print-serialized-graphs to print all extracted graphs serialized
    #[clap(long)]
    print_serialized_graphs: bool,
}

impl Args {
    fn process_cli_args(&self, collector: &SuperCollector) {
        if self.print_terms_tree {
            collector.terms_tree.print_dot();
        }
        if self.print_features_graph {
            collector.features_graph.print_dot();
        }
        if self.print_artifacts_tree {
            collector.artifacts_tree.print_dot();
        }
        // TODO: centralità
        // if self.print_centrality {
        //     collector.print_centrality();
        // }
        // TODO: serializazzione
        // if self.print_serialized_graphs {
        //     collector.print_serialized_graphs();
        // }
    }
}

fn main() {
    let mut super_collector = SuperCollector {
        terms_tree: TermsTree::new(),
        features_graph: FeaturesGraph::new(),
        artifacts_tree: ArtifactsTree::new(),
    };

    super_collector.init_global_scope();

    let args = Args::parse();

    for file_path in &args.files {
        let file =
            File::open(file_path).unwrap_or_else(|_| panic!("Error: file {} not found", file_path));
        let collector: SimpleSerialization = serde_json::from_reader(file)
            .unwrap_or_else(|_| panic!("Error: {} deserialization failed", file_path));
        super_collector.import_terms_tree(collector.terms_tree.graph, file_path.to_string());
        super_collector.import_features_graph(collector.features_graph.graph);
        super_collector
            .import_artifacts_tree(collector.artifacts_tree.graph, file_path.to_string());
    }

    args.process_cli_args(&super_collector);
}
