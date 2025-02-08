/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::mem;
use std::os::raw::c_void;
use std::rc::Rc;

use servo::compositing::CompositeTarget;
pub use servo::webrender_api::units::DeviceIntRect;
use servo::webrender_traits::SurfmanRenderingContext;
/// The EventLoopWaker::wake function will be called from any thread.
/// It will be called to notify embedder that some events are available,
/// and that perform_updates need to be called
pub use servo::EventLoopWaker;
use servo::{self, resources, Servo};
pub use servo::{InputMethodType, MediaSessionPlaybackState, PromptResult};
use surfman::{Connection, SurfaceType};

use crate::egl::android::resources::ResourceReaderInstance;
use crate::egl::app_state::{
    Coordinates, RunningAppState, ServoEmbedderCallbacks, ServoWindowCallbacks,
};
use crate::egl::host_trait::HostTrait;
use crate::prefs::{parse_command_line_arguments, ArgumentParsingResult};

thread_local! {
    pub static APP: RefCell<Option<Rc<RunningAppState>>> = const { RefCell::new(None) };
}

pub struct InitOptions {
    pub args: Vec<String>,
    pub url: Option<String>,
    pub coordinates: Coordinates,
    pub density: f32,
    #[cfg(feature = "webxr")]
    pub xr_discovery: Option<servo::webxr::Discovery>,
    pub surfman_integration: SurfmanIntegration,
}

/// Controls how this embedding's rendering will integrate with the embedder.
pub enum SurfmanIntegration {
    /// Render directly to a provided native widget (see surfman::NativeWidget).
    Widget(*mut c_void),
}

/// Initialize Servo. At that point, we need a valid GL context.
/// In the future, this will be done in multiple steps.
pub fn init(
    mut init_opts: InitOptions,
    waker: Box<dyn EventLoopWaker>,
    callbacks: Box<dyn HostTrait>,
) -> Result<(), &'static str> {
    crate::init_crypto();
    resources::set(Box::new(ResourceReaderInstance::new()));

    // `parse_command_line_arguments` expects the first argument to be the binary name.
    let mut args = mem::take(&mut init_opts.args);
    args.insert(0, "servo".to_string());

    let (opts, preferences, servoshell_preferences) = match parse_command_line_arguments(args) {
        ArgumentParsingResult::ContentProcess(..) => {
            unreachable!("Android does not have support for multiprocess yet.")
        },
        ArgumentParsingResult::ChromeProcess(opts, preferences, servoshell_preferences) => {
            (opts, preferences, servoshell_preferences)
        },
    };

    crate::init_tracing(servoshell_preferences.tracing_filter.as_deref());

    // Initialize surfman
    let connection = Connection::new().or(Err("Failed to create connection"))?;
    let adapter = connection
        .create_adapter()
        .or(Err("Failed to create adapter"))?;
    let surface_type = match init_opts.surfman_integration {
        SurfmanIntegration::Widget(native_widget) => {
            let native_widget = unsafe {
                connection.create_native_widget_from_ptr(
                    native_widget,
                    init_opts.coordinates.framebuffer.to_untyped(),
                )
            };
            SurfaceType::Widget { native_widget }
        },
    };
    let rendering_context = SurfmanRenderingContext::create(&connection, &adapter, None)
        .or(Err("Failed to create surface manager"))?;
    let surface = rendering_context
        .create_surface(surface_type)
        .or(Err("Failed to create surface"))?;
    rendering_context
        .bind_surface(surface)
        .or(Err("Failed to bind surface"))?;

    let window_callbacks = Rc::new(ServoWindowCallbacks::new(
        callbacks,
        RefCell::new(init_opts.coordinates),
        init_opts.density,
    ));

    let embedder_callbacks = Box::new(ServoEmbedderCallbacks::new(
        waker,
        #[cfg(feature = "webxr")]
        init_opts.xr_discovery,
    ));

    let servo = Servo::new(
        opts,
        preferences,
        Rc::new(rendering_context.clone()),
        embedder_callbacks,
        window_callbacks.clone(),
        None,
        CompositeTarget::ContextFbo,
    );

    APP.with(|app| {
        let app_state = RunningAppState::new(
            init_opts.url,
            rendering_context,
            servo,
            window_callbacks,
            servoshell_preferences,
        );
        *app.borrow_mut() = Some(app_state);
    });

    Ok(())
}

pub fn deinit() {
    APP.with(|app| {
        let app = app.replace(None).unwrap();
        if let Some(app_state) = Rc::into_inner(app) {
            app_state.deinit()
        }
    });
}
