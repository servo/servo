/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::gc::RootedVec;
use js::jsapi::{Heap, JSObject};
use rustc_hash::FxHashMap;
use servo_base::id::{
    BlobId, CryptoKeyId, DomExceptionId, DomMatrixId, DomPointId, DomQuadId, DomRectId, FileId,
    FileListId, ImageBitmapId, ImageDataId, MessagePortId, OffscreenCanvasId, QuotaExceededErrorId,
};
use servo_constellation_traits::{
    BlobImpl, DomException, DomMatrix, DomPoint, DomQuad, DomRect, MessagePortImpl,
    SerializableCryptoKey, SerializableFile, SerializableFileList, SerializableImageBitmap,
    SerializableImageData, SerializableQuotaExceededError, TransferableOffscreenCanvas,
    TransformStreamData,
};

use crate::error::Error;

pub enum StructuredData<'a, 'b> {
    Reader(&'a mut StructuredDataReader<'b>),
    Writer(&'a mut StructuredDataWriter),
}

/// Reader and writer structs for results from, and inputs to, structured-data read/write operations.
/// <https://html.spec.whatwg.org/multipage/#safe-passing-of-structured-data>
#[repr(C)]
pub struct StructuredDataReader<'a> {
    /// A error record.
    pub error: Option<Error>,
    /// Rooted copies of every deserialized object to ensure they are not garbage collected.
    pub roots: RootedVec<'a, Box<Heap<*mut JSObject>>>,
    /// A map of port implementations,
    /// used as part of the "transfer-receiving" steps of ports,
    /// to produce the DOM ports stored in `message_ports` above.
    pub port_impls: Option<FxHashMap<MessagePortId, MessagePortImpl>>,
    /// A map of transform stream implementations,
    pub transform_streams_port_impls: Option<FxHashMap<MessagePortId, TransformStreamData>>,
    /// A map of blob implementations,
    /// used as part of the "deserialize" steps of blobs,
    /// to produce the DOM blobs stored in `blobs` above.
    pub blob_impls: Option<FxHashMap<BlobId, BlobImpl>>,
    /// A map of serialized files.
    pub files: Option<FxHashMap<FileId, SerializableFile>>,
    /// A map of serialized file lists.
    pub file_lists: Option<FxHashMap<FileListId, SerializableFileList>>,
    /// A map of serialized points.
    pub points: Option<FxHashMap<DomPointId, DomPoint>>,
    /// A map of serialized rects.
    pub rects: Option<FxHashMap<DomRectId, DomRect>>,
    /// A map of serialized quads.
    pub quads: Option<FxHashMap<DomQuadId, DomQuad>>,
    /// A map of serialized matrices.
    pub matrices: Option<FxHashMap<DomMatrixId, DomMatrix>>,
    /// A map of serialized exceptions.
    pub exceptions: Option<FxHashMap<DomExceptionId, DomException>>,
    /// A map of serialized quota exceeded errors.
    pub quota_exceeded_errors:
        Option<FxHashMap<QuotaExceededErrorId, SerializableQuotaExceededError>>,
    // A map of serialized image bitmaps.
    pub image_bitmaps: Option<FxHashMap<ImageBitmapId, SerializableImageBitmap>>,
    /// A map of transferred image bitmaps.
    pub transferred_image_bitmaps: Option<FxHashMap<ImageBitmapId, SerializableImageBitmap>>,
    /// A map of transferred offscreen canvases.
    pub offscreen_canvases: Option<FxHashMap<OffscreenCanvasId, TransferableOffscreenCanvas>>,
    // A map of serialized image data.
    pub image_data: Option<FxHashMap<ImageDataId, SerializableImageData>>,
    // A map of serialized crypto keys.
    pub crypto_keys: Option<FxHashMap<CryptoKeyId, SerializableCryptoKey>>,
}

/// A data holder for transferred and serialized objects.
#[derive(Default)]
#[repr(C)]
pub struct StructuredDataWriter {
    /// Error record.
    pub error: Option<Error>,
    /// Transferred ports.
    pub ports: Option<FxHashMap<MessagePortId, MessagePortImpl>>,
    /// Transferred transform streams.
    pub transform_streams_port: Option<FxHashMap<MessagePortId, TransformStreamData>>,
    /// Serialized points.
    pub points: Option<FxHashMap<DomPointId, DomPoint>>,
    /// Serialized rects.
    pub rects: Option<FxHashMap<DomRectId, DomRect>>,
    /// Serialized quads.
    pub quads: Option<FxHashMap<DomQuadId, DomQuad>>,
    /// Serialized matrices.
    pub matrices: Option<FxHashMap<DomMatrixId, DomMatrix>>,
    /// Serialized exceptions.
    pub exceptions: Option<FxHashMap<DomExceptionId, DomException>>,
    /// Serialized quota exceeded errors.
    pub quota_exceeded_errors:
        Option<FxHashMap<QuotaExceededErrorId, SerializableQuotaExceededError>>,
    /// Serialized blobs.
    pub blobs: Option<FxHashMap<BlobId, BlobImpl>>,
    /// Serialized files.
    pub files: Option<FxHashMap<FileId, SerializableFile>>,
    /// Serialized file lists.
    pub file_lists: Option<FxHashMap<FileListId, SerializableFileList>>,
    /// Serialized image bitmaps.
    pub image_bitmaps: Option<FxHashMap<ImageBitmapId, SerializableImageBitmap>>,
    /// Transferred image bitmaps.
    pub transferred_image_bitmaps: Option<FxHashMap<ImageBitmapId, SerializableImageBitmap>>,
    /// Transferred offscreen canvases.
    pub offscreen_canvases: Option<FxHashMap<OffscreenCanvasId, TransferableOffscreenCanvas>>,
    // A map of serialized image data.
    pub image_data: Option<FxHashMap<ImageDataId, SerializableImageData>>,
    // A map of serialized crypto keys.
    pub crypto_keys: Option<FxHashMap<CryptoKeyId, SerializableCryptoKey>>,
}

/// A marker to ensure that the `[Serializable]` attribute is present on
/// types that can be serialized. This trait should not be implemented manually.
pub trait MarkedAsSerializableInIdl {
    /// Used to define compile-time assertions about the type implementing this trait.
    fn assert_serializable();
}

/// A marker to ensure that the `[Transferable]` attribute is present on
/// types that can be transferred. This trait should not be implemented manually.
pub trait MarkedAsTransferableInIdl {
    /// Used to define compile-time assertions about the type implementing this trait.
    fn assert_transferable();
}
