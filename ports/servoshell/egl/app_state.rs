/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

use dpi::PhysicalSize;
use ipc_channel::ipc::IpcSender;
use keyboard_types::{CompositionEvent, CompositionState};
use log::{debug, error, info, warn};
use raw_window_handle::{RawWindowHandle, WindowHandle};
use servo::base::id::WebViewId;
use servo::euclid::{Point2D, Rect, Scale, Size2D, Vector2D};
use servo::servo_geometry::DeviceIndependentPixel;
use servo::webrender_api::ScrollLocation;
use servo::webrender_api::units::{DeviceIntRect, DeviceIntSize, DevicePixel};
use servo::{
    AllowOrDenyRequest, ContextMenuResult, ImeEvent, InputEvent, InputMethodType, Key, KeyState,
    KeyboardEvent, LoadStatus, MediaSessionActionType, MediaSessionEvent, MouseButton,
    MouseButtonAction, MouseButtonEvent, MouseMoveEvent, NavigationRequest, PermissionRequest,
    RenderingContext, ScreenGeometry, Servo, ServoDelegate, ServoError, SimpleDialog, TouchEvent,
    TouchEventType, TouchId, WebView, WebViewBuilder, WebViewDelegate, WindowRenderingContext,
};
use url::Url;

use crate::egl::host_trait::HostTrait;
use crate::output_image::save_output_image_if_necessary;
use crate::prefs::ServoShellPreferences;

#[derive(Clone, Debug)]
pub struct Coordinates {
    pub viewport: Rect<i32, DevicePixel>,
}

impl Coordinates {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Coordinates {
        Coordinates {
            viewport: Rect::new(Point2D::new(x, y), Size2D::new(width, height)),
        }
    }

    pub fn origin(&self) -> Point2D<i32, DevicePixel> {
        self.viewport.origin
    }

    pub fn size(&self) -> Size2D<i32, DevicePixel> {
        self.viewport.size
    }
}

pub(super) struct ServoWindowCallbacks {
    host_callbacks: Box<dyn HostTrait>,
    coordinates: RefCell<Coordinates>,
}

impl ServoWindowCallbacks {
    pub(super) fn new(
        host_callbacks: Box<dyn HostTrait>,
        coordinates: RefCell<Coordinates>,
    ) -> Self {
        Self {
            host_callbacks,
            coordinates,
        }
    }
}

pub struct RunningAppState {
    servo: Servo,
    rendering_context: Rc<WindowRenderingContext>,
    callbacks: Rc<ServoWindowCallbacks>,
    inner: RefCell<RunningAppStateInner>,
    /// servoshell specific preferences created during startup of the application.
    servoshell_preferences: ServoShellPreferences,
}

struct RunningAppStateInner {
    need_present: bool,
    /// List of top-level browsing contexts.
    /// Modified by EmbedderMsg::WebViewOpened and EmbedderMsg::WebViewClosed,
    /// and we exit if it ever becomes empty.
    webviews: HashMap<WebViewId, WebView>,

    /// The order in which the webviews were created.
    creation_order: Vec<WebViewId>,

    /// The webview that is currently focused.
    /// Modified by EmbedderMsg::WebViewFocused and EmbedderMsg::WebViewBlurred.
    focused_webview_id: Option<WebViewId>,

    context_menu_sender: Option<IpcSender<ContextMenuResult>>,

    /// Whether or not the animation state has changed. This is used to trigger
    /// host callbacks indicating that animation state has changed.
    animating_state_changed: Rc<Cell<bool>>,

    /// The HiDPI scaling factor to use for the display of [`WebView`]s.
    hidpi_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
}

struct ServoShellServoDelegate {
    animating_state_changed: Rc<Cell<bool>>,
}

impl ServoDelegate for ServoShellServoDelegate {
    fn notify_devtools_server_started(&self, _servo: &Servo, port: u16, _token: String) {
        info!("Devtools Server running on port {port}");
    }

    fn request_devtools_connection(&self, _servo: &Servo, request: AllowOrDenyRequest) {
        request.allow();
    }

    fn notify_error(&self, _servo: &Servo, error: ServoError) {
        error!("Saw Servo error: {error:?}!");
    }

    fn notify_animating_changed(&self, _animating: bool) {
        self.animating_state_changed.set(true);
    }
}

impl WebViewDelegate for RunningAppState {
    fn screen_geometry(&self, _webview: WebView) -> Option<ScreenGeometry> {
        let coord = self.callbacks.coordinates.borrow();
        let offset = coord.origin();
        let available_size = coord.size();
        let screen_size = coord.size();
        Some(ScreenGeometry {
            size: screen_size,
            available_size,
            offset,
        })
    }

    fn notify_page_title_changed(&self, _webview: servo::WebView, title: Option<String>) {
        self.callbacks.host_callbacks.on_title_changed(title);
    }

    fn notify_history_changed(&self, _webview: WebView, entries: Vec<Url>, current: usize) {
        let can_go_back = current > 0;
        let can_go_forward = current < entries.len() - 1;
        self.callbacks
            .host_callbacks
            .on_history_changed(can_go_back, can_go_forward);
        self.callbacks
            .host_callbacks
            .on_url_changed(entries[current].clone().to_string());
    }

    fn notify_load_status_changed(&self, _webview: WebView, load_status: LoadStatus) {
        self.callbacks
            .host_callbacks
            .notify_load_status_changed(load_status);

        #[cfg(feature = "tracing")]
        if load_status == LoadStatus::Complete {
            #[cfg(feature = "tracing-hitrace")]
            let (snd, recv) = ipc_channel::ipc::channel().expect("Could not create channel");
            self.servo.create_memory_report(snd);
            std::thread::spawn(move || {
                let result = recv.recv().expect("Could not get memory report");
                let reports = result
                    .results
                    .first()
                    .expect("We should have some memory report");
                for report in &reports.reports {
                    let path = String::from("servo_memory_profiling:") + &report.path.join("/");
                    hitrace::trace_metric_str(&path, report.size as i64);
                }
            });
        }
    }

    fn notify_closed(&self, webview: WebView) {
        {
            let mut inner_mut = self.inner_mut();
            inner_mut.webviews.retain(|&id, _| id != webview.id());
            inner_mut.creation_order.retain(|&id| id != webview.id());
            inner_mut.focused_webview_id = None;
        }

        if let Some(newest_webview) = self.newest_webview() {
            newest_webview.focus();
        } else {
            self.servo.start_shutting_down();
        }
    }

    fn notify_focus_changed(&self, webview: WebView, focused: bool) {
        if focused {
            self.inner_mut().focused_webview_id = Some(webview.id());
            webview.show(true);
        } else if self.inner().focused_webview_id == Some(webview.id()) {
            self.inner_mut().focused_webview_id = None;
        }
    }

    fn notify_media_session_event(&self, _webview: WebView, event: MediaSessionEvent) {
        match event {
            MediaSessionEvent::SetMetadata(metadata) => self
                .callbacks
                .host_callbacks
                .on_media_session_metadata(metadata.title, metadata.artist, metadata.album),
            MediaSessionEvent::PlaybackStateChange(state) => self
                .callbacks
                .host_callbacks
                .on_media_session_playback_state_change(state),
            MediaSessionEvent::SetPositionState(position_state) => self
                .callbacks
                .host_callbacks
                .on_media_session_set_position_state(
                    position_state.duration,
                    position_state.position,
                    position_state.playback_rate,
                ),
        };
    }

    fn notify_crashed(&self, _webview: WebView, reason: String, backtrace: Option<String>) {
        self.callbacks.host_callbacks.on_panic(reason, backtrace);
    }

    fn notify_new_frame_ready(&self, _webview: WebView) {
        self.inner_mut().need_present = true;
    }

    fn request_navigation(&self, _webview: WebView, navigation_request: NavigationRequest) {
        if self
            .callbacks
            .host_callbacks
            .on_allow_navigation(navigation_request.url.to_string())
        {
            navigation_request.allow();
        } else {
            navigation_request.deny();
        }
    }

    fn request_open_auxiliary_webview(&self, parent_webview: WebView) -> Option<WebView> {
        let webview = WebViewBuilder::new_auxiliary(&self.servo)
            .delegate(parent_webview.delegate())
            .hidpi_scale_factor(self.inner().hidpi_scale_factor)
            .build();
        self.add(webview.clone());
        Some(webview)
    }

    fn request_permission(&self, webview: WebView, request: PermissionRequest) {
        self.callbacks
            .host_callbacks
            .request_permission(webview, request);
    }

    fn request_resize_to(&self, _webview: WebView, size: DeviceIntSize) {
        warn!("Received resize event (to {size:?}). Currently only the user can resize windows");
    }

    fn show_context_menu(
        &self,
        _webview: WebView,
        result_sender: IpcSender<ContextMenuResult>,
        title: Option<String>,
        items: Vec<String>,
    ) {
        if self.inner().context_menu_sender.is_some() {
            warn!("Trying to show a context menu when a context menu is already active");
            let _ = result_sender.send(ContextMenuResult::Ignored);
        } else {
            self.inner_mut().context_menu_sender = Some(result_sender);
            self.callbacks
                .host_callbacks
                .show_context_menu(title, items);
        }
    }

    fn show_simple_dialog(&self, webview: WebView, dialog: SimpleDialog) {
        self.callbacks
            .host_callbacks
            .show_simple_dialog(webview, dialog);
    }

    fn show_ime(
        &self,
        _webview: WebView,
        input_method_type: InputMethodType,
        text: Option<(String, i32)>,
        multiline: bool,
        position: DeviceIntRect,
    ) {
        self.callbacks
            .host_callbacks
            .on_ime_show(input_method_type, text, multiline, position);
    }

    fn hide_ime(&self, _webview: WebView) {
        self.callbacks.host_callbacks.on_ime_hide();
    }
}

#[allow(unused)]
impl RunningAppState {
    pub(super) fn new(
        initial_url: Option<String>,
        hidpi_scale_factor: f32,
        rendering_context: Rc<WindowRenderingContext>,
        servo: Servo,
        callbacks: Rc<ServoWindowCallbacks>,
        servoshell_preferences: ServoShellPreferences,
    ) -> Rc<Self> {
        let initial_url = initial_url.and_then(|string| Url::parse(&string).ok());
        let initial_url = initial_url
            .or_else(|| Url::parse(&servoshell_preferences.homepage).ok())
            .or_else(|| Url::parse("about:blank").ok())
            .unwrap();

        let animating_state_changed = Rc::new(Cell::new(false));
        servo.set_delegate(Rc::new(ServoShellServoDelegate {
            animating_state_changed: animating_state_changed.clone(),
        }));

        let app_state = Rc::new(Self {
            rendering_context,
            servo,
            callbacks,
            servoshell_preferences,
            inner: RefCell::new(RunningAppStateInner {
                need_present: false,
                context_menu_sender: None,
                webviews: Default::default(),
                creation_order: vec![],
                focused_webview_id: None,
                animating_state_changed,
                hidpi_scale_factor: Scale::new(hidpi_scale_factor),
            }),
        });

        app_state.new_toplevel_webview(initial_url);
        app_state
    }

    pub(crate) fn new_toplevel_webview(self: &Rc<Self>, url: Url) {
        let webview = WebViewBuilder::new(&self.servo)
            .url(url)
            .hidpi_scale_factor(self.inner().hidpi_scale_factor)
            .delegate(self.clone())
            .build();

        webview.focus();
        self.add(webview.clone());
    }

    pub(crate) fn add(&self, webview: WebView) {
        self.inner_mut().creation_order.push(webview.id());
        self.inner_mut().webviews.insert(webview.id(), webview);
    }

    fn inner(&self) -> Ref<RunningAppStateInner> {
        self.inner.borrow()
    }

    fn inner_mut(&self) -> RefMut<RunningAppStateInner> {
        self.inner.borrow_mut()
    }

    fn get_browser_id(&self) -> Result<WebViewId, &'static str> {
        let webview_id = match self.inner().focused_webview_id {
            Some(id) => id,
            None => return Err("No focused WebViewId yet."),
        };
        Ok(webview_id)
    }

    fn newest_webview(&self) -> Option<WebView> {
        self.inner()
            .creation_order
            .last()
            .and_then(|id| self.inner().webviews.get(id).cloned())
    }

    fn active_webview(&self) -> WebView {
        self.inner()
            .focused_webview_id
            .and_then(|id| self.inner().webviews.get(&id).cloned())
            .or(self.newest_webview())
            .expect("Should always have an active WebView")
    }

    /// Request shutdown. Will call on_shutdown_complete.
    pub fn request_shutdown(&self) {
        self.servo.start_shutting_down();
        self.perform_updates();
    }

    /// Call after on_shutdown_complete
    pub fn deinit(self) {
        self.servo.deinit();
    }

    /// This is the Servo heartbeat. This needs to be called
    /// everytime wakeup is called or when embedder wants Servo
    /// to act on its pending events.
    pub fn perform_updates(&self) {
        let should_continue = self.servo.spin_event_loop();
        if !should_continue {
            self.callbacks.host_callbacks.on_shutdown_complete();
        }
        if self.inner().animating_state_changed.get() {
            self.inner().animating_state_changed.set(false);
            self.callbacks
                .host_callbacks
                .on_animating_changed(self.servo.animating());
        }
    }

    /// Load an URL.
    pub fn load_uri(&self, url: &str) {
        info!("load_uri: {}", url);

        let Some(url) =
            crate::parser::location_bar_input_to_url(url, &self.servoshell_preferences.searchpage)
        else {
            warn!("Cannot parse URL");
            return;
        };

        self.active_webview().load(url.into_url());
    }

    /// Reload the page.
    pub fn reload(&self) {
        info!("reload");
        self.active_webview().reload();
        self.perform_updates();
    }

    /// Stop loading the page.
    pub fn stop(&self) {
        warn!("TODO can't stop won't stop");
    }

    /// Go back in history.
    pub fn go_back(&self) {
        info!("go_back");
        self.active_webview().go_back(1);
        self.perform_updates();
    }

    /// Go forward in history.
    pub fn go_forward(&self) {
        info!("go_forward");
        self.active_webview().go_forward(1);
        self.perform_updates();
    }

    /// Let Servo know that the window has been resized.
    pub fn resize(&self, coordinates: Coordinates) {
        info!("resize to {:?}", coordinates,);
        self.active_webview().resize(PhysicalSize::new(
            coordinates.viewport.width() as u32,
            coordinates.viewport.height() as u32,
        ));
        *self.callbacks.coordinates.borrow_mut() = coordinates;
        self.perform_updates();
    }

    /// Start scrolling.
    /// x/y are scroll coordinates.
    /// dx/dy are scroll deltas.
    #[cfg(not(target_env = "ohos"))]
    pub fn scroll_start(&self, dx: f32, dy: f32, x: i32, y: i32) {
        let delta = Vector2D::new(dx, dy);
        let scroll_location = ScrollLocation::Delta(delta);
        self.active_webview().notify_scroll_event(
            scroll_location,
            Point2D::new(x, y),
            TouchEventType::Down,
        );
        self.perform_updates();
    }

    /// Scroll.
    /// x/y are scroll coordinates.
    /// dx/dy are scroll deltas.
    pub fn scroll(&self, dx: f32, dy: f32, x: i32, y: i32) {
        let delta = Vector2D::new(dx, dy);
        let scroll_location = ScrollLocation::Delta(delta);
        self.active_webview().notify_scroll_event(
            scroll_location,
            Point2D::new(x, y),
            TouchEventType::Move,
        );
        self.perform_updates();
    }

    /// End scrolling.
    /// x/y are scroll coordinates.
    /// dx/dy are scroll deltas.
    #[cfg(not(target_env = "ohos"))]
    pub fn scroll_end(&self, dx: f32, dy: f32, x: i32, y: i32) {
        let delta = Vector2D::new(dx, dy);
        let scroll_location = ScrollLocation::Delta(delta);
        self.active_webview().notify_scroll_event(
            scroll_location,
            Point2D::new(x, y),
            TouchEventType::Up,
        );
        self.perform_updates();
    }

    /// Touch event: press down
    pub fn touch_down(&self, x: f32, y: f32, pointer_id: i32) {
        self.active_webview()
            .notify_input_event(InputEvent::Touch(TouchEvent::new(
                TouchEventType::Down,
                TouchId(pointer_id),
                Point2D::new(x, y),
            )));
        self.perform_updates();
    }

    /// Touch event: move touching finger
    pub fn touch_move(&self, x: f32, y: f32, pointer_id: i32) {
        self.active_webview()
            .notify_input_event(InputEvent::Touch(TouchEvent::new(
                TouchEventType::Move,
                TouchId(pointer_id),
                Point2D::new(x, y),
            )));
        self.perform_updates();
    }

    /// Touch event: Lift touching finger
    pub fn touch_up(&self, x: f32, y: f32, pointer_id: i32) {
        self.active_webview()
            .notify_input_event(InputEvent::Touch(TouchEvent::new(
                TouchEventType::Up,
                TouchId(pointer_id),
                Point2D::new(x, y),
            )));
        self.perform_updates();
    }

    /// Cancel touch event
    pub fn touch_cancel(&self, x: f32, y: f32, pointer_id: i32) {
        self.active_webview()
            .notify_input_event(InputEvent::Touch(TouchEvent::new(
                TouchEventType::Cancel,
                TouchId(pointer_id),
                Point2D::new(x, y),
            )));
        self.perform_updates();
    }

    /// Register a mouse movement.
    pub fn mouse_move(&self, x: f32, y: f32) {
        self.active_webview()
            .notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(Point2D::new(
                x, y,
            ))));
        self.perform_updates();
    }

    /// Register a mouse button press.
    pub fn mouse_down(&self, x: f32, y: f32, button: MouseButton) {
        self.active_webview()
            .notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                MouseButtonAction::Down,
                button,
                Point2D::new(x, y),
            )));
        self.perform_updates();
    }

    /// Register a mouse button release.
    pub fn mouse_up(&self, x: f32, y: f32, button: MouseButton) {
        self.active_webview()
            .notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                MouseButtonAction::Up,
                button,
                Point2D::new(x, y),
            )));
        self.perform_updates();
    }

    /// Start pinchzoom.
    /// x/y are pinch origin coordinates.
    pub fn pinchzoom_start(&self, factor: f32, _x: u32, _y: u32) {
        self.active_webview().set_pinch_zoom(factor);
        self.perform_updates();
    }

    /// Pinchzoom.
    /// x/y are pinch origin coordinates.
    pub fn pinchzoom(&self, factor: f32, _x: u32, _y: u32) {
        self.active_webview().set_pinch_zoom(factor);
        self.perform_updates();
    }

    /// End pinchzoom.
    /// x/y are pinch origin coordinates.
    pub fn pinchzoom_end(&self, factor: f32, _x: u32, _y: u32) {
        self.active_webview().set_pinch_zoom(factor);
        self.perform_updates();
    }

    /// Perform a click.
    pub fn click(&self, x: f32, y: f32) {
        self.active_webview()
            .notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                MouseButtonAction::Click,
                MouseButton::Left,
                Point2D::new(x, y),
            )));
        self.perform_updates();
    }

    pub fn key_down(&self, key: Key) {
        let key_event = KeyboardEvent {
            state: KeyState::Down,
            key,
            ..KeyboardEvent::default()
        };
        self.active_webview()
            .notify_input_event(InputEvent::Keyboard(key_event));
        self.perform_updates();
    }

    pub fn key_up(&self, key: Key) {
        let key_event = KeyboardEvent {
            state: KeyState::Up,
            key,
            ..KeyboardEvent::default()
        };
        self.active_webview()
            .notify_input_event(InputEvent::Keyboard(key_event));
        self.perform_updates();
    }

    pub fn ime_insert_text(&self, text: String) {
        // In OHOS, we get empty text after the intended text.
        if text.is_empty() {
            return;
        }
        let active_webview = self.active_webview();
        active_webview.notify_input_event(InputEvent::Keyboard(KeyboardEvent {
            state: KeyState::Down,
            key: Key::Process,
            ..KeyboardEvent::default()
        }));
        active_webview.notify_input_event(InputEvent::Ime(ImeEvent::Composition(
            CompositionEvent {
                state: CompositionState::End,
                data: text,
            },
        )));
        active_webview.notify_input_event(InputEvent::Keyboard(KeyboardEvent {
            state: KeyState::Up,
            key: Key::Process,
            ..KeyboardEvent::default()
        }));
        self.perform_updates();
    }

    pub fn notify_vsync(&self) {
        self.active_webview().notify_vsync();
        self.perform_updates();
    }

    pub fn pause_compositor(&self) {
        if let Err(e) = self.rendering_context.take_window() {
            warn!("Unbinding native surface from context failed ({:?})", e);
        }
        self.perform_updates();
    }

    pub fn resume_compositor(&self, window_handle: RawWindowHandle, coords: Coordinates) {
        let window_handle = unsafe { WindowHandle::borrow_raw(window_handle) };
        let size = coords.viewport.size.to_u32();
        if let Err(e) = self
            .rendering_context
            .set_window(window_handle, PhysicalSize::new(size.width, size.height))
        {
            warn!("Binding native surface to context failed ({:?})", e);
        }
        self.perform_updates();
    }

    pub fn media_session_action(&self, action: MediaSessionActionType) {
        info!("Media session action {:?}", action);
        self.active_webview()
            .notify_media_session_action_event(action);
        self.perform_updates();
    }

    pub fn set_throttled(&self, throttled: bool) {
        info!("set_throttled");
        self.active_webview().set_throttled(throttled);
        self.perform_updates();
    }

    pub fn ime_dismissed(&self) {
        info!("ime_dismissed");
        self.active_webview()
            .notify_input_event(InputEvent::Ime(ImeEvent::Dismissed));
        self.perform_updates();
    }

    pub fn on_context_menu_closed(&self, result: ContextMenuResult) -> Result<(), &'static str> {
        if let Some(sender) = self.inner_mut().context_menu_sender.take() {
            let _ = sender.send(result);
        } else {
            warn!("Trying to close a context menu when no context menu is active");
        }
        Ok(())
    }

    pub fn present_if_needed(&self) {
        if self.inner().need_present {
            self.inner_mut().need_present = false;
            if !self.active_webview().paint() {
                return;
            }
            save_output_image_if_necessary(&self.servoshell_preferences, &self.rendering_context);
            self.rendering_context.present();
            if self.servoshell_preferences.exit_after_stable_image {
                self.request_shutdown();
            }
        }
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
        debug!("XrDiscoveryWebXrRegistry::register");
        if let Some(discovery) = self.xr_discovery.take() {
            registry.register(discovery);
        }
    }
}
