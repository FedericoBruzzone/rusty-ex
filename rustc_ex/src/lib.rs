#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use clap::Parser;
use rustc_ast::{ast::*, visit::*};
use rustc_instrument::{CrateFilter, RustcPlugin, RustcPluginArgs, Utf8Path};
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
#[derive(Parser, Serialize, Deserialize, Default)]
pub struct PrintAstArgs {
    /// Pass --print-dot to print the DOT graph
    #[clap(long)]
    print_dot: bool,

    /// Pass --print-crate to print the crate
    #[clap(long)]
    print_crate: bool,

    /// Pass --print-graph to print the graph
    #[clap(long)]
    print_graph: bool,

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

    // In the CLI, we ask Clap to parse arguments and also specify a CrateFilter.
    // If one of the CLI arguments was a specific file to analyze, then you
    // could provide a different filter.
    fn args(&self, _target_dir: &Utf8Path) -> RustcPluginArgs<Self::Args> {
        let args = PrintAstArgs::parse_from(env::args().skip(1));
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
    // TODO: Consider to remove `&mut` from CollectVisitor
    fn process_cli_args(&self, collector: &mut CollectVisitor, krate: &Crate) {
        if self.args.print_crate {
            println!("{:#?}", krate);
        }
        if self.args.print_graph {
            println!("{:?}", collector.graph);
        }
        if self.args.print_dot {
            collector.print_graph_dot();
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
            .unwrap()
            .enter(|tcx: rustc_middle::ty::TyCtxt| {
                // estrarre l'AST
                let resolver_and_krate = tcx.resolver_for_lowering(()).borrow();
                let krate = &*resolver_and_krate.1;

                // visitare l'AST
                let collector = &mut CollectVisitor {
                    log: false,
                    statements: Vec::new(),
                    features: Vec::new(),
                    nodes: HashMap::new(),
                    graph: graph::DiGraph::new(),
                };
                collector.visit_crate(krate);

                self.process_cli_args(collector, krate);
            });

        rustc_driver::Compilation::Stop
    }
}

/// Definizioni per l'estrazione delle feature dall'AST, lo statement annotato e la/le feature
#[derive(Clone, Debug)]
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

/// Definizioni per il grafo
#[derive(Clone, Debug)]
struct Node {
    ident: String,
    _node_id: NodeId,
    feature: Vec<FeatureType>,
}
#[derive(Clone, Debug)]
struct Edge {
    weight: f32,
}

/// Visitor per la visita :) dell'AST
struct CollectVisitor {
    log: bool,
    // stack parallelo: statements con rispettive feature
    statements: Vec<AnnotatedType>,
    features: Vec<Option<Vec<FeatureType>>>,
    // grafo delle dipendenze
    nodes: HashMap<NodeId, (NodeIndex, Rc<RefCell<Node>>)>,
    graph: graph::DiGraph<Rc<RefCell<Node>>, Edge>,
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

    /// Stampa il grafo in formato DOT (per Graphviz)
    fn print_graph_dot(&self) {
        let get_edge_attr = |_g: &graph::DiGraph<Rc<RefCell<Node>>, Edge>,
                             edge: graph::EdgeReference<Edge>| {
            format!("label=\"{}\"", edge.weight().weight)
        };

        let get_node_attr =
            |_g: &graph::DiGraph<Rc<RefCell<Node>>, Edge>,
             node: (graph::NodeIndex, &Rc<RefCell<Node>>)| {
                format!(
                    "label=\"{} ({})\"",
                    node.1.borrow().ident,
                    CollectVisitor::features_to_string(&node.1.borrow().feature)
                )
            };

        println!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
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
        fn rec_expand(nested_meta: Vec<NestedMetaItem>) -> Vec<FeatureType> {
            let mut cfgs = Vec::new();

            for meta in nested_meta {
                match meta.name_or_empty() {
                    sym::feature => {
                        cfgs.push(FeatureType::Feat(meta.value_str().unwrap().to_string()))
                    }
                    sym::not => cfgs.push(FeatureType::Not(rec_expand(
                        meta.meta_item_list().unwrap().to_vec(),
                    ))),
                    sym::all => cfgs.push(FeatureType::All(rec_expand(
                        meta.meta_item_list().unwrap().to_vec(),
                    ))),
                    sym::any => cfgs.push(FeatureType::Any(rec_expand(
                        meta.meta_item_list().unwrap().to_vec(),
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
                    self.features.push(Some(rec_expand(list.to_vec())));
                }
            }
        }

        walk_attribute(self, attr);
    }

    /// Visita espressione: (quasi) tutto può essere annotato e quindi va pushato sullo
    /// stack degli statements per evitare di far crescere quello delle feature senza
    /// che ci sia un corrispettivo statement
    fn visit_expr(&mut self, ex: &'ast Expr) {
        // TODO: controllare tutti i tipi di expr che possono avere attributi cfg
        // quelli che non possono, devono venir trattati qua altrimenti potrebbero rubare
        // cfg alle funzioni
        // TODO: provare a fare questa roba sempre, a prescindere dal tipo di expr
        match ex.kind {
            ExprKind::Call(_, _) | ExprKind::MethodCall(_) => {
                self.statements.push(AnnotatedType::Expression(ex.id));
                self.features.push(None);

                walk_expr(self, ex);

                if let Some(AnnotatedType::Expression(id)) = self.statements.pop() {
                    assert_eq!(
                        id, ex.id,
                        "Stack not synced. Expected Expression {:?}, found Expression {:?}",
                        ex.id, id
                    );
                    let cfg = self.features.pop().unwrap();

                    if self.log {
                        println!("Expression {:?}\n{:?}\n", id, cfg);
                    }
                } else {
                    panic!(
                        "Stack not synced. Expected Expression {:?}, found {:?}",
                        ex.id,
                        self.statements.last()
                    );
                }
            }
            _ => {
                walk_expr(self, ex);
            }
        }
    }

    /// Visita item: le dichiarazione di funzioni sono item, dentro `walk_item` vengono
    /// visitati anche gli attributi, quindi è dopo la chiamata le feature sono già
    /// nello stack.
    /// Non viene utilizzato `visit_fn` per analizzare le funzioni dato che gli attributi
    /// non sono visitati da `walk_fn` (ma vengono visitati dopo), di conseguenza non
    /// sarebbe possibile associarli alla rispettiva funzione.
    fn visit_item(&mut self, i: &'ast Item) {
        // TODO: controllare tutti i tipi di item, altri potrebbero essere interessanti
        // TODO: controllare quali altri tipi di item possono avere attributi
        match i.kind {
            ItemKind::Fn(..) => {
                // creazione nodo del grafo (e cella Rc)
                let mem_node = Rc::new(RefCell::new(Node {
                    ident: i.ident.to_string(),
                    _node_id: i.id,
                    feature: Vec::new(),
                }));
                let graph_node = self.graph.add_node(Rc::clone(&mem_node));
                self.nodes.insert(i.id, (graph_node, Rc::clone(&mem_node)));

                // aggiornamento stack per trovare cfg
                self.statements.push(AnnotatedType::FunctionDeclaration(
                    i.id,
                    i.ident.to_string(),
                ));
                self.features.push(None);

                // visitare (anche) gli attributi (quindi le cfg)
                walk_item(self, i);

                // estrarre dallo stack dati sulle cfg
                let ident = self.statements.pop().unwrap();
                let cfg = self.features.pop().unwrap();

                if self.log {
                    println!(
                        "Item {:?}\n{:?}\n{:?}\nPARENT: {:?}\n",
                        i.id, ident, cfg, self.statements
                    );
                }

                // aggiornare il nodo con le cfg trovate
                self.nodes.entry(i.id).and_modify(|e| {
                    e.1.borrow_mut().feature = cfg.unwrap_or_default();
                });

                // creare eventuale arco del grafo
                if let Some(AnnotatedType::FunctionDeclaration(id, _ident)) = self.statements.last()
                {
                    self.graph.add_edge(
                        self.nodes.get(id).unwrap().0,
                        self.nodes.get(&i.id).unwrap().0,
                        Edge { weight: 1.0 },
                    );
                }
            }
            _ => {
                walk_item(self, i);
            }
        }
    }
}
