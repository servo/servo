/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;

use crate::WebView;

pub struct StringRequest {
    pub(crate) result_sender: IpcSender<Result<String, String>>,
    response_sent: bool,
}

impl StringRequest {
    pub fn success(mut self, string: String) {
        let _ = self.result_sender.send(Ok(string));
        self.response_sent = true;
    }

    pub fn failure(mut self, message: String) {
        let _ = self.result_sender.send(Err(message));
        self.response_sent = true;
    }
}

impl From<IpcSender<Result<String, String>>> for StringRequest {
    fn from(result_sender: IpcSender<Result<String, String>>) -> Self {
        Self {
            result_sender,
            response_sent: false,
        }
    }
}

impl Drop for StringRequest {
    fn drop(&mut self) {
        if !self.response_sent {
            let _ = self
                .result_sender
                .send(Err("No response sent to request.".into()));
        }
    }
}

/// A delegate that is responsible for accessing the system clipboard. On Mac, Windows, and
/// Linux if the `clipboard` feature is enabled, a default delegate is automatically used
/// that implements clipboard support. An embedding application can override this delegate
/// by using this trait.
pub trait ClipboardDelegate {
    /// A request to clear all contents of the system clipboard.
    fn clear(&self, _webview: WebView) {}

    /// A request to get the text contents of the system clipboard. Once the contents are
    /// retrieved the embedder should call [`StringRequest::success`] with the text or
    /// [`StringRequest::failure`] with a failure message.
    fn get_text(&self, _webview: WebView, _request: StringRequest) {}

    /// A request to set the text contents of the system clipboard to `new_contents`.
    fn set_text(&self, _webview: WebView, _new_contents: String) {}
}

pub(crate) struct DefaultClipboardDelegate;

impl ClipboardDelegate for DefaultClipboardDelegate {
    fn clear(&self, _webview: WebView) {
        clipboard::clear();
    }

    fn get_text(&self, _webview: WebView, request: StringRequest) {
        clipboard::get_text(request);
    }

    fn set_text(&self, _webview: WebView, new_contents: String) {
        clipboard::set_text(new_contents);
    }
}

#[cfg(all(
    feature = "clipboard",
    not(any(target_os = "android", target_env = "ohos"))
))]
mod clipboard {
    use std::sync::OnceLock;

    use arboard::Clipboard;
    use parking_lot::Mutex;

    use super::StringRequest;

    /// A shared clipboard for use by the [`DefaultClipboardDelegate`]. This is protected by
    /// a mutex so that it can only be used by one thread at a time. The `arboard` documentation
    /// suggests that more than one thread shouldn't try to access the Windows clipboard at a
    /// time. See <https://docs.rs/arboard/latest/arboard/struct.Clipboard.html>.
    static SHARED_CLIPBOARD: OnceLock<Option<Mutex<Clipboard>>> = OnceLock::new();

    fn with_shared_clipboard(callback: impl FnOnce(&mut Clipboard)) {
        if let Some(clipboard_mutex) =
            SHARED_CLIPBOARD.get_or_init(|| Clipboard::new().ok().map(Mutex::new))
        {
            callback(&mut clipboard_mutex.lock())
        }
    }

    pub(super) fn clear() {
        with_shared_clipboard(|clipboard| {
            let _ = clipboard.clear();
        });
    }

    pub(super) fn get_text(request: StringRequest) {
        with_shared_clipboard(move |clipboard| match clipboard.get_text() {
            Ok(text) => request.success(text),
            Err(error) => request.failure(format!("{error:?}")),
        });
    }

    pub(super) fn set_text(new_contents: String) {
        with_shared_clipboard(move |clipboard| {
            let _ = clipboard.set_text(new_contents);
        });
    }
}

#[cfg(any(not(feature = "clipboard"), target_os = "android", target_env = "ohos"))]
mod clipboard {
    use super::StringRequest;

    pub(super) fn clear() {}
    pub(super) fn get_text(_: StringRequest) {}
    pub(super) fn set_text(_: String) {}
}
