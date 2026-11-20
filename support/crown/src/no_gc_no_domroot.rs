/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::FnKind;
use rustc_hir::{intravisit, Body, Expr, ExprKind, FnDecl, QPath, Ty, TyKind};
use rustc_lint::{LateContext, LateLintPass, Lint, LintContext, LintPass, LintStore};
use rustc_macros::Diagnostic;
use rustc_session::declare_tool_lint;
use rustc_span::symbol::Symbol;
use rustc_span::Span;

use crate::common::is_dom_root_ty;
use crate::symbols;

declare_tool_lint! {
    pub crown::NO_GC_NO_DOMROOT,
    Warn,
    "Warn and report function calls that return a `DomRoot` inside a method that takes `NoGC`"
}

pub fn register(lint_store: &mut LintStore) {
    let symbols = Symbols::new();
    lint_store.register_lints(&[NO_GC_NO_DOMROOT]);
    lint_store.register_late_pass(move |_| Box::new(NoGcNoDomRootPass::new(symbols.clone())));
}

#[derive(Diagnostic)]
#[diag("method call that returns DomRoot inside a method that has NoGC")]
struct DomRootCallInNoGcMethod {}

pub(crate) struct NoGcNoDomRootPass {
    symbols: Symbols,
}

impl NoGcNoDomRootPass {
    pub(crate) fn new(symbols: Symbols) -> NoGcNoDomRootPass {
        NoGcNoDomRootPass { symbols }
    }
}

impl LintPass for NoGcNoDomRootPass {
    fn name(&self) -> &'static str {
        "ServoNoGcNoDomRootPass"
    }

    fn get_lints(&self) -> Vec<&'static Lint> {
        vec![NO_GC_NO_DOMROOT]
    }
}

struct FnDefVisitor<'v, 'tcx: 'v> {
    symbols: &'v Symbols,
    cx: &'v LateContext<'tcx>,
}

impl<'v, 'tcx> intravisit::Visitor<'tcx> for FnDefVisitor<'v, 'tcx> {
    fn visit_expr(&mut self, ex: &'tcx Expr) -> Self::Result {
        let ExprKind::MethodCall(_, _, _, span) = ex.kind else {
            intravisit::walk_expr(self, ex);
            return;
        };
        let return_type = self.cx.typeck_results().expr_ty(ex);
        let sym = self.symbols;
        if is_dom_root_ty(
            &[sym.script_bindings, sym.root, sym.Root],
            self.cx,
            return_type,
        ) {
            self.cx
                .emit_span_lint(NO_GC_NO_DOMROOT, span, DomRootCallInNoGcMethod {});
        } else {
            intravisit::walk_expr(self, ex);
        }
    }
}

impl<'tcx> NoGcNoDomRootPass {
    fn is_no_gc_type(&self, ty: &Ty) -> bool {
        let TyKind::Ref(_, ref inner) = ty.kind else {
            return false;
        };
        let TyKind::Path(QPath::Resolved(_, path)) = inner.ty.kind else {
            return false;
        };
        path.segments
            .iter()
            .any(|segment| segment.ident.name == self.symbols.NoGC)
    }
}

impl<'tcx> LateLintPass<'tcx> for NoGcNoDomRootPass {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        decl: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        _: Span,
        _: LocalDefId,
    ) {
        if !decl.inputs.iter().any(|arg| self.is_no_gc_type(arg)) {
            return;
        }
        let mut visitor = FnDefVisitor {
            symbols: &self.symbols,
            cx,
        };
        let _ = intravisit::walk_expr(&mut visitor, body.value);
    }
}

symbols! {
    NoGC
    script_bindings
    root
    Root
}
