/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rustc_ast::ast::{AttrKind, Attribute};
use rustc_hir::{self as hir, intravisit as visit, ExprKind};
use rustc_lint::{LateContext, LateLintPass, LintContext, LintPass, LintStore};
use rustc_middle::ty;
use rustc_session::declare_tool_lint;
use rustc_span::def_id::LocalDefId;
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
    lint_store.register_lints(&[&UNROOTED_MUST_ROOT]);
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

fn has_lint_attr(sym: &Symbols, attrs: &[Attribute], name: Symbol) -> bool {
    attrs.iter().any(|attr| {
        matches!(
            &attr.kind,
            AttrKind::Normal(normal)
            if normal.item.path.segments.len() == 3 &&
            normal.item.path.segments[0].ident.name == sym.crown &&
            normal.item.path.segments[1].ident.name == sym.unrooted_must_root_lint &&
            normal.item.path.segments[2].ident.name == name
        )
    })
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
        let recur_into_subtree = match t.kind() {
            ty::Adt(did, substs) => {
                let has_attr =
                    |did, name| has_lint_attr(sym, cx.tcx.get_attrs_unchecked(did), name);
                if has_attr(did.did(), sym.must_root) {
                    ret = true;
                    false
                } else if has_attr(did.did(), sym.allow_unrooted_interior) {
                    false
                } else if match_def_path(cx, did.did(), &[sym.alloc, sym.rc, sym.Rc]) {
                    // Rc<Promise> is okay
                    let inner = substs.type_at(0);
                    if let ty::Adt(did, _) = inner.kind() {
                        !has_attr(did.did(), sym.allow_unrooted_in_rc)
                    } else {
                        true
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
            ty::FnDef(..) | ty::FnPtr(_) => false,

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
}

impl<'tcx> LateLintPass<'tcx> for UnrootedPass {
    /// All structs containing #[crown::unrooted_must_root_lint::must_root] types
    /// must be #[crown::unrooted_must_root_lint::must_root] themselves
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item) {
        let attrs = cx.tcx.hir().attrs(item.hir_id());
        if has_lint_attr(&self.symbols, attrs, self.symbols.must_root) {
            return;
        }
        if let hir::ItemKind::Struct(def, ..) = &item.kind {
            for field in def.fields() {
                let field_type = cx.tcx.type_of(field.def_id);
                if is_unrooted_ty(&self.symbols, cx, field_type.skip_binder(), false) {
                    cx.lint(
                        UNROOTED_MUST_ROOT,
                        "Type must be rooted, use #[crown::unrooted_must_root_lint::must_root] \
                         on the struct definition to propagate",
                        |lint| {
                            lint.span(field.span);
                        },
                    )
                }
            }
        }
    }

    /// All enums containing #[crown::unrooted_must_root_lint::must_root] types
    /// must be #[crown::unrooted_must_root_lint::must_root] themselves
    fn check_variant(&mut self, cx: &LateContext, var: &hir::Variant) {
        let map = &cx.tcx.hir();
        let parent_item = map.expect_item(map.get_parent_item(var.hir_id).def_id);
        let attrs = cx.tcx.hir().attrs(parent_item.hir_id());
        if !has_lint_attr(&self.symbols, attrs, self.symbols.must_root) {
            match var.data {
                hir::VariantData::Tuple(fields, ..) => {
                    for field in fields {
                        let field_type = cx.tcx.type_of(field.def_id);
                        if is_unrooted_ty(&self.symbols, cx, field_type.skip_binder(), false) {
                            cx.lint(
                                UNROOTED_MUST_ROOT,
                                "Type must be rooted, \
                                use #[crown::unrooted_must_root_lint::must_root] \
                                on the enum definition to propagate",
                                |lint| {
                                    lint.span(field.ty.span);
                                },
                            )
                        }
                    }
                },
                _ => (), // Struct variants already caught by check_struct_def
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
                n.as_str() == "new" || n.as_str().starts_with("new_")
            },
            visit::FnKind::Closure => return,
        };

        if !in_derive_expn(span) {
            let sig = cx.tcx.type_of(def_id).skip_binder().fn_sig(cx.tcx);

            for (arg, ty) in decl.inputs.iter().zip(sig.inputs().skip_binder().iter()) {
                if is_unrooted_ty(&self.symbols, cx, *ty, false) {
                    cx.lint(UNROOTED_MUST_ROOT, "Type must be rooted", |lint| {
                        lint.span(arg.span);
                    })
                }
            }

            if !in_new_function &&
                is_unrooted_ty(&self.symbols, cx, sig.output().skip_binder(), false)
            {
                cx.lint(UNROOTED_MUST_ROOT, "Type must be rooted", |lint| {
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
                cx.lint(
                    UNROOTED_MUST_ROOT,
                    format!("Expression of type {:?} must be rooted", ty),
                    |lint| {
                        lint.span(subexpr.span);
                    },
                )
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
            hir::PatKind::Binding(hir::BindingAnnotation::NONE, ..) |
            hir::PatKind::Binding(hir::BindingAnnotation::MUT, ..) => {
                let ty = cx.typeck_results().pat_ty(pat);
                if is_unrooted_ty(self.symbols, cx, ty, self.in_new_function) {
                    cx.lint(
                        UNROOTED_MUST_ROOT,
                        format!("Expression of type {:?} must be rooted", ty),
                        |lint| {
                            lint.span(pat.span);
                        },
                    )
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
