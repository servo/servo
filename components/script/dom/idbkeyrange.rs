/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::gc::MutableHandleValue;
use js::rust::HandleValue;
use net_traits::indexeddb_thread::IndexedDBKeyRange;
use script_bindings::codegen::GenericBindings::IDBKeyRangeBinding::IDBKeyRangeMethods;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;

use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::import::module::SafeJSContext;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::globalscope::GlobalScope;
use crate::indexed_db::{convert_value_to_key, key_type_to_jsval};

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
    // https://www.w3.org/TR/IndexedDB-2/#dom-idbkeyrange-lower
    fn Lower(&self, cx: SafeJSContext, answer: MutableHandleValue) {
        if let Some(lower) = self.inner.lower.as_ref() {
            key_type_to_jsval(cx, lower, answer);
        }
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbkeyrange-upper
    fn Upper(&self, cx: SafeJSContext, answer: MutableHandleValue) {
        if let Some(upper) = self.inner.upper.as_ref() {
            key_type_to_jsval(cx, upper, answer);
        }
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbkeyrange-loweropen
    fn LowerOpen(&self) -> bool {
        self.inner.lower_open
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbkeyrange-upperopen
    fn UpperOpen(&self) -> bool {
        self.inner.upper_open
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbkeyrange-only
    fn Only(
        cx: SafeJSContext,
        global: &GlobalScope,
        value: HandleValue,
    ) -> Fallible<DomRoot<IDBKeyRange>> {
        let key = convert_value_to_key(cx, value, None)?;
        let inner = IndexedDBKeyRange::only(key);
        Ok(IDBKeyRange::new(global, inner, CanGc::note()))
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbkeyrange-lowerbound
    fn LowerBound(
        cx: SafeJSContext,
        global: &GlobalScope,
        lower: HandleValue,
        open: bool,
    ) -> Fallible<DomRoot<IDBKeyRange>> {
        let key = convert_value_to_key(cx, lower, None)?;
        let inner = IndexedDBKeyRange::lower_bound(key, open);
        Ok(IDBKeyRange::new(global, inner, CanGc::note()))
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbkeyrange-upperbound
    fn UpperBound(
        cx: SafeJSContext,
        global: &GlobalScope,
        upper: HandleValue,
        open: bool,
    ) -> Fallible<DomRoot<IDBKeyRange>> {
        let key = convert_value_to_key(cx, upper, None)?;
        let inner = IndexedDBKeyRange::upper_bound(key, open);
        Ok(IDBKeyRange::new(global, inner, CanGc::note()))
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbkeyrange-bound
    fn Bound(
        cx: SafeJSContext,
        global: &GlobalScope,
        lower: HandleValue,
        upper: HandleValue,
        lower_open: bool,
        upper_open: bool,
    ) -> Fallible<DomRoot<IDBKeyRange>> {
        let lower_key = convert_value_to_key(cx, lower, None)?;
        let upper_key = convert_value_to_key(cx, upper, None)?;
        let inner =
            IndexedDBKeyRange::new(Some(lower_key), Some(upper_key), lower_open, upper_open);
        Ok(IDBKeyRange::new(global, inner, CanGc::note()))
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbkeyrange-_includes
    fn Includes(&self, cx: SafeJSContext, value: HandleValue) -> Fallible<bool> {
        let key = convert_value_to_key(cx, value, None)?;
        if self.inner.contains(&key) {
            return Ok(true);
        }
        Ok(false)
    }
}
