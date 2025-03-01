/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rustc_hir::{self as hir, intravisit as visit, ExprKind};
use rustc_lint::{LateContext, LateLintPass, Lint, LintContext, LintPass, LintStore};
use rustc_middle::ty;
use rustc_session::declare_tool_lint;
use rustc_span::def_id::{DefId, LocalDefId};
use rustc_span::symbol::{sym, Symbol};

use crate::common::{in_derive_expn, match_def_path};
use crate::symbols;

declare_tool_lint! {
    pub crown::UNROOTED_MUST_ROOT,
    Deny,
    "Warn and report usage of unrooted jsmanaged objects"
}

pub fn register(lint_store: &mut LintStore) {
    let symbols = Symbols::new();
    lint_store.register_lints(&[UNROOTED_MUST_ROOT]);
    lint_store.register_late_pass(move |_| Box::new(UnrootedPass::new(symbols.clone())));
}

/// Lint for ensuring safe usage of unrooted pointers
///
/// This lint (disable with `-A unrooted-must-root`/`#[allow(unrooted_must_root)]`) ensures that
/// `#[crown::unrooted_must_root_lint::must_root]` values are used correctly.
///
/// "Incorrect" usage includes:
///
///  - Not being used in a struct/enum field which is not `#[crown::unrooted_must_root_lint::must_root]` itself
///  - Not being used as an argument to a function (Except onces named `new` and `new_inherited`)
///  - Not being bound locally in a `let` statement, assignment, `for` loop, or `match` statement.
///
/// This helps catch most situations where pointers like `JS<T>` are used in a way that they can be invalidated by a
/// GC pass.
///
/// Structs which have their own mechanism of rooting their unrooted contents (e.g. `ScriptThread`)
/// can be marked as `#[allow(unrooted_must_root)]`. Smart pointers which root their interior type
/// can be marked as `#[crown::unrooted_must_root_lint::allow_unrooted_interior]`
pub(crate) struct UnrootedPass {
    symbols: Symbols,
}

impl UnrootedPass {
    pub(crate) fn new(symbols: Symbols) -> UnrootedPass {
        UnrootedPass { symbols }
    }
}

/// For a given associated type for a trait implementation, checks if a given crown annotation
/// is present on that type.
fn associated_type_has_attr<'tcx>(
    sym: &'_ Symbols,
    cx: &LateContext<'tcx>,
    ty: ty::Ty<'tcx>,
    attr: Symbol,
) -> bool {
    let mut walker = ty.walk();
    while let Some(generic_arg) = walker.next() {
        let t = match generic_arg.unpack() {
            rustc_middle::ty::GenericArgKind::Type(t) => t,
            _ => {
                walker.skip_current_subtree();
                continue;
            },
        };
        match t.kind() {
            ty::Adt(did, _substs) => {
                return cx.tcx.has_attrs_with_path(
                    did.did(),
                    &[sym.crown, sym.unrooted_must_root_lint, attr],
                );
            },
            ty::Alias(
                ty::AliasTyKind::Projection | ty::AliasTyKind::Inherent | ty::AliasTyKind::Weak,
                ty,
            ) => {
                return cx.tcx.has_attrs_with_path(
                    ty.def_id,
                    &[sym.crown, sym.unrooted_must_root_lint, attr],
                )
            },
            _ => {},
        }
    }
    false
}

/// Checks if a type is unrooted or contains any owned unrooted types
fn is_unrooted_ty<'tcx>(
    sym: &'_ Symbols,
    cx: &LateContext<'tcx>,
    ty: ty::Ty<'tcx>,
    in_new_function: bool,
) -> bool {
    let mut ret = false;
    let mut walker = ty.walk();
    while let Some(generic_arg) = walker.next() {
        let t = match generic_arg.unpack() {
            rustc_middle::ty::GenericArgKind::Type(t) => t,
            _ => {
                walker.skip_current_subtree();
                continue;
            },
        };
        let has_attr = |did, name| {
            cx.tcx
                .has_attrs_with_path(did, &[sym.crown, sym.unrooted_must_root_lint, name])
        };
        let recur_into_subtree = match t.kind() {
            ty::Adt(did, substs) => {
                if has_attr(did.did(), sym.must_root) {
                    ret = true;
                    false
                } else if has_attr(did.did(), sym.allow_unrooted_interior) {
                    false
                } else if match_def_path(cx, did.did(), &[sym.alloc, sym.rc, sym.Rc]) {
                    // Rc<Promise> is okay
                    let inner = substs.type_at(0);
                    match inner.kind() {
                        ty::Adt(did, _) => !has_attr(did.did(), sym.allow_unrooted_in_rc),
                        ty::Alias(
                            ty::AliasTyKind::Projection |
                            ty::AliasTyKind::Inherent |
                            ty::AliasTyKind::Weak,
                            ty,
                        ) => !has_attr(ty.def_id, sym.allow_unrooted_in_rc),
                        _ => true,
                    }
                } else if match_def_path(cx, did.did(), &[sym::core, sym.cell, sym.Ref]) ||
                    match_def_path(cx, did.did(), &[sym::core, sym.cell, sym.RefMut]) ||
                    match_def_path(cx, did.did(), &[sym::core, sym::slice, sym::iter, sym.Iter]) ||
                    match_def_path(
                        cx,
                        did.did(),
                        &[sym::core, sym::slice, sym::iter, sym.IterMut],
                    ) ||
                    match_def_path(cx, did.did(), &[sym.accountable_refcell, sym.Ref]) ||
                    match_def_path(cx, did.did(), &[sym.accountable_refcell, sym.RefMut]) ||
                    match_def_path(
                        cx,
                        did.did(),
                        &[sym::std, sym.collections, sym.hash, sym.map, sym.Entry],
                    ) ||
                    match_def_path(
                        cx,
                        did.did(),
                        &[
                            sym::std,
                            sym.collections,
                            sym.hash,
                            sym.map,
                            sym.OccupiedEntry,
                        ],
                    ) ||
                    match_def_path(
                        cx,
                        did.did(),
                        &[
                            sym::std,
                            sym.collections,
                            sym.hash,
                            sym.map,
                            sym.VacantEntry,
                        ],
                    ) ||
                    match_def_path(
                        cx,
                        did.did(),
                        &[sym::std, sym.collections, sym.hash, sym.map, sym.Iter],
                    ) ||
                    match_def_path(
                        cx,
                        did.did(),
                        &[sym::std, sym.collections, sym.hash, sym.set, sym.Iter],
                    )
                {
                    // Structures which are semantically similar to an &ptr.
                    false
                } else if did.is_box() && in_new_function {
                    // box in new() is okay
                    false
                } else {
                    true
                }
            },
            ty::Ref(..) => false,    // don't recurse down &ptrs
            ty::RawPtr(..) => false, // don't recurse down *ptrs
            ty::FnDef(..) | ty::FnPtr(..) => false,
            ty::Alias(
                ty::AliasTyKind::Projection | ty::AliasTyKind::Inherent | ty::AliasTyKind::Weak,
                ty,
            ) => {
                if has_attr(ty.def_id, sym.must_root) {
                    ret = true;
                    false
                } else if has_attr(ty.def_id, sym.allow_unrooted_interior) {
                    false
                } else {
                    true
                }
            },
            _ => true,
        };
        if !recur_into_subtree {
            walker.skip_current_subtree();
        }
    }
    ret
}

impl LintPass for UnrootedPass {
    fn name(&self) -> &'static str {
        "ServoUnrootedPass"
    }

    fn get_lints(&self) -> Vec<&'static Lint> {
        vec![UNROOTED_MUST_ROOT]
    }
}

impl<'tcx> LateLintPass<'tcx> for UnrootedPass {
    /// All structs containing #[crown::unrooted_must_root_lint::must_root] types
    /// must be #[crown::unrooted_must_root_lint::must_root] themselves
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item) {
        let sym = &self.symbols;
        let has_attr = |symbol| {
            cx.tcx.has_attrs_with_path(
                item.hir_id().expect_owner(),
                &[sym.crown, sym.unrooted_must_root_lint, symbol],
            )
        };
        if has_attr(sym.must_root) || has_attr(sym.allow_unrooted_interior) {
            return;
        }
        if let hir::ItemKind::Struct(def, ..) = &item.kind {
            for field in def.fields() {
                let field_type = cx.tcx.type_of(field.def_id);
                if is_unrooted_ty(&self.symbols, cx, field_type.skip_binder(), false) {
                    cx.lint(UNROOTED_MUST_ROOT, |lint| {
                          lint.primary_message(
                              "Type must be rooted, use #[crown::unrooted_must_root_lint::must_root] \
                               on the struct definition to propagate."
                              );
                          lint.span(field.span);
                      })
                }
            }
        }
    }

    /// All enums containing #[crown::unrooted_must_root_lint::must_root] types
    /// must be #[crown::unrooted_must_root_lint::must_root] themselves
    fn check_variant(&mut self, cx: &LateContext, var: &hir::Variant) {
        let map = &cx.tcx.hir();
        let parent_item = map.expect_item(map.get_parent_item(var.hir_id).def_id);
        let sym = &self.symbols;
        if !cx.tcx.has_attrs_with_path(
            parent_item.hir_id().expect_owner(),
            &[sym.crown, sym.unrooted_must_root_lint, sym.must_root],
        ) {
            #[allow(clippy::single_match)]
            match var.data {
                hir::VariantData::Tuple(fields, ..) => {
                    for field in fields {
                        let field_type = cx.tcx.type_of(field.def_id);
                        if is_unrooted_ty(&self.symbols, cx, field_type.skip_binder(), false) {
                            cx.lint(UNROOTED_MUST_ROOT, |lint| {
                                lint.primary_message(
                                    "Type must be rooted, \
                                      use #[crown::unrooted_must_root_lint::must_root] \
                                      on the enum definition to propagate.",
                                );
                                lint.span(field.ty.span);
                            })
                        }
                    }
                },
                _ => (), // Struct variants already caught by check_struct_def
            }
        }
    }

    /// for trait_type_impl_must_root test
    fn check_trait_item(
        &mut self,
        cx: &LateContext<'tcx>,
        trait_item: &'tcx rustc_hir::TraitItem<'tcx>,
    ) {
        let hir::TraitItemKind::Type(_, _) = trait_item.kind else {
            return;
        };

        let sym = &self.symbols;
        let has_attr = |did, name| {
            cx.tcx
                .has_attrs_with_path(did, &[sym.crown, sym.unrooted_must_root_lint, name])
        };

        let def_id: DefId = trait_item.hir_id().expect_owner().into();
        let must_root_present = has_attr(def_id, sym.must_root);

        let allow_unrooted_interior_present = has_attr(def_id, sym.allow_unrooted_interior);

        let allow_unrooted_in_rc_present = has_attr(def_id, sym.allow_unrooted_in_rc);

        let trait_id = cx
            .tcx
            .trait_of_item(trait_item.hir_id().expect_owner().to_def_id())
            .unwrap();
        // we need to make sure that each impl has same crown attrs
        let impls = cx.tcx.trait_impls_of(trait_id);
        for (_ty, impl_def_ids) in impls.non_blanket_impls() {
            for impl_def_id in impl_def_ids {
                let type_impl = cx
                    .tcx
                    .associated_items(impl_def_id)
                    .find_by_name_and_kind(cx.tcx, trait_item.ident, ty::AssocKind::Type, trait_id)
                    .unwrap();

                let mir_ty = cx.tcx.type_of(type_impl.def_id).skip_binder();

                let impl_ty_must_root = associated_type_has_attr(sym, cx, mir_ty, sym.must_root);
                let impl_ty_allow_unrooted =
                    associated_type_has_attr(sym, cx, mir_ty, sym.allow_unrooted_interior);
                let impl_ty_allow_rc =
                    associated_type_has_attr(sym, cx, mir_ty, sym.allow_unrooted_in_rc);

                if impl_ty_must_root != must_root_present {
                    if !must_root_present && impl_ty_must_root {
                        cx.lint(UNROOTED_MUST_ROOT, |lint| {
                            lint.primary_message(
                                "Type trait declaration must be marked with \
                                 #[crown::unrooted_must_root_lint::must_root] \
                                 to allow binding must_root types in associated types.",
                            );
                            lint.span(trait_item.span);
                        });
                    } else {
                        cx.lint(UNROOTED_MUST_ROOT, |lint| {
                            lint.primary_message(
                                "Mismatched use of \
                                 #[crown::unrooted_must_root_lint::must_root] \
                                 between associated type declaration and impl definition.",
                            );
                            lint.span(trait_item.span);
                        });
                    }
                }

                if impl_ty_allow_unrooted != allow_unrooted_interior_present {
                    cx.lint(UNROOTED_MUST_ROOT, |lint| {
                        lint.primary_message(
                            "Mismatched use of \
                             #[crown::unrooted_must_root_lint::allow_unrooted_interior] \
                             between associated type declaration and impl definition.",
                        );
                        lint.span(trait_item.span);
                    });
                }

                if impl_ty_allow_rc != allow_unrooted_in_rc_present {
                    cx.lint(UNROOTED_MUST_ROOT, |lint| {
                        lint.primary_message(
                            "Mismatched use of \
                             #[crown::unrooted_must_root_lint::allow_unrooted_interior_in_rc] \
                             between associated type declaration and impl definition.",
                        );
                        lint.span(trait_item.span);
                    });
                }
            }
        }
    }

    /// Function arguments that are #[crown::unrooted_must_root_lint::must_root] types are not allowed
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        kind: visit::FnKind<'tcx>,
        decl: &'tcx hir::FnDecl,
        body: &'tcx hir::Body,
        span: rustc_span::Span,
        def_id: LocalDefId,
    ) {
        let in_new_function = match kind {
            visit::FnKind::ItemFn(n, _, _) | visit::FnKind::Method(n, _) => {
                n.as_str() == "new" ||
                    n.as_str().starts_with("new_") ||
                    n.as_str() == "default" ||
                    n.as_str() == "Wrap"
            },
            visit::FnKind::Closure => return,
        };

        if !in_derive_expn(span) {
            let sig = cx.tcx.type_of(def_id).skip_binder().fn_sig(cx.tcx);

            for (arg, ty) in decl.inputs.iter().zip(sig.inputs().skip_binder().iter()) {
                if is_unrooted_ty(&self.symbols, cx, *ty, in_new_function) {
                    cx.lint(UNROOTED_MUST_ROOT, |lint| {
                        lint.primary_message("Type must be rooted.");
                        lint.span(arg.span);
                    })
                }
            }

            if !in_new_function &&
                is_unrooted_ty(&self.symbols, cx, sig.output().skip_binder(), false)
            {
                cx.lint(UNROOTED_MUST_ROOT, |lint| {
                    lint.primary_message("Type must be rooted.");
                    lint.span(decl.output.span());
                })
            }
        }

        let mut visitor = FnDefVisitor {
            symbols: &self.symbols,
            cx,
            in_new_function,
        };
        visit::walk_expr(&mut visitor, body.value);
    }
}

struct FnDefVisitor<'a, 'tcx: 'a> {
    symbols: &'a Symbols,
    cx: &'a LateContext<'tcx>,
    in_new_function: bool,
}

impl<'a, 'tcx> visit::Visitor<'tcx> for FnDefVisitor<'a, 'tcx> {
    type Map = rustc_middle::hir::map::Map<'tcx>;

    fn visit_expr(&mut self, expr: &'tcx hir::Expr) {
        let cx = self.cx;

        let require_rooted = |cx: &LateContext, in_new_function: bool, subexpr: &hir::Expr| {
            let ty = cx.typeck_results().expr_ty(subexpr);
            if is_unrooted_ty(self.symbols, cx, ty, in_new_function) {
                cx.lint(UNROOTED_MUST_ROOT, |lint| {
                    lint.primary_message(format!("Expression of type {:?} must be rooted.", ty));
                    lint.span(subexpr.span);
                })
            }
        };

        match expr.kind {
            // Trait casts from #[crown::unrooted_must_root_lint::must_root] types are not allowed
            ExprKind::Cast(subexpr, _) => require_rooted(cx, self.in_new_function, subexpr),
            // This catches assignments... the main point of this would be to catch mutable
            // references to `JS<T>`.
            // FIXME: Enable this? Triggers on certain kinds of uses of DomRefCell.
            // hir::ExprAssign(_, ref rhs) => require_rooted(cx, self.in_new_function, &*rhs),
            // This catches calls; basically, this enforces the constraint that only constructors
            // can call other constructors.
            // FIXME: Enable this? Currently triggers with constructs involving DomRefCell, and
            // constructs like Vec<JS<T>> and RootedVec<JS<T>>.
            // hir::ExprCall(..) if !self.in_new_function => {
            //     require_rooted(cx, self.in_new_function, expr);
            // }
            _ => {
                // TODO(pcwalton): Check generics with a whitelist of allowed generics.
            },
        }

        visit::walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &'tcx hir::Pat) {
        let cx = self.cx;

        // We want to detect pattern bindings that move a value onto the stack.
        // When "default binding modes" https://github.com/rust-lang/rust/issues/42640
        // are implemented, the `Unannotated` case could cause false-positives.
        // These should be fixable by adding an explicit `ref`.
        match pat.kind {
            hir::PatKind::Binding(hir::BindingMode::NONE, ..) |
            hir::PatKind::Binding(hir::BindingMode::MUT, ..) => {
                let ty = cx.typeck_results().pat_ty(pat);
                if is_unrooted_ty(self.symbols, cx, ty, self.in_new_function) {
                    cx.lint(UNROOTED_MUST_ROOT, |lint| {
                        lint.primary_message(format!(
                            "Expression of type {:?} must be rooted.",
                            ty
                        ));
                        lint.span(pat.span);
                    })
                }
            },
            _ => {},
        }

        visit::walk_pat(self, pat);
    }

    fn visit_ty(&mut self, _: &'tcx hir::Ty) {}

    fn nested_visit_map(&mut self) -> Self::Map {
        self.cx.tcx.hir()
    }
}

symbols! {
    crown
    unrooted_must_root_lint
    allow_unrooted_interior
    allow_unrooted_in_rc
    must_root
    alloc
    rc
    Rc
    cell
    accountable_refcell
    Ref
    RefMut
    Iter
    IterMut
    collections
    hash
    map
    set
    Entry
    OccupiedEntry
    VacantEntry
}
