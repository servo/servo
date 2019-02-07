/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(non_snake_case)]

#[macro_use]
extern crate log;

use android_logger::{self, Filter};
use jni::objects::{GlobalRef, JClass, JObject, JString, JValue};
use jni::sys::{jboolean, jfloat, jint, jstring, JNI_TRUE};
use jni::{errors, JNIEnv, JavaVM};
use libc::{dup2, pipe, read};
use log::Level;
use simpleservo::{self, gl_glue, EventLoopWaker, HostTrait, InitOptions, ServoGlue, SERVO};
use std::os::raw::{c_char, c_int, c_void};
use std::sync::Arc;
use std::thread;

struct HostCallbacks {
    callbacks: GlobalRef,
    jvm: JavaVM,
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
pub fn Java_org_mozilla_servoview_JNIServo_version(env: JNIEnv, _class: JClass) -> jstring {
    let v = simpleservo::servo_version();
    new_string(&env, &v).unwrap_or_else(|null| null)
}

#[no_mangle]
pub fn Java_org_mozilla_servoview_JNIServo_init(
    env: JNIEnv,
    _: JClass,
    activity: JObject,
    opts: JObject,
    callbacks_obj: JObject,
) {
    let (opts, log, log_str) = match get_options(&env, opts) {
        Ok((opts, log, log_str)) => (opts, log, log_str),
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
            "simpleservo::api",
            "simpleservo::jniapi",
            "simpleservo::gl_glue::egl",
            // Show JS errors by default.
            "script::dom::bindings::error",
            // Show GL errors by default.
            "canvas::webgl_thread",
            "compositing::compositor",
            "constellation::constellation",
        ];
        let mut filter = Filter::default().with_min_level(Level::Debug);
        for &module in &filters {
            filter = filter.with_allowed_module_path(module);
        }
        if let Some(log_str) = log_str {
            for module in log_str.split(',') {
                filter = filter.with_allowed_module_path(module);
            }
        }
        android_logger::init_once(filter, Some("simpleservo"));
    }

    info!("init");

    initialize_android_glue(&env, activity);
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

    if let Err(err) =
        gl_glue::egl::init().and_then(|gl| simpleservo::init(opts, gl, wakeup, callbacks))
    {
        throw(&env, err)
    };
}

#[no_mangle]
pub fn Java_org_mozilla_servoview_JNIServo_setBatchMode(env: JNIEnv, _: JClass, batch: jboolean) {
    debug!("setBatchMode");
    call(&env, |s| s.set_batch_mode(batch == JNI_TRUE));
}

#[no_mangle]
pub fn Java_org_mozilla_servoview_JNIServo_requestShutdown(env: JNIEnv, _class: JClass) {
    debug!("requestShutdown");
    call(&env, |s| s.request_shutdown());
}

#[no_mangle]
pub fn Java_org_mozilla_servoview_JNIServo_deinit(_env: JNIEnv, _class: JClass) {
    debug!("deinit");
    simpleservo::deinit();
}

#[no_mangle]
pub fn Java_org_mozilla_servoview_JNIServo_resize(
    env: JNIEnv,
    _: JClass,
    width: jint,
    height: jint,
) {
    debug!("resize {}/{}", width, height);
    call(&env, |s| s.resize(width as u32, height as u32));
}

#[no_mangle]
pub fn Java_org_mozilla_servoview_JNIServo_performUpdates(env: JNIEnv, _class: JClass) {
    debug!("performUpdates");
    call(&env, |s| s.perform_updates());
}

#[no_mangle]
pub fn Java_org_mozilla_servoview_JNIServo_loadUri(env: JNIEnv, _class: JClass, url: JString) {
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
pub fn Java_org_mozilla_servoview_JNIServo_reload(env: JNIEnv, _class: JClass) {
    debug!("reload");
    call(&env, |s| s.reload());
}

#[no_mangle]
pub fn Java_org_mozilla_servoview_JNIServo_stop(env: JNIEnv, _class: JClass) {
    debug!("stop");
    call(&env, |s| s.stop());
}

#[no_mangle]
pub fn Java_org_mozilla_servoview_JNIServo_refresh(env: JNIEnv, _class: JClass) {
    debug!("refresh");
    call(&env, |s| s.refresh());
}

#[no_mangle]
pub fn Java_org_mozilla_servoview_JNIServo_goBack(env: JNIEnv, _class: JClass) {
    debug!("goBack");
    call(&env, |s| s.go_back());
}

#[no_mangle]
pub fn Java_org_mozilla_servoview_JNIServo_goForward(env: JNIEnv, _class: JClass) {
    debug!("goForward");
    call(&env, |s| s.go_forward());
}

#[no_mangle]
pub fn Java_org_mozilla_servoview_JNIServo_scrollStart(
    env: JNIEnv,
    _: JClass,
    dx: jint,
    dy: jint,
    x: jint,
    y: jint,
) {
    debug!("scrollStart");
    call(&env, |s| {
        s.scroll_start(dx as i32, dy as i32, x as u32, y as u32)
    });
}

#[no_mangle]
pub fn Java_org_mozilla_servoview_JNIServo_scrollEnd(
    env: JNIEnv,
    _: JClass,
    dx: jint,
    dy: jint,
    x: jint,
    y: jint,
) {
    debug!("scrollEnd");
    call(&env, |s| {
        s.scroll_end(dx as i32, dy as i32, x as u32, y as u32)
    });
}

#[no_mangle]
pub fn Java_org_mozilla_servoview_JNIServo_scroll(
    env: JNIEnv,
    _: JClass,
    dx: jint,
    dy: jint,
    x: jint,
    y: jint,
) {
    debug!("scroll");
    call(&env, |s| s.scroll(dx as i32, dy as i32, x as u32, y as u32));
}

#[no_mangle]
pub fn Java_org_mozilla_servoview_JNIServo_touchDown(
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
pub fn Java_org_mozilla_servoview_JNIServo_touchUp(
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
pub fn Java_org_mozilla_servoview_JNIServo_touchMove(
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
pub fn Java_org_mozilla_servoview_JNIServo_touchCancel(
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
pub fn Java_org_mozilla_servoview_JNIServo_pinchZoomStart(
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
pub fn Java_org_mozilla_servoview_JNIServo_pinchZoom(
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
pub fn Java_org_mozilla_servoview_JNIServo_pinchZoomEnd(
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
pub fn Java_org_mozilla_servoview_JNIServo_click(env: JNIEnv, _: JClass, x: jint, y: jint) {
    debug!("click");
    call(&env, |s| s.click(x as u32, y as u32));
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
    fn clone(&self) -> Box<EventLoopWaker + Send> {
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
    fn flush(&self) {
        debug!("flush");
        let env = self.jvm.get_env().unwrap();
        env.call_method(self.callbacks.as_obj(), "flush", "()V", &[])
            .unwrap();
    }

    fn make_current(&self) {
        debug!("make_current");
        let env = self.jvm.get_env().unwrap();
        env.call_method(self.callbacks.as_obj(), "makeCurrent", "()V", &[])
            .unwrap();
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

    fn on_title_changed(&self, title: String) {
        debug!("on_title_changed");
        let env = self.jvm.get_env().unwrap();
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
}

fn initialize_android_glue(env: &JNIEnv, activity: JObject) {
    use android_injected_glue::{ffi, ANDROID_APP};

    // From jni-rs to android_injected_glue

    let mut app: ffi::android_app = unsafe { std::mem::zeroed() };
    let mut native_activity: ffi::ANativeActivity = unsafe { std::mem::zeroed() };

    let clazz = Box::into_raw(Box::new(env.new_global_ref(activity).unwrap()));
    native_activity.clazz = unsafe { (*clazz).as_obj().into_inner() as *mut c_void };

    let vm = env.get_java_vm().unwrap().get_java_vm_pointer();
    native_activity.vm = vm as *mut ffi::_JavaVM;

    app.activity = Box::into_raw(Box::new(native_activity));

    unsafe {
        ANDROID_APP = Box::into_raw(Box::new(app));
    }
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

fn get_field<'a>(
    env: &'a JNIEnv,
    obj: JObject,
    field: &str,
    type_: &str,
) -> Result<Option<JValue<'a>>, String> {
    if env.get_field_id(obj, field, type_).is_err() {
        return Err(format!("Can't find `{}` field", &field));
    }
    env.get_field(obj, field, type_)
        .map(|value| Some(value))
        .or_else(|e| match *e.kind() {
            errors::ErrorKind::NullPtr(_) => Ok(None),
            _ => Err(format!(
                "Can't find `{}` field: {}",
                &field,
                e.description()
            )),
        })
}

fn get_non_null_field<'a>(
    env: &'a JNIEnv,
    obj: JObject,
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

fn get_options(env: &JNIEnv, opts: JObject) -> Result<(InitOptions, bool, Option<String>), String> {
    let args = get_string(env, opts, "args")?;
    let url = get_string(env, opts, "url")?;
    let log_str = get_string(env, opts, "logStr")?;
    let width = get_non_null_field(env, opts, "width", "I")?
        .i()
        .map_err(|_| "width not an int")? as u32;
    let height = get_non_null_field(env, opts, "height", "I")?
        .i()
        .map_err(|_| "height not an int")? as u32;
    let density = get_non_null_field(env, opts, "density", "F")?
        .f()
        .map_err(|_| "densitiy not a float")? as f32;
    let log = get_non_null_field(env, opts, "enableLogs", "Z")?
        .z()
        .map_err(|_| "enableLogs not a boolean")?;
    let enable_subpixel_text_antialiasing =
        get_non_null_field(env, opts, "enableSubpixelTextAntialiasing", "Z")?
            .z()
            .map_err(|_| "enableSubpixelTextAntialiasing not a boolean")?;
    let vr_pointer = get_non_null_field(env, opts, "VRExternalContext", "J")?
        .j()
        .map_err(|_| "VRExternalContext is not a long")? as *mut c_void;
    let opts = InitOptions {
        args,
        url,
        width,
        height,
        density,
        enable_subpixel_text_antialiasing,
        vr_pointer: if vr_pointer.is_null() {
            None
        } else {
            Some(vr_pointer)
        },
    };
    Ok((opts, log, log_str))
}
