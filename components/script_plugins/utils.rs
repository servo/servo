/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc::hir::def_id::DefId;
use rustc::lint::{LateContext, LintContext};
use syntax::codemap::{ExpnFormat, Span};

/// check if a DefId's path matches the given absolute type path
/// usage e.g. with
/// `match_def_path(cx, id, &["core", "option", "Option"])`
pub fn match_def_path(cx: &LateContext, def_id: DefId, path: &[&str]) -> bool {
    let krate = &cx.tcx.crate_name(def_id.krate);
    if krate != &path[0] {
        return false;
    }

    let path = &path[1..];
    let other = cx.tcx.def_path(def_id).data;

    if other.len() != path.len() {
        return false;
    }

    other.into_iter()
         .map(|e| e.data)
         .zip(path)
         .all(|(nm, p)| &*nm.as_interned_str() == *p)
}

pub fn in_derive_expn(cx: &LateContext, span: Span) -> bool {
    cx.sess().codemap().with_expn_info(span.expn_id,
            |info| {
                if let Some(i) = info {
                    if let ExpnFormat::MacroAttribute(n) = i.callee.format {
                        if n.as_str().contains("derive") {
                            true
                        } else { false }
                    } else { false }
                } else { false }
            })
}
