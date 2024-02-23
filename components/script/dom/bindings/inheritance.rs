/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `Castable` trait.

use std::mem;

pub use crate::dom::bindings::codegen::InheritTypes::*;
use crate::dom::bindings::conversions::{get_dom_class, DerivedFrom, IDLInterface};
use crate::dom::bindings::reflector::DomObject;
use crate::script_runtime::runtime_is_alive;

/// A trait to hold the cast functions of IDL interfaces that either derive
/// or are derived from other interfaces.
pub trait Castable: IDLInterface + DomObject + Sized {
    /// Check whether a DOM object implements one of its deriving interfaces.
    fn is<T>(&self) -> bool
    where
        T: DerivedFrom<Self>,
    {
        // This is a weird place for this check to live, but it should catch any
        // attempts to interact with DOM objects from Drop implementations that run
        // as a result of the runtime shutting down and finalizing all remaining objects.
        debug_assert!(
            runtime_is_alive(),
            "Attempting to interact with DOM objects after JS runtime has shut down."
        );

        let class = unsafe { get_dom_class(self.reflector().get_jsobject().get()).unwrap() };
        T::derives(class)
    }

    /// Cast a DOM object upwards to one of the interfaces it derives from.
    fn upcast<T>(&self) -> &T
    where
        T: Castable,
        Self: DerivedFrom<T>,
    {
        unsafe { mem::transmute(self) }
    }

    /// Cast a DOM object downwards to one of the interfaces it might implement.
    fn downcast<T>(&self) -> Option<&T>
    where
        T: DerivedFrom<Self>,
    {
        if self.is::<T>() {
            Some(unsafe { mem::transmute(self) })
        } else {
            None
        }
    }
}

pub trait HasParent {
    type Parent;
    fn as_parent(&self) -> &Self::Parent;
}
