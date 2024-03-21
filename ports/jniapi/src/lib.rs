/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(non_snake_case)]

mod gl_glue;
mod simpleservo;

use std::collections::HashMap;
use std::os::raw::{c_char, c_int, c_void};
use std::sync::Arc;
use std::thread;

use android_logger::{self, Config, FilterBuilder};
use jni::objects::{GlobalRef, JClass, JObject, JString, JValue};
use jni::sys::{jboolean, jfloat, jint, jstring, JNI_TRUE};
use jni::{JNIEnv, JavaVM};
use libc::{dup2, pipe, read};
use log::{debug, error, info, warn};
use simpleservo::{
    Coordinates, DeviceIntRect, EventLoopWaker, HostTrait, InitOptions, InputMethodType,
    MediaSessionPlaybackState, PromptResult, ServoGlue, SERVO,
};

struct HostCallbacks {
    callbacks: GlobalRef,
    jvm: JavaVM,
}

extern "C" {
    fn ANativeWindow_fromSurface(env: *mut jni::sys::JNIEnv, surface: JObject) -> *mut c_void;
}

#[no_mangle]
pub extern "C" fn android_main() {
    // FIXME(mukilan): this android_main is only present to stop
    // the java side 'System.loadLibrary('simpleservo') call from
    // failing due to undefined reference to android_main introduced
    // by winit's android-activity crate. There is no way to disable
    // this currently.
}

fn call<F>(env: &JNIEnv, f: F)
where
    F: Fn(&mut ServoGlue) -> Result<(), &str>,
{
    SERVO.with(|s| {
        if let Err(error) = match s.borrow_mut().as_mut() {
            Some(ref mut s) => (f)(s),
            None => Err("Servo not available in this thread"),
        } {
            throw(env, error);
        }
    });
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_version(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let v = simpleservo::servo_version();
    new_string(&env, &v).unwrap_or_else(|null| null)
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_init(
    env: JNIEnv,
    _: JClass,
    _activity: JObject,
    opts: JObject,
    callbacks_obj: JObject,
    surface: JObject,
) {
    let (mut opts, log, log_str, _gst_debug_str) = match get_options(&env, opts, surface) {
        Ok((opts, log, log_str, gst_debug_str)) => (opts, log, log_str, gst_debug_str),
        Err(err) => {
            throw(&env, &err);
            return;
        },
    };

    if log {
        // Note: Android debug logs are stripped from a release build.
        // debug!() will only show in a debug build. Use info!() if logs
        // should show up in adb logcat with a release build.
        let filters = [
            "servo",
            "simpleservo",
            "simpleservo::jniapi",
            "simpleservo::gl_glue::egl",
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
                .with_tag("simpleservo"),
        )
    }

    info!("init");

    redirect_stdout_to_logcat();

    let callbacks_ref = match env.new_global_ref(callbacks_obj) {
        Ok(r) => r,
        Err(_) => {
            throw(&env, "Failed to get global reference of callback argument");
            return;
        },
    };

    let wakeup = Box::new(WakeupCallback::new(callbacks_ref.clone(), &env));
    let callbacks = Box::new(HostCallbacks::new(callbacks_ref, &env));

    if let Err(err) = gl_glue::egl::init()
        .and_then(|egl_init| simpleservo::init(opts, egl_init.gl_wrapper, wakeup, callbacks))
    {
        throw(&env, err)
    };
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_setBatchMode(
    env: JNIEnv,
    _: JClass,
    batch: jboolean,
) {
    debug!("setBatchMode");
    call(&env, |s| s.set_batch_mode(batch == JNI_TRUE));
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_requestShutdown(env: JNIEnv, _class: JClass) {
    debug!("requestShutdown");
    call(&env, |s| s.request_shutdown());
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_deinit(_env: JNIEnv, _class: JClass) {
    debug!("deinit");
    simpleservo::deinit();
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_resize(
    env: JNIEnv,
    _: JClass,
    coordinates: JObject,
) {
    let coords = jni_coords_to_rust_coords(&env, coordinates);
    debug!("resize {:#?}", coords);
    match coords {
        Ok(coords) => call(&env, |s| s.resize(coords.clone())),
        Err(error) => throw(&env, &error),
    }
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_performUpdates(env: JNIEnv, _class: JClass) {
    debug!("performUpdates");
    call(&env, |s| s.perform_updates());
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_loadUri(
    env: JNIEnv,
    _class: JClass,
    url: JString,
) {
    debug!("loadUri");
    match env.get_string(url) {
        Ok(url) => {
            let url: String = url.into();
            call(&env, |s| s.load_uri(&url));
        },
        Err(_) => {
            throw(&env, "Failed to convert Java string");
        },
    };
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_reload(env: JNIEnv, _class: JClass) {
    debug!("reload");
    call(&env, |s| s.reload());
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_stop(env: JNIEnv, _class: JClass) {
    debug!("stop");
    call(&env, |s| s.stop());
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_refresh(env: JNIEnv, _class: JClass) {
    debug!("refresh");
    call(&env, |s| s.refresh());
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_goBack(env: JNIEnv, _class: JClass) {
    debug!("goBack");
    call(&env, |s| s.go_back());
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_goForward(env: JNIEnv, _class: JClass) {
    debug!("goForward");
    call(&env, |s| s.go_forward());
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_scrollStart(
    env: JNIEnv,
    _: JClass,
    dx: jint,
    dy: jint,
    x: jint,
    y: jint,
) {
    debug!("scrollStart");
    call(&env, |s| {
        s.scroll_start(dx as f32, dy as f32, x as i32, y as i32)
    });
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_scrollEnd(
    env: JNIEnv,
    _: JClass,
    dx: jint,
    dy: jint,
    x: jint,
    y: jint,
) {
    debug!("scrollEnd");
    call(&env, |s| {
        s.scroll_end(dx as f32, dy as f32, x as i32, y as i32)
    });
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_scroll(
    env: JNIEnv,
    _: JClass,
    dx: jint,
    dy: jint,
    x: jint,
    y: jint,
) {
    debug!("scroll");
    call(&env, |s| s.scroll(dx as f32, dy as f32, x as i32, y as i32));
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_touchDown(
    env: JNIEnv,
    _: JClass,
    x: jfloat,
    y: jfloat,
    pointer_id: jint,
) {
    debug!("touchDown");
    call(&env, |s| s.touch_down(x, y, pointer_id as i32));
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_touchUp(
    env: JNIEnv,
    _: JClass,
    x: jfloat,
    y: jfloat,
    pointer_id: jint,
) {
    debug!("touchUp");
    call(&env, |s| s.touch_up(x, y, pointer_id as i32));
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_touchMove(
    env: JNIEnv,
    _: JClass,
    x: jfloat,
    y: jfloat,
    pointer_id: jint,
) {
    debug!("touchMove");
    call(&env, |s| s.touch_move(x, y, pointer_id as i32));
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_touchCancel(
    env: JNIEnv,
    _: JClass,
    x: jfloat,
    y: jfloat,
    pointer_id: jint,
) {
    debug!("touchCancel");
    call(&env, |s| s.touch_cancel(x, y, pointer_id as i32));
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_pinchZoomStart(
    env: JNIEnv,
    _: JClass,
    factor: jfloat,
    x: jint,
    y: jint,
) {
    debug!("pinchZoomStart");
    call(&env, |s| {
        s.pinchzoom_start(factor as f32, x as u32, y as u32)
    });
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_pinchZoom(
    env: JNIEnv,
    _: JClass,
    factor: jfloat,
    x: jint,
    y: jint,
) {
    debug!("pinchZoom");
    call(&env, |s| s.pinchzoom(factor as f32, x as u32, y as u32));
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_pinchZoomEnd(
    env: JNIEnv,
    _: JClass,
    factor: jfloat,
    x: jint,
    y: jint,
) {
    debug!("pinchZoomEnd");
    call(&env, |s| s.pinchzoom_end(factor as f32, x as u32, y as u32));
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_click(
    env: JNIEnv,
    _: JClass,
    x: jfloat,
    y: jfloat,
) {
    debug!("click");
    call(&env, |s| s.click(x as f32, y as f32));
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_pauseCompositor(env: JNIEnv, _: JClass) {
    debug!("pauseCompositor");
    call(&env, |s| s.pause_compositor());
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_resumeCompositor(
    env: JNIEnv,
    _: JClass,
    surface: JObject,
    coordinates: JObject,
) {
    debug!("resumeCompositor");
    let widget = unsafe { ANativeWindow_fromSurface(env.get_native_interface(), surface) };
    let coords = jni_coords_to_rust_coords(&env, coordinates);
    match coords {
        Ok(coords) => call(&env, |s| s.resume_compositor(widget, coords.clone())),
        Err(error) => throw(&env, &error),
    }
}

#[no_mangle]
pub extern "C" fn Java_org_mozilla_servoview_JNIServo_mediaSessionAction(
    env: JNIEnv,
    _: JClass,
    action: jint,
) {
    debug!("mediaSessionAction");
    call(&env, |s| s.media_session_action((action as i32).into()));
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
        let env = self.jvm.attach_current_thread().unwrap();
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
    fn prompt_alert(&self, message: String, _trusted: bool) {
        debug!("prompt_alert");
        let env = self.jvm.get_env().unwrap();
        let s = match new_string(&env, &message) {
            Ok(s) => s,
            Err(_) => return,
        };
        let s = JValue::from(JObject::from(s));
        env.call_method(
            self.callbacks.as_obj(),
            "onAlert",
            "(Ljava/lang/String;)V",
            &[s],
        )
        .unwrap();
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

    fn on_load_started(&self) {
        debug!("on_load_started");
        let env = self.jvm.get_env().unwrap();
        env.call_method(self.callbacks.as_obj(), "onLoadStarted", "()V", &[])
            .unwrap();
    }

    fn on_load_ended(&self) {
        debug!("on_load_ended");
        let env = self.jvm.get_env().unwrap();
        env.call_method(self.callbacks.as_obj(), "onLoadEnded", "()V", &[])
            .unwrap();
    }

    fn on_shutdown_complete(&self) {
        debug!("on_shutdown_complete");
        let env = self.jvm.get_env().unwrap();
        env.call_method(self.callbacks.as_obj(), "onShutdownComplete", "()V", &[])
            .unwrap();
    }

    fn on_title_changed(&self, title: Option<String>) {
        debug!("on_title_changed");
        let env = self.jvm.get_env().unwrap();
        let title = title.unwrap_or_else(String::new);
        let s = match new_string(&env, &title) {
            Ok(s) => s,
            Err(_) => return,
        };
        let s = JValue::from(JObject::from(s));
        env.call_method(
            self.callbacks.as_obj(),
            "onTitleChanged",
            "(Ljava/lang/String;)V",
            &[s],
        )
        .unwrap();
    }

    fn on_allow_navigation(&self, url: String) -> bool {
        debug!("on_allow_navigation");
        let env = self.jvm.get_env().unwrap();
        let s = match new_string(&env, &url) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let s = JValue::from(JObject::from(s));
        let allow = env.call_method(
            self.callbacks.as_obj(),
            "onAllowNavigation",
            "(Ljava/lang/String;)Z",
            &[s],
        );
        match allow {
            Ok(allow) => return allow.z().unwrap(),
            Err(_) => return true,
        }
    }

    fn on_url_changed(&self, url: String) {
        debug!("on_url_changed");
        let env = self.jvm.get_env().unwrap();
        let s = match new_string(&env, &url) {
            Ok(s) => s,
            Err(_) => return,
        };
        let s = JValue::Object(JObject::from(s));
        env.call_method(
            self.callbacks.as_obj(),
            "onUrlChanged",
            "(Ljava/lang/String;)V",
            &[s],
        )
        .unwrap();
    }

    fn on_history_changed(&self, can_go_back: bool, can_go_forward: bool) {
        debug!("on_history_changed");
        let env = self.jvm.get_env().unwrap();
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
        let env = self.jvm.get_env().unwrap();
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

    fn get_clipboard_contents(&self) -> Option<String> {
        None
    }

    fn set_clipboard_contents(&self, _contents: String) {}

    fn on_media_session_metadata(&self, title: String, artist: String, album: String) {
        info!("on_media_session_metadata");
        let env = self.jvm.get_env().unwrap();
        let title = match new_string(&env, &title) {
            Ok(s) => s,
            Err(_) => return,
        };
        let title = JValue::Object(JObject::from(title));

        let artist = match new_string(&env, &artist) {
            Ok(s) => s,
            Err(_) => return,
        };
        let artist = JValue::Object(JObject::from(artist));

        let album = match new_string(&env, &album) {
            Ok(s) => s,
            Err(_) => return,
        };
        let album = JValue::Object(JObject::from(album));
        env.call_method(
            self.callbacks.as_obj(),
            "onMediaSessionMetadata",
            "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
            &[title, artist, album],
        )
        .unwrap();
    }

    fn on_media_session_playback_state_change(&self, state: MediaSessionPlaybackState) {
        info!("on_media_session_playback_state_change {:?}", state);
        let env = self.jvm.get_env().unwrap();
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
        let env = self.jvm.get_env().unwrap();
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

    fn on_devtools_started(&self, port: Result<u16, ()>, _token: String) {
        match port {
            Ok(p) => info!("Devtools Server running on port {}", p),
            Err(()) => error!("Error running devtools server"),
        }
    }

    fn show_context_menu(&self, _title: Option<String>, _items: Vec<String>) {}

    fn on_panic(&self, _reason: String, _backtrace: Option<String>) {}
}

extern "C" {
    pub fn __android_log_write(prio: c_int, tag: *const c_char, text: *const c_char) -> c_int;
}

fn redirect_stdout_to_logcat() {
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

        let tag = b"simpleservo\0".as_ptr() as _;

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
                unsafe {
                    __android_log_write(
                        3,
                        tag,
                        b"error in log thread; closing\0".as_ptr() as *const _,
                    );
                }
                return;
            } else {
                result as usize + cursor
            };

            // Only modify the portion of the buffer that contains real data.
            let buf = &mut buf[0..end];

            if let Some(last_newline_pos) = buf.iter().rposition(|&c| c == b'\n' as c_char) {
                buf[last_newline_pos] = b'\0' as c_char;
                unsafe {
                    __android_log_write(3, tag, buf.as_ptr());
                }
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
                unsafe {
                    __android_log_write(3, tag, buf.as_ptr());
                }
                cursor = 0;
            } else {
                cursor = end;
            }
        }
    });
}

fn throw(env: &JNIEnv, err: &str) {
    if let Err(e) = env.throw(("java/lang/Exception", err)) {
        warn!(
            "Failed to throw Java exception: `{}`. Exception was: `{}`",
            e, err
        );
    }
}

fn new_string(env: &JNIEnv, s: &str) -> Result<jstring, jstring> {
    match env.new_string(s) {
        Ok(s) => Ok(s.into_inner()),
        Err(_) => {
            throw(&env, "Couldn't create java string");
            Err(JObject::null().into_inner())
        },
    }
}

fn jni_coords_to_rust_coords(env: &JNIEnv, obj: JObject) -> Result<Coordinates, String> {
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
    let fb_width = get_non_null_field(env, obj, "fb_width", "I")?
        .i()
        .map_err(|_| "fb_width not an int")? as i32;
    let fb_height = get_non_null_field(env, obj, "fb_height", "I")?
        .i()
        .map_err(|_| "fb_height not an int")? as i32;
    Ok(Coordinates::new(x, y, width, height, fb_width, fb_height))
}

fn get_field<'a>(
    env: &'a JNIEnv,
    obj: JObject<'a>,
    field: &str,
    type_: &str,
) -> Result<Option<JValue<'a>>, String> {
    if env.get_field_id(obj, field, type_).is_err() {
        return Err(format!("Can't find `{}` field", field));
    }
    env.get_field(obj, field, type_)
        .map(|value| Some(value))
        .or_else(|_| Err(format!("Can't find `{}` field", field)))
}

fn get_non_null_field<'a>(
    env: &'a JNIEnv,
    obj: JObject<'a>,
    field: &str,
    type_: &str,
) -> Result<JValue<'a>, String> {
    match get_field(env, obj, field, type_)? {
        None => Err(format!("Field {} is null", field)),
        Some(f) => Ok(f),
    }
}

fn get_string(env: &JNIEnv, obj: JObject, field: &str) -> Result<Option<String>, String> {
    let value = get_field(env, obj, field, "Ljava/lang/String;")?;
    match value {
        Some(value) => {
            let string = value
                .l()
                .map_err(|_| format!("field `{}` is not an Object", field))?
                .into();
            Ok(env.get_string(string).map(|s| s.into()).ok())
        },
        None => Ok(None),
    }
}

fn get_options(
    env: &JNIEnv,
    opts: JObject,
    surface: JObject,
) -> Result<(InitOptions, bool, Option<String>, Option<String>), String> {
    let args = get_string(env, opts, "args")?;
    let url = get_string(env, opts, "url")?;
    let log_str = get_string(env, opts, "logStr")?;
    let gst_debug_str = get_string(env, opts, "gstDebugStr")?;
    let density = get_non_null_field(env, opts, "density", "F")?
        .f()
        .map_err(|_| "densitiy not a float")? as f32;
    let log = get_non_null_field(env, opts, "enableLogs", "Z")?
        .z()
        .map_err(|_| "enableLogs not a boolean")?;
    let coordinates = get_non_null_field(
        env,
        opts,
        "coordinates",
        "Lorg/mozilla/servoview/JNIServo$ServoCoordinates;",
    )?
    .l()
    .map_err(|_| "coordinates is not an object")?;
    let coordinates = jni_coords_to_rust_coords(&env, coordinates)?;

    let args = match args {
        Some(args) => serde_json::from_str(&args)
            .map_err(|_| "Invalid arguments. Servo arguments must be formatted as a JSON array")?,
        None => None,
    };

    let native_window = unsafe { ANativeWindow_fromSurface(env.get_native_interface(), surface) };

    // FIXME: enable JIT compilation on Android after the startup crash issue (#31134) is fixed.
    let mut prefs = HashMap::new();
    prefs.insert("js.baseline_interpreter.enabled".to_string(), false.into());
    prefs.insert("js.baseline_jit.enabled".to_string(), false.into());
    prefs.insert("js.ion.enabled".to_string(), false.into());

    let opts = InitOptions {
        args: args.unwrap_or(vec![]),
        coordinates,
        density,
        xr_discovery: None,
        surfman_integration: simpleservo::SurfmanIntegration::Widget(native_window),
        prefs: Some(prefs),
    };
    Ok((opts, log, log_str, gst_debug_str))
}
