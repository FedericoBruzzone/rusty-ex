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
                let modified_content = content.replace("#[cfg(", "#[feat(");
                Ok(modified_content)
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
                collector.visit_crate(krate);
                collector.build_f_graph();

                self.process_cli_args(collector, krate);
            });

        rustc_driver::Compilation::Stop
    }
}

/// Definizioni per l'estrazione delle feature dall'AST, lo statement annotato e la/le feature
#[derive(Clone, Debug, PartialEq)]
enum AnnotatedType {
    FunctionDeclaration(NodeId, String),
    Expression(NodeId),
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
    weight: f32,
}
#[derive(Clone, Debug)]
struct Artifact {
    ident: String,
    _node_id: NodeId,
    features: (Vec<FeatureType>, Vec<WeightedFeature>),
}
#[derive(Clone, Debug)]
struct Edge {
    weight: f32,
}

/// Visitor per la visita :) dell'AST
struct CollectVisitor {
    // stack parallelo: statements con rispettive feature
    statements: Vec<AnnotatedType>,
    features: Vec<Option<Vec<FeatureType>>>,

    // grafo delle features
    f_nodes: HashMap<Feature, (NodeIndex, Rc<RefCell<Feature>>)>,
    f_graph: graph::DiGraph<Rc<RefCell<Feature>>, Edge>,

    // grafo delle dipendenze
    a_nodes: HashMap<NodeId, (NodeIndex, Rc<RefCell<Artifact>>)>,
    a_graph: graph::DiGraph<Rc<RefCell<Artifact>>, Edge>,
}

impl FeatureType {
    /// Feature a stringa
    fn to_string(&self) -> String {
        match self {
            FeatureType::Feat(name) => name.clone(),
            FeatureType::Not(features) => format!(
                "not({})",
                features
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            FeatureType::All(features) => format!(
                "all({})",
                features
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            FeatureType::Any(features) => format!(
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
    /// Lista di features a Stringa
    fn features_to_string(features: &[FeatureType]) -> String {
        features
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Costruisce il grafo delle features dal grafo degli artefatti
    fn build_f_graph(&mut self) {
        for (_from_node_id, (from_node_index, from_artifact)) in self.a_nodes.iter() {
            let from_features = from_artifact.borrow().features.1.clone();

            for to_node_index in self.a_graph.neighbors(*from_node_index) {
                let to_node_id = self.a_graph[to_node_index].borrow()._node_id;
                let to_features = self
                    .a_nodes
                    .get(&to_node_id)
                    .expect("Error: cannot find artifact node creating edge")
                    .1
                    .borrow()
                    .features
                    .1
                    .clone();

                for WeightedFeature {
                    feature: f_feat,
                    weight: _f_weight,
                } in &from_features
                {
                    for WeightedFeature {
                        feature: t_feat,
                        weight: t_weight,
                    } in &to_features
                    {
                        self.f_graph.add_edge(
                            self.f_nodes
                                .get(&f_feat)
                                .expect("Error: cannot find feature node creating features graph")
                                .0,
                            self.f_nodes
                                .get(&t_feat)
                                .expect("Error: cannot find feature node creating features graph")
                                .0,
                            Edge { weight: *t_weight },
                        );
                    }
                }
            }
        }
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
                match node.1.borrow().not {
                    true => format!("label=\"!{}\"", node.1.borrow().name),
                    false => format!("label=\"{}\"", node.1.borrow().name),
                }
            };

        println!(
            "{:?}",
            Dot::with_attr_getters(
                &self.f_graph,
                &[Config::EdgeNoLabel],
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
                format!(
                    "label=\"{}\n({})\"",
                    node.1.borrow().ident,
                    CollectVisitor::features_to_string(&node.1.borrow().features.0)
                )
            };

        println!(
            "{:?}",
            Dot::with_attr_getters(
                &self.a_graph,
                &[Config::EdgeNoLabel],
                &get_edge_attr,
                &get_node_attr,
            )
        )
    }
}

impl<'ast> Visitor<'ast> for CollectVisitor {
    /// Visita attributo: le feature sono degli attributi
    fn visit_attribute(&mut self, attr: &'ast Attribute) {
        /// Visita ricorsiva delle feature nestate (all, any, not)
        fn rec_expand(
            visitor: &mut CollectVisitor,
            nested_meta: Vec<MetaItemInner>,
        ) -> Vec<FeatureType> {
            let mut cfgs = Vec::new();

            for meta in nested_meta {
                match meta.name_or_empty() {
                    sym::feature => {
                        // FIXME: esistono meta con `value_str` a None?
                        let name = meta.value_str().unwrap().to_string();

                        let feature = Feature {
                            name: name.clone(),
                            not: false,
                        };
                        let not_feature = Feature {
                            name: name.clone(),
                            not: true,
                        };

                        // creazione nodi feature e not feature se non esistono già
                        if let None = visitor.f_nodes.get(&feature) {
                            assert_eq!(None, visitor.f_nodes.get(&not_feature));

                            // feature
                            let feat_node = Rc::new(RefCell::new(feature.clone()));
                            let graph_node = visitor.f_graph.add_node(Rc::clone(&feat_node));
                            visitor
                                .f_nodes
                                .insert(feature.clone(), (graph_node, Rc::clone(&feat_node)));

                            // not feature
                            let feat_node = Rc::new(RefCell::new(not_feature.clone()));
                            let graph_node = visitor.f_graph.add_node(Rc::clone(&feat_node));
                            visitor
                                .f_nodes
                                .insert(not_feature.clone(), (graph_node, Rc::clone(&feat_node)));
                        }

                        assert!(visitor.f_nodes.contains_key(&feature));
                        assert!(visitor.f_nodes.contains_key(&not_feature));

                        cfgs.push(FeatureType::Feat(name))
                    }
                    sym::not => cfgs.push(FeatureType::Not(rec_expand(
                        visitor,
                        meta.meta_item_list()
                            .expect("Error: empty `not` feature attribute")
                            .to_vec(),
                    ))),
                    sym::all => cfgs.push(FeatureType::All(rec_expand(
                        visitor,
                        meta.meta_item_list()
                            .expect("Error: empty `all` feature attribute")
                            .to_vec(),
                    ))),
                    sym::any => cfgs.push(FeatureType::Any(rec_expand(
                        visitor,
                        meta.meta_item_list()
                            .expect("Error: empty `any` feature attribute")
                            .to_vec(),
                    ))),
                    _ => (),
                }
            }

            cfgs
        }

        if let Some(meta) = attr.meta() {
            if meta.name_or_empty() == Symbol::intern("feat") {
                if let MetaItemKind::List(ref list) = meta.kind {
                    self.features.pop();
                    let feat = Some(rec_expand(self, list.to_vec()));
                    self.features.push(feat);
                }
            }
        }

        walk_attribute(self, attr);
    }

    /// Visita espressione: (quasi) tutto può essere annotato e quindi va pushato sullo
    /// stack degli statements per evitare di far crescere quello delle feature senza
    /// che ci sia un corrispettivo statement
    fn visit_expr(&mut self, ex: &'ast Expr) {
        self.statements.push(AnnotatedType::Expression(ex.id));
        self.features.push(None);

        walk_expr(self, ex);

        if let (Some(AnnotatedType::Expression(id)), Some(_cfg)) =
            (self.statements.pop(), self.features.pop())
        {
            assert_eq!(
                id, ex.id,
                "Stack not synced. Expected Expression {:?}, found Expression {:?}",
                ex.id, id
            );
        } else {
            panic!(
                "Stack not synced. Expected Expression {:?}, found {:?}",
                ex.id,
                self.statements.last()
            );
        }

        // TODO: tenere tutte le espressioni che possono avere features
        match ex.kind {
            // ExprKind::Array(thin_vec) => todo!(),
            // ExprKind::ConstBlock(anon_const) => todo!(),
            // ExprKind::Call(p, thin_vec) => todo!(),
            // ExprKind::MethodCall(method_call) => todo!(),
            // ExprKind::Tup(thin_vec) => todo!(),
            // ExprKind::Binary(spanned, p, p1) => todo!(),
            // ExprKind::Unary(un_op, p) => todo!(),
            // ExprKind::Lit(lit) => todo!(),
            // ExprKind::Cast(p, p1) => todo!(),
            // ExprKind::Type(p, p1) => todo!(),
            // ExprKind::Let(p, p1, span, recovered) => todo!(),
            // ExprKind::If(p, p1, p2) => todo!(),
            // ExprKind::While(p, p1, label) => todo!(),
            // ExprKind::ForLoop { pat, iter, body, label, kind } => todo!(),
            // ExprKind::Loop(p, label, span) => todo!(),
            // ExprKind::Match(p, thin_vec, match_kind) => todo!(),
            // ExprKind::Closure(closure) => todo!(),
            // ExprKind::Block(p, label) => todo!(),
            // ExprKind::Gen(capture_by, p, gen_block_kind, span) => todo!(),
            // ExprKind::Await(p, span) => todo!(),
            // ExprKind::TryBlock(p) => todo!(),
            // ExprKind::Assign(p, p1, span) => todo!(),
            // ExprKind::AssignOp(spanned, p, p1) => todo!(),
            // ExprKind::Field(p, ident) => todo!(),
            // ExprKind::Index(p, p1, span) => todo!(),
            // ExprKind::Range(p, p1, range_limits) => todo!(),
            // ExprKind::Underscore => todo!(),
            // ExprKind::Path(p, path) => todo!(),
            // ExprKind::AddrOf(borrow_kind, mutability, p) => todo!(),
            // ExprKind::Break(label, p) => todo!(),
            // ExprKind::Continue(label) => todo!(),
            // ExprKind::Ret(p) => todo!(),
            // ExprKind::InlineAsm(p) => todo!(),
            // ExprKind::OffsetOf(p, p1) => todo!(),
            // ExprKind::MacCall(p) => todo!(),
            // ExprKind::Struct(p) => todo!(),
            // ExprKind::Repeat(p, anon_const) => todo!(),
            // ExprKind::Paren(p) => todo!(),
            // ExprKind::Try(p) => todo!(),
            // ExprKind::Yield(p) => todo!(),
            // ExprKind::Yeet(p) => todo!(),
            // ExprKind::Become(p) => todo!(),
            // ExprKind::IncludedBytes(rc) => todo!(),
            // ExprKind::FormatArgs(p) => todo!(),
            // ExprKind::Err(error_guaranteed) => todo!(),
            // ExprKind::Dummy => todo!(),
            _ => (),
        }
    }

    /// Visita item: le dichiarazione di funzioni sono item, dentro `walk_item` vengono
    /// visitati anche gli attributi, quindi è dopo la chiamata le feature sono già
    /// nello stack.
    /// Non viene utilizzato `visit_fn` per analizzare le funzioni dato che gli attributi
    /// non sono visitati da `walk_fn` (ma vengono visitati dopo), di conseguenza non
    /// sarebbe possibile associarli alla rispettiva funzione.
    fn visit_item(&mut self, i: &'ast Item) {
        fn rec_weight_feature(features: Vec<FeatureType>) -> Vec<WeightedFeature> {
            let mut weights = Vec::new();

            for feat in features {
                match feat {
                    FeatureType::Feat(name) => weights.push(WeightedFeature {
                        feature: Feature { name, not: false },
                        weight: 1.0,
                    }),
                    FeatureType::Not(nested) => {
                        weights.extend(rec_weight_feature(nested).into_iter().map(
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
                        let size: f32 = nested.len() as f32;
                        let rec = rec_weight_feature(nested);
                        weights.extend(rec.into_iter().map(
                            |WeightedFeature { feature, weight }| WeightedFeature {
                                feature,
                                weight: weight / size as f32,
                            },
                        ))
                    }
                    FeatureType::Any(nested) => weights.extend(rec_weight_feature(nested)),
                }
            }

            weights
        }

        match i.kind {
            ItemKind::Fn(..) => {
                // creazione nodo del grafo (e cella Rc)
                let mem_node = Rc::new(RefCell::new(Artifact {
                    ident: i.ident.to_string(),
                    _node_id: i.id,
                    features: (Vec::new(), Vec::new()),
                }));
                let graph_node = self.a_graph.add_node(Rc::clone(&mem_node));
                self.a_nodes
                    .insert(i.id, (graph_node, Rc::clone(&mem_node)));

                // aggiornamento stack per trovare cfg
                self.statements.push(AnnotatedType::FunctionDeclaration(
                    i.id,
                    i.ident.to_string(),
                ));
                self.features.push(None);

                // visitare (anche) gli attributi (quindi le cfg)
                walk_item(self, i);

                // estrarre dallo stack dati sulle cfg
                let ident = self
                    .statements
                    .pop()
                    .expect("Error: stack is empty while in item");
                assert_eq!(
                    ident,
                    AnnotatedType::FunctionDeclaration(i.id, i.ident.to_string(),)
                );
                let cfg = self
                    .features
                    .pop()
                    .expect("Error: stack is empty while in item")
                    .unwrap_or_default();

                // aggiornare il nodo con le cfg trovate e pesate
                self.a_nodes.entry(i.id).and_modify(|e| {
                    e.1.borrow_mut().features = (cfg.clone(), rec_weight_feature(cfg.clone()));
                });

                // creare eventuale arco del grafo
                if let Some(AnnotatedType::FunctionDeclaration(id, _ident)) = self.statements.last()
                {
                    self.a_graph.add_edge(
                        self.a_nodes
                            .get(id)
                            .expect("Error: cannot find artifact node creating artifacts graph")
                            .0,
                        self.a_nodes
                            .get(&i.id)
                            .expect("Error: cannot find artifact node creating artifacts graph")
                            .0,
                        Edge { weight: 0.0 },
                    );
                }
            }

            // TODO: tenere tutti gli item che possono avere features
            // ItemKind::ExternCrate(symbol) => walk_item(self, i),
            // ItemKind::Use(use_tree) => walk_item(self, i),
            // ItemKind::Static(static_item) => walk_item(self, i),
            // ItemKind::Const(const_item) => walk_item(self, i),
            // ItemKind::Mod(safety, mod_kind) => walk_item(self, i),
            // ItemKind::ForeignMod(foreign_mod) => walk_item(self, i),
            // ItemKind::GlobalAsm(inline_asm) => walk_item(self, i),
            // ItemKind::TyAlias(ty_alias) => walk_item(self, i),
            // ItemKind::Enum(enum_def, generics) => walk_item(self, i),
            // ItemKind::Struct(variant_data, generics) => walk_item(self, i),
            // ItemKind::Union(variant_data, generics) => walk_item(self, i),
            // ItemKind::Trait(_) => walk_item(self, i),
            // ItemKind::TraitAlias(generics, vec) => walk_item(self, i),
            // ItemKind::Impl(_) => walk_item(self, i),
            // ItemKind::MacCall(p) => walk_item(self, i),
            // ItemKind::MacroDef(macro_def) => walk_item(self, i),
            // ItemKind::Delegation(delegation) => walk_item(self, i),
            // ItemKind::DelegationMac(delegation_mac) => walk_item(self, i),
            _ => walk_item(self, i),
        }
    }
}
