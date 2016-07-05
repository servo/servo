/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use blob_url_store::{BlobURLStoreEntry, BlobURLStoreError};
use ipc_channel::ipc::IpcSender;
use num_traits::ToPrimitive;
use std::cmp::{max, min};
use std::ops::Range;
use std::path::PathBuf;
use super::{LoadConsumer, LoadData};

// HACK: We should send Origin directly instead of this in future, blocked on #11722
/// File manager store entry's origin
pub type FileOrigin = String;

/// Relative slice positions of a sequence,
/// whose semantic should be consistent with (start, end) parameters in
/// https://w3c.github.io/FileAPI/#dfn-slice
#[derive(Clone, Deserialize, Serialize)]
pub struct RelativePos {
    /// Relative to first byte if non-negative,
    /// relative to one past last byte if negative,
    pub start: i64,
    /// Relative offset from first byte if Some(non-negative),
    /// relative to one past last byte if Some(negative),
    /// None if one past last byte
    pub end: Option<i64>,
}

impl RelativePos {
    /// Full range from start to end
    pub fn full_range() -> RelativePos {
        RelativePos {
            start: 0,
            end: Some(0),
        }
    }

    /// Instantiate optional slice position parameters
    pub fn from_opts(start: Option<i64>, end: Option<i64>) -> RelativePos {
        RelativePos {
            start: start.unwrap_or(0),
            end: end,
        }
    }

    /// Slice the inner sliced range by repositioning
    pub fn slice_inner(&self, rel_pos: &RelativePos) -> RelativePos {
        RelativePos {
            start: self.start + rel_pos.start,
            end: match (self.end, rel_pos.end) {
                (Some(old_end), Some(rel_end)) => Some(old_end + rel_end),
                (old, None) => old,
                (None, rel) => rel,
            }
        }
    }

    /// Compute absolute range by giving the total size
    /// https://w3c.github.io/FileAPI/#slice-method-algo
    pub fn to_abs_range(&self, size: usize) -> Range<usize> {
        let size = size as i64;

        let start = {
            if self.start < 0 {
                max(size + self.start, 0)
            } else {
                min(self.start, size)
            }
        };

        let end = match self.end {
            Some(rel_end) => {
                if rel_end < 0 {
                    max(size + rel_end, 0)
                } else {
                    min(rel_end, size)
                }
            }
            None => size,
        };

        let span: i64 = max(end - start, 0);

        Range {
            start: start.to_usize().unwrap(),
            end: (start + span).to_usize().unwrap(),
        }
    }

    /// Inverse operation of to_abs_range
    pub fn from_abs_range(range: Range<usize>, size: usize) -> RelativePos {
        RelativePos {
            start: range.start as i64,
            end: Some(size as i64 - range.end as i64),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SelectedFileId(pub String);

#[derive(Debug, Deserialize, Serialize)]
pub struct SelectedFile {
    pub id: SelectedFileId,
    pub filename: PathBuf,
    pub modified: u64,
    // https://w3c.github.io/FileAPI/#dfn-type
    pub type_string: String,
}

/// Filter for file selection
/// the content is expected to be extension (e.g, "doc", without the prefixing ".")
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FilterPattern(pub String);

#[derive(Deserialize, Serialize)]
pub enum FileManagerThreadMsg {
    /// Select a single file, return triple (FileID, FileName, lastModified)
    SelectFile(Vec<FilterPattern>, IpcSender<FileManagerResult<SelectedFile>>, FileOrigin),

    /// Select multiple files, return a vector of triples
    SelectFiles(Vec<FilterPattern>, IpcSender<FileManagerResult<Vec<SelectedFile>>>, FileOrigin),

    /// Read file, return the bytes
    ReadFile(IpcSender<FileManagerResult<Vec<u8>>>, SelectedFileId, FileOrigin),

    /// Load resource by Blob URL
    LoadBlob(LoadData, LoadConsumer),

    /// Add an entry and send back the associated uuid
    TransferMemory(BlobURLStoreEntry, IpcSender<Result<SelectedFileId, BlobURLStoreError>>, FileOrigin),

    /// Add a sliced entry pointing to the parent id with a relative slicing positing
    AddSlicedEntry(SelectedFileId, RelativePos, IpcSender<Result<SelectedFileId, BlobURLStoreError>>, FileOrigin),

    /// Decrease reference count
    DecRef(SelectedFileId, FileOrigin, IpcSender<Result<(), BlobURLStoreError>>),

    /// Increase reference count
    IncRef(SelectedFileId, FileOrigin),

    /// Shut down this thread
    Exit,
}

pub type FileManagerResult<T> = Result<T, FileManagerThreadError>;

#[derive(Debug, Deserialize, Serialize)]
pub enum FileManagerThreadError {
    /// The selection action is invalid due to exceptional reason
    InvalidSelection,
    /// The selection action is cancelled by user
    UserCancelled,
    /// Failure to process file information such as file name, modified time etc.
    FileInfoProcessingError,
    /// Failure to read the file content
    ReadFileError,
}
