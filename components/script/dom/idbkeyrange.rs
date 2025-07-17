/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::gc::MutableHandleValue;
use js::rust::HandleValue;

use net_traits::indexeddb_thread::IndexedDBKeyRange;

use script_bindings::codegen::GenericBindings::IDBKeyRangeBinding::IDBKeyRangeMethods;
use script_bindings::script_runtime::CanGc;
use script_bindings::root::DomRoot;

use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::import::module::SafeJSContext;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::globalscope::GlobalScope;
use crate::indexed_db::{key_type_to_jsval, convert_value_to_key};

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
}

impl IDBKeyRangeMethods<crate::DomTypeHolder> for IDBKeyRange {
    fn Lower(&self, cx: SafeJSContext, answer: MutableHandleValue) {
        if let Some(lower) = self.inner.lower.as_ref() {
            key_type_to_jsval(cx, lower, answer);
        }
    }

    fn Upper(&self, cx: SafeJSContext, answer: MutableHandleValue) {
        if let Some(upper) = self.inner.upper.as_ref() {
            key_type_to_jsval(cx, upper, answer);
        }
    }

    fn LowerOpen(&self) -> bool { self.inner.lower_open }
    fn UpperOpen(&self) -> bool { self.inner.upper_open }

    fn Only(cx: SafeJSContext, global: &GlobalScope, value: HandleValue) -> Fallible<DomRoot<IDBKeyRange>> {
        let key = convert_value_to_key(cx, value, None)?;
        let inner = IndexedDBKeyRange::only(key);
        Ok(IDBKeyRange::new(global, inner, CanGc::note()))
    }

    fn LowerBound(cx: SafeJSContext, global: &GlobalScope, lower: HandleValue, open: bool) -> Fallible<DomRoot<IDBKeyRange>> {
        let key = convert_value_to_key(cx, lower, None)?;
        let inner = IndexedDBKeyRange::lower_bound(key, open);
        Ok(IDBKeyRange::new(global, inner, CanGc::note()))
    }

    fn UpperBound(cx: SafeJSContext, global: &GlobalScope, upper: HandleValue, open: bool) -> Fallible<DomRoot<IDBKeyRange>> {
        let key = convert_value_to_key(cx, upper, None)?;
        let inner = IndexedDBKeyRange::upper_bound(key, open);
        Ok(IDBKeyRange::new(global, inner, CanGc::note()))
    }

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
        let inner = IndexedDBKeyRange::new(Some(lower_key), Some(upper_key), lower_open, upper_open);
        Ok(IDBKeyRange::new(global, inner, CanGc::note()))
    }
}
