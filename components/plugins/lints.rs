/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syntax::{ast, codemap, visit};
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

pub struct TransmutePass;
pub struct UnrootedPass;

impl LintPass for TransmutePass {
    fn get_lints(&self) -> LintArray {
        lint_array!(TRANSMUTE_TYPE_LINT)
    }

    fn check_expr(&mut self, cx: &Context, ex: &ast::Expr) {
        match ex.node {
            ast::ExprCall(ref expr, ref args) => {
                match expr.node {
                    ast::ExprPath(ref path) => {
                        if path.segments.last()
                                        .map_or(false, |ref segment| segment.identifier.name.as_str() == "transmute")
                           && args.len() == 1 {
                            let tcx = cx.tcx();
                            cx.span_lint(TRANSMUTE_TYPE_LINT, ex.span,
                                         format!("Transmute from {} to {} detected",
                                                 expr_ty(tcx, ex).repr(tcx),
                                                 expr_ty(tcx, &**args.get(0)).repr(tcx)
                                        ).as_slice());
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn lint_unrooted_ty(cx: &Context, ty: &ast::Ty, warning: &str) {
    match ty.node {
        ast::TyBox(ref t) | ast::TyUniq(ref t) |
        ast::TyVec(ref t) | ast::TyFixedLengthVec(ref t, _) |
        ast::TyPtr(ast::MutTy { ty: ref t, ..}) | ast::TyRptr(_, ast::MutTy { ty: ref t, ..}) => lint_unrooted_ty(cx, &**t, warning),
        ast::TyPath(_, _, id) => {
                match cx.tcx.def_map.borrow().get_copy(&id) {
                    def::DefTy(def_id) => {
                        if ty::has_attr(cx.tcx, def_id, "must_root") {
                            cx.span_lint(UNROOTED_MUST_ROOT, ty.span, warning);
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
    };
}

impl LintPass for UnrootedPass {
    fn get_lints(&self) -> LintArray {
        lint_array!(UNROOTED_MUST_ROOT)
    }

    fn check_struct_def(&mut self, cx: &Context, def: &ast::StructDef, _i: ast::Ident, _gen: &ast::Generics, id: ast::NodeId) {
        if cx.tcx.map.expect_item(id).attrs.iter().all(|a| !a.check_name("must_root")) {
            for ref field in def.fields.iter() {
                lint_unrooted_ty(cx, &*field.node.ty,
                                 "Type must be rooted, use #[must_root] on the struct definition to propagate");
            }
        }
    }

    fn check_variant(&mut self, cx: &Context, var: &ast::Variant, _gen: &ast::Generics) {
        let ref map = cx.tcx.map;
        if map.expect_item(map.get_parent(var.node.id)).attrs.iter().all(|a| !a.check_name("must_root")) {
            match var.node.kind {
                ast::TupleVariantKind(ref vec) => {
                    for ty in vec.iter() {
                        lint_unrooted_ty(cx, &*ty.ty,
                                         "Type must be rooted, use #[must_root] on the enum definition to propagate")
                    }
                }
                _ => () // Struct variants already caught by check_struct_def
            }
        }
    }

    fn check_fn(&mut self, cx: &Context, kind: visit::FnKind, decl: &ast::FnDecl,
                block: &ast::Block, _span: codemap::Span, _id: ast::NodeId) {
        match kind {
            visit::FkItemFn(i, _, _, _) |
            visit::FkMethod(i, _, _) if i.as_str() == "new" || i.as_str() == "new_inherited" => {
                return;
            }
            _ => ()
        }
        match block.rules {
            ast::DefaultBlock => {
                for arg in decl.inputs.iter() {
                    lint_unrooted_ty(cx, &*arg.ty,
                                     "Type must be rooted, use #[must_root] on the fn definition to propagate")
                }
            }
            _ => () // fn is `unsafe`
        }
    }

    // Partially copied from rustc::middle::lint::builtin
    // Catches `let` statements which store a #[must_root] value
    // Expressions which return out of blocks eventually end up in a `let`
    // statement or a function return (which will be caught when it is used elsewhere)
    fn check_stmt(&mut self, cx: &Context, s: &ast::Stmt) {
        // Catch the let binding
        let expr = match s.node {
            ast::StmtDecl(ref decl, _) => match decl.node {
                ast::DeclLocal(ref loc) => match loc.init {
                        Some(ref e) => &**e,
                        _ => return
                },
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

