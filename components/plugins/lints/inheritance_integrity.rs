/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc::lint::{Context, LintPass, LintArray, Level};
use rustc::middle::def;
use rustc::middle::def_id::DefId;
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

    fn check_struct_def(&mut self, cx: &Context, def: &ast::StructDef, _i: ast::Ident,
                        _gen: &ast::Generics, id: ast::NodeId) {
        // Lints are run post expansion, so it's fine to use
        // #[_dom_struct_marker] here without also checking for #[dom_struct]
        if cx.tcx.has_attr(DefId::local(id), "_dom_struct_marker") {
            // Find the reflector, if any
            let reflector_span = def.fields.iter().enumerate()
                                    .find(|&(ctr, f)| {
                                        if match_lang_ty(cx, &*f.node.ty, "reflector") {
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
            let dom_spans: Vec<_> = def.fields.iter().enumerate().filter_map(|(ctr, f)| {
                if let ast::TyPath(..) = f.node.ty.node {
                    if let Some(&def::PathResolution { base_def: def::DefTy(def_id, _), .. }) =
                            cx.tcx.def_map.borrow().get(&f.node.ty.id) {
                        if cx.tcx.has_attr(def_id, "_dom_struct_marker") {
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
                }
                None
            }).collect();

            // We should not have both a reflector and a dom struct field
            if let Some(sp) = reflector_span {
                if dom_spans.len() > 0 {
                    cx.span_lint(INHERITANCE_INTEGRITY, cx.tcx.map.expect_item(id).span,
                                 "This DOM struct has both Reflector and bare DOM struct members");
                    if cx.current_level(INHERITANCE_INTEGRITY) != Level::Allow {
                        let sess = cx.sess();
                        sess.span_note(sp, "Reflector found here");
                        for span in &dom_spans {
                            sess.span_note(*span, "Bare DOM struct found here");
                        }
                    }
                }
            // Nor should we have more than one dom struct field
            } else if dom_spans.len() > 1 {
                cx.span_lint(INHERITANCE_INTEGRITY, cx.tcx.map.expect_item(id).span,
                             "This DOM struct has multiple DOM struct members, only one is allowed");
                if cx.current_level(INHERITANCE_INTEGRITY) != Level::Allow {
                    for span in &dom_spans {
                        cx.sess().span_note(*span, "Bare DOM struct found here");
                    }
                }
            } else if dom_spans.is_empty() {
                cx.span_lint(INHERITANCE_INTEGRITY, cx.tcx.map.expect_item(id).span,
                             "This DOM struct has no reflector or parent DOM struct");
            }
        }
    }
}
