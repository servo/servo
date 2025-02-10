/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;

use base::id::PipelineId;
use compositing_traits::ConstellationMsg;
use embedder_traits::{
    AllowOrDeny, CompositorEventVariant, ContextMenuResult, Cursor, FilterPattern,
    GamepadHapticEffectType, InputMethodType, LoadStatus, MediaSessionEvent, PermissionFeature,
    PromptDefinition, PromptOrigin, WebResourceRequest, WebResourceResponseMsg,
};
use ipc_channel::ipc::IpcSender;
use keyboard_types::KeyboardEvent;
use url::Url;
use webrender_api::units::{DeviceIntPoint, DeviceIntRect, DeviceIntSize};

use crate::{ConstellationProxy, WebView};

/// A request to navigate a [`WebView`] or one of its inner frames. This can be handled
/// asynchronously. If not handled, the request will automatically be allowed.
pub struct NavigationRequest {
    pub url: Url,
    pub(crate) pipeline_id: PipelineId,
    pub(crate) constellation_proxy: ConstellationProxy,
    pub(crate) response_sent: bool,
}

impl NavigationRequest {
    pub fn allow(mut self) {
        self.constellation_proxy
            .send(ConstellationMsg::AllowNavigationResponse(
                self.pipeline_id,
                true,
            ));
        self.response_sent = true;
    }

    pub fn deny(mut self) {
        self.constellation_proxy
            .send(ConstellationMsg::AllowNavigationResponse(
                self.pipeline_id,
                false,
            ));
        self.response_sent = true;
    }
}

impl Drop for NavigationRequest {
    fn drop(&mut self) {
        if !self.response_sent {
            self.constellation_proxy
                .send(ConstellationMsg::AllowNavigationResponse(
                    self.pipeline_id,
                    true,
                ));
        }
    }
}

/// A permissions request for a [`WebView`] The embedder should allow or deny the request,
/// either by reading a cached value or querying the user for permission via the user
/// interface.
pub struct PermissionRequest {
    pub(crate) requested_feature: PermissionFeature,
    pub(crate) allow_deny_request: AllowOrDenyRequest,
}

impl PermissionRequest {
    pub fn feature(&self) -> PermissionFeature {
        self.requested_feature
    }

    pub fn allow(self) {
        self.allow_deny_request.allow();
    }

    pub fn deny(self) {
        self.allow_deny_request.deny();
    }
}

pub struct AllowOrDenyRequest {
    pub(crate) response_sender: IpcSender<AllowOrDeny>,
    pub(crate) response_sent: bool,
    pub(crate) default_response: AllowOrDeny,
}

impl AllowOrDenyRequest {
    pub fn allow(mut self) {
        let _ = self.response_sender.send(AllowOrDeny::Allow);
        self.response_sent = true;
    }

    pub fn deny(mut self) {
        let _ = self.response_sender.send(AllowOrDeny::Deny);
        self.response_sent = true;
    }
}

impl Drop for AllowOrDenyRequest {
    fn drop(&mut self) {
        if !self.response_sent {
            let _ = self.response_sender.send(self.default_response);
        }
    }
}

pub trait WebViewDelegate {
    /// The URL of the currently loaded page in this [`WebView`] has changed. The new
    /// URL can accessed via [`WebView::url`].
    fn notify_url_changed(&self, _webview: WebView, _url: Url) {}
    /// The page title of the currently loaded page in this [`WebView`] has changed. The new
    /// title can accessed via [`WebView::page_title`].
    fn notify_page_title_changed(&self, _webview: WebView, _title: Option<String>) {}
    /// The status text of the currently loaded page in this [`WebView`] has changed. The new
    /// status text can accessed via [`WebView::status_text`].
    fn notify_status_text_changed(&self, _webview: WebView, _status: Option<String>) {}
    /// This [`WebView`] has either become focused or lost focus. Whether or not the
    /// [`WebView`] is focused can be accessed via [`WebView::focused`].
    fn notify_focus_changed(&self, _webview: WebView, _focused: bool) {}
    /// The `LoadStatus` of the currently loading or loaded page in this [`WebView`] has changed. The new
    /// status can accessed via [`WebView::load_status`].
    fn notify_load_status_changed(&self, _webview: WebView, _status: LoadStatus) {}
    /// The [`Cursor`] of the currently loaded page in this [`WebView`] has changed. The new
    /// cursor can accessed via [`WebView::cursor`].
    fn notify_cursor_changed(&self, _webview: WebView, _: Cursor) {}
    /// The favicon [`Url`] of the currently loaded page in this [`WebView`] has changed. The new
    /// favicon [`Url`] can accessed via [`WebView::favicon_url`].
    fn notify_favicon_url_changed(&self, _webview: WebView, _: Url) {}

    /// A [`WebView`] was created and is now ready to show in the user interface.
    fn notify_ready_to_show(&self, _webview: WebView) {}
    /// Notify the embedder that it needs to present a new frame.
    fn notify_new_frame_ready(&self, _webview: WebView) {}
    /// The given event was delivered to a pipeline in the given webview.
    fn notify_event_delivered(&self, _webview: WebView, _event: CompositorEventVariant) {}
    /// The history state has changed.
    // changed pattern; maybe wasteful if embedder doesn’t care?
    fn notify_history_changed(&self, _webview: WebView, _: Vec<Url>, _: usize) {}
    /// Page content has closed this [`WebView`] via `window.close()`. It's the embedder's
    /// responsibility to remove the [`WebView`] from the interface when this notification
    /// occurs.
    fn notify_closed(&self, _webview: WebView) {}

    /// A keyboard event has been sent to Servo, but remains unprocessed. This allows the
    /// embedding application to handle key events while first letting the [`WebView`]
    /// have an opportunity to handle it first. Apart from builtin keybindings, page
    /// content may expose custom keybindings as well.
    fn notify_keyboard_event(&self, _webview: WebView, _: KeyboardEvent) {}
    /// A pipeline in the webview panicked. First string is the reason, second one is the backtrace.
    fn notify_crashed(&self, _webview: WebView, _reason: String, _backtrace: Option<String>) {}
    /// Notifies the embedder about media session events
    /// (i.e. when there is metadata for the active media session, playback state changes...).
    fn notify_media_session_event(&self, _webview: WebView, _event: MediaSessionEvent) {}

    /// Whether or not to allow a [`WebView`] to load a URL in its main frame or one of its
    /// nested `<iframe>`s. [`NavigationRequest`]s are accepted by default.
    fn request_navigation(&self, _webview: WebView, _navigation_request: NavigationRequest) {}
    /// Whether or not to allow a [`WebView`]  to unload a `Document` in its main frame or one
    /// of its nested `<iframe>`s. By default, unloads are allowed.
    fn request_unload(&self, _webview: WebView, _unload_request: AllowOrDenyRequest) {}
    /// Move the window to a point
    fn request_move_to(&self, _webview: WebView, _: DeviceIntPoint) {}
    /// Resize the window to size
    fn request_resize_to(&self, _webview: WebView, _: DeviceIntSize) {}
    /// Whether or not to allow script to open a new `WebView`. If not handled by the
    /// embedder, these requests are automatically denied.
    fn request_open_auxiliary_webview(&self, _parent_webview: WebView) -> Option<WebView> {
        None
    }

    /// Content in a [`WebView`] is requesting permission to access a feature requiring
    /// permission from the user. The embedder should allow or deny the request, either by
    /// reading a cached value or querying the user for permission via the user interface.
    fn request_permission(&self, _webview: WebView, _: PermissionRequest) {}

    /// Show dialog to user
    /// TODO: This API needs to be reworked to match the new model of how responses are sent.
    fn show_prompt(&self, _webview: WebView, prompt: PromptDefinition, _: PromptOrigin) {
        let _ = match prompt {
            PromptDefinition::Alert(_, response_sender) => response_sender.send(()),
            PromptDefinition::OkCancel(_, response_sender) => {
                response_sender.send(embedder_traits::PromptResult::Dismissed)
            },
            PromptDefinition::Input(_, _, response_sender) => response_sender.send(None),
            PromptDefinition::Credentials(response_sender) => {
                response_sender.send(Default::default())
            },
        };
    }
    /// Show a context menu to the user
    fn show_context_menu(
        &self,
        _webview: WebView,
        result_sender: IpcSender<ContextMenuResult>,
        _: Option<String>,
        _: Vec<String>,
    ) {
        let _ = result_sender.send(ContextMenuResult::Ignored);
    }

    /// Enter or exit fullscreen
    fn request_fullscreen_state_change(&self, _webview: WebView, _: bool) {}
    /// Open dialog to select bluetooth device.
    /// TODO: This API needs to be reworked to match the new model of how responses are sent.
    fn show_bluetooth_device_dialog(
        &self,
        _webview: WebView,
        _: Vec<String>,
        response_sender: IpcSender<Option<String>>,
    ) {
        let _ = response_sender.send(None);
    }

    /// Open file dialog to select files. Set boolean flag to true allows to select multiple files.
    fn show_file_selection_dialog(
        &self,
        _webview: WebView,
        _filter_pattern: Vec<FilterPattern>,
        _allow_select_mutiple: bool,
        response_sender: IpcSender<Option<Vec<PathBuf>>>,
    ) {
        let _ = response_sender.send(None);
    }

    /// Request to present an IME to the user when an editable element is focused.
    /// If `type` is [`InputMethodType::Text`], then the `text` parameter specifies
    /// the pre-existing text content and the zero-based index into the string
    /// of the insertion point.
    fn show_ime(
        &self,
        _webview: WebView,
        _type: InputMethodType,
        _text: Option<(String, i32)>,
        _multiline: bool,
        _position: DeviceIntRect,
    ) {
    }

    /// Request to hide the IME when the editable element is blurred.
    fn hide_ime(&self, _webview: WebView) {}

    /// Request to play a haptic effect on a connected gamepad.
    fn play_gamepad_haptic_effect(
        &self,
        _webview: WebView,
        _: usize,
        _: GamepadHapticEffectType,
        _: IpcSender<bool>,
    ) {
    }
    /// Request to stop a haptic effect on a connected gamepad.
    fn stop_gamepad_haptic_effect(&self, _webview: WebView, _: usize, _: IpcSender<bool>) {}

    /// Potentially intercept a resource request. If not handled, the request will not be intercepted.
    ///
    /// Note: The `ServoDelegate` will also receive this notification and have a chance to intercept
    /// the request.
    ///
    /// TODO: This API needs to be reworked to match the new model of how responses are sent.
    fn intercept_web_resource_load(
        &self,
        _webview: WebView,
        _request: &WebResourceRequest,
        _response_sender: IpcSender<WebResourceResponseMsg>,
    ) {
    }
}

pub(crate) struct DefaultWebViewDelegate;
impl WebViewDelegate for DefaultWebViewDelegate {}
