/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::RefCell;
use std::collections::HashMap;
use std::os::raw::c_void;
use std::rc::Rc;

use ipc_channel::ipc::IpcSender;
use keyboard_types::{CompositionEvent, CompositionState};
use log::{debug, error, info, warn};
use servo::base::id::WebViewId;
use servo::compositing::windowing::{
    AnimationState, EmbedderCoordinates, EmbedderMethods, MouseWindowEvent, WindowMethods,
};
use servo::euclid::{Box2D, Point2D, Rect, Scale, Size2D, Vector2D};
use servo::servo_geometry::DeviceIndependentPixel;
use servo::webrender_api::units::{DevicePixel, DeviceRect};
use servo::webrender_api::ScrollLocation;
use servo::webrender_traits::SurfmanRenderingContext;
use servo::{
    ContextMenuResult, EmbedderMsg, EmbedderProxy, EventLoopWaker, Key, KeyState, KeyboardEvent,
    MediaSessionActionType, MediaSessionEvent, MouseButton, PermissionPrompt, PermissionRequest,
    PromptDefinition, PromptOrigin, PromptResult, Servo, TouchEventType, TouchId, WebView,
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

pub struct ServoGlue {
    rendering_context: SurfmanRenderingContext,
    servo: Servo,
    need_present: bool,
    callbacks: Rc<ServoWindowCallbacks>,
    context_menu_sender: Option<IpcSender<ContextMenuResult>>,

    /// List of top-level browsing contexts.
    /// Modified by EmbedderMsg::WebViewOpened and EmbedderMsg::WebViewClosed,
    /// and we exit if it ever becomes empty.
    webviews: HashMap<WebViewId, WebView>,

    /// The order in which the webviews were created.
    creation_order: Vec<WebViewId>,

    /// The webview that is currently focused.
    /// Modified by EmbedderMsg::WebViewFocused and EmbedderMsg::WebViewBlurred.
    focused_webview_id: Option<WebViewId>,

    /// servoshell specific preferences created during startup of the application.
    servoshell_preferences: ServoShellPreferences,
}

#[allow(unused)]
impl ServoGlue {
    pub(super) fn new(
        initial_url: Option<String>,
        rendering_context: SurfmanRenderingContext,
        servo: Servo,
        callbacks: Rc<ServoWindowCallbacks>,
        servoshell_preferences: ServoShellPreferences,
    ) -> Self {
        let initial_url = initial_url.and_then(|string| Url::parse(&string).ok());
        let initial_url = initial_url
            .or_else(|| Url::parse(&servoshell_preferences.homepage).ok())
            .or_else(|| Url::parse("about:blank").ok())
            .unwrap();

        let webview = servo.new_webview(initial_url);
        let webview_id = webview.id();
        let webviews = [(webview_id, webview)].into();

        Self {
            rendering_context,
            servo,
            need_present: false,
            callbacks,
            context_menu_sender: None,
            webviews,
            creation_order: vec![],
            focused_webview_id: Some(webview_id),
            servoshell_preferences,
        }
    }

    fn get_browser_id(&self) -> Result<WebViewId, &'static str> {
        let webview_id = match self.focused_webview_id {
            Some(id) => id,
            None => return Err("No focused WebViewId yet."),
        };
        Ok(webview_id)
    }

    fn newest_webview(&self) -> Option<&WebView> {
        self.creation_order
            .last()
            .and_then(|id| self.webviews.get(id))
    }

    fn active_webview(&self) -> &WebView {
        self.focused_webview_id
            .and_then(|id| self.webviews.get(&id))
            .or(self.newest_webview())
            .expect("Should always have an active WebView")
    }

    /// Request shutdown. Will call on_shutdown_complete.
    pub fn request_shutdown(&mut self) {
        self.servo.start_shutting_down();
        self.maybe_perform_updates();
    }

    /// Call after on_shutdown_complete
    pub fn deinit(self) {
        self.servo.deinit();
    }

    /// Returns the webrender surface management integration interface.
    /// This provides the embedder access to the current front buffer.
    pub fn surfman(&self) -> SurfmanRenderingContext {
        self.rendering_context.clone()
    }

    /// This is the Servo heartbeat. This needs to be called
    /// everytime wakeup is called or when embedder wants Servo
    /// to act on its pending events.
    pub fn perform_updates(&mut self) {
        debug!("perform_updates");
        self.servo.handle_events(vec![]);
        let _ = self.handle_servo_events();
        debug!("done perform_updates");
    }

    /// Load an URL.
    pub fn load_uri(&mut self, url: &str) {
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
    pub fn reload(&mut self) {
        info!("reload");
        self.active_webview().reload();
        self.maybe_perform_updates()
    }

    /// Redraw the page.
    pub fn refresh(&mut self) {
        info!("refresh");
        self.active_webview().composite();
        self.maybe_perform_updates()
    }

    /// Stop loading the page.
    pub fn stop(&mut self) {
        warn!("TODO can't stop won't stop");
    }

    /// Go back in history.
    pub fn go_back(&mut self) {
        info!("go_back");
        self.active_webview().go_back(1);
        self.maybe_perform_updates()
    }

    /// Go forward in history.
    pub fn go_forward(&mut self) {
        info!("go_forward");
        self.active_webview().go_forward(1);
        self.maybe_perform_updates()
    }

    /// Let Servo know that the window has been resized.
    pub fn resize(&mut self, coordinates: Coordinates) {
        info!("resize to {:?}", coordinates);
        let size = coordinates.viewport.size;
        let _ = self
            .rendering_context
            .resize(Size2D::new(size.width, size.height))
            .inspect_err(|e| error!("Failed to resize rendering context: {e:?}"));
        *self.callbacks.coordinates.borrow_mut() = coordinates;
        self.active_webview().notify_rendering_context_resized();
        self.active_webview()
            .move_resize(DeviceRect::from_size(size.to_f32()));
        self.maybe_perform_updates()
    }

    /// Start scrolling.
    /// x/y are scroll coordinates.
    /// dx/dy are scroll deltas.
    #[cfg(not(target_env = "ohos"))]
    pub fn scroll_start(&mut self, dx: f32, dy: f32, x: i32, y: i32) {
        let delta = Vector2D::new(dx, dy);
        let scroll_location = ScrollLocation::Delta(delta);
        self.active_webview().notify_scroll_event(
            scroll_location,
            Point2D::new(x, y),
            TouchEventType::Down,
        );
        self.maybe_perform_updates()
    }

    /// Scroll.
    /// x/y are scroll coordinates.
    /// dx/dy are scroll deltas.
    pub fn scroll(&mut self, dx: f32, dy: f32, x: i32, y: i32) {
        let delta = Vector2D::new(dx, dy);
        let scroll_location = ScrollLocation::Delta(delta);
        self.active_webview().notify_scroll_event(
            scroll_location,
            Point2D::new(x, y),
            TouchEventType::Move,
        );
        self.maybe_perform_updates()
    }

    /// End scrolling.
    /// x/y are scroll coordinates.
    /// dx/dy are scroll deltas.
    #[cfg(not(target_env = "ohos"))]
    pub fn scroll_end(&mut self, dx: f32, dy: f32, x: i32, y: i32) {
        let delta = Vector2D::new(dx, dy);
        let scroll_location = ScrollLocation::Delta(delta);
        self.active_webview().notify_scroll_event(
            scroll_location,
            Point2D::new(x, y),
            TouchEventType::Up,
        );
        self.maybe_perform_updates()
    }

    /// Touch event: press down
    pub fn touch_down(&mut self, x: f32, y: f32, pointer_id: i32) {
        self.active_webview().notify_touch_event(
            TouchEventType::Down,
            TouchId(pointer_id),
            Point2D::new(x, y),
        );
        self.maybe_perform_updates()
    }

    /// Touch event: move touching finger
    pub fn touch_move(&mut self, x: f32, y: f32, pointer_id: i32) {
        self.active_webview().notify_touch_event(
            TouchEventType::Move,
            TouchId(pointer_id),
            Point2D::new(x, y),
        );
        self.maybe_perform_updates()
    }

    /// Touch event: Lift touching finger
    pub fn touch_up(&mut self, x: f32, y: f32, pointer_id: i32) {
        self.active_webview().notify_touch_event(
            TouchEventType::Up,
            TouchId(pointer_id),
            Point2D::new(x, y),
        );
        self.maybe_perform_updates()
    }

    /// Cancel touch event
    pub fn touch_cancel(&mut self, x: f32, y: f32, pointer_id: i32) {
        self.active_webview().notify_touch_event(
            TouchEventType::Cancel,
            TouchId(pointer_id),
            Point2D::new(x, y),
        );
        self.maybe_perform_updates()
    }

    /// Register a mouse movement.
    pub fn mouse_move(&mut self, x: f32, y: f32) {
        self.active_webview()
            .notify_pointer_move_event(Point2D::new(x, y));
        self.maybe_perform_updates()
    }

    /// Register a mouse button press.
    pub fn mouse_down(&mut self, x: f32, y: f32, button: MouseButton) {
        self.active_webview()
            .notify_pointer_button_event(MouseWindowEvent::MouseDown(button, Point2D::new(x, y)));
        self.maybe_perform_updates()
    }

    /// Register a mouse button release.
    pub fn mouse_up(&mut self, x: f32, y: f32, button: MouseButton) {
        self.active_webview()
            .notify_pointer_button_event(MouseWindowEvent::MouseUp(button, Point2D::new(x, y)));
        self.maybe_perform_updates()
    }

    /// Start pinchzoom.
    /// x/y are pinch origin coordinates.
    pub fn pinchzoom_start(&mut self, factor: f32, _x: u32, _y: u32) {
        self.active_webview().set_pinch_zoom(factor);
        self.maybe_perform_updates()
    }

    /// Pinchzoom.
    /// x/y are pinch origin coordinates.
    pub fn pinchzoom(&mut self, factor: f32, _x: u32, _y: u32) {
        self.active_webview().set_pinch_zoom(factor);
        self.maybe_perform_updates()
    }

    /// End pinchzoom.
    /// x/y are pinch origin coordinates.
    pub fn pinchzoom_end(&mut self, factor: f32, _x: u32, _y: u32) {
        self.active_webview().set_pinch_zoom(factor);
        self.maybe_perform_updates()
    }

    /// Perform a click.
    pub fn click(&mut self, x: f32, y: f32) {
        self.active_webview()
            .notify_pointer_button_event(MouseWindowEvent::Click(
                MouseButton::Left,
                Point2D::new(x, y),
            ));
        self.maybe_perform_updates()
    }

    pub fn key_down(&mut self, key: Key) {
        let key_event = KeyboardEvent {
            state: KeyState::Down,
            key,
            ..KeyboardEvent::default()
        };
        self.active_webview().notify_keyboard_event(key_event);
        self.maybe_perform_updates()
    }

    pub fn key_up(&mut self, key: Key) {
        let key_event = KeyboardEvent {
            state: KeyState::Up,
            key,
            ..KeyboardEvent::default()
        };
        self.active_webview().notify_keyboard_event(key_event);
        self.maybe_perform_updates()
    }

    pub fn ime_insert_text(&mut self, text: String) {
        self.active_webview().notify_ime_event(CompositionEvent {
            state: CompositionState::End,
            data: text,
        });
        self.maybe_perform_updates()
    }

    pub fn notify_vsync(&mut self) {
        self.active_webview().notify_vsync();
        self.maybe_perform_updates()
    }

    pub fn pause_compositor(&mut self) {
        if let Err(e) = self.rendering_context.unbind_native_surface_from_context() {
            warn!("Unbinding native surface from context failed ({:?})", e);
        }
        self.maybe_perform_updates();
    }

    pub fn resume_compositor(&mut self, native_surface: *mut c_void, coords: Coordinates) {
        if native_surface.is_null() {
            panic!("null passed for native_surface");
        }
        let connection = self.rendering_context.connection();
        let native_widget = unsafe {
            connection
                .create_native_widget_from_ptr(native_surface, coords.framebuffer.to_untyped())
        };
        if let Err(e) = self
            .rendering_context
            .bind_native_surface_to_context(native_widget)
        {
            warn!("Binding native surface to context failed ({:?})", e);
        }
        self.maybe_perform_updates()
    }

    pub fn media_session_action(&mut self, action: MediaSessionActionType) {
        info!("Media session action {:?}", action);
        self.active_webview()
            .notify_media_session_action_event(action);
        self.maybe_perform_updates()
    }

    pub fn set_throttled(&mut self, throttled: bool) {
        info!("set_throttled");
        self.active_webview().set_throttled(throttled);
        self.maybe_perform_updates()
    }

    pub fn ime_dismissed(&mut self) {
        info!("ime_dismissed");
        self.active_webview().notify_ime_dismissed_event();
        self.maybe_perform_updates()
    }

    pub fn on_context_menu_closed(
        &mut self,
        result: ContextMenuResult,
    ) -> Result<(), &'static str> {
        if let Some(sender) = self.context_menu_sender.take() {
            let _ = sender.send(result);
        } else {
            warn!("Trying to close a context menu when no context menu is active");
        }
        Ok(())
    }

    fn maybe_perform_updates(&mut self) {
        self.perform_updates();
    }

    fn handle_servo_events(&mut self) -> Result<(), &'static str> {
        let mut need_update = false;
        let messages = self.servo.get_events();
        for message in messages {
            match message {
                EmbedderMsg::ChangePageTitle(_, title) => {
                    self.callbacks.host_callbacks.on_title_changed(title);
                },
                EmbedderMsg::AllowNavigationRequest(_, pipeline_id, url) => {
                    let data: bool = self
                        .callbacks
                        .host_callbacks
                        .on_allow_navigation(url.to_string());
                    self.servo.allow_navigation_response(pipeline_id, data);
                    need_update = true;
                },
                EmbedderMsg::HistoryChanged(_, entries, current) => {
                    let can_go_back = current > 0;
                    let can_go_forward = current < entries.len() - 1;
                    self.callbacks
                        .host_callbacks
                        .on_history_changed(can_go_back, can_go_forward);
                    self.callbacks
                        .host_callbacks
                        .on_url_changed(entries[current].clone().to_string());
                },
                EmbedderMsg::NotifyLoadStatusChanged(_, load_status) => {
                    self.callbacks
                        .host_callbacks
                        .notify_load_status_changed(load_status);
                },
                EmbedderMsg::GetSelectedBluetoothDevice(_, _, sender) => {
                    let _ = sender.send(None);
                },
                EmbedderMsg::AllowUnload(_, sender) => {
                    let _ = sender.send(true);
                },
                EmbedderMsg::ShowContextMenu(_, sender, title, items) => {
                    if self.context_menu_sender.is_some() {
                        warn!(
                            "Trying to show a context menu when a context menu is already active"
                        );
                        let _ = sender.send(ContextMenuResult::Ignored);
                    } else {
                        self.context_menu_sender = Some(sender);
                        self.callbacks
                            .host_callbacks
                            .show_context_menu(title, items);
                    }
                },
                EmbedderMsg::Prompt(_, definition, origin) => {
                    let cb = &self.callbacks.host_callbacks;
                    let trusted = origin == PromptOrigin::Trusted;
                    let res = match definition {
                        PromptDefinition::Alert(message, sender) => {
                            cb.prompt_alert(message, trusted);
                            sender.send(())
                        },
                        PromptDefinition::OkCancel(message, sender) => {
                            sender.send(cb.prompt_ok_cancel(message, trusted))
                        },
                        PromptDefinition::Input(message, default, sender) => {
                            sender.send(cb.prompt_input(message, default, trusted))
                        },
                        PromptDefinition::Credentials(_) => {
                            warn!("implement credentials prompt for OpenHarmony OS and Android");
                            Ok(())
                        },
                    };
                    if let Err(e) = res {
                        self.active_webview()
                            .send_error(format!("Failed to send Prompt response: {e}"));
                    }
                },
                EmbedderMsg::AllowOpeningWebView(_, response_chan) => {
                    let new_webview = self.servo.new_auxiliary_webview();
                    let new_webview_id = new_webview.id();
                    self.webviews.insert(new_webview_id, new_webview);
                    self.creation_order.push(new_webview_id);

                    if let Err(e) = response_chan.send(Some(new_webview_id)) {
                        warn!("Failed to send AllowOpeningBrowser response: {}", e);
                    };
                },
                EmbedderMsg::WebViewOpened(new_webview_id) => {
                    if let Some(webview) = self.webviews.get(&new_webview_id) {
                        webview.focus();
                    }
                },
                EmbedderMsg::WebViewClosed(webview_id) => {
                    self.webviews.retain(|&id, _| id != webview_id);
                    self.creation_order.retain(|&id| id != webview_id);
                    self.focused_webview_id = None;

                    if let Some(newest_webview) = self.newest_webview() {
                        newest_webview.focus();
                    } else {
                        self.servo.start_shutting_down();
                    }
                },
                EmbedderMsg::WebViewFocused(webview_id) => {
                    self.focused_webview_id = Some(webview_id);
                    if let Some(webview) = self.webviews.get(&webview_id) {
                        webview.show(true);
                    }
                },
                EmbedderMsg::WebViewBlurred => {
                    self.focused_webview_id = None;
                },
                EmbedderMsg::GetClipboardContents(_, sender) => {
                    let contents = self.callbacks.host_callbacks.get_clipboard_contents();
                    let _ = sender.send(contents.unwrap_or("".to_owned()));
                },
                EmbedderMsg::SetClipboardContents(_, text) => {
                    self.callbacks.host_callbacks.set_clipboard_contents(text);
                },
                EmbedderMsg::Shutdown => {
                    self.callbacks.host_callbacks.on_shutdown_complete();
                },
                EmbedderMsg::PromptPermission(_, prompt, sender) => {
                    let message = match prompt {
                        PermissionPrompt::Request(permission_name) => {
                            format!("Do you want to grant permission for {:?}?", permission_name)
                        },
                        PermissionPrompt::Insecure(permission_name) => {
                            format!(
                                "The {:?} feature is only safe to use in secure context, but servo can't guarantee\n\
                                that the current context is secure. Do you want to proceed and grant permission?",
                                permission_name
                            )
                        },
                    };

                    let result = match self.callbacks.host_callbacks.prompt_yes_no(message, true) {
                        PromptResult::Primary => PermissionRequest::Granted,
                        PromptResult::Secondary | PromptResult::Dismissed => {
                            PermissionRequest::Denied
                        },
                    };

                    let _ = sender.send(result);
                },
                EmbedderMsg::ShowIME(_, kind, text, multiline, bounds) => {
                    self.callbacks
                        .host_callbacks
                        .on_ime_show(kind, text, multiline, bounds);
                },
                EmbedderMsg::HideIME(_) => {
                    self.callbacks.host_callbacks.on_ime_hide();
                },
                EmbedderMsg::MediaSessionEvent(_, event) => {
                    match event {
                        MediaSessionEvent::SetMetadata(metadata) => {
                            self.callbacks.host_callbacks.on_media_session_metadata(
                                metadata.title,
                                metadata.artist,
                                metadata.album,
                            )
                        },
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
                },
                EmbedderMsg::OnDevtoolsStarted(port, token) => {
                    self.callbacks
                        .host_callbacks
                        .on_devtools_started(port, token);
                },
                EmbedderMsg::RequestDevtoolsConnection(result_sender) => {
                    result_sender.send(true);
                },
                EmbedderMsg::Panic(_, reason, backtrace) => {
                    self.callbacks.host_callbacks.on_panic(reason, backtrace);
                },
                EmbedderMsg::ReadyToPresent(_webview_ids) => {
                    self.need_present = true;
                },
                EmbedderMsg::ResizeTo(_, size) => {
                    warn!("Received resize event (to {size:?}). Currently only the user can resize windows");
                },
                EmbedderMsg::Keyboard(..) |
                EmbedderMsg::Status(..) |
                EmbedderMsg::SelectFiles(..) |
                EmbedderMsg::MoveTo(..) |
                EmbedderMsg::SetCursor(..) |
                EmbedderMsg::NewFavicon(..) |
                EmbedderMsg::SetFullscreenState(..) |
                EmbedderMsg::ReportProfile(..) |
                EmbedderMsg::EventDelivered(..) |
                EmbedderMsg::PlayGamepadHapticEffect(..) |
                EmbedderMsg::StopGamepadHapticEffect(..) |
                EmbedderMsg::ClearClipboardContents(..) |
                EmbedderMsg::WebResourceRequested(..) => {},
            }
        }

        if need_update {
            self.perform_updates();
        }
        Ok(())
    }

    pub fn present_if_needed(&mut self) {
        if self.need_present {
            self.need_present = false;
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
