/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Servo's compiler plugin/macro crate
//!
//! Attributes this crate provides:
//!
//!  - `#[privatize]` : Forces all fields in a struct/enum to be private
//!  - `#[jstraceable]` : Auto-derives an implementation of `JSTraceable` for a struct in the script crate
//!  - `#[must_root]` : Prevents data of the marked type from being used on the stack. See the lints module for more details
//!  - `#[dom_struct]` : Implies `#[privatize]`,`#[jstraceable]`, and `#[must_root]`.
//!     Use this for structs that correspond to a DOM type

#![feature(macro_rules, plugin_registrar, quote, phase)]

#![deny(unused_imports)]
#![deny(unused_variables)]

#[phase(plugin,link)]
extern crate syntax;
#[phase(plugin, link)]
extern crate rustc;
#[cfg(test)]
extern crate sync;

use rustc::lint::LintPassObject;
use rustc::plugin::Registry;
use syntax::ext::base::{Decorator, Modifier};

use syntax::parse::token::intern;

// Public for documentation to show up
/// Handles the auto-deriving for `#[jstraceable]`
pub mod jstraceable;
pub mod lints;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(intern("dom_struct"), Modifier(box jstraceable::expand_dom_struct));
    reg.register_syntax_extension(intern("jstraceable"), Decorator(box jstraceable::expand_jstraceable));
    reg.register_lint_pass(box lints::TransmutePass as LintPassObject);
    reg.register_lint_pass(box lints::UnrootedPass as LintPassObject);
    reg.register_lint_pass(box lints::PrivatizePass as LintPassObject);
}

