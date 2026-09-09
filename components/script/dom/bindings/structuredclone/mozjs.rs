/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! mozjs adapter for Servo's engine-neutral structured-clone API.

use std::marker::PhantomData;

use js::context::JSContext;
use js::gc::RootedVec;
use js::jsval::JSVal;
use servo_structured_clone::{
    ArrayBuffer, NodeId, PlatformValue, SerializationMode, StructuredCloneGraph, TransferData,
    ViewKind, ViewLength, Wtf16String,
};

use super::jsapi::{
    Allocation, ArrayBufferTransfer, ArrayBufferTransferEligibility, InspectedValue,
    ObjectSnapshot, PlatformReceive, PlatformSnapshot, PlatformTransferData, PlatformTransferProbe,
    StructuredCloneJsApi,
};
use super::{StructuredDataReader, StructuredDataWriter, interfaces};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageport::MessagePort;

#[derive(Clone, Copy, Debug)]
pub(super) struct ValueToken(u32);

#[derive(Clone, Copy, Debug)]
pub(super) struct ObjectToken(ValueToken);

pub(super) struct MozJs<'cx, 'values, 'roots, 'global, 'reader> {
    marker: PhantomData<(
        &'cx mut JSContext,
        &'values mut RootedVec<'roots, JSVal>,
        &'global GlobalScope,
        &'reader (),
    )>,
}

impl<'cx, 'values, 'roots> MozJs<'cx, 'values, 'roots, 'static, 'static> {
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    pub(super) fn for_serialization(
        _cx: &'cx mut JSContext,
        _values: &'values mut RootedVec<'roots, JSVal>,
    ) -> Fallible<Self> {
        // Implementation detail: retain all opaque engine values and moving-GC-safe object
        // identities for the duration of one serialization operation.
        todo!()
    }
}

impl<'cx, 'values, 'roots, 'global, 'reader> MozJs<'cx, 'values, 'roots, 'global, 'reader> {
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
    pub(super) fn for_deserialization(
        _cx: &'cx mut JSContext,
        _values: &'values mut RootedVec<'roots, JSVal>,
        _global: &'global GlobalScope,
    ) -> Self {
        // Implementation detail: retain the target realm and all allocated engine values for one
        // deserialization operation.
        todo!()
    }
}

impl<'cx, 'values, 'roots, 'global, 'reader> StructuredCloneJsApi
    for MozJs<'cx, 'values, 'roots, 'global, 'reader>
{
    type Value = ValueToken;
    type Object = ObjectToken;
    type SerializationSidecar = StructuredDataWriter;
    type DeserializationSidecar = StructuredDataReader<'reader>;
    type PlatformTransfer = interfaces::TransferableMatch;
    type PlatformSerialization = interfaces::SerializableMatch;
    type TransferredPort = DomRoot<MessagePort>;

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn inspect_value(&mut self, _value: Self::Value) -> Fallible<InspectedValue<Self::Object>> {
        // Step 4. If value is undefined, null, a Boolean, a Number, a BigInt, or a String, then
        // return { [[Type]]: "primitive", [[Value]]: value }.
        // Step 5. If value is a Symbol, then throw a "DataCloneError" DOMException.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn lookup_object(&mut self, _object: Self::Object) -> Fallible<Option<NodeId>> {
        // Step 2. If memory[value] exists, then return memory[value].
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn insert_object(&mut self, _object: Self::Object, _node: NodeId) -> Fallible<()> {
        // Step 25. Set memory[value] to serialized.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn inspect_object(&mut self, _object: Self::Object) -> Fallible<ObjectSnapshot<Self::Value>> {
        // Step 6. Let serialized be an uninitialized value.
        // Step 7. If value has a [[BooleanData]] internal slot, then set serialized to { [[Type]]:
        // "Boolean", [[BooleanData]]: value.[[BooleanData]] }.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn snapshot_map_entries(
        &mut self,
        _object: Self::Object,
    ) -> Fallible<Vec<(Self::Value, Self::Value)>> {
        // Step 26.1.1. Let copiedList be a new empty List.
        // Step 26.1.2. For each Record { [[Key]], [[Value]] } entry of value.[[MapData]]:
        // Step 26.1.2.1. Let copiedEntry be a new Record { [[Key]]: entry.[[Key]], [[Value]]:
        // entry.[[Value]] }.
        // Step 26.1.2.2. If copiedEntry.[[Key]] is not the special value empty, append copiedEntry
        // to copiedList.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn snapshot_set_values(&mut self, _object: Self::Object) -> Fallible<Vec<Self::Value>> {
        // Step 26.2.1. Let copiedList be a new empty List.
        // Step 26.2.2. For each entry of value.[[SetData]]:
        // Step 26.2.2.1. If entry is not the special value empty, append entry to copiedList.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn snapshot_shared_array_buffer(&mut self, _object: Self::Object) -> Fallible<ArrayBuffer> {
        // Step 13.1.3. If value has an [[ArrayBufferMaxByteLength]] internal slot, then set
        // serialized to { [[Type]]: "GrowableSharedArrayBuffer", [[ArrayBufferData]]:
        // value.[[ArrayBufferData]], [[ArrayBufferByteLengthData]]:
        // value.[[ArrayBufferByteLengthData]], [[ArrayBufferMaxByteLength]]:
        // value.[[ArrayBufferMaxByteLength]], [[AgentCluster]]: the surrounding agent's agent
        // cluster }.
        // Step 13.1.4. Otherwise, set serialized to { [[Type]]: "SharedArrayBuffer",
        // [[ArrayBufferData]]: value.[[ArrayBufferData]], [[ArrayBufferByteLength]]:
        // value.[[ArrayBufferByteLength]], [[AgentCluster]]: the surrounding agent's agent
        // cluster }.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn enumerable_own_keys(&mut self, _object: Self::Object) -> Fallible<Vec<Wtf16String>> {
        // Step 26.4. Otherwise, for each key in ! EnumerableOwnProperties(value, key):
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn get_own_property(
        &mut self,
        _object: Self::Object,
        _key: &Wtf16String,
    ) -> Fallible<Option<Self::Value>> {
        // Step 26.4.1. If ! HasOwnProperty(value, key) is true:
        // Step 26.4.1.1. Let inputValue be ? value.[[Get]](key, value).
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn recognize_platform(
        &mut self,
        _object: Self::Object,
    ) -> Fallible<Option<Self::PlatformSerialization>> {
        // Step 19. Otherwise, if value is a platform object that is a serializable object:
        // Step 19.1. If value has a [[Detached]] internal slot whose value is true, then throw a
        // "DataCloneError" DOMException.
        // Step 19.2. Let typeString be the identifier of the primary interface of value.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn serialize_platform(
        &mut self,
        _object: Self::Object,
        _platform: Self::PlatformSerialization,
        _mode: SerializationMode,
        _sidecar: &mut Self::SerializationSidecar,
    ) -> Fallible<PlatformSnapshot<Self::Value>> {
        // Step 26.3. Otherwise, if value is a platform object that is a serializable object, then
        // perform the serialization steps for value's primary interface, given value, serialized,
        // and forStorage.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
    fn array_buffer_transfer_eligibility(
        &mut self,
        _object: Self::Object,
    ) -> Fallible<ArrayBufferTransferEligibility> {
        // Step 2.1. If transferable has neither an [[ArrayBufferData]] internal slot nor a
        // [[Detached]] internal slot, then throw a "DataCloneError" DOMException.
        // Step 2.2. If transferable has an [[ArrayBufferData]] internal slot and
        // IsSharedArrayBuffer(transferable) is true, then throw a "DataCloneError" DOMException.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
    fn validate_array_buffer_transfer(&mut self, _object: Self::Object) -> Fallible<()> {
        // Step 5.1. If transferable has an [[ArrayBufferData]] internal slot and
        // IsDetachedBuffer(transferable) is true, then throw a "DataCloneError" DOMException.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
    fn copy_array_buffer_transfer(
        &mut self,
        _object: Self::Object,
    ) -> Fallible<ArrayBufferTransfer> {
        // Step 5.4.1.2. Set dataHolder.[[ArrayBufferData]] to transferable.[[ArrayBufferData]].
        // Step 5.4.1.3. Set dataHolder.[[ArrayBufferByteLength]] to
        // transferable.[[ArrayBufferByteLength]].
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
    fn detach_array_buffer_transfer(&mut self, _object: Self::Object) -> Fallible<()> {
        // Step 5.4.3. Perform ? DetachArrayBuffer(transferable).
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
    fn probe_platform_transfer(
        &mut self,
        _object: Self::Object,
    ) -> Fallible<Option<PlatformTransferProbe<Self::PlatformTransfer>>> {
        // Step 2.1. If transferable has neither an [[ArrayBufferData]] internal slot nor a
        // [[Detached]] internal slot, then throw a "DataCloneError" DOMException.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
    fn validate_platform_transfer(
        &mut self,
        _object: Self::Object,
        _transfer: Self::PlatformTransfer,
    ) -> Fallible<()> {
        // Step 5.2. If transferable has a [[Detached]] internal slot and transferable.[[Detached]]
        // is true, then throw a "DataCloneError" DOMException.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
    fn commit_platform_transfer(
        &mut self,
        _object: Self::Object,
        _transfer: Self::PlatformTransfer,
        _sidecar: &mut Self::SerializationSidecar,
    ) -> Fallible<PlatformTransferData> {
        // Step 5.5.4. Perform the appropriate transfer steps for the interface identified by
        // interfaceName, given transferable and dataHolder.
        // Step 5.5.5. Set transferable.[[Detached]] to true.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
    fn allocate(&mut self, _request: Allocation<'_>) -> Fallible<Self::Value> {
        // Step 4. Let value be an uninitialized value.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserializewithtransfer>
    fn receive_array_buffer_transfer(
        &mut self,
        _bytes: &[u8],
        _max_byte_length: Option<u64>,
    ) -> Fallible<Self::Value> {
        // Step 3.2. If transferDataHolder.[[Type]] is "ArrayBuffer", then set value to a new
        // ArrayBuffer object in targetRealm whose [[ArrayBufferData]] internal slot value is
        // transferDataHolder.[[ArrayBufferData]], and whose [[ArrayBufferByteLength]] internal slot
        // value is transferDataHolder.[[ArrayBufferByteLength]].
        // Step 3.3. Otherwise, if transferDataHolder.[[Type]] is "ResizableArrayBuffer", then set
        // value to a new ArrayBuffer object in targetRealm whose [[ArrayBufferData]] internal slot
        // value is transferDataHolder.[[ArrayBufferData]], whose [[ArrayBufferByteLength]] internal
        // slot value is transferDataHolder.[[ArrayBufferByteLength]], and whose
        // [[ArrayBufferMaxByteLength]] internal slot value is
        // transferDataHolder.[[ArrayBufferMaxByteLength]].
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
    fn allocate_view(
        &mut self,
        _kind: ViewKind,
        _buffer: Self::Value,
        _byte_offset: u64,
        _length: ViewLength,
    ) -> Fallible<Self::Value> {
        // Step 16.2. If serialized.[[Constructor]] is "DataView", then set value to a new DataView
        // object in targetRealm whose [[ViewedArrayBuffer]] internal slot value is
        // deserializedArrayBuffer, whose [[ByteLength]] internal slot value is
        // serialized.[[ByteLength]], and whose [[ByteOffset]] internal slot value is
        // serialized.[[ByteOffset]].
        // Step 16.3. Otherwise, set value to a new typed array object in targetRealm, using the
        // constructor given by serialized.[[Constructor]], whose [[ViewedArrayBuffer]] internal
        // slot value is deserializedArrayBuffer, whose [[TypedArrayName]] internal slot value is
        // serialized.[[Constructor]], whose [[ByteLength]] internal slot value is
        // serialized.[[ByteLength]], whose [[ByteOffset]] internal slot value is
        // serialized.[[ByteOffset]], and whose [[ArrayLength]] internal slot value is
        // serialized.[[ArrayLength]].
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
    fn append_map_entry(
        &mut self,
        _target: Self::Value,
        _key: Self::Value,
        _value: Self::Value,
    ) -> Fallible<()> {
        // Step 24.1.1.3. Append { [[Key]]: deserializedKey, [[Value]]: deserializedValue } to
        // value.[[MapData]].
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
    fn append_set_value(&mut self, _target: Self::Value, _value: Self::Value) -> Fallible<()> {
        // Step 24.2.1.2. Append deserializedEntry to value.[[SetData]].
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
    fn create_data_property(
        &mut self,
        _target: Self::Value,
        _key: &Wtf16String,
        _value: Self::Value,
    ) -> Fallible<()> {
        // Step 24.3.1.2. Let result be ! CreateDataProperty(value, entry.[[Key]],
        // deserializedValue).
        // Step 24.3.1.3. Assert: result is true.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
    fn attach_error_cause(&mut self, _target: Self::Value, _cause: Self::Value) -> Fallible<()> {
        // Step 21.13. Any interesting accompanying data attached to serialized should be
        // deserialized and attached to value.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
    fn allocate_platform(
        &mut self,
        _value: &PlatformValue,
        _sidecar: &mut Self::DeserializationSidecar,
    ) -> Fallible<Self::Value> {
        // Step 22.3. Set value to a new instance of the interface identified by interfaceName,
        // created in targetRealm.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
    fn populate_platform(
        &mut self,
        _target: Self::Value,
        _value: &PlatformValue,
        _graph: &StructuredCloneGraph,
        _references: &[Self::Value],
        _sidecar: &mut Self::DeserializationSidecar,
    ) -> Fallible<()> {
        // Step 24.4.1. Perform the appropriate deserialization steps for the interface identified
        // by serialized.[[Type]], given serialized, value, and targetRealm.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserializewithtransfer>
    fn receive_platform_transfer(
        &mut self,
        _data: &TransferData,
        _sidecar: &mut Self::DeserializationSidecar,
    ) -> Fallible<PlatformReceive<Self::Value, Self::TransferredPort>> {
        // Step 3.4.4. Perform the appropriate transfer-receiving steps for the interface identified
        // by interfaceName given transferDataHolder and value.
        todo!()
    }
}
