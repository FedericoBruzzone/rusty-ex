#![feature(rustc_private)]

use rustc_ex::types::*;
use std::fs::File;

pub struct SuperCollector {
    /// Relationships between all nodes of the AST (both annotated or not)
    ast_graph: AstGraph<SuperAstKey>,
    /// Multigraph storing relationships between features
    features_graph: FeaturesGraph,
    /// Graph storing only annotated artifacts (AST nodes with features)
    artifacts_graph: ArtifactsGraph<SuperArtifactKey>,
}

fn main() {
    // TODO: prendere in maniera decente i file da unire
    let results = vec!["collector.json", "collector2.json"];

    let mut super_collector = SuperCollector {
        ast_graph: AstGraph::new(),
        features_graph: FeaturesGraph::new(),
        artifacts_graph: ArtifactsGraph::new(),
    };

    for file_path in results {
        let file = File::open(file_path).unwrap();
        let collector: SimpleSerialization = serde_json::from_reader(file).unwrap();

        // TODO: unire i vari grafi in un unico SuperCollector
        collector.artifacts_graph.print_dot();
    }
}
