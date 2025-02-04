/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;
use std::time::Duration;

use base::id::WebViewId;
use compositing::windowing::{MouseWindowEvent, WebRenderDebugOption};
use compositing::IOCompositor;
use compositing_traits::ConstellationMsg;
use embedder_traits::{
    ClipboardEventType, GamepadEvent, MediaSessionActionType, Theme, TouchEventType, TouchId,
    TraversalDirection, WheelDelta,
};
use keyboard_types::{CompositionEvent, KeyboardEvent};
use url::Url;
use webrender_api::units::{DeviceIntPoint, DeviceIntSize, DevicePoint, DeviceRect};
use webrender_api::ScrollLocation;

use crate::ConstellationProxy;

#[derive(Clone)]
pub struct WebView(Rc<WebViewInner>);

struct WebViewInner {
    // TODO: ensure that WebView instances interact with the correct Servo instance
    pub(crate) id: WebViewId,
    pub(crate) constellation_proxy: ConstellationProxy,
    pub(crate) compositor: Rc<RefCell<IOCompositor>>,
}

impl Drop for WebViewInner {
    fn drop(&mut self) {
        self.constellation_proxy
            .send(ConstellationMsg::CloseWebView(self.id));
    }
}

/// Handle for a webview.
///
/// - The webview exists for exactly as long as there are WebView handles
///   (FIXME: this is not true yet; webviews can still close of their own volition)
/// - All methods are infallible; if the constellation dies, the embedder finds out when calling
///   [Servo::handle_events](crate::Servo::handle_events)
impl WebView {
    pub(crate) fn new(
        constellation_proxy: &ConstellationProxy,
        compositor: Rc<RefCell<IOCompositor>>,
        url: Url,
    ) -> Self {
        let webview_id = WebViewId::new();
        constellation_proxy.send(ConstellationMsg::NewWebView(url.into(), webview_id));

        Self(Rc::new(WebViewInner {
            id: webview_id,
            constellation_proxy: constellation_proxy.clone(),
            compositor,
        }))
    }

    /// FIXME: Remove this once we have a webview delegate.
    pub(crate) fn new_auxiliary(
        constellation_proxy: &ConstellationProxy,
        compositor: Rc<RefCell<IOCompositor>>,
    ) -> Self {
        let webview_id = WebViewId::new();

        Self(
            WebViewInner {
                id: webview_id,
                constellation_proxy: constellation_proxy.clone(),
                compositor,
            }
            .into(),
        )
    }

    /// FIXME: Remove this once we have a webview delegate.
    pub fn id(&self) -> WebViewId {
        self.0.id
    }

    pub fn focus(&self) {
        self.0
            .constellation_proxy
            .send(ConstellationMsg::FocusWebView(self.id()));
    }

    pub fn blur(&self) {
        self.0
            .constellation_proxy
            .send(ConstellationMsg::BlurWebView);
    }

    pub fn move_resize(&self, rect: DeviceRect) {
        self.0
            .compositor
            .borrow_mut()
            .move_resize_webview(self.id(), rect);
    }

    pub fn show(&self, hide_others: bool) {
        self.0
            .compositor
            .borrow_mut()
            .show_webview(self.id(), hide_others)
            .expect("BUG: invalid WebView instance");
    }

    pub fn hide(&self) {
        self.0
            .compositor
            .borrow_mut()
            .hide_webview(self.id())
            .expect("BUG: invalid WebView instance");
    }

    pub fn raise_to_top(&self, hide_others: bool) {
        self.0
            .compositor
            .borrow_mut()
            .raise_webview_to_top(self.id(), hide_others)
            .expect("BUG: invalid WebView instance");
    }

    pub fn notify_theme_change(&self, theme: Theme) {
        self.0
            .constellation_proxy
            .send(ConstellationMsg::ThemeChange(theme))
    }

    pub fn load(&self, url: Url) {
        self.0
            .constellation_proxy
            .send(ConstellationMsg::LoadUrl(self.id(), url.into()))
    }

    pub fn reload(&self) {
        self.0
            .constellation_proxy
            .send(ConstellationMsg::Reload(self.id()))
    }

    pub fn go_back(&self, amount: usize) {
        self.0
            .constellation_proxy
            .send(ConstellationMsg::TraverseHistory(
                self.id(),
                TraversalDirection::Back(amount),
            ))
    }

    pub fn go_forward(&self, amount: usize) {
        self.0
            .constellation_proxy
            .send(ConstellationMsg::TraverseHistory(
                self.id(),
                TraversalDirection::Forward(amount),
            ))
    }

    pub fn notify_pointer_button_event(&self, event: MouseWindowEvent) {
        self.0
            .compositor
            .borrow_mut()
            .on_mouse_window_event_class(event);
    }

    pub fn notify_pointer_move_event(&self, event: DevicePoint) {
        self.0
            .compositor
            .borrow_mut()
            .on_mouse_window_move_event_class(event);
    }

    pub fn notify_touch_event(&self, event_type: TouchEventType, id: TouchId, point: DevicePoint) {
        self.0
            .compositor
            .borrow_mut()
            .on_touch_event(event_type, id, point);
    }

    pub fn notify_wheel_event(&self, delta: WheelDelta, point: DevicePoint) {
        self.0.compositor.borrow_mut().on_wheel_event(delta, point);
    }

    pub fn notify_scroll_event(
        &self,
        location: ScrollLocation,
        point: DeviceIntPoint,
        touch_event_type: TouchEventType,
    ) {
        self.0
            .compositor
            .borrow_mut()
            .on_scroll_event(location, point, touch_event_type);
    }

    pub fn notify_keyboard_event(&self, event: KeyboardEvent) {
        self.0
            .constellation_proxy
            .send(ConstellationMsg::Keyboard(self.id(), event))
    }

    pub fn notify_ime_event(&self, event: CompositionEvent) {
        self.0
            .constellation_proxy
            .send(ConstellationMsg::IMECompositionEvent(event))
    }

    pub fn notify_ime_dismissed_event(&self) {
        self.0
            .constellation_proxy
            .send(ConstellationMsg::IMEDismissed);
    }

    pub fn notify_gamepad_event(&self, event: GamepadEvent) {
        self.0
            .constellation_proxy
            .send(ConstellationMsg::Gamepad(event));
    }

    pub fn notify_media_session_action_event(&self, event: MediaSessionActionType) {
        self.0
            .constellation_proxy
            .send(ConstellationMsg::MediaSessionAction(event));
    }

    pub fn notify_clipboard_event(&self, event: ClipboardEventType) {
        self.0
            .constellation_proxy
            .send(ConstellationMsg::Clipboard(event));
    }

    pub fn notify_vsync(&self) {
        self.0.compositor.borrow_mut().on_vsync();
    }

    pub fn notify_rendering_context_resized(&self) {
        self.0
            .compositor
            .borrow_mut()
            .on_rendering_context_resized();
    }

    pub fn set_zoom(&self, new_zoom: f32) {
        self.0
            .compositor
            .borrow_mut()
            .on_zoom_window_event(new_zoom);
    }

    pub fn reset_zoom(&self) {
        self.0.compositor.borrow_mut().on_zoom_reset_window_event();
    }

    pub fn set_pinch_zoom(&self, new_pinch_zoom: f32) {
        self.0
            .compositor
            .borrow_mut()
            .on_pinch_zoom_window_event(new_pinch_zoom);
    }

    pub fn exit_fullscreen(&self) {
        self.0
            .constellation_proxy
            .send(ConstellationMsg::ExitFullScreen(self.id()));
    }

    pub fn set_throttled(&self, throttled: bool) {
        self.0
            .constellation_proxy
            .send(ConstellationMsg::SetWebViewThrottled(self.id(), throttled));
    }

    pub fn toggle_webrender_debugging(&self, debugging: WebRenderDebugOption) {
        self.0
            .compositor
            .borrow_mut()
            .toggle_webrender_debug(debugging);
    }

    pub fn capture_webrender(&self) {
        self.0.compositor.borrow_mut().capture_webrender();
    }

    pub fn invalidate_native_surface(&self) {
        self.0.compositor.borrow_mut().invalidate_native_surface();
    }

    pub fn composite(&self) {
        self.0.compositor.borrow_mut().composite();
    }

    pub fn replace_native_surface(&self, native_widget: *mut c_void, size: DeviceIntSize) {
        self.0
            .compositor
            .borrow_mut()
            .replace_native_surface(native_widget, size);
    }

    pub fn toggle_sampling_profiler(&self, rate: Duration, max_duration: Duration) {
        self.0
            .constellation_proxy
            .send(ConstellationMsg::ToggleProfiler(rate, max_duration));
    }

    pub fn send_error(&self, message: String) {
        self.0
            .constellation_proxy
            .send(ConstellationMsg::SendError(Some(self.id()), message));
    }
}
