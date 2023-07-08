/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Servo's compiler plugin/macro crate
//!
//! This crate provides the `#[unrooted_must_root_lint::must_root]` lint. This lint prevents data
//! of the marked type from being used on the stack. See the source for more details.

#![deny(unsafe_code)]
#![feature(plugin)]
#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use rustc_driver::plugin::Registry;
use rustc_hir::def_id::DefId;
use rustc_lint::LateContext;
use rustc_span::source_map::{ExpnKind, MacroKind, Span};
use rustc_span::symbol::Symbol;

#[cfg(feature = "unrooted_must_root_lint")]
mod unrooted_must_root;

#[allow(unsafe_code)] // #[no_mangle] is unsafe
#[no_mangle]
fn __rustc_plugin_registrar(reg: &mut Registry) {
    #[cfg(feature = "unrooted_must_root_lint")]
    unrooted_must_root::register(reg);
}

/// check if a DefId's path matches the given absolute type path
/// usage e.g. with
/// `match_def_path(cx, id, &["core", "option", "Option"])`
fn match_def_path(cx: &LateContext, def_id: DefId, path: &[Symbol]) -> bool {
    let def_path = cx.tcx.def_path(def_id);
    let krate = &cx.tcx.crate_name(def_path.krate);
    if krate != &path[0] {
        return false;
    }

    let path = &path[1..];
    let other = def_path.data;

    if other.len() != path.len() {
        return false;
    }

    other
        .into_iter()
        .zip(path)
        .all(|(e, p)| e.data.get_opt_name().as_ref() == Some(p))
}

fn in_derive_expn(span: Span) -> bool {
    matches!(
        span.ctxt().outer_expn_data().kind,
        ExpnKind::Macro(MacroKind::Derive, ..)
    )
}
