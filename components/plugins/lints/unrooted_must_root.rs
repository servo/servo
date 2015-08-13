/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syntax::{ast, codemap, visit};
use syntax::attr::AttrMetaMethods;
use rustc::ast_map;
use rustc::lint::{Context, LintPass, LintArray};
use rustc::middle::ty;
use rustc::middle::astconv_util::ast_ty_to_prim_ty;
use utils::{match_def_path, unsafe_context};

declare_lint!(UNROOTED_MUST_ROOT, Deny,
              "Warn and report usage of unrooted jsmanaged objects");

/// Lint for ensuring safe usage of unrooted pointers
///
/// This lint (disable with `-A unrooted-must-root`/`#[allow(unrooted_must_root)]`) ensures that `#[must_root]`
/// values are used correctly.
///
/// "Incorrect" usage includes:
///
///  - Not being used in a struct/enum field which is not `#[must_root]` itself
///  - Not being used as an argument to a function (Except onces named `new` and `new_inherited`)
///  - Not being bound locally in a `let` statement, assignment, `for` loop, or `match` statement.
///
/// This helps catch most situations where pointers like `JS<T>` are used in a way that they can be invalidated by a
/// GC pass.
///
/// Structs which have their own mechanism of rooting their unrooted contents (e.g. `ScriptTask`)
/// can be marked as `#[allow(unrooted_must_root)]`. Smart pointers which root their interior type
/// can be marked as `#[allow_unrooted_interior]`
pub struct UnrootedPass {
    in_new_function: bool
}

impl UnrootedPass {
    pub fn new() -> UnrootedPass {
        UnrootedPass {
            in_new_function: true
        }
    }
}

/// Checks if a type is unrooted or contains any owned unrooted types
fn is_unrooted_ty(cx: &Context, ty: &ty::TyS, in_new_function: bool) -> bool {
    let mut ret = false;
    ty.maybe_walk(|t| {
        match t.sty {
            ty::TyStruct(did, _) |
            ty::TyEnum(did, _) => {
                if cx.tcx.has_attr(did.did, "must_root") {
                    ret = true;
                    false
                } else if cx.tcx.has_attr(did.did, "allow_unrooted_interior") {
                    false
                } else if match_def_path(cx, did.did, &["core", "cell", "Ref"])
                        || match_def_path(cx, did.did, &["core", "cell", "RefMut"]) {
                        // Ref and RefMut are borrowed pointers, okay to hold unrooted stuff
                        // since it will be rooted elsewhere
                    false
                } else {
                    true
                }
            },
            ty::TyBox(..) if in_new_function => false, // box in new() is okay
            ty::TyRef(..) => false, // don't recurse down &ptrs
            ty::TyRawPtr(..) => false, // don't recurse down *ptrs
            _ => true
        }
    });
    ret
}

impl LintPass for UnrootedPass {
    fn get_lints(&self) -> LintArray {
        lint_array!(UNROOTED_MUST_ROOT)
    }
    /// All structs containing #[must_root] types must be #[must_root] themselves
    fn check_struct_def(&mut self,
                        cx: &Context,
                        def: &ast::StructDef,
                        _i: ast::Ident,
                        _gen: &ast::Generics,
                        id: ast::NodeId) {
        let item = match cx.tcx.map.get(id) {
            ast_map::Node::NodeItem(item) => item,
            _ => cx.tcx.map.expect_item(cx.tcx.map.get_parent(id)),
        };
        if item.attrs.iter().all(|a| !a.check_name("must_root")) {
            for ref field in def.fields.iter() {
                if is_unrooted_ty(cx, cx.tcx.node_id_to_type(field.node.id), false) {
                    cx.span_lint(UNROOTED_MUST_ROOT, field.span,
                                 "Type must be rooted, use #[must_root] on the struct definition to propagate")
                }
            }
        }
    }
    /// All enums containing #[must_root] types must be #[must_root] themselves
    fn check_variant(&mut self, cx: &Context, var: &ast::Variant, _gen: &ast::Generics) {
        let ref map = cx.tcx.map;
        if map.expect_item(map.get_parent(var.node.id)).attrs.iter().all(|a| !a.check_name("must_root")) {
            match var.node.kind {
                ast::TupleVariantKind(ref vec) => {
                    for ty in vec.iter() {
                        ast_ty_to_prim_ty(cx.tcx, &*ty.ty).map(|t| {
                            if is_unrooted_ty(cx, t, false) {
                                cx.span_lint(UNROOTED_MUST_ROOT, ty.ty.span,
                                             "Type must be rooted, use #[must_root] on \
                                              the enum definition to propagate")
                            }
                        });
                    }
                }
                _ => () // Struct variants already caught by check_struct_def
            }
        }
    }
    /// Function arguments that are #[must_root] types are not allowed
    fn check_fn(&mut self, cx: &Context, kind: visit::FnKind, decl: &ast::FnDecl,
                block: &ast::Block, _span: codemap::Span, id: ast::NodeId) {
        match kind {
            visit::FkItemFn(i, _, _, _, _, _) |
            visit::FkMethod(i, _, _) if i.name.as_str() == "new"
                                        || i.name.as_str() == "new_inherited"
                                        || i.name.as_str() == "new_initialized" => {
                self.in_new_function = true;
                return;
            },
            visit::FkItemFn(_, _, style, _, _, _) => match style {
                ast::Unsafety::Unsafe => return,
                _ => ()
            },
            _ => ()
        }
        self.in_new_function = false;

        if unsafe_context(&cx.tcx.map, id) {
            return;
        }

        match block.rules {
            ast::DefaultBlock => {
                for arg in decl.inputs.iter() {
                    ast_ty_to_prim_ty(cx.tcx, &*arg.ty).map(|t| {
                        if is_unrooted_ty(cx, t, false) {
                            cx.span_lint(UNROOTED_MUST_ROOT, arg.ty.span, "Type must be rooted")
                        }
                    });
                }
            }
            _ => () // fn is `unsafe`
        }
    }

    /// Trait casts from #[must_root] types are not allowed
    fn check_expr(&mut self, cx: &Context, expr: &ast::Expr) {
        fn require_rooted(cx: &Context, in_new_function: bool, subexpr: &ast::Expr) {
            let ty = cx.tcx.expr_ty(&*subexpr);
            if is_unrooted_ty(cx, ty, in_new_function) {
                cx.span_lint(UNROOTED_MUST_ROOT,
                             subexpr.span,
                             &format!("Expression of type {:?} must be rooted", ty))
            }
        };

        match expr.node {
            ast::ExprCast(ref subexpr, _) => require_rooted(cx, self.in_new_function, &*subexpr),
            _ => {
                // TODO(pcwalton): Check generics with a whitelist of allowed generics.
            }
        }
    }

    // Partially copied from rustc::middle::lint::builtin
    // Catches `let` statements and assignments which store a #[must_root] value
    // Expressions which return out of blocks eventually end up in a `let` or assignment
    // statement or a function return (which will be caught when it is used elsewhere)
    fn check_stmt(&mut self, cx: &Context, s: &ast::Stmt) {
        match s.node {
            ast::StmtDecl(_, id) |
            ast::StmtExpr(_, id) |
            ast::StmtSemi(_, id) if unsafe_context(&cx.tcx.map, id) => {
                return
            },
            _ => ()
        };

        let expr = match s.node {
            // Catch a `let` binding
            ast::StmtDecl(ref decl, _) => match decl.node {
                ast::DeclLocal(ref loc) => match loc.init {
                    Some(ref e) => &**e,
                    _ => return
                },
                _ => return
            },
            ast::StmtExpr(ref expr, _) => match expr.node {
                // This catches deferred `let` statements
                ast::ExprAssign(_, ref e) |
                // Match statements allow you to bind onto the variable later in an arm
                // We need not check arms individually since enum/struct fields are already
                // linted in `check_struct_def` and `check_variant`
                // (so there is no way of destructuring out a `#[must_root]` field)
                ast::ExprMatch(ref e, _, _) => &**e,
                // These are able to bind local variables, but are desugared into
                // loops and matches pre-lint so should not be encountered
                ast::ExprForLoop(..) => unreachable!(),
                ast::ExprIfLet(..) => unreachable!(),
                ast::ExprWhileLet(..) => unreachable!(),
                _ => return
            },
            _ => return
        };

        let ty = cx.tcx.expr_ty(&*expr);
        if is_unrooted_ty(cx, ty, self.in_new_function) {
            cx.span_lint(UNROOTED_MUST_ROOT, expr.span,
                                     &format!("Expression of type {:?} must be rooted", ty))
        }
    }
}
