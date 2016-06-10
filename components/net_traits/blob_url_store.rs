/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;
use std::str::FromStr;
use url::Url;
use uuid::Uuid;

/// Errors returns to BlobURLStoreMsg::Request
#[derive(Clone, Serialize, Deserialize)]
pub enum BlobURLStoreError {
    /// Invalid UUID key
    InvalidKey,
    /// Invalid URL origin
    InvalidOrigin,
}

#[derive(Serialize, Deserialize)]
pub enum BlobURLStoreMsg {
    /// Add an entry and send back the associated uuid
    /// XXX: Second field is an unicode-serialized Origin, it is a temporary workaround
    ///      and should not be trusted. See issue https://github.com/servo/servo/issues/11722
    AddEntry(BlobURLStoreEntry, String, IpcSender<Result<String, BlobURLStoreError>>),
    /// Delete an entry by uuid
    DeleteEntry(String),
}

/// Blob URL store entry, a packaged form of Blob DOM object
#[derive(Clone, Serialize, Deserialize)]
pub struct BlobURLStoreEntry {
    /// MIME type string
    pub type_string: String,
    /// Some filename if the backend of Blob is a file
    pub filename: Option<String>,
    /// Size of content in bytes
    pub size: u64,
    /// Content of blob
    pub bytes: Vec<u8>,
}

/// Parse URL as Blob URL scheme's definition
/// https://w3c.github.io/FileAPI/#DefinitionOfScheme
pub fn parse_blob_url(url: &Url) -> Option<(Uuid, Option<&str>)> {
    url.path_segments().and_then(|mut segments| {
        let id_str = match (segments.next(), segments.next()) {
            (Some(s), None) => s,
            _ => return None,
        };

        Uuid::from_str(id_str).map(|id| (id, url.fragment())).ok()
    })
}
