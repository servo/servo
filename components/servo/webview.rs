/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Ref, RefCell, RefMut};
use std::hash::Hash;
use std::rc::{Rc, Weak};
use std::time::Duration;

use base::generic_channel::GenericSender;
use base::id::WebViewId;
use constellation_traits::{EmbedderToConstellationMessage, TraversalDirection};
use dpi::PhysicalSize;
use embedder_traits::{
    ContextMenuAction, ContextMenuItem, Cursor, EmbedderControlId, EmbedderControlRequest, Image,
    InputEvent, InputEventAndId, InputEventId, JSValue, JavaScriptEvaluationError, LoadStatus,
    MediaSessionActionType, NewWebViewDetails, ScreenGeometry, ScreenshotCaptureError, Scroll,
    Theme, TraversalId, ViewportDetails, WebViewPoint, WebViewRect,
};
use euclid::{Scale, Size2D};
use image::RgbaImage;
use paint_api::WebViewTrait;
use paint_api::rendering_context::RenderingContext;
use servo_geometry::DeviceIndependentPixel;
use servo_url::ServoUrl;
use style_traits::CSSPixel;
use url::Url;
use webrender_api::units::{DeviceIntRect, DevicePixel, DevicePoint, DeviceSize};

use crate::clipboard_delegate::{ClipboardDelegate, DefaultClipboardDelegate};
use crate::credential_management_delegate::{
    CredentialManagementDelegate, DefaultCredentialManagementDelegate,
};
use crate::responders::IpcResponder;
use crate::webview_delegate::{CreateNewWebViewRequest, DefaultWebViewDelegate, WebViewDelegate};
use crate::{
    ColorPicker, ContextMenu, EmbedderControl, InputMethodControl, SelectElement, Servo,
    UserContentManager, WebRenderDebugOption,
};

pub(crate) const MINIMUM_WEBVIEW_SIZE: Size2D<i32, DevicePixel> = Size2D::new(1, 1);

/// A handle to a Servo webview. If you clone this handle, it does not create a new webview,
/// but instead creates a new handle to the webview. Once the last handle is dropped, Servo
/// considers that the webview has closed and will clean up all associated resources related
/// to this webview.
///
/// ## Rendering Model
///
/// Every [`WebView`] has a [`RenderingContext`](crate::RenderingContext). The embedder manages when
/// the contents of the [`WebView`] paint to the [`RenderingContext`](crate::RenderingContext). When
/// a [`WebView`] needs to be painted, for instance, because its contents have changed, Servo will
/// call [`WebViewDelegate::notify_new_frame_ready`] in order to signal that it is time to repaint
/// the [`WebView`] using [`WebView::paint`].
///
/// An example of how this flow might work is:
///
/// 1. [`WebViewDelegate::notify_new_frame_ready`] is called. The applications triggers a request
///    to repaint the window that contains this [`WebView`].
/// 2. During window repainting, the application calls [`WebView::paint`] and the contents of the
///    [`RenderingContext`][crate::RenderingContext] are updated.
/// 3. If the [`RenderingContext`][crate::RenderingContext] is double-buffered, the
///    application then calls [`crate::RenderingContext::present()`] in order to swap the back buffer
///    to the front, finally displaying the updated [`WebView`] contents.
///
/// In cases where the [`WebView`] contents have not been updated, but a repaint is necessary, for
/// instance when repainting a window due to damage, an application may simply perform the final two
/// steps and Servo will repaint even without first calling the
/// [`WebViewDelegate::notify_new_frame_ready`] method.
#[derive(Clone)]
pub struct WebView(Rc<RefCell<WebViewInner>>);

impl PartialEq for WebView {
    fn eq(&self, other: &Self) -> bool {
        self.inner().id == other.inner().id
    }
}

impl Hash for WebView {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner().id.hash(state);
    }
}

pub(crate) struct WebViewInner {
    pub(crate) id: WebViewId,
    pub(crate) servo: Servo,
    pub(crate) delegate: Rc<dyn WebViewDelegate>,
    pub(crate) clipboard_delegate: Rc<dyn ClipboardDelegate>,
    pub(crate) credential_management_delegate: Rc<dyn CredentialManagementDelegate>,

    rendering_context: Rc<dyn RenderingContext>,
    user_content_manager: Option<Rc<UserContentManager>>,
    hidpi_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    load_status: LoadStatus,
    status_text: Option<String>,
    page_title: Option<String>,
    favicon: Option<Image>,
    focused: bool,
    animating: bool,
    cursor: Cursor,

    /// The back / forward list of this WebView.
    back_forward_list: Vec<Url>,

    /// The current index in the back / forward list.
    back_forward_list_index: usize,
}

impl Drop for WebViewInner {
    fn drop(&mut self) {
        self.servo
            .constellation_proxy()
            .send(EmbedderToConstellationMessage::CloseWebView(self.id));
        self.servo.paint_mut().remove_webview(self.id);
    }
}

impl WebView {
    pub(crate) fn new(mut builder: WebViewBuilder) -> Self {
        let servo = builder.servo;
        let painter_id = servo
            .paint_mut()
            .register_rendering_context(builder.rendering_context.clone());

        let id = WebViewId::new(painter_id);
        let webview = Self(Rc::new(RefCell::new(WebViewInner {
            id,
            servo: servo.clone(),
            rendering_context: builder.rendering_context,
            delegate: builder.delegate,
            clipboard_delegate: Rc::new(DefaultClipboardDelegate),
            credential_management_delegate: Rc::new(DefaultCredentialManagementDelegate::default()),
            hidpi_scale_factor: builder.hidpi_scale_factor,
            load_status: LoadStatus::Started,
            status_text: None,
            page_title: None,
            favicon: None,
            focused: false,
            animating: false,
            cursor: Cursor::Pointer,
            back_forward_list: Default::default(),
            back_forward_list_index: 0,
            user_content_manager: builder.user_content_manager.clone(),
        })));

        let viewport_details = webview.viewport_details();
        servo.paint().add_webview(
            Box::new(ServoRendererWebView {
                weak_handle: webview.weak_handle(),
                id,
            }),
            viewport_details,
        );

        servo
            .webviews_mut()
            .insert(webview.id(), webview.weak_handle());

        let user_content_manager_id = builder
            .user_content_manager
            .as_ref()
            .map(|user_content_manager| user_content_manager.id());

        let new_webview_details = NewWebViewDetails {
            webview_id: webview.id(),
            viewport_details,
            user_content_manager_id,
        };

        // There are two possibilities here. Either the WebView is a new toplevel
        // WebView in which case `Self::create_new_webview_responder` is `None` or this
        // is the response to a `WebViewDelegate::request_create_new` method in which
        // case script expects that we just return the information directly back to
        // the `ScriptThread`.
        match builder.create_new_webview_responder.as_mut() {
            Some(responder) => {
                let _ = responder.send(Some(new_webview_details));
            },
            None => {
                let url = builder.url.unwrap_or(
                    Url::parse("about:blank")
                        .expect("Should always be able to parse 'about:blank'."),
                );

                servo
                    .constellation_proxy()
                    .send(EmbedderToConstellationMessage::NewWebView(
                        url.into(),
                        new_webview_details,
                    ));
            },
        }

        webview
    }

    fn inner(&self) -> Ref<'_, WebViewInner> {
        self.0.borrow()
    }

    fn inner_mut(&self) -> RefMut<'_, WebViewInner> {
        self.0.borrow_mut()
    }

    pub(crate) fn request_create_new(
        &self,
        response_sender: GenericSender<Option<NewWebViewDetails>>,
    ) {
        let request = CreateNewWebViewRequest {
            servo: self.inner().servo.clone(),
            responder: IpcResponder::new(response_sender, None),
        };
        self.delegate().request_create_new(self.clone(), request);
    }

    pub(crate) fn viewport_details(&self) -> ViewportDetails {
        // The division by 1 represents the page's default zoom of 100%,
        // and gives us the appropriate CSSPixel type for the viewport.
        let inner = self.inner();
        let scaled_viewport_size =
            inner.rendering_context.size2d().to_f32() / inner.hidpi_scale_factor;
        ViewportDetails {
            size: scaled_viewport_size / Scale::new(1.0),
            hidpi_scale_factor: Scale::new(inner.hidpi_scale_factor.0),
        }
    }

    pub(crate) fn from_weak_handle(inner: &Weak<RefCell<WebViewInner>>) -> Option<Self> {
        inner.upgrade().map(WebView)
    }

    pub(crate) fn weak_handle(&self) -> Weak<RefCell<WebViewInner>> {
        Rc::downgrade(&self.0)
    }

    pub fn delegate(&self) -> Rc<dyn WebViewDelegate> {
        self.inner().delegate.clone()
    }

    pub fn set_delegate(&self, delegate: Rc<dyn WebViewDelegate>) {
        self.inner_mut().delegate = delegate;
    }

    pub fn clipboard_delegate(&self) -> Rc<dyn ClipboardDelegate> {
        self.inner().clipboard_delegate.clone()
    }

    pub fn set_clipboard_delegate(&self, delegate: Rc<dyn ClipboardDelegate>) {
        self.inner_mut().clipboard_delegate = delegate;
    }

    pub fn credential_management_delegate(&self) -> Rc<dyn CredentialManagementDelegate> {
        self.inner().credential_management_delegate.clone()
    }

    pub fn set_credential_management_delegate(
        &self,
        delegate: Rc<dyn CredentialManagementDelegate>,
    ) {
        self.inner_mut().credential_management_delegate = delegate;
    }

    pub fn id(&self) -> WebViewId {
        self.inner().id
    }

    pub fn load_status(&self) -> LoadStatus {
        self.inner().load_status
    }

    pub(crate) fn set_load_status(self, new_value: LoadStatus) {
        if self.inner().load_status == new_value {
            return;
        }
        self.inner_mut().load_status = new_value;
        self.delegate().notify_load_status_changed(self, new_value);
    }

    pub fn url(&self) -> Option<Url> {
        let inner = self.inner();
        inner
            .back_forward_list
            .get(inner.back_forward_list_index)
            .cloned()
    }

    pub fn status_text(&self) -> Option<String> {
        self.inner().status_text.clone()
    }

    pub(crate) fn set_status_text(self, new_value: Option<String>) {
        if self.inner().status_text == new_value {
            return;
        }
        self.inner_mut().status_text = new_value.clone();
        self.delegate().notify_status_text_changed(self, new_value);
    }

    pub fn page_title(&self) -> Option<String> {
        self.inner().page_title.clone()
    }

    pub(crate) fn set_page_title(self, new_value: Option<String>) {
        if self.inner().page_title == new_value {
            return;
        }
        self.inner_mut().page_title = new_value.clone();
        self.delegate().notify_page_title_changed(self, new_value);
    }

    pub fn favicon(&self) -> Option<Ref<'_, Image>> {
        Ref::filter_map(self.inner(), |inner| inner.favicon.as_ref()).ok()
    }

    pub(crate) fn set_favicon(self, new_value: Image) {
        self.inner_mut().favicon = Some(new_value);
        self.delegate().notify_favicon_changed(self);
    }

    pub fn focused(&self) -> bool {
        self.inner().focused
    }

    pub(crate) fn set_focused(self, new_value: bool) {
        if self.inner().focused == new_value {
            return;
        }
        self.inner_mut().focused = new_value;
        self.delegate().notify_focus_changed(self, new_value);
    }

    pub fn cursor(&self) -> Cursor {
        self.inner().cursor
    }

    pub(crate) fn set_cursor(self, new_value: Cursor) {
        if self.inner().cursor == new_value {
            return;
        }
        self.inner_mut().cursor = new_value;
        self.delegate().notify_cursor_changed(self, new_value);
    }

    pub fn focus(&self) {
        self.inner()
            .servo
            .constellation_proxy()
            .send(EmbedderToConstellationMessage::FocusWebView(self.id()));
    }

    pub fn blur(&self) {
        self.inner()
            .servo
            .constellation_proxy()
            .send(EmbedderToConstellationMessage::BlurWebView);
    }

    /// Whether or not this [`WebView`] has animating content, such as a CSS animation or
    /// transition or is running `requestAnimationFrame` callbacks. This indicates that the
    /// embedding application should be spinning the Servo event loop on regular intervals
    /// in order to trigger animation updates.
    pub fn animating(self) -> bool {
        self.inner().animating
    }

    pub(crate) fn set_animating(self, new_value: bool) {
        if self.inner().animating == new_value {
            return;
        }
        self.inner_mut().animating = new_value;
        self.delegate().notify_animating_changed(self, new_value);
    }

    /// The size of this [`WebView`]'s [`RenderingContext`].
    pub fn size(&self) -> DeviceSize {
        self.inner().rendering_context.size2d().to_f32()
    }

    /// Request that the given [`WebView`]'s [`RenderingContext`] be resized. Note that the
    /// minimum size for a WebView is 1 pixel by 1 pixel so any requested size will be
    /// clamped by that value.
    ///
    /// This will also resize any other [`WebView`] using the same [`RenderingContext`]. A
    /// [`WebView`] is always as big as its [`RenderingContext`].
    pub fn resize(&self, new_size: PhysicalSize<u32>) {
        let new_size = PhysicalSize {
            width: new_size.width.max(MINIMUM_WEBVIEW_SIZE.width as u32),
            height: new_size.height.max(MINIMUM_WEBVIEW_SIZE.height as u32),
        };

        self.inner()
            .servo
            .paint()
            .resize_rendering_context(self.id(), new_size);
    }

    pub fn hidpi_scale_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
        self.inner().hidpi_scale_factor
    }

    pub fn set_hidpi_scale_factor(
        &self,
        new_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    ) {
        if self.inner().hidpi_scale_factor == new_scale_factor {
            return;
        }

        self.inner_mut().hidpi_scale_factor = new_scale_factor;
        self.inner()
            .servo
            .paint()
            .set_hidpi_scale_factor(self.id(), new_scale_factor);
    }

    pub fn show(&self) {
        self.inner()
            .servo
            .paint()
            .show_webview(self.id())
            .expect("BUG: invalid WebView instance");
    }

    pub fn hide(&self) {
        self.inner()
            .servo
            .paint()
            .hide_webview(self.id())
            .expect("BUG: invalid WebView instance");
    }

    pub fn notify_theme_change(&self, theme: Theme) {
        self.inner()
            .servo
            .constellation_proxy()
            .send(EmbedderToConstellationMessage::ThemeChange(
                self.id(),
                theme,
            ))
    }

    pub fn load(&self, url: Url) {
        self.inner()
            .servo
            .constellation_proxy()
            .send(EmbedderToConstellationMessage::LoadUrl(
                self.id(),
                url.into(),
            ))
    }

    pub fn reload(&self) {
        self.inner_mut().load_status = LoadStatus::Started;
        self.inner()
            .servo
            .constellation_proxy()
            .send(EmbedderToConstellationMessage::Reload(self.id()))
    }

    pub fn can_go_back(&self) -> bool {
        self.inner().back_forward_list_index != 0
    }

    pub fn go_back(&self, amount: usize) -> TraversalId {
        let traversal_id = TraversalId::new();
        self.inner().servo.constellation_proxy().send(
            EmbedderToConstellationMessage::TraverseHistory(
                self.id(),
                TraversalDirection::Back(amount),
                traversal_id.clone(),
            ),
        );
        traversal_id
    }

    pub fn can_go_forward(&self) -> bool {
        let inner = self.inner();
        inner.back_forward_list.len() > inner.back_forward_list_index + 1
    }

    pub fn go_forward(&self, amount: usize) -> TraversalId {
        let traversal_id = TraversalId::new();
        self.inner().servo.constellation_proxy().send(
            EmbedderToConstellationMessage::TraverseHistory(
                self.id(),
                TraversalDirection::Forward(amount),
                traversal_id.clone(),
            ),
        );
        traversal_id
    }

    /// Ask the [`WebView`] to scroll web content. Note that positive scroll offsets reveal more
    /// content on the bottom and right of the page.
    pub fn notify_scroll_event(&self, scroll: Scroll, point: WebViewPoint) {
        self.inner()
            .servo
            .paint()
            .notify_scroll_event(self.id(), scroll, point);
    }

    pub fn notify_input_event(&self, event: InputEvent) -> InputEventId {
        let event: InputEventAndId = event.into();
        let event_id = event.id;

        // Events with a `point` first go to `Paint` for hit testing.
        if event.event.point().is_some() {
            self.inner()
                .servo
                .paint()
                .notify_input_event(self.id(), event);
        } else {
            self.inner().servo.constellation_proxy().send(
                EmbedderToConstellationMessage::ForwardInputEvent(
                    self.id(),
                    event,
                    None, /* hit_test */
                ),
            );
        }

        event_id
    }

    pub fn notify_media_session_action_event(&self, event: MediaSessionActionType) {
        self.inner()
            .servo
            .constellation_proxy()
            .send(EmbedderToConstellationMessage::MediaSessionAction(event));
    }

    /// Set the page zoom of the [`WebView`]. This sets the final page zoom value of the
    /// [`WebView`]. Unlike [`WebView::pinch_zoom`] *it is not* multiplied by the current
    /// page zoom value, but overrides it.
    ///
    /// [`WebView`]s have two types of zoom, pinch zoom and page zoom. This adjusts page
    /// zoom, which will adjust the `devicePixelRatio` of the page and cause it to modify
    /// its layout.
    ///
    /// These values will be clamped internally. The values used for clamping can be
    /// adjusted by page content when `<meta viewport>` parsing is enabled via
    /// `Prefs::viewport_meta_enabled`.
    pub fn set_page_zoom(&self, new_zoom: f32) {
        self.inner()
            .servo
            .paint()
            .set_page_zoom(self.id(), new_zoom);
    }

    /// Get the page zoom of the [`WebView`].
    pub fn page_zoom(&self) -> f32 {
        self.inner().servo.paint().page_zoom(self.id())
    }

    /// Adjust the pinch zoom on this [`WebView`] multiplying the current pinch zoom
    /// level with the provided `pinch_zoom_delta`.
    ///
    /// [`WebView`]s have two types of zoom, pinch zoom and page zoom. This adjusts pinch
    /// zoom, which is a type of zoom which does not modify layout, and instead simply
    /// magnifies the view in the viewport.
    ///
    /// The final pinch zoom values will be clamped to reasonable defaults (currently to
    /// the inclusive range [1.0, 10.0]).
    pub fn pinch_zoom(&self, pinch_zoom_delta: f32, center: DevicePoint) {
        self.inner()
            .servo
            .paint()
            .pinch_zoom(self.id(), pinch_zoom_delta, center);
    }

    pub fn device_pixels_per_css_pixel(&self) -> Scale<f32, CSSPixel, DevicePixel> {
        self.inner()
            .servo
            .paint()
            .device_pixels_per_page_pixel(self.id())
    }

    pub fn exit_fullscreen(&self) {
        self.inner()
            .servo
            .constellation_proxy()
            .send(EmbedderToConstellationMessage::ExitFullScreen(self.id()));
    }

    pub fn set_throttled(&self, throttled: bool) {
        self.inner().servo.constellation_proxy().send(
            EmbedderToConstellationMessage::SetWebViewThrottled(self.id(), throttled),
        );
    }

    pub fn toggle_webrender_debugging(&self, debugging: WebRenderDebugOption) {
        self.inner().servo.paint().toggle_webrender_debug(debugging);
    }

    pub fn capture_webrender(&self) {
        self.inner().servo.paint().capture_webrender(self.id());
    }

    pub fn toggle_sampling_profiler(&self, rate: Duration, max_duration: Duration) {
        self.inner().servo.constellation_proxy().send(
            EmbedderToConstellationMessage::ToggleProfiler(rate, max_duration),
        );
    }

    pub fn send_error(&self, message: String) {
        self.inner()
            .servo
            .constellation_proxy()
            .send(EmbedderToConstellationMessage::SendError(
                Some(self.id()),
                message,
            ));
    }

    /// Paint the contents of this [`WebView`] into its `RenderingContext`.
    pub fn paint(&self) {
        self.inner().servo.paint().render(self.id());
    }

    /// Get the [`UserContentManager`] associated with this [`WebView`].
    pub fn user_content_manager(&self) -> Option<Rc<UserContentManager>> {
        self.inner().user_content_manager.clone()
    }

    /// Evaluate the specified string of JavaScript code. Once execution is complete or an error
    /// occurs, Servo will call `callback`.
    pub fn evaluate_javascript<T: ToString>(
        &self,
        script: T,
        callback: impl FnOnce(Result<JSValue, JavaScriptEvaluationError>) + 'static,
    ) {
        self.inner().servo.javascript_evaluator_mut().evaluate(
            self.id(),
            script.to_string(),
            Box::new(callback),
        );
    }

    /// Asynchronously take a screenshot of the [`WebView`] contents, given a `rect` or the whole
    /// viewport, if no `rect` is given.
    ///
    /// This method will wait until the [`WebView`] is ready before the screenshot is taken.
    /// This includes waiting for:
    ///
    ///  - all frames to fire their `load` event.
    ///  - all render blocking elements, such as stylesheets included via the `<link>`
    ///    element, to stop blocking the rendering.
    ///  - all images to be loaded and displayed.
    ///  - all web fonts are loaded.
    ///  - the `reftest-wait` and `test-wait` classes have been removed from the root element.
    ///  - the rendering is up-to-date
    ///
    /// Once all these conditions are met and the rendering does not have any pending frames
    /// to render, the provided `callback` will be called with the results of the screenshot
    /// operation.
    pub fn take_screenshot(
        &self,
        rect: Option<WebViewRect>,
        callback: impl FnOnce(Result<RgbaImage, ScreenshotCaptureError>) + 'static,
    ) {
        self.inner()
            .servo
            .paint()
            .request_screenshot(self.id(), rect, Box::new(callback));
    }

    pub(crate) fn set_history(self, new_back_forward_list: Vec<ServoUrl>, new_index: usize) {
        {
            let mut inner_mut = self.inner_mut();
            inner_mut.back_forward_list_index = new_index;
            inner_mut.back_forward_list = new_back_forward_list
                .into_iter()
                .map(ServoUrl::into_url)
                .collect();
        }

        let back_forward_list = self.inner().back_forward_list.clone();
        let back_forward_list_index = self.inner().back_forward_list_index;
        self.delegate().notify_url_changed(
            self.clone(),
            back_forward_list[back_forward_list_index].clone(),
        );
        self.delegate().notify_history_changed(
            self.clone(),
            back_forward_list,
            back_forward_list_index,
        );
    }

    pub(crate) fn show_embedder_control(
        self,
        control_id: EmbedderControlId,
        position: DeviceIntRect,
        embedder_control_request: EmbedderControlRequest,
    ) {
        let constellation_proxy = self.inner().servo.constellation_proxy().clone();
        let embedder_control = match embedder_control_request {
            EmbedderControlRequest::SelectElement(options, selected_option) => {
                EmbedderControl::SelectElement(SelectElement {
                    id: control_id,
                    options,
                    selected_option,
                    position,
                    constellation_proxy,
                    response_sent: false,
                })
            },
            EmbedderControlRequest::ColorPicker(current_color) => {
                EmbedderControl::ColorPicker(ColorPicker {
                    id: control_id,
                    current_color: Some(current_color),
                    position,
                    constellation_proxy,
                    response_sent: false,
                })
            },
            EmbedderControlRequest::InputMethod(input_method_request) => {
                EmbedderControl::InputMethod(InputMethodControl {
                    id: control_id,
                    input_method_type: input_method_request.input_method_type,
                    text: input_method_request.text,
                    insertion_point: input_method_request.insertion_point,
                    position,
                    multiline: input_method_request.multiline,
                })
            },
            EmbedderControlRequest::ContextMenu(mut context_menu_request) => {
                for item in context_menu_request.items.iter_mut() {
                    match item {
                        ContextMenuItem::Item {
                            action: ContextMenuAction::GoBack,
                            enabled,
                            ..
                        } => *enabled = self.can_go_back(),
                        ContextMenuItem::Item {
                            action: ContextMenuAction::GoForward,
                            enabled,
                            ..
                        } => *enabled = self.can_go_forward(),
                        _ => {},
                    }
                }
                EmbedderControl::ContextMenu(ContextMenu {
                    id: control_id,
                    position,
                    items: context_menu_request.items,
                    element_info: context_menu_request.element_info,
                    constellation_proxy,
                    response_sent: false,
                })
            },
            EmbedderControlRequest::FilePicker { .. } => {
                unreachable!("This message should be routed through the FileManagerThread")
            },
        };

        self.delegate()
            .show_embedder_control(self.clone(), embedder_control);
    }
}

/// A structure used to expose a view of the [`WebView`] to the Servo
/// renderer, without having the Servo renderer depend on the embedding layer.
struct ServoRendererWebView {
    id: WebViewId,
    weak_handle: Weak<RefCell<WebViewInner>>,
}

impl WebViewTrait for ServoRendererWebView {
    fn id(&self) -> WebViewId {
        self.id
    }

    fn screen_geometry(&self) -> Option<ScreenGeometry> {
        let webview = WebView::from_weak_handle(&self.weak_handle)?;
        webview.delegate().screen_geometry(webview)
    }

    fn set_animating(&self, new_value: bool) {
        if let Some(webview) = WebView::from_weak_handle(&self.weak_handle) {
            webview.set_animating(new_value);
        }
    }
}

pub struct WebViewBuilder {
    servo: Servo,
    rendering_context: Rc<dyn RenderingContext>,
    delegate: Rc<dyn WebViewDelegate>,
    url: Option<Url>,
    hidpi_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    create_new_webview_responder: Option<IpcResponder<Option<NewWebViewDetails>>>,
    user_content_manager: Option<Rc<UserContentManager>>,
}

impl WebViewBuilder {
    pub fn new(servo: &Servo, rendering_context: Rc<dyn RenderingContext>) -> Self {
        Self {
            servo: servo.clone(),
            rendering_context,
            url: None,
            hidpi_scale_factor: Scale::new(1.0),
            delegate: Rc::new(DefaultWebViewDelegate),
            create_new_webview_responder: None,
            user_content_manager: None,
        }
    }

    pub(crate) fn new_for_create_request(
        servo: &Servo,
        rendering_context: Rc<dyn RenderingContext>,
        responder: IpcResponder<Option<NewWebViewDetails>>,
    ) -> Self {
        let mut builder = Self::new(servo, rendering_context);
        builder.create_new_webview_responder = Some(responder);
        builder
    }

    pub fn delegate(mut self, delegate: Rc<dyn WebViewDelegate>) -> Self {
        self.delegate = delegate;
        self
    }

    pub fn url(mut self, url: Url) -> Self {
        self.url = Some(url);
        self
    }

    pub fn hidpi_scale_factor(
        mut self,
        hidpi_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    ) -> Self {
        self.hidpi_scale_factor = hidpi_scale_factor;
        self
    }

    /// Set the [`UserContentManager`] for the `WebView` being created. The same
    /// `UserContentManager` can be shared among multiple `WebView`s. Any updates
    /// to the `UserContentManager` will take effect only after the document is reloaded>
    pub fn user_content_manager(mut self, user_content_manager: Rc<UserContentManager>) -> Self {
        self.user_content_manager = Some(user_content_manager);
        self
    }

    pub fn build(self) -> WebView {
        WebView::new(self)
    }
}
