/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(non_snake_case)]

use android_logger::{self, Filter};
use api::{self, EventLoopWaker, ServoGlue, SERVO, HostTrait, ReadFileTrait};
use gl_glue;
use jni::{JNIEnv, JavaVM};
use jni::objects::{GlobalRef, JClass, JObject, JString, JValue};
use jni::sys::{jboolean, jint, jstring, JNI_TRUE};
use log::Level;
use std;
use std::os::raw::c_void;
use std::sync::{Arc, Mutex};

struct HostCallbacks {
    callbacks: GlobalRef,
    jvm: JavaVM,
}

fn call<F>(env: JNIEnv, f: F)
where
    F: Fn(&mut ServoGlue) -> Result<(), &'static str>,
{
    SERVO.with(|s| {
        if let Err(error) = match s.borrow_mut().as_mut() {
            Some(ref mut s) => (f)(s),
            None => Err("Servo not available in this thread"),
        } {
            env.throw(("java/lang/Exception", error))
                .expect("Error while throwing");
        }
    });
}

#[no_mangle]
pub fn Java_com_mozilla_servoview_JNIServo_version(env: JNIEnv, _class: JClass) -> jstring {
    let v = api::servo_version();
    let output = env.new_string(v).expect("Couldn't create java string");
    output.into_inner()
}

#[no_mangle]
pub fn Java_com_mozilla_servoview_JNIServo_init(
    env: JNIEnv,
    _: JClass,
    activity: JObject,
    args: JString,
    url: JString,
    callbacks_obj: JObject,
    width: jint,
    height: jint,
    log: jboolean,
) {
    if log == JNI_TRUE {
        // Note: Android debug logs are stripped from a release build.
        // debug!() will only show in a debug build. Use info!() if logs
        // should show up in adb logcat with a release build.
        android_logger::init_once(
            Filter::default()
                .with_min_level(Level::Debug)
                .with_allowed_module_path("simpleservo::api")
                .with_allowed_module_path("simpleservo::jniapi")
                .with_allowed_module_path("simpleservo::gl_glue::egl"),
            Some("simpleservo")
        );
    }

    info!("init");

    initialize_android_glue(&env, activity);

    let args = env.get_string(args)
        .expect("Couldn't get java string")
        .into();

    let url = if url.is_null() {
        None
    } else {
        Some(env.get_string(url).expect("Couldn't get java string").into())
    };

    let callbacks_ref = env.new_global_ref(callbacks_obj).unwrap();

    let wakeup = Box::new(WakeupCallback::new(callbacks_ref.clone(), &env));
    let readfile = Box::new(ReadFileCallback::new(callbacks_ref.clone(), &env));
    let callbacks = Box::new(HostCallbacks::new(callbacks_ref, &env));

    gl_glue::egl::init().and_then(|gl| {
        api::init(
            gl,
            args,
            url,
            wakeup,
            readfile,
            callbacks,
            width as u32,
            height as u32)
    }).or_else(|err| {
        env.throw(("java/lang/Exception", err))
    }).unwrap();
}

#[no_mangle]
pub fn Java_com_mozilla_servoview_JNIServo_setBatchMode(
    env: JNIEnv,
    _: JClass,
    batch: jboolean,
) {
    debug!("setBatchMode");
    call(env, |s| s.set_batch_mode(batch == JNI_TRUE));
}

#[no_mangle]
pub fn Java_com_mozilla_servoview_JNIServo_resize(
    env: JNIEnv,
    _: JClass,
    width: jint,
    height: jint,
) {
    debug!("resize {}/{}", width, height);
    call(env, |s| s.resize(width as u32, height as u32));
}

#[no_mangle]
pub fn Java_com_mozilla_servoview_JNIServo_performUpdates(env: JNIEnv, _class: JClass) {
    debug!("performUpdates");
    call(env, |s| s.perform_updates());
}

#[no_mangle]
pub fn Java_com_mozilla_servoview_JNIServo_loadUri(env: JNIEnv, _class: JClass, url: JString) {
    debug!("loadUri");
    let url: String = env.get_string(url).unwrap().into();
    call(env, |s| s.load_uri(&url));
}

#[no_mangle]
pub fn Java_com_mozilla_servoview_JNIServo_reload(env: JNIEnv, _class: JClass) {
    debug!("reload");
    call(env, |s| s.reload());
}

#[no_mangle]
pub fn Java_com_mozilla_servoview_JNIServo_stop(env: JNIEnv, _class: JClass) {
    debug!("stop");
    call(env, |s| s.stop());
}

#[no_mangle]
pub fn Java_com_mozilla_servoview_JNIServo_goBack(env: JNIEnv, _class: JClass) {
    debug!("goBack");
    call(env, |s| s.go_back());
}

#[no_mangle]
pub fn Java_com_mozilla_servoview_JNIServo_goForward(env: JNIEnv, _class: JClass) {
    debug!("goForward");
    call(env, |s| s.go_forward());
}

#[no_mangle]
pub fn Java_com_mozilla_servoview_JNIServo_scrollStart(
    env: JNIEnv,
    _: JClass,
    dx: jint,
    dy: jint,
    x: jint,
    y: jint,
) {
    debug!("scrollStart");
    call(env, |s| s.scroll_start(dx as i32, dy as i32, x as u32, y as u32));
}

#[no_mangle]
pub fn Java_com_mozilla_servoview_JNIServo_scrollEnd(
    env: JNIEnv,
    _: JClass,
    dx: jint,
    dy: jint,
    x: jint,
    y: jint,
) {
    debug!("scrollEnd");
    call(env, |s| s.scroll_end(dx as i32, dy as i32, x as u32, y as u32));
}


#[no_mangle]
pub fn Java_com_mozilla_servoview_JNIServo_scroll(
    env: JNIEnv,
    _: JClass,
    dx: jint,
    dy: jint,
    x: jint,
    y: jint,
) {
    debug!("scroll");
    call(env, |s| s.scroll(dx as i32, dy as i32, x as u32, y as u32));
}

#[no_mangle]
pub fn Java_com_mozilla_servoview_JNIServo_click(env: JNIEnv, _: JClass, x: jint, y: jint) {
    debug!("click");
    call(env, |s| s.click(x as u32, y as u32));
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

pub struct ReadFileCallback {
    callback: Mutex<GlobalRef>,
    jvm: JavaVM,
}

impl ReadFileCallback {
    pub fn new(callback: GlobalRef, env: &JNIEnv) -> ReadFileCallback {
        let jvm = env.get_java_vm().unwrap();
        let callback = Mutex::new(callback);
        ReadFileCallback { callback, jvm }
    }
}

impl ReadFileTrait for ReadFileCallback {
    fn readfile(&self, file: &str) -> Vec<u8> {
        // FIXME: we'd rather use attach_current_thread but it detaches the VM too early.
        let env = self.jvm.attach_current_thread_as_daemon().unwrap();
        let s = env.new_string(&file)
            .expect("Couldn't create java string")
            .into_inner();
        let s = JValue::from(JObject::from(s));
        let array = env.call_method(
            self.callback.lock().unwrap().as_obj(),
            "readfile",
            "(Ljava/lang/String;)[B",
            &[s],
        );
        let array = array.unwrap().l().unwrap().into_inner();
        env.convert_byte_array(array).unwrap()
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

    fn on_title_changed(&self, title: String) {
        debug!("on_title_changed");
        let env = self.jvm.get_env().unwrap();
        let s = env.new_string(&title)
            .expect("Couldn't create java string")
            .into_inner();
        let s = JValue::from(JObject::from(s));
        env.call_method(
            self.callbacks.as_obj(),
            "onTitleChanged",
            "(Ljava/lang/String;)V",
            &[s],
        ).unwrap();
    }

    fn on_url_changed(&self, url: String) {
        debug!("on_url_changed");
        let env = self.jvm.get_env().unwrap();
        let s = env.new_string(&url)
            .expect("Couldn't create java string")
            .into_inner();
        let s = JValue::Object(JObject::from(s));
        env.call_method(
            self.callbacks.as_obj(),
            "onUrlChanged",
            "(Ljava/lang/String;)V",
            &[s],
        ).unwrap();
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
        ).unwrap();
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
        ).unwrap();
    }
}

fn initialize_android_glue(env: &JNIEnv, activity: JObject) {
    use android_injected_glue::{ANDROID_APP, ffi};

    // From jni-rs to android_injected_glue

    let mut app: ffi::android_app = unsafe {
        std::mem::zeroed()
    };
    let mut native_activity: ffi::ANativeActivity = unsafe {
        std::mem::zeroed()
    };

    let clazz = Box::into_raw(Box::new(env.new_global_ref(activity).unwrap()));
    native_activity.clazz = unsafe {
        (*clazz).as_obj().into_inner() as *mut c_void
    };

    let vm = env.get_java_vm().unwrap().get_java_vm_pointer();
    native_activity.vm = vm as *mut ffi::_JavaVM;

    app.activity = Box::into_raw(Box::new(native_activity));

    unsafe {
        ANDROID_APP = Box::into_raw(Box::new(app));
    }
}
