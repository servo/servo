/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rustc_hir::{self as hir};
use rustc_lint::{LateContext, LateLintPass, Lint, LintContext, LintPass, LintStore};
use rustc_macros::Diagnostic;
use rustc_middle::ty;
use rustc_session::declare_tool_lint;
use rustc_span::symbol::Symbol;
use rustc_span::Span;

use crate::common::{is_expr_kind_empty_str, match_def_path, value_if_expr_is_str};
use crate::symbols;

declare_tool_lint! {
    pub crown::MANUAL_DOMSTRING_NEW,
    Warn,
    "Warn and report creation of manual domstrings"
}

pub fn register(lint_store: &mut LintStore) {
    let symbols = Symbols::new();
    lint_store.register_lints(&[MANUAL_DOMSTRING_NEW]);
    lint_store.register_late_pass(move |_| Box::new(ManualDOMStringPass::new(symbols.clone())));
}

#[derive(Diagnostic)]
#[diag("use DOMString::new() instead")]
struct EmptyDOMStringDiagnostic {
    #[suggestion(
        "use constructor instead",
        applicability = "machine-applicable",
        code = "DOMString::new()"
    )]
    span: Span,
}

#[derive(Diagnostic)]
#[diag("use DOMString::from_static(\"{$value}\") instead")]
struct StaticDOMStringDiagnostic {
    value: String,
    #[suggestion(
        "use from_static instead",
        applicability = "machine-applicable",
        code = "DOMString::from_static(\"{value}\")"
    )]
    span: Span,
}

/// Lint for checking if manual dom strings are using appropriate APIs
///
/// This lint (disable with `-A crown::manual_domstring_new`/`#[allow(crown::manual_domstring_new)]`) ensures that
/// static domstrings are created appropriately.
///
/// "Incorrect" usage includes:
///
///  - DomString::from("Static string")
///  - "Static string".into()
///  - DomString::from("")
///  - "".into()
///
/// "Correct" usage for these:
///
///  - DomString::from_static("Static string")
///  - DomString::from_static("Static string")
///  - DomString::new()
///  - DomString::new()
///
pub(crate) struct ManualDOMStringPass {
    symbols: Symbols,
}

impl ManualDOMStringPass {
    pub(crate) fn new(symbols: Symbols) -> ManualDOMStringPass {
        ManualDOMStringPass { symbols }
    }
}

impl LintPass for ManualDOMStringPass {
    fn name(&self) -> &'static str {
        "ServoManualDOMStringPass"
    }

    fn get_lints(&self) -> Vec<&'static Lint> {
        vec![MANUAL_DOMSTRING_NEW]
    }
}

impl<'tcx> ManualDOMStringPass {
    fn is_dom_string_path(&self, def_path: hir::definitions::DefPath) -> bool {
        def_path
            .data
            .iter()
            .any(|def| def.data.get_opt_name() == Some(self.symbols.DOMString))
    }

    fn is_domstring_type(&self, cx: &LateContext<'tcx>, ty: ty::Ty) -> bool {
        match ty.kind() {
            ty::Adt(did, _) => {
                let def_path = cx.tcx.def_path(did.did());
                self.is_dom_string_path(def_path)
            },
            _ => false,
        }
    }

    fn maybe_report_for_string_value(
        cx: &LateContext<'tcx>,
        span: Span,
        expr_kind: &hir::ExprKind<'_>,
        hir_id: hir::HirId,
    ) {
        if is_expr_kind_empty_str(expr_kind) {
            cx.emit_span_lint(
                MANUAL_DOMSTRING_NEW,
                span,
                EmptyDOMStringDiagnostic { span },
            );
        } else if let Some(value) = value_if_expr_is_str(cx, expr_kind, hir_id) {
            cx.emit_span_lint(
                MANUAL_DOMSTRING_NEW,
                span,
                StaticDOMStringDiagnostic { value, span },
            );
        }
    }
}

// NoTrace correct usage of NoTrace must only be checked on Struct (item) and Enums (variants)
// as these are the only ones that are actually traced
impl<'tcx> LateLintPass<'tcx> for ManualDOMStringPass {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr) {
        let sym = &self.symbols;
        match expr.kind {
            // Any static function call like `DOMString::from`
            hir::ExprKind::Call(callee, args) => {
                let ty = cx.typeck_results().expr_ty(callee);
                let ty::FnDef(fn_, def_id) = ty.kind() else {
                    return;
                };
                let Some(inner) = def_id.get(0).and_then(|arg| arg.as_type()) else {
                    return;
                };
                // The type we struct we call a function on is the DOMString
                // and the function we call is `From::from`
                if !self.is_domstring_type(cx, inner) {
                    return;
                }
                if !match_def_path(cx, *fn_, &[sym.core, sym.convert, sym.From, sym.from]) {
                    return;
                }
                if args.len() != 1 {
                    return;
                }
                Self::maybe_report_for_string_value(cx, expr.span, &args[0].kind, args[0].hir_id);
            },
            hir::ExprKind::MethodCall(path, callee, _, _) => {
                let ty = cx.typeck_results().expr_ty(expr);
                // The type the object we call a function on is the DOMString
                if !self.is_domstring_type(cx, ty) {
                    return;
                }
                // The function we call is `Into::into`
                if path.ident.name != sym.into {
                    return;
                }
                Self::maybe_report_for_string_value(cx, expr.span, &callee.kind, callee.hir_id);
            },
            _ => {},
        }
    }
}

symbols! {
    DOMString
    core
    convert
    From
    from
    into
}
