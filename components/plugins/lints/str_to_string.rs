/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc::lint::{Context, LintPass, LintArray};
use rustc::middle::ty;
use syntax::ast;

declare_lint!(STR_TO_STRING, Deny,
              "Warn when a String could use to_owned() instead of to_string()");

/// Prefer str.to_owned() over str.to_string()
///
/// The latter creates a `Formatter` and is 5x slower than the former
pub struct StrToStringPass;

impl LintPass for StrToStringPass {
    fn get_lints(&self) -> LintArray {
        lint_array!(STR_TO_STRING)
    }

    fn check_expr(&mut self, cx: &Context, expr: &ast::Expr) {
        match expr.node {
            ast::ExprMethodCall(ref method, _, ref args)
                if method.node.name.as_str() == "to_string"
                && is_str(cx, &*args[0]) => {
                cx.span_lint(STR_TO_STRING, expr.span,
                             "str.to_owned() is more efficient than str.to_string(), please use it instead");
            },
            _ => ()
        }

        fn is_str(cx: &Context, expr: &ast::Expr) -> bool {
            fn walk_ty<'t>(ty: ty::Ty<'t>) -> ty::Ty<'t> {
                match ty.sty {
                    ty::TyRef(_, ref tm) | ty::TyRawPtr(ref tm) => walk_ty(tm.ty),
                    _ => ty
                }
            }
            match walk_ty(cx.tcx.expr_ty(expr)).sty {
                ty::TyStr => true,
                _ => false
            }
        }
    }
}
