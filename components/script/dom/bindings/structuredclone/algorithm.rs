/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! HTML structured serialization algorithms and their required ordering.

use servo_structured_clone::{NodeId, PreparedEncoding, StructuredCloneGraph};

use super::jsapi::{SerializationCapabilities, StructuredCloneJsApi};
use crate::dom::bindings::error::Fallible;

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
pub(super) fn structured_serialize_internal<B: StructuredCloneJsApi>(
    _backend: &mut B,
    _value: B::Value,
    _for_storage: bool,
    _memory: &mut Vec<(B::Object, NodeId)>,
    _capabilities: SerializationCapabilities,
    _sidecar: &mut B::SerializationSidecar,
) -> Fallible<NodeId> {
    // Step 1. If memory was not supplied, let memory be an empty map.
    // Step 2. If memory[value] exists, then return memory[value].
    // Step 3. Let deep be false.
    // Step 4. If value is undefined, null, a Boolean, a Number, a BigInt, or a String, then return
    // { [[Type]]: "primitive", [[Value]]: value }.
    // Step 5. If value is a Symbol, then throw a "DataCloneError" DOMException.
    // Step 6. Let serialized be an uninitialized value.
    // Step 7. If value has a [[BooleanData]] internal slot, then set serialized to { [[Type]]:
    // "Boolean", [[BooleanData]]: value.[[BooleanData]] }.
    // Step 8. Otherwise, if value has a [[NumberData]] internal slot, then set serialized to {
    // [[Type]]: "Number", [[NumberData]]: value.[[NumberData]] }.
    // Step 9. Otherwise, if value has a [[BigIntData]] internal slot, then set serialized to {
    // [[Type]]: "BigInt", [[BigIntData]]: value.[[BigIntData]] }.
    // Step 10. Otherwise, if value has a [[StringData]] internal slot, then set serialized to {
    // [[Type]]: "String", [[StringData]]: value.[[StringData]] }.
    // Step 11. Otherwise, if value has a [[DateValue]] internal slot, then set serialized to {
    // [[Type]]: "Date", [[DateValue]]: value.[[DateValue]] }.
    // Step 12. Otherwise, if value has a [[RegExpMatcher]] internal slot, then set serialized to {
    // [[Type]]: "RegExp", [[RegExpMatcher]]: value.[[RegExpMatcher]], [[OriginalSource]]:
    // value.[[OriginalSource]], [[OriginalFlags]]: value.[[OriginalFlags]] }.
    // Step 13. Otherwise, if value has an [[ArrayBufferData]] internal slot:
    // Step 13.1. If IsSharedArrayBuffer(value) is true:
    // Step 13.1.1. If the current settings object's cross-origin isolated capability is false,
    // then throw a "DataCloneError" DOMException.
    // Step 13.1.2. If forStorage is true, then throw a "DataCloneError" DOMException.
    // Step 13.1.3. If value has an [[ArrayBufferMaxByteLength]] internal slot, then set serialized
    // to { [[Type]]: "GrowableSharedArrayBuffer", [[ArrayBufferData]]: value.[[ArrayBufferData]],
    // [[ArrayBufferByteLengthData]]: value.[[ArrayBufferByteLengthData]],
    // [[ArrayBufferMaxByteLength]]: value.[[ArrayBufferMaxByteLength]], [[AgentCluster]]: the
    // surrounding agent's agent cluster }.
    // Step 13.1.4. Otherwise, set serialized to { [[Type]]: "SharedArrayBuffer",
    // [[ArrayBufferData]]: value.[[ArrayBufferData]], [[ArrayBufferByteLength]]:
    // value.[[ArrayBufferByteLength]], [[AgentCluster]]: the surrounding agent's agent cluster }.
    // Step 13.2. Otherwise:
    // Step 13.2.1. If IsDetachedBuffer(value) is true, then throw a "DataCloneError"
    // DOMException.
    // Step 13.2.2. Let size be value.[[ArrayBufferByteLength]].
    // Step 13.2.3. Let dataCopy be ? CreateByteDataBlock(size).
    // Step 13.2.4. Perform CopyDataBlockBytes(dataCopy, 0, value.[[ArrayBufferData]], 0, size).
    // Step 13.2.5. If value has an [[ArrayBufferMaxByteLength]] internal slot, then set serialized
    // to { [[Type]]: "ResizableArrayBuffer", [[ArrayBufferData]]: dataCopy,
    // [[ArrayBufferByteLength]]: size, [[ArrayBufferMaxByteLength]]:
    // value.[[ArrayBufferMaxByteLength]] }.
    // Step 13.2.6. Otherwise, set serialized to { [[Type]]: "ArrayBuffer", [[ArrayBufferData]]:
    // dataCopy, [[ArrayBufferByteLength]]: size }.
    // Step 14. Otherwise, if value has a [[ViewedArrayBuffer]] internal slot:
    // Step 14.1. If IsArrayBufferViewOutOfBounds(value) is true, then throw a "DataCloneError"
    // DOMException.
    // Step 14.2. Let buffer be the value of value's [[ViewedArrayBuffer]] internal slot.
    // Step 14.3. Let bufferSerialized be ? StructuredSerializeInternal(buffer, forStorage, memory).
    // Step 14.4. Assert: bufferSerialized.[[Type]] is "ArrayBuffer", "ResizableArrayBuffer",
    // "SharedArrayBuffer", or "GrowableSharedArrayBuffer".
    // Step 14.5. If value has a [[DataView]] internal slot, then set serialized to { [[Type]]:
    // "ArrayBufferView", [[Constructor]]: "DataView", [[ArrayBufferSerialized]]:
    // bufferSerialized, [[ByteLength]]: value.[[ByteLength]], [[ByteOffset]]:
    // value.[[ByteOffset]] }.
    // Step 14.6. Otherwise:
    // Step 14.6.1. Assert: value has a [[TypedArrayName]] internal slot.
    // Step 14.6.2. Set serialized to { [[Type]]: "ArrayBufferView", [[Constructor]]:
    // value.[[TypedArrayName]], [[ArrayBufferSerialized]]: bufferSerialized, [[ByteLength]]:
    // value.[[ByteLength]], [[ByteOffset]]: value.[[ByteOffset]], [[ArrayLength]]:
    // value.[[ArrayLength]] }.
    // Step 15. Otherwise, if value has a [[MapData]] internal slot:
    // Step 15.1. Set serialized to { [[Type]]: "Map", [[MapData]]: a new empty List }.
    // Step 15.2. Set deep to true.
    // Step 16. Otherwise, if value has a [[SetData]] internal slot:
    // Step 16.1. Set serialized to { [[Type]]: "Set", [[SetData]]: a new empty List }.
    // Step 16.2. Set deep to true.
    // Step 17. Otherwise, if value has an [[ErrorData]] internal slot and value is not a platform
    // object:
    // Step 17.1. Let name be ? Get(value, "name").
    // Step 17.2. If name is not one of "Error", "EvalError", "RangeError", "ReferenceError",
    // "SyntaxError", "TypeError", or "URIError", then set name to "Error".
    // Step 17.3. Let valueMessageDesc be ? value.[[GetOwnProperty]]("message").
    // Step 17.4. Let message be undefined if IsDataDescriptor(valueMessageDesc) is false, and ?
    // ToString(valueMessageDesc.[[Value]]) otherwise.
    // Step 17.5. Let stack be an implementation-defined string that represents value.[[Stack]].
    // [JSERRORSTACKACCESSOR] [JSERRORSTACKS]
    // Step 17.6. Set serialized to { [[Type]]: "Error", [[Name]]: name, [[Message]]: message,
    // [[Stack]]: stack }.
    // Step 17.7. User agents should attach a serialized representation of any interesting
    // accompanying data which are not yet specified to serialized.
    // Step 18. Otherwise, if value is an Array exotic object:
    // Step 18.1. Let valueLenDescriptor be ? OrdinaryGetOwnProperty(value, "length").
    // Step 18.2. Let valueLen be valueLenDescriptor.[[Value]].
    // Step 18.3. Set serialized to { [[Type]]: "Array", [[Length]]: valueLen, [[Properties]]: a new
    // empty List }.
    // Step 18.4. Set deep to true.
    // Step 19. Otherwise, if value is a platform object that is a serializable object:
    // Step 19.1. If value has a [[Detached]] internal slot whose value is true, then throw a
    // "DataCloneError" DOMException.
    // Step 19.2. Let typeString be the identifier of the primary interface of value.
    // Step 19.3. Set serialized to { [[Type]]: typeString }.
    // Step 19.4. Set deep to true.
    // Step 20. Otherwise, if value is a platform object, then throw a "DataCloneError"
    // DOMException.
    // Step 21. Otherwise, if IsCallable(value) is true, then throw a "DataCloneError"
    // DOMException.
    // Step 22. Otherwise, if value has any internal slot other than [[Prototype]], [[Extensible]],
    // or [[PrivateElements]], then throw a "DataCloneError" DOMException.
    // Step 23. Otherwise, if value is an exotic object and value is not the %Object.prototype%
    // intrinsic object associated with any realm, then throw a "DataCloneError" DOMException.
    // Step 24. Otherwise:
    // Step 24.1. Set serialized to { [[Type]]: "Object", [[Properties]]: a new empty List }.
    // Step 24.2. Set deep to true.
    // Step 25. Set memory[value] to serialized.
    // Step 26. If deep is true:
    // Step 26.1. If value has a [[MapData]] internal slot:
    // Step 26.1.1. Let copiedList be a new empty List.
    // Step 26.1.2. For each Record { [[Key]], [[Value]] } entry of value.[[MapData]]:
    // Step 26.1.2.1. Let copiedEntry be a new Record { [[Key]]: entry.[[Key]], [[Value]]:
    // entry.[[Value]] }.
    // Step 26.1.2.2. If copiedEntry.[[Key]] is not the special value empty, append copiedEntry to
    // copiedList.
    // Step 26.1.3. For each Record { [[Key]], [[Value]] } entry of copiedList:
    // Step 26.1.3.1. Let serializedKey be ? StructuredSerializeInternal(entry.[[Key]], forStorage,
    // memory).
    // Step 26.1.3.2. Let serializedValue be ? StructuredSerializeInternal(entry.[[Value]],
    // forStorage, memory).
    // Step 26.1.3.3. Append { [[Key]]: serializedKey, [[Value]]: serializedValue } to
    // serialized.[[MapData]].
    // Step 26.2. Otherwise, if value has a [[SetData]] internal slot:
    // Step 26.2.1. Let copiedList be a new empty List.
    // Step 26.2.2. For each entry of value.[[SetData]]:
    // Step 26.2.2.1. If entry is not the special value empty, append entry to copiedList.
    // Step 26.2.3. For each entry of copiedList:
    // Step 26.2.3.1. Let serializedEntry be ? StructuredSerializeInternal(entry, forStorage,
    // memory).
    // Step 26.2.3.2. Append serializedEntry to serialized.[[SetData]].
    // Step 26.3. Otherwise, if value is a platform object that is a serializable object, then
    // perform the serialization steps for value's primary interface, given value, serialized, and
    // forStorage.
    // Step 26.4. Otherwise, for each key in ! EnumerableOwnProperties(value, key):
    // Step 26.4.1. If ! HasOwnProperty(value, key) is true:
    // Step 26.4.1.1. Let inputValue be ? value.[[Get]](key, value).
    // Step 26.4.1.2. Let outputValue be ? StructuredSerializeInternal(inputValue, forStorage,
    // memory).
    // Step 26.4.1.3. Append { [[Key]]: key, [[Value]]: outputValue } to
    // serialized.[[Properties]].
    // Step 27. Return serialized.
    todo!()
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserialize>
pub(super) fn structured_serialize<B: StructuredCloneJsApi>(
    _backend: &mut B,
    _value: B::Value,
    _capabilities: SerializationCapabilities,
    _sidecar: &mut B::SerializationSidecar,
) -> Fallible<(StructuredCloneGraph, PreparedEncoding)> {
    // Step 1. Return ? StructuredSerializeInternal(value, false).
    todo!()
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeforstorage>
pub(super) fn structured_serialize_for_storage<B: StructuredCloneJsApi>(
    _backend: &mut B,
    _value: B::Value,
    _capabilities: SerializationCapabilities,
    _sidecar: &mut B::SerializationSidecar,
) -> Fallible<(StructuredCloneGraph, PreparedEncoding)> {
    // Step 1. Return ? StructuredSerializeInternal(value, true).
    todo!()
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
pub(super) fn structured_deserialize<B: StructuredCloneJsApi>(
    _backend: &mut B,
    _graph: &StructuredCloneGraph,
    _sidecar: &mut B::DeserializationSidecar,
    _memory: Option<Vec<Option<B::Value>>>,
) -> Fallible<B::Value> {
    // Step 1. If memory was not supplied, let memory be an empty map.
    // Step 2. If memory[serialized] exists, then return memory[serialized].
    // Step 3. Let deep be false.
    // Step 4. Let value be an uninitialized value.
    // Step 5. If serialized.[[Type]] is "primitive", then set value to serialized.[[Value]].
    // Step 6. Otherwise, if serialized.[[Type]] is "Boolean", then set value to a new Boolean
    // object in targetRealm whose [[BooleanData]] internal slot value is
    // serialized.[[BooleanData]].
    // Step 7. Otherwise, if serialized.[[Type]] is "Number", then set value to a new Number object
    // in targetRealm whose [[NumberData]] internal slot value is serialized.[[NumberData]].
    // Step 8. Otherwise, if serialized.[[Type]] is "BigInt", then set value to a new BigInt object
    // in targetRealm whose [[BigIntData]] internal slot value is serialized.[[BigIntData]].
    // Step 9. Otherwise, if serialized.[[Type]] is "String", then set value to a new String object
    // in targetRealm whose [[StringData]] internal slot value is serialized.[[StringData]].
    // Step 10. Otherwise, if serialized.[[Type]] is "Date", then set value to a new Date object in
    // targetRealm whose [[DateValue]] internal slot value is serialized.[[DateValue]].
    // Step 11. Otherwise, if serialized.[[Type]] is "RegExp", then set value to a new RegExp
    // object in targetRealm whose [[RegExpMatcher]] internal slot value is
    // serialized.[[RegExpMatcher]], whose [[OriginalSource]] internal slot value is
    // serialized.[[OriginalSource]], and whose [[OriginalFlags]] internal slot value is
    // serialized.[[OriginalFlags]].
    // Step 12. Otherwise, if serialized.[[Type]] is "SharedArrayBuffer":
    // Step 12.1. If targetRealm's corresponding agent cluster is not serialized.[[AgentCluster]],
    // then throw a "DataCloneError" DOMException.
    // Step 12.2. Otherwise, set value to a new SharedArrayBuffer object in targetRealm whose
    // [[ArrayBufferData]] internal slot value is serialized.[[ArrayBufferData]] and whose
    // [[ArrayBufferByteLength]] internal slot value is serialized.[[ArrayBufferByteLength]].
    // Step 13. Otherwise, if serialized.[[Type]] is "GrowableSharedArrayBuffer":
    // Step 13.1. If targetRealm's corresponding agent cluster is not serialized.[[AgentCluster]],
    // then throw a "DataCloneError" DOMException.
    // Step 13.2. Otherwise, set value to a new SharedArrayBuffer object in targetRealm whose
    // [[ArrayBufferData]] internal slot value is serialized.[[ArrayBufferData]], whose
    // [[ArrayBufferByteLengthData]] internal slot value is serialized.[[ArrayBufferByteLengthData]],
    // and whose [[ArrayBufferMaxByteLength]] internal slot value is
    // serialized.[[ArrayBufferMaxByteLength]].
    // Step 14. Otherwise, if serialized.[[Type]] is "ArrayBuffer", then set value to a new
    // ArrayBuffer object in targetRealm whose [[ArrayBufferData]] internal slot value is
    // serialized.[[ArrayBufferData]], and whose [[ArrayBufferByteLength]] internal slot value is
    // serialized.[[ArrayBufferByteLength]].
    // Step 15. Otherwise, if serialized.[[Type]] is "ResizableArrayBuffer", then set value to a
    // new ArrayBuffer object in targetRealm whose [[ArrayBufferData]] internal slot value is
    // serialized.[[ArrayBufferData]], whose [[ArrayBufferByteLength]] internal slot value is
    // serialized.[[ArrayBufferByteLength]], and whose [[ArrayBufferMaxByteLength]] internal slot
    // value is serialized.[[ArrayBufferMaxByteLength]].
    // Step 16. Otherwise, if serialized.[[Type]] is "ArrayBufferView":
    // Step 16.1. Let deserializedArrayBuffer be ? StructuredDeserialize(
    // serialized.[[ArrayBufferSerialized]], targetRealm, memory).
    // Step 16.2. If serialized.[[Constructor]] is "DataView", then set value to a new DataView
    // object in targetRealm whose [[ViewedArrayBuffer]] internal slot value is
    // deserializedArrayBuffer, whose [[ByteLength]] internal slot value is
    // serialized.[[ByteLength]], and whose [[ByteOffset]] internal slot value is
    // serialized.[[ByteOffset]].
    // Step 16.3. Otherwise, set value to a new typed array object in targetRealm, using the
    // constructor given by serialized.[[Constructor]], whose [[ViewedArrayBuffer]] internal slot
    // value is deserializedArrayBuffer, whose [[TypedArrayName]] internal slot value is
    // serialized.[[Constructor]], whose [[ByteLength]] internal slot value is
    // serialized.[[ByteLength]], whose [[ByteOffset]] internal slot value is
    // serialized.[[ByteOffset]], and whose [[ArrayLength]] internal slot value is
    // serialized.[[ArrayLength]].
    // Step 17. Otherwise, if serialized.[[Type]] is "Map":
    // Step 17.1. Set value to a new Map object in targetRealm whose [[MapData]] internal slot value
    // is a new empty List.
    // Step 17.2. Set deep to true.
    // Step 18. Otherwise, if serialized.[[Type]] is "Set":
    // Step 18.1. Set value to a new Set object in targetRealm whose [[SetData]] internal slot value
    // is a new empty List.
    // Step 18.2. Set deep to true.
    // Step 19. Otherwise, if serialized.[[Type]] is "Array":
    // Step 19.1. Let outputProto be targetRealm.[[Intrinsics]].[[%Array.prototype%]].
    // Step 19.2. Set value to ! ArrayCreate(serialized.[[Length]], outputProto).
    // Step 19.3. Set deep to true.
    // Step 20. Otherwise, if serialized.[[Type]] is "Object":
    // Step 20.1. Set value to a new Object in targetRealm.
    // Step 20.2. Set deep to true.
    // Step 21. Otherwise, if serialized.[[Type]] is "Error":
    // Step 21.1. Let prototype be %Error.prototype%.
    // Step 21.2. If serialized.[[Name]] is "EvalError", then set prototype to
    // %EvalError.prototype%.
    // Step 21.3. If serialized.[[Name]] is "RangeError", then set prototype to
    // %RangeError.prototype%.
    // Step 21.4. If serialized.[[Name]] is "ReferenceError", then set prototype to
    // %ReferenceError.prototype%.
    // Step 21.5. If serialized.[[Name]] is "SyntaxError", then set prototype to
    // %SyntaxError.prototype%.
    // Step 21.6. If serialized.[[Name]] is "TypeError", then set prototype to
    // %TypeError.prototype%.
    // Step 21.7. If serialized.[[Name]] is "URIError", then set prototype to
    // %URIError.prototype%.
    // Step 21.8. Let message be serialized.[[Message]].
    // Step 21.9. Set value to OrdinaryObjectCreate(prototype, « [[ErrorData]], [[Stack]] »).
    // Step 21.10. Let messageDesc be PropertyDescriptor { [[Value]]: message, [[Writable]]: true,
    // [[Enumerable]]: false, [[Configurable]]: true }.
    // Step 21.11. If message is not undefined, then perform ! OrdinaryDefineOwnProperty(value,
    // "message", messageDesc).
    // Step 21.12. Set value.[[Stack]] to serialized.[[Stack]].
    // Step 21.13. Any interesting accompanying data attached to serialized should be deserialized
    // and attached to value.
    // Step 22. Otherwise:
    // Step 22.1. Let interfaceName be serialized.[[Type]].
    // Step 22.2. If the interface identified by interfaceName is not exposed in targetRealm, then
    // throw a "DataCloneError" DOMException.
    // Step 22.3. Set value to a new instance of the interface identified by interfaceName, created
    // in targetRealm.
    // Step 22.4. Set deep to true.
    // Step 23. Set memory[serialized] to value.
    // Step 24. If deep is true:
    // Step 24.1. If serialized.[[Type]] is "Map":
    // Step 24.1.1. For each Record { [[Key]], [[Value]] } entry of serialized.[[MapData]]:
    // Step 24.1.1.1. Let deserializedKey be ? StructuredDeserialize(entry.[[Key]], targetRealm,
    // memory).
    // Step 24.1.1.2. Let deserializedValue be ? StructuredDeserialize(entry.[[Value]],
    // targetRealm, memory).
    // Step 24.1.1.3. Append { [[Key]]: deserializedKey, [[Value]]: deserializedValue } to
    // value.[[MapData]].
    // Step 24.2. Otherwise, if serialized.[[Type]] is "Set":
    // Step 24.2.1. For each entry of serialized.[[SetData]]:
    // Step 24.2.1.1. Let deserializedEntry be ? StructuredDeserialize(entry, targetRealm, memory).
    // Step 24.2.1.2. Append deserializedEntry to value.[[SetData]].
    // Step 24.3. Otherwise, if serialized.[[Type]] is "Array" or "Object":
    // Step 24.3.1. For each Record { [[Key]], [[Value]] } entry of serialized.[[Properties]]:
    // Step 24.3.1.1. Let deserializedValue be ? StructuredDeserialize(entry.[[Value]],
    // targetRealm, memory).
    // Step 24.3.1.2. Let result be ! CreateDataProperty(value, entry.[[Key]], deserializedValue).
    // Step 24.3.1.3. Assert: result is true.
    // Step 24.4. Otherwise:
    // Step 24.4.1. Perform the appropriate deserialization steps for the interface identified by
    // serialized.[[Type]], given serialized, value, and targetRealm.
    // Step 25. Return value.
    todo!()
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
pub(super) fn structured_serialize_with_transfer<B: StructuredCloneJsApi>(
    _backend: &mut B,
    _value: B::Value,
    _transfer_list: &[B::Object],
    _capabilities: SerializationCapabilities,
    _sidecar: &mut B::SerializationSidecar,
) -> Fallible<(StructuredCloneGraph, PreparedEncoding)> {
    // Step 1. Let memory be an empty map.
    // Step 2. For each transferable of transferList:
    // Step 2.1. If transferable has neither an [[ArrayBufferData]] internal slot nor a
    // [[Detached]] internal slot, then throw a "DataCloneError" DOMException.
    // Step 2.2. If transferable has an [[ArrayBufferData]] internal slot and
    // IsSharedArrayBuffer(transferable) is true, then throw a "DataCloneError" DOMException.
    // Step 2.3. If memory[transferable] exists, then throw a "DataCloneError" DOMException.
    // Step 2.4. Set memory[transferable] to { [[Type]]: an uninitialized value }.
    // Step 3. Let serialized be ? StructuredSerializeInternal(value, false, memory).
    // Step 4. Let transferDataHolders be a new empty List.
    // Step 5. For each transferable of transferList:
    // Step 5.1. If transferable has an [[ArrayBufferData]] internal slot and
    // IsDetachedBuffer(transferable) is true, then throw a "DataCloneError" DOMException.
    // Step 5.2. If transferable has a [[Detached]] internal slot and transferable.[[Detached]] is
    // true, then throw a "DataCloneError" DOMException.
    // Step 5.3. Let dataHolder be memory[transferable].
    // Step 5.4. If transferable has an [[ArrayBufferData]] internal slot:
    // Step 5.4.1. If transferable has an [[ArrayBufferMaxByteLength]] internal slot:
    // Step 5.4.1.1. Set dataHolder.[[Type]] to "ResizableArrayBuffer".
    // Step 5.4.1.2. Set dataHolder.[[ArrayBufferData]] to transferable.[[ArrayBufferData]].
    // Step 5.4.1.3. Set dataHolder.[[ArrayBufferByteLength]] to
    // transferable.[[ArrayBufferByteLength]].
    // Step 5.4.1.4. Set dataHolder.[[ArrayBufferMaxByteLength]] to
    // transferable.[[ArrayBufferMaxByteLength]].
    // Step 5.4.2. Otherwise:
    // Step 5.4.2.1. Set dataHolder.[[Type]] to "ArrayBuffer".
    // Step 5.4.2.2. Set dataHolder.[[ArrayBufferData]] to transferable.[[ArrayBufferData]].
    // Step 5.4.2.3. Set dataHolder.[[ArrayBufferByteLength]] to
    // transferable.[[ArrayBufferByteLength]].
    // Step 5.4.3. Perform ? DetachArrayBuffer(transferable).
    // Step 5.5. Otherwise:
    // Step 5.5.1. Assert: transferable is a platform object that is a transferable object.
    // Step 5.5.2. Let interfaceName be the identifier of the primary interface of transferable.
    // Step 5.5.3. Set dataHolder.[[Type]] to interfaceName.
    // Step 5.5.4. Perform the appropriate transfer steps for the interface identified by
    // interfaceName, given transferable and dataHolder.
    // Step 5.5.5. Set transferable.[[Detached]] to true.
    // Step 5.6. Append dataHolder to transferDataHolders.
    // Step 6. Return { [[Serialized]]: serialized, [[TransferDataHolders]]:
    // transferDataHolders }.
    todo!()
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserializewithtransfer>
pub(super) fn structured_deserialize_with_transfer<B: StructuredCloneJsApi>(
    _backend: &mut B,
    _graph: StructuredCloneGraph,
    _sidecar: &mut B::DeserializationSidecar,
) -> Fallible<(B::Value, Vec<B::Value>, Vec<B::TransferredPort>)> {
    // Step 1. Let memory be an empty map.
    // Step 2. Let transferredValues be a new empty List.
    // Step 3. For each transferDataHolder of serializeWithTransferResult.[[TransferDataHolders]]:
    // Step 3.1. Let value be an uninitialized value.
    // Step 3.2. If transferDataHolder.[[Type]] is "ArrayBuffer", then set value to a new
    // ArrayBuffer object in targetRealm whose [[ArrayBufferData]] internal slot value is
    // transferDataHolder.[[ArrayBufferData]], and whose [[ArrayBufferByteLength]] internal slot
    // value is transferDataHolder.[[ArrayBufferByteLength]].
    // Step 3.3. Otherwise, if transferDataHolder.[[Type]] is "ResizableArrayBuffer", then set value
    // to a new ArrayBuffer object in targetRealm whose [[ArrayBufferData]] internal slot value is
    // transferDataHolder.[[ArrayBufferData]], whose [[ArrayBufferByteLength]] internal slot value is
    // transferDataHolder.[[ArrayBufferByteLength]], and whose [[ArrayBufferMaxByteLength]] internal
    // slot value is transferDataHolder.[[ArrayBufferMaxByteLength]].
    // Step 3.4. Otherwise:
    // Step 3.4.1. Let interfaceName be transferDataHolder.[[Type]].
    // Step 3.4.2. If the interface identified by interfaceName is not exposed in targetRealm, then
    // throw a "DataCloneError" DOMException.
    // Step 3.4.3. Set value to a new instance of the interface identified by interfaceName, created
    // in targetRealm.
    // Step 3.4.4. Perform the appropriate transfer-receiving steps for the interface identified by
    // interfaceName given transferDataHolder and value.
    // Step 3.5. Set memory[transferDataHolder] to value.
    // Step 3.6. Append value to transferredValues.
    // Step 4. Let deserialized be ? StructuredDeserialize(
    // serializeWithTransferResult.[[Serialized]], targetRealm, memory).
    // Step 5. Return { [[Deserialized]]: deserialized, [[TransferredValues]]: transferredValues }.
    todo!()
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#dom-structuredclone>
pub(super) fn structured_clone<B: StructuredCloneJsApi>(
    _backend: &mut B,
    _value: B::Value,
    _transfer: &[B::Object],
    _capabilities: SerializationCapabilities,
    _serialization_sidecar: &mut B::SerializationSidecar,
    _deserialization_sidecar: &mut B::DeserializationSidecar,
) -> Fallible<B::Value> {
    // Step 1. Let serialized be ? StructuredSerializeWithTransfer(value, options["transfer"]).
    // Step 2. Let deserializeRecord be ? StructuredDeserializeWithTransfer(serialized, this's
    // relevant realm).
    // Step 3. Return deserializeRecord.[[Deserialized]].
    todo!()
}
