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
use std::sync::Arc;
use std::{borrow::Cow, env};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use std::{fs, io};

// Costanti per la feature globale
const GLOBAL_NODE_ID: u32 = 4294967040;
const GLOBAL_FEATURE_NAME: &str = "__GLOBAL__";

// This struct is the plugin provided to the rustc_plugin framework,
// and it must be exported for use by the CLI/driver binaries.
pub struct RustcEx;

// To parse CLI arguments, we use Clap for this example. But that
// detail is up to you.
#[derive(Parser, Serialize, Deserialize, Debug, Default)]
pub struct PrintAstArgs {
    /// Pass --print-artifacts-dot to print the DOT graph
    #[clap(long)]
    print_artifacts_dot: bool,

    /// Pass --print-features-dot to print the DOT graph
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
        if self.args.print_artifacts_dot {
            collector.print_a_graph_dot();
        }
        if self.args.print_features_dot {
            collector.print_f_graph_dot();
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
                    .replace("#[cfg(", "#[rustcex_cfg(")
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
                // estrarre l'AST
                let resolver_and_krate = tcx.resolver_for_lowering().borrow();
                let krate = &*resolver_and_krate.1;

                // visitare l'AST
                let collector = &mut CollectVisitor {
                    f_nodes: HashMap::new(),
                    f_graph: graph::DiGraph::new(),

                    statements: Vec::new(),
                    features: Vec::new(),

                    a_nodes: HashMap::new(),
                    a_graph: graph::DiGraph::new(),
                };

                collector.init_global_scope();
                collector.visit_crate(krate);
                collector.build_f_graph();

                self.process_cli_args(collector, krate);
            });

        rustc_driver::Compilation::Stop
    }
}

/// Definizioni per l'estrazione delle feature dall'AST, lo statement annotato e la/le feature
#[derive(Clone, Debug, PartialEq)]
struct Annotated {
    node_id: NodeId,
    ident: Option<String>,
}
#[derive(Clone, Debug)]
enum FeatureType {
    Feat(String),
    Not(Vec<FeatureType>),
    All(Vec<FeatureType>),
    Any(Vec<FeatureType>),
}

/// Definizioni per i grafi
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Feature {
    name: String,
    not: bool,
}
#[derive(Clone, Debug)]
struct WeightedFeature {
    feature: Feature,
    weight: f64,
}
#[derive(Clone, Debug)]
struct Artifact {
    ident: Option<String>,
    node_id: NodeId,
    features: Vec<FeatureType>,
}
#[derive(Clone, Debug)]
struct Edge {
    weight: f64,
}

/// Visitor per la visita :) dell'AST
struct CollectVisitor {
    // stack parallelo: statements con rispettive feature
    statements: Vec<Annotated>,
    features: Vec<Option<Vec<FeatureType>>>,

    // grafo delle features
    f_nodes: HashMap<Feature, (NodeIndex, Rc<RefCell<Feature>>)>,
    f_graph: graph::DiGraph<Rc<RefCell<Feature>>, Edge>,

    // grafo delle dipendenze
    a_nodes: HashMap<NodeId, (NodeIndex, Rc<RefCell<Artifact>>)>,
    a_graph: graph::DiGraph<Rc<RefCell<Artifact>>, Edge>,
}

impl std::fmt::Display for FeatureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeatureType::Feat(name) => write!(f, "{}", name),
            FeatureType::Not(features) => write!(
                f,
                "not({})",
                features
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            FeatureType::All(features) => write!(
                f,
                "all({})",
                features
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            FeatureType::Any(features) => write!(
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

impl CollectVisitor {
    /// Crea l'artefatto (nodo) e lo aggiunge al grafo degli artefatti e alla hashmap dei nodi degli artefatti
    fn create_artifact(
        &mut self,
        ident: Option<String>,
        node_id: NodeId,
        features: Vec<FeatureType>,
    ) {
        // creazione nodo del grafo (e cella Rc)
        let mem_node = Rc::new(RefCell::new(Artifact {
            ident,
            node_id,
            features,
        }));
        let graph_node = self.a_graph.add_node(Rc::clone(&mem_node));
        self.a_nodes
            .insert(node_id, (graph_node, Rc::clone(&mem_node)));
    }

    /// Crea una feature (nodo) e la aggiunge al grafo delle features e alla hashmap dei nodi delle features
    fn create_feature_node(&mut self, feature: Feature) {
        if !self.f_nodes.contains_key(&feature) {
            let feat_node = Rc::new(RefCell::new(feature.clone()));
            let graph_node = self.f_graph.add_node(Rc::clone(&feat_node));
            self.f_nodes
                .insert(feature.clone(), (graph_node, Rc::clone(&feat_node)));
        }
    }

    /// Inizializza lo scope globale (feature e artefatto padre di tutti)
    fn init_global_scope(&mut self) {
        self.create_artifact(
            Some(GLOBAL_FEATURE_NAME.to_string()),
            NodeId::from_u32(GLOBAL_NODE_ID),
            Vec::from([FeatureType::Feat(GLOBAL_FEATURE_NAME.to_string())]),
        );

        self.create_feature_node(Feature {
            name: GLOBAL_FEATURE_NAME.to_string(),
            not: false,
        });
    }

    /// Visita ricorsiva delle feature nestate (all, any, not)
    fn rec_expand(&mut self, nested_meta: Vec<MetaItemInner>, not: bool) -> Vec<FeatureType> {
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
                    };
                    self.create_feature_node(feature.clone());
                    assert!(self.f_nodes.contains_key(&feature));

                    cfgs.push(FeatureType::Feat(name))
                }
                sym::not => cfgs.push(FeatureType::Not(
                    self.rec_expand(
                        meta.meta_item_list()
                            .expect("Error: empty `not` feature attribute")
                            .to_vec(),
                        !not,
                    ),
                )),
                sym::all => cfgs.push(FeatureType::All(
                    self.rec_expand(
                        meta.meta_item_list()
                            .expect("Error: empty `all` feature attribute")
                            .to_vec(),
                        not,
                    ),
                )),
                sym::any => cfgs.push(FeatureType::Any(
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

    /// Pesa le features "orizzontalmente", considerando solo i "fratelli"
    fn rec_weight_feature(features: Vec<FeatureType>) -> Vec<WeightedFeature> {
        let mut weights = Vec::new();

        for feat in features {
            match feat {
                FeatureType::Feat(name) => weights.push(WeightedFeature {
                    feature: Feature { name, not: false },
                    weight: 1.0,
                }),
                FeatureType::Not(nested) => {
                    weights.extend(CollectVisitor::rec_weight_feature(nested).into_iter().map(
                        |WeightedFeature { feature, weight }| WeightedFeature {
                            feature: Feature {
                                name: feature.name,
                                not: !feature.not,
                            },
                            weight,
                        },
                    ))
                }
                FeatureType::All(nested) => {
                    let size = nested.len() as f64;
                    let rec = CollectVisitor::rec_weight_feature(nested);
                    weights.extend(rec.into_iter().map(|WeightedFeature { feature, weight }| {
                        WeightedFeature {
                            feature,
                            weight: weight / size,
                        }
                    }))
                }
                FeatureType::Any(nested) => {
                    weights.extend(CollectVisitor::rec_weight_feature(nested))
                }
            }
        }

        weights
    }

    /// Aggiorna l'artefatto con le feature trovate
    fn update_artifact_features(&mut self, node_id: NodeId, features: Vec<FeatureType>) {
        // aggiornare il nodo con le cfg trovate e pesate
        self.a_nodes.entry(node_id).and_modify(|e| {
            e.1.try_borrow_mut()
                .expect("Error: borrow mut failed on artifacts nodes update")
                .features = features.clone();
        });

        // creare arco del grafo, al padre o allo scope global
        match self.statements.last() {
            Some(Annotated {
                node_id: parent_id, ..
            }) => {
                self.a_graph.add_edge(
                    self.a_nodes
                        .get(&node_id)
                        .expect("Error: cannot find artifact node creating artifacts graph")
                        .0,
                    self.a_nodes
                        .get(parent_id)
                        .expect("Error: cannot find artifact node creating artifacts graph")
                        .0,
                    Edge { weight: 0.0 },
                );
            }
            None => {
                self.a_graph.add_edge(
                    self.a_nodes
                        .get(&node_id)
                        .expect("Error: cannot find artifact node creating artifacts graph")
                        .0,
                    self.a_nodes
                        .get(&NodeId::from_u32(GLOBAL_NODE_ID))
                        .expect("Error: cannot find artifact node creating artifacts graph")
                        .0,
                    Edge { weight: 0.0 },
                );
            }
        }
    }

    /// Inizializza un nuovo artefatto e aggiorna gli stack degli statement e delle feature
    fn pre_walk(&mut self, ident: Option<String>, node_id: NodeId, stmt: Annotated) {
        self.create_artifact(ident, node_id, Vec::new());
        self.statements.push(stmt);
        self.features.push(None);
    }

    /// Estrae le feature dell'artefatto dagli stack e aggiorna il grafo degli artefatti
    fn post_walk(&mut self, node_id: NodeId, stmt: Annotated) {
        // estrarre dallo stack dati sulle cfg
        let ident = self
            .statements
            .pop()
            .expect("Error: stack is empty while in expression");
        assert_eq!(ident, stmt);
        let cfg = self
            .features
            .pop()
            .expect("Error: stack is empty while in expression")
            .unwrap_or_default();

        self.update_artifact_features(node_id, cfg);
    }

    /// Costruisce il grafo delle features dal grafo degli artefatti
    fn build_f_graph(&mut self) {
        let global_node_index = self
            .a_nodes
            .get(&NodeId::from_u32(GLOBAL_NODE_ID))
            .expect("Error: missing global index")
            .0;

        for (_child_node_id, (child_node_index, child_artifact)) in self.a_nodes.iter() {
            let child_features = CollectVisitor::rec_weight_feature(
                child_artifact
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

                assert!(self.a_graph.neighbors(*cur).count() == 1);
                parent_node_index = self
                    .a_graph
                    .neighbors(*cur)
                    .next()
                    .expect("Error: missing parent index building features graph");

                let parent_node_id = self.a_graph[parent_node_index]
                    .try_borrow()
                    .expect("Error: borrow failed on parent nodeid creating features graph")
                    .node_id;

                let parent_features = CollectVisitor::rec_weight_feature(
                    self.a_nodes
                        .get(&parent_node_id)
                        .expect("Error: cannot find artifact node creating edge")
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

            for WeightedFeature {
                feature: child_feat,
                weight: child_weight,
            } in &child_features
            {
                for WeightedFeature {
                    feature: parent_feat,
                    weight: _parent_weight,
                } in &parent_features
                {
                    self.f_graph.add_edge(
                        self.f_nodes
                            .get(child_feat)
                            .expect("Error: cannot find feature node creating features graph")
                            .0,
                        self.f_nodes
                            .get(parent_feat)
                            .expect("Error: cannot find feature node creating features graph")
                            .0,
                        Edge {
                            weight: *child_weight,
                        },
                    );
                }
            }
        }
    }

    /// Lista di features a Stringa
    fn features_to_string(features: &[FeatureType]) -> String {
        features
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Stampa il grafo degli artefatti in formato DOT (per Graphviz)
    fn print_f_graph_dot(&self) {
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
                &self.f_graph,
                &[Config::NodeNoLabel, Config::EdgeNoLabel],
                &get_edge_attr,
                &get_node_attr,
            )
        )
    }

    /// Stampa il grafo delle features in formato DOT (per Graphviz)
    fn print_a_graph_dot(&self) {
        let get_edge_attr = |_g: &graph::DiGraph<Rc<RefCell<Artifact>>, Edge>,
                             edge: graph::EdgeReference<Edge>| {
            format!("label=\"{}\"", edge.weight().weight)
        };

        let get_node_attr =
            |_g: &graph::DiGraph<Rc<RefCell<Artifact>>, Edge>,
             node: (graph::NodeIndex, &Rc<RefCell<Artifact>>)| {
                let artifact = node
                    .1
                    .try_borrow()
                    .expect("Error: borrow failed on artifact graph print");
                match &artifact.ident {
                    Some(ident) => format!(
                        "label=\"{} #[{}]\"",
                        ident,
                        CollectVisitor::features_to_string(&artifact.features)
                    ),
                    None => format!(
                        "label=\"#[{}]\"",
                        CollectVisitor::features_to_string(&artifact.features)
                    ),
                }
            };

        println!(
            "{:?}",
            Dot::with_attr_getters(
                &self.a_graph,
                &[Config::NodeNoLabel, Config::EdgeNoLabel],
                &get_edge_attr,
                &get_node_attr,
            )
        )
    }

    fn print_centrality(&self) {
        let katz: rustworkx_core::Result<Option<Vec<f64>>> =
            rustworkx_core::centrality::katz_centrality(
                &self.f_graph,
                |e| Ok(e.weight().weight),
                None,
                None,
                None,
                None,
                None,
            );

        let closeness = rustworkx_core::centrality::closeness_centrality(&self.f_graph, false);

        let eigenvector: rustworkx_core::Result<Option<Vec<f64>>> =
            rustworkx_core::centrality::eigenvector_centrality(
                &self.f_graph,
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
    // TODO: rilevare features sulle macro

    /// Visita attributo: le feature sono degli attributi
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

    /// Visita espressione: (quasi) tutto può essere annotato e quindi va pushato sullo
    /// stack degli statements per evitare di far crescere quello delle feature senza
    /// che ci sia un corrispettivo statement
    fn visit_expr(&mut self, cur_ex: &'ast Expr) {
        let ident = None;
        let node_id = cur_ex.id;
        let stmt = Annotated {
            node_id,
            ident: ident.clone(),
        };

        self.pre_walk(ident, node_id, stmt.clone());
        walk_expr(self, cur_ex);
        self.post_walk(node_id, stmt);
    }

    /// Visita item: le dichiarazione di funzioni sono item, dentro `walk_item` vengono
    /// visitati anche gli attributi, quindi è dopo la chiamata le feature sono già
    /// nello stack.
    /// Non viene utilizzato `visit_fn` per analizzare le funzioni dato che gli attributi
    /// non sono visitati da `walk_fn` (ma vengono visitati dopo), di conseguenza non
    /// sarebbe possibile associarli alla rispettiva funzione.
    fn visit_item(&mut self, cur_item: &'ast Item) {
        let ident = Some(cur_item.ident.to_string());
        let node_id = cur_item.id;
        let stmt = Annotated {
            node_id,
            ident: ident.clone(),
        };

        self.pre_walk(ident, node_id, stmt.clone());
        walk_item(self, cur_item);
        self.post_walk(node_id, stmt);
    }

    /// Visita i campi di una definizione, come i campi di una struct
    fn visit_field_def(&mut self, cur_field: &'ast FieldDef) -> Self::Result {
        let ident = None;
        let node_id = cur_field.id;
        let stmt = Annotated {
            node_id,
            ident: ident.clone(),
        };

        self.pre_walk(ident, node_id, stmt.clone());
        walk_field_def(self, cur_field);
        self.post_walk(node_id, stmt);
    }

    /// Visita uno statement, come gli assegamenti
    fn visit_stmt(&mut self, cur_stmt: &'ast Stmt) -> Self::Result {
        let ident = None;
        let node_id = cur_stmt.id;
        let stmt = Annotated {
            node_id,
            ident: ident.clone(),
        };

        self.pre_walk(ident, node_id, stmt.clone());
        walk_stmt(self, cur_stmt);
        self.post_walk(node_id, stmt);
    }
}
