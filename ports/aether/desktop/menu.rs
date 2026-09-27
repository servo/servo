/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Application hamburger menu. Add new entries in [`AppMenu::contents`].

use egui::{
    Button, CornerRadius, Id, Popup, RectAlign, Sense, Stroke, TextWrapMode, Vec2, WidgetInfo,
    WidgetType, vec2,
};
use euclid::Point2D;
use servo::DeviceIndependentPixel;

/// Matches other chrome menus (see `dialog.rs`).
const APP_MENU_MIN_WIDTH: f32 = 350.0;

const APP_MENU_MIN_HEIGHT: f32 = 50.0;

/// Gap between the toolbar button and the panel.
const APP_MENU_OFFSET: f32 = 4.0;

/// Rounded hover/press fill on menu rows, matching toolbar buttons.
const APP_MENU_ITEM_CORNER_RADIUS: u8 = 4;

/// Padding between a menu label and its hover/press rectangle.
/// Matches the 5px inset used by browser tabs.
const APP_MENU_ITEM_PADDING: f32 = 5.0;

const APP_MENU_ID: &str = "app_menu";

/// An action chosen in the application menu.
pub(crate) enum AppMenuAction {
    NewTab,
    NewWindow,
    History,
}

/// Application menu opened from the toolbar hamburger button.
pub(crate) struct AppMenu {
    open: bool,
    rect: egui::Rect,
}

impl Default for AppMenu {
    fn default() -> Self {
        Self {
            open: false,
            rect: egui::Rect::NOTHING,
        }
    }
}

impl AppMenu {
    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    pub(crate) fn close(&mut self) {
        self.open = false;
        self.rect = egui::Rect::NOTHING;
    }

    pub(crate) fn close_ui(&mut self, ctx: &egui::Context) {
        self.close();
        Popup::close_id(ctx, Id::new(APP_MENU_ID));
    }

    pub(crate) fn contains_pointer(&self, position: Point2D<f32, DeviceIndependentPixel>) -> bool {
        self.open && self.rect.contains(egui::pos2(position.x, position.y))
    }

    /// Toggle the panel from the toolbar button and draw it when open.
    pub(crate) fn update(&mut self, button: &egui::Response) -> Option<AppMenuAction> {
        let inner = Popup::menu(button)
            .id(Id::new(APP_MENU_ID))
            .align(RectAlign::BOTTOM_END)
            .gap(APP_MENU_OFFSET)
            .width(APP_MENU_MIN_WIDTH)
            .show(|ui| {
                ui.set_min_size(vec2(APP_MENU_MIN_WIDTH, APP_MENU_MIN_HEIGHT));
                Self::contents(ui)
            });

        match inner {
            Some(response) => {
                self.open = true;
                self.rect = response.response.rect;
                response.inner
            },
            None => {
                self.close();
                None
            },
        }
    }

    /// Menu body. Insert new items with [`Self::item`].
    fn contents(ui: &mut egui::Ui) -> Option<AppMenuAction> {
        ui.set_min_width(APP_MENU_MIN_WIDTH);
        ui.spacing_mut().item_spacing.y = 2.0;
        ui.spacing_mut().button_padding = vec2(APP_MENU_ITEM_PADDING, APP_MENU_ITEM_PADDING);
        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = ui.visuals().panel_fill;
        ui.style_mut().visuals.widgets.inactive.bg_fill = ui.visuals().panel_fill;

        if let Some(action) = Self::action_item(ui, "New Tab", AppMenuAction::NewTab) {
            return Some(action);
        }
        if let Some(action) = Self::action_item(ui, "New Window", AppMenuAction::NewWindow) {
            return Some(action);
        }
        if let Some(action) = Self::action_item(ui, "History", AppMenuAction::History) {
            return Some(action);
        }

        None
    }

    fn action_item(ui: &mut egui::Ui, label: &str, action: AppMenuAction) -> Option<AppMenuAction> {
        let response = Self::item(ui, label);
        response.widget_info(|| {
            let mut info = WidgetInfo::new(WidgetType::Button);
            info.label = Some(label.into());
            info
        });
        if response.clicked() {
            ui.close();
            Some(action)
        } else {
            None
        }
    }

    /// A full-width row ready to host a menu action.
    fn item(ui: &mut egui::Ui, label: &str) -> egui::Response {
        let button = Button::new(label)
            .corner_radius(CornerRadius::same(APP_MENU_ITEM_CORNER_RADIUS))
            .stroke(Stroke::NONE)
            .wrap_mode(TextWrapMode::Extend)
            .min_size(Vec2 {
                x: APP_MENU_MIN_WIDTH,
                y: 0.0,
            })
            .sense(Sense::click());
        ui.add(button)
    }
}

#[cfg(test)]
mod tests {
    use euclid::Point2D;

    use super::*;

    #[test]
    fn default_menu_is_closed() {
        let menu = AppMenu::default();
        assert!(!menu.is_open());
        assert!(!menu.contains_pointer(Point2D::new(10.0, 10.0)));
    }

    #[test]
    fn close_clears_open_state() {
        let mut menu = AppMenu::default();
        menu.open = true;
        menu.close();
        assert!(!menu.is_open());
        assert!(!menu.contains_pointer(Point2D::new(0.0, 0.0)));
    }

    #[test]
    fn open_menu_contains_pointer_only_inside_its_rect() {
        let mut menu = AppMenu::default();
        menu.open = true;
        menu.rect = egui::Rect::from_min_max(egui::pos2(10.0, 20.0), egui::pos2(110.0, 80.0));

        // egui rects are closed: min and max edges are inside.
        assert!(menu.contains_pointer(Point2D::new(10.0, 20.0)));
        assert!(menu.contains_pointer(Point2D::new(60.0, 40.0)));
        assert!(menu.contains_pointer(Point2D::new(110.0, 40.0)));
        assert!(menu.contains_pointer(Point2D::new(60.0, 80.0)));
        assert!(!menu.contains_pointer(Point2D::new(0.0, 0.0)));
        assert!(!menu.contains_pointer(Point2D::new(110.1, 40.0)));
        assert!(!menu.contains_pointer(Point2D::new(60.0, 80.1)));

        menu.open = false;
        assert!(!menu.contains_pointer(Point2D::new(60.0, 40.0)));
    }
}
