/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

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
pub fn parse_blob_url(url: &ServoUrl) -> Result<Uuid, &'static str> {
    if url.query().is_some() {
        return Err("URL should not contain a query");
    }

    let Some((_, uuid)) = url.path().rsplit_once('/') else {
        return Err("Failed to split origin from uuid");
    };

    Uuid::from_str(uuid).map_err(|_| "Failed to parse UUID from path segment")
}

/// This type upholds the variant that if the URL is a valid `blob` URL, then it has
/// a token. Violating this invariant causes logic errors, but no unsafety.
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct UrlWithBlobClaim {
    url: ServoUrl,
    token: Option<TokenSerializationGuard>,
}

impl UrlWithBlobClaim {
    pub fn into_transfer(self) -> TransferableUrlWithBlobClaim {
        TransferableUrlWithBlobClaim {
            url: self.url,
            token: self.token.map(|guard| {
                Arc::new(BlobTransferState {
                    token: Mutex::new(Some(guard.token.refresh())),
                    serialized: AtomicBool::new(false),
                })
            }),
        }
    }

    pub fn new(url: ServoUrl, token: Option<TokenSerializationGuard>) -> Self {
        Self { url, token }
    }

    pub fn token(&self) -> Option<&BlobToken> {
        self.token.as_ref().map(|guard| guard.token.as_ref())
    }

    pub fn blob_id(&self) -> Option<Uuid> {
        self.token.as_ref().map(|guard| guard.token.file_id)
    }

    /// <https://url.spec.whatwg.org/#concept-url-origin>
    pub fn origin(&self) -> ImmutableOrigin {
        // > The origin of a URL url is the origin returned by running these steps,
        // > switching on url’s scheme:
        let url = &self.url;
        if url.scheme() == "blob" {
            // Step 1. If url’s blob URL entry is non-null,
            // then return url’s blob URL entry’s environment’s origin.
            if let Some(guard) = self.token.as_ref() {
                return guard.token.origin.clone();
            }

            // Step 2. Let pathURL be the result of parsing the result of URL path serializing url.
            Url::parse(url.path())
                .ok()
                // Step 4. If pathURL’s scheme is "http", "https", or "file",
                // then return pathURL’s origin.
                .filter(|url| matches!(url.scheme(), "http" | "https" | "file"))
                .map(|url| ImmutableOrigin::new(&url))
                // Step 3. If pathURL is failure, then return a new opaque origin.
                // Step 5. Return a new opaque origin.
                .unwrap_or(ImmutableOrigin::new_opaque())
        } else {
            // > Return the tuple origin (url’s scheme, url’s host, url’s port, null).
            url.origin()
        }
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

#[derive(Debug, Deserialize, Serialize)]
pub struct TransferableUrlWithBlobClaim {
    url: ServoUrl,
    token: Option<Arc<BlobTransferState>>,
}

impl TransferableUrlWithBlobClaim {
    pub fn url(&self) -> ServoUrl {
        self.url.clone()
    }

    pub fn transfer_guard(&self) -> BlobUrlTransferGuard {
        BlobUrlTransferGuard(self.token.clone())
    }

    pub fn into_url(self) -> UrlWithBlobClaim {
        UrlWithBlobClaim {
            url: self.url,
            token: self.token.map(|state| TokenSerializationGuard {
                token: Arc::new(state.token.lock().take().expect("unconsumed blob transfer")),
            }),
        }
    }
}

impl Deref for TransferableUrlWithBlobClaim {
    type Target = ServoUrl;

    fn deref(&self) -> &Self::Target {
        &self.url
    }
}

#[derive(Debug)]
struct BlobTransferState {
    token: Mutex<Option<BlobToken>>,
    serialized: AtomicBool,
}

impl Serialize for BlobTransferState {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let token = self.token.lock();
        let token = token
            .as_ref()
            .ok_or_else(|| serde::ser::Error::custom("blob transfer has already been consumed"))?;
        let result = token.serialize(serializer);
        if result.is_ok() {
            self.serialized.store(true, Ordering::SeqCst);
        }
        result
    }
}

impl<'de> Deserialize<'de> for BlobTransferState {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self {
            token: Mutex::new(Some(BlobToken::deserialize(deserializer)?)),
            serialized: AtomicBool::new(false),
        })
    }
}

#[must_use]
pub struct BlobUrlTransferGuard(Option<Arc<BlobTransferState>>);

impl BlobUrlTransferGuard {
    pub fn commit(self) {
        if let Some(state) = &self.0 &&
            state.serialized.load(Ordering::SeqCst) &&
            let Some(token) = state.token.lock().as_mut()
        {
            token.disown();
        }
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
        let file_id = parse_blob_url(url)
            .inspect_err(|error| log::warn!("Failed to acquire token for {url}: {error}"))
            .ok()?;
        let (sender, receiver) = generic_channel::channel().unwrap();
        self.resource_threads
            .send(CoreResourceMsg::ToFileManager(
                FileManagerThreadMsg::GetTokenForFile(file_id, sender),
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

#[cfg(test)]
mod tests {
    use ipc_channel::ipc;
    use servo_base::generic_channel::GenericReceiver;

    use super::*;

    fn transfer() -> (
        TransferableUrlWithBlobClaim,
        GenericReceiver<CoreResourceMsg>,
    ) {
        let (sender, receiver) = generic_channel::channel().expect("resource channel");
        let file_id = Uuid::new_v4();
        let url =
            ServoUrl::parse(&format!("blob:https://example.test/{file_id}")).expect("blob URL");
        let token = BlobToken {
            token: Uuid::new_v4(),
            file_id,
            disowned: false,
            origin: ServoUrl::parse("https://example.test")
                .expect("origin")
                .origin(),
            communicator: Arc::new(Mutex::new(BlobTokenCommunicator {
                revoke_sender: sender.clone(),
                refresh_token_sender: sender,
            })),
        };
        (
            TransferableUrlWithBlobClaim {
                url,
                token: Some(Arc::new(BlobTransferState {
                    token: Mutex::new(Some(token)),
                    serialized: AtomicBool::new(false),
                })),
            },
            receiver,
        )
    }

    fn assert_revoked(receiver: &GenericReceiver<CoreResourceMsg>) {
        assert!(matches!(
            receiver.try_recv(),
            Ok(CoreResourceMsg::RevokeTokenForFile(_))
        ));
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn sharedworker_blob_transfer_preserves_preceding_ipc_endpoints() {
        let (url, resource_requests) = transfer();
        let guard = url.transfer_guard();
        let (extra, extra_receiver) = ipc::channel::<u32>().expect("extra endpoint");
        let (sender, receiver) = ipc::channel().expect("transfer channel");
        sender.send((extra, url)).expect("send prepared claim");
        guard.commit();
        assert!(resource_requests.try_recv().is_err());
        let (extra, url) = receiver.recv().expect("receive prepared claim");
        extra.send(7).expect("use preceding endpoint");
        assert_eq!(extra_receiver.recv().expect("preceding endpoint value"), 7);
        drop(url.into_url());
        assert_revoked(&resource_requests);
    }

    #[test]
    fn sharedworker_blob_transfer_keeps_in_process_receiver_ownership() {
        let (url, resource_requests) = transfer();
        let guard = url.transfer_guard();
        let received = url.into_url();
        guard.commit();
        assert!(resource_requests.try_recv().is_err());
        drop(received);
        assert_revoked(&resource_requests);
    }

    #[test]
    fn sharedworker_blob_transfer_releases_an_unsent_claim() {
        let (url, resource_requests) = transfer();
        let guard = url.transfer_guard();
        drop(url);
        assert!(resource_requests.try_recv().is_err());
        drop(guard);
        assert_revoked(&resource_requests);
    }

    #[test]
    fn sharedworker_blob_transfer_releases_a_failed_ipc_send() {
        let (url, resource_requests) = transfer();
        let guard = url.transfer_guard();
        let (sender, receiver) = ipc::channel().expect("transfer channel");
        drop(receiver);
        assert!(sender.send(url).is_err());
        drop(guard);
        assert_revoked(&resource_requests);
    }
}
