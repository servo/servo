/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

use dpi::PhysicalSize;
use ipc_channel::ipc::IpcSender;
use keyboard_types::{CompositionEvent, CompositionState};
use log::{debug, error, info, warn};
use raw_window_handle::{RawWindowHandle, WindowHandle};
use servo::base::id::WebViewId;
use servo::compositing::windowing::{
    AnimationState, EmbedderCoordinates, EmbedderMethods, WindowMethods,
};
use servo::euclid::{Box2D, Point2D, Rect, Scale, Size2D, Vector2D};
use servo::servo_geometry::DeviceIndependentPixel;
use servo::webrender_api::units::{DeviceIntRect, DeviceIntSize, DevicePixel, DeviceRect};
use servo::webrender_api::ScrollLocation;
use servo::{
    AllowOrDenyRequest, ContextMenuResult, EmbedderProxy, EventLoopWaker, ImeEvent, InputEvent,
    InputMethodType, Key, KeyState, KeyboardEvent, LoadStatus, MediaSessionActionType,
    MediaSessionEvent, MouseButton, MouseButtonAction, MouseButtonEvent, MouseMoveEvent,
    NavigationRequest, PermissionRequest, PromptDefinition, PromptOrigin, PromptResult,
    RenderingContext, Servo, ServoDelegate, ServoError, TouchAction, TouchEvent, TouchEventType,
    TouchId, WebView, WebViewDelegate, WindowRenderingContext,
};
use url::Url;

use crate::egl::host_trait::HostTrait;
use crate::prefs::ServoShellPreferences;

#[derive(Clone, Debug)]
pub struct Coordinates {
    pub viewport: Rect<i32, DevicePixel>,
    pub framebuffer: Size2D<i32, DevicePixel>,
}

impl Coordinates {
    pub fn new(
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        fb_width: i32,
        fb_height: i32,
    ) -> Coordinates {
        Coordinates {
            viewport: Rect::new(Point2D::new(x, y), Size2D::new(width, height)),
            framebuffer: Size2D::new(fb_width, fb_height),
        }
    }

    pub(crate) fn framebuffer_size(&self) -> PhysicalSize<u32> {
        PhysicalSize::new(
            self.framebuffer.width as u32,
            self.framebuffer.height as u32,
        )
    }
}

pub(super) struct ServoWindowCallbacks {
    host_callbacks: Box<dyn HostTrait>,
    coordinates: RefCell<Coordinates>,
    hidpi_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
}

impl ServoWindowCallbacks {
    pub(super) fn new(
        host_callbacks: Box<dyn HostTrait>,
        coordinates: RefCell<Coordinates>,
        hidpi_factor: f32,
    ) -> Self {
        Self {
            host_callbacks,
            coordinates,
            hidpi_factor: Scale::new(hidpi_factor),
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
}

struct ServoShellServoDelegate;
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
}

impl WebViewDelegate for RunningAppState {
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
    }

    fn notify_ready_to_show(&self, webview: WebView) {
        webview.focus();
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

    fn request_open_auxiliary_webview(&self, _parent_webview: WebView) -> Option<WebView> {
        let new_webview = self.servo.new_auxiliary_webview();
        self.add(new_webview.clone());
        Some(new_webview)
    }

    fn request_permission(&self, _webview: WebView, request: PermissionRequest) {
        let message = format!(
            "Do you want to grant permission for {:?}?",
            request.feature()
        );
        let result = match self.callbacks.host_callbacks.prompt_yes_no(message, true) {
            PromptResult::Primary => request.allow(),
            PromptResult::Secondary | PromptResult::Dismissed => request.deny(),
        };
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

    fn show_prompt(&self, _webview: WebView, prompt: PromptDefinition, origin: PromptOrigin) {
        let cb = &self.callbacks.host_callbacks;
        let trusted = origin == PromptOrigin::Trusted;
        let _ = match prompt {
            PromptDefinition::Alert(message, response_sender) => {
                cb.prompt_alert(message, trusted);
                response_sender.send(())
            },
            PromptDefinition::OkCancel(message, response_sender) => {
                response_sender.send(cb.prompt_ok_cancel(message, trusted))
            },
            PromptDefinition::Input(message, default, response_sender) => {
                response_sender.send(cb.prompt_input(message, default, trusted))
            },
        };
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

        servo.set_delegate(Rc::new(ServoShellServoDelegate));

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
            }),
        });

        app_state.new_toplevel_webview(initial_url);
        app_state
    }

    pub(crate) fn new_toplevel_webview(self: &Rc<Self>, url: Url) {
        let webview = self.servo.new_webview(url);
        webview.set_delegate(self.clone());
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
        debug!("perform_updates");
        let should_continue = self.servo.spin_event_loop();
        if !should_continue {
            self.callbacks.host_callbacks.on_shutdown_complete();
        }
        debug!("done perform_updates");
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
        info!("resize to {:?}", coordinates);
        let size = coordinates.viewport.size;
        self.rendering_context
            .resize(Size2D::new(size.width, size.height));
        *self.callbacks.coordinates.borrow_mut() = coordinates;
        self.active_webview().notify_rendering_context_resized();
        self.active_webview()
            .move_resize(DeviceRect::from_size(size.to_f32()));
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
            .notify_input_event(InputEvent::Touch(TouchEvent {
                event_type: TouchEventType::Down,
                id: TouchId(pointer_id),
                point: Point2D::new(x, y),
                action: TouchAction::NoAction,
            }));
        self.perform_updates();
    }

    /// Touch event: move touching finger
    pub fn touch_move(&self, x: f32, y: f32, pointer_id: i32) {
        self.active_webview()
            .notify_input_event(InputEvent::Touch(TouchEvent {
                event_type: TouchEventType::Move,
                id: TouchId(pointer_id),
                point: Point2D::new(x, y),
                action: TouchAction::NoAction,
            }));
        self.perform_updates();
    }

    /// Touch event: Lift touching finger
    pub fn touch_up(&self, x: f32, y: f32, pointer_id: i32) {
        self.active_webview()
            .notify_input_event(InputEvent::Touch(TouchEvent {
                event_type: TouchEventType::Up,
                id: TouchId(pointer_id),
                point: Point2D::new(x, y),
                action: TouchAction::NoAction,
            }));
        self.perform_updates();
    }

    /// Cancel touch event
    pub fn touch_cancel(&self, x: f32, y: f32, pointer_id: i32) {
        self.active_webview()
            .notify_input_event(InputEvent::Touch(TouchEvent {
                event_type: TouchEventType::Cancel,
                id: TouchId(pointer_id),
                point: Point2D::new(x, y),
                action: TouchAction::NoAction,
            }));
        self.perform_updates();
    }

    /// Register a mouse movement.
    pub fn mouse_move(&self, x: f32, y: f32) {
        self.active_webview()
            .notify_input_event(InputEvent::MouseMove(MouseMoveEvent {
                point: Point2D::new(x, y),
            }));
        self.perform_updates();
    }

    /// Register a mouse button press.
    pub fn mouse_down(&self, x: f32, y: f32, button: MouseButton) {
        self.active_webview()
            .notify_input_event(InputEvent::MouseButton(MouseButtonEvent {
                action: MouseButtonAction::Down,
                button,
                point: Point2D::new(x, y),
            }));
        self.perform_updates();
    }

    /// Register a mouse button release.
    pub fn mouse_up(&self, x: f32, y: f32, button: MouseButton) {
        self.active_webview()
            .notify_input_event(InputEvent::MouseButton(MouseButtonEvent {
                action: MouseButtonAction::Up,
                button,
                point: Point2D::new(x, y),
            }));
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
            .notify_input_event(InputEvent::MouseButton(MouseButtonEvent {
                action: MouseButtonAction::Click,
                button: MouseButton::Left,
                point: Point2D::new(x, y),
            }));
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
        self.active_webview()
            .notify_input_event(InputEvent::Ime(ImeEvent::Composition(CompositionEvent {
                state: CompositionState::End,
                data: text,
            })));
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
        if let Err(e) = self
            .rendering_context
            .set_window(window_handle, &coords.framebuffer_size())
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
            self.active_webview().paint_immediately();
            self.servo.present();
        }
    }
}

pub(super) struct ServoEmbedderCallbacks {
    waker: Box<dyn EventLoopWaker>,
    #[cfg(feature = "webxr")]
    xr_discovery: Option<servo::webxr::Discovery>,
}

impl ServoEmbedderCallbacks {
    pub(super) fn new(
        waker: Box<dyn EventLoopWaker>,
        #[cfg(feature = "webxr")] xr_discovery: Option<servo::webxr::Discovery>,
    ) -> Self {
        Self {
            waker,
            #[cfg(feature = "webxr")]
            xr_discovery,
        }
    }
}

impl EmbedderMethods for ServoEmbedderCallbacks {
    fn create_event_loop_waker(&mut self) -> Box<dyn EventLoopWaker> {
        debug!("EmbedderMethods::create_event_loop_waker");
        self.waker.clone()
    }

    #[cfg(feature = "webxr")]
    fn register_webxr(
        &mut self,
        registry: &mut servo::webxr::MainThreadRegistry,
        _embedder_proxy: EmbedderProxy,
    ) {
        debug!("EmbedderMethods::register_xr");
        if let Some(discovery) = self.xr_discovery.take() {
            registry.register(discovery);
        }
    }
}

impl WindowMethods for ServoWindowCallbacks {
    fn get_coordinates(&self) -> EmbedderCoordinates {
        let coords = self.coordinates.borrow();
        let screen_size = (coords.viewport.size.to_f32() / self.hidpi_factor).to_i32();
        EmbedderCoordinates {
            viewport: coords.viewport.to_box2d(),
            framebuffer: coords.framebuffer,
            window_rect: Box2D::from_origin_and_size(Point2D::zero(), screen_size),
            screen_size,
            available_screen_size: screen_size,
            hidpi_factor: self.hidpi_factor,
        }
    }

    fn set_animation_state(&self, state: AnimationState) {
        debug!("WindowMethods::set_animation_state: {:?}", state);
        self.host_callbacks
            .on_animating_changed(state == AnimationState::Animating);
    }
}
