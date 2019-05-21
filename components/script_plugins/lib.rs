/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Servo's compiler plugin/macro crate
//!
//! Attributes this crate provides:
//!
//!  - `#[derive(DenyPublicFields)]` : Forces all fields in a struct/enum to be private
//!  - `#[derive(JSTraceable)]` : Auto-derives an implementation of `JSTraceable` for a struct in the script crate
//!  - `#[must_root]` : Prevents data of the marked type from being used on the stack.
//!                     See the lints module for more details
//!  - `#[dom_struct]` : Implies #[derive(JSTraceable, DenyPublicFields)]`, and `#[must_root]`.
//!                       Use this for structs that correspond to a DOM type

#![deny(unsafe_code)]
#![feature(plugin)]
#![feature(plugin_registrar)]
#![feature(rustc_private)]

#[cfg(feature = "unrooted_must_root_lint")]
#[macro_use]
extern crate rustc;

extern crate rustc_plugin;
extern crate syntax;

extern crate weedle;

use rustc_plugin::Registry;
use syntax::feature_gate::AttributeType::Whitelisted;
use syntax::symbol::Symbol;

#[cfg(feature = "unrooted_must_root_lint")]
mod unrooted_must_root;

#[cfg(feature = "webidl_lint")]
mod webidl_must_inherit;

/// Utilities for writing plugins
#[cfg(feature = "unrooted_must_root_lint")]
mod utils;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    let symbols = crate::Symbols::new();

    #[cfg(feature = "unrooted_must_root_lint")]
    reg.register_late_lint_pass(Box::new(unrooted_must_root::UnrootedPass::new(
        symbols.clone(),
    )));

    #[cfg(feature = "webidl_lint")]
    reg.register_late_lint_pass(Box::new(webidl_must_inherit::WebIdlPass::new(
        symbols.clone(),
    )));

    reg.register_attribute(symbols.allow_unrooted_interior, Whitelisted);
    reg.register_attribute(symbols.allow_unrooted_in_rc, Whitelisted);
    reg.register_attribute(symbols.must_root, Whitelisted);
    reg.register_attribute(symbols.webidl, Whitelisted);
}

macro_rules! symbols {
    ($($s: ident)+) => {
        #[derive(Clone)]
        #[allow(non_snake_case)]
        struct Symbols {
            $( $s: Symbol, )+
        }

        impl Symbols {
            fn new() -> Self {
                Symbols {
                    $( $s: Symbol::intern(stringify!($s)), )+
                }
            }
        }
    }
}

symbols! {
    allow_unrooted_interior
    allow_unrooted_in_rc
    must_root
    webidl
    alloc
    rc
    Rc
    cell
    Ref
    RefMut
    slice
    Iter
    IterMut
    collections
    hash
    map
    set
    Entry
    OccupiedEntry
    VacantEntry
}
