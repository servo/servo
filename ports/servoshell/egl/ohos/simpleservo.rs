/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::RefCell;
use std::convert::TryInto;
use std::os::raw::c_void;
use std::path::PathBuf;
use std::rc::Rc;

use log::{debug, error, info};
use servo::base::id::WebViewId;
use servo::compositing::windowing::EmbedderEvent;
use servo::compositing::CompositeTarget;
use servo::embedder_traits::resources;
/// The EventLoopWaker::wake function will be called from any thread.
/// It will be called to notify embedder that some events are available,
/// and that perform_updates need to be called
pub use servo::embedder_traits::EventLoopWaker;
use servo::euclid::Size2D;
use servo::servo_url::ServoUrl;
use servo::webrender_traits::RenderingContext;
use servo::{self, Servo};
use surfman::{Connection, SurfaceType};
use xcomponent_sys::{OH_NativeXComponent, OH_NativeXComponent_GetXComponentSize};

use crate::egl::host_trait::HostTrait;
use crate::egl::ohos::resources::ResourceReaderInstance;
use crate::egl::ohos::InitOpts;
use crate::egl::servo_glue::{
    Coordinates, ServoEmbedderCallbacks, ServoGlue, ServoWindowCallbacks,
};
use crate::prefs::{parse_command_line_arguments, ArgumentParsingResult};

/// Initialize Servo. At that point, we need a valid GL context.
/// In the future, this will be done in multiple steps.
pub fn init(
    options: InitOpts,
    native_window: *mut c_void,
    xcomponent: *mut OH_NativeXComponent,
    waker: Box<dyn EventLoopWaker>,
    callbacks: Box<dyn HostTrait>,
) -> Result<ServoGlue, &'static str> {
    info!("Entered simpleservo init function");
    crate::init_tracing();
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

    // Initialize surfman
    let connection = Connection::new().or(Err("Failed to create connection"))?;
    let adapter = connection
        .create_adapter()
        .or(Err("Failed to create adapter"))?;

    let mut width: u64 = 0;
    let mut height: u64 = 0;
    let res = unsafe {
        OH_NativeXComponent_GetXComponentSize(
            xcomponent,
            native_window,
            &mut width as *mut _,
            &mut height as *mut _,
        )
    };
    assert_eq!(res, 0, "OH_NativeXComponent_GetXComponentSize failed");
    let width: i32 = width.try_into().expect("Width too large");
    let height: i32 = height.try_into().expect("Height too large");

    debug!("Creating surfman widget with width {width} and height {height}");
    let native_widget = unsafe {
        connection.create_native_widget_from_ptr(native_window, Size2D::new(width, height))
    };
    let surface_type = SurfaceType::Widget { native_widget };

    info!("Creating rendering context");
    let rendering_context = RenderingContext::create(&connection, &adapter, None)
        .or(Err("Failed to create surface manager"))?;
    let surface = rendering_context
        .create_surface(surface_type)
        .or(Err("Failed to create surface"))?;
    rendering_context
        .bind_surface(surface)
        .or(Err("Failed to bind surface"))?;

    info!("before ServoWindowCallbacks...");

    let window_callbacks = Rc::new(ServoWindowCallbacks::new(
        callbacks,
        RefCell::new(Coordinates::new(0, 0, width, height, width, height)),
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
        CompositeTarget::Window,
    );

    let mut servo_glue = ServoGlue::new(
        rendering_context,
        servo,
        window_callbacks,
        servoshell_preferences,
    );

    let initial_url = ServoUrl::parse(options.url.as_str())
        .inspect_err(|e| error!("Invalid initial Servo URL `{}`. Error: {e:?}", options.url))
        .ok()
        .unwrap_or_else(|| ServoUrl::parse("about:blank").expect("Infallible"));

    let _ = servo_glue.process_event(EmbedderEvent::NewWebView(initial_url, WebViewId::new()));

    Ok(servo_glue)
}
