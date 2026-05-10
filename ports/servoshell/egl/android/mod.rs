/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(non_snake_case)]

use std::cell::RefCell;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::{Arc, OnceLock};

use android_logger::{self, Config, FilterBuilder};
use euclid::{Point2D, Rect, Scale, Size2D};
use jni::errors::{Error, ThrowRuntimeExAndDefault};
use jni::objects::{Global, JClass, JObject, JString, JValue, JValueOwned};
use jni::strings::JNIStr;
use jni::sys::{jboolean, jfloat, jint, jobject};
use jni::{Env, EnvUnowned, JavaVM, jni_sig, jni_str};
use keyboard_types::{Key, NamedKey};
use log::{debug, error, info, warn};
use raw_window_handle::{
    AndroidDisplayHandle, AndroidNdkWindowHandle, DisplayHandle, RawDisplayHandle, RawWindowHandle,
    WindowHandle,
};
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

static CALLBACK_OBJECT: OnceLock<Global<JObject<'static>>> = OnceLock::new();

fn callback_ref() -> &'static JObject<'static> {
    CALLBACK_OBJECT.get().expect("Servo init failed").as_ref()
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

fn call<F>(env: &mut Env, f: F)
where
    F: FnOnce(&App),
{
    APP.with(|app| match app.borrow().as_ref() {
        Some(app) => (f)(app),
        None => throw(env, jni_str!("Servo not available in this thread")),
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_version<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
) -> JString<'local> {
    let version = crate::VERSION;
    env.with_env(|env| -> jni::errors::Result<_> { env.new_string(version) })
        .resolve::<ThrowRuntimeExAndDefault>()
}

/// Initialize Servo. At that point, we need a valid GL context. In the future, this will
/// be done in multiple steps.
#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_init<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    _activity: JObject<'local>,
    opts: JObject<'local>,
    callbacks_obj: JObject<'local>,
    surface: JObject<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        let (init_opts, log, log_str, _gst_debug_str) = get_options(env, &opts, &surface)?;

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
                "servo_canvas::webgl_thread",
                "paint::paint",
                "servo_constellation::constellation",
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

        let callbacks: Global<JObject<'static>> = env.new_global_ref(callbacks_obj)?;

        CALLBACK_OBJECT
            .set(callbacks)
            .expect("CALLBACK_OBJECT was already initialized.");

        let jvm = env.get_java_vm()?;
        let event_loop_waker = Box::new(WakeupCallback::new(jvm.clone()));

        let host = Rc::new(HostCallbacks::new(jvm));

        crate::init_crypto();

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
                None,
            );
            *app.borrow_mut() = Some(new_app);
        });
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_setExperimentalMode<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    enable: jboolean,
) {
    debug!("setExperimentalMode {enable}");
    env.with_env(|env| -> jni::errors::Result<_> {
        call(env, |s| {
            for pref in EXPERIMENTAL_PREFS {
                s.servo().set_preference(pref, PrefValue::Bool(enable));
            }
        });
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_resize<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    coordinates: JObject<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        let viewport_rect = jni_coordinate_to_rust_viewport_rect(env, &coordinates)?;
        debug!("resize {viewport_rect:#?}");
        call(env, |s| s.resize(viewport_rect));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_performUpdates<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("performUpdates");
        call(env, |app| app.spin_event_loop());
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_loadUri<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    url: JString<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("loadUri");
        call(env, |s| s.load_uri(&url.to_string()));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_reload<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("reload");
        call(env, |s| s.reload());
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_stop<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("stop");
        call(env, |s| s.stop());
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_goBack<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("goBack");
        call(env, |s| s.go_back());
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_goForward<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("goForward");
        call(env, |s| s.go_forward());
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_scroll<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    dx: jint,
    dy: jint,
    x: jint,
    y: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("scroll");
        call(env, |s| s.scroll(dx as f32, dy as f32, x as f32, y as f32));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_doFrame<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        call(env, |s| s.notify_vsync());
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
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
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    keycode: jint,
    unicode: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("keydown {keycode}");
        if let Some(key) = key_from_unicode_keycode(unicode as u32, keycode) {
            call(env, move |s| s.key_down(key));
        }
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_keyup<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    keycode: jint,
    unicode: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("keyup {keycode}");
        if let Some(key) = key_from_unicode_keycode(unicode as u32, keycode) {
            call(env, move |s| s.key_up(key));
        }
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_touchDown<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    x: jfloat,
    y: jfloat,
    pointer_id: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("touchDown");
        call(env, |s| s.touch_down(x, y, pointer_id));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_touchUp<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    x: jfloat,
    y: jfloat,
    pointer_id: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("touchUp");
        call(env, |s| s.touch_up(x, y, pointer_id));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_touchMove<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    x: jfloat,
    y: jfloat,
    pointer_id: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("touchMove");
        call(env, |s| s.touch_move(x, y, pointer_id));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_touchCancel<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    x: jfloat,
    y: jfloat,
    pointer_id: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("touchCancel");
        call(env, |s| s.touch_cancel(x, y, pointer_id));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_pinchZoomStart<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    factor: jfloat,
    x: jfloat,
    y: jfloat,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("pinchZoomStart");
        call(env, |s| s.pinchzoom_start(factor, x, y));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_pinchZoom<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    factor: jfloat,
    x: jfloat,
    y: jfloat,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("pinchZoom");
        call(env, |s| s.pinchzoom(factor, x, y));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_pinchZoomEnd<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    factor: jfloat,
    x: jfloat,
    y: jfloat,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("pinchZoomEnd");
        call(env, |s| s.pinchzoom_end(factor, x, y));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_click(
    mut env: EnvUnowned,
    _: JClass,
    x: jfloat,
    y: jfloat,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("click");
        call(env, |s| {
            s.mouse_down(x, y, MouseButton::Left);
            s.mouse_up(x, y, MouseButton::Left);
        });
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_pausePainting(
    mut env: EnvUnowned,
    _: JClass<'_>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("pausePainting");
        call(env, |s| s.pause_painting());
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_resumePainting<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    surface: JObject<'local>,
    coordinates: JObject<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("resumePainting");
        let viewport_rect = jni_coordinate_to_rust_viewport_rect(env, &coordinates)?;
        let (_, window_handle) = display_and_window_handle(env, &surface);

        call(env, |s| {
            s.resume_painting(window_handle, viewport_rect);
        });
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_mediaSessionAction<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    action: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
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
            _ => {
                warn!("Ignoring unknown MediaSessionAction");
                return Ok(());
            },
        };
        call(env, |s| s.media_session_action(action.clone()));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

pub struct WakeupCallback {
    jvm: Arc<JavaVM>,
}

impl WakeupCallback {
    fn new(jvm: JavaVM) -> WakeupCallback {
        WakeupCallback { jvm: Arc::new(jvm) }
    }
}

impl EventLoopWaker for WakeupCallback {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        let jvm = self.jvm.clone();
        Box::new(WakeupCallback { jvm })
    }
    fn wake(&self) {
        debug!("wakeup");
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                env.call_method(callback_ref(), jni_str!("wakeup"), jni_sig!("()V"), &[])?;
                Ok(())
            })
            .unwrap();
    }
}

impl HostCallbacks {
    fn new(jvm: JavaVM) -> HostCallbacks {
        HostCallbacks { jvm }
    }
}

impl HostTrait for HostCallbacks {
    fn show_alert(&self, message: String) {
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                let Ok(string) = new_string_as_jvalue(env, &message) else {
                    return Ok(());
                };
                env.call_method(
                    callback_ref(),
                    jni_str!("onAlert"),
                    jni_sig!("(Ljava/lang/String;)V"),
                    &[(&string).into()],
                )?;
                Ok(())
            })
            .unwrap();
    }

    fn notify_load_status_changed(&self, load_status: LoadStatus) {
        debug!("notify_load_status_changed: {load_status:?}");
        self.jvm
            .attach_current_thread(|env| match load_status {
                LoadStatus::Started => env
                    .call_method(
                        callback_ref(),
                        jni_str!("onLoadStarted"),
                        jni_sig!("()V"),
                        &[],
                    )
                    .map(|_| ()),
                LoadStatus::HeadParsed => Ok(()),
                LoadStatus::Complete => env
                    .call_method(
                        callback_ref(),
                        jni_str!("onLoadEnded"),
                        jni_sig!("()V"),
                        &[],
                    )
                    .map(|_| ()),
            })
            .unwrap();
    }

    fn on_shutdown_complete(&self) {
        debug!("on_shutdown_complete");
    }

    fn on_title_changed(&self, title: Option<String>) {
        debug!("on_title_changed");
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                let title = title.unwrap_or_default();
                let Ok(title_string) = new_string_as_jvalue(env, &title) else {
                    return Ok(());
                };
                env.call_method(
                    callback_ref(),
                    jni_str!("onTitleChanged"),
                    jni_sig!("(Ljava/lang/String;)V"),
                    &[(&title_string).into()],
                )?;
                Ok(())
            })
            .unwrap();
    }

    fn on_url_changed(&self, url: String) {
        debug!("on_url_changed");
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                let Ok(url_string) = new_string_as_jvalue(env, &url) else {
                    return Ok(());
                };

                env.call_method(
                    callback_ref(),
                    jni_str!("onUrlChanged"),
                    jni_sig!("(Ljava/lang/String;)V"),
                    &[(&url_string).into()],
                )?;
                Ok(())
            })
            .unwrap();
    }

    fn on_history_changed(&self, can_go_back: bool, can_go_forward: bool) {
        debug!("on_history_changed");
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                let can_go_back = JValue::Bool(can_go_back as jboolean);
                let can_go_forward = JValue::Bool(can_go_forward as jboolean);
                env.call_method(
                    callback_ref(),
                    jni_str!("onHistoryChanged"),
                    jni_sig!("(ZZ)V"),
                    &[can_go_back, can_go_forward],
                )?;
                Ok(())
            })
            .unwrap();
    }

    fn on_ime_show(&self, _: InputMethodControl) {
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                env.call_method(callback_ref(), jni_str!("onImeShow"), jni_sig!("()V"), &[])?;
                Ok(())
            })
            .unwrap();
    }

    fn on_ime_hide(&self) {
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                env.call_method(callback_ref(), jni_str!("onImeHide"), jni_sig!("()V"), &[])?;
                Ok(())
            })
            .unwrap();
    }

    fn on_media_session_metadata(&self, title: String, artist: String, album: String) {
        info!("on_media_session_metadata");
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                let Ok(title) = new_string_as_jvalue(env, &title) else {
                    return Ok(());
                };

                let Ok(artist) = new_string_as_jvalue(env, &artist) else {
                    return Ok(());
                };

                let Ok(album) = new_string_as_jvalue(env, &album) else {
                    return Ok(());
                };

                env.call_method(
                    callback_ref(),
                    jni_str!("onMediaSessionMetadata"),
                    jni_sig!("(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V"),
                    &[(&title).into(), (&artist).into(), (&album).into()],
                )?;
                Ok(())
            })
            .unwrap();
    }

    fn on_media_session_playback_state_change(&self, state: MediaSessionPlaybackState) {
        info!("on_media_session_playback_state_change {:?}", state);
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                let state = state as i32;
                let state = JValue::Int(state as jint);
                env.call_method(
                    callback_ref(),
                    jni_str!("onMediaSessionPlaybackStateChange"),
                    jni_sig!("(I)V"),
                    &[state],
                )?;
                Ok(())
            })
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
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                let duration = JValue::Float(duration as jfloat);
                let position = JValue::Float(position as jfloat);
                let playback_rate = JValue::Float(playback_rate as jfloat);

                env.call_method(
                    callback_ref(),
                    jni_str!("onMediaSessionSetPositionState"),
                    jni_sig!("(FFF)V"),
                    &[duration, position, playback_rate],
                )?;
                Ok(())
            })
            .unwrap();
    }

    fn on_panic(&self, _reason: String, _backtrace: Option<String>) {}
}

unsafe extern "C" {
    pub fn __android_log_write(prio: c_int, tag: *const c_char, text: *const c_char) -> c_int;
}

fn throw(env: &mut Env, err: &JNIStr) {
    if let Err(e) = env.throw(err) {
        warn!(
            "Failed to throw Java exception: `{}`. Exception was: `{}`",
            e, err
        );
    }
}

fn new_string_as_jvalue<'local>(
    env: &mut Env<'local>,
    input_string: &str,
) -> Result<JValueOwned<'local>, &'static str> {
    let jstring = match env.new_string(input_string) {
        Ok(jstring) => jstring,
        Err(_) => {
            throw(env, jni_str!("Couldn't create Java string"));
            return Err("Couldn't create Java string");
        },
    };
    Ok(JValueOwned::from(jstring))
}

fn jni_coordinate_to_rust_viewport_rect<'local>(
    env: &mut Env<'local>,
    obj: &JObject<'local>,
) -> Result<Rect<i32, DevicePixel>, Error> {
    let x = env.get_field(obj, jni_str!("x"), jni_sig!("I"))?.i()?;
    let y = env.get_field(obj, jni_str!("y"), jni_sig!("I"))?.i()?;

    let width = env.get_field(obj, jni_str!("width"), jni_sig!("I"))?.i()?;
    let height = env.get_field(obj, jni_str!("height"), jni_sig!("I"))?.i()?;

    Ok(Rect::new(Point2D::new(x, y), Size2D::new(width, height)))
}

fn get_field_as_string<'local>(
    env: &mut Env<'local>,
    obj: &JObject<'local>,
    field: &JNIStr,
) -> Result<String, Error> {
    let string_value = env
        .get_field(obj, field, jni_sig!("Ljava/lang/String;"))?
        .l()?;
    JString::cast_local(env, string_value)?.try_to_string(env)
}

fn get_options<'local>(
    env: &mut Env<'local>,
    opts: &JObject<'local>,
    surface: &JObject<'local>,
) -> Result<(InitOptions, bool, Option<String>, Option<String>), Error> {
    let args = get_field_as_string(env, opts, jni_str!("args")).ok();
    let url = get_field_as_string(env, opts, jni_str!("url")).ok();
    let log_str = get_field_as_string(env, opts, jni_str!("logStr")).ok();
    let gst_debug_str = get_field_as_string(env, opts, jni_str!("gstDebugStr")).ok();

    let experimental_mode = env
        .get_field(opts, jni_str!("experimentalMode"), jni_sig!("Z"))?
        .z()?;

    let density = env
        .get_field(opts, jni_str!("density"), jni_sig!("F"))?
        .f()?;

    let log = env
        .get_field(opts, jni_str!("enableLogs"), jni_sig!("Z"))?
        .z()?;

    let coordinates = env
        .get_field(
            opts,
            jni_str!("coordinates"),
            jni_sig!("Lorg/servo/servoview/JNIServo$ServoCoordinates;"),
        )?
        .l()?;

    let viewport_rect = jni_coordinate_to_rust_viewport_rect(env, &coordinates)?;

    let mut args: Vec<String> = args
        .and_then(|args| {
            serde_json::from_str(&args)
                .inspect_err(|_| {
                    error!("Invalid arguments. Servo arguments must be formatted as a JSON array")
                })
                .ok()
        })
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
    env: &mut Env<'_>,
    surface: &JObject<'_>,
) -> (RawDisplayHandle, RawWindowHandle) {
    let native_window = unsafe { ANativeWindow_fromSurface(env.get_raw(), surface.as_raw()) };
    let native_window = NonNull::new(native_window).expect("Could not get Android window");
    (
        RawDisplayHandle::Android(AndroidDisplayHandle::new()),
        RawWindowHandle::AndroidNdk(AndroidNdkWindowHandle::new(native_window)),
    )
}
