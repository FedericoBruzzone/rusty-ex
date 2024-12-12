#![feature(rustc_private)]

pub mod types;
pub mod instrument;

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use clap::Parser;
use types::*;
use instrument::{CrateFilter, RustcPlugin, RustcPluginArgs, Utf8Path};
use linked_hash_set::LinkedHashSet;
use rustc_ast::{ast::*, visit::*};
use rustc_span::symbol::*;
use rustworkx_core::petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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

    /// Pass --print-serialized-graphs to print all extracted data serialized
    #[clap(long)]
    print_serialized_graphs: bool,

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
        log::debug!("Running plugin with compiler args: {:?}", compiler_args);
        log::debug!("Running plugin with args: {:?}", plugin_args);
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
            collector.ast_graph.print_dot();
        }
        if self.args.print_features_graph {
            collector.features_graph.print_dot();
        }
        if self.args.print_artifacts_graph {
            collector.artifacts_graph.print_dot();
        }
        if self.args.print_centrality {
            collector.print_centrality();
        }
        if self.args.print_serialized_graphs {
            collector.print_serialized_graphs();
        }
        // TODO: aggiungere deserializzatore
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
                    // Features are discarded before the `after_expansion` hook, so are lost.
                    // To avoid this, we replace all `cfg` directives with a custom config.
                    .replace("#[cfg(", "#[rustcex_cfg(")
                    // The `cfg!` macro is evaluated before the `after_expansion` hook, so we replace it with a custom one.
                    // The replacement is not a macro because the macro would still be evaluated before the hook,
                    // giving an error in the AST.
                    .replace("cfg!", "rustcex_cfg"))
            }

            fn read_binary_file(&self, _path: &std::path::Path) -> io::Result<Arc<[u8]>> {
                // TODO: fare il replace anche nella lettura di file binari
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

                    ast_graph: AstGraph::new(),
                    features_graph: FeaturesGraph::new(),
                    artifacts_graph: ArtifactsGraph::new(),

                    idents_weights: HashMap::new(),
                    weights_to_resolve: LinkedHashSet::new(),
                };

                // initialize global scope (global feature and artifact)
                collector.init_global_scope();

                // visit AST and build AST graph
                collector.visit_crate(krate);

                // build features and artifacts graphs visiting AST graph
                collector.build_feat_graph();
                collector.build_arti_graph();

                // calculate weights of AST nodes
                collector.ast_graph.graph.reverse(); // reverse graph
                collector.rec_weight_ast_graph(AstIndex::new(GLOBAL_NODE_INDEX));
                collector.resolve_weights_in_wait();
                collector.ast_graph.graph.reverse(); // restore graph

                self.process_cli_args(collector, krate);
            });

        rustc_driver::Compilation::Stop
    }
}

/// Constant for the global feature NodeId.
/// 4294967040 is not used because it is used by the compiler for "Dummy" nodes
const GLOBAL_NODE_ID: NodeId = NodeId::from_u32(4294967039);
/// Constant for the global feature name
const GLOBAL_FEATURE_NAME: &str = "__GLOBAL__";
/// Index of the global ASTNode/Feature/Artifact in the graphs
const GLOBAL_NODE_INDEX: usize = 0;

/// AST visitor to collect data to build the graphs
pub struct CollectVisitor {
    /// Stack to keep track of the AST nodes dependencies
    stack: Vec<(AstIndex, ComplexFeature)>,

    /// Relationships between all nodes of the AST (both annotated or not)
    ast_graph: AstGraph,
    /// Multigraph storing relationships between features
    features_graph: FeaturesGraph,
    /// Graph storing only annotated artifacts (AST nodes with features)
    artifacts_graph: ArtifactsGraph,

    /// Weights of the already weighted idents (needed to weight the Calls).
    /// The weights for a single ident can be multiple, because a function can
    /// be defined multiple times (with different #[cfg] attributes)
    idents_weights: HashMap<String, Vec<f64>>,
    /// Weights of the ASTNodes that are waiting for something to be resolved.
    /// This needs to be a set, but with insertion order preserved (a "unique" queue)
    weights_to_resolve: LinkedHashSet<AstIndex>,
}

impl CollectVisitor {
    /// Initialize the global scope (AST node, feature node, artifact node)
    fn init_global_scope(&mut self) {
        let ident = Some(GLOBAL_FEATURE_NAME.to_string());
        let node_id = GLOBAL_NODE_ID;
        let feature = Feature {
            name: GLOBAL_FEATURE_NAME.to_string(),
            not: false,
        };
        let features = ComplexFeature::Feature(feature.clone());
        let artifact = ArtifactKey(node_id);

        let index = self.ast_graph.create_node(
            AstKey(node_id),
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
            artifact,
            ident,
            self.rec_features_to_indexes(&features),
            NodeWeight::ToBeCalculated,
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
                    self.features_graph
                        .create_node(FeatureKey(feature.clone()), None);

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
                feature: FeatureKey(feature.clone()),
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

    /// Update the AST node with the found features and create the dependency (edge)
    /// in the AST graph. The parent ASTNode should already exist (anothe node or global scope)
    fn update_ast_node_features(&mut self, node_id: NodeId, features: ComplexFeature) {
        // update the node with the found and weighted cfgs
        let node_index: &AstIndex = self
            .ast_graph
            .nodes
            .get(&AstKey(node_id))
            .expect("Error: cannot find AST node updating features");

        self.ast_graph
            .graph
            .node_weight_mut(*node_index)
            .expect("Error: cannot find AST node updating features")
            .features = features;

        // create edge in the graph, to the parent or to the global scope
        match self.stack.last() {
            Some((parent_index, ..)) => {
                self.ast_graph
                    .graph
                    .add_edge(*node_index, *parent_index, Edge { weight: 0.0 });
            }
            None => {
                self.ast_graph.graph.add_edge(
                    *node_index,
                    AstIndex::new(GLOBAL_NODE_INDEX),
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
                indexes.push(
                    *self
                        .features_graph
                        .nodes
                        .get(&FeatureKey(f.clone()))
                        .expect(
                            "Error: cannot find feature node index converting features to indexes",
                        ),
                );
            }
            ComplexFeature::All(fs) | ComplexFeature::Any(fs) => {
                for f in fs {
                    indexes.extend(self.rec_features_to_indexes(f));
                }
            }
        }

        indexes
    }

    /// Initialize a new AST node and update the stack
    fn pre_walk(&mut self, kind: NodeWeightKind, ident: Option<String>, node_id: NodeId) {
        let ast_index = self.ast_graph.create_node(
            AstKey(node_id),
            ident,
            ComplexFeature::None,
            kind,
            NodeWeight::ToBeCalculated,
        );
        self.stack.push((ast_index, ComplexFeature::None));
    }

    /// Extract the features of the AST node from the stack and update the AST graph
    fn post_walk(&mut self, node_id: NodeId) {
        let (node_index, features) = self
            .stack
            .pop()
            .expect("Error: stack is empty while in expression");

        let ast_node = self
            .ast_graph
            .graph
            .node_weight_mut(node_index)
            .expect("Error: missing node post AST walk");

        assert_eq!(
            node_id, ast_node.node_id.0,
            "Error: node id mismatch post AST walk"
        );

        // create artifact if some features are found
        if features != ComplexFeature::None {
            let ident = ast_node.ident.clone();
            // convert features to index of the features (the features node already exist)
            let features_indexes = self.rec_features_to_indexes(&features);

            self.artifacts_graph.create_node(
                ArtifactKey(node_id),
                ident,
                features_indexes,
                NodeWeight::ToBeCalculated,
            );
        }

        // insert found features in node
        self.update_ast_node_features(node_id, features);
    }

    /// Recursively fet the first parent node with features. The only node with no
    /// annotated parents is the global scope
    fn rec_get_annotated_parent(
        graph: &DiGraph<AstNode, Edge>,
        start_index: AstIndex,
    ) -> Option<AstIndex> {
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
            ComplexFeature::None => CollectVisitor::rec_get_annotated_parent(graph, parent),
            _ => Some(parent),
        }
    }

    /// Recursively weight (in place) the AST nodes in the AST graph, starting from the global node
    fn rec_weight_ast_graph(&mut self, start_index: AstIndex) -> NodeWeight {
        // TODO: ci sono altre cose fa considerare come Call?

        // IMPORTANT! We cannot skip already weighted nodes because a re-evaluation may be needed.
        // For example, if the weight an ident has changed (multiple definitions), then we need to
        // update the weight of all calls to that ident.
        // This works because the root (GLOBAL) is in Wait state until everything is resolved.

        let adjacents = self
            .ast_graph
            .graph
            .neighbors(start_index)
            .collect::<Vec<_>>()
            .into_iter()
            .map(|index| self.rec_weight_ast_graph(index))
            .collect::<Vec<_>>();

        let mut child_weight = 0.0;
        for adj_weight in adjacents {
            match adj_weight {
                NodeWeight::ToBeCalculated => {
                    panic!("Error: recursion not working while weighting AST graph")
                }
                NodeWeight::Wait(dep) => {
                    self.update_weight(start_index, NodeWeight::Wait(dep.clone()));
                    self.weights_to_resolve.insert(start_index);
                    return NodeWeight::Wait(dep);
                }
                NodeWeight::Weight(w) => child_weight += w,
            }
        }

        let weight = match &self
            .ast_graph
            .graph
            .node_weight(start_index)
            .expect("Error: cannot find AST node weighting AST graph")
            .weight_kind
        {
            NodeWeightKind::Leaf(..) => NodeWeight::Weight(1.0 + child_weight),
            NodeWeightKind::Block(..) => NodeWeight::Weight(child_weight),
            NodeWeightKind::Call(.., Some(to)) => match self.idents_weights.get(to) {
                Some(vec_fn_weight) => NodeWeight::Weight(
                    (vec_fn_weight.iter().sum::<f64>() / vec_fn_weight.len() as f64) + child_weight,
                ),
                None => {
                    self.weights_to_resolve.insert(start_index);
                    NodeWeight::Wait(to.to_string())
                }
            },
            NodeWeightKind::Call(.., None) => NodeWeight::Weight(child_weight),
            NodeWeightKind::NoWeight(..) => NodeWeight::Weight(0.0),
        };

        self.update_weight(start_index, weight.clone());
        weight
    }

    /// Calculate the weights of the nodes in wait
    fn resolve_weights_in_wait(&mut self) {
        let mut seen = HashSet::new();

        while let Some(cur_index) = self.weights_to_resolve.pop_front() {
            // prevent infinte loop:
            // if the same index is seen with the same number of unsolved weights, then it's a loop
            if seen.contains(&(cur_index, self.weights_to_resolve.len())) {
                return;
            }
            seen.insert((cur_index, self.weights_to_resolve.len()));

            // try to weight the node, if it's not possible, it will be added again to the queue
            self.rec_weight_ast_graph(cur_index);
        }
    }

    /// Update the weight of the AST node.
    /// Remove the updated node from nodes in wait (only if the weight is not a Wait).
    /// Add the weight to the `idents_weights` map if the node has an ident
    fn update_weight(&mut self, ast_index: AstIndex, weight: NodeWeight) {
        let ast_node = self
            .ast_graph
            .graph
            .node_weight_mut(ast_index)
            .expect("Error: cannot find AST node updating weight");

        if ast_node.features != ComplexFeature::None {
            let artifact_index = self
                .artifacts_graph
                .nodes
                .get(&ArtifactKey(ast_node.node_id.0))
                .expect("Error: cannot find artifact index updating weight");
            let artifact_node = self
                .artifacts_graph
                .graph
                .node_weight_mut(*artifact_index)
                .expect("Error: cannot find artifact node updating weight");
            artifact_node.weight = weight.clone();
        }

        ast_node.weight = weight.clone();

        // remove from nodes in wait
        if let NodeWeight::Weight(..) = weight {
            self.weights_to_resolve.remove(&ast_index);
        }

        // add to idents map if it has an ident
        if let (NodeWeight::Weight(weight), Some(ident)) = (weight, ast_node.ident.clone()) {
            self.idents_weights.entry(ident).or_default().push(weight);
        }
    }

    /// Build the features graph from the AST graph
    fn build_feat_graph(&mut self) {
        self.ast_graph
            .nodes
            .iter()
            // ignore global node
            .filter(|(_, node_index)| *node_index != &NodeIndex::new(GLOBAL_NODE_INDEX))
            .for_each(|(.., child_index)| {
                let child_node = &self
                    .ast_graph
                    .graph
                    .node_weight(*child_index)
                    .expect("Error: cannot find child node creating features graph");
                let child_features = CollectVisitor::rec_weight_feature(&child_node.features);

                let parent_index =
                    CollectVisitor::rec_get_annotated_parent(&self.ast_graph.graph, *child_index)
                        .expect("Error: cannot find parent creating features graph");
                let parent_features = CollectVisitor::rec_weight_feature(
                    &self
                        .ast_graph
                        .graph
                        .node_weight(parent_index)
                        .expect("Error: cannot find parent node creating features graph")
                        .features,
                );

                child_features
                    .iter()
                    // cartesian product
                    .flat_map(|x| parent_features.iter().map(move |y| (x, y)))
                    .for_each(|(child_feat, parent_feat)| {
                        self.features_graph.graph.add_edge(
                            *self
                                .features_graph
                                .nodes
                                .get(&child_feat.feature)
                                .expect("Error: cannot find feature node creating features graph"),
                            *self
                                .features_graph
                                .nodes
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
        self.ast_graph
            .nodes
            .iter()
            // ignore global node
            .filter(|(.., node_index)| *node_index != &NodeIndex::new(GLOBAL_NODE_INDEX))
            // ignore nodes with no features
            .filter(|(.., node_index)| {
                let child_node = &self
                    .ast_graph
                    .graph
                    .node_weight(**node_index)
                    .expect("Error: cannot find child node creating features graph");

                child_node.features != ComplexFeature::None
            })
            // get first annotated parent for each node
            // the index is in the AST graph, not in the artifacts graph
            .map(|(.., child_index)| {
                let parent_index =
                    CollectVisitor::rec_get_annotated_parent(&self.ast_graph.graph, *child_index)
                        .expect("Error: cannot find parent creating features graph");

                (child_index, parent_index)
            })
            // add edge between child and parent in the artifacts graph
            // we need to convert the AstIndex to the ArtifactIndex
            .for_each(|(child_ast_index, parent_ast_index)| {
                let child_ast_node = &self
                    .ast_graph
                    .graph
                    .node_weight(*child_ast_index)
                    .expect("Error: cannot find child node creating features graph");
                let parent_ast_node = &self
                    .ast_graph
                    .graph
                    .node_weight(parent_ast_index)
                    .expect("Error: cannot find parent node creating features graph");

                let child_arti_index = self
                    .artifacts_graph
                    .nodes
                    .get(&ArtifactKey(child_ast_node.node_id.0))
                    .expect("Error: cannot find child artifact node creating artifacts graph");
                let parent_arti_index = self
                    .artifacts_graph
                    .nodes
                    .get(&ArtifactKey(parent_ast_node.node_id.0))
                    .expect("Error: cannot find child artifact node creating artifacts graph");

                self.artifacts_graph.graph.add_edge(
                    *child_arti_index,
                    *parent_arti_index,
                    Edge { weight: 0.0 },
                );
            });
    }

    /// Print some centrality measures of the features graph
    fn print_centrality(&self) {
        let katz: rustworkx_core::Result<Option<Vec<f64>>> =
            rustworkx_core::centrality::katz_centrality(
                &self.features_graph.graph,
                |e| Ok(e.weight().weight),
                None,
                None,
                None,
                None,
                None,
            );

        let closeness =
            rustworkx_core::centrality::closeness_centrality(&self.features_graph.graph, false);

        let eigenvector: rustworkx_core::Result<Option<Vec<f64>>> =
            rustworkx_core::centrality::eigenvector_centrality(
                &self.features_graph.graph,
                |e| Ok(e.weight().weight),
                None,
                Some(1e-2),
            );

        let graph_nodes = self
            .features_graph
            .graph
            .node_indices()
            .map(|n| {
                let feat = self.features_graph.graph.node_weight(n).unwrap();
                (
                    n,
                    (feat.feature.0.name.clone(), feat.feature.0.not, feat.weight),
                )
            })
            .collect::<Vec<_>>();

        let katz = katz.unwrap().unwrap();
        let katz_zip = katz.iter().zip(graph_nodes.iter()).collect::<Vec<_>>();

        let closeness_zip = closeness.iter().zip(graph_nodes.iter()).collect::<Vec<_>>();

        let eigenvector = eigenvector.unwrap().unwrap();
        let eigenvector_zip = eigenvector
            .iter()
            .zip(graph_nodes.iter())
            .collect::<Vec<_>>();

        println!("katz {:?}", katz_zip); // println!("katz {:?}", katz.unwrap().unwrap());
        println!("clos {:?}", closeness_zip); // println!("clos {:?}", closeness);
        println!("eige {:?}", eigenvector_zip); // println!("eige {:?}", eigenvector);
    }

    /// Print all extracted graphs serialized
    fn print_serialized_graphs(&self) {
        #[derive(Serialize, Deserialize)]
        struct Serialized {
            ast_graph: DiGraph<AstNode, Edge>,
            features_graph: DiGraph<FeatureNode, Edge>,
            artifacts_graph: DiGraph<ArtifactNode, Edge>,
        }

        let graphs = Serialized {
            ast_graph: self.ast_graph.graph.clone(),
            features_graph: self.features_graph.graph.clone(),
            artifacts_graph: self.artifacts_graph.graph.clone(),
        };

        println!(
            "{}",
            serde_json::to_string(&graphs).expect("Error: cannot serialize data")
        );
    }
}

impl<'ast> Visitor<'ast> for CollectVisitor {
    // TODO: le features sulle call di macro NON vengono correttamente rilevate,
    // le macro hanno un comportamento particolare, in questo stato dell'AST
    // sono già parzialmente espanse.
    // Il banale workaround di trasformare tutte le chiamate a macro in funzioni
    // (togliendo il `!`) non basta, dato che esistono anche le macro con `[]` e
    // con `{}`, causando errori di sintassi.

    // TODO: le feature vengono rilevate solo se scritte con sintassi `#[cfg(feature = ...)]`,
    // la macro `cfg!` non è rilevata.
    // Come descritto nel TODO precedente non basta fare il replace di `cfg!` con ad
    // esempio `rustcex_cfg!` dato che anche questa macro viene già espansa (in un errore
    // garantito, dato che non la trova).
    // Un workaround potrebbe essere quello di rimpiazzarla con un nome di funzione valido
    // come `rustcex_cfg`.

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

                            match parsed_features.len() {
                                // well-formed without feature (we can ignore): #[cfg(windows)]
                                0 => {
                                    self.stack.push((astnode_index, ComplexFeature::None));
                                }
                                // well-formed with feature (we need the feature): #[cfg(feature = "a"))]
                                1 => {
                                    self.stack
                                        .push((astnode_index, parsed_features[0].to_owned()));
                                }
                                // malformed (panic): #[cfg(feature = "a", feature = "b")]
                                _ => {
                                    panic!("Error: multiple (not nested) features in cfg attribute")
                                }
                            }
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
        let kind_string = NodeWeightKind::parse_kind_variant_name(format!("{:?}", &cur_ex.kind));
        let kind = match &cur_ex.kind {
            // blocks
            ExprKind::Array(..)
            | ExprKind::ConstBlock(..)
            | ExprKind::Tup(..)
            | ExprKind::Binary(..)
            | ExprKind::Unary(..)
            | ExprKind::Cast(..)
            | ExprKind::Type(..)
            | ExprKind::Let(..)
            | ExprKind::If(..)
            | ExprKind::While(..)
            | ExprKind::ForLoop { .. }
            | ExprKind::Loop(..)
            | ExprKind::Match(..)
            | ExprKind::Closure(..)
            | ExprKind::Block(..)
            | ExprKind::Gen(..)
            | ExprKind::Await(..)
            | ExprKind::TryBlock(..)
            | ExprKind::Assign(..)
            | ExprKind::AssignOp(..)
            | ExprKind::Field(..)
            | ExprKind::Index(..)
            | ExprKind::Range(..)
            | ExprKind::AddrOf(..)
            | ExprKind::Struct(..)
            | ExprKind::Repeat(..)
            | ExprKind::Paren(..)
            | ExprKind::Try(..) => NodeWeightKind::Block(kind_string),

            // calls
            ExprKind::Call(call, ..) => {
                let ident = match &call.kind {
                    ExprKind::Path(None, Path { segments, .. }) => Some(
                        segments
                            .iter()
                            .map(|seg| seg.ident.to_string())
                            .collect::<Vec<_>>()
                            .join("::"),
                    ),
                    _ => None,
                };
                NodeWeightKind::Call(kind_string, ident)
            }
            ExprKind::MethodCall(method_call) => {
                let ident = Some(method_call.seg.ident.to_string());
                NodeWeightKind::Call(kind_string, ident)
            }
            ExprKind::MacCall(mac_call) => {
                let ident = Some(
                    mac_call
                        .path
                        .segments
                        .iter()
                        .map(|seg| seg.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::"),
                );
                NodeWeightKind::Call(kind_string, ident)
            }

            // leafs
            ExprKind::Lit(..)
            | ExprKind::Break(..)
            | ExprKind::Continue(..)
            | ExprKind::Ret(..)
            | ExprKind::InlineAsm(..)
            | ExprKind::Yield(..)
            | ExprKind::Yeet(..)
            | ExprKind::Become(..)
            | ExprKind::Err(..) => NodeWeightKind::Leaf(kind_string),

            // no weight
            ExprKind::Underscore
            | ExprKind::Path(..)
            | ExprKind::OffsetOf(..)
            | ExprKind::IncludedBytes(..)
            | ExprKind::FormatArgs(..)
            | ExprKind::Dummy => NodeWeightKind::NoWeight(kind_string),
        };

        self.pre_walk(kind, ident, node_id);
        walk_expr(self, cur_ex);
        self.post_walk(node_id);
    }

    /// Visit item, like functions, structs, enums
    fn visit_item(&mut self, cur_item: &'ast Item) {
        let ident = Some(cur_item.ident.to_string());
        let node_id = cur_item.id;
        let kind_string = NodeWeightKind::parse_kind_variant_name(format!("{:?}", &cur_item.kind));
        let kind = match &cur_item.kind {
            // blocks
            ItemKind::Fn(..)
            | ItemKind::Mod(..)
            | ItemKind::Enum(..)
            | ItemKind::Struct(..)
            | ItemKind::Trait(..)
            | ItemKind::Impl(..)
            | ItemKind::Union(..)
            | ItemKind::TraitAlias(..)
            | ItemKind::MacroDef(..) => NodeWeightKind::Block(kind_string),

            // calls
            ItemKind::MacCall(mac_call) => {
                let ident = Some(
                    mac_call
                        .path
                        .segments
                        .iter()
                        .map(|seg| seg.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::"),
                );
                NodeWeightKind::Call(kind_string, ident)
            }

            // leafs
            ItemKind::Static(..)
            | ItemKind::Const(..)
            | ItemKind::GlobalAsm(..)
            | ItemKind::TyAlias(..) => NodeWeightKind::Leaf(kind_string),

            // no weight
            ItemKind::Use(..)
            | ItemKind::ExternCrate(..)
            | ItemKind::ForeignMod(..)
            | ItemKind::Delegation(..)
            | ItemKind::DelegationMac(..) => NodeWeightKind::NoWeight(kind_string),
        };

        self.pre_walk(kind, ident, node_id);
        walk_item(self, cur_item);
        self.post_walk(node_id);
    }

    /// Visit associated items, like methods in impls
    fn visit_assoc_item(&mut self, cur_aitem: &'ast AssocItem, ctxt: AssocCtxt) -> Self::Result {
        let ident = Some(cur_aitem.ident.to_string());
        let node_id = cur_aitem.id;
        let kind_string = NodeWeightKind::parse_kind_variant_name(format!("{:?}", &cur_aitem.kind));
        let kind = match &cur_aitem.kind {
            // blocks
            AssocItemKind::Fn(..) => NodeWeightKind::Block(kind_string),

            // calls
            AssocItemKind::MacCall(mac_call) => {
                let ident = Some(
                    mac_call
                        .path
                        .segments
                        .iter()
                        .map(|seg| seg.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::"),
                );
                NodeWeightKind::Call(kind_string, ident)
            }

            // leafs
            AssocItemKind::Const(..) => NodeWeightKind::Leaf(kind_string),

            // no weight
            AssocItemKind::Type(..) => NodeWeightKind::NoWeight(kind_string),
            AssocItemKind::Delegation(..) => NodeWeightKind::NoWeight(kind_string),
            AssocItemKind::DelegationMac(..) => NodeWeightKind::NoWeight(kind_string),
        };

        self.pre_walk(kind, ident, node_id);
        walk_assoc_item(self, cur_aitem, ctxt);
        self.post_walk(node_id);
    }

    /// Visit statement, like let, if, while
    fn visit_stmt(&mut self, cur_stmt: &'ast Stmt) -> Self::Result {
        let ident = None;
        let node_id = cur_stmt.id;
        let kind_string = NodeWeightKind::parse_kind_variant_name(format!("{:?}", &cur_stmt.kind));
        let kind = match &cur_stmt.kind {
            // blocks
            StmtKind::Item(..) => NodeWeightKind::Block(kind_string),

            // calls
            StmtKind::MacCall(mac_call) => {
                let ident = Some(
                    mac_call
                        .mac
                        .path
                        .segments
                        .iter()
                        .map(|seg| seg.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::"),
                );
                NodeWeightKind::Call(kind_string, ident)
            }

            // leafs
            StmtKind::Let(..) | StmtKind::Expr(..) | StmtKind::Semi(..) => {
                NodeWeightKind::Leaf(kind_string)
            }

            // no weight
            StmtKind::Empty => NodeWeightKind::NoWeight(kind_string),
        };

        self.pre_walk(kind, ident, node_id);
        walk_stmt(self, cur_stmt);
        self.post_walk(node_id);
    }

    /// Visit definition fields, like struct fields
    fn visit_field_def(&mut self, cur_field: &'ast FieldDef) -> Self::Result {
        let ident = None;
        let node_id = cur_field.id;
        let kind_string = "FieldDef".to_string();
        let kind = NodeWeightKind::Leaf(kind_string);

        self.pre_walk(kind, ident, node_id);
        walk_field_def(self, cur_field);
        self.post_walk(node_id);
    }

    /// Visit enum variant
    fn visit_variant(&mut self, cur_var: &'ast Variant) -> Self::Result {
        let ident = Some(cur_var.ident.to_string());
        let node_id = cur_var.id;
        let kind_string = "Variant".to_string();
        let kind = NodeWeightKind::Leaf(kind_string);

        self.pre_walk(kind, ident, node_id);
        walk_variant(self, cur_var);
        self.post_walk(node_id);
    }

    /// Visits match arm
    fn visit_arm(&mut self, cur_arm: &'ast Arm) -> Self::Result {
        let ident = None;
        let node_id = cur_arm.id;
        let kind_string = "Arm".to_string();
        let kind = NodeWeightKind::Leaf(kind_string);

        self.pre_walk(kind, ident, node_id);
        walk_arm(self, cur_arm);
        self.post_walk(node_id);
    }

    /// Visits a function parameter
    fn visit_param(&mut self, cur_par: &'ast Param) -> Self::Result {
        let ident = None;
        let node_id = cur_par.id;
        let kind_string = "Param".to_string();
        let kind = NodeWeightKind::NoWeight(kind_string);

        self.pre_walk(kind, ident, node_id);
        walk_param(self, cur_par);
        self.post_walk(node_id);
    }
}
