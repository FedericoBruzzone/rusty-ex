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
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::{borrow::Cow, env};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use std::{fs, io};

// This struct is the plugin provided to the rustc_plugin framework,
// and it must be exported for use by the CLI/driver binaries.
pub struct RustcEx;

// To parse CLI arguments, we use Clap for this example. But that
// detail is up to you.
#[derive(Parser, Serialize, Deserialize, Debug, Default)]
pub struct PrintAstArgs {
    /// Pass --print-temporary-graph to print the DOT graph
    #[clap(long)]
    print_temporary_dot: bool,

    /// Pass --print-features-graph to print the DOT graph
    #[clap(long)]
    print_features_dot: bool,

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
        if self.args.print_temporary_dot {
            collector.print_temp_graph_dot();
        }
        if self.args.print_features_dot {
            collector.print_feat_graph_dot();
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
                    // parallel stacks: AST nodes with respective features
                    ast_nodes: Vec::new(),
                    features: Vec::new(),

                    // temporary graph to store AST nodes with features
                    temp_nodes: HashMap::new(),
                    temp_graph: graph::DiGraph::new(),

                    // features graph
                    feat_nodes: HashMap::new(),
                    feat_graph: graph::DiGraph::new(),
                };

                collector.init_global_scope();
                collector.visit_crate(krate);
                collector.build_f_graph();

                self.process_cli_args(collector, krate);
            });

        rustc_driver::Compilation::Stop
    }
}

/// Constant for the global feature NodeId
const GLOBAL_NODE_ID: u32 = 4294967040;
/// Constant for the global feature name
const GLOBAL_FEATURE_NAME: &str = "__GLOBAL__";

/// AST node, can be annotated with features
#[derive(Clone, Debug)]
struct ASTNode {
    node_id: NodeId,
    ident: Option<String>,
    features: Vec<ComplexFeature>,
}

/// Simple feature, can be weighted
#[derive(Clone, Debug)]
struct Feature {
    name: String,
    not: bool,
    weight: Option<f64>,
}

/// Complex feature, can be a single feature (not already included), an all or an any
#[derive(Clone, Debug)]
enum ComplexFeature {
    Feature(Feature),
    All(Vec<ComplexFeature>),
    Any(Vec<ComplexFeature>),
}

/// Graphs edge, with weight
#[derive(Clone, Debug)]
struct Edge {
    weight: f64,
}

/// AST visitor to collect data to build the graphs
struct CollectVisitor {
    // parallel stacks: AST nodes with respective features
    ast_nodes: Vec<ASTNode>,
    features: Vec<Option<Vec<ComplexFeature>>>,

    // temporary graph to store AST nodes with features
    temp_nodes: HashMap<NodeId, (NodeIndex, Rc<RefCell<ASTNode>>)>,
    temp_graph: graph::DiGraph<Rc<RefCell<ASTNode>>, Edge>,

    // features graph
    feat_nodes: HashMap<Feature, (NodeIndex, Rc<RefCell<Feature>>)>,
    feat_graph: graph::DiGraph<Rc<RefCell<Feature>>, Edge>,
}

/// Features are comparable
impl Eq for Feature {}

/// Comparison of features (weight is ignored)
impl PartialEq for Feature {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.not == other.not
    }
}

/// Hash of features (weight is ignored)
impl Hash for Feature {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.not.hash(state);
    }
}

impl std::fmt::Display for ComplexFeature {
    /// Complex feature to string
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplexFeature::Feature(Feature { name, not, weight }) => {
                let name = match not {
                    true => "!".to_string() + name,
                    false => name.to_string(),
                };
                let weight = match weight {
                    Some(w) => format!("{:.2}", w),
                    None => "-".to_string(),
                };
                write!(f, "{} ({})", name, weight)
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

impl ComplexFeature {
    /// Complex features list to string
    fn list_to_string(features: &[ComplexFeature]) -> String {
        features
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl CollectVisitor {
    /// Create the AST node and add it to the temporary graph and to the AST nodes hashmap
    fn create_ast_node(
        &mut self,
        ident: Option<String>,
        node_id: NodeId,
        features: Vec<ComplexFeature>,
    ) {
        let mem_node = Rc::new(RefCell::new(ASTNode {
            ident,
            node_id,
            features,
        }));
        let graph_node = self.temp_graph.add_node(Rc::clone(&mem_node));
        self.temp_nodes
            .insert(node_id, (graph_node, Rc::clone(&mem_node)));
    }

    /// Create a feature (node) and add it to the features graph and to the features nodes hashmap
    fn create_feature_node(&mut self, feature: Feature) {
        if !self.feat_nodes.contains_key(&feature) {
            let feat_node = Rc::new(RefCell::new(feature.clone()));
            let graph_node = self.feat_graph.add_node(Rc::clone(&feat_node));
            self.feat_nodes
                .insert(feature.clone(), (graph_node, Rc::clone(&feat_node)));
        }
    }

    /// Initialize the global scope (feature and AST node parent of all)
    fn init_global_scope(&mut self) {
        self.create_ast_node(
            Some(GLOBAL_FEATURE_NAME.to_string()),
            NodeId::from_u32(GLOBAL_NODE_ID),
            Vec::from([ComplexFeature::Feature(Feature {
                name: GLOBAL_FEATURE_NAME.to_string(),
                not: false,
                weight: None,
            })]),
        );

        self.create_feature_node(Feature {
            name: GLOBAL_FEATURE_NAME.to_string(),
            not: false,
            weight: None,
        });
    }

    /// Recursively visit nested features (all, any, not)
    fn rec_expand(&mut self, nested_meta: Vec<MetaItemInner>, not: bool) -> Vec<ComplexFeature> {
        let mut cfgs = Vec::new();

        for meta in nested_meta {
            match meta.name_or_empty() {
                sym::feature => {
                    let name = meta
                        .value_str()
                        .expect("Error: malformed feature without value `#[cfg(feature)]`")
                        .to_string();

                    let feature = Feature {
                        name: name.clone(),
                        not,
                        weight: None,
                    };
                    self.create_feature_node(feature.clone());
                    assert!(self.feat_nodes.contains_key(&feature));

                    cfgs.push(ComplexFeature::Feature(feature))
                }
                sym::not => cfgs.extend(
                    self.rec_expand(
                        meta.meta_item_list()
                            .expect("Error: empty `not` feature attribute")
                            .to_vec(),
                        !not,
                    ),
                ),
                sym::all => cfgs.push(ComplexFeature::All(
                    self.rec_expand(
                        meta.meta_item_list()
                            .expect("Error: empty `all` feature attribute")
                            .to_vec(),
                        not,
                    ),
                )),
                sym::any => cfgs.push(ComplexFeature::Any(
                    self.rec_expand(
                        meta.meta_item_list()
                            .expect("Error: empty `any` feature attribute")
                            .to_vec(),
                        not,
                    ),
                )),
                _ => (),
            }
        }

        cfgs
    }

    /// Weight features horizontally, considering only the "siblings"
    fn rec_weight_feature(features: Vec<ComplexFeature>) -> Vec<Feature> {
        let mut weights: Vec<Feature> = Vec::new();

        for feat in features {
            match feat {
                ComplexFeature::Feature(Feature { name, not, .. }) => weights.push(Feature {
                    name,
                    not,
                    weight: Some(1.0),
                }),
                ComplexFeature::All(nested) => {
                    let size = nested.len() as f64;
                    let rec = CollectVisitor::rec_weight_feature(nested);
                    weights.extend(
                        rec.into_iter()
                            .map(|Feature { name, not, weight }| Feature {
                                name,
                                not,
                                weight: Some(
                                    weight.expect(
                                        "Error: feature without weight while weighting features",
                                    ) / size,
                                ),
                            }),
                    )
                }
                ComplexFeature::Any(nested) => {
                    weights.extend(CollectVisitor::rec_weight_feature(nested))
                }
            }
        }

        weights
    }

    /// Update the AST node with the found features
    fn update_ast_node_features(&mut self, node_id: NodeId, features: Vec<ComplexFeature>) {
        // update the node with the found and weighted cfgs
        self.temp_nodes.entry(node_id).and_modify(|e| {
            e.1.try_borrow_mut()
                .expect("Error: borrow mut failed on temp nodes update")
                .features = features.clone();
        });

        // create edge in the graph, to the parent or to the global scope
        match self.ast_nodes.last() {
            Some(ASTNode {
                node_id: parent_id, ..
            }) => {
                self.temp_graph.add_edge(
                    self.temp_nodes
                        .get(&node_id)
                        .expect("Error: cannot find AST node creating temp graph")
                        .0,
                    self.temp_nodes
                        .get(parent_id)
                        .expect("Error: cannot find AST node creating temp graph")
                        .0,
                    Edge { weight: 0.0 },
                );
            }
            None => {
                self.temp_graph.add_edge(
                    self.temp_nodes
                        .get(&node_id)
                        .expect("Error: cannot find AST node creating temp graph")
                        .0,
                    self.temp_nodes
                        .get(&NodeId::from_u32(GLOBAL_NODE_ID))
                        .expect("Error: cannot find AST node creating temp graph")
                        .0,
                    Edge { weight: 0.0 },
                );
            }
        }
    }

    /// Initialize a new AST node and update the AST nodes and features stacks
    fn pre_walk(&mut self, ident: Option<String>, node_id: NodeId, stmt: ASTNode) {
        self.create_ast_node(ident, node_id, Vec::new());
        self.ast_nodes.push(stmt);
        self.features.push(None);
    }

    /// Extract the features of the AST node from the stacks and update the temporary graph
    fn post_walk(&mut self, node_id: NodeId, stmt: ASTNode) {
        let ident = self
            .ast_nodes
            .pop()
            .expect("Error: stack is empty while in expression");
        assert_eq!(ident.node_id, stmt.node_id);
        let cfg = self
            .features
            .pop()
            .expect("Error: stack is empty while in expression")
            .unwrap_or_default();

        self.update_ast_node_features(node_id, cfg);
    }

    /// Build the features graph from the temporary graph
    fn build_f_graph(&mut self) {
        let global_node_index = self
            .temp_nodes
            .get(&NodeId::from_u32(GLOBAL_NODE_ID))
            .expect("Error: missing global index")
            .0;

        for (_child_node_id, (child_node_index, child_ast_node)) in self.temp_nodes.iter() {
            let child_features = CollectVisitor::rec_weight_feature(
                child_ast_node
                    .try_borrow()
                    .expect("Error: borrow failed on child features creating features graph")
                    .features
                    .clone(),
            );

            if child_features.is_empty() {
                continue;
            }

            // FIXME: porcate varie
            let mut cur = child_node_index;
            let mut parent_node_index;

            let parent_features = loop {
                if cur == &global_node_index {
                    break Vec::new();
                }

                assert!(self.temp_graph.neighbors(*cur).count() == 1);
                parent_node_index = self
                    .temp_graph
                    .neighbors(*cur)
                    .next()
                    .expect("Error: missing parent index building features graph");

                let parent_node_id = self.temp_graph[parent_node_index]
                    .try_borrow()
                    .expect("Error: borrow failed on parent nodeid creating features graph")
                    .node_id;

                let parent_features = CollectVisitor::rec_weight_feature(
                    self.temp_nodes
                        .get(&parent_node_id)
                        .expect("Error: cannot find AST node creating edge")
                        .1
                        .try_borrow()
                        .expect("Error: borrow failed on parent features creating features graph")
                        .features
                        .clone(),
                );

                if parent_features.is_empty() {
                    cur = &parent_node_index;
                } else {
                    break parent_features;
                }
            };

            for child_feat in &child_features {
                for parent_feat in &parent_features {
                    self.feat_graph.add_edge(
                        self.feat_nodes
                            .get(child_feat)
                            .expect("Error: cannot find feature node creating features graph")
                            .0,
                        self.feat_nodes
                            .get(parent_feat)
                            .expect("Error: cannot find feature node creating features graph")
                            .0,
                        Edge {
                            weight: child_feat
                                .weight
                                .expect("Error: feature without weight creating features graph"),
                        },
                    );
                }
            }
        }
    }

    /// Print features graph in DOT format
    fn print_feat_graph_dot(&self) {
        let get_edge_attr = |_g: &graph::DiGraph<Rc<RefCell<Feature>>, Edge>,
                             edge: graph::EdgeReference<Edge>| {
            format!("label=\"{:.2}\"", edge.weight().weight)
        };

        let get_node_attr =
            |_g: &graph::DiGraph<Rc<RefCell<Feature>>, Edge>,
             node: (graph::NodeIndex, &Rc<RefCell<Feature>>)| {
                let feature = node
                    .1
                    .try_borrow()
                    .expect("Error: borrow failed on feature graph print");
                match feature.not {
                    true => format!("label=\"!{}\"", feature.name),
                    false => format!("label=\"{}\"", feature.name),
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

    /// Print temporary graph in DOT format
    fn print_temp_graph_dot(&self) {
        let get_edge_attr = |_g: &graph::DiGraph<Rc<RefCell<ASTNode>>, Edge>,
                             edge: graph::EdgeReference<Edge>| {
            format!("label=\"{}\"", edge.weight().weight)
        };

        let get_node_attr =
            |_g: &graph::DiGraph<Rc<RefCell<ASTNode>>, Edge>,
             node: (graph::NodeIndex, &Rc<RefCell<ASTNode>>)| {
                let ast_node = node
                    .1
                    .try_borrow()
                    .expect("Error: borrow failed on temp graph print");
                match &ast_node.ident {
                    Some(ident) => format!(
                        "label=\"{} #[{}]\"",
                        ident,
                        ComplexFeature::list_to_string(&ast_node.features)
                    ),
                    None => format!(
                        "label=\"#[{}]\"",
                        ComplexFeature::list_to_string(&ast_node.features)
                    ),
                }
            };

        println!(
            "{:?}",
            Dot::with_attr_getters(
                &self.temp_graph,
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
                    self.features.pop();
                    let feat = Some(self.rec_expand(list.to_vec(), false));
                    self.features.push(feat);
                }
            }
        }

        walk_attribute(self, attr);
    }

    /// Visit expression, like function calls
    fn visit_expr(&mut self, cur_ex: &'ast Expr) {
        let ident = None;
        let node_id = cur_ex.id;
        let stmt = ASTNode {
            node_id,
            ident: ident.clone(),
            features: Vec::new(),
        };

        self.pre_walk(ident, node_id, stmt.clone());
        walk_expr(self, cur_ex);
        self.post_walk(node_id, stmt);
    }

    /// Visit item, like functions, structs, enums
    fn visit_item(&mut self, cur_item: &'ast Item) {
        let ident = Some(cur_item.ident.to_string());
        let node_id = cur_item.id;
        let stmt = ASTNode {
            node_id,
            ident: ident.clone(),
            features: Vec::new(),
        };

        self.pre_walk(ident, node_id, stmt.clone());
        walk_item(self, cur_item);
        self.post_walk(node_id, stmt);
    }

    /// Visit definition fields, like struct fields
    fn visit_field_def(&mut self, cur_field: &'ast FieldDef) -> Self::Result {
        let ident = None;
        let node_id = cur_field.id;
        let stmt = ASTNode {
            node_id,
            ident: ident.clone(),
            features: Vec::new(),
        };

        self.pre_walk(ident, node_id, stmt.clone());
        walk_field_def(self, cur_field);
        self.post_walk(node_id, stmt);
    }

    /// Visit statement, like let, if, while
    fn visit_stmt(&mut self, cur_stmt: &'ast Stmt) -> Self::Result {
        let ident = None;
        let node_id = cur_stmt.id;
        let stmt = ASTNode {
            node_id,
            ident: ident.clone(),
            features: Vec::new(),
        };

        self.pre_walk(ident, node_id, stmt.clone());
        walk_stmt(self, cur_stmt);
        self.post_walk(node_id, stmt);
    }

    /// Visit enum variant
    fn visit_variant(&mut self, cur_var: &'ast Variant) -> Self::Result {
        let ident = Some(cur_var.ident.to_string());
        let node_id = cur_var.id;
        let stmt = ASTNode {
            node_id,
            ident: ident.clone(),
            features: Vec::new(),
        };

        self.pre_walk(ident, node_id, stmt.clone());
        walk_variant(self, cur_var);
        self.post_walk(node_id, stmt);
    }

    /// Visita match arm
    fn visit_arm(&mut self, cur_arm: &'ast Arm) -> Self::Result {
        let ident = None;
        let node_id = cur_arm.id;
        let stmt = ASTNode {
            node_id,
            ident: ident.clone(),
            features: Vec::new(),
        };

        self.pre_walk(ident, node_id, stmt.clone());
        walk_arm(self, cur_arm);
        self.post_walk(node_id, stmt);
    }
}
