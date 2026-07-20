/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg_attr(crown, feature(register_tool))]
// Register the linter `crown`, which is the Servo-specific linter for the script crate.
#![cfg_attr(crown, register_tool(crown))]

pub mod gpuadapterinfo;
pub mod gpubufferusage;
pub mod gpucommandbuffer;
pub mod gpucompilationinfo;
pub mod gpucompilationmessage;
pub mod gpudevicelostinfo;
pub mod gpumapmode;

pub(crate) use js::gc::Traceable as JSTraceable;
pub(crate) use jstraceable_derive::JSTraceable;
pub(crate) use script_bindings::reflector::{DomObject, MutDomObject, Reflector};

pub(crate) use crate::dom::bindings::inheritance::HasParent;

// Reexports
pub(crate) mod dom {
    pub(crate) mod types {}
    pub(crate) mod bindings {
        pub(crate) use script_bindings::*;
    }
}

/// Generated JS-Rust bindings.
#[allow(missing_docs, non_snake_case)]
pub(crate) mod codegen {
    #[expect(unused)]
    pub(crate) mod Bindings {
        use std::ptr;

        use js::context::JSContext;
        use js::gc::HandleObject;
        pub(crate) use script_bindings::DomTypes;
        use script_bindings::conversions::IDLInterface;
        use script_bindings::reflector::DomObjectWrap;
        pub(crate) use script_bindings::reflector::Reflector;
        use script_bindings::root::{Dom, DomRoot, Root};
        use script_bindings::utils::DOMClass;

        use crate::gpuadapterinfo::GPUAdapterInfo;
        use crate::gpubufferusage::GPUBufferUsage;
        use crate::gpucommandbuffer::GPUCommandBuffer;
        use crate::gpucompilationinfo::GPUCompilationInfo;
        use crate::gpucompilationmessage::GPUCompilationMessage;
        use crate::gpudevicelostinfo::GPUDeviceLostInfo;
        use crate::gpumapmode::GPUMapMode;
        include!(concat!(
            env!("OUT_DIR"),
            "/ConcreteBindings/WebGPUBinding.rs"
        ));
    }
}
