/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "freebsd"))]
use std::fs;
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "freebsd"))]
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use dpi::PhysicalSize;
use egui::text::{CCursor, CCursorRange};
use egui::text_edit::TextEditState;
use egui::{
    Button, FontDefinitions, Id, Key, Label, LayerId, Modifiers, Order, PaintCallback, Panel, RichText,
    ScrollArea, Vec2, WidgetInfo, WidgetType, pos2,
};
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "freebsd"))]
use egui::{FontData, FontFamily};
use egui_glow::{CallbackFn, EguiGlow};
use egui_winit::EventResponse;
use euclid::{Length, Point2D, Rect, Scale, Size2D};
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "freebsd"))]
use log::info;
use log::warn;
use servo::{
    DeviceIndependentPixel, DevicePixel, Image, LoadStatus, OffscreenRenderingContext, PixelFormat,
    RenderingContext, WebView, WebViewId,
};
use url::Url;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::Window;

use crate::desktop::event_loop::AppEvent;
use crate::desktop::headed_window;
use crate::running_app_state::{RunningAppState, UserInterfaceCommand};
use crate::window::RingtailWindow;

/// Global buffer for console log messages
pub static CONSOLE_LOGS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Global cache of approved domains for lock_approved.svg
pub static APPROVED_DOMAINS: Mutex<Option<Vec<String>>> = Mutex::new(None);
pub static APPROVED_DOMAINS_LOADED: AtomicBool = AtomicBool::new(false);

/// Fetch approved domains from remote JSON
pub async fn fetch_approved_domains() {
    if APPROVED_DOMAINS_LOADED.load(Ordering::Relaxed) {
        return;
    }

    match reqwest::get("https://voxelite.neocities.org/ringtail/dns/approved.json").await {
        Ok(response) => {
            let text = match response.text().await {
                Ok(t) => {
                    t
                },
                Err(e) => {
                    warn!("Failed to read response text: {}", e);
                    return;
                },
            };


            if let Ok(domains) = serde_json::from_str::<Vec<String>>(&text) {
                *APPROVED_DOMAINS.lock().unwrap() = Some(domains);
                APPROVED_DOMAINS_LOADED.store(true, Ordering::Relaxed);
                return;
            }
            
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(domains) = data.get("domains").and_then(|d| d.as_array()) {
                    let domain_strings: Vec<String> = domains
                        .iter()
                        .filter_map(|d| d.as_str().map(|s| s.to_string()))
                        .collect();
                    *APPROVED_DOMAINS.lock().unwrap() = Some(domain_strings);
                    APPROVED_DOMAINS_LOADED.store(true, Ordering::Relaxed);
                    return;
                }

                if let Some(obj) = data.as_object() {
                    let domain_strings: Vec<String> = obj.keys().map(|k| k.clone()).collect();
                    *APPROVED_DOMAINS.lock().unwrap() = Some(domain_strings);
                    APPROVED_DOMAINS_LOADED.store(true, Ordering::Relaxed);
                    return;
                }
            }

            if text.trim().starts_with('{') && text.trim().ends_with('}') {
                let mut domains = Vec::new();
                for line in text.lines() {
                    let line = line.trim();
                    if line.starts_with('"') && line.ends_with(',') {
                        let domain = line[1..line.len()-1].trim();
                        if !domain.is_empty() {
                            domains.push(domain.to_string());
                        }
                    } else if line.starts_with('"') && line.ends_with('"') {
                        let domain = &line[1..line.len()-1];
                        if !domain.is_empty() {
                            domains.push(domain.to_string());
                        }
                    }
                }
                if !domains.is_empty() {
                    *APPROVED_DOMAINS.lock().unwrap() = Some(domains);
                    APPROVED_DOMAINS_LOADED.store(true, Ordering::Relaxed);
                    return;
                }
            }

            warn!("Failed to parse approved domains JSON: unknown format");
        },
        Err(e) => {
            warn!("Failed to fetch approved domains: {}", e);
        },
    }
}

/// Check if a domain is in the approved list (supports wildcards like *.neocities.org)
fn is_domain_approved(domain: &str) -> bool {
    if let Some(approved) = APPROVED_DOMAINS.lock().unwrap().as_ref() {
        for pattern in approved {
            if pattern.starts_with("*.") {
                let suffix = &pattern[2..];
                if domain.ends_with(suffix) || domain == suffix {
                    return true;
                }
            } else if domain == pattern {
                return true;
            }
        }
    }
    false
}

/// Load an SVG icon from the resources directory
fn load_svg_icon(ctx: &egui::Context, filename: &str) -> Option<egui::TextureHandle> {
    let resources_dir = crate::resources::resource_protocol_dir_path();
    let icon_path = resources_dir.join(filename);
    
    if !icon_path.exists() {
        warn!("Icon file not found: {:?}", icon_path);
        return None;
    }

    match std::fs::read(&icon_path) {
        Ok(svg_data) => {
            // Use resvg to render the SVG to an image
            let opts = resvg::usvg::Options::default();
            let tree = resvg::usvg::Tree::from_data(&svg_data, &opts).ok()?;
            
            let size = tree.size();
            let width = size.width().ceil() as u32;
            let height = size.height().ceil() as u32;
            
            let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)?;
            pixmap.fill(resvg::tiny_skia::Color::TRANSPARENT);
            
            // Render the tree to the pixmap
            resvg::render(&tree, resvg::tiny_skia::Transform::identity(), &mut pixmap.as_mut());
            
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [width as usize, height as usize],
                pixmap.data(),
            );
            let handle = ctx.load_texture(
                format!("icon-{}", filename),
                color_image,
                Default::default(),
            );
            Some(handle)
        },
        Err(e) => {
            warn!("Failed to read icon file {:?}: {}", icon_path, e);
            None
        },
    }
}

/// The user interface of a headed servoshell. Currently this is implemented via
/// egui.
pub struct Gui {
    rendering_context: Rc<OffscreenRenderingContext>,
    context: EguiGlow,
    toolbar_height: Length<f32, DeviceIndependentPixel>,

    location: String,

    /// Whether the location has been edited by the user without clicking Go.
    location_dirty: bool,

    /// The [`LoadStatus`] of the active `WebView`.
    load_status: LoadStatus,

    /// The text to display in the status bar on the bottom of the window.
    status_text: Option<String>,

    /// Whether or not the current `WebView` can navigate backward.
    can_go_back: bool,

    /// Whether or not the current `WebView` can navigate forward.
    can_go_forward: bool,

    /// Handle to the GPU texture of the favicon.
    ///
    /// These need to be cached across egui draw calls.
    favicon_textures: HashMap<WebViewId, (egui::TextureHandle, egui::load::SizedTexture)>,

    /// AccessKit tree updates pending the next egui tick.
    /// This allows us to ensure that graft nodes are sent before the subtrees they graft.
    pending_accesskit_updates: Vec<accesskit::TreeUpdate>,

    /// Whether the console sidebar is visible.
    console_visible: bool,

    /// Whether the current URL is secure (HTTPS).
    is_secure: bool,

    /// Whether the current domain is approved (for lock_approved.svg).
    is_approved: bool,

    /// Texture handle for the lock icon.
    lock_icon: Option<egui::TextureHandle>,

    /// Texture handle for the unlock icon.
    unlock_icon: Option<egui::TextureHandle>,

    /// Texture handle for the lock approved icon.
    lock_approved_icon: Option<egui::TextureHandle>,

    /// Texture handle for the exp icon (experimental preferences enabled).
    exp_icon: Option<egui::TextureHandle>,

    /// Texture handle for the exp_off icon (experimental preferences disabled).
    exp_off_icon: Option<egui::TextureHandle>,
}

fn truncate_with_ellipsis(input: &str, max_length: usize) -> String {
    if input.chars().count() > max_length {
        let truncated: String = input.chars().take(max_length.saturating_sub(1)).collect();
        format!("{}…", truncated)
    } else {
        input.to_string()
    }
}

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "freebsd"))]
fn load_cjk_fonts(font_candidates: &[(&str, &str)]) -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let mut loaded_font_names = Vec::new();

    for (path_str, font_name) in font_candidates.iter() {
        let font_path = Path::new(path_str);
        if font_path.exists() {
            match fs::read(font_path) {
                Ok(bytes) => {
                    if !fonts.font_data.contains_key(*font_name) {
                        fonts
                            .font_data
                            .insert(font_name.to_string(), Arc::new(FontData::from_owned(bytes)));
                        loaded_font_names.push(font_name.to_string());
                        info!("Loaded font: {}", font_name);
                    }
                },
                Err(error) => {
                    info!("Failed to read font {}: {}", font_name, error);
                },
            }
        }
    }

    if !loaded_font_names.is_empty() {
        let proportional = fonts.families.get_mut(&FontFamily::Proportional).unwrap();
        for font_name in loaded_font_names.iter() {
            proportional.insert(0, font_name.clone());
        }
    }

    fonts
}

#[cfg(target_os = "windows")]
fn configure_fonts() -> FontDefinitions {
    load_cjk_fonts(&[
        (r"C:\Windows\Fonts\malgun.ttf", "Malgun Gothic"), // Korean
        (r"C:\Windows\Fonts\msyh.ttc", "Microsoft YaHei"), // Chinese + Japanese
    ])
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn configure_fonts() -> FontDefinitions {
    load_cjk_fonts(&[
        (
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "Noto Sans CJK",
        ), // Ubuntu/Debian
        (
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "Noto Sans CJK",
        ), // Fedora/Arch
        // FreeBSD splits the Noto CJK fonts into regional subsets
        (
            "/usr/local/share/fonts/noto/NotoSansCJKhk-Regular.otf",
            "Noto Sans CJK HK",
        ),
        (
            "/usr/local/share/fonts/noto/NotoSansCJKjp-Regular.otf",
            "Noto Sans CJK JP",
        ),
        (
            "/usr/local/share/fonts/noto/NotoSansCJKkr-Regular.otf",
            "Noto Sans CJK KR",
        ),
        (
            "/usr/local/share/fonts/noto/NotoSansCJKsc-Regular.otf",
            "Noto Sans CJK SC",
        ),
        (
            "/usr/local/share/fonts/noto/NotoSansCJKtc-Regular.otf",
            "Noto Sans CJK TC",
        ),
        (
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "WenQuanYi Micro Hei",
        ), // common fallback
        (
            "/usr/local/share/fonts/wqy/wqy-microhei.ttc",
            "WenQuanYi Micro Hei",
        ), // FreeBSD
    ])
}

#[cfg(target_os = "macos")]
fn configure_fonts() -> FontDefinitions {
    // TODO: Default proportional fonts: ["Ubuntu-Light", "NotoEmoji-Regular", "emoji-icon-font"]
    // does not support CJK. Add them for Mac.
    FontDefinitions::default()
}

impl Drop for Gui {
    fn drop(&mut self) {
        self.rendering_context
            .make_current()
            .expect("Could not make window RenderingContext current");
        self.context.destroy();
    }
}

impl Gui {
    pub(crate) fn new(
        winit_window: &Window,
        event_loop: &ActiveEventLoop,
        event_loop_proxy: EventLoopProxy<AppEvent>,
        rendering_context: Rc<OffscreenRenderingContext>,
        initial_url: Url,
    ) -> Self {
        rendering_context
            .make_current()
            .expect("Could not make window RenderingContext current");
        let mut context = EguiGlow::new(
            event_loop,
            rendering_context.glow_gl_api(),
            None,
            None,
            false,
        );

        let font_definitions = configure_fonts();
        context.egui_ctx.set_fonts(font_definitions);

        context
            .egui_winit
            .init_accesskit(event_loop, winit_window, event_loop_proxy);
        winit_window.set_visible(true);

        context.egui_ctx.options_mut(|options| {
            options.zoom_with_keyboard = false;
            options.fallback_theme = egui::Theme::Light;
        });

        // 1. Assign to a mutable variable first
        let mut gui = Self {
            rendering_context,
            context,
            toolbar_height: Default::default(),
            location: initial_url.to_string(),
            location_dirty: false,
            load_status: LoadStatus::Complete,
            status_text: None,
            can_go_back: false,
            can_go_forward: false,
            favicon_textures: Default::default(),
            pending_accesskit_updates: vec![],
            console_visible: false,
            is_secure: false,
            is_approved: false,
            lock_icon: None,
            unlock_icon: None,
            lock_approved_icon: None,
            exp_icon: None,
            exp_off_icon: None,
        };

        // 2. Call load_icons now that the egui context is fully prepared
        gui.load_icons();

        // 3. Return the populated gui instance
        gui
    }

    /// Load the lock/unlock icons (call this after the context is available)
    pub(crate) fn load_icons(&mut self) {
        let ctx = &self.context.egui_ctx;
        self.lock_icon = load_svg_icon(ctx, "lock.svg");
        self.unlock_icon = load_svg_icon(ctx, "unlock.svg");
        self.lock_approved_icon = load_svg_icon(ctx, "lock_approved.svg");
        self.exp_icon = load_svg_icon(ctx, "exp.svg");
        self.exp_off_icon = load_svg_icon(ctx, "exp_off.svg");
    }

    pub(crate) fn has_keyboard_focus(&self) -> bool {
        self.context
            .egui_ctx
            .memory(|memory| memory.focused().is_some())
    }

    pub(crate) fn surrender_focus(&self) {
        self.context.egui_ctx.memory_mut(|memory| {
            if let Some(focused) = memory.focused() {
                memory.surrender_focus(focused);
            }
        });
    }

    pub(crate) fn toggle_console(&mut self) {
        self.console_visible = !self.console_visible;
    }

    pub(crate) fn is_console_visible(&self) -> bool {
        self.console_visible
    }

    pub(crate) fn on_window_event(
        &mut self,
        winit_window: &Window,
        event: &WindowEvent,
    ) -> EventResponse {
        self.context.on_window_event(winit_window, event)
    }

    /// The height of the top toolbar of this user inteface ie the distance from the top of the
    /// window to the position of the `WebView`.
    pub(crate) fn toolbar_height(&self) -> Length<f32, DeviceIndependentPixel> {
        self.toolbar_height
    }

    /// Return true iff the given position is over the egui toolbar.
    pub(crate) fn is_in_egui_toolbar_rect(
        &self,
        position: Point2D<f32, DeviceIndependentPixel>,
    ) -> bool {
        position.y < self.toolbar_height.get()
    }

    /// Create a frameless button with square sizing, as used in the toolbar.
    fn toolbar_button(text: &str) -> egui::Button<'_> {
        egui::Button::new(text)
            .frame(false)
            .min_size(Vec2 { x: 20.0, y: 20.0 })
    }

    /// Draws a browser tab, checking for clicks and queues appropriate [`UserInterfaceCommand`]s.
    /// Using a custom widget here would've been nice, but it doesn't seem as though egui
    /// supports that, so we arrange multiple Widgets in a way that they look connected.
    fn browser_tab(
        ui: &mut egui::Ui,
        window: &RingtailWindow,
        webview: WebView,
        favicon_texture: Option<egui::load::SizedTexture>,
    ) {
        let label = match (webview.page_title(), webview.url()) {
            (Some(title), _) if !title.is_empty() => title,
            (_, Some(url)) => url.to_string(),
            _ => "New Tab".into(),
        };

        let inactive_bg_color = ui.visuals().window_fill;
        let active_bg_color = ui.visuals().widgets.active.weak_bg_fill;
        let active = window.active_webview().map(|webview| webview.id()) == Some(webview.id());

        // Setup a tab frame that will contain the favicon, title and close button
        let mut tab_frame = egui::Frame::NONE.corner_radius(4).begin(ui);
        {
            tab_frame.content_ui.add_space(5.0);

            let visuals = tab_frame.content_ui.visuals_mut();
            // Remove the stroke so we don't see the border between the close button and the label
            visuals.widgets.inactive.bg_stroke.width = 0.0;
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
            visuals.widgets.inactive.expansion = 0.0;
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
                    active,
                    truncate_with_ellipsis(&label, 20),
                ))
                .on_hover_ui(|ui| {
                    ui.label(&label);
                });

            let close_button = tab_frame
                .content_ui
                .add(egui::Button::new("×").fill(egui::Color32::TRANSPARENT));
            close_button.widget_info(|| {
                let mut info = WidgetInfo::new(WidgetType::Button);
                info.label = Some("Close".into());
                info
            });
            if close_button.clicked() || close_button.middle_clicked() || tab.middle_clicked() {
                window
                    .queue_user_interface_command(UserInterfaceCommand::CloseWebView(webview.id()));
            } else if !active && tab.clicked() {
                window.activate_webview(webview.id());
            }
        }

        let response = tab_frame.allocate_space(ui);
        let fill_color = if active || response.hovered() {
            active_bg_color
        } else {
            inactive_bg_color
        };
        tab_frame.frame.fill = fill_color;
        tab_frame.end(ui);
    }

    /// Update the user interface, but do not paint the updated state.
    pub(crate) fn update(
        &mut self,
        state: &RunningAppState,
        window: &RingtailWindow,
        headed_window: &headed_window::HeadedWindow,
    ) {
        self.rendering_context
            .make_current()
            .expect("Could not make RenderingContext current");
        let Self {
            rendering_context,
            context,
            toolbar_height,
            location,
            location_dirty,
            favicon_textures,
            ..
        } = self;

        let winit_window = headed_window.winit_window();
        context.run(winit_window, |ctx| {
            load_pending_favicons(ctx, window, favicon_textures);

            // TODO: While in fullscreen add some way to mitigate the increased phishing risk
            // when not displaying the URL bar: https://github.com/servo/servo/issues/32443
            if winit_window.fullscreen().is_none() {
                let frame = egui::Frame::default()
                    .fill(ctx.style().visuals.window_fill)
                    .inner_margin(4.0);
                Panel::top("toolbar").frame(frame).show_inside(ctx, |ui| {
                    ui.allocate_ui_with_layout(
                        ui.available_size(),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            let back_button =
                                ui.add_enabled(self.can_go_back, Gui::toolbar_button("⏴"));
                            back_button.widget_info(|| {
                                let mut info = WidgetInfo::new(WidgetType::Button);
                                info.label = Some("Back".into());
                                info
                            });
                            if back_button.clicked() {
                                *location_dirty = false;
                                window.queue_user_interface_command(UserInterfaceCommand::Back);
                            }

                            let forward_button =
                                ui.add_enabled(self.can_go_forward, Gui::toolbar_button("⏵"));
                            forward_button.widget_info(|| {
                                let mut info = WidgetInfo::new(WidgetType::Button);
                                info.label = Some("Forward".into());
                                info
                            });
                            if forward_button.clicked() {
                                *location_dirty = false;
                                window.queue_user_interface_command(UserInterfaceCommand::Forward);
                            }

                            match self.load_status {
                                LoadStatus::Started | LoadStatus::HeadParsed => {
                                    let stop_button = ui.add(Gui::toolbar_button("×"));
                                    stop_button.widget_info(|| {
                                        let mut info = WidgetInfo::new(WidgetType::Button);
                                        info.label = Some("Stop".into());
                                        info
                                    });
                                    if stop_button.clicked() {
                                        warn!("Do not support stop yet.");
                                    }
                                },
                                LoadStatus::Complete => {
                                    let reload_button = ui.add(Gui::toolbar_button("↻"));
                                    reload_button.widget_info(|| {
                                        let mut info = WidgetInfo::new(WidgetType::Button);
                                        info.label = Some("Reload".into());
                                        info
                                    });
                                    if reload_button.clicked() {
                                        *location_dirty = false;
                                        window.queue_user_interface_command(
                                            UserInterfaceCommand::Reload,
                                        );
                                    }
                                },
                            }

                            // Show lock icon next to refresh button
                            let url = window.active_webview().and_then(|webview| webview.url());
                            let scheme = url.as_ref().and_then(|u| Some(u.scheme()));

                            if let Some(icon) = if self.is_approved {
                                self.lock_approved_icon.as_ref()
                            } else if self.is_secure {
                                self.lock_icon.as_ref()
                            } else {
                                self.unlock_icon.as_ref()
                            } {
                                let hover_text = if let Some(scheme) = scheme {
                                    match scheme {
                                        "peanut" => "Your connection is secure.\nSensitive data is encrypted by design with E2EE, preventing man-in-the-middle attacks.\nThe server itself may still steal information, so double-check the server.",
                                        "ringtail" => "Your data is secure.\nThis is an internal browser page stored locally on your computer, so no server is contacted.",
                                        _ if self.is_approved => "Your connection is secure.\nSensitive data is encrypted preventing man-in-the-middle attacks.\nThe server itself has been verified to not steal data; so you can trust it.",
                                        _ if self.is_secure => "Your connection is secure.\nSensitive data is encrypted, preventing man-in-the-middle attacks.\nThe server itself may still steal information, so double-check the server.",
                                        _ => "Your connection is not encrypted.\nDo not enter sensitive data, as hackers may be able to intercept it; as well as the server itself.",
                                    }
                                } else {
                                    "Unknown connection type"
                                };
                                ui.add(
                                    egui::Image::from_texture(icon)
                                        .fit_to_exact_size(egui::vec2(16.0, 16.0))
                                        .bg_fill(egui::Color32::TRANSPARENT)
                                ).on_hover_text(hover_text);
                            }

                            ui.add_space(2.0);

                            ui.allocate_ui_with_layout(
                                ui.available_size(),
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let mut experimental_preferences_enabled =
                                        state.experimental_preferences_enabled();

                                    // Show exp or exp_off icon based on state
                                    let icon = if experimental_preferences_enabled {
                                        self.exp_icon.as_ref()
                                    } else {
                                        self.exp_off_icon.as_ref()
                                    };

                                    if let Some(icon) = icon {
                                        let image = egui::Image::from_texture(icon).fit_to_exact_size(egui::vec2(16.0, 16.0));
                                        let prefs_button = ui.add_sized(
                                            [16.0, ui.available_height()],
                                            egui::Button::image(image)
                                                .fill(egui::Color32::TRANSPARENT)
                                        ).on_hover_text("Enable experimental preferences");

                                        if prefs_button.clicked() {
                                            experimental_preferences_enabled = !experimental_preferences_enabled;
                                            state.set_experimental_preferences_enabled(
                                                experimental_preferences_enabled,
                                            );
                                            *location_dirty = false;
                                            window.queue_user_interface_command(
                                                UserInterfaceCommand::ReloadAll,
                                            );
                                        }
                                    } else {
                                        // Fallback to emoji if icons not loaded
                                        let prefs_toggle = ui
                                            .toggle_value(&mut experimental_preferences_enabled, "☢")
                                            .on_hover_text("Enable experimental preferences");
                                        prefs_toggle.widget_info(|| {
                                            let mut info = WidgetInfo::new(WidgetType::Button);
                                            info.label = Some("Enable experimental preferences".into());
                                            info.selected = Some(experimental_preferences_enabled);
                                            info
                                        });
                                        if prefs_toggle.clicked() {
                                            state.set_experimental_preferences_enabled(
                                                experimental_preferences_enabled,
                                            );
                                            *location_dirty = false;
                                            window.queue_user_interface_command(
                                                UserInterfaceCommand::ReloadAll,
                                            );
                                        }
                                    }

                                    let location_id = egui::Id::new("location_input");
                                    let location_field = ui.add_sized(
                                        ui.available_size(),
                                        egui::TextEdit::singleline(location)
                                            .id(location_id)
                                            .hint_text("Search or enter address"),
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
                                    if location_field.gained_focus() &&
                                        let Some(mut state) =
                                            TextEditState::load(ui.ctx(), location_id)
                                    {
                                        // Select the whole input.
                                        state.cursor.set_char_range(Some(CCursorRange::two(
                                            CCursor::new(0),
                                            CCursor::new(location.len()),
                                        )));
                                        state.store(ui.ctx(), location_id);
                                    }
                                    // Navigate to address when enter is pressed in the address bar.
                                    if location_field.lost_focus() &&
                                        ui.input(|i| i.clone().key_pressed(Key::Enter))
                                    {
                                        window.queue_user_interface_command(
                                            UserInterfaceCommand::Go(location.clone()),
                                        );
                                    }
                                },
                            );
                        },
                    );
                });

                // A simple Tab header strip
                let outer = Panel::top("tabs").show_inside(ctx, |ui| {
                    // Add scroll for overflowing tabs
                    egui::ScrollArea::horizontal()
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                        .show(ui, |ui| {
                            ui.allocate_ui_with_layout(
                                ui.available_size(),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    for (id, webview) in window.webviews().into_iter() {
                                        let favicon = favicon_textures
                                            .get(&id)
                                            .map(|(_, favicon)| favicon)
                                            .copied();
                                        Self::browser_tab(ui, window, webview, favicon);
                                    }

                                    let new_tab_button = ui.add(Gui::toolbar_button("+"));
                                    new_tab_button.widget_info(|| {
                                        let mut info = WidgetInfo::new(WidgetType::Button);
                                        info.label = Some("New tab".into());
                                        info
                                    });
                                    if new_tab_button.clicked() {
                                        window.queue_user_interface_command(
                                            UserInterfaceCommand::NewWebView,
                                        );
                                    }

                                    let new_window_button = ui.add(Gui::toolbar_button("⊞"));
                                    new_window_button.widget_info(|| {
                                        let mut info = WidgetInfo::new(WidgetType::Button);
                                        info.label = Some("New window".into());
                                        info
                                    });
                                    if new_window_button.clicked() {
                                        window.queue_user_interface_command(
                                            UserInterfaceCommand::NewWindow,
                                        );
                                    }
                                },
                            );
                        })
                });

                *toolbar_height = Length::new(outer.response.rect.max.y);
            } else {
                *toolbar_height = Length::default();
            }

            let scale =
                Scale::<_, DeviceIndependentPixel, DevicePixel>::new(ctx.pixels_per_point());

            headed_window.for_each_active_dialog(window, |dialog| dialog.update(ctx));

            // Show console panel on the right if visible
            // This must be drawn before the webview so the webview gets the remaining space
            if self.console_visible {
                let mut should_close = false;
                egui::Panel::right("console_panel")
                    .default_size(400.0)
                    .show_inside(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Console Output:");
                            if ui.button("Clear").clicked() {
                                if let Ok(mut logs) = CONSOLE_LOGS.lock() {
                                    logs.clear();
                                }
                            }
                            if ui.button("Close").clicked() {
                                should_close = true;
                            }
                        });
                        
                        ui.separator();

                        egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                            if let Ok(logs) = CONSOLE_LOGS.lock() {
                                if logs.is_empty() {
                                    ui.weak("No logs captured yet.");
                                } else {
                                    for log_line in logs.iter() {
                                        ui.label(egui::RichText::new(log_line).size(10.0).monospace());
                                    }
                                }
                            }
                        });
                    });
                if should_close {
                    self.console_visible = false;
                }
            }

            // If the top parts of the GUI changed size, then update the size of the WebView and also
            // the size of its RenderingContext.
            let available_rect = ctx.available_rect_before_wrap();

            // Build a graft node for each WebView.
            for (webview_id, webview) in window.webviews() {
                if let Some(tree_id) = webview.accesskit_tree_id() {
                    let id = egui::Id::new(webview_id);
                    ctx.accesskit_node_builder(id, |node| {
                        node.set_tree_id(tree_id);
                    });
                }
            }
            let size = Size2D::new(available_rect.width(), available_rect.height()) * scale;
            if let Some(webview) = window.active_webview() &&
                size != webview.size()
            {
                // `rect` is sized to just the WebView viewport, which is required by
                // `OffscreenRenderingContext` See:
                // <https://github.com/servo/servo/issues/38369#issuecomment-3138378527>
                webview.resize(PhysicalSize::new(size.width as u32, size.height as u32))
            }

            if let Some(status_text) = &self.status_text {
                egui::Tooltip::always_open(
                    ctx.clone(),
                    LayerId::new(Order::Tooltip, Id::new("tooltip")),
                    "tooltip layer".into(),
                    pos2(0.0, available_rect.max.y),
                )
                .show(|ui| ui.add(Label::new(status_text.clone()).extend()));
            }

            window.repaint_webviews();

            if let Some(render_to_parent) = rendering_context.render_to_parent_callback() {
                ctx.layer_painter(LayerId::background()).add(PaintCallback {
                    rect: available_rect,
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
        });

        // If any egui widget requested a repaint, also request a repaint for our
        // containing window. This allows egui widget to animate on their own.
        if self.context.egui_ctx.has_requested_repaint() {
            window.set_needs_repaint();
        }

        let adapter = self
            .context
            .egui_winit
            .accesskit
            .as_mut()
            .expect("guaranteed by Gui::new()");
        for tree_update in self.pending_accesskit_updates.drain(..) {
            adapter.update_if_active(|| tree_update);
        }
    }

    /// Paint the GUI, as of the last update.
    pub(crate) fn paint(&mut self, window: &Window) {
        self.rendering_context
            .make_current()
            .expect("Could not make RenderingContext current");
        self.rendering_context
            .parent_context()
            .prepare_for_rendering();
        self.context.paint(window);
        self.rendering_context.parent_context().present();
    }

    /// Updates the location field from the given [`RunningAppState`], unless the user has started
    /// editing it without clicking Go, returning true iff it has changed (needing an egui update).
    fn update_location_in_toolbar(&mut self, window: &RingtailWindow) -> bool {
        // User edited without clicking Go?
        if self.location_dirty {
            return false;
        }

        let current_url_string = window
            .active_webview()
            .and_then(|webview| Some(webview.url()?.to_string()));
        match current_url_string {
            Some(location) if location != self.location => {
                self.location = location;
                true
            },
            _ => false,
        }
    }

    fn update_load_status(&mut self, window: &RingtailWindow) -> bool {
        let state_status = window
            .active_webview()
            .map(|webview| webview.load_status())
            .unwrap_or(LoadStatus::Complete);
        let old_status = std::mem::replace(&mut self.load_status, state_status);
        let status_changed = old_status != self.load_status;

        // When the load status changes, we want the new changes to the URL to start
        // being reflected in the location bar.
        if status_changed {
            self.location_dirty = false;
        }

        status_changed
    }

    fn update_status_text(&mut self, window: &RingtailWindow) -> bool {
        let state_status = window
            .active_webview()
            .and_then(|webview| webview.status_text());
        let old_status = std::mem::replace(&mut self.status_text, state_status);
        old_status != self.status_text
    }

    fn update_can_go_back_and_forward(&mut self, window: &RingtailWindow) -> bool {
        let (can_go_back, can_go_forward) = window
            .active_webview()
            .map(|webview| (webview.can_go_back(), webview.can_go_forward()))
            .unwrap_or((false, false));
        let old_can_go_back = std::mem::replace(&mut self.can_go_back, can_go_back);
        let old_can_go_forward = std::mem::replace(&mut self.can_go_forward, can_go_forward);
        old_can_go_back != self.can_go_back || old_can_go_forward != self.can_go_forward
    }

    fn update_is_secure(&mut self, window: &RingtailWindow) -> bool {
        let url_opt = window.active_webview().and_then(|webview| webview.url());
        let (is_secure, is_approved) = if let Some(url) = url_opt {
            let scheme = url.scheme();
            // ringtail, peanut, and https are secure
            let is_secure = scheme == "https" || scheme == "ringtail" || scheme == "peanut";
            
            let is_approved = if let Some(host) = url.host_str() {
                is_domain_approved(host.trim_end_matches('.'))
            } else if url.host_str() == Some("neocities.org") || url.host_str() == Some("voxelite.neocities.org") {
                true
            } else {
                false
            };
            (is_secure, is_approved)
        } else {
            (false, false)
        };

        let old_is_secure = std::mem::replace(&mut self.is_secure, is_secure);
        let old_is_approved = std::mem::replace(&mut self.is_approved, is_approved);
        
        old_is_secure != self.is_secure || old_is_approved != self.is_approved
    }

    /// Updates all fields taken from the given [`RingtailWindow`], such as the location field.
    /// Returns true iff the egui needs an update.
    pub(crate) fn update_webview_data(&mut self, window: &RingtailWindow) -> bool {
        // Check if the background thread finished loading domains since the last frame
        let domains_loaded = APPROVED_DOMAINS_LOADED.load(Ordering::Relaxed);
        
        // If domains are loaded but the local state doesn't match yet
        self.update_load_status(window) |
            self.update_location_in_toolbar(window) |
            self.update_status_text(window) |
            self.update_can_go_back_and_forward(window) |
            self.update_is_secure(window) | 
            domains_loaded
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

    pub(crate) fn notify_accessibility_tree_update(&mut self, tree_update: accesskit::TreeUpdate) {
        self.pending_accesskit_updates.push(tree_update);
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
    window: &RingtailWindow,
    texture_cache: &mut HashMap<WebViewId, (egui::TextureHandle, egui::load::SizedTexture)>,
) {
    for id in window.take_pending_favicon_loads() {
        let Some(webview) = window.webview_by_id(id) else {
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
