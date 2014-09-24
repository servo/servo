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

 pub fn expand_jstraceable(cx: &mut ExtCtxt, span: Span, mitem: &MetaItem, item: &Item, push: |P<Item>|) {
    let trait_def = TraitDef {
        span: span,
        attributes: Vec::new(),
        path: ty::Path::new(vec!("dom","bindings","trace","JSTraceable")),
        additional_bounds: Vec::new(),
        generics: ty::LifetimeBounds::empty(),
        methods: vec!(
            MethodDef {
                name: "trace",
                generics: ty::LifetimeBounds::empty(),
                explicit_self: ty::borrowed_explicit_self(),
                args: vec!(ty::Ptr(box ty::Literal(ty::Path::new(vec!("js","jsapi","JSTracer"))), ty::Raw(ast::MutMutable))),
                ret_ty: ty::nil_ty(),
                attributes: vec!(),
                combine_substructure: combine_substructure(|a, b, c| {
                    jstraceable_substructure(a, b, c)
                })
            }
        )
    };
    trait_def.expand(cx, mitem, item, push)
}

// Mostly copied from syntax::ext::deriving::hash
fn jstraceable_substructure(cx: &mut ExtCtxt, trait_span: Span, substr: &Substructure) -> P<Expr> {
    let state_expr = match substr.nonself_args {
        [ref state_expr] => state_expr,
        _ => cx.span_bug(trait_span, "incorrect number of arguments in `jstraceable`")
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
