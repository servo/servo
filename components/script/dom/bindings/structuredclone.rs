/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module implements structured cloning, as defined by [HTML]
//! (https://html.spec.whatwg.org/multipage/#safe-passing-of-structured-data).

use dom::bindings::conversions::root_from_handleobject;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::reflector::DomObject;
use dom::bindings::root::DomRoot;
use dom::blob::{Blob, BlobImpl};
use dom::globalscope::GlobalScope;
use js::glue::CopyJSStructuredCloneData;
use js::glue::DeleteJSAutoStructuredCloneBuffer;
use js::glue::GetLengthOfJSStructuredCloneData;
use js::glue::NewJSAutoStructuredCloneBuffer;
use js::glue::WriteBytesToJSStructuredCloneData;
use js::jsapi::{JSAutoCompartment, JSContext};
use js::jsapi::{JSStructuredCloneCallbacks, JSStructuredCloneReader, JSStructuredCloneWriter};
use js::jsapi::{JS_ClearPendingException, JSObject};
use js::jsapi::{JS_ReadBytes, JS_WriteBytes};
use js::jsapi::{JS_ReadUint32Pair, JS_WriteUint32Pair};
use js::jsapi::CloneDataPolicy;
use js::jsapi::HandleObject as RawHandleObject;
use js::jsapi::JS_STRUCTURED_CLONE_VERSION;
use js::jsapi::MutableHandleObject as RawMutableHandleObject;
use js::jsapi::StructuredCloneScope;
use js::jsapi::TransferableOwnership;
use js::rust::{Handle, HandleValue, MutableHandleValue};
use js::rust::wrappers::{JS_WriteStructuredClone, JS_ReadStructuredClone};
use libc::size_t;
use std::os::raw;
use std::ptr;
use std::slice;

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
unsafe fn write_length(w: *mut JSStructuredCloneWriter,
                       length: usize) {
  let high: u32 = (length >> 32) as u32;
  let low: u32 = length as u32;
  assert!(JS_WriteUint32Pair(w, high, low));
}

#[cfg(target_pointer_width = "32")]
unsafe fn write_length(w: *mut JSStructuredCloneWriter,
                       length: usize) {
  assert!(JS_WriteUint32Pair(w, length as u32, 0));
}

#[cfg(target_pointer_width = "64")]
unsafe fn read_length(r: *mut JSStructuredCloneReader)
                      -> usize {
  let mut high: u32 = 0;
  let mut low: u32 = 0;
  assert!(JS_ReadUint32Pair(r, &mut high as *mut u32, &mut low as *mut u32));
  return (low << high) as usize;
}

#[cfg(target_pointer_width = "32")]
unsafe fn read_length(r: *mut JSStructuredCloneReader)
                      -> usize {
  let mut length: u32 = 0;
  let mut zero: u32 = 0;
  assert!(JS_ReadUint32Pair(r, &mut length as *mut u32, &mut zero as *mut u32));
  return length as usize;
}

struct StructuredCloneWriter {
    w: *mut JSStructuredCloneWriter,
}

impl StructuredCloneWriter {
    unsafe fn write_slice(&self, v: &[u8]) {
        let type_length = v.len();
        write_length(self.w, type_length);
        assert!(JS_WriteBytes(self.w, v.as_ptr() as *const raw::c_void, type_length));
    }
    unsafe fn write_str(&self, s: &str) {
        self.write_slice(s.as_bytes());
    }
}

struct StructuredCloneReader {
    r: *mut JSStructuredCloneReader,
}

impl StructuredCloneReader {
    unsafe fn read_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0u8; read_length(self.r)];
        let blob_length = bytes.len();
        assert!(JS_ReadBytes(self.r, bytes.as_mut_ptr() as *mut raw::c_void, blob_length));
        return bytes;
    }
    unsafe fn read_str(&self) -> String {
        let str_buffer = self.read_bytes();
        return String::from_utf8_unchecked(str_buffer);
    }
}

unsafe fn read_blob(cx: *mut JSContext,
                    r: *mut JSStructuredCloneReader,
                    sc_holder: &mut StructuredCloneHolder)
                    -> *mut JSObject {
    let structured_reader = StructuredCloneReader { r: r };
    let blob_buffer = structured_reader.read_bytes();
    let type_str = structured_reader.read_str();
    let target_global = GlobalScope::from_context(cx);
    let blob = Blob::new(&target_global, BlobImpl::new_from_bytes(blob_buffer), type_str);
    let js_object = blob.reflector().get_jsobject().get();
    sc_holder.blob = Some(blob);
    js_object
}

unsafe fn write_blob(blob: DomRoot<Blob>,
                     w: *mut JSStructuredCloneWriter)
                     -> Result<(), ()> {
    let structured_writer = StructuredCloneWriter { w: w };
    let blob_vec = blob.get_bytes()?;
    assert!(JS_WriteUint32Pair(w, StructuredCloneTags::DomBlob as u32, 0));
    structured_writer.write_slice(&blob_vec);
    structured_writer.write_str(&blob.type_string());
    return Ok(())
}

unsafe extern "C" fn read_callback(cx: *mut JSContext,
                                   r: *mut JSStructuredCloneReader,
                                   tag: u32,
                                   _data: u32,
                                   closure: *mut raw::c_void)
                                   -> *mut JSObject {
    assert!(tag < StructuredCloneTags::Max as u32, "tag should be lower than StructuredCloneTags::Max");
    assert!(tag > StructuredCloneTags::Min as u32, "tag should be higher than StructuredCloneTags::Min");
    if tag == StructuredCloneTags::DomBlob as u32 {
        return read_blob(cx, r, &mut *(closure as *mut StructuredCloneHolder))
    }
    return ptr::null_mut()
}

unsafe extern "C" fn write_callback(_cx: *mut JSContext,
                                    w: *mut JSStructuredCloneWriter,
                                    obj: RawHandleObject,
                                    _closure: *mut raw::c_void)
                                    -> bool {
    if let Ok(blob) = root_from_handleobject::<Blob>(Handle::from_raw(obj)) {
        return write_blob(blob, w).is_ok()
    }
    return false
}

unsafe extern "C" fn read_transfer_callback(_cx: *mut JSContext,
                                            _r: *mut JSStructuredCloneReader,
                                            _tag: u32,
                                            _content: *mut raw::c_void,
                                            _extra_data: u64,
                                            _closure: *mut raw::c_void,
                                            _return_object: RawMutableHandleObject)
                                            -> bool {
    false
}

unsafe extern "C" fn write_transfer_callback(_cx: *mut JSContext,
                                             _obj: RawHandleObject,
                                             _closure: *mut raw::c_void,
                                             _tag: *mut u32,
                                             _ownership: *mut TransferableOwnership,
                                             _content:  *mut *mut raw::c_void,
                                             _extra_data: *mut u64)
                                             -> bool {
    false
}

unsafe extern "C" fn free_transfer_callback(_tag: u32,
                                            _ownership: TransferableOwnership,
                                            _content: *mut raw::c_void,
                                            _extra_data: u64,
                                            _closure: *mut raw::c_void) {
}

unsafe extern "C" fn report_error_callback(_cx: *mut JSContext, _errorid: u32) {
}

static STRUCTURED_CLONE_CALLBACKS: JSStructuredCloneCallbacks = JSStructuredCloneCallbacks {
    read: Some(read_callback),
    write: Some(write_callback),
    reportError: Some(report_error_callback),
    readTransfer: Some(read_transfer_callback),
    writeTransfer: Some(write_transfer_callback),
    freeTransfer: Some(free_transfer_callback),
};

struct StructuredCloneHolder {
    blob: Option<DomRoot<Blob>>
}

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
        unsafe {
            let scbuf = NewJSAutoStructuredCloneBuffer(StructuredCloneScope::DifferentProcess,
                                                       &STRUCTURED_CLONE_CALLBACKS);
            let scdata = &mut ((*scbuf).data_);
            let policy = CloneDataPolicy {
                // TODO: SAB?
                sharedArrayBuffer_: false,
            };
            let result = JS_WriteStructuredClone(cx,
                                                 message,
                                                 scdata,
                                                 StructuredCloneScope::DifferentProcess,
                                                 policy,
                                                 &STRUCTURED_CLONE_CALLBACKS,
                                                 ptr::null_mut(),
                                                 HandleValue::undefined());
            if !result {
                JS_ClearPendingException(cx);
                return Err(Error::DataClone);
            }

            let nbytes = GetLengthOfJSStructuredCloneData(scdata);
            let mut data = Vec::with_capacity(nbytes);
            CopyJSStructuredCloneData(scdata, data.as_mut_ptr());
            data.set_len(nbytes);

            DeleteJSAutoStructuredCloneBuffer(scbuf);

            Ok(StructuredCloneData::Vector(data))
        }
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
        let cx = global.get_cx();
        let globalhandle = global.reflector().get_jsobject();
        let _ac = JSAutoCompartment::new(cx, globalhandle.get());
        let mut sc_holder = StructuredCloneHolder { blob: None };
        let sc_holder_ptr = &mut sc_holder as *mut _;
        unsafe {
            let scbuf = NewJSAutoStructuredCloneBuffer(StructuredCloneScope::DifferentProcess,
                                                       &STRUCTURED_CLONE_CALLBACKS);
            let scdata = &mut ((*scbuf).data_);

            WriteBytesToJSStructuredCloneData(data as *const u8, nbytes, scdata);

            assert!(JS_ReadStructuredClone(cx,
                                           scdata,
                                           JS_STRUCTURED_CLONE_VERSION,
                                           StructuredCloneScope::DifferentProcess,
                                           rval,
                                           &STRUCTURED_CLONE_CALLBACKS,
                                           sc_holder_ptr as *mut raw::c_void));

            DeleteJSAutoStructuredCloneBuffer(scbuf);
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
