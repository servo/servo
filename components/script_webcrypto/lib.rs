/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg_attr(crown, feature(register_tool))]
// Register the linter `crown`, which is the Servo-specific linter for the script crate.
#![cfg_attr(crown, register_tool(crown))]

pub mod crypto;
pub mod cryptokey;
pub mod subtlecrypto;
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
        pub(crate) mod import {
            pub(crate) use std::ptr;

            pub(crate) use script_bindings::DomTypes;
            pub(crate) use script_bindings::codegen::PrototypeList;
            pub(crate) use script_bindings::conversions::IDLInterface;
            pub(crate) use script_bindings::reflector::Reflector;
            pub(crate) use script_bindings::utils::DOMClass;

            pub(crate) use crate::traits::Equivalence;
        }

        pub(crate) mod CryptoBinding {
            use crate::codegen::Bindings::import::*;
            use crate::crypto::Crypto;
            include!(concat!(
                env!("OUT_DIR"),
                "/ConcreteBindings/CryptoBinding.rs"
            ));
        }
        pub(crate) mod CryptoKeyBinding {
            use crate::codegen::Bindings::import::*;
            use crate::cryptokey::CryptoKey;
            include!(concat!(
                env!("OUT_DIR"),
                "/ConcreteBindings/CryptoKeyBinding.rs"
            ));
        }
        pub(crate) mod SubtleCryptoBinding {
            use crate::codegen::Bindings::import::*;
            use crate::subtlecrypto::SubtleCrypto;
            include!(concat!(
                env!("OUT_DIR"),
                "/ConcreteBindings/SubtleCryptoBinding.rs"
            ));
        }
    }
}
