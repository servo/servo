/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

mod prefs;

#[cfg(target_os = "windows")]
mod vslogger;

#[cfg(not(target_os = "windows"))]
use env_logger;
use keyboard_types::Key;
use log::LevelFilter;
use simpleservo::{self, gl_glue, ServoGlue, SERVO};
use simpleservo::{
    ContextMenuResult, Coordinates, DeviceIntRect, EventLoopWaker, HostTrait, InitOptions,
    InputMethodType, MediaSessionActionType, MediaSessionPlaybackState, MouseButton, PromptResult,
    SurfmanIntegration,
};
use std::ffi::{CStr, CString};
#[cfg(target_os = "windows")]
use std::mem;
use std::os::raw::{c_char, c_uint, c_void};
use std::slice;
use std::str::FromStr;
use std::sync::{Mutex, RwLock};

extern "C" fn default_panic_handler(msg: *const c_char) {
    let c_str: &CStr = unsafe { CStr::from_ptr(msg) };
    error!("{}", c_str.to_str().unwrap());
}

type LogHandlerFn = extern "C" fn(buffer: *const c_char, len: u32);

lazy_static! {
    static ref ON_PANIC: RwLock<extern "C" fn(*const c_char)> = RwLock::new(default_panic_handler);
    static ref SERVO_VERSION: CString =
        CString::new(simpleservo::servo_version()).expect("Can't create string");
    pub(crate) static ref OUTPUT_LOG_HANDLER: Mutex<Option<LogHandlerFn>> = Mutex::new(None);
}

#[no_mangle]
pub extern "C" fn register_panic_handler(on_panic: extern "C" fn(*const c_char)) {
    *ON_PANIC.write().unwrap() = on_panic;
}

/// Report panic to embedder.
fn report_panic(reason: &str, backtrace: Option<String>) {
    let message = match backtrace {
        Some(bt) => format!("Servo panic ({})\n{}", reason, bt),
        None => format!("Servo panic ({})", reason),
    };
    let error = CString::new(message).expect("Can't create string");
    (ON_PANIC.read().unwrap())(error.as_ptr());
    // At this point, embedder should probably have thrown, so we never reach
    // this point. But if it didn't, don't recursively panic.
}

#[cfg(not(target_os = "windows"))]
fn redirect_stdout_stderr(_handler: LogHandlerFn) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "windows")]
fn redirect_stdout_stderr(handler: LogHandlerFn) -> Result<(), String> {
    do_redirect_stdout_stderr(handler).map_err(|()| {
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
fn do_redirect_stdout_stderr(handler: LogHandlerFn) -> Result<(), ()> {
    use std::thread;
    use winapi::shared;
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
        // to the provided handler function.
        let _handler = thread::spawn(move || loop {
            let mut read_buf: [i8; BUF_LENGTH] = [0; BUF_LENGTH];

            let result = libc::read(
                h_read_pipe_fd,
                read_buf.as_mut_ptr() as *mut _,
                read_buf.len() as u32 - 1,
            );

            if result == -1 {
                break;
            }
            handler(read_buf.as_ptr(), result as u32);
        });
    }

    Ok(())
}

fn call<T, F>(f: F) -> T
where
    F: FnOnce(&mut ServoGlue) -> Result<T, &'static str>,
{
    match SERVO.with(|s| match s.borrow_mut().as_mut() {
        Some(ref mut s) => (f)(s),
        None => Err("Servo not available in this thread"),
    }) {
        Err(e) => panic!("{}", e),
        Ok(r) => r,
    }
}

/// Callback used by Servo internals
#[repr(C)]
pub struct CHostCallbacks {
    pub on_load_started: extern "C" fn(),
    pub on_load_ended: extern "C" fn(),
    pub on_title_changed: extern "C" fn(title: *const c_char),
    pub on_allow_navigation: extern "C" fn(url: *const c_char) -> bool,
    pub on_url_changed: extern "C" fn(url: *const c_char),
    pub on_history_changed: extern "C" fn(can_go_back: bool, can_go_forward: bool),
    pub on_animating_changed: extern "C" fn(animating: bool),
    pub on_shutdown_complete: extern "C" fn(),
    pub on_ime_show: extern "C" fn(
        text: *const c_char,
        text_index: i32,
        multiline: bool,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ),
    pub on_ime_hide: extern "C" fn(),
    pub get_clipboard_contents: extern "C" fn() -> *const c_char,
    pub set_clipboard_contents: extern "C" fn(contents: *const c_char),
    pub on_media_session_metadata:
        extern "C" fn(title: *const c_char, album: *const c_char, artist: *const c_char),
    pub on_media_session_playback_state_change: extern "C" fn(state: CMediaSessionPlaybackState),
    pub on_media_session_set_position_state:
        extern "C" fn(duration: f64, position: f64, playback_rate: f64),
    pub prompt_alert: extern "C" fn(message: *const c_char, trusted: bool),
    pub prompt_ok_cancel: extern "C" fn(message: *const c_char, trusted: bool) -> CPromptResult,
    pub prompt_yes_no: extern "C" fn(message: *const c_char, trusted: bool) -> CPromptResult,
    pub prompt_input:
        extern "C" fn(message: *const c_char, def: *const c_char, trusted: bool) -> *const c_char,
    pub on_devtools_started:
        extern "C" fn(result: CDevtoolsServerState, port: c_uint, token: *const c_char),
    pub show_context_menu:
        extern "C" fn(title: *const c_char, items_list: *const *const c_char, items_size: u32),
    pub on_log_output: extern "C" fn(buffer: *const c_char, buffer_length: u32),
}

/// Servo options
#[repr(C)]
pub struct CInitOptions {
    pub args: *const c_char,
    pub width: i32,
    pub height: i32,
    pub density: f32,
    pub vslogger_mod_list: *const *const c_char,
    pub vslogger_mod_size: u32,
    pub native_widget: *mut c_void,
    pub prefs: *const prefs::CPrefList,
}

#[repr(C)]
pub enum CMouseButton {
    Left,
    Right,
    Middle,
}

impl CMouseButton {
    pub fn convert(&self) -> MouseButton {
        match self {
            CMouseButton::Left => MouseButton::Left,
            CMouseButton::Right => MouseButton::Right,
            CMouseButton::Middle => MouseButton::Middle,
        }
    }
}

#[repr(C)]
pub enum CPromptResult {
    Dismissed,
    Primary,
    Secondary,
}

impl CPromptResult {
    pub fn convert(&self) -> PromptResult {
        match self {
            CPromptResult::Primary => PromptResult::Primary,
            CPromptResult::Secondary => PromptResult::Secondary,
            CPromptResult::Dismissed => PromptResult::Dismissed,
        }
    }
}

#[repr(C)]
pub enum CMediaSessionPlaybackState {
    None = 1,
    Playing,
    Paused,
}

#[repr(C)]
pub enum CDevtoolsServerState {
    Started,
    Error,
}

impl From<MediaSessionPlaybackState> for CMediaSessionPlaybackState {
    fn from(state: MediaSessionPlaybackState) -> Self {
        match state {
            MediaSessionPlaybackState::None_ => CMediaSessionPlaybackState::None,
            MediaSessionPlaybackState::Playing => CMediaSessionPlaybackState::Playing,
            MediaSessionPlaybackState::Paused => CMediaSessionPlaybackState::Paused,
        }
    }
}

#[repr(C)]
pub enum CMediaSessionActionType {
    Play = 1,
    Pause,
    SeekBackward,
    SeekForward,
    PreviousTrack,
    NextTrack,
    SkipAd,
    Stop,
    SeekTo,
}

impl CMediaSessionActionType {
    pub fn convert(&self) -> MediaSessionActionType {
        match self {
            CMediaSessionActionType::Play => MediaSessionActionType::Play,
            CMediaSessionActionType::Pause => MediaSessionActionType::Pause,
            CMediaSessionActionType::SeekBackward => MediaSessionActionType::SeekBackward,
            CMediaSessionActionType::SeekForward => MediaSessionActionType::SeekForward,
            CMediaSessionActionType::PreviousTrack => MediaSessionActionType::PreviousTrack,
            CMediaSessionActionType::NextTrack => MediaSessionActionType::NextTrack,
            CMediaSessionActionType::SkipAd => MediaSessionActionType::SkipAd,
            CMediaSessionActionType::Stop => MediaSessionActionType::Stop,
            CMediaSessionActionType::SeekTo => MediaSessionActionType::SeekTo,
        }
    }
}

#[repr(C)]
pub enum CContextMenuResult {
    Ignored,
    Selected,
    // Can't use Dismissed. Already used by PromptResult. See:
    // https://github.com/servo/servo/issues/25986
    Dismissed_,
}

impl CContextMenuResult {
    pub fn convert(&self, idx: u32) -> ContextMenuResult {
        match self {
            CContextMenuResult::Ignored => ContextMenuResult::Ignored,
            CContextMenuResult::Dismissed_ => ContextMenuResult::Dismissed,
            CContextMenuResult::Selected => ContextMenuResult::Selected(idx as usize),
        }
    }
}

/// The returned string is not freed. This will leak.
#[no_mangle]
pub extern "C" fn servo_version() -> *const c_char {
    SERVO_VERSION.as_ptr()
}

#[cfg(target_os = "windows")]
fn init_logger(modules: &[*const c_char], level: LevelFilter) {
    use crate::vslogger::LOG_MODULE_FILTERS;
    use std::sync::Once;
    use vslogger::VSLogger;

    static LOGGER: VSLogger = VSLogger;
    static LOGGER_INIT: Once = Once::new();

    if !modules.is_empty() {
        *LOG_MODULE_FILTERS.lock().unwrap() = modules
            .iter()
            .map(|modules| unsafe { CStr::from_ptr(*modules).to_string_lossy().into_owned() })
            .collect::<Vec<_>>();
    }

    LOGGER_INIT.call_once(|| {
        log::set_logger(&LOGGER)
            .map(|_| log::set_max_level(level))
            .unwrap();
    });
}

#[cfg(not(target_os = "windows"))]
fn init_logger(_modules: &[*const c_char], _level: LevelFilter) {
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
    // Catch any panics.
    std::panic::set_hook(Box::new(|info| {
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &**s,
                None => "Box<Any>",
            },
        };
        let current_thread = std::thread::current();
        let name = current_thread.name().unwrap_or("<unnamed>");
        let details = if let Some(location) = info.location() {
            format!(
                "{} (thread {}, at {}:{})",
                msg,
                name,
                location.file(),
                location.line()
            )
        } else {
            format!("{} (thread {})", msg, name)
        };
        report_panic("General panic handler", Some(details));
    }));

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

    let logger_level = if let Some(level_index) = args.iter().position(|s| s == "--vslogger-level")
    {
        if args.len() >= level_index + 1 {
            LevelFilter::from_str(&args[level_index + 1]).unwrap_or(LevelFilter::Warn)
        } else {
            LevelFilter::Warn
        }
    } else {
        LevelFilter::Warn
    };

    let logger_modules = if opts.vslogger_mod_list.is_null() {
        &[]
    } else {
        slice::from_raw_parts(opts.vslogger_mod_list, opts.vslogger_mod_size as usize)
    };

    *OUTPUT_LOG_HANDLER.lock().unwrap() = Some(callbacks.on_log_output);
    init_logger(logger_modules, logger_level);

    if let Err(reason) = redirect_stdout_stderr(callbacks.on_log_output) {
        warn!("Error redirecting stdout/stderr: {}", reason);
    }

    let coordinates = Coordinates::new(0, 0, opts.width, opts.height, opts.width, opts.height);

    let prefs = if opts.prefs.is_null() {
        None
    } else {
        Some((*opts.prefs).convert())
    };

    let opts = InitOptions {
        args,
        coordinates,
        prefs,
        density: opts.density,
        xr_discovery: None,
        gl_context_pointer: gl_context,
        native_display_pointer: display,
        surfman_integration: SurfmanIntegration::Widget(opts.native_widget),
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
}

#[cfg(any(
    target_os = "linux",
    all(target_os = "windows", not(feature = "no-wgl")),
    target_os = "macos"
))]
#[no_mangle]
pub extern "C" fn init_with_gl(
    opts: CInitOptions,
    wakeup: extern "C" fn(),
    callbacks: CHostCallbacks,
) {
    let gl = gl_glue::gl::init().unwrap();
    unsafe { init(opts, gl, None, None, wakeup, callbacks) }
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
pub extern "C" fn on_context_menu_closed(result: CContextMenuResult, item: u32) {
    debug!("on_context_menu_closed");
    call(|s| s.on_context_menu_closed(result.convert(item)));
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
pub extern "C" fn perform_updates() {
    debug!("perform_updates");
    // We might have allocated some memory to respond to a potential
    // request, from the embedder, for a copy of Servo's preferences.
    prefs::free_prefs();
    call(|s| s.perform_updates());
}

#[no_mangle]
pub extern "C" fn is_uri_valid(url: *const c_char) -> bool {
    debug!("is_uri_valid");
    let url = unsafe { CStr::from_ptr(url) };
    let url = url.to_str().expect("Can't read string");
    simpleservo::is_uri_valid(url)
}

#[no_mangle]
pub extern "C" fn load_uri(url: *const c_char) -> bool {
    debug!("load_url");
    let url = unsafe { CStr::from_ptr(url) };
    let url = url.to_str().expect("Can't read string");
    call(|s| Ok(s.load_uri(url).is_ok()))
}

#[no_mangle]
pub extern "C" fn clear_cache() {
    debug!("clear_cache");
    call(|s| s.clear_cache())
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
pub extern "C" fn mouse_move(x: f32, y: f32) {
    debug!("mouse_move");
    call(|s| s.mouse_move(x, y));
}

#[no_mangle]
pub extern "C" fn mouse_down(x: f32, y: f32, button: CMouseButton) {
    debug!("mouse_down");
    call(|s| s.mouse_down(x, y, button.convert()));
}

#[no_mangle]
pub extern "C" fn mouse_up(x: f32, y: f32, button: CMouseButton) {
    debug!("mouse_up");
    call(|s| s.mouse_up(x, y, button.convert()));
}

#[no_mangle]
pub extern "C" fn click(x: f32, y: f32) {
    debug!("click");
    call(|s| s.click(x, y));
}

#[no_mangle]
pub extern "C" fn key_down(name: *const c_char) {
    debug!("key_up");
    key_event(name, false);
}

#[no_mangle]
pub extern "C" fn key_up(name: *const c_char) {
    debug!("key_up");
    key_event(name, true);
}

fn key_event(name: *const c_char, up: bool) {
    let name = unsafe { CStr::from_ptr(name) };
    let name = match name.to_str() {
        Ok(name) => name,
        Err(..) => {
            warn!("Couldn't not read str");
            return;
        },
    };
    let key = Key::from_str(&name);
    if let Ok(key) = key {
        call(|s| if up { s.key_up(key) } else { s.key_down(key) });
    } else {
        warn!("Received unknown keys");
    }
}

#[no_mangle]
pub extern "C" fn media_session_action(action: CMediaSessionActionType) {
    debug!("media_session_action");
    call(|s| s.media_session_action(action.convert()));
}

#[no_mangle]
pub extern "C" fn change_visibility(visible: bool) {
    debug!("change_visibility");
    call(|s| s.change_visibility(visible));
}

#[no_mangle]
pub extern "C" fn ime_dismissed() {
    debug!("ime_dismissed");
    call(|s| s.ime_dismissed());
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
    fn on_load_started(&self) {
        debug!("on_load_started");
        (self.0.on_load_started)();
    }

    fn on_load_ended(&self) {
        debug!("on_load_ended");
        (self.0.on_load_ended)();
    }

    fn on_title_changed(&self, title: Option<String>) {
        debug!("on_title_changed");
        match title {
            None => (self.0.on_title_changed)(std::ptr::null()),
            Some(title) => {
                let title = CString::new(title).expect("Can't create string");
                (self.0.on_title_changed)(title.as_ptr());
            },
        };
    }

    fn on_allow_navigation(&self, url: String) -> bool {
        debug!("on_allow_navigation");
        let url = CString::new(url).expect("Can't create string");
        (self.0.on_allow_navigation)(url.as_ptr())
    }

    fn on_url_changed(&self, url: String) {
        debug!("on_url_changed");
        let url = CString::new(url).expect("Can't create string");
        (self.0.on_url_changed)(url.as_ptr());
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

    fn on_ime_show(
        &self,
        _input_type: InputMethodType,
        text: Option<(String, i32)>,
        multiline: bool,
        bounds: DeviceIntRect,
    ) {
        debug!("on_ime_show");
        let text_index = text.as_ref().map_or(0, |(_, i)| *i);
        let text = text.and_then(|(s, _)| CString::new(s).ok());
        let text_ptr = text
            .as_ref()
            .map(|cstr| cstr.as_ptr())
            .unwrap_or(std::ptr::null());
        (self.0.on_ime_show)(
            text_ptr,
            text_index,
            multiline,
            bounds.origin.x,
            bounds.origin.y,
            bounds.size.width,
            bounds.size.height,
        );
    }

    fn on_ime_hide(&self) {
        debug!("on_ime_hide");
        (self.0.on_ime_hide)();
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
        (self.0.set_clipboard_contents)(contents.as_ptr());
    }

    fn on_media_session_metadata(&self, title: String, artist: String, album: String) {
        debug!(
            "on_media_session_metadata ({:?} {:?} {:?})",
            title, artist, album
        );
        let title = CString::new(title).expect("Can't create string");
        let artist = CString::new(artist).expect("Can't create string");
        let album = CString::new(album).expect("Can't create string");
        (self.0.on_media_session_metadata)(title.as_ptr(), artist.as_ptr(), album.as_ptr());
    }

    fn on_media_session_playback_state_change(&self, state: MediaSessionPlaybackState) {
        debug!("on_media_session_playback_state_change {:?}", state);
        (self.0.on_media_session_playback_state_change)(state.into());
    }

    fn on_media_session_set_position_state(
        &self,
        duration: f64,
        position: f64,
        playback_rate: f64,
    ) {
        debug!(
            "on_media_session_set_position_state ({:?} {:?} {:?})",
            duration, position, playback_rate
        );
        (self.0.on_media_session_set_position_state)(duration, position, playback_rate);
    }

    fn prompt_alert(&self, message: String, trusted: bool) {
        debug!("prompt_alert");
        let message = CString::new(message).expect("Can't create string");
        (self.0.prompt_alert)(message.as_ptr(), trusted);
    }

    fn prompt_ok_cancel(&self, message: String, trusted: bool) -> PromptResult {
        debug!("prompt_ok_cancel");
        let message = CString::new(message).expect("Can't create string");
        (self.0.prompt_ok_cancel)(message.as_ptr(), trusted).convert()
    }

    fn prompt_yes_no(&self, message: String, trusted: bool) -> PromptResult {
        debug!("prompt_yes_no");
        let message = CString::new(message).expect("Can't create string");
        (self.0.prompt_yes_no)(message.as_ptr(), trusted).convert()
    }

    fn prompt_input(&self, message: String, default: String, trusted: bool) -> Option<String> {
        debug!("prompt_input");
        let message = CString::new(message).expect("Can't create string");
        let default = CString::new(default).expect("Can't create string");
        let raw_contents = (self.0.prompt_input)(message.as_ptr(), default.as_ptr(), trusted);
        if raw_contents.is_null() {
            return None;
        }
        let c_str = unsafe { CStr::from_ptr(raw_contents) };
        let contents_str = c_str.to_str().expect("Can't create str");
        Some(contents_str.to_owned())
    }

    fn on_panic(&self, reason: String, details: Option<String>) {
        report_panic(&reason, details);
    }

    fn on_devtools_started(&self, port: Result<u16, ()>, token: String) {
        let token = CString::new(token).expect("Can't create string");
        match port {
            Ok(p) => {
                info!("Devtools Server running on port {}", p);
                (self.0.on_devtools_started)(
                    CDevtoolsServerState::Started,
                    p.into(),
                    token.as_ptr(),
                );
            },
            Err(()) => {
                error!("Error running devtools server");
                (self.0.on_devtools_started)(CDevtoolsServerState::Error, 0, token.as_ptr());
            },
        }
    }

    fn show_context_menu(&self, title: Option<String>, items: Vec<String>) {
        debug!("show_context_menu");
        let items_size = items.len() as u32;
        let cstrs: Vec<CString> = items
            .into_iter()
            .map(|i| CString::new(i).expect("Can't create string"))
            .collect();
        let items: Vec<*const c_char> = cstrs.iter().map(|cstr| cstr.as_ptr()).collect();
        let title = title.map(|s| CString::new(s).expect("Can't create string"));
        let title_ptr = title
            .as_ref()
            .map(|cstr| cstr.as_ptr())
            .unwrap_or(std::ptr::null());
        (self.0.show_context_menu)(title_ptr, items.as_ptr(), items_size);
    }
}
