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

#![feature(plugin_registrar, quote, plugin, box_syntax, rustc_private)]

#[macro_use]
extern crate syntax;
#[macro_use]
extern crate rustc;

extern crate tenacious;
#[cfg(feature = "clippy")]
extern crate clippy;

use rustc::lint::LintPassObject;
use rustc::plugin::Registry;
use syntax::ext::base::*;

use syntax::parse::token::intern;
use syntax::feature_gate::AttributeType::Whitelisted;

// Public for documentation to show up
/// Handles the auto-deriving for `#[derive(JSTraceable)]`
pub mod jstraceable;
/// Handles the auto-deriving for `#[derive(HeapSizeOf)]`
pub mod heap_size;
/// Autogenerates implementations of Reflectable on DOM structs
pub mod reflector;
pub mod lints;
/// Utilities for writing plugins
pub mod utils;
pub mod casing;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(intern("dom_struct"), MultiModifier(box jstraceable::expand_dom_struct));
    reg.register_syntax_extension(intern("derive_JSTraceable"), MultiDecorator(box jstraceable::expand_jstraceable));
    reg.register_syntax_extension(intern("_generate_reflector"), MultiDecorator(box reflector::expand_reflector));
    reg.register_syntax_extension(intern("derive_HeapSizeOf"), MultiDecorator(box heap_size::expand_heap_size));
    reg.register_macro("to_lower", casing::expand_lower);
    reg.register_macro("to_upper", casing::expand_upper);
    reg.register_lint_pass(box lints::transmute_type::TransmutePass as LintPassObject);
    reg.register_lint_pass(box lints::unrooted_must_root::UnrootedPass::new() as LintPassObject);
    reg.register_lint_pass(box lints::privatize::PrivatizePass as LintPassObject);
    reg.register_lint_pass(box lints::inheritance_integrity::InheritancePass as LintPassObject);
    reg.register_lint_pass(box lints::ban::BanPass as LintPassObject);
    reg.register_lint_pass(box tenacious::TenaciousPass as LintPassObject);
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
    reg.register_lint_pass(box lints::str_to_string::StrToStringPass as LintPassObject);
}
