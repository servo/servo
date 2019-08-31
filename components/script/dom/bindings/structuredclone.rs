/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module implements structured cloning, as defined by [HTML]
//! (https://html.spec.whatwg.org/multipage/#safe-passing-of-structured-data).

use crate::compartments::enter_realm;
use crate::dom::bindings::conversions::{root_from_object, ToJSValConvertible};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::serializable::Serializable;
use crate::dom::bindings::transferable::Transferable;
use crate::dom::blob::Blob;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageport::MessagePort;
use js::glue::CopyJSStructuredCloneData;
use js::glue::DeleteJSAutoStructuredCloneBuffer;
use js::glue::GetLengthOfJSStructuredCloneData;
use js::glue::NewJSAutoStructuredCloneBuffer;
use js::glue::WriteBytesToJSStructuredCloneData;
use js::jsapi::CloneDataPolicy;
use js::jsapi::HandleObject as RawHandleObject;
use js::jsapi::JSContext;
use js::jsapi::MutableHandleObject as RawMutableHandleObject;
use js::jsapi::StructuredCloneScope;
use js::jsapi::TransferableOwnership;
use js::jsapi::JS_STRUCTURED_CLONE_VERSION;
use js::jsapi::{JSObject, JS_ClearPendingException};
use js::jsapi::{JSStructuredCloneCallbacks, JSStructuredCloneReader, JSStructuredCloneWriter};
use js::jsapi::{JS_ReadUint32Pair, JS_WriteUint32Pair};
use js::jsval::UndefinedValue;
use js::rust::wrappers::{JS_ReadStructuredClone, JS_WriteStructuredClone};
use js::rust::{CustomAutoRooterGuard, HandleValue, MutableHandleValue};
use msg::constellation_msg::{BlobId, MessagePortId};
use script_traits::serializable::BlobImpl;
use script_traits::transferable::MessagePortImpl;
use script_traits::StructuredSerializedData;
use std::collections::HashMap;
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
    MessagePort = 0xFFFF8002,
    Max = 0xFFFFFFFF,
}

unsafe fn read_blob(
    owner: &DomRoot<GlobalScope>,
    r: *mut JSStructuredCloneReader,
    mut sc_holder: &mut StructuredCloneHolder,
) -> *mut JSObject {
    let mut name_space: u32 = 0;
    let mut index: u32 = 0;
    assert!(JS_ReadUint32Pair(
        r,
        &mut name_space as *mut u32,
        &mut index as *mut u32
    ));
    if let Ok(index) =
        <Blob as Serializable>::deserialize(&owner, &mut sc_holder, (name_space, index))
    {
        if let Some(blob) = sc_holder.blobs.get(index) {
            return blob.reflector().get_jsobject().get();
        }
    }
    warn!(
        "Reading structured data for a blob failed in {:?}.",
        owner.get_url()
    );
    ptr::null_mut()
}

unsafe fn write_blob(
    owner: &DomRoot<GlobalScope>,
    blob: DomRoot<Blob>,
    w: *mut JSStructuredCloneWriter,
    sc_holder: &mut StructuredCloneHolder,
) -> bool {
    if let Ok(data) = blob.serialize(sc_holder) {
        assert!(JS_WriteUint32Pair(
            w,
            StructuredCloneTags::DomBlob as u32,
            0
        ));
        assert!(JS_WriteUint32Pair(w, data.0, data.1));
        return true;
    }
    warn!(
        "Writing structured data for a blob failed in {:?}.",
        owner.get_url()
    );
    return false;
}

unsafe extern "C" fn read_callback(
    cx: *mut JSContext,
    r: *mut JSStructuredCloneReader,
    tag: u32,
    _data: u32,
    closure: *mut raw::c_void,
) -> *mut JSObject {
    assert!(
        tag < StructuredCloneTags::Max as u32,
        "tag should be lower than StructuredCloneTags::Max"
    );
    assert!(
        tag > StructuredCloneTags::Min as u32,
        "tag should be higher than StructuredCloneTags::Min"
    );
    if tag == StructuredCloneTags::DomBlob as u32 {
        return read_blob(
            &GlobalScope::from_context(cx),
            r,
            &mut *(closure as *mut StructuredCloneHolder),
        );
    }
    return ptr::null_mut();
}

unsafe extern "C" fn write_callback(
    cx: *mut JSContext,
    w: *mut JSStructuredCloneWriter,
    obj: RawHandleObject,
    closure: *mut raw::c_void,
) -> bool {
    if let Ok(blob) = root_from_object::<Blob>(*obj, cx) {
        return write_blob(
            &GlobalScope::from_context(cx),
            blob,
            w,
            &mut *(closure as *mut StructuredCloneHolder),
        );
    }
    return false;
}

unsafe extern "C" fn read_transfer_callback(
    cx: *mut JSContext,
    _r: *mut JSStructuredCloneReader,
    tag: u32,
    _content: *mut raw::c_void,
    extra_data: u64,
    closure: *mut raw::c_void,
    return_object: RawMutableHandleObject,
) -> bool {
    if tag == StructuredCloneTags::MessagePort as u32 {
        let mut sc_holder = &mut *(closure as *mut StructuredCloneHolder);
        let owner = GlobalScope::from_context(cx);
        if let Ok(_) = <MessagePort as Transferable>::transfer_receive(
            &owner,
            &mut sc_holder,
            extra_data,
            return_object,
        ) {
            return true;
        }
    }
    false
}

/// <https://html.spec.whatwg.org/multipage/#structuredserializewithtransfer>
unsafe extern "C" fn write_transfer_callback(
    cx: *mut JSContext,
    obj: RawHandleObject,
    closure: *mut raw::c_void,
    tag: *mut u32,
    ownership: *mut TransferableOwnership,
    _content: *mut *mut raw::c_void,
    extra_data: *mut u64,
) -> bool {
    if let Ok(port) = root_from_object::<MessagePort>(*obj, cx) {
        if port.detached() {
            return false;
        }
        *tag = StructuredCloneTags::MessagePort as u32;
        *ownership = TransferableOwnership::SCTAG_TMO_CUSTOM;
        let mut sc_holder = &mut *(closure as *mut StructuredCloneHolder);
        if let Ok(data) = port.transfer(&mut sc_holder) {
            *extra_data = data;
            return true;
        }
    }
    false
}

unsafe extern "C" fn free_transfer_callback(
    _tag: u32,
    _ownership: TransferableOwnership,
    _content: *mut raw::c_void,
    _extra_data: u64,
    _closure: *mut raw::c_void,
) {
}

unsafe extern "C" fn can_transfer_callback(
    cx: *mut JSContext,
    obj: RawHandleObject,
    _closure: *mut raw::c_void,
) -> bool {
    if let Ok(_port) = root_from_object::<MessagePort>(*obj, cx) {
        return true;
    }
    false
}

unsafe extern "C" fn report_error_callback(_cx: *mut JSContext, _errorid: u32) {}

static STRUCTURED_CLONE_CALLBACKS: JSStructuredCloneCallbacks = JSStructuredCloneCallbacks {
    read: Some(read_callback),
    write: Some(write_callback),
    reportError: Some(report_error_callback),
    readTransfer: Some(read_transfer_callback),
    writeTransfer: Some(write_transfer_callback),
    freeTransfer: Some(free_transfer_callback),
    canTransfer: Some(can_transfer_callback),
};

pub struct StructuredCloneHolder {
    pub blobs: Vec<DomRoot<Blob>>,
    pub message_ports: Vec<DomRoot<MessagePort>>,
    pub ports_impl: HashMap<MessagePortId, MessagePortImpl>,
    pub blob_impls: HashMap<BlobId, BlobImpl>,
}

// TODO: should this be unsafe?
/// Writes a structured clone. Returns a `DataClone` error if that fails.
pub fn write(
    cx: *mut JSContext,
    message: HandleValue,
    transfer: Option<CustomAutoRooterGuard<Vec<*mut JSObject>>>,
) -> Fallible<StructuredSerializedData> {
    unsafe {
        rooted!(in(cx) let mut val = UndefinedValue());
        if let Some(transfer) = transfer {
            transfer.to_jsval(cx, val.handle_mut());
        }

        let mut sc_holder = StructuredCloneHolder {
            blobs: vec![],
            message_ports: vec![],
            ports_impl: HashMap::new(),
            blob_impls: HashMap::new(),
        };
        let sc_holder_ptr = &mut sc_holder as *mut _;

        let scbuf = NewJSAutoStructuredCloneBuffer(
            StructuredCloneScope::DifferentProcess,
            &STRUCTURED_CLONE_CALLBACKS,
        );
        let scdata = &mut ((*scbuf).data_);
        let policy = CloneDataPolicy {
            // TODO: SAB?
            sharedArrayBuffer_: false,
        };
        let result = JS_WriteStructuredClone(
            cx,
            message,
            scdata,
            StructuredCloneScope::DifferentProcess,
            policy,
            &STRUCTURED_CLONE_CALLBACKS,
            sc_holder_ptr as *mut raw::c_void,
            val.handle(),
        );
        if !result {
            JS_ClearPendingException(cx);
            return Err(Error::DataClone);
        }

        let nbytes = GetLengthOfJSStructuredCloneData(scdata);
        let mut data = Vec::with_capacity(nbytes);
        CopyJSStructuredCloneData(scdata, data.as_mut_ptr());
        data.set_len(nbytes);

        DeleteJSAutoStructuredCloneBuffer(scbuf);

        let ports = match sc_holder.ports_impl.len() {
            0 => None,
            _ => Some(sc_holder.ports_impl),
        };

        let blobs = match sc_holder.blob_impls.len() {
            0 => None,
            _ => Some(sc_holder.blob_impls),
        };

        let data = StructuredSerializedData {
            serialized: data,
            ports: ports,
            blobs: blobs,
        };

        Ok(data)
    }
}

/// Read structured serialized data, possibly containing transferred objects.
/// Returns a holder of transferred object, or an error.
pub fn read(
    global: &GlobalScope,
    mut data: StructuredSerializedData,
    rval: MutableHandleValue,
) -> Result<StructuredCloneHolder, ()> {
    let cx = global.get_cx();
    let _ac = enter_realm(&*global);
    let ports = match data.ports.take() {
        Some(ports) => ports,
        None => HashMap::new(),
    };
    let blobs = match data.blobs.take() {
        Some(blobs) => blobs,
        None => HashMap::new(),
    };
    let mut sc_holder = StructuredCloneHolder {
        blobs: vec![],
        message_ports: vec![],
        ports_impl: ports,
        blob_impls: blobs,
    };
    let sc_holder_ptr = &mut sc_holder as *mut _;
    unsafe {
        let scbuf = NewJSAutoStructuredCloneBuffer(
            StructuredCloneScope::DifferentProcess,
            &STRUCTURED_CLONE_CALLBACKS,
        );
        let scdata = &mut ((*scbuf).data_);

        WriteBytesToJSStructuredCloneData(
            data.serialized.as_mut_ptr() as *const u8,
            data.serialized.len(),
            scdata,
        );

        let result = JS_ReadStructuredClone(
            *cx,
            scdata,
            JS_STRUCTURED_CLONE_VERSION,
            StructuredCloneScope::DifferentProcess,
            rval,
            &STRUCTURED_CLONE_CALLBACKS,
            sc_holder_ptr as *mut raw::c_void,
        );

        DeleteJSAutoStructuredCloneBuffer(scbuf);

        if result {
            return Ok(sc_holder);
        }
        Err(())
    }
}
