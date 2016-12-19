/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc::hir;
use rustc::lint::{LateContext, LintPass, LintArray, LateLintPass, LintContext};
use syntax::ast;

declare_lint!(PRIVATIZE, Deny,
              "Allows to enforce private fields for struct definitions");

/// Lint for keeping DOM fields private
///
/// This lint (disable with `-A privatize`/`#[allow(privatize)]`) ensures all types marked with `#[privatize]`
/// have no public fields
pub struct PrivatizePass;

impl LintPass for PrivatizePass {
    fn get_lints(&self) -> LintArray {
        lint_array!(PRIVATIZE)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for PrivatizePass {
    fn check_struct_def(&mut self,
                        cx: &LateContext,
                        def: &hir::VariantData,
                        _n: ast::Name,
                        _gen: &hir::Generics,
                        id: ast::NodeId) {
        if cx.tcx.has_attr(cx.tcx.map.local_def_id(id), "privatize") {
            for field in def.fields() {
                if field.vis == hir::Public {
                    cx.span_lint(PRIVATIZE, field.span,
                                 &format!("Field {} is public where only private fields are allowed",
                                          field.name));
                }
            }
        }
    }
}
