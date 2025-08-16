/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

use std::sync::Arc;

use ipc_channel::ipc::IpcSharedMemory;
use malloc_size_of_derive::MallocSizeOf;
use range::{RangeIndex, int_range_index};
use serde::{Deserialize, Serialize};

int_range_index! {
    #[derive(Deserialize, MallocSizeOf, Serialize)]
    /// An index that refers to a byte offset in a text run. This could
    /// the middle of a glyph.
    struct ByteIndex(isize)
}

pub type StylesheetWebFontLoadFinishedCallback = Arc<dyn Fn(bool) + Send + Sync + 'static>;

/// A data structure to store data for fonts. Data is stored internally in an
/// [`IpcSharedMemory`] handle, so that it can be sent without serialization
/// across IPC channels.
#[derive(Clone, MallocSizeOf)]
pub struct FontData(#[conditional_malloc_size_of] pub(crate) Arc<IpcSharedMemory>);

impl FontData {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(Arc::new(IpcSharedMemory::from_bytes(bytes)))
    }

    pub fn as_ipc_shared_memory(&self) -> Arc<IpcSharedMemory> {
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
#[derive(Clone)]
pub struct FontDataAndIndex {
    /// The raw font file data (.ttf, .otf, .ttc, etc)
    pub data: FontData,
    /// The index of the font within the file (0 if the file is not a ttc)
    pub index: u32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FontDataError {
    FailedToLoad,
}
