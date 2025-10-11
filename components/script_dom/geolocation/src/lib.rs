/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg_attr(crown, feature(register_tool))]
#![deny(unsafe_code)]
#![doc = "The geolocation crate contains geolocation DOM implementations."]
// Register the linter `crown`, which is the Servo-specific linter for the script crate.
#![cfg_attr(crown, register_tool(crown))]

mod geolocationcoordinates;

mod codegen {
    pub(crate) mod DomTypeHolder {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/DomTypeHolder.rs"));
    }
}
pub(crate) use crate::codegen::DomTypeHolder::DomTypeHolder;
pub(crate) use js::gc::Traceable as JSTraceable;
pub(crate) use script_bindings::reflector::{DomObject, Reflector, MutDomObject};
pub(crate) use script_bindings::inheritance::HasParent;
pub(crate) use script_bindings::DomTypes;
