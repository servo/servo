/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use dpi::PhysicalSize;
use egui::text::{CCursor, CCursorRange};
use egui::text_edit::TextEditState;
use egui::{Button, Key, Label, LayerId, Modifiers, PaintCallback, TopBottomPanel, Vec2, pos2};
use egui_glow::{CallbackFn, EguiGlow};
use egui_winit::EventResponse;
use euclid::{Box2D, Length, Point2D, Rect, Scale, Size2D};
use log::{trace, warn};
use servo::base::id::WebViewId;
use servo::servo_geometry::DeviceIndependentPixel;
use servo::servo_url::ServoUrl;
use servo::webrender_api::units::DevicePixel;
use servo::{
    Image, LoadStatus, OffscreenRenderingContext, PixelFormat, PrefValue, RenderingContext, WebView,
};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use super::app_state::RunningAppState;
use super::events_loop::EventLoopProxy;
use super::geometry::winit_position_to_euclid_point;
use super::headed_window::Window as ServoWindow;
use crate::desktop::headed_window::INITIAL_WINDOW_TITLE;
use crate::desktop::window_trait::WindowPortsMethods;
use crate::prefs::{EXPERIMENTAL_PREFS, ServoShellPreferences};

pub struct Minibrowser {
    rendering_context: Rc<OffscreenRenderingContext>,
    context: EguiGlow,
    event_queue: Vec<MinibrowserEvent>,
    toolbar_height: Length<f32, DeviceIndependentPixel>,

    last_update: Instant,
    last_mouse_position: Option<Point2D<f32, DeviceIndependentPixel>>,
    location: String,

    /// Whether the location has been edited by the user without clicking Go.
    location_dirty: bool,

    load_status: LoadStatus,

    status_text: Option<String>,

    /// Handle to the GPU texture of the favicon.
    ///
    /// These need to be cached across egui draw calls.
    favicon_textures: HashMap<WebViewId, (egui::TextureHandle, egui::load::SizedTexture)>,

    /// Whether the user has enabled experimental preferences.
    experimental_prefs_enabled: bool,
}

pub enum MinibrowserEvent {
    /// Go button clicked.
    Go(String),
    Back,
    Forward,
    Reload,
    ReloadAll,
    NewWebView,
    CloseWebView(WebViewId),
}

fn truncate_with_ellipsis(input: &str, max_length: usize) -> String {
    if input.chars().count() > max_length {
        let truncated: String = input.chars().take(max_length.saturating_sub(1)).collect();
        format!("{}…", truncated)
    } else {
        input.to_string()
    }
}

impl Drop for Minibrowser {
    fn drop(&mut self) {
        self.context.destroy();
    }
}

impl Minibrowser {
    pub fn new(
        window: &ServoWindow,
        event_loop: &ActiveEventLoop,
        event_loop_proxy: EventLoopProxy,
        initial_url: ServoUrl,
        preferences: &ServoShellPreferences,
    ) -> Self {
        let rendering_context = window.offscreen_rendering_context();
        #[allow(clippy::arc_with_non_send_sync)]
        let mut context = EguiGlow::new(
            event_loop,
            rendering_context.glow_gl_api(),
            None,
            None,
            false,
        );

        let winit_window = window.winit_window().unwrap();
        context
            .egui_winit
            .init_accesskit(event_loop, winit_window, event_loop_proxy);
        winit_window.set_visible(true);

        context.egui_ctx.options_mut(|options| {
            // Disable the builtin egui handlers for the Ctrl+Plus, Ctrl+Minus and Ctrl+0
            // shortcuts as they don't work well with servoshell's `device-pixel-ratio` CLI argument.
            options.zoom_with_keyboard = false;

            // On platforms where winit fails to obtain a system theme, fall back to a light theme
            // since it is the more common default.
            options.fallback_theme = egui::Theme::Light;
        });

        Self {
            rendering_context,
            context,
            event_queue: vec![],
            toolbar_height: Default::default(),
            last_update: Instant::now(),
            last_mouse_position: None,
            location: initial_url.to_string(),
            location_dirty: false,
            load_status: LoadStatus::Complete,
            status_text: None,
            favicon_textures: Default::default(),
            experimental_prefs_enabled: preferences.experimental_prefs_enabled,
        }
    }

    pub(crate) fn take_events(&mut self) -> Vec<MinibrowserEvent> {
        std::mem::take(&mut self.event_queue)
    }

    /// Preprocess the given [winit::event::WindowEvent], returning unconsumed for mouse events in
    /// the Servo browser rect. This is needed because the CentralPanel we create for our webview
    /// would otherwise make egui report events in that area as consumed.
    pub fn on_window_event(
        &mut self,
        window: &Window,
        app_state: &RunningAppState,
        event: &WindowEvent,
    ) -> EventResponse {
        let mut result = self.context.on_window_event(window, event);

        if app_state.has_active_dialog() {
            result.consumed = true;
            return result;
        }

        result.consumed &= match event {
            WindowEvent::CursorMoved { position, .. } => {
                let scale = Scale::<_, DeviceIndependentPixel, _>::new(
                    self.context.egui_ctx.pixels_per_point(),
                );
                self.last_mouse_position =
                    Some(winit_position_to_euclid_point(*position).to_f32() / scale);
                self.last_mouse_position
                    .is_some_and(|p| self.is_in_egui_toolbar_rect(p))
            },
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Forward,
                ..
            } => {
                self.event_queue.push(MinibrowserEvent::Forward);
                true
            },
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Back,
                ..
            } => {
                self.event_queue.push(MinibrowserEvent::Back);
                true
            },
            WindowEvent::MouseWheel { .. } | WindowEvent::MouseInput { .. } => self
                .last_mouse_position
                .is_some_and(|p| self.is_in_egui_toolbar_rect(p)),
            _ => true,
        };
        result
    }

    /// Return true iff the given position is over the egui toolbar.
    fn is_in_egui_toolbar_rect(&self, position: Point2D<f32, DeviceIndependentPixel>) -> bool {
        position.y < self.toolbar_height.get()
    }

    /// Create a frameless button with square sizing, as used in the toolbar.
    fn toolbar_button(text: &str) -> egui::Button<'_> {
        egui::Button::new(text)
            .frame(false)
            .min_size(Vec2 { x: 20.0, y: 20.0 })
    }

    /// Draws a browser tab, checking for clicks and queues appropriate `MinibrowserEvent`s.
    /// Using a custom widget here would've been nice, but it doesn't seem as though egui
    /// supports that, so we arrange multiple Widgets in a way that they look connected.
    fn browser_tab(
        ui: &mut egui::Ui,
        webview: WebView,
        event_queue: &mut Vec<MinibrowserEvent>,
        favicon_texture: Option<egui::load::SizedTexture>,
    ) {
        let label = match (webview.page_title(), webview.url()) {
            (Some(title), _) if !title.is_empty() => title,
            (_, Some(url)) => url.to_string(),
            _ => "New Tab".into(),
        };

        let inactive_bg_color = ui.visuals().window_fill;
        let active_bg_color = ui.visuals().widgets.active.weak_bg_fill;
        let selected = webview.focused();

        // Setup a tab frame that will contain the favicon, title and close button
        let mut tab_frame = egui::Frame::NONE.corner_radius(4).begin(ui);
        {
            tab_frame.content_ui.add_space(5.0);

            let visuals = tab_frame.content_ui.visuals_mut();
            // Remove the stroke so we don't see the border between the close button and the label
            visuals.widgets.active.bg_stroke.width = 0.0;
            visuals.widgets.hovered.bg_stroke.width = 0.0;
            // Now we make sure the fill color is always the same, irrespective of state, that way
            // we can make sure that both the label and close button have the same background color
            visuals.widgets.noninteractive.weak_bg_fill = inactive_bg_color;
            visuals.widgets.inactive.weak_bg_fill = inactive_bg_color;
            visuals.widgets.hovered.weak_bg_fill = active_bg_color;
            visuals.widgets.active.weak_bg_fill = active_bg_color;
            visuals.selection.bg_fill = active_bg_color;
            visuals.selection.stroke.color = visuals.widgets.active.fg_stroke.color;
            visuals.widgets.hovered.fg_stroke.color = visuals.widgets.active.fg_stroke.color;

            // Expansion would also show that they are 2 separate widgets
            visuals.widgets.active.expansion = 0.0;
            visuals.widgets.hovered.expansion = 0.0;

            if let Some(favicon) = favicon_texture {
                tab_frame.content_ui.add(
                    egui::Image::from_texture(favicon)
                        .fit_to_exact_size(egui::vec2(16.0, 16.0))
                        .bg_fill(egui::Color32::TRANSPARENT),
                );
            }

            let tab = tab_frame
                .content_ui
                .add(Button::selectable(
                    selected,
                    truncate_with_ellipsis(&label, 20),
                ))
                .on_hover_ui(|ui| {
                    ui.label(&label);
                });

            let close_button = tab_frame
                .content_ui
                .add(egui::Button::new("X").fill(egui::Color32::TRANSPARENT));
            if close_button.clicked() || close_button.middle_clicked() || tab.middle_clicked() {
                event_queue.push(MinibrowserEvent::CloseWebView(webview.id()))
            } else if !selected && tab.clicked() {
                webview.focus();
            }
        }

        let response = tab_frame.allocate_space(ui);
        let fill_color = if selected || response.hovered() {
            active_bg_color
        } else {
            inactive_bg_color
        };
        tab_frame.frame.fill = fill_color;
        tab_frame.end(ui);
    }

    /// Update the minibrowser, but don’t paint.
    /// If `servo_framebuffer_id` is given, set up a paint callback to blit its contents to our
    /// CentralPanel when [`Minibrowser::paint`] is called.
    pub fn update(
        &mut self,
        window: &dyn WindowPortsMethods,
        state: &RunningAppState,
        reason: &'static str,
    ) {
        let now = Instant::now();
        let winit_window = window.winit_window().unwrap();
        trace!(
            "{:?} since last update ({})",
            now - self.last_update,
            reason
        );
        let Self {
            rendering_context,
            context,
            event_queue,
            toolbar_height,
            last_update,
            location,
            location_dirty,
            favicon_textures,
            ..
        } = self;

        context.run(winit_window, |ctx| {
            load_pending_favicons(ctx, state, favicon_textures);

            // TODO: While in fullscreen add some way to mitigate the increased phishing risk
            // when not displaying the URL bar: https://github.com/servo/servo/issues/32443
            if winit_window.fullscreen().is_none() {
                let frame = egui::Frame::default()
                    .fill(ctx.style().visuals.window_fill)
                    .inner_margin(4.0);
                TopBottomPanel::top("toolbar").frame(frame).show(ctx, |ui| {
                    ui.allocate_ui_with_layout(
                        ui.available_size(),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            if ui.add(Minibrowser::toolbar_button("⏴")).clicked() {
                                event_queue.push(MinibrowserEvent::Back);
                            }
                            if ui.add(Minibrowser::toolbar_button("⏵")).clicked() {
                                event_queue.push(MinibrowserEvent::Forward);
                            }

                            match self.load_status {
                                LoadStatus::Started | LoadStatus::HeadParsed => {
                                    if ui.add(Minibrowser::toolbar_button("X")).clicked() {
                                        warn!("Do not support stop yet.");
                                    }
                                },
                                LoadStatus::Complete => {
                                    if ui.add(Minibrowser::toolbar_button("↻")).clicked() {
                                        event_queue.push(MinibrowserEvent::Reload);
                                    }
                                },
                            }
                            ui.add_space(2.0);

                            ui.allocate_ui_with_layout(
                                ui.available_size(),
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let prefs_toggle = ui
                                        .toggle_value(&mut self.experimental_prefs_enabled, "☢")
                                        .on_hover_text("Enable experimental prefs");
                                    if prefs_toggle.clicked() {
                                        let enable = self.experimental_prefs_enabled;
                                        for pref in EXPERIMENTAL_PREFS {
                                            state
                                                .servo()
                                                .set_preference(pref, PrefValue::Bool(enable));
                                        }
                                        event_queue.push(MinibrowserEvent::ReloadAll);
                                    }

                                    let location_id = egui::Id::new("location_input");
                                    let location_field = ui.add_sized(
                                        ui.available_size(),
                                        egui::TextEdit::singleline(location).id(location_id),
                                    );

                                    if location_field.changed() {
                                        *location_dirty = true;
                                    }
                                    // Handle adddress bar shortcut.
                                    if ui.input(|i| {
                                        if cfg!(target_os = "macos") {
                                            i.clone().consume_key(Modifiers::COMMAND, Key::L)
                                        } else {
                                            i.clone().consume_key(Modifiers::COMMAND, Key::L) ||
                                                i.clone().consume_key(Modifiers::ALT, Key::D)
                                        }
                                    }) {
                                        // The focus request immediately makes gained_focus return true.
                                        location_field.request_focus();
                                    }
                                    // Select address bar text when it's focused (click or shortcut).
                                    if location_field.gained_focus() {
                                        if let Some(mut state) =
                                            TextEditState::load(ui.ctx(), location_id)
                                        {
                                            // Select the whole input.
                                            state.cursor.set_char_range(Some(CCursorRange::two(
                                                CCursor::new(0),
                                                CCursor::new(location.len()),
                                            )));
                                            state.store(ui.ctx(), location_id);
                                        }
                                    }
                                    // Navigate to address when enter is pressed in the address bar.
                                    if location_field.lost_focus() &&
                                        ui.input(|i| i.clone().key_pressed(Key::Enter))
                                    {
                                        event_queue.push(MinibrowserEvent::Go(location.clone()));
                                    }
                                },
                            );
                        },
                    );
                });
            };

            // A simple Tab header strip
            TopBottomPanel::top("tabs").show(ctx, |ui| {
                ui.allocate_ui_with_layout(
                    ui.available_size(),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        for (id, webview) in state.webviews().into_iter() {
                            let favicon = favicon_textures
                                .get(&id)
                                .map(|(_, favicon)| favicon)
                                .copied();
                            Self::browser_tab(ui, webview, event_queue, favicon);
                        }
                        if ui.add(Minibrowser::toolbar_button("+")).clicked() {
                            event_queue.push(MinibrowserEvent::NewWebView);
                        }
                    },
                );
            });

            // The toolbar height is where the Context’s available rect starts.
            // For reasons that are unclear, the TopBottomPanel’s ui cursor exceeds this by one egui
            // point, but the Context is correct and the TopBottomPanel is wrong.
            *toolbar_height = Length::new(ctx.available_rect().min.y);
            window.set_toolbar_height(*toolbar_height);

            let scale =
                Scale::<_, DeviceIndependentPixel, DevicePixel>::new(ctx.pixels_per_point());

            state.for_each_active_dialog(|dialog| dialog.update(ctx));

            let Some(webview) = state.focused_webview() else {
                return;
            };

            // If the top parts of the GUI changed size, then update the size of the WebView and also
            // the size of its RenderingContext.
            let rect = ctx.available_rect();
            let size = Size2D::new(rect.width(), rect.height()) * scale;
            let rect = Box2D::from_origin_and_size(Point2D::origin(), size);
            if rect != webview.rect() {
                webview.move_resize(rect);
                // `rect` is sized to just the WebView viewport, which is required by
                // `OffscreenRenderingContext` See:
                // <https://github.com/servo/servo/issues/38369#issuecomment-3138378527>
                webview.resize(PhysicalSize::new(size.width as u32, size.height as u32))
            }

            if let Some(status_text) = &self.status_text {
                egui::Tooltip::always_open(
                    ctx.clone(),
                    LayerId::background(),
                    "tooltip layer".into(),
                    pos2(0.0, ctx.available_rect().max.y),
                )
                .show(|ui| ui.add(Label::new(status_text.clone()).extend()));
            }

            state.repaint_servo_if_necessary();

            if let Some(render_to_parent) = rendering_context.render_to_parent_callback() {
                ctx.layer_painter(LayerId::background()).add(PaintCallback {
                    rect: ctx.available_rect(),
                    callback: Arc::new(CallbackFn::new(move |info, painter| {
                        let clip = info.viewport_in_pixels();
                        let rect_in_parent = Rect::new(
                            Point2D::new(clip.left_px, clip.from_bottom_px),
                            Size2D::new(clip.width_px, clip.height_px),
                        );
                        render_to_parent(painter.gl(), rect_in_parent)
                    })),
                });
            }

            *last_update = now;
        });
    }

    /// Paint the minibrowser, as of the last update.
    pub fn paint(&mut self, window: &Window) {
        self.rendering_context
            .parent_context()
            .prepare_for_rendering();
        self.context.paint(window);
        self.rendering_context.parent_context().present();
    }

    /// Updates the location field from the given [WebViewManager], unless the user has started
    /// editing it without clicking Go, returning true iff it has changed (needing an egui update).
    pub fn update_location_in_toolbar(&mut self, state: &RunningAppState) -> bool {
        // User edited without clicking Go?
        if self.location_dirty {
            return false;
        }

        let current_url_string = state
            .focused_webview()
            .and_then(|webview| Some(webview.url()?.to_string()));
        match current_url_string {
            Some(location) if location != self.location => {
                self.location = location.to_owned();
                true
            },
            _ => false,
        }
    }

    pub fn update_location_dirty(&mut self, dirty: bool) {
        self.location_dirty = dirty;
    }

    pub fn update_load_status(&mut self, state: &RunningAppState) -> bool {
        let state_status = state
            .focused_webview()
            .map(|webview| webview.load_status())
            .unwrap_or(LoadStatus::Complete);
        let old_status = std::mem::replace(&mut self.load_status, state_status);
        old_status != self.load_status
    }

    pub fn update_status_text(&mut self, state: &RunningAppState) -> bool {
        let state_status = state
            .focused_webview()
            .and_then(|webview| webview.status_text());
        let old_status = std::mem::replace(&mut self.status_text, state_status);
        old_status != self.status_text
    }

    fn update_title(
        &mut self,
        state: &RunningAppState,
        window: Rc<dyn WindowPortsMethods>,
    ) -> bool {
        if let Some(webview) = state.focused_webview() {
            let title = webview
                .page_title()
                .filter(|title| !title.is_empty())
                .map(|title| title.to_string())
                .or_else(|| webview.url().map(|url| url.to_string()))
                .unwrap_or_else(|| INITIAL_WINDOW_TITLE.to_string());

            window.set_title_if_changed(&title)
        } else {
            false
        }
    }

    /// Updates all fields taken from the given [WebViewManager], such as the location field.
    /// Returns true iff the egui needs an update.
    pub fn update_webview_data(
        &mut self,
        state: &RunningAppState,
        window: Rc<dyn WindowPortsMethods>,
    ) -> bool {
        // Note: We must use the "bitwise OR" (|) operator here instead of "logical OR" (||)
        //       because logical OR would short-circuit if any of the functions return true.
        //       We want to ensure that all functions are called. The "bitwise OR" operator
        //       does not short-circuit.
        self.update_location_in_toolbar(state) |
            self.update_load_status(state) |
            self.update_status_text(state) |
            self.update_title(state, window)
    }

    /// Returns true if a redraw is required after handling the provided event.
    pub(crate) fn handle_accesskit_event(
        &mut self,
        event: &egui_winit::accesskit_winit::WindowEvent,
    ) -> bool {
        match event {
            egui_winit::accesskit_winit::WindowEvent::InitialTreeRequested => {
                self.context.egui_ctx.enable_accesskit();
                true
            },
            egui_winit::accesskit_winit::WindowEvent::ActionRequested(req) => {
                self.context
                    .egui_winit
                    .on_accesskit_action_request(req.clone());
                true
            },
            egui_winit::accesskit_winit::WindowEvent::AccessibilityDeactivated => {
                self.context.egui_ctx.disable_accesskit();
                false
            },
        }
    }

    pub(crate) fn set_zoom_factor(&self, factor: f32) {
        self.context.egui_ctx.set_zoom_factor(factor);
    }
}

fn embedder_image_to_egui_image(image: &Image) -> egui::ColorImage {
    let width = image.width as usize;
    let height = image.height as usize;

    match image.format {
        PixelFormat::K8 => egui::ColorImage::from_gray([width, height], image.data()),
        PixelFormat::KA8 => {
            // Convert to rgba
            let data: Vec<u8> = image
                .data()
                .chunks_exact(2)
                .flat_map(|pixel| [pixel[0], pixel[0], pixel[0], pixel[1]])
                .collect();
            egui::ColorImage::from_rgba_unmultiplied([width, height], &data)
        },
        PixelFormat::RGB8 => egui::ColorImage::from_rgb([width, height], image.data()),
        PixelFormat::RGBA8 => {
            egui::ColorImage::from_rgba_unmultiplied([width, height], image.data())
        },
        PixelFormat::BGRA8 => {
            // Convert from BGRA to RGBA
            let data: Vec<u8> = image
                .data()
                .chunks_exact(4)
                .flat_map(|chunk| [chunk[2], chunk[1], chunk[0], chunk[3]])
                .collect();
            egui::ColorImage::from_rgba_unmultiplied([width, height], &data)
        },
    }
}

/// Uploads all favicons that have not yet been processed to the GPU.
fn load_pending_favicons(
    ctx: &egui::Context,
    state: &RunningAppState,
    texture_cache: &mut HashMap<WebViewId, (egui::TextureHandle, egui::load::SizedTexture)>,
) {
    for id in state.take_pending_favicon_loads() {
        let Some(webview) = state.webview_by_id(id) else {
            continue;
        };
        let Some(favicon) = webview.favicon() else {
            continue;
        };

        let egui_image = embedder_image_to_egui_image(&favicon);
        let handle = ctx.load_texture(format!("favicon-{id:?}"), egui_image, Default::default());
        let texture = egui::load::SizedTexture::new(
            handle.id(),
            egui::vec2(favicon.width as f32, favicon.height as f32),
        );

        // We don't need the handle anymore but we can't drop it either since that would cause
        // the texture to be freed.
        texture_cache.insert(id, (handle, texture));
    }
}
