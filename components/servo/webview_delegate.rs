/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;
use std::rc::Rc;

use accesskit::TreeUpdate;
use base::generic_channel::GenericSender;
use base::id::PipelineId;
use compositing_traits::rendering_context::RenderingContext;
use constellation_traits::EmbedderToConstellationMessage;
#[cfg(feature = "gamepad")]
use embedder_traits::GamepadHapticEffectType;
use embedder_traits::{
    AlertResponse, AllowOrDeny, AuthenticationResponse, ConfirmResponse, ConsoleLogLevel,
    ContextMenuAction, ContextMenuElementInformation, ContextMenuItem, Cursor, EmbedderControlId,
    EmbedderControlResponse, FilePickerRequest, FilterPattern, InputEventId, InputEventResult,
    InputMethodType, LoadStatus, MediaSessionEvent, NewWebViewDetails, Notification,
    PermissionFeature, PromptResponse, RgbColor, ScreenGeometry, SelectElementOptionOrOptgroup,
    SimpleDialogRequest, TraversalId, WebResourceRequest, WebResourceResponse,
    WebResourceResponseMsg,
};
use url::Url;
use webrender_api::units::{DeviceIntPoint, DeviceIntRect, DeviceIntSize};

use crate::proxies::ConstellationProxy;
use crate::responders::{IpcResponder, ServoErrorSender};
use crate::{RegisterOrUnregister, Servo, WebView, WebViewBuilder};

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
        response_sender: GenericSender<AllowOrDeny>,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolHandlerRegistration {
    pub scheme: String,
    pub url: Url,
    pub register_or_unregister: RegisterOrUnregister,
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
        response_sender: GenericSender<Option<AuthenticationResponse>>,
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
        response_sender: GenericSender<WebResourceResponseMsg>,
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
    pub(crate) response_sender: GenericSender<WebResourceResponseMsg>,
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
pub enum EmbedderControl {
    /// The picker of a `<select>` element.
    SelectElement(SelectElement),
    /// The picker of a `<input type=color>` element.
    ColorPicker(ColorPicker),
    /// The picker of a `<input type=file>` element.
    FilePicker(FilePicker),
    /// Request to present an input method (IME) interface to the user when an
    /// editable element is focused.
    InputMethod(InputMethodControl),
    /// A [simple dialog](https://html.spec.whatwg.org/multipage/#simple-dialogs) initiated by
    /// script (`alert()`, `confirm()`, or `prompt()`). Since their messages are controlled by web
    /// content, they should be presented to the user in a way that makes them impossible to
    /// mistake for browser UI.
    SimpleDialog(SimpleDialog),
    /// A context menu. This can be triggered by things like right-clicking on web content.
    /// The menu that is actually shown the user may be customized, but custom menu entries
    /// must be handled by the embedder.
    ContextMenu(ContextMenu),
}

impl EmbedderControl {
    pub fn id(&self) -> EmbedderControlId {
        match self {
            EmbedderControl::SelectElement(select_element) => select_element.id,
            EmbedderControl::ColorPicker(color_picker) => color_picker.id,
            EmbedderControl::FilePicker(file_picker) => file_picker.id,
            EmbedderControl::InputMethod(input_method) => input_method.id,
            EmbedderControl::SimpleDialog(simple_dialog) => simple_dialog.id(),
            EmbedderControl::ContextMenu(context_menu) => context_menu.id,
        }
    }
}

/// Represents a context menu opened on web content.
pub struct ContextMenu {
    pub(crate) id: EmbedderControlId,
    pub(crate) position: DeviceIntRect,
    pub(crate) items: Vec<ContextMenuItem>,
    pub(crate) element_info: ContextMenuElementInformation,
    pub(crate) response_sent: bool,
    pub(crate) constellation_proxy: ConstellationProxy,
}

impl ContextMenu {
    /// Return the [`EmbedderControlId`] associated with this element.
    pub fn id(&self) -> EmbedderControlId {
        self.id
    }

    /// Return the area occupied by the element on which this context menu was triggered.
    ///
    /// The embedder should use this value to position the prompt that is shown to the user.
    pub fn position(&self) -> DeviceIntRect {
        self.position
    }

    /// A [`ContextMenuElementInformation`] giving details about the element that this [`ContextMenu`]
    /// was activated on.
    pub fn element_info(&self) -> &ContextMenuElementInformation {
        &self.element_info
    }

    /// Resolve the context menu by activating the given context menu action.
    pub fn items(&self) -> &[ContextMenuItem] {
        &self.items
    }

    /// Resolve the context menu by activating the given context menu action.
    pub fn select(mut self, action: ContextMenuAction) {
        self.constellation_proxy
            .send(EmbedderToConstellationMessage::EmbedderControlResponse(
                self.id,
                EmbedderControlResponse::ContextMenu(Some(action)),
            ));
        self.response_sent = true;
    }

    /// Tell Servo that the context menu was dismissed with no selection.
    pub fn dismiss(mut self) {
        self.constellation_proxy
            .send(EmbedderToConstellationMessage::EmbedderControlResponse(
                self.id,
                EmbedderControlResponse::ContextMenu(None),
            ));
        self.response_sent = true;
    }
}

impl Drop for ContextMenu {
    fn drop(&mut self) {
        if !self.response_sent {
            self.constellation_proxy
                .send(EmbedderToConstellationMessage::EmbedderControlResponse(
                    self.id,
                    EmbedderControlResponse::ContextMenu(None),
                ));
        }
    }
}

/// Represents a dialog triggered by clicking a `<select>` element.
pub struct SelectElement {
    pub(crate) id: EmbedderControlId,
    pub(crate) options: Vec<SelectElementOptionOrOptgroup>,
    pub(crate) selected_option: Option<usize>,
    pub(crate) position: DeviceIntRect,
    pub(crate) constellation_proxy: ConstellationProxy,
    pub(crate) response_sent: bool,
}

impl SelectElement {
    /// Return the [`EmbedderControlId`] associated with this element.
    pub fn id(&self) -> EmbedderControlId {
        self.id
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
        self.response_sent = true;
        self.constellation_proxy
            .send(EmbedderToConstellationMessage::EmbedderControlResponse(
                self.id,
                EmbedderControlResponse::SelectElement(self.selected_option()),
            ));
    }
}

impl Drop for SelectElement {
    fn drop(&mut self) {
        if !self.response_sent {
            self.constellation_proxy
                .send(EmbedderToConstellationMessage::EmbedderControlResponse(
                    self.id,
                    EmbedderControlResponse::SelectElement(self.selected_option()),
                ));
        }
    }
}

/// Represents a dialog triggered by clicking a `<input type=color>` element.
pub struct ColorPicker {
    pub(crate) id: EmbedderControlId,
    pub(crate) current_color: Option<RgbColor>,
    pub(crate) position: DeviceIntRect,
    pub(crate) constellation_proxy: ConstellationProxy,
    pub(crate) response_sent: bool,
}

impl ColorPicker {
    /// Return the [`EmbedderControlId`] associated with this element.
    pub fn id(&self) -> EmbedderControlId {
        self.id
    }

    /// Get the area occupied by the `<input>` element that triggered the prompt.
    ///
    /// The embedder should use this value to position the prompt that is shown to the user.
    pub fn position(&self) -> DeviceIntRect {
        self.position
    }

    /// Get the currently selected color for this [`ColorPicker`]. This is initially the selected color
    /// before the picker is opened.
    pub fn current_color(&self) -> Option<RgbColor> {
        self.current_color
    }

    pub fn select(&mut self, color: Option<RgbColor>) {
        self.current_color = color;
    }

    /// Resolve the prompt with the options that have been selected by calling [select] previously.
    pub fn submit(mut self) {
        self.response_sent = true;
        self.constellation_proxy
            .send(EmbedderToConstellationMessage::EmbedderControlResponse(
                self.id,
                EmbedderControlResponse::ColorPicker(self.current_color),
            ));
    }
}

impl Drop for ColorPicker {
    fn drop(&mut self) {
        if !self.response_sent {
            self.constellation_proxy
                .send(EmbedderToConstellationMessage::EmbedderControlResponse(
                    self.id,
                    EmbedderControlResponse::ColorPicker(self.current_color),
                ));
        }
    }
}

/// Represents a dialog triggered by clicking a `<input type=color>` element.
pub struct FilePicker {
    pub(crate) id: EmbedderControlId,
    pub(crate) file_picker_request: FilePickerRequest,
    pub(crate) response_sender: GenericSender<Option<Vec<PathBuf>>>,
    pub(crate) response_sent: bool,
}

impl FilePicker {
    /// Return the [`EmbedderControlId`] associated with this element.
    pub fn id(&self) -> EmbedderControlId {
        self.id
    }

    pub fn filter_patterns(&self) -> &[FilterPattern] {
        &self.file_picker_request.filter_patterns
    }

    pub fn allow_select_multiple(&self) -> bool {
        self.file_picker_request.allow_select_multiple
    }

    /// Get the currently selected files in this [`FilePicker`]. This is initially the files that
    /// were previously selected before the picker is opened.
    pub fn current_paths(&self) -> &[PathBuf] {
        &self.file_picker_request.current_paths
    }

    pub fn select(&mut self, paths: &[PathBuf]) {
        self.file_picker_request.current_paths = paths.to_owned();
    }

    /// Resolve the prompt with the options that have been selected by calling [select] previously.
    pub fn submit(mut self) {
        let _ = self.response_sender.send(Some(std::mem::take(
            &mut self.file_picker_request.current_paths,
        )));
        self.response_sent = true;
    }

    /// Tell Servo that the file picker was dismissed with no selection.
    pub fn dismiss(mut self) {
        let _ = self.response_sender.send(None);
        self.response_sent = true;
    }
}

impl Drop for FilePicker {
    fn drop(&mut self) {
        if !self.response_sent {
            let _ = self.response_sender.send(None);
        }
    }
}

/// Represents a request to enable the system input method interface.
pub struct InputMethodControl {
    pub(crate) id: EmbedderControlId,
    pub(crate) input_method_type: InputMethodType,
    pub(crate) text: String,
    pub(crate) insertion_point: Option<u32>,
    pub(crate) position: DeviceIntRect,
    pub(crate) multiline: bool,
}

impl InputMethodControl {
    /// Return the type of input method that initated this request.
    pub fn input_method_type(&self) -> InputMethodType {
        self.input_method_type
    }

    /// Return the current string value of the input field.
    pub fn text(&self) -> String {
        self.text.clone()
    }

    /// The current zero-based insertion point / cursor position if it is within the field or `None`
    /// if it is not.
    pub fn insertion_point(&self) -> Option<u32> {
        self.insertion_point
    }

    /// Get the area occupied by the `<input>` element that triggered the input method.
    ///
    /// The embedder should use this value to position the input method interface that is
    /// shown to the user.
    pub fn position(&self) -> DeviceIntRect {
        self.position
    }

    /// Whether or not this field is a multiline field.
    pub fn multiline(&self) -> bool {
        self.multiline
    }
}

/// [Simple dialogs](https://html.spec.whatwg.org/multipage/#simple-dialogs) are synchronous dialogs
/// that can be opened by web content. Since their messages are controlled by web content, they
/// should be presented to the user in a way that makes them impossible to mistake for browser UI.
pub enum SimpleDialog {
    Alert(AlertDialog),
    Confirm(ConfirmDialog),
    Prompt(PromptDialog),
}

impl SimpleDialog {
    pub fn message(&self) -> &str {
        match self {
            SimpleDialog::Alert(alert_dialog) => alert_dialog.message(),
            SimpleDialog::Confirm(confirm_dialog) => confirm_dialog.message(),
            SimpleDialog::Prompt(prompt_dialog) => prompt_dialog.message(),
        }
    }

    pub fn confirm(self) {
        match self {
            SimpleDialog::Alert(alert_dialog) => alert_dialog.confirm(),
            SimpleDialog::Confirm(confirm_dialog) => confirm_dialog.confirm(),
            SimpleDialog::Prompt(prompt_dialog) => prompt_dialog.confirm(),
        }
    }

    pub fn dismiss(self) {
        match self {
            SimpleDialog::Alert(alert_dialog) => alert_dialog.confirm(),
            SimpleDialog::Confirm(confirm_dialog) => confirm_dialog.dismiss(),
            SimpleDialog::Prompt(prompt_dialog) => prompt_dialog.dismiss(),
        }
    }
}

impl SimpleDialog {
    fn id(&self) -> EmbedderControlId {
        match self {
            SimpleDialog::Alert(alert_dialog) => alert_dialog.id,
            SimpleDialog::Confirm(confirm_dialog) => confirm_dialog.id,
            SimpleDialog::Prompt(prompt_dialog) => prompt_dialog.id,
        }
    }
}

impl From<SimpleDialogRequest> for SimpleDialog {
    fn from(simple_dialog_request: SimpleDialogRequest) -> Self {
        match simple_dialog_request {
            SimpleDialogRequest::Alert {
                id,
                message,
                response_sender,
            } => Self::Alert(AlertDialog {
                id,
                message,
                response_sender,
                response_sent: false,
            }),
            SimpleDialogRequest::Confirm {
                id,
                message,
                response_sender,
            } => Self::Confirm(ConfirmDialog {
                id,
                message,
                response_sender,
                response_sent: false,
            }),
            SimpleDialogRequest::Prompt {
                id,
                message,
                default,
                response_sender,
            } => Self::Prompt(PromptDialog {
                id,
                message,
                current_value: default,
                response_sender,
                response_sent: false,
            }),
        }
    }
}

/// [`alert()`](https://html.spec.whatwg.org/multipage/#dom-alert).
///
/// The confirm dialog is expected to be represented by a message and an "Ok" button.
/// Pressing "Ok" always causes the DOM API to return `undefined`.
pub struct AlertDialog {
    id: EmbedderControlId,
    message: String,
    response_sender: GenericSender<AlertResponse>,
    response_sent: bool,
}

impl Drop for AlertDialog {
    fn drop(&mut self) {
        if !self.response_sent {
            let _ = self.response_sender.send(AlertResponse::Ok);
        }
    }
}

impl AlertDialog {
    pub fn message(&self) -> &str {
        &self.message
    }

    /// This should be called when the dialog button is pressed.
    pub fn confirm(self) {
        // The result will be send via the `Drop` implementation.
    }
}

/// [`confirm()`](https://html.spec.whatwg.org/multipage/#dom-confirm).
///
/// The confirm dialog is expected to be represented by a message and "Ok" and "Cancel"
/// buttons. When "Ok" is selected `true` is sent as a response to the DOM API, while
/// "Cancel" will send `false`.
pub struct ConfirmDialog {
    id: EmbedderControlId,
    message: String,
    response_sender: GenericSender<ConfirmResponse>,
    response_sent: bool,
}

impl ConfirmDialog {
    pub fn message(&self) -> &str {
        &self.message
    }

    /// This should be called when the dialog "Cancel" button is pressed.
    pub fn dismiss(mut self) {
        let _ = self.response_sender.send(ConfirmResponse::Cancel);
        self.response_sent = true;
    }

    /// This should be called when the dialog "Ok" button is pressed.
    pub fn confirm(mut self) {
        let _ = self.response_sender.send(ConfirmResponse::Ok);
        self.response_sent = true;
    }
}

impl Drop for ConfirmDialog {
    fn drop(&mut self) {
        if !self.response_sent {
            let _ = self.response_sender.send(ConfirmResponse::Cancel);
        }
    }
}

/// A [`prompt()`](https://html.spec.whatwg.org/multipage/#dom-prompt).
///
/// The prompt dialog is expected to be represented by a mesage, a text entry field, and
/// an "Ok" and "Cancel" buttons. When "Ok" is selected the current prompt value is sent
/// as the response to the DOM API. A default value may be sent with the [`PromptDialog`],
/// which be be retrieved by calling [`Self::current_value`]. Before calling [`Self::ok`]
/// or as the prompt field changes, the embedder is expected to call
/// [`Self::set_current_value`].
pub struct PromptDialog {
    id: EmbedderControlId,
    message: String,
    current_value: String,
    response_sender: GenericSender<PromptResponse>,
    response_sent: bool,
}

impl Drop for PromptDialog {
    fn drop(&mut self) {
        if !self.response_sent {
            let _ = self.response_sender.send(PromptResponse::Cancel);
        }
    }
}

impl PromptDialog {
    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn current_value(&self) -> &str {
        &self.current_value
    }

    pub fn set_current_value(&mut self, new_value: &str) {
        self.current_value = new_value.to_owned()
    }

    /// This should be called when the dialog "Cancel" button is pressed.
    pub fn dismiss(mut self) {
        let _ = self.response_sender.send(PromptResponse::Cancel);
        self.response_sent = true;
    }

    /// This should be called when the dialog "Ok" button is pressed, the current prompt value will
    /// be sent to web content.
    pub fn confirm(mut self) {
        let _ = self
            .response_sender
            .send(PromptResponse::Ok(self.current_value.clone()));
        self.response_sent = true;
    }
}

pub struct CreateNewWebViewRequest {
    pub(crate) servo: Servo,
    pub(crate) responder: IpcResponder<Option<NewWebViewDetails>>,
}

impl CreateNewWebViewRequest {
    pub fn builder(self, rendering_context: Rc<dyn RenderingContext>) -> WebViewBuilder {
        WebViewBuilder::new_for_create_request(&self.servo, rendering_context, self.responder)
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
    /// The favicon of the currently loaded page in this [`WebView`] has changed. The new
    /// favicon [`Image`] can accessed via [`WebView::favicon`].
    fn notify_favicon_changed(&self, _webview: WebView) {}
    /// Notify the embedder that it needs to present a new frame.
    fn notify_new_frame_ready(&self, _webview: WebView) {}
    /// The navigation history of this [`WebView`] has changed. The navigation history is represented
    /// as a `Vec<Url>` and `_current` denotes the current index in the history. New navigations,
    /// back navigation, and forward navigation modify this index.
    fn notify_history_changed(&self, _webview: WebView, _entries: Vec<Url>, _current: usize) {}
    /// A history traversal operation is complete.
    fn notify_traversal_complete(&self, _webview: WebView, _: TraversalId) {}
    /// Page content has closed this [`WebView`] via `window.close()`. It's the embedder's
    /// responsibility to remove the [`WebView`] from the interface when this notification
    /// occurs.
    fn notify_closed(&self, _webview: WebView) {}

    /// An input event passed to this [`WebView`] via [`WebView::notify_input_event`] has been handled
    /// by Servo. This allows post-procesing of input events, such as chaining up unhandled events
    /// to parent UI elements.
    fn notify_input_event_handled(&self, _webview: WebView, _: InputEventId, _: InputEventResult) {}
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
    /// Whether or not to allow a [`WebView`] to (un)register a protocol handler (e.g. `mailto:`).
    /// Typically an embedder application will show a permissions prompt when this happens
    /// to confirm a protocol handler is allowed. By default, requests are denied.
    /// For more information, see the specification:
    /// <https://html.spec.whatwg.org/multipage/#custom-handlers>
    fn request_protocol_handler(
        &self,
        _webview: WebView,
        _protocol_handler_registration: ProtocolHandlerRegistration,
        _allow_deny_request: AllowOrDenyRequest,
    ) {
    }
    /// Try to resize the window that contains this [`WebView`] to the provided outer
    /// size. These resize requests can come from page content. Servo will ensure that the
    /// values are greater than zero, but it is up to the embedder to limit the maximum
    /// size. For instance, a reasonable limitation might be that the final size is no
    /// larger than the screen size.
    fn request_resize_to(&self, _webview: WebView, _requested_outer_size: DeviceIntSize) {}
    /// This method is called when web content makes a request to open a new
    /// `WebView`, such as via the [`window.open`] DOM API. If this request is
    /// ignored, no new `WebView` will be opened. Embedders can handle this method by
    /// using the provided [`CreateNewWebViewRequest`] to build a new `WebView`.
    ///
    /// ```rust
    /// fn request_create_new(&self, parent_webview: WebView, request: CreateNewWebViewRequest) {
    ///     let webview = request
    ///         .builder(self.rendering_context())
    ///         .delegate(parent_webview.delegate())
    ///         .build();
    ///     self.register_webview(webview);
    /// }
    /// ```
    ///
    /// **Important:** It is important to keep a live handle to the new `WebView` in the application or
    /// it will be immediately destroyed.
    ///
    /// [`window.open`]: https://developer.mozilla.org/en-US/docs/Web/API/Window/open
    fn request_create_new(&self, _parent_webview: WebView, _: CreateNewWebViewRequest) {}
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

    /// Open dialog to select bluetooth device.
    /// TODO: This API needs to be reworked to match the new model of how responses are sent.
    fn show_bluetooth_device_dialog(
        &self,
        _webview: WebView,
        _: Vec<String>,
        response_sender: GenericSender<Option<String>>,
    ) {
        let _ = response_sender.send(None);
    }

    /// Request that the embedder show UI elements for form controls that are not integrated
    /// into page content, such as dropdowns for `<select>` elements.
    fn show_embedder_control(&self, _webview: WebView, _embedder_control: EmbedderControl) {}

    /// Request that the embedder hide and ignore a previous [`EmbedderControl`] request, if it hasn’t
    /// already responded to it.
    ///
    /// After this point, any further responses to that request will be ignored.
    fn hide_embedder_control(&self, _webview: WebView, _control_id: EmbedderControlId) {}

    /// Request to play a haptic effect on a connected gamepad. The embedder is expected to
    /// call the provided callback when the effect is complete with `true` for success
    /// and `false` for failure.
    #[cfg(feature = "gamepad")]
    fn play_gamepad_haptic_effect(
        &self,
        _webview: WebView,
        _: usize,
        _: GamepadHapticEffectType,
        _: Box<dyn FnOnce(bool)>,
    ) {
    }
    /// Request to stop a haptic effect on a connected gamepad. The embedder is expected to
    /// call the provided callback when the effect is complete with `true` for success
    /// and `false` for failure.
    #[cfg(feature = "gamepad")]
    fn stop_gamepad_haptic_effect(&self, _webview: WebView, _: usize, _: Box<dyn FnOnce(bool)>) {}

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

    /// A console message was logged by content in this [`WebView`].
    /// <https://developer.mozilla.org/en-US/docs/Web/API/Console_API>
    fn show_console_message(&self, _webview: WebView, _level: ConsoleLogLevel, _message: String) {}

    fn hacky_accessibility_tree_update(&self, _webview: WebView, _tree_update: TreeUpdate) {}
}

pub(crate) struct DefaultWebViewDelegate;
impl WebViewDelegate for DefaultWebViewDelegate {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_allow_deny_request() {
        use base::generic_channel;

        use crate::responders::ServoErrorChannel;

        for default_response in [AllowOrDeny::Allow, AllowOrDeny::Deny] {
            // Explicit allow yields allow and nothing else
            let errors = ServoErrorChannel::default();
            let (sender, receiver) =
                generic_channel::channel().expect("Failed to create IPC channel");
            let request = AllowOrDenyRequest::new(sender, default_response, errors.sender());
            request.allow();
            assert_eq!(receiver.try_recv().ok(), Some(AllowOrDeny::Allow));
            assert_eq!(receiver.try_recv().ok(), None);
            assert!(errors.try_recv().is_none());

            // Explicit deny yields deny and nothing else
            let errors = ServoErrorChannel::default();
            let (sender, receiver) =
                generic_channel::channel().expect("Failed to create IPC channel");
            let request = AllowOrDenyRequest::new(sender, default_response, errors.sender());
            request.deny();
            assert_eq!(receiver.try_recv().ok(), Some(AllowOrDeny::Deny));
            assert_eq!(receiver.try_recv().ok(), None);
            assert!(errors.try_recv().is_none());

            // No response yields default response and nothing else
            let errors = ServoErrorChannel::default();
            let (sender, receiver) =
                generic_channel::channel().expect("Failed to create IPC channel");
            let request = AllowOrDenyRequest::new(sender, default_response, errors.sender());
            drop(request);
            assert_eq!(receiver.try_recv().ok(), Some(default_response));
            assert_eq!(receiver.try_recv().ok(), None);
            assert!(errors.try_recv().is_none());

            // Explicit allow when receiver disconnected yields error
            let errors = ServoErrorChannel::default();
            let (sender, receiver) =
                generic_channel::channel().expect("Failed to create IPC channel");
            let request = AllowOrDenyRequest::new(sender, default_response, errors.sender());
            drop(receiver);
            request.allow();
            assert!(errors.try_recv().is_some());

            // Explicit deny when receiver disconnected yields error
            let errors = ServoErrorChannel::default();
            let (sender, receiver) =
                generic_channel::channel().expect("Failed to create IPC channel");
            let request = AllowOrDenyRequest::new(sender, default_response, errors.sender());
            drop(receiver);
            request.deny();
            assert!(errors.try_recv().is_some());

            // No response when receiver disconnected yields no error
            let errors = ServoErrorChannel::default();
            let (sender, receiver) =
                generic_channel::channel().expect("Failed to create IPC channel");
            let request = AllowOrDenyRequest::new(sender, default_response, errors.sender());
            drop(receiver);
            drop(request);
            assert!(errors.try_recv().is_none());
        }
    }

    #[test]
    fn test_authentication_request() {
        use base::generic_channel;

        use crate::responders::ServoErrorChannel;

        let url = Url::parse("https://example.com").expect("Guaranteed by argument");

        // Explicit response yields that response and nothing else
        let errors = ServoErrorChannel::default();
        let (sender, receiver) = generic_channel::channel().expect("Failed to create IPC channel");
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
        let (sender, receiver) = generic_channel::channel().expect("Failed to create IPC channel");
        let request = AuthenticationRequest::new(url.clone(), false, sender, errors.sender());
        drop(request);
        assert_eq!(receiver.try_recv().ok(), Some(None));
        assert_eq!(receiver.try_recv().ok(), None);
        assert!(errors.try_recv().is_none());

        // Explicit response when receiver disconnected yields error
        let errors = ServoErrorChannel::default();
        let (sender, receiver) = generic_channel::channel().expect("Failed to create IPC channel");
        let request = AuthenticationRequest::new(url.clone(), false, sender, errors.sender());
        drop(receiver);
        request.authenticate("diffie".to_owned(), "hunter2".to_owned());
        assert!(errors.try_recv().is_some());

        // No response when receiver disconnected yields no error
        let errors = ServoErrorChannel::default();
        let (sender, receiver) = generic_channel::channel().expect("Failed to create IPC channel");
        let request = AuthenticationRequest::new(url.clone(), false, sender, errors.sender());
        drop(receiver);
        drop(request);
        assert!(errors.try_recv().is_none());
    }

    #[test]
    fn test_web_resource_load() {
        use base::generic_channel;
        use http::{HeaderMap, Method, StatusCode};

        use crate::responders::ServoErrorChannel;

        let web_resource_request = || WebResourceRequest {
            method: Method::GET,
            headers: HeaderMap::default(),
            url: Url::parse("https://example.com").expect("Guaranteed by argument"),
            is_for_main_frame: false,
            is_redirect: false,
        };
        let web_resource_response = || {
            WebResourceResponse::new(
                Url::parse("https://diffie.test").expect("Guaranteed by argument"),
            )
            .status_code(StatusCode::IM_A_TEAPOT)
        };

        // Explicit intercept with explicit cancel yields Start and Cancel and nothing else
        let errors = ServoErrorChannel::default();
        let (sender, receiver) = generic_channel::channel().expect("Failed to create IPC channel");
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
        let (sender, receiver) = generic_channel::channel().expect("Failed to create IPC channel");
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
        let (sender, receiver) = generic_channel::channel().expect("Failed to create IPC channel");
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
        let (sender, receiver) = generic_channel::channel().expect("Failed to create IPC channel");
        let request = WebResourceLoad::new(web_resource_request(), sender, errors.sender());
        drop(receiver);
        request.intercept(web_resource_response()).cancel();
        assert!(errors.try_recv().is_some());

        // Explicit intercept with no further action when receiver disconnected yields error
        let errors = ServoErrorChannel::default();
        let (sender, receiver) = generic_channel::channel().expect("Failed to create IPC channel");
        let request = WebResourceLoad::new(web_resource_request(), sender, errors.sender());
        drop(receiver);
        drop(request.intercept(web_resource_response()));
        assert!(errors.try_recv().is_some());

        // No response when receiver disconnected yields no error
        let errors = ServoErrorChannel::default();
        let (sender, receiver) = generic_channel::channel().expect("Failed to create IPC channel");
        let request = WebResourceLoad::new(web_resource_request(), sender, errors.sender());
        drop(receiver);
        drop(request);
        assert!(errors.try_recv().is_none());
    }
}
