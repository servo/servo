/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp::{max, min};
use std::ops::Range;
use std::path::PathBuf;

use embedder_traits::FilterPattern;
use ipc_channel::ipc::IpcSender;
use malloc_size_of_derive::MallocSizeOf;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::blob_url_store::{BlobBuf, BlobURLStoreError};

// HACK: Not really process-safe now, we should send Origin
//       directly instead of this in future, blocked on #11722
/// File manager store entry's origin
pub type FileOrigin = String;

/// A token modulating access to a file for a blob URL.
pub enum FileTokenCheck {
    /// Checking against a token not required,
    /// used for accessing a file
    /// that isn't linked to from a blob URL.
    NotRequired,
    /// Checking against token required.
    Required(Uuid),
    /// Request should always fail,
    /// used for cases when a check is required,
    /// but no token could be acquired.
    ShouldFail,
}

/// Relative slice positions of a sequence,
/// whose semantic should be consistent with (start, end) parameters in
/// <https://w3c.github.io/FileAPI/#dfn-slice>
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
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
            end: None,
        }
    }

    /// Instantiate optional slice position parameters
    pub fn from_opts(start: Option<i64>, end: Option<i64>) -> RelativePos {
        RelativePos {
            start: start.unwrap_or(0),
            end,
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
            },
        }
    }

    /// Compute absolute range by giving the total size
    /// <https://w3c.github.io/FileAPI/#slice-method-algo>
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
            },
            None => size,
        };

        let span: i64 = max(end - start, 0);

        Range {
            start: start.to_usize().unwrap(),
            end: (start + span).to_usize().unwrap(),
        }
    }
}

/// Response to file selection request
#[derive(Debug, Deserialize, Serialize)]
pub struct SelectedFile {
    pub id: Uuid,
    pub filename: PathBuf,
    pub modified: u64,
    pub size: u64,
    // https://w3c.github.io/FileAPI/#dfn-type
    pub type_string: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum FileManagerThreadMsg {
    /// Select a single file. Last field is pre-selected file path for testing
    SelectFile(
        Vec<FilterPattern>,
        IpcSender<FileManagerResult<SelectedFile>>,
        FileOrigin,
        Option<String>,
    ),

    /// Select multiple files. Last field is pre-selected file paths for testing
    SelectFiles(
        Vec<FilterPattern>,
        IpcSender<FileManagerResult<Vec<SelectedFile>>>,
        FileOrigin,
        Option<Vec<String>>,
    ),

    /// Read FileID-indexed file in chunks, optionally check URL validity based on boolean flag
    ReadFile(
        IpcSender<FileManagerResult<ReadFileProgress>>,
        Uuid,
        FileOrigin,
    ),

    /// Add an entry as promoted memory-based blob
    PromoteMemory(Uuid, BlobBuf, bool, FileOrigin),

    /// Add a sliced entry pointing to the parent FileID, and send back the associated FileID
    /// as part of a valid Blob URL
    AddSlicedURLEntry(
        Uuid,
        RelativePos,
        IpcSender<Result<Uuid, BlobURLStoreError>>,
        FileOrigin,
    ),

    /// Decrease reference count and send back the acknowledgement
    DecRef(Uuid, FileOrigin, IpcSender<Result<(), BlobURLStoreError>>),

    /// Activate an internal FileID so it becomes valid as part of a Blob URL
    ActivateBlobURL(Uuid, IpcSender<Result<(), BlobURLStoreError>>, FileOrigin),

    /// Revoke Blob URL and send back the acknowledgement
    RevokeBlobURL(Uuid, FileOrigin, IpcSender<Result<(), BlobURLStoreError>>),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ReadFileProgress {
    Meta(BlobBuf),
    Partial(Vec<u8>),
    EOF,
}

pub type FileManagerResult<T> = Result<T, FileManagerThreadError>;

#[derive(Debug, Deserialize, Serialize)]
pub enum FileManagerThreadError {
    /// The selection action is invalid due to exceptional reason
    InvalidSelection,
    /// The selection action is cancelled by user
    UserCancelled,
    /// Errors returned from file system request
    FileSystemError(String),
    /// Blob URL Store error
    BlobURLStoreError(BlobURLStoreError),
}
