/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module implements structured cloning, as defined by [HTML]
//! (https://html.spec.whatwg.org/multipage/#safe-passing-of-structured-data).

use dom::bindings::conversions::root_from_handleobject;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::js::Root;
use dom::bindings::reflector::DomObject;
use dom::blob::{Blob, BlobImpl};
use dom::globalscope::GlobalScope;
use js;
use js::jsapi;
use std::os::raw;
use std::ptr;

// TODO: Should we add Min and Max const to https://github.com/servo/rust-mozjs/blob/master/src/consts.rs?
// TODO: Determine for sure which value Min and Max should have.
// NOTE: Current values found at https://dxr.mozilla.org/mozilla-central/
// rev/ff04d410e74b69acfab17ef7e73e7397602d5a68/js/public/StructuredClone.h#323
#[repr(u32)]
enum StructuredCloneTags {
    /// To support additional types, add new tags with values incremented from the last one before Max.
    Min = 0xFFFF8000,
    DomBlob = 0xFFFF8001,
    Max = 0xFFFFFFFF,
}

#[cfg(target_pointer_width = "64")]
unsafe fn write_length(w: *mut jsapi::JSStructuredCloneWriter,
                       length: usize) {
  let high: u32 = (length >> 32) as u32;
  let low: u32 = length as u32;
  assert!(jsapi::JS_WriteUint32Pair(w, high, low));
}

#[cfg(target_pointer_width = "32")]
unsafe fn write_length(w: *mut jsapi::JSStructuredCloneWriter,
                       length: usize) {
  assert!(jsapi::JS_WriteUint32Pair(w, length as u32, 0));
}

#[cfg(target_pointer_width = "64")]
unsafe fn read_length(r: *mut jsapi::JSStructuredCloneReader)
                      -> usize {
  let mut high: u32 = 0;
  let mut low: u32 = 0;
  assert!(jsapi::JS_ReadUint32Pair(r, &mut high as *mut u32, &mut low as *mut u32));
  return (low << high) as usize;
}

#[cfg(target_pointer_width = "32")]
unsafe fn read_length(r: *mut jsapi::JSStructuredCloneReader)
                      -> usize {
  let mut length: u32 = 0;
  let mut zero: u32 = 0;
  assert!(jsapi::JS_ReadUint32Pair(r, &mut length as *mut u32, &mut zero as *mut u32));
  return length as usize;
}

unsafe fn read_blob(cx: *mut jsapi::JSContext,
                    r: *mut jsapi::JSStructuredCloneReader)
                    -> *mut jsapi::JSObject {
    let blob_length = read_length(r);
    let type_str_length = read_length(r);
    let mut blob_buffer = vec![0u8; blob_length];
    assert!(jsapi::JS_ReadBytes(r, blob_buffer.as_mut_ptr() as *mut raw::c_void, blob_length));
    let mut type_str_buffer = vec![0u8; type_str_length];
    assert!(jsapi::JS_ReadBytes(r, type_str_buffer.as_mut_ptr() as *mut raw::c_void, type_str_length));
    let type_str = String::from_utf8_unchecked(type_str_buffer);
    let target_global = GlobalScope::from_context(cx);
    let blob = Blob::new(&target_global, BlobImpl::new_from_bytes(blob_buffer), type_str);
    return blob.reflector().get_jsobject().get()
}

unsafe fn write_blob(blob: Root<Blob>,
                     w: *mut jsapi::JSStructuredCloneWriter)
                     -> Result<(), ()> {
    let blob_vec = blob.get_bytes()?;
    let blob_length = blob_vec.len();
    let type_string_bytes = blob.type_string().as_bytes().to_vec();
    let type_string_length = type_string_bytes.len();
    assert!(jsapi::JS_WriteUint32Pair(w, StructuredCloneTags::DomBlob as u32, 0));
    write_length(w, blob_length);
    write_length(w, type_string_length);
    assert!(jsapi::JS_WriteBytes(w, blob_vec.as_ptr() as *const raw::c_void, blob_length));
    assert!(jsapi::JS_WriteBytes(w, type_string_bytes.as_ptr() as *const raw::c_void, type_string_length));
    return Ok(())
}

unsafe extern "C" fn read_callback(cx: *mut jsapi::JSContext,
                                   r: *mut jsapi::JSStructuredCloneReader,
                                   tag: u32,
                                   _data: u32,
                                   _closure: *mut raw::c_void)
                                   -> *mut jsapi::JSObject {
    assert!(tag < StructuredCloneTags::Max as u32, "tag should be lower than StructuredCloneTags::Max");
    assert!(tag > StructuredCloneTags::Min as u32, "tag should be higher than StructuredCloneTags::Min");
    if tag == StructuredCloneTags::DomBlob as u32 {
        return read_blob(cx, r)
    }
    return ptr::null_mut()
}

unsafe extern "C" fn write_callback(_cx: *mut jsapi::JSContext,
                                    w: *mut jsapi::JSStructuredCloneWriter,
                                    obj: jsapi::JS::HandleObject,
                                    _closure: *mut raw::c_void)
                                    -> bool {
    if let Ok(blob) = root_from_handleobject::<Blob>(obj) {
        return write_blob(blob, w).is_ok()
    }
    return false
}

unsafe extern "C" fn read_transfer_callback(_cx: *mut jsapi::JSContext,
                                            _r: *mut jsapi::JSStructuredCloneReader,
                                            _tag: u32,
                                            _content: *mut raw::c_void,
                                            _extra_data: u64,
                                            _closure: *mut raw::c_void,
                                            _return_object: jsapi::JS::MutableHandleObject)
                                            -> bool {
    false
}

unsafe extern "C" fn write_transfer_callback(_cx: *mut jsapi::JSContext,
                                             _obj: jsapi::JS::Handle<*mut jsapi::JSObject>,
                                             _closure: *mut raw::c_void,
                                             _tag: *mut u32,
                                             _ownership: *mut jsapi::JS::TransferableOwnership,
                                             _content:  *mut *mut raw::c_void,
                                             _extra_data: *mut u64)
                                             -> bool {
    false
}

unsafe extern "C" fn free_transfer_callback(_tag: u32,
                                            _ownership: jsapi::JS::TransferableOwnership,
                                            _content: *mut raw::c_void,
                                            _extra_data: u64,
                                            _closure: *mut raw::c_void) {
}

unsafe extern "C" fn report_error_callback(_cx: *mut jsapi::JSContext, _errorid: u32) {
}

static STRUCTURED_CLONE_CALLBACKS: jsapi::JSStructuredCloneCallbacks = jsapi::JSStructuredCloneCallbacks {
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
    Struct(js::sc::StructuredCloneBuffer),
    /// A variant that can be serialized
    Vector(Vec<u8>)
}

impl StructuredCloneData {
    /// Writes a structured clone. Returns a `DataClone` error if that fails.
    pub fn write(cx: *mut jsapi::JSContext, message: jsapi::JS::HandleValue) -> Fallible<StructuredCloneData> {
        let scope = jsapi::JS::StructuredCloneScope::SameProcessDifferentThread;
        let callbacks = &STRUCTURED_CLONE_CALLBACKS;
        let mut data = js::sc::StructuredCloneBuffer::new(scope, callbacks);

        let result = data.write(message, callbacks);
        if !result {
            unsafe {
                jsapi::JS_ClearPendingException(cx);
            }
            return Err(Error::DataClone);
        }
        Ok(StructuredCloneData::Struct(data))
    }

    /// Converts a StructuredCloneData to Vec<u8> for inter-thread sharing
    pub fn move_to_arraybuffer(self) -> Vec<u8> {
        match self {
            StructuredCloneData::Struct(data) => data.copy_to_vec(),
            StructuredCloneData::Vector(msg) => msg
        }
    }

    /// Reads a structured clone.
    ///
    /// Panics if `jsapi::JS_ReadStructuredClone` fails.
    fn read_clone(global: &GlobalScope,
                  data: &mut js::sc::StructuredCloneBuffer,
                  rval: jsapi::JS::MutableHandleValue) {
        let cx = global.get_cx();
        let globalhandle = global.reflector().get_jsobject();
        let _ac = unsafe {
            js::ac::AutoCompartment::with_obj(cx, globalhandle.get())
        };
        assert!(data.read(rval, &STRUCTURED_CLONE_CALLBACKS));
    }

    /// Thunk for the actual `read_clone` method. Resolves proper variant for read_clone.
    pub fn read(mut self, global: &GlobalScope, rval: jsapi::JS::MutableHandleValue) {
        match self {
            StructuredCloneData::Vector(vec_msg) => {
                let scope = jsapi::JS::StructuredCloneScope::SameProcessDifferentThread;
                let callbacks = &STRUCTURED_CLONE_CALLBACKS;
                let mut data = js::sc::StructuredCloneBuffer::new(scope, callbacks);
                assert!(data.write_bytes(&vec_msg[..]));
                StructuredCloneData::read_clone(global, &mut data, rval);
            }
            StructuredCloneData::Struct(ref mut data) => {
                StructuredCloneData::read_clone(global, data, rval);
            }
        }
    }
}

unsafe impl Send for StructuredCloneData {}
