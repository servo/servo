/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use egl::egl::EGLContext;
use egl::egl::EGLDisplay;
use egl::egl::EGLSurface;
use egl::egl::MakeCurrent;
use egl::egl::SwapBuffers;
use libc::{dup2, pipe, read};
use log::info;
use log::warn;
use rust_webvr::api::MagicLeapVRService;
use servo::euclid::Scale;
use servo::keyboard_types::Key;
use servo::servo_url::ServoUrl;
use servo::webrender_api::units::{DevicePixel, DevicePoint, LayoutPixel};
use simpleservo::{self, deinit, gl_glue, MouseButton, ServoGlue, SERVO};
use simpleservo::{
    Coordinates, EventLoopWaker, HostTrait, InitOptions, PromptResult, VRInitOptions,
};
use smallvec::SmallVec;
use std::cell::Cell;
use std::ffi::CStr;
use std::ffi::CString;
use std::io::Write;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_void;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use webxr::magicleap::MagicLeapDiscovery;

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
#[derive(Clone, Copy)]
pub struct MLLogger(Option<extern "C" fn(MLLogLevel, *const c_char)>);

#[repr(transparent)]
pub struct MLHistoryUpdate(Option<extern "C" fn(MLApp, bool, bool)>);

#[repr(transparent)]
pub struct MLURLUpdate(Option<extern "C" fn(MLApp, *const c_char)>);

#[repr(transparent)]
pub struct MLKeyboard(Option<extern "C" fn(MLApp, bool)>);

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
    landscape: bool,
    app: MLApp,
    logger: MLLogger,
    history_update: MLHistoryUpdate,
    url_update: MLURLUpdate,
    keyboard: MLKeyboard,
    url: *const c_char,
    default_args: *const c_char,
    width: u32,
    height: u32,
    hidpi: f32,
) -> *mut ServoInstance {
    redirect_stdout_to_log(logger);
    let _ = log::set_boxed_logger(Box::new(logger));
    log::set_max_level(LOG_LEVEL);

    let gl = gl_glue::egl::init().expect("EGL initialization failure");

    let coordinates = Coordinates::new(
        0,
        0,
        width as i32,
        height as i32,
        width as i32,
        height as i32,
    );

    let mut url = CStr::from_ptr(url).to_str().unwrap_or("about:blank");

    // If the URL has a space in it, then treat everything before the space as arguments
    let args = if let Some(i) = url.rfind(' ') {
        let (front, back) = url.split_at(i);
        url = back;
        front.split(' ').map(|s| s.to_owned()).collect()
    } else if !default_args.is_null() {
        CStr::from_ptr(default_args)
            .to_str()
            .unwrap_or("")
            .split(' ')
            .map(|s| s.to_owned())
            .collect()
    } else {
        Vec::new()
    };

    info!("got args: {:?}", args);

    let vr_init = if !landscape {
        let name = String::from("Magic Leap VR Display");
        let (service, heartbeat) = MagicLeapVRService::new(name, ctxt, gl.gl_wrapper.clone())
            .expect("Failed to create VR service");
        let service = Box::new(service);
        let heartbeat = Box::new(heartbeat);
        VRInitOptions::VRService(service, heartbeat)
    } else {
        VRInitOptions::None
    };

    let xr_discovery: Option<Box<dyn webxr_api::Discovery>> = if !landscape {
        let discovery = MagicLeapDiscovery::new(ctxt, gl.gl_wrapper.clone());
        Some(Box::new(discovery))
    } else {
        None
    };

    let opts = InitOptions {
        args,
        url: Some(url.to_string()),
        density: hidpi,
        enable_subpixel_text_antialiasing: false,
        vr_init,
        xr_discovery,
        coordinates,
        gl_context_pointer: Some(ctxt),
        native_display_pointer: Some(disp),
    };
    let wakeup = Box::new(EventLoopWakerInstance);
    let shut_down_complete = Rc::new(Cell::new(false));
    let callbacks = Box::new(HostCallbacks {
        app,
        ctxt,
        surf,
        disp,
        landscape,
        shut_down_complete: shut_down_complete.clone(),
        history_update,
        url_update,
        keyboard,
    });
    info!("Starting servo");
    simpleservo::init(opts, gl.gl_wrapper, wakeup, callbacks).expect("error initializing Servo");

    let result = Box::new(ServoInstance {
        scroll_state: ScrollState::TriggerUp,
        scroll_scale: Scale::new(SCROLL_SCALE / hidpi),
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
                let _ = call(|s| s.mouse_move(x, y));
            },
            ScrollState::TriggerDown(start)
                if (start - point).square_length() < DRAG_CUTOFF_SQUARED =>
            {
                return;
            }
            ScrollState::TriggerDown(start) => {
                servo.scroll_state = ScrollState::TriggerDragging(start, point);
                let _ = call(|s| s.mouse_move(x, y));
                let delta = (point - start) * servo.scroll_scale;
                let start = start.to_i32();
                let _ = call(|s| s.scroll_start(delta.x, delta.y, start.x, start.y));
            },
            ScrollState::TriggerDragging(start, prev) => {
                servo.scroll_state = ScrollState::TriggerDragging(start, point);
                let _ = call(|s| s.mouse_move(x, y));
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
                let _ = call(|s| s.click(start.x as f32, start.y as f32));
                let _ = call(|s| s.mouse_move(start.x, start.y));
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
    landscape: bool,
    shut_down_complete: Rc<Cell<bool>>,
    history_update: MLHistoryUpdate,
    url_update: MLURLUpdate,
    app: MLApp,
    keyboard: MLKeyboard,
}

impl HostTrait for HostCallbacks {
    fn flush(&self) {
        // Immersive and landscape apps have different requirements for who calls SwapBuffers.
        if self.landscape {
            SwapBuffers(self.disp, self.surf);
        }
    }

    fn make_current(&self) {
        MakeCurrent(self.disp, self.surf, self.surf, self.ctxt);
    }

    fn prompt_alert(&self, message: String, _trusted: bool) {
        warn!("Prompt Alert: {}", message);
    }

    fn prompt_ok_cancel(&self, message: String, _trusted: bool) -> PromptResult {
        warn!("Prompt not implemented. Cancelled. {}", message);
        PromptResult::Secondary
    }

    fn prompt_yes_no(&self, message: String, _trusted: bool) -> PromptResult {
        warn!("Prompt not implemented. Cancelled. {}", message);
        PromptResult::Secondary
    }

    fn prompt_input(&self, message: String, default: String, _trusted: bool) -> Option<String> {
        warn!("Input prompt not implemented. {}", message);
        Some(default)
    }

    fn on_load_started(&self) {}
    fn on_load_ended(&self) {}
    fn on_title_changed(&self, _title: String) {}
    fn on_allow_navigation(&self, _url: String) -> bool {
        true
    }
    fn on_url_changed(&self, url: String) {
        if let Ok(cstr) = CString::new(url.as_str()) {
            if let Some(url_update) = self.url_update.0 {
                url_update(self.app, cstr.as_ptr());
            }
        }
    }

    fn on_history_changed(&self, can_go_back: bool, can_go_forward: bool) {
        if let Some(history_update) = self.history_update.0 {
            history_update(self.app, can_go_back, can_go_forward);
        }
    }

    fn on_animating_changed(&self, _animating: bool) {}

    fn on_shutdown_complete(&self) {
        self.shut_down_complete.set(true);
    }

    fn on_ime_state_changed(&self, show: bool) {
        if let Some(keyboard) = self.keyboard.0 {
            keyboard(self.app, show)
        }
    }

    fn get_clipboard_contents(&self) -> Option<String> {
        None
    }

    fn set_clipboard_contents(&self, _contents: String) {}
}

pub struct ServoInstance {
    scroll_state: ScrollState,
    scroll_scale: Scale<f32, DevicePixel, LayoutPixel>,
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
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(EventLoopWakerInstance)
    }

    fn wake(&self) {}
}

impl log::Log for MLLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= LOG_LEVEL
    }

    fn log(&self, record: &log::Record) {
        if let Some(log) = self.0 {
            let lvl = match record.level() {
                log::Level::Error => MLLogLevel::Error,
                log::Level::Warn => MLLogLevel::Warning,
                log::Level::Info => MLLogLevel::Info,
                log::Level::Debug => MLLogLevel::Debug,
                log::Level::Trace => MLLogLevel::Verbose,
            };
            let mut msg = SmallVec::<[u8; 128]>::new();
            write!(msg, "{}\0", record.args()).unwrap();
            log(lvl, &msg[0] as *const _ as *const _);
        }
    }

    fn flush(&self) {}
}

fn redirect_stdout_to_log(logger: MLLogger) {
    let log = match logger.0 {
        None => return,
        Some(log) => log,
    };

    // The first step is to redirect stdout and stderr to the logs.
    // We redirect stdout and stderr to a custom descriptor.
    let mut pfd: [c_int; 2] = [0, 0];
    unsafe {
        pipe(pfd.as_mut_ptr());
        dup2(pfd[1], 1);
        dup2(pfd[1], 2);
    }

    let descriptor = pfd[0];

    // Then we spawn a thread whose only job is to read from the other side of the
    // pipe and redirect to the logs.
    let _detached = thread::spawn(move || {
        const BUF_LENGTH: usize = 512;
        let mut buf = vec![b'\0' as c_char; BUF_LENGTH];

        // Always keep at least one null terminator
        const BUF_AVAILABLE: usize = BUF_LENGTH - 1;
        let buf = &mut buf[..BUF_AVAILABLE];

        let mut cursor = 0_usize;

        loop {
            let result = {
                let read_into = &mut buf[cursor..];
                unsafe {
                    read(
                        descriptor,
                        read_into.as_mut_ptr() as *mut _,
                        read_into.len(),
                    )
                }
            };

            let end = if result == 0 {
                return;
            } else if result < 0 {
                log(
                    MLLogLevel::Error,
                    b"error in log thread; closing\0".as_ptr() as *const _,
                );
                return;
            } else {
                result as usize + cursor
            };

            // Only modify the portion of the buffer that contains real data.
            let buf = &mut buf[0..end];

            if let Some(last_newline_pos) = buf.iter().rposition(|&c| c == b'\n' as c_char) {
                buf[last_newline_pos] = b'\0' as c_char;
                log(MLLogLevel::Info, buf.as_ptr());
                if last_newline_pos < buf.len() - 1 {
                    let pos_after_newline = last_newline_pos + 1;
                    let len_not_logged_yet = buf[pos_after_newline..].len();
                    for j in 0..len_not_logged_yet as usize {
                        buf[j] = buf[pos_after_newline + j];
                    }
                    cursor = len_not_logged_yet;
                } else {
                    cursor = 0;
                }
            } else if end == BUF_AVAILABLE {
                // No newline found but the buffer is full, flush it anyway.
                // `buf.as_ptr()` is null-terminated by BUF_LENGTH being 1 less than BUF_AVAILABLE.
                log(MLLogLevel::Info, buf.as_ptr());
                cursor = 0;
            } else {
                cursor = end;
            }
        }
    });
}
