/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc::hir;
use rustc::lint::{LateContext, LintPass, LintArray, LateLintPass, LintContext};

declare_lint!(TRANSMUTE_TYPE_LINT, Allow,
              "Warn and report types being transmuted");

/// Lint for auditing transmutes
///
/// This lint (off by default, enable with `-W transmute-type-lint`) warns about all the transmutes
/// being used, along with the types they transmute to/from.
pub struct TransmutePass;

impl LintPass for TransmutePass {
    fn get_lints(&self) -> LintArray {
        lint_array!(TRANSMUTE_TYPE_LINT)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for TransmutePass {
    fn check_expr(&mut self, cx: &LateContext, ex: &hir::Expr) {
        match ex.node {
            hir::ExprCall(ref expr, ref args) => {
                match expr.node {
                    hir::ExprPath(hir::QPath::Resolved(_, ref path)) => {
                        if path.segments.last()
                                        .map_or(false, |ref segment| &*segment.name.as_str() == "transmute") &&
                           args.len() == 1 {
                            cx.span_lint(TRANSMUTE_TYPE_LINT, ex.span,
                                         &format!("Transmute to {:?} from {:?} detected",
                                                  cx.tables.expr_ty(ex),
                                                  cx.tables.expr_ty(&args.get(0).unwrap())
                                        ));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
