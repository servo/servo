/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use dpi::PhysicalSize;
use euclid::{Rect, Scale};
use keyboard_types::{CompositionEvent, CompositionState, Key, KeyState, NamedKey};
use log::{info, warn};
use raw_window_handle::{DisplayHandle, RawWindowHandle, WindowHandle};
use servo::{
    DeviceIndependentIntRect, DeviceIndependentPixel, DeviceIntSize, DevicePixel, DevicePoint,
    DeviceVector2D, EmbedderControl, EmbedderControlId, EventLoopWaker, ImeEvent, InputEvent,
    KeyboardEvent, LoadStatus, MouseButton, MouseButtonAction, MouseButtonEvent, MouseMoveEvent,
    Opts, Preferences, RefreshDriver, RenderingContext, ScreenGeometry, Scroll, Servo,
    ServoBuilder, SimpleDialog, TouchEvent, TouchEventType, TouchId, TouchPointerType,
    UserContentManager, WebView, WebViewId, WebViewPoint, WebViewVector, WindowRenderingContext,
    convert_rect_to_css_pixel,
};
use url::Url;

use crate::egl::host_trait::HostTrait;
use crate::prefs::ServoShellPreferences;
use crate::running_app_state::{RunningAppState, UserInterfaceCommand};
use crate::window::{
    PlatformWindow, ServoShellWindow, ServoShellWindowId, TopLevelWebViewCreationRequest,
};

pub(crate) struct EmbeddedPlatformWindow {
    host: Rc<dyn HostTrait>,
    rendering_context: Rc<WindowRenderingContext>,
    refresh_driver: Rc<VsyncRefreshDriver>,
    viewport_rect: RefCell<Rect<i32, DevicePixel>>,
    /// The HiDPI scaling factor to use for the display of [`WebView`]s.
    hidpi_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    /// A list of showing [`InputMethod`] interfaces.
    visible_input_methods: RefCell<Vec<EmbedderControlId>>,
    /// The current title of the active WebView in this window.
    current_title: RefCell<Option<String>>,
    /// The current URL of the active WebView in this window.
    current_url: RefCell<Option<Url>>,
    /// Whether or not the active WebView is currently able to go back.
    current_can_go_back: Cell<bool>,
    /// Whether or not the active WebView is currently able to go forward.
    current_can_go_forward: Cell<bool>,
    /// The current load status of the active WebView.
    current_load_status: Cell<Option<LoadStatus>>,

    id: ServoShellWindowId,
}

impl PlatformWindow for EmbeddedPlatformWindow {
    fn as_headed_window(&self) -> Option<&Self> {
        Some(self)
    }

    fn id(&self) -> ServoShellWindowId {
        self.id
    }

    fn screen_geometry(&self) -> ScreenGeometry {
        let viewport_rect = self.viewport_rect.borrow();
        ScreenGeometry {
            size: viewport_rect.size,
            available_size: viewport_rect.size,
            window_rect: viewport_rect.to_box2d(),
        }
    }

    fn device_hidpi_scale_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
        self.hidpi_scale_factor
    }

    fn hidpi_scale_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
        self.hidpi_scale_factor
    }

    fn get_fullscreen(&self) -> bool {
        false
    }

    fn update_user_interface_state(
        &self,
        state: &RunningAppState,
        window: &ServoShellWindow,
    ) -> bool {
        let Some(active_webview) = window.active_webview() else {
            return false;
        };

        let new_title = active_webview.page_title();
        let title_changed = new_title != *self.current_title.borrow();
        if title_changed {
            *self.current_title.borrow_mut() = new_title.clone();
            self.host.on_title_changed(new_title);
        }

        let new_url = active_webview.url();
        let url_changed = new_url != *self.current_url.borrow();
        if url_changed {
            let new_url_string = new_url.as_ref().map(Url::to_string).unwrap_or_default();
            *self.current_url.borrow_mut() = new_url;
            if self.has_platform_focus() {
                self.host.on_url_changed(new_url_string);
            }
        }

        let new_back_forward = (
            active_webview.can_go_back(),
            active_webview.can_go_forward(),
        );
        let old_back_forward = (
            self.current_can_go_back.get(),
            self.current_can_go_forward.get(),
        );
        let back_forward_changed = new_back_forward != old_back_forward;
        if back_forward_changed {
            self.current_can_go_back.set(new_back_forward.0);
            self.current_can_go_forward.set(new_back_forward.1);
            self.host
                .on_history_changed(new_back_forward.0, new_back_forward.1);
        }

        let new_load_status = active_webview.load_status();
        let load_status_changed = Some(new_load_status) != self.current_load_status.get();
        if load_status_changed {
            self.host.notify_load_status_changed(new_load_status);
            if state.servoshell_preferences.memory_output && new_load_status == LoadStatus::Complete
            {
                #[cfg(all(feature = "tracing", feature = "tracing-hitrace"))]
                {
                    let (callback, receiver) =
                        servo_base::generic_channel::GenericCallback::new_blocking()
                            .expect("Could not create channel");
                    state.servo().create_memory_report(callback);
                    std::thread::spawn(move || {
                        let result = receiver.recv().expect("Could not get memory report");
                        let reports = result
                            .results
                            .first()
                            .expect("We should have some memory report");
                        let search_string = String::from("resident-according-to-smaps");
                        let sum = reports
                            .reports
                            .iter()
                            .filter(|report| report.path.contains(&search_string))
                            .map(|report| report.size)
                            .sum::<usize>();
                        hitrace::trace_metric_str(
                            "servo_memory_profiling:resident-according-to-smaps/sum",
                            sum as i64,
                        );
                    });
                }
            }
        }

        title_changed || url_changed || back_forward_changed || load_status_changed
    }

    fn request_repaint(&self, window: &ServoShellWindow) {
        window.repaint_webviews();
    }

    fn request_resize(&self, _: &WebView, _: DeviceIntSize) -> Option<DeviceIntSize> {
        None
    }

    fn rendering_context(&self) -> Rc<dyn RenderingContext> {
        self.rendering_context.clone()
    }

    fn window_rect(&self) -> DeviceIndependentIntRect {
        convert_rect_to_css_pixel(
            self.viewport_rect.borrow().to_box2d(),
            self.hidpi_scale_factor(),
        )
    }

    fn show_embedder_control(&self, webview_id: WebViewId, embedder_control: EmbedderControl) {
        let control_id = embedder_control.id();
        match embedder_control {
            EmbedderControl::InputMethod(input_method_control)
                if input_method_control.allow_virtual_keyboard() =>
            {
                self.visible_input_methods.borrow_mut().push(control_id);
                self.host.on_ime_show(input_method_control);
            },
            EmbedderControl::SelectElement(prompt) => {
                self.host.on_show_select_element(webview_id, prompt);
            },
            EmbedderControl::SimpleDialog(SimpleDialog::Alert(alert_dialog)) => {
                self.host.show_alert(alert_dialog.message().into());
                alert_dialog.confirm();
            },
            _ => {}, // The drop implementation will send the default response.
        }
    }

    fn hide_embedder_control(&self, _: WebViewId, control_id: servo::EmbedderControlId) {
        let mut visible_input_methods = self.visible_input_methods.borrow_mut();
        if let Some(index) = visible_input_methods
            .iter()
            .position(|visible_id| *visible_id == control_id)
        {
            visible_input_methods.remove(index);
            self.host.on_ime_hide();
        }
    }

    fn notify_crashed(&self, _webview: WebView, reason: String, backtrace: Option<String>) {
        self.host.on_panic(reason, backtrace);
    }

    fn show_console_message(&self, level: servo::ConsoleLogLevel, message: &str) {
        log::log!(level.into(), "{message}");
    }
}

pub(crate) struct VsyncRefreshDriver {
    start_frame_callbacks: RefCell<Vec<Box<dyn Fn() + Send>>>,
    /// On OHOS we own the `OH_NativeVSync` handle and request a single callback
    /// only when an observer asks for one.
    #[cfg(target_env = "ohos")]
    native_vsync: ohos_vsync::NativeVsync,
}

impl VsyncRefreshDriver {
    pub(crate) fn new() -> Self {
        Self {
            start_frame_callbacks: Default::default(),
            #[cfg(target_env = "ohos")]
            native_vsync: ohos_vsync::NativeVsync::new("ServoVsync")
                .expect("Failed to create NativeVsync"),
        }
    }

    fn notify_vsync(&self) {
        let start_frame_callbacks: Vec<_> =
            self.start_frame_callbacks.borrow_mut().drain(..).collect();
        for start_frame_callback in start_frame_callbacks {
            start_frame_callback()
        }
    }

    /// Requests the next vsync callback, unconditionally.
    ///
    /// The frames of this port are requested on demand, and a caller that is not one of servo's own
    /// observers is not served by [`RefreshDriver::observe_next_frame`]: that entry point only arms
    /// the native vsync when its queue of callbacks was empty, and the arm that accompanied the
    /// callbacks already in the queue is *consumed* by the vsync being handled right now. Pushing
    /// another callback can therefore be a no-op with nothing left pending — and if servo itself
    /// stops observing in that same frame, no frame would ever be requested again, leaving an
    /// animation in flight with no frames to advance it.
    ///
    /// **Guarantee**: as long as the caller calls this on every frame it handles, a vsync callback
    /// always arrives — whatever the state of the callback queue, and whether or not servo's own
    /// observers are still alive. The caller stops calling it when its animation ends. Requesting
    /// the frame directly cannot storm the platform either: a raw callback executes at most once per
    /// vsync period (`egl/ohos/mod.rs`, `request_vsync_callback`: "Calling this multiple times in
    /// the same vsync period will result in one callback execution").
    #[cfg(target_env = "ohos")]
    pub(crate) fn request_next_frame(&self) {
        super::ohos::request_vsync_callback(&self.native_vsync);
    }
}

impl RefreshDriver for VsyncRefreshDriver {
    fn observe_next_frame(&self, new_start_frame_callback: Box<dyn Fn() + Send + 'static>) {
        // We only need to request a vsync callback if the queue is empty,
        // otherwise we will already have a pending callback.
        #[cfg_attr(not(target_env = "ohos"), allow(unused_variables))]
        let was_empty = {
            let mut callbacks = self.start_frame_callbacks.borrow_mut();
            let was_empty = callbacks.is_empty();
            callbacks.push(new_start_frame_callback);
            was_empty
        };
        #[cfg(target_env = "ohos")]
        if was_empty {
            super::ohos::request_vsync_callback(&self.native_vsync);
        }
    }
}

pub(crate) struct AppInitOptions {
    pub host: Rc<dyn HostTrait>,
    pub event_loop_waker: Box<dyn EventLoopWaker>,
    pub initial_url: Option<String>,
    pub opts: Opts,
    pub preferences: Preferences,
    pub servoshell_preferences: ServoShellPreferences,
    #[cfg(feature = "webxr")]
    pub xr_discovery: Option<servo::webxr::Discovery>,
}

pub struct App {
    pub(crate) state: Rc<RunningAppState>,
    // TODO: multi-window support, like desktop version.
    // This is just an intermediate state, to split refactoring into
    // multiple PRs.
    host: Rc<dyn HostTrait>,
    initial_url: Url,
}

#[expect(unused)]
impl App {
    #[servo::servo_tracing::instrument(skip_all, name = "App::new", level = "info")]
    pub(super) fn new(init: AppInitOptions) -> Rc<Self> {
        let mut servo_builder = ServoBuilder::default()
            .opts(init.opts)
            .preferences(init.preferences.clone())
            .event_loop_waker(init.event_loop_waker.clone());
        let servo = servo_builder.build();
        #[cfg(feature = "webxr")]
        servo.register_webxr_registry(Box::new(XrDiscoveryWebXrRegistry::new(init.xr_discovery)));

        let initial_url = init.initial_url.and_then(|string| Url::parse(&string).ok());
        let initial_url = initial_url
            .or_else(|| Url::parse(&init.servoshell_preferences.homepage).ok())
            .or_else(|| Url::parse("about:blank").ok())
            .expect("Failed to parse initial URL");

        let user_content_manager = Rc::new(UserContentManager::new(&servo));
        let state = Rc::new(RunningAppState::new(
            servo,
            init.servoshell_preferences,
            init.event_loop_waker,
            user_content_manager,
            init.preferences,
        ));

        Rc::new(Self {
            state,
            host: init.host,
            initial_url,
        })
    }

    pub(crate) fn add_platform_window(
        &self,
        display_handle: DisplayHandle,
        window_handle: WindowHandle,
        viewport_rect: Rect<i32, DevicePixel>,
        hidpi_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
        window_id: Option<ServoShellWindowId>,
    ) {
        let viewport_size = viewport_rect.size;
        let refresh_driver = Rc::new(VsyncRefreshDriver::new());
        let rendering_context = Rc::new(
            WindowRenderingContext::new_with_refresh_driver(
                display_handle,
                window_handle,
                PhysicalSize::new(viewport_size.width as u32, viewport_size.height as u32),
                refresh_driver.clone(),
            )
            .expect("Could not create RenderingContext"),
        );
        let id = window_id.unwrap_or(ServoShellWindowId::next());
        let platform_window = Rc::new(EmbeddedPlatformWindow {
            id,
            host: self.host.clone(),
            rendering_context,
            refresh_driver,
            viewport_rect: RefCell::new(viewport_rect),
            hidpi_scale_factor,
            visible_input_methods: Default::default(),
            current_title: Default::default(),
            current_url: Default::default(),
            current_can_go_back: Default::default(),
            current_can_go_forward: Default::default(),
            current_load_status: Default::default(),
        });
        self.state.open_window(
            platform_window,
            TopLevelWebViewCreationRequest::WithUrl(self.initial_url.clone()),
        );
    }

    pub(crate) fn servo(&self) -> &Servo {
        &self.state.servo
    }

    pub(crate) fn servoshell_preferences(&self) -> &ServoShellPreferences {
        &self.state.servoshell_preferences
    }

    pub(crate) fn window(&self) -> Rc<ServoShellWindow> {
        self.state
            .focused_window()
            .expect("There is always an active window")
    }

    pub(crate) fn active_or_newest_webview(&self) -> Option<WebView> {
        self.window().active_or_newest_webview()
    }

    pub(crate) fn initial_url(&self) -> Url {
        self.initial_url.clone()
    }

    pub(crate) fn create_and_activate_toplevel_webview(self: &Rc<Self>, url: Url) -> WebView {
        self.window().create_and_activate_toplevel_webview(
            self.state.clone(),
            TopLevelWebViewCreationRequest::WithUrl(url),
        )
    }

    /// The active webview will be immediately valid via `active_or_newest_webview()`
    pub(crate) fn activate_webview(&self, id: WebViewId) {
        let Some(webview) = self.state.webview_by_id(id) else {
            return;
        };
        self.state.window_for_webview(&webview).activate_webview(id);
    }

    /// This is the Servo heartbeat. This needs to be called
    /// everytime wakeup is called or when embedder wants Servo
    /// to act on its pending events.
    pub fn spin_event_loop(&self) {
        if !self
            .state
            .spin_event_loop(None /* create_platform_window */)
        {
            self.host.on_shutdown_complete()
        }
    }

    /// Load an URL.
    pub fn load_uri(&self, location: &str) {
        self.window()
            .queue_user_interface_command(UserInterfaceCommand::Go(location.into()));
        self.spin_event_loop();
    }

    /// Reload the page.
    pub fn reload(&self) {
        self.window()
            .queue_user_interface_command(UserInterfaceCommand::Reload);
        self.spin_event_loop();
    }

    /// Stop loading the page.
    pub fn stop(&self) {
        warn!("TODO can't stop won't stop");
    }

    /// Go back in history.
    pub fn go_back(&self) {
        self.window()
            .queue_user_interface_command(UserInterfaceCommand::Back);
        self.spin_event_loop();
    }

    /// Go forward in history.
    pub fn go_forward(&self) {
        self.window()
            .queue_user_interface_command(UserInterfaceCommand::Forward);
        self.spin_event_loop();
    }

    /// Let Servo know that the window has been resized.
    pub fn resize(&self, viewport_rect: Rect<i32, DevicePixel>) {
        if let Some(webview) = self.active_or_newest_webview() {
            info!("Setting viewport to {viewport_rect:?}");
            let size = viewport_rect.size;
            webview.resize(PhysicalSize::new(size.width as u32, size.height as u32));
        }
        let window = self.window().platform_window();
        let embedded_platform_window = window.as_headed_window().expect("No headed window");
        *embedded_platform_window.viewport_rect.borrow_mut() = viewport_rect;

        self.spin_event_loop();
    }

    /// Scroll.
    /// x/y are scroll coordinates.
    /// dx/dy are scroll deltas.
    pub fn scroll(&self, dx: f32, dy: f32, x: f32, y: f32) {
        if let Some(webview) = self.active_or_newest_webview() {
            let scroll = Scroll::Delta(DeviceVector2D::new(dx, dy).into());
            let point = DevicePoint::new(x, y).into();
            webview.notify_scroll_event(scroll, point);
            self.spin_event_loop();
        }
    }

    /// Queues a scroll for the active webview *without* pumping the event loop.
    ///
    /// This is the entry point of the frames of the coasting animation. They must not go through
    /// [`App::wheel`], and not through a synthetic wheel event at all: an `InputEvent::Wheel` is
    /// only registered in `WebViewRenderer::pending_wheel_events`
    /// (`components/paint/webview_renderer.rs:484-487` stores the raw, unscaled event), and the
    /// scroll itself then waits for the script thread to answer `InputEventsHandled`
    /// (`components/script/dom/document_event_handler.rs:346-424` →
    /// `components/script/script_thread.rs:1495-1543` → `components/servo/servo.rs:340-363`). That
    /// round trip is not synchronised with the frames, and a document that is not fully active, or
    /// is render-blocked, never answers at all: the motion would be lost, and its entry in
    /// `pending_wheel_events` would leak.
    ///
    /// A scroll notification takes the path of the touch fling of servo itself
    /// (`components/paint/touch.rs:564/608` → `WebView::notify_scroll_event`,
    /// `components/servo/webview.rs:601-607`), which ends in
    /// `WebViewRenderer::pending_scroll_zoom_events` (`components/paint/webview_renderer.rs:779-782`)
    /// and is consumed by `Painter::perform_updates` (`components/paint/painter.rs:310-311`). That
    /// runs unconditionally inside `Servo::spin_event_loop` (`components/servo/servo.rs:360`), so
    /// the motion is applied by the very pump that follows this call, with nothing to wait for.
    ///
    /// The values are the ones the wheel event would have carried, so the distance does not change:
    /// the wheel path ends up calling this same function
    /// (`components/paint/webview_renderer.rs:1192-1198`) with
    /// `DeviceVector2D::new(-wheel_event.delta.x, -wheel_event.delta.y)` and the point of the event.
    /// `Scroll::Delta` is positive towards the right and down, and `WebViewVector::Device` and
    /// `WebViewPoint::Device` are used as they are (`components/shared/embedder/lib.rs:143-157`,
    /// `:66-73`), so `delta` is the wheel delta *inverted* — the single flip the wheel path applies
    /// to the value this module is given.
    ///
    /// [`App::wheel`] keeps pumping, because a platform wheel or touchpad event arrives outside of
    /// any frame and must take effect immediately.
    #[cfg_attr(
        not(target_env = "ohos"),
        expect(
            dead_code,
            reason = "only the OpenHarmony port dispatches frames from inside a vsync"
        )
    )]
    pub(crate) fn queue_scroll(&self, delta: f32, x: f32, y: f32) {
        if let Some(webview) = self.active_or_newest_webview() {
            let scroll = Scroll::Delta(WebViewVector::Device(DeviceVector2D::new(0.0, -delta)));
            webview.notify_scroll_event(scroll, WebViewPoint::Device(DevicePoint::new(x, y)));
        }
    }

    /// Touch event: press down
    pub fn touch_down(&self, x: f32, y: f32, pointer_id: i32) {
        if let Some(webview) = self.active_or_newest_webview() {
            webview.notify_input_event(InputEvent::Touch(TouchEvent::new(
                TouchEventType::Down,
                TouchId(pointer_id),
                DevicePoint::new(x, y).into(),
                TouchPointerType::Touch,
            )));
            self.spin_event_loop();
        }
    }

    /// Touch event: move touching finger
    pub fn touch_move(&self, x: f32, y: f32, pointer_id: i32) {
        if let Some(webview) = self.active_or_newest_webview() {
            webview.notify_input_event(InputEvent::Touch(TouchEvent::new(
                TouchEventType::Move,
                TouchId(pointer_id),
                DevicePoint::new(x, y).into(),
                TouchPointerType::Touch,
            )));
            self.spin_event_loop();
        }
    }

    /// Touch event: Lift touching finger
    pub fn touch_up(&self, x: f32, y: f32, pointer_id: i32) {
        if let Some(webview) = self.active_or_newest_webview() {
            webview.notify_input_event(InputEvent::Touch(TouchEvent::new(
                TouchEventType::Up,
                TouchId(pointer_id),
                DevicePoint::new(x, y).into(),
                TouchPointerType::Touch,
            )));
            self.spin_event_loop();
        }
    }

    /// Cancel touch event
    pub fn touch_cancel(&self, x: f32, y: f32, pointer_id: i32) {
        if let Some(webview) = self.active_or_newest_webview() {
            webview.notify_input_event(InputEvent::Touch(TouchEvent::new(
                TouchEventType::Cancel,
                TouchId(pointer_id),
                DevicePoint::new(x, y).into(),
                TouchPointerType::Touch,
            )));
            self.spin_event_loop();
        }
    }

    /// Register a mouse movement.
    pub fn mouse_move(&self, x: f32, y: f32) {
        if let Some(webview) = self.active_or_newest_webview() {
            webview.notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(
                DevicePoint::new(x, y).into(),
            )));
            self.spin_event_loop();
        }
    }

    /// Register a mouse button press.
    pub fn mouse_down(&self, x: f32, y: f32, button: MouseButton) {
        if let Some(webview) = self.active_or_newest_webview() {
            webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                MouseButtonAction::Down,
                button,
                DevicePoint::new(x, y).into(),
            )));
            self.spin_event_loop();
        }
    }

    /// Register a mouse button release.
    pub fn mouse_up(&self, x: f32, y: f32, button: MouseButton) {
        if let Some(webview) = self.active_or_newest_webview() {
            webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                MouseButtonAction::Up,
                button,
                DevicePoint::new(x, y).into(),
            )));
            self.spin_event_loop();
        }
    }

    /// Start pinchzoom.
    /// x/y are pinch origin coordinates.
    pub fn pinchzoom_start(&self, factor: f32, x: f32, y: f32) {
        if let Some(webview) = self.active_or_newest_webview() {
            webview.adjust_pinch_zoom(factor, DevicePoint::new(x, y));
            self.spin_event_loop();
        }
    }

    /// Pinchzoom.
    /// x/y are pinch origin coordinates.
    pub fn pinchzoom(&self, factor: f32, x: f32, y: f32) {
        if let Some(webview) = self.active_or_newest_webview() {
            webview.adjust_pinch_zoom(factor, DevicePoint::new(x, y));
            self.spin_event_loop();
        }
    }

    /// End pinchzoom.
    /// x/y are pinch origin coordinates.
    pub fn pinchzoom_end(&self, factor: f32, x: f32, y: f32) {
        if let Some(webview) = self.active_or_newest_webview() {
            webview.adjust_pinch_zoom(factor, DevicePoint::new(x, y));
            self.spin_event_loop();
        }
    }

    pub fn key_down(&self, key: Key) {
        if let Some(webview) = self.active_or_newest_webview() {
            webview.notify_input_event(InputEvent::Keyboard(KeyboardEvent::from_state_and_key(
                KeyState::Down,
                key,
            )));
            self.spin_event_loop();
        }
    }

    pub fn key_up(&self, key: Key) {
        if let Some(webview) = self.active_or_newest_webview() {
            webview.notify_input_event(InputEvent::Keyboard(KeyboardEvent::from_state_and_key(
                KeyState::Up,
                key,
            )));
            self.spin_event_loop();
        }
    }

    pub fn key_event(&self, event: keyboard_types::KeyboardEvent) {
        if let Some(webview) = self.active_or_newest_webview() {
            webview.notify_input_event(InputEvent::Keyboard(KeyboardEvent::new(event)));
            self.spin_event_loop();
        }
    }

    pub fn ime_insert_text(&self, text: String) {
        // In OHOS, we get empty text after the intended text.
        if text.is_empty() {
            return;
        }

        let Some(webview) = self.active_or_newest_webview() else {
            return;
        };

        webview.notify_input_event(InputEvent::Keyboard(KeyboardEvent::from_state_and_key(
            KeyState::Down,
            Key::Named(NamedKey::Process),
        )));
        webview.notify_input_event(InputEvent::Ime(ImeEvent::Composition(CompositionEvent {
            state: CompositionState::End,
            data: text,
        })));
        webview.notify_input_event(InputEvent::Keyboard(KeyboardEvent::from_state_and_key(
            KeyState::Up,
            Key::Named(NamedKey::Process),
        )));
        self.spin_event_loop();
    }

    pub fn ime_dismissed(&self) {
        if let Some(webview) = self.active_or_newest_webview() {
            webview.notify_input_event(InputEvent::Ime(ImeEvent::Dismissed));
        }
        self.host.on_ime_hide();
        self.spin_event_loop();
    }

    pub fn notify_vsync(&self) {
        let platform_window = self.window().platform_window();
        let embedded_platform_window = platform_window
            .as_headed_window()
            .expect("No headed window");
        embedded_platform_window.refresh_driver.notify_vsync();
        self.spin_event_loop();
    }

    /// Asks for one more vsync, even when nothing else is waiting for a frame.
    ///
    /// The coasting animation is driven by the frames of the port without being one of servo's own
    /// observers, so a glide would freeze halfway through as soon as servo no longer needs a frame
    /// — with the animation still armed, which would make the next unrelated vsync dump the whole
    /// remaining distance in a single jump. Requesting the frame here is what keeps the glide
    /// alive; it stops being requested on the frame that finishes the animation.
    #[cfg(target_env = "ohos")]
    pub(crate) fn request_next_frame(&self) {
        let platform_window = self.window().platform_window();
        let embedded_platform_window = platform_window
            .as_headed_window()
            .expect("No headed window");
        embedded_platform_window.refresh_driver.request_next_frame();
    }
    pub fn pause_painting(&self) {
        let platform_window = self.window().platform_window();
        let embedded_platform_window = platform_window
            .as_headed_window()
            .expect("No headed window");
        if let Err(error) = embedded_platform_window.rendering_context.take_window() {
            warn!("Unbinding native surface from context failed ({error:?})");
        }
        self.spin_event_loop();
    }

    pub fn resume_painting(
        &self,
        window_handle: RawWindowHandle,
        viewport_rect: Rect<i32, DevicePixel>,
    ) {
        let window_handle = unsafe { WindowHandle::borrow_raw(window_handle) };
        let size = viewport_rect.size.to_u32();
        let platform_window = self.window().platform_window();
        let embedded_platform_window = platform_window
            .as_headed_window()
            .expect("No headed window");
        if let Err(error) = embedded_platform_window
            .rendering_context
            .set_window(window_handle, PhysicalSize::new(size.width, size.height))
        {
            warn!("Binding native surface to context failed ({error:?})");
        }
        embedded_platform_window.request_repaint(&self.window());
        self.spin_event_loop();
    }
}

#[cfg(feature = "webxr")]
pub(crate) struct XrDiscoveryWebXrRegistry {
    xr_discovery: RefCell<Option<servo::webxr::Discovery>>,
}

#[cfg(feature = "webxr")]
impl XrDiscoveryWebXrRegistry {
    pub(crate) fn new(xr_discovery: Option<servo::webxr::Discovery>) -> Self {
        Self {
            xr_discovery: RefCell::new(xr_discovery),
        }
    }
}

#[cfg(feature = "webxr")]
impl servo::webxr::WebXrRegistry for XrDiscoveryWebXrRegistry {
    fn register(&self, registry: &mut servo::webxr::MainThreadRegistry) {
        log::debug!("XrDiscoveryWebXrRegistry::register");
        if let Some(discovery) = self.xr_discovery.take() {
            registry.register(discovery);
        }
    }
}
