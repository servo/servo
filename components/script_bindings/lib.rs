/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg_attr(crown, feature(register_tool))]
// Register the linter `crown`, which is the Servo-specific linter for the script crate.
#![cfg_attr(crown, register_tool(crown))]

#[macro_use]
extern crate js;
#[macro_use]
extern crate jstraceable_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate malloc_size_of_derive;

pub mod callback;
mod constant;
mod constructor;
pub mod conversions;
pub mod domstring;
pub mod error;
mod finalize;
mod guard;
mod import;
pub mod inheritance;
pub mod interface;
pub mod interfaces;
pub mod iterable;
pub mod like;
mod lock;
mod mem;
mod namespace;
pub mod num;
pub mod principals;
pub mod proxyhandler;
pub mod realms;
pub mod record;
pub mod reflector;
pub mod root;
pub mod script_runtime;
pub mod settings_stack;
pub mod str;
pub mod structuredclone;
pub mod trace;
pub mod utils;
pub mod weakref;

#[allow(non_snake_case, unsafe_op_in_unsafe_fn)]
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
    pub(crate) mod DomTypes {
        include!(concat!(env!("OUT_DIR"), "/DomTypes.rs"));
    }
    #[allow(
        dead_code,
        clippy::extra_unused_type_parameters,
        clippy::missing_safety_doc,
        clippy::result_unit_err
    )]
    pub mod GenericBindings {
        include!(concat!(env!("OUT_DIR"), "/Bindings/mod.rs"));
    }
    #[allow(
        non_camel_case_types,
        unused_imports,
        unused_variables,
        clippy::large_enum_variant,
        clippy::upper_case_acronyms,
        clippy::enum_variant_names
    )]
    pub mod GenericUnionTypes {
        include!(concat!(env!("OUT_DIR"), "/GenericUnionTypes.rs"));
    }
    pub mod RegisterBindings {
        include!(concat!(env!("OUT_DIR"), "/RegisterBindings.rs"));
    }
}

// These trait exports are public, because they are used in the DOM bindings.
// Since they are used in derive macros,
// it is useful that they are accessible at the root of the crate.
pub(crate) use js::gc::Traceable as JSTraceable;

pub use crate::codegen::DomTypes::DomTypes;
pub(crate) use crate::reflector::{DomObject, MutDomObject, Reflector};
pub(crate) use crate::trace::CustomTraceable;
