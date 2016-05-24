/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct SelectedFile {
    pub id: Uuid,
    pub filename: PathBuf,
    pub modified: u64,
    // https://w3c.github.io/FileAPI/#dfn-type
    pub type_string: String,
}

#[derive(Deserialize, Serialize)]
pub enum FileManagerThreadMsg {
    /// Select a single file, return triple (FileID, FileName, lastModified)
    SelectFile(IpcSender<FileManagerResult<SelectedFile>>),

    /// Select multiple files, return a vector of triples
    SelectFiles(IpcSender<FileManagerResult<Vec<SelectedFile>>>),

    /// Read file, return the bytes
    ReadFile(IpcSender<FileManagerResult<Vec<u8>>>, Uuid),

    /// Delete the FileID entry
    DeleteFileID(Uuid),

    /// Shut down this thread
    Exit,
}

pub type FileManagerResult<T> = Result<T, FileManagerThreadError>;

#[derive(Debug, Deserialize, Serialize)]
pub enum FileManagerThreadError {
    /// The selection action is invalid, nothing is selected
    InvalidSelection,
    /// Failure to process file information such as file name, modified time etc.
    FileInfoProcessingError,
    /// Failure to read the file content
    ReadFileError,
}
