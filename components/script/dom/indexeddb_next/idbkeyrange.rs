/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::gc::MutableHandleValue;
use js::rust::HandleValue;
use script_bindings::codegen::GenericBindings::IDBKeyRangeBinding::IDBKeyRangeMethods;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;

use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::import::module::SafeJSContext;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::globalscope::GlobalScope;

/// An "object" implementing the spec’s IDBKeyRange interface:
/// <https://w3c.github.io/IndexedDB/#keyrange>.
///
/// The IDBKeyRange interface represents a key range:
/// <https://w3c.github.io/IndexedDB/#range-construct>.
///
/// A key range is a continuous interval over the data type used for keys.
#[dom_struct]
pub struct IDBKeyRange {
    reflector_: Reflector,
}

impl IDBKeyRange {
    pub fn _new_inherited() -> Self {
        IDBKeyRange {
            reflector_: Reflector::new(),
        }
    }

    pub fn _new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<Self> {
        reflect_dom_object(Box::new(IDBKeyRange::_new_inherited()), global, can_gc)
    }
}

impl IDBKeyRangeMethods<crate::DomTypeHolder> for IDBKeyRange {
    /// <https://w3c.github.io/IndexedDB/#dom-idbkeyrange-lower>
    fn Lower(&self, _cx: SafeJSContext, _can_gc: CanGc, _answer: MutableHandleValue) {}

    /// <https://w3c.github.io/IndexedDB/#dom-idbkeyrange-upper>>
    fn Upper(&self, _cx: SafeJSContext, _can_gc: CanGc, _answer: MutableHandleValue) {}

    /// <https://w3c.github.io/IndexedDB/#dom-idbkeyrange-loweropen>
    fn LowerOpen(&self) -> bool {
        false
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbkeyrange-upperopen>
    fn UpperOpen(&self) -> bool {
        false
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbkeyrange-only>
    fn Only(
        _cx: SafeJSContext,
        _global: &GlobalScope,
        _value: HandleValue,
    ) -> Fallible<DomRoot<IDBKeyRange>> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbkeyrange-lowerbound>
    fn LowerBound(
        _cx: SafeJSContext,
        _global: &GlobalScope,
        _lower: HandleValue,
        _open: bool,
    ) -> Fallible<DomRoot<IDBKeyRange>> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbkeyrange-upperopen>
    fn UpperBound(
        _cx: SafeJSContext,
        _global: &GlobalScope,
        _upper: HandleValue,
        _open: bool,
    ) -> Fallible<DomRoot<IDBKeyRange>> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbkeyrange-bound>
    fn Bound(
        _cx: SafeJSContext,
        _global: &GlobalScope,
        _lower: HandleValue,
        _upper: HandleValue,
        _lower_open: bool,
        _upper_open: bool,
    ) -> Fallible<DomRoot<IDBKeyRange>> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbkeyrange-includes>
    fn Includes(&self, _cx: SafeJSContext, _value: HandleValue) -> Fallible<bool> {
        Err(Error::NotSupported)
    }
}
