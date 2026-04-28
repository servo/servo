/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::{conversions::ToJSValConvertible, rooted};
use js::jsapi::{HandleObject, Heap, JS_FreezeObject};
use js::jsval::JSVal;
use js::rust::MutableHandleValue;
use jstraceable_derive::{JSTraceable, JSTraceable2};
use script_bindings::script_runtime::{CanGc, JSContext};

use crate::cell::DomRefCell;

/// Returns a JSVal representing the frozen JavaScript array
pub(crate) fn to_frozen_array<T: ToJSValConvertible>(
    convertibles: &[T],
    cx: JSContext,
    mut rval: MutableHandleValue,
    can_gc: CanGc,
) {
    script_bindings::conversions::SafeToJSValConvertible::safe_to_jsval(
        convertibles,
        cx,
        rval.reborrow(),
        can_gc,
    );

    rooted!(in(*cx) let obj = rval.to_object());
    unsafe { JS_FreezeObject(*cx, HandleObject::from(obj.handle())) };
}


#[derive(JSTraceable2)]
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
        f: F,
        cx: JSContext,
        mut retval: MutableHandleValue,
        can_gc: CanGc,
    ) {
        if let Some(inner) = &*self.frozen_value.borrow() {
            retval.set(inner.get());
            return;
        }

        let array = f();
        to_frozen_array(array.as_slice(), cx, retval.reborrow(), can_gc);

        // Safety: need to create the Heap value in its final memory location before setting it.
        *self.frozen_value.borrow_mut() = Some(Heap::default());
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
