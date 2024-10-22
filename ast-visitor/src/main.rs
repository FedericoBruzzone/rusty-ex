#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

// use rustc_middle::ty::TyCtxt;
use rustc_ast::{ast::*, visit::*};
use rustc_span::symbol::*;
use rustc_span::Span;

use clap::Parser;
use rustc_instrument::{CrateFilter, RustcPlugin, RustcPluginArgs, Utf8Path};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, env};

fn main() {
    env_logger::init();
    rustc_instrument::driver_main(RustcEx);
}

// This struct is the plugin provided to the rustc_plugin framework,
// and it must be exported for use by the CLI/driver binaries.
pub struct RustcEx;

// To parse CLI arguments, we use Clap for this example. But that
// detail is up to you.
#[derive(Parser, Serialize, Deserialize, Default)]
pub struct PrintAstArgs {
    /// Pass --allcaps to print all item names in uppercase.
    #[clap(long)]
    allcaps: bool,

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
        let mut callbacks = PrintAstCallbacks { _args: plugin_args };
        let compiler = rustc_driver::RunCompiler::new(&compiler_args, &mut callbacks);
        compiler.run()
    }
}

struct PrintAstCallbacks {
    _args: PrintAstArgs,
}

impl rustc_driver::Callbacks for PrintAstCallbacks {
    /// Called before creating the compiler instance
    fn config(&mut self, _config: &mut rustc_interface::interface::Config) {}

    /// Called after parsing the crate root. Submodules are not yet parsed when
    /// this callback is called. Return value instructs the compiler whether to
    /// continue the compilation afterwards (defaults to `Compilation::Continue`)
    fn after_crate_root_parsing<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        _queries: &'tcx rustc_interface::Queries<'tcx>,
    ) -> rustc_driver::Compilation {
        // println!("----- 1 - after_crate_root_parsing");
        rustc_driver::Compilation::Continue
    }

    /// Called after expansion. Return value instructs the compiler whether to
    /// continue the compilation afterwards (defaults to `Compilation::Continue`)
    fn after_expansion<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>,
    ) -> rustc_driver::Compilation {
        // println!("----- 2 - after_expansion");

        queries
            .global_ctxt()
            .unwrap()
            .enter(|tcx: rustc_middle::ty::TyCtxt| {
                let resolver_and_krate = tcx.resolver_for_lowering(()).borrow();
                let krate = &*resolver_and_krate.1;

                let collector = &mut CollectVisitor {
                    idents: Vec::new(),
                    cfgs: Vec::new(),
                };
                collector.visit_crate(krate);

                print_results(&collector);

                println!("\n\n{:#?}\n\n", krate);
            });

        rustc_driver::Compilation::Stop
    }

    /// Called after analysis. Return value instructs the compiler whether to
    /// continue the compilation afterwards (defaults to `Compilation::Continue`)
    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>,
    ) -> rustc_driver::Compilation {
        // Not executed, compilation stopped earlier
        // println!("----- 3 - after_analysis");
        rustc_driver::Compilation::Continue
    }
}

fn print_results(collector: &CollectVisitor) {
    assert_eq!(collector.idents.len(), collector.cfgs.len());

    for (ident, cfg) in collector.idents.iter().zip(collector.cfgs.iter()) {
        match ident {
            AnnotatedType::FunctionDeclaration(ident) => {
                println!("Fn {:?}:\t{:?}", ident, cfg);
            }
            AnnotatedType::Expression(id) => {
                println!("Ex {:?}:\t{:?}", id, cfg);
            }
        }
    }
}

#[derive(Debug)]
enum AnnotatedType {
    FunctionDeclaration(String),
    Expression(NodeId),
}

#[derive(Debug)]
enum FeatureType {
    Feat(String),
    Not(Vec<FeatureType>),
    All(Vec<FeatureType>),
    Any(Vec<FeatureType>),
}

struct CollectVisitor {
    idents: Vec<AnnotatedType>,
    cfgs: Vec<Option<Vec<FeatureType>>>,
}

impl<'ast> Visitor<'ast> for CollectVisitor {
    fn visit_attribute(&mut self, attr: &'ast Attribute) {
        // println!("attr {:?}", attr.ident());
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

        let meta = attr.meta().unwrap();
        if meta.name_or_empty() == sym::cfg {
            if let MetaItemKind::List(ref list) = meta.kind {
                self.cfgs.pop();
                self.cfgs.push(Some(rec_expand(list.to_vec())));
            }
        }

        walk_attribute(self, attr);
    }

    fn visit_expr(&mut self, ex: &'ast Expr) {
        self.idents.push(AnnotatedType::Expression(ex.id));
        self.cfgs.push(None);

        walk_expr(self, ex);
    }
    fn visit_fn(&mut self, fk: FnKind<'ast>, _: Span, _: NodeId) {
        walk_fn(self, fk);

        self.idents.push(AnnotatedType::FunctionDeclaration(
            fk.ident().unwrap().to_string(),
        ));
        self.cfgs.push(None);
    }
}

/*
    fn visit_ident(&mut self, _ident: Ident) {}
    fn visit_foreign_item(&mut self, i: &'ast ForeignItem) {
        walk_foreign_item(self, i)
    }
    fn visit_item(&mut self, i: &'ast Item) {
        walk_item(self, i)
    }
    fn visit_local(&mut self, l: &'ast Local) {
        walk_local(self, l)
    }
    fn visit_block(&mut self, b: &'ast Block) {
        walk_block(self, b)
    }
    fn visit_stmt(&mut self, s: &'ast Stmt) {
        walk_stmt(self, s)
    }
    fn visit_param(&mut self, param: &'ast Param) {
        walk_param(self, param)
    }
    fn visit_arm(&mut self, a: &'ast Arm) {
        walk_arm(self, a)
    }
    fn visit_pat(&mut self, p: &'ast Pat) {
        walk_pat(self, p)
    }
    fn visit_anon_const(&mut self, c: &'ast AnonConst) {
        walk_anon_const(self, c)
    }
    fn visit_expr(&mut self, ex: &'ast Expr) {
        walk_expr(self, ex)
    }
    /// This method is a hack to workaround unstable of `stmt_expr_attributes`.
    /// It can be removed once that feature is stabilized.
    fn visit_method_receiver_expr(&mut self, ex: &'ast Expr) {
        self.visit_expr(ex)
    }
    fn visit_expr_post(&mut self, _ex: &'ast Expr) {}
    fn visit_ty(&mut self, t: &'ast Ty) {
        walk_ty(self, t)
    }
    fn visit_generic_param(&mut self, param: &'ast GenericParam) {
        walk_generic_param(self, param)
    }
    fn visit_generics(&mut self, g: &'ast Generics) {
        walk_generics(self, g)
    }
    fn visit_closure_binder(&mut self, b: &'ast ClosureBinder) {
        walk_closure_binder(self, b)
    }
    fn visit_where_predicate(&mut self, p: &'ast WherePredicate) {
        walk_where_predicate(self, p)
    }
    fn visit_fn(&mut self, fk: FnKind<'ast>, _: Span, _: NodeId) {
        walk_fn(self, fk)
    }
    fn visit_assoc_item(&mut self, i: &'ast AssocItem, ctxt: AssocCtxt) {
        walk_assoc_item(self, i, ctxt)
    }
    fn visit_trait_ref(&mut self, t: &'ast TraitRef) {
        walk_trait_ref(self, t)
    }
    fn visit_param_bound(&mut self, bounds: &'ast GenericBound, _ctxt: BoundKind) {
        walk_param_bound(self, bounds)
    }
    fn visit_poly_trait_ref(&mut self, t: &'ast PolyTraitRef) {
        walk_poly_trait_ref(self, t)
    }
    fn visit_variant_data(&mut self, s: &'ast VariantData) {
        walk_struct_def(self, s)
    }
    fn visit_field_def(&mut self, s: &'ast FieldDef) {
        walk_field_def(self, s)
    }
    fn visit_enum_def(&mut self, enum_definition: &'ast EnumDef) {
        walk_enum_def(self, enum_definition)
    }
    fn visit_variant(&mut self, v: &'ast Variant) {
        walk_variant(self, v)
    }
    fn visit_variant_discr(&mut self, discr: &'ast AnonConst) {
        self.visit_anon_const(discr);
    }
    fn visit_label(&mut self, label: &'ast Label) {
        walk_label(self, label)
    }
    fn visit_lifetime(&mut self, lifetime: &'ast Lifetime, _: LifetimeCtxt) {
        walk_lifetime(self, lifetime)
    }
    fn visit_mac_call(&mut self, mac: &'ast MacCall) {
        walk_mac(self, mac)
    }
    fn visit_mac_def(&mut self, _mac: &'ast MacroDef, _id: NodeId) {
        // Nothing to do
    }
    fn visit_path(&mut self, path: &'ast Path, _id: NodeId) {
        walk_path(self, path)
    }
    fn visit_use_tree(&mut self, use_tree: &'ast UseTree, id: NodeId, _nested: bool) {
        walk_use_tree(self, use_tree, id)
    }
    fn visit_path_segment(&mut self, path_segment: &'ast PathSegment) {
        walk_path_segment(self, path_segment)
    }
    fn visit_generic_args(&mut self, generic_args: &'ast GenericArgs) {
        walk_generic_args(self, generic_args)
    }
    fn visit_generic_arg(&mut self, generic_arg: &'ast GenericArg) {
        walk_generic_arg(self, generic_arg)
    }
    fn visit_assoc_constraint(&mut self, constraint: &'ast AssocConstraint) {
        walk_assoc_constraint(self, constraint)
    }
    fn visit_attribute(&mut self, attr: &'ast Attribute) {
        // Macro attributes are here
        walk_attribute(self, attr)
    }
    fn visit_vis(&mut self, vis: &'ast Visibility) {
        walk_vis(self, vis)
    }
    fn visit_fn_ret_ty(&mut self, ret_ty: &'ast FnRetTy) {
        walk_fn_ret_ty(self, ret_ty)
    }
    fn visit_fn_header(&mut self, _header: &'ast FnHeader) {
        // Nothing to do
    }
    fn visit_expr_field(&mut self, f: &'ast ExprField) {
        walk_expr_field(self, f)
    }
    fn visit_pat_field(&mut self, fp: &'ast PatField) {
        walk_pat_field(self, fp)
    }
    fn visit_crate(&mut self, krate: &'ast Crate) {
        walk_crate(self, krate)
    }
    fn visit_inline_asm(&mut self, asm: &'ast InlineAsm) {
        walk_inline_asm(self, asm)
    }
    fn visit_format_args(&mut self, fmt: &'ast FormatArgs) {
        walk_format_args(self, fmt)
    }
    fn visit_inline_asm_sym(&mut self, sym: &'ast InlineAsmSym) {
        walk_inline_asm_sym(self, sym)
    }
    fn visit_capture_by(&mut self, _capture_by: &'ast CaptureBy) {
        // Nothing to do
    }
*/
