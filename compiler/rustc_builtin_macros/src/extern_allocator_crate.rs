// Code that injects an extern statement pointing to the `global_allocator` or
// `alloc_error_handler` sysroot crates. Should
// only be used when the `incomplete_dylib` compiler feature is enabled, an allocator is needed,
// and a global allocator or alloc error handler is not already found in the crate graph.

// mattmatt could/should this be fully generic and used for other pre-existing extern imports as
// well or what

use rustc_ast as ast;
use rustc_expand::base::{ExtCtxt, ResolverExpand};
use rustc_expand::expand::{AstFragment, ExpansionConfig};
use rustc_session::Session;
use rustc_span::edition::Edition::Edition2018;
use rustc_span::hygiene::AstPass;
use rustc_span::symbol::{Ident, Symbol};
use rustc_span::DUMMY_SP;
use smallvec::smallvec;
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
}
