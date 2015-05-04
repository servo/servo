/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syntax::ext::base::ExtCtxt;
use syntax::codemap::Span;
use syntax::ptr::P;
use syntax::ast::{Item, MetaItem, Expr};
use syntax::ast;
use syntax::ext::build::AstBuilder;
use syntax::ext::deriving::generic::{combine_substructure, EnumMatching, FieldInfo, MethodDef, Struct, Substructure, TraitDef, ty};

pub fn expand_dom_struct(cx: &mut ExtCtxt, _: Span, _: &MetaItem, item: P<Item>) -> P<Item> {
    let mut item2 = (*item).clone();
    item2.attrs.push(quote_attr!(cx, #[must_root]));
    item2.attrs.push(quote_attr!(cx, #[privatize]));
    item2.attrs.push(quote_attr!(cx, #[jstraceable]));

    // The following attributes are only for internal usage
    item2.attrs.push(quote_attr!(cx, #[_generate_reflector]));
    // #[dom_struct] gets consumed, so this lets us keep around a residue
    // Do NOT register a modifier/decorator on this attribute
    item2.attrs.push(quote_attr!(cx, #[_dom_struct_marker]));
    P(item2)
}

/// Provides the hook to expand `#[jstraceable]` into an implementation of `JSTraceable`
///
/// The expansion basically calls `trace()` on all of the fields of the struct/enum, erroring if they do not implement the method.
pub fn expand_jstraceable(cx: &mut ExtCtxt, span: Span, mitem: &MetaItem, item: &Item, push: &mut FnMut(P<Item>)) {
    let trait_def = TraitDef {
        span: span,
        attributes: Vec::new(),
        path: ty::Path::new(vec!("dom","bindings","trace","JSTraceable")),
        additional_bounds: Vec::new(),
        generics: ty::LifetimeBounds::empty(),
        methods: vec![
            MethodDef {
                name: "trace",
                generics: ty::LifetimeBounds::empty(),
                explicit_self: ty::borrowed_explicit_self(),
                args: vec!(ty::Ptr(box ty::Literal(ty::Path::new(vec!("js","jsapi","JSTracer"))), ty::Raw(ast::MutMutable))),
                ret_ty: ty::nil_ty(),
                attributes: vec![quote_attr!(cx, #[inline(always)])],
                combine_substructure: combine_substructure(box jstraceable_substructure)
            }
        ],
        associated_types: vec![],
    };
    trait_def.expand(cx, mitem, item, push)
}

// Mostly copied from syntax::ext::deriving::hash
/// Defines how the implementation for `trace()` is to be generated
fn jstraceable_substructure(cx: &mut ExtCtxt, trait_span: Span, substr: &Substructure) -> P<Expr> {
    let state_expr = if substr.nonself_args.len() == 1 {
        &substr.nonself_args[0]
    } else {
        cx.span_bug(trait_span, "incorrect number of arguments in `jstraceable`")
    };
    let trace_ident = substr.method_ident;
    let call_trace = |span, thing_expr| {
        let expr = cx.expr_method_call(span, thing_expr, trace_ident, vec!(state_expr.clone()));
        cx.stmt_expr(expr)
    };
    let mut stmts = Vec::new();

    let fields = match *substr.fields {
        Struct(ref fs) | EnumMatching(_, _, ref fs) => fs,
        _ => cx.span_bug(trait_span, "impossible substructure in `jstraceable`")
    };

    for &FieldInfo { ref self_, span, .. } in fields.iter() {
        stmts.push(call_trace(span, self_.clone()));
    }

    cx.expr_block(cx.block(trait_span, stmts, None))
}
