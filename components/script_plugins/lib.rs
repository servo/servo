/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Servo's compiler plugin/macro crate
//!
//! This crate provides the `#[unrooted_must_root_lint::must_root]` lint. This lint prevents data
//! of the marked type from being used on the stack. See the source for more details.

#![deny(unsafe_code)]
#![feature(plugin)]
#![feature(plugin_registrar)]
#![feature(rustc_private)]
#![cfg(feature = "unrooted_must_root_lint")]

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use rustc_ast::ast::{AttrKind, Attribute};
use rustc_driver::plugin::Registry;
use rustc_hir::def_id::DefId;
use rustc_hir::intravisit as visit;
use rustc_hir::{self as hir, ExprKind, HirId};
use rustc_lint::{LateContext, LateLintPass, LintContext, LintPass};
use rustc_middle::ty;
use rustc_session::declare_lint;
use rustc_span::source_map;
use rustc_span::source_map::{ExpnKind, MacroKind, Span};
use rustc_span::symbol::sym;
use rustc_span::symbol::Symbol;

#[allow(deprecated)]
#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    registrar(reg)
}

fn registrar(reg: &mut Registry) {
    let symbols = Symbols::new();
    reg.lint_store.register_lints(&[&UNROOTED_MUST_ROOT]);
    reg.lint_store
        .register_late_pass(move || Box::new(UnrootedPass::new(symbols.clone())));
}

declare_lint!(
    UNROOTED_MUST_ROOT,
    Deny,
    "Warn and report usage of unrooted jsmanaged objects"
);

/// Lint for ensuring safe usage of unrooted pointers
///
/// This lint (disable with `-A unrooted-must-root`/`#[allow(unrooted_must_root)]`) ensures that
/// `#[unrooted_must_root_lint::must_root]` values are used correctly.
///
/// "Incorrect" usage includes:
///
///  - Not being used in a struct/enum field which is not `#[unrooted_must_root_lint::must_root]` itself
///  - Not being used as an argument to a function (Except onces named `new` and `new_inherited`)
///  - Not being bound locally in a `let` statement, assignment, `for` loop, or `match` statement.
///
/// This helps catch most situations where pointers like `JS<T>` are used in a way that they can be invalidated by a
/// GC pass.
///
/// Structs which have their own mechanism of rooting their unrooted contents (e.g. `ScriptThread`)
/// can be marked as `#[allow(unrooted_must_root)]`. Smart pointers which root their interior type
/// can be marked as `#[unrooted_must_root_lint::allow_unrooted_interior]`
pub(crate) struct UnrootedPass {
    symbols: Symbols,
}

impl UnrootedPass {
    pub fn new(symbols: Symbols) -> UnrootedPass {
        UnrootedPass { symbols }
    }
}

fn has_lint_attr(sym: &Symbols, attrs: &[Attribute], name: Symbol) -> bool {
    attrs.iter().any(|attr| {
        matches!(
            &attr.kind,
            AttrKind::Normal(attr_item, _)
            if attr_item.path.segments.len() == 2 &&
            attr_item.path.segments[0].ident.name == sym.unrooted_must_root_lint &&
            attr_item.path.segments[1].ident.name == name
        )
    })
}

/// Checks if a type is unrooted or contains any owned unrooted types
fn is_unrooted_ty(sym: &Symbols, cx: &LateContext, ty: &ty::TyS, in_new_function: bool) -> bool {
    let mut ret = false;
    let mut walker = ty.walk();
    while let Some(generic_arg) = walker.next() {
        let t = match generic_arg.unpack() {
            rustc_middle::ty::subst::GenericArgKind::Type(t) => t,
            _ => {
                walker.skip_current_subtree();
                continue;
            },
        };
        let recur_into_subtree = match t.kind() {
            ty::Adt(did, substs) => {
                let has_attr = |did, name| has_lint_attr(sym, &cx.tcx.get_attrs(did), name);
                if has_attr(did.did, sym.must_root) {
                    ret = true;
                    false
                } else if has_attr(did.did, sym.allow_unrooted_interior) {
                    false
                } else if match_def_path(cx, did.did, &[sym.alloc, sym.rc, sym.Rc]) {
                    // Rc<Promise> is okay
                    let inner = substs.type_at(0);
                    if let ty::Adt(did, _) = inner.kind() {
                        if has_attr(did.did, sym.allow_unrooted_in_rc) {
                            false
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                } else if match_def_path(cx, did.did, &[sym::core, sym.cell, sym.Ref]) ||
                    match_def_path(cx, did.did, &[sym::core, sym.cell, sym.RefMut]) ||
                    match_def_path(cx, did.did, &[sym::core, sym::slice, sym::iter, sym.Iter]) ||
                    match_def_path(
                        cx,
                        did.did,
                        &[sym::core, sym::slice, sym::iter, sym.IterMut],
                    ) ||
                    match_def_path(cx, did.did, &[sym.accountable_refcell, sym.Ref]) ||
                    match_def_path(cx, did.did, &[sym.accountable_refcell, sym.RefMut]) ||
                    match_def_path(
                        cx,
                        did.did,
                        &[sym::std, sym.collections, sym.hash, sym.map, sym.Entry],
                    ) ||
                    match_def_path(
                        cx,
                        did.did,
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
                        did.did,
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
                        did.did,
                        &[sym::std, sym.collections, sym.hash, sym.map, sym.Iter],
                    ) ||
                    match_def_path(
                        cx,
                        did.did,
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
    /// All structs containing #[unrooted_must_root_lint::must_root] types
    /// must be #[unrooted_must_root_lint::must_root] themselves
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item) {
        let attrs = cx.tcx.hir().attrs(item.hir_id());
        if has_lint_attr(&self.symbols, &attrs, self.symbols.must_root) {
            return;
        }
        if let hir::ItemKind::Struct(def, ..) = &item.kind {
            for ref field in def.fields() {
                let def_id = cx.tcx.hir().local_def_id(field.hir_id);
                if is_unrooted_ty(&self.symbols, cx, cx.tcx.type_of(def_id), false) {
                    cx.lint(UNROOTED_MUST_ROOT, |lint| {
                        lint.build(
                            "Type must be rooted, use #[unrooted_must_root_lint::must_root] \
                             on the struct definition to propagate",
                        )
                        .set_span(field.span)
                        .emit()
                    })
                }
            }
        }
    }

    /// All enums containing #[unrooted_must_root_lint::must_root] types
    /// must be #[unrooted_must_root_lint::must_root] themselves
    fn check_variant(&mut self, cx: &LateContext, var: &hir::Variant) {
        let ref map = cx.tcx.hir();
        let parent_item = map.expect_item(map.get_parent_item(var.id));
        let attrs = cx.tcx.hir().attrs(parent_item.hir_id());
        if !has_lint_attr(&self.symbols, &attrs, self.symbols.must_root) {
            match var.data {
                hir::VariantData::Tuple(fields, ..) => {
                    for field in fields {
                        let def_id = cx.tcx.hir().local_def_id(field.hir_id);
                        if is_unrooted_ty(&self.symbols, cx, cx.tcx.type_of(def_id), false) {
                            cx.lint(UNROOTED_MUST_ROOT, |lint| {
                                lint.build(
                                    "Type must be rooted, \
                                    use #[unrooted_must_root_lint::must_root] \
                                    on the enum definition to propagate",
                                )
                                .set_span(field.ty.span)
                                .emit()
                            })
                        }
                    }
                },
                _ => (), // Struct variants already caught by check_struct_def
            }
        }
    }
    /// Function arguments that are #[unrooted_must_root_lint::must_root] types are not allowed
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        kind: visit::FnKind<'tcx>,
        decl: &'tcx hir::FnDecl,
        body: &'tcx hir::Body,
        span: source_map::Span,
        id: HirId,
    ) {
        let in_new_function = match kind {
            visit::FnKind::ItemFn(n, _, _, _) | visit::FnKind::Method(n, _, _) => {
                &*n.as_str() == "new" || n.as_str().starts_with("new_")
            },
            visit::FnKind::Closure => return,
        };

        if !in_derive_expn(span) {
            let def_id = cx.tcx.hir().local_def_id(id);
            let sig = cx.tcx.type_of(def_id).fn_sig(cx.tcx);

            for (arg, ty) in decl.inputs.iter().zip(sig.inputs().skip_binder().iter()) {
                if is_unrooted_ty(&self.symbols, cx, ty, false) {
                    cx.lint(UNROOTED_MUST_ROOT, |lint| {
                        lint.build("Type must be rooted").set_span(arg.span).emit()
                    })
                }
            }

            if !in_new_function {
                if is_unrooted_ty(&self.symbols, cx, sig.output().skip_binder(), false) {
                    cx.lint(UNROOTED_MUST_ROOT, |lint| {
                        lint.build("Type must be rooted")
                            .set_span(decl.output.span())
                            .emit()
                    })
                }
            }
        }

        let mut visitor = FnDefVisitor {
            symbols: &self.symbols,
            cx: cx,
            in_new_function: in_new_function,
        };
        visit::walk_expr(&mut visitor, &body.value);
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
            let ty = cx.typeck_results().expr_ty(&subexpr);
            if is_unrooted_ty(&self.symbols, cx, ty, in_new_function) {
                cx.lint(UNROOTED_MUST_ROOT, |lint| {
                    lint.build(&format!("Expression of type {:?} must be rooted", ty))
                        .set_span(subexpr.span)
                        .emit()
                })
            }
        };

        match expr.kind {
            // Trait casts from #[unrooted_must_root_lint::must_root] types are not allowed
            ExprKind::Cast(ref subexpr, _) => require_rooted(cx, self.in_new_function, &*subexpr),
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
            hir::PatKind::Binding(hir::BindingAnnotation::Unannotated, ..) |
            hir::PatKind::Binding(hir::BindingAnnotation::Mutable, ..) => {
                let ty = cx.typeck_results().pat_ty(pat);
                if is_unrooted_ty(&self.symbols, cx, ty, self.in_new_function) {
                    cx.lint(UNROOTED_MUST_ROOT, |lint| {
                        lint.build(&format!("Expression of type {:?} must be rooted", ty))
                            .set_span(pat.span)
                            .emit()
                    })
                }
            },
            _ => {},
        }

        visit::walk_pat(self, pat);
    }

    fn visit_ty(&mut self, _: &'tcx hir::Ty) {}

    fn nested_visit_map(&mut self) -> hir::intravisit::NestedVisitorMap<Self::Map> {
        hir::intravisit::NestedVisitorMap::OnlyBodies(self.cx.tcx.hir())
    }
}

/// check if a DefId's path matches the given absolute type path
/// usage e.g. with
/// `match_def_path(cx, id, &["core", "option", "Option"])`
fn match_def_path(cx: &LateContext, def_id: DefId, path: &[Symbol]) -> bool {
    let def_path = cx.tcx.def_path(def_id);
    let krate = &cx.tcx.crate_name(def_path.krate);
    if krate != &path[0] {
        return false;
    }

    let path = &path[1..];
    let other = def_path.data;

    if other.len() != path.len() {
        return false;
    }

    other
        .into_iter()
        .zip(path)
        .all(|(e, p)| e.data.get_opt_name().as_ref() == Some(p))
}

fn in_derive_expn(span: Span) -> bool {
    matches!(
        span.ctxt().outer_expn_data().kind,
        ExpnKind::Macro(MacroKind::Derive, ..)
    )
}

macro_rules! symbols {
    ($($s: ident)+) => {
        #[derive(Clone)]
        #[allow(non_snake_case)]
        struct Symbols {
            $( $s: Symbol, )+
        }

        impl Symbols {
            fn new() -> Self {
                Symbols {
                    $( $s: Symbol::intern(stringify!($s)), )+
                }
            }
        }
    }
}

symbols! {
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
