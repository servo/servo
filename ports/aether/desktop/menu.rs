/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Application hamburger menu. Add new entries in [`AppMenu::root_page`].

use egui::{
    Button, CornerRadius, Id, Label, Popup, PopupCloseBehavior, RectAlign, Sense, Stroke,
    TextWrapMode, Vec2, WidgetInfo, WidgetType, vec2,
};
use euclid::Point2D;
use servo::DeviceIndependentPixel;

use crate::desktop::icons::{ToolbarIcon, ToolbarIconCache};

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
    Settings,
    Processes,
    DeveloperTools,
    Exit,
}

/// Which page of the hamburger menu is showing.
///
/// Nested pages replace the menu body in place. The popup stays open.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AppMenuPage {
    Root,
    MoreTools,
}

/// Application menu opened from the toolbar hamburger button.
pub(crate) struct AppMenu {
    open: bool,
    rect: egui::Rect,
    page: AppMenuPage,
    /// Content height of the root page, so nested pages keep the same panel size.
    root_content_height: f32,
}

impl Default for AppMenu {
    fn default() -> Self {
        Self {
            open: false,
            rect: egui::Rect::NOTHING,
            page: AppMenuPage::Root,
            root_content_height: 0.0,
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
        self.page = AppMenuPage::Root;
        self.root_content_height = 0.0;
    }

    pub(crate) fn close_ui(&mut self, ctx: &egui::Context) {
        self.close();
        Popup::close_id(ctx, Id::new(APP_MENU_ID));
    }

    pub(crate) fn contains_pointer(&self, position: Point2D<f32, DeviceIndependentPixel>) -> bool {
        self.open && self.rect.contains(egui::pos2(position.x, position.y))
    }

    /// Height of the menu body. Nested pages reuse the root page's height.
    fn content_min_height(&self) -> f32 {
        match self.page {
            AppMenuPage::Root => APP_MENU_MIN_HEIGHT,
            AppMenuPage::MoreTools => self.root_content_height.max(APP_MENU_MIN_HEIGHT),
        }
    }

    /// Toggle the panel from the toolbar button and draw it when open.
    pub(crate) fn update(
        &mut self,
        button: &egui::Response,
        icons: &mut ToolbarIconCache,
    ) -> Option<AppMenuAction> {
        let drawn_page = self.page;
        let min_height = self.content_min_height();
        let inner = Popup::menu(button)
            .id(Id::new(APP_MENU_ID))
            .align(RectAlign::BOTTOM_END)
            .gap(APP_MENU_OFFSET)
            .width(APP_MENU_MIN_WIDTH)
            // Stay open while switching pages. Actions still call `ui.close()`.
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                ui.set_min_size(vec2(APP_MENU_MIN_WIDTH, min_height));
                let action = self.contents(ui, icons);
                if drawn_page == AppMenuPage::Root {
                    self.root_content_height = ui.min_rect().height();
                }
                action
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

    /// Menu body. Insert new root items in [`Self::root_page`].
    fn contents(
        &mut self,
        ui: &mut egui::Ui,
        icons: &mut ToolbarIconCache,
    ) -> Option<AppMenuAction> {
        ui.set_min_width(APP_MENU_MIN_WIDTH);
        ui.spacing_mut().item_spacing.y = 2.0;
        ui.spacing_mut().button_padding = vec2(APP_MENU_ITEM_PADDING, APP_MENU_ITEM_PADDING);
        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = ui.visuals().panel_fill;
        ui.style_mut().visuals.widgets.inactive.bg_fill = ui.visuals().panel_fill;

        match self.page {
            AppMenuPage::Root => self.root_page(ui, icons),
            AppMenuPage::MoreTools => self.more_tools_page(ui, icons),
        }
    }

    fn root_page(
        &mut self,
        ui: &mut egui::Ui,
        icons: &mut ToolbarIconCache,
    ) -> Option<AppMenuAction> {
        if let Some(action) = Self::action_item(ui, "New Tab", AppMenuAction::NewTab) {
            return Some(action);
        }
        if let Some(action) = Self::action_item(ui, "New Window", AppMenuAction::NewWindow) {
            return Some(action);
        }
        if let Some(action) = Self::action_item(ui, "History", AppMenuAction::History) {
            return Some(action);
        }
        if let Some(action) = Self::action_item(ui, "Settings", AppMenuAction::Settings) {
            return Some(action);
        }
        if Self::submenu_item(ui, icons, "More Tools") {
            self.page = AppMenuPage::MoreTools;
        }
        ui.separator();
        if let Some(action) = Self::action_item(ui, "Exit", AppMenuAction::Exit) {
            return Some(action);
        }

        None
    }

    fn more_tools_page(
        &mut self,
        ui: &mut egui::Ui,
        icons: &mut ToolbarIconCache,
    ) -> Option<AppMenuAction> {
        if Self::back_title(ui, icons, "More tools") {
            self.page = AppMenuPage::Root;
            return None;
        }
        if let Some(action) =
            Self::action_item(ui, "Developer Tools", AppMenuAction::DeveloperTools)
        {
            return Some(action);
        }
        if let Some(action) = Self::action_item(ui, "Processes", AppMenuAction::Processes) {
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

    /// Full-width row that opens another page of this menu. The label and the
    /// trailing chevron are one click target, and the popup stays open.
    fn submenu_item(ui: &mut egui::Ui, icons: &mut ToolbarIconCache, label: &str) -> bool {
        let mut button = Button::new(label)
            .corner_radius(CornerRadius::same(APP_MENU_ITEM_CORNER_RADIUS))
            .stroke(Stroke::NONE)
            .wrap_mode(TextWrapMode::Extend)
            .min_size(Vec2 {
                x: APP_MENU_MIN_WIDTH,
                y: 0.0,
            })
            .sense(Sense::click());
        if let Some(icon) = icons.image(ui, ToolbarIcon::ChevronRight) {
            button = button.right_text(icon).image_tint_follows_text_color(true);
        }
        let response = ui.add(button);
        response.widget_info(|| {
            let mut info = WidgetInfo::new(WidgetType::Button);
            info.label = Some(label.into());
            info
        });
        response.clicked()
    }

    /// Nested-page heading. Only the leading arrow is a button; the title is not.
    fn back_title(ui: &mut egui::Ui, icons: &mut ToolbarIconCache, title: &str) -> bool {
        ui.horizontal(|ui| {
            let response = ui.add(
                icons
                    .image_button(ui, ToolbarIcon::ChevronLeft)
                    .corner_radius(CornerRadius::same(APP_MENU_ITEM_CORNER_RADIUS))
                    .stroke(Stroke::NONE)
                    .min_size(Vec2::ZERO),
            );
            response.widget_info(|| {
                let mut info = WidgetInfo::new(WidgetType::Button);
                info.label = Some("Back".into());
                info
            });
            ui.add(Label::new(title).selectable(false));
            response.clicked()
        })
        .inner
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
    fn close_returns_to_the_root_page() {
        let mut menu = AppMenu::default();
        menu.page = AppMenuPage::MoreTools;
        menu.root_content_height = 180.0;
        menu.close();
        assert_eq!(menu.page, AppMenuPage::Root);
        assert_eq!(menu.root_content_height, 0.0);
    }

    #[test]
    fn child_page_keeps_the_root_menu_height() {
        let mut menu = AppMenu::default();
        assert_eq!(menu.content_min_height(), APP_MENU_MIN_HEIGHT);

        menu.root_content_height = 180.0;
        menu.page = AppMenuPage::MoreTools;
        assert_eq!(menu.content_min_height(), 180.0);

        menu.page = AppMenuPage::Root;
        assert_eq!(menu.content_min_height(), APP_MENU_MIN_HEIGHT);
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

    fn screen_input(events: Vec<egui::Event>) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 300.0),
            )),
            events,
            ..Default::default()
        }
    }

    fn click(pos: egui::Pos2) -> Vec<egui::Event> {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            },
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            },
        ]
    }

    #[test]
    fn more_tools_developer_tools_row_selects_that_action() {
        let ctx = egui::Context::default();
        let mut icons = ToolbarIconCache::default();
        ctx.run_ui(screen_input(vec![]), |ui| {
            let mut menu = AppMenu::default();
            menu.page = AppMenuPage::MoreTools;
            assert!(menu.more_tools_page(ui, &mut icons).is_none());
        });

        let mut found = false;
        for y in (0..160).step_by(8) {
            let mut action = None;
            ctx.run_ui(screen_input(click(egui::pos2(40.0, y as f32))), |ui| {
                let mut menu = AppMenu::default();
                menu.page = AppMenuPage::MoreTools;
                action = menu.more_tools_page(ui, &mut icons);
            });
            if matches!(action, Some(AppMenuAction::DeveloperTools)) {
                found = true;
                break;
            }
        }
        assert!(found, "Developer Tools row should be clickable");
    }
}
