/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The code to expose the DOM to JavaScript through IDL bindings.
//!
//! Exposing a DOM object to JavaScript
//! ===================================
//!
//! As [explained earlier](../index.html#a-dom-object-and-its-reflector), the
//! implementation of an interface `Foo` involves two objects: the DOM object
//! (implemented in Rust) and the reflector (a `JSObject`).
//!
//! In order to expose the interface's members to the web, properties
//! corresponding to the operations and attributes are defined on an object in
//! the reflector's prototype chain or on the reflector itself.
//!
//! Typically, these properties are either value properties whose value is a
//! function (for operations) or accessor properties that have a getter and
//! optionally a setter function (for attributes, depending on whether they are
//! marked `readonly`).
//!
//! All these JavaScript functions are set up such that, when they're called,
//! they call a Rust function in the generated glue code. This glue code does
//! some sanity checks and [argument conversions](conversions/index.html), and
//! calls into API implementation for the DOM object.

#![allow(unsafe_code)]
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

