/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use android_logger::{self, Filter};
use api::{self, EventLoopWaker, ServoGlue, SERVO, HostTrait, ReadFileTrait};
use gl_glue;
use jni::{JNIEnv, JavaVM};
use jni::objects::{GlobalRef, JClass, JObject, JString, JValue};
use jni::sys::{jboolean, jint, jstring};
use log::Level;
use std::sync::Arc;

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
#[allow(non_snake_case)]
pub fn Java_com_mozilla_servoview_NativeServo_version(env: JNIEnv, _class: JClass) -> jstring {
    let v = api::servo_version();
    let output = env.new_string(v).expect("Couldn't create java string");
    output.into_inner()
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn Java_com_mozilla_servoview_NativeServo_init(
    env: JNIEnv,
    _: JClass,
    url: JString,
    wakeup_obj: JObject,
    readfile_obj: JObject,
    callbacks_obj: JObject,
    width: jint,
    height: jint,
    log: jboolean,
) {
    if log == 1 {
        android_logger::init_once(
            Filter::default()
                .with_min_level(Level::Debug)
                .with_allowed_module_path("servobridge::api")
                .with_allowed_module_path("servobridge::jniwrapper"),
        );
    }

    debug!("init");

    let url = env.get_string(url)
        .expect("Couldn't get java string")
        .into();

    let wakeup = Box::new(WakeupCallback::new(wakeup_obj, &env));
    let readfile = Box::new(ReadFileCallback::new(readfile_obj, &env));
    let callbacks = Box::new(HostCallbacks::new(callbacks_obj, &env));

    let gl = gl_glue::egl::init();
    api::init(
        gl,
        url,
        wakeup,
        readfile,
        callbacks,
        width as u32,
        height as u32,
    ).or_else(|err| env.throw(("java/lang/Exception", err)))
        .expect("Error while throwing");
}

/// Needs to be called from the EGL thread
#[no_mangle]
#[allow(non_snake_case)]
pub fn Java_com_mozilla_servoview_NativeServo_resize(
    env: JNIEnv,
    _: JClass,
    width: jint,
    height: jint,
) {
    debug!("resize {}/{}", width, height);
    call(env, |s| s.resize(width as u32, height as u32));
}

/// This is the Servo heartbeat. This needs to be called
/// everytime wakeup is called.
#[no_mangle]
#[allow(non_snake_case)]
pub fn Java_com_mozilla_servoview_NativeServo_performUpdates(env: JNIEnv, _class: JClass) {
    debug!("performUpdates");
    call(env, |s| {
        s.perform_updates().and_then(|_| s.handle_servo_events())
    });
}

/// Load an URL. This needs to be a valid url.
#[no_mangle]
#[allow(non_snake_case)]
pub fn Java_com_mozilla_servoview_NativeServo_loadUri(env: JNIEnv, _class: JClass, url: JString) {
    debug!("loadUri");
    let url: String = env.get_string(url).unwrap().into();
    call(env, |s| s.load_uri(&url));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn Java_com_mozilla_servoview_NativeServo_reload(env: JNIEnv, _class: JClass) {
    debug!("reload");
    call(env, |s| s.reload());
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn Java_com_mozilla_servoview_NativeServo_goBack(env: JNIEnv, _class: JClass) {
    debug!("goBack");
    call(env, |s| s.go_back());
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn Java_com_mozilla_servoview_NativeServo_goForward(env: JNIEnv, _class: JClass) {
    debug!("goForward");
    call(env, |s| s.go_forward());
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn Java_com_mozilla_servoview_NativeServo_scrollStart(
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
#[allow(non_snake_case)]
pub fn Java_com_mozilla_servoview_NativeServo_scrollEnd(
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
#[allow(non_snake_case)]
pub fn Java_com_mozilla_servoview_NativeServo_scroll(
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
#[allow(non_snake_case)]
pub fn Java_com_mozilla_servoview_NativeServo_click(env: JNIEnv, _: JClass, x: jint, y: jint) {
    debug!("click");
    call(env, |s| s.click(x as u32, y as u32));
}

pub struct WakeupCallback {
    callback: GlobalRef,
    jvm: Arc<JavaVM>,
}

impl WakeupCallback {
    pub fn new(jobject: JObject, env: &JNIEnv) -> WakeupCallback {
        let jvm = Arc::new(env.get_java_vm().unwrap());
        WakeupCallback {
            callback: env.new_global_ref(jobject).unwrap(),
            jvm,
        }
    }
}

impl EventLoopWaker for WakeupCallback {
    fn clone(&self) -> Box<EventLoopWaker + Send> {
        Box::new(WakeupCallback {
            callback: self.callback.clone(),
            jvm: self.jvm.clone(),
        })
    }
    /// Will be called from any thread.
    /// Will be called to notify embedder that some events
    /// are available, and that perform_updates need to be called
    fn wake(&self) {
        debug!("wakeup");
        let env = self.jvm.attach_current_thread().unwrap();
        env.call_method(self.callback.as_obj(), "wakeup", "()V", &[])
            .unwrap();
    }
}

pub struct ReadFileCallback {
    callback: GlobalRef,
    jvm: JavaVM,
}

impl ReadFileCallback {
    pub fn new(jobject: JObject, env: &JNIEnv) -> ReadFileCallback {
        let jvm = env.get_java_vm().unwrap();
        let callback = env.new_global_ref(jobject).unwrap();
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
            self.callback.as_obj(),
            "readfile",
            "(Ljava/lang/String;)[B",
            &[s],
        );
        let array = array.unwrap().l().unwrap().into_inner();
        env.convert_byte_array(array).unwrap()
    }
}

impl HostCallbacks {
    pub fn new(jobject: JObject, env: &JNIEnv) -> HostCallbacks {
        let jvm = env.get_java_vm().unwrap();
        HostCallbacks {
            callbacks: env.new_global_ref(jobject).unwrap(),
            jvm,
        }
    }
}

impl HostTrait for HostCallbacks {
    /// Will be called from the thread used for the init call
    /// Will be called when the GL buffer has been updated.
    fn flush(&self) {
        debug!("flush");
        let env = self.jvm.get_env().unwrap();
        env.call_method(self.callbacks.as_obj(), "flush", "()V", &[])
            .unwrap();
    }

    /// Page starts loading.
    /// "Reload button" becomes "Stop button".
    /// Throbber starts spinning.
    fn on_load_started(&self) {
        debug!("on_load_started");
        let env = self.jvm.get_env().unwrap();
        env.call_method(self.callbacks.as_obj(), "onLoadStarted", "()V", &[])
            .unwrap();
    }

    /// Page has loaded.
    /// "Stop button" becomes "Reload button".
    /// Throbber stops spinning.
    fn on_load_ended(&self) {
        debug!("on_load_ended");
        let env = self.jvm.get_env().unwrap();
        env.call_method(self.callbacks.as_obj(), "onLoadEnded", "()V", &[])
            .unwrap();
    }

    /// Title changed.
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

    /// Back/forward state changed.
    /// Back/forward buttons need to be disabled/enabled.
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

    /// Animations are (not) running.
    /// If animations are running, the embedder needs to call perform_updates
    /// constantly (flush/swap_buffer will block to ensure it doesn't do unnecessary
    /// flushes.
    /// If animations are not running, the embedder needs to wait for the wakeup
    /// call to call perform_updates.
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
