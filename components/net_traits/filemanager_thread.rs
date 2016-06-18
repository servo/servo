/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use blob_url_store::{BlobURLStoreEntry, BlobURLStoreError};
use ipc_channel::ipc::IpcSender;
use std::path::PathBuf;
use super::{LoadConsumer, LoadData};

#[derive(Clone, Deserialize, Serialize)]
pub struct RelativePos {
    pub start: Option<i64>,
    pub end: Option<i64>,
}

impl RelativePos {
    pub fn full() -> RelativePos {
        RelativePos {
            start: None,
            end: None,
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
    SelectFile(Vec<FilterPattern>, IpcSender<FileManagerResult<SelectedFile>>, String),

    /// Select multiple files, return a vector of triples
    SelectFiles(Vec<FilterPattern>, IpcSender<FileManagerResult<Vec<SelectedFile>>>, String),

    /// Read file, return the bytes
    ReadFile(IpcSender<FileManagerResult<Vec<u8>>>, SelectedFileId, RelativePos, String),

    /// Delete the FileID entry
    DeleteFileID(SelectedFileId, String),

    /// Load resource by Blob URL
    LoadBlob(LoadData, LoadConsumer),

    /// Add an entry and send back the associated uuid
    /// XXX: Second field is an unicode-serialized Origin, it is a temporary workaround
    ///      and should not be trusted. See issue https://github.com/servo/servo/issues/11722
    AddEntry(BlobURLStoreEntry, IpcSender<Result<String, BlobURLStoreError>>, String),

    /// Create ID out of ID
    AddIndirectEntry(SelectedFileId, RelativePos, IpcSender<Result<String, BlobURLStoreError>>, String),

    /// Increate reference
    IncRef(SelectedFileId, String),

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
