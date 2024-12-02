/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use arboard::Clipboard;
use base::id::WebViewId;
use compositing_traits::ConstellationMsg;
use crossbeam_channel::Sender;
use embedder_traits::{Cursor, EmbedderMsg};
use euclid::{Point2D, Size2D};
use glutin::config::{ConfigTemplateBuilder, GlConfig};
use glutin::surface::{Surface, WindowSurface};
use glutin_winit::DisplayBuilder;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use muda::{Menu as MudaMenu, MenuEvent, MenuEventReceiver, MenuItem};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use raw_window_handle::HasWindowHandle;
use script_traits::{TouchEventType, TraversalDirection, WheelDelta, WheelMode};
use servo_url::ServoUrl;
use webrender_api::units::{
    DeviceIntPoint, DeviceIntRect, DeviceIntSize, DevicePoint, LayoutVector2D,
};
use webrender_api::ScrollLocation;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, TouchPhase, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::ModifiersState;
#[cfg(any(linux, target_os = "windows"))]
use winit::window::ResizeDirection;
use winit::window::{CursorIcon, Window as WinitWindow, WindowAttributes, WindowId};

use crate::compositor::{IOCompositor, MouseWindowEvent};
use crate::context_menu::{ContextMenu, Menu};
use crate::keyboard::keyboard_event_from_winit;
use crate::rendering::{gl_config_picker, RenderingContext};
use crate::servo::send_to_constellation;
use crate::webview::{Panel, WebView};

/// A Servo window is a Winit window containing several web views.
pub struct Window {
    /// Access to Winit window
    pub(crate) window: WinitWindow,
    /// GL surface of the window
    pub(crate) surface: Surface<WindowSurface>,
    /// The main panel of this window.
    pub(crate) panel: Option<Panel>,
    /// The WebView of this window.
    pub(crate) webview: Option<WebView>,
    /// The mouse physical position in the web view.
    mouse_position: Cell<Option<PhysicalPosition<f64>>>,
    /// Modifiers state of the keyboard.
    modifiers_state: Cell<ModifiersState>,
    /// Browser history of the window.
    history: Vec<ServoUrl>,
    /// Current history index.
    current_history_index: usize,
    /// State to indicate if the window is resizing.
    pub(crate) resizing: bool,
    // TODO: These two fields should unified once we figure out servo's menu events.
    /// Context menu webview. This is only used in wayland currently.
    #[cfg(linux)]
    pub(crate) context_menu: Option<ContextMenu>,
    /// Global menu event receiver for muda crate
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    menu_event_receiver: MenuEventReceiver,
}

impl Window {
    /// Create a Servo window from Winit window and return the rendering context.
    pub fn new(
        evl: &ActiveEventLoop,
        window_attributes: WindowAttributes,
    ) -> (Self, RenderingContext) {
        let window_attributes = window_attributes
            .with_transparent(true)
            .with_decorations(false);

        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_transparency(cfg!(macos));

        let (window, gl_config) = DisplayBuilder::new()
            .with_window_attributes(Some(window_attributes))
            .build(evl, template, gl_config_picker)
            .expect("Failed to create window and gl config");

        let window = window.ok_or("Failed to create window").unwrap();

        log::debug!("Picked a config with {} samples", gl_config.num_samples());

        #[cfg(macos)]
        unsafe {
            let rwh = window.window_handle().expect("Failed to get window handle");
            if let RawWindowHandle::AppKit(AppKitWindowHandle { ns_view, .. }) = rwh.as_ref() {
                decorate_window(
                    ns_view.as_ptr() as *mut AnyObject,
                    LogicalPosition::new(8.0, 40.0),
                );
            }
        }
        let (rendering_context, surface) = RenderingContext::create(&window, &gl_config)
            .expect("Failed to create rendering context");
        log::trace!("Created rendering context for window {:?}", window);

        (
            Self {
                window,
                surface,
                panel: None,
                webview: None,
                mouse_position: Default::default(),
                modifiers_state: Cell::new(ModifiersState::default()),
                history: vec![],
                current_history_index: 0,
                resizing: false,
                #[cfg(linux)]
                context_menu: None,
                #[cfg(any(target_os = "macos", target_os = "windows"))]
                menu_event_receiver: MenuEvent::receiver().clone(),
            },
            rendering_context,
        )
    }

    /// Create a Servo window with the rendering context.
    pub fn new_with_compositor(evl: &ActiveEventLoop, compositor: &mut IOCompositor) -> Self {
        let window_attrs = WinitWindow::default_attributes()
            .with_decorations(false)
            .with_transparent(true);
        let window = evl
            .create_window(window_attrs)
            .expect("Failed to create window.");

        #[cfg(macos)]
        unsafe {
            let rwh = window.window_handle().expect("Failed to get window handle");
            if let RawWindowHandle::AppKit(AppKitWindowHandle { ns_view, .. }) = rwh.as_ref() {
                decorate_window(
                    ns_view.as_ptr() as *mut AnyObject,
                    LogicalPosition::new(8.0, 40.0),
                );
            }
        }
        let surface = compositor
            .rendering_context
            .create_surface(&window)
            .unwrap();

        let mut window = Self {
            window,
            surface,
            panel: None,
            webview: None,
            mouse_position: Default::default(),
            modifiers_state: Cell::new(ModifiersState::default()),
            history: vec![],
            current_history_index: 0,
            resizing: false,
            #[cfg(linux)]
            context_menu: None,
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            menu_event_receiver: MenuEvent::receiver().clone(),
        };
        compositor.swap_current_window(&mut window);
        window
    }

    /// Get the content area size for the webview to draw on
    pub fn get_content_size(&self, mut size: DeviceIntRect) -> DeviceIntRect {
        if self.panel.is_some() {
            size.min.y = size.max.y.min(100);
            size.min.x += 10;
            size.max.y -= 10;
            size.max.x -= 10;
        }
        size
    }

    /// Send the constellation message to start Panel UI
    pub fn create_panel(
        &mut self,
        constellation_sender: &Sender<ConstellationMsg>,
        initial_url: Option<url::Url>,
    ) {
        let size = self.window.inner_size();
        let size = Size2D::new(size.width as i32, size.height as i32);
        let panel_id = WebViewId::new();
        self.panel = Some(Panel {
            webview: WebView::new(panel_id, DeviceIntRect::from_size(size)),
            initial_url: if let Some(initial_url) = initial_url {
                ServoUrl::from_url(initial_url)
            } else {
                ServoUrl::parse("https://example.com").unwrap()
            },
        });

        let url = ServoUrl::parse("servo://panel.html").unwrap();
        send_to_constellation(
            constellation_sender,
            ConstellationMsg::NewWebView(url, panel_id),
        );
    }

    /// Create a new webview and send the constellation message to load the initial URL
    pub fn create_webview(
        &mut self,
        constellation_sender: &Sender<ConstellationMsg>,
        initial_url: ServoUrl,
    ) {
        let webview_id = WebViewId::new();
        let size = self.size();
        let rect = DeviceIntRect::from_size(size);
        let mut webview = WebView::new(webview_id, rect);
        webview.set_size(self.get_content_size(rect));
        self.webview.replace(webview);
        send_to_constellation(
            constellation_sender,
            ConstellationMsg::NewWebView(initial_url, webview_id),
        );
        log::debug!("Servo Window {:?} adds webview {}", self.id(), webview_id);
    }

    /// Handle Winit window event and return a boolean to indicate if the compositor should repaint immediately.
    pub fn handle_winit_window_event(
        &mut self,
        sender: &Sender<ConstellationMsg>,
        compositor: &mut IOCompositor,
        event: &winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                if compositor.ready_to_present {
                    self.window.pre_present_notify();
                    if let Err(err) = compositor.rendering_context.present(&self.surface) {
                        log::warn!("Failed to present surface: {:?}", err);
                    }
                    compositor.ready_to_present = false;
                }
            },
            WindowEvent::Focused(focused) => {
                if *focused {
                    compositor.swap_current_window(self);
                }
            },
            WindowEvent::Resized(size) => {
                if self.window.has_focus() {
                    self.resizing = true;
                }
                let size = Size2D::new(size.width, size.height);
                compositor.resize(size.to_i32(), self);
            },
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                compositor.on_scale_factor_event(*scale_factor as f32, self);
            },
            WindowEvent::CursorEntered { .. } => {
                compositor.swap_current_window(self);
            },
            WindowEvent::CursorLeft { .. } => {
                self.mouse_position.set(None);
            },
            WindowEvent::CursorMoved { position, .. } => {
                let cursor: DevicePoint = DevicePoint::new(position.x as f32, position.y as f32);
                self.mouse_position.set(Some(*position));
                compositor.on_mouse_window_move_event_class(cursor);

                // handle Windows and Linux non-decoration window resize cursor
                #[cfg(any(linux, target_os = "windows"))]
                {
                    if self.is_resizable() {
                        let direction = self.get_drag_resize_direction();
                        self.set_drag_resize_cursor(direction);
                    }
                }
            },
            WindowEvent::MouseInput { state, button, .. } => {
                let position = match self.mouse_position.get() {
                    Some(position) => Point2D::new(position.x as f32, position.y as f32),
                    None => {
                        log::trace!("Mouse position is None, skipping MouseInput event.");
                        return;
                    },
                };

                /* handle context menu */
                // TODO(context-menu): should create on ShowContextMenu event

                match (state, button) {
                    #[cfg(any(target_os = "macos", target_os = "windows"))]
                    (ElementState::Pressed, winit::event::MouseButton::Right) => {
                        self.show_context_menu();
                        // FIXME: there's chance to lose the event since the channel is async.
                        if let Ok(event) = self.menu_event_receiver.try_recv() {
                            self.handle_context_menu_event(sender, event);
                        }
                    },
                    #[cfg(linux)]
                    (ElementState::Pressed, winit::event::MouseButton::Right) => {
                        if self.context_menu.is_none() {
                            self.context_menu = Some(self.show_context_menu(sender));
                            return;
                        }
                    },
                    #[cfg(linux)]
                    // TODO(context-menu): ignore first release event after context menu open or close to prevent click
                    // on background element
                    (ElementState::Released, winit::event::MouseButton::Right) => {
                        if self.context_menu.is_some() {
                            return;
                        }
                    },
                    _ => {},
                }

                /* handle Windows and Linux non-decoration window resize */
                #[cfg(any(linux, target_os = "windows"))]
                {
                    if *state == ElementState::Pressed && *button == winit::event::MouseButton::Left
                    {
                        if self.is_resizable() {
                            self.drag_resize_window();
                        }
                    }
                }

                /* handle mouse events */

                let button: script_traits::MouseButton = match button {
                    winit::event::MouseButton::Left => script_traits::MouseButton::Left,
                    winit::event::MouseButton::Right => script_traits::MouseButton::Right,
                    winit::event::MouseButton::Middle => script_traits::MouseButton::Middle,
                    _ => {
                        log::trace!(
                            "Servo Window isn't supporting this mouse button yet: {button:?}"
                        );
                        return;
                    },
                };

                let event: MouseWindowEvent = match state {
                    ElementState::Pressed => MouseWindowEvent::MouseDown(button, position),
                    ElementState::Released => {
                        self.resizing = false;
                        MouseWindowEvent::MouseUp(button, position)
                    },
                };
                compositor.on_mouse_window_event_class(event);

                // Winit didn't send click event, so we send it after mouse up
                if *state == ElementState::Released {
                    let event: MouseWindowEvent = MouseWindowEvent::Click(button, position);
                    compositor.on_mouse_window_event_class(event);
                }
            },
            WindowEvent::PinchGesture { delta, .. } => {
                compositor.on_zoom_window_event(1.0 + *delta as f32, self);
            },
            WindowEvent::MouseWheel { delta, phase, .. } => {
                let position = match self.mouse_position.get() {
                    Some(position) => position,
                    None => {
                        log::trace!("Mouse position is None, skipping MouseWheel event.");
                        return;
                    },
                };

                // FIXME: Pixels per line, should be configurable (from browser setting?) and vary by zoom level.
                const LINE_HEIGHT: f32 = 38.0;

                let (mut x, mut y, mode) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        (*x as f64, (*y * LINE_HEIGHT) as f64, WheelMode::DeltaLine)
                    },
                    winit::event::MouseScrollDelta::PixelDelta(position) => {
                        let position = position.to_logical::<f64>(self.window.scale_factor());
                        (position.x, position.y, WheelMode::DeltaPixel)
                    },
                };

                // Wheel Event
                compositor.on_wheel_event(
                    WheelDelta { x, y, z: 0.0, mode },
                    DevicePoint::new(position.x as f32, position.y as f32),
                );

                // Scroll Event
                // Do one axis at a time.
                if y.abs() >= x.abs() {
                    x = 0.0;
                } else {
                    y = 0.0;
                }

                let phase: TouchEventType = match phase {
                    TouchPhase::Started => TouchEventType::Down,
                    TouchPhase::Moved => TouchEventType::Move,
                    TouchPhase::Ended => TouchEventType::Up,
                    TouchPhase::Cancelled => TouchEventType::Cancel,
                };

                compositor.on_scroll_event(
                    ScrollLocation::Delta(LayoutVector2D::new(x as f32, y as f32)),
                    DeviceIntPoint::new(position.x as i32, position.y as i32),
                    phase,
                );
            },
            WindowEvent::ModifiersChanged(modifier) => self.modifiers_state.set(modifier.state()),
            WindowEvent::KeyboardInput { event, .. } => {
                let event = keyboard_event_from_winit(event, self.modifiers_state.get());
                log::trace!("Servo is handling {:?}", event);
                let msg = ConstellationMsg::Keyboard(event);
                send_to_constellation(sender, msg);
            },
            e => log::trace!("Servo Window isn't supporting this window event yet: {e:?}"),
        }
    }

    /// Handle servo messages. Return true if it requests a new window
    pub fn handle_servo_message(
        &mut self,
        webview_id: WebViewId,
        message: EmbedderMsg,
        sender: &Sender<ConstellationMsg>,
        clipboard: Option<&mut Clipboard>,
        compositor: &mut IOCompositor,
    ) -> bool {
        // Handle message in Servo Panel
        if let Some(panel) = &self.panel {
            if panel.webview.webview_id == webview_id {
                return self.handle_servo_messages_with_panel(
                    webview_id, message, sender, clipboard, compositor,
                );
            }
        }
        #[cfg(linux)]
        if let Some(context_menu) = &self.context_menu {
            if context_menu.webview().webview_id == webview_id {
                self.handle_servo_messages_with_context_menu(
                    webview_id, message, sender, clipboard, compositor,
                );
                return false;
            }
        }
        // Handle message in Servo WebView
        self.handle_servo_messages_with_webview(webview_id, message, sender, clipboard, compositor);
        false
    }

    /// Queues a Winit `WindowEvent::RedrawRequested` event to be emitted
    /// that aligns with the windowing system drawing loop.
    pub fn request_redraw(&self) {
        self.window.request_redraw()
    }

    /// Size of the window that's used by webrender.
    pub fn size(&self) -> DeviceIntSize {
        let size = self.window.inner_size();
        Size2D::new(size.width as i32, size.height as i32)
    }

    /// Get Winit window ID of the window.
    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    /// Scale factor of the window. This is also known as HIDPI.
    pub fn scale_factor(&self) -> f64 {
        self.window.scale_factor()
    }

    /// Check if the window has such webview.
    pub fn has_webview(&self, id: WebViewId) -> bool {
        #[cfg(linux)]
        if self
            .context_menu
            .as_ref()
            .map_or(false, |w| w.webview().webview_id == id)
        {
            return true;
        }

        self.panel
            .as_ref()
            .map_or(false, |w| w.webview.webview_id == id) ||
            self.webview.as_ref().map_or(false, |w| w.webview_id == id)
    }

    /// Remove the webview in this window by provided webview ID. If this is the panel, it will
    /// shut down the compositor and then close whole application.
    pub fn remove_webview(
        &mut self,
        id: WebViewId,
        compositor: &mut IOCompositor,
    ) -> (Option<WebView>, bool) {
        #[cfg(linux)]
        if self
            .context_menu
            .as_ref()
            .filter(|menu| menu.webview().webview_id == id)
            .is_some()
        {
            let context_menu = self.context_menu.take().expect("Context menu should exist");
            return (Some(context_menu.webview().clone()), false);
        }

        if self
            .panel
            .as_ref()
            .filter(|w| w.webview.webview_id == id)
            .is_some()
        {
            if let Some(w) = self.webview.as_ref() {
                send_to_constellation(
                    &compositor.constellation_chan,
                    ConstellationMsg::CloseWebView(w.webview_id),
                )
            }
            (self.panel.take().map(|panel| panel.webview), false)
        } else if self
            .webview
            .as_ref()
            .filter(|w| w.webview_id == id)
            .is_some()
        {
            (self.webview.take(), self.panel.is_none())
        } else {
            (None, false)
        }
    }

    /// Get the painting order of this window.
    pub fn painting_order(&self) -> Vec<&WebView> {
        let mut order = vec![];
        if let Some(panel) = &self.panel {
            order.push(&panel.webview);
        }
        if let Some(webview) = &self.webview {
            order.push(webview);
        }

        #[cfg(linux)]
        if let Some(context_menu) = &self.context_menu {
            order.push(context_menu.webview());
        }

        order
    }

    /// Set cursor icon of the window.
    pub fn set_cursor_icon(&self, cursor: Cursor) {
        let winit_cursor = match cursor {
            Cursor::Default => CursorIcon::Default,
            Cursor::Pointer => CursorIcon::Pointer,
            Cursor::ContextMenu => CursorIcon::ContextMenu,
            Cursor::Help => CursorIcon::Help,
            Cursor::Progress => CursorIcon::Progress,
            Cursor::Wait => CursorIcon::Wait,
            Cursor::Cell => CursorIcon::Cell,
            Cursor::Crosshair => CursorIcon::Crosshair,
            Cursor::Text => CursorIcon::Text,
            Cursor::VerticalText => CursorIcon::VerticalText,
            Cursor::Alias => CursorIcon::Alias,
            Cursor::Copy => CursorIcon::Copy,
            Cursor::Move => CursorIcon::Move,
            Cursor::NoDrop => CursorIcon::NoDrop,
            Cursor::NotAllowed => CursorIcon::NotAllowed,
            Cursor::Grab => CursorIcon::Grab,
            Cursor::Grabbing => CursorIcon::Grabbing,
            Cursor::EResize => CursorIcon::EResize,
            Cursor::NResize => CursorIcon::NResize,
            Cursor::NeResize => CursorIcon::NeResize,
            Cursor::NwResize => CursorIcon::NwResize,
            Cursor::SResize => CursorIcon::SResize,
            Cursor::SeResize => CursorIcon::SeResize,
            Cursor::SwResize => CursorIcon::SwResize,
            Cursor::WResize => CursorIcon::WResize,
            Cursor::EwResize => CursorIcon::EwResize,
            Cursor::NsResize => CursorIcon::NsResize,
            Cursor::NeswResize => CursorIcon::NeswResize,
            Cursor::NwseResize => CursorIcon::NwseResize,
            Cursor::ColResize => CursorIcon::ColResize,
            Cursor::RowResize => CursorIcon::RowResize,
            Cursor::AllScroll => CursorIcon::AllScroll,
            Cursor::ZoomIn => CursorIcon::ZoomIn,
            Cursor::ZoomOut => CursorIcon::ZoomOut,
            _ => CursorIcon::Default,
        };
        self.window.set_cursor(winit_cursor);
    }

    /// Update the history of the window.
    pub fn update_history(&mut self, history: &[ServoUrl], current_index: usize) {
        self.history = history.to_vec();
        self.current_history_index = current_index;
    }
}

// Context Menu methods
impl Window {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub(crate) fn show_context_menu(&self) {
        let history_len = self.history.len();

        // items
        let back = MenuItem::with_id("back", "Back", self.current_history_index > 0, None);
        let forward = MenuItem::with_id(
            "forward",
            "Forward",
            self.current_history_index + 1 < history_len,
            None,
        );
        let reload = MenuItem::with_id("reload", "Reload", true, None);

        let menu = MudaMenu::new();
        let _ = menu.append_items(&[&back, &forward, &reload]);

        let context_menu = ContextMenu::new_with_menu(Menu(menu));
        context_menu.show(self.window.window_handle().unwrap());
    }

    #[cfg(linux)]
    pub(crate) fn show_context_menu(&mut self, sender: &Sender<ConstellationMsg>) -> ContextMenu {
        use crate::context_menu::MenuItem;

        let history_len = self.history.len();

        // items
        let back = MenuItem::new(Some("back"), "Back", self.current_history_index > 0);
        let forward = MenuItem::new(
            Some("forward"),
            "Forward",
            self.current_history_index + 1 < history_len,
        );
        let reload = MenuItem::new(Some("reload"), "Reload", true);

        let mut context_menu = ContextMenu::new_with_menu(Menu(vec![back, forward, reload]));

        let position = self.mouse_position.get().unwrap();
        context_menu.show(sender, self, position);

        context_menu
    }

    /// Close window's context menu
    pub(crate) fn close_context_menu(&self, _sender: &Sender<ConstellationMsg>) {
        #[cfg(linux)]
        if let Some(context_menu) = &self.context_menu {
            send_to_constellation(
                _sender,
                ConstellationMsg::CloseWebView(context_menu.webview().webview_id),
            );
        }
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    fn handle_context_menu_event(&self, sender: &Sender<ConstellationMsg>, event: MenuEvent) {
        // TODO: should be more flexible to handle different menu items
        match event.id().0.as_str() {
            "back" => {
                send_to_constellation(
                    sender,
                    ConstellationMsg::TraverseHistory(
                        self.webview.as_ref().unwrap().webview_id,
                        TraversalDirection::Back(1),
                    ),
                );
            },
            "forward" => {
                send_to_constellation(
                    sender,
                    ConstellationMsg::TraverseHistory(
                        self.webview.as_ref().unwrap().webview_id,
                        TraversalDirection::Forward(1),
                    ),
                );
            },
            "reload" => {
                send_to_constellation(
                    sender,
                    ConstellationMsg::Reload(self.webview.as_ref().unwrap().webview_id),
                );
            },
            _ => {},
        }
    }

    /// Handle linux context menu event
    // TODO(context-menu): should make the call in synchronous way after calling show_context_menu, otherwise
    // we'll have to deal with constellation sender and other parameter's lifetime, also we lose the context
    // that why this context menu popup
    #[cfg(linux)]
    pub(crate) fn handle_context_menu_event(
        &mut self,
        sender: &Sender<ConstellationMsg>,
        event: crate::context_menu::ContextMenuResult,
    ) {
        self.close_context_menu(sender);
        match event.id.as_str() {
            "back" => {
                send_to_constellation(
                    sender,
                    ConstellationMsg::TraverseHistory(
                        self.webview.as_ref().unwrap().webview_id,
                        TraversalDirection::Back(1),
                    ),
                );
            },
            "forward" => {
                send_to_constellation(
                    sender,
                    ConstellationMsg::TraverseHistory(
                        self.webview.as_ref().unwrap().webview_id,
                        TraversalDirection::Forward(1),
                    ),
                );
            },
            "reload" => {
                send_to_constellation(
                    sender,
                    ConstellationMsg::Reload(self.webview.as_ref().unwrap().webview_id),
                );
            },
            _ => {},
        }
    }
}

// Non-decorated window resizing for Windows and Linux.
#[cfg(any(linux, target_os = "windows"))]
impl Window {
    /// Check current window state is allowed to drag-resize.
    fn is_resizable(&self) -> bool {
        // TODO: Check if the window is in fullscreen mode.
        !self.window.is_maximized() && self.window.is_resizable()
    }

    /// Drag resize the window.
    fn drag_resize_window(&self) {
        if let Some(direction) = self.get_drag_resize_direction() {
            if let Err(err) = self.window.drag_resize_window(direction) {
                log::error!("Failed to drag-resize window: {:?}", err);
            }
        }
    }

    /// Get drag-resize direction.
    fn get_drag_resize_direction(&self) -> Option<ResizeDirection> {
        let mouse_position = match self.mouse_position.get() {
            Some(position) => position,
            None => {
                return None;
            },
        };

        let window_size = self.window.outer_size();
        let border_size = 5.0 * self.window.scale_factor();

        let x_direction = if mouse_position.x < border_size {
            Some(ResizeDirection::West)
        } else if mouse_position.x > (window_size.width as f64 - border_size) {
            Some(ResizeDirection::East)
        } else {
            None
        };

        let y_direction = if mouse_position.y < border_size {
            Some(ResizeDirection::North)
        } else if mouse_position.y > (window_size.height as f64 - border_size) {
            Some(ResizeDirection::South)
        } else {
            None
        };

        let direction = match (x_direction, y_direction) {
            (Some(ResizeDirection::East), None) => ResizeDirection::East,
            (Some(ResizeDirection::West), None) => ResizeDirection::West,
            (None, Some(ResizeDirection::South)) => ResizeDirection::South,
            (None, Some(ResizeDirection::North)) => ResizeDirection::North,
            (Some(ResizeDirection::East), Some(ResizeDirection::North)) => {
                ResizeDirection::NorthEast
            },
            (Some(ResizeDirection::West), Some(ResizeDirection::North)) => {
                ResizeDirection::NorthWest
            },
            (Some(ResizeDirection::East), Some(ResizeDirection::South)) => {
                ResizeDirection::SouthEast
            },
            (Some(ResizeDirection::West), Some(ResizeDirection::South)) => {
                ResizeDirection::SouthWest
            },
            _ => return None,
        };

        Some(direction)
    }

    /// Set drag-resize cursor when mouse is hover on the border of the window.
    fn set_drag_resize_cursor(&self, direction: Option<ResizeDirection>) {
        let cursor = match direction {
            Some(direction) => match direction {
                ResizeDirection::East => CursorIcon::EResize,
                ResizeDirection::West => CursorIcon::WResize,
                ResizeDirection::South => CursorIcon::SResize,
                ResizeDirection::North => CursorIcon::NResize,
                ResizeDirection::NorthEast => CursorIcon::NeResize,
                ResizeDirection::NorthWest => CursorIcon::NwResize,
                ResizeDirection::SouthEast => CursorIcon::SeResize,
                ResizeDirection::SouthWest => CursorIcon::SwResize,
            },
            None => CursorIcon::Default,
        };

        self.window.set_cursor(cursor);
    }
}

/* window decoration */
#[cfg(macos)]
use objc2::runtime::AnyObject;
#[cfg(macos)]
use raw_window_handle::{AppKitWindowHandle, RawWindowHandle};
#[cfg(macos)]
use winit::dpi::LogicalPosition;

/// Window decoration for macOS.
#[cfg(macos)]
pub unsafe fn decorate_window(view: *mut AnyObject, _position: LogicalPosition<f64>) {
    use objc2::rc::Id;
    use objc2_app_kit::{NSView, NSWindowStyleMask, NSWindowTitleVisibility};

    let ns_view: Id<NSView> = unsafe { Id::retain(view.cast()) }.unwrap();
    let window = ns_view
        .window()
        .expect("view was not installed in a window");
    window.setTitlebarAppearsTransparent(true);
    window.setTitleVisibility(NSWindowTitleVisibility::NSWindowTitleHidden);
    window.setStyleMask(
        NSWindowStyleMask::Titled |
            NSWindowStyleMask::FullSizeContentView |
            NSWindowStyleMask::Closable |
            NSWindowStyleMask::Resizable |
            NSWindowStyleMask::Miniaturizable,
    );
}
