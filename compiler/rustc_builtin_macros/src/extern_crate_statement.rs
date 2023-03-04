// Injects `extern crate foo;` into the AST to satisfy implicit dependencies.

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
    let ecfg = ExpansionConfig::default("extern_crate_injection".to_string());
    let mut cx = ExtCtxt::new(sess, ecfg, resolver, None);

    let expn_id = cx.resolver.expansion_for_ast_pass(DUMMY_SP, AstPass::ExternCrateInjection, &[], None);

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

    krate.items.insert(0, expanded_fragment);
}
