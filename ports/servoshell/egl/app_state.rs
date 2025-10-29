/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

use crossbeam_channel::Receiver;
use dpi::PhysicalSize;
use euclid::{Point2D, Rect, Scale, Size2D, Vector2D};
use image::{DynamicImage, ImageFormat};
use keyboard_types::{CompositionEvent, CompositionState, Key, KeyState, NamedKey};
use log::{debug, error, info, warn};
use raw_window_handle::{RawWindowHandle, WindowHandle};
use servo::base::generic_channel::GenericSender;
use servo::base::id::WebViewId;
use servo::ipc_channel::ipc::IpcSender;
use servo::servo_geometry::DeviceIndependentPixel;
use servo::webrender_api::ScrollLocation;
use servo::webrender_api::units::{DeviceIntRect, DeviceIntSize, DevicePixel, DevicePoint};
use servo::{
    AllowOrDenyRequest, ContextMenuResult, EmbedderControl, EmbedderControlId, ImeEvent,
    InputEvent, KeyboardEvent, LoadStatus, MediaSessionActionType, MediaSessionEvent, MouseButton,
    MouseButtonAction, MouseButtonEvent, MouseMoveEvent, NavigationRequest, PermissionRequest,
    RefreshDriver, RenderingContext, ScreenGeometry, Servo, ServoDelegate, ServoError,
    SimpleDialog, TouchEvent, TouchEventType, TouchId, TraversalId, WebDriverCommandMsg,
    WebDriverJSResult, WebDriverLoadStatus, WebDriverScriptCommand, WebDriverSenders, WebView,
    WebViewBuilder, WebViewDelegate, WindowRenderingContext,
};
use url::Url;

use crate::egl::host_trait::HostTrait;
use crate::prefs::ServoShellPreferences;

#[derive(Clone, Debug)]
pub struct Coordinates {
    pub viewport: Rect<i32, DevicePixel>,
}

impl Coordinates {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Coordinates {
        Coordinates {
            viewport: Rect::new(Point2D::new(x, y), Size2D::new(width, height)),
        }
    }

    pub fn origin(&self) -> Point2D<i32, DevicePixel> {
        self.viewport.origin
    }

    pub fn size(&self) -> Size2D<i32, DevicePixel> {
        self.viewport.size
    }
}

pub(super) struct ServoWindowCallbacks {
    host_callbacks: Box<dyn HostTrait>,
    coordinates: RefCell<Coordinates>,
}

impl ServoWindowCallbacks {
    pub(super) fn new(
        host_callbacks: Box<dyn HostTrait>,
        coordinates: RefCell<Coordinates>,
    ) -> Self {
        Self {
            host_callbacks,
            coordinates,
        }
    }
}

pub struct RunningAppState {
    servo: Servo,
    rendering_context: Rc<WindowRenderingContext>,
    callbacks: Rc<ServoWindowCallbacks>,
    refresh_driver: Option<Rc<VsyncRefreshDriver>>,
    inner: RefCell<RunningAppStateInner>,
    /// servoshell specific preferences created during startup of the application.
    servoshell_preferences: ServoShellPreferences,
    /// A [`Receiver`] for receiving commands from a running WebDriver server, if WebDriver
    /// was enabled.
    webdriver_receiver: Option<Receiver<WebDriverCommandMsg>>,
    webdriver_senders: RefCell<WebDriverSenders>,
}

struct RunningAppStateInner {
    need_present: bool,
    /// List of top-level browsing contexts.
    /// Modified by EmbedderMsg::WebViewOpened and EmbedderMsg::WebViewClosed,
    /// and we exit if it ever becomes empty.
    webviews: HashMap<WebViewId, WebView>,

    /// The order in which the webviews were created.
    creation_order: Vec<WebViewId>,

    /// The webview that is currently focused.
    /// Modified by EmbedderMsg::WebViewFocused and EmbedderMsg::WebViewBlurred.
    focused_webview_id: Option<WebViewId>,

    context_menu_sender: Option<GenericSender<ContextMenuResult>>,

    /// Whether or not the animation state has changed. This is used to trigger
    /// host callbacks indicating that animation state has changed.
    animating_state_changed: Rc<Cell<bool>>,

    /// The HiDPI scaling factor to use for the display of [`WebView`]s.
    hidpi_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,

    /// Whether or not the application has achieved stable image output. This is used
    /// for the `exit_after_stable_image` option.
    achieved_stable_image: Rc<Cell<bool>>,

    /// A list of showing [`InputMethod`] interfaces.
    visible_input_methods: Vec<EmbedderControlId>,
}

struct ServoShellServoDelegate {
    animating_state_changed: Rc<Cell<bool>>,
}

impl ServoDelegate for ServoShellServoDelegate {
    fn notify_devtools_server_started(&self, _servo: &Servo, port: u16, _token: String) {
        info!("Devtools Server running on port {port}");
    }

    fn request_devtools_connection(&self, _servo: &Servo, request: AllowOrDenyRequest) {
        request.allow();
    }

    fn notify_error(&self, _servo: &Servo, error: ServoError) {
        error!("Saw Servo error: {error:?}!");
    }

    fn notify_animating_changed(&self, _animating: bool) {
        self.animating_state_changed.set(true);
    }
}

impl WebViewDelegate for RunningAppState {
    fn screen_geometry(&self, _webview: WebView) -> Option<ScreenGeometry> {
        let coord = self.callbacks.coordinates.borrow();
        let available_size = coord.size();
        let screen_size = coord.size();
        Some(ScreenGeometry {
            size: screen_size,
            available_size,
            window_rect: DeviceIntRect::from_origin_and_size(coord.origin(), coord.size()),
        })
    }

    fn notify_page_title_changed(&self, _webview: servo::WebView, title: Option<String>) {
        self.callbacks.host_callbacks.on_title_changed(title);
    }

    fn notify_history_changed(&self, _webview: WebView, entries: Vec<Url>, current: usize) {
        let can_go_back = current > 0;
        let can_go_forward = current < entries.len() - 1;
        self.callbacks
            .host_callbacks
            .on_history_changed(can_go_back, can_go_forward);
        self.callbacks
            .host_callbacks
            .on_url_changed(entries[current].clone().to_string());
    }

    fn notify_load_status_changed(&self, webview: WebView, load_status: LoadStatus) {
        self.callbacks
            .host_callbacks
            .notify_load_status_changed(load_status);

        if load_status == LoadStatus::Complete {
            if let Some(sender) = self
                .webdriver_senders
                .borrow_mut()
                .load_status_senders
                .remove(&webview.id())
            {
                let _ = sender.send(WebDriverLoadStatus::Complete);
            }
            self.maybe_request_screenshot(webview);
        }

        #[cfg(feature = "tracing")]
        if load_status == LoadStatus::Complete {
            #[cfg(feature = "tracing-hitrace")]
            let (snd, recv) = ipc_channel::ipc::channel().expect("Could not create channel");
            self.servo.create_memory_report(snd);
            std::thread::spawn(move || {
                let result = recv.recv().expect("Could not get memory report");
                let reports = result
                    .results
                    .first()
                    .expect("We should have some memory report");
                for report in &reports.reports {
                    let path = String::from("servo_memory_profiling:") + &report.path.join("/");
                    hitrace::trace_metric_str(&path, report.size as i64);
                }
            });
        }
    }

    fn notify_closed(&self, webview: WebView) {
        {
            let mut inner_mut = self.inner_mut();
            inner_mut.webviews.retain(|&id, _| id != webview.id());
            inner_mut.creation_order.retain(|&id| id != webview.id());
            inner_mut.focused_webview_id = None;
        }

        if let Some(newest_webview) = self.newest_webview() {
            newest_webview.focus();
        } else {
            self.servo.start_shutting_down();
        }
    }

    fn notify_focus_changed(&self, webview: WebView, focused: bool) {
        if focused {
            self.inner_mut().focused_webview_id = Some(webview.id());
            webview.show(true);
        } else if self.inner().focused_webview_id == Some(webview.id()) {
            self.inner_mut().focused_webview_id = None;
        }
    }

    fn notify_traversal_complete(&self, _webview: servo::WebView, traversal_id: TraversalId) {
        let mut webdriver_state = self.webdriver_senders.borrow_mut();
        if let std::collections::hash_map::Entry::Occupied(entry) =
            webdriver_state.pending_traversals.entry(traversal_id)
        {
            let sender = entry.remove();
            let _ = sender.send(WebDriverLoadStatus::Complete);
        }
    }

    fn notify_media_session_event(&self, _webview: WebView, event: MediaSessionEvent) {
        match event {
            MediaSessionEvent::SetMetadata(metadata) => self
                .callbacks
                .host_callbacks
                .on_media_session_metadata(metadata.title, metadata.artist, metadata.album),
            MediaSessionEvent::PlaybackStateChange(state) => self
                .callbacks
                .host_callbacks
                .on_media_session_playback_state_change(state),
            MediaSessionEvent::SetPositionState(position_state) => self
                .callbacks
                .host_callbacks
                .on_media_session_set_position_state(
                    position_state.duration,
                    position_state.position,
                    position_state.playback_rate,
                ),
        };
    }

    fn notify_crashed(&self, _webview: WebView, reason: String, backtrace: Option<String>) {
        self.callbacks.host_callbacks.on_panic(reason, backtrace);
    }

    fn notify_new_frame_ready(&self, _webview: WebView) {
        self.inner_mut().need_present = true;
    }

    fn request_navigation(&self, _webview: WebView, navigation_request: NavigationRequest) {
        if self
            .callbacks
            .host_callbacks
            .on_allow_navigation(navigation_request.url.to_string())
        {
            navigation_request.allow();
        } else {
            navigation_request.deny();
        }
    }

    fn request_open_auxiliary_webview(&self, parent_webview: WebView) -> Option<WebView> {
        let webview = WebViewBuilder::new_auxiliary(&self.servo)
            .delegate(parent_webview.delegate())
            .hidpi_scale_factor(self.inner().hidpi_scale_factor)
            .build();
        self.add(webview.clone());
        Some(webview)
    }

    fn request_permission(&self, webview: WebView, request: PermissionRequest) {
        self.callbacks
            .host_callbacks
            .request_permission(webview, request);
    }

    fn request_resize_to(&self, _webview: WebView, size: DeviceIntSize) {
        warn!("Received resize event (to {size:?}). Currently only the user can resize windows");
    }

    fn show_context_menu(
        &self,
        _webview: WebView,
        result_sender: GenericSender<ContextMenuResult>,
        title: Option<String>,
        items: Vec<String>,
    ) {
        if self.inner().context_menu_sender.is_some() {
            warn!("Trying to show a context menu when a context menu is already active");
            let _ = result_sender.send(ContextMenuResult::Ignored);
        } else {
            self.inner_mut().context_menu_sender = Some(result_sender);
            self.callbacks
                .host_callbacks
                .show_context_menu(title, items);
        }
    }

    fn show_embedder_control(&self, webview: WebView, embedder_control: EmbedderControl) {
        let control_id = embedder_control.id();
        match embedder_control {
            EmbedderControl::InputMethod(input_method_control) => {
                self.inner_mut().visible_input_methods.push(control_id);
                self.callbacks
                    .host_callbacks
                    .on_ime_show(input_method_control);
            },
            EmbedderControl::SimpleDialog(simple_dialog) => self
                .callbacks
                .host_callbacks
                .show_simple_dialog(webview, simple_dialog),
            _ => {},
        }
    }

    fn hide_embedder_control(&self, _webview: WebView, control_id: servo::EmbedderControlId) {
        let mut inner_mut = self.inner_mut();
        if let Some(index) = inner_mut
            .visible_input_methods
            .iter()
            .position(|visible_id| *visible_id == control_id)
        {
            inner_mut.visible_input_methods.remove(index);
            self.callbacks.host_callbacks.on_ime_hide();
        }
    }
}

#[derive(Default)]
pub(crate) struct VsyncRefreshDriver {
    start_frame_callback: RefCell<Option<Box<dyn Fn() + Send>>>,
}

impl VsyncRefreshDriver {
    fn notify_vsync(&self) {
        let Some(start_frame_callback) = self.start_frame_callback.borrow_mut().take() else {
            return;
        };
        start_frame_callback();
    }
}

impl RefreshDriver for VsyncRefreshDriver {
    fn observe_next_frame(&self, new_start_frame_callback: Box<dyn Fn() + Send + 'static>) {
        let mut start_frame_callback = self.start_frame_callback.borrow_mut();
        if start_frame_callback.is_some() {
            warn!("Already observing the next frame.");
        }
        *start_frame_callback = Some(new_start_frame_callback);
    }
}

#[allow(unused)]
impl RunningAppState {
    pub(super) fn new(
        initial_url: Option<String>,
        hidpi_scale_factor: f32,
        rendering_context: Rc<WindowRenderingContext>,
        servo: Servo,
        callbacks: Rc<ServoWindowCallbacks>,
        refresh_driver: Option<Rc<VsyncRefreshDriver>>,
        servoshell_preferences: ServoShellPreferences,
        webdriver_receiver: Option<Receiver<WebDriverCommandMsg>>,
    ) -> Rc<Self> {
        let initial_url = initial_url.and_then(|string| Url::parse(&string).ok());
        let initial_url = initial_url
            .or_else(|| Url::parse(&servoshell_preferences.homepage).ok())
            .or_else(|| Url::parse("about:blank").ok())
            .unwrap();

        let animating_state_changed = Rc::new(Cell::new(false));
        servo.set_delegate(Rc::new(ServoShellServoDelegate {
            animating_state_changed: animating_state_changed.clone(),
        }));

        let app_state = Rc::new(Self {
            rendering_context,
            servo,
            callbacks,
            refresh_driver,
            servoshell_preferences,
            webdriver_receiver,
            webdriver_senders: RefCell::default(),
            inner: RefCell::new(RunningAppStateInner {
                need_present: false,
                context_menu_sender: None,
                webviews: Default::default(),
                creation_order: vec![],
                focused_webview_id: None,
                animating_state_changed,
                hidpi_scale_factor: Scale::new(hidpi_scale_factor),
                achieved_stable_image: Default::default(),
                visible_input_methods: Default::default(),
            }),
        });

        app_state.create_and_focus_toplevel_webview(initial_url);
        app_state
    }

    pub(crate) fn set_script_command_interrupt_sender(
        &self,
        sender: Option<IpcSender<WebDriverJSResult>>,
    ) {
        self.webdriver_senders
            .borrow_mut()
            .script_evaluation_interrupt_sender = sender;
    }

    pub(crate) fn set_pending_traversal(
        &self,
        traversal_id: TraversalId,
        sender: GenericSender<WebDriverLoadStatus>,
    ) {
        self.webdriver_senders
            .borrow_mut()
            .pending_traversals
            .insert(traversal_id, sender);
    }

    pub fn webviews(&self) -> Vec<(WebViewId, WebView)> {
        let inner = self.inner();
        inner
            .creation_order
            .iter()
            .map(|id| (*id, inner.webviews.get(id).unwrap().clone()))
            .collect()
    }

    pub(crate) fn create_and_focus_toplevel_webview(self: &Rc<Self>, url: Url) -> WebView {
        let webview = WebViewBuilder::new(&self.servo)
            .url(url)
            .hidpi_scale_factor(self.inner().hidpi_scale_factor)
            .delegate(self.clone())
            .build();

        webview.focus();
        self.add(webview.clone());
        webview
    }

    pub(crate) fn add(&self, webview: WebView) {
        let webview_id = webview.id();
        self.inner_mut().creation_order.push(webview_id);
        self.inner_mut().webviews.insert(webview_id, webview);
        info!(
            "Added webview with ID: {:?}, total webviews: {}",
            webview_id,
            self.inner().webviews.len()
        );
    }

    /// The focused webview will not be immediately valid via `active_webview()`
    pub(crate) fn focus_webview(&self, id: WebViewId) {
        if let Some(webview) = self.inner().webviews.get(&id) {
            webview.focus();
        } else {
            error!("We could not find the webview with this id {id}");
        }
    }

    fn inner(&self) -> Ref<'_, RunningAppStateInner> {
        self.inner.borrow()
    }

    fn inner_mut(&self) -> RefMut<'_, RunningAppStateInner> {
        self.inner.borrow_mut()
    }

    pub(crate) fn servo(&self) -> &Servo {
        &self.servo
    }

    pub(crate) fn webdriver_receiver(&self) -> Option<&Receiver<WebDriverCommandMsg>> {
        self.webdriver_receiver.as_ref()
    }

    fn get_browser_id(&self) -> Result<WebViewId, &'static str> {
        let webview_id = match self.inner().focused_webview_id {
            Some(id) => id,
            None => return Err("No focused WebViewId yet."),
        };
        Ok(webview_id)
    }

    pub(crate) fn newest_webview(&self) -> Option<WebView> {
        self.inner()
            .creation_order
            .last()
            .and_then(|id| self.inner().webviews.get(id).cloned())
    }

    pub(crate) fn active_webview(&self) -> WebView {
        self.inner()
            .focused_webview_id
            .and_then(|id| self.inner().webviews.get(&id).cloned())
            .or(self.newest_webview())
            .expect("Should always have an active WebView")
    }

    fn handle_webdriver_script_command(&self, msg: &WebDriverScriptCommand) {
        match msg {
            WebDriverScriptCommand::ExecuteScript(_webview_id, response_sender) |
            WebDriverScriptCommand::ExecuteAsyncScript(_webview_id, response_sender) => {
                // Give embedder a chance to interrupt the script command.
                // Webdriver only handles 1 script command at a time, so we can
                // safely set a new interrupt sender and remove the previous one here.
                self.set_script_command_interrupt_sender(Some(response_sender.clone()));
            },
            WebDriverScriptCommand::AddLoadStatusSender(webview_id, load_status_sender) => {
                self.set_load_status_sender(*webview_id, load_status_sender.clone());
            },
            WebDriverScriptCommand::RemoveLoadStatusSender(webview_id) => {
                self.remove_load_status_sender(*webview_id);
            },
            _ => {
                self.set_script_command_interrupt_sender(None);
            },
        }
    }

    /// Request shutdown. Will call on_shutdown_complete.
    pub fn request_shutdown(&self) {
        self.servo.start_shutting_down();
        self.perform_updates();
    }

    /// Call after on_shutdown_complete
    pub fn deinit(self) {
        self.servo.deinit();
    }

    /// This is the Servo heartbeat. This needs to be called
    /// everytime wakeup is called or when embedder wants Servo
    /// to act on its pending events.
    pub fn perform_updates(&self) {
        let should_continue = self.servo.spin_event_loop();
        if !should_continue {
            self.callbacks.host_callbacks.on_shutdown_complete();
        }
        if self.inner().animating_state_changed.get() {
            self.inner().animating_state_changed.set(false);
            self.callbacks
                .host_callbacks
                .on_animating_changed(self.servo.animating());
        }
    }

    /// Load an URL.
    pub fn load_uri(&self, url: &str) {
        info!("load_uri: {}", url);

        let Some(url) =
            crate::parser::location_bar_input_to_url(url, &self.servoshell_preferences.searchpage)
        else {
            warn!("Cannot parse URL");
            return;
        };

        self.active_webview().load(url.into_url());
    }

    /// Reload the page.
    pub fn reload(&self) {
        info!("reload");
        self.active_webview().reload();
        self.perform_updates();
    }

    /// Stop loading the page.
    pub fn stop(&self) {
        warn!("TODO can't stop won't stop");
    }

    /// Go back in history.
    pub fn go_back(&self) {
        info!("go_back");
        self.active_webview().go_back(1);
        self.perform_updates();
    }

    /// Go forward in history.
    pub fn go_forward(&self) {
        info!("go_forward");
        self.active_webview().go_forward(1);
        self.perform_updates();
    }

    /// Let Servo know that the window has been resized.
    pub fn resize(&self, coordinates: Coordinates) {
        info!("resize to {:?}", coordinates,);
        let size = coordinates.viewport.size;
        self.active_webview().move_resize(size.to_f32().into());
        self.active_webview()
            .resize(PhysicalSize::new(size.width as u32, size.height as u32));
        *self.callbacks.coordinates.borrow_mut() = coordinates;
        self.perform_updates();
    }

    /// Scroll.
    /// x/y are scroll coordinates.
    /// dx/dy are scroll deltas.
    pub fn scroll(&self, dx: f32, dy: f32, x: f32, y: f32) {
        let scroll_location = ScrollLocation::Delta(Vector2D::new(dx, dy));
        let point = DevicePoint::new(x, y).into();
        self.active_webview()
            .notify_scroll_event(scroll_location, point);
        self.perform_updates();
    }

    /// WebDriver message handling methods
    pub fn webview_by_id(&self, id: WebViewId) -> Option<WebView> {
        self.inner().webviews.get(&id).cloned()
    }

    pub fn handle_webdriver_messages(self: &Rc<Self>) {
        if let Some(webdriver_receiver) = &self.webdriver_receiver {
            while let Ok(msg) = webdriver_receiver.try_recv() {
                match msg {
                    WebDriverCommandMsg::LoadUrl(webview_id, url, load_status_sender) => {
                        info!("Loading URL in webview {}: {}", webview_id, url);

                        if let Some(webview) = self.webview_by_id(webview_id) {
                            self.set_load_status_sender(webview_id, load_status_sender.clone());
                            self.inner_mut().focused_webview_id = Some(webview_id);
                            webview.focus();
                            let url_string = url.to_string();
                            webview.load(url.into_url());
                            info!(
                                "Successfully loaded URL {} in focused webview {}",
                                url_string, webview_id
                            );
                        } else {
                            warn!("WebView {} not found for LoadUrl command", webview_id);
                        }
                    },
                    WebDriverCommandMsg::NewWebView(response_sender, load_status_sender) => {
                        info!("Creating new webview via WebDriver");
                        let new_webview = self
                            .create_and_focus_toplevel_webview(Url::parse("about:blank").unwrap());

                        if let Err(error) = response_sender.send(new_webview.id()) {
                            warn!("Failed to send response of NewWebview: {error}");
                        }
                        if let Some(load_status_sender) = load_status_sender {
                            self.set_load_status_sender(new_webview.id(), load_status_sender);
                        }
                    },
                    WebDriverCommandMsg::CloseWebView(webview_id, response_sender) => {
                        info!("(Not Implemented) Closing webview {}", webview_id);
                    },
                    WebDriverCommandMsg::FocusWebView(webview_id) => {
                        if let Some(webview) = self.webview_by_id(webview_id) {
                            let focus_id = webview.focus();
                            info!("Successfully focused webview {}", webview_id);
                        }
                    },
                    WebDriverCommandMsg::IsWebViewOpen(webview_id, response_sender) => {
                        let context = self.webview_by_id(webview_id);

                        if let Err(error) = response_sender.send(context.is_some()) {
                            warn!("Failed to send response of IsWebViewOpen: {error}");
                        }
                    },
                    WebDriverCommandMsg::IsBrowsingContextOpen(..) => {
                        self.servo().execute_webdriver_command(msg);
                    },
                    WebDriverCommandMsg::GetFocusedWebView(response_sender) => {
                        let focused_id = self
                            .inner()
                            .focused_webview_id
                            .and_then(|id| self.inner().webviews.get(&id).cloned());

                        if let Err(error) = response_sender.send(focused_id.map(|w| w.id())) {
                            warn!("Failed to send response of GetFocusedWebView: {error}");
                        }
                    },
                    WebDriverCommandMsg::Refresh(webview_id, load_status_sender) => {
                        info!("Refreshing webview {}", webview_id);
                        if let Some(webview) = self.webview_by_id(webview_id) {
                            self.set_load_status_sender(webview_id, load_status_sender);
                            webview.reload();
                        } else {
                            warn!("WebView {} not found for Refresh command", webview_id);
                        }
                    },
                    WebDriverCommandMsg::GoBack(webview_id, load_status_sender) => {
                        info!("Going back in webview {}", webview_id);
                        if let Some(webview) = self.webview_by_id(webview_id) {
                            let traversal_id = webview.go_back(1);
                            self.set_pending_traversal(traversal_id, load_status_sender);
                        } else {
                            warn!("WebView {} not found for GoBack command", webview_id);
                        }
                    },
                    WebDriverCommandMsg::GoForward(webview_id, load_status_sender) => {
                        info!("Going forward in webview {}", webview_id);
                        if let Some(webview) = self.webview_by_id(webview_id) {
                            let traversal_id = webview.go_forward(1);
                            self.set_pending_traversal(traversal_id, load_status_sender);
                        } else {
                            warn!("WebView {} not found for GoForward command", webview_id);
                        }
                    },
                    WebDriverCommandMsg::GetAllWebViews(response_sender) => {
                        let webviews = self
                            .webviews()
                            .iter()
                            .map(|(id, _)| *id)
                            .collect::<Vec<_>>();

                        if let Err(error) = response_sender.send(webviews) {
                            warn!("Failed to send response of GetAllWebViews: {error}");
                        }
                    },
                    WebDriverCommandMsg::ScriptCommand(_, ref webdriver_script_command) => {
                        info!("Handling ScriptCommand: {:?}", webdriver_script_command);
                        self.handle_webdriver_script_command(webdriver_script_command);
                        self.servo().execute_webdriver_command(msg);
                    },
                    WebDriverCommandMsg::CurrentUserPrompt(webview_id, response_sender) => {
                        info!("Handling CurrentUserPrompt for webview {}", webview_id);
                        if let Err(error) = response_sender.send(None) {
                            warn!("Failed to send response of CurrentUserPrompt: {error}");
                        };
                    },
                    WebDriverCommandMsg::HandleUserPrompt(webview_id, action, response_sender) => {
                        info!(
                            "Handling HandleUserPrompt for webview {} with action {:?}",
                            webview_id, action
                        );

                        if let Err(error) = response_sender.send(Err(())) {
                            warn!("Failed to send response of HandleUserPrompt: {error}");
                        };
                    },
                    WebDriverCommandMsg::GetAlertText(webview_id, response_sender) => {
                        info!("Handling GetAlertText for webview {}", webview_id);
                        let _ = response_sender.send(Err(()));
                    },
                    WebDriverCommandMsg::SendAlertText(webview_id, text) => {
                        info!(
                            "Handling SendAlertText for webview {} with text: {}",
                            webview_id, text
                        );
                    },
                    WebDriverCommandMsg::GetViewportSize(webview_id, response_sender) => {
                        info!("Handling GetViewportSize for webview {}", webview_id);
                        let _ = response_sender.send(self.rendering_context.size2d());
                    },
                    _ => {
                        info!("Received WebDriver command: {:?}", msg);
                    },
                }
            }
        }
    }

    pub(crate) fn set_load_status_sender(
        &self,
        webview_id: WebViewId,
        sender: GenericSender<WebDriverLoadStatus>,
    ) {
        self.webdriver_senders
            .borrow_mut()
            .load_status_senders
            .insert(webview_id, sender);
    }

    pub(crate) fn remove_load_status_sender(&self, webview_id: WebViewId) {
        self.webdriver_senders
            .borrow_mut()
            .load_status_senders
            .remove(&webview_id);
    }
    /// Touch event: press down
    pub fn touch_down(&self, x: f32, y: f32, pointer_id: i32) {
        self.active_webview()
            .notify_input_event(InputEvent::Touch(TouchEvent::new(
                TouchEventType::Down,
                TouchId(pointer_id),
                DevicePoint::new(x, y).into(),
            )));
        self.perform_updates();
    }

    /// Touch event: move touching finger
    pub fn touch_move(&self, x: f32, y: f32, pointer_id: i32) {
        self.active_webview()
            .notify_input_event(InputEvent::Touch(TouchEvent::new(
                TouchEventType::Move,
                TouchId(pointer_id),
                DevicePoint::new(x, y).into(),
            )));
        self.perform_updates();
    }

    /// Touch event: Lift touching finger
    pub fn touch_up(&self, x: f32, y: f32, pointer_id: i32) {
        self.active_webview()
            .notify_input_event(InputEvent::Touch(TouchEvent::new(
                TouchEventType::Up,
                TouchId(pointer_id),
                DevicePoint::new(x, y).into(),
            )));
        self.perform_updates();
    }

    /// Cancel touch event
    pub fn touch_cancel(&self, x: f32, y: f32, pointer_id: i32) {
        self.active_webview()
            .notify_input_event(InputEvent::Touch(TouchEvent::new(
                TouchEventType::Cancel,
                TouchId(pointer_id),
                DevicePoint::new(x, y).into(),
            )));
        self.perform_updates();
    }

    /// Register a mouse movement.
    pub fn mouse_move(&self, x: f32, y: f32) {
        self.active_webview()
            .notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(
                DevicePoint::new(x, y).into(),
            )));
        self.perform_updates();
    }

    /// Register a mouse button press.
    pub fn mouse_down(&self, x: f32, y: f32, button: MouseButton) {
        self.active_webview()
            .notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                MouseButtonAction::Down,
                button,
                DevicePoint::new(x, y).into(),
            )));
        self.perform_updates();
    }

    /// Register a mouse button release.
    pub fn mouse_up(&self, x: f32, y: f32, button: MouseButton) {
        self.active_webview()
            .notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                MouseButtonAction::Up,
                button,
                DevicePoint::new(x, y).into(),
            )));
        self.perform_updates();
    }

    /// Start pinchzoom.
    /// x/y are pinch origin coordinates.
    pub fn pinchzoom_start(&self, factor: f32, x: f32, y: f32) {
        self.active_webview()
            .pinch_zoom(factor, DevicePoint::new(x, y));
        self.perform_updates();
    }

    /// Pinchzoom.
    /// x/y are pinch origin coordinates.
    pub fn pinchzoom(&self, factor: f32, x: f32, y: f32) {
        self.active_webview()
            .pinch_zoom(factor, DevicePoint::new(x, y));
        self.perform_updates();
    }

    /// End pinchzoom.
    /// x/y are pinch origin coordinates.
    pub fn pinchzoom_end(&self, factor: f32, x: f32, y: f32) {
        self.active_webview()
            .pinch_zoom(factor, DevicePoint::new(x, y));
        self.perform_updates();
    }

    pub fn key_down(&self, key: Key) {
        let key_event = KeyboardEvent::from_state_and_key(KeyState::Down, key);
        self.active_webview()
            .notify_input_event(InputEvent::Keyboard(key_event));
        self.perform_updates();
    }

    pub fn key_up(&self, key: Key) {
        let key_event = KeyboardEvent::from_state_and_key(KeyState::Up, key);
        self.active_webview()
            .notify_input_event(InputEvent::Keyboard(key_event));
        self.perform_updates();
    }

    pub fn ime_insert_text(&self, text: String) {
        // In OHOS, we get empty text after the intended text.
        if text.is_empty() {
            return;
        }
        let active_webview = self.active_webview();
        active_webview.notify_input_event(InputEvent::Keyboard(KeyboardEvent::from_state_and_key(
            KeyState::Down,
            Key::Named(NamedKey::Process),
        )));
        active_webview.notify_input_event(InputEvent::Ime(ImeEvent::Composition(
            CompositionEvent {
                state: CompositionState::End,
                data: text,
            },
        )));
        active_webview.notify_input_event(InputEvent::Keyboard(KeyboardEvent::from_state_and_key(
            KeyState::Up,
            Key::Named(NamedKey::Process),
        )));
        self.perform_updates();
    }

    pub fn notify_vsync(&self) {
        if let Some(refresh_driver) = &self.refresh_driver {
            refresh_driver.notify_vsync();
        };
        self.perform_updates();
    }

    pub fn pause_compositor(&self) {
        if let Err(e) = self.rendering_context.take_window() {
            warn!("Unbinding native surface from context failed ({:?})", e);
        }
        self.perform_updates();
    }

    pub fn resume_compositor(&self, window_handle: RawWindowHandle, coords: Coordinates) {
        let window_handle = unsafe { WindowHandle::borrow_raw(window_handle) };
        let size = coords.viewport.size.to_u32();
        if let Err(e) = self
            .rendering_context
            .set_window(window_handle, PhysicalSize::new(size.width, size.height))
        {
            warn!("Binding native surface to context failed ({:?})", e);
        }
        self.perform_updates();
    }

    pub fn media_session_action(&self, action: MediaSessionActionType) {
        info!("Media session action {:?}", action);
        self.active_webview()
            .notify_media_session_action_event(action);
        self.perform_updates();
    }

    pub fn set_throttled(&self, throttled: bool) {
        info!("set_throttled");
        self.active_webview().set_throttled(throttled);
        self.perform_updates();
    }

    pub fn ime_dismissed(&self) {
        info!("ime_dismissed");
        self.active_webview()
            .notify_input_event(InputEvent::Ime(ImeEvent::Dismissed));
        self.perform_updates();
    }

    pub fn on_context_menu_closed(&self, result: ContextMenuResult) -> Result<(), &'static str> {
        if let Some(sender) = self.inner_mut().context_menu_sender.take() {
            let _ = sender.send(result);
        } else {
            warn!("Trying to close a context menu when no context menu is active");
        }
        Ok(())
    }

    pub fn present_if_needed(&self) {
        if !self.inner().need_present {
            return;
        }

        self.inner_mut().need_present = false;
        self.active_webview().paint();
        self.rendering_context.present();

        if self.servoshell_preferences.exit_after_stable_image &&
            self.inner().achieved_stable_image.get()
        {
            self.request_shutdown();
        }
    }

    /// If we are exiting after achieving a stable image or we want to save the display of the
    /// [`WebView`] to an image file, request a screenshot of the [`WebView`].
    fn maybe_request_screenshot(&self, webview: WebView) {
        let output_path = self.servoshell_preferences.output_image_path.clone();
        if !self.servoshell_preferences.exit_after_stable_image && output_path.is_none() {
            return;
        }

        // Never request more than a single screenshot for now.
        let achieved_stable_image = self.inner().achieved_stable_image.clone();
        if achieved_stable_image.get() {
            return;
        }

        webview.take_screenshot(None, move |image| {
            achieved_stable_image.set(true);

            let Some(output_path) = output_path else {
                return;
            };

            let image = match image {
                Ok(image) => image,
                Err(error) => {
                    error!("Could not take screenshot: {error:?}");
                    return;
                },
            };

            let image_format = ImageFormat::from_path(&output_path).unwrap_or(ImageFormat::Png);
            if let Err(error) =
                DynamicImage::ImageRgba8(image).save_with_format(output_path, image_format)
            {
                error!("Failed to save screenshot: {error}.");
            }
        });
    }
}

#[cfg(feature = "webxr")]
pub(crate) struct XrDiscoveryWebXrRegistry {
    xr_discovery: RefCell<Option<servo::webxr::Discovery>>,
}

#[cfg(feature = "webxr")]
#[cfg_attr(target_env = "ohos", allow(dead_code))]
impl XrDiscoveryWebXrRegistry {
    pub(crate) fn new(xr_discovery: Option<servo::webxr::Discovery>) -> Self {
        Self {
            xr_discovery: RefCell::new(xr_discovery),
        }
    }
}

#[cfg(feature = "webxr")]
impl servo::webxr::WebXrRegistry for XrDiscoveryWebXrRegistry {
    fn register(&self, registry: &mut servo::webxr::MainThreadRegistry) {
        debug!("XrDiscoveryWebXrRegistry::register");
        if let Some(discovery) = self.xr_discovery.take() {
            registry.register(discovery);
        }
    }
}
