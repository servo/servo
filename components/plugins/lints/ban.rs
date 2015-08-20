/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc::lint::{Context, LintPass, LintArray};
use syntax::ast;
use utils::match_ty_unwrap;

declare_lint!(BANNED_TYPE, Deny,
              "Ban various unsafe type combinations");

/// Lint for banning various unsafe types
///
/// Banned types:
///
/// - `Cell<JSVal>`
/// - `Cell<JS<T>>`
pub struct BanPass;

impl LintPass for BanPass {
    fn get_lints(&self) -> LintArray {
        lint_array!(BANNED_TYPE)
    }

    fn check_ty(&mut self, cx: &Context, ty: &ast::Ty) {
        if match_ty_unwrap(ty, &["std","cell","Cell"])
            .and_then(|t| t.get(0))
            .and_then(|t| match_ty_unwrap(&**t, &["dom", "bindings", "js", "JS"]))
            .is_some() {
            cx.span_lint(BANNED_TYPE, ty.span, "Banned type Cell<JS<T>> detected. Use MutHeap<JS<T>> instead")
        }
        if match_ty_unwrap(ty, &["std","cell","Cell"])
            .and_then(|t| t.get(0))
            .and_then(|t| match_ty_unwrap(&**t, &["js", "jsval", "JSVal"]))
            .is_some() {
            cx.span_lint(BANNED_TYPE, ty.span, "Banned type Cell<JSVal> detected. Use MutHeap<JSVal> instead")
        }
    }
}
