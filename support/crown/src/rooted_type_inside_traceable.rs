/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rustc_hir::{Item, ItemKind};
use rustc_lint::{LateContext, LateLintPass, Lint, LintContext, LintPass, LintStore};
use rustc_macros::Diagnostic;
use rustc_middle::ty;
use rustc_session::declare_tool_lint;
use rustc_span::symbol::Symbol;

use crate::common::{is_jstraceable, match_def_path};
use crate::symbols;

declare_tool_lint! {
    pub crown::DOMROOT_INSIDE_DOM_STRUCT,
    Warn,
    "Warn about usage of `Root` inside Traceable types"
}

pub fn register(lint_store: &mut LintStore) {
    let symbols = Symbols::new();
    lint_store.register_lints(&[DOMROOT_INSIDE_DOM_STRUCT]);
    lint_store.register_late_pass(move |_| Box::new(NoDomRootPass::new(symbols.clone())));
}

pub(crate) struct NoDomRootPass {
    symbols: Symbols,
}

impl NoDomRootPass {
    pub(crate) fn new(symbols: Symbols) -> Self {
        Self { symbols }
    }
}

impl LintPass for NoDomRootPass {
    fn name(&self) -> &'static str {
        "ServoNoDomRootPass"
    }

    fn get_lints(&self) -> Vec<&'static Lint> {
        vec![DOMROOT_INSIDE_DOM_STRUCT]
    }
}

#[derive(Diagnostic)]
#[diag("storing a rooted type can lead to circular references")]
struct TraceableTypeShouldNotContainRootedTypeDiagnostic;

fn is_dom_root_ty<'tcx>(sym: &'_ Symbols, cx: &LateContext<'tcx>, ty: ty::Ty<'tcx>) -> bool {
    let mut walker = ty.walk();
    while let Some(generic_arg) = walker.next() {
        let ty::GenericArgKind::Type(ty) = generic_arg.kind() else {
            walker.skip_current_subtree();
            continue;
        };

        if let ty::Adt(did, _) = ty.kind() {
            // DomRoot<T> desugars to `Root<Dom<T>>`
            // In fact it's `Root` that can cause issues.
            if match_def_path(cx, did.did(), &[sym.script_bindings, sym.root, sym.Root]) {
                return true;
            }

            // The lint relies on the property that each type recursively implements `Traceable` and
            // only checks its direct fields, avoiding recursive searches.
            // Code-generated types are currently allowed, due to the changes required to avoid
            // storing a `Root<T>`.
            // This should be safe as long as we ensure they are not used elsewhere.
            let def_path = cx.tcx.def_path(did.did());
            if cx.tcx.crate_name(def_path.krate) == sym.script_bindings {
                let maybe_codegen = def_path.data.get(0).and_then(|e| e.data.get_opt_name());
                let inner_module = def_path.data.get(1).and_then(|e| e.data.get_opt_name());

                if maybe_codegen == Some(sym.codegen) {
                    if inner_module == Some(sym.GenericUnionTypes) ||
                        inner_module == Some(sym.GenericBindings)
                    {
                        let adt_def = cx.tcx.adt_def(did.did());

                        for variant in adt_def.variants() {
                            for field in &variant.fields {
                                let field_type = cx.tcx.type_of(field.did);
                                if is_dom_root_ty(sym, cx, field_type.skip_binder()) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

impl<'tcx> LateLintPass<'tcx> for NoDomRootPass {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item) {
        let variants = match &item.kind {
            ItemKind::Struct(_, _, variant_data) => vec![variant_data],
            ItemKind::Enum(_, _, enum_def) => enum_def.variants.iter().map(|v| &v.data).collect(),
            _ => return,
        };

        let item_type = cx.tcx.type_of(item.owner_id.def_id);

        // Filter out types that are not visible to GC
        if !is_jstraceable(cx, item_type.skip_binder()) {
            return;
        }

        // Then check if any field is DomRoot<T>
        for variant_data in variants {
            for field in variant_data.fields() {
                let field_type = cx.tcx.type_of(field.def_id);
                if is_dom_root_ty(&self.symbols, cx, field_type.skip_binder()) {
                    cx.emit_span_lint(
                        DOMROOT_INSIDE_DOM_STRUCT,
                        field.span,
                        TraceableTypeShouldNotContainRootedTypeDiagnostic,
                    );
                }
            }
        }
    }
}

symbols! {
    script_bindings
    root
    Root
    codegen
    GenericUnionTypes
    GenericBindings
}
