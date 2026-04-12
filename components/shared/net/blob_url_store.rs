/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::sync::Arc;

use malloc_size_of_derive::MallocSizeOf;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use servo_base::generic_channel::{self, GenericSend, GenericSender};
use servo_url::{ImmutableOrigin, ServoUrl};
use url::Url;
use uuid::Uuid;

use crate::{
    BlobTokenRefreshRequest, BlobTokenRevocationRequest, CoreResourceMsg, FileManagerThreadMsg,
    ResourceThreads,
};

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
///
/// FIXME: This function should never be used to obtain the origin of a blob url, because
/// it doesn't consider [blob URL entries].
///
/// [blob URL entries]: https://url.spec.whatwg.org/#concept-url-blob-entry
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
        .map(|url| ImmutableOrigin::new(&url))
        .unwrap_or(ImmutableOrigin::new_opaque());

    let id = Uuid::from_str(uuid).map_err(|_| "Failed to parse UUID from path segment")?;

    Ok((id, origin))
}

/// This type upholds the variant that if the URL is a valid `blob` URL, then it has
/// a token. Violating this invariant causes logic errors, but no unsafety.
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct UrlWithBlobClaim {
    url: ServoUrl,
    token: Option<TokenSerializationGuard>,
}

impl UrlWithBlobClaim {
    pub fn new(url: ServoUrl, token: Option<TokenSerializationGuard>) -> Self {
        Self { url, token }
    }

    pub fn token(&self) -> Option<&BlobToken> {
        self.token.as_ref().map(|guard| guard.token.as_ref())
    }

    pub fn blob_id(&self) -> Option<Uuid> {
        self.token.as_ref().map(|guard| guard.token.file_id)
    }

    pub fn origin(&self) -> ImmutableOrigin {
        if let Some(guard) = self.token.as_ref() {
            return guard.token.origin.clone();
        }

        self.url.origin()
    }

    /// Constructs a [UrlWithBlobClaim] for URLs that are not `blob` URLs
    /// (Such URLs don't need to claim anything).
    ///
    /// Returns an `Err` containing the original URL if it's a `blob` URL,
    /// so it can be reused without cloning.
    pub fn for_url(url: ServoUrl) -> Result<Self, ServoUrl> {
        if url.scheme() == "blob" {
            return Err(url);
        }

        Ok(Self { url, token: None })
    }

    /// This method should only exist temporarily, and all callers should either
    /// claim the blob or guarantee that the URL is not a `blob` URL.
    pub fn from_url_without_having_claimed_blob(url: ServoUrl) -> Self {
        if url.scheme() == "blob" {
            // See https://github.com/servo/servo/issues/25226 for more details
            log::warn!(
                "Creating blob URL without claiming its associated blob entry. This might cause race conditions if the URL is revoked."
            );
        }
        Self { url, token: None }
    }

    pub fn url(&self) -> ServoUrl {
        self.url.clone()
    }
}

impl Deref for UrlWithBlobClaim {
    type Target = ServoUrl;

    fn deref(&self) -> &Self::Target {
        &self.url
    }
}

impl DerefMut for UrlWithBlobClaim {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.url
    }
}

/// Guarantees that blob entries kept alive the contained token are not deallocated even
/// if this token is serialized, dropped, and then later deserialized (possibly in a different thread).
#[derive(Clone, Debug, MallocSizeOf)]
pub struct TokenSerializationGuard {
    #[conditional_malloc_size_of]
    token: Arc<BlobToken>,
}

impl serde::Serialize for TokenSerializationGuard {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut new_token = self.token.refresh();
        let result = new_token.serialize(serializer);
        if result.is_ok() {
            // This token belongs to whoever receives the serialized message, so don't free it.
            new_token.disown();
        }
        result
    }
}

impl<'a> serde::Deserialize<'a> for TokenSerializationGuard {
    fn deserialize<D>(de: D) -> Result<Self, <D as serde::Deserializer<'a>>::Error>
    where
        D: serde::Deserializer<'a>,
    {
        struct TokenGuardVisitor;

        impl<'de> serde::de::Visitor<'de> for TokenGuardVisitor {
            type Value = TokenSerializationGuard;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a TokenSerializationGuard")
            }

            fn visit_newtype_struct<D>(
                self,
                deserializer: D,
            ) -> Result<Self::Value, <D as serde::Deserializer<'de>>::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Ok(TokenSerializationGuard {
                    token: Arc::new(BlobToken::deserialize(deserializer)?),
                })
            }
        }

        de.deserialize_newtype_struct("TokenSerializationGuard", TokenGuardVisitor)
    }
}

#[derive(Clone, MallocSizeOf)]
pub struct BlobResolver<'a> {
    pub origin: ImmutableOrigin,
    pub resource_threads: &'a ResourceThreads,
}

#[derive(Clone, Deserialize, MallocSizeOf, Serialize)]
/// A reference to a blob URL that will revoke the blob when dropped,
/// unless the `disown` method is invoked.
pub struct BlobToken {
    pub token: Uuid,
    pub file_id: Uuid,
    pub disowned: bool,
    pub origin: ImmutableOrigin,
    // We need a mutex here because BlobTokens are shared among threads, and accessing
    // `GenericSender<CoreResourceMsg>` from different threads is not safe.
    //
    // We need a Arc because the Communicator is shared among different BlobTokens.
    #[conditional_malloc_size_of]
    pub communicator: Arc<Mutex<BlobTokenCommunicator>>,
}

#[derive(Clone, Deserialize, MallocSizeOf, Serialize)]
pub struct BlobTokenCommunicator {
    pub revoke_sender: GenericSender<CoreResourceMsg>,
    pub refresh_token_sender: GenericSender<CoreResourceMsg>,
}

impl BlobTokenCommunicator {
    pub fn stub_for_testing() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            revoke_sender: generic_channel::channel().unwrap().0,
            refresh_token_sender: generic_channel::channel().unwrap().0,
        }))
    }
}

impl BlobToken {
    fn refresh(&self) -> Self {
        let (new_token_sender, new_token_receiver) = generic_channel::channel().unwrap();
        let refresh_request = BlobTokenRefreshRequest {
            blob_id: self.file_id,
            new_token_sender,
        };
        self.communicator
            .lock()
            .refresh_token_sender
            .send(CoreResourceMsg::RefreshTokenForFile(refresh_request))
            .unwrap();
        let new_token = new_token_receiver.recv().unwrap();

        BlobToken {
            token: new_token,
            file_id: self.file_id,
            communicator: self.communicator.clone(),
            disowned: false,
            origin: self.origin.clone(),
        }
    }

    /// Prevents this token from revoking itself when it is dropped.
    fn disown(&mut self) {
        self.disowned = true;
    }
}

impl<'a> BlobResolver<'a> {
    pub fn acquire_blob_token_for(&self, url: &ServoUrl) -> Option<TokenSerializationGuard> {
        if url.scheme() != "blob" {
            return None;
        }
        let (file_id, origin) = parse_blob_url(url)
            .inspect_err(|error| log::warn!("Failed to acquire token for {url}: {error}"))
            .ok()?;
        let (sender, receiver) = generic_channel::channel().unwrap();
        self.resource_threads
            .send(CoreResourceMsg::ToFileManager(
                FileManagerThreadMsg::GetTokenForFile(file_id, origin, sender),
            ))
            .ok()?;
        let reply = receiver.recv().ok()?;
        reply.token.map(|token_id| {
            let token = BlobToken {
                token: token_id,
                file_id,
                communicator: Arc::new(Mutex::new(BlobTokenCommunicator {
                    revoke_sender: reply.revoke_sender,
                    refresh_token_sender: reply.refresh_sender,
                })),
                disowned: false,
                origin: self.origin.clone(),
            };

            TokenSerializationGuard {
                token: Arc::new(token),
            }
        })
    }
}

impl Drop for BlobToken {
    fn drop(&mut self) {
        if self.disowned {
            return;
        }

        let revocation_request = BlobTokenRevocationRequest {
            token: self.token,
            blob_id: self.file_id,
        };
        let _ = self
            .communicator
            .lock()
            .revoke_sender
            .send(CoreResourceMsg::RevokeTokenForFile(revocation_request));
    }
}

impl fmt::Debug for BlobToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlobToken")
            .field("token", &self.token)
            .field("file_id", &self.file_id)
            .field("disowned", &self.disowned)
            .field("origin", &self.origin)
            .finish()
    }
}
