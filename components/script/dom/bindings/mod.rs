/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The code to expose the DOM to JavaScript through IDL bindings.

#![allow(unsafe_blocks)]
#![deny(missing_docs, non_snake_case)]

pub mod cell;
pub mod global;
pub mod js;
pub mod refcounted;
pub mod utils;
pub mod callback;
pub mod error;
pub mod conversions;
pub mod proxyhandler;
pub mod str;
pub mod structuredclone;
pub mod trace;

/// Generated JS-Rust bindings.
#[allow(missing_docs, non_snake_case)]
pub mod codegen {
    #[allow(unrooted_must_root)]
    pub mod Bindings;
    pub mod InterfaceTypes;
    pub mod InheritTypes;
    pub mod PrototypeList;
    pub mod RegisterBindings;
    pub mod UnionTypes;
}

