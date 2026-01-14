/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, Ref, RefCell, RefMut};
use std::cmp::max;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::time::Duration;

use background_hang_monitor::HangMonitorRegister;
use base::generic_channel::{GenericCallback, GenericSender, RoutedReceiver};
pub use base::id::WebViewId;
use base::id::{PipelineNamespace, PipelineNamespaceId};
#[cfg(feature = "bluetooth")]
use bluetooth::BluetoothThreadFactory;
#[cfg(feature = "bluetooth")]
use bluetooth_traits::BluetoothRequest;
use compositing::{InitialPaintState, Paint};
pub use compositing_traits::rendering_context::RenderingContext;
use compositing_traits::{CrossProcessPaintApi, PaintMessage, PaintProxy};
#[cfg(all(
    not(target_os = "windows"),
    not(target_os = "ios"),
    not(target_os = "android"),
    not(target_arch = "arm"),
    not(target_arch = "aarch64"),
    not(target_env = "ohos"),
))]
use constellation::content_process_sandbox_profile;
use constellation::{
    Constellation, FromEmbedderLogger, FromScriptLogger, InitialConstellationState,
    NewScriptEventLoopProcessInfo, UnprivilegedContent,
};
use constellation_traits::{EmbedderToConstellationMessage, ScriptToConstellationSender};
use crossbeam_channel::{Receiver, Sender, unbounded};
pub use embedder_traits::*;
use env_logger::Builder as EnvLoggerBuilder;
use fonts::SystemFontService;
#[cfg(all(
    not(target_os = "windows"),
    not(target_os = "ios"),
    not(target_os = "android"),
    not(target_arch = "arm"),
    not(target_arch = "aarch64"),
    not(target_env = "ohos"),
))]
use gaol::sandbox::{ChildSandbox, ChildSandboxMethods};
use ipc_channel::ipc::{self, IpcSender};
use layout::LayoutFactoryImpl;
use layout_api::ScriptThreadFactory;
use log::{Log, Metadata, Record, debug, warn};
use media::{GlApi, NativeDisplay, WindowGLContext};
use net::image_cache::ImageCacheFactoryImpl;
use net::protocols::ProtocolRegistry;
use net::resource_thread::new_resource_threads;
use net_traits::{ResourceThreads, exit_fetch_thread, start_fetch_thread};
use profile::{mem as profile_mem, system_reporter, time as profile_time};
use profile_traits::mem::{MemoryReportResult, ProfilerMsg, Reporter};
use profile_traits::{mem, time};
use rustc_hash::FxHashMap;
use script::{JSEngineSetup, ServiceWorkerManager};
use servo_config::opts::Opts;
use servo_config::prefs::{PrefValue, Preferences};
use servo_config::{opts, pref, prefs};
use servo_geometry::{
    DeviceIndependentIntRect, convert_rect_to_css_pixel, convert_size_to_css_pixel,
};
use servo_media::ServoMedia;
use servo_media::player::context::GlContext;
use storage::new_storage_threads;
use storage_traits::StorageThreads;
use style::global_style_data::StyleThreadPool;

use crate::clipboard_delegate::StringRequest;
use crate::javascript_evaluator::JavaScriptEvaluator;
use crate::network_manager::NetworkManager;
use crate::proxies::ConstellationProxy;
use crate::responders::ServoErrorChannel;
use crate::servo_delegate::{DefaultServoDelegate, ServoDelegate, ServoError};
use crate::site_data_manager::SiteDataManager;
use crate::webview::{MINIMUM_WEBVIEW_SIZE, WebView, WebViewInner};
use crate::webview_delegate::{
    AllowOrDenyRequest, AuthenticationRequest, EmbedderControl, FilePicker, NavigationRequest,
    PermissionRequest, ProtocolHandlerRegistration, WebResourceLoad,
};

#[cfg(feature = "media-gstreamer")]
mod media_platform {
    #[cfg(any(windows, target_os = "macos"))]
    mod gstreamer_plugins {
        include!(concat!(env!("OUT_DIR"), "/gstreamer_plugins.rs"));
    }

    use servo_media_gstreamer::GStreamerBackend;

    use super::ServoMedia;

    #[cfg(any(windows, target_os = "macos"))]
    pub fn init() {
        ServoMedia::init_with_backend(|| {
            let mut plugin_dir = std::env::current_exe().unwrap();
            plugin_dir.pop();

            if cfg!(target_os = "macos") {
                plugin_dir.push("lib");
            }

            match GStreamerBackend::init_with_plugins(
                plugin_dir,
                gstreamer_plugins::GSTREAMER_PLUGINS,
            ) {
                Ok(b) => b,
                Err(e) => {
                    log::error!("Error initializing GStreamer: {:?}", e);
                    std::process::exit(1);
                },
            }
        });
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    pub fn init() {
        ServoMedia::init::<GStreamerBackend>();
    }
}

#[cfg(not(feature = "media-gstreamer"))]
mod media_platform {
    use super::ServoMedia;
    pub fn init() {
        ServoMedia::init::<servo_media_dummy::DummyBackend>();
    }
}

struct ServoInner {
    delegate: RefCell<Rc<dyn ServoDelegate>>,
    paint: Rc<RefCell<Paint>>,
    constellation_proxy: ConstellationProxy,
    embedder_receiver: Receiver<EmbedderMsg>,
    network_manager: Rc<RefCell<NetworkManager>>,
    site_data_manager: Rc<RefCell<SiteDataManager>>,
    /// A struct that tracks ongoing JavaScript evaluations and is responsible for
    /// calling the callback when the evaluation is complete.
    javascript_evaluator: Rc<RefCell<JavaScriptEvaluator>>,
    /// Tracks whether we are in the process of shutting down, or have shut down.
    /// This is shared with `WebView`s and the `ServoRenderer`.
    shutdown_state: Rc<Cell<ShutdownState>>,
    /// A map  [`WebView`]s that are managed by this [`Servo`] instance. These are stored
    /// as `Weak` references so that the embedding application can control their lifetime.
    /// When accessed, `Servo` will be reponsible for cleaning up the invalid `Weak`
    /// references.
    webviews: RefCell<FxHashMap<WebViewId, Weak<RefCell<WebViewInner>>>>,
    servo_errors: ServoErrorChannel,
    /// For single-process Servo instances, this field controls the initialization
    /// and deinitialization of the JS Engine. Multiprocess Servo instances have their
    /// own instance that exists in the content process instead.
    _js_engine_setup: Option<JSEngineSetup>,
}

impl ServoInner {
    fn get_webview_handle(&self, id: WebViewId) -> Option<WebView> {
        self.webviews
            .borrow()
            .get(&id)
            .and_then(WebView::from_weak_handle)
    }

    fn spin_event_loop(&self) -> bool {
        if self.shutdown_state.get() == ShutdownState::FinishedShuttingDown {
            return false;
        }

        {
            let paint = self.paint.borrow();
            let mut messages = Vec::new();
            while let Ok(message) = paint.receiver().try_recv() {
                match message {
                    Ok(message) => messages.push(message),
                    Err(error) => {
                        warn!("Router deserialization error: {error}. Ignoring this PaintMessage.")
                    },
                }
            }
            paint.handle_messages(messages);
        }

        // Only handle incoming embedder messages if `Paint` hasn't already started shutting down.
        while let Ok(message) = self.embedder_receiver.try_recv() {
            self.handle_embedder_message(message);

            if self.shutdown_state.get() == ShutdownState::FinishedShuttingDown {
                break;
            }
        }

        if self.constellation_proxy.disconnected() {
            self.delegate
                .borrow()
                .notify_error(ServoError::LostConnectionWithBackend);
        }

        self.paint.borrow_mut().perform_updates();
        self.send_new_frame_ready_messages();
        self.handle_delegate_errors();
        self.clean_up_destroyed_webview_handles();

        if self.shutdown_state.get() == ShutdownState::FinishedShuttingDown {
            return false;
        }

        true
    }

    fn send_new_frame_ready_messages(&self) {
        let webviews_needing_repaint = self.paint.borrow().webviews_needing_repaint();

        for webview in webviews_needing_repaint
            .iter()
            .filter_map(|webview_id| self.get_webview_handle(*webview_id))
        {
            webview.delegate().notify_new_frame_ready(webview);
        }
    }

    fn handle_delegate_errors(&self) {
        while let Some(error) = self.servo_errors.try_recv() {
            self.delegate.borrow().notify_error(error);
        }
    }

    fn clean_up_destroyed_webview_handles(&self) {
        // Remove any webview handles that have been destroyed and would not be upgradable.
        // Note that `retain` is O(capacity) because it visits empty buckets, so it may be worth
        // calling `shrink_to_fit` at some point to deal with cases where a long-running Servo
        // instance goes from many open webviews to only a few.
        self.webviews
            .borrow_mut()
            .retain(|_webview_id, webview| webview.strong_count() > 0);
    }

    fn finish_shutting_down(&self) {
        debug!("Servo received message that Constellation shutdown is complete");
        self.shutdown_state.set(ShutdownState::FinishedShuttingDown);
        self.paint.borrow_mut().finish_shutting_down();
    }

    fn handle_embedder_message(&self, message: EmbedderMsg) {
        match message {
            EmbedderMsg::ShutdownComplete => self.finish_shutting_down(),
            EmbedderMsg::Status(webview_id, status_text) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.set_status_text(status_text);
                }
            },
            EmbedderMsg::ChangePageTitle(webview_id, title) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.set_page_title(title);
                }
            },
            EmbedderMsg::MoveTo(webview_id, position) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.delegate().request_move_to(webview, position);
                }
            },
            EmbedderMsg::ResizeTo(webview_id, size) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview
                        .delegate()
                        .request_resize_to(webview, size.max(MINIMUM_WEBVIEW_SIZE));
                }
            },
            EmbedderMsg::ShowSimpleDialog(webview_id, simple_dialog) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.delegate().show_embedder_control(
                        webview,
                        EmbedderControl::SimpleDialog(simple_dialog.into()),
                    );
                }
            },
            EmbedderMsg::AllowNavigationRequest(webview_id, pipeline_id, servo_url) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    let request = NavigationRequest {
                        url: servo_url.into_url(),
                        pipeline_id,
                        constellation_proxy: self.constellation_proxy.clone(),
                        response_sent: false,
                    };
                    webview.delegate().request_navigation(webview, request);
                }
            },
            EmbedderMsg::AllowProtocolHandlerRequest(
                webview_id,
                registration_update,
                response_sender,
            ) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    let ProtocolHandlerUpdateRegistration {
                        scheme,
                        url,
                        register_or_unregister,
                    } = registration_update;
                    let protocol_handler_registration = ProtocolHandlerRegistration {
                        scheme,
                        url: url.into_url(),
                        register_or_unregister,
                    };
                    let allow_deny_request = AllowOrDenyRequest::new(
                        response_sender,
                        AllowOrDeny::Deny,
                        self.servo_errors.sender(),
                    );
                    webview.delegate().request_protocol_handler(
                        webview,
                        protocol_handler_registration,
                        allow_deny_request,
                    );
                }
            },
            EmbedderMsg::AllowOpeningWebView(webview_id, response_sender) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.request_create_new(response_sender);
                }
            },
            EmbedderMsg::WebViewClosed(webview_id) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.delegate().notify_closed(webview);
                }
            },
            EmbedderMsg::WebViewFocused(webview_id, focus_result) => {
                if focus_result {
                    for id in self.webviews.borrow().keys() {
                        if let Some(webview) = self.get_webview_handle(*id) {
                            let focused = webview.id() == webview_id;
                            webview.set_focused(focused);
                        }
                    }
                }
            },
            EmbedderMsg::WebViewBlurred => {
                for id in self.webviews.borrow().keys() {
                    if let Some(webview) = self.get_webview_handle(*id) {
                        webview.set_focused(false);
                    }
                }
            },
            EmbedderMsg::AllowUnload(webview_id, response_sender) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    let request = AllowOrDenyRequest::new(
                        response_sender,
                        AllowOrDeny::Allow,
                        self.servo_errors.sender(),
                    );
                    webview.delegate().request_unload(webview, request);
                }
            },
            EmbedderMsg::FinishJavaScriptEvaluation(evaluation_id, result) => {
                self.javascript_evaluator
                    .borrow_mut()
                    .finish_evaluation(evaluation_id, result);
            },
            EmbedderMsg::InputEventHandled(webview_id, input_event_id, result) => {
                self.paint.borrow_mut().notify_input_event_handled(
                    webview_id,
                    input_event_id,
                    result,
                );

                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview
                        .delegate()
                        .notify_input_event_handled(webview, input_event_id, result);
                }
            },
            EmbedderMsg::ClearClipboard(webview_id) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.clipboard_delegate().clear(webview);
                }
            },
            EmbedderMsg::GetClipboardText(webview_id, result_sender) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview
                        .clipboard_delegate()
                        .get_text(webview, StringRequest::from(result_sender));
                }
            },
            EmbedderMsg::SetClipboardText(webview_id, string) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.clipboard_delegate().set_text(webview, string);
                }
            },
            EmbedderMsg::SetCursor(webview_id, cursor) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.set_cursor(cursor);
                }
            },
            EmbedderMsg::NewFavicon(webview_id, image) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.set_favicon(image);
                }
            },
            EmbedderMsg::NotifyLoadStatusChanged(webview_id, load_status) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.set_load_status(load_status);
                }
            },
            EmbedderMsg::HistoryTraversalComplete(webview_id, traversal_id) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview
                        .delegate()
                        .notify_traversal_complete(webview.clone(), traversal_id);
                }
            },
            EmbedderMsg::HistoryChanged(webview_id, new_back_forward_list, current_list_index) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.set_history(new_back_forward_list, current_list_index);
                }
            },
            EmbedderMsg::NotifyFullscreenStateChanged(webview_id, fullscreen) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview
                        .delegate()
                        .notify_fullscreen_state_changed(webview, fullscreen);
                }
            },
            EmbedderMsg::WebResourceRequested(
                webview_id,
                web_resource_request,
                response_sender,
            ) => {
                if let Some(webview) =
                    webview_id.and_then(|webview_id| self.get_webview_handle(webview_id))
                {
                    let web_resource_load = WebResourceLoad::new(
                        web_resource_request,
                        response_sender,
                        self.servo_errors.sender(),
                    );
                    webview
                        .delegate()
                        .load_web_resource(webview, web_resource_load);
                } else {
                    let web_resource_load = WebResourceLoad::new(
                        web_resource_request,
                        response_sender,
                        self.servo_errors.sender(),
                    );
                    self.delegate.borrow().load_web_resource(web_resource_load);
                }
            },
            EmbedderMsg::Panic(webview_id, reason, backtrace) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview
                        .delegate()
                        .notify_crashed(webview, reason, backtrace);
                }
            },
            EmbedderMsg::GetSelectedBluetoothDevice(webview_id, items, response_sender) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.delegate().show_bluetooth_device_dialog(
                        webview,
                        items,
                        response_sender,
                    );
                }
            },
            EmbedderMsg::SelectFiles(control_id, file_picker_request, response_sender) => {
                if file_picker_request.accept_current_paths_for_testing {
                    let _ = response_sender.send(Some(file_picker_request.current_paths));
                    return;
                }
                if let Some(webview) = self.get_webview_handle(control_id.webview_id) {
                    webview.delegate().show_embedder_control(
                        webview,
                        EmbedderControl::FilePicker(FilePicker {
                            id: control_id,
                            file_picker_request,
                            response_sender,
                            response_sent: false,
                        }),
                    );
                }
            },
            EmbedderMsg::RequestAuthentication(webview_id, url, for_proxy, response_sender) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    let authentication_request = AuthenticationRequest::new(
                        url.into_url(),
                        for_proxy,
                        response_sender,
                        self.servo_errors.sender(),
                    );
                    webview
                        .delegate()
                        .request_authentication(webview, authentication_request);
                }
            },
            EmbedderMsg::PromptPermission(webview_id, requested_feature, response_sender) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    let permission_request = PermissionRequest {
                        requested_feature,
                        allow_deny_request: AllowOrDenyRequest::new(
                            response_sender,
                            AllowOrDeny::Deny,
                            self.servo_errors.sender(),
                        ),
                    };
                    webview
                        .delegate()
                        .request_permission(webview, permission_request);
                }
            },
            EmbedderMsg::ReportProfile(_items) => {},
            EmbedderMsg::MediaSessionEvent(webview_id, media_session_event) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview
                        .delegate()
                        .notify_media_session_event(webview, media_session_event);
                }
            },
            EmbedderMsg::OnDevtoolsStarted(port, token) => match port {
                Ok(port) => self
                    .delegate
                    .borrow()
                    .notify_devtools_server_started(port, token),
                Err(()) => self
                    .delegate
                    .borrow()
                    .notify_error(ServoError::DevtoolsFailedToStart),
            },
            EmbedderMsg::RequestDevtoolsConnection(response_sender) => {
                self.delegate
                    .borrow()
                    .request_devtools_connection(AllowOrDenyRequest::new(
                        response_sender,
                        AllowOrDeny::Deny,
                        self.servo_errors.sender(),
                    ));
            },
            #[cfg(feature = "gamepad")]
            EmbedderMsg::PlayGamepadHapticEffect(
                webview_id,
                gamepad_index,
                gamepad_haptic_effect_type,
                callback,
            ) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.delegate().play_gamepad_haptic_effect(
                        webview,
                        gamepad_index,
                        gamepad_haptic_effect_type,
                        Box::new(move |success| {
                            callback
                                .send(success)
                                .expect("Could not send message via callback")
                        }),
                    );
                }
            },
            #[cfg(feature = "gamepad")]
            EmbedderMsg::StopGamepadHapticEffect(webview_id, gamepad_index, callback) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.delegate().stop_gamepad_haptic_effect(
                        webview,
                        gamepad_index,
                        Box::new(move |success| {
                            callback
                                .send(success)
                                .expect("Could not send message via callback")
                        }),
                    );
                }
            },
            EmbedderMsg::ShowNotification(webview_id, notification) => {
                match webview_id.and_then(|webview_id| self.get_webview_handle(webview_id)) {
                    Some(webview) => webview.delegate().show_notification(webview, notification),
                    None => self.delegate.borrow().show_notification(notification),
                }
            },
            EmbedderMsg::ShowConsoleApiMessage(webview_id, level, message) => {
                match webview_id.and_then(|webview_id| self.get_webview_handle(webview_id)) {
                    Some(webview) => webview
                        .delegate()
                        .show_console_message(webview, level, message),
                    None => self.delegate.borrow().show_console_message(level, message),
                }
            },
            EmbedderMsg::ShowEmbedderControl(control_id, position, embedder_control_request) => {
                if let Some(webview) = self.get_webview_handle(control_id.webview_id) {
                    webview.show_embedder_control(control_id, position, embedder_control_request);
                }
            },
            EmbedderMsg::HideEmbedderControl(control_id) => {
                if let Some(webview) = self.get_webview_handle(control_id.webview_id) {
                    webview
                        .delegate()
                        .hide_embedder_control(webview, control_id);
                }
            },
            EmbedderMsg::GetWindowRect(webview_id, response_sender) => {
                let window_rect = || {
                    let Some(webview) = self.get_webview_handle(webview_id) else {
                        return DeviceIndependentIntRect::default();
                    };
                    let hidpi_scale_factor = webview.hidpi_scale_factor();
                    let Some(screen_geometry) = webview.delegate().screen_geometry(webview) else {
                        return DeviceIndependentIntRect::default();
                    };

                    convert_rect_to_css_pixel(screen_geometry.window_rect, hidpi_scale_factor)
                };

                if let Err(error) = response_sender.send(window_rect()) {
                    warn!("Failed to respond to GetWindowRect: {error}");
                }
            },
            EmbedderMsg::GetScreenMetrics(webview_id, response_sender) => {
                let screen_metrics = || {
                    let Some(webview) = self.get_webview_handle(webview_id) else {
                        return ScreenMetrics::default();
                    };
                    let hidpi_scale_factor = webview.hidpi_scale_factor();
                    let Some(screen_geometry) = webview.delegate().screen_geometry(webview) else {
                        return ScreenMetrics::default();
                    };

                    ScreenMetrics {
                        screen_size: convert_size_to_css_pixel(
                            screen_geometry.size,
                            hidpi_scale_factor,
                        ),
                        available_size: convert_size_to_css_pixel(
                            screen_geometry.available_size,
                            hidpi_scale_factor,
                        ),
                    }
                };
                if let Err(error) = response_sender.send(screen_metrics()) {
                    warn!("Failed to respond to GetScreenMetrics: {error}");
                }
            },
            EmbedderMsg::HackyAccessibilityTreeUpdate(webview_id, tree_update) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview
                        .delegate()
                        .hacky_accessibility_tree_update(webview, tree_update);
                }
            },
        }
    }
}

impl Drop for ServoInner {
    fn drop(&mut self) {
        self.constellation_proxy
            .send(EmbedderToConstellationMessage::Exit);
        self.shutdown_state.set(ShutdownState::ShuttingDown);
        while self.spin_event_loop() {
            std::thread::sleep(Duration::from_micros(500));
        }
    }
}

/// An in-process handle to a `Servo` instance. Cloning the handle does not create a new instance
/// of `Servo`.
///
/// A `Servo` instance does everything necessary to render the web, primarily orchestrating the
/// interaction between JavaScript, CSS layout, rendering, and the client window.
///
/// Clients create an event loop to pump messages between the embedding
/// application and various browser components.
#[derive(Clone)]
pub struct Servo(Rc<ServoInner>);

impl Servo {
    #[servo_tracing::instrument(skip(builder))]
    fn new(builder: ServoBuilder) -> Self {
        // Global configuration options, parsed from the command line.
        let opts = builder.opts.map(|opts| *opts);
        opts::initialize_options(opts.unwrap_or_default());
        let opts = opts::get();

        // Set the preferences globally.
        // TODO: It would be better to make these private to a particular Servo instance.
        let preferences = builder.preferences.map(|opts| *opts);
        servo_config::prefs::set(preferences.unwrap_or_default());

        use std::sync::atomic::Ordering;

        style::context::DEFAULT_DISABLE_STYLE_SHARING_CACHE.store(
            !pref!(layout_style_sharing_cache_enabled),
            Ordering::Relaxed,
        );
        style::context::DEFAULT_DUMP_STYLE_STATISTICS
            .store(opts.debug.style_statistics, Ordering::Relaxed);
        style::traversal::IS_SERVO_NONINCREMENTAL_LAYOUT
            .store(opts.nonincremental_layout, Ordering::Relaxed);

        if !opts.multiprocess {
            media_platform::init();
        }

        // Reserving a namespace to create WebViewId.
        PipelineNamespace::install(PipelineNamespaceId(0));

        // Get both endpoints of a special channel for communication between
        // the client window and `Paint`. This channel is unique because
        // messages to client may need to pump a platform-specific event loop
        // to deliver the message.
        let event_loop_waker = builder.event_loop_waker;
        let (paint_proxy, paint_receiver) = create_paint_channel(event_loop_waker.clone());
        let (constellation_proxy, embedder_to_constellation_receiver) = ConstellationProxy::new();
        let (embedder_proxy, embedder_receiver) = create_embedder_channel(event_loop_waker.clone());
        let time_profiler_chan = profile_time::Profiler::create(
            &opts.time_profiling,
            opts.time_profiler_trace_path.clone(),
        );
        let mem_profiler_chan = profile_mem::Profiler::create();

        let devtools_sender = if pref!(devtools_server_enabled) {
            Some(devtools::start_server(
                pref!(devtools_server_port) as u16,
                embedder_proxy.clone(),
            ))
        } else {
            None
        };

        // Important that this call is done in a single-threaded fashion, we
        // can't defer it after `create_constellation` has started.
        let js_engine_setup = if !opts.multiprocess {
            Some(script::init())
        } else {
            None
        };

        // Create the constellation, which maintains the engine pipelines, including script and
        // layout, as well as the navigation context.
        let mut protocols = ProtocolRegistry::with_internal_protocols();
        protocols.merge(builder.protocol_registry);

        // The `Paint` coordinates with the client window to create the final
        // rendered page and display it somewhere.
        let shutdown_state = Rc::new(Cell::new(ShutdownState::NotShuttingDown));
        let paint = Paint::new(InitialPaintState {
            paint_proxy: paint_proxy.clone(),
            receiver: paint_receiver,
            embedder_to_constellation_sender: constellation_proxy.sender().clone(),
            time_profiler_chan: time_profiler_chan.clone(),
            mem_profiler_chan: mem_profiler_chan.clone(),
            shutdown_state: shutdown_state.clone(),
            event_loop_waker,
            #[cfg(feature = "webxr")]
            webxr_registry: builder.webxr_registry,
        });

        let protocols = Arc::new(protocols);
        let (public_resource_threads, private_resource_threads, async_runtime) =
            new_resource_threads(
                devtools_sender.clone(),
                time_profiler_chan.clone(),
                mem_profiler_chan.clone(),
                embedder_proxy.clone(),
                opts.config_dir.clone(),
                opts.certificate_path.clone(),
                opts.ignore_certificate_errors,
                protocols.clone(),
            );

        let (private_storage_threads, public_storage_threads) =
            new_storage_threads(mem_profiler_chan.clone(), opts.config_dir.clone());

        create_constellation(
            embedder_to_constellation_receiver,
            &paint.borrow(),
            embedder_proxy,
            paint_proxy.clone(),
            time_profiler_chan,
            mem_profiler_chan,
            devtools_sender,
            protocols,
            public_resource_threads.clone(),
            private_resource_threads.clone(),
            async_runtime,
            public_storage_threads.clone(),
            private_storage_threads.clone(),
        );

        if opts::get().multiprocess {
            prefs::add_observer(Box::new(constellation_proxy.clone()));
        }

        Servo(Rc::new(ServoInner {
            delegate: RefCell::new(Rc::new(DefaultServoDelegate)),
            paint,
            network_manager: Rc::new(RefCell::new(NetworkManager::new(
                public_resource_threads.clone(),
                private_resource_threads.clone(),
            ))),
            site_data_manager: Rc::new(RefCell::new(SiteDataManager::new(
                public_resource_threads,
                private_resource_threads,
                public_storage_threads,
                private_storage_threads,
            ))),
            javascript_evaluator: Rc::new(RefCell::new(JavaScriptEvaluator::new(
                constellation_proxy.clone(),
            ))),
            constellation_proxy,
            embedder_receiver,
            shutdown_state,
            webviews: Default::default(),
            servo_errors: ServoErrorChannel::default(),
            _js_engine_setup: js_engine_setup,
        }))
    }

    pub fn delegate(&self) -> Rc<dyn ServoDelegate> {
        self.0.delegate.borrow().clone()
    }

    pub fn set_delegate(&self, delegate: Rc<dyn ServoDelegate>) {
        *self.0.delegate.borrow_mut() = delegate;
    }

    /// **EXPERIMENTAL:** Intialize GL accelerated media playback. This currently only works on a limited number
    /// of platforms. This should be run *before* calling [`Servo::new`] and creating the first [`WebView`].
    pub fn initialize_gl_accelerated_media(display: NativeDisplay, api: GlApi, context: GlContext) {
        WindowGLContext::initialize(display, api, context)
    }

    /// Spin the Servo event loop, which:
    ///
    ///   - Performs updates in `Paint`, such as queued pinch zoom events
    ///   - Runs delebgate methods on all `WebView`s and `Servo` itself
    ///   - Maybe update the rendered `Paint` output, but *without* swapping buffers.
    pub fn spin_event_loop(&self) {
        self.0.spin_event_loop();
    }

    pub fn setup_logging(&self) {
        let constellation_chan = self.0.constellation_proxy.sender();
        let env = env_logger::Env::default();
        let env_logger = EnvLoggerBuilder::from_env(env).build();
        let con_logger = FromEmbedderLogger::new(constellation_chan);

        let filter = max(env_logger.filter(), con_logger.filter());
        let logger = BothLogger(env_logger, con_logger);

        log::set_boxed_logger(Box::new(logger)).expect("Failed to set logger.");
        log::set_max_level(filter);
    }

    pub fn create_memory_report(&self, snd: GenericCallback<MemoryReportResult>) {
        self.0
            .constellation_proxy
            .send(EmbedderToConstellationMessage::CreateMemoryReport(snd));
    }

    pub fn constellation_sender(&self) -> Sender<EmbedderToConstellationMessage> {
        self.0.constellation_proxy.sender()
    }

    pub fn execute_webdriver_command(&self, command: WebDriverCommandMsg) {
        self.0
            .constellation_proxy
            .send(EmbedderToConstellationMessage::WebDriverCommand(command));
    }

    pub fn set_preference(&self, name: &str, value: PrefValue) {
        let mut preferences = prefs::get().clone();
        preferences.set_value(name, value);
        prefs::set(preferences);
    }

    pub fn network_manager<'a>(&'a self) -> Ref<'a, NetworkManager> {
        self.0.network_manager.borrow()
    }

    pub fn site_data_manager<'a>(&'a self) -> Ref<'a, SiteDataManager> {
        self.0.site_data_manager.borrow()
    }

    pub(crate) fn paint<'a>(&'a self) -> Ref<'a, Paint> {
        self.0.paint.borrow()
    }

    pub(crate) fn paint_mut<'a>(&'a self) -> RefMut<'a, Paint> {
        self.0.paint.borrow_mut()
    }

    pub(crate) fn webviews_mut<'a>(
        &'a self,
    ) -> RefMut<'a, FxHashMap<WebViewId, Weak<RefCell<WebViewInner>>>> {
        self.0.webviews.borrow_mut()
    }

    pub(crate) fn constellation_proxy(&self) -> &ConstellationProxy {
        &self.0.constellation_proxy
    }

    pub(crate) fn javascript_evaluator_mut<'a>(&'a self) -> RefMut<'a, JavaScriptEvaluator> {
        self.0.javascript_evaluator.borrow_mut()
    }
}

fn create_embedder_channel(
    event_loop_waker: Box<dyn EventLoopWaker>,
) -> (EmbedderProxy, Receiver<EmbedderMsg>) {
    let (sender, receiver) = unbounded();
    (
        EmbedderProxy {
            sender,
            event_loop_waker,
        },
        receiver,
    )
}

fn create_paint_channel(
    event_loop_waker: Box<dyn EventLoopWaker>,
) -> (PaintProxy, RoutedReceiver<PaintMessage>) {
    let (sender, receiver) = unbounded();
    let sender_clone = sender.clone();
    let event_loop_waker_clone = event_loop_waker.clone();
    // This callback is equivalent to `PaintProxy::send`
    let result_callback = move |msg: Result<PaintMessage, ipc_channel::Error>| {
        if let Err(err) = sender_clone.send(msg) {
            warn!("Failed to send response ({:?}).", err);
        }
        event_loop_waker_clone.wake();
    };

    let generic_callback =
        GenericCallback::new(result_callback).expect("Failed to create callback");
    let cross_process_paint_api = CrossProcessPaintApi::new(generic_callback);
    let paint_proxy = PaintProxy {
        sender,
        cross_process_paint_api,
        event_loop_waker,
    };

    (paint_proxy, receiver)
}

#[expect(clippy::too_many_arguments)]
fn create_constellation(
    embedder_to_constellation_receiver: Receiver<EmbedderToConstellationMessage>,
    paint: &Paint,
    embedder_proxy: EmbedderProxy,
    paint_proxy: PaintProxy,
    time_profiler_chan: time::ProfilerChan,
    mem_profiler_chan: mem::ProfilerChan,
    devtools_sender: Option<Sender<devtools_traits::DevtoolsControlMsg>>,
    protocols: Arc<ProtocolRegistry>,
    public_resource_threads: ResourceThreads,
    private_resource_threads: ResourceThreads,
    async_runtime: Box<dyn net_traits::AsyncRuntime>,
    public_storage_threads: StorageThreads,
    private_storage_threads: StorageThreads,
) {
    // Global configuration options, parsed from the command line.
    let opts = opts::get();

    #[cfg(feature = "bluetooth")]
    let bluetooth_thread: GenericSender<BluetoothRequest> =
        BluetoothThreadFactory::new(embedder_proxy.clone());

    let privileged_urls = protocols.privileged_urls();

    let system_font_service = Arc::new(
        SystemFontService::spawn(
            paint_proxy.cross_process_paint_api.clone(),
            mem_profiler_chan.clone(),
        )
        .to_proxy(),
    );

    let initial_state = InitialConstellationState {
        paint_proxy,
        embedder_proxy,
        devtools_sender,
        #[cfg(feature = "bluetooth")]
        bluetooth_thread,
        system_font_service,
        public_resource_threads,
        private_resource_threads,
        public_storage_threads,
        private_storage_threads,
        time_profiler_chan,
        mem_profiler_chan,
        #[cfg(feature = "webxr")]
        webxr_registry: Some(paint.webxr_main_thread_registry()),
        #[cfg(not(feature = "webxr"))]
        webxr_registry: None,
        webgl_threads: Some(paint.webgl_threads()),
        webrender_external_image_id_manager: paint.webrender_external_image_id_manager(),
        #[cfg(feature = "webgpu")]
        wgpu_image_map: paint.webgpu_image_map(),
        async_runtime,
        privileged_urls,
    };

    let layout_factory = Arc::new(LayoutFactoryImpl());

    Constellation::<script::ScriptThread, script::ServiceWorkerManager>::start(
        embedder_to_constellation_receiver,
        initial_state,
        layout_factory,
        opts.random_pipeline_closure_probability,
        opts.random_pipeline_closure_seed,
        opts.hard_fail,
    );
}

// A logger that logs to two downstream loggers.
// This should probably be in the log crate.
struct BothLogger<Log1, Log2>(Log1, Log2);

impl<Log1, Log2> Log for BothLogger<Log1, Log2>
where
    Log1: Log,
    Log2: Log,
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.0.enabled(metadata) || self.1.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        self.0.log(record);
        self.1.log(record);
    }

    fn flush(&self) {
        self.0.flush();
        self.1.flush();
    }
}

fn set_logger(script_to_constellation_sender: ScriptToConstellationSender) {
    let con_logger = FromScriptLogger::new(script_to_constellation_sender);
    let env = env_logger::Env::default();
    let env_logger = EnvLoggerBuilder::from_env(env).build();

    let filter = max(env_logger.filter(), con_logger.filter());
    let logger = BothLogger(env_logger, con_logger);

    log::set_boxed_logger(Box::new(logger)).expect("Failed to set logger.");
    log::set_max_level(filter);
}

/// Content process entry point.
pub fn run_content_process(token: String) {
    let (unprivileged_content_sender, unprivileged_content_receiver) =
        ipc::channel::<UnprivilegedContent>().unwrap();
    let connection_bootstrap: IpcSender<IpcSender<UnprivilegedContent>> =
        IpcSender::connect(token).unwrap();
    connection_bootstrap
        .send(unprivileged_content_sender)
        .unwrap();

    let unprivileged_content = unprivileged_content_receiver.recv().unwrap();
    opts::initialize_options(unprivileged_content.opts());
    prefs::set(unprivileged_content.prefs().clone());

    // Enter the sandbox if necessary.
    if opts::get().sandbox {
        create_sandbox();
    }

    let _js_engine_setup = script::init();

    match unprivileged_content {
        UnprivilegedContent::ScriptEventLoop(new_event_loop_info) => {
            media_platform::init();

            // Start the fetch thread for this content process.
            let fetch_thread_join_handle = start_fetch_thread();

            set_logger(
                new_event_loop_info
                    .initial_script_state
                    .script_to_constellation_sender
                    .clone(),
            );

            register_system_memory_reporter_for_event_loop(&new_event_loop_info);

            let (background_hang_monitor_register, background_hang_monitor_join_handle) =
                HangMonitorRegister::init(
                    new_event_loop_info.bhm_to_constellation_sender.clone(),
                    new_event_loop_info.constellation_to_bhm_receiver,
                    opts::get().background_hang_monitor,
                );

            let layout_factory = Arc::new(LayoutFactoryImpl());
            let script_join_handle = script::ScriptThread::create(
                new_event_loop_info.initial_script_state,
                layout_factory,
                Arc::new(ImageCacheFactoryImpl::new(
                    new_event_loop_info.broken_image_icon_data,
                )),
                background_hang_monitor_register,
            );

            script_join_handle
                .join()
                .expect("Failed to join on the script thread.");
            background_hang_monitor_join_handle
                .join()
                .expect("Failed to join on the BHM background thread.");

            StyleThreadPool::shutdown();

            // Shut down the fetch thread started above.
            exit_fetch_thread();
            fetch_thread_join_handle
                .join()
                .expect("Failed to join on the fetch thread in the constellation");
        },
        UnprivilegedContent::ServiceWorker(content) => {
            content.start::<ServiceWorkerManager>();
        },
    }
}

#[cfg(all(
    not(target_os = "windows"),
    not(target_os = "ios"),
    not(target_os = "android"),
    not(target_arch = "arm"),
    not(target_arch = "aarch64"),
    not(target_env = "ohos"),
))]
fn create_sandbox() {
    ChildSandbox::new(content_process_sandbox_profile())
        .activate()
        .expect("Failed to activate sandbox!");
}

#[cfg(any(
    target_os = "windows",
    target_os = "ios",
    target_os = "android",
    target_arch = "arm",
    target_arch = "aarch64",
    target_env = "ohos",
))]
fn create_sandbox() {
    panic!("Sandboxing is not supported on Windows, iOS, ARM targets and android.");
}

struct DefaultEventLoopWaker;

impl EventLoopWaker for DefaultEventLoopWaker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(DefaultEventLoopWaker)
    }
}

#[cfg(feature = "webxr")]
struct DefaultWebXrRegistry;
#[cfg(feature = "webxr")]
impl webxr::WebXrRegistry for DefaultWebXrRegistry {}

pub struct ServoBuilder {
    opts: Option<Box<Opts>>,
    preferences: Option<Box<Preferences>>,
    event_loop_waker: Box<dyn EventLoopWaker>,
    protocol_registry: ProtocolRegistry,
    #[cfg(feature = "webxr")]
    webxr_registry: Box<dyn webxr::WebXrRegistry>,
}

impl Default for ServoBuilder {
    fn default() -> Self {
        Self {
            opts: Default::default(),
            preferences: Default::default(),
            event_loop_waker: Box::new(DefaultEventLoopWaker),
            protocol_registry: Default::default(),
            #[cfg(feature = "webxr")]
            webxr_registry: Box::new(DefaultWebXrRegistry),
        }
    }
}

impl ServoBuilder {
    pub fn build(self) -> Servo {
        Servo::new(self)
    }

    pub fn opts(mut self, opts: Opts) -> Self {
        self.opts = Some(Box::new(opts));
        self
    }

    pub fn preferences(mut self, preferences: Preferences) -> Self {
        self.preferences = Some(Box::new(preferences));
        self
    }

    pub fn event_loop_waker(mut self, event_loop_waker: Box<dyn EventLoopWaker>) -> Self {
        self.event_loop_waker = event_loop_waker;
        self
    }

    pub fn protocol_registry(mut self, protocol_registry: ProtocolRegistry) -> Self {
        self.protocol_registry = protocol_registry;
        self
    }

    #[cfg(feature = "webxr")]
    pub fn webxr_registry(mut self, webxr_registry: Box<dyn webxr::WebXrRegistry>) -> Self {
        self.webxr_registry = webxr_registry;
        self
    }
}

fn register_system_memory_reporter_for_event_loop(
    new_event_loop_info: &NewScriptEventLoopProcessInfo,
) {
    // Register the system memory reporter, which will run on its own thread. It never needs to
    // be unregistered, because as long as the memory profiler is running the system memory
    // reporter can make measurements.
    let callback = GenericCallback::new(|message| {
        if let Ok(request) = message {
            system_reporter::collect_reports(request);
        }
    })
    .expect("Could not create memory reporter callback");
    new_event_loop_info
        .initial_script_state
        .memory_profiler_sender
        .send(ProfilerMsg::RegisterReporter(
            format!("system-content-{}", std::process::id()),
            Reporter(callback),
        ));
}
