/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;
use uuid::Uuid;
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub enum FileManagerThreadMsg {
    /// Select a single file, return triple (FileID, FileName, lastModified)
    SelectFile(IpcSender<FileManagerResult<(Uuid, PathBuf, u64)>>),

    /// Read file, return the bytes
    ReadFile(IpcSender<FileManagerResult<Vec<u8>>>, Uuid),

    /// Delete the FileID entry
    DeleteFileID(Uuid),
}

pub type FileManagerResult<T> = Result<T, FileManagerThreadError>;

#[derive(Deserialize, Serialize)]
pub enum FileManagerThreadError {
    /// The selection action is invalid, nothing is selected
    InvalidSelection,
    /// Failure to process file information such as file name, modified time etc.
    FileInfoProcessingError,
    /// Failure to read the file content
    ReadFileError,
}
