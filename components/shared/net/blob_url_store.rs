/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use malloc_size_of_derive::MallocSizeOf;
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

/// This type upholds the variant that the URL is
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct ServoUrlWithBlobLock {
    url: ServoUrl,
    token: Option<TokenSerializationGuard>,
}

impl ServoUrlWithBlobLock {
    pub fn new(url: ServoUrl, token: Option<TokenSerializationGuard>) -> Self {
        Self { url, token }
    }

    pub fn blob_id(&self) -> Option<Uuid> {
        self.token.as_ref().map(|guard| guard.token.file_id.clone())
    }

    pub fn origin(&self) -> ImmutableOrigin {
        if let Some(guard) = self.token.as_ref() {
            return guard.token.origin.clone();
        }

        self.url.origin()
    }

    /// Returns an `Err` containing the original URL if it's a `blob:` URL,
    /// so it can be reused without cloning.
    pub fn for_url(url: ServoUrl) -> Result<Self, ServoUrl> {
        if url.scheme() == "blob" {
            return Err(url);
        }

        Ok(Self { url, token: None })
    }

    /// This method should only exist temporarily
    pub fn from_url_without_having_acquired_blob_lock(url: ServoUrl) -> Self {
        Self { url, token: None }
    }

    pub fn url(&self) -> ServoUrl {
        self.url.clone()
    }

    pub fn set_scheme(&self, scheme: &str) -> Result<(), ()> {
        self.url().as_mut_url().set_scheme(scheme)
    }
}

impl Deref for ServoUrlWithBlobLock {
    type Target = ServoUrl;

    fn deref(&self) -> &Self::Target {
        &self.url
    }
}

impl DerefMut for ServoUrlWithBlobLock {
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
            new_token.neuter();
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
pub struct BlobToken {
    pub token: Uuid,
    pub file_id: Uuid,
    pub neutered: bool,
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

impl BlobToken {
    fn refresh(&self) -> Self {
        let (new_token_sender, new_token_receiver) = generic_channel::channel().unwrap();
        let refresh_request = BlobTokenRefreshRequest {
            blob_id: self.file_id.clone(),
            new_token_sender,
        };
        self.communicator
            .lock()
            .unwrap()
            .refresh_token_sender
            .send(CoreResourceMsg::RefreshTokenForFile(refresh_request))
            .unwrap();
        let new_token = new_token_receiver.recv().unwrap();

        BlobToken {
            token: new_token,
            file_id: self.file_id.clone(),
            communicator: self.communicator.clone(),
            neutered: false,
            origin: self.origin.clone(),
        }
    }

    fn neuter(&mut self) {
        self.neutered = true;
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
        let serializable_token = reply.token.map(|token_id| {
            let token = BlobToken {
                token: token_id,
                file_id,
                communicator: Arc::new(Mutex::new(BlobTokenCommunicator {
                    revoke_sender: reply.revoke_sender,
                    refresh_token_sender: reply.refresh_sender,
                })),
                neutered: false,
                origin: self.origin.clone(),
            };

            TokenSerializationGuard {
                token: Arc::new(token),
            }
        });
        serializable_token
    }
}

impl Drop for BlobToken {
    fn drop(&mut self) {
        if self.neutered {
            return;
        }

        let revocation_request = BlobTokenRevocationRequest {
            token: self.token.clone(),
            blob_id: self.file_id.clone(),
        };
        let _ = self
            .communicator
            .lock()
            .unwrap()
            .revoke_sender
            .send(CoreResourceMsg::RevokeTokenForFile(revocation_request));
    }
}

impl fmt::Debug for BlobToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlobToken")
            .field("token", &self.token)
            .field("file_id", &self.file_id)
            .field("neutered", &self.neutered)
            .field("origin", &self.origin)
            .finish()
    }
}
