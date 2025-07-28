/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;

use base::id::PipelineId;
use constellation_traits::EmbedderToConstellationMessage;
use embedder_traits::{
    AllowOrDeny, AuthenticationResponse, ContextMenuResult, Cursor, FilterPattern, FocusId,
    GamepadHapticEffectType, InputMethodType, KeyboardEvent, LoadStatus, MediaSessionEvent,
    Notification, PermissionFeature, RgbColor, ScreenGeometry, SelectElementOptionOrOptgroup,
    SimpleDialog, TraversalId, WebResourceRequest, WebResourceResponse, WebResourceResponseMsg,
};
use ipc_channel::ipc::IpcSender;
use serde::Serialize;
use url::Url;
use webrender_api::units::{DeviceIntPoint, DeviceIntRect, DeviceIntSize};

use crate::responders::ServoErrorSender;
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
            .send(EmbedderToConstellationMessage::AllowNavigationResponse(
                self.pipeline_id,
                true,
            ));
        self.response_sent = true;
    }

    pub fn deny(mut self) {
        self.constellation_proxy
            .send(EmbedderToConstellationMessage::AllowNavigationResponse(
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
                .send(EmbedderToConstellationMessage::AllowNavigationResponse(
                    self.pipeline_id,
                    true,
                ));
        }
    }
}

/// Sends a response over an IPC channel, or a default response on [`Drop`] if no response was sent.
pub(crate) struct IpcResponder<T: Serialize> {
    response_sender: IpcSender<T>,
    response_sent: bool,
    /// Always present, except when taken by [`Drop`].
    default_response: Option<T>,
}

impl<T: Serialize> IpcResponder<T> {
    pub(crate) fn new(response_sender: IpcSender<T>, default_response: T) -> Self {
        Self {
            response_sender,
            response_sent: false,
            default_response: Some(default_response),
        }
    }

    pub(crate) fn send(&mut self, response: T) -> bincode::Result<()> {
        let result = self.response_sender.send(response);
        self.response_sent = true;
        result
    }

    pub(crate) fn into_inner(self) -> IpcSender<T> {
        self.response_sender.clone()
    }
}

impl<T: Serialize> Drop for IpcResponder<T> {
    fn drop(&mut self) {
        if !self.response_sent {
            let response = self
                .default_response
                .take()
                .expect("Guaranteed by inherent impl");
            // Don’t notify embedder about send errors for the default response,
            // since they didn’t send anything and probably don’t care.
            let _ = self.response_sender.send(response);
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

pub struct AllowOrDenyRequest(IpcResponder<AllowOrDeny>, ServoErrorSender);

impl AllowOrDenyRequest {
    pub(crate) fn new(
        response_sender: IpcSender<AllowOrDeny>,
        default_response: AllowOrDeny,
        error_sender: ServoErrorSender,
    ) -> Self {
        Self(
            IpcResponder::new(response_sender, default_response),
            error_sender,
        )
    }

    pub fn allow(mut self) {
        if let Err(error) = self.0.send(AllowOrDeny::Allow) {
            self.1.raise_response_send_error(error);
        }
    }

    pub fn deny(mut self) {
        if let Err(error) = self.0.send(AllowOrDeny::Deny) {
            self.1.raise_response_send_error(error);
        }
    }
}

/// A request to authenticate a [`WebView`] navigation. Embedders may choose to prompt
/// the user to enter credentials or simply ignore this request (in which case credentials
/// will not be used).
pub struct AuthenticationRequest {
    pub(crate) url: Url,
    pub(crate) for_proxy: bool,
    pub(crate) responder: IpcResponder<Option<AuthenticationResponse>>,
    pub(crate) error_sender: ServoErrorSender,
}

impl AuthenticationRequest {
    pub(crate) fn new(
        url: Url,
        for_proxy: bool,
        response_sender: IpcSender<Option<AuthenticationResponse>>,
        error_sender: ServoErrorSender,
    ) -> Self {
        Self {
            url,
            for_proxy,
            responder: IpcResponder::new(response_sender, None),
            error_sender,
        }
    }

    /// The URL of the request that triggered this authentication.
    pub fn url(&self) -> &Url {
        &self.url
    }
    /// Whether or not this authentication request is associated with a proxy server authentication.
    pub fn for_proxy(&self) -> bool {
        self.for_proxy
    }
    /// Respond to the [`AuthenticationRequest`] with the given username and password.
    pub fn authenticate(mut self, username: String, password: String) {
        if let Err(error) = self
            .responder
            .send(Some(AuthenticationResponse { username, password }))
        {
            self.error_sender.raise_response_send_error(error);
        }
    }
}

/// Information related to the loading of a web resource. These are created for all HTTP requests.
/// The client may choose to intercept the load of web resources and send an alternate response
/// by calling [`WebResourceLoad::intercept`].
pub struct WebResourceLoad {
    pub request: WebResourceRequest,
    pub(crate) responder: IpcResponder<WebResourceResponseMsg>,
    pub(crate) error_sender: ServoErrorSender,
}

impl WebResourceLoad {
    pub(crate) fn new(
        web_resource_request: WebResourceRequest,
        response_sender: IpcSender<WebResourceResponseMsg>,
        error_sender: ServoErrorSender,
    ) -> Self {
        Self {
            request: web_resource_request,
            responder: IpcResponder::new(response_sender, WebResourceResponseMsg::DoNotIntercept),
            error_sender,
        }
    }

    /// The [`WebResourceRequest`] associated with this [`WebResourceLoad`].
    pub fn request(&self) -> &WebResourceRequest {
        &self.request
    }
    /// Intercept this [`WebResourceLoad`] and control the response via the returned
    /// [`InterceptedWebResourceLoad`].
    pub fn intercept(mut self, response: WebResourceResponse) -> InterceptedWebResourceLoad {
        if let Err(error) = self.responder.send(WebResourceResponseMsg::Start(response)) {
            self.error_sender.raise_response_send_error(error);
        }
        InterceptedWebResourceLoad {
            request: self.request.clone(),
            response_sender: self.responder.into_inner(),
            finished: false,
            error_sender: self.error_sender,
        }
    }
}

/// An intercepted web resource load. This struct allows the client to send an alternative response
/// after calling [`WebResourceLoad::intercept`]. In order to send chunks of body data, the client
/// must call [`InterceptedWebResourceLoad::send_body_data`]. When the interception is complete, the client
/// should call [`InterceptedWebResourceLoad::finish`]. If neither `finish()` or `cancel()` are called,
/// this interception will automatically be finished when dropped.
pub struct InterceptedWebResourceLoad {
    pub request: WebResourceRequest,
    pub(crate) response_sender: IpcSender<WebResourceResponseMsg>,
    pub(crate) finished: bool,
    pub(crate) error_sender: ServoErrorSender,
}

impl InterceptedWebResourceLoad {
    /// Send a chunk of response body data. It's possible to make subsequent calls to
    /// this method when streaming body data.
    pub fn send_body_data(&self, data: Vec<u8>) {
        if let Err(error) = self
            .response_sender
            .send(WebResourceResponseMsg::SendBodyData(data))
        {
            self.error_sender.raise_response_send_error(error);
        }
    }
    /// Finish this [`InterceptedWebResourceLoad`] and complete the response.
    pub fn finish(mut self) {
        if let Err(error) = self
            .response_sender
            .send(WebResourceResponseMsg::FinishLoad)
        {
            self.error_sender.raise_response_send_error(error);
        }
        self.finished = true;
    }
    /// Cancel this [`InterceptedWebResourceLoad`], which will trigger a network error.
    pub fn cancel(mut self) {
        if let Err(error) = self
            .response_sender
            .send(WebResourceResponseMsg::CancelLoad)
        {
            self.error_sender.raise_response_send_error(error);
        }
        self.finished = true;
    }
}

impl Drop for InterceptedWebResourceLoad {
    fn drop(&mut self) {
        if !self.finished {
            if let Err(error) = self
                .response_sender
                .send(WebResourceResponseMsg::FinishLoad)
            {
                self.error_sender.raise_response_send_error(error);
            }
        }
    }
}

/// The controls of an interactive form element.
pub enum FormControl {
    /// The picker of a `<select>` element.
    SelectElement(SelectElement),
    /// The picker of a `<input type=color>` element.
    ColorPicker(ColorPicker),
}

/// Represents a dialog triggered by clicking a `<select>` element.
pub struct SelectElement {
    pub(crate) options: Vec<SelectElementOptionOrOptgroup>,
    pub(crate) selected_option: Option<usize>,
    pub(crate) position: DeviceIntRect,
    pub(crate) responder: IpcResponder<Option<usize>>,
}

impl SelectElement {
    pub(crate) fn new(
        options: Vec<SelectElementOptionOrOptgroup>,
        selected_option: Option<usize>,
        position: DeviceIntRect,
        ipc_sender: IpcSender<Option<usize>>,
    ) -> Self {
        Self {
            options,
            selected_option,
            position,
            responder: IpcResponder::new(ipc_sender, None),
        }
    }

    /// Return the area occupied by the `<select>` element that triggered the prompt.
    ///
    /// The embedder should use this value to position the prompt that is shown to the user.
    pub fn position(&self) -> DeviceIntRect {
        self.position
    }

    /// Consecutive `<option>` elements outside of an `<optgroup>` will be combined
    /// into a single anonymous group, whose [`label`](SelectElementGroup::label) is `None`.
    pub fn options(&self) -> &[SelectElementOptionOrOptgroup] {
        &self.options
    }

    /// Mark a single option as selected.
    ///
    /// If there is already a selected option and the `<select>` element does not
    /// support selecting multiple options, then the previous option will be unselected.
    pub fn select(&mut self, id: Option<usize>) {
        self.selected_option = id;
    }

    pub fn selected_option(&self) -> Option<usize> {
        self.selected_option
    }

    /// Resolve the prompt with the options that have been selected by calling [select] previously.
    pub fn submit(mut self) {
        let _ = self.responder.send(self.selected_option);
    }
}

/// Represents a dialog triggered by clicking a `<input type=color>` element.
pub struct ColorPicker {
    pub(crate) current_color: RgbColor,
    pub(crate) position: DeviceIntRect,
    pub(crate) responder: IpcResponder<Option<RgbColor>>,
    pub(crate) error_sender: ServoErrorSender,
}

impl ColorPicker {
    pub(crate) fn new(
        current_color: RgbColor,
        position: DeviceIntRect,
        ipc_sender: IpcSender<Option<RgbColor>>,
        error_sender: ServoErrorSender,
    ) -> Self {
        Self {
            current_color,
            position,
            responder: IpcResponder::new(ipc_sender, None),
            error_sender,
        }
    }

    /// Get the area occupied by the `<input>` element that triggered the prompt.
    ///
    /// The embedder should use this value to position the prompt that is shown to the user.
    pub fn position(&self) -> DeviceIntRect {
        self.position
    }

    /// Get the color that was selected before the prompt was opened.
    pub fn current_color(&self) -> RgbColor {
        self.current_color
    }

    pub fn select(&mut self, color: Option<RgbColor>) {
        if let Err(error) = self.responder.send(color) {
            self.error_sender.raise_response_send_error(error);
        }
    }
}

pub trait WebViewDelegate {
    /// Get the [`ScreenGeometry`] for this [`WebView`]. If this is unimplemented or returns `None`
    /// the screen will have the size of the [`WebView`]'s `RenderingContext` and `WebView` will be
    /// considered to be positioned at the screen's origin.
    fn screen_geometry(&self, _webview: WebView) -> Option<ScreenGeometry> {
        None
    }
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
    /// A focus operation that was initiated by this webview has completed.
    /// The current focus status of this [`WebView`] can be accessed via [`WebView::focused`].
    fn notify_focus_complete(&self, _webview: WebView, _focus_id: FocusId) {}
    /// This [`WebView`] has either started to animate or stopped animating. When a
    /// [`WebView`] is animating, it is up to the embedding application ensure that
    /// `Servo::spin_event_loop` is called at regular intervals in order to update the
    /// painted contents of the [`WebView`].
    fn notify_animating_changed(&self, _webview: WebView, _animating: bool) {}
    /// The `LoadStatus` of the currently loading or loaded page in this [`WebView`] has changed. The new
    /// status can accessed via [`WebView::load_status`].
    fn notify_load_status_changed(&self, _webview: WebView, _status: LoadStatus) {}
    /// The [`Cursor`] of the currently loaded page in this [`WebView`] has changed. The new
    /// cursor can accessed via [`WebView::cursor`].
    fn notify_cursor_changed(&self, _webview: WebView, _: Cursor) {}
    /// The favicon [`Url`] of the currently loaded page in this [`WebView`] has changed. The new
    /// favicon [`Url`] can accessed via [`WebView::favicon_url`].
    fn notify_favicon_url_changed(&self, _webview: WebView, _: Url) {}

    /// Notify the embedder that it needs to present a new frame.
    fn notify_new_frame_ready(&self, _webview: WebView) {}
    /// The history state has changed.
    // changed pattern; maybe wasteful if embedder doesn’t care?
    fn notify_history_changed(&self, _webview: WebView, _: Vec<Url>, _: usize) {}
    /// A history traversal operation is complete.
    fn notify_traversal_complete(&self, _webview: WebView, _: TraversalId) {}
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
    /// A notification that the [`WebView`] has entered or exited fullscreen mode. This is an
    /// opportunity for the embedder to transition the containing window into or out of fullscreen
    /// mode and to show or hide extra UI elements. Regardless of how the notification is handled,
    /// the page will enter or leave fullscreen state internally according to the [Fullscreen
    /// API](https://fullscreen.spec.whatwg.org/).
    fn notify_fullscreen_state_changed(&self, _webview: WebView, _: bool) {}

    /// Whether or not to allow a [`WebView`] to load a URL in its main frame or one of its
    /// nested `<iframe>`s. [`NavigationRequest`]s are accepted by default.
    fn request_navigation(&self, _webview: WebView, _navigation_request: NavigationRequest) {}
    /// Whether or not to allow a [`WebView`]  to unload a `Document` in its main frame or one
    /// of its nested `<iframe>`s. By default, unloads are allowed.
    fn request_unload(&self, _webview: WebView, _unload_request: AllowOrDenyRequest) {}
    /// Move the window to a point.
    fn request_move_to(&self, _webview: WebView, _: DeviceIntPoint) {}
    /// Try to resize the window that contains this [`WebView`] to the provided outer size.
    fn request_resize_to(&self, _webview: WebView, _requested_outer_size: DeviceIntSize) {}
    /// Whether or not to allow script to open a new `WebView`. If not handled by the
    /// embedder, these requests are automatically denied.
    fn request_open_auxiliary_webview(&self, _parent_webview: WebView) -> Option<WebView> {
        None
    }

    /// Content in a [`WebView`] is requesting permission to access a feature requiring
    /// permission from the user. The embedder should allow or deny the request, either by
    /// reading a cached value or querying the user for permission via the user interface.
    fn request_permission(&self, _webview: WebView, _: PermissionRequest) {}

    fn request_authentication(
        &self,
        _webview: WebView,
        _authentication_request: AuthenticationRequest,
    ) {
    }

    /// Show the user a [simple dialog](https://html.spec.whatwg.org/multipage/#simple-dialogs) (`alert()`, `confirm()`,
    /// or `prompt()`). Since their messages are controlled by web content, they should be presented to the user in a
    /// way that makes them impossible to mistake for browser UI.
    /// TODO: This API needs to be reworked to match the new model of how responses are sent.
    fn show_simple_dialog(&self, _webview: WebView, dialog: SimpleDialog) {
        // Return the DOM-specified default value for when we **cannot show simple dialogs**.
        let _ = match dialog {
            SimpleDialog::Alert {
                response_sender, ..
            } => response_sender.send(Default::default()),
            SimpleDialog::Confirm {
                response_sender, ..
            } => response_sender.send(Default::default()),
            SimpleDialog::Prompt {
                response_sender, ..
            } => response_sender.send(Default::default()),
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

    /// Request that the embedder show UI elements for form controls that are not integrated
    /// into page content, such as dropdowns for `<select>` elements.
    fn show_form_control(&self, _webview: WebView, _form_control: FormControl) {}

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

    /// Triggered when this [`WebView`] will load a web (HTTP/HTTPS) resource. The load may be
    /// intercepted and alternate contents can be loaded by the client by calling
    /// [`WebResourceLoad::intercept`]. If not handled, the load will continue as normal.
    ///
    /// Note: This delegate method is called for all resource loads associated with a [`WebView`].
    /// For loads not associated with a [`WebView`], such as those for service workers, Servo
    /// will call [`crate::ServoDelegate::load_web_resource`].
    fn load_web_resource(&self, _webview: WebView, _load: WebResourceLoad) {}

    /// Request to display a notification.
    fn show_notification(&self, _webview: WebView, _notification: Notification) {}
}

pub(crate) struct DefaultWebViewDelegate;
impl WebViewDelegate for DefaultWebViewDelegate {}

#[test]
fn test_allow_deny_request() {
    use ipc_channel::ipc;

    use crate::ServoErrorChannel;

    for default_response in [AllowOrDeny::Allow, AllowOrDeny::Deny] {
        // Explicit allow yields allow and nothing else
        let errors = ServoErrorChannel::default();
        let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel");
        let request = AllowOrDenyRequest::new(sender, default_response, errors.sender());
        request.allow();
        assert_eq!(receiver.try_recv().ok(), Some(AllowOrDeny::Allow));
        assert_eq!(receiver.try_recv().ok(), None);
        assert!(errors.try_recv().is_none());

        // Explicit deny yields deny and nothing else
        let errors = ServoErrorChannel::default();
        let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel");
        let request = AllowOrDenyRequest::new(sender, default_response, errors.sender());
        request.deny();
        assert_eq!(receiver.try_recv().ok(), Some(AllowOrDeny::Deny));
        assert_eq!(receiver.try_recv().ok(), None);
        assert!(errors.try_recv().is_none());

        // No response yields default response and nothing else
        let errors = ServoErrorChannel::default();
        let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel");
        let request = AllowOrDenyRequest::new(sender, default_response, errors.sender());
        drop(request);
        assert_eq!(receiver.try_recv().ok(), Some(default_response));
        assert_eq!(receiver.try_recv().ok(), None);
        assert!(errors.try_recv().is_none());

        // Explicit allow when receiver disconnected yields error
        let errors = ServoErrorChannel::default();
        let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel");
        let request = AllowOrDenyRequest::new(sender, default_response, errors.sender());
        drop(receiver);
        request.allow();
        assert!(errors.try_recv().is_some());

        // Explicit deny when receiver disconnected yields error
        let errors = ServoErrorChannel::default();
        let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel");
        let request = AllowOrDenyRequest::new(sender, default_response, errors.sender());
        drop(receiver);
        request.deny();
        assert!(errors.try_recv().is_some());

        // No response when receiver disconnected yields no error
        let errors = ServoErrorChannel::default();
        let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel");
        let request = AllowOrDenyRequest::new(sender, default_response, errors.sender());
        drop(receiver);
        drop(request);
        assert!(errors.try_recv().is_none());
    }
}

#[test]
fn test_authentication_request() {
    use ipc_channel::ipc;

    use crate::ServoErrorChannel;

    let url = Url::parse("https://example.com").expect("Guaranteed by argument");

    // Explicit response yields that response and nothing else
    let errors = ServoErrorChannel::default();
    let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel");
    let request = AuthenticationRequest::new(url.clone(), false, sender, errors.sender());
    request.authenticate("diffie".to_owned(), "hunter2".to_owned());
    assert_eq!(
        receiver.try_recv().ok(),
        Some(Some(AuthenticationResponse {
            username: "diffie".to_owned(),
            password: "hunter2".to_owned(),
        }))
    );
    assert_eq!(receiver.try_recv().ok(), None);
    assert!(errors.try_recv().is_none());

    // No response yields None response and nothing else
    let errors = ServoErrorChannel::default();
    let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel");
    let request = AuthenticationRequest::new(url.clone(), false, sender, errors.sender());
    drop(request);
    assert_eq!(receiver.try_recv().ok(), Some(None));
    assert_eq!(receiver.try_recv().ok(), None);
    assert!(errors.try_recv().is_none());

    // Explicit response when receiver disconnected yields error
    let errors = ServoErrorChannel::default();
    let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel");
    let request = AuthenticationRequest::new(url.clone(), false, sender, errors.sender());
    drop(receiver);
    request.authenticate("diffie".to_owned(), "hunter2".to_owned());
    assert!(errors.try_recv().is_some());

    // No response when receiver disconnected yields no error
    let errors = ServoErrorChannel::default();
    let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel");
    let request = AuthenticationRequest::new(url.clone(), false, sender, errors.sender());
    drop(receiver);
    drop(request);
    assert!(errors.try_recv().is_none());
}

#[test]
fn test_web_resource_load() {
    use http::{HeaderMap, Method, StatusCode};
    use ipc_channel::ipc;

    use crate::ServoErrorChannel;

    let web_resource_request = || WebResourceRequest {
        method: Method::GET,
        headers: HeaderMap::default(),
        url: Url::parse("https://example.com").expect("Guaranteed by argument"),
        is_for_main_frame: false,
        is_redirect: false,
    };
    let web_resource_response = || {
        WebResourceResponse::new(Url::parse("https://diffie.test").expect("Guaranteed by argument"))
            .status_code(StatusCode::IM_A_TEAPOT)
    };

    // Explicit intercept with explicit cancel yields Start and Cancel and nothing else
    let errors = ServoErrorChannel::default();
    let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel");
    let request = WebResourceLoad::new(web_resource_request(), sender, errors.sender());
    request.intercept(web_resource_response()).cancel();
    assert!(matches!(
        receiver.try_recv(),
        Ok(WebResourceResponseMsg::Start(_))
    ));
    assert!(matches!(
        receiver.try_recv(),
        Ok(WebResourceResponseMsg::CancelLoad)
    ));
    assert!(matches!(receiver.try_recv(), Err(_)));
    assert!(errors.try_recv().is_none());

    // Explicit intercept with no further action yields Start and FinishLoad and nothing else
    let errors = ServoErrorChannel::default();
    let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel");
    let request = WebResourceLoad::new(web_resource_request(), sender, errors.sender());
    drop(request.intercept(web_resource_response()));
    assert!(matches!(
        receiver.try_recv(),
        Ok(WebResourceResponseMsg::Start(_))
    ));
    assert!(matches!(
        receiver.try_recv(),
        Ok(WebResourceResponseMsg::FinishLoad)
    ));
    assert!(matches!(receiver.try_recv(), Err(_)));
    assert!(errors.try_recv().is_none());

    // No response yields DoNotIntercept and nothing else
    let errors = ServoErrorChannel::default();
    let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel");
    let request = WebResourceLoad::new(web_resource_request(), sender, errors.sender());
    drop(request);
    assert!(matches!(
        receiver.try_recv(),
        Ok(WebResourceResponseMsg::DoNotIntercept)
    ));
    assert!(matches!(receiver.try_recv(), Err(_)));
    assert!(errors.try_recv().is_none());

    // Explicit intercept with explicit cancel when receiver disconnected yields error
    let errors = ServoErrorChannel::default();
    let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel");
    let request = WebResourceLoad::new(web_resource_request(), sender, errors.sender());
    drop(receiver);
    request.intercept(web_resource_response()).cancel();
    assert!(errors.try_recv().is_some());

    // Explicit intercept with no further action when receiver disconnected yields error
    let errors = ServoErrorChannel::default();
    let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel");
    let request = WebResourceLoad::new(web_resource_request(), sender, errors.sender());
    drop(receiver);
    drop(request.intercept(web_resource_response()));
    assert!(errors.try_recv().is_some());

    // No response when receiver disconnected yields no error
    let errors = ServoErrorChannel::default();
    let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel");
    let request = WebResourceLoad::new(web_resource_request(), sender, errors.sender());
    drop(receiver);
    drop(request);
    assert!(errors.try_recv().is_none());
}
