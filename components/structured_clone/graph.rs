/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;

/// Selects the HTML structured serialization operation represented by a graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SerializationMode {
    Normal,
    ForStorage,
}

/// An index into [`StructuredCloneGraph::nodes`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeId(pub u32);

/// JavaScript string data stored as UTF-16 code units, including lone surrogates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Wtf16String(pub Vec<u16>);

/// An arbitrary-precision integer stored as a sign and little-endian unsigned magnitude.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BigIntValue {
    pub negative: bool,
    pub magnitude: Vec<u8>,
}

/// A stable reference to shared backing storage held by the embedding container.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SharedBufferRef(pub u32);

/// An index into [`StructuredCloneGraph::transfers`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TransferId(pub u32);

/// The kind of an ArrayBuffer serialized record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArrayBufferKind {
    Fixed,
    Resizable { max_byte_length: u64 },
    Shared,
    GrowableShared { max_byte_length: u64 },
}

/// Backing storage owned by the graph or referenced from its embedding container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArrayBufferStorage {
    Bytes(Vec<u8>),
    Shared(SharedBufferRef),
    Transferred(TransferId),
}

/// A fixed, resizable, shared, or growable shared ArrayBuffer record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArrayBuffer {
    pub kind: ArrayBufferKind,
    pub byte_length: u64,
    pub storage: ArrayBufferStorage,
}

/// A TypedArray constructor name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypedArrayKind {
    Int8,
    Uint8,
    Uint8Clamped,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Float16,
    Float32,
    Float64,
    BigInt64,
    BigUint64,
}

/// The constructor used to recreate an ArrayBuffer view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewKind {
    DataView,
    TypedArray(TypedArrayKind),
}

/// Whether an ArrayBuffer view has a fixed or length-tracking extent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewLength {
    Fixed {
        byte_length: u64,
        array_length: Option<u64>,
    },
    LengthTracking,
}

/// A serialized DataView or TypedArray descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArrayBufferView {
    pub kind: ViewKind,
    pub buffer: NodeId,
    pub byte_offset: u64,
    pub length: ViewLength,
}

/// A serialized own enumerable string-keyed property.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Property {
    pub key: Wtf16String,
    pub value: NodeId,
}

/// A serialized Map entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MapEntry {
    pub key: NodeId,
    pub value: NodeId,
}

/// The standard Error constructor selected during deserialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorName {
    Error,
    EvalError,
    RangeError,
    ReferenceError,
    SyntaxError,
    TypeError,
    UriError,
}

/// The implementation-defined Error stack string.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErrorStack {
    pub string: Wtf16String,
}

/// The implementation-defined and standardized fields retained for an Error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErrorValue {
    pub name: ErrorName,
    pub message: Option<Wtf16String>,
    pub stack: ErrorStack,
    pub cause: Option<NodeId>,
}

/// The source and flags needed to recreate a RegExp.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegExpValue {
    pub source: Wtf16String,
    pub flags: Wtf16String,
}

/// An embedding-defined serializable platform-object record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformValue {
    pub interface: u32,
    pub key: u64,
    pub references: Vec<NodeId>,
}

/// Data held while an entry of a transfer list is moved to another realm.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransferData {
    ArrayBuffer {
        bytes: Vec<u8>,
        max_byte_length: Option<u64>,
    },
    Platform {
        interface: u32,
        key: u64,
    },
}

/// A graph placeholder and its transfer data holder, retained in transfer-list order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferRecord {
    pub value: NodeId,
    pub data: TransferData,
}

/// A realm-independent structured serialization record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Node {
    Undefined,
    Null,
    Boolean(bool),
    Number(u64),
    BigInt(BigIntValue),
    String(Wtf16String),
    BooleanObject(bool),
    NumberObject(u64),
    BigIntObject(BigIntValue),
    StringObject(Wtf16String),
    Date(u64),
    RegExp(RegExpValue),
    Error(ErrorValue),
    ArrayBuffer(ArrayBuffer),
    ArrayBufferView(ArrayBufferView),
    Map(Vec<MapEntry>),
    Set(Vec<NodeId>),
    Array {
        length: u64,
        properties: Vec<Property>,
    },
    Object(Vec<Property>),
    Platform(PlatformValue),
}

/// The arena graph produced by structured serialization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuredCloneGraph {
    pub mode: SerializationMode,
    pub root: NodeId,
    pub nodes: Vec<Node>,
    pub transfers: Vec<TransferRecord>,
}

/// Failure to add another addressable record to a graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphSizeError {
    TooManyNodes,
    AllocationFailed,
}

/// A violation of the invariants required by the versioned graph format.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationError {
    AllocationFailed,
    TooManyNodes,
    TooManyTransfers,
    MissingRoot(NodeId),
    MissingReference { from: NodeId, target: NodeId },
    NonCanonicalBigInt(NodeId),
    ByteLengthMismatch(NodeId),
    InvalidResizableLength(NodeId),
    InvalidBufferStorage(NodeId),
    SharedBufferForStorage(NodeId),
    TransferForStorage,
    DuplicateTransfer(NodeId),
    MissingTransfer { from: NodeId, transfer: TransferId },
    MissingTransferValue { transfer: TransferId, value: NodeId },
    InvalidTransfer { from: NodeId, transfer: TransferId },
    InvalidRegExpFlags(NodeId),
    InvalidDateValue(NodeId),
    InvalidViewBuffer { view: NodeId, buffer: NodeId },
    InvalidViewLength(NodeId),
    ViewOutOfBounds(NodeId),
    InvalidArrayLength(NodeId),
    DuplicateProperty(NodeId),
    InvalidPropertyOrder(NodeId),
    InvalidArrayProperty(NodeId),
    UnreachableNode(NodeId),
}

impl StructuredCloneGraph {
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    pub fn push(&mut self, _node: Node) -> Result<NodeId, GraphSizeError> {
        // Implementation detail: add one addressable record to the graph arena.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    pub fn get(&self, _id: NodeId) -> Option<&Node> {
        // Implementation detail: resolve a record pointer through the graph arena.
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    pub fn get_mut(&mut self, _id: NodeId) -> Option<&mut Node> {
        // Implementation detail: mutably resolve a record pointer through the graph arena.
        todo!()
    }
}

/// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
pub fn validate(_graph: &StructuredCloneGraph) -> Result<(), ValidationError> {
    // Implementation detail: validate all graph, identity, ordering, buffer, and transfer
    // invariants before encoding or deserializing an untrusted representation.
    todo!()
}

impl fmt::Display for ValidationError {
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Implementation detail: expose validation failures without creating an exception.
        todo!()
    }
}

impl std::error::Error for ValidationError {}

impl fmt::Display for GraphSizeError {
    /// <https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal>
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Implementation detail: expose graph-allocation failures without creating an exception.
        todo!()
    }
}

impl std::error::Error for GraphSizeError {}
