/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(non_snake_case)]

mod resources;
mod simpleservo;

use std::os::raw::{c_char, c_int, c_void};
use std::ptr::NonNull;
use std::sync::Arc;

use android_logger::{self, Config, FilterBuilder};
use jni::objects::{GlobalRef, JClass, JObject, JString, JValue, JValueOwned};
use jni::sys::{jboolean, jfloat, jint, jobject};
use jni::{JNIEnv, JavaVM};
use log::{debug, error, info, warn};
use raw_window_handle::{
    AndroidDisplayHandle, AndroidNdkWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use servo::{
    AlertResponse, LoadStatus, MediaSessionActionType, PermissionRequest, SimpleDialog, WebView,
};
use simpleservo::{
    DeviceIntRect, EventLoopWaker, InitOptions, InputMethodType, MediaSessionPlaybackState, APP,
};

use super::app_state::{Coordinates, RunningAppState};
use super::host_trait::HostTrait;

struct HostCallbacks {
    callbacks: GlobalRef,
    jvm: JavaVM,
}

extern "C" {
    fn ANativeWindow_fromSurface(env: *mut jni::sys::JNIEnv, surface: jobject) -> *mut c_void;
}

#[no_mangle]
pub extern "C" fn android_main() {
    // FIXME(mukilan): this android_main is only present to stop
    // the java side 'System.loadLibrary('servoshell') call from
    // failing due to undefined reference to android_main introduced
    // by winit's android-activity crate. There is no way to disable
    // this currently.
}

fn call<F>(env: &mut JNIEnv, f: F)
where
    F: Fn(&RunningAppState),
{
    APP.with(|app| match app.borrow().as_ref() {
        Some(ref app_state) => (f)(app_state),
        None => throw(env, "Servo not available in this thread"),
    });
}

#[no_mangle]
pub extern "C" fn Java_org_servo_servoview_JNIServo_version<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> JString<'local> {
    let v = crate::servo_version();
    env.new_string(&v)
        .unwrap_or_else(|_str| JObject::null().into())
}

#[no_mangle]
pub extern "C" fn Java_org_servo_servoview_JNIServo_init<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    _activity: JObject<'local>,
    opts: JObject<'local>,
    callbacks_obj: JObject<'local>,
    surface: JObject<'local>,
) {
    let (opts, log, log_str, _gst_debug_str) = match get_options(&mut env, &opts, &surface) {
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
            "compositing::compositor",
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

    let wakeup = Box::new(WakeupCallback::new(callbacks_ref.clone(), &env));
    let callbacks = Box::new(HostCallbacks::new(callbacks_ref, &env));

    if let Err(err) = simpleservo::init(opts, wakeup, callbacks) {
        throw(&mut env, err)
    };
}

#[no_mangle]
pub extern "C" fn Java_org_servo_servoview_JNIServo_requestShutdown<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) {
    debug!("requestShutdown");
    call(&mut env, |s| s.request_shutdown());
}

#[no_mangle]
pub extern "C" fn Java_org_servo_servoview_JNIServo_deinit<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
) {
    debug!("deinit");
    simpleservo::deinit();
}

#[no_mangle]
pub extern "C" fn Java_org_servo_servoview_JNIServo_resize<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    coordinates: JObject<'local>,
) {
    let coords = jni_coords_to_rust_coords(&mut env, &coordinates);
    debug!("resize {:#?}", coords);
    match coords {
        Ok(coords) => call(&mut env, |s| s.resize(coords.clone())),
        Err(error) => throw(&mut env, &error),
    }
}

#[no_mangle]
pub extern "C" fn Java_org_servo_servoview_JNIServo_performUpdates<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) {
    debug!("performUpdates");
    call(&mut env, |s| {
        s.perform_updates();
        s.present_if_needed();
    });
}

#[no_mangle]
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

#[no_mangle]
pub extern "C" fn Java_org_servo_servoview_JNIServo_reload<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) {
    debug!("reload");
    call(&mut env, |s| s.reload());
}

#[no_mangle]
pub extern "C" fn Java_org_servo_servoview_JNIServo_stop<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) {
    debug!("stop");
    call(&mut env, |s| s.stop());
}

#[no_mangle]
pub extern "C" fn Java_org_servo_servoview_JNIServo_goBack<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) {
    debug!("goBack");
    call(&mut env, |s| s.go_back());
}

#[no_mangle]
pub extern "C" fn Java_org_servo_servoview_JNIServo_goForward<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) {
    debug!("goForward");
    call(&mut env, |s| s.go_forward());
}

#[no_mangle]
pub extern "C" fn Java_org_servo_servoview_JNIServo_scrollStart<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    dx: jint,
    dy: jint,
    x: jint,
    y: jint,
) {
    debug!("scrollStart");
    call(&mut env, |s| s.scroll_start(dx as f32, dy as f32, x, y));
}

#[no_mangle]
pub extern "C" fn Java_org_servo_servoview_JNIServo_scrollEnd<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    dx: jint,
    dy: jint,
    x: jint,
    y: jint,
) {
    debug!("scrollEnd");
    call(&mut env, |s| s.scroll_end(dx as f32, dy as f32, x, y));
}

#[no_mangle]
pub extern "C" fn Java_org_servo_servoview_JNIServo_scroll<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    dx: jint,
    dy: jint,
    x: jint,
    y: jint,
) {
    debug!("scroll");
    call(&mut env, |s| s.scroll(dx as f32, dy as f32, x, y));
}

#[no_mangle]
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

#[no_mangle]
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

#[no_mangle]
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

#[no_mangle]
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

#[no_mangle]
pub extern "C" fn Java_org_servo_servoview_JNIServo_pinchZoomStart<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    factor: jfloat,
    x: jint,
    y: jint,
) {
    debug!("pinchZoomStart");
    call(&mut env, |s| s.pinchzoom_start(factor, x as u32, y as u32));
}

#[no_mangle]
pub extern "C" fn Java_org_servo_servoview_JNIServo_pinchZoom<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    factor: jfloat,
    x: jint,
    y: jint,
) {
    debug!("pinchZoom");
    call(&mut env, |s| s.pinchzoom(factor, x as u32, y as u32));
}

#[no_mangle]
pub extern "C" fn Java_org_servo_servoview_JNIServo_pinchZoomEnd<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    factor: jfloat,
    x: jint,
    y: jint,
) {
    debug!("pinchZoomEnd");
    call(&mut env, |s| s.pinchzoom_end(factor, x as u32, y as u32));
}

#[no_mangle]
pub extern "C" fn Java_org_servo_servoview_JNIServo_click(
    mut env: JNIEnv,
    _: JClass,
    x: jfloat,
    y: jfloat,
) {
    debug!("click");
    call(&mut env, |s| s.click(x, y));
}

#[no_mangle]
pub extern "C" fn Java_org_servo_servoview_JNIServo_pauseCompositor(
    mut env: JNIEnv,
    _: JClass<'_>,
) {
    debug!("pauseCompositor");
    call(&mut env, |s| s.pause_compositor());
}

#[no_mangle]
pub extern "C" fn Java_org_servo_servoview_JNIServo_resumeCompositor<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    surface: JObject<'local>,
    coordinates: JObject<'local>,
) {
    debug!("resumeCompositor");
    let coords = match jni_coords_to_rust_coords(&mut env, &coordinates) {
        Ok(coords) => coords,
        Err(error) => return throw(&mut env, &error),
    };

    let (_, window_handle) = display_and_window_handle(&mut env, &surface);
    call(&mut env, |s| {
        s.resume_compositor(window_handle, coords.clone())
    });
}

#[no_mangle]
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
}

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
        let mut env = self.jvm.get_env().unwrap();
        env.call_method(self.callbacks.as_obj(), "onShutdownComplete", "()V", &[])
            .unwrap();
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

    fn on_allow_navigation(&self, url: String) -> bool {
        debug!("on_allow_navigation");
        let mut env = self.jvm.get_env().unwrap();
        let Ok(url_string) = new_string_as_jvalue(&mut env, &url) else {
            return false;
        };
        let allow = env.call_method(
            self.callbacks.as_obj(),
            "onAllowNavigation",
            "(Ljava/lang/String;)Z",
            &[(&url_string).into()],
        );
        match allow {
            Ok(allow) => allow.z().unwrap(),
            Err(_) => true,
        }
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

    fn on_animating_changed(&self, animating: bool) {
        debug!("on_animating_changed");
        let mut env = self.jvm.get_env().unwrap();
        let animating = JValue::Bool(animating as jboolean);
        env.call_method(
            self.callbacks.as_obj(),
            "onAnimatingChanged",
            "(Z)V",
            &[animating],
        )
        .unwrap();
    }

    fn on_ime_show(
        &self,
        _input_type: InputMethodType,
        _text: Option<(String, i32)>,
        _multiline: bool,
        _rect: DeviceIntRect,
    ) {
    }
    fn on_ime_hide(&self) {}

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

    fn show_context_menu(&self, _title: Option<String>, _items: Vec<String>) {}

    fn on_panic(&self, _reason: String, _backtrace: Option<String>) {}
}

extern "C" {
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

fn jni_coords_to_rust_coords<'local>(
    env: &mut JNIEnv<'local>,
    obj: &JObject<'local>,
) -> Result<Coordinates, String> {
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
    Ok(Coordinates::new(x, y, width, height))
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
    let coordinates = jni_coords_to_rust_coords(env, &coordinates)?;

    let args = match args {
        Some(args) => serde_json::from_str(&args)
            .map_err(|_| "Invalid arguments. Servo arguments must be formatted as a JSON array")?,
        None => None,
    };

    let (display_handle, window_handle) = display_and_window_handle(env, surface);
    let opts = InitOptions {
        args: args.unwrap_or(vec![]),
        url,
        coordinates,
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
