/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use filemanager_thread::FileOrigin;
use std::str::FromStr;
use url::Url;
use uuid::Uuid;

/// Errors returned to Blob URL Store request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BlobURLStoreError {
    /// Invalid File UUID
    InvalidFileID,
    /// Invalid URL origin
    InvalidOrigin,
    /// Invalid entry content
    InvalidEntry,
    /// External error, from like file system, I/O etc.
    External(String),
}

/// Standalone blob buffer object
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlobBuf {
    pub filename: Option<String>,
    /// MIME type string
    pub type_string: String,
    /// Size of content in bytes
    pub size: u64,
    /// Content of blob
    pub bytes: Vec<u8>,
}

/// Parse URL as Blob URL scheme's definition
/// https://w3c.github.io/FileAPI/#DefinitionOfScheme
pub fn parse_blob_url(url: &Url) -> Result<(Uuid, FileOrigin, Option<String>), ()> {
    let url_inner = try!(Url::parse(url.path()).map_err(|_| ()));
    let fragment = url_inner.fragment().map(|s| s.to_string());
    let mut segs = try!(url_inner.path_segments().ok_or(()));
    let id = try!(segs.nth(0).ok_or(()));
    let id = try!(Uuid::from_str(id).map_err(|_| ()));
    Ok((id, get_blob_origin(&url_inner), fragment))
}

/// Given an URL, returning the Origin that a Blob created under this
/// URL should have.
/// HACK(izgzhen): Not well-specified on spec, and it is a bit a hack
/// both due to ambiguity of spec and that we have to serialization the
/// Origin here.
pub fn get_blob_origin(url: &Url) -> FileOrigin {
    if url.scheme() == "file" {
        // NOTE: by default this is "null" (Opaque), which is not ideal
        "file://".to_string()
    } else {
        url.origin().unicode_serialization()
    }
}
