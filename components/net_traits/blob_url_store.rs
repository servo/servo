/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::str::FromStr;
use url::Url;
use uuid::Uuid;

/// Errors returns to BlobURLStoreMsg::Request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BlobURLStoreError {
    /// Invalid File UUID
    InvalidFileID,
    /// Invalid URL origin
    InvalidOrigin,
    /// Invalid entry content
    InvalidEntry,
    /// External error, from like file system, I/O etc.
    External,
}

/// Blob URL store entry, a packaged form of Blob DOM object
#[derive(Clone, Serialize, Deserialize)]
pub struct BlobURLStoreEntry {
    /// MIME type string
    pub type_string: String,
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
