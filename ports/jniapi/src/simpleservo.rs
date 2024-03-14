/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;
use std::os::raw::c_void;
use std::path::PathBuf;
use std::rc::Rc;

use getopts::Options;
use ipc_channel::ipc::IpcSender;
use log::{debug, info, warn};
use servo::compositing::windowing::{
    AnimationState, EmbedderCoordinates, EmbedderEvent, EmbedderMethods, MouseWindowEvent,
    WindowMethods,
};
use servo::compositing::CompositeTarget;
use servo::config::prefs::pref_map;
pub use servo::config::prefs::{add_user_prefs, PrefValue};
use servo::embedder_traits::resources::{self, Resource, ResourceReaderMethods};
pub use servo::embedder_traits::{
    ContextMenuResult, MediaSessionPlaybackState, PermissionPrompt, PermissionRequest, PromptResult,
};
use servo::embedder_traits::{
    EmbedderMsg, EmbedderProxy, MediaSessionEvent, PromptDefinition, PromptOrigin,
};
use servo::euclid::{Point2D, Rect, Scale, Size2D, Vector2D};
use servo::keyboard_types::{Key, KeyState, KeyboardEvent};
pub use servo::msg::constellation_msg::InputMethodType;
use servo::msg::constellation_msg::{TraversalDirection, WebViewId};
use servo::rendering_context::RenderingContext;
pub use servo::script_traits::{MediaSessionActionType, MouseButton};
use servo::script_traits::{TouchEventType, TouchId};
use servo::servo_config::{opts, pref};
use servo::servo_url::ServoUrl;
pub use servo::webrender_api::units::DeviceIntRect;
use servo::webrender_api::units::DevicePixel;
use servo::webrender_api::ScrollLocation;
use servo::{self, gl, Servo, TopLevelBrowsingContextId};
use surfman::{Connection, SurfaceType};

thread_local! {
    pub static SERVO: RefCell<Option<ServoGlue>> = RefCell::new(None);
}

/// The EventLoopWaker::wake function will be called from any thread.
/// It will be called to notify embedder that some events are available,
/// and that perform_updates need to be called
pub use servo::embedder_traits::EventLoopWaker;

pub struct InitOptions {
    pub args: Vec<String>,
    pub coordinates: Coordinates,
    pub density: f32,
    pub xr_discovery: Option<webxr::Discovery>,
    pub surfman_integration: SurfmanIntegration,
    pub prefs: Option<HashMap<String, PrefValue>>,
}

/// Controls how this embedding's rendering will integrate with the embedder.
pub enum SurfmanIntegration {
    /// Render directly to a provided native widget (see surfman::NativeWidget).
    Widget(*mut c_void),
    /// Render to an offscreen surface.
    Surface,
}

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

/// Callbacks. Implemented by embedder. Called by Servo.
pub trait HostTrait {
    /// Show alert.
    fn prompt_alert(&self, msg: String, trusted: bool);
    /// Ask Yes/No question.
    fn prompt_yes_no(&self, msg: String, trusted: bool) -> PromptResult;
    /// Ask Ok/Cancel question.
    fn prompt_ok_cancel(&self, msg: String, trusted: bool) -> PromptResult;
    /// Ask for string
    fn prompt_input(&self, msg: String, default: String, trusted: bool) -> Option<String>;
    /// Show context menu
    fn show_context_menu(&self, title: Option<String>, items: Vec<String>);
    /// Page starts loading.
    /// "Reload button" should be disabled.
    /// "Stop button" should be enabled.
    /// Throbber starts spinning.
    fn on_load_started(&self);
    /// Page has loaded.
    /// "Reload button" should be enabled.
    /// "Stop button" should be disabled.
    /// Throbber stops spinning.
    fn on_load_ended(&self);
    /// Page title has changed.
    fn on_title_changed(&self, title: Option<String>);
    /// Allow Navigation.
    fn on_allow_navigation(&self, url: String) -> bool;
    /// Page URL has changed.
    fn on_url_changed(&self, url: String);
    /// Back/forward state has changed.
    /// Back/forward buttons need to be disabled/enabled.
    fn on_history_changed(&self, can_go_back: bool, can_go_forward: bool);
    /// Page animation state has changed. If animating, it's recommended
    /// that the embedder doesn't wait for the wake function to be called
    /// to call perform_updates. Usually, it means doing:
    /// while true { servo.perform_updates() }. This will end up calling flush
    /// which will call swap_buffer which will be blocking long enough to limit
    /// drawing at 60 FPS.
    /// If not animating, call perform_updates only when needed (when the embedder
    /// has events for Servo, or Servo has woken up the embedder event loop via
    /// EventLoopWaker).
    fn on_animating_changed(&self, animating: bool);
    /// Servo finished shutting down.
    fn on_shutdown_complete(&self);
    /// A text input is focused.
    fn on_ime_show(
        &self,
        input_type: InputMethodType,
        text: Option<(String, i32)>,
        multiline: bool,
        bounds: DeviceIntRect,
    );
    /// Input lost focus
    fn on_ime_hide(&self);
    /// Gets sytem clipboard contents.
    fn get_clipboard_contents(&self) -> Option<String>;
    /// Sets system clipboard contents.
    fn set_clipboard_contents(&self, contents: String);
    /// Called when we get the media session metadata/
    fn on_media_session_metadata(&self, title: String, artist: String, album: String);
    /// Called when the media session playback state changes.
    fn on_media_session_playback_state_change(&self, state: MediaSessionPlaybackState);
    /// Called when the media session position state is set.
    fn on_media_session_set_position_state(&self, duration: f64, position: f64, playback_rate: f64);
    /// Called when devtools server is started
    fn on_devtools_started(&self, port: Result<u16, ()>, token: String);
    /// Called when we get a panic message from constellation
    fn on_panic(&self, reason: String, backtrace: Option<String>);
}

pub struct ServoGlue {
    rendering_context: RenderingContext,
    servo: Servo<ServoWindowCallbacks>,
    batch_mode: bool,
    callbacks: Rc<ServoWindowCallbacks>,
    events: Vec<EmbedderEvent>,
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

#[derive(Debug)]
pub struct WebView {}

pub fn servo_version() -> String {
    format!(
        "Servo {}-{}",
        env!("CARGO_PKG_VERSION"),
        env!("VERGEN_GIT_SHA")
    )
}

/// Test if a url is valid.
pub fn is_uri_valid(url: &str) -> bool {
    info!("load_uri: {}", url);
    ServoUrl::parse(url).is_ok()
}

/// Retrieve a snapshot of the current preferences
pub fn get_prefs() -> HashMap<String, (PrefValue, bool)> {
    pref_map()
        .iter()
        .map(|(key, value)| {
            let is_default = pref_map().is_default(&key).unwrap();
            (key, (value, is_default))
        })
        .collect()
}

/// Retrieve a preference.
pub fn get_pref(key: &str) -> (PrefValue, bool) {
    if let Ok(is_default) = pref_map().is_default(&key) {
        (pref_map().get(key), is_default)
    } else {
        (PrefValue::Missing, false)
    }
}

/// Restore a preference to its default value.
pub fn reset_pref(key: &str) -> bool {
    pref_map().reset(key).is_ok()
}

/// Restore all the preferences to their default values.
pub fn reset_all_prefs() {
    pref_map().reset_all();
}

/// Change the value of a preference.
pub fn set_pref(key: &str, val: PrefValue) -> Result<(), &'static str> {
    pref_map()
        .set(key, val)
        .map(|_| ())
        .map_err(|_| "Pref set failed")
}

/// Initialize Servo. At that point, we need a valid GL context.
/// In the future, this will be done in multiple steps.
pub fn init(
    mut init_opts: InitOptions,
    gl: Rc<dyn gl::Gl>,
    waker: Box<dyn EventLoopWaker>,
    callbacks: Box<dyn HostTrait>,
) -> Result<(), &'static str> {
    resources::set(Box::new(ResourceReaderInstance::new()));

    if let Some(prefs) = init_opts.prefs {
        add_user_prefs(prefs);
    }

    let mut args = mem::replace(&mut init_opts.args, vec![]);
    // opts::from_cmdline_args expects the first argument to be the binary name.
    args.insert(0, "servo".to_string());
    opts::from_cmdline_args(Options::new(), &args);

    let pref_url = ServoUrl::parse(&pref!(shell.homepage)).ok();
    let blank_url = ServoUrl::parse("about:blank").ok();

    let url = pref_url.or(blank_url).unwrap();

    gl.clear_color(1.0, 1.0, 1.0, 1.0);
    gl.clear(gl::COLOR_BUFFER_BIT);
    gl.finish();

    // Initialize surfman
    let connection = Connection::new().or(Err("Failed to create connection"))?;
    let adapter = connection
        .create_adapter()
        .or(Err("Failed to create adapter"))?;
    let surface_type = match init_opts.surfman_integration {
        SurfmanIntegration::Widget(native_widget) => {
            let native_widget = unsafe {
                connection.create_native_widget_from_ptr(
                    native_widget,
                    init_opts.coordinates.framebuffer.to_untyped(),
                )
            };
            SurfaceType::Widget { native_widget }
        },
        SurfmanIntegration::Surface => {
            let size = init_opts.coordinates.framebuffer.to_untyped();
            SurfaceType::Generic { size }
        },
    };
    let rendering_context = RenderingContext::create(&connection, &adapter, surface_type)
        .or(Err("Failed to create surface manager"))?;

    let window_callbacks = Rc::new(ServoWindowCallbacks {
        host_callbacks: callbacks,
        coordinates: RefCell::new(init_opts.coordinates),
        density: init_opts.density,
        rendering_context: rendering_context.clone(),
    });

    let embedder_callbacks = Box::new(ServoEmbedderCallbacks {
        xr_discovery: init_opts.xr_discovery,
        waker,
        gl: gl.clone(),
    });

    let servo = Servo::new(
        embedder_callbacks,
        window_callbacks.clone(),
        None,
        CompositeTarget::Window,
    );

    SERVO.with(|s| {
        let mut servo_glue = ServoGlue {
            rendering_context,
            servo: servo.servo,
            batch_mode: false,
            callbacks: window_callbacks,
            events: vec![],
            context_menu_sender: None,
            webviews: HashMap::default(),
            creation_order: vec![],
            focused_webview_id: None,
        };
        let _ = servo_glue.process_event(EmbedderEvent::NewWebView(url, servo.browser_id));
        *s.borrow_mut() = Some(servo_glue);
    });

    Ok(())
}

pub fn deinit() {
    SERVO.with(|s| s.replace(None).unwrap().deinit());
}

impl ServoGlue {
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

    /// Load an URL. This needs to be a valid url.
    pub fn load_uri(&mut self, url: &str) -> Result<(), &'static str> {
        info!("load_uri: {}", url);
        ServoUrl::parse(url)
            .map_err(|_| "Can't parse URL")
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
        self.process_event(EmbedderEvent::Resize)
    }

    /// Start scrolling.
    /// x/y are scroll coordinates.
    /// dx/dy are scroll deltas.
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

    pub fn change_visibility(&mut self, visible: bool) -> Result<(), &'static str> {
        info!("change_visibility");
        if let Ok(id) = self.get_browser_id() {
            let event = EmbedderEvent::WebViewVisibilityChanged(id, visible);
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

    fn process_event(&mut self, event: EmbedderEvent) -> Result<(), &'static str> {
        self.events.push(event);
        if !self.batch_mode {
            self.perform_updates()
        } else {
            Ok(())
        }
    }

    fn handle_servo_events(&mut self) -> Result<(), &'static str> {
        let mut need_update = false;
        let mut need_present = false;
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
                EmbedderMsg::ReadyToPresent => {
                    need_present = true;
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
                EmbedderMsg::EventDelivered(..) => {},
            }
        }

        if need_update {
            let _ = self.perform_updates();
        }
        if need_present {
            self.servo.present();
        }
        Ok(())
    }
}

struct ServoEmbedderCallbacks {
    waker: Box<dyn EventLoopWaker>,
    xr_discovery: Option<webxr::Discovery>,
    #[allow(unused)]
    gl: Rc<dyn gl::Gl>,
}

struct ServoWindowCallbacks {
    host_callbacks: Box<dyn HostTrait>,
    coordinates: RefCell<Coordinates>,
    density: f32,
    rendering_context: RenderingContext,
}

impl EmbedderMethods for ServoEmbedderCallbacks {
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

    fn create_event_loop_waker(&mut self) -> Box<dyn EventLoopWaker> {
        debug!("EmbedderMethods::create_event_loop_waker");
        self.waker.clone()
    }
}

impl WindowMethods for ServoWindowCallbacks {
    fn rendering_context(&self) -> RenderingContext {
        self.rendering_context.clone()
    }

    fn set_animation_state(&self, state: AnimationState) {
        debug!("WindowMethods::set_animation_state: {:?}", state);
        self.host_callbacks
            .on_animating_changed(state == AnimationState::Animating);
    }

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
}

struct ResourceReaderInstance;

impl ResourceReaderInstance {
    fn new() -> ResourceReaderInstance {
        ResourceReaderInstance
    }
}

impl ResourceReaderMethods for ResourceReaderInstance {
    fn read(&self, res: Resource) -> Vec<u8> {
        Vec::from(match res {
            Resource::Preferences => &include_bytes!(concat!(env!("OUT_DIR"), "/prefs.json"))[..],
            Resource::HstsPreloadList => {
                &include_bytes!("../../../resources/hsts_preload.json")[..]
            },
            Resource::BadCertHTML => &include_bytes!("../../../resources/badcert.html")[..],
            Resource::NetErrorHTML => &include_bytes!("../../../resources/neterror.html")[..],
            Resource::UserAgentCSS => &include_bytes!("../../../resources/user-agent.css")[..],
            Resource::ServoCSS => &include_bytes!("../../../resources/servo.css")[..],
            Resource::PresentationalHintsCSS => {
                &include_bytes!("../../../resources/presentational-hints.css")[..]
            },
            Resource::QuirksModeCSS => &include_bytes!("../../../resources/quirks-mode.css")[..],
            Resource::RippyPNG => &include_bytes!("../../../resources/rippy.png")[..],
            Resource::DomainList => &include_bytes!("../../../resources/public_domains.txt")[..],
            Resource::BluetoothBlocklist => {
                &include_bytes!("../../../resources/gatt_blocklist.txt")[..]
            },
            Resource::MediaControlsCSS => {
                &include_bytes!("../../../resources/media-controls.css")[..]
            },
            Resource::MediaControlsJS => {
                &include_bytes!("../../../resources/media-controls.js")[..]
            },
            Resource::CrashHTML => &include_bytes!("../../../resources/crash.html")[..],
        })
    }

    fn sandbox_access_files(&self) -> Vec<PathBuf> {
        vec![]
    }

    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf> {
        vec![]
    }
}
