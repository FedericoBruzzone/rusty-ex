#![feature(rustc_private)]

pub mod configs;
pub mod instrument;
pub mod types;
mod utils;

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use clap::Parser;
use configs::centrality::{Centrality, CentralityKind};
use instrument::{CrateFilter, RustcPlugin, RustcPluginArgs, Utf8Path};
use linked_hash_set::LinkedHashSet;
use rustc_ast::{ast::*, visit::*};
use rustc_span::symbol::*;
use rustworkx_core::dag_algo::longest_path;
use rustworkx_core::petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::{borrow::Cow, env};
use std::{fs, io, panic};
use types::*;

// This struct is the plugin provided to the rustc_plugin framework,
// and it must be exported for use by the CLI/driver binaries.
pub struct RustcEx;

// To parse CLI arguments, we use Clap for this example. But that
// detail is up to you.
#[derive(Parser, Serialize, Deserialize, Debug, Default)]
pub struct PrintAstArgs {
    /// Pass --print-terms-tree to print the Terms Tree in DOT format
    #[clap(long)]
    print_terms_tree: bool,

    /// Pass --print-features-multigraph to print the Features Graph before squashing edges in DOT format
    #[clap(long)]
    print_features_multigraph: bool,

    /// Pass --print-features-graph to print the Features Graph in DOT format
    #[clap(long)]
    print_features_graph: bool,

    /// Pass --print-artifacts-tree to print the Artifacts Tree in DOT format
    #[clap(long)]
    print_artifacts_tree: bool,

    /// Pass --print-crate to print the crate AST
    #[clap(long)]
    print_crate: bool,

    /// Pass --print-centrality to print some centrality measures on Features Graph
    #[clap(long)]
    pretty_print_centrality: bool,

    /// Pass --serialized-centrality followed by the centrality measure to print the serialized centrality
    #[clap(long, value_enum)]
    serialized_centrality: Option<CentralityKind>,

    /// Pass --print-serialized-graphs to print all extracted graphs serialized
    #[clap(long)]
    print_serialized_graphs: bool,

    /// Pass --print-metadata to print the metadata of the extracted data
    #[clap(long)]
    print_metadata: bool,

    #[clap(last = true)]
    // mytool --allcaps -- some extra args here
    //                     ^^^^^^^^^^^^^^^^^^^^ these are cargo args
    cargo_args: Vec<String>,
}

impl clap::ValueEnum for CentralityKind {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::All, Self::Katz, Self::Closeness, Self::Eigenvector]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Self::All => Some(clap::builder::PossibleValue::new("all")),
            Self::Katz => Some(clap::builder::PossibleValue::new("katz")),
            Self::Closeness => Some(clap::builder::PossibleValue::new("closeness")),
            Self::Eigenvector => Some(clap::builder::PossibleValue::new("eigenvector")),
        }
    }
}

impl RustcPlugin for RustcEx {
    type Args = PrintAstArgs;

    fn version(&self) -> Cow<'static, str> {
        env!("CARGO_PKG_VERSION").into()
    }

    fn driver_name(&self) -> Cow<'static, str> {
        "rusty-ex-driver".into()
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
        // In tests we run something like `cargo rusty-ex --print-dot` because the plugin is installed as a binary in a temporary directory.
        // It is expanded to `/tmp/rusty-ex/bin/cargo-rusty-ex rusty-ex --print-dot`, so we need to skip the first argument because it is the `cargo` command.
        //
        // ## Cli
        // In the CLI we run something like `cargo run --bin rusty-ex -- --print-dot` or `./target/debug/cargo-rusty-ex --print-dot`.
        // It is expanded to `.target/debug/cargo-rusty-ex --print-dot`, so we don't need to skip the first argument.
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
        if self.args.print_terms_tree {
            collector.terms_tree.print_dot();
        }
        if self.args.print_features_graph {
            collector.squash_feature_graph_edges().print_dot();
        }
        if self.args.print_features_multigraph {
            collector.features_graph.print_dot();
        }
        if self.args.print_artifacts_tree {
            collector.artifacts_tree.print_dot();
        }
        if self.args.pretty_print_centrality {
            collector.pretty_print_centrality();
        }
        if let Some(centrality) = &self.args.serialized_centrality {
            collector.serialized_centrality(centrality);
        }
        if self.args.print_serialized_graphs {
            collector.print_serialized_graphs();
        }
        if self.args.print_metadata {
            collector.print_metadata();
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
                    // Features are discarded before the `after_expansion` hook, so are lost.
                    // To avoid this, we replace all `cfg` directives with a custom config.
                    .replace("#[cfg(", "#[rustex_cfg(")
                    // The `cfg!` macro is evaluated before the `after_expansion` hook, so we replace it with a custom one.
                    // The replacement is not a macro because the macro would still be evaluated before the hook,
                    // giving an error in the AST.
                    .replace("cfg!", "rustex_cfg"))
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

    // TODO: macros are not expanded in `after_crate_root_parsing`, but external files are not parsed!
    // fn after_crate_root_parsing<'tcx>(
    //     &mut self,
    //     _compiler: &rustc_interface::interface::Compiler,
    //     queries: &'tcx rustc_interface::Queries<'tcx>,
    // ) -> rustc_driver::Compilation {
    //     queries
    //         .global_ctxt()
    //         .expect("Error: global context not found")
    //         .enter(|tcx: rustc_middle::ty::TyCtxt| {
    //             let krate = &tcx.crate_for_resolver(()).steal().0;

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
                    node_id_incr: 1, // 0 is reserved for global scope
                    stack: Vec::new(),

                    terms_tree: TermsTree::new(),
                    features_graph: FeaturesGraph::new(),
                    artifacts_tree: ArtifactsTree::new(),

                    idents_weights: HashMap::new(),
                    weights_to_resolve: LinkedHashSet::new(),

                    centrality: None,
                };

                // initialize global scope (global feature and artifact)
                collector.init_global_scope();

                // visit AST and build Terms Tree (UIR)
                collector.visit_crate(krate);

                // build features and artifacts tree visiting Terms Tree
                collector.build_feat_graph();
                collector.build_arti_graph();

                // calculate weights of Terms
                collector.terms_tree.graph.reverse(); // reverse graph
                collector.rec_weight_terms_tree(TermIndex::new(GLOBAL_NODE_INDEX));
                collector.resolve_weights_in_wait();
                collector.terms_tree.graph.reverse(); // restore graph

                collector.add_dummy_centrality_node_edges();

                let refiner_hm = collector
                    .artifacts_tree
                    .refiner_hash_map(&collector.features_graph, true);
                let centrality = Centrality::new(&collector.features_graph, true);
                let refined = centrality.refine(refiner_hm);
                collector.centrality.replace(refined);

                self.process_cli_args(collector, krate);
            });

        rustc_driver::Compilation::Stop
    }
}

/// Constant for the global feature NodeId.
/// 4294967040 is not used because it is used by the compiler for "Dummy" nodes
pub const GLOBAL_NODE_ID: NodeId = NodeId::from_u32(4294967039);
/// Constant for the global feature name
pub const GLOBAL_FEATURE_NAME: &str = "__GLOBAL__";
/// Index of the global Term/Feature/Artifact in the graphs
pub const GLOBAL_NODE_INDEX: usize = 0;

/// Constant for the global dummy node name
pub const GLOBAL_DUMMY_NAME: &str = "__DUMMY__";
/// Index of the dummy feature node in the features graph
pub const GLOBAL_DUMMY_INDEX: usize = 1;

/// Weight assigned to Term nodes that cannot be resolved
pub const RECOVERY_WEIGHT: TermWeight = TermWeight::Weight(7.0); // TODO: this should be the mean of all external function weights. This, of course, needs to be calculated a priori and passed as an argument. For now, we use `7.0` because of its intrinsic beauty.

/// AST visitor to collect data to build the graphs
pub struct CollectVisitor {
    /// NodeId in nodes are not resolved yet, so we need to increment it manually
    node_id_incr: u32,
    /// Stack to keep track of the terms dependencies
    stack: Vec<(TermIndex, ComplexFeature<Feature>)>,

    /// Relationships between all terms (all pieces of code annotated or not)
    terms_tree: TermsTree<SimpleTermKey>,
    /// Multigraph storing relationships between features
    features_graph: FeaturesGraph,
    /// Relationships between all artifacts (terms nodes with features)
    artifacts_tree: ArtifactsTree<SimpleArtifactKey>,

    /// Weights of the already weighted idents (needed to weight the References).
    /// The weights for a single ident can be multiple, because a function can
    /// be defined multiple times (with different #[cfg] attributes)
    idents_weights: HashMap<String, Vec<f64>>,
    /// Terms that are waiting for something to be resolved.
    /// This needs to be a set, but with insertion order preserved (a "unique" queue)
    weights_to_resolve: LinkedHashSet<TermIndex>,

    /// Centrality measures of the Features Graph
    centrality: Option<Centrality>,
}

impl CollectVisitor {
    /// Returns the next NodeId and increments the counter. This is needed because
    /// the compiler has not resolved the ids yet
    fn get_node_id(&mut self) -> NodeId {
        let node_id = self.node_id_incr;
        self.node_id_incr += 1;
        NodeId::from_u32(node_id)
    }

    /// Initialize the global scope (term node, feature node, artifact node)
    fn init_global_scope(&mut self) {
        let ident = Some(GLOBAL_FEATURE_NAME.to_string());
        let node_id = GLOBAL_NODE_ID;
        let feature = Feature {
            name: GLOBAL_FEATURE_NAME.to_string(),
            not: false,
        };
        let features = ComplexFeature::Simple(feature.clone());
        let artifact = SimpleArtifactKey(node_id);

        let index = self.terms_tree.create_node(
            SimpleTermKey(node_id),
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
        assert!(complex_feature.insert(ComplexFeature::Simple(feature.clone())));
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
            artifact,
            ident,
            ComplexFeature::Simple(feature.clone()),
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
        assert!(complex_feature.insert(ComplexFeature::Simple(dummy_feature.clone())));
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

    /// Recursively visit nested features (all, any, not), creating features nodes
    fn rec_expand_features(
        &mut self,
        nested_meta: Vec<MetaItemInner>,
        not: bool,
    ) -> Vec<ComplexFeature<Feature>> {
        let mut features = Vec::new();

        for meta in nested_meta {
            match meta.name_or_empty() {
                sym::feature => {
                    let name = meta
                        .value_str()
                        .expect("Error: malformed feature without value `#[cfg(feature)]`")
                        .to_string();

                    let feature = Feature { name, not };
                    self.features_graph.create_node(
                        FeatureKey(feature.clone()),
                        None,           // to be valued later
                        HashSet::new(), // to be valued later
                    );

                    features.push(ComplexFeature::Simple(feature));
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
                name => {
                    // Covering built-in cfg, e.g.,#[cfg(linux)]
                    let feature = Feature {
                        name: name.to_ident_string(),
                        not,
                    };
                    self.features_graph.create_node(
                        FeatureKey(feature.clone()),
                        None,           // to be valued later
                        HashSet::new(), // to be valued later
                    );
                    features.push(ComplexFeature::Simple(feature));
                }
            }
        }

        features
    }

    /// Weight features horizontally, considering only the "siblings"
    fn rec_weight_feature(features: &ComplexFeature<Feature>) -> Vec<(FeatureKey, f64)> {
        match features {
            ComplexFeature::None => Vec::new(),
            ComplexFeature::Simple(feature) => Vec::from([(FeatureKey(feature.clone()), 1.0)]),
            ComplexFeature::All(nested) => {
                let size = nested.len() as f64;

                nested
                    .iter()
                    .flat_map(|features| {
                        CollectVisitor::rec_weight_feature(features)
                            .into_iter()
                            .map(|(feature, weight)| (feature, weight / size))
                    })
                    .collect()
            }
            ComplexFeature::Any(nested) => nested
                .iter()
                .flat_map(CollectVisitor::rec_weight_feature)
                .collect(),
        }
    }

    /// Update the term node with the found features and create the dependency (edge)
    /// in the Terms Tree. The parent TermNode should already exist (anothe node or global scope)
    fn update_term_node_features(&mut self, node_id: NodeId, features: ComplexFeature<Feature>) {
        // update the node with the found and weighted cfgs
        let node_index: &TermIndex = self
            .terms_tree
            .nodes
            .get(&SimpleTermKey(node_id))
            .expect("Error: cannot find Term node updating features");

        self.terms_tree
            .graph
            .node_weight_mut(*node_index)
            .expect("Error: cannot find Term node updating features")
            .features = features;

        // create edge in the graph, to the parent or to the global scope
        match self.stack.last() {
            Some((parent_index, ..)) => {
                self.terms_tree
                    .graph
                    .add_edge(*node_index, *parent_index, Edge { weight: 0.0 });
            }
            None => {
                self.terms_tree.graph.add_edge(
                    *node_index,
                    TermIndex::new(GLOBAL_NODE_INDEX),
                    Edge { weight: 0.0 },
                );
            }
        }
    }

    /// Initialize a new Term node and update the stack
    fn pre_walk(&mut self, kind: TermWeightKind, ident: Option<String>, node_id: NodeId) {
        let term_index = self.terms_tree.create_node(
            SimpleTermKey(node_id),
            ident,
            ComplexFeature::None,
            kind,
            TermWeight::ToBeCalculated,
        );
        self.stack.push((term_index, ComplexFeature::None));
    }

    /// Extract the features of the Term node from the stack and update the Terms Tree
    fn post_walk(&mut self, node_id: NodeId) {
        let (node_index, features) = self
            .stack
            .pop()
            .expect("Error: stack is empty while in expression");

        let term_node = self
            .terms_tree
            .graph
            .node_weight_mut(node_index)
            .expect("Error: missing node post AST walk");

        assert_eq!(
            node_id, term_node.node_id.0,
            "Error: node id mismatch post AST walk"
        );

        // create artifact if some features are found
        if features != ComplexFeature::None {
            let ident = term_node.ident.clone();
            // convert features to index of the features (the features node already exist)

            self.artifacts_tree.create_node(
                SimpleArtifactKey(node_id),
                ident,
                features.clone(),
                TermWeight::ToBeCalculated,
            );
        }

        // insert found features in node
        self.update_term_node_features(node_id, features);
    }

    /// Recursively fet the first parent node with features. The only node with no
    /// annotated parents is the global scope
    fn rec_get_annotated_parent(
        graph: &DiGraph<TermNode<SimpleTermKey>, Edge>,
        start_index: TermIndex,
    ) -> Option<TermIndex> {
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

    /// Recursively weight (in place) the Term nodes in the Terms Tree, starting from the global node
    fn rec_weight_terms_tree(&mut self, start_index: TermIndex) -> TermWeight {
        // TODO: ci sono altre cose fa considerare come Reference weight?

        // IMPORTANT! We cannot skip already weighted nodes because a re-evaluation may be needed.
        // For example, if the weight an ident has changed (multiple definitions), then we need to
        // update the weight of all calls to that ident.
        // This works because the root (GLOBAL) is in Wait state until everything is resolved.

        let adjacents = self
            .terms_tree
            .graph
            .neighbors(start_index)
            .collect::<Vec<_>>()
            .into_iter()
            .map(|index| self.rec_weight_terms_tree(index))
            .collect::<Vec<_>>();

        let mut child_weight = 0.0;
        for adj_weight in adjacents {
            match adj_weight {
                TermWeight::ToBeCalculated => {
                    panic!("Error: recursion not working while weighting Terms Tree")
                }
                TermWeight::Wait(dep) => {
                    self.update_weight(start_index, TermWeight::Wait(dep.clone()));
                    self.weights_to_resolve.insert(start_index);
                    return TermWeight::Wait(dep);
                }
                TermWeight::Weight(w) => child_weight += w,
            }
        }

        let weight = match &self
            .terms_tree
            .graph
            .node_weight(start_index)
            .expect("Error: cannot find Term node weighting Terms Tree")
            .weight_kind
        {
            TermWeightKind::Intrinsic(..) => TermWeight::Weight(1.0 + child_weight),
            TermWeightKind::Children(..) => TermWeight::Weight(child_weight),
            TermWeightKind::Reference(.., Some(to)) => match self.idents_weights.get(to) {
                Some(vec_fn_weight) => TermWeight::Weight(
                    (vec_fn_weight.iter().sum::<f64>() / vec_fn_weight.len() as f64) + child_weight,
                ),
                None => {
                    self.weights_to_resolve.insert(start_index);
                    TermWeight::Wait(to.to_string())
                }
            },
            TermWeightKind::Reference(.., None) => TermWeight::Weight(child_weight),
            TermWeightKind::No(..) => TermWeight::Weight(0.0),
        };

        self.update_weight(start_index, weight.clone());
        weight
    }

    /// Calculate the weights of the nodes in wait
    fn resolve_weights_in_wait(&mut self) {
        let mut seen = HashSet::new();

        while let Some(cur_index) = self.weights_to_resolve.pop_front() {
            // prevent infinte loop: if the same index is seen with the same number
            // of unsolved weights, then it's a loop. Enter recovery mode: weight the
            // deepest node and continue
            if seen.contains(&(cur_index, self.weights_to_resolve.len())) {
                // because of how the queue is formed, the node that first detects a loop
                // is also the "deepest" one, the one that has all adjacents fully resolved
                self.update_weight(cur_index, RECOVERY_WEIGHT);

                continue;
            }

            seen.insert((cur_index, self.weights_to_resolve.len()));

            // try to weight the node, if it's not possible, it will be added again to the queue
            self.rec_weight_terms_tree(cur_index);
        }
    }

    /// Update the weight of the Term node.
    /// Remove the updated node from nodes in wait (only if the weight is not a Wait).
    /// Add the weight to the `idents_weights` map if the node has an ident
    fn update_weight(&mut self, term_index: TermIndex, weight: TermWeight) {
        let term_node = self
            .terms_tree
            .graph
            .node_weight_mut(term_index)
            .expect("Error: cannot find Term node updating weight");

        if term_node.features != ComplexFeature::None {
            let artifact_index = self
                .artifacts_tree
                .nodes
                .get(&SimpleArtifactKey(term_node.node_id.0))
                .expect("Error: cannot find artifact index updating weight");
            let artifact_node = self
                .artifacts_tree
                .graph
                .node_weight_mut(*artifact_index)
                .expect("Error: cannot find artifact node updating weight");
            artifact_node.weight = weight.clone();
        }

        term_node.weight = weight.clone();

        // remove from nodes in wait
        if let TermWeight::Weight(..) = weight {
            self.weights_to_resolve.remove(&term_index);
        }

        // add to idents map if it has an ident
        if let (TermWeight::Weight(weight), Some(ident)) = (weight, term_node.ident.clone()) {
            self.idents_weights.entry(ident).or_default().push(weight);
        }
    }

    /// Build the features graph from the Terms Tree
    fn build_feat_graph(&mut self) {
        self.terms_tree
            .nodes
            .iter()
            // ignore global node
            .filter(|(_, node_index)| *node_index != &NodeIndex::new(GLOBAL_NODE_INDEX))
            .for_each(|(.., child_term_index)| {
                let child_term_node = &self
                    .terms_tree
                    .graph
                    .node_weight(*child_term_index)
                    .expect("Error: cannot find child node creating features graph");
                let child_features = CollectVisitor::rec_weight_feature(&child_term_node.features);

                let parent_term_index = CollectVisitor::rec_get_annotated_parent(
                    &self.terms_tree.graph,
                    *child_term_index,
                )
                .expect("Error: cannot find parent creating features graph");
                let parent_features = CollectVisitor::rec_weight_feature(
                    &self
                        .terms_tree
                        .graph
                        .node_weight(parent_term_index)
                        .expect("Error: cannot find parent node creating features graph")
                        .features,
                );

                child_features
                    .iter()
                    // cartesian product
                    .flat_map(|x| parent_features.iter().map(move |y| (x, y)))
                    .for_each(|((child_feat, child_weight), (parent_feat, ..))| {
                        let child_index = self
                            .features_graph
                            .nodes
                            .get(child_feat)
                            .expect("Error: cannot find feature node creating features graph");

                        let child_node = self
                            .features_graph
                            .graph
                            .node_weight_mut(*child_index)
                            .expect("Error: cannot find feature node creating features graph");

                        // TODO: vedere se rimuovere weight o accumularlo in qualche modo
                        child_node.weight = Some(*child_weight);
                        child_node
                            .complex_feature
                            .insert(child_term_node.features.clone());

                        self.features_graph.graph.add_edge(
                            *child_index,
                            *self
                                .features_graph
                                .nodes
                                .get(parent_feat)
                                .expect("Error: cannot find feature node creating features graph"),
                            Edge {
                                weight: *child_weight,
                            },
                        );
                    });
            });
    }

    /// Build the artifacts tree from the Terms Tree
    fn build_arti_graph(&mut self) {
        self.terms_tree
            .nodes
            .iter()
            // ignore global node
            .filter(|(.., node_index)| *node_index != &NodeIndex::new(GLOBAL_NODE_INDEX))
            // ignore nodes with no features
            .filter(|(.., node_index)| {
                let child_node = &self
                    .terms_tree
                    .graph
                    .node_weight(**node_index)
                    .expect("Error: cannot find child node creating features graph");

                child_node.features != ComplexFeature::None
            })
            // get first annotated parent for each node
            // the index is in the Terms Tree, not in the artifacts tree
            .map(|(.., child_index)| {
                let parent_index =
                    CollectVisitor::rec_get_annotated_parent(&self.terms_tree.graph, *child_index)
                        .expect("Error: cannot find parent creating features graph");

                (child_index, parent_index)
            })
            // add edge between child and parent in the artifacts tree
            // we need to convert the TermIndex to the ArtifactIndex
            .for_each(|(child_term_index, parent_term_index)| {
                let child_term_node = &self
                    .terms_tree
                    .graph
                    .node_weight(*child_term_index)
                    .expect("Error: cannot find child node creating features graph");
                let parent_term_node = &self
                    .terms_tree
                    .graph
                    .node_weight(parent_term_index)
                    .expect("Error: cannot find parent node creating features graph");

                let child_arti_index = self
                    .artifacts_tree
                    .nodes
                    .get(&SimpleArtifactKey(child_term_node.node_id.0))
                    .expect("Error: cannot find child artifact node creating artifacts tree");
                let parent_arti_index = self
                    .artifacts_tree
                    .nodes
                    .get(&SimpleArtifactKey(parent_term_node.node_id.0))
                    .expect("Error: cannot find child artifact node creating artifacts tree");

                self.artifacts_tree.graph.add_edge(
                    *child_arti_index,
                    *parent_arti_index,
                    Edge { weight: 0.0 },
                );
            });
    }

    /// Serialize the centrality measures of the Features Graph
    fn serialized_centrality(&self, kind: &CentralityKind) {
        match kind {
            CentralityKind::All => {
                let measures = &self.centrality.as_ref().unwrap();
                println!(
                    "{}",
                    serde_json::to_string(measures).expect("Error: cannot serialize data")
                );
            }
            CentralityKind::Katz => {
                let katz = self.centrality.as_ref().unwrap().katz();
                println!(
                    "{}",
                    serde_json::to_string(&katz).expect("Error: cannot serialize data")
                );
            }
            CentralityKind::Closeness => {
                let closeness = self.centrality.as_ref().unwrap().closeness();
                println!(
                    "{}",
                    serde_json::to_string(&closeness).expect("Error: cannot serialize data")
                );
            }
            CentralityKind::Eigenvector => {
                let eigenvector = self.centrality.as_ref().unwrap().eigenvector();
                println!(
                    "{}",
                    serde_json::to_string(&eigenvector).expect("Error: cannot serialize data")
                );
            }
        }
    }

    /// Print some centrality measures of the features graph
    fn pretty_print_centrality(&self) {
        self.centrality
            .as_ref()
            .unwrap()
            .pretty_print(&self.features_graph)
    }

    /// Print all extracted graphs serialized
    fn print_serialized_graphs(&self) {
        let graphs = SimpleSerialization {
            terms_tree: self.terms_tree.clone(),
            features_graph: self.features_graph.clone(),
            artifacts_tree: self.artifacts_tree.clone(),
        };

        println!(
            "{}",
            serde_json::to_string(&graphs).expect("Error: cannot serialize data")
        );
    }

    /// Add a dummy node in features graph, connected from the root (global feature)
    /// to all nodes. This makes the graph strongly connected, so we can calculate
    /// centrality measures, without affecting much the results because all nodes have
    /// just one more incoming edge
    fn add_dummy_centrality_node_edges(&mut self) {
        self.features_graph
            .graph
            .node_indices()
            .filter(|n| {
                *n != FeatureIndex::new(GLOBAL_DUMMY_INDEX)
                    && *n != FeatureIndex::new(GLOBAL_NODE_INDEX)
            })
            .for_each(|n| {
                self.features_graph.graph.add_edge(
                    FeatureIndex::new(GLOBAL_DUMMY_INDEX),
                    n,
                    Edge { weight: 1.0 },
                );
            });
    }

    /// Print metadata about the graphs in json format
    fn print_metadata(&self) {
        #[derive(Serialize)]
        struct Metadata {
            term_nodes: u32,
            term_edges: u32,
            term_height: u32,

            features_nodes: u32,
            features_edges: u32,
            features_squashed_edges: u32,

            artifacts_nodes: u32,
            artifacts_edges: u32,
        }

        let metadata = Metadata {
            term_nodes: self.terms_tree.graph.node_count() as u32,
            term_edges: self.terms_tree.graph.edge_count() as u32,
            term_height: longest_path(&self.terms_tree.graph, |_| Ok::<i64, &str>(1))
                .expect("Error: cannot calculate longest path")
                .expect("Error: cannot calculate longest path")
                .1 as u32,

            features_nodes: self.features_graph.graph.node_count() as u32,
            features_edges: self.features_graph.graph.edge_count() as u32,
            features_squashed_edges: self.squash_feature_graph_edges().graph.edge_count() as u32,

            artifacts_nodes: self.artifacts_tree.graph.node_count() as u32,
            artifacts_edges: self.artifacts_tree.graph.edge_count() as u32,
        };

        println!(
            "{}",
            serde_json::to_string(&metadata).expect("Error: cannot serialize metadata")
        );
    }

    /// Squash the edges making the multigraph a graph. Edges weight is summed up
    fn squash_feature_graph_edges(&self) -> FeaturesGraph {
        let mut new_graph = FeaturesGraph::new();

        for node in self.features_graph.graph.node_indices() {
            let node_weight = self.features_graph.graph.node_weight(node).unwrap().clone();
            new_graph.create_node(
                node_weight.feature,
                node_weight.weight,
                node_weight.complex_feature,
            );
        }

        for edge in self.features_graph.graph.edge_indices() {
            let (source, target) = self.features_graph.graph.edge_endpoints(edge).unwrap();
            let weight = self.features_graph.graph.edge_weight(edge).unwrap().weight;

            if let Some(existing_edge) = new_graph.graph.find_edge(source, target) {
                let existing_weight = new_graph.graph.edge_weight_mut(existing_edge).unwrap();
                existing_weight.weight += weight;
            } else {
                new_graph.graph.add_edge(source, target, Edge { weight });
            }
        }

        new_graph
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
    // esempio `rustex_cfg!` dato che anche questa macro viene già espansa (in un errore
    // garantito, dato che non la trova).
    // Un workaround potrebbe essere quello di rimpiazzarla con un nome di funzione valido
    // come `rustex_cfg`.

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
            if meta.name_or_empty() == Symbol::intern("rustex_cfg") {
                if let MetaItemKind::List(ref list) = meta.kind {
                    match self.stack.pop() {
                        Some((term_index, ComplexFeature::None)) => {
                            let parsed_features = self.rec_expand_features(list.to_vec(), false);
                            log::info!("Parsed features: {:?}", parsed_features);

                            match parsed_features.len() {
                                // This should be `unreachable!()` because in `rec_expand_features`
                                // we always return at least one feature
                                0 => {
                                    self.stack.push((term_index, ComplexFeature::None));
                                }
                                // well-formed with built-in feature (we need the ignore): #[cfg(windows)]
                                // well-formed with feature (we need the feature): #[cfg(feature = "a"))]
                                1 => {
                                    self.stack.push((term_index, parsed_features[0].to_owned()));
                                }
                                // malformed (panic): #[cfg(feature = "a", feature = "b")]
                                _ => {
                                    panic!("Error: multiple (not nested) features in cfg attribute")
                                }
                            }
                        }
                        Some((.., ComplexFeature::Simple(..)))
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
        let node_id = self.get_node_id();
        let kind_string = TermWeightKind::parse_kind_variant_name(format!("{:?}", &cur_ex.kind));
        let kind = match &cur_ex.kind {
            // children weight
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
            | ExprKind::Try(..) => TermWeightKind::Children(kind_string),

            // reference weight
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
                TermWeightKind::Reference(kind_string, ident)
            }
            ExprKind::MethodCall(method_call) => {
                let ident = Some(method_call.seg.ident.to_string());
                TermWeightKind::Reference(kind_string, ident)
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
                TermWeightKind::Reference(kind_string, ident)
            }

            // intrinsic weight
            ExprKind::Lit(..)
            | ExprKind::Break(..)
            | ExprKind::Continue(..)
            | ExprKind::Ret(..)
            | ExprKind::InlineAsm(..)
            | ExprKind::Yield(..)
            | ExprKind::Yeet(..)
            | ExprKind::Become(..)
            | ExprKind::Path(..)
            | ExprKind::Err(..) => TermWeightKind::Intrinsic(kind_string),

            // no weight
            ExprKind::Underscore
            | ExprKind::OffsetOf(..)
            | ExprKind::IncludedBytes(..)
            | ExprKind::FormatArgs(..)
            | ExprKind::Dummy => TermWeightKind::No(kind_string),
        };

        self.pre_walk(kind, ident, node_id);
        walk_expr(self, cur_ex);
        self.post_walk(node_id);
    }

    /// Visit item, like functions, structs, enums
    fn visit_item(&mut self, cur_item: &'ast Item) {
        let ident = Some(cur_item.ident.to_string());
        let node_id = self.get_node_id();
        let kind_string = TermWeightKind::parse_kind_variant_name(format!("{:?}", &cur_item.kind));
        let kind = match &cur_item.kind {
            // children weight
            ItemKind::Fn(..)
            | ItemKind::Mod(..)
            | ItemKind::Enum(..)
            | ItemKind::Struct(..)
            | ItemKind::Trait(..)
            | ItemKind::Impl(..)
            | ItemKind::Union(..)
            | ItemKind::TraitAlias(..)
            | ItemKind::MacroDef(..) => TermWeightKind::Children(kind_string),

            // reference weight
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
                TermWeightKind::Reference(kind_string, ident)
            }

            // intrinsic weight
            ItemKind::Static(..)
            | ItemKind::Const(..)
            | ItemKind::GlobalAsm(..)
            | ItemKind::TyAlias(..) => TermWeightKind::Intrinsic(kind_string),

            // no weight
            ItemKind::Use(..)
            | ItemKind::ExternCrate(..)
            | ItemKind::ForeignMod(..)
            | ItemKind::Delegation(..)
            | ItemKind::DelegationMac(..) => TermWeightKind::No(kind_string),
        };

        self.pre_walk(kind, ident, node_id);
        walk_item(self, cur_item);
        self.post_walk(node_id);
    }

    /// Visit associated items, like methods in impls
    fn visit_assoc_item(&mut self, cur_aitem: &'ast AssocItem, ctxt: AssocCtxt) -> Self::Result {
        let ident = Some(cur_aitem.ident.to_string());
        let node_id = self.get_node_id();
        let kind_string = TermWeightKind::parse_kind_variant_name(format!("{:?}", &cur_aitem.kind));
        let kind = match &cur_aitem.kind {
            // children weight
            AssocItemKind::Fn(..) => TermWeightKind::Children(kind_string),

            // reference weight
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
                TermWeightKind::Reference(kind_string, ident)
            }

            // instrinsic weight
            AssocItemKind::Const(..) => TermWeightKind::Intrinsic(kind_string),

            // no weight
            AssocItemKind::Type(..) => TermWeightKind::No(kind_string),
            AssocItemKind::Delegation(..) => TermWeightKind::No(kind_string),
            AssocItemKind::DelegationMac(..) => TermWeightKind::No(kind_string),
        };

        self.pre_walk(kind, ident, node_id);
        walk_assoc_item(self, cur_aitem, ctxt);
        self.post_walk(node_id);
    }

    /// Visit statement, like let, if, while
    fn visit_stmt(&mut self, cur_stmt: &'ast Stmt) -> Self::Result {
        let ident = None;
        let node_id = self.get_node_id();
        let kind_string = TermWeightKind::parse_kind_variant_name(format!("{:?}", &cur_stmt.kind));
        let kind = match &cur_stmt.kind {
            // children weight
            StmtKind::Item(..) | StmtKind::Semi(..) => TermWeightKind::Children(kind_string),

            // reference weight
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
                TermWeightKind::Reference(kind_string, ident)
            }

            // intrinsic weight
            StmtKind::Let(..) | StmtKind::Expr(..) => TermWeightKind::Intrinsic(kind_string),

            // no weight
            StmtKind::Empty => TermWeightKind::No(kind_string),
        };

        self.pre_walk(kind, ident, node_id);
        walk_stmt(self, cur_stmt);
        self.post_walk(node_id);
    }

    /// Visit definition fields, like struct fields
    fn visit_field_def(&mut self, cur_field: &'ast FieldDef) -> Self::Result {
        let ident = None;
        let node_id = self.get_node_id();
        let kind_string = "FieldDef".to_string();
        let kind = TermWeightKind::Intrinsic(kind_string);

        self.pre_walk(kind, ident, node_id);
        walk_field_def(self, cur_field);
        self.post_walk(node_id);
    }

    /// Visit enum variant
    fn visit_variant(&mut self, cur_var: &'ast Variant) -> Self::Result {
        let ident = Some(cur_var.ident.to_string());
        let node_id = self.get_node_id();
        let kind_string = "Variant".to_string();
        let kind = TermWeightKind::Intrinsic(kind_string);

        self.pre_walk(kind, ident, node_id);
        walk_variant(self, cur_var);
        self.post_walk(node_id);
    }

    /// Visits match arm
    fn visit_arm(&mut self, cur_arm: &'ast Arm) -> Self::Result {
        let ident = None;
        let node_id = self.get_node_id();
        let kind_string = "Arm".to_string();
        let kind = TermWeightKind::Children(kind_string);

        self.pre_walk(kind, ident, node_id);
        walk_arm(self, cur_arm);
        self.post_walk(node_id);
    }

    /// Visits a function parameter
    fn visit_param(&mut self, cur_par: &'ast Param) -> Self::Result {
        let ident = cur_par.pat.descr();
        let node_id = self.get_node_id();
        let kind_string = "Param".to_string();
        let kind = TermWeightKind::No(kind_string);

        self.pre_walk(kind, ident, node_id);
        walk_param(self, cur_par);
        self.post_walk(node_id);
    }
}
