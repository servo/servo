/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module implements structured cloning, as defined by [HTML](https://html.spec.whatwg.org/multipage/#safe-passing-of-structured-data).

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::os::raw;
use std::ptr;

use base::id::{BlobId, MessagePortId, PipelineNamespaceId};
use js::glue::{
    CopyJSStructuredCloneData, DeleteJSAutoStructuredCloneBuffer, GetLengthOfJSStructuredCloneData,
    NewJSAutoStructuredCloneBuffer, WriteBytesToJSStructuredCloneData,
};
use js::jsapi::{
    CloneDataPolicy, HandleObject as RawHandleObject, JS_ClearPendingException, JS_ReadUint32Pair,
    JS_STRUCTURED_CLONE_VERSION, JS_WriteUint32Pair, JSContext, JSObject,
    JSStructuredCloneCallbacks, JSStructuredCloneReader, JSStructuredCloneWriter,
    MutableHandleObject as RawMutableHandleObject, StructuredCloneScope, TransferableOwnership,
};
use js::jsval::UndefinedValue;
use js::rust::wrappers::{JS_ReadStructuredClone, JS_WriteStructuredClone};
use js::rust::{CustomAutoRooterGuard, HandleValue, MutableHandleValue};
use script_bindings::conversions::IDLInterface;
use script_traits::serializable::BlobImpl;
use script_traits::transferable::MessagePortImpl;
use script_traits::{
    Serializable as SerializableInterface, StructuredSerializedData,
    Transferrable as TransferrableInterface,
};
use strum::IntoEnumIterator;

use crate::dom::bindings::conversions::{ToJSValConvertible, root_from_object};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::serializable::{IntoStorageKey, Serializable, StorageKey};
use crate::dom::bindings::transferable::{ExtractComponents, IdFromComponents, Transferable};
use crate::dom::blob::Blob;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageport::MessagePort;
use crate::realms::{AlreadyInRealm, InRealm, enter_realm};
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

impl From<SerializableInterface> for StructuredCloneTags {
    fn from(v: SerializableInterface) -> Self {
        match v {
            SerializableInterface::Blob => StructuredCloneTags::DomBlob,
        }
    }
}

impl From<TransferrableInterface> for StructuredCloneTags {
    fn from(v: TransferrableInterface) -> Self {
        match v {
            TransferrableInterface::MessagePort => StructuredCloneTags::MessagePort,
        }
    }
}

fn reader_for_type(
    val: SerializableInterface,
) -> unsafe fn(
    &GlobalScope,
    *mut JSStructuredCloneReader,
    &mut StructuredDataReader,
    CanGc,
) -> *mut JSObject {
    match val {
        SerializableInterface::Blob => read_object::<Blob>,
    }
}

unsafe fn read_object<T: Serializable>(
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

    // 1. Re-build the key for the storage location
    // of the serialized object.
    let id = T::Id::from(storage_key);

    // 2. Get the transferred object from its storage, using the key.
    let objects = T::serialized_storage(StructuredData::Reader(sc_reader));
    let objects_map = objects
        .as_mut()
        .expect("The SC holder does not have any relevant objects");
    let serialized = objects_map
        .remove(&id)
        .expect("No object to be deserialized found.");
    if objects_map.is_empty() {
        *objects = None;
    }

    if let Ok(obj) = T::deserialize(owner, serialized, can_gc) {
        let destination = T::deserialized_storage(sc_reader).get_or_insert_with(HashMap::new);
        let reflector = obj.reflector().get_jsobject().get();
        destination.insert(storage_key, obj);
        return reflector;
    }
    warn!("Reading structured data failed in {:?}.", owner.get_url());
    ptr::null_mut()
}

unsafe fn write_object<T: Serializable>(
    interface: SerializableInterface,
    owner: &GlobalScope,
    object: &T,
    w: *mut JSStructuredCloneWriter,
    sc_writer: &mut StructuredDataWriter,
) -> bool {
    if let Ok((new_id, serialized)) = object.serialize() {
        let objects = T::serialized_storage(StructuredData::Writer(sc_writer))
            .get_or_insert_with(HashMap::new);
        objects.insert(new_id, serialized);
        let storage_key = new_id.into_storage_key();

        assert!(JS_WriteUint32Pair(
            w,
            StructuredCloneTags::from(interface) as u32,
            0
        ));
        assert!(JS_WriteUint32Pair(
            w,
            storage_key.name_space,
            storage_key.index
        ));
        return true;
    }
    warn!("Writing structured data failed in {:?}.", owner.get_url());
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

    let sc_reader = &mut *(closure as *mut StructuredDataReader);
    let in_realm_proof = AlreadyInRealm::assert_for_cx(SafeJSContext::from_ptr(cx));
    let global = GlobalScope::from_context(cx, InRealm::Already(&in_realm_proof));
    for serializable in SerializableInterface::iter() {
        if tag == StructuredCloneTags::from(serializable) as u32 {
            let reader = reader_for_type(serializable);
            return reader(&global, r, sc_reader, CanGc::note());
        }
    }

    ptr::null_mut()
}

struct InterfaceDoesNotMatch;

unsafe fn try_serialize<T: Serializable + IDLInterface>(
    val: SerializableInterface,
    cx: *mut JSContext,
    obj: RawHandleObject,
    global: &GlobalScope,
    w: *mut JSStructuredCloneWriter,
    writer: &mut StructuredDataWriter,
) -> Result<bool, InterfaceDoesNotMatch> {
    if let Ok(obj) = root_from_object::<T>(*obj, cx) {
        return Ok(write_object(val, global, &*obj, w, writer));
    }
    Err(InterfaceDoesNotMatch)
}

fn serialize_for_type(
    val: SerializableInterface,
) -> unsafe fn(
    SerializableInterface,
    *mut JSContext,
    RawHandleObject,
    &GlobalScope,
    *mut JSStructuredCloneWriter,
    &mut StructuredDataWriter,
) -> Result<bool, InterfaceDoesNotMatch> {
    match val {
        SerializableInterface::Blob => try_serialize::<Blob>,
    }
}

unsafe extern "C" fn write_callback(
    cx: *mut JSContext,
    w: *mut JSStructuredCloneWriter,
    obj: RawHandleObject,
    _same_process_scope_required: *mut bool,
    closure: *mut raw::c_void,
) -> bool {
    let sc_writer = &mut *(closure as *mut StructuredDataWriter);
    let in_realm_proof = AlreadyInRealm::assert_for_cx(SafeJSContext::from_ptr(cx));
    let global = GlobalScope::from_context(cx, InRealm::Already(&in_realm_proof));
    for serializable in SerializableInterface::iter() {
        let serializer = serialize_for_type(serializable);
        if let Ok(result) = serializer(serializable, cx, obj, &global, w, sc_writer) {
            return result;
        }
    }
    false
}

fn receiver_for_type(
    val: TransferrableInterface,
) -> fn(&GlobalScope, &mut StructuredDataReader, u64, RawMutableHandleObject) -> Result<(), ()> {
    match val {
        TransferrableInterface::MessagePort => receive_object::<MessagePort>,
    }
}

fn receive_object<T: Transferable>(
    owner: &GlobalScope,
    sc_reader: &mut StructuredDataReader,
    extra_data: u64,
    return_object: RawMutableHandleObject,
) -> Result<(), ()> {
    // 1. Re-build the key for the storage location
    // of the transferred object.
    let big: [u8; 8] = extra_data.to_ne_bytes();
    let (name_space, index) = big.split_at(4);

    let namespace_id = PipelineNamespaceId(u32::from_ne_bytes(
        name_space
            .try_into()
            .expect("name_space to be a slice of four."),
    ));
    let id = <T::Id as IdFromComponents>::from(
        namespace_id,
        NonZeroU32::new(u32::from_ne_bytes(
            index.try_into().expect("index to be a slice of four."),
        ))
        .expect("Index to be non-zero"),
    );

    // 2. Get the transferred object from its storage, using the key.
    let storage = T::serialized_storage(StructuredData::Reader(sc_reader));
    let serialized = if let Some(objects) = storage.as_mut() {
        let object = objects.remove(&id).expect("Transferred port to be stored");
        if objects.is_empty() {
            *storage = None;
        }
        object
    } else {
        panic!(
            "An interface was transfer-received, yet the SC holder does not have any serialized objects"
        );
    };

    if let Ok(received) = T::transfer_receive(&owner, id, serialized) {
        return_object.set(received.reflector().rootable().get());
        let storage = T::deserialized_storage(sc_reader).get_or_insert_with(Vec::new);
        storage.push(received);
        return Ok(());
    }
    Err(())
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
    let sc_reader = &mut *(closure as *mut StructuredDataReader);
    let in_realm_proof = AlreadyInRealm::assert_for_cx(SafeJSContext::from_ptr(cx));
    let owner = GlobalScope::from_context(cx, InRealm::Already(&in_realm_proof));
    for transferrable in TransferrableInterface::iter() {
        if tag == StructuredCloneTags::from(transferrable) as u32 {
            let transfer_receiver = receiver_for_type(transferrable);
            if transfer_receiver(&owner, sc_reader, extra_data, return_object).is_ok() {
                return true;
            }
        }
    }
    false
}

unsafe fn try_transfer<T: Transferable + IDLInterface>(
    interface: TransferrableInterface,
    obj: RawHandleObject,
    cx: *mut JSContext,
    sc_writer: &mut StructuredDataWriter,
    tag: *mut u32,
    ownership: *mut TransferableOwnership,
    extra_data: *mut u64,
) -> Result<(), ()> {
    if let Ok(object) = root_from_object::<T>(*obj, cx) {
        *tag = StructuredCloneTags::from(interface) as u32;
        *ownership = TransferableOwnership::SCTAG_TMO_CUSTOM;
        if let Ok((id, object)) = object.transfer() {
            // 2. Store the transferred object at a given key.
            let objects = T::serialized_storage(StructuredData::Writer(sc_writer))
                .get_or_insert_with(HashMap::new);
            objects.insert(id, object);

            let (PipelineNamespaceId(name_space), index) = id.components();
            let index = index.get();

            let mut big: [u8; 8] = [0; 8];
            let name_space = name_space.to_ne_bytes();
            let index = index.to_ne_bytes();

            let (left, right) = big.split_at_mut(4);
            left.copy_from_slice(&name_space);
            right.copy_from_slice(&index);

            // 3. Return a u64 representation of the key where the object is stored.
            *extra_data = u64::from_ne_bytes(big);
            return Ok(());
        }
    }
    Err(())
}

fn transfer_for_type(
    val: TransferrableInterface,
) -> unsafe fn(
    TransferrableInterface,
    RawHandleObject,
    *mut JSContext,
    &mut StructuredDataWriter,
    *mut u32,
    *mut TransferableOwnership,
    *mut u64,
) -> Result<(), ()> {
    match val {
        TransferrableInterface::MessagePort => try_transfer::<MessagePort>,
    }
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
    let sc_writer = &mut *(closure as *mut StructuredDataWriter);
    for transferable in TransferrableInterface::iter() {
        let try_transfer = transfer_for_type(transferable);
        if try_transfer(transferable, obj, cx, sc_writer, tag, ownership, extra_data).is_ok() {
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

pub(crate) enum StructuredData<'a> {
    Reader(&'a mut StructuredDataReader),
    Writer(&'a mut StructuredDataWriter),
}

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
