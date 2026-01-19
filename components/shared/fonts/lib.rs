/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

mod font_descriptor;
mod font_identifier;
mod font_template;
mod system_font_service_proxy;

use std::ops::{Deref, Range};
use std::sync::Arc;

use base::generic_channel::GenericSharedMemory;
pub use font_descriptor::*;
pub use font_identifier::*;
pub use font_template::*;
use malloc_size_of_derive::MallocSizeOf;
use num_derive::{NumOps, One, Zero};
use serde::{Deserialize, Serialize};
pub use system_font_service_proxy::*;
use webrender_api::euclid::num::One;

/// An index that refers to a byte offset in a text run. This could
/// the middle of a glyph.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Eq,
    MallocSizeOf,
    NumOps,
    Ord,
    One,
    PartialEq,
    PartialOrd,
    Serialize,
    Zero,
)]
pub struct ByteIndex(pub usize);

impl ByteIndex {
    pub fn get(&self) -> usize {
        self.0
    }
}

/// A range of UTF-8 bytes in a text run. This is used to identify glyphs in a `GlyphRun`
/// by their original character byte offsets in the text.
#[derive(Clone, Debug, Default, Deserialize, MallocSizeOf, Serialize)]
pub struct TextByteRange(Range<ByteIndex>);

impl TextByteRange {
    pub fn len(&self) -> ByteIndex {
        self.0.end - self.0.start
    }

    #[inline]
    pub fn intersect(&self, other: &Self) -> Self {
        let begin = self.start.max(other.start);
        let end = self.end.min(other.end);

        if end < begin {
            Self::default()
        } else {
            Self::new(begin, end)
        }
    }

    #[inline]
    pub fn contains_inclusive(&self, index: ByteIndex) -> bool {
        index >= self.start && index <= self.end
    }
}

impl Deref for TextByteRange {
    type Target = Range<ByteIndex>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Iterator for TextByteRange {
    type Item = ByteIndex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.start == self.0.end {
            None
        } else {
            let next = self.0.start;
            self.0.start = self.0.start + ByteIndex::one();
            Some(next)
        }
    }
}

impl DoubleEndedIterator for TextByteRange {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.0.start == self.0.end {
            None
        } else {
            self.0.end = self.0.end - ByteIndex::one();
            Some(self.0.end)
        }
    }
}

impl TextByteRange {
    pub fn new(start: ByteIndex, end: ByteIndex) -> Self {
        Self(start..end)
    }

    pub fn iter(&self) -> Range<ByteIndex> {
        self.0.clone()
    }
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
