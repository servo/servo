/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;

use base::id::WebViewId;
use embedder_traits::{
    AuthenticationResponse, EmbedderControlId, FilePickerRequest, WebResourceRequest,
    WebResourceResponseMsg,
};
use servo_url::ServoUrl;
use tokio::sync::mpsc::UnboundedSender as TokioSender;
use tokio::sync::oneshot::Sender as TokioOneshotSender;

/// Messages sent from the network threads to the embedder.
pub enum NetEmbedderMsg {
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
}
