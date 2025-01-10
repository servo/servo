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
    JS_ClearPendingException, JS_ReadUint32Pair, JS_WriteUint32Pair,
    MutableHandleObject as RawMutableHandleObject, StructuredCloneScope, TransferableOwnership,
    JS_STRUCTURED_CLONE_VERSION,
};
use js::jsval::UndefinedValue;
use js::rust::wrappers::{JS_ReadStructuredClone, JS_WriteStructuredClone};
use js::rust::{CustomAutoRooterGuard, HandleValue, MutableHandleValue};
use script_traits::serializable::BlobImpl;
use script_traits::transferable::MessagePortImpl;
use script_traits::StructuredSerializedData;

use crate::dom::bindings::conversions::{root_from_object, ToJSValConvertible};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::serializable::{Serializable, StorageKey};
use crate::dom::bindings::transferable::Transferable;
use crate::dom::blob::Blob;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageport::MessagePort;
use crate::realms::{enter_realm, AlreadyInRealm, InRealm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

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
    Max = 0xFFFFFFFF,
}

unsafe fn read_blob(
    owner: &GlobalScope,
    r: *mut JSStructuredCloneReader,
    sc_reader: &mut StructuredDataReader,
    can_gc: CanGc,
) -> *mut JSObject {
    let mut name_space: u32 = 0;
    let mut index: u32 = 0;
    assert!(JS_ReadUint32Pair(
        r,
        &mut name_space as *mut u32,
        &mut index as *mut u32
    ));
    let storage_key = StorageKey { index, name_space };
    if <Blob as Serializable>::deserialize(owner, sc_reader, storage_key, can_gc).is_ok() {
        if let Some(blobs) = &sc_reader.blobs {
            let blob = blobs
                .get(&storage_key)
                .expect("No blob found at storage key.");
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
    owner: &GlobalScope,
    blob: DomRoot<Blob>,
    w: *mut JSStructuredCloneWriter,
    sc_writer: &mut StructuredDataWriter,
) -> bool {
    if let Ok(storage_key) = blob.serialize(sc_writer) {
        assert!(JS_WriteUint32Pair(
            w,
            StructuredCloneTags::DomBlob as u32,
            0
        ));
        assert!(JS_WriteUint32Pair(
            w,
            storage_key.name_space,
            storage_key.index
        ));
        return true;
    }
    warn!(
        "Writing structured data for a blob failed in {:?}.",
        owner.get_url()
    );
    false
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
    if tag == StructuredCloneTags::DomBlob as u32 {
        let in_realm_proof = AlreadyInRealm::assert_for_cx(SafeJSContext::from_ptr(cx));
        return read_blob(
            &GlobalScope::from_context(cx, InRealm::Already(&in_realm_proof)),
            r,
            &mut *(closure as *mut StructuredDataReader),
            CanGc::note(),
        );
    }
    ptr::null_mut()
}

unsafe extern "C" fn write_callback(
    cx: *mut JSContext,
    w: *mut JSStructuredCloneWriter,
    obj: RawHandleObject,
    _same_process_scope_required: *mut bool,
    closure: *mut raw::c_void,
) -> bool {
    if let Ok(blob) = root_from_object::<Blob>(*obj, cx) {
        let in_realm_proof = AlreadyInRealm::assert_for_cx(SafeJSContext::from_ptr(cx));
        return write_blob(
            &GlobalScope::from_context(cx, InRealm::Already(&in_realm_proof)),
            blob,
            w,
            &mut *(closure as *mut StructuredDataWriter),
        );
    }
    false
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
        let sc_reader = &mut *(closure as *mut StructuredDataReader);
        let in_realm_proof = AlreadyInRealm::assert_for_cx(SafeJSContext::from_ptr(cx));
        let owner = GlobalScope::from_context(cx, InRealm::Already(&in_realm_proof));
        if <MessagePort as Transferable>::transfer_receive(
            &owner,
            sc_reader,
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
        let sc_writer = &mut *(closure as *mut StructuredDataWriter);
        if let Ok(data) = port.transfer(sc_writer) {
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

/// Reader and writer structs for results from, and inputs to, structured-data read/write operations.
/// <https://html.spec.whatwg.org/multipage/#safe-passing-of-structured-data>
pub(crate) struct StructuredDataReader {
    /// A map of deserialized blobs, stored temporarily here to keep them rooted.
    pub(crate) blobs: Option<HashMap<StorageKey, DomRoot<Blob>>>,
    /// A vec of transfer-received DOM ports,
    /// to be made available to script through a message event.
    pub(crate) message_ports: Option<Vec<DomRoot<MessagePort>>>,
    /// A map of port implementations,
    /// used as part of the "transfer-receiving" steps of ports,
    /// to produce the DOM ports stored in `message_ports` above.
    pub(crate) port_impls: Option<HashMap<MessagePortId, MessagePortImpl>>,
    /// A map of blob implementations,
    /// used as part of the "deserialize" steps of blobs,
    /// to produce the DOM blobs stored in `blobs` above.
    pub(crate) blob_impls: Option<HashMap<BlobId, BlobImpl>>,
}

/// A data holder for transferred and serialized objects.
pub(crate) struct StructuredDataWriter {
    /// Transferred ports.
    pub(crate) ports: Option<HashMap<MessagePortId, MessagePortImpl>>,
    /// Serialized blobs.
    pub(crate) blobs: Option<HashMap<BlobId, BlobImpl>>,
}

/// Writes a structured clone. Returns a `DataClone` error if that fails.
pub(crate) fn write(
    cx: SafeJSContext,
    message: HandleValue,
    transfer: Option<CustomAutoRooterGuard<Vec<*mut JSObject>>>,
) -> Fallible<StructuredSerializedData> {
    unsafe {
        rooted!(in(*cx) let mut val = UndefinedValue());
        if let Some(transfer) = transfer {
            transfer.to_jsval(*cx, val.handle_mut());
        }
        let mut sc_writer = StructuredDataWriter {
            ports: None,
            blobs: None,
        };
        let sc_writer_ptr = &mut sc_writer as *mut _;

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
            sc_writer_ptr as *mut raw::c_void,
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

        let data = StructuredSerializedData {
            serialized: data,
            ports: sc_writer.ports.take(),
            blobs: sc_writer.blobs.take(),
        };

        Ok(data)
    }
}

/// Read structured serialized data, possibly containing transferred objects.
/// Returns a vec of rooted transfer-received ports, or an error.
pub(crate) fn read(
    global: &GlobalScope,
    mut data: StructuredSerializedData,
    rval: MutableHandleValue,
) -> Result<Vec<DomRoot<MessagePort>>, ()> {
    let cx = GlobalScope::get_cx();
    let _ac = enter_realm(global);
    let mut sc_reader = StructuredDataReader {
        blobs: None,
        message_ports: None,
        port_impls: data.ports.take(),
        blob_impls: data.blobs.take(),
    };
    let sc_reader_ptr = &mut sc_reader as *mut _;
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
            sc_reader_ptr as *mut raw::c_void,
        );

        DeleteJSAutoStructuredCloneBuffer(scbuf);

        if result {
            // Any transfer-received port-impls should have been taken out.
            assert!(sc_reader.port_impls.is_none());

            match sc_reader.message_ports.take() {
                Some(ports) => return Ok(ports),
                None => return Ok(Vec::with_capacity(0)),
            }
        }
        Err(())
    }
}
