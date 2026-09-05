/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

mod font_descriptor;
mod font_identifier;
mod font_template;
mod system_font_service_proxy;

use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

pub use font_descriptor::*;
pub use font_identifier::*;
pub use font_template::*;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use servo_arc::Arc as ServoArc;
use servo_base::generic_channel::GenericSharedMemory;
use style::font_face::Descriptors;
use style::stylesheets::LockedFontFaceRule;
pub use system_font_service_proxy::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WebFontLoadEvent {
    LoadedSuccessfully,
    UnblockedFontReadyPromise,
}

pub type StylesheetWebFontLoadFinishedCallback =
    Arc<dyn Fn(WebFontLoadEvent) + Send + Sync + 'static>;

/// A data structure to store data for fonts. Data is stored internally in an
/// [`GenericSharedMemory`] handle, so that it can be sent without serialization
/// across IPC channels.
#[derive(Clone, Deserialize, MallocSizeOf, Serialize)]
pub struct FontData(#[conditional_malloc_size_of] pub(crate) Arc<GenericSharedMemory>);

impl FontData {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(Arc::new(GenericSharedMemory::from_bytes(bytes)))
    }

    /// This is in single process mode more efficient because we do not have to copy the vector.
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self(Arc::new(GenericSharedMemory::from_vec(bytes)))
    }

    pub fn as_ipc_shared_memory(&self) -> Arc<GenericSharedMemory> {
        self.0.clone()
    }
}

impl AsRef<[u8]> for FontData {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Raw font data and an index
///
/// If the font data is of a TTC (TrueType collection) file, then the index of a specific font within
/// the collection. If the font data is for is single font then the index will always be 0.
#[derive(Deserialize, Clone, Serialize, MallocSizeOf)]
pub struct FontDataAndIndex {
    /// The raw font file data (.ttf, .otf, .ttc, etc)
    pub data: FontData,
    /// The index of the font within the file (0 if the file is not a ttc)
    pub index: u32,
}

#[derive(Copy, Clone, PartialEq)]
pub enum FontDataError {
    FailedToLoad,
}

/// Describes how the set of active `@font-face` rules was changed after a call to `FontContext::rebuild_font_face_set`.
#[derive(Clone, Default)]
pub struct WebFontSetDifference {
    /// A list of `@font-face` rules that were added in this update.
    pub added_font_faces: Vec<ServoArc<FontFaceRuleInfo>>,
    /// A list of `@font-face` rules that were removed in this update.
    pub removed_font_faces: Vec<ServoArc<FontFaceRuleInfo>>,
    /// Whether the cascade index of any `@font-face` rule changed during this update.
    ///
    /// This can cause different fonts to be selected during font matching.
    pub cascade_index_of_any_rule_changed: bool,
}

impl WebFontSetDifference {
    /// Returns `true` iff the font face set remained unchanged by the update.
    pub fn is_empty(&self) -> bool {
        self.added_font_faces.is_empty() && self.removed_font_faces.is_empty()
    }
}

#[derive(MallocSizeOf)]
pub struct FontFaceRuleInfo {
    /// The index of this `@font-face` in the cascade, relative to all
    /// other `@font-face` rules.
    pub cascade_index: AtomicUsize,
    /// The descriptors on the `@font-face` rule.
    pub descriptors: Descriptors,
    /// The CSS rule that created this `@font-face`.
    ///
    /// This does *not* uniquely identify this struct across updates
    /// to the set of live `@font-face` rules.
    #[conditional_malloc_size_of]
    pub rule: ServoArc<LockedFontFaceRule>,
}
