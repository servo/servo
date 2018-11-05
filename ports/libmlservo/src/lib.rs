/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate egl;
#[macro_use] extern crate log;
extern crate servo;
extern crate smallvec;

use egl::egl::EGLContext;
use egl::egl::EGLDisplay;
use egl::egl::EGLSurface;
use egl::egl::MakeCurrent;
use egl::egl::SwapBuffers;
use egl::eglext::eglGetProcAddress;
use servo::BrowserId;
use servo::Servo;
use servo::compositing::windowing::AnimationState;
use servo::compositing::windowing::EmbedderCoordinates;
use servo::compositing::windowing::MouseWindowEvent;
use servo::compositing::windowing::WindowEvent;
use servo::compositing::windowing::WindowMethods;
use servo::embedder_traits::EmbedderMsg;
use servo::embedder_traits::EventLoopWaker;
use servo::embedder_traits::resources::Resource;
use servo::embedder_traits::resources::ResourceReaderMethods;
use servo::euclid::TypedPoint2D;
use servo::euclid::TypedRect;
use servo::euclid::TypedScale;
use servo::euclid::TypedSize2D;
use servo::gl;
use servo::gl::Gl;
use servo::gl::GlesFns;
use servo::msg::constellation_msg::TraversalDirection;
use servo::script_traits::MouseButton;
use servo::servo_url::ServoUrl;
use servo::webrender_api::DevicePoint;
use smallvec::SmallVec;
use std::ffi::CStr;
use std::ffi::CString;
use std::io::Write;
use std::os::raw::c_char;
use std::os::raw::c_void;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

#[repr(u32)]
pub enum MLLogLevel {
    Fatal = 0,
    Error = 1,
    Warning = 2,
    Info = 3,
    Debug = 4,
    Verbose = 5,
}

#[repr(transparent)]
pub struct MLLogger(extern "C" fn (MLLogLevel, *const c_char));

#[repr(transparent)]
pub struct MLHistoryUpdate(extern "C" fn (MLApp, bool, *const c_char, bool));

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct MLApp(*mut c_void);

const LOG_LEVEL: log::LevelFilter = log::LevelFilter::Info;

#[no_mangle]
pub unsafe extern "C" fn init_servo(ctxt: EGLContext,
                                    surf: EGLSurface,
                                    disp: EGLDisplay,
                                    app: MLApp,
                                    logger: MLLogger,
                                    history_update: MLHistoryUpdate,
                                    url: *const c_char,
                                    width: u32,
                                    height: u32,
                                    hidpi: f32) -> *mut ServoInstance
{
    // Servo initialization goes here!
    servo::embedder_traits::resources::set(Box::new(ResourceReaderInstance::new()));
    let _ = log::set_boxed_logger(Box::new(logger));
    log::set_max_level(LOG_LEVEL);
    let gl = GlesFns::load_with(|symbol| {
        let cstr = CString::new(symbol).expect("Failed to convert GL symbol to a char*");
        eglGetProcAddress(cstr.as_ptr() as _) as _
    });

    info!("OpenGL version {}", gl.get_string(gl::VERSION));
    let window = Rc::new(WindowInstance {
        ctxt: ctxt,
        surf: surf,
        disp: disp,
        gl: gl,
        width: width,
        height: height,
        hidpi: hidpi,
    });

    info!("Starting servo");
    let mut servo = Servo::new(window);
    let browser_id = BrowserId::new();

    let blank_url = ServoUrl::parse("about:blank").expect("Failed to parse about:blank!");
    let url = CStr::from_ptr(url).to_str().unwrap_or("about:blank");
    let url = ServoUrl::parse(url).unwrap_or(blank_url);
    servo.handle_events(vec![
        WindowEvent::NewBrowser(url, browser_id),
    ]);

    let result = Box::new(ServoInstance {
        app: app,
        browser_id: browser_id,
        history_update: history_update,
        servo: servo,
    });
    Box::into_raw(result)
}

#[no_mangle]
pub unsafe extern "C" fn heartbeat_servo(servo: *mut ServoInstance) {
    // Servo heartbeat goes here!
    if let Some(servo) = servo.as_mut() {
        servo.servo.handle_events(vec![]);
        for ((_browser_id, event)) in servo.servo.get_events() {
            match event {
                // Respond to any messages with a response channel
                // to avoid deadlocking the constellation
                EmbedderMsg::AllowNavigation(_url, sender) => {
                    let _ = sender.send(true);
                },
                EmbedderMsg::GetSelectedBluetoothDevice(_, sender) => {
                    let _ = sender.send(None);
                },
                EmbedderMsg::AllowUnload(sender) => {
                    let _ = sender.send(true);
                },
                EmbedderMsg::Alert(_, sender) => {
                    let _ = sender.send(());
                },
                EmbedderMsg::AllowOpeningBrowser(sender) => {
                    let _ = sender.send(false);
                },
                // Update the history UI
                EmbedderMsg::HistoryChanged(urls, index) => {
                    if let Some(url) = urls.get(index) {
                        if let Ok(cstr) = CString::new(url.as_str()) {
                            let can_go_back = index > 0;
                            let can_go_fwd = (index + 1) < urls.len();
                            (servo.history_update.0)(servo.app, can_go_back, cstr.as_ptr(), can_go_fwd);
                        }
                    }
                },
                // Ignore most messages for now
                EmbedderMsg::ChangePageTitle(..) |
                EmbedderMsg::BrowserCreated(..) |
                EmbedderMsg::LoadStart |
                EmbedderMsg::LoadComplete |
                EmbedderMsg::CloseBrowser |
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
                EmbedderMsg::Shutdown |
                EmbedderMsg::Panic(..) => {},
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cursor_servo(servo: *mut ServoInstance, x: f32, y: f32, trigger: bool) {
    // Servo was triggered
    if let Some(servo) = servo.as_mut() {
        let point = DevicePoint::new(x, y);
        let window_event = if trigger {
            WindowEvent::MouseWindowEventClass(MouseWindowEvent::Click(MouseButton::Left, point))
        } else {
            WindowEvent::MouseWindowMoveEventClass(point)
        };
        servo.servo.handle_events(vec![window_event]);
    }
}

#[no_mangle]
pub unsafe extern "C" fn traverse_servo(servo: *mut ServoInstance, delta: i32) {
    // Traverse the session history
    if let Some(servo) = servo.as_mut() {
        let window_event = if delta == 0 {
            WindowEvent::Reload(servo.browser_id)
        } else if delta < 0 {
            WindowEvent::Navigation(servo.browser_id, TraversalDirection::Back(-delta as usize))
        } else {
            WindowEvent::Navigation(servo.browser_id, TraversalDirection::Forward(delta as usize))
        };
        servo.servo.handle_events(vec![window_event]);
    }
}

#[no_mangle]
pub unsafe extern "C" fn navigate_servo(servo: *mut ServoInstance, text: *const c_char) {
    if let Some(servo) = servo.as_mut() {
        let text = CStr::from_ptr(text).to_str().expect("Failed to convert text to UTF-8");
        let url = ServoUrl::parse(text).unwrap_or_else(|_| {
            let mut search = ServoUrl::parse("http://google.com/search")
               .expect("Failed to parse search URL")
               .into_url();
            search.query_pairs_mut().append_pair("q", text);
            ServoUrl::from_url(search)
        });
        let window_event = WindowEvent::LoadUrl(servo.browser_id, url);
        servo.servo.handle_events(vec![window_event]);
    }
}

// Some magic numbers for shutdown
const SHUTDOWN_DURATION: Duration = Duration::from_secs(10);
const SHUTDOWN_POLL_INTERVAL: Duration = Duration::from_millis(100);

#[no_mangle]
pub unsafe extern "C" fn discard_servo(servo: *mut ServoInstance) {
    if let Some(servo) = servo.as_mut() {
        let mut servo = Box::from_raw(servo);
        let finish = Instant::now() + SHUTDOWN_DURATION;
        servo.servo.handle_events(vec![WindowEvent::Quit]);
        'outer: loop {
            for (_, msg) in servo.servo.get_events() {
                if let EmbedderMsg::Shutdown = msg {
                    break 'outer;
                }
            }
            if Instant::now() > finish {
               warn!("Incomplete shutdown.");
               break 'outer;
            }
            thread::sleep(SHUTDOWN_POLL_INTERVAL);
            servo.servo.handle_events(vec![]);
        }
        servo.servo.deinit();
    }
}

pub struct ServoInstance {
    app: MLApp,
    browser_id: BrowserId,
    history_update: MLHistoryUpdate,
    servo: Servo<WindowInstance>,
}

struct WindowInstance {
    ctxt: EGLContext,
    surf: EGLSurface,
    disp: EGLDisplay,
    gl: Rc<Gl>,
    width: u32,
    height: u32,
    hidpi: f32,
}

impl WindowMethods for WindowInstance {
    fn present(&self) {
        SwapBuffers(self.disp, self.surf);
    }

    fn prepare_for_composite(&self) -> bool {
        MakeCurrent(self.disp, self.surf, self.surf, self.ctxt);
        self.gl.viewport(0, 0, self.width as i32, self.height as i32);
        true
    }

    fn gl(&self) -> Rc<Gl> {
        self.gl.clone()
    }

    fn create_event_loop_waker(&self) -> Box<EventLoopWaker> {
        Box::new(EventLoopWakerInstance::new())
    }

    fn get_coordinates(&self) -> EmbedderCoordinates {
        EmbedderCoordinates {
            hidpi_factor: TypedScale::new(self.hidpi),
            screen: TypedSize2D::new(self.width, self.height),
            screen_avail: TypedSize2D::new(self.width, self.height),
            window: (TypedSize2D::new(self.width, self.height), TypedPoint2D::new(0, 0)),
            framebuffer: TypedSize2D::new(self.width, self.height),
            viewport: TypedRect::new(TypedPoint2D::new(0, 0), TypedSize2D::new(self.width, self.height)),
        }
    }

    fn set_animation_state(&self, _state: AnimationState) {
    }
}

struct EventLoopWakerInstance;

impl EventLoopWakerInstance {
    fn new() -> EventLoopWakerInstance {
        EventLoopWakerInstance
    }
}

impl EventLoopWaker for EventLoopWakerInstance {
    fn clone(&self) -> Box<EventLoopWaker + Send> {
        Box::new(EventLoopWakerInstance)
    }

    fn wake(&self) {
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
            Resource::Preferences => &include_bytes!("../../../resources/prefs.json")[..],
            Resource::HstsPreloadList => &include_bytes!("../../../resources/hsts_preload.json")[..],
            Resource::SSLCertificates => &include_bytes!("../../../resources/certs")[..],
            Resource::BadCertHTML => &include_bytes!("../../../resources/badcert.html")[..],
            Resource::NetErrorHTML => &include_bytes!("../../../resources/neterror.html")[..],
            Resource::UserAgentCSS => &include_bytes!("../../../resources/user-agent.css")[..],
            Resource::ServoCSS => &include_bytes!("../../../resources/servo.css")[..],
            Resource::PresentationalHintsCSS => &include_bytes!("../../../resources/presentational-hints.css")[..],
            Resource::QuirksModeCSS => &include_bytes!("../../../resources/quirks-mode.css")[..],
            Resource::RippyPNG => &include_bytes!("../../../resources/rippy.png")[..],
            Resource::DomainList => &include_bytes!("../../../resources/public_domains.txt")[..],
            Resource::BluetoothBlocklist => &include_bytes!("../../../resources/gatt_blocklist.txt")[..],
        })
    }

    fn sandbox_access_files(&self) -> Vec<PathBuf> {
        vec![]
    }

    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf> {
        vec![]
    }
}

impl log::Log for MLLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= LOG_LEVEL
    }

    fn log(&self, record: &log::Record) {
        let lvl = match record.level() {
            log::Level::Error => MLLogLevel::Error,
            log::Level::Warn => MLLogLevel::Warning,
            log::Level::Info => MLLogLevel::Info,
            log::Level::Debug => MLLogLevel::Debug,
            log::Level::Trace => MLLogLevel::Verbose,
        };
        let mut msg = SmallVec::<[c_char; 128]>::new();
        write!(msg, "{}\0", record.args());
        (self.0)(lvl, &msg[0] as *const _ as *const _);
    }

    fn flush(&self) {}
}
