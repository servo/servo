/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;
use std::path::PathBuf;
use super::{LoadConsumer, LoadData};

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FilterPattern(pub String);

#[derive(Deserialize, Serialize)]
pub enum FileManagerThreadMsg {
    /// Select a single file, return triple (FileID, FileName, lastModified)
    SelectFile(Vec<FilterPattern>, IpcSender<FileManagerResult<SelectedFile>>),

    /// Select multiple files, return a vector of triples
    SelectFiles(Vec<FilterPattern>, IpcSender<FileManagerResult<Vec<SelectedFile>>>),

    /// Read file, return the bytes
    ReadFile(IpcSender<FileManagerResult<Vec<u8>>>, SelectedFileId),

    /// Delete the FileID entry
    DeleteFileID(SelectedFileId),

    /// Load resource by Blob URL
    LoadBlob(LoadData, LoadConsumer),

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
