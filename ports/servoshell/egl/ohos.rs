/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
#![allow(non_snake_case)]

use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::os::raw::c_void;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{LazyLock, Once, OnceLock, mpsc};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use keyboard_types::Key;
use log::{LevelFilter, debug, error, info, trace, warn};
use napi_derive_ohos::{module_exports, napi};
use napi_ohos::bindgen_prelude::Function;
use napi_ohos::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_ohos::{Env, JsObject, JsString, NapiRaw};
use ohos_ime::{AttachOptions, Ime, ImeProxy, RawTextEditorProxy};
use ohos_ime_sys::types::InputMethod_EnterKeyType;
use servo::style::Zero;
use servo::{
    AlertResponse, EventLoopWaker, InputMethodType, LoadStatus, MediaSessionPlaybackState,
    PermissionRequest, SimpleDialog, WebView,
};
use xcomponent_sys::{
    OH_NativeXComponent, OH_NativeXComponent_Callback, OH_NativeXComponent_GetKeyEvent,
    OH_NativeXComponent_GetKeyEventAction, OH_NativeXComponent_GetKeyEventCode,
    OH_NativeXComponent_GetTouchEvent, OH_NativeXComponent_GetXComponentOffset,
    OH_NativeXComponent_GetXComponentSize, OH_NativeXComponent_KeyAction,
    OH_NativeXComponent_KeyCode, OH_NativeXComponent_KeyEvent,
    OH_NativeXComponent_RegisterCallback, OH_NativeXComponent_RegisterKeyEventCallback,
    OH_NativeXComponent_TouchEvent, OH_NativeXComponent_TouchEventType,
};

use super::app_state::{Coordinates, RunningAppState};
use super::host_trait::HostTrait;

mod resources;
mod simpleservo;

#[napi(object)]
#[derive(Debug)]
pub struct InitOpts {
    pub url: String,
    pub device_type: String,
    pub os_full_name: String,
    /// Path to application data bundled with the servo app, e.g. web-pages.
    pub resource_dir: String,
    pub cache_dir: String,
    pub display_density: f64,
    pub commandline_args: String,
}

#[derive(Debug)]
enum CallError {
    ChannelNotInitialized,
    ChannelDied,
}

fn call(action: ServoAction) -> Result<(), CallError> {
    let tx = SERVO_CHANNEL
        .get()
        .ok_or(CallError::ChannelNotInitialized)?;
    tx.send(action).map_err(|_| CallError::ChannelDied)?;
    Ok(())
}

#[repr(transparent)]
struct XComponentWrapper(*mut OH_NativeXComponent);
#[repr(transparent)]
struct WindowWrapper(*mut c_void);
unsafe impl Send for XComponentWrapper {}
unsafe impl Send for WindowWrapper {}

#[derive(Clone, Copy, Debug)]
pub(super) enum TouchEventType {
    Down,
    Up,
    Move,
    Cancel,
    Unknown,
}

#[derive(Debug)]
pub(super) enum ServoAction {
    WakeUp,
    LoadUrl(String),
    GoBack,
    GoForward,
    TouchEvent {
        kind: TouchEventType,
        x: f32,
        y: f32,
        pointer_id: i32,
    },
    KeyUp(Key),
    KeyDown(Key),
    InsertText(String),
    ImeDeleteForward(usize),
    ImeDeleteBackward(usize),
    ImeSendEnter,
    Initialize(Box<InitOpts>),
    Vsync,
    Resize {
        width: i32,
        height: i32,
    },
}

/// Queue length for the thread-safe function to submit URL updates to ArkTS
const UPDATE_URL_QUEUE_SIZE: usize = 1;
/// Queue length for the thread-safe function to submit prompts to ArkTS
///
/// We can submit alerts in a non-blocking fashion, but alerts will always come from the
/// embedder thread. Specifying 4 as a max queue size seems reasonable for now, and can
/// be adjusted later.
const PROMPT_QUEUE_SIZE: usize = 4;
// Todo: Need to check if OnceLock is suitable, or if the TS function can be destroyed, e.g.
// if the activity gets suspended.
static SET_URL_BAR_CB: OnceLock<
    ThreadsafeFunction<String, (), String, false, false, UPDATE_URL_QUEUE_SIZE>,
> = OnceLock::new();
static TERMINATE_CALLBACK: OnceLock<ThreadsafeFunction<(), (), (), false, false, 1>> =
    OnceLock::new();
static PROMPT_TOAST: OnceLock<
    ThreadsafeFunction<String, (), String, false, false, PROMPT_QUEUE_SIZE>,
> = OnceLock::new();

impl ServoAction {
    fn dispatch_touch_event(
        servo: &RunningAppState,
        kind: TouchEventType,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) {
        match kind {
            TouchEventType::Down => servo.touch_down(x, y, pointer_id),
            TouchEventType::Up => servo.touch_up(x, y, pointer_id),
            TouchEventType::Move => servo.touch_move(x, y, pointer_id),
            TouchEventType::Cancel => servo.touch_cancel(x, y, pointer_id),
            TouchEventType::Unknown => warn!("Can't dispatch Unknown Touch Event"),
        }
    }

    // todo: consider making this take `self`, so we don't need to needlessly clone.
    fn do_action(&self, servo: &RunningAppState) {
        use ServoAction::*;
        match self {
            WakeUp => servo.perform_updates(),
            LoadUrl(url) => servo.load_uri(url.as_str()),
            GoBack => servo.go_back(),
            GoForward => servo.go_forward(),
            TouchEvent {
                kind,
                x,
                y,
                pointer_id,
            } => Self::dispatch_touch_event(servo, *kind, *x, *y, *pointer_id),
            KeyUp(k) => servo.key_up(k.clone()),
            KeyDown(k) => servo.key_down(k.clone()),
            InsertText(text) => servo.ime_insert_text(text.clone()),
            ImeDeleteForward(len) => {
                for _ in 0..*len {
                    servo.key_down(Key::Delete);
                    servo.key_up(Key::Delete);
                }
            },
            ImeDeleteBackward(len) => {
                for _ in 0..*len {
                    servo.key_down(Key::Backspace);
                    servo.key_up(Key::Backspace);
                }
            },
            ImeSendEnter => {
                servo.key_down(Key::Enter);
                servo.key_up(Key::Enter);
            },
            Initialize(_init_opts) => {
                panic!("Received Initialize event, even though servo is already initialized")
            },
            Vsync => {
                servo.notify_vsync();
                servo.present_if_needed();
            },
            Resize { width, height } => servo.resize(Coordinates::new(0, 0, *width, *height)),
        };
    }
}

/// Vsync callback
///
/// # Safety
///
/// The caller should pass a valid raw NativeVsync object to us via
/// `native_vsync.request_raw_callback_with_self(Some(on_vsync_cb))`
unsafe extern "C" fn on_vsync_cb(
    timestamp: ::core::ffi::c_longlong,
    data: *mut ::core::ffi::c_void,
) {
    trace!("Vsync callback at time {timestamp}");
    // SAFETY: We require the function registering us as a callback provides a valid
    //  `OH_NativeVSync` object. We do not use `data` after this point.
    let native_vsync = unsafe { ohos_vsync::NativeVsync::from_raw(data.cast()) };
    call(ServoAction::Vsync).unwrap();
    // Todo: Do we have a callback for when the frame finished rendering?
    unsafe {
        native_vsync
            .request_raw_callback_with_self(Some(on_vsync_cb))
            .unwrap();
    }
}

static SERVO_CHANNEL: OnceLock<Sender<ServoAction>> = OnceLock::new();

#[unsafe(no_mangle)]
extern "C" fn on_surface_created_cb(xcomponent: *mut OH_NativeXComponent, window: *mut c_void) {
    info!("on_surface_created_cb");
    #[cfg(feature = "tracing-hitrace")]
    let _ = hitrace::ScopedTrace::start_trace(&c"on_surface_created_cb");

    let xc_wrapper = XComponentWrapper(xcomponent);
    let window_wrapper = WindowWrapper(window);

    // Todo: Perhaps it would be better to move this thread into the vsync signal thread.
    // This would allow us to save one thread and the IPC for the vsync signal.
    //
    // Each thread will send its id via the channel
    let _main_surface_thread = thread::spawn(move || {
        let (tx, rx): (Sender<ServoAction>, Receiver<ServoAction>) = mpsc::channel();

        SERVO_CHANNEL
            .set(tx.clone())
            .expect("Servo channel already initialized");

        let wakeup = Box::new(WakeupCallback::new(tx));
        let callbacks = Box::new(HostCallbacks::new());

        let xc = xc_wrapper;
        let window = window_wrapper;
        let init_opts = if let Ok(ServoAction::Initialize(init_opts)) = rx.recv() {
            init_opts
        } else {
            panic!("Servos GL thread received another event before it was initialized")
        };
        let servo = simpleservo::init(*init_opts, window.0, xc.0, wakeup, callbacks)
            .expect("Servo initialization failed");

        info!("Surface created!");
        let native_vsync =
            ohos_vsync::NativeVsync::new("ServoVsync").expect("Failed to create NativeVsync");
        // get_period() returns an error - perhaps we need to wait until the first callback?
        // info!("Native vsync period is {} nanoseconds", native_vsync.get_period().unwrap());
        unsafe {
            native_vsync
                .request_raw_callback_with_self(Some(on_vsync_cb))
                .expect("Failed to request vsync callback")
        }
        info!("Enabled Vsync!");

        while let Ok(action) = rx.recv() {
            trace!("Wakeup message received!");
            action.do_action(&servo);
        }

        info!("Sender disconnected - Terminating main surface thread");
    });

    info!("Returning from on_surface_created_cb");
}

/// Returns the offset of the surface relative to its parent's top left corner
///
/// # Safety
///
/// `xcomponent` and `native_window` must be valid, non-null and aligned pointers to a
/// live xcomponent and associated native window surface.
unsafe fn get_xcomponent_offset(
    xcomponent: *mut OH_NativeXComponent,
    native_window: *mut c_void,
) -> Result<(i32, i32), i32> {
    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;

    let result = unsafe {
        OH_NativeXComponent_GetXComponentOffset(xcomponent, native_window, &raw mut x, &raw mut y)
    };
    if result != 0 {
        error!("OH_NativeXComponent_GetXComponentOffset failed with {result}");
        return Err(result);
    }

    Ok((
        (x.round() as i64).try_into().expect("X offset too large"),
        (y.round() as i64).try_into().expect("Y offset too large"),
    ))
}

/// Returns the size of the surface
///
/// # Safety
///
/// `xcomponent` and `native_window` must be valid, non-null and aligned pointers to a
/// live xcomponent and associated native window surface.
unsafe fn get_xcomponent_size(
    xcomponent: *mut OH_NativeXComponent,
    native_window: *mut c_void,
) -> Result<euclid::default::Size2D<i32>, i32> {
    let mut width: u64 = 0;
    let mut height: u64 = 0;
    let result = unsafe {
        OH_NativeXComponent_GetXComponentSize(
            xcomponent,
            native_window,
            &raw mut width,
            &raw mut height,
        )
    };
    if result != 0 {
        debug!("OH_NativeXComponent_GetXComponentSize failed with {result}");
        return Err(result);
    }

    let width: i32 = width.try_into().expect("Width too large");
    let height: i32 = height.try_into().expect("Height too large");
    Ok(euclid::Size2D::new(width, height))
}

extern "C" fn on_surface_changed_cb(
    xcomponent: *mut OH_NativeXComponent,
    native_window: *mut c_void,
) {
    debug!("on_surface_changed_cb: xc: {xcomponent:?}, window: {native_window:?}");
    // SAFETY: We just obtained these pointers from the callback, so we can assume them to be valid.
    if let Ok(size) = unsafe { get_xcomponent_size(xcomponent, native_window) } {
        info!("on_surface_changed_cb: Resizing to {size:?}");
        call(ServoAction::Resize {
            width: size.width,
            height: size.height,
        })
        .unwrap();
    } else {
        error!("on_surface_changed_cb: Surface changed, but failed to obtain new size")
    }
}

extern "C" fn on_surface_destroyed_cb(_component: *mut OH_NativeXComponent, _window: *mut c_void) {
    error!("on_surface_destroyed_cb is currently not implemented");
}

extern "C" fn on_dispatch_touch_event_cb(component: *mut OH_NativeXComponent, window: *mut c_void) {
    info!("DispatchTouchEvent");
    let mut touch_event: MaybeUninit<OH_NativeXComponent_TouchEvent> = MaybeUninit::uninit();
    let res =
        unsafe { OH_NativeXComponent_GetTouchEvent(component, window, touch_event.as_mut_ptr()) };
    if res != 0 {
        error!("OH_NativeXComponent_GetTouchEvent failed with {res}");
        return;
    }
    let touch_event = unsafe { touch_event.assume_init() };
    let kind: TouchEventType = match touch_event.type_ {
        OH_NativeXComponent_TouchEventType::OH_NATIVEXCOMPONENT_DOWN => TouchEventType::Down,
        OH_NativeXComponent_TouchEventType::OH_NATIVEXCOMPONENT_UP => TouchEventType::Up,
        OH_NativeXComponent_TouchEventType::OH_NATIVEXCOMPONENT_MOVE => TouchEventType::Move,
        OH_NativeXComponent_TouchEventType::OH_NATIVEXCOMPONENT_CANCEL => TouchEventType::Cancel,
        _ => {
            error!(
                "Failed to dispatch call for touch Event {:?}",
                touch_event.type_
            );
            TouchEventType::Unknown
        },
    };
    if let Err(e) = call(ServoAction::TouchEvent {
        kind,
        x: touch_event.x,
        y: touch_event.y,
        pointer_id: touch_event.id,
    }) {
        error!("Failed to dispatch call for touch Event {kind:?}: {e:?}");
    }
}

extern "C" fn on_dispatch_key_event(xc: *mut OH_NativeXComponent, _window: *mut c_void) {
    info!("DispatchKeyEvent");
    let mut event: *mut OH_NativeXComponent_KeyEvent = core::ptr::null_mut();
    let res = unsafe { OH_NativeXComponent_GetKeyEvent(xc, &mut event as *mut *mut _) };
    assert_eq!(res, 0);

    let mut action = OH_NativeXComponent_KeyAction::OH_NATIVEXCOMPONENT_KEY_ACTION_UNKNOWN;
    let res = unsafe { OH_NativeXComponent_GetKeyEventAction(event, &mut action as *mut _) };
    assert_eq!(res, 0);

    let mut keycode = OH_NativeXComponent_KeyCode::KEY_UNKNOWN;
    let res = unsafe { OH_NativeXComponent_GetKeyEventCode(event, &mut keycode as *mut _) };
    assert_eq!(res, 0);

    // Simplest possible impl, just for testing purposes
    let code: keyboard_types::Code = keycode.into();
    // There currently doesn't seem to be an API to query keymap / keyboard layout, so
    // we don't even bother implementing modifier support for now, since we expect to be using the
    // IME most of the time anyway. We can revisit this when someone has an OH device with a
    // physical keyboard.
    let char = code.to_string();
    let key = Key::Character(char);
    match action {
        OH_NativeXComponent_KeyAction::OH_NATIVEXCOMPONENT_KEY_ACTION_UP => {
            call(ServoAction::KeyUp(key)).expect("Call failed")
        },
        OH_NativeXComponent_KeyAction::OH_NATIVEXCOMPONENT_KEY_ACTION_DOWN => {
            call(ServoAction::KeyDown(key)).expect("Call failed")
        },
        _ => error!("Unknown key action {:?}", action),
    }
}

static LOGGER: LazyLock<hilog::Logger> = LazyLock::new(|| {
    let mut builder = hilog::Builder::new();
    builder.set_domain(hilog::LogDomain::new(0xE0C3));
    let filters = [
        "fonts",
        "servo",
        "servoshell",
        "servoshell::egl:gl_glue",
        // Show redirected stdout / stderr by default
        "servoshell::egl::log",
        // Show JS errors by default.
        "script::dom::bindings::error",
        // Show GL errors by default.
        "canvas::webgl_thread",
        "compositing::compositor",
        "compositing::touch",
        "constellation::constellation",
        "ohos_ime",
    ];
    for &module in &filters {
        builder.filter_module(module, log::LevelFilter::Debug);
    }

    builder.filter_level(LevelFilter::Warn).build()
});

fn initialize_logging_once() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let logger: &'static hilog::Logger = &LOGGER;
        let max_level = logger.filter();
        let r = log::set_logger(logger).and_then(|()| Ok(log::set_max_level(max_level)));
        debug!("Attempted to register the logger: {r:?} and set max level to: {max_level}");
        info!("Servo Register callback called!");

        std::panic::set_hook(Box::new(|info| {
            error!("Panic in Rust code");
            error!("PanicInfo: {info}");
            let msg = match info.payload().downcast_ref::<&'static str>() {
                Some(s) => *s,
                None => match info.payload().downcast_ref::<String>() {
                    Some(s) => &**s,
                    None => "Box<Any>",
                },
            };
            let current_thread = thread::current();
            let name = current_thread.name().unwrap_or("<unnamed>");
            if let Some(location) = info.location() {
                error!(
                    "{} (thread {}, at {}:{})",
                    msg,
                    name,
                    location.file(),
                    location.line()
                );
            } else {
                error!("{} (thread {})", msg, name);
            }

            crate::backtrace::print_ohos();
        }));

        // We only redirect stdout and stderr for non-production builds, since it is
        // only used for debugging purposes. This saves us one thread in production.
        #[cfg(not(servo_production))]
        if let Err(e) = super::log::redirect_stdout_and_stderr() {
            error!("Failed to redirect stdout and stderr to hilog due to: {e:?}");
        }
    });
}

pub fn set_log_filter(filter: Option<&str>) {
    let Some(filter) = filter else {
        debug!("Called ohos::set_log_filter without providing a filter");
        return;
    };

    debug!("Updating log filter to {filter}");
    let mut builder = env_filter::Builder::new();
    let filter = match builder.try_parse(filter) {
        Result::Ok(filter) => filter,
        Result::Err(err) => {
            error!("Failed to parse log filter: {err}");
            return;
        },
    };
    let filter = filter.build();
    (*LOGGER).set_filter(filter);
}

fn register_xcomponent_callbacks(env: &Env, xcomponent: &JsObject) -> napi_ohos::Result<()> {
    info!("napi_get_named_property call successfull");
    let raw = unsafe { xcomponent.raw() };
    let raw_env = env.raw();
    let mut nativeXComponent: *mut OH_NativeXComponent = core::ptr::null_mut();
    unsafe {
        let res = napi_ohos::sys::napi_unwrap(
            raw_env,
            raw,
            &mut nativeXComponent as *mut *mut OH_NativeXComponent as *mut *mut c_void,
        );
        assert!(res.is_zero());
    }
    info!("Got nativeXComponent!");
    let cbs = Box::new(OH_NativeXComponent_Callback {
        OnSurfaceCreated: Some(on_surface_created_cb),
        OnSurfaceChanged: Some(on_surface_changed_cb),
        OnSurfaceDestroyed: Some(on_surface_destroyed_cb),
        DispatchTouchEvent: Some(on_dispatch_touch_event_cb),
    });
    let res =
        unsafe { OH_NativeXComponent_RegisterCallback(nativeXComponent, Box::leak(cbs) as *mut _) };
    if res != 0 {
        error!("Failed to register callbacks");
    } else {
        info!("Registered callbacks successfully");
    }

    let res = unsafe {
        OH_NativeXComponent_RegisterKeyEventCallback(nativeXComponent, Some(on_dispatch_key_event))
    };
    if res != 0 {
        error!("Failed to register key event callbacks");
    } else {
        debug!("Registered key event callbacks successfully");
    }
    Ok(())
}

#[allow(unused)]
fn debug_jsobject(obj: &JsObject, obj_name: &str) -> napi_ohos::Result<()> {
    let names = obj.get_property_names()?;
    error!("Getting property names of object {obj_name}");
    let len = names.get_array_length()?;
    error!("{obj_name} has {len} elements");
    for i in 0..len {
        let name: JsString = names.get_element(i)?;
        let name = name.into_utf8()?;
        error!("{obj_name} property {i}: {}", name.as_str()?)
    }
    Ok(())
}

#[module_exports]
fn init(exports: JsObject, env: Env) -> napi_ohos::Result<()> {
    initialize_logging_once();
    info!("simpleservo init function called");
    if let Ok(xcomponent) = exports.get_named_property::<JsObject>("__NATIVE_XCOMPONENT_OBJ__") {
        register_xcomponent_callbacks(&env, &xcomponent)?;
    }

    info!("Finished init");
    Ok(())
}

#[napi(js_name = "loadURL")]
pub fn load_url(url: String) {
    debug!("load url");
    call(ServoAction::LoadUrl(url)).expect("Failed to load url");
}

#[napi]
pub fn go_back() {
    call(ServoAction::GoBack).expect("Failed to call servo");
}

#[napi]
pub fn go_forward() {
    call(ServoAction::GoForward).expect("Failed to call servo");
}

#[napi(js_name = "registerURLcallback")]
pub fn register_url_callback(callback: Function<String, ()>) -> napi_ohos::Result<()> {
    debug!("register_url_callback called!");
    let tsfn_builder = callback.build_threadsafe_function();
    let function = tsfn_builder
        .max_queue_size::<UPDATE_URL_QUEUE_SIZE>()
        .build()?;

    SET_URL_BAR_CB.set(function).map_err(|_e| {
        napi_ohos::Error::from_reason(
            "Failed to set URL callback - register_url_callback called twice?",
        )
    })
}

#[napi(js_name = "registerTerminateCallback")]
pub fn register_terminate_callback(callback: Function<(), ()>) -> napi_ohos::Result<()> {
    let tsfn_builder = callback.build_threadsafe_function();
    let function = tsfn_builder.max_queue_size::<1>().build()?;
    TERMINATE_CALLBACK
        .set(function)
        .map_err(|_| napi_ohos::Error::from_reason("Failed to set terminate function"))
}

#[napi]
pub fn register_prompt_toast_callback(callback: Function<String, ()>) -> napi_ohos::Result<()> {
    debug!("register_prompt_toast_callback called!");
    let tsfn_builder = callback.build_threadsafe_function();
    let function = tsfn_builder.max_queue_size::<PROMPT_QUEUE_SIZE>().build()?;

    PROMPT_TOAST
        .set(function)
        .map_err(|_e| napi_ohos::Error::from_reason("Failed to set prompt toast callback"))
}

#[napi]
pub fn init_servo(init_opts: InitOpts) -> napi_ohos::Result<()> {
    info!("Servo is being initialised with the following Options: ");
    info!(
        "Device Type: {}, DisplayDensity: {}",
        init_opts.device_type, init_opts.display_density
    );
    info!("Initial URL: {}", init_opts.url);
    let channel = if let Some(channel) = SERVO_CHANNEL.get() {
        channel
    } else {
        warn!("Servo GL thread has not initialized yet. Retrying.....");
        let mut iter_count = 0;
        loop {
            if let Some(channel) = SERVO_CHANNEL.get() {
                break channel;
            } else {
                iter_count += 1;
                if iter_count > 10 {
                    error!("Servo GL thread not reachable");
                    panic!("Servo GL thread not reachable");
                }
                sleep(Duration::from_millis(100));
            }
        }
    };
    channel
        .send(ServoAction::Initialize(Box::new(init_opts)))
        .expect("Failed to connect to servo GL thread");
    Ok(())
}

struct OhosImeOptions {
    input_type: ohos_ime_sys::types::InputMethod_TextInputType,
    enterkey_type: InputMethod_EnterKeyType,
}

/// TODO: This needs some more consideration and perhaps both more information from
/// servos side as well as clarification on the meaning of some of the openharmony variants.
/// For now for example we just ignore the `multiline` parameter in all the cases where it
/// seems like it wouldn't make sense, but this needs a closer look.
fn convert_ime_options(input_method_type: InputMethodType, multiline: bool) -> OhosImeOptions {
    use ohos_ime_sys::types::InputMethod_TextInputType as IME_TextInputType;
    // There are a couple of cases when the mapping is not quite clear to me,
    // so we clearly mark them with `input_fallback` and come back to this later.
    let input_fallback = IME_TextInputType::IME_TEXT_INPUT_TYPE_TEXT;
    let input_type = match input_method_type {
        InputMethodType::Color => input_fallback,
        InputMethodType::Date => input_fallback,
        InputMethodType::DatetimeLocal => IME_TextInputType::IME_TEXT_INPUT_TYPE_DATETIME,
        InputMethodType::Email => IME_TextInputType::IME_TEXT_INPUT_TYPE_EMAIL_ADDRESS,
        InputMethodType::Month => input_fallback,
        InputMethodType::Number => IME_TextInputType::IME_TEXT_INPUT_TYPE_NUMBER,
        // There is no type "password", but "new password" seems closest.
        InputMethodType::Password => IME_TextInputType::IME_TEXT_INPUT_TYPE_NEW_PASSWORD,
        InputMethodType::Search => IME_TextInputType::IME_TEXT_INPUT_TYPE_TEXT,
        InputMethodType::Tel => IME_TextInputType::IME_TEXT_INPUT_TYPE_PHONE,
        InputMethodType::Text => {
            if multiline {
                IME_TextInputType::IME_TEXT_INPUT_TYPE_MULTILINE
            } else {
                IME_TextInputType::IME_TEXT_INPUT_TYPE_TEXT
            }
        },
        InputMethodType::Time => input_fallback,
        InputMethodType::Url => IME_TextInputType::IME_TEXT_INPUT_TYPE_URL,
        InputMethodType::Week => input_fallback,
    };
    let enterkey_type = match (input_method_type, multiline) {
        (InputMethodType::Text, true) => InputMethod_EnterKeyType::IME_ENTER_KEY_NEWLINE,
        (InputMethodType::Text, false) => InputMethod_EnterKeyType::IME_ENTER_KEY_DONE,
        (InputMethodType::Search, false) => InputMethod_EnterKeyType::IME_ENTER_KEY_SEARCH,
        _ => InputMethod_EnterKeyType::IME_ENTER_KEY_UNSPECIFIED,
    };
    OhosImeOptions {
        input_type,
        enterkey_type,
    }
}

#[derive(Clone)]
pub struct WakeupCallback {
    chan: Sender<ServoAction>,
}

impl WakeupCallback {
    pub(crate) fn new(chan: Sender<ServoAction>) -> Self {
        WakeupCallback { chan }
    }
}

impl EventLoopWaker for WakeupCallback {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(self.clone())
    }

    fn wake(&self) {
        log::trace!("wake called!");
        self.chan.send(ServoAction::WakeUp).unwrap_or_else(|e| {
            error!("Failed to send wake message with: {e}");
        });
    }
}

struct HostCallbacks {
    ime_proxy: RefCell<Option<ohos_ime::ImeProxy>>,
}

impl HostCallbacks {
    pub fn new() -> Self {
        HostCallbacks {
            ime_proxy: RefCell::new(None),
        }
    }

    pub fn show_alert(&self, message: String) {
        match PROMPT_TOAST.get() {
            Some(prompt_fn) => {
                let status = prompt_fn.call(message, ThreadsafeFunctionCallMode::NonBlocking);
                if status != napi_ohos::Status::Ok {
                    // Queue could be full.
                    error!("show_alert failed with {status}");
                }
            },
            None => error!("PROMPT_TOAST not set. Dropping message {message}"),
        }
    }
}

struct ServoIme {
    text_config: ohos_ime::TextConfig,
}
impl Ime for ServoIme {
    fn insert_text(&self, text: String) {
        call(ServoAction::InsertText(text)).unwrap()
    }
    fn delete_forward(&self, len: usize) {
        call(ServoAction::ImeDeleteForward(len)).unwrap()
    }
    fn delete_backward(&self, len: usize) {
        call(ServoAction::ImeDeleteBackward(len)).unwrap()
    }

    fn get_text_config(&self) -> &ohos_ime::TextConfig {
        &self.text_config
    }

    fn send_enter_key(&self, _enter_key: InputMethod_EnterKeyType) {
        call(ServoAction::ImeSendEnter).unwrap()
    }
}

#[allow(unused)]
impl HostTrait for HostCallbacks {
    fn request_permission(&self, _webview: WebView, request: PermissionRequest) {
        warn!("Permissions prompt not implemented. Denied.");
        request.deny();
    }

    fn show_simple_dialog(&self, _webview: WebView, dialog: SimpleDialog) {
        let _ = match dialog {
            SimpleDialog::Alert {
                message,
                response_sender,
            } => {
                debug!("SimpleDialog::Alert");
                // TODO: Indicate that this message is untrusted, and what origin it came from.
                self.show_alert(message);
                response_sender.send(AlertResponse::Ok)
            },
            SimpleDialog::Confirm {
                message,
                response_sender,
            } => {
                warn!("Confirm dialog not implemented. Cancelled. {}", message);
                response_sender.send(Default::default())
            },
            SimpleDialog::Prompt {
                message,
                response_sender,
                ..
            } => {
                warn!("Prompt dialog not implemented. Cancelled. {}", message);
                response_sender.send(Default::default())
            },
        };
    }

    fn show_context_menu(&self, title: Option<String>, items: Vec<String>) {
        warn!("show_context_menu not implemented")
    }

    fn notify_load_status_changed(&self, load_status: LoadStatus) {
        // Note: It seems that we don't necessarily get 1 `LoadStatus::Complete` for each
        // each time `LoadStatus::Started` is called. Presumably this requires some API changes,
        // e.g. including webview id, perhaps URL and some additional investigation effort.
        // For now we just add a trace event here, so that we can see in the trace if we
        // successfully loaded **a** page.
        if load_status == LoadStatus::Complete {
            #[cfg(feature = "tracing-hitrace")]
            let _scope = hitrace::ScopedTrace::start_trace(&c"PageLoadEndedPrompt");
            self.show_alert("Page finished loading!".to_string());
        } else {
            #[cfg(feature = "tracing-hitrace")]
            let _ = hitrace::ScopedTrace::start_trace_str(&format!(
                "load status changed {:?}",
                load_status
            ));
        }
    }

    fn on_title_changed(&self, title: Option<String>) {
        warn!("on_title_changed not implemented")
    }

    fn on_allow_navigation(&self, url: String) -> bool {
        true
    }

    fn on_url_changed(&self, url: String) {
        debug!("Hosttrait `on_url_changed` called with new url: {url}");
        match SET_URL_BAR_CB.get() {
            Some(update_url_fn) => {
                let status = update_url_fn.call(url, ThreadsafeFunctionCallMode::Blocking);
                if status != napi_ohos::Status::Ok {
                    error!("on_url_changed failed with {status}");
                }
            },
            None => error!("`on_url_changed` called without a registered callback"),
        }
    }

    fn on_history_changed(&self, can_go_back: bool, can_go_forward: bool) {}

    fn on_animating_changed(&self, animating: bool) {
        // todo: should we tell the vsync thread that it should perform updates?
    }

    fn on_shutdown_complete(&self) {
        if let Some(terminate_fn) = TERMINATE_CALLBACK.get() {
            terminate_fn.call((), ThreadsafeFunctionCallMode::Blocking);
        } else {
            error!("Could not shut down despite servo shutting down");
        }
    }

    /// Shows the Inputmethod
    ///
    /// Most basic implementation for now, which just ignores all the input parameters
    /// and shows the soft keyboard with default settings.
    fn on_ime_show(
        &self,
        input_type: InputMethodType,
        _text: Option<(String, i32)>,
        multiline: bool,
        _bounds: servo::webrender_api::units::DeviceIntRect,
    ) {
        debug!("IME show!");
        let mut ime_proxy = self.ime_proxy.borrow_mut();
        let ime = ime_proxy.get_or_insert_with(|| {
            let attach_options = AttachOptions::new(true);
            let configbuilder = ohos_ime::TextConfigBuilder::new();
            let options = convert_ime_options(input_type, multiline);
            let text_config = configbuilder
                .input_type(options.input_type)
                .enterkey_type(options.enterkey_type)
                .build();
            let editor = RawTextEditorProxy::new(Box::new(ServoIme { text_config }));
            ImeProxy::new(editor, attach_options)
        });
        match ime.show_keyboard() {
            Ok(()) => debug!("IME show keyboard - success"),
            Err(_e) => error!("IME show keyboard error"),
        }
    }

    fn on_ime_hide(&self) {
        debug!("IME hide!");
        let mut ime_proxy = self.ime_proxy.take();
        if let Some(ime) = ime_proxy {
            match ime.hide_keyboard() {
                Ok(()) => debug!("IME hide keyboard - success"),
                Err(_e) => error!("IME hide keyboard error"),
            }
        } else {
            warn!("IME hide called, but no active IME found!")
        }
    }

    fn on_media_session_metadata(&self, title: String, artist: String, album: String) {
        warn!("on_media_session_metadata not implemented");
    }

    fn on_media_session_playback_state_change(&self, state: MediaSessionPlaybackState) {
        warn!("on_media_session_playback_state_change not implemented");
    }

    fn on_media_session_set_position_state(
        &self,
        duration: f64,
        position: f64,
        playback_rate: f64,
    ) {
        warn!("on_media_session_set_position_state not implemented");
    }

    fn on_panic(&self, reason: String, backtrace: Option<String>) {
        error!("Panic: {reason},");
        if let Some(bt) = backtrace {
            error!("Backtrace: {bt:?}")
        }
        self.show_alert("Servo crashed!".to_string());
        self.show_alert(reason);
    }
}
