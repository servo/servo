/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
#![allow(non_snake_case)]

mod resources;

use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::os::raw::c_void;
use std::path::PathBuf;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{LazyLock, Mutex, Once, OnceLock, mpsc};
use std::thread::sleep;
use std::time::Duration;
use std::{fs, thread};

use dpi::PhysicalSize;
use euclid::{Point2D, Rect, Size2D};
use keyboard_types::{Key, NamedKey};
use log::{LevelFilter, debug, error, info, trace, warn};
use napi_derive_ohos::napi;
use napi_ohos::bindgen_prelude::{Function, JsObjectValue, Object};
use napi_ohos::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_ohos::{Env, JsString, JsValue};
use ohos_abilitykit_sys::runtime::application_context;
use ohos_ime::{
    AttachOptions, CreateImeProxyError, CreateTextEditorProxyError, Ime, ImeProxy,
    RawTextEditorProxy,
};
use ohos_ime_sys::types::InputMethod_EnterKeyType;
use ohos_window_manager_sys::display_manager;
use raw_window_handle::{
    DisplayHandle, OhosDisplayHandle, OhosNdkWindowHandle, RawDisplayHandle, RawWindowHandle,
    WindowHandle,
};
use servo::{
    self, DevicePixel, EventLoopWaker, InputMethodControl, InputMethodType, LoadStatus,
    MediaSessionPlaybackState, PrefValue, WebViewId, WindowRenderingContext, Zero,
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

use super::app::{App, AppInitOptions, VsyncRefreshDriver};
use super::host_trait::HostTrait;
use crate::egl::ohos::resources::ResourceReaderInstance;
use crate::prefs::{ArgumentParsingResult, parse_command_line_arguments};

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
    ThreadsafeFunction<String, (), String, napi_ohos::Status, false, false, UPDATE_URL_QUEUE_SIZE>,
> = OnceLock::new();
static TERMINATE_CALLBACK: OnceLock<
    ThreadsafeFunction<(), (), (), napi_ohos::Status, false, false, 1>,
> = OnceLock::new();
static PROMPT_TOAST: OnceLock<
    ThreadsafeFunction<String, (), String, napi_ohos::Status, false, false, PROMPT_QUEUE_SIZE>,
> = OnceLock::new();

/// Currently we do not support different contexts for different windows but we might want to change tabs.
/// For this we store the window context for every tab and change the compositor by hand.
static NATIVE_WEBVIEWS: Mutex<Vec<NativeWebViewComponents>> = Mutex::new(Vec::new());

static SERVO_CHANNEL: OnceLock<Sender<ServoAction>> = OnceLock::new();

pub(crate) fn get_raw_window_handle(
    xcomponent: *mut OH_NativeXComponent,
    window: *mut c_void,
) -> (RawWindowHandle, Rect<i32, DevicePixel>) {
    let window_size = unsafe { get_xcomponent_size(xcomponent, window) }
        .expect("Could not get native window size");
    let window_origin = unsafe { get_xcomponent_offset(xcomponent, window) }
        .expect("Could not get native window offset");
    let viewport_rect = Rect::new(window_origin, window_size);
    let native_window = NonNull::new(window).expect("Could not get native window");
    let window_handle = RawWindowHandle::OhosNdk(OhosNdkWindowHandle::new(native_window));
    (window_handle, viewport_rect)
}

#[derive(Debug)]
struct NativeValues {
    cache_dir: String,
    display_density: f32,
    device_type: ohos_deviceinfo::OhosDeviceType,
    os_full_name: String,
}

/// Gets the resource and cache directory from the native c methods.
fn get_native_values() -> NativeValues {
    let cache_dir = {
        const BUFFER_SIZE: i32 = 100;
        let mut buffer: Vec<u8> = Vec::with_capacity(BUFFER_SIZE as usize);
        let mut write_length = 0;
        unsafe {
            application_context::OH_AbilityRuntime_ApplicationContextGetCacheDir(
                buffer.as_mut_ptr().cast(),
                BUFFER_SIZE,
                &mut write_length,
            )
            .expect("Call to cache dir failed");
            buffer.set_len(write_length as usize);
            String::from_utf8(buffer).expect("UTF-8")
        }
    };
    let display_density = unsafe {
        let mut density: f32 = 0_f32;
        display_manager::OH_NativeDisplayManager_GetDefaultDisplayDensityPixels(&mut density)
            .expect("Could not get displaydensity");
        density
    };

    NativeValues {
        cache_dir,
        display_density,
        device_type: ohos_deviceinfo::get_device_type(),
        os_full_name: String::from(ohos_deviceinfo::get_os_full_name().unwrap_or("Undefined")),
    }
}

/// Initialize the servoshell [`App`]. At that point, we need a valid GL context. In the
/// future, this will be done in multiple steps.
fn init_app(
    options: InitOpts,
    native_window: *mut c_void,
    xcomponent: *mut OH_NativeXComponent,
    event_loop_waker: Box<dyn EventLoopWaker>,
    host: Box<dyn HostTrait>,
) -> Result<Rc<App>, &'static str> {
    info!("Entered servoshell init function");
    crate::init_crypto();

    let native_values = get_native_values();
    info!("Device Type {:?}", native_values.device_type);
    info!("OS Full Name {:?}", native_values.os_full_name);
    info!("ResourceDir {:?}", options.resource_dir);

    let resource_dir = PathBuf::from(&options.resource_dir).join("servo");
    debug!("Resources are located at: {:?}", resource_dir);
    servo::resources::set(Box::new(ResourceReaderInstance::new(resource_dir.clone())));

    // It would be nice if `from_cmdline_args()` could accept str slices, to avoid allocations here.
    // Then again, this code could and maybe even should be disabled in production builds.
    let mut args = vec!["servoshell".to_string()];
    args.extend(
        options
            .commandline_args
            .split("\u{1f}")
            .map(|arg| arg.to_string()),
    );
    debug!("Servo commandline args: {:?}", args);

    let config_dir = PathBuf::from(&native_values.cache_dir).join("servo");
    debug!("Configs are located at: {:?}", config_dir);
    let _ = crate::prefs::DEFAULT_CONFIG_DIR
        .set(config_dir.clone())
        .inspect_err(|e| {
            warn!(
                "Default Prefs Dir already previously filled. Got error {}",
                e.display()
            );
        });

    // Ensure cache dir exists before copy `prefs.json`
    let _ = crate::prefs::default_config_dir().inspect(|path| {
        if !path.exists() {
            fs::create_dir_all(path).unwrap_or_else(|e| {
                log::error!("Failed to create config directory at {:?}: {:?}", path, e)
            })
        }
    });

    // Try copy `prefs.json` from {this.context.resource_prefsDir}/servo/
    // to `config_dir` if none exist
    let source_prefs = resource_dir.join("prefs.json");
    let target_prefs = config_dir.join("prefs.json");
    if !target_prefs.exists() && source_prefs.exists() {
        debug!("Copy {:?} to {:?}", source_prefs, target_prefs);
        fs::copy(&source_prefs, &target_prefs).unwrap_or_else(|e| {
            debug!("Copy failed! {:?}", e);
            0
        });
    }

    let (opts, mut preferences, servoshell_preferences) = match parse_command_line_arguments(args) {
        ArgumentParsingResult::ContentProcess(..) => {
            unreachable!("OHOS does not have support for multiprocess yet.")
        },
        ArgumentParsingResult::ChromeProcess(opts, preferences, servoshell_preferences) => {
            (opts, preferences, servoshell_preferences)
        },
        ArgumentParsingResult::Exit => std::process::exit(0),
        ArgumentParsingResult::ErrorParsing => std::process::exit(1),
    };

    if native_values.device_type == ohos_deviceinfo::OhosDeviceType::Phone {
        preferences.set_value("viewport_meta_enabled", PrefValue::Bool(true));
    }

    if servoshell_preferences.log_to_file {
        let mut servo_log = PathBuf::from(&native_values.cache_dir);
        servo_log.push("servo.log");
        if crate::egl::ohos::LOGGER.set_file_writer(servo_log).is_err() {
            warn!("Could not set log file");
        }
    }

    crate::init_tracing(servoshell_preferences.tracing_filter.as_deref());
    #[cfg(target_env = "ohos")]
    crate::egl::ohos::set_log_filter(servoshell_preferences.log_filter.as_deref());

    let (window_handle, viewport_rect) = get_raw_window_handle(xcomponent, native_window);
    let display_handle = RawDisplayHandle::Ohos(OhosDisplayHandle::new());
    let display_handle = unsafe { DisplayHandle::borrow_raw(display_handle) };
    let window_handle = unsafe { WindowHandle::borrow_raw(window_handle) };

    let viewport_size = viewport_rect.size;
    let refresh_driver = Rc::new(VsyncRefreshDriver::default());
    let rendering_context = Rc::new(
        WindowRenderingContext::new_with_refresh_driver(
            display_handle,
            window_handle,
            PhysicalSize::new(viewport_size.width as u32, viewport_size.height as u32),
            refresh_driver.clone(),
        )
        .expect("Could not create RenderingContext"),
    );
    Ok(App::new(AppInitOptions {
        host,
        event_loop_waker,
        viewport_rect,
        hidpi_scale_factor: native_values.display_density as f32,
        rendering_context,
        refresh_driver,
        initial_url: Some(options.url),
        opts,
        preferences,
        servoshell_preferences,
        #[cfg(feature = "webxr")]
        xr_discovery: None,
    }))
}

#[napi(object)]
#[derive(Debug)]
pub struct InitOpts {
    pub url: String,
    /// Path to application data bundled with the servo app, e.g. web-pages.
    pub resource_dir: String,
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
#[derive(Clone)]
pub(crate) struct XComponentWrapper(*mut OH_NativeXComponent);
#[repr(transparent)]
#[derive(Clone)]
pub(crate) struct WindowWrapper(*mut c_void);
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
    FocusWebview(u32),
    NewWebview(XComponentWrapper, WindowWrapper),
}

/// Storing webview related items
struct NativeWebViewComponents {
    /// The id of the related webview
    id: WebViewId,
    /// The XComponentWrapper for the above webview
    xcomponent: XComponentWrapper,
    /// The WindowWrapper for the above webview
    window: WindowWrapper,
}

impl ServoAction {
    fn dispatch_touch_event(servo: &App, kind: TouchEventType, x: f32, y: f32, pointer_id: i32) {
        match kind {
            TouchEventType::Down => servo.touch_down(x, y, pointer_id),
            TouchEventType::Up => servo.touch_up(x, y, pointer_id),
            TouchEventType::Move => servo.touch_move(x, y, pointer_id),
            TouchEventType::Cancel => servo.touch_cancel(x, y, pointer_id),
            TouchEventType::Unknown => warn!("Can't dispatch Unknown Touch Event"),
        }
    }

    // todo: consider making this take `self`, so we don't need to needlessly clone.
    fn do_action(&self, servo: &Rc<App>) {
        use ServoAction::*;
        match self {
            WakeUp => {
                servo.spin_event_loop();
            },
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
                    servo.key_down(Key::Named(NamedKey::Delete));
                    servo.key_up(Key::Named(NamedKey::Delete));
                }
            },
            ImeDeleteBackward(len) => {
                for _ in 0..*len {
                    servo.key_down(Key::Named(NamedKey::Backspace));
                    servo.key_up(Key::Named(NamedKey::Backspace));
                }
            },
            ImeSendEnter => {
                servo.key_down(Key::Named(NamedKey::Enter));
                servo.key_up(Key::Named(NamedKey::Enter));
            },
            Initialize(_init_opts) => {
                panic!("Received Initialize event, even though servo is already initialized")
            },
            Vsync => {
                servo.notify_vsync();
            },
            Resize { width, height } => {
                servo.resize(Rect::new(Point2D::origin(), Size2D::new(*width, *height)))
            },
            FocusWebview(arkts_id) => {
                if let Some(native_webview_components) =
                    NATIVE_WEBVIEWS.lock().unwrap().get(*arkts_id as usize)
                {
                    let webview = servo
                        .active_or_newest_webview()
                        .expect("Should always start with at least one WebView");
                    if webview.id() != native_webview_components.id {
                        servo.activate_webview(native_webview_components.id);
                        servo.pause_painting();
                        let (window_handle, viewport_rect) = get_raw_window_handle(
                            native_webview_components.xcomponent.0,
                            native_webview_components.window.0,
                        );
                        servo.resume_painting(window_handle, viewport_rect);
                        let url = webview
                            .url()
                            .map(|u| u.to_string())
                            .unwrap_or(String::from("about:blank"));
                        SET_URL_BAR_CB
                            .get()
                            .map(|f| f.call(url, ThreadsafeFunctionCallMode::Blocking));
                    }
                } else {
                    error!("Could not find webview to activate");
                }
            },
            NewWebview(xcomponent, window) => {
                servo.pause_painting();
                let webview =
                    servo.create_and_activate_toplevel_webview("about:blank".parse().unwrap());
                let (window_handle, viewport_rect) = get_raw_window_handle(xcomponent.0, window.0);

                servo.resume_painting(window_handle, viewport_rect);
                let id = webview.id();
                NATIVE_WEBVIEWS
                    .lock()
                    .unwrap()
                    .push(NativeWebViewComponents {
                        id,
                        xcomponent: xcomponent.clone(),
                        window: window.clone(),
                    });
                let url = webview
                    .url()
                    .map(|u| u.to_string())
                    .unwrap_or(String::from("about:blank"));
                SET_URL_BAR_CB
                    .get()
                    .map(|f| f.call(url, ThreadsafeFunctionCallMode::Blocking));
            },
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

#[unsafe(no_mangle)]
extern "C" fn on_surface_created_cb(xcomponent: *mut OH_NativeXComponent, window: *mut c_void) {
    info!("on_surface_created_cb");
    #[cfg(feature = "tracing-hitrace")]
    let _ = hitrace::ScopedTrace::start_trace(&c"on_surface_created_cb");

    let xc_wrapper = XComponentWrapper(xcomponent);
    let window_wrapper = WindowWrapper(window);

    if SERVO_CHANNEL.get().is_none() {
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
            let servo = init_app(*init_opts, window.0, xc.0, wakeup, callbacks)
                .expect("Servo initialization failed");
            let id = servo
                .active_or_newest_webview()
                .expect("Should always start with at least one WebView")
                .id();
            NATIVE_WEBVIEWS
                .lock()
                .unwrap()
                .push(NativeWebViewComponents {
                    id,
                    xcomponent: xc,
                    window,
                });

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
    } else {
        call(ServoAction::NewWebview(xc_wrapper, window_wrapper))
            .expect("Could not create new webview");
    }
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
) -> Result<Point2D<i32, DevicePixel>, i32> {
    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;

    let result = unsafe {
        OH_NativeXComponent_GetXComponentOffset(xcomponent, native_window, &raw mut x, &raw mut y)
    };
    if result != 0 {
        error!("OH_NativeXComponent_GetXComponentOffset failed with {result}");
        return Err(result);
    }
    let x = (x.round() as i64).try_into().expect("X offset too large");
    let y = (y.round() as i64).try_into().expect("Y offset too large");
    Ok(Point2D::new(x, y))
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
) -> Result<Size2D<i32, DevicePixel>, i32> {
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
    Ok(Size2D::new(width, height))
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
        "script::dom::console",
        // Show GL errors by default.
        "canvas::webgl_thread",
        "compositing::paint",
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
        let r = log::set_logger(logger).map(|()| log::set_max_level(max_level));
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

fn register_xcomponent_callbacks(env: &Env, xcomponent: &Object) -> napi_ohos::Result<()> {
    info!("napi_get_named_property call successfull");
    let raw = xcomponent.raw();
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
fn debug_jsobject(obj: &Object, obj_name: &str) -> napi_ohos::Result<()> {
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

#[napi(module_exports)]
fn init(exports: Object, env: Env) -> napi_ohos::Result<()> {
    initialize_logging_once();
    info!("servoshell init function called");
    if let Ok(xcomponent) = exports.get_named_property::<Object>("__NATIVE_XCOMPONENT_OBJ__") {
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

#[napi]
fn focus_webview(id: u32) {
    debug!("Focusing webview {id} from napi");
    call(ServoAction::FocusWebview(id)).expect("Could not focus webview");
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

#[expect(dead_code)]
#[derive(Debug)]
enum ImeError {
    TextEditorProxy(CreateTextEditorProxyError),
    ImeProxy(CreateImeProxyError),
}

impl HostCallbacks {
    pub fn new() -> Self {
        HostCallbacks {
            ime_proxy: RefCell::new(None),
        }
    }

    fn try_create_ime_proxy(
        &self,
        input_type: InputMethodType,
        multiline: bool,
    ) -> Result<ImeProxy, ImeError> {
        let attach_options = AttachOptions::new(true);
        let options = convert_ime_options(input_type, multiline);
        let text_config = ohos_ime::TextConfigBuilder::new()
            .input_type(options.input_type)
            .enterkey_type(options.enterkey_type)
            .build();
        let editor = RawTextEditorProxy::new(Box::new(ServoIme { text_config }))
            .map_err(|e| ImeError::TextEditorProxy(e))?;
        ImeProxy::new(editor, attach_options).map_err(|e| ImeError::ImeProxy(e))
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
    fn show_alert(&self, message: String) {
        // forward it to tracing
        #[cfg(feature = "tracing-hitrace")]
        {
            if message.contains("TESTCASE_PROFILING") {
                if let Some((tag, number)) = message.rsplit_once(":") {
                    hitrace::trace_metric_str(tag, number.parse::<i64>().unwrap_or(-1));
                }
            }
        }

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

    fn notify_load_status_changed(&self, load_status: LoadStatus) {
        // Note: It seems that we don't necessarily get 1 `LoadStatus::Complete` for each
        // each time `LoadStatus::Started` is called. Presumably this requires some API changes,
        // e.g. including webview id, perhaps URL and some additional investigation effort.
        // For now we just add a trace event here, so that we can see in the trace if we
        // successfully loaded **a** page.
        if load_status == LoadStatus::Complete {
            #[cfg(feature = "tracing-hitrace")]
            let _scope = hitrace::ScopedTrace::start_trace(&c"PageLoadEndedPrompt");
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
    /// When the keyboard cannot be shown (because the application is not in focus)
    /// we just continue and try next time.
    fn on_ime_show(&self, control: InputMethodControl) {
        debug!("IME show!");
        let mut ime_proxy = self.ime_proxy.borrow_mut();
        if ime_proxy.is_none() {
            *ime_proxy =
                match self.try_create_ime_proxy(control.input_method_type(), control.multiline()) {
                    Err(ref e) => {
                        error!("Could not show keyboard because of {e:?}");
                        None
                    },
                    Ok(proxy) => Some(proxy),
                };
        }

        if let Some(ref ime) = *ime_proxy {
            match ime.show_keyboard() {
                Ok(()) => debug!("IME show keyboard - success"),
                Err(_e) => error!("IME show keyboard error"),
            }
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
