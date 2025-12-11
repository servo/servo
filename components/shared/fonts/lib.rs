/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

mod font_descriptor;
mod font_identifier;
mod font_template;
mod system_font_service_proxy;

use std::sync::Arc;

use base::generic_channel::GenericSharedMemory;
pub use font_descriptor::*;
pub use font_identifier::*;
pub use font_template::*;
use malloc_size_of_derive::MallocSizeOf;
use range::{RangeIndex, int_range_index};
use serde::{Deserialize, Serialize};
pub use system_font_service_proxy::*;

int_range_index! {
    #[derive(Deserialize, MallocSizeOf, Serialize)]
    /// An index that refers to a byte offset in a text run. This could
    /// the middle of a glyph.
    struct ByteIndex(isize)
}

pub type StylesheetWebFontLoadFinishedCallback = Arc<dyn Fn(bool) + Send + Sync + 'static>;

/// A data structure to store data for fonts. Data is stored internally in an
/// [`GenericSharedMemory`] handle, so that it can be sent without serialization
/// across IPC channels.
#[derive(Clone, Deserialize, MallocSizeOf, Serialize)]
pub struct FontData(#[conditional_malloc_size_of] pub(crate) Arc<GenericSharedMemory>);

impl FontData {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(Arc::new(GenericSharedMemory::from_bytes(bytes)))
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
#[derive(Deserialize, Clone, Serialize)]
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
