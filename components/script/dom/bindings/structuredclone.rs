/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module implements structured cloning, as defined by [HTML](https://html.spec.whatwg.org/multipage/#safe-passing-of-structured-data).

use std::collections::HashMap;
use std::os::raw;
use std::ptr;

use base::id::{BlobId, MessagePortId};
use js::glue::{
    CopyJSStructuredCloneData, DeleteJSAutoStructuredCloneBuffer, GetLengthOfJSStructuredCloneData,
    NewJSAutoStructuredCloneBuffer, WriteBytesToJSStructuredCloneData,
};
use js::jsapi::{
    CloneDataPolicy, HandleObject as RawHandleObject, JSContext, JSObject,
    JSStructuredCloneCallbacks, JSStructuredCloneReader, JSStructuredCloneWriter,
    JS_ClearPendingException, JS_WriteDouble, JS_WriteUint32Pair,
    MutableHandleObject as RawMutableHandleObject, StructuredCloneScope, TransferableOwnership,
    JS_STRUCTURED_CLONE_VERSION,
};
use js::jsval::UndefinedValue;
use js::rust::wrappers::{JS_ReadStructuredClone, JS_WriteStructuredClone};
use js::rust::{CustomAutoRooterGuard, HandleValue, MutableHandleValue};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use script_traits::serializable::BlobImpl;
use script_traits::transferable::MessagePortImpl;
use script_traits::StructuredSerializedData;

use crate::dom::bindings::conversions::{root_from_object, IDLInterface, ToJSValConvertible};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::serializable::{
    FromStructuredClone, Serializable, SerializeOperation, ToSerializeOperations,
};
use crate::dom::bindings::transferable::Transferable;
use crate::dom::blob::Blob;
use crate::dom::dompointreadonly::DOMPointReadOnly;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageport::MessagePort;
use crate::realms::{enter_realm, AlreadyInRealm, InRealm};
use crate::script_runtime::JSContext as SafeJSContext;

// TODO: Should we add Min and Max const to https://github.com/servo/rust-mozjs/blob/master/src/consts.rs?
// TODO: Determine for sure which value Min and Max should have.
// NOTE: Current values found at https://dxr.mozilla.org/mozilla-central/
// rev/ff04d410e74b69acfab17ef7e73e7397602d5a68/js/public/StructuredClone.h#323
#[repr(u32)]
pub(super) enum StructuredCloneTags {
    /// To support additional types, add new tags with values incremented from the last one before Max.
    Min = 0xFFFF8000,
    DomBlob = 0xFFFF8001,
    MessagePort = 0xFFFF8002,
    Principals = 0xFFFF8003,
    DomPointReadonly = 0xFFFF8004,
    Max = 0xFFFFFFFF,
}

unsafe fn deserialize_interface<T: Serializable>(
    owner: &GlobalScope,
    r: *mut JSStructuredCloneReader,
    sc_holder: &mut StructuredReadDataHolder,
) -> *mut JSObject {
    let data = <<T as Serializable>::Data as FromStructuredClone>::from_structured_clone(r);
    if let Ok(obj) = <T as Serializable>::deserialize(owner, sc_holder, data) {
        return obj.reflector().get_jsobject().get();
    }
    ptr::null_mut()
}

unsafe fn attempt_serialization<T: Serializable + IDLInterface>(
    cx: *mut JSContext,
    obj: RawHandleObject,
    w: *mut JSStructuredCloneWriter,
    holder: &mut StructuredWriteDataHolder,
) -> Result<bool, ()> {
    root_from_object::<T>(*obj, cx).map(|obj| {
        let Ok(data) = obj.serialize(holder) else {
            return false;
        };
        SerializeOperation::Uint32Pair(<T as Serializable>::TAG as u32, 0).invoke(w);
        for operation in data.to_serialize_operations() {
            operation.invoke(w);
        }
        true
    })
}

impl SerializeOperation {
    fn invoke(&self, w: *mut JSStructuredCloneWriter) {
        unsafe {
            match self {
                SerializeOperation::Uint32Pair(a, b) => assert!(JS_WriteUint32Pair(w, *a, *b)),
                SerializeOperation::Double(f) => assert!(JS_WriteDouble(w, *f)),
            }
        }
    }
}

type DomObjectReadCallback = unsafe fn(
    &GlobalScope,
    *mut JSStructuredCloneReader,
    &mut StructuredReadDataHolder,
) -> *mut JSObject;
type DomObjectWriteCallback = unsafe fn(
    *mut JSContext,
    obj: RawHandleObject,
    w: *mut JSStructuredCloneWriter,
    holder: &mut StructuredWriteDataHolder,
) -> Result<bool, ()>;

#[derive(FromPrimitive, enum_iterator::Sequence)]
pub enum CloneableObject {
    Blob = StructuredCloneTags::DomBlob as isize,
    DomPointReadonly = StructuredCloneTags::DomPointReadonly as isize,
}

impl CloneableObject {
    fn read_callback(&self) -> DomObjectReadCallback {
        match self {
            CloneableObject::Blob => deserialize_interface::<Blob>,
            CloneableObject::DomPointReadonly => deserialize_interface::<DOMPointReadOnly>,
        }
    }

    fn write_callback(&self) -> DomObjectWriteCallback {
        match self {
            CloneableObject::Blob => attempt_serialization::<Blob>,
            CloneableObject::DomPointReadonly => attempt_serialization::<DOMPointReadOnly>,
        }
    }
}

unsafe extern "C" fn read_callback(
    cx: *mut JSContext,
    r: *mut JSStructuredCloneReader,
    _policy: *const CloneDataPolicy,
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
    let Some(cloneable): Option<CloneableObject> = FromPrimitive::from_u32(tag) else {
        return ptr::null_mut();
    };

    let in_realm_proof = AlreadyInRealm::assert_for_cx(SafeJSContext::from_ptr(cx));
    let global = GlobalScope::from_context(cx, InRealm::Already(&in_realm_proof));
    (cloneable.read_callback())(&global, r, &mut *(closure as *mut StructuredReadDataHolder))
}

unsafe extern "C" fn write_callback(
    cx: *mut JSContext,
    w: *mut JSStructuredCloneWriter,
    obj: RawHandleObject,
    _same_process_scope_required: *mut bool,
    closure: *mut raw::c_void,
) -> bool {
    let holder = &mut *(closure as *mut StructuredWriteDataHolder);
    enum_iterator::all::<CloneableObject>()
        .map(|interface| (interface.write_callback())(cx, obj, w, holder))
        .filter_map(|result| result.ok())
        .next()
        .unwrap_or_default()
}

unsafe extern "C" fn read_transfer_callback(
    cx: *mut JSContext,
    _r: *mut JSStructuredCloneReader,
    _policy: *const CloneDataPolicy,
    tag: u32,
    _content: *mut raw::c_void,
    extra_data: u64,
    closure: *mut raw::c_void,
    return_object: RawMutableHandleObject,
) -> bool {
    if tag == StructuredCloneTags::MessagePort as u32 {
        let sc_holder = &mut *(closure as *mut StructuredReadDataHolder);
        let in_realm_proof = AlreadyInRealm::assert_for_cx(SafeJSContext::from_ptr(cx));
        let owner = GlobalScope::from_context(cx, InRealm::Already(&in_realm_proof));
        if <MessagePort as Transferable>::transfer_receive(
            &owner,
            sc_holder,
            extra_data,
            return_object,
        )
        .is_ok()
        {
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
        *tag = StructuredCloneTags::MessagePort as u32;
        *ownership = TransferableOwnership::SCTAG_TMO_CUSTOM;
        let sc_holder = &mut *(closure as *mut StructuredWriteDataHolder);
        if let Ok(data) = port.transfer(sc_holder) {
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
    _same_process_scope_required: *mut bool,
    _closure: *mut raw::c_void,
) -> bool {
    if let Ok(_port) = root_from_object::<MessagePort>(*obj, cx) {
        return true;
    }
    false
}

unsafe extern "C" fn report_error_callback(
    _cx: *mut JSContext,
    _errorid: u32,
    _closure: *mut ::std::os::raw::c_void,
    _error_message: *const ::std::os::raw::c_char,
) {
}

unsafe extern "C" fn sab_cloned_callback(
    _cx: *mut JSContext,
    _receiving: bool,
    _closure: *mut ::std::os::raw::c_void,
) -> bool {
    false
}

static STRUCTURED_CLONE_CALLBACKS: JSStructuredCloneCallbacks = JSStructuredCloneCallbacks {
    read: Some(read_callback),
    write: Some(write_callback),
    reportError: Some(report_error_callback),
    readTransfer: Some(read_transfer_callback),
    writeTransfer: Some(write_transfer_callback),
    freeTransfer: Some(free_transfer_callback),
    canTransfer: Some(can_transfer_callback),
    sabCloned: Some(sab_cloned_callback),
};

/// A data holder for results from, and inputs to, structured-data read/write operations.
/// <https://html.spec.whatwg.org/multipage/#safe-passing-of-structured-data>
pub struct StructuredReadDataHolder {
    /// A vec of transfer-received DOM ports,
    /// to be made available to script through a message event.
    pub message_ports: Option<Vec<DomRoot<MessagePort>>>,
    /// A map of port implementations,
    /// used as part of the "transfer-receiving" steps of ports,
    /// to produce the DOM ports stored in `message_ports` above.
    pub port_impls: Option<HashMap<MessagePortId, MessagePortImpl>>,
    /// A map of blob implementations,
    /// used as part of the "deserialize" steps of blobs,
    /// to produce the DOM blobs stored in `blobs` above.
    pub blob_impls: Option<HashMap<BlobId, BlobImpl>>,
}

/// A data holder for transferred and serialized objects.
pub struct StructuredWriteDataHolder {
    /// Transferred ports.
    pub ports: Option<HashMap<MessagePortId, MessagePortImpl>>,
    /// Serialized blobs.
    pub blobs: Option<HashMap<BlobId, BlobImpl>>,
}

/// Writes a structured clone. Returns a `DataClone` error if that fails.
pub fn write(
    cx: SafeJSContext,
    message: HandleValue,
    transfer: Option<CustomAutoRooterGuard<Vec<*mut JSObject>>>,
) -> Fallible<StructuredSerializedData> {
    unsafe {
        rooted!(in(*cx) let mut val = UndefinedValue());
        if let Some(transfer) = transfer {
            transfer.to_jsval(*cx, val.handle_mut());
        }
        let mut sc_holder = StructuredWriteDataHolder {
            ports: None,
            blobs: None,
        };
        let sc_holder_ptr = &mut sc_holder as *mut _;

        let scbuf = NewJSAutoStructuredCloneBuffer(
            StructuredCloneScope::DifferentProcess,
            &STRUCTURED_CLONE_CALLBACKS,
        );
        let scdata = &mut ((*scbuf).data_);
        let policy = CloneDataPolicy {
            allowIntraClusterClonableSharedObjects_: false,
            allowSharedMemoryObjects_: false,
        };
        let result = JS_WriteStructuredClone(
            *cx,
            message,
            scdata,
            StructuredCloneScope::DifferentProcess,
            &policy,
            &STRUCTURED_CLONE_CALLBACKS,
            sc_holder_ptr as *mut raw::c_void,
            val.handle(),
        );
        if !result {
            JS_ClearPendingException(*cx);
            return Err(Error::DataClone);
        }

        let nbytes = GetLengthOfJSStructuredCloneData(scdata);
        let mut data = Vec::with_capacity(nbytes);
        CopyJSStructuredCloneData(scdata, data.as_mut_ptr());
        data.set_len(nbytes);

        DeleteJSAutoStructuredCloneBuffer(scbuf);

        let StructuredWriteDataHolder {
            blobs: mut blob_impls,
            ports: mut port_impls,
        } = sc_holder;

        let data = StructuredSerializedData {
            serialized: data,
            ports: port_impls.take(),
            blobs: blob_impls.take(),
        };

        Ok(data)
    }
}

/// Read structured serialized data, possibly containing transferred objects.
/// Returns a vec of rooted transfer-received ports, or an error.
pub fn read(
    global: &GlobalScope,
    mut data: StructuredSerializedData,
    rval: MutableHandleValue,
) -> Result<Vec<DomRoot<MessagePort>>, ()> {
    let cx = GlobalScope::get_cx();
    let _ac = enter_realm(global);
    let mut sc_holder = StructuredReadDataHolder {
        message_ports: None,
        port_impls: data.ports.take(),
        blob_impls: data.blobs.take(),
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
            &CloneDataPolicy {
                allowIntraClusterClonableSharedObjects_: false,
                allowSharedMemoryObjects_: false,
            },
            &STRUCTURED_CLONE_CALLBACKS,
            sc_holder_ptr as *mut raw::c_void,
        );

        DeleteJSAutoStructuredCloneBuffer(scbuf);

        if result {
            let StructuredReadDataHolder {
                mut message_ports,
                port_impls,
                ..
            } = sc_holder;

            // Any transfer-received port-impls should have been taken out.
            assert!(port_impls.is_none());

            match message_ports.take() {
                Some(ports) => return Ok(ports),
                None => return Ok(Vec::with_capacity(0)),
            }
        }
        Err(())
    }
}
