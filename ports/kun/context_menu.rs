/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/* macOS, Windows Native Implementation */
#[cfg(linux)]
use base::id::WebViewId;
#[cfg(linux)]
use compositing_traits::ConstellationMsg;
#[cfg(linux)]
use crossbeam_channel::Sender;
#[cfg(linux)]
use euclid::{Point2D, Size2D};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use muda::{ContextMenu as MudaContextMenu, Menu as MudaMenu};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
#[cfg(linux)]
use serde::{Deserialize, Serialize};
#[cfg(linux)]
use servo_url::ServoUrl;
#[cfg(linux)]
use webrender_api::units::DeviceIntRect;
#[cfg(linux)]
use winit::dpi::PhysicalPosition;

/* Wayland Implementation */
#[cfg(linux)]
use crate::{servo::send_to_constellation, webview::WebView, window::Window};

/// Basic menu type building block
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub struct Menu(pub MudaMenu);
/// Basic menu type building block
#[cfg(linux)]
#[derive(Clone, Debug)]
pub struct Menu(pub Vec<MenuItem>);

/// The Context Menu of the Window. It will be opened when users right click on any window's
/// webview.
///
/// **Platform Specific**
/// - macOS / Windows: This will be native context menu supported by each OS.
/// - Wayland: Winit doesn't support popup surface of Wayland at the moment. So we utilize a custom
/// webview implementation.
#[derive(Clone)]
pub struct ContextMenu {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    menu: MudaMenu,
    #[cfg(linux)]
    menu_items: Vec<MenuItem>,
    /// The webview that the context menu is attached to
    #[cfg(linux)]
    webview: WebView,
}

impl ContextMenu {
    /// Create context menu with custom items
    ///
    /// **Platform Specific**
    /// - macOS / Windows: Creates a context menu by muda crate with natvie OS support
    /// - Wayland: Creates a context menu with webview implementation
    pub fn new_with_menu(menu: Menu) -> Self {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            Self { menu: menu.0 }
        }
        #[cfg(linux)]
        {
            let webview_id = WebViewId::new();
            let webview = WebView::new(webview_id, DeviceIntRect::zero());

            Self {
                menu_items: menu.0,
                webview,
            }
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
impl ContextMenu {
    /// Show the context menu on current cursor position
    ///
    /// This function returns when the context menu is dismissed
    pub fn show(&self, rwh: impl HasWindowHandle) {
        // Show the context menu
        unsafe {
            let wh = rwh.window_handle().unwrap();
            match wh.as_raw() {
                #[cfg(target_os = "macos")]
                RawWindowHandle::AppKit(handle) => {
                    // use objc2
                    assert!(
                        objc2_foundation::is_main_thread(),
                        "can only access AppKit handles on the main thread"
                    );
                    let ns_view = handle.ns_view.as_ptr();
                    self.menu.show_context_menu_for_nsview(ns_view, None);
                },
                #[cfg(target_os = "windows")]
                RawWindowHandle::Win32(handle) => {
                    let hwnd = handle.hwnd;
                    self.menu.show_context_menu_for_hwnd(hwnd.into(), None);
                },
                handle => unreachable!("unknown handle {handle:?} for platform"),
            }
        }
    }
}

#[cfg(linux)]
impl ContextMenu {
    /// Show the context menu to current cursor position
    pub fn show(
        &mut self,
        sender: &Sender<ConstellationMsg>,
        window: &mut Window,
        position: PhysicalPosition<f64>,
    ) {
        let scale_factor = window.scale_factor();
        self.set_position(window, position, scale_factor);

        send_to_constellation(
            sender,
            ConstellationMsg::NewWebView(self.resource_url(), self.webview.webview_id),
        );
    }

    /// Get webview of the context menu
    pub fn webview(&self) -> &WebView {
        &self.webview
    }

    /// Get resource URL of the context menu
    fn resource_url(&self) -> ServoUrl {
        let items_json: String = self.to_items_json();
        let url_str = format!("servo://context_menu.html?items={}", items_json);
        ServoUrl::parse(&url_str).unwrap()
    }

    /// Set the position of the context menu
    fn set_position(
        &mut self,
        window: &Window,
        position: PhysicalPosition<f64>,
        scale_factor: f64,
    ) {
        // Calculate menu size
        // Each menu item is 30px height
        // Menu has 10px padding top and bottom
        let height = (self.menu_items.len() * 30 + 20) as f64 * scale_factor;
        let width = 200.0 * scale_factor;
        let menu_size = Size2D::new(width as i32, height as i32);

        // Translate position to origin
        let mut origin = Point2D::new(position.x as i32, position.y as i32);

        // Avoid overflow to the window, adjust position if necessary
        let window_size = window.size();
        let x_overflow: i32 = origin.x + menu_size.width - window_size.width;
        let y_overflow: i32 = origin.y + menu_size.height - window_size.height;

        if x_overflow >= 0 {
            // check if the menu can be shown on left side of the cursor
            if (origin.x - menu_size.width) >= 0 {
                origin.x = i32::max(0, origin.x - menu_size.width);
            } else {
                // if menu can't fit to left side of the cursor,
                // shift left the menu, but not less than zero.
                // TODO: if still smaller than screen, should show scroller
                origin.x = i32::max(0, origin.x - x_overflow);
            }
        }
        if y_overflow >= 0 {
            // check if the menu can be shown above the cursor
            if (origin.y - menu_size.height) >= 0 {
                origin.y = i32::max(0, origin.y - menu_size.height);
            } else {
                // if menu can't fit to top of the cursor
                // shift up the menu, but not less than zero.
                // TODO: if still smaller than screen, should show scroller
                origin.y = i32::max(0, origin.y - y_overflow);
            }
        }

        self.webview
            .set_size(DeviceIntRect::from_origin_and_size(origin, menu_size));
    }

    /// get item json
    fn to_items_json(&self) -> String {
        serde_json::to_string(&self.menu_items).unwrap()
    }
}

/// Menu Item
#[cfg(linux)]
#[derive(Clone, Debug, Serialize)]
pub struct MenuItem {
    id: String,
    /// label of the menu item
    pub label: String,
    /// Whether the menu item is enabled
    pub enabled: bool,
}

#[cfg(linux)]
impl MenuItem {
    /// Create a new menu item
    pub fn new(id: Option<&str>, label: &str, enabled: bool) -> Self {
        let id = id.unwrap_or(label);
        Self {
            id: id.to_string(),
            label: label.to_string(),
            enabled,
        }
    }
    /// Get the id of the menu item
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Context Menu Click Result
#[cfg(linux)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContextMenuResult {
    /// The id of the menu ite    /// Get the label of the menu item
    pub id: String,
    /// Close the context menu
    pub close: bool,
}
