/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg_attr(crown, feature(register_tool))]
// Register the linter `crown`, which is the Servo-specific linter for the script crate.
#![cfg_attr(crown, register_tool(crown))]

pub mod crypto;
pub mod traits;

pub(crate) use js::gc::Traceable as JSTraceable;
pub(crate) use script_bindings::inheritance::HasParent;
pub(crate) use script_bindings::reflector::{DomObject, MutDomObject, Reflector};

// Reexports
pub(crate) mod dom {
    pub(crate) mod types {}
    pub(crate) mod bindings {
        pub(crate) use script_bindings::*;
    }
}

/// Generated JS-Rust bindings.
#[expect(non_snake_case)]
pub(crate) mod codegen {
    #[expect(unused)]
    pub(crate) mod Bindings {
        use std::ptr;

        use script_bindings::DomTypes;
        use script_bindings::codegen::PrototypeList;
        use script_bindings::conversions::IDLInterface;
        use script_bindings::reflector::Reflector;
        use script_bindings::utils::DOMClass;

        use crate::crypto::Crypto;
        use crate::traits::Equivalence;
        include!(concat!(
            env!("OUT_DIR"),
            "/ConcreteBindings/CryptoBinding.rs"
        ));
    }
}
