#![feature(rustc_private)]

pub mod instrument;

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use clap::Parser;
use instrument::{CrateFilter, RustcPlugin, RustcPluginArgs, Utf8Path};
use rustc_ast::{ast::*, visit::*};
use rustc_span::symbol::*;
use rustworkx_core::petgraph::dot::{Config, Dot};
use rustworkx_core::petgraph::graph::{self, NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::{borrow::Cow, env};
use std::{fs, io, panic};

// This struct is the plugin provided to the rustc_plugin framework,
// and it must be exported for use by the CLI/driver binaries.
pub struct RustcEx;

// To parse CLI arguments, we use Clap for this example. But that
// detail is up to you.
#[derive(Parser, Serialize, Deserialize, Debug, Default)]
pub struct PrintAstArgs {
    /// Pass --print-ast-graph to print the DOT graph
    #[clap(long)]
    print_ast_graph: bool,

    /// Pass --print-features-graph to print the DOT graph
    #[clap(long)]
    print_features_graph: bool,

    /// Pass --print-artifacts-graph to print the DOT graph
    #[clap(long)]
    print_artifacts_graph: bool,

    /// Pass --print-crate to print the crate
    #[clap(long)]
    print_crate: bool,

    /// Pass --print-centrality to print some feature graph centrality
    #[clap(long)]
    print_centrality: bool,

    #[clap(last = true)]
    // mytool --allcaps -- some extra args here
    //                     ^^^^^^^^^^^^^^^^^^^^ these are cargo args
    cargo_args: Vec<String>,
}

impl RustcPlugin for RustcEx {
    type Args = PrintAstArgs;

    fn version(&self) -> Cow<'static, str> {
        env!("CARGO_PKG_VERSION").into()
    }

    fn driver_name(&self) -> Cow<'static, str> {
        "rustc-ex-driver".into()
    }

    fn modify_cargo(&self, cargo: &mut std::process::Command, args: &Self::Args) {
        cargo.args(&args.cargo_args);
    }

    // In the CLI, we ask Clap to parse arguments and also specify a CrateFilter.
    // If one of the CLI arguments was a specific file to analyze, then you
    // could provide a different filter.
    fn args(&self, _target_dir: &Utf8Path) -> RustcPluginArgs<Self::Args> {
        // We cannot use `#[cfg(test)]` here because the test suite installs the plugin.
        // In other words, in the test suite we need to compile (install) the plugin with
        // `--features test-mode` to skip the first argument that is the `cargo` command.
        //
        // # Explanation:
        //
        // ## Test
        //
        // In tests we run something like `cargo rustc-ex --print-dot` because the plugin is installed as a binary in a temporary directory.
        // It is expanded to `/tmp/rustc-ex/bin/cargo-rustc-ex rustc-ex --print-dot`, so we need to skip the first argument because it is the `cargo` command.
        //
        // ## Cli
        // In the CLI we run something like `cargo run --bin rustc-ex -- --print-dot` or `./target/debug/cargo-rustc-ex --print-dot`.
        // It is expanded to `.target/debug/cargo-rustc-ex --print-dot`, so we don't need to skip the first argument.
        #[cfg(feature = "test-mode")]
        let args = PrintAstArgs::parse_from(env::args().skip(1));

        #[cfg(not(feature = "test-mode"))]
        let args = PrintAstArgs::parse_from(env::args());

        let filter = CrateFilter::AllCrates;
        RustcPluginArgs { args, filter }
    }

    // In the driver, we use the Rustc API to start a compiler session
    // for the arguments given to us by rustc_plugin.
    fn run(
        self,
        compiler_args: Vec<String>,
        plugin_args: Self::Args,
    ) -> rustc_interface::interface::Result<()> {
        let mut callbacks = PrintAstCallbacks { args: plugin_args };
        let compiler = rustc_driver::RunCompiler::new(&compiler_args, &mut callbacks);
        compiler.run()
    }
}

struct PrintAstCallbacks {
    args: PrintAstArgs,
}

impl PrintAstCallbacks {
    fn process_cli_args(&self, collector: &CollectVisitor, krate: &Crate) {
        if self.args.print_crate {
            println!("{:#?}", krate);
        }
        if self.args.print_ast_graph {
            collector.print_ast_graph();
        }
        if self.args.print_features_graph {
            collector.print_features_graph();
        }
        if self.args.print_artifacts_graph {
            collector.print_artifacts_graph();
        }
        if self.args.print_centrality {
            collector.print_centrality();
        }
    }
}

impl rustc_driver::Callbacks for PrintAstCallbacks {
    /// Called before creating the compiler instance
    fn config(&mut self, config: &mut rustc_interface::Config) {
        /// Custom file loader to replace all `cfg` directives with `feat`
        struct CustomFileLoader;
        impl rustc_span::source_map::FileLoader for CustomFileLoader {
            fn file_exists(&self, path: &std::path::Path) -> bool {
                path.exists()
            }

            fn read_file(&self, path: &std::path::Path) -> io::Result<String> {
                let content = fs::read_to_string(path)?;
                Ok(content
                    // HACK: workarounds
                    // Features are discarded before the `after_expansion` hook, so are lost.
                    // To avoid this, we replace all `cfg` directives with a custom config.
                    .replace("#[cfg(", "#[rustcex_cfg(")
                    // The `cfg!` macro is evaluated before the `after_expansion` hook, so we replace it with a custom one.
                    // The replacement is not a macro because the macro would still be evaluated before the hook,
                    // giving an error in the AST.
                    .replace("cfg!", "rustcex_cfg"))
            }

            fn read_binary_file(&self, _path: &std::path::Path) -> io::Result<Arc<[u8]>> {
                // TODO: fare anche questo
                todo!()
            }
        }

        config.file_loader = Some(Box::new(CustomFileLoader));

        // Set the session creation callback to initialize the Fluent bundle.
        // It will make the compiler silent and use the fallback bundle.
        // Errors will not be printed in the `stderr`.
        config.psess_created = Some(Box::new(|sess| {
            let fallback_bundle = rustc_errors::fallback_fluent_bundle(
                rustc_driver::DEFAULT_LOCALE_RESOURCES.to_vec(),
                false,
            );

            sess.dcx().make_silent(fallback_bundle, None, false);
        }));
    }

    /// Called after expansion. Return value instructs the compiler whether to
    /// continue the compilation afterwards (defaults to `Compilation::Continue`)
    fn after_expansion<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>,
    ) -> rustc_driver::Compilation {
        queries
            .global_ctxt()
            .expect("Error: global context not found")
            .enter(|tcx: rustc_middle::ty::TyCtxt| {
                // extract AST
                let resolver_and_krate = tcx.resolver_for_lowering().borrow();
                let krate = &*resolver_and_krate.1;

                // visit AST
                let collector = &mut CollectVisitor {
                    stack: Vec::new(),

                    ast_graph: graph::DiGraph::new(),
                    ast_nodes: HashMap::new(),

                    feat_graph: graph::DiGraph::new(),
                    feat_nodes: HashMap::new(),

                    arti_graph: graph::DiGraph::new(),
                    arti_nodes: HashMap::new(),
                };

                collector.init_global_scope();
                collector.visit_crate(krate);
                collector.build_feat_graph();
                collector.build_arti_graph();

                self.process_cli_args(collector, krate);
            });

        rustc_driver::Compilation::Stop
    }
}

/// Constant for the global feature NodeId
const GLOBAL_NODE_ID: NodeId = NodeId::from_u32(4294967040);
/// Constant for the global feature name
const GLOBAL_FEATURE_NAME: &str = "__GLOBAL__";
/// Index of the global ASTNode/Feature/Artifact in the graphs
const GLOBAL_NODE_INDEX: usize = 0;

/// AST node, can be annotated with features
#[derive(Clone, Debug)]
struct ASTNode {
    node_id: NodeId,
    ident: Option<String>,
    features: ComplexFeature,
}

/// Simple feature, key of the features hashmap
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Feature {
    name: String,
    not: bool,
}

/// Complex feature, can be a single feature (not already included), an all or an any
#[derive(Clone, Debug, PartialEq)]
enum ComplexFeature {
    None,
    Feature(Feature),
    All(Vec<ComplexFeature>),
    Any(Vec<ComplexFeature>),
}

/// Node of the features graph, a feature with its weight
#[derive(Clone, Debug)]
struct FeatureNode {
    feature: Feature,
    weight: Option<f64>,
}

/// Artifact, key of the artifacts hashmap
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Artifact {
    node_id: NodeId,
}

/// Node of the artifacts graph, an artifact with its features and its weight
#[derive(Clone, Debug)]
struct ArtifactNode {
    artifact: Artifact,
    ident: Option<String>,
    features: Vec<FeatureIndex>, // index in features graph
    weight: Option<f64>,
}

/// Graphs edge, with weight
#[derive(Clone, Debug)]
struct Edge {
    weight: f64,
}

/// Index of the AST node in the AST graph
type ASTIndex = NodeIndex;
/// Index of the feature node in the features graph
type FeatureIndex = NodeIndex;
/// Index of the artifact node in the artifacts graph
type ArtifactIndex = NodeIndex;

/// AST visitor to collect data to build the graphs
struct CollectVisitor {
    // stack to keep track of the AST nodes dependencies
    stack: Vec<(ASTIndex, ComplexFeature)>,

    /// Relationships between all nodes of the AST.
    /// A `ASTNode` also stores features and ident of the `NodeId`
    ast_graph: graph::DiGraph<ASTNode, Edge>,
    /// Node of the AST -> Index in the temporary graph
    ast_nodes: HashMap<NodeId, ASTIndex>,

    /// Features graph, created from the AST graph.
    /// A `FeatureNode` also stores the weight of the `Feature`, calculated
    /// "horizontally" (considering only the "siblings")
    feat_graph: graph::DiGraph<FeatureNode, Edge>,
    /// Feature -> Index in the features graph
    feat_nodes: HashMap<Feature, FeatureIndex>,

    /// Artifacts graph, created from the AST graph.
    /// Only annotated artifacts are considered (artifacts with a feature).
    /// An `ArtifactNode` also stores the ident, the weight of the `Artifact`
    /// and the indexes of the features in the features graph
    arti_graph: graph::DiGraph<ArtifactNode, Edge>,
    /// Artifact -> Index in the artifacts graph
    arti_nodes: HashMap<Artifact, ArtifactIndex>,
}

impl CollectVisitor {
    /// Create an AST node in the AST graph and add it to the AST nodes hashmap.
    /// Return the index of the created node.
    fn create_ast_node(
        &mut self,
        ident: Option<String>,
        node_id: NodeId,
        features: ComplexFeature,
    ) -> ASTIndex {
        if let Some(index) = self.ast_nodes.get(&node_id) {
            return *index;
        }

        let index: ASTIndex = self.ast_graph.add_node(ASTNode {
            ident,
            node_id,
            features,
        });
        self.ast_nodes.insert(node_id, index);

        index
    }

    /// Create a feature node in the features graph and add it to the features nodes hashmap
    fn create_feature_node(&mut self, feature: Feature, weight: Option<f64>) -> FeatureIndex {
        if let Some(index) = self.feat_nodes.get(&feature) {
            return *index;
        }

        let index: FeatureIndex = self.feat_graph.add_node(FeatureNode {
            feature: feature.clone(),
            weight,
        });
        self.feat_nodes.insert(feature, index);

        index
    }

    /// Create an artifact node in the artifacts graph and add it to the artifacts nodes hashmap
    fn create_artifact_node(
        &mut self,
        artifact: Artifact,
        ident: Option<String>,
        features: Vec<FeatureIndex>,
        weight: Option<f64>,
    ) -> ArtifactIndex {
        if let Some(index) = self.arti_nodes.get(&artifact) {
            return *index;
        };

        let index: ArtifactIndex = self.arti_graph.add_node(ArtifactNode {
            artifact: artifact.clone(),
            ident,
            features,
            weight,
        });
        self.arti_nodes.insert(artifact, index);

        index
    }

    /// Initialize the global scope (AST node, feature, artifact)
    fn init_global_scope(&mut self) {
        let ident = Some(GLOBAL_FEATURE_NAME.to_string());
        let node_id = GLOBAL_NODE_ID;
        let feature = Feature {
            name: GLOBAL_FEATURE_NAME.to_string(),
            not: false,
        };
        let features = ComplexFeature::Feature(feature.clone());
        let artifact = Artifact { node_id };

        let index = self.create_ast_node(ident.clone(), node_id, features.clone());
        assert_eq!(
            index,
            ASTIndex::new(GLOBAL_NODE_INDEX),
            "Error: global AST node has an index != 0"
        );
        let index = self.create_feature_node(feature, None);
        assert_eq!(
            index,
            FeatureIndex::new(GLOBAL_NODE_INDEX),
            "Error: global feature node has an index != 0"
        );
        let index = self.create_artifact_node(
            artifact,
            ident,
            self.rec_features_to_indexes(&features),
            None,
        );
        assert_eq!(
            index,
            ArtifactIndex::new(GLOBAL_NODE_INDEX),
            "Error: global artifact node has an index != 0"
        );
    }

    /// Recursively visit nested features (all, any, not), creating features nodes
    fn rec_expand_features(
        &mut self,
        nested_meta: Vec<MetaItemInner>,
        not: bool,
    ) -> Vec<ComplexFeature> {
        let mut features = Vec::new();

        for meta in nested_meta {
            match meta.name_or_empty() {
                sym::feature => {
                    let name = meta
                        .value_str()
                        .expect("Error: malformed feature without value `#[cfg(feature)]`")
                        .to_string();

                    let feature = Feature { name, not };
                    self.create_feature_node(feature.clone(), None);

                    features.push(ComplexFeature::Feature(feature));
                }
                sym::not => features.extend(
                    self.rec_expand_features(
                        meta.meta_item_list()
                            .expect("Error: empty `not` feature attribute")
                            .to_vec(),
                        !not,
                    ),
                ),
                sym::all => features.push(ComplexFeature::All(
                    self.rec_expand_features(
                        meta.meta_item_list()
                            .expect("Error: empty `all` feature attribute")
                            .to_vec(),
                        not,
                    ),
                )),
                sym::any => features.push(ComplexFeature::Any(
                    self.rec_expand_features(
                        meta.meta_item_list()
                            .expect("Error: empty `any` feature attribute")
                            .to_vec(),
                        not,
                    ),
                )),
                _ => (),
            }
        }

        features
    }

    /// Weight features horizontally, considering only the "siblings"
    fn rec_weight_feature(features: &ComplexFeature) -> Vec<FeatureNode> {
        match features {
            ComplexFeature::None => Vec::new(),
            ComplexFeature::Feature(feature) => Vec::from([FeatureNode {
                feature: feature.clone(),
                weight: Some(1.0),
            }]),
            ComplexFeature::All(nested) => {
                let size = nested.len() as f64;

                nested
                    .iter()
                    .flat_map(|features| {
                        CollectVisitor::rec_weight_feature(features)
                            .into_iter()
                            .map(|feature| FeatureNode {
                                feature: feature.feature,
                                weight: Some(
                                    feature.weight.expect("Error: feature without weight") / size,
                                ),
                            })
                    })
                    .collect()
            }
            ComplexFeature::Any(nested) => nested
                .iter()
                .flat_map(CollectVisitor::rec_weight_feature)
                .collect(),
        }
    }

    /// Update the AST node with the found features. The parent ASTNode should already exist
    fn update_ast_node_features(&mut self, node_id: NodeId, features: ComplexFeature) {
        // update the node with the found and weighted cfgs
        let node_index: &ASTIndex = self
            .ast_nodes
            .get(&node_id)
            .expect("Error: cannot find AST node updating features");

        self.ast_graph
            .node_weight_mut(*node_index)
            .expect("Error: cannot find AST node updating features")
            .features = features;

        // create edge in the graph, to the parent or to the global scope
        match self.stack.last() {
            Some((parent_index, ..)) => {
                self.ast_graph
                    .add_edge(*node_index, *parent_index, Edge { weight: 0.0 });
            }
            None => {
                self.ast_graph.add_edge(
                    *node_index,
                    ASTIndex::new(GLOBAL_NODE_INDEX),
                    Edge { weight: 0.0 },
                );
            }
        }
    }

    /// Recursively convert features to node indexes in the features graph
    fn rec_features_to_indexes(&self, features: &ComplexFeature) -> Vec<NodeIndex> {
        let mut indexes = Vec::new();

        match features {
            ComplexFeature::None => (),
            ComplexFeature::Feature(f) => {
                indexes.push(*self.feat_nodes.get(f).expect(
                    "Error: cannot find feature node index converting features to indexes",
                ));
            }
            ComplexFeature::All(fs) | ComplexFeature::Any(fs) => {
                for f in fs {
                    indexes.extend(self.rec_features_to_indexes(f));
                }
            }
        }

        indexes
    }

    /// Initialize a new AST node and update the AST nodes and features stacks
    fn pre_walk(&mut self, ident: Option<String>, node_id: NodeId) {
        let astnode_index = self.create_ast_node(ident, node_id, ComplexFeature::None);
        self.stack.push((astnode_index, ComplexFeature::None));
    }

    /// Extract the features of the AST node from the stacks and update the AST graph
    fn post_walk(&mut self, node_id: NodeId) {
        let (node_index, features) = self
            .stack
            .pop()
            .expect("Error: stack is empty while in expression");

        let ast_node = self
            .ast_graph
            .node_weight_mut(node_index)
            .expect("Error: missing node post AST walk");

        assert_eq!(
            node_id, ast_node.node_id,
            "Error: node id mismatch post AST walk"
        );

        // create artifact if some features are found
        if features != ComplexFeature::None {
            let ident = ast_node.ident.clone();
            // convert features to index of the features (the features node already exist)
            let features_indexes = self.rec_features_to_indexes(&features);

            self.create_artifact_node(Artifact { node_id }, ident, features_indexes, None);
        }

        // insert found features in node
        self.update_ast_node_features(node_id, features);
    }

    fn get_annotated_parent(
        graph: &graph::DiGraph<ASTNode, Edge>,
        start_index: ASTIndex,
    ) -> Option<ASTIndex> {
        // global node (no parents)
        if start_index == NodeIndex::new(GLOBAL_NODE_INDEX) {
            return None;
        }

        assert!(
            graph.neighbors(start_index).count() == 1,
            "Error: node has multiple parents"
        );

        let parent = graph
            .neighbors(start_index)
            .next()
            .expect("Error: missing parent index");

        let parent_features = &graph
            .node_weight(parent)
            .expect("Error: missing parent node")
            .features;

        match parent_features {
            ComplexFeature::None => CollectVisitor::get_annotated_parent(graph, parent),
            _ => Some(parent),
        }
    }

    /// Build the features graph from the AST graph
    fn build_feat_graph(&mut self) {
        self.ast_nodes
            .iter()
            // ignore global node
            .filter(|(_, node_index)| *node_index != &NodeIndex::new(GLOBAL_NODE_INDEX))
            .for_each(|(.., child_index)| {
                let child_node = &self
                    .ast_graph
                    .node_weight(*child_index)
                    .expect("Error: cannot find child node creating features graph");
                let child_features = CollectVisitor::rec_weight_feature(&child_node.features);

                let parent_index =
                    CollectVisitor::get_annotated_parent(&self.ast_graph, *child_index)
                        .expect("Error: cannot find parent creating features graph");
                let parent_features = CollectVisitor::rec_weight_feature(
                    &self
                        .ast_graph
                        .node_weight(parent_index)
                        .expect("Error: cannot find parent node creating features graph")
                        .features,
                );

                child_features
                    .iter()
                    // cartesian product
                    .flat_map(|x| parent_features.iter().map(move |y| (x, y)))
                    .for_each(|(child_feat, parent_feat)| {
                        self.feat_graph.add_edge(
                            *self
                                .feat_nodes
                                .get(&child_feat.feature)
                                .expect("Error: cannot find feature node creating features graph"),
                            *self
                                .feat_nodes
                                .get(&parent_feat.feature)
                                .expect("Error: cannot find feature node creating features graph"),
                            Edge {
                                weight: child_feat.weight.expect(
                                    "Error: feature without weight creating features graph",
                                ),
                            },
                        );
                    });
            });
    }

    /// Build the artifacts graph from the AST graph
    fn build_arti_graph(&mut self) {
        self.ast_nodes
            .iter()
            // ignore global node
            .filter(|(.., node_index)| *node_index != &NodeIndex::new(GLOBAL_NODE_INDEX))
            // ignore nodes with no features
            .filter(|(.., node_index)| {
                let child_node = &self
                    .ast_graph
                    .node_weight(**node_index)
                    .expect("Error: cannot find child node creating features graph");

                child_node.features != ComplexFeature::None
            })
            // get first annotated parent for each node
            // the index is in the AST graph, not in the artifacts graph
            .map(|(.., child_index)| {
                let parent_index =
                    CollectVisitor::get_annotated_parent(&self.ast_graph, *child_index)
                        .expect("Error: cannot find parent creating features graph");

                (child_index, parent_index)
            })
            // add edge between child and parent in the artifacts graph
            // we need to convert the ASTIndex to the ArtifactIndex
            .for_each(|(child_ast_index, parent_ast_index)| {
                let child_ast_node = &self
                    .ast_graph
                    .node_weight(*child_ast_index)
                    .expect("Error: cannot find child node creating features graph");
                let parent_ast_node = &self
                    .ast_graph
                    .node_weight(parent_ast_index)
                    .expect("Error: cannot find parent node creating features graph");

                let child_arti_index = self
                    .arti_nodes
                    .get(&Artifact {
                        node_id: child_ast_node.node_id,
                    })
                    .expect("Error: cannot find child artifact node creating artifacts graph");
                let parent_arti_index = self
                    .arti_nodes
                    .get(&Artifact {
                        node_id: parent_ast_node.node_id,
                    })
                    .expect("Error: cannot find child artifact node creating artifacts graph");

                self.arti_graph.add_edge(
                    *child_arti_index,
                    *parent_arti_index,
                    Edge {
                        weight: 0.0, // TODO: pesare i nodi degli artefatti
                    },
                );
            });
    }

    /// Print features graph in DOT format
    fn print_features_graph(&self) {
        let get_edge_attr = |_g: &graph::DiGraph<FeatureNode, Edge>,
                             edge: graph::EdgeReference<Edge>| {
            format!("label=\"{:.2}\"", edge.weight().weight)
        };

        let get_node_attr = |_g: &graph::DiGraph<FeatureNode, Edge>,
                             node: (graph::NodeIndex, &FeatureNode)| {
            let feature = node.1;
            match feature.feature.not {
                true => format!("label=\"!{}\"", feature.feature.name),
                false => format!("label=\"{}\"", feature.feature.name),
            }
        };

        println!(
            "{:?}",
            Dot::with_attr_getters(
                &self.feat_graph,
                &[Config::NodeNoLabel, Config::EdgeNoLabel],
                &get_edge_attr,
                &get_node_attr,
            )
        )
    }

    /// Print artifacts graph in DOT format
    fn print_artifacts_graph(&self) {
        let get_edge_attr = |_g: &graph::DiGraph<ArtifactNode, Edge>,
                             edge: graph::EdgeReference<Edge>| {
            format!("label=\"{:.2}\"", edge.weight().weight)
        };

        // TODO: formattare meglio
        let get_node_attr = |_g: &graph::DiGraph<ArtifactNode, Edge>,
                             node: (graph::NodeIndex, &ArtifactNode)| {
            let artifact = node.1;
            format!(
                "label=\"{} ({}) #[{:?}] {:.2}\"",
                artifact.artifact.node_id,
                artifact.ident.clone().unwrap_or("-".to_string()),
                artifact.features,
                artifact.weight.unwrap_or(0.0)
            )
        };

        println!(
            "{:?}",
            Dot::with_attr_getters(
                &self.arti_graph,
                &[Config::NodeNoLabel, Config::EdgeNoLabel],
                &get_edge_attr,
                &get_node_attr,
            )
        )
    }

    /// Print AST graph in DOT format
    fn print_ast_graph(&self) {
        let get_edge_attr = |_g: &graph::DiGraph<ASTNode, Edge>,
                             edge: graph::EdgeReference<Edge>| {
            format!("label=\"{}\"", edge.weight().weight)
        };

        // TODO: formattare meglio
        let get_node_attr = |_g: &graph::DiGraph<ASTNode, Edge>,
                             node: (graph::NodeIndex, &ASTNode)| {
            let ast_node = node.1;
            format!(
                "label=\"{} ({}) #[{}]\"",
                ast_node.node_id,
                ast_node.ident.clone().unwrap_or("-".to_string()),
                ast_node.features,
            )
        };

        println!(
            "{:?}",
            Dot::with_attr_getters(
                &self.ast_graph,
                &[Config::NodeNoLabel, Config::EdgeNoLabel],
                &get_edge_attr,
                &get_node_attr,
            )
        )
    }

    /// Print some centrality measures of the features graph
    fn print_centrality(&self) {
        let katz: rustworkx_core::Result<Option<Vec<f64>>> =
            rustworkx_core::centrality::katz_centrality(
                &self.feat_graph,
                |e| Ok(e.weight().weight),
                None,
                None,
                None,
                None,
                None,
            );

        let closeness = rustworkx_core::centrality::closeness_centrality(&self.feat_graph, false);

        let eigenvector: rustworkx_core::Result<Option<Vec<f64>>> =
            rustworkx_core::centrality::eigenvector_centrality(
                &self.feat_graph,
                |e| Ok(e.weight().weight),
                None,
                Some(1e-2),
            );

        println!("katz {:?}", katz.unwrap().unwrap());
        println!("clos {:?}", closeness);
        println!("eige {:?}", eigenvector);
    }
}

impl<'ast> Visitor<'ast> for CollectVisitor {
    // TODO: rilevare anche `cfg!` (trasformato a call `rustcex_cfg`, NON macro)
    // TODO: rilevare features sulle call di macro

    // The features (cfg) are attributes, but attributes are (almost) always
    // at the same level of the Node they are annotating. So the features are (almost)
    // always NOT available inside the visit *node* function, but are available only
    // to its parent. This is why we need to use a stack to keep track of the features.
    // Example:
    // ```
    // #[cfg(feature = "foo")]
    // fn bar() {}
    // ```
    // The `visit_fn` method will NOT visit the `cfg` attribute, NOT even after
    // the `walk_fn` call. The attributes are available to the parent of the function,
    // in this case an item (which calls both `visit_fn` and `visit_attribute`).

    /// Visit attribute: features are attributes
    fn visit_attribute(&mut self, attr: &'ast Attribute) {
        if let Some(meta) = attr.meta() {
            if meta.name_or_empty() == Symbol::intern("rustcex_cfg") {
                if let MetaItemKind::List(ref list) = meta.kind {
                    match self.stack.pop() {
                        Some((astnode_index, ComplexFeature::None)) => {
                            let parsed_features = self.rec_expand_features(list.to_vec(), false);
                            assert!(
                                parsed_features.len() == 1,
                                "Error: multiple (not nested) features in cfg attribute"
                            );
                            let feat = parsed_features[0].to_owned();

                            self.stack.push((astnode_index, feat));
                        }
                        Some((.., ComplexFeature::Feature(..)))
                        | Some((.., ComplexFeature::All(..)))
                        | Some((.., ComplexFeature::Any(..))) => {
                            panic!("Error: node on stack already has a feature visiting attribute")
                        }
                        None => panic!("Error: stack is empty while in attribute (cfg) visit"),
                    }
                }
            }
        }

        walk_attribute(self, attr);
    }

    /// Visit expression, like function calls
    fn visit_expr(&mut self, cur_ex: &'ast Expr) {
        let ident = None;
        let node_id = cur_ex.id;

        self.pre_walk(ident, node_id);
        walk_expr(self, cur_ex);
        self.post_walk(node_id);
    }

    /// Visit item, like functions, structs, enums
    fn visit_item(&mut self, cur_item: &'ast Item) {
        let ident = Some(cur_item.ident.to_string());
        let node_id = cur_item.id;

        self.pre_walk(ident, node_id);
        walk_item(self, cur_item);
        self.post_walk(node_id);
    }

    /// Visit definition fields, like struct fields
    fn visit_field_def(&mut self, cur_field: &'ast FieldDef) -> Self::Result {
        let ident = None;
        let node_id = cur_field.id;

        self.pre_walk(ident, node_id);
        walk_field_def(self, cur_field);
        self.post_walk(node_id);
    }

    /// Visit statement, like let, if, while
    fn visit_stmt(&mut self, cur_stmt: &'ast Stmt) -> Self::Result {
        let ident = None;
        let node_id = cur_stmt.id;

        self.pre_walk(ident, node_id);
        walk_stmt(self, cur_stmt);
        self.post_walk(node_id);
    }

    /// Visit enum variant
    fn visit_variant(&mut self, cur_var: &'ast Variant) -> Self::Result {
        let ident = Some(cur_var.ident.to_string());
        let node_id = cur_var.id;

        self.pre_walk(ident, node_id);
        walk_variant(self, cur_var);
        self.post_walk(node_id);
    }

    /// Visita match arm
    fn visit_arm(&mut self, cur_arm: &'ast Arm) -> Self::Result {
        let ident = None;
        let node_id = cur_arm.id;

        self.pre_walk(ident, node_id);
        walk_arm(self, cur_arm);
        self.post_walk(node_id);
    }
}

impl std::fmt::Display for ComplexFeature {
    /// Complex feature to string
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplexFeature::None => write!(f, ""),
            ComplexFeature::Feature(Feature { name, not }) => {
                let name = match not {
                    true => "!".to_string() + name,
                    false => name.to_string(),
                };
                write!(f, "{}", name)
            }
            ComplexFeature::All(features) => write!(
                f,
                "all({})",
                features
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ComplexFeature::Any(features) => write!(
                f,
                "any({})",
                features
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}
