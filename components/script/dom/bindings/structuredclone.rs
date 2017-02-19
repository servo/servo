/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module implements structured cloning, as defined by [HTML]
//! (https://html.spec.whatwg.org/multipage/#safe-passing-of-structured-data).

use dom::bindings::conversions::root_from_handleobject;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::reflector::DomObject;
use dom::blob::{Blob, BlobImpl};
use dom::globalscope::GlobalScope;
use js::jsapi::{Handle, HandleObject, HandleValue, MutableHandleValue};
use js::jsapi::{Heap, JSContext};
use js::jsapi::{JSStructuredCloneCallbacks, JSStructuredCloneReader, JSStructuredCloneWriter};
use js::jsapi::{JS_ClearPendingException, JSObject, JS_ReadStructuredClone};
use js::jsapi::{JS_ReadBytes, JS_WriteBytes};
use js::jsapi::{JS_STRUCTURED_CLONE_VERSION, JS_WriteStructuredClone, JS_WriteUint32Pair};
use js::jsapi::{MutableHandleObject, TransferableOwnership};
use libc::size_t;
use std::mem;
use std::os::raw;
use std::ptr;
use std::slice;

///TODO: Add Min and Max const to https://github.com/servo/rust-mozjs/blob/master/src/consts.rs?
///TODO: Determine which value Min and Max should have.
///NOTE: Current values found at https://dxr.mozilla.org/mozilla-central/source/js/public/StructuredClone.h#323
#[allow(dead_code)]
#[repr(u32)]
enum StructuredCloneTags {
    ScTagMin = 0xFFFF8000,
    ScTagDomBlob = 0xFFFF8001,
    ///NOTE: To support additional types, add new tags here.
    ScTagMax = 0xFFFFFFFF,
}

#[allow(dead_code)]
unsafe extern "C" fn read_callback(cx: *mut JSContext,
                                   r: *mut JSStructuredCloneReader,
                                   tag: u32,
                                   data: u32,
                                   _closure: *mut raw::c_void) -> *mut JSObject {
    if tag == StructuredCloneTags::ScTagDomBlob as u32 {
        let mut empty_vec: Vec<u8> = Vec::with_capacity(data as usize);
        let vec_mut_ptr: *mut raw::c_void = empty_vec.as_mut_ptr() as *mut _ as *mut raw::c_void;
        mem::forget(empty_vec);
        if JS_ReadBytes(r, vec_mut_ptr, data as usize) {
            let blob_bytes: Vec<u8> = Vec::from_raw_parts(vec_mut_ptr as *mut u8, data as usize, data as usize);
            let js_context = GlobalScope::from_context(cx);
            /// NOTE: missing the content-type string here.
            /// Use JS_WriteString in write_callback? Make Blob.type_string pub?
            let root = Blob::new(&js_context, BlobImpl::new_from_bytes(blob_bytes), "".to_string());
            return *root.reflector().get_jsobject().ptr
        }
    }
    return Heap::default().get()
}

#[allow(dead_code)]
unsafe extern "C" fn write_callback(_cx: *mut JSContext,
                                    w: *mut JSStructuredCloneWriter,
                                    obj: HandleObject,
                                    _closure: *mut raw::c_void) -> bool {
    if let Ok(blob) = root_from_handleobject::<Blob>(obj) {
        if let Ok(blob_vec) = blob.get_bytes() {
            let mut blob_vec_copy = blob_vec.to_vec();
            blob_vec_copy.shrink_to_fit();
            let blob_vec_copy_len = blob_vec_copy.len();
            if JS_WriteUint32Pair(w,
                                  StructuredCloneTags::ScTagDomBlob as u32,
                                  blob_vec_copy_len as u32) {
                let blob_vec_ptr: *const raw::c_void = blob_vec_copy.as_ptr() as *const _ as *const raw::c_void;
                mem::forget(blob_vec_copy);
                return JS_WriteBytes(w, blob_vec_ptr, blob_vec_copy_len)
            }
        }
    }
    return false
}

#[allow(dead_code)]
unsafe extern "C" fn read_transfer_callback(_cx: *mut JSContext,
                                            _r: *mut JSStructuredCloneReader,
                                            _tag: u32,
                                            _content: *mut raw::c_void,
                                            _extra_data: u64,
                                            _closure: *mut raw::c_void,
                                            _return_object: MutableHandleObject) -> bool {
    false
}

#[allow(dead_code)]
unsafe extern "C" fn write_transfer_callback(_cx: *mut JSContext,
                                             _obj: Handle<*mut JSObject>,
                                             _closure: *mut raw::c_void,
                                             _tag: *mut u32,
                                             _ownership: *mut TransferableOwnership,
                                             _content:  *mut *mut raw::c_void,
                                             _extra_data: *mut u64) -> bool {
    false
}

#[allow(dead_code)]
unsafe extern "C" fn free_transfer_callback(_tag: u32,
                                            _ownership: TransferableOwnership,
                                            _content: *mut raw::c_void,
                                            _extra_data: u64,
                                            _closure: *mut raw::c_void) {
}

#[allow(dead_code)]
unsafe extern "C" fn report_error_callback(_cx: *mut JSContext, _errorid: u32) {
}

#[allow(dead_code)]
static STRUCTURED_CLONE_CALLBACKS: JSStructuredCloneCallbacks = JSStructuredCloneCallbacks {
    read: Some(read_callback),
    write: Some(write_callback),
    reportError: Some(report_error_callback),
    readTransfer: Some(read_transfer_callback),
    writeTransfer: Some(write_transfer_callback),
    freeTransfer: Some(free_transfer_callback),
};

/// A buffer for a structured clone.
pub enum StructuredCloneData {
    /// A non-serializable (default) variant
    Struct(*mut u64, size_t),
    /// A variant that can be serialized
    Vector(Vec<u8>)
}

impl StructuredCloneData {
    /// Writes a structured clone. Returns a `DataClone` error if that fails.
    pub fn write(cx: *mut JSContext, message: HandleValue) -> Fallible<StructuredCloneData> {
        let mut data = ptr::null_mut();
        let mut nbytes = 0;
        let result = unsafe {
            JS_WriteStructuredClone(cx,
                                    message,
                                    &mut data,
                                    &mut nbytes,
                                    &STRUCTURED_CLONE_CALLBACKS,
                                    ptr::null_mut(),
                                    HandleValue::undefined())
        };
        if !result {
            unsafe {
                JS_ClearPendingException(cx);
            }
            return Err(Error::DataClone);
        }
        Ok(StructuredCloneData::Struct(data, nbytes))
    }

    /// Converts a StructuredCloneData to Vec<u8> for inter-thread sharing
    pub fn move_to_arraybuffer(self) -> Vec<u8> {
        match self {
            StructuredCloneData::Struct(data, nbytes) => {
                unsafe {
                    slice::from_raw_parts(data as *mut u8, nbytes).to_vec()
                }
            }
            StructuredCloneData::Vector(msg) => msg
        }
    }

    /// Reads a structured clone.
    ///
    /// Panics if `JS_ReadStructuredClone` fails.
    fn read_clone(global: &GlobalScope,
                  data: *mut u64,
                  nbytes: size_t,
                  rval: MutableHandleValue) {
        unsafe {
            assert!(JS_ReadStructuredClone(global.get_cx(),
                                           data,
                                           nbytes,
                                           JS_STRUCTURED_CLONE_VERSION,
                                           rval,
                                           &STRUCTURED_CLONE_CALLBACKS,
                                           ptr::null_mut()));
        }
    }

    /// Thunk for the actual `read_clone` method. Resolves proper variant for read_clone.
    pub fn read(self, global: &GlobalScope, rval: MutableHandleValue) {
        match self {
            StructuredCloneData::Vector(mut vec_msg) => {
                let nbytes = vec_msg.len();
                let data = vec_msg.as_mut_ptr() as *mut u64;
                StructuredCloneData::read_clone(global, data, nbytes, rval);
            }
            StructuredCloneData::Struct(data, nbytes) => StructuredCloneData::read_clone(global, data, nbytes, rval)
        }
    }
}

unsafe impl Send for StructuredCloneData {}
