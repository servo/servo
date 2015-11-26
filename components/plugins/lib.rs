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

#![feature(plugin_registrar, quote, plugin, box_syntax, rustc_private, slice_patterns)]

#[macro_use]
extern crate syntax;
#[macro_use]
extern crate rustc;

extern crate rustc_front;

extern crate tenacious;
#[cfg(feature = "clippy")]
extern crate clippy;

extern crate url;

use rustc::plugin::Registry;
use syntax::ext::base::*;
use syntax::feature_gate::AttributeType::Whitelisted;
use syntax::parse::token::intern;

// Public for documentation to show up
/// Handles the auto-deriving for `#[derive(JSTraceable)]`
pub mod jstraceable;
/// Handles the auto-deriving for `#[derive(HeapSizeOf)]`
pub mod heap_size;
pub mod lints;
/// Autogenerates implementations of Reflectable on DOM structs
pub mod reflector;
/// Utilities for writing plugins
pub mod casing;
mod url_plugin;
pub mod utils;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(intern("dom_struct"), MultiModifier(box jstraceable::expand_dom_struct));
    reg.register_syntax_extension(intern("derive_JSTraceable"), MultiDecorator(box jstraceable::expand_jstraceable));
    reg.register_syntax_extension(intern("_generate_reflector"), MultiDecorator(box reflector::expand_reflector));
    reg.register_syntax_extension(intern("derive_HeapSizeOf"), MultiDecorator(box heap_size::expand_heap_size));
    reg.register_macro("to_lower", casing::expand_lower);
    reg.register_macro("to_upper", casing::expand_upper);
    reg.register_macro("url", url_plugin::expand_url);
    reg.register_late_lint_pass(box lints::transmute_type::TransmutePass);
    reg.register_late_lint_pass(box lints::unrooted_must_root::UnrootedPass::new());
    reg.register_late_lint_pass(box lints::privatize::PrivatizePass);
    reg.register_late_lint_pass(box lints::inheritance_integrity::InheritancePass);
    reg.register_early_lint_pass(box lints::ban::BanPass);
    reg.register_late_lint_pass(box tenacious::TenaciousPass);
    reg.register_attribute("must_root".to_string(), Whitelisted);
    reg.register_attribute("servo_lang".to_string(), Whitelisted);
    reg.register_attribute("allow_unrooted_interior".to_string(), Whitelisted);
    register_clippy(reg);
}

#[cfg(feature = "clippy")]
fn register_clippy(reg: &mut Registry) {
    ::clippy::plugin_registrar(reg);
}
#[cfg(not(feature = "clippy"))]
fn register_clippy(reg: &mut Registry) {
    reg.register_late_lint_pass(box lints::str_to_string::StrToStringPass);
}
