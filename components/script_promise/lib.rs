/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) use self::promise::*;
#[allow(clippy::module_inception, reason = "The interface name is Promise")]
pub(crate) mod promise;
pub(crate) mod promisenativehandler;
use js::conversions::{ConversionResult, FromJSValConvertible};
pub(crate) use js::gc::Traceable as JSTraceable;
use js::jsapi::{HandleValue, JSContext};
pub(crate) use jstraceable_derive::JSTraceable;
use script_bindings::cformat;
pub(crate) use script_bindings::inheritance::HasParent;
pub(crate) use script_bindings::reflector::{DomObject, MutDomObject, Reflector};

pub(crate) mod dom {
    pub(crate) mod types {}
    pub(crate) mod bindings {
        pub(crate) use script_bindings::*;
        pub(crate) mod import {
            pub(crate) mod module {
                pub(crate) use std::ptr;

                pub(crate) use script_bindings::codegen::PrototypeList;
                pub(crate) use script_bindings::conversions::IDLInterface;
                pub(crate) use script_bindings::utils::DOMClass;

                pub(crate) use crate::promise::Promise;
                pub(crate) use crate::promisenativehandler::PromiseNativeHandler;
            }
        }
    }
}

/// Generated JS-Rust bindings.
#[allow(missing_docs, non_snake_case)]
pub(crate) mod codegen {
    pub(crate) mod IDLInterface {
        #[expect(unused)]
        include!(concat!(
            env!("OUT_DIR"),
            "/PromiseIDLInterfaceBindings/PromiseBinding.rs"
        ));
        include!(concat!(
            env!("OUT_DIR"),
            "/PromiseIDLInterfaceBindings/PromiseNativeHandlerBinding.rs"
        ));
    }
    pub(crate) mod ConcreteInheritTypes {
        pub(crate) use crate::promisenativehandler::PromiseNativeHandler;
        include!(concat!(env!("OUT_DIR"), "/PromiseConcreteInheritTypes.rs"));
    }
}

pub use promise::{
    EnqueueWaitForallMicrotask, Promise, WaitForAllSuccessStepsMicrotask, wait_for_all_promise,
};
pub use promisenativehandler::{Callback, PromiseNativeHandler};
