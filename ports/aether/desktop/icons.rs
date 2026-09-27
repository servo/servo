/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! SVG toolbar icons. Add new buttons by dropping an SVG in `resources/icons/`
//! and a variant on [`ToolbarIcon`].

use std::collections::HashMap;

use egui::load::SizedTexture;
use log::warn;
use resvg::{tiny_skia, usvg};

/// Logical size of a toolbar icon, in egui points.
const TOOLBAR_ICON_SIZE: f32 = 14.0;

/// Toolbar button hit target.
const TOOLBAR_BUTTON_SIZE: f32 = 20.0;

/// Rounded square used to mark a toolbar button on hover and press.
const TOOLBAR_BUTTON_CORNER_RADIUS: u8 = 4;

/// Toolbar button: icon only at rest, rounded square on hover, darker on press.
pub(crate) fn add_toolbar_button(ui: &mut egui::Ui, button: egui::Button<'_>) -> egui::Response {
    // Reserve a paint slot behind the icon so hover/press fill never covers it.
    let background = ui.painter().add(egui::Shape::Noop);
    let response = ui.add(
        button
            .frame(false)
            .min_size(egui::vec2(TOOLBAR_BUTTON_SIZE, TOOLBAR_BUTTON_SIZE)),
    );
    if let Some(fill) = toolbar_button_fill_color(
        ui.visuals(),
        toolbar_button_visual_state(
            ui.is_enabled(),
            response.hovered(),
            response.is_pointer_button_down_on(),
        ),
    ) {
        ui.painter().set(
            background,
            egui::Shape::rect_filled(
                response.rect,
                egui::CornerRadius::same(TOOLBAR_BUTTON_CORNER_RADIUS),
                fill,
            ),
        );
    }
    response
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ToolbarButtonVisualState {
    Rest,
    Hover,
    Press,
}

fn toolbar_button_visual_state(
    enabled: bool,
    hovered: bool,
    pressed: bool,
) -> ToolbarButtonVisualState {
    if enabled && pressed {
        ToolbarButtonVisualState::Press
    } else if enabled && hovered {
        ToolbarButtonVisualState::Hover
    } else {
        ToolbarButtonVisualState::Rest
    }
}

fn toolbar_button_fill_color(
    visuals: &egui::Visuals,
    state: ToolbarButtonVisualState,
) -> Option<egui::Color32> {
    match state {
        ToolbarButtonVisualState::Rest => None,
        ToolbarButtonVisualState::Hover => Some(visuals.widgets.hovered.weak_bg_fill),
        ToolbarButtonVisualState::Press => Some(visuals.widgets.active.weak_bg_fill),
    }
}

/// A bundled SVG used by the chrome toolbar.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ToolbarIcon {
    Back,
    ChevronLeft,
    ChevronRight,
    Close,
    Forward,
    Home,
    Menu,
    Plus,
    Reload,
    Stop,
}

impl ToolbarIcon {
    fn svg_bytes(self) -> &'static [u8] {
        match self {
            Self::Back => include_bytes!("../../../resources/icons/back.svg"),
            Self::ChevronLeft => include_bytes!("../../../resources/icons/chevron-left.svg"),
            Self::ChevronRight => include_bytes!("../../../resources/icons/chevron-right.svg"),
            Self::Close => include_bytes!("../../../resources/icons/close.svg"),
            Self::Forward => include_bytes!("../../../resources/icons/forward.svg"),
            Self::Home => include_bytes!("../../../resources/icons/home.svg"),
            Self::Menu => include_bytes!("../../../resources/icons/menu.svg"),
            Self::Plus => include_bytes!("../../../resources/icons/plus.svg"),
            Self::Reload => include_bytes!("../../../resources/icons/reload.svg"),
            Self::Stop => include_bytes!("../../../resources/icons/stop-reload.svg"),
        }
    }

    fn texture_name(self) -> &'static str {
        match self {
            Self::Back => "toolbar-back",
            Self::ChevronLeft => "toolbar-chevron-left",
            Self::ChevronRight => "toolbar-chevron-right",
            Self::Close => "toolbar-close",
            Self::Forward => "toolbar-forward",
            Self::Home => "toolbar-home",
            Self::Menu => "toolbar-menu",
            Self::Plus => "toolbar-plus",
            Self::Reload => "toolbar-reload",
            Self::Stop => "toolbar-stop",
        }
    }
}

/// Rasterized SVG textures, cached by icon and physical pixel size.
#[derive(Default)]
pub(crate) struct ToolbarIconCache {
    textures: HashMap<(ToolbarIcon, u32), egui::TextureHandle>,
}

impl ToolbarIconCache {
    pub(crate) fn button(&mut self, ui: &mut egui::Ui, icon: ToolbarIcon) -> egui::Response {
        let button = self.image_button(ui, icon);
        add_toolbar_button(ui, button)
    }

    /// Icon button without the toolbar hover square, for chrome that paints its own background.
    pub(crate) fn image_button(&mut self, ui: &egui::Ui, icon: ToolbarIcon) -> egui::Button<'_> {
        let button = match self.image(ui, icon) {
            Some(image) => egui::Button::new(image),
            None => egui::Button::new(""),
        };
        button.image_tint_follows_text_color(true)
    }

    /// A tintable icon image for embedding in another widget, such as a menu row.
    pub(crate) fn image(
        &mut self,
        ui: &egui::Ui,
        icon: ToolbarIcon,
    ) -> Option<egui::Image<'static>> {
        let pixel_size = (TOOLBAR_ICON_SIZE * ui.pixels_per_point()).round().max(1.0) as u32;
        let handle = self.texture(ui.ctx(), icon, pixel_size)?;
        Some(
            egui::Image::from_texture(SizedTexture::new(
                handle.id(),
                egui::vec2(TOOLBAR_ICON_SIZE, TOOLBAR_ICON_SIZE),
            ))
            .fit_to_exact_size(egui::vec2(TOOLBAR_ICON_SIZE, TOOLBAR_ICON_SIZE)),
        )
    }

    fn texture(
        &mut self,
        ctx: &egui::Context,
        icon: ToolbarIcon,
        pixel_size: u32,
    ) -> Option<egui::TextureHandle> {
        if let Some(handle) = self.textures.get(&(icon, pixel_size)) {
            return Some(handle.clone());
        }

        let color_image = rasterize_svg(icon.svg_bytes(), pixel_size)?;
        let handle = ctx.load_texture(
            format!("{}@{pixel_size}", icon.texture_name()),
            color_image,
            egui::TextureOptions::LINEAR,
        );
        self.textures.insert((icon, pixel_size), handle.clone());
        Some(handle)
    }
}

/// SVGs use `context-fill` / `context-fill-opacity`, which resvg
/// treats as an invalid paint (fully transparent). Map them to an opaque white
/// fill so egui can tint the icon to the current text color.
fn prepare_toolbar_svg(svg: &[u8]) -> std::borrow::Cow<'_, [u8]> {
    let Ok(text) = std::str::from_utf8(svg) else {
        return std::borrow::Cow::Borrowed(svg);
    };
    if !text.contains("context-fill") {
        return std::borrow::Cow::Borrowed(svg);
    }
    std::borrow::Cow::Owned(
        text.replace("context-fill-opacity", "1")
            .replace("context-fill", "#ffffff")
            .into_bytes(),
    )
}

pub(crate) fn rasterize_svg(svg: &[u8], size_px: u32) -> Option<egui::ColorImage> {
    let svg = prepare_toolbar_svg(svg);
    let tree = usvg::Tree::from_data(&svg, &usvg::Options::default())
        .map_err(|error| {
            warn!("Failed to parse toolbar SVG: {error}");
            error
        })
        .ok()?;
    let mut pixmap = tiny_skia::Pixmap::new(size_px, size_px)?;
    let svg_size = tree.size();
    let transform = tiny_skia::Transform::from_scale(
        size_px as f32 / svg_size.width(),
        size_px as f32 / svg_size.height(),
    );
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    Some(egui::ColorImage::from_rgba_premultiplied(
        [size_px as usize, size_px as usize],
        pixmap.data(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn back_svg_rasterizes() {
        let image =
            rasterize_svg(ToolbarIcon::Back.svg_bytes(), 32).expect("back.svg should rasterize");
        assert_eq!(image.size, [32, 32]);
        assert!(
            image.pixels.iter().any(|pixel| pixel.a() > 0),
            "back icon should not be fully transparent"
        );
    }

    #[test]
    fn chevron_svgs_rasterize() {
        for icon in [ToolbarIcon::ChevronLeft, ToolbarIcon::ChevronRight] {
            let image = rasterize_svg(icon.svg_bytes(), 32).expect("chevron should rasterize");
            assert_eq!(image.size, [32, 32]);
            assert!(
                image.pixels.iter().any(|pixel| pixel.a() > 0),
                "chevron icon should not be fully transparent"
            );
        }
    }

    #[test]
    fn close_svg_rasterizes() {
        let image =
            rasterize_svg(ToolbarIcon::Close.svg_bytes(), 32).expect("close.svg should rasterize");
        assert_eq!(image.size, [32, 32]);
        assert!(
            image.pixels.iter().any(|pixel| pixel.a() > 0),
            "close icon should not be fully transparent"
        );
    }

    #[test]
    fn forward_svg_rasterizes() {
        let image = rasterize_svg(ToolbarIcon::Forward.svg_bytes(), 32)
            .expect("forward.svg should rasterize");
        assert_eq!(image.size, [32, 32]);
        assert!(
            image.pixels.iter().any(|pixel| pixel.a() > 0),
            "forward icon should not be fully transparent"
        );
    }

    #[test]
    fn home_svg_rasterizes() {
        let image =
            rasterize_svg(ToolbarIcon::Home.svg_bytes(), 32).expect("home.svg should rasterize");
        assert_eq!(image.size, [32, 32]);
        assert!(
            image.pixels.iter().any(|pixel| pixel.a() > 0),
            "home icon should not be fully transparent"
        );
    }

    #[test]
    fn menu_svg_rasterizes() {
        let image =
            rasterize_svg(ToolbarIcon::Menu.svg_bytes(), 32).expect("menu.svg should rasterize");
        assert_eq!(image.size, [32, 32]);
        assert!(
            image.pixels.iter().any(|pixel| pixel.a() > 0),
            "menu icon should not be fully transparent"
        );
    }

    #[test]
    fn plus_svg_rasterizes() {
        let image =
            rasterize_svg(ToolbarIcon::Plus.svg_bytes(), 32).expect("plus.svg should rasterize");
        assert_eq!(image.size, [32, 32]);
        assert!(
            image.pixels.iter().any(|pixel| pixel.a() > 0),
            "plus icon should not be fully transparent"
        );
    }

    #[test]
    fn reload_svg_rasterizes() {
        let image = rasterize_svg(ToolbarIcon::Reload.svg_bytes(), 32)
            .expect("reload.svg should rasterize");
        assert_eq!(image.size, [32, 32]);
        assert!(
            image.pixels.iter().any(|pixel| pixel.a() > 0),
            "reload icon should not be fully transparent"
        );
    }

    #[test]
    fn stop_svg_rasterizes() {
        let image = rasterize_svg(ToolbarIcon::Stop.svg_bytes(), 32)
            .expect("stop-reload.svg should rasterize");
        assert_eq!(image.size, [32, 32]);
        assert!(
            image.pixels.iter().any(|pixel| pixel.a() > 0),
            "stop icon should not be fully transparent"
        );
    }

    #[test]
    fn context_fill_svg_rasterizes() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" width="16" height="16" fill="context-fill" fill-opacity="context-fill-opacity"><rect width="16" height="16"/></svg>"#;
        let image = rasterize_svg(svg, 32).expect("context-fill SVG should rasterize");
        assert_eq!(image.size, [32, 32]);
        assert!(
            image.pixels.iter().any(|pixel| pixel.a() > 0),
            "context-fill should be treated as an opaque white fill"
        );
    }

    #[test]
    fn rest_state_has_no_fill() {
        assert_eq!(
            toolbar_button_visual_state(true, false, false),
            ToolbarButtonVisualState::Rest
        );
        assert_eq!(
            toolbar_button_fill_color(&egui::Visuals::dark(), ToolbarButtonVisualState::Rest),
            None
        );
    }

    #[test]
    fn hover_state_uses_rounded_square_fill() {
        assert_eq!(
            toolbar_button_visual_state(true, true, false),
            ToolbarButtonVisualState::Hover
        );
        let visuals = egui::Visuals::dark();
        assert_eq!(
            toolbar_button_fill_color(&visuals, ToolbarButtonVisualState::Hover),
            Some(visuals.widgets.hovered.weak_bg_fill)
        );
    }

    #[test]
    fn press_state_darkens_the_hover_square() {
        assert_eq!(
            toolbar_button_visual_state(true, true, true),
            ToolbarButtonVisualState::Press
        );
        for visuals in [egui::Visuals::dark(), egui::Visuals::light()] {
            let hover = toolbar_button_fill_color(&visuals, ToolbarButtonVisualState::Hover)
                .expect("hover should paint a square");
            let press = toolbar_button_fill_color(&visuals, ToolbarButtonVisualState::Press)
                .expect("press should paint a square");
            assert!(
                color_luma(press) < color_luma(hover),
                "pressed square should be darker than hover"
            );
        }
    }

    #[test]
    fn disabled_button_stays_at_rest() {
        assert_eq!(
            toolbar_button_visual_state(false, true, true),
            ToolbarButtonVisualState::Rest
        );
    }

    fn color_luma(color: egui::Color32) -> u16 {
        color.r() as u16 + color.g() as u16 + color.b() as u16
    }
}
