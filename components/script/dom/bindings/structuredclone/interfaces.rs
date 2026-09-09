/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Servo WebIDL-interface serialization and transfer dispatch.

use js::context::JSContext;
use js::gc::{HandleObject, RootedTraceableBox};
use js::jsapi::{Heap, JSObject};
use servo_constellation_traits::{
    Serializable as SerializableInterface, Transferrable as TransferrableInterface,
};
use servo_structured_clone::{
    PlatformValue, SerializationMode, StructuredCloneGraph, TransferData,
};

use super::{StructuredDataReader, StructuredDataWriter};
use crate::dom::bindings::error::Error;
use crate::dom::globalscope::GlobalScope;

#[derive(Debug)]
pub(super) enum InterfaceError {
    InvalidInterface,
    InvalidStorageKey,
    UnexpectedTransferData,
    MissingSidecarData,
    SerializationFailed,
    DeserializationFailed,
    CannotTransfer,
    Exception(Error),
}

impl InterfaceError {
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    pub(super) fn into_dom_error(self) -> Error {
        // Implementation detail: preserve platform exceptions and map dispatcher failures to the
        // error required by the calling structured serialization algorithm.
        todo!()
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct TransferableMatch {
    pub(super) interface: TransferrableInterface,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct SerializableMatch {
    pub(super) interface: SerializableInterface,
}

pub(super) type RootedObject = RootedTraceableBox<Heap<*mut JSObject>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ReferenceShape {
    Array,
    Object,
    Other,
}

/// The realm-independent platform fields plus values selected by sub-serialization steps.
pub(super) struct InterfaceSerialization {
    pub(super) interface: u32,
    pub(super) key: u64,
    pub(super) references: Vec<RootedObject>,
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
pub(super) fn recognize_serializable(
    _cx: &mut JSContext,
    _object: HandleObject<'_>,
) -> Result<Option<SerializableMatch>, InterfaceError> {
    // Step 19. Otherwise, if value is a platform object that is a serializable object:
    // Step 19.1. If value has a [[Detached]] internal slot whose value is true, then throw a
    // "DataCloneError" DOMException.
    // Step 19.2. Let typeString be the identifier of the primary interface of value.
    todo!()
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
pub(super) fn root_object(_object: *mut JSObject) -> Result<RootedObject, InterfaceError> {
    // Implementation detail: retain a platform sub-serialization value across engine operations.
    todo!()
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
pub(super) fn serialization_steps(
    _cx: &mut JSContext,
    _object: HandleObject<'_>,
    _matched: SerializableMatch,
    _mode: SerializationMode,
    _writer: &mut StructuredDataWriter,
) -> Result<InterfaceSerialization, InterfaceError> {
    // Step 26.3. Otherwise, if value is a platform object that is a serializable object, then
    // perform the serialization steps for value's primary interface, given value, serialized, and
    // forStorage.
    todo!()
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
pub(super) fn create_deserialization_target(
    _cx: &mut JSContext,
    _owner: &GlobalScope,
    _value: &PlatformValue,
    _reader: &mut StructuredDataReader<'_>,
) -> Result<*mut JSObject, InterfaceError> {
    // Step 22.1. Let interfaceName be serialized.[[Type]].
    // Step 22.2. If the interface identified by interfaceName is not exposed in targetRealm, then
    // throw a "DataCloneError" DOMException.
    // Step 22.3. Set value to a new instance of the interface identified by interfaceName, created
    // in targetRealm.
    todo!()
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
pub(super) fn deserialization_steps(
    _cx: &mut JSContext,
    _target: *mut JSObject,
    _value: &PlatformValue,
    _graph: &StructuredCloneGraph,
    _references: &[RootedObject],
    _reference_shapes: &[ReferenceShape],
    _reader: &mut StructuredDataReader<'_>,
) -> Result<(), InterfaceError> {
    // Step 24.4.1. Perform the appropriate deserialization steps for the interface identified by
    // serialized.[[Type]], given serialized, value, and targetRealm.
    todo!()
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
pub(super) fn recognize_transfer(
    _cx: &mut JSContext,
    _object: HandleObject<'_>,
) -> Option<TransferableMatch> {
    // Step 2. For each transferable of transferList:
    // Step 2.1. If transferable has neither an [[ArrayBufferData]] internal slot nor a
    // [[Detached]] internal slot, then throw a "DataCloneError" DOMException.
    todo!()
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
pub(super) fn validate_transfer(
    _cx: &mut JSContext,
    _object: HandleObject<'_>,
    _matched: TransferableMatch,
) -> Result<(), InterfaceError> {
    // Step 5.2. If transferable has a [[Detached]] internal slot and transferable.[[Detached]] is
    // true, then throw a "DataCloneError" DOMException.
    todo!()
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
pub(super) fn transfer_steps(
    _cx: &mut JSContext,
    _object: HandleObject<'_>,
    _matched: TransferableMatch,
    _writer: &mut StructuredDataWriter,
) -> Result<TransferData, InterfaceError> {
    // Step 5.5.4. Perform the appropriate transfer steps for the interface identified by
    // interfaceName, given transferable and dataHolder.
    // Step 5.5.5. Set transferable.[[Detached]] to true.
    todo!()
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserializewithtransfer>
pub(super) fn transfer_receiving_steps(
    _cx: &mut JSContext,
    _owner: &GlobalScope,
    _data: &TransferData,
    _reader: &mut StructuredDataReader<'_>,
) -> Result<*mut JSObject, InterfaceError> {
    // Step 3.4.1. Let interfaceName be transferDataHolder.[[Type]].
    // Step 3.4.2. If the interface identified by interfaceName is not exposed in targetRealm, then
    // throw a "DataCloneError" DOMException.
    // Step 3.4.3. Set value to a new instance of the interface identified by interfaceName, created
    // in targetRealm.
    // Step 3.4.4. Perform the appropriate transfer-receiving steps for the interface identified by
    // interfaceName given transferDataHolder and value.
    todo!()
}
