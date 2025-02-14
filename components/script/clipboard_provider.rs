/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::WebViewId;
use embedder_traits::EmbedderMsg;
use ipc_channel::ipc::channel;
use script_traits::{ScriptMsg, ScriptToConstellationChan};

/// A trait which abstracts access to the embedder's clipboard in order to allow unit
/// testing clipboard-dependent parts of `script`.
pub trait ClipboardProvider {
    /// Get the text content of the clipboard.
    fn get_text(&mut self) -> Result<String, String>;
    /// Set the text content of the clipboard.
    fn set_text(&mut self, _: String);
}

pub(crate) struct EmbedderClipboardProvider {
    pub constellation_sender: ScriptToConstellationChan,
    pub webview_id: WebViewId,
}

impl ClipboardProvider for EmbedderClipboardProvider {
    fn get_text(&mut self) -> Result<String, String> {
        let (tx, rx) = channel().unwrap();
        self.constellation_sender
            .send(ScriptMsg::ForwardToEmbedder(EmbedderMsg::GetClipboardText(
                self.webview_id,
                tx,
            )))
            .unwrap();
        rx.recv().unwrap()
    }
    fn set_text(&mut self, s: String) {
        self.constellation_sender
            .send(ScriptMsg::ForwardToEmbedder(EmbedderMsg::SetClipboardText(
                self.webview_id,
                s,
            )))
            .unwrap();
    }
}
