/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Ref, RefCell, RefMut};
use std::hash::Hash;
use std::rc::{Rc, Weak};
use std::time::Duration;

use base::id::WebViewId;
use compositing::windowing::WebRenderDebugOption;
use compositing::IOCompositor;
use compositing_traits::ConstellationMsg;
use embedder_traits::{
    Cursor, InputEvent, LoadStatus, MediaSessionActionType, Theme, TouchEventType,
    TraversalDirection,
};
use url::Url;
use webrender_api::units::{DeviceIntPoint, DeviceRect};
use webrender_api::ScrollLocation;

use crate::clipboard_delegate::{ClipboardDelegate, DefaultClipboardDelegate};
use crate::webview_delegate::{DefaultWebViewDelegate, WebViewDelegate};
use crate::ConstellationProxy;

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

    rect: DeviceRect,
    load_status: LoadStatus,
    url: Option<Url>,
    status_text: Option<String>,
    page_title: Option<String>,
    favicon_url: Option<Url>,
    focused: bool,
    cursor: Cursor,
}

impl Drop for WebViewInner {
    fn drop(&mut self) {
        self.constellation_proxy
            .send(ConstellationMsg::CloseWebView(self.id));
    }
}

impl WebView {
    pub(crate) fn new(
        constellation_proxy: &ConstellationProxy,
        compositor: Rc<RefCell<IOCompositor>>,
    ) -> Self {
        Self(Rc::new(RefCell::new(WebViewInner {
            id: WebViewId::new(),
            constellation_proxy: constellation_proxy.clone(),
            compositor,
            delegate: Rc::new(DefaultWebViewDelegate),
            clipboard_delegate: Rc::new(DefaultClipboardDelegate),
            rect: DeviceRect::zero(),
            load_status: LoadStatus::Complete,
            url: None,
            status_text: None,
            page_title: None,
            favicon_url: None,
            focused: false,
            cursor: Cursor::Pointer,
        })))
    }

    fn inner(&self) -> Ref<'_, WebViewInner> {
        self.0.borrow()
    }

    fn inner_mut(&self) -> RefMut<'_, WebViewInner> {
        self.0.borrow_mut()
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
            .constellation_proxy
            .send(ConstellationMsg::FocusWebView(self.id()));
    }

    pub fn blur(&self) {
        self.inner()
            .constellation_proxy
            .send(ConstellationMsg::BlurWebView);
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
            .send(ConstellationMsg::ThemeChange(theme))
    }

    pub fn load(&self, url: Url) {
        self.inner()
            .constellation_proxy
            .send(ConstellationMsg::LoadUrl(self.id(), url.into()))
    }

    pub fn reload(&self) {
        self.inner()
            .constellation_proxy
            .send(ConstellationMsg::Reload(self.id()))
    }

    pub fn go_back(&self, amount: usize) {
        self.inner()
            .constellation_proxy
            .send(ConstellationMsg::TraverseHistory(
                self.id(),
                TraversalDirection::Back(amount),
            ))
    }

    pub fn go_forward(&self, amount: usize) {
        self.inner()
            .constellation_proxy
            .send(ConstellationMsg::TraverseHistory(
                self.id(),
                TraversalDirection::Forward(amount),
            ))
    }

    pub fn notify_scroll_event(
        &self,
        location: ScrollLocation,
        point: DeviceIntPoint,
        touch_event_action: TouchEventType,
    ) {
        self.inner()
            .compositor
            .borrow_mut()
            .on_scroll_event(location, point, touch_event_action);
    }

    pub fn notify_input_event(&self, event: InputEvent) {
        // Events with a `point` first go to the compositor for hit testing.
        if event.point().is_some() {
            self.inner().compositor.borrow_mut().on_input_event(event);
            return;
        }

        self.inner()
            .constellation_proxy
            .send(ConstellationMsg::ForwardInputEvent(
                event, None, /* hit_test */
            ))
    }

    pub fn notify_media_session_action_event(&self, event: MediaSessionActionType) {
        self.inner()
            .constellation_proxy
            .send(ConstellationMsg::MediaSessionAction(event));
    }

    pub fn notify_vsync(&self) {
        self.inner().compositor.borrow_mut().on_vsync();
    }

    pub fn notify_rendering_context_resized(&self) {
        self.inner()
            .compositor
            .borrow_mut()
            .on_rendering_context_resized();
    }

    pub fn notify_embedder_window_moved(&self) {
        self.inner()
            .compositor
            .borrow_mut()
            .on_embedder_window_moved();
    }

    pub fn set_zoom(&self, new_zoom: f32) {
        self.inner()
            .compositor
            .borrow_mut()
            .on_zoom_window_event(new_zoom);
    }

    pub fn reset_zoom(&self) {
        self.inner()
            .compositor
            .borrow_mut()
            .on_zoom_reset_window_event();
    }

    pub fn set_pinch_zoom(&self, new_pinch_zoom: f32) {
        self.inner()
            .compositor
            .borrow_mut()
            .on_pinch_zoom_window_event(new_pinch_zoom);
    }

    pub fn exit_fullscreen(&self) {
        self.inner()
            .constellation_proxy
            .send(ConstellationMsg::ExitFullScreen(self.id()));
    }

    pub fn set_throttled(&self, throttled: bool) {
        self.inner()
            .constellation_proxy
            .send(ConstellationMsg::SetWebViewThrottled(self.id(), throttled));
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
            .send(ConstellationMsg::ToggleProfiler(rate, max_duration));
    }

    pub fn send_error(&self, message: String) {
        self.inner()
            .constellation_proxy
            .send(ConstellationMsg::SendError(Some(self.id()), message));
    }

    pub fn paint(&self) {
        self.inner().compositor.borrow_mut().composite();
    }
}
