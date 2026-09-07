/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! JavaScript-engine operations required by Servo's structured-clone algorithms.
//!
//! Implementations own their engine context, target realm, and GC roots. Values
//! and objects cross this boundary only as opaque, implementation-defined tokens.

use servo_structured_clone::{
    ArrayBuffer, BigIntValue, ErrorName, ErrorStack, NodeId, PlatformValue, RegExpValue,
    SerializationMode, StructuredCloneGraph, TransferData, ViewKind, ViewLength, Wtf16String,
};

use crate::dom::bindings::error::Fallible;

/// The engine-independent result of inspecting a JavaScript language value.
#[derive(Clone, Debug)]
pub(super) enum InspectedValue<O> {
    Undefined,
    Null,
    Boolean(bool),
    Number(u64),
    BigInt(BigIntValue),
    String(Wtf16String),
    Symbol,
    Object(O),
}

/// Engine data captured from an object before Servo recursively serializes it.
#[derive(Clone, Debug)]
pub(super) enum ObjectSnapshot<V> {
    BooleanObject(bool),
    NumberObject(u64),
    BigIntObject(BigIntValue),
    StringObject(Wtf16String),
    Date(u64),
    RegExp(RegExpValue),
    Error {
        name: Option<Wtf16String>,
        message: Option<Wtf16String>,
        stack: ErrorStack,
        cause: Option<V>,
    },
    ArrayBuffer(ArrayBuffer),
    SharedArrayBuffer,
    ArrayBufferView {
        kind: ViewKind,
        buffer: V,
        byte_offset: u64,
        length: ViewLength,
    },
    Map,
    Set,
    Array {
        length: u64,
    },
    OrdinaryOrPlatform,
    Proxy,
    Callable,
    UnsupportedPlatform,
    UnsupportedInternalSlots,
}

/// Capability inputs consulted by Servo's structured serialization algorithm.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SerializationCapabilities {
    pub(super) cross_origin_isolated_capability: bool,
}

/// Whether an object can take the ArrayBuffer branch of transfer-list handling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ArrayBufferTransferEligibility {
    NotArrayBuffer,
    ArrayBuffer {
        maximum_byte_length: u64,
        resizable: bool,
    },
    SharedArrayBuffer,
}

/// Bytes moved out of an ArrayBuffer by its irreversible transfer operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ArrayBufferTransfer {
    pub(super) bytes: Vec<u8>,
    pub(super) max_byte_length: Option<u64>,
}

/// Platform transfer-holder fields produced by interface-specific transfer steps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PlatformTransferData {
    pub(super) interface: u32,
    pub(super) key: u64,
}

/// A platform object plus the values produced by its sub-serialization steps.
#[derive(Clone, Debug)]
pub(super) struct PlatformSnapshot<V> {
    pub(super) interface: u32,
    pub(super) key: u64,
    pub(super) references: Vec<V>,
}

/// An opaque platform-transfer primary-interface match.
#[derive(Clone, Copy, Debug)]
pub(super) struct PlatformTransferProbe<T> {
    pub(super) token: T,
}

/// A received platform value and an optional embedding-level transferred port.
#[derive(Debug)]
pub(super) struct PlatformReceive<V, P> {
    pub(super) value: V,
    pub(super) port: Option<P>,
}

/// An intrinsic allocation selected by Servo's deserialization algorithm.
#[derive(Clone, Copy, Debug)]
pub(super) enum Allocation<'a> {
    Undefined,
    Null,
    Boolean(bool),
    Number(u64),
    BigInt(&'a BigIntValue),
    String(&'a Wtf16String),
    BooleanObject(bool),
    NumberObject(u64),
    BigIntObject(&'a BigIntValue),
    StringObject(&'a Wtf16String),
    Date(u64),
    RegExp(&'a RegExpValue),
    Error {
        name: ErrorName,
        message: Option<&'a Wtf16String>,
        stack: &'a ErrorStack,
    },
    ArrayBuffer(&'a ArrayBuffer),
    Map,
    Set,
    Array {
        length: u64,
    },
    Object,
}

/// Engine operations used by Servo's structured serialization and
/// deserialization algorithms.
///
/// Implementations must keep every [`Self::Value`] and [`Self::Object`] token
/// alive and stable until the operation using this backend has finished.
pub(super) trait StructuredCloneJsApi {
    /// An opaque token for a rooted language value.
    type Value: Copy;

    /// An opaque token for a rooted object.
    type Object: Copy;

    /// Servo sidecar storage populated while serializing platform objects.
    type SerializationSidecar;

    /// Servo sidecar storage consumed while deserializing platform objects.
    type DeserializationSidecar;

    /// Opaque state retained between platform-transfer validation and commit.
    type PlatformTransfer: Copy;

    /// Opaque primary-interface selection retained until platform serialization.
    type PlatformSerialization: Copy;

    /// The embedding-level port returned by platform transfer-receiving steps.
    type TransferredPort;

    /// Implementation detail: inspect a rooted value without deciding which
    /// serialized record Servo will create for it.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn inspect_value(&mut self, value: Self::Value) -> Fallible<InspectedValue<Self::Object>>;

    /// Implementation detail: perform a moving-GC-safe identity lookup without
    /// changing the table.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn lookup_object(&mut self, object: Self::Object) -> Fallible<Option<NodeId>>;

    /// Implementation detail: insert one already-created record pointer into
    /// the moving-GC-safe identity table.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn insert_object(&mut self, object: Self::Object, node: NodeId) -> Fallible<()>;

    /// Implementation detail: copy engine internal-slot data into a neutral
    /// snapshot without recursively capturing Map or Set entries.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn inspect_object(&mut self, object: Self::Object) -> Fallible<ObjectSnapshot<Self::Value>>;

    /// Implementation detail: capture Map entries without observable iteration
    /// after Servo has inserted the Map into its identity table.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn snapshot_map_entries(
        &mut self,
        object: Self::Object,
    ) -> Fallible<Vec<(Self::Value, Self::Value)>>;

    /// Implementation detail: capture Set values without observable iteration
    /// after Servo has inserted the Set into its identity table.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn snapshot_set_values(&mut self, object: Self::Object) -> Fallible<Vec<Self::Value>>;

    /// Implementation detail: capture shared backing-storage metadata only
    /// after Servo has applied the capability and storage-mode policy checks.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn snapshot_shared_array_buffer(&mut self, object: Self::Object) -> Fallible<ArrayBuffer>;

    /// Implementation detail: return a stable snapshot of the object's own
    /// enumerable string keys in engine enumeration order.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn enumerable_own_keys(&mut self, object: Self::Object) -> Fallible<Vec<Wtf16String>>;

    /// Implementation detail: recheck ownership and perform the ordinary,
    /// potentially observable property get. `None` means the key disappeared.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn get_own_property(
        &mut self,
        object: Self::Object,
        key: &Wtf16String,
    ) -> Fallible<Option<Self::Value>>;

    /// Implementation detail: recognize a serializable platform object's
    /// primary interface and detached eligibility without running serialization
    /// or subserialization steps.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn recognize_platform(
        &mut self,
        object: Self::Object,
    ) -> Fallible<Option<Self::PlatformSerialization>>;

    /// Implementation detail: run the already-recognized primary interface's
    /// serialization hook without recursively serializing returned references.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn serialize_platform(
        &mut self,
        object: Self::Object,
        platform: Self::PlatformSerialization,
        mode: SerializationMode,
        sidecar: &mut Self::SerializationSidecar,
    ) -> Fallible<PlatformSnapshot<Self::Value>>;

    /// Implementation detail: distinguish ArrayBuffer, SharedArrayBuffer, and
    /// non-buffer transfer-list entries without testing detached state or
    /// otherwise changing observable state.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
    fn array_buffer_transfer_eligibility(
        &mut self,
        object: Self::Object,
    ) -> Fallible<ArrayBufferTransferEligibility>;

    /// Implementation detail: test the current detached state immediately
    /// before the algorithm retrieves the transfer data holder.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
    fn validate_array_buffer_transfer(&mut self, object: Self::Object) -> Fallible<()>;

    /// Implementation detail: recheck the buffer state for memory safety,
    /// allocate a holder, and copy the current bytes without detaching the
    /// source buffer.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
    fn copy_array_buffer_transfer(&mut self, object: Self::Object)
    -> Fallible<ArrayBufferTransfer>;

    /// Implementation detail: detach the ArrayBuffer whose current bytes were
    /// copied immediately beforehand. This must not allocate.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
    fn detach_array_buffer_transfer(&mut self, object: Self::Object) -> Fallible<()>;

    /// Implementation detail: recognize a platform transfer-list entry and
    /// retain its primary-interface token without testing its current detached
    /// state or executing its irreversible transfer steps.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
    fn probe_platform_transfer(
        &mut self,
        object: Self::Object,
    ) -> Fallible<Option<PlatformTransferProbe<Self::PlatformTransfer>>>;

    /// Implementation detail: test the platform object's current detached
    /// state immediately before the algorithm retrieves the transfer data
    /// holder.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
    fn validate_platform_transfer(
        &mut self,
        object: Self::Object,
        transfer: Self::PlatformTransfer,
    ) -> Fallible<()>;

    /// Implementation detail: execute the transfer represented by the
    /// previously validated primary-interface token and populate its Servo
    /// sidecar.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
    fn commit_platform_transfer(
        &mut self,
        object: Self::Object,
        transfer: Self::PlatformTransfer,
        sidecar: &mut Self::SerializationSidecar,
    ) -> Fallible<PlatformTransferData>;

    /// Implementation detail: allocate one engine intrinsic selected by
    /// Servo. Error causes and container contents are populated separately.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
    fn allocate(&mut self, request: Allocation<'_>) -> Fallible<Self::Value>;

    /// Implementation detail: copy transferred bytes into a new fixed or
    /// resizable ArrayBuffer in the target realm.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserializewithtransfer>
    fn receive_array_buffer_transfer(
        &mut self,
        bytes: &[u8],
        max_byte_length: Option<u64>,
    ) -> Fallible<Self::Value>;

    /// Implementation detail: create a DataView or TypedArray over an already
    /// allocated backing buffer using the supplied internal-slot values.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
    fn allocate_view(
        &mut self,
        kind: ViewKind,
        buffer: Self::Value,
        byte_offset: u64,
        length: ViewLength,
    ) -> Fallible<Self::Value>;

    /// Implementation detail: append one key/value pair directly to a native
    /// Map's internal storage without invoking user-observable Map methods. The
    /// implementation must reject a key already present under SameValueZero,
    /// since such a duplicate cannot be produced by StructuredSerializeInternal.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
    fn append_map_entry(
        &mut self,
        target: Self::Value,
        key: Self::Value,
        value: Self::Value,
    ) -> Fallible<()>;

    /// Implementation detail: append one value directly to a native Set's
    /// internal storage without invoking user-observable Set methods. The
    /// implementation must reject a value already present under SameValueZero,
    /// since such a duplicate cannot be produced by StructuredSerializeInternal.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
    fn append_set_value(&mut self, target: Self::Value, value: Self::Value) -> Fallible<()>;

    /// Implementation detail: create an own data property from an exact UTF-16
    /// key after Servo has deserialized its value.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
    fn create_data_property(
        &mut self,
        target: Self::Value,
        key: &Wtf16String,
        value: Self::Value,
    ) -> Fallible<()>;

    /// Implementation detail: attach an already deserialized cause to an Error
    /// object created without that accompanying value.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
    fn attach_error_cause(&mut self, target: Self::Value, cause: Self::Value) -> Fallible<()>;

    /// Implementation detail: create the platform object before Servo resolves
    /// its sub-deserialized references so graph identity and cycles are retained.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
    fn allocate_platform(
        &mut self,
        value: &PlatformValue,
        sidecar: &mut Self::DeserializationSidecar,
    ) -> Fallible<Self::Value>;

    /// Implementation detail: run the platform object's deserialization steps
    /// after Servo has resolved its sub-deserialized references.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
    fn populate_platform(
        &mut self,
        target: Self::Value,
        value: &PlatformValue,
        graph: &StructuredCloneGraph,
        references: &[Self::Value],
        sidecar: &mut Self::DeserializationSidecar,
    ) -> Fallible<()>;

    /// Implementation detail: run the embedding's platform transfer-receiving
    /// hooks and return both the rooted value and any separately retained port.
    ///
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserializewithtransfer>
    fn receive_platform_transfer(
        &mut self,
        data: &TransferData,
        sidecar: &mut Self::DeserializationSidecar,
    ) -> Fallible<PlatformReceive<Self::Value, Self::TransferredPort>>;
}
