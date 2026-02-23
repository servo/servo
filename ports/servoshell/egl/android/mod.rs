/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(non_snake_case)]

mod resources;

use std::cell::RefCell;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::Arc;

use android_logger::{self, Config, FilterBuilder};
use euclid::{Point2D, Rect, Scale, Size2D};
use jni::objects::{GlobalRef, JClass, JObject, JString, JValue, JValueOwned};
use jni::sys::{jboolean, jfloat, jint, jobject};
use jni::{JNIEnv, JavaVM};
use keyboard_types::{Key, NamedKey};
use log::{debug, error, info, warn};
use raw_window_handle::{
    AndroidDisplayHandle, AndroidNdkWindowHandle, DisplayHandle, RawDisplayHandle, RawWindowHandle,
    WindowHandle,
};
use resources::ResourceReaderInstance;
pub use servo::MediaSessionPlaybackState;
use servo::{
    self, DevicePixel, EventLoopWaker, InputMethodControl, LoadStatus, MediaSessionActionType,
    MouseButton, PrefValue,
};

use super::app::{App, AppInitOptions};
use super::host_trait::HostTrait;
use crate::prefs::{ArgumentParsingResult, EXPERIMENTAL_PREFS, parse_command_line_arguments};

thread_local! {
    pub static APP: RefCell<Option<Rc<App>>> = const { RefCell::new(None) };
}

struct InitOptions {
    args: Vec<String>,
    url: Option<String>,
    viewport_rect: Rect<i32, DevicePixel>,
    density: f32,
    #[cfg(feature = "webxr")]
    xr_discovery: Option<servo::webxr::Discovery>,
    window_handle: RawWindowHandle,
    display_handle: RawDisplayHandle,
}

struct HostCallbacks {
    callbacks: GlobalRef,
    jvm: JavaVM,
}

unsafe extern "C" {
    fn ANativeWindow_fromSurface(env: *mut jni::sys::JNIEnv, surface: jobject) -> *mut c_void;
}

#[unsafe(no_mangle)]
pub extern "C" fn android_main() {
    // FIXME(mukilan): this android_main is only present to stop
    // the java side 'System.loadLibrary('servoshell') call from
    // failing due to undefined reference to android_main introduced
    // by winit's android-activity crate. There is no way to disable
    // this currently.
}

fn call<F>(env: &mut JNIEnv, f: F)
where
    F: FnOnce(&App),
{
    APP.with(|app| match app.borrow().as_ref() {
        Some(app) => (f)(app),
        None => throw(env, "Servo not available in this thread"),
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_version<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> JString<'local> {
    let version = crate::VERSION;
    env.new_string(version)
        .unwrap_or_else(|_str| JObject::null().into())
}

/// Initialize Servo. At that point, we need a valid GL context. In the future, this will
/// be done in multiple steps.
#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_init<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    _activity: JObject<'local>,
    opts: JObject<'local>,
    callbacks_obj: JObject<'local>,
    surface: JObject<'local>,
) {
    let (init_opts, log, log_str, _gst_debug_str) = match get_options(&mut env, &opts, &surface) {
        Ok((opts, log, log_str, gst_debug_str)) => (opts, log, log_str, gst_debug_str),
        Err(err) => {
            throw(&mut env, &err);
            return;
        },
    };

    if log {
        // Note: Android debug logs are stripped from a release build.
        // debug!() will only show in a debug build. Use info!() if logs
        // should show up in adb logcat with a release build.
        let filters = [
            "servo",
            "servoshell",
            "servoshell::egl:gl_glue",
            // Show redirected stdout / stderr by default
            "servoshell::egl::log",
            // Show JS errors by default.
            "script::dom::bindings::error",
            // Show GL errors by default.
            "canvas::webgl_thread",
            "paint::paint",
            "constellation::constellation",
        ];
        let mut filter_builder = FilterBuilder::new();
        for &module in &filters {
            filter_builder.filter_module(module, log::LevelFilter::Debug);
        }
        if let Some(log_str) = log_str {
            for module in log_str.split(',') {
                filter_builder.filter_module(module, log::LevelFilter::Debug);
            }
        }

        android_logger::init_once(
            Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_filter(filter_builder.build())
                .with_tag("servoshell"),
        )
    }

    info!("init");

    // We only redirect stdout and stderr for non-production builds, since it is
    // only used for debugging purposes. This saves us one thread in production.
    #[cfg(not(servo_production))]
    if let Err(e) = super::log::redirect_stdout_and_stderr() {
        error!("Failed to redirect stdout and stderr to logcat due to: {e:?}");
    }

    let callbacks_ref = match env.new_global_ref(callbacks_obj) {
        Ok(r) => r,
        Err(_) => {
            throw(
                &mut env,
                "Failed to get global reference of callback argument",
            );
            return;
        },
    };

    let event_loop_waker = Box::new(WakeupCallback::new(callbacks_ref.clone(), &env));
    let host = Rc::new(HostCallbacks::new(callbacks_ref, &env));

    crate::init_crypto();
    servo::resources::set(Box::new(ResourceReaderInstance::new()));

    let (opts, mut preferences, servoshell_preferences) =
        match parse_command_line_arguments(init_opts.args.as_slice()) {
            ArgumentParsingResult::ContentProcess(..) => {
                unreachable!("Android does not have support for multiprocess yet.")
            },
            ArgumentParsingResult::ChromeProcess(opts, preferences, servoshell_preferences) => {
                (opts, preferences, servoshell_preferences)
            },
            ArgumentParsingResult::Exit => {
                std::process::exit(0);
            },
            ArgumentParsingResult::ErrorParsing => std::process::exit(1),
        };

    preferences.set_value("viewport_meta_enabled", servo::PrefValue::Bool(true));

    crate::init_tracing(servoshell_preferences.tracing_filter.as_deref());

    let (display_handle, window_handle) = unsafe {
        (
            DisplayHandle::borrow_raw(init_opts.display_handle),
            WindowHandle::borrow_raw(init_opts.window_handle),
        )
    };

    let hidpi_scale_factor = Scale::new(init_opts.density);

    APP.with(|app| {
        let new_app = App::new(AppInitOptions {
            host,
            event_loop_waker,
            initial_url: init_opts.url,
            opts,
            preferences,
            servoshell_preferences,
            #[cfg(feature = "webxr")]
            xr_discovery: init_opts.xr_discovery,
        });
        new_app.add_platform_window(
            display_handle,
            window_handle,
            init_opts.viewport_rect,
            hidpi_scale_factor,
        );
        *app.borrow_mut() = Some(new_app);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_setExperimentalMode<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    enable: jboolean,
) {
    debug!("setExperimentalMode {enable}");
    call(&mut env, |s| {
        for pref in EXPERIMENTAL_PREFS {
            s.servo().set_preference(pref, PrefValue::Bool(enable != 0));
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_resize<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    coordinates: JObject<'local>,
) {
    let viewport_rect = jni_coordinate_to_rust_viewport_rect(&mut env, &coordinates);
    debug!("resize {viewport_rect:#?}");
    match viewport_rect {
        Ok(viewport_rect) => call(&mut env, |s| s.resize(viewport_rect)),
        Err(error) => throw(&mut env, &error),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_performUpdates<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) {
    debug!("performUpdates");
    call(&mut env, |app| {
        app.spin_event_loop();
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_loadUri<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    url: JString<'local>,
) {
    debug!("loadUri");
    match env.get_string(&url) {
        Ok(url) => {
            let url: String = url.into();
            call(&mut env, |s| s.load_uri(&url));
        },
        Err(_) => {
            throw(&mut env, "Failed to convert Java string");
        },
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_reload<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) {
    debug!("reload");
    call(&mut env, |s| s.reload());
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_stop<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) {
    debug!("stop");
    call(&mut env, |s| s.stop());
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_goBack<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) {
    debug!("goBack");
    call(&mut env, |s| s.go_back());
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_goForward<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) {
    debug!("goForward");
    call(&mut env, |s| s.go_forward());
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_scroll<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    dx: jint,
    dy: jint,
    x: jint,
    y: jint,
) {
    debug!("scroll");
    call(&mut env, |s| {
        s.scroll(dx as f32, dy as f32, x as f32, y as f32)
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_doFrame<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
) {
    call(&mut env, |s| s.notify_vsync());
}

enum KeyCode {
    Delete,
    ForwardDelete,
    Enter,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
}

impl TryFrom<i32> for KeyCode {
    type Error = ();

    // Values derived from <https://developer.android.com/reference/android/view/KeyEvent>
    fn try_from(keycode: i32) -> Result<KeyCode, ()> {
        Ok(match keycode {
            66 => KeyCode::Enter,
            67 => KeyCode::Delete,
            112 => KeyCode::ForwardDelete,
            21 => KeyCode::ArrowLeft,
            22 => KeyCode::ArrowRight,
            19 => KeyCode::ArrowUp,
            20 => KeyCode::ArrowDown,
            _ => return Err(()),
        })
    }
}

impl From<KeyCode> for Key {
    fn from(keycode: KeyCode) -> Key {
        Key::Named(match keycode {
            KeyCode::Enter => NamedKey::Enter,
            KeyCode::Delete => NamedKey::Backspace,
            KeyCode::ForwardDelete => NamedKey::Delete,
            KeyCode::ArrowLeft => NamedKey::ArrowLeft,
            KeyCode::ArrowRight => NamedKey::ArrowRight,
            KeyCode::ArrowUp => NamedKey::ArrowUp,
            KeyCode::ArrowDown => NamedKey::ArrowDown,
        })
    }
}

fn key_from_unicode_keycode(unicode: u32, keycode: i32) -> Option<Key> {
    char::from_u32(unicode)
        .filter(|c| *c != '\0')
        .map(|c| Key::Character(String::from(c)))
        .or_else(|| KeyCode::try_from(keycode).ok().map(Key::from))
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_keydown<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    keycode: jint,
    unicode: jint,
) {
    debug!("keydown {keycode}");
    if let Some(key) = key_from_unicode_keycode(unicode as u32, keycode) {
        call(&mut env, move |s| s.key_down(key));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_keyup<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    keycode: jint,
    unicode: jint,
) {
    debug!("keyup {keycode}");
    if let Some(key) = key_from_unicode_keycode(unicode as u32, keycode) {
        call(&mut env, move |s| s.key_up(key));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_touchDown<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    x: jfloat,
    y: jfloat,
    pointer_id: jint,
) {
    debug!("touchDown");
    call(&mut env, |s| s.touch_down(x, y, pointer_id));
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_touchUp<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    x: jfloat,
    y: jfloat,
    pointer_id: jint,
) {
    debug!("touchUp");
    call(&mut env, |s| s.touch_up(x, y, pointer_id));
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_touchMove<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    x: jfloat,
    y: jfloat,
    pointer_id: jint,
) {
    debug!("touchMove");
    call(&mut env, |s| s.touch_move(x, y, pointer_id));
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_touchCancel<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    x: jfloat,
    y: jfloat,
    pointer_id: jint,
) {
    debug!("touchCancel");
    call(&mut env, |s| s.touch_cancel(x, y, pointer_id));
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_pinchZoomStart<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    factor: jfloat,
    x: jfloat,
    y: jfloat,
) {
    debug!("pinchZoomStart");
    call(&mut env, |s| s.pinchzoom_start(factor, x, y));
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_pinchZoom<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    factor: jfloat,
    x: jfloat,
    y: jfloat,
) {
    debug!("pinchZoom");
    call(&mut env, |s| s.pinchzoom(factor, x, y));
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_pinchZoomEnd<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    factor: jfloat,
    x: jfloat,
    y: jfloat,
) {
    debug!("pinchZoomEnd");
    call(&mut env, |s| s.pinchzoom_end(factor, x, y));
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_click(
    mut env: JNIEnv,
    _: JClass,
    x: jfloat,
    y: jfloat,
) {
    debug!("click");
    call(&mut env, |s| {
        s.mouse_down(x, y, MouseButton::Left);
        s.mouse_up(x, y, MouseButton::Left);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_pausePainting(mut env: JNIEnv, _: JClass<'_>) {
    debug!("pausePainting");
    call(&mut env, |s| s.pause_painting());
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_resumePainting<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    surface: JObject<'local>,
    coordinates: JObject<'local>,
) {
    debug!("resumePainting");
    let viewport_rect = match jni_coordinate_to_rust_viewport_rect(&mut env, &coordinates) {
        Ok(viewport_rect) => viewport_rect,
        Err(error) => return throw(&mut env, &error),
    };

    let (_, window_handle) = display_and_window_handle(&mut env, &surface);
    call(&mut env, |app| {
        app.resume_painting(window_handle, viewport_rect);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_mediaSessionAction<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    action: jint,
) {
    debug!("mediaSessionAction");

    let action = match action {
        1 => MediaSessionActionType::Play,
        2 => MediaSessionActionType::Pause,
        3 => MediaSessionActionType::SeekBackward,
        4 => MediaSessionActionType::SeekForward,
        5 => MediaSessionActionType::PreviousTrack,
        6 => MediaSessionActionType::NextTrack,
        7 => MediaSessionActionType::SkipAd,
        8 => MediaSessionActionType::Stop,
        9 => MediaSessionActionType::SeekTo,
        _ => return warn!("Ignoring unknown MediaSessionAction"),
    };
    call(&mut env, |s| s.media_session_action(action.clone()));
}

pub struct WakeupCallback {
    callback: GlobalRef,
    jvm: Arc<JavaVM>,
}

impl WakeupCallback {
    pub fn new(callback: GlobalRef, env: &JNIEnv) -> WakeupCallback {
        let jvm = Arc::new(env.get_java_vm().unwrap());
        WakeupCallback { callback, jvm }
    }
}

impl EventLoopWaker for WakeupCallback {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(WakeupCallback {
            callback: self.callback.clone(),
            jvm: self.jvm.clone(),
        })
    }
    fn wake(&self) {
        debug!("wakeup");
        let mut env = self.jvm.attach_current_thread().unwrap();
        env.call_method(self.callback.as_obj(), "wakeup", "()V", &[])
            .unwrap();
    }
}

impl HostCallbacks {
    pub fn new(callbacks: GlobalRef, env: &JNIEnv) -> HostCallbacks {
        let jvm = env.get_java_vm().unwrap();
        HostCallbacks { callbacks, jvm }
    }
}

impl HostTrait for HostCallbacks {
    fn show_alert(&self, message: String) {
        let mut env = self.jvm.get_env().unwrap();
        let Ok(string) = new_string_as_jvalue(&mut env, &message) else {
            return;
        };
        env.call_method(
            self.callbacks.as_obj(),
            "onAlert",
            "(Ljava/lang/String;)V",
            &[(&string).into()],
        )
        .unwrap();
    }

    fn notify_load_status_changed(&self, load_status: LoadStatus) {
        debug!("notify_load_status_changed: {load_status:?}");
        let mut env = self.jvm.get_env().unwrap();
        match load_status {
            LoadStatus::Started => {
                env.call_method(self.callbacks.as_obj(), "onLoadStarted", "()V", &[])
                    .unwrap();
            },
            LoadStatus::HeadParsed => {},
            LoadStatus::Complete => {
                env.call_method(self.callbacks.as_obj(), "onLoadEnded", "()V", &[])
                    .unwrap();
            },
        };
    }

    fn on_shutdown_complete(&self) {
        debug!("on_shutdown_complete");
    }

    fn on_title_changed(&self, title: Option<String>) {
        debug!("on_title_changed");
        let mut env = self.jvm.get_env().unwrap();
        let title = title.unwrap_or_default();
        let Ok(title_string) = new_string_as_jvalue(&mut env, &title) else {
            return;
        };
        env.call_method(
            self.callbacks.as_obj(),
            "onTitleChanged",
            "(Ljava/lang/String;)V",
            &[(&title_string).into()],
        )
        .unwrap();
    }

    fn on_url_changed(&self, url: String) {
        debug!("on_url_changed");
        let mut env = self.jvm.get_env().unwrap();
        let Ok(url_string) = new_string_as_jvalue(&mut env, &url) else {
            return;
        };
        env.call_method(
            self.callbacks.as_obj(),
            "onUrlChanged",
            "(Ljava/lang/String;)V",
            &[(&url_string).into()],
        )
        .unwrap();
    }

    fn on_history_changed(&self, can_go_back: bool, can_go_forward: bool) {
        debug!("on_history_changed");
        let mut env = self.jvm.get_env().unwrap();
        let can_go_back = JValue::Bool(can_go_back as jboolean);
        let can_go_forward = JValue::Bool(can_go_forward as jboolean);
        env.call_method(
            self.callbacks.as_obj(),
            "onHistoryChanged",
            "(ZZ)V",
            &[can_go_back, can_go_forward],
        )
        .unwrap();
    }

    fn on_ime_show(&self, _: InputMethodControl) {
        let mut env = self.jvm.get_env().unwrap();
        env.call_method(self.callbacks.as_obj(), "onImeShow", "()V", &[])
            .unwrap();
    }

    fn on_ime_hide(&self) {
        let mut env = self.jvm.get_env().unwrap();
        env.call_method(self.callbacks.as_obj(), "onImeHide", "()V", &[])
            .unwrap();
    }

    fn on_media_session_metadata(&self, title: String, artist: String, album: String) {
        info!("on_media_session_metadata");
        let mut env = self.jvm.get_env().unwrap();
        let Ok(title) = new_string_as_jvalue(&mut env, &title) else {
            return;
        };

        let Ok(artist) = new_string_as_jvalue(&mut env, &artist) else {
            return;
        };

        let Ok(album) = new_string_as_jvalue(&mut env, &album) else {
            return;
        };

        env.call_method(
            self.callbacks.as_obj(),
            "onMediaSessionMetadata",
            "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
            &[(&title).into(), (&artist).into(), (&album).into()],
        )
        .unwrap();
    }

    fn on_media_session_playback_state_change(&self, state: MediaSessionPlaybackState) {
        info!("on_media_session_playback_state_change {:?}", state);
        let mut env = self.jvm.get_env().unwrap();
        let state = state as i32;
        let state = JValue::Int(state as jint);
        env.call_method(
            self.callbacks.as_obj(),
            "onMediaSessionPlaybackStateChange",
            "(I)V",
            &[state],
        )
        .unwrap();
    }

    fn on_media_session_set_position_state(
        &self,
        duration: f64,
        position: f64,
        playback_rate: f64,
    ) {
        info!(
            "on_media_session_playback_state_change ({:?}, {:?}, {:?})",
            duration, position, playback_rate
        );
        let mut env = self.jvm.get_env().unwrap();
        let duration = JValue::Float(duration as jfloat);
        let position = JValue::Float(position as jfloat);
        let playback_rate = JValue::Float(playback_rate as jfloat);

        env.call_method(
            self.callbacks.as_obj(),
            "onMediaSessionSetPositionState",
            "(FFF)V",
            &[duration, position, playback_rate],
        )
        .unwrap();
    }

    fn on_panic(&self, _reason: String, _backtrace: Option<String>) {}
}

unsafe extern "C" {
    pub fn __android_log_write(prio: c_int, tag: *const c_char, text: *const c_char) -> c_int;
}

fn throw(env: &mut JNIEnv, err: &str) {
    if let Err(e) = env.throw(("java/lang/Exception", err)) {
        warn!(
            "Failed to throw Java exception: `{}`. Exception was: `{}`",
            e, err
        );
    }
}

fn new_string_as_jvalue<'local>(
    env: &mut JNIEnv<'local>,
    input_string: &str,
) -> Result<JValueOwned<'local>, &'static str> {
    let jstring = match env.new_string(input_string) {
        Ok(jstring) => jstring,
        Err(_) => {
            throw(env, "Couldn't create Java string");
            return Err("Couldn't create Java string");
        },
    };
    Ok(JValueOwned::from(jstring))
}

fn jni_coordinate_to_rust_viewport_rect<'local>(
    env: &mut JNIEnv<'local>,
    obj: &JObject<'local>,
) -> Result<Rect<i32, DevicePixel>, String> {
    let x = get_non_null_field(env, obj, "x", "I")?
        .i()
        .map_err(|_| "x not an int")? as i32;
    let y = get_non_null_field(env, obj, "y", "I")?
        .i()
        .map_err(|_| "y not an int")? as i32;
    let width = get_non_null_field(env, obj, "width", "I")?
        .i()
        .map_err(|_| "width not an int")? as i32;
    let height = get_non_null_field(env, obj, "height", "I")?
        .i()
        .map_err(|_| "height not an int")? as i32;
    Ok(Rect::new(Point2D::new(x, y), Size2D::new(width, height)))
}

fn get_field<'local>(
    env: &mut JNIEnv<'local>,
    obj: &JObject<'local>,
    field: &str,
    type_: &str,
) -> Result<Option<JValueOwned<'local>>, String> {
    let Ok(class) = env.get_object_class(obj) else {
        return Err("Can't get object class".to_owned());
    };

    if env.get_field_id(class, field, type_).is_err() {
        return Err(format!("Can't find `{}` field", field));
    }

    env.get_field(obj, field, type_)
        .map(Some)
        .map_err(|_| format!("Can't find `{}` field", field))
}

fn get_non_null_field<'local>(
    env: &mut JNIEnv<'local>,
    obj: &JObject<'local>,
    field: &str,
    type_: &str,
) -> Result<JValueOwned<'local>, String> {
    match get_field(env, obj, field, type_)? {
        None => Err(format!("Field {} is null", field)),
        Some(f) => Ok(f),
    }
}

fn get_field_as_string<'local>(
    env: &mut JNIEnv<'local>,
    obj: &JObject<'local>,
    field: &str,
) -> Result<Option<String>, String> {
    let string = {
        let value = get_field(env, obj, field, "Ljava/lang/String;")?;
        let Some(value) = value else {
            return Ok(None);
        };
        value
            .l()
            .map_err(|_| format!("field `{}` is not an Object", field))?
    };

    Ok(env.get_string((&string).into()).map(|s| s.into()).ok())
}

fn get_options<'local>(
    env: &mut JNIEnv<'local>,
    opts: &JObject<'local>,
    surface: &JObject<'local>,
) -> Result<(InitOptions, bool, Option<String>, Option<String>), String> {
    let args = get_field_as_string(env, opts, "args")?;
    let url = get_field_as_string(env, opts, "url")?;
    let log_str = get_field_as_string(env, opts, "logStr")?;
    let gst_debug_str = get_field_as_string(env, opts, "gstDebugStr")?;
    let experimental_mode = get_non_null_field(env, opts, "experimentalMode", "Z")?
        .z()
        .map_err(|_| "experimentalMode not a boolean")?;
    let density = get_non_null_field(env, opts, "density", "F")?
        .f()
        .map_err(|_| "density not a float")? as f32;
    let log = get_non_null_field(env, opts, "enableLogs", "Z")?
        .z()
        .map_err(|_| "enableLogs not a boolean")?;
    let coordinates = get_non_null_field(
        env,
        opts,
        "coordinates",
        "Lorg/servo/servoview/JNIServo$ServoCoordinates;",
    )?
    .l()
    .map_err(|_| "coordinates is not an object")?;
    let viewport_rect = jni_coordinate_to_rust_viewport_rect(env, &coordinates)?;

    let mut args: Vec<String> = match args {
        Some(args) => serde_json::from_str(&args)
            .map_err(|_| "Invalid arguments. Servo arguments must be formatted as a JSON array")?,
        None => None,
    }
    .unwrap_or_default();
    if experimental_mode {
        args.push("--enable-experimental-web-platform-features".to_owned());
    }

    let (display_handle, window_handle) = display_and_window_handle(env, surface);
    let opts = InitOptions {
        args,
        url,
        viewport_rect,
        density,
        xr_discovery: None,
        window_handle,
        display_handle,
    };

    Ok((opts, log, log_str, gst_debug_str))
}

fn display_and_window_handle(
    env: &mut JNIEnv<'_>,
    surface: &JObject<'_>,
) -> (RawDisplayHandle, RawWindowHandle) {
    let native_window =
        unsafe { ANativeWindow_fromSurface(env.get_native_interface(), surface.as_raw()) };
    let native_window = NonNull::new(native_window).expect("Could not get Android window");
    (
        RawDisplayHandle::Android(AndroidDisplayHandle::new()),
        RawWindowHandle::AndroidNdk(AndroidNdkWindowHandle::new(native_window)),
    )
}
