/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::FnKind;
use rustc_hir::{Body, FnDecl, ImplicitSelfKind, TyKind};
use rustc_lint::{LateContext, LateLintPass, Lint, LintContext, LintPass, LintStore};
use rustc_macros::Diagnostic;
use rustc_session::declare_tool_lint;
use rustc_span::symbol::Symbol;
use rustc_span::Span;

use crate::symbols;

declare_tool_lint! {
    pub crown::JSCONTEXT_FIRST_ARG,
    Warn,
    "Warn and report functions that don't have JSContext as their first argument"
}

pub fn register(lint_store: &mut LintStore) {
    let symbols = Symbols::new();
    lint_store.register_lints(&[JSCONTEXT_FIRST_ARG]);
    lint_store.register_late_pass(move |_| Box::new(JSContextFirstArgPass::new(symbols.clone())));
}

#[derive(Diagnostic)]
#[diag("the first argument should be JSContext")]
struct JSContextNotFirstArg {}

/// Lint for checking if JSContext is the first argument of functions
///
/// This lint (disable with `-A crown::jscontext_first_arg`/`#[allow(crown::jscontext_first_arg)]`) ensures that
/// JSContext types are always the first argument
///
/// "Incorrect" usage includes:
///
///  - fn(&self, foo: String, cx: &mut JSContext)
///  - fn(foo: String, cx: &mut JSContext)
///  - fn(&self, foo: String, cx: &JSContext)
///  - fn(foo: String, cx: &JSContext)
///
/// "Correct" usage for these:
///
///  - fn(&self, cx: &mut JSContext, foo: String)
///  - fn(cx: &mut JSContext, foo: String)
///  - fn(&self, cx: &JSContext, foo: String)
///  - fn(cx: &JSContext, foo: String)
///
pub(crate) struct JSContextFirstArgPass {
    symbols: Symbols,
}

impl JSContextFirstArgPass {
    pub(crate) fn new(symbols: Symbols) -> JSContextFirstArgPass {
        JSContextFirstArgPass { symbols }
    }
}

impl LintPass for JSContextFirstArgPass {
    fn name(&self) -> &'static str {
        "ServoJSContextFirstArgPass"
    }

    fn get_lints(&self) -> Vec<&'static Lint> {
        vec![JSCONTEXT_FIRST_ARG]
    }
}

impl<'tcx> JSContextFirstArgPass {
    fn is_jscontext_type(&self, cx: &LateContext<'tcx>, ty: &rustc_hir::Ty) -> bool {
        let TyKind::Ref(_, inner) = ty.kind else {
            return false;
        };
        let TyKind::Path(qpath) = inner.ty.kind else {
            return false;
        };
        let Some(def_id) = cx
            .typeck_results()
            .qpath_res(&qpath, inner.ty.hir_id)
            .opt_def_id()
        else {
            return false;
        };
        let def_path = cx.tcx.def_path(def_id);
        def_path
            .data
            .iter()
            .any(|def| def.data.get_opt_name() == Some(self.symbols.JSContext))
    }
}

impl<'tcx> LateLintPass<'tcx> for JSContextFirstArgPass {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        decl: &'tcx FnDecl<'tcx>,
        _: &'tcx Body<'tcx>,
        span: Span,
        _: LocalDefId,
    ) {
        let num_skip_args = if matches!(decl.implicit_self(), ImplicitSelfKind::None) {
            // Only skip the first argument
            1
        } else {
            // Skip both `self` (with any possible reference) and the first argument
            2
        };
        if decl
            .inputs
            .iter()
            .skip(num_skip_args)
            .any(|arg| self.is_jscontext_type(cx, arg))
        {
            cx.emit_span_lint(JSCONTEXT_FIRST_ARG, span, JSContextNotFirstArg {});
        }
    }
}

symbols! {
    JSContext
}
