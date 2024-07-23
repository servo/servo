/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
#![allow(non_snake_case)]

use std::mem::MaybeUninit;
use std::os::raw::c_void;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Once, OnceLock};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use log::{debug, error, info, warn, LevelFilter};
use napi_derive_ohos::{module_exports, napi};
use napi_ohos::bindgen_prelude::Undefined;
use napi_ohos::threadsafe_function::{
    ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode,
};
use napi_ohos::{Env, JsFunction, JsObject, JsString, NapiRaw};
use ohos_sys::xcomponent::{
    OH_NativeXComponent, OH_NativeXComponent_Callback, OH_NativeXComponent_GetTouchEvent,
    OH_NativeXComponent_RegisterCallback, OH_NativeXComponent_TouchEvent,
    OH_NativeXComponent_TouchEventType,
};
use servo::embedder_traits::PromptResult;
use servo::euclid::Point2D;
use servo::style::Zero;
use simpleservo::EventLoopWaker;

use super::gl_glue;
use super::host_trait::HostTrait;
use super::servo_glue::ServoGlue;

mod simpleservo;

// Todo: in the future these libraries should be added by Rust sys-crates
#[link(name = "ace_napi.z")]
#[link(name = "ace_ndk.z")]
#[link(name = "hilog_ndk.z")]
#[link(name = "native_window")]
#[link(name = "clang_rt.builtins", kind = "static")]
extern "C" {}

#[napi(object)]
#[derive(Debug)]
pub struct InitOpts {
    pub url: String,
    pub device_type: String,
    pub os_full_name: String,
    pub display_density: f64,
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
enum TouchEventType {
    Down,
    Up,
    Move,
    Scroll { dx: f32, dy: f32 },
    Cancel,
    Unknown,
}

#[derive(Debug)]
enum ServoAction {
    WakeUp,
    LoadUrl(String),
    TouchEvent {
        kind: TouchEventType,
        x: f32,
        y: f32,
        pointer_id: i32,
    },
    Initialize(Box<InitOpts>),
}

#[derive(Clone, Copy, Debug, Default)]
enum Direction2D {
    Horizontal,
    Vertical,
    #[default]
    Free,
}
#[derive(Clone, Debug)]
struct TouchTracker {
    last_position: Point2D<f32, f32>,
}

impl TouchTracker {
    fn new(first_point: Point2D<f32, f32>) -> Self {
        TouchTracker {
            last_position: first_point,
        }
    }
}

// Todo: Need to check if OnceLock is suitable, or if the TS function can be destroyed, e.g.
// if the activity gets suspended.
static SET_URL_BAR_CB: OnceLock<ThreadsafeFunction<String, ErrorStrategy::Fatal>> = OnceLock::new();

struct TsThreadState {
    // last_touch_event: Option<OH_NativeXComponent_TouchEvent>,
    velocity_tracker: Option<TouchTracker>,
}

impl TsThreadState {
    const fn new() -> Self {
        Self {
            velocity_tracker: None,
        }
    }
}

static mut TS_THREAD_STATE: TsThreadState = TsThreadState::new();

impl ServoAction {
    fn dispatch_touch_event(
        servo: &mut ServoGlue,
        kind: TouchEventType,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> Result<(), &'static str> {
        match kind {
            TouchEventType::Down => servo.touch_down(x, y, pointer_id),
            TouchEventType::Up => servo.touch_up(x, y, pointer_id),
            TouchEventType::Scroll { dx, dy } => servo.scroll(dx, dy, x as i32, y as i32),
            TouchEventType::Move => servo.touch_move(x, y, pointer_id),
            TouchEventType::Cancel => servo.touch_cancel(x, y, pointer_id),
            TouchEventType::Unknown => Err("Can't dispatch Unknown Touch Event"),
        }
    }

    fn do_action(&self, servo: &mut ServoGlue) {
        use ServoAction::*;
        let res = match self {
            WakeUp => servo.perform_updates(),
            LoadUrl(url) => servo.load_uri(url.as_str()),
            TouchEvent {
                kind,
                x,
                y,
                pointer_id,
            } => Self::dispatch_touch_event(servo, *kind, *x, *y, *pointer_id),
            Initialize(_init_opts) => {
                panic!("Received Initialize event, even though servo is already initialized")
            },
        };
        if let Err(e) = res {
            error!("Failed to do {self:?} with error {e}");
        }
    }
}

static SERVO_CHANNEL: OnceLock<Sender<ServoAction>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn on_surface_created_cb(xcomponent: *mut OH_NativeXComponent, window: *mut c_void) {
    info!("on_surface_created_cb");

    let xc_wrapper = XComponentWrapper(xcomponent);
    let window_wrapper = WindowWrapper(window);

    // Each thread will send its id via the channel
    let _main_surface_thread = thread::spawn(move || {
        let (tx, rx): (Sender<ServoAction>, Receiver<ServoAction>) = mpsc::channel();

        SERVO_CHANNEL
            .set(tx.clone())
            .expect("Servo channel already initialized");

        let wakeup = Box::new(WakeupCallback::new(tx));
        let callbacks = Box::new(HostCallbacks::new());

        let egl_init = gl_glue::init().expect("egl::init() failed");
        let xc = xc_wrapper;
        let window = window_wrapper;
        let init_opts = if let Ok(ServoAction::Initialize(init_opts)) = rx.recv() {
            init_opts
        } else {
            panic!("Servos GL thread received another event before it was initialized")
        };
        let mut servo = simpleservo::init(
            *init_opts,
            window.0,
            xc.0,
            egl_init.gl_wrapper,
            wakeup,
            callbacks,
        )
        .expect("Servo initialization failed");

        info!("Surface created!");

        while let Ok(action) = rx.recv() {
            info!("Wakeup message received!");
            action.do_action(&mut servo);
        }

        info!("Sender disconnected - Terminating main surface thread");
    });

    info!("Returning from on_surface_created_cb");
}

// Todo: Probably we need to block here, until the main thread has processed the change.
pub extern "C" fn on_surface_changed_cb(
    _component: *mut OH_NativeXComponent,
    _window: *mut c_void,
) {
    error!("on_surface_changed_cb is currently not implemented!");
}

pub extern "C" fn on_surface_destroyed_cb(
    _component: *mut OH_NativeXComponent,
    _window: *mut c_void,
) {
    error!("on_surface_destroyed_cb is currently not implemented");
}

pub extern "C" fn on_dispatch_touch_event_cb(
    component: *mut OH_NativeXComponent,
    window: *mut c_void,
) {
    info!("DispatchTouchEvent");
    let mut touch_event: MaybeUninit<OH_NativeXComponent_TouchEvent> = MaybeUninit::uninit();
    let res =
        unsafe { OH_NativeXComponent_GetTouchEvent(component, window, touch_event.as_mut_ptr()) };
    if res != 0 {
        error!("OH_NativeXComponent_GetTouchEvent failed with {res}");
        return;
    }
    let touch_event = unsafe { touch_event.assume_init() };
    let kind: TouchEventType =
        match touch_event.type_ {
            OH_NativeXComponent_TouchEventType::OH_NATIVEXCOMPONENT_DOWN => {
                if touch_event.id == 0 {
                    unsafe {
                        let old = TS_THREAD_STATE.velocity_tracker.replace(TouchTracker::new(
                            Point2D::new(touch_event.x, touch_event.y),
                        ));
                        assert!(old.is_none());
                    }
                }
                TouchEventType::Down
            },
            OH_NativeXComponent_TouchEventType::OH_NATIVEXCOMPONENT_UP => {
                if touch_event.id == 0 {
                    unsafe {
                        let old = TS_THREAD_STATE.velocity_tracker.take();
                        assert!(old.is_some());
                    }
                }
                TouchEventType::Up
            },
            OH_NativeXComponent_TouchEventType::OH_NATIVEXCOMPONENT_MOVE => {
                // SAFETY: We only access TS_THREAD_STATE from the main TS thread.
                if touch_event.id == 0 {
                    let (lastX, lastY) = unsafe {
                        if let Some(last_event) = &mut TS_THREAD_STATE.velocity_tracker {
                            let touch_point = last_event.last_position;
                            last_event.last_position = Point2D::new(touch_event.x, touch_event.y);
                            (touch_point.x, touch_point.y)
                        } else {
                            error!("Move Event received, but no previous touch event was stored!");
                            // todo: handle this error case
                            panic!("Move Event received, but no previous touch event was stored!");
                        }
                    };
                    let dx = touch_event.x - lastX;
                    let dy = touch_event.y - lastY;
                    TouchEventType::Scroll { dx, dy }
                } else {
                    TouchEventType::Move
                }
            },
            OH_NativeXComponent_TouchEventType::OH_NATIVEXCOMPONENT_CANCEL => {
                if touch_event.id == 0 {
                    unsafe {
                        let old = TS_THREAD_STATE.velocity_tracker.take();
                        assert!(old.is_some());
                    }
                }
                TouchEventType::Cancel
            },
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

fn initialize_logging_once() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let mut builder = hilog::Builder::new();
        let filters = [
            "fonts",
            "servo",
            "servoshell",
            "servoshell::egl:gl_glue",
            // Show JS errors by default.
            "script::dom::bindings::error",
            // Show GL errors by default.
            "canvas::webgl_thread",
            "compositing::compositor",
            "constellation::constellation",
        ];
        let mut filter_builder = env_filter::Builder::new();
        for &module in &filters {
            filter_builder.filter_module(module, log::LevelFilter::Debug);
        }

        builder.filter_level(LevelFilter::Debug).init();

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
                let _ = error!(
                    "{} (thread {}, at {}:{})",
                    msg,
                    name,
                    location.file(),
                    location.line()
                );
            } else {
                let _ = error!("{} (thread {})", msg, name);
            }

            let _ = crate::backtrace::print_ohos();
        }));
    })
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
        info!("Registerd callbacks successfully");
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
pub fn load_url(url: String) -> Undefined {
    debug!("load url");
    call(ServoAction::LoadUrl(url)).expect("Failed to load url");
}

#[napi(js_name = "registerURLcallback")]
pub fn register_url_callback(cb: JsFunction) -> napi_ohos::Result<()> {
    info!("register_url_callback called!");
    let tsfn: ThreadsafeFunction<String, ErrorStrategy::Fatal> =
        cb.create_threadsafe_function(1, |ctx| {
            debug!(
                "url callback argument transformer called with arg {}",
                ctx.value
            );
            let s = ctx
                .env
                .create_string_from_std(ctx.value)
                .inspect_err(|e| error!("Failed to create JsString: {e:?}"))?;
            Ok(vec![s])
        })?;
    // We ignore any error for now - but probably we should propagate it back to the TS layer.
    let _ = SET_URL_BAR_CB
        .set(tsfn)
        .inspect_err(|_| warn!("Failed to set URL callback - register_url_callback called twice?"));
    Ok(())
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
        info!("wake called!");
        self.chan.send(ServoAction::WakeUp).unwrap_or_else(|e| {
            error!("Failed to send wake message with: {e}");
        });
    }
}

struct HostCallbacks {}

impl HostCallbacks {
    pub fn new() -> Self {
        HostCallbacks {}
    }
}

#[allow(unused)]
impl HostTrait for HostCallbacks {
    fn prompt_alert(&self, msg: String, trusted: bool) {
        warn!("prompt_alert not implemented. Cancelled. {}", msg);
    }

    fn prompt_yes_no(&self, msg: String, trusted: bool) -> PromptResult {
        warn!("Prompt not implemented. Cancelled. {}", msg);
        PromptResult::Secondary
    }

    fn prompt_ok_cancel(&self, msg: String, trusted: bool) -> PromptResult {
        warn!("Prompt not implemented. Cancelled. {}", msg);
        PromptResult::Secondary
    }

    fn prompt_input(&self, msg: String, default: String, trusted: bool) -> Option<String> {
        warn!("Input prompt not implemented. Cancelled. {}", msg);
        Some(default)
    }

    fn show_context_menu(&self, title: Option<String>, items: Vec<String>) {
        warn!("show_context_menu not implemented")
    }

    fn on_load_started(&self) {
        warn!("on_load_started not implemented")
    }

    fn on_load_ended(&self) {
        warn!("on_load_ended not implemented")
    }

    fn on_title_changed(&self, title: Option<String>) {
        warn!("on_title_changed not implemented")
    }

    fn on_allow_navigation(&self, url: String) -> bool {
        true
    }

    fn on_url_changed(&self, url: String) {
        debug!("Hosttrait `on_url_changed` called with new url: {url}");
        if let Some(cb) = SET_URL_BAR_CB.get() {
            cb.call(url, ThreadsafeFunctionCallMode::Blocking);
        } else {
            warn!("`on_url_changed` called without a registered callback")
        }
    }

    fn on_history_changed(&self, can_go_back: bool, can_go_forward: bool) {}

    fn on_animating_changed(&self, animating: bool) {}

    fn on_shutdown_complete(&self) {}

    fn on_ime_show(
        &self,
        input_type: servo::embedder_traits::InputMethodType,
        text: Option<(String, i32)>,
        multiline: bool,
        bounds: servo::webrender_api::units::DeviceIntRect,
    ) {
        warn!("on_title_changed not implemented")
    }

    fn on_ime_hide(&self) {
        warn!("on_title_changed not implemented")
    }

    fn get_clipboard_contents(&self) -> Option<String> {
        warn!("get_clipboard_contents not implemented");
        None
    }

    fn set_clipboard_contents(&self, contents: String) {
        warn!("set_clipboard_contents not implemented");
    }

    fn on_media_session_metadata(&self, title: String, artist: String, album: String) {
        warn!("on_media_session_metadata not implemented");
    }

    fn on_media_session_playback_state_change(
        &self,
        state: servo::embedder_traits::MediaSessionPlaybackState,
    ) {
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

    fn on_devtools_started(&self, port: Result<u16, ()>, token: String) {
        warn!("on_devtools_started not implemented");
    }

    fn on_panic(&self, reason: String, backtrace: Option<String>) {
        error!("Panic: {reason},");
        if let Some(bt) = backtrace {
            error!("Backtrace: {bt:?}")
        }
    }
}
