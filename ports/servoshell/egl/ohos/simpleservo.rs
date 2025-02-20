/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::RefCell;
use std::os::raw::c_void;
use std::path::PathBuf;
use std::ptr::NonNull;
use std::rc::Rc;

use log::{debug, info};
use raw_window_handle::{
    DisplayHandle, OhosDisplayHandle, OhosNdkWindowHandle, RawDisplayHandle, RawWindowHandle,
    WindowHandle,
};
/// The EventLoopWaker::wake function will be called from any thread.
/// It will be called to notify embedder that some events are available,
/// and that perform_updates need to be called
pub use servo::EventLoopWaker;
use servo::{self, resources, Servo, WindowRenderingContext};
use xcomponent_sys::OH_NativeXComponent;

use crate::egl::app_state::{
    Coordinates, RunningAppState, ServoEmbedderCallbacks, ServoWindowCallbacks,
};
use crate::egl::host_trait::HostTrait;
use crate::egl::ohos::resources::ResourceReaderInstance;
use crate::egl::ohos::InitOpts;
use crate::prefs::{parse_command_line_arguments, ArgumentParsingResult};

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
    let resource_dir = PathBuf::from(&options.resource_dir).join("servo");
    resources::set(Box::new(ResourceReaderInstance::new(resource_dir)));

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

    let (opts, preferences, servoshell_preferences) = match parse_command_line_arguments(args) {
        ArgumentParsingResult::ContentProcess(..) => {
            unreachable!("OHOS does not have support for multiprocess yet.")
        },
        ArgumentParsingResult::ChromeProcess(opts, preferences, servoshell_preferences) => {
            (opts, preferences, servoshell_preferences)
        },
    };

    crate::init_tracing(servoshell_preferences.tracing_filter.as_deref());

    let Ok(window_size) = (unsafe { super::get_xcomponent_size(xcomponent, native_window) }) else {
        return Err("Failed to get xcomponent size");
    };
    let coordinates = Coordinates::new(
        0,
        0,
        window_size.width,
        window_size.height,
        window_size.width,
        window_size.height,
    );

    let display_handle = RawDisplayHandle::Ohos(OhosDisplayHandle::new());
    let display_handle = unsafe { DisplayHandle::borrow_raw(display_handle) };

    let native_window = NonNull::new(native_window).expect("Could not get native window");
    let window_handle = RawWindowHandle::OhosNdk(OhosNdkWindowHandle::new(native_window));
    let window_handle = unsafe { WindowHandle::borrow_raw(window_handle) };

    let rendering_context = Rc::new(
        WindowRenderingContext::new(
            display_handle,
            window_handle,
            &coordinates.framebuffer_size(),
        )
        .expect("Could not create RenderingContext"),
    );

    info!("before ServoWindowCallbacks...");

    let window_callbacks = Rc::new(ServoWindowCallbacks::new(
        callbacks,
        RefCell::new(coordinates),
        options.display_density as f32,
    ));

    let embedder_callbacks = Box::new(ServoEmbedderCallbacks::new(
        waker,
        #[cfg(feature = "webxr")]
        None,
    ));

    let servo = Servo::new(
        opts,
        preferences,
        rendering_context.clone(),
        embedder_callbacks,
        window_callbacks.clone(),
        None, /* user_agent */
    );

    let app_state = RunningAppState::new(
        Some(options.url),
        rendering_context,
        servo,
        window_callbacks,
        servoshell_preferences,
    );

    Ok(app_state)
}
