/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use egl::egl::EGLContext;
use egl::egl::EGLDisplay;
use egl::egl::EGLSurface;
use egl::egl::MakeCurrent;
use egl::egl::SwapBuffers;
use log::info;
use log::warn;
use servo::euclid::TypedScale;
use servo::keyboard_types::Key;
use servo::servo_url::ServoUrl;
use servo::webrender_api::DevicePixel;
use servo::webrender_api::DevicePoint;
use servo::webrender_api::LayoutPixel;
use simpleservo::{
    self, deinit, gl_glue, Coordinates, EventLoopWaker, HostTrait, InitOptions, MouseButton,
    ServoGlue, SERVO,
};
use smallvec::SmallVec;
use std::cell::Cell;
use std::ffi::CStr;
use std::ffi::CString;
use std::io::Write;
use std::os::raw::c_char;
use std::os::raw::c_void;
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

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum MLKeyType {
    kNone,
    kCharacter,
    kBackspace,
    kShift,
    kSpeechToText,
    kPageEmoji,
    kPageLowerLetters,
    kPageNumericSymbols,
    kCancel,
    kSubmit,
    kPrevious,
    kNext,
    kClear,
    kClose,
    kEnter,
    kCustom1,
    kCustom2,
    kCustom3,
    kCustom4,
    kCustom5,
}

#[repr(transparent)]
pub struct MLLogger(extern "C" fn(MLLogLevel, *const c_char));

#[repr(transparent)]
pub struct MLHistoryUpdate(extern "C" fn(MLApp, bool, bool));

#[repr(transparent)]
pub struct MLURLUpdate(extern "C" fn(MLApp, *const c_char));

#[repr(transparent)]
pub struct MLKeyboard(extern "C" fn(MLApp, bool));

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct MLApp(*mut c_void);

const LOG_LEVEL: log::LevelFilter = log::LevelFilter::Info;

fn call<F, T>(f: F) -> Result<T, &'static str>
where
    F: FnOnce(&mut ServoGlue) -> Result<T, &'static str>,
{
    SERVO.with(|s| match s.borrow_mut().as_mut() {
        Some(ref mut s) => (f)(s),
        None => Err("Servo is not available in this thread"),
    })
}

#[no_mangle]
pub unsafe extern "C" fn init_servo(
    ctxt: EGLContext,
    surf: EGLSurface,
    disp: EGLDisplay,
    app: MLApp,
    logger: MLLogger,
    history_update: MLHistoryUpdate,
    url_update: MLURLUpdate,
    keyboard: MLKeyboard,
    url: *const c_char,
    width: u32,
    height: u32,
    hidpi: f32,
) -> *mut ServoInstance {
    let _ = log::set_boxed_logger(Box::new(logger));
    log::set_max_level(LOG_LEVEL);

    let gl = gl_glue::egl::init().expect("EGL initialization failure");

    let url = CStr::from_ptr(url).to_str().unwrap_or("about:blank");
    let coordinates = Coordinates::new(
        0,
        0,
        width as i32,
        height as i32,
        width as i32,
        height as i32,
    );
    let opts = InitOptions {
        args: None,
        url: Some(url.to_string()),
        density: hidpi,
        enable_subpixel_text_antialiasing: false,
        vr_pointer: None,
        coordinates,
    };
    let wakeup = Box::new(EventLoopWakerInstance);
    let shut_down_complete = Rc::new(Cell::new(false));
    let callbacks = Box::new(HostCallbacks {
        app,
        ctxt,
        surf,
        disp,
        shut_down_complete: shut_down_complete.clone(),
        history_update,
        url_update,
        keyboard,
    });
    info!("Starting servo");
    simpleservo::init(opts, gl, wakeup, callbacks).expect("error initializing Servo");

    let result = Box::new(ServoInstance {
        scroll_state: ScrollState::TriggerUp,
        scroll_scale: TypedScale::new(SCROLL_SCALE / hidpi),
        shut_down_complete,
    });
    Box::into_raw(result)
}

#[no_mangle]
pub unsafe extern "C" fn heartbeat_servo(_servo: *mut ServoInstance) {
    let _ = call(|s| s.perform_updates());
}

#[no_mangle]
pub unsafe extern "C" fn keyboard_servo(
    _servo: *mut ServoInstance,
    key_code: char,
    key_type: MLKeyType,
) {
    let key = match key_type {
        MLKeyType::kCharacter => Key::Character([key_code].iter().collect()),
        MLKeyType::kBackspace => Key::Backspace,
        MLKeyType::kEnter => Key::Enter,
        _ => return,
    };
    // TODO: can the ML1 generate separate press and release events?
    let key2 = key.clone();
    let _ = call(move |s| s.key_down(key2));
    let _ = call(move |s| s.key_up(key));
}

// Some magic numbers.

// How far does the cursor have to move for it to count as a drag rather than a click?
// (In device pixels squared, to avoid taking a sqrt when calculating move distance.)
const DRAG_CUTOFF_SQUARED: f32 = 900.0;

// How much should we scale scrolling by?
const SCROLL_SCALE: f32 = 3.0;

#[no_mangle]
pub unsafe extern "C" fn move_servo(servo: *mut ServoInstance, x: f32, y: f32) {
    // Servo's cursor was moved
    if let Some(servo) = servo.as_mut() {
        let point = DevicePoint::new(x, y);
        match servo.scroll_state {
            ScrollState::TriggerUp => {
                servo.scroll_state = ScrollState::TriggerUp;
                let _ = call(|s| s.move_mouse(x, y));
            },
            ScrollState::TriggerDown(start)
                if (start - point).square_length() < DRAG_CUTOFF_SQUARED =>
            {
                return;
            }
            ScrollState::TriggerDown(start) => {
                servo.scroll_state = ScrollState::TriggerDragging(start, point);
                let _ = call(|s| s.move_mouse(x, y));
                let delta = (point - start) * servo.scroll_scale;
                let start = start.to_i32();
                let _ = call(|s| s.scroll_start(delta.x, delta.y, start.x, start.y));
            },
            ScrollState::TriggerDragging(start, prev) => {
                servo.scroll_state = ScrollState::TriggerDragging(start, point);
                let _ = call(|s| s.move_mouse(x, y));
                let delta = (point - prev) * servo.scroll_scale;
                let start = start.to_i32();
                let _ = call(|s| s.scroll(delta.x, delta.y, start.x, start.y));
            },
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn trigger_servo(servo: *mut ServoInstance, x: f32, y: f32, down: bool) {
    // Servo was triggered
    if let Some(servo) = servo.as_mut() {
        let point = DevicePoint::new(x, y);
        match servo.scroll_state {
            ScrollState::TriggerUp if down => {
                servo.scroll_state = ScrollState::TriggerDown(point);
                let _ = call(|s| s.mouse_down(x, y, MouseButton::Left));
            },
            ScrollState::TriggerDown(start) if !down => {
                servo.scroll_state = ScrollState::TriggerUp;
                let _ = call(|s| s.mouse_up(start.x, start.y, MouseButton::Left));
                let _ = call(|s| s.click(start.x, start.y));
                let _ = call(|s| s.move_mouse(start.x, start.y));
            },
            ScrollState::TriggerDragging(start, prev) if !down => {
                servo.scroll_state = ScrollState::TriggerUp;
                let delta = (point - prev) * servo.scroll_scale;
                let start = start.to_i32();
                let _ = call(|s| s.scroll_end(delta.x, delta.y, start.x, start.y));
                let _ = call(|s| s.mouse_up(x, y, MouseButton::Left));
            },
            _ => return,
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn traverse_servo(_servo: *mut ServoInstance, delta: i32) {
    // Traverse the session history
    if delta == 0 {
        let _ = call(|s| s.reload());
    } else if delta < 0 {
        let _ = call(|s| s.go_back());
    } else {
        let _ = call(|s| s.go_forward());
    }
}

#[no_mangle]
pub unsafe extern "C" fn navigate_servo(_servo: *mut ServoInstance, text: *const c_char) {
    let text = CStr::from_ptr(text)
        .to_str()
        .expect("Failed to convert text to UTF-8");
    let url = ServoUrl::parse(text).unwrap_or_else(|_| {
        let mut search = ServoUrl::parse("https://duckduckgo.com")
            .expect("Failed to parse search URL")
            .into_url();
        search.query_pairs_mut().append_pair("q", text);
        ServoUrl::from_url(search)
    });
    let _ = call(|s| s.load_uri(url.as_str()));
}

// Some magic numbers for shutdown
const SHUTDOWN_DURATION: Duration = Duration::from_secs(10);
const SHUTDOWN_POLL_INTERVAL: Duration = Duration::from_millis(100);

#[no_mangle]
pub unsafe extern "C" fn discard_servo(servo: *mut ServoInstance) {
    if let Some(servo) = servo.as_mut() {
        let servo = Box::from_raw(servo);
        let finish = Instant::now() + SHUTDOWN_DURATION;
        let _ = call(|s| s.request_shutdown());
        while !servo.shut_down_complete.get() {
            let _ = call(|s| s.perform_updates());
            if Instant::now() > finish {
                warn!("Incomplete shutdown.");
            }
            thread::sleep(SHUTDOWN_POLL_INTERVAL);
        }
        deinit();
    }
}

struct HostCallbacks {
    ctxt: EGLContext,
    surf: EGLSurface,
    disp: EGLDisplay,
    shut_down_complete: Rc<Cell<bool>>,
    history_update: MLHistoryUpdate,
    url_update: MLURLUpdate,
    app: MLApp,
    keyboard: MLKeyboard,
}

impl HostTrait for HostCallbacks {
    fn flush(&self) {
        SwapBuffers(self.disp, self.surf);
    }

    fn make_current(&self) {
        MakeCurrent(self.disp, self.surf, self.surf, self.ctxt);
    }

    fn on_load_started(&self) {}
    fn on_load_ended(&self) {}
    fn on_title_changed(&self, _title: String) {}
    fn on_url_changed(&self, url: String) {
        if let Ok(cstr) = CString::new(url.as_str()) {
            (self.url_update.0)(self.app, cstr.as_ptr());
        }
    }

    fn on_history_changed(&self, can_go_back: bool, can_go_forward: bool) {
        (self.history_update.0)(self.app, can_go_back, can_go_forward);
    }

    fn on_animating_changed(&self, _animating: bool) {}

    fn on_shutdown_complete(&self) {
        self.shut_down_complete.set(true);
    }

    fn on_ime_state_changed(&self, show: bool) {
        (self.keyboard.0)(self.app, show)
    }
}

pub struct ServoInstance {
    scroll_state: ScrollState,
    scroll_scale: TypedScale<f32, DevicePixel, LayoutPixel>,
    shut_down_complete: Rc<Cell<bool>>,
}

#[derive(Clone, Copy)]
enum ScrollState {
    TriggerUp,
    TriggerDown(DevicePoint),
    TriggerDragging(DevicePoint, DevicePoint),
}

struct EventLoopWakerInstance;

impl EventLoopWaker for EventLoopWakerInstance {
    fn clone(&self) -> Box<EventLoopWaker + Send> {
        Box::new(EventLoopWakerInstance)
    }

    fn wake(&self) {}
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
        let mut msg = SmallVec::<[u8; 128]>::new();
        write!(msg, "{}\0", record.args()).unwrap();
        (self.0)(lvl, &msg[0] as *const _ as *const _);
    }

    fn flush(&self) {}
}
