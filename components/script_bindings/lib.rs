/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg_attr(crown, feature(register_tool))]
// Register the linter `crown`, which is the Servo-specific linter for the script
// crate. Issue a warning if `crown` is not being used to compile, but not when
// building rustdoc or running clippy.
#![cfg_attr(crown, register_tool(crown))]
#![cfg_attr(any(doc, clippy), allow(unknown_lints))]
#![deny(crown_is_not_used)]

#[macro_use]
extern crate jstraceable_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate malloc_size_of_derive;

pub mod callback;
pub mod constant;
pub mod conversions;
pub mod error;
pub mod inheritance;
pub mod iterable;
pub mod like;
pub mod reflector;
pub mod root;
pub mod script_runtime;
pub mod str;
pub mod trace;
pub mod utils;
pub mod weakref;

#[allow(non_snake_case)]
pub mod codegen {
    pub mod Globals {
        include!(concat!(env!("OUT_DIR"), "/Globals.rs"));
    }
    #[allow(dead_code, unused_imports, clippy::enum_variant_names)]
    pub mod InheritTypes {
        include!(concat!(env!("OUT_DIR"), "/InheritTypes.rs"));
    }
    #[allow(clippy::upper_case_acronyms)]
    pub mod PrototypeList {
        include!(concat!(env!("OUT_DIR"), "/PrototypeList.rs"));
    }
}

// These trait exports are public, because they are used in the DOM bindings.
// Since they are used in derive macros,
// it is useful that they are accessible at the root of the crate.
pub(crate) use js::gc::Traceable as JSTraceable;
