/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic finalizer implementations for DOM binding implementations.

use std::any::type_name;
use std::{mem, ptr};

use js::glue::JS_GetReservedSlot;
use js::jsapi::JSObject;
use js::jsval::UndefinedValue;
use js::rust::GCMethods;

use crate::codegen::PrototypeList::PROTO_OR_IFACE_LENGTH;
use crate::utils::{ProtoOrIfaceArray, get_proto_or_iface_array};
use crate::weakref::{DOM_WEAK_SLOT, WeakBox, WeakReferenceable};

/// Drop the resources held by reserved slots of a global object
unsafe fn do_finalize_global(obj: *mut JSObject) {
    let protolist = get_proto_or_iface_array(obj);
    let list = (*protolist).as_mut_ptr();
    for idx in 0..PROTO_OR_IFACE_LENGTH as isize {
        let entry = list.offset(idx);
        let value = *entry;
        <*mut JSObject>::post_barrier(entry, value, ptr::null_mut());
    }
    let _: Box<ProtoOrIfaceArray> = Box::from_raw(protolist);
}

/// # Safety
/// `this` must point to a valid, non-null instance of T.
pub(crate) unsafe fn finalize_common<T>(this: *const T) {
    if !this.is_null() {
        // The pointer can be null if the object is the unforgeable holder of that interface.
        let _ = Box::from_raw(this as *mut T);
    }
    debug!("{} finalize: {:p}", type_name::<T>(), this);
}

/// # Safety
/// `obj` must point to a valid, non-null JS object.
/// `this` must point to a valid, non-null instance of T.
pub(crate) unsafe fn finalize_global<T>(obj: *mut JSObject, this: *const T) {
    do_finalize_global(obj);
    finalize_common::<T>(this);
}

/// # Safety
/// `obj` must point to a valid, non-null JS object.
/// `this` must point to a valid, non-null instance of T.
pub(crate) unsafe fn finalize_weak_referenceable<T: WeakReferenceable>(
    obj: *mut JSObject,
    this: *const T,
) {
    let mut slot = UndefinedValue();
    JS_GetReservedSlot(obj, DOM_WEAK_SLOT, &mut slot);
    let weak_box_ptr = slot.to_private() as *mut WeakBox<T>;
    if !weak_box_ptr.is_null() {
        let count = {
            let weak_box = &*weak_box_ptr;
            assert!(weak_box.value.get().is_some());
            assert!(weak_box.count.get() > 0);
            weak_box.value.set(None);
            let count = weak_box.count.get() - 1;
            weak_box.count.set(count);
            count
        };
        if count == 0 {
            mem::drop(Box::from_raw(weak_box_ptr));
        }
    }
    finalize_common::<T>(this);
}
