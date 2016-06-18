/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use blob_url_store::{BlobURLStoreEntry, BlobURLStoreError};
use ipc_channel::ipc::IpcSender;
use num_traits::ToPrimitive;
use std::cmp::{max, min};
use std::path::PathBuf;
use super::{LoadConsumer, LoadData};

// HACK: We should send Origin directly instead of this in future, blocked on #11722
pub type FileOrigin = String;

#[derive(Clone, Deserialize, Serialize)]
pub struct RelativePos {
    pub start: i64,
    pub end: Option<i64>,
}

impl RelativePos {
    pub fn full() -> RelativePos {
        RelativePos {
            start: 0,
            end: None,
        }
    }

    pub fn from(start: Option<i64>, end: Option<i64>) -> RelativePos {
        RelativePos {
            start: start.unwrap_or(0),
            end: end,
        }
    }

    pub fn recalculate(&self, rel_pos: &RelativePos) -> RelativePos {
        RelativePos {
            start: self.start + rel_pos.start,
            end: match (self.end, rel_pos.end) {
                (Some(old_end), Some(rel_end)) => Some(old_end + rel_end),
                (old, None) => old,
                (None, rel) => rel,
            }
        }
    }
}

/// Compute slice position
/// https://w3c.github.io/FileAPI/#slice-method-algo
pub fn compute_slice_pos(start: Option<i64>, end: Option<i64>, size: usize) -> (usize, usize) {
    let size = size as i64;

    let relative_start: i64 = match start {
        None => 0,
        Some(start) => {
            if start < 0 {
                max(size + start, 0)
            } else {
                min(start, size)
            }
        }
    };
    let relative_end: i64 = match end {
        None => size,
        Some(end) => {
            if end < 0 {
                max(size + end, 0)
            } else {
                min(end, size)
            }
        }
    };

    let span: i64 = max(relative_end - relative_start, 0);
    let start = relative_start.to_usize().unwrap();
    let end = (relative_start + span).to_usize().unwrap();

    (start, end)
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

    /// Delete the FileID entry
    DeleteFileID(SelectedFileId, FileOrigin),

    /// Load resource by Blob URL
    LoadBlob(LoadData, LoadConsumer),

    /// Add an entry and send back the associated uuid
    TransferMemory(BlobURLStoreEntry, RelativePos, IpcSender<Result<SelectedFileId, BlobURLStoreError>>, FileOrigin),

    /// Add an indirect entry pointing to the parent id with a relative slicing positing
    AddIndirectEntry(SelectedFileId, RelativePos, IpcSender<Result<SelectedFileId, BlobURLStoreError>>, FileOrigin),

    /// Increment reference count
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
