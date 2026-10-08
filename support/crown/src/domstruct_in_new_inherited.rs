/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rustc_hir::def_id::{DefId, LOCAL_CRATE};
use rustc_hir::{self as hir};
use rustc_lint::{LateContext, LateLintPass, Lint, LintContext, LintPass, LintStore};
use rustc_macros::Diagnostic;
use rustc_middle::ty;
use rustc_session::declare_tool_lint;
use rustc_span::symbol::Symbol;

use crate::common::{implements_trait, match_def_path, trait_in_crate};
use crate::symbols;

declare_tool_lint! {
    pub crown::DOMSTRUCT_IN_NEW_INHERITED,
    Deny,
    "Deny and report struct instantiation outside of `new_inherited` methods"
}

pub fn register(lint_store: &mut LintStore) {
    lint_store.register_lints(&[DOMSTRUCT_IN_NEW_INHERITED]);
    lint_store
        .register_late_pass(move |_| Box::new(DomStructInNewInheritedPass::new(Symbols::new())));
}

#[derive(Diagnostic)]
#[diag("dom objects must only be constructed inside a `new_inherited` method on that dom object")]
struct NotInNewInheritedDiagnostic {}

#[derive(Diagnostic)]
#[diag("new_inherited must be passed to `Box::new` and inside a `reflect_dom_object`")]
struct NewInheritedPassedToReflect {}

/// Lint for checking if manual dom strings are using appropriate APIs
///
/// This lint (disable with `-A crown::domstruct_in_new_inherited`/`#[allow(crown::domstruct_in_new_inherited)]`) ensures that
/// dom structs are only instantiated in a method `new_inherited` on that struct.
///
/// "Incorrect" usage includes:
///
///  - ```
///  impl DomObject {
///    fn some_other_method() {
///      DomObject {}
///    }
///  }
///  ```
///
///  - ```
///  fn some_other_method() {
///    DomObject {}
///  }
///  ```
///
/// "Correct" usage for these:
///
///  ```
///  impl DomObject {
///    fn new_inherited() {
///      DomObject {}
///    }
///    fn new(cx: &mut JSContext, global: &GlobalScope) {
///      reflect_dom_object(
///         cx,
///         Box::new(Self::new_inherited()),
///         global,
///      )
///    }
///  }
///  ```
///
pub(crate) struct DomStructInNewInheritedPass {
    symbols: Symbols,
}

impl DomStructInNewInheritedPass {
    pub(crate) fn new(symbols: Symbols) -> DomStructInNewInheritedPass {
        DomStructInNewInheritedPass { symbols }
    }
}

impl LintPass for DomStructInNewInheritedPass {
    fn name(&self) -> &'static str {
        "ServoDomStructInNewInheritedPass"
    }

    fn get_lints(&self) -> Vec<&'static Lint> {
        vec![DOMSTRUCT_IN_NEW_INHERITED]
    }
}

fn is_domobject<'tcx>(cx: &LateContext<'tcx>, ty: ty::Ty<'tcx>) -> bool {
    trait_in_crate(&cx.tcx, LOCAL_CRATE, Symbol::intern("DomObject"))
        .map(|trait_id| implements_trait(cx, ty, trait_id, &[]))
        .unwrap_or_default()
}
impl<'tcx> DomStructInNewInheritedPass {
    fn is_inside_new_inherited_method(cx: &LateContext<'tcx>, expr: &'tcx hir::Expr) -> bool {
        let Some(hir::intravisit::FnKind::Method(ident, _)) = cx
            .tcx
            .hir_parent_iter(expr.hir_id)
            .find_map(|ancestor| ancestor.1.fn_kind())
        else {
            return false;
        };
        ident.as_str() == "new_inherited"
    }

    fn def_id_for_path(
        cx: &LateContext<'tcx>,
        path: &hir::QPath<'_>,
        hir_id: hir::HirId,
    ) -> Option<DefId> {
        let hir::def::Res::Def(_, def_id) = cx.typeck_results().qpath_res(path, hir_id) else {
            return None;
        };
        Some(def_id)
    }

    fn def_id_if_expr_is_call(cx: &LateContext<'tcx>, expr: &'tcx hir::Expr) -> Option<DefId> {
        let hir::ExprKind::Call(callee, _) = expr.kind else {
            return None;
        };
        let hir::ExprKind::Path(path) = callee.kind else {
            return None;
        };
        Self::def_id_for_path(cx, &path, callee.hir_id)
    }

    fn check_struct_instantiation(cx: &LateContext<'tcx>, expr: &'tcx hir::Expr) {
        let ty = cx.typeck_results().expr_ty(expr);
        let ty::Adt(adt_def, _) = ty.kind() else {
            return;
        };
        if !is_domobject(cx, ty) {
            return;
        };
        let span = expr.span;
        if !Self::is_inside_new_inherited_method(cx, expr) {
            cx.emit_span_lint(
                DOMSTRUCT_IN_NEW_INHERITED,
                span,
                NotInNewInheritedDiagnostic {},
            );
            return;
        }
        // Find the nearest `impl` declaration and check if it is the same declaration
        // as the struct instantiation.
        let Some(impl_def) = cx.tcx.hir_parent_iter(expr.hir_id).find_map(|ancestor| {
            if let hir::Node::Item(item) = ancestor.1 {
                if let hir::ItemKind::Impl(impl_def) = item.kind {
                    return Some(impl_def);
                }
            }
            None
        }) else {
            cx.emit_span_lint(
                DOMSTRUCT_IN_NEW_INHERITED,
                span,
                NotInNewInheritedDiagnostic {},
            );
            return;
        };
        let hir::TyKind::Path(path) = impl_def.self_ty.kind else {
            return;
        };
        let def_id_of_impl = Self::def_id_for_path(cx, &path, impl_def.self_ty.hir_id);
        if def_id_of_impl != Some(adt_def.did()) {
            cx.emit_span_lint(
                DOMSTRUCT_IN_NEW_INHERITED,
                span,
                NotInNewInheritedDiagnostic {},
            );
            return;
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for DomStructInNewInheritedPass {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr) {
        if matches!(expr.kind, hir::ExprKind::Struct(..)) {
            return Self::check_struct_instantiation(cx, expr);
        }
        let Some(def_id) = Self::def_id_if_expr_is_call(cx, expr) else {
            return;
        };
        let path = cx.tcx.def_path(def_id);
        let Some(last_part) = path.data.last() else {
            return;
        };
        if last_part.as_sym(false).as_str() != "new_inherited" {
            return;
        }
        // At this point, we know we are calling `Self::new_inherited()`
        // Check if this call itself isn't also inside a `fn new_inherited()`.
        // That's the case for subclasses
        if Self::is_inside_new_inherited_method(cx, expr) {
            return;
        }
        let span = expr.span;
        let hir::Node::Expr(parent) = cx.tcx.parent_hir_node(expr.hir_id) else {
            cx.emit_span_lint(
                DOMSTRUCT_IN_NEW_INHERITED,
                span,
                NewInheritedPassedToReflect {},
            );
            return;
        };
        let Some(parent_def_id) = Self::def_id_if_expr_is_call(cx, parent) else {
            cx.emit_span_lint(
                DOMSTRUCT_IN_NEW_INHERITED,
                span,
                NewInheritedPassedToReflect {},
            );
            return;
        };
        // Check if `new_inherited` is directly passed into `Box::new()` or `Rc::new()`
        let sym = &self.symbols;
        if !match_def_path(
            cx,
            parent_def_id,
            &[sym.alloc, sym.boxed, sym.Impl, sym.new],
        ) && !match_def_path(cx, parent_def_id, &[sym.alloc, sym.rc, sym.Impl, sym.new])
        {
            cx.emit_span_lint(
                DOMSTRUCT_IN_NEW_INHERITED,
                span,
                NewInheritedPassedToReflect {},
            );
            return;
        }
        let hir::Node::Expr(parent_of_parent) = cx.tcx.parent_hir_node(parent.hir_id) else {
            cx.emit_span_lint(
                DOMSTRUCT_IN_NEW_INHERITED,
                span,
                NewInheritedPassedToReflect {},
            );
            return;
        };
        let Some(parent_of_parent_def_id) = Self::def_id_if_expr_is_call(cx, parent_of_parent)
        else {
            cx.emit_span_lint(
                DOMSTRUCT_IN_NEW_INHERITED,
                span,
                NewInheritedPassedToReflect {},
            );
            return;
        };
        let path = cx.tcx.def_path(parent_of_parent_def_id);
        let Some(last_part) = path.data.last() else {
            return;
        };
        let part_name = last_part.as_sym(false);
        let part_name = part_name.as_str();
        if !part_name.starts_with("reflect_dom_object") &&
            !part_name.starts_with("reflect_weak_referenceable_dom_object") &&
            part_name != "reflect_node_with_proto"
        {
            cx.emit_span_lint(
                DOMSTRUCT_IN_NEW_INHERITED,
                span,
                NewInheritedPassedToReflect {},
            );
            return;
        }
    }
}

symbols! {
    alloc
    boxed
    Impl
    new
    rc
}
