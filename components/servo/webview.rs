/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Ref, RefCell, RefMut};
use std::hash::Hash;
use std::rc::{Rc, Weak};
use std::time::Duration;

use base::id::WebViewId;
use compositing::IOCompositor;
use compositing_traits::WebViewTrait;
use constellation_traits::{EmbedderToConstellationMessage, TraversalDirection};
use dpi::PhysicalSize;
use embedder_traits::{
    Cursor, FocusId, InputEvent, JSValue, JavaScriptEvaluationError, LoadStatus,
    MediaSessionActionType, ScreenGeometry, Theme, TraversalId, ViewportDetails,
};
use euclid::{Point2D, Scale, Size2D};
use servo_geometry::DeviceIndependentPixel;
use url::Url;
use webrender_api::ScrollLocation;
use webrender_api::units::{DeviceIntPoint, DevicePixel, DeviceRect};

use crate::clipboard_delegate::{ClipboardDelegate, DefaultClipboardDelegate};
use crate::javascript_evaluator::JavaScriptEvaluator;
use crate::webview_delegate::{DefaultWebViewDelegate, WebViewDelegate};
use crate::{ConstellationProxy, Servo, WebRenderDebugOption};

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
    // TODO: ensure that WebView instances interact with the correct Servo instance
    pub(crate) id: WebViewId,
    pub(crate) constellation_proxy: ConstellationProxy,
    pub(crate) compositor: Rc<RefCell<IOCompositor>>,
    pub(crate) delegate: Rc<dyn WebViewDelegate>,
    pub(crate) clipboard_delegate: Rc<dyn ClipboardDelegate>,
    javascript_evaluator: Rc<RefCell<JavaScriptEvaluator>>,
    /// The rectangle of the [`WebView`] in device pixels, which is the viewport.
    rect: DeviceRect,
    hidpi_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    load_status: LoadStatus,
    url: Option<Url>,
    status_text: Option<String>,
    page_title: Option<String>,
    favicon_url: Option<Url>,
    focused: bool,
    animating: bool,
    cursor: Cursor,
}

impl Drop for WebViewInner {
    fn drop(&mut self) {
        self.constellation_proxy
            .send(EmbedderToConstellationMessage::CloseWebView(self.id));
    }
}

impl WebView {
    pub(crate) fn new(builder: WebViewBuilder) -> Self {
        let id = WebViewId::new();
        let servo = builder.servo;
        let size = builder.size.map_or_else(
            || {
                builder
                    .servo
                    .compositor
                    .borrow()
                    .rendering_context_size()
                    .to_f32()
            },
            |size| Size2D::new(size.width as f32, size.height as f32),
        );

        let webview = Self(Rc::new(RefCell::new(WebViewInner {
            id,
            constellation_proxy: servo.constellation_proxy.clone(),
            compositor: servo.compositor.clone(),
            delegate: builder.delegate,
            clipboard_delegate: Rc::new(DefaultClipboardDelegate),
            javascript_evaluator: servo.javascript_evaluator.clone(),
            rect: DeviceRect::from_origin_and_size(Point2D::origin(), size),
            hidpi_scale_factor: builder.hidpi_scale_factor,
            load_status: LoadStatus::Started,
            url: None,
            status_text: None,
            page_title: None,
            favicon_url: None,
            focused: false,
            animating: false,
            cursor: Cursor::Pointer,
        })));

        let viewport_details = webview.viewport_details();
        servo.compositor.borrow_mut().add_webview(
            Box::new(ServoRendererWebView {
                weak_handle: webview.weak_handle(),
                id,
            }),
            viewport_details,
        );

        servo
            .webviews
            .borrow_mut()
            .insert(webview.id(), webview.weak_handle());

        if !builder.auxiliary {
            let url = builder.url.unwrap_or(
                Url::parse("about:blank").expect("Should always be able to parse 'about:blank'."),
            );

            builder
                .servo
                .constellation_proxy
                .send(EmbedderToConstellationMessage::NewWebView(
                    url.into(),
                    webview.id(),
                    viewport_details,
                ));
        }

        webview
    }

    fn inner(&self) -> Ref<'_, WebViewInner> {
        self.0.borrow()
    }

    fn inner_mut(&self) -> RefMut<'_, WebViewInner> {
        self.0.borrow_mut()
    }

    pub(crate) fn viewport_details(&self) -> ViewportDetails {
        // The division by 1 represents the page's default zoom of 100%,
        // and gives us the appropriate CSSPixel type for the viewport.
        let inner = self.inner();
        let scaled_viewport_size = inner.rect.size() / inner.hidpi_scale_factor;
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
        self.inner().url.clone()
    }

    pub(crate) fn set_url(self, new_value: Url) {
        if self
            .inner()
            .url
            .as_ref()
            .is_some_and(|url| url == &new_value)
        {
            return;
        }
        self.inner_mut().url = Some(new_value.clone());
        self.delegate().notify_url_changed(self, new_value);
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

    pub fn favicon_url(&self) -> Option<Url> {
        self.inner().favicon_url.clone()
    }

    pub(crate) fn set_favicon_url(self, new_value: Url) {
        if self
            .inner()
            .favicon_url
            .as_ref()
            .is_some_and(|url| url == &new_value)
        {
            return;
        }
        self.inner_mut().favicon_url = Some(new_value.clone());
        self.delegate().notify_favicon_url_changed(self, new_value);
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

    pub(crate) fn complete_focus(self, focus_id: FocusId) {
        self.delegate().notify_focus_complete(self, focus_id);
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

    pub fn focus(&self) -> FocusId {
        let focus_id = FocusId::new();
        self.inner()
            .constellation_proxy
            .send(EmbedderToConstellationMessage::FocusWebView(
                self.id(),
                focus_id.clone(),
            ));
        focus_id
    }

    pub fn blur(&self) {
        self.inner()
            .constellation_proxy
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

    pub fn rect(&self) -> DeviceRect {
        self.inner().rect
    }

    pub fn move_resize(&self, rect: DeviceRect) {
        if self.inner().rect == rect {
            return;
        }

        self.inner_mut().rect = rect;
        self.inner()
            .compositor
            .borrow_mut()
            .move_resize_webview(self.id(), rect);
    }

    pub fn resize(&self, new_size: PhysicalSize<u32>) {
        self.inner()
            .compositor
            .borrow_mut()
            .resize_rendering_context(new_size);
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
            .compositor
            .borrow_mut()
            .set_hidpi_scale_factor(self.id(), new_scale_factor);
    }

    pub fn show(&self, hide_others: bool) {
        self.inner()
            .compositor
            .borrow_mut()
            .show_webview(self.id(), hide_others)
            .expect("BUG: invalid WebView instance");
    }

    pub fn hide(&self) {
        self.inner()
            .compositor
            .borrow_mut()
            .hide_webview(self.id())
            .expect("BUG: invalid WebView instance");
    }

    pub fn raise_to_top(&self, hide_others: bool) {
        self.inner()
            .compositor
            .borrow_mut()
            .raise_webview_to_top(self.id(), hide_others)
            .expect("BUG: invalid WebView instance");
    }

    pub fn notify_theme_change(&self, theme: Theme) {
        self.inner()
            .constellation_proxy
            .send(EmbedderToConstellationMessage::ThemeChange(
                self.id(),
                theme,
            ))
    }

    pub fn load(&self, url: Url) {
        self.inner()
            .constellation_proxy
            .send(EmbedderToConstellationMessage::LoadUrl(
                self.id(),
                url.into(),
            ))
    }

    pub fn reload(&self) {
        self.inner()
            .constellation_proxy
            .send(EmbedderToConstellationMessage::Reload(self.id()))
    }

    pub fn go_back(&self, amount: usize) -> TraversalId {
        let traversal_id = TraversalId::new();
        self.inner()
            .constellation_proxy
            .send(EmbedderToConstellationMessage::TraverseHistory(
                self.id(),
                TraversalDirection::Back(amount),
                traversal_id.clone(),
            ));
        traversal_id
    }

    pub fn go_forward(&self, amount: usize) -> TraversalId {
        let traversal_id = TraversalId::new();
        self.inner()
            .constellation_proxy
            .send(EmbedderToConstellationMessage::TraverseHistory(
                self.id(),
                TraversalDirection::Forward(amount),
                traversal_id.clone(),
            ));
        traversal_id
    }

    /// Ask the [`WebView`] to scroll web content. Note that positive scroll offsets reveal more
    /// content on the bottom and right of the page.
    pub fn notify_scroll_event(&self, location: ScrollLocation, point: DeviceIntPoint) {
        self.inner()
            .compositor
            .borrow_mut()
            .notify_scroll_event(self.id(), location, point);
    }

    pub fn notify_input_event(&self, event: InputEvent) {
        // Events with a `point` first go to the compositor for hit testing.
        if event.point().is_some() {
            self.inner()
                .compositor
                .borrow_mut()
                .notify_input_event(self.id(), event);
            return;
        }

        self.inner()
            .constellation_proxy
            .send(EmbedderToConstellationMessage::ForwardInputEvent(
                self.id(),
                event,
                None, /* hit_test */
            ))
    }

    pub fn notify_media_session_action_event(&self, event: MediaSessionActionType) {
        self.inner()
            .constellation_proxy
            .send(EmbedderToConstellationMessage::MediaSessionAction(event));
    }

    pub fn notify_vsync(&self) {
        self.inner().compositor.borrow_mut().on_vsync(self.id());
    }

    pub fn set_zoom(&self, new_zoom: f32) {
        self.inner()
            .compositor
            .borrow_mut()
            .on_zoom_window_event(self.id(), new_zoom);
    }

    pub fn reset_zoom(&self) {
        self.inner()
            .compositor
            .borrow_mut()
            .on_zoom_reset_window_event(self.id());
    }

    pub fn set_pinch_zoom(&self, new_pinch_zoom: f32) {
        self.inner()
            .compositor
            .borrow_mut()
            .set_pinch_zoom(self.id(), new_pinch_zoom);
    }

    pub fn exit_fullscreen(&self) {
        self.inner()
            .constellation_proxy
            .send(EmbedderToConstellationMessage::ExitFullScreen(self.id()));
    }

    pub fn set_throttled(&self, throttled: bool) {
        self.inner()
            .constellation_proxy
            .send(EmbedderToConstellationMessage::SetWebViewThrottled(
                self.id(),
                throttled,
            ));
    }

    pub fn toggle_webrender_debugging(&self, debugging: WebRenderDebugOption) {
        self.inner()
            .compositor
            .borrow_mut()
            .toggle_webrender_debug(debugging);
    }

    pub fn capture_webrender(&self) {
        self.inner().compositor.borrow_mut().capture_webrender();
    }

    pub fn toggle_sampling_profiler(&self, rate: Duration, max_duration: Duration) {
        self.inner()
            .constellation_proxy
            .send(EmbedderToConstellationMessage::ToggleProfiler(
                rate,
                max_duration,
            ));
    }

    pub fn send_error(&self, message: String) {
        self.inner()
            .constellation_proxy
            .send(EmbedderToConstellationMessage::SendError(
                Some(self.id()),
                message,
            ));
    }

    /// Paint the contents of this [`WebView`] into its `RenderingContext`. This will
    /// always paint, unless the `Opts::wait_for_stable_image` option is enabled. In
    /// that case, this might do nothing. Returns true if a paint was actually performed.
    pub fn paint(&self) -> bool {
        self.inner().compositor.borrow_mut().render()
    }

    /// Evaluate the specified string of JavaScript code. Once execution is complete or an error
    /// occurs, Servo will call `callback`.
    pub fn evaluate_javascript<T: ToString>(
        &self,
        script: T,
        callback: impl FnOnce(Result<JSValue, JavaScriptEvaluationError>) + 'static,
    ) {
        self.inner().javascript_evaluator.borrow_mut().evaluate(
            self.id(),
            script.to_string(),
            Box::new(callback),
        );
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

pub struct WebViewBuilder<'servo> {
    servo: &'servo Servo,
    delegate: Rc<dyn WebViewDelegate>,
    auxiliary: bool,
    url: Option<Url>,
    size: Option<PhysicalSize<u32>>,
    hidpi_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
}

impl<'servo> WebViewBuilder<'servo> {
    pub fn new(servo: &'servo Servo) -> Self {
        Self {
            servo,
            auxiliary: false,
            url: None,
            size: None,
            hidpi_scale_factor: Scale::new(1.0),
            delegate: Rc::new(DefaultWebViewDelegate),
        }
    }

    pub fn new_auxiliary(servo: &'servo Servo) -> Self {
        let mut builder = Self::new(servo);
        builder.auxiliary = true;
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

    pub fn size(mut self, size: PhysicalSize<u32>) -> Self {
        self.size = Some(size);
        self
    }

    pub fn hidpi_scale_factor(
        mut self,
        hidpi_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    ) -> Self {
        self.hidpi_scale_factor = hidpi_scale_factor;
        self
    }

    pub fn build(self) -> WebView {
        WebView::new(self)
    }
}
