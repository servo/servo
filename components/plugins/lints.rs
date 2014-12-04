/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syntax::{ast, ast_util, codemap, visit};
use syntax::ast::Public;
use syntax::attr::AttrMetaMethods;
use rustc::lint::{Context, LintPass, LintArray};
use rustc::middle::ty::expr_ty;
use rustc::middle::{ty, def};
use rustc::middle::typeck::astconv::AstConv;
use rustc::util::ppaux::Repr;

declare_lint!(TRANSMUTE_TYPE_LINT, Allow,
              "Warn and report types being transmuted")
declare_lint!(UNROOTED_MUST_ROOT, Deny,
              "Warn and report usage of unrooted jsmanaged objects")
declare_lint!(PRIVATIZE, Deny,
              "Allows to enforce private fields for struct definitions")

/// Lint for auditing transmutes
///
/// This lint (off by default, enable with `-W transmute-type-lint`) warns about all the transmutes
/// being used, along with the types they transmute to/from.
pub struct TransmutePass;

/// Lint for ensuring safe usage of unrooted pointers
///
/// This lint (disable with `-A unrooted-must-root`/`#[allow(unrooted_must_root)]`) ensures that `#[must_root]` values are used correctly.
/// "Incorrect" usage includes:
///
///  - Not being used in a struct/enum field which is not `#[must_root]` itself
///  - Not being used as an argument to a function (Except onces named `new` and `new_inherited`)
///  - Not being bound locally in a `let` statement, assignment, `for` loop, or `match` statement.
///
/// This helps catch most situations where pointers like `JS<T>` are used in a way that they can be invalidated by a GC pass.
pub struct UnrootedPass;

/// Lint for keeping DOM fields private
///
/// This lint (disable with `-A privatize`/`#[allow(privatize)]`) ensures all types marked with `#[privatize]` have no private fields
pub struct PrivatizePass;

impl LintPass for TransmutePass {
    fn get_lints(&self) -> LintArray {
        lint_array!(TRANSMUTE_TYPE_LINT)
    }

    fn check_expr(&mut self, cx: &Context, ex: &ast::Expr) {
        if let ast::ExprCall(ref expr, ref args) = ex.node {
            if let ast::ExprPath(ref path) = expr.node {
                if path.segments.last()
                                .map_or(false, |ref segment| segment.identifier.name.as_str() == "transmute")
                   && args.len() == 1 {
                    let tcx = cx.tcx();
                    cx.span_lint(TRANSMUTE_TYPE_LINT, ex.span,
                                 format!("Transmute from {} to {} detected",
                                         expr_ty(tcx, ex).repr(tcx),
                                         expr_ty(tcx, &**args.get(0).unwrap()).repr(tcx)
                                ).as_slice());
                }
            }
        }
    }
}

// Checks if a type has the #[must_root] annotation.
// Unwraps pointers as well
// TODO (#3874, sort of): unwrap other types like Vec/Option/HashMap/etc
fn lint_unrooted_ty(cx: &Context, ty: &ast::Ty, warning: &str) {
    match ty.node {
        ast::TyUniq(ref t) |
        ast::TyVec(ref t) | ast::TyFixedLengthVec(ref t, _) |
        ast::TyPtr(ast::MutTy { ty: ref t, ..}) | ast::TyRptr(_, ast::MutTy { ty: ref t, ..}) => lint_unrooted_ty(cx, &**t, warning),
        ast::TyPath(_, _, id) => {
                if let def::DefTy(def_id, _) = cx.tcx.def_map.borrow().get_copy(&id) {
                    if ty::has_attr(cx.tcx, def_id, "must_root") {
                        cx.span_lint(UNROOTED_MUST_ROOT, ty.span, warning);
                    }
                }
            }
            _ => (),
    };
}

impl LintPass for UnrootedPass {
    fn get_lints(&self) -> LintArray {
        lint_array!(UNROOTED_MUST_ROOT)
    }
    /// All structs containing #[must_root] types must be #[must_root] themselves
    fn check_struct_def(&mut self, cx: &Context, def: &ast::StructDef, _i: ast::Ident, _gen: &ast::Generics, id: ast::NodeId) {
        if cx.tcx.map.expect_item(id).attrs.iter().all(|a| !a.check_name("must_root")) {
            for ref field in def.fields.iter() {
                lint_unrooted_ty(cx, &*field.node.ty,
                                 "Type must be rooted, use #[must_root] on the struct definition to propagate");
            }
        }
    }
    /// All enums containing #[must_root] types must be #[must_root] themselves
    fn check_variant(&mut self, cx: &Context, var: &ast::Variant, _gen: &ast::Generics) {
        let ref map = cx.tcx.map;
        if map.expect_item(map.get_parent(var.node.id)).attrs.iter().all(|a| !a.check_name("must_root")) {
            if let ast::TupleVariantKind(ref vec) = var.node.kind {
                for ty in vec.iter() {
                    lint_unrooted_ty(cx, &*ty.ty,
                                     "Type must be rooted, use #[must_root] on the enum definition to propagate")
                }
            }
        }
    }
    /// Function arguments that are #[must_root] types are not allowed
    fn check_fn(&mut self, cx: &Context, kind: visit::FnKind, decl: &ast::FnDecl,
                block: &ast::Block, _span: codemap::Span, _id: ast::NodeId) {
        match kind {
            visit::FkItemFn(i, _, _, _) |
            visit::FkMethod(i, _, _) if i.as_str() == "new" || i.as_str() == "new_inherited" => {
                return;
            }
            _ => ()
        }
        if let ast::DefaultBlock = block.rules {
            for arg in decl.inputs.iter() {
                lint_unrooted_ty(cx, &*arg.ty,
                                 "Type must be rooted")
            }
        }
    }

    // Partially copied from rustc::middle::lint::builtin
    // Catches `let` statements and assignments which store a #[must_root] value
    // Expressions which return out of blocks eventually end up in a `let` or assignment
    // statement or a function return (which will be caught when it is used elsewhere)
    fn check_stmt(&mut self, cx: &Context, s: &ast::Stmt) {
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
                ast::ExprMatch(ref e, _, _) |
                // For loops allow you to bind a return value locally
                ast::ExprForLoop(_, ref e, _, _) => &**e,
                // XXXManishearth look into `if let` once it lands in our rustc
                _ => return
            },
            _ => return
        };

        let t = expr_ty(cx.tcx, &*expr);
        match ty::get(t).sty {
            ty::ty_struct(did, _) |
            ty::ty_enum(did, _) => {
                if ty::has_attr(cx.tcx, did, "must_root") {
                    cx.span_lint(UNROOTED_MUST_ROOT, expr.span,
                                 format!("Expression of type {} must be rooted", t.repr(cx.tcx)).as_slice());
                }
            }
            _ => {}
        }
    }
}

impl LintPass for PrivatizePass {
    fn get_lints(&self) -> LintArray {
        lint_array!(PRIVATIZE)
    }

    fn check_struct_def(&mut self, cx: &Context, def: &ast::StructDef, _i: ast::Ident, _gen: &ast::Generics, id: ast::NodeId) {
        if ty::has_attr(cx.tcx, ast_util::local_def(id), "privatize") {
            for field in def.fields.iter() {
                if let ast::StructField_ { kind: ast::NamedField(ident, visibility), .. } = field.node {
                    if visibility == Public {
                        cx.span_lint(PRIVATIZE, field.span,
                                     format!("Field {} is public where only private fields are allowed", ident.name).as_slice());
                    }
                }
            }
        }
    }
}
