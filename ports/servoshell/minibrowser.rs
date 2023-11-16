/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::sync::Arc;
use std::time::Instant;

use egui::{Key, Modifiers, TopBottomPanel, CentralPanel, Pos2, Vec2, InnerResponse, Id, Sense};
use euclid::{Length, Scale, Point2D, Size2D, Rect};
use log::{trace, warn};
use servo::compositing::windowing::EmbedderEvent;
use servo::servo_geometry::DeviceIndependentPixel;
use servo::style_traits::DevicePixel;
use servo::webrender_surfman::WebrenderSurfman;

use crate::browser::BrowserManager;
use crate::egui_glue::EguiGlow;
use crate::events_loop::EventsLoop;
use crate::parser::location_bar_input_to_url;
use crate::window_trait::WindowPortsMethods;

pub struct Minibrowser {
    pub context: EguiGlow,
    pub event_queue: RefCell<Vec<MinibrowserEvent>>,
    pub toolbar_height: Cell<Length<f32, DeviceIndependentPixel>>,
    last_update: Instant,
    location: RefCell<String>,

    /// Whether the location has been edited by the user without clicking Go.
    location_dirty: Cell<bool>,
}

pub enum MinibrowserEvent {
    /// Go button clicked.
    Go,
}

impl Minibrowser {
    pub fn new(
        webrender_surfman: &WebrenderSurfman,
        events_loop: &EventsLoop,
        window: &dyn WindowPortsMethods,
    ) -> Self {
        let gl = unsafe {
            glow::Context::from_loader_function(|s| webrender_surfman.get_proc_address(s))
        };

        // Adapted from https://github.com/emilk/egui/blob/9478e50d012c5138551c38cbee16b07bc1fcf283/crates/egui_glow/examples/pure_glow.rs
        let context = EguiGlow::new(events_loop.as_winit(), Arc::new(gl), None);
        context
            .egui_ctx
            .set_pixels_per_point(window.hidpi_factor().get());

        Self {
            context,
            event_queue: RefCell::new(vec![]),
            toolbar_height: Default::default(),
            last_update: Instant::now(),
            location: RefCell::new(String::default()),
            location_dirty: false.into(),
        }
    }

    /// Update the minibrowser, but don’t paint.
    pub fn update(&mut self, window: &winit::window::Window, browsers: &mut BrowserManager<dyn WindowPortsMethods>, reason: &'static str) {
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
            last_update,
            location,
            location_dirty,
        } = self;
        let _duration = context.run(window, |ctx| {
            let InnerResponse { inner: height, .. } = TopBottomPanel::top("toolbar").show(ctx, |ui| {
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
                ui.cursor().min.y
            });
            toolbar_height.set(Length::new(height));
            CentralPanel::default().show(ctx, |_| {});

            // Add an egui window for each top-level browsing context.
            let scale = Scale::<_, DeviceIndependentPixel, DevicePixel>::new(ctx.pixels_per_point());
            let toolbar_size = Size2D::new(0.0, toolbar_height.get().get());
            let painting_order = browsers.painting_order()
                .map(|(&id, _)| id)
                .collect::<Vec<_>>();
            let mut embedder_events = vec![];
            for browser_id in painting_order {
                if let Some(browser) = browsers.get_mut(browser_id) {
                    let mut open = true;
                    egui::Window::new(format!("Window({:?})", browser_id))
                        .id(Id::new(format!("Window({:?})", browser_id)))
                        .default_pos(browser.rect.origin.to_tuple())
                        .default_size(browser.rect.size.to_tuple())
                        .collapsible(false)
                        .open(&mut open)
                        .show(ctx, |ui| {
                            let Pos2 { x, y } = ui.cursor().min;
                            let origin = Point2D::new(x, y) - toolbar_size;
                            let Vec2 { x, y } = ui.available_size();
                            let size = Size2D::new(x, y);
                            let rect = Rect::new(origin, size) * scale;
                            if rect != browser.rect {
                                browser.rect = rect;
                                embedder_events.push(EmbedderEvent::MoveResizeBrowser(browser_id, rect));
                            }

                            let min = ui.cursor().min;
                            let size = ui.available_size();
                            let rect = egui::Rect::from_min_size(min, size);
                            ui.allocate_space(size);
                            let _todo = ui.interact(rect, Id::new(format!("interact({:?})", browser_id)), Sense::click_and_drag());
                        });
                    if !open {
                        embedder_events.push(EmbedderEvent::CloseBrowser(browser_id));
                    }
                }
            }
            if !embedder_events.is_empty() {
                browsers.handle_window_events(embedder_events);
            }

            *last_update = now;
        });
    }

    /// Paint the minibrowser, as of the last update.
    pub fn paint(&mut self, window: &winit::window::Window) {
        self.context.paint(window);
    }

    /// Takes any outstanding events from the [Minibrowser], converting them to [EmbedderEvent] and
    /// routing those to the App event queue.
    pub fn queue_embedder_events_for_minibrowser_events(
        &self,
        browser: &BrowserManager<dyn WindowPortsMethods>,
        app_event_queue: &mut Vec<EmbedderEvent>,
    ) {
        for event in self.event_queue.borrow_mut().drain(..) {
            match event {
                MinibrowserEvent::Go => {
                    let browser_id = browser.browser_id().unwrap();
                    let location = self.location.borrow();
                    if let Some(url) = location_bar_input_to_url(&location.clone()) {
                        app_event_queue.push(EmbedderEvent::LoadUrl(browser_id, url));
                    } else {
                        warn!("failed to parse location");
                        break;
                    }
                },
            }
        }
    }

    /// Updates the location field from the given [Browser], unless the user has started editing it
    /// without clicking Go, returning true iff the location has changed (needing an egui update).
    pub fn update_location_in_toolbar(
        &mut self,
        browser: &mut BrowserManager<dyn WindowPortsMethods>,
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
