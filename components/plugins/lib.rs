/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Servo's compiler plugin/macro crate
//!
//! Attributes this crate provides:
//!
//!  - `#[privatize]` : Forces all fields in a struct/enum to be private
//!  - `#[derive(JSTraceable)]` : Auto-derives an implementation of `JSTraceable` for a struct in the script crate
//!  - `#[must_root]` : Prevents data of the marked type from being used on the stack.
//!                     See the lints module for more details
//!  - `#[dom_struct]` : Implies `#[privatize]`,`#[derive(JSTraceable)]`, and `#[must_root]`.
//!                       Use this for structs that correspond to a DOM type


#![feature(box_syntax, plugin, plugin_registrar, quote, rustc_private, slice_patterns)]

#![deny(unsafe_code)]

#[cfg(feature = "clippy")]
extern crate clippy_lints;
#[macro_use]
extern crate rustc;
extern crate rustc_plugin;
extern crate syntax;

use rustc_plugin::Registry;
use syntax::ext::base::*;
use syntax::feature_gate::AttributeType::Whitelisted;
use syntax::symbol::Symbol;

// Public for documentation to show up
/// Handles the auto-deriving for `#[derive(JSTraceable)]`
pub mod jstraceable;
pub mod lints;
/// Utilities for writing plugins
mod utils;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(
        Symbol::intern("dom_struct"),
        MultiModifier(box jstraceable::expand_dom_struct));

    reg.register_late_lint_pass(box lints::unrooted_must_root::UnrootedPass::new());
    reg.register_late_lint_pass(box lints::privatize::PrivatizePass);
    reg.register_late_lint_pass(box lints::inheritance_integrity::InheritancePass);
    reg.register_late_lint_pass(box lints::transmute_type::TransmutePass);
    reg.register_early_lint_pass(box lints::ban::BanPass);
    reg.register_attribute("_dom_struct_marker".to_string(), Whitelisted);
    reg.register_attribute("allow_unrooted_interior".to_string(), Whitelisted);
    reg.register_attribute("must_root".to_string(), Whitelisted);
    reg.register_attribute("privatize".to_string(), Whitelisted);
    reg.register_attribute("servo_lang".to_string(), Whitelisted);
    register_clippy(reg);
}

#[cfg(feature = "clippy")]
fn register_clippy(reg: &mut Registry) {
    ::clippy_lints::register_plugins(reg);
}
#[cfg(not(feature = "clippy"))]
fn register_clippy(_reg: &mut Registry) {
}
