/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains implementations of structured data as described in
//! <https://html.spec.whatwg.org/multipage/#safe-passing-of-structured-data>

mod serializable;
mod transferable;

use std::collections::HashMap;

use base::id::{
    BlobId, DomExceptionId, DomPointId, ImageBitmapId, MessagePortId, OffscreenCanvasId,
};
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
pub use serializable::*;
use strum::IntoEnumIterator;
pub use transferable::*;

/// A data-holder for serialized data and transferred objects.
/// <https://html.spec.whatwg.org/multipage/#structuredserializewithtransfer>
#[derive(Debug, Default, Deserialize, MallocSizeOf, Serialize)]
pub struct StructuredSerializedData {
    /// Data serialized by SpiderMonkey.
    pub serialized: Vec<u8>,
    /// Serialized in a structured callback,
    pub blobs: Option<HashMap<BlobId, BlobImpl>>,
    /// Serialized point objects.
    pub points: Option<HashMap<DomPointId, DomPoint>>,
    /// Serialized exception objects.
    pub exceptions: Option<HashMap<DomExceptionId, DomException>>,
    /// Transferred objects.
    pub ports: Option<HashMap<MessagePortId, MessagePortImpl>>,
    /// Transform streams transferred objects.
    pub transform_streams: Option<HashMap<MessagePortId, TransformStreamData>>,
    /// Serialized image bitmap objects.
    pub image_bitmaps: Option<HashMap<ImageBitmapId, SerializableImageBitmap>>,
    /// Transferred image bitmap objects.
    pub transferred_image_bitmaps: Option<HashMap<ImageBitmapId, SerializableImageBitmap>>,
    /// Transferred offscreen canvas objects.
    pub offscreen_canvases: Option<HashMap<OffscreenCanvasId, TransferableOffscreenCanvas>>,
}

impl StructuredSerializedData {
    fn is_empty(&self, val: Transferrable) -> bool {
        fn is_field_empty<K, V>(field: &Option<HashMap<K, V>>) -> bool {
            field.as_ref().is_none_or(|h| h.is_empty())
        }
        match val {
            Transferrable::ImageBitmap => is_field_empty(&self.transferred_image_bitmaps),
            Transferrable::MessagePort => is_field_empty(&self.ports),
            Transferrable::OffscreenCanvas => is_field_empty(&self.offscreen_canvases),
            Transferrable::ReadableStream => is_field_empty(&self.ports),
            Transferrable::WritableStream => is_field_empty(&self.ports),
            Transferrable::TransformStream => is_field_empty(&self.ports),
        }
    }

    /// Clone all values of the same type stored in this StructuredSerializedData
    /// into another instance.
    fn clone_all_of_type<T: BroadcastClone>(&self, cloned: &mut StructuredSerializedData) {
        let existing = T::source(self);
        let Some(existing) = existing else { return };
        let mut clones = HashMap::with_capacity(existing.len());

        for (original_id, obj) in existing.iter() {
            if let Some(clone) = obj.clone_for_broadcast() {
                clones.insert(*original_id, clone);
            }
        }

        *T::destination(cloned) = Some(clones);
    }

    /// Clone the serialized data for use with broadcast-channels.
    pub fn clone_for_broadcast(&self) -> StructuredSerializedData {
        for transferrable in Transferrable::iter() {
            if !self.is_empty(transferrable) {
                // Not panicking only because this is called from the constellation.
                warn!(
                    "Attempt to broadcast structured serialized data including {:?} (should never happen).",
                    transferrable,
                );
            }
        }

        let serialized = self.serialized.clone();

        let mut cloned = StructuredSerializedData {
            serialized,
            ..Default::default()
        };

        for serializable in Serializable::iter() {
            let clone_impl = serializable.clone_values();
            clone_impl(self, &mut cloned);
        }

        cloned
    }
}
