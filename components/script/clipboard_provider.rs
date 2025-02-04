/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::WebViewId;
use embedder_traits::EmbedderMsg;
use ipc_channel::ipc::channel;
use script_traits::{ScriptMsg, ScriptToConstellationChan};

pub trait ClipboardProvider {
    // blocking method to get the clipboard contents
    fn clipboard_contents(&mut self) -> String;
    // blocking method to set the clipboard contents
    fn set_clipboard_contents(&mut self, _: String);
}

pub(crate) struct EmbedderClipboardProvider {
    pub constellation_sender: ScriptToConstellationChan,
    pub webview_id: WebViewId,
}

impl ClipboardProvider for EmbedderClipboardProvider {
    fn clipboard_contents(&mut self) -> String {
        let (tx, rx) = channel().unwrap();
        self.constellation_sender
            .send(ScriptMsg::ForwardToEmbedder(
                EmbedderMsg::GetClipboardContents(self.webview_id, tx),
            ))
            .unwrap();
        rx.recv().unwrap()
    }
    fn set_clipboard_contents(&mut self, s: String) {
        self.constellation_sender
            .send(ScriptMsg::ForwardToEmbedder(
                EmbedderMsg::SetClipboardContents(self.webview_id, s),
            ))
            .unwrap();
    }
}
