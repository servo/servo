/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::gc::MutableHandleValue;
use js::rust::HandleValue;
use script_bindings::codegen::GenericBindings::IDBKeyRangeBinding::IDBKeyRangeMethods;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use storage_traits::indexeddb::IndexedDBKeyRange;

use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::globalscope::GlobalScope;
use crate::indexeddb::{convert_value_to_key, key_type_to_jsval};

#[dom_struct]
pub struct IDBKeyRange {
    reflector_: Reflector,
    #[no_trace]
    inner: IndexedDBKeyRange,
}

impl IDBKeyRange {
    pub fn new_inherited(inner: IndexedDBKeyRange) -> Self {
        IDBKeyRange {
            reflector_: Reflector::new(),
            inner,
        }
    }

    pub fn new(global: &GlobalScope, inner: IndexedDBKeyRange, can_gc: CanGc) -> DomRoot<Self> {
        reflect_dom_object(Box::new(IDBKeyRange::new_inherited(inner)), global, can_gc)
    }

    pub fn inner(&self) -> &IndexedDBKeyRange {
        &self.inner
    }
}

impl IDBKeyRangeMethods<crate::DomTypeHolder> for IDBKeyRange {
    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbkeyrange-lower>
    fn Lower(&self, cx: &mut JSContext, answer: MutableHandleValue) {
        if let Some(lower) = self.inner.lower.as_ref() {
            key_type_to_jsval(cx, lower, answer);
        }
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbkeyrange-upper>
    fn Upper(&self, cx: &mut JSContext, answer: MutableHandleValue) {
        if let Some(upper) = self.inner.upper.as_ref() {
            key_type_to_jsval(cx, upper, answer);
        }
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbkeyrange-loweropen>
    fn LowerOpen(&self) -> bool {
        self.inner.lower_open
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbkeyrange-upperopen>
    fn UpperOpen(&self) -> bool {
        self.inner.upper_open
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbkeyrange-only>
    fn Only(
        cx: &mut JSContext,
        global: &GlobalScope,
        value: HandleValue,
    ) -> Fallible<DomRoot<IDBKeyRange>> {
        // Step 1. Let key be the result of converting a value to a key with value. Rethrow any
        // exceptions.
        // Step 2. If key is "invalid value" or "invalid type", throw a "DataError"
        // DOMException.
        let key = convert_value_to_key(cx, value, None)?.into_result()?;

        // Step 3. Create and return a new key range containing only key.
        let inner = IndexedDBKeyRange::only(key);
        Ok(IDBKeyRange::new(global, inner, CanGc::from_cx(cx)))
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbkeyrange-lowerbound>
    fn LowerBound(
        cx: &mut JSContext,
        global: &GlobalScope,
        lower: HandleValue,
        open: bool,
    ) -> Fallible<DomRoot<IDBKeyRange>> {
        // Step 1. Let lowerKey be the result of converting a value to a key with lower. Rethrow
        // any exceptions.
        // Step 2. If lowerKey is invalid, throw a "DataError" DOMException.
        let lower_key = convert_value_to_key(cx, lower, None)?.into_result()?;

        // Step 3. Create and return a new key range with lower bound set to lowerKey, lower open
        // flag set to open, upper bound set to null, and upper open flag set to true.
        let inner = IndexedDBKeyRange::lower_bound(lower_key, open);
        Ok(IDBKeyRange::new(global, inner, CanGc::from_cx(cx)))
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbkeyrange-upperbound>
    fn UpperBound(
        cx: &mut JSContext,
        global: &GlobalScope,
        upper: HandleValue,
        open: bool,
    ) -> Fallible<DomRoot<IDBKeyRange>> {
        // Step 1. Let upperKey be the result of converting a value to a key with upper. Rethrow
        // any exceptions.
        // Step 2. If upperKey is "invalid value" or "invalid type", throw a "DataError"
        // DOMException.
        let upper_key = convert_value_to_key(cx, upper, None)?.into_result()?;

        // Step 3. Create and return a new key range with lower bound set to null, lower open flag
        // set to true, upper bound set to upperKey, and upper open flag set to open.
        let inner = IndexedDBKeyRange::upper_bound(upper_key, open);
        Ok(IDBKeyRange::new(global, inner, CanGc::from_cx(cx)))
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbkeyrange-bound>
    fn Bound(
        cx: &mut JSContext,
        global: &GlobalScope,
        lower: HandleValue,
        upper: HandleValue,
        lower_open: bool,
        upper_open: bool,
    ) -> Fallible<DomRoot<IDBKeyRange>> {
        // Step 1. Let lowerKey be the result of converting a value to a key with lower. Rethrow
        // any exceptions.
        // Step 2. If lowerKey is "invalid value" or "invalid type", throw a "DataError"
        // DOMException.
        let lower_key = convert_value_to_key(cx, lower, None)?.into_result()?;

        // Step 3. Let upperKey be the result of converting a value to a key with upper. Rethrow
        // any exceptions.
        // Step 4. If upperKey is "invalid value" or "invalid type", throw a "DataError"
        // DOMException.
        let upper_key = convert_value_to_key(cx, upper, None)?.into_result()?;

        // Step 5. If lowerKey is greater than upperKey, throw a "DataError" DOMException.
        if lower_key > upper_key {
            return Err(Error::Data(None));
        }

        // Step 6. Create and return a new key range with lower bound set to lowerKey, lower open
        // flag set to lowerOpen, upper bound set to upperKey and upper open flag set to upperOpen.
        let inner =
            IndexedDBKeyRange::new(Some(lower_key), Some(upper_key), lower_open, upper_open);
        Ok(IDBKeyRange::new(global, inner, CanGc::from_cx(cx)))
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbkeyrange-_includes>
    fn Includes(&self, cx: &mut JSContext, key: HandleValue) -> Fallible<bool> {
        // Step 1. Let k be the result of converting a value to a key with key. Rethrow any
        // exceptions.
        // Step 2. If k is "invalid value" or "invalid type", throw a "DataError" DOMException.
        let k = convert_value_to_key(cx, key, None)?.into_result()?;

        // Step 3. Return true if k is in this range, and false otherwise.
        Ok(self.inner.contains(&k))
    }
}
