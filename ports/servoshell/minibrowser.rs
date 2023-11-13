/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::sync::Arc;
use std::time::Instant;

use egui::{Key, Modifiers, TopBottomPanel};
use euclid::Length;
use log::{trace, warn};
use servo::compositing::windowing::EmbedderEvent;
use servo::servo_geometry::DeviceIndependentPixel;
use servo::webrender_surfman::WebrenderSurfman;

use crate::browser::Browser;
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
    pub fn update(&mut self, window: &winit::window::Window, reason: &'static str) {
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
            TopBottomPanel::top("toolbar").show(ctx, |ui| {
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
            });

            toolbar_height.set(Length::new(ctx.used_rect().height()));
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
        browser: &Browser<dyn WindowPortsMethods>,
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
        browser: &mut Browser<dyn WindowPortsMethods>,
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
