/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Handles the auto-deriving for `#[derive(HeapSizeOf)]`
//!
//! This provides the `#[derive(HeapSizeOf)]` decorator, which
//! generates a `HeapSizeOf` implementation that adds up
//! calls to heap_size_of_children() for all the fields
//! of a struct or enum variant.
//!
//! Fields marked `#[ignore_heap_size_of = "reason"]` will
//! be ignored in this calculation. Providing a reason is compulsory.


use syntax::ast::*;
use syntax::attr::AttrMetaMethods;
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::ext::deriving::generic::*;
use syntax::ptr::P;

pub fn expand_heap_size(cx: &mut ExtCtxt, span: Span, mitem: &MetaItem,
                        item: &Annotatable, push: &mut FnMut(Annotatable)) {
    let trait_def = TraitDef {
        span: span,
        attributes: Vec::new(),
        path: ty::Path::new(vec!("util", "mem", "HeapSizeOf")),
        additional_bounds: Vec::new(),
        generics: ty::LifetimeBounds::empty(),
        methods: vec![
            MethodDef {
                name: "heap_size_of_children",
                generics: ty::LifetimeBounds::empty(),
                explicit_self: ty::borrowed_explicit_self(),
                args: vec!(),
                ret_ty: ty::Literal(ty::Path::new_local("usize")),
                attributes: vec!(),
                is_unsafe: false,
                combine_substructure: combine_substructure(Box::new(heap_size_substructure))
            }
        ],
        associated_types: vec![],
    };
    trait_def.expand(cx, mitem, item, push)
}

/// Defines how the implementation for `heap_size_of_children()` is to be generated.
fn heap_size_substructure(cx: &mut ExtCtxt, trait_span: Span, substr: &Substructure) -> P<Expr> {
    let fields = match *substr.fields {
        Struct(ref fs) | EnumMatching(_, _, ref fs) => fs,
        _ => cx.span_bug(trait_span, "impossible substructure in `#[derive(HeapSizeOf)]`")
    };

    fields.iter().fold(cx.expr_usize(trait_span, 0), |acc, ref item| {
        if item.attrs.iter()
               .find(|ref a| {
                    if a.check_name("ignore_heap_size_of") {
                        match a.node.value.node {
                            MetaNameValue(..) => (),
                            _ => cx.span_err(a.span, "#[ignore_heap_size_of] \
                                                      should have an explanation, \
                                                      e.g. #[ignore_heap_size_of = \"\"]")
                        }
                        true
                    } else {
                        false
                    }
                })
               .is_some() {
            acc
        } else {
            cx.expr_binary(item.span, BiAdd, acc,
                           cx.expr_method_call(item.span,
                                               item.self_.clone(),
                                               substr.method_ident,
                                               Vec::new()))
                        }
    })
}
