/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Instant;

use egui::{CentralPanel, Frame, Key, Modifiers, PaintCallback, TopBottomPanel};
use egui_glow::CallbackFn;
use egui_winit::EventResponse;
use euclid::{Length, Point2D, Scale};
use gleam::gl;
use glow::NativeFramebuffer;
use log::{trace, warn};
use servo::compositing::windowing::EmbedderEvent;
use servo::msg::constellation_msg::TraversalDirection;
use servo::rendering_context::RenderingContext;
use servo::servo_geometry::DeviceIndependentPixel;
use servo::servo_url::ServoUrl;

use crate::egui_glue::EguiGlow;
use crate::events_loop::EventsLoop;
use crate::geometry::winit_position_to_euclid_point;
use crate::parser::location_bar_input_to_url;
use crate::webview::WebViewManager;
use crate::window_trait::WindowPortsMethods;

pub struct Minibrowser {
    pub context: EguiGlow,
    pub event_queue: RefCell<Vec<MinibrowserEvent>>,
    pub toolbar_height: Length<f32, DeviceIndependentPixel>,

    /// The framebuffer object name for the widget surface we should draw to, or None if our widget
    /// surface does not use a framebuffer object.
    widget_surface_fbo: Option<NativeFramebuffer>,

    last_update: Instant,
    last_mouse_position: Option<Point2D<f32, DeviceIndependentPixel>>,
    location: RefCell<String>,

    /// Whether the location has been edited by the user without clicking Go.
    location_dirty: Cell<bool>,
}

pub enum MinibrowserEvent {
    /// Go button clicked.
    Go,
    Back,
    Forward,
}

impl Minibrowser {
    pub fn new(
        rendering_context: &RenderingContext,
        events_loop: &EventsLoop,
        window: &dyn WindowPortsMethods,
        initial_url: ServoUrl,
    ) -> Self {
        let gl = unsafe {
            glow::Context::from_loader_function(|s| rendering_context.get_proc_address(s))
        };

        // Adapted from https://github.com/emilk/egui/blob/9478e50d012c5138551c38cbee16b07bc1fcf283/crates/egui_glow/examples/pure_glow.rs
        let context = EguiGlow::new(events_loop.as_winit(), Arc::new(gl), None);
        context
            .egui_ctx
            .set_pixels_per_point(window.hidpi_factor().get());

        let widget_surface_fbo = match rendering_context.context_surface_info() {
            Ok(Some(info)) => NonZeroU32::new(info.framebuffer_object).map(NativeFramebuffer),
            Ok(None) => panic!("Failed to get widget surface info from surfman!"),
            Err(error) => panic!("Failed to get widget surface info from surfman! {error:?}"),
        };

        Self {
            context,
            event_queue: RefCell::new(vec![]),
            toolbar_height: Default::default(),
            widget_surface_fbo,
            last_update: Instant::now(),
            last_mouse_position: None,
            location: RefCell::new(initial_url.to_string()),
            location_dirty: false.into(),
        }
    }

    /// Preprocess the given [winit::event::WindowEvent], returning unconsumed for mouse events in
    /// the Servo browser rect. This is needed because the CentralPanel we create for our webview
    /// would otherwise make egui report events in that area as consumed.
    pub fn on_window_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> EventResponse {
        let mut result = self.context.on_window_event(window, event);
        result.consumed &= match event {
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let scale = Scale::<_, DeviceIndependentPixel, _>::new(
                    self.context.egui_ctx.pixels_per_point(),
                );
                self.last_mouse_position =
                    Some(winit_position_to_euclid_point(*position).to_f32() / scale);
                self.last_mouse_position
                    .map_or(false, |p| self.is_in_browser_rect(p))
            },
            winit::event::WindowEvent::MouseWheel { .. } |
            winit::event::WindowEvent::MouseInput { .. } => self
                .last_mouse_position
                .map_or(false, |p| self.is_in_browser_rect(p)),
            _ => true,
        };
        result
    }

    /// Return true iff the given position is in the Servo browser rect.
    fn is_in_browser_rect(&self, position: Point2D<f32, DeviceIndependentPixel>) -> bool {
        position.y < self.toolbar_height.get()
    }

    /// Update the minibrowser, but don’t paint.
    /// If `servo_framebuffer_id` is given, set up a paint callback to blit its contents to our
    /// CentralPanel when [`Minibrowser::paint`] is called.
    pub fn update(
        &mut self,
        window: &winit::window::Window,
        servo_framebuffer_id: Option<gl::GLuint>,
        reason: &'static str,
    ) {
        let now = Instant::now();
        trace!(
            "{:?} since last update ({})",
            now - self.last_update,
            reason
        );
        let Self {
            context,
            event_queue,
            toolbar_height,
            widget_surface_fbo,
            last_update,
            location,
            location_dirty,
            ..
        } = self;
        let widget_fbo = *widget_surface_fbo;
        let _duration = context.run(window, |ctx| {
            TopBottomPanel::top("toolbar").show(ctx, |ui| {
                ui.allocate_ui_with_layout(
                    ui.available_size(),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        if ui.button("back").clicked() {
                            event_queue.borrow_mut().push(MinibrowserEvent::Back);
                        }
                        if ui.button("forward").clicked() {
                            event_queue.borrow_mut().push(MinibrowserEvent::Forward);
                        }
                        ui.allocate_ui_with_layout(
                            ui.available_size(),
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ui.button("go").clicked() {
                                    event_queue.borrow_mut().push(MinibrowserEvent::Go);
                                    location_dirty.set(false);
                                }

                                let location_field = ui.add_sized(
                                    ui.available_size(),
                                    egui::TextEdit::singleline(&mut *location.borrow_mut()),
                                );

                                if location_field.changed() {
                                    location_dirty.set(true);
                                }
                                if ui.input(|i| i.clone().consume_key(Modifiers::COMMAND, Key::L)) {
                                    location_field.request_focus();
                                }
                                if location_field.lost_focus() &&
                                    ui.input(|i| i.clone().key_pressed(Key::Enter))
                                {
                                    event_queue.borrow_mut().push(MinibrowserEvent::Go);
                                    location_dirty.set(false);
                                }
                            },
                        );
                    },
                );
            });

            // The toolbar height is where the Context’s available rect starts.
            // For reasons that are unclear, the TopBottomPanel’s ui cursor exceeds this by one egui
            // point, but the Context is correct and the TopBottomPanel is wrong.
            *toolbar_height = Length::new(ctx.available_rect().min.y);

            CentralPanel::default()
                .frame(Frame::none())
                .show(ctx, |ui| {
                    let min = ui.cursor().min;
                    let size = ui.available_size();
                    let rect = egui::Rect::from_min_size(min, size);
                    ui.allocate_space(size);

                    let Some(servo_fbo) = servo_framebuffer_id else {
                        return;
                    };
                    ui.painter().add(PaintCallback {
                        rect,
                        callback: Arc::new(CallbackFn::new(move |info, painter| {
                            use glow::HasContext as _;
                            let clip = info.viewport_in_pixels();
                            let x = clip.left_px as gl::GLint;
                            let y = clip.from_bottom_px as gl::GLint;
                            let width = clip.width_px as gl::GLsizei;
                            let height = clip.height_px as gl::GLsizei;
                            unsafe {
                                painter.gl().clear_color(0.0, 0.0, 0.0, 0.0);
                                painter.gl().scissor(x, y, width, height);
                                painter.gl().enable(gl::SCISSOR_TEST);
                                painter.gl().clear(gl::COLOR_BUFFER_BIT);
                                painter.gl().disable(gl::SCISSOR_TEST);

                                let servo_fbo = NonZeroU32::new(servo_fbo).map(NativeFramebuffer);
                                painter
                                    .gl()
                                    .bind_framebuffer(gl::READ_FRAMEBUFFER, servo_fbo);
                                painter
                                    .gl()
                                    .bind_framebuffer(gl::DRAW_FRAMEBUFFER, widget_fbo);
                                painter.gl().blit_framebuffer(
                                    x,
                                    y,
                                    x + width,
                                    y + height,
                                    x,
                                    y,
                                    x + width,
                                    y + height,
                                    gl::COLOR_BUFFER_BIT,
                                    gl::NEAREST,
                                );
                                painter.gl().bind_framebuffer(gl::FRAMEBUFFER, widget_fbo);
                            }
                        })),
                    });
                });

            *last_update = now;
        });
    }

    /// Paint the minibrowser, as of the last update.
    pub fn paint(&mut self, window: &winit::window::Window) {
        unsafe {
            use glow::HasContext as _;
            self.context
                .painter
                .gl()
                .bind_framebuffer(gl::FRAMEBUFFER, self.widget_surface_fbo);
        }
        self.context.paint(window);
    }

    /// Takes any outstanding events from the [Minibrowser], converting them to [EmbedderEvent] and
    /// routing those to the App event queue.
    pub fn queue_embedder_events_for_minibrowser_events(
        &self,
        browser: &WebViewManager<dyn WindowPortsMethods>,
        app_event_queue: &mut Vec<EmbedderEvent>,
    ) {
        for event in self.event_queue.borrow_mut().drain(..) {
            match event {
                MinibrowserEvent::Go => {
                    let browser_id = browser.webview_id().unwrap();
                    let location = self.location.borrow();
                    if let Some(url) = location_bar_input_to_url(&location.clone()) {
                        app_event_queue.push(EmbedderEvent::LoadUrl(browser_id, url));
                    } else {
                        warn!("failed to parse location");
                        break;
                    }
                },
                MinibrowserEvent::Back => {
                    let browser_id = browser.webview_id().unwrap();
                    app_event_queue.push(EmbedderEvent::Navigation(
                        browser_id,
                        TraversalDirection::Back(1),
                    ));
                },
                MinibrowserEvent::Forward => {
                    let browser_id = browser.webview_id().unwrap();
                    app_event_queue.push(EmbedderEvent::Navigation(
                        browser_id,
                        TraversalDirection::Forward(1),
                    ));
                },
            }
        }
    }

    /// Updates the location field from the given [WebViewManager], unless the user has started
    /// editing it without clicking Go, returning true iff it has changed (needing an egui update).
    pub fn update_location_in_toolbar(
        &mut self,
        browser: &mut WebViewManager<dyn WindowPortsMethods>,
    ) -> bool {
        // User edited without clicking Go?
        if self.location_dirty.get() {
            return false;
        }

        match browser.current_url_string() {
            Some(location) if location != self.location.get_mut() => {
                self.location = RefCell::new(location.to_owned());
                true
            },
            _ => false,
        }
    }
}
