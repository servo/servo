/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::RefCell;
use std::convert::TryInto;
use std::os::raw::c_void;
use std::path::PathBuf;
use std::rc::Rc;

use log::{debug, error, info};
use ohos_sys::xcomponent::{OH_NativeXComponent, OH_NativeXComponent_GetXComponentSize};
use servo::compositing::windowing::EmbedderEvent;
use servo::compositing::CompositeTarget;
use servo::embedder_traits::resources;
/// The EventLoopWaker::wake function will be called from any thread.
/// It will be called to notify embedder that some events are available,
/// and that perform_updates need to be called
pub use servo::embedder_traits::EventLoopWaker;
use servo::euclid::Size2D;
use servo::servo_config::opts;
use servo::servo_config::opts::ArgumentParsingResult;
use servo::servo_url::ServoUrl;
use servo::webrender_traits::RenderingContext;
use servo::{self, gl, Servo};
use surfman::{Connection, SurfaceType};

use crate::egl::host_trait::HostTrait;
use crate::egl::ohos::resources::ResourceReaderInstance;
use crate::egl::ohos::InitOpts;
use crate::egl::servo_glue::{
    Coordinates, ServoEmbedderCallbacks, ServoGlue, ServoWindowCallbacks,
};

/// Initialize Servo. At that point, we need a valid GL context.
/// In the future, this will be done in multiple steps.
pub fn init(
    options: InitOpts,
    native_window: *mut c_void,
    xcomponent: *mut OH_NativeXComponent,
    gl: Rc<dyn gl::Gl>,
    waker: Box<dyn EventLoopWaker>,
    callbacks: Box<dyn HostTrait>,
) -> Result<ServoGlue, &'static str> {
    info!("Entered simpleservo init function");
    crate::init_tracing();
    let resource_dir = PathBuf::from(&options.resource_dir).join("servo");
    resources::set(Box::new(ResourceReaderInstance::new(resource_dir)));
    let mut args = vec!["servoshell".to_string()];
    // It would be nice if `from_cmdline_args()` could accept str slices, to avoid allocations here.
    // Then again, this code could and maybe even should be disabled in production builds.
    let split_args: Vec<String> = options
        .commandline_args
        .split("\u{1f}")
        .map(|arg| arg.to_string())
        .collect();
    args.extend(split_args);
    debug!("Servo commandline args: {:?}", args);

    let mut opts = getopts::Options::new();
    opts.optopt(
        "u",
        "user-agent",
        "Set custom user agent string (or ios / android / desktop for platform default)",
        "NCSA Mosaic/1.0 (X11;SunOS 4.1.4 sun4m)",
    );
    opts.optmulti(
        "",
        "pref",
        "A preference to set to enable",
        "dom.bluetooth.enabled",
    );
    opts.optmulti(
        "",
        "pref",
        "A preference to set to disable",
        "dom.webgpu.enabled=false",
    );
    opts.optmulti(
        "",
        "prefs-file",
        "Load in additional prefs from a file.",
        "--prefs-file /path/to/prefs.json",
    );

    let opts_matches;
    let content_process_token;
    match opts::from_cmdline_args(opts, &args) {
        ArgumentParsingResult::ContentProcess(matches, token) => {
            error!("Content Process mode not supported / tested yet on OpenHarmony!");
            opts_matches = matches;
            content_process_token = Some(token);
        },
        ArgumentParsingResult::ChromeProcess(matches) => {
            opts_matches = matches;
            content_process_token = None;
        },
    };

    crate::prefs::register_user_prefs(&opts_matches);

    gl.clear_color(1.0, 1.0, 1.0, 1.0);
    gl.clear(gl::COLOR_BUFFER_BIT);
    gl.finish();

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
    let rendering_context = RenderingContext::create(&connection, &adapter, surface_type)
        .or(Err("Failed to create surface manager"))?;

    info!("before ServoWindowCallbacks...");

    let window_callbacks = Rc::new(ServoWindowCallbacks::new(
        callbacks,
        RefCell::new(Coordinates::new(0, 0, width, height, width, height)),
        options.display_density as f32,
        rendering_context.clone(),
    ));

    let embedder_callbacks = Box::new(ServoEmbedderCallbacks::new(waker, None, gl.clone()));

    let servo = Servo::new(
        embedder_callbacks,
        window_callbacks.clone(),
        // User agent: Mozilla/5.0 (<Phone|PC|Tablet>; HarmonyOS 5.0) bla bla
        None,
        CompositeTarget::Window,
    );

    let mut servo_glue = ServoGlue::new(
        rendering_context,
        servo.servo,
        window_callbacks,
        Some(options.resource_dir),
    );

    let initial_url = ServoUrl::parse(options.url.as_str())
        .inspect_err(|e| error!("Invalid initial Servo URL `{}`. Error: {e:?}", options.url))
        .ok()
        .unwrap_or_else(|| ServoUrl::parse("about:blank").expect("Infallible"));

    let _ = servo_glue.process_event(EmbedderEvent::NewWebView(initial_url, servo.browser_id));

    Ok(servo_glue)
}
