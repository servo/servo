/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate log;

#[cfg(target_os = "windows")]
mod vslogger;

#[cfg(not(target_os = "windows"))]
use env_logger;
use simpleservo::{self, gl_glue, ServoGlue, SERVO};
use simpleservo::{Coordinates, EventLoopWaker, HostTrait, InitOptions, VRInitOptions};
use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::{c_char, c_void};
use std::panic::{self, AssertUnwindSafe, UnwindSafe};

/// Catch any panic function used by extern "C" functions.
fn catch_any_panic<F: FnOnce() + UnwindSafe>(function: F) -> bool {
    panic::catch_unwind(function).is_ok()
}

#[cfg(not(target_os = "windows"))]
fn redirect_stdout_stderr() -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "windows")]
fn redirect_stdout_stderr() -> Result<(), String> {
    do_redirect_stdout_stderr().map_err(|()| {
        format!("GetLastError() = {}", unsafe {
            winapi::um::errhandlingapi::GetLastError()
        })
    })
}

#[cfg(target_os = "windows")]
// Function to redirect STDOUT (1) and STDERR(2) to Windows API
// OutputDebugString().
// Return Value: Result<(), String>
//              Ok() - stdout and stderr redirects.
//              Err(str) - The Err value can contain the string value of GetLastError.
fn do_redirect_stdout_stderr() -> Result<(), ()> {
    use std::thread;
    use winapi::shared;
    use winapi::um::debugapi;
    use winapi::um::handleapi;
    use winapi::um::minwinbase;
    use winapi::um::namedpipeapi;
    use winapi::um::processenv;
    use winapi::um::winbase;
    use winapi::um::winnt;

    let mut h_read_pipe: winnt::HANDLE = handleapi::INVALID_HANDLE_VALUE;
    let mut h_write_pipe: winnt::HANDLE = handleapi::INVALID_HANDLE_VALUE;
    let mut secattr: minwinbase::SECURITY_ATTRIBUTES = unsafe { mem::zeroed() };
    const BUF_LENGTH: usize = 1024;

    secattr.nLength = mem::size_of::<minwinbase::SECURITY_ATTRIBUTES>() as u32;
    secattr.bInheritHandle = shared::minwindef::TRUE;
    secattr.lpSecurityDescriptor = shared::ntdef::NULL;

    unsafe {
        if namedpipeapi::CreatePipe(
            &mut h_read_pipe,
            &mut h_write_pipe,
            &mut secattr,
            BUF_LENGTH as u32,
        ) == 0
        {
            return Err(());
        }

        if processenv::SetStdHandle(winbase::STD_OUTPUT_HANDLE, h_write_pipe) == 0 ||
            processenv::SetStdHandle(winbase::STD_ERROR_HANDLE, h_write_pipe) == 0
        {
            return Err(());
        }

        if handleapi::SetHandleInformation(
            h_read_pipe,
            winbase::HANDLE_FLAG_INHERIT,
            winbase::HANDLE_FLAG_INHERIT,
        ) == 0 ||
            handleapi::SetHandleInformation(
                h_write_pipe,
                winbase::HANDLE_FLAG_INHERIT,
                winbase::HANDLE_FLAG_INHERIT,
            ) == 0
        {
            return Err(());
        }

        let h_read_pipe_fd = libc::open_osfhandle(h_read_pipe as libc::intptr_t, libc::O_RDONLY);
        let h_write_pipe_fd = libc::open_osfhandle(h_write_pipe as libc::intptr_t, libc::O_WRONLY);

        if h_read_pipe_fd == -1 || h_write_pipe_fd == -1 {
            return Err(());
        }

        // 0 indicates success.
        if libc::dup2(h_write_pipe_fd, 1) != 0 || libc::dup2(h_write_pipe_fd, 2) != 0 {
            return Err(());
        }

        // If SetStdHandle(winbase::STD_OUTPUT_HANDLE, hWritePipe) is not called prior,
        // this will fail.  GetStdHandle() is used to make certain "servo" has the stdout
        // file descriptor associated.
        let h_stdout = processenv::GetStdHandle(winbase::STD_OUTPUT_HANDLE);
        if h_stdout == handleapi::INVALID_HANDLE_VALUE || h_stdout == shared::ntdef::NULL {
            return Err(());
        }

        // If SetStdHandle(winbase::STD_ERROR_HANDLE, hWritePipe) is not called prior,
        // this will fail.  GetStdHandle() is used to make certain "servo" has the stderr
        // file descriptor associated.
        let h_stderr = processenv::GetStdHandle(winbase::STD_ERROR_HANDLE);
        if h_stderr == handleapi::INVALID_HANDLE_VALUE || h_stderr == shared::ntdef::NULL {
            return Err(());
        }

        // Spawn a thread.  The thread will redirect all STDOUT and STDERR messages
        // to OutputDebugString()
        let _handler = thread::spawn(move || {
            loop {
                let mut read_buf: [i8; BUF_LENGTH] = [0; BUF_LENGTH];

                let result = libc::read(
                    h_read_pipe_fd,
                    read_buf.as_mut_ptr() as *mut _,
                    read_buf.len() as u32 - 1,
                );

                if result == -1 {
                    break;
                }

                // Write to Debug port.
                debugapi::OutputDebugStringA(read_buf.as_mut_ptr() as winnt::LPSTR);
            }
        });
    }

    Ok(())
}

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
    pub on_alert: extern "C" fn(message: *const c_char),
    pub on_load_started: extern "C" fn(),
    pub on_load_ended: extern "C" fn(),
    pub on_title_changed: extern "C" fn(title: *const c_char),
    pub on_allow_navigation: extern "C" fn(url: *const c_char) -> bool,
    pub on_url_changed: extern "C" fn(url: *const c_char),
    pub on_history_changed: extern "C" fn(can_go_back: bool, can_go_forward: bool),
    pub on_animating_changed: extern "C" fn(animating: bool),
    pub on_shutdown_complete: extern "C" fn(),
    pub on_ime_state_changed: extern "C" fn(show: bool),
    pub get_clipboard_contents: extern "C" fn() -> *const c_char,
    pub set_clipboard_contents: extern "C" fn(contents: *const c_char),
}

/// Servo options
#[repr(C)]
pub struct CInitOptions {
    pub args: *const c_char,
    pub url: *const c_char,
    pub width: i32,
    pub height: i32,
    pub density: f32,
    pub vr_pointer: *mut c_void,
    pub enable_subpixel_text_antialiasing: bool,
}

/// The returned string is not freed. This will leak.
#[no_mangle]
pub extern "C" fn servo_version() -> *const c_char {
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let v = simpleservo::servo_version();
        let text = CString::new(v).expect("Can't create string");
        let ptr: *const c_char = text.as_ptr();
        mem::forget(text);
        ptr
    }));

    match result {
        Ok(ptr) => ptr,
        Err(_) => std::ptr::null(),
    }
}

#[cfg(target_os = "windows")]
fn init_logger() {
    use log::LevelFilter;
    use std::sync::Once;
    use vslogger::VSLogger;

    static LOGGER: VSLogger = VSLogger;
    static LOGGER_INIT: Once = Once::new();
    LOGGER_INIT.call_once(|| {
        log::set_logger(&LOGGER)
            .map(|_| log::set_max_level(LevelFilter::Debug))
            .unwrap();
    });
}

#[cfg(not(target_os = "windows"))]
fn init_logger() {
    crate::env_logger::init();
}

unsafe fn init(
    opts: CInitOptions,
    gl: gl_glue::ServoGl,
    gl_context: Option<*const c_void>,
    display: Option<*const c_void>,
    wakeup: extern "C" fn(),
    callbacks: CHostCallbacks,
) {
    init_logger();

    if let Err(reason) = redirect_stdout_stderr() {
        warn!("Error redirecting stdout/stderr: {}", reason);
    }

    let args = if !opts.args.is_null() {
        let args = CStr::from_ptr(opts.args);
        args.to_str()
            .unwrap_or("")
            .split(' ')
            .map(|s| s.to_owned())
            .collect()
    } else {
        vec![]
    };

    let url = CStr::from_ptr(opts.url);
    let url = url.to_str().map(|s| s.to_string()).ok();

    let coordinates = Coordinates::new(0, 0, opts.width, opts.height, opts.width, opts.height);

    let opts = InitOptions {
        args,
        url,
        coordinates,
        density: opts.density,
        vr_init: if opts.vr_pointer.is_null() {
            VRInitOptions::None
        } else {
            VRInitOptions::VRExternal(opts.vr_pointer)
        },
        xr_discovery: None,
        enable_subpixel_text_antialiasing: opts.enable_subpixel_text_antialiasing,
        gl_context_pointer: gl_context,
        native_display_pointer: display,
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
) -> bool {
    catch_any_panic(|| {
        init_logger();
        let gl = gl_glue::egl::init().unwrap();
        unsafe {
            init(
                opts,
                gl.gl_wrapper,
                Some(gl.gl_context),
                Some(gl.display),
                wakeup,
                callbacks,
            )
        }
    })
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
#[no_mangle]
pub extern "C" fn init_with_gl(
    opts: CInitOptions,
    wakeup: extern "C" fn(),
    callbacks: CHostCallbacks,
) -> bool {
    catch_any_panic(|| {
        init_logger();
        let gl = gl_glue::gl::init().unwrap();
        unsafe { init(opts, gl, None, None, wakeup, callbacks) }
    })
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
pub extern "C" fn resize(width: i32, height: i32) {
    debug!("resize {}/{}", width, height);
    call(|s| {
        let coordinates = Coordinates::new(0, 0, width, height, width, height);
        s.resize(coordinates)
    });
}

#[no_mangle]
pub extern "C" fn perform_updates() -> bool {
    catch_any_panic(|| {
        debug!("perform_updates");
        call(|s| s.perform_updates());
    })
}

#[no_mangle]
pub extern "C" fn load_uri(url: *const c_char) -> bool {
    catch_any_panic(|| {
        debug!("load_url");
        let url = unsafe { CStr::from_ptr(url) };
        let url = url.to_str().expect("Can't read string");
        call(|s| s.load_uri(url));
    })
}

#[no_mangle]
pub extern "C" fn reload() -> bool {
    catch_any_panic(|| {
        debug!("reload");
        call(|s| s.reload());
    })
}

#[no_mangle]
pub extern "C" fn stop() -> bool {
    catch_any_panic(|| {
        debug!("stop");
        call(|s| s.stop());
    })
}

#[no_mangle]
pub extern "C" fn refresh() -> bool {
    catch_any_panic(|| {
        debug!("refresh");
        call(|s| s.refresh());
    })
}

#[no_mangle]
pub extern "C" fn go_back() -> bool {
    catch_any_panic(|| {
        debug!("go_back");
        call(|s| s.go_back());
    })
}

#[no_mangle]
pub extern "C" fn go_forward() -> bool {
    catch_any_panic(|| {
        debug!("go_forward");
        call(|s| s.go_forward());
    })
}

#[no_mangle]
pub extern "C" fn scroll_start(dx: i32, dy: i32, x: i32, y: i32) -> bool {
    catch_any_panic(|| {
        debug!("scroll_start");
        call(|s| s.scroll_start(dx as f32, dy as f32, x, y));
    })
}

#[no_mangle]
pub extern "C" fn scroll_end(dx: i32, dy: i32, x: i32, y: i32) -> bool {
    catch_any_panic(|| {
        debug!("scroll_end");
        call(|s| s.scroll_end(dx as f32, dy as f32, x, y));
    })
}

#[no_mangle]
pub extern "C" fn scroll(dx: i32, dy: i32, x: i32, y: i32) -> bool {
    catch_any_panic(|| {
        debug!("scroll");
        call(|s| s.scroll(dx as f32, dy as f32, x, y));
    })
}

#[no_mangle]
pub extern "C" fn touch_down(x: f32, y: f32, pointer_id: i32) -> bool {
    catch_any_panic(|| {
        debug!("touch down");
        call(|s| s.touch_down(x, y, pointer_id));
    })
}

#[no_mangle]
pub extern "C" fn touch_up(x: f32, y: f32, pointer_id: i32) -> bool {
    catch_any_panic(|| {
        debug!("touch up");
        call(|s| s.touch_up(x, y, pointer_id));
    })
}

#[no_mangle]
pub extern "C" fn touch_move(x: f32, y: f32, pointer_id: i32) -> bool {
    catch_any_panic(|| {
        debug!("touch move");
        call(|s| s.touch_move(x, y, pointer_id));
    })
}

#[no_mangle]
pub extern "C" fn touch_cancel(x: f32, y: f32, pointer_id: i32) -> bool {
    catch_any_panic(|| {
        debug!("touch cancel");
        call(|s| s.touch_cancel(x, y, pointer_id));
    })
}

#[no_mangle]
pub extern "C" fn pinchzoom_start(factor: f32, x: i32, y: i32) -> bool {
    catch_any_panic(|| {
        debug!("pinchzoom_start");
        call(|s| s.pinchzoom_start(factor, x as u32, y as u32));
    })
}

#[no_mangle]
pub extern "C" fn pinchzoom(factor: f32, x: i32, y: i32) -> bool {
    catch_any_panic(|| {
        debug!("pinchzoom");
        call(|s| s.pinchzoom(factor, x as u32, y as u32));
    })
}

#[no_mangle]
pub extern "C" fn pinchzoom_end(factor: f32, x: i32, y: i32) -> bool {
    catch_any_panic(|| {
        debug!("pinchzoom_end");
        call(|s| s.pinchzoom_end(factor, x as u32, y as u32));
    })
}

#[no_mangle]
pub extern "C" fn click(x: i32, y: i32) -> bool {
    catch_any_panic(|| {
        debug!("click");
        call(|s| s.click(x as f32, y as f32));
    })
}

pub struct WakeupCallback(extern "C" fn());

impl WakeupCallback {
    fn new(callback: extern "C" fn()) -> WakeupCallback {
        WakeupCallback(callback)
    }
}

impl EventLoopWaker for WakeupCallback {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
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

    fn on_alert(&self, message: String) {
        debug!("on_alert");
        let message = CString::new(message).expect("Can't create string");
        let msg_ptr = message.as_ptr();
        mem::forget(message);
        (self.0.on_alert)(msg_ptr);
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

    fn on_allow_navigation(&self, url: String) -> bool {
        debug!("on_allow_navigation");
        let url = CString::new(url).expect("Can't create string");
        let url_ptr = url.as_ptr();
        mem::forget(url);
        (self.0.on_allow_navigation)(url_ptr)
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

    fn get_clipboard_contents(&self) -> Option<String> {
        debug!("get_clipboard_contents");
        let raw_contents = (self.0.get_clipboard_contents)();
        if raw_contents.is_null() {
            return None;
        }
        let c_str = unsafe { CStr::from_ptr(raw_contents) };
        let contents_str = c_str.to_str().expect("Can't create str");
        Some(contents_str.to_owned())
    }

    fn set_clipboard_contents(&self, contents: String) {
        debug!("set_clipboard_contents");
        let contents = CString::new(contents).expect("Can't create string");
        let contents_ptr = contents.as_ptr();
        mem::forget(contents);
        (self.0.set_clipboard_contents)(contents_ptr);
    }
}
