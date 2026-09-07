/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![forbid(unsafe_code)]

//! An engine-independent representation of the HTML structured serialization format.

mod codec;
mod graph;

pub use codec::{
    DecodeError, EncodeError, FORMAT_MAGIC, FORMAT_VERSION, FormatProbe, PreparedEncoding,
    TransferEncodingPlan, decode, encode, prepare_encoding, probe_format,
};
pub use graph::{
    ArrayBuffer, ArrayBufferKind, ArrayBufferStorage, ArrayBufferView, BigIntValue, ErrorName,
    ErrorStack, ErrorValue, GraphSizeError, MapEntry, Node, NodeId, PlatformValue, Property,
    RegExpValue, SerializationMode, SharedBufferRef, StructuredCloneGraph, TransferData,
    TransferId, TransferRecord, TypedArrayKind, ValidationError, ViewKind, ViewLength, Wtf16String,
    validate,
};
