/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;
use std::os::raw::c_void;
use std::rc::Rc;

use ipc_channel::ipc::IpcSender;
use log::{debug, info, warn};
use servo::base::id::WebViewId;
use servo::compositing::windowing::{
    AnimationState, EmbedderCoordinates, EmbedderEvent, EmbedderMethods, MouseWindowEvent,
    WindowMethods,
};
use servo::embedder_traits::{
    ContextMenuResult, EmbedderMsg, EmbedderProxy, EventLoopWaker, MediaSessionEvent,
    PermissionPrompt, PermissionRequest, PromptDefinition, PromptOrigin, PromptResult,
};
use servo::euclid::{Point2D, Rect, Scale, Size2D, Vector2D};
use servo::keyboard_types::{Key, KeyState, KeyboardEvent};
use servo::script_traits::{
    MediaSessionActionType, MouseButton, TouchEventType, TouchId, TraversalDirection,
};
use servo::servo_url::ServoUrl;
use servo::style_traits::DevicePixel;
use servo::webrender_api::ScrollLocation;
use servo::webrender_traits::RenderingContext;
use servo::{gl, Servo, TopLevelBrowsingContextId};

use crate::egl::host_trait::HostTrait;

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
    density: f32,
    rendering_context: RenderingContext,
}

impl ServoWindowCallbacks {
    pub(super) fn new(
        host_callbacks: Box<dyn HostTrait>,
        coordinates: RefCell<Coordinates>,
        density: f32,
        rendering_context: RenderingContext,
    ) -> Self {
        Self {
            host_callbacks,
            coordinates,
            density,
            rendering_context,
        }
    }
}

#[derive(Debug)]
pub struct WebView {}

pub struct ServoGlue {
    rendering_context: RenderingContext,
    servo: Servo<ServoWindowCallbacks>,
    batch_mode: bool,
    need_present: bool,
    callbacks: Rc<ServoWindowCallbacks>,
    events: Vec<EmbedderEvent>,
    resource_dir: Option<String>,
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
}

#[allow(unused)]
impl ServoGlue {
    pub(super) fn new(
        rendering_context: RenderingContext,
        servo: Servo<ServoWindowCallbacks>,
        callbacks: Rc<ServoWindowCallbacks>,
        resource_dir: Option<String>,
    ) -> Self {
        Self {
            rendering_context,
            servo,
            batch_mode: false,
            need_present: false,
            callbacks,
            events: vec![],
            resource_dir,
            context_menu_sender: None,
            webviews: HashMap::default(),
            creation_order: vec![],
            focused_webview_id: None,
        }
    }

    fn get_browser_id(&self) -> Result<TopLevelBrowsingContextId, &'static str> {
        let webview_id = match self.focused_webview_id {
            Some(id) => id,
            None => return Err("No focused WebViewId yet."),
        };
        Ok(webview_id)
    }

    /// Request shutdown. Will call on_shutdown_complete.
    pub fn request_shutdown(&mut self) -> Result<(), &'static str> {
        self.process_event(EmbedderEvent::Quit)
    }

    /// Call after on_shutdown_complete
    pub fn deinit(self) {
        self.servo.deinit();
    }

    /// Returns the webrender surface management integration interface.
    /// This provides the embedder access to the current front buffer.
    pub fn surfman(&self) -> RenderingContext {
        self.rendering_context.clone()
    }

    /// This is the Servo heartbeat. This needs to be called
    /// everytime wakeup is called or when embedder wants Servo
    /// to act on its pending events.
    pub fn perform_updates(&mut self) -> Result<(), &'static str> {
        debug!("perform_updates");
        let events = mem::replace(&mut self.events, Vec::new());
        self.servo.handle_events(events);
        let r = self.handle_servo_events();
        debug!("done perform_updates");
        r
    }

    /// In batch mode, Servo won't call perform_updates automatically.
    /// This can be useful when the embedder wants to control when Servo
    /// acts on its pending events. For example, if the embedder wants Servo
    /// to act on the scroll events only at a certain time, not everytime
    /// scroll() is called.
    pub fn set_batch_mode(&mut self, batch: bool) -> Result<(), &'static str> {
        debug!("set_batch_mode");
        self.batch_mode = batch;
        Ok(())
    }

    /// Load an URL.
    pub fn load_uri(&mut self, url: &str) -> Result<(), &'static str> {
        info!("load_uri: {}", url);
        crate::parser::location_bar_input_to_url(url)
            .ok_or("Can't parse URL")
            .and_then(|url| {
                let browser_id = self.get_browser_id()?;
                let event = EmbedderEvent::LoadUrl(browser_id, url);
                self.process_event(event)
            })
    }

    /// Reload the page.
    pub fn clear_cache(&mut self) -> Result<(), &'static str> {
        info!("clear_cache");
        let event = EmbedderEvent::ClearCache;
        self.process_event(event)
    }

    /// Reload the page.
    pub fn reload(&mut self) -> Result<(), &'static str> {
        info!("reload");
        let browser_id = self.get_browser_id()?;
        let event = EmbedderEvent::Reload(browser_id);
        self.process_event(event)
    }

    /// Redraw the page.
    pub fn refresh(&mut self) -> Result<(), &'static str> {
        info!("refresh");
        self.process_event(EmbedderEvent::Refresh)
    }

    /// Stop loading the page.
    pub fn stop(&mut self) -> Result<(), &'static str> {
        warn!("TODO can't stop won't stop");
        Ok(())
    }

    /// Go back in history.
    pub fn go_back(&mut self) -> Result<(), &'static str> {
        info!("go_back");
        let browser_id = self.get_browser_id()?;
        let event = EmbedderEvent::Navigation(browser_id, TraversalDirection::Back(1));
        self.process_event(event)
    }

    /// Go forward in history.
    pub fn go_forward(&mut self) -> Result<(), &'static str> {
        info!("go_forward");
        let browser_id = self.get_browser_id()?;
        let event = EmbedderEvent::Navigation(browser_id, TraversalDirection::Forward(1));
        self.process_event(event)
    }

    /// Let Servo know that the window has been resized.
    pub fn resize(&mut self, coordinates: Coordinates) -> Result<(), &'static str> {
        info!("resize");
        *self.callbacks.coordinates.borrow_mut() = coordinates;
        self.process_event(EmbedderEvent::WindowResize)
    }

    /// Start scrolling.
    /// x/y are scroll coordinates.
    /// dx/dy are scroll deltas.
    #[cfg(not(target_env = "ohos"))]
    pub fn scroll_start(&mut self, dx: f32, dy: f32, x: i32, y: i32) -> Result<(), &'static str> {
        let delta = Vector2D::new(dx, dy);
        let scroll_location = ScrollLocation::Delta(delta);
        let event =
            EmbedderEvent::Scroll(scroll_location, Point2D::new(x, y), TouchEventType::Down);
        self.process_event(event)
    }

    /// Scroll.
    /// x/y are scroll coordinates.
    /// dx/dy are scroll deltas.
    pub fn scroll(&mut self, dx: f32, dy: f32, x: i32, y: i32) -> Result<(), &'static str> {
        let delta = Vector2D::new(dx, dy);
        let scroll_location = ScrollLocation::Delta(delta);
        let event =
            EmbedderEvent::Scroll(scroll_location, Point2D::new(x, y), TouchEventType::Move);
        self.process_event(event)
    }

    /// End scrolling.
    /// x/y are scroll coordinates.
    /// dx/dy are scroll deltas.
    #[cfg(not(target_env = "ohos"))]
    pub fn scroll_end(&mut self, dx: f32, dy: f32, x: i32, y: i32) -> Result<(), &'static str> {
        let delta = Vector2D::new(dx, dy);
        let scroll_location = ScrollLocation::Delta(delta);
        let event = EmbedderEvent::Scroll(scroll_location, Point2D::new(x, y), TouchEventType::Up);
        self.process_event(event)
    }

    /// Touch event: press down
    pub fn touch_down(&mut self, x: f32, y: f32, pointer_id: i32) -> Result<(), &'static str> {
        let event = EmbedderEvent::Touch(
            TouchEventType::Down,
            TouchId(pointer_id),
            Point2D::new(x as f32, y as f32),
        );
        self.process_event(event)
    }

    /// Touch event: move touching finger
    pub fn touch_move(&mut self, x: f32, y: f32, pointer_id: i32) -> Result<(), &'static str> {
        let event = EmbedderEvent::Touch(
            TouchEventType::Move,
            TouchId(pointer_id),
            Point2D::new(x as f32, y as f32),
        );
        self.process_event(event)
    }

    /// Touch event: Lift touching finger
    pub fn touch_up(&mut self, x: f32, y: f32, pointer_id: i32) -> Result<(), &'static str> {
        let event = EmbedderEvent::Touch(
            TouchEventType::Up,
            TouchId(pointer_id),
            Point2D::new(x as f32, y as f32),
        );
        self.process_event(event)
    }

    /// Cancel touch event
    pub fn touch_cancel(&mut self, x: f32, y: f32, pointer_id: i32) -> Result<(), &'static str> {
        let event = EmbedderEvent::Touch(
            TouchEventType::Cancel,
            TouchId(pointer_id),
            Point2D::new(x as f32, y as f32),
        );
        self.process_event(event)
    }

    /// Register a mouse movement.
    pub fn mouse_move(&mut self, x: f32, y: f32) -> Result<(), &'static str> {
        let point = Point2D::new(x, y);
        let event = EmbedderEvent::MouseWindowMoveEventClass(point);
        self.process_event(event)
    }

    /// Register a mouse button press.
    pub fn mouse_down(&mut self, x: f32, y: f32, button: MouseButton) -> Result<(), &'static str> {
        let point = Point2D::new(x, y);
        let event =
            EmbedderEvent::MouseWindowEventClass(MouseWindowEvent::MouseDown(button, point));
        self.process_event(event)
    }

    /// Register a mouse button release.
    pub fn mouse_up(&mut self, x: f32, y: f32, button: MouseButton) -> Result<(), &'static str> {
        let point = Point2D::new(x, y);
        let event = EmbedderEvent::MouseWindowEventClass(MouseWindowEvent::MouseUp(button, point));
        self.process_event(event)
    }

    /// Start pinchzoom.
    /// x/y are pinch origin coordinates.
    pub fn pinchzoom_start(&mut self, factor: f32, _x: u32, _y: u32) -> Result<(), &'static str> {
        self.process_event(EmbedderEvent::PinchZoom(factor))
    }

    /// Pinchzoom.
    /// x/y are pinch origin coordinates.
    pub fn pinchzoom(&mut self, factor: f32, _x: u32, _y: u32) -> Result<(), &'static str> {
        self.process_event(EmbedderEvent::PinchZoom(factor))
    }

    /// End pinchzoom.
    /// x/y are pinch origin coordinates.
    pub fn pinchzoom_end(&mut self, factor: f32, _x: u32, _y: u32) -> Result<(), &'static str> {
        self.process_event(EmbedderEvent::PinchZoom(factor))
    }

    /// Perform a click.
    pub fn click(&mut self, x: f32, y: f32) -> Result<(), &'static str> {
        let mouse_event = MouseWindowEvent::Click(MouseButton::Left, Point2D::new(x, y));
        let event = EmbedderEvent::MouseWindowEventClass(mouse_event);
        self.process_event(event)
    }

    pub fn key_down(&mut self, key: Key) -> Result<(), &'static str> {
        let key_event = KeyboardEvent {
            state: KeyState::Down,
            key,
            ..KeyboardEvent::default()
        };
        self.process_event(EmbedderEvent::Keyboard(key_event))
    }

    pub fn key_up(&mut self, key: Key) -> Result<(), &'static str> {
        let key_event = KeyboardEvent {
            state: KeyState::Up,
            key,
            ..KeyboardEvent::default()
        };
        self.process_event(EmbedderEvent::Keyboard(key_event))
    }

    pub fn pause_compositor(&mut self) -> Result<(), &'static str> {
        self.process_event(EmbedderEvent::InvalidateNativeSurface)
    }

    pub fn resume_compositor(
        &mut self,
        native_surface: *mut c_void,
        coords: Coordinates,
    ) -> Result<(), &'static str> {
        if native_surface.is_null() {
            panic!("null passed for native_surface");
        }
        self.process_event(EmbedderEvent::ReplaceNativeSurface(
            native_surface,
            coords.framebuffer,
        ))
    }

    pub fn media_session_action(
        &mut self,
        action: MediaSessionActionType,
    ) -> Result<(), &'static str> {
        info!("Media session action {:?}", action);
        self.process_event(EmbedderEvent::MediaSessionAction(action))
    }

    pub fn set_throttled(&mut self, throttled: bool) -> Result<(), &'static str> {
        info!("set_throttled");
        if let Ok(id) = self.get_browser_id() {
            let event = EmbedderEvent::SetWebViewThrottled(id, throttled);
            self.process_event(event)
        } else {
            // Ignore visibility change if no browser has been created yet.
            Ok(())
        }
    }

    pub fn ime_dismissed(&mut self) -> Result<(), &'static str> {
        info!("ime_dismissed");
        self.process_event(EmbedderEvent::IMEDismissed)
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

    pub(super) fn process_event(&mut self, event: EmbedderEvent) -> Result<(), &'static str> {
        self.events.push(event);
        if !self.batch_mode {
            self.perform_updates()
        } else {
            Ok(())
        }
    }

    fn handle_servo_events(&mut self) -> Result<(), &'static str> {
        let mut need_update = false;
        for (browser_id, event) in self.servo.get_events() {
            match event {
                EmbedderMsg::ChangePageTitle(title) => {
                    self.callbacks.host_callbacks.on_title_changed(title);
                },
                EmbedderMsg::AllowNavigationRequest(pipeline_id, url) => {
                    if let Some(_browser_id) = browser_id {
                        let data: bool = self
                            .callbacks
                            .host_callbacks
                            .on_allow_navigation(url.to_string());
                        let window_event =
                            EmbedderEvent::AllowNavigationResponse(pipeline_id, data);
                        self.events.push(window_event);
                        need_update = true;
                    }
                },
                EmbedderMsg::HistoryChanged(entries, current) => {
                    let can_go_back = current > 0;
                    let can_go_forward = current < entries.len() - 1;
                    self.callbacks
                        .host_callbacks
                        .on_history_changed(can_go_back, can_go_forward);
                    self.callbacks
                        .host_callbacks
                        .on_url_changed(entries[current].clone().to_string());
                },
                EmbedderMsg::LoadStart => {
                    self.callbacks.host_callbacks.on_load_started();
                },
                EmbedderMsg::LoadComplete => {
                    self.callbacks.host_callbacks.on_load_ended();
                },
                EmbedderMsg::GetSelectedBluetoothDevice(_, sender) => {
                    let _ = sender.send(None);
                },
                EmbedderMsg::AllowUnload(sender) => {
                    let _ = sender.send(true);
                },
                EmbedderMsg::ShowContextMenu(sender, title, items) => {
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
                EmbedderMsg::Prompt(definition, origin) => {
                    let cb = &self.callbacks.host_callbacks;
                    let trusted = origin == PromptOrigin::Trusted;
                    let res = match definition {
                        PromptDefinition::Alert(message, sender) => {
                            sender.send(cb.prompt_alert(message, trusted))
                        },
                        PromptDefinition::OkCancel(message, sender) => {
                            sender.send(cb.prompt_ok_cancel(message, trusted))
                        },
                        PromptDefinition::YesNo(message, sender) => {
                            sender.send(cb.prompt_yes_no(message, trusted))
                        },
                        PromptDefinition::Input(message, default, sender) => {
                            sender.send(cb.prompt_input(message, default, trusted))
                        },
                    };
                    if let Err(e) = res {
                        let reason = format!("Failed to send Prompt response: {}", e);
                        self.events
                            .push(EmbedderEvent::SendError(browser_id, reason));
                    }
                },
                EmbedderMsg::AllowOpeningWebView(response_chan) => {
                    // Note: would be a place to handle pop-ups config.
                    // see Step 7 of #the-rules-for-choosing-a-browsing-context-given-a-browsing-context-name
                    if let Err(e) = response_chan.send(true) {
                        warn!("Failed to send AllowOpeningBrowser response: {}", e);
                    };
                },
                EmbedderMsg::WebViewOpened(new_webview_id) => {
                    self.webviews.insert(new_webview_id, WebView {});
                    self.creation_order.push(new_webview_id);
                    self.events
                        .push(EmbedderEvent::FocusWebView(new_webview_id));
                },
                EmbedderMsg::WebViewClosed(webview_id) => {
                    self.webviews.retain(|&id, _| id != webview_id);
                    self.creation_order.retain(|&id| id != webview_id);
                    self.focused_webview_id = None;
                    if let Some(&newest_webview_id) = self.creation_order.last() {
                        self.events
                            .push(EmbedderEvent::FocusWebView(newest_webview_id));
                    } else {
                        self.events.push(EmbedderEvent::Quit);
                    }
                },
                EmbedderMsg::WebViewFocused(webview_id) => {
                    self.focused_webview_id = Some(webview_id);
                },
                EmbedderMsg::WebViewBlurred => {
                    self.focused_webview_id = None;
                },
                EmbedderMsg::GetClipboardContents(sender) => {
                    let contents = self.callbacks.host_callbacks.get_clipboard_contents();
                    let _ = sender.send(contents.unwrap_or("".to_owned()));
                },
                EmbedderMsg::SetClipboardContents(text) => {
                    self.callbacks.host_callbacks.set_clipboard_contents(text);
                },
                EmbedderMsg::Shutdown => {
                    self.callbacks.host_callbacks.on_shutdown_complete();
                },
                EmbedderMsg::PromptPermission(prompt, sender) => {
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
                EmbedderMsg::ShowIME(kind, text, multiline, bounds) => {
                    self.callbacks
                        .host_callbacks
                        .on_ime_show(kind, text, multiline, bounds);
                },
                EmbedderMsg::HideIME => {
                    self.callbacks.host_callbacks.on_ime_hide();
                },
                EmbedderMsg::MediaSessionEvent(event) => {
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
                EmbedderMsg::Panic(reason, backtrace) => {
                    self.callbacks.host_callbacks.on_panic(reason, backtrace);
                },
                EmbedderMsg::ReadyToPresent(_webview_ids) => {
                    self.need_present = true;
                },
                EmbedderMsg::Status(..) |
                EmbedderMsg::SelectFiles(..) |
                EmbedderMsg::MoveTo(..) |
                EmbedderMsg::ResizeTo(..) |
                EmbedderMsg::Keyboard(..) |
                EmbedderMsg::SetCursor(..) |
                EmbedderMsg::NewFavicon(..) |
                EmbedderMsg::HeadParsed |
                EmbedderMsg::SetFullscreenState(..) |
                EmbedderMsg::ReportProfile(..) |
                EmbedderMsg::EventDelivered(..) |
                EmbedderMsg::PlayGamepadHapticEffect(..) |
                EmbedderMsg::StopGamepadHapticEffect(..) => {},
            }
        }

        if need_update {
            let _ = self.perform_updates();
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
    xr_discovery: Option<webxr::Discovery>,
    #[allow(unused)]
    gl: Rc<dyn gl::Gl>,
}

impl ServoEmbedderCallbacks {
    pub(super) fn new(
        waker: Box<dyn EventLoopWaker>,
        xr_discovery: Option<webxr::Discovery>,
        gl: Rc<dyn gl::Gl>,
    ) -> Self {
        Self {
            waker,
            xr_discovery,
            gl,
        }
    }
}

impl EmbedderMethods for ServoEmbedderCallbacks {
    fn create_event_loop_waker(&mut self) -> Box<dyn EventLoopWaker> {
        debug!("EmbedderMethods::create_event_loop_waker");
        self.waker.clone()
    }

    fn register_webxr(
        &mut self,
        registry: &mut webxr::MainThreadRegistry,
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
        EmbedderCoordinates {
            viewport: coords.viewport.to_box2d(),
            framebuffer: coords.framebuffer,
            window: (coords.viewport.size, Point2D::new(0, 0)),
            screen: coords.viewport.size,
            screen_avail: coords.viewport.size,
            hidpi_factor: Scale::new(self.density),
        }
    }

    fn set_animation_state(&self, state: AnimationState) {
        debug!("WindowMethods::set_animation_state: {:?}", state);
        self.host_callbacks
            .on_animating_changed(state == AnimationState::Animating);
    }

    fn rendering_context(&self) -> RenderingContext {
        self.rendering_context.clone()
    }
}
