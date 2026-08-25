/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use js::conversions::ToJSValConvertible;
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::MutableHandleValue;
use script_bindings::cell::DomRefCell;

use crate::dom::bindings::utils::to_frozen_array;

#[derive(JSTraceable)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct CachedFrozenArray {
    frozen_value: DomRefCell<Option<Heap<JSVal>>>,
}

impl CachedFrozenArray {
    pub(crate) fn new() -> CachedFrozenArray {
        CachedFrozenArray {
            frozen_value: DomRefCell::new(None),
        }
    }

    pub(crate) fn get_or_init<F: FnOnce() -> Vec<T>, T: ToJSValConvertible>(
        &self,
        cx: &mut JSContext,
        f: F,
        mut retval: MutableHandleValue,
    ) {
        if let Some(inner) = &*self.frozen_value.borrow() {
            retval.set(inner.get());
            return;
        }

        let array = f();
        to_frozen_array(cx, array.as_slice(), retval.reborrow());

        // Safety: need to create the Heap value in its final memory location before setting it.
        *self.frozen_value.safe_borrow_mut(cx.no_gc()) = Some(Heap::default());
        self.frozen_value
            .borrow()
            .as_ref()
            .unwrap()
            .set(retval.get());
    }

    pub(crate) fn clear(&self) {
        *self.frozen_value.borrow_mut() = None;
    }
}
