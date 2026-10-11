/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;

use cookie::Cookie;
use embedder_traits::{
    AuthenticationResponse, EmbedderControlId, FilePickerRequest, WebResourceRequest,
    WebResourceResponseMsg,
};
use net_traits::CookieOperationId;
use servo_base::id::WebViewId;
use servo_url::ServoUrl;
use tokio::sync::mpsc::UnboundedSender as TokioSender;
use tokio::sync::oneshot::Sender as TokioOneshotSender;

/// Response to a certificate error request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertificateErrorResponse {
    /// Allow the connection despite the certificate error.
    Allow,
    /// Deny the connection (certificate error will be reported).
    Deny,
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
    /// Request certificate error handling for a load or navigation from the embedder.
    RequestCertificateError(
        WebViewId,
        ServoUrl,
        i32, /* error_code */
        String, /* error_text */
        Vec<u8>, /* certificate_der */
        TokioOneshotSender<CertificateErrorResponse>,
    ),
    /// Response to an asynchronous request from SiteDataManager with cookies result.
    EmbedderCookieOperationResponseWithCookies(CookieOperationId, Vec<Cookie<'static>>),
    /// Response to an asynchronous request from SiteDataManager.
    EmbedderCookieOperationResponse(CookieOperationId),
}
