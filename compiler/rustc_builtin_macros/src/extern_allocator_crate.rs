// Code that injects an extern statement pointing to the `global_allocator` or
// `alloc_error_handler` sysroot crates. Should
// only be used when the `incomplete_dylib` compiler feature is enabled, an allocator is needed,
// and a global allocator or alloc error handler is not already found in the crate graph.

// mattmatt could/should this be fully generic and used for other pre-existing extern imports as
// well or what

#![allow(unused)]

use rustc_ast as ast;
use rustc_ast::ast::StrStyle;
use rustc_ast::expand::allocator::{
    AllocatorKind, AllocatorMethod, AllocatorTy, ALLOCATOR_METHODS,
};
use rustc_ast::ptr::P;
use rustc_ast::{
    Fn, FnHeader, FnSig, ForeignItem, ForeignItemKind, ForeignMod, Generics, Mutability,
    Param, StrLit, Ty, TyKind, Unsafe,
};
use rustc_expand::base::{ExtCtxt, ResolverExpand};
use rustc_expand::expand::{AstFragment, ExpansionConfig};
use rustc_session::Session;
use rustc_span::edition::Edition::Edition2018;
use rustc_span::hygiene::AstPass;
use rustc_span::symbol::{kw, sym, Ident, Symbol};
use rustc_span::{Span, DUMMY_SP};
use smallvec::smallvec;
// use std::vec;
use thin_vec::thin_vec;

pub fn inject(
    sess: &Session,
    resolver: &mut dyn ResolverExpand,
    krate: &mut ast::Crate,
    sym_name: Symbol,
) {
    let ecfg = ExpansionConfig::default("allocator_crate_injection".to_string());
    let mut cx = ExtCtxt::new(sess, ecfg, resolver, None);

    let expn_id = cx.resolver.expansion_for_ast_pass(DUMMY_SP, AstPass::AllocatorCrates, &[], None);

    // mattmatt i am guessing i want to inject extern definitions for the allocator functions. we
    // codegen the allocator in, say, the bin, but the definitions are found in the
    // global_allocator.so (or .rlib)

    let span = DUMMY_SP.with_def_site_ctxt(expn_id.to_expn_id());
    let call_site = DUMMY_SP.with_call_site_ctxt(expn_id.to_expn_id());
    let ident = if sess.parse_sess.edition >= Edition2018 {
        Ident::new(sym_name, span)
    } else {
        Ident::new(sym_name, call_site)
    };

    let extern_stmt = cx.item(span, ident, thin_vec![], ast::ItemKind::ExternCrate(None));
    let fragment = AstFragment::Items(smallvec![extern_stmt]);
    let expanded_fragment =
        cx.monotonic_expander().fully_expand_fragment(fragment).make_items().pop().unwrap();

    println!("mattmatt injecting {}", sym_name);
    krate.items.insert(0, expanded_fragment); // mattmatt 0 may be wrong

    /*
    if sym_name == sym::global_allocator {
        println!("mattmatt trying to make extern module lol");
        // inject extern decls for the allocator functions so that when we dynamically link the
        // compiler will expect them to be found elsewhere
        // extern "Rust" {
        //   fn __rg_alloc();
        //   fn __rg_realloc();
        //   fn __rg_dealloc();
        //   etc
        // }

        let factory = AllocFnFactory { span, kind: AllocatorKind::Global, cx: &cx };
        println!("mattmatt factory created");

        let items = ALLOCATOR_METHODS.iter().map(|method| factory.allocator_fn(method)).collect();
        println!("mattmatt allocator ForeignItems created");

        let abi = Symbol::intern("Rust");
        println!("mattmatt extern Rust symbol created");
        let fmod = ast::ItemKind::ForeignMod(ForeignMod {
            unsafety: Unsafe::Yes(span),
            abi: Some(StrLit {
                symbol: abi,
                suffix: None,
                symbol_unescaped: abi,
                style: StrStyle::Cooked,
                span: span,
            }),
            items: items,
        });
        println!("mattmatt foreign module created");

        let fmod_item = cx.item(span, Ident::new(kw::Underscore, span), thin_vec![], fmod);
        println!("mattmatt injecting allocator externs");

        let fmod_fragment = AstFragment::Items(smallvec![fmod_item]);
        let expanded_fmod_fragment =
            cx.monotonic_expander().fully_expand_fragment(fmod_fragment).make_items().pop().unwrap();
        krate.items.push(expanded_fmod_fragment);
    }
    */
}

struct AllocFnFactory<'a, 'b> {
    span: Span,
    kind: AllocatorKind,
    cx: &'b ExtCtxt<'a>,
}

impl AllocFnFactory<'_, '_> {
    fn allocator_fn(&self, method: &AllocatorMethod) -> P<ForeignItem> {
        println!("mattmatt generating ForeignItem for {}", self.kind.fn_name(method.name));
        let mut abi_args = Vec::new();
        let mut i = 0;
        let mut mk = || {
            let name = Ident::from_str_and_span(&format!("arg{}", i), self.span);
            i += 1;
            name
        };
        println!("mattmatt about to populate abi_args");
        let _args: Vec<()> =
            method.inputs.iter().map(|ty| self.arg_ty(ty, &mut abi_args, &mut mk)).collect();

        println!("mattmatt populated abi_args");
        let output_ty = self.ret_ty(&method.output);
        println!("mattmatt got return type");
        let decl = self.cx.fn_decl(abi_args, ast::FnRetTy::Ty(output_ty));
        println!("mattmatt got fn declaration");
        let header = FnHeader { ..FnHeader::default() };
        let sig = FnSig { decl, header, span: self.span };
        println!("mattmatt got fn signature");
        let body = None;

        let kind = ForeignItemKind::Fn(Box::new(Fn {
            defaultness: ast::Defaultness::Final,
            sig,
            generics: Generics::default(),
            body,
        }));
        println!("mattmatt made ForeignItemKind");
        let foreign_item = P(ast::Item {
            ident: Ident::from_str_and_span(&self.kind.fn_name(method.name), self.span),
            attrs: thin_vec![],
            id: ast::DUMMY_NODE_ID,
            kind: kind,
            vis: ast::Visibility {
                span: self.span.shrink_to_lo(),
                kind: ast::VisibilityKind::Inherited, // mattmatt maybe make this public
                tokens: None,
            },
            span: self.span,
            tokens: None,
        });
        println!("mattmatt made ForeignItem");

        foreign_item
    }

    fn arg_ty(
        &self,
        ty: &AllocatorTy,
        args: &mut Vec<Param>,
        ident: &mut dyn FnMut() -> Ident,
    ) {
//    ) -> P<Expr> {
        match *ty {
            AllocatorTy::Layout => {
                println!("mattmatt layout return type");
                let usize = self.cx.path_ident(self.span, Ident::new(sym::usize, self.span));
                let ty_usize = self.cx.ty_path(usize);
                let size = ident();
                let align = ident();
                println!("mattmatt about to push args");
                args.push(self.cx.param(self.span, size, ty_usize.clone()));
                args.push(self.cx.param(self.span, align, ty_usize));
                println!("mattmatt layout pushed to args");

                /*
                let layout_new =
                    self.cx.std_path(&[sym::alloc, sym::Layout, sym::from_size_align_unchecked]);
                println!("mattmatt std_path done");
                let layout_new = self.cx.expr_path(self.cx.path(self.span, layout_new));
                println!("mattmatt expr_path");
                let size = self.cx.expr_ident(self.span, size);
                println!("mattmatt first expr_ident");
                let align = self.cx.expr_ident(self.span, align);
                println!("mattmatt second expr_ident");
                let layout = self.cx.expr_call(self.span, layout_new, vec![size, align]);
                println!("matt returning after expr_call");
                layout
                */
            }

            AllocatorTy::Ptr => {
                println!("mattmatt ptr rreturn type");
                let ident = ident();
                args.push(self.cx.param(self.span, ident, self.ptr_u8()));
                println!("mattmatt pushed to args");
                /*
                let arg = self.cx.expr_ident(self.span, ident);
                println!("mattmatt expr_ident, about to return expr_cast");
                self.cx.expr_cast(self.span, arg, self.ptr_u8())
                */
            }

            AllocatorTy::Usize => {
                println!("mattmatt usize reutrn type");
                let ident = ident();
                args.push(self.cx.param(self.span, ident, self.usize()));
                println!("mattmatt pushed to args, about to reutnr expr_ident");
                /*
                self.cx.expr_ident(self.span, ident)
                */
            }

            AllocatorTy::ResultPtr | AllocatorTy::Unit => {
                println!("mattmatt about to panic");
                panic!("can't convert AllocatorTy to an argument")
            }
        }
    }

    fn ret_ty(&self, ty: &AllocatorTy) -> P<Ty> {
        match *ty {
            AllocatorTy::ResultPtr => self.ptr_u8(),

            AllocatorTy::Unit => self.cx.ty(self.span, TyKind::Tup(Vec::new())),

            AllocatorTy::Layout | AllocatorTy::Usize | AllocatorTy::Ptr => {
                panic!("can't convert `AllocatorTy` to an output")
            }
        }
    }

    fn usize(&self) -> P<Ty> {
        let usize = self.cx.path_ident(self.span, Ident::new(sym::usize, self.span));
        self.cx.ty_path(usize)
    }

    fn ptr_u8(&self) -> P<Ty> {
        let u8 = self.cx.path_ident(self.span, Ident::new(sym::u8, self.span));
        let ty_u8 = self.cx.ty_path(u8);
        self.cx.ty_ptr(self.span, ty_u8, Mutability::Mut)
    }
}
