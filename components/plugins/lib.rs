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

#![feature(plugin_registrar, quote, plugin, box_syntax)]

#![deny(unused_imports)]
#![deny(unused_variables)]
#![allow(missing_copy_implementations)]

#[plugin]
#[macro_use]
extern crate syntax;
#[plugin]
#[macro_use]
extern crate rustc;

use rustc::lint::LintPassObject;
use rustc::plugin::Registry;
use syntax::ext::base::{Decorator, Modifier};

use syntax::parse::token::intern;

// Public for documentation to show up
/// Handles the auto-deriving for `#[jstraceable]`
pub mod jstraceable;
/// Autogenerates implementations of Reflectable on DOM structs
pub mod reflector;
pub mod lints;
/// Utilities for writing plugins
pub mod utils;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(intern("dom_struct"), Modifier(Box::new(jstraceable::expand_dom_struct)));
    reg.register_syntax_extension(intern("jstraceable"), Decorator(Box::new(jstraceable::expand_jstraceable)));
    reg.register_syntax_extension(intern("_generate_reflector"), Decorator(Box::new(reflector::expand_reflector)));
    reg.register_lint_pass(Box::new(lints::transmute_type::TransmutePass) as LintPassObject);
    reg.register_lint_pass(Box::new(lints::unrooted_must_root::UnrootedPass) as LintPassObject);
    reg.register_lint_pass(Box::new(lints::privatize::PrivatizePass) as LintPassObject);
    reg.register_lint_pass(Box::new(lints::inheritance_integrity::InheritancePass) as LintPassObject);
    reg.register_lint_pass(Box::new(lints::str_to_string::StrToStringPass) as LintPassObject);
    reg.register_lint_pass(Box::new(lints::ban::BanPass) as LintPassObject);
}


#[macro_export]
macro_rules! match_ignore_ascii_case {
    ( $value: expr=> $( $string: expr => $result: expr ),+ _ => $fallback: expr, ) => {
        match_ignore_ascii_case! { $value:
            $( $string => $result ),+
            _ => $fallback
        }
    };
    ( $value: expr=> $( $string: expr => $result: expr ),+ _ => $fallback: expr ) => {
        {
            use std::ascii::AsciiExt;
            match $value.as_slice() {
                $( s if s.eq_ignore_ascii_case($string) => $result, )+
                _ => $fallback
            }
        }
    };
}
