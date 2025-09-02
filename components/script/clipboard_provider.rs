/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::WebViewId;
use embedder_traits::{EmbedderMsg, ScriptToEmbedderChan};
use ipc_channel::ipc::channel;
use malloc_size_of_derive::MallocSizeOf;

/// A trait which abstracts access to the embedder's clipboard in order to allow unit
/// testing clipboard-dependent parts of `script`.
pub trait ClipboardProvider {
    /// Get the text content of the clipboard.
    fn get_text(&mut self) -> Result<String, String>;
    /// Set the text content of the clipboard.
    fn set_text(&mut self, _: String);
}

#[derive(MallocSizeOf)]
pub(crate) struct EmbedderClipboardProvider {
    pub embedder_sender: ScriptToEmbedderChan,
    pub webview_id: WebViewId,
}

impl ClipboardProvider for EmbedderClipboardProvider {
    fn get_text(&mut self) -> Result<String, String> {
        let (tx, rx) = channel().unwrap();
        self.embedder_sender
            .send(EmbedderMsg::GetClipboardText(self.webview_id, tx))
            .unwrap();
        rx.recv().unwrap()
    }
    fn set_text(&mut self, s: String) {
        self.embedder_sender
            .send(EmbedderMsg::SetClipboardText(self.webview_id, s))
            .unwrap();
    }
}
