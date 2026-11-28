/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;
use std::sync::Arc;

use bytes::Bytes;

use cookie::Cookie;
use embedder_traits::{
    AuthenticationResponse, EmbedderControlId, FilePickerRequest, WebResourceRequest,
    WebResourceResponseMsg,
};
use net_traits::CookieOperationId;
use net_traits::NetworkError;
use net_traits::request::RequestId;
use servo_base::id::WebViewId;
use servo_url::ServoUrl;
use tokio::sync::mpsc::{self, UnboundedSender as TokioSender};
use tokio::sync::oneshot::Sender as TokioOneshotSender;

use crate::fetch::methods::CancellationListener;

type DownloadChunk = Result<Option<Bytes>, NetworkError>;

pub enum DownloadDecision {
    Accept(mpsc::Sender<DownloadChunk>),
    Decline,
}

/// Stops the fetch, including when the server stalls between chunks.
#[derive(Clone)]
pub struct DownloadCancellation(pub(crate) Arc<CancellationListener>);

impl DownloadCancellation {
    pub fn is_cancelled(&self) -> bool {
        self.0.cancelled()
    }

    pub fn cancel(&self) {
        self.0.cancel();
    }
}

/// A response body owned by the embedder after accepting a download.
pub struct DownloadStream {
    cancellation: DownloadCancellation,
    receiver: mpsc::Receiver<DownloadChunk>,
}

impl DownloadStream {
    pub fn accept(
        responder: TokioOneshotSender<DownloadDecision>,
        cancellation: DownloadCancellation,
    ) -> Option<Self> {
        let (sender, receiver) = mpsc::channel(8);
        responder.send(DownloadDecision::Accept(sender)).ok()?;
        Some(Self {
            cancellation,
            receiver,
        })
    }

    pub fn cancellation(&self) -> DownloadCancellation {
        self.cancellation.clone()
    }

    /// Read the next chunk on a blocking worker. An empty response end is
    /// reported only after Servo has received the complete body.
    pub fn read_chunk(&mut self) -> Result<Option<Bytes>, NetworkError> {
        self.receiver
            .blocking_recv()
            .unwrap_or(Err(NetworkError::LoadCancelled))
    }
}

impl Drop for DownloadStream {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

/// Messages sent from the network threads to the embedder.
pub enum NetToEmbedderMsg {
    /// Open file dialog to select files. Set boolean flag to true allows to select multiple files.
    SelectFiles(
        EmbedderControlId,
        FilePickerRequest,
        TokioOneshotSender<Option<Vec<PathBuf>>>,
    ),
    WebResourceRequested(
        Option<WebViewId>,
        WebResourceRequest,
        TokioSender<WebResourceResponseMsg>,
    ),
    /// Request authentication for a load or navigation from the embedder.
    RequestAuthentication(
        WebViewId,
        ServoUrl,
        bool, /* for proxy */
        TokioOneshotSender<Option<AuthenticationResponse>>,
    ),
    /// Response to an asynchronous request from SiteDataManager with cookies result.
    EmbedderCookieOperationResponseWithCookies(CookieOperationId, Vec<Cookie<'static>>),
    /// Response to an asynchronous request from SiteDataManager.
    EmbedderCookieOperationResponse(CookieOperationId),
    /// A navigation response the renderer cannot display inline. The embedder
    /// decides whether to take it over (typically to save it to disk).
    OfferUnsupportedResponse {
        webview_id: WebViewId,
        request_id: RequestId,
        url: ServoUrl,
        content_type: Option<mime::Mime>,
        default_filename: String,
        content_length: Option<u64>,
        can_retry: bool,
        cancellation: DownloadCancellation,
        responder: TokioOneshotSender<DownloadDecision>,
    },
}
