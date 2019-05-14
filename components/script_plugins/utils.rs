/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rustc::hir::def_id::DefId;
use rustc::lint::LateContext;
use syntax::source_map::{ExpnFormat, Span};
use syntax::symbol::Symbol;

/// check if a DefId's path matches the given absolute type path
/// usage e.g. with
/// `match_def_path(cx, id, &["core", "option", "Option"])`
pub fn match_def_path(cx: &LateContext, def_id: DefId, path: &[Symbol]) -> bool {
    let krate = &cx.tcx.crate_name(def_id.krate);
    if krate != &path[0] {
        return false;
    }

    let path = &path[1..];
    let other = cx.tcx.def_path(def_id).data;

    if other.len() != path.len() {
        return false;
    }

    other
        .into_iter()
        .zip(path)
        .all(|(e, p)| e.data.as_interned_str().as_symbol() == *p)
}

pub fn in_derive_expn(span: Span) -> bool {
    if let Some(i) = span.ctxt().outer().expn_info() {
        if let ExpnFormat::MacroAttribute(n) = i.format {
            n.as_str().contains("derive")
        } else {
            false
        }
    } else {
        false
    }
}
