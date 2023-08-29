/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

 use std::{cell::{RefCell, Cell}, sync::Arc};

use egui::{TopBottomPanel, Modifiers, Key};
use servo::{servo_url::ServoUrl, compositing::windowing::EmbedderEvent};
use servo::webrender_surfman::WebrenderSurfman;

use crate::{egui_glue::EguiGlow, events_loop::EventsLoop, browser::Browser, window_trait::WindowPortsMethods};

pub struct Minibrowser {
    pub context: EguiGlow,
    pub event_queue: RefCell<Vec<MinibrowserEvent>>,
    pub toolbar_height: Cell<f32>,
    location: RefCell<String>,

    /// Whether the location has been edited by the user without clicking Go.
    location_dirty: Cell<bool>,
}

pub enum MinibrowserEvent {
    /// Go button clicked.
    Go,
}

impl Minibrowser {
    pub fn new(webrender_surfman: &WebrenderSurfman, events_loop: &EventsLoop) -> Self {
        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                webrender_surfman.get_proc_address(s)
            })
        };

        Self {
            context: EguiGlow::new(events_loop.as_winit(), Arc::new(gl), None),
            event_queue: RefCell::new(vec![]),
            toolbar_height: 0f32.into(),
            location: RefCell::new(String::default()),
            location_dirty: false.into(),
        }
    }

    /// Update the minibrowser, but don’t paint.
    pub fn update(&mut self, window: &winit::window::Window) {
        let Self { context, event_queue, location, location_dirty, toolbar_height } = self;
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
                        if location_field.lost_focus() && ui.input(|i| i.clone().key_pressed(Key::Enter)) {
                            event_queue.borrow_mut().push(MinibrowserEvent::Go);
                            location_dirty.set(false);
                        }
                    },
                );
            });

            toolbar_height.set(ctx.used_rect().height());
        });
    }

    /// Paint the minibrowser, as of the last update.
    pub fn paint(&mut self, window: &winit::window::Window) {
        self.context.paint(window);
    }

    /// Takes any outstanding events from the [Minibrowser], converting them to [EmbedderEvent] and
    /// routing those to the App event queue.
    pub fn queue_embedder_events_for_minibrowser_events(
        &self, browser: &Browser<dyn WindowPortsMethods>,
        app_event_queue: &mut Vec<EmbedderEvent>,
    ) {
        for event in self.event_queue.borrow_mut().drain(..) {
            match event {
                MinibrowserEvent::Go => {
                    let browser_id = browser.browser_id().unwrap();
                    let location = self.location.borrow();
                    let Ok(url) = ServoUrl::parse(&location) else {
                        warn!("failed to parse location");
                        break;
                    };
                    app_event_queue.push(EmbedderEvent::LoadUrl(browser_id, url));
                },
            }
        }
    }

    /// Updates the location field when the [Browser] says it has changed, unless the user has
    /// started editing it without clicking Go.
    pub fn update_location_in_toolbar(&mut self, browser: &mut Browser<dyn WindowPortsMethods>) -> bool {
        if !self.location_dirty.get() {
            if let Some(location) = browser.current_url_string() {
                self.location = RefCell::new(location.to_owned());
                return true;
            }
        }

        false
    }
}
