/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;

use crate::graph::{NodeId, StructuredCloneGraph, ValidationError};

pub const FORMAT_MAGIC: [u8; 8] = *b"SERVO_SC";
pub const FORMAT_VERSION: u16 = 3;

/// Classification of a structured-clone byte sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FormatProbe {
    Unrecognized,
    Supported,
    UnsupportedVersion(u16),
    TruncatedHeader,
}

/// A failure while writing the versioned graph representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EncodeError {
    InvalidGraph(ValidationError),
    InvalidTransferPlan,
    LengthOutOfRange,
    ResourceLimitExceeded,
    AllocationFailed,
}

/// A controlled failure while reading an untrusted graph representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecodeError {
    InvalidMagic,
    TruncatedHeader,
    UnsupportedVersion(u16),
    UnexpectedEof { offset: usize, needed: usize },
    InvalidTag { kind: &'static str, tag: u8 },
    LengthOutOfRange,
    ResourceLimitExceeded,
    AllocationFailed,
    TrailingBytes(usize),
    InvalidGraph(ValidationError),
}

/// The fixed wire shape and stable buffer bound prepared for one transfer data holder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferEncodingPlan {
    ArrayBuffer {
        value: NodeId,
        maximum_byte_length: u64,
        resizable: bool,
    },
    Platform {
        value: NodeId,
    },
}

/// A versioned encoding prepared before irreversible transfer steps begin.
pub struct PreparedEncoding;

impl PreparedEncoding {
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
    pub fn finish(self, _graph: &StructuredCloneGraph) -> Vec<u8> {
        // Implementation detail: finish the already-allocated versioned representation after
        // transfer data holders have been populated in specification order.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
    pub fn reserve_array_buffer_payload(
        &mut self,
        _transfer_index: usize,
        _byte_length: usize,
    ) -> Result<(), EncodeError> {
        // Implementation detail: reserve the current ArrayBuffer payload before detaching it.
        todo!()
    }
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
pub fn probe_format(_input: &[u8]) -> FormatProbe {
    // Implementation detail: reject unmarked, truncated, or unsupported wire representations.
    todo!()
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializewithtransfer>
pub fn prepare_encoding(
    _graph: &StructuredCloneGraph,
    _transfers: &[TransferEncodingPlan],
) -> Result<PreparedEncoding, EncodeError> {
    // Implementation detail: validate and preallocate the fixed wire representation before any
    // irreversible transfer step is performed.
    todo!()
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
pub fn encode(_graph: &StructuredCloneGraph) -> Result<Vec<u8>, EncodeError> {
    // Implementation detail: encode the realm-independent graph in the versioned Rust format.
    todo!()
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
pub fn decode(_input: &[u8]) -> Result<StructuredCloneGraph, DecodeError> {
    // Implementation detail: decode and validate one complete versioned graph representation.
    todo!()
}

impl fmt::Display for EncodeError {
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Implementation detail: expose controlled encoding failures without creating exceptions.
        todo!()
    }
}

impl std::error::Error for EncodeError {}

impl fmt::Display for DecodeError {
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structureddeserialize>
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Implementation detail: expose controlled decoding failures without creating exceptions.
        todo!()
    }
}

impl std::error::Error for DecodeError {}
