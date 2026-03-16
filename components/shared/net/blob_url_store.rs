/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use servo_url::{ImmutableOrigin, ServoUrl};
use url::Url;
use uuid::Uuid;

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
/// <https://w3c.github.io/FileAPI/#url-intro>
pub fn parse_blob_url(url: &ServoUrl) -> Result<(Uuid, ImmutableOrigin), &'static str> {
    if url.query().is_some() {
        return Err("URL should not contain a query");
    }

    let Some((_, uuid)) = url.path().rsplit_once('/') else {
        return Err("Failed to split origin from uuid");
    };

    // See https://url.spec.whatwg.org/#origin - "blob" case
    let origin = Url::parse(url.path())
        .ok()
        .filter(|url| matches!(url.scheme(), "http" | "https" | "file"))
        .map(|url| url.origin())
        .map(ImmutableOrigin::new)
        .unwrap_or(ImmutableOrigin::new_opaque());

    let id = Uuid::from_str(uuid).map_err(|_| "Failed to parse UUID from path segment")?;

    Ok((id, origin))
}
