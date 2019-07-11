/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate log;

pub mod gl_glue;

pub use servo::script_traits::MouseButton;

use servo::compositing::windowing::{
    AnimationState, EmbedderCoordinates, EmbedderMethods, MouseWindowEvent, WindowEvent,
    WindowMethods,
};
use servo::embedder_traits::resources::{self, Resource, ResourceReaderMethods};
use servo::embedder_traits::EmbedderMsg;
use servo::euclid::{TypedPoint2D, TypedRect, TypedScale, TypedSize2D, TypedVector2D};
use servo::keyboard_types::{Key, KeyState, KeyboardEvent};
use servo::msg::constellation_msg::TraversalDirection;
use servo::script_traits::{TouchEventType, TouchId};
use servo::servo_config::opts;
use servo::servo_config::{pref, set_pref};
use servo::servo_url::ServoUrl;
use servo::webrender_api::units::DevicePixel;
use servo::webrender_api::ScrollLocation;
use servo::webvr::{VRExternalShmemPtr, VRMainThreadHeartbeat, VRService, VRServiceManager};
use servo::{self, gl, BrowserId, Servo};
use servo_media::player::context as MediaPlayerContext;
use std::cell::RefCell;
use std::mem;
use std::os::raw::c_void;
use std::path::PathBuf;
use std::rc::Rc;

thread_local! {
    pub static SERVO: RefCell<Option<ServoGlue>> = RefCell::new(None);
}

/// The EventLoopWaker::wake function will be called from any thread.
/// It will be called to notify embedder that some events are available,
/// and that perform_updates need to be called
pub use servo::embedder_traits::EventLoopWaker;

pub struct InitOptions {
    pub args: Vec<String>,
    pub url: Option<String>,
    pub coordinates: Coordinates,
    pub density: f32,
    pub vr_init: VRInitOptions,
    pub enable_subpixel_text_antialiasing: bool,
    pub gl_context_pointer: Option<*const c_void>,
    pub native_display_pointer: Option<*const c_void>,
}

pub enum VRInitOptions {
    None,
    VRExternal(*mut c_void),
    VRService(Box<dyn VRService>, Box<dyn VRMainThreadHeartbeat>),
}

#[derive(Clone, Debug)]
pub struct Coordinates {
    pub viewport: TypedRect<i32, DevicePixel>,
    pub framebuffer: TypedSize2D<i32, DevicePixel>,
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
            viewport: TypedRect::new(TypedPoint2D::new(x, y), TypedSize2D::new(width, height)),
            framebuffer: TypedSize2D::new(fb_width, fb_height),
        }
    }
}

/// Callbacks. Implemented by embedder. Called by Servo.
pub trait HostTrait {
    /// Will be called from the thread used for the init call.
    /// Will be called when the GL buffer has been updated.
    fn flush(&self);
    /// Will be called before drawing.
    /// Time to make the targetted GL context current.
    fn make_current(&self);
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
    fn on_title_changed(&self, title: String);
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
    fn on_ime_state_changed(&self, show: bool);
    /// Gets sytem clipboard contents
    fn get_clipboard_contents(&self) -> Option<String>;
    /// Sets system clipboard contents
    fn set_clipboard_contents(&self, contents: String);
}

pub struct ServoGlue {
    servo: Servo<ServoWindowCallbacks>,
    batch_mode: bool,
    callbacks: Rc<ServoWindowCallbacks>,
    /// id of the top level browsing context. It is unique as tabs
    /// are not supported yet. None until created.
    browser_id: Option<BrowserId>,
    // A rudimentary stack of "tabs".
    // EmbedderMsg::BrowserCreated will push onto it.
    // EmbedderMsg::CloseBrowser will pop from it,
    // and exit if it is empty afterwards.
    browsers: Vec<BrowserId>,
    events: Vec<WindowEvent>,
    current_url: Option<ServoUrl>,
}

pub fn servo_version() -> String {
    servo::config::servo_version()
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

    let mut args = mem::replace(&mut init_opts.args, vec![]);
    if !args.is_empty() {
        // opts::from_cmdline_args expects the first argument to be the binary name.
        args.insert(0, "servo".to_string());

        set_pref!(
            gfx.subpixel_text_antialiasing.enabled,
            init_opts.enable_subpixel_text_antialiasing
        );
        opts::from_cmdline_args(&args);
    }

    let embedder_url = init_opts.url.as_ref().and_then(|s| ServoUrl::parse(s).ok());
    let cmdline_url = opts::get().url.clone();
    let pref_url = ServoUrl::parse(&pref!(shell.homepage)).ok();
    let blank_url = ServoUrl::parse("about:blank").ok();

    let url = embedder_url
        .or(cmdline_url)
        .or(pref_url)
        .or(blank_url)
        .unwrap();

    gl.clear_color(1.0, 1.0, 1.0, 1.0);
    gl.clear(gl::COLOR_BUFFER_BIT);
    gl.finish();

    let window_callbacks = Rc::new(ServoWindowCallbacks {
        gl: gl.clone(),
        host_callbacks: callbacks,
        coordinates: RefCell::new(init_opts.coordinates),
        density: init_opts.density,
        gl_context_pointer: init_opts.gl_context_pointer,
        native_display_pointer: init_opts.native_display_pointer,
    });

    let embedder_callbacks = Box::new(ServoEmbedderCallbacks {
        vr_init: init_opts.vr_init,
        waker,
    });

    let servo = Servo::new(embedder_callbacks, window_callbacks.clone());

    SERVO.with(|s| {
        let mut servo_glue = ServoGlue {
            servo,
            batch_mode: false,
            callbacks: window_callbacks,
            browser_id: None,
            browsers: vec![],
            events: vec![],
            current_url: Some(url.clone()),
        };
        let browser_id = BrowserId::new();
        let _ = servo_glue.process_event(WindowEvent::NewBrowser(url, browser_id));
        *s.borrow_mut() = Some(servo_glue);
    });

    Ok(())
}

pub fn deinit() {
    SERVO.with(|s| s.replace(None).unwrap().deinit());
}

impl ServoGlue {
    fn get_browser_id(&self) -> Result<BrowserId, &'static str> {
        let browser_id = match self.browser_id {
            Some(id) => id,
            None => return Err("No BrowserId set yet."),
        };
        Ok(browser_id)
    }

    /// Request shutdown. Will call on_shutdown_complete.
    pub fn request_shutdown(&mut self) -> Result<(), &'static str> {
        self.process_event(WindowEvent::Quit)
    }

    /// Call after on_shutdown_complete
    pub fn deinit(self) {
        self.servo.deinit();
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
                let event = WindowEvent::LoadUrl(browser_id, url);
                self.process_event(event)
            })
    }

    /// Reload the page.
    pub fn reload(&mut self) -> Result<(), &'static str> {
        info!("reload");
        let browser_id = self.get_browser_id()?;
        let event = WindowEvent::Reload(browser_id);
        self.process_event(event)
    }

    /// Redraw the page.
    pub fn refresh(&mut self) -> Result<(), &'static str> {
        info!("refresh");
        self.process_event(WindowEvent::Refresh)
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
        let event = WindowEvent::Navigation(browser_id, TraversalDirection::Back(1));
        self.process_event(event)
    }

    /// Go forward in history.
    pub fn go_forward(&mut self) -> Result<(), &'static str> {
        info!("go_forward");
        let browser_id = self.get_browser_id()?;
        let event = WindowEvent::Navigation(browser_id, TraversalDirection::Forward(1));
        self.process_event(event)
    }

    /// Let Servo know that the window has been resized.
    pub fn resize(&mut self, coordinates: Coordinates) -> Result<(), &'static str> {
        info!("resize");
        *self.callbacks.coordinates.borrow_mut() = coordinates;
        self.process_event(WindowEvent::Resize)
    }

    /// Start scrolling.
    /// x/y are scroll coordinates.
    /// dx/dy are scroll deltas.
    pub fn scroll_start(&mut self, dx: f32, dy: f32, x: i32, y: i32) -> Result<(), &'static str> {
        let delta = TypedVector2D::new(dx, dy);
        let scroll_location = ScrollLocation::Delta(delta);
        let event = WindowEvent::Scroll(
            scroll_location,
            TypedPoint2D::new(x, y),
            TouchEventType::Down,
        );
        self.process_event(event)
    }

    /// Scroll.
    /// x/y are scroll coordinates.
    /// dx/dy are scroll deltas.
    pub fn scroll(&mut self, dx: f32, dy: f32, x: i32, y: i32) -> Result<(), &'static str> {
        let delta = TypedVector2D::new(dx, dy);
        let scroll_location = ScrollLocation::Delta(delta);
        let event = WindowEvent::Scroll(
            scroll_location,
            TypedPoint2D::new(x, y),
            TouchEventType::Move,
        );
        self.process_event(event)
    }

    /// End scrolling.
    /// x/y are scroll coordinates.
    /// dx/dy are scroll deltas.
    pub fn scroll_end(&mut self, dx: f32, dy: f32, x: i32, y: i32) -> Result<(), &'static str> {
        let delta = TypedVector2D::new(dx, dy);
        let scroll_location = ScrollLocation::Delta(delta);
        let event =
            WindowEvent::Scroll(scroll_location, TypedPoint2D::new(x, y), TouchEventType::Up);
        self.process_event(event)
    }

    /// Touch event: press down
    pub fn touch_down(&mut self, x: f32, y: f32, pointer_id: i32) -> Result<(), &'static str> {
        let event = WindowEvent::Touch(
            TouchEventType::Down,
            TouchId(pointer_id),
            TypedPoint2D::new(x as f32, y as f32),
        );
        self.process_event(event)
    }

    /// Touch event: move touching finger
    pub fn touch_move(&mut self, x: f32, y: f32, pointer_id: i32) -> Result<(), &'static str> {
        let event = WindowEvent::Touch(
            TouchEventType::Move,
            TouchId(pointer_id),
            TypedPoint2D::new(x as f32, y as f32),
        );
        self.process_event(event)
    }

    /// Touch event: Lift touching finger
    pub fn touch_up(&mut self, x: f32, y: f32, pointer_id: i32) -> Result<(), &'static str> {
        let event = WindowEvent::Touch(
            TouchEventType::Up,
            TouchId(pointer_id),
            TypedPoint2D::new(x as f32, y as f32),
        );
        self.process_event(event)
    }

    /// Cancel touch event
    pub fn touch_cancel(&mut self, x: f32, y: f32, pointer_id: i32) -> Result<(), &'static str> {
        let event = WindowEvent::Touch(
            TouchEventType::Cancel,
            TouchId(pointer_id),
            TypedPoint2D::new(x as f32, y as f32),
        );
        self.process_event(event)
    }

    /// Register a mouse movement.
    pub fn move_mouse(&mut self, x: f32, y: f32) -> Result<(), &'static str> {
        let point = TypedPoint2D::new(x, y);
        let event = WindowEvent::MouseWindowMoveEventClass(point);
        self.process_event(event)
    }

    /// Register a mouse button press.
    pub fn mouse_down(&mut self, x: f32, y: f32, button: MouseButton) -> Result<(), &'static str> {
        let point = TypedPoint2D::new(x, y);
        let event = WindowEvent::MouseWindowEventClass(MouseWindowEvent::MouseDown(button, point));
        self.process_event(event)
    }

    /// Register a mouse button release.
    pub fn mouse_up(&mut self, x: f32, y: f32, button: MouseButton) -> Result<(), &'static str> {
        let point = TypedPoint2D::new(x, y);
        let event = WindowEvent::MouseWindowEventClass(MouseWindowEvent::MouseUp(button, point));
        self.process_event(event)
    }

    /// Start pinchzoom.
    /// x/y are pinch origin coordinates.
    pub fn pinchzoom_start(&mut self, factor: f32, _x: u32, _y: u32) -> Result<(), &'static str> {
        self.process_event(WindowEvent::PinchZoom(factor))
    }

    /// Pinchzoom.
    /// x/y are pinch origin coordinates.
    pub fn pinchzoom(&mut self, factor: f32, _x: u32, _y: u32) -> Result<(), &'static str> {
        self.process_event(WindowEvent::PinchZoom(factor))
    }

    /// End pinchzoom.
    /// x/y are pinch origin coordinates.
    pub fn pinchzoom_end(&mut self, factor: f32, _x: u32, _y: u32) -> Result<(), &'static str> {
        self.process_event(WindowEvent::PinchZoom(factor))
    }

    /// Perform a click.
    pub fn click(&mut self, x: f32, y: f32) -> Result<(), &'static str> {
        let mouse_event = MouseWindowEvent::Click(MouseButton::Left, TypedPoint2D::new(x, y));
        let event = WindowEvent::MouseWindowEventClass(mouse_event);
        self.process_event(event)
    }

    pub fn key_down(&mut self, key: Key) -> Result<(), &'static str> {
        let key_event = KeyboardEvent {
            state: KeyState::Down,
            key,
            ..KeyboardEvent::default()
        };
        self.process_event(WindowEvent::Keyboard(key_event))
    }

    pub fn key_up(&mut self, key: Key) -> Result<(), &'static str> {
        let key_event = KeyboardEvent {
            state: KeyState::Up,
            key,
            ..KeyboardEvent::default()
        };
        self.process_event(WindowEvent::Keyboard(key_event))
    }

    fn process_event(&mut self, event: WindowEvent) -> Result<(), &'static str> {
        self.events.push(event);
        if !self.batch_mode {
            self.perform_updates()
        } else {
            Ok(())
        }
    }

    fn handle_servo_events(&mut self) -> Result<(), &'static str> {
        for (browser_id, event) in self.servo.get_events() {
            match event {
                EmbedderMsg::ChangePageTitle(title) => {
                    let fallback_title: String = if let Some(ref current_url) = self.current_url {
                        current_url.to_string()
                    } else {
                        String::from("Untitled")
                    };
                    let title = match title {
                        Some(ref title) if title.len() > 0 => &**title,
                        _ => &fallback_title,
                    };
                    let title = format!("{} - Servo", title);
                    self.callbacks.host_callbacks.on_title_changed(title);
                },
                EmbedderMsg::AllowNavigationRequest(pipeline_id, url) => {
                    if let Some(_browser_id) = browser_id {
                        let data: bool = self
                            .callbacks
                            .host_callbacks
                            .on_allow_navigation(url.to_string());
                        let window_event = WindowEvent::AllowNavigationResponse(pipeline_id, data);
                        let _ = self.process_event(window_event);
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
                    self.current_url = Some(entries[current].clone());
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
                EmbedderMsg::Alert(message, sender) => {
                    info!("Alert: {}", message);
                    let _ = sender.send(());
                },
                EmbedderMsg::AllowOpeningBrowser(response_chan) => {
                    // Note: would be a place to handle pop-ups config.
                    // see Step 7 of #the-rules-for-choosing-a-browsing-context-given-a-browsing-context-name
                    if let Err(e) = response_chan.send(true) {
                        warn!("Failed to send AllowOpeningBrowser response: {}", e);
                    };
                },
                EmbedderMsg::BrowserCreated(new_browser_id) => {
                    // TODO: properly handle a new "tab"
                    self.browsers.push(new_browser_id);
                    if self.browser_id.is_none() {
                        self.browser_id = Some(new_browser_id);
                    }
                    self.events.push(WindowEvent::SelectBrowser(new_browser_id));
                },
                EmbedderMsg::GetClipboardContents(sender) => {
                    let contents = self.callbacks.host_callbacks.get_clipboard_contents();
                    let _ = sender.send(contents.unwrap_or("".to_owned()));
                },
                EmbedderMsg::SetClipboardContents(text) => {
                    self.callbacks.host_callbacks.set_clipboard_contents(text);
                },
                EmbedderMsg::CloseBrowser => {
                    // TODO: close the appropriate "tab".
                    let _ = self.browsers.pop();
                    if let Some(prev_browser_id) = self.browsers.last() {
                        self.browser_id = Some(*prev_browser_id);
                        self.events
                            .push(WindowEvent::SelectBrowser(*prev_browser_id));
                    } else {
                        self.events.push(WindowEvent::Quit);
                    }
                },
                EmbedderMsg::Shutdown => {
                    self.callbacks.host_callbacks.on_shutdown_complete();
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
                EmbedderMsg::ShowIME(..) |
                EmbedderMsg::HideIME |
                EmbedderMsg::Panic(..) |
                EmbedderMsg::ReportProfile(..) => {},
            }
        }
        Ok(())
    }
}

struct ServoEmbedderCallbacks {
    waker: Box<dyn EventLoopWaker>,
    vr_init: VRInitOptions,
}

struct ServoWindowCallbacks {
    gl: Rc<dyn gl::Gl>,
    host_callbacks: Box<dyn HostTrait>,
    coordinates: RefCell<Coordinates>,
    density: f32,
    gl_context_pointer: Option<*const c_void>,
    native_display_pointer: Option<*const c_void>,
}

impl EmbedderMethods for ServoEmbedderCallbacks {
    fn register_vr_services(
        &mut self,
        services: &mut VRServiceManager,
        heartbeats: &mut Vec<Box<dyn VRMainThreadHeartbeat>>,
    ) {
        debug!("EmbedderMethods::register_vrexternal");
        match mem::replace(&mut self.vr_init, VRInitOptions::None) {
            VRInitOptions::None => {},
            VRInitOptions::VRExternal(ptr) => {
                services.register_vrexternal(VRExternalShmemPtr::new(ptr));
            },
            VRInitOptions::VRService(service, heartbeat) => {
                services.register(service);
                heartbeats.push(heartbeat);
            },
        }
    }

    fn create_event_loop_waker(&mut self) -> Box<dyn EventLoopWaker> {
        debug!("EmbedderMethods::create_event_loop_waker");
        self.waker.clone()
    }
}

impl WindowMethods for ServoWindowCallbacks {
    fn prepare_for_composite(&self) {
        debug!("WindowMethods::prepare_for_composite");
        self.host_callbacks.make_current();
    }

    fn present(&self) {
        debug!("WindowMethods::present");
        self.host_callbacks.flush();
    }

    fn gl(&self) -> Rc<dyn gl::Gl> {
        debug!("WindowMethods::gl");
        self.gl.clone()
    }

    fn set_animation_state(&self, state: AnimationState) {
        debug!("WindowMethods::set_animation_state: {:?}", state);
        self.host_callbacks
            .on_animating_changed(state == AnimationState::Animating);
    }

    fn get_coordinates(&self) -> EmbedderCoordinates {
        let coords = self.coordinates.borrow();
        EmbedderCoordinates {
            viewport: coords.viewport,
            framebuffer: coords.framebuffer,
            window: (coords.viewport.size, TypedPoint2D::new(0, 0)),
            screen: coords.viewport.size,
            screen_avail: coords.viewport.size,
            hidpi_factor: TypedScale::new(self.density),
        }
    }

    fn get_gl_context(&self) -> MediaPlayerContext::GlContext {
        match self.gl_context_pointer {
            Some(context) => MediaPlayerContext::GlContext::Egl(context as usize),
            None => MediaPlayerContext::GlContext::Unknown,
        }
    }

    fn get_native_display(&self) -> MediaPlayerContext::NativeDisplay {
        match self.native_display_pointer {
            Some(display) => MediaPlayerContext::NativeDisplay::Egl(display as usize),
            None => MediaPlayerContext::NativeDisplay::Unknown,
        }
    }

    fn get_gl_api(&self) -> MediaPlayerContext::GlApi {
        MediaPlayerContext::GlApi::Gles2
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
            Resource::Preferences => &include_bytes!("../../../../resources/prefs.json")[..],
            Resource::HstsPreloadList => {
                &include_bytes!("../../../../resources/hsts_preload.json")[..]
            },
            Resource::SSLCertificates => &include_bytes!("../../../../resources/certs")[..],
            Resource::BadCertHTML => &include_bytes!("../../../../resources/badcert.html")[..],
            Resource::NetErrorHTML => &include_bytes!("../../../../resources/neterror.html")[..],
            Resource::UserAgentCSS => &include_bytes!("../../../../resources/user-agent.css")[..],
            Resource::ServoCSS => &include_bytes!("../../../../resources/servo.css")[..],
            Resource::PresentationalHintsCSS => {
                &include_bytes!("../../../../resources/presentational-hints.css")[..]
            },
            Resource::QuirksModeCSS => &include_bytes!("../../../../resources/quirks-mode.css")[..],
            Resource::RippyPNG => &include_bytes!("../../../../resources/rippy.png")[..],
            Resource::DomainList => &include_bytes!("../../../../resources/public_domains.txt")[..],
            Resource::BluetoothBlocklist => {
                &include_bytes!("../../../../resources/gatt_blocklist.txt")[..]
            },
        })
    }

    fn sandbox_access_files(&self) -> Vec<PathBuf> {
        vec![]
    }

    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf> {
        vec![]
    }
}
