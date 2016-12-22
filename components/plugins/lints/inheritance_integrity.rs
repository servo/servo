/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc::hir::{self, def};
use rustc::lint::{LateContext, LintPass, LintArray, Level, LateLintPass, LintContext};
use syntax::ast;
use utils::match_lang_ty;

declare_lint!(INHERITANCE_INTEGRITY, Deny,
              "Ensures that struct fields are properly laid out for inheritance to work");

/// Lint for ensuring proper layout of DOM structs
///
/// A DOM struct must have one Reflector field or one field
/// which itself is a DOM struct (in which case it must be the first field).
pub struct InheritancePass;

impl LintPass for InheritancePass {
    fn get_lints(&self) -> LintArray {
        lint_array!(INHERITANCE_INTEGRITY)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for InheritancePass {
    fn check_struct_def(&mut self, cx: &LateContext, def: &hir::VariantData, _n: ast::Name,
                        _gen: &hir::Generics, id: ast::NodeId) {
        // Lints are run post expansion, so it's fine to use
        // #[_dom_struct_marker] here without also checking for #[dom_struct]
        if cx.tcx.has_attr(cx.tcx.map.local_def_id(id), "_dom_struct_marker") {
            // Find the reflector, if any
            let reflector_span = def.fields().iter().enumerate()
                                    .find(|&(ctr, f)| {
                                        if match_lang_ty(cx, &*f.ty, "reflector") {
                                            if ctr > 0 {
                                                cx.span_lint(INHERITANCE_INTEGRITY, f.span,
                                                             "The Reflector should be the first field of the DOM \
                                                             struct");
                                            }
                                            return true;
                                        }
                                        false
                                    })
                                    .map(|(_, f)| f.span);
            // Find all #[dom_struct] fields
            let dom_spans: Vec<_> = def.fields().iter().enumerate().filter_map(|(ctr, f)| {
                if let hir::TyPath(hir::QPath::Resolved(_, ref path)) = f.ty.node {
                    if let def::Def::PrimTy(_) = path.def {
                        return None;
                    }
                    if cx.tcx.has_attr(path.def.def_id(), "_dom_struct_marker") {
                        // If the field is not the first, it's probably
                        // being misused (a)
                        if ctr > 0 {
                            cx.span_lint(INHERITANCE_INTEGRITY, f.span,
                                         "Bare DOM structs should only be used as the first field of a \
                                          DOM struct. Consider using JS<T> instead.");
                        }
                        return Some(f.span)
                    }
                }
                None
            }).collect();

            // We should not have both a reflector and a dom struct field
            if let Some(sp) = reflector_span {
                if dom_spans.len() > 0 {
                    let mut db = cx.struct_span_lint(INHERITANCE_INTEGRITY,
                                                     cx.tcx.map.expect_item(id).span,
                                                     "This DOM struct has both Reflector \
                                                      and bare DOM struct members");
                    if cx.current_level(INHERITANCE_INTEGRITY) != Level::Allow {
                        db.span_note(sp, "Reflector found here");
                        for span in &dom_spans {
                            db.span_note(*span, "Bare DOM struct found here");
                        }
                    }
                }
            // Nor should we have more than one dom struct field
            } else if dom_spans.len() > 1 {
                let mut db = cx.struct_span_lint(INHERITANCE_INTEGRITY,
                                                 cx.tcx.map.expect_item(id).span,
                                                 "This DOM struct has multiple \
                                                  DOM struct members, only one is allowed");
                if cx.current_level(INHERITANCE_INTEGRITY) != Level::Allow {
                    for span in &dom_spans {
                        db.span_note(*span, "Bare DOM struct found here");
                    }
                }
            } else if dom_spans.is_empty() {
                cx.span_lint(INHERITANCE_INTEGRITY, cx.tcx.map.expect_item(id).span,
                             "This DOM struct has no reflector or parent DOM struct");
            }
        }
    }
}
