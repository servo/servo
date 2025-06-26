/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::RefCell;
use std::fs;
use std::os::raw::c_void;
use std::path::PathBuf;
use std::ptr::NonNull;
use std::rc::Rc;

use dpi::PhysicalSize;
use log::{debug, info, warn};
use ohos_abilitykit_sys::runtime::application_context;
use ohos_window_manager_sys::display_manager;
use raw_window_handle::{
    DisplayHandle, OhosDisplayHandle, OhosNdkWindowHandle, RawDisplayHandle, RawWindowHandle,
    WindowHandle,
};
use servo::{self, EventLoopWaker, ServoBuilder, WindowRenderingContext, resources};
use xcomponent_sys::OH_NativeXComponent;

use crate::egl::app_state::{Coordinates, RunningAppState, ServoWindowCallbacks};
use crate::egl::host_trait::HostTrait;
use crate::egl::ohos::InitOpts;
use crate::egl::ohos::resources::ResourceReaderInstance;
use crate::prefs::{ArgumentParsingResult, parse_command_line_arguments};

pub(crate) fn get_raw_window_handle(
    xcomponent: *mut OH_NativeXComponent,
    window: *mut c_void,
) -> (RawWindowHandle, euclid::default::Size2D<i32>, Coordinates) {
    let window_size = unsafe { super::get_xcomponent_size(xcomponent, window) }
        .expect("Could not get native window size");
    let (x, y) = unsafe { super::get_xcomponent_offset(xcomponent, window) }
        .expect("Could not get native window offset");
    let coordinates = Coordinates::new(x, y, window_size.width, window_size.height);
    let native_window = NonNull::new(window).expect("Could not get native window");
    let window_handle = RawWindowHandle::OhosNdk(OhosNdkWindowHandle::new(native_window));
    (window_handle, window_size, coordinates)
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

/// Initialize Servo. At that point, we need a valid GL context.
/// In the future, this will be done in multiple steps.
pub fn init(
    options: InitOpts,
    native_window: *mut c_void,
    xcomponent: *mut OH_NativeXComponent,
    waker: Box<dyn EventLoopWaker>,
    callbacks: Box<dyn HostTrait>,
) -> Result<Rc<RunningAppState>, &'static str> {
    info!("Entered simpleservo init function");
    crate::init_crypto();

    let native_values = get_native_values();
    info!("Device Type {:?}", native_values.device_type);
    info!("OS Full Name {:?}", native_values.os_full_name);
    info!("ResourceDir {:?}", options.resource_dir);

    let resource_dir = PathBuf::from(&options.resource_dir).join("servo");
    debug!("Resources are located at: {:?}", resource_dir);
    resources::set(Box::new(ResourceReaderInstance::new(resource_dir.clone())));

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

    let (opts, preferences, servoshell_preferences) = match parse_command_line_arguments(args) {
        ArgumentParsingResult::ContentProcess(..) => {
            unreachable!("OHOS does not have support for multiprocess yet.")
        },
        ArgumentParsingResult::ChromeProcess(opts, preferences, servoshell_preferences) => {
            (opts, preferences, servoshell_preferences)
        },
    };

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

    let (window_handle, window_size, coordinates) =
        get_raw_window_handle(xcomponent, native_window);

    let display_handle = RawDisplayHandle::Ohos(OhosDisplayHandle::new());
    let display_handle = unsafe { DisplayHandle::borrow_raw(display_handle) };

    let window_handle = unsafe { WindowHandle::borrow_raw(window_handle) };

    let rendering_context = Rc::new(
        WindowRenderingContext::new(
            display_handle,
            window_handle,
            PhysicalSize::new(window_size.width as u32, window_size.height as u32),
        )
        .expect("Could not create RenderingContext"),
    );

    info!("before ServoWindowCallbacks...");

    let window_callbacks = Rc::new(ServoWindowCallbacks::new(
        callbacks,
        RefCell::new(coordinates),
    ));

    let servo = ServoBuilder::new(rendering_context.clone())
        .opts(opts)
        .preferences(preferences)
        .event_loop_waker(waker)
        .build();

    let app_state = RunningAppState::new(
        Some(options.url),
        native_values.display_density as f32,
        rendering_context,
        servo,
        window_callbacks,
        servoshell_preferences,
    );

    Ok(app_state)
}
