/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate log;

use env_logger;
use simpleservo::{self, gl_glue, EventLoopWaker, HostTrait, InitOptions, ServoGlue, SERVO};
use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::{c_char, c_void};

fn call<F>(f: F)
where
    F: Fn(&mut ServoGlue) -> Result<(), &'static str>,
{
    SERVO.with(|s| {
        if let Err(error) = match s.borrow_mut().as_mut() {
            Some(ref mut s) => (f)(s),
            None => Err("Servo not available in this thread"),
        } {
            // FIXME: All C calls should have a have generic Result-like
            // return type. For now, we just panic instead of notifying
            // the embedder.
            panic!(error);
        }
    });
}

/// Callback used by Servo internals
#[repr(C)]
pub struct CHostCallbacks {
    pub flush: extern "C" fn(),
    pub make_current: extern "C" fn(),
    pub on_load_started: extern "C" fn(),
    pub on_load_ended: extern "C" fn(),
    pub on_title_changed: extern "C" fn(title: *const c_char),
    pub on_url_changed: extern "C" fn(url: *const c_char),
    pub on_history_changed: extern "C" fn(can_go_back: bool, can_go_forward: bool),
    pub on_animating_changed: extern "C" fn(animating: bool),
    pub on_shutdown_complete: extern "C" fn(),
    pub on_ime_state_changed: extern "C" fn(show: bool),
}

/// Servo options
#[repr(C)]
pub struct CInitOptions {
    pub args: *const c_char,
    pub url: *const c_char,
    pub width: u32,
    pub height: u32,
    pub density: f32,
    pub vr_pointer: *mut c_void,
    pub enable_subpixel_text_antialiasing: bool,
}

/// The returned string is not freed. This will leak.
#[no_mangle]
pub extern "C" fn servo_version() -> *const c_char {
    let v = simpleservo::servo_version();
    let text = CString::new(v).expect("Can't create string");
    let ptr = text.as_ptr();
    mem::forget(text);
    ptr
}

fn init(
    opts: CInitOptions,
    gl: gl_glue::ServoGl,
    wakeup: extern "C" fn(),
    callbacks: CHostCallbacks,
) {
    crate::env_logger::init();

    let args = unsafe { CStr::from_ptr(opts.args) };
    let args = args.to_str().map(|s| s.to_string()).ok();

    let url = unsafe { CStr::from_ptr(opts.url) };
    let url = url.to_str().map(|s| s.to_string()).ok();

    let opts = InitOptions {
        args,
        url,
        width: opts.width,
        height: opts.height,
        density: opts.density,
        vr_pointer: if opts.vr_pointer.is_null() {
            None
        } else {
            Some(opts.vr_pointer)
        },
        enable_subpixel_text_antialiasing: opts.enable_subpixel_text_antialiasing,
    };

    let wakeup = Box::new(WakeupCallback::new(wakeup));
    let callbacks = Box::new(HostCallbacks::new(callbacks));

    simpleservo::init(opts, gl, wakeup, callbacks).unwrap();
}

#[cfg(target_os = "windows")]
#[no_mangle]
pub extern "C" fn init_with_egl(
    opts: CInitOptions,
    wakeup: extern "C" fn(),
    callbacks: CHostCallbacks,
) {
    let gl = gl_glue::egl::init().unwrap();
    init(opts, gl, wakeup, callbacks)
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
#[no_mangle]
pub extern "C" fn init_with_gl(
    opts: CInitOptions,
    wakeup: extern "C" fn(),
    callbacks: CHostCallbacks,
) {
    let gl = gl_glue::gl::init().unwrap();
    init(opts, gl, wakeup, callbacks)
}

#[no_mangle]
pub extern "C" fn deinit() {
    debug!("deinit");
    simpleservo::deinit();
}

#[no_mangle]
pub extern "C" fn request_shutdown() {
    debug!("request_shutdown");
    call(|s| s.request_shutdown());
}

#[no_mangle]
pub extern "C" fn set_batch_mode(batch: bool) {
    debug!("set_batch_mode");
    call(|s| s.set_batch_mode(batch));
}

#[no_mangle]
pub extern "C" fn resize(width: u32, height: u32) {
    debug!("resize {}/{}", width, height);
    call(|s| s.resize(width, height));
}

#[no_mangle]
pub extern "C" fn perform_updates() {
    debug!("perform_updates");
    call(|s| s.perform_updates());
}

#[no_mangle]
pub extern "C" fn load_uri(url: *const c_char) {
    debug!("load_url");
    let url = unsafe { CStr::from_ptr(url) };
    let url = url.to_str().expect("Can't read string");
    call(|s| s.load_uri(url));
}

#[no_mangle]
pub extern "C" fn reload() {
    debug!("reload");
    call(|s| s.reload());
}

#[no_mangle]
pub extern "C" fn stop() {
    debug!("stop");
    call(|s| s.stop());
}

#[no_mangle]
pub extern "C" fn refresh() {
    debug!("refresh");
    call(|s| s.refresh());
}

#[no_mangle]
pub extern "C" fn go_back() {
    debug!("go_back");
    call(|s| s.go_back());
}

#[no_mangle]
pub extern "C" fn go_forward() {
    debug!("go_forward");
    call(|s| s.go_forward());
}

#[no_mangle]
pub extern "C" fn scroll_start(dx: i32, dy: i32, x: i32, y: i32) {
    debug!("scroll_start");
    call(|s| s.scroll_start(dx as f32, dy as f32, x, y));
}

#[no_mangle]
pub extern "C" fn scroll_end(dx: i32, dy: i32, x: i32, y: i32) {
    debug!("scroll_end");
    call(|s| s.scroll_end(dx as f32, dy as f32, x, y));
}

#[no_mangle]
pub extern "C" fn scroll(dx: i32, dy: i32, x: i32, y: i32) {
    debug!("scroll");
    call(|s| s.scroll(dx as f32, dy as f32, x, y));
}

#[no_mangle]
pub extern "C" fn touch_down(x: f32, y: f32, pointer_id: i32) {
    debug!("touch down");
    call(|s| s.touch_down(x, y, pointer_id));
}

#[no_mangle]
pub extern "C" fn touch_up(x: f32, y: f32, pointer_id: i32) {
    debug!("touch up");
    call(|s| s.touch_up(x, y, pointer_id));
}

#[no_mangle]
pub extern "C" fn touch_move(x: f32, y: f32, pointer_id: i32) {
    debug!("touch move");
    call(|s| s.touch_move(x, y, pointer_id));
}

#[no_mangle]
pub extern "C" fn touch_cancel(x: f32, y: f32, pointer_id: i32) {
    debug!("touch cancel");
    call(|s| s.touch_cancel(x, y, pointer_id));
}

#[no_mangle]
pub extern "C" fn pinchzoom_start(factor: f32, x: i32, y: i32) {
    debug!("pinchzoom_start");
    call(|s| s.pinchzoom_start(factor, x as u32, y as u32));
}

#[no_mangle]
pub extern "C" fn pinchzoom(factor: f32, x: i32, y: i32) {
    debug!("pinchzoom");
    call(|s| s.pinchzoom(factor, x as u32, y as u32));
}

#[no_mangle]
pub extern "C" fn pinchzoom_end(factor: f32, x: i32, y: i32) {
    debug!("pinchzoom_end");
    call(|s| s.pinchzoom_end(factor, x as u32, y as u32));
}

#[no_mangle]
pub extern "C" fn click(x: i32, y: i32) {
    debug!("click");
    call(|s| s.click(x as f32, y as f32));
}

pub struct WakeupCallback(extern "C" fn());

impl WakeupCallback {
    fn new(callback: extern "C" fn()) -> WakeupCallback {
        WakeupCallback(callback)
    }
}

impl EventLoopWaker for WakeupCallback {
    fn clone(&self) -> Box<dyn EventLoopWaker + Send> {
        Box::new(WakeupCallback(self.0))
    }
    fn wake(&self) {
        (self.0)();
    }
}

struct HostCallbacks(CHostCallbacks);

impl HostCallbacks {
    fn new(callback: CHostCallbacks) -> HostCallbacks {
        HostCallbacks(callback)
    }
}

impl HostTrait for HostCallbacks {
    fn flush(&self) {
        debug!("flush");
        (self.0.flush)();
    }

    fn make_current(&self) {
        debug!("make_current");
        (self.0.make_current)();
    }

    fn on_load_started(&self) {
        debug!("on_load_ended");
        (self.0.on_load_started)();
    }

    fn on_load_ended(&self) {
        debug!("on_load_ended");
        (self.0.on_load_ended)();
    }

    fn on_title_changed(&self, title: String) {
        debug!("on_title_changed");
        let title = CString::new(title).expect("Can't create string");
        let title_ptr = title.as_ptr();
        mem::forget(title);
        (self.0.on_title_changed)(title_ptr);
    }

    fn on_url_changed(&self, url: String) {
        debug!("on_url_changed");
        let url = CString::new(url).expect("Can't create string");
        let url_ptr = url.as_ptr();
        mem::forget(url);
        (self.0.on_url_changed)(url_ptr);
    }

    fn on_history_changed(&self, can_go_back: bool, can_go_forward: bool) {
        debug!("on_history_changed");
        (self.0.on_history_changed)(can_go_back, can_go_forward);
    }

    fn on_animating_changed(&self, animating: bool) {
        debug!("on_animating_changed");
        (self.0.on_animating_changed)(animating);
    }

    fn on_shutdown_complete(&self) {
        debug!("on_shutdown_complete");
        (self.0.on_shutdown_complete)();
    }

    fn on_ime_state_changed(&self, show: bool) {
        debug!("on_ime_state_changed");
        (self.0.on_ime_state_changed)(show);
    }
}
