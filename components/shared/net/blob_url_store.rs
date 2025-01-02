/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;
use url::Url;
use uuid::Uuid;

use crate::filemanager_thread::FileOrigin;

/// Errors returned to Blob URL Store request
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum BlobURLStoreError {
    /// Invalid File UUID
    InvalidFileID,
    /// Invalid URL origin
    InvalidOrigin,
    /// Invalid entry content
    InvalidEntry,
    /// Invalid range
    InvalidRange,
    /// External error, from like file system, I/O etc.
    External(String),
}

/// Standalone blob buffer object
#[derive(Clone, Debug, Deserialize, Serialize)]
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
///
/// <https://w3c.github.io/FileAPI/#DefinitionOfScheme>
pub fn parse_blob_url(url: &ServoUrl) -> Result<(Uuid, FileOrigin), &'static str> {
    let url_inner = Url::parse(url.path()).map_err(|_| "Failed to parse URL path")?;
    let segs = url_inner
        .path_segments()
        .map(|c| c.collect::<Vec<_>>())
        .ok_or("URL has no path segments")?;

    if url.query().is_some() {
        return Err("URL should not contain a query");
    }

    if segs.len() > 1 {
        return Err("URL should not have more than one path segment");
    }

    let id = {
        let id = segs.first().ok_or("URL has no path segments")?;
        Uuid::from_str(id).map_err(|_| "Failed to parse UUID from path segment")?
    };
    Ok((id, get_blob_origin(&ServoUrl::from_url(url_inner))))
}

/// Given an URL, returning the Origin that a Blob created under this
/// URL should have.
///
/// HACK(izgzhen): Not well-specified on spec, and it is a bit a hack
/// both due to ambiguity of spec and that we have to serialization the
/// Origin here.
pub fn get_blob_origin(url: &ServoUrl) -> FileOrigin {
    if url.scheme() == "file" {
        // NOTE: by default this is "null" (Opaque), which is not ideal
        "file://".to_string()
    } else {
        url.origin().ascii_serialization()
    }
}
