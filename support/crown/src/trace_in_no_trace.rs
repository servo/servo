/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rustc_ast::ast::{AttrKind, Attribute};
use rustc_ast::token::TokenKind;
use rustc_ast::tokenstream::TokenTree;
use rustc_ast::AttrArgs;
use rustc_error_messages::MultiSpan;
use rustc_hir::{self as hir};
use rustc_lint::{LateContext, LateLintPass, LintContext, LintPass, LintStore};
use rustc_middle::ty;
use rustc_session::declare_tool_lint;
use rustc_span::symbol::Symbol;

use crate::common::{get_local_trait_def_id, get_trait_def_id, implements_trait};
use crate::symbols;

declare_tool_lint! {
    pub crown::TRACE_IN_NO_TRACE,
    Deny,
    "Warn and report incorrect usage of Traceable (jsmanaged) objects in must_not_have_traceable marked wrappers"
}

declare_tool_lint! {
    pub crown::EMPTY_TRACE_IN_NO_TRACE,
    Warn,
    "Warn about usage of empty Traceable objects in must_not_have_traceable marked wrappers"
}

const EMPTY_TRACE_IN_NO_TRACE_MSG: &str =
    "must_not_have_traceable marked wrapper is not needed for types that implements \
empty Traceable (like primitive types). Consider removing the wrapper.";

pub fn register(lint_store: &mut LintStore) {
    let symbols = Symbols::new();
    lint_store.register_lints(&[&TRACE_IN_NO_TRACE, &EMPTY_TRACE_IN_NO_TRACE]);
    lint_store.register_late_pass(move |_| Box::new(NotracePass::new(symbols.clone())));
}

/// Lint for ensuring safe usage of NoTrace wrappers
///
/// This lint (disable with `-A trace-in-no-trace`/`#[allow(trace_in_no_trace)]`) ensures that
/// wrappers marked with must_not_have_traceable(i: usize) only stores
/// non-jsmanaged (DOES NOT implement JSTraceble) type in i-th generic
///
/// For example usage look at the tests
pub(crate) struct NotracePass {
    symbols: Symbols,
}

impl NotracePass {
    pub(crate) fn new(symbols: Symbols) -> Self {
        Self { symbols }
    }
}

impl LintPass for NotracePass {
    fn name(&self) -> &'static str {
        "ServoNotracePass"
    }
}

fn get_must_not_have_traceable(sym: &Symbols, attrs: &[Attribute]) -> Option<usize> {
    attrs
        .iter()
        .find(|attr| {
            matches!(
                &attr.kind,
                AttrKind::Normal(normal)
                if normal.item.path.segments.len() == 3 &&
                normal.item.path.segments[0].ident.name == sym.crown &&
                normal.item.path.segments[1].ident.name == sym.trace_in_no_trace_lint &&
                normal.item.path.segments[2].ident.name == sym.must_not_have_traceable
            )
        })
        .map(|x| match &x.get_normal_item().args {
            AttrArgs::Empty => 0,
            AttrArgs::Delimited(a) => match a
                .tokens
                .trees()
                .next()
                .expect("Arguments not found for must_not_have_traceable")
            {
                TokenTree::Token(tok, _) => match tok.kind {
                    TokenKind::Literal(lit) => lit.symbol.as_str().parse().unwrap(),
                    _ => panic!("must_not_have_traceable expected integer literal here"),
                },
                TokenTree::Delimited(..) => {
                    todo!("must_not_have_traceable does not support multiple notraceable positions")
                },
            },
            _ => {
                panic!("must_not_have_traceable does not support key-value arguments")
            },
        })
}

fn is_jstraceable<'tcx>(cx: &LateContext<'tcx>, ty: ty::Ty<'tcx>) -> bool {
    // TODO(sagudev): get_trait_def_id is expensive, use lazy and cache it for whole pass
    if let Some(trait_id) = get_trait_def_id(cx, &["mozjs", "gc", "Traceable"]) {
        return implements_trait(cx, ty, trait_id, &[]);
    }
    // when running tests
    if let Some(trait_id) = get_local_trait_def_id(cx, "JSTraceable") {
        return implements_trait(cx, ty, trait_id, &[]);
    }
    panic!("JSTraceable not found");
}

/// Gives warrning or errors for incorect usage of NoTrace like `NoTrace<impl Traceable>`.
fn incorrect_no_trace<'tcx, I: Into<MultiSpan> + Copy>(
    sym: &'_ Symbols,
    cx: &LateContext<'tcx>,
    ty: ty::Ty<'tcx>,
    span: I,
) {
    let mut walker = ty.walk();
    while let Some(generic_arg) = walker.next() {
        let t = match generic_arg.unpack() {
            rustc_middle::ty::GenericArgKind::Type(t) => t,
            _ => {
                walker.skip_current_subtree();
                continue;
            },
        };
        let recur_into_subtree = match t.kind() {
            ty::Adt(did, substs) => {
                if let Some(pos) =
                    get_must_not_have_traceable(sym, cx.tcx.get_attrs_unchecked(did.did()))
                {
                    let inner = substs.type_at(pos);
                    if inner.is_primitive_ty() {
                        cx.lint(
                            EMPTY_TRACE_IN_NO_TRACE,
                            EMPTY_TRACE_IN_NO_TRACE_MSG,
                            |lint| {
                                lint.span(span);
                            },
                        )
                    } else if is_jstraceable(cx, inner) {
                        cx.lint(
                            TRACE_IN_NO_TRACE,
                            format!(
                                "must_not_have_traceable marked wrapper must not have \
jsmanaged inside on {pos}-th position. Consider removing the wrapper."
                            ),
                            |lint| {
                                lint.span(span);
                            },
                        )
                    }
                    false
                } else {
                    true
                }
            },
            _ => !t.is_primitive_ty(),
        };
        if !recur_into_subtree {
            walker.skip_current_subtree();
        }
    }
}

// NoTrace correct usage of NoTrace must only be checked on Struct (item) and Enums (variants)
// as these are the only ones that are actually traced
impl<'tcx> LateLintPass<'tcx> for NotracePass {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item) {
        // TODO: better performance if we limit with lint attr???
        /*let attrs = cx.tcx.hir().attrs(item.hir_id());
        if has_lint_attr(&self.symbols, &attrs, self.symbols.must_root) {
            return;
        }*/
        if let hir::ItemKind::Struct(def, ..) = &item.kind {
            for field in def.fields() {
                let field_type = cx.tcx.type_of(field.def_id);
                incorrect_no_trace(&self.symbols, cx, field_type.skip_binder(), field.span);
            }
        }
    }

    fn check_variant(&mut self, cx: &LateContext, var: &hir::Variant) {
        match var.data {
            hir::VariantData::Tuple(fields, ..) => {
                for field in fields {
                    let field_type = cx.tcx.type_of(field.def_id);
                    incorrect_no_trace(&self.symbols, cx, field_type.skip_binder(), field.ty.span);
                }
            },
            _ => (), // Struct variants already caught by check_struct_def
        }
    }
}

symbols! {
    crown
    trace_in_no_trace_lint
    must_not_have_traceable
}
