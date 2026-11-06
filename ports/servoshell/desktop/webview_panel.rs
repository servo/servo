/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use accesskit::Affine;
use egui::{EventFilter, Rect, Sense, Widget};
use euclid::Scale;
use servo::{DeviceIndependentPixel, DevicePixel, WebView, WebViewId};

pub(crate) struct WebViewPanel {
    id: egui::Id,
    webview: WebView,
    scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    available_rect: Rect,
    active: bool,
    contains_focus: bool,
}

impl WebViewPanel {
    pub fn new(
        webview_id: WebViewId,
        webview: WebView,
        scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
        available_rect: Rect,
        active: bool,
        contains_focus: bool,
    ) -> Self {
        Self {
            id: egui::Id::new(webview_id),
            webview,
            scale_factor,
            available_rect,
            active,
            contains_focus,
        }
    }

    pub fn id(&self) -> egui::Id {
        self.id
    }
}

impl Widget for WebViewPanel {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // Always build an accesskit graft node, even if the webview isn't currently showing, so that any tree updates
        // from the webview won't cause a panic.
        if let Some(tree_id) = self.webview.accesskit_tree_id() {
            let affine = {
                // The grafted WebView tree reports bounds in device pixels relative to the
                // WebView's own origin, so this node supplies the offset of the WebView within
                // the window, and scales it to the same scale as the rest of the nodes in egui's
                // AccessKit tree.
                let scale = (1.0 / self.scale_factor.get()) as f64;
                let x = self.available_rect.min.x as f64;
                let y = self.available_rect.min.y as f64;
                Affine::new([scale, 0.0, 0.0, scale, x, y])
            };
            ui.accesskit_node_builder(self.id, |node| {
                node.set_tree_id(tree_id);
                // Only the transform is set: AccessKit consumers exclude graft nodes from
                // the presented tree, so bounds on this node would never be read.
                node.set_transform(affine);
            });
        }

        if !self.active || !self.webview.focused() {
            ui.memory_mut(|mem| mem.surrender_focus(self.id));
            return ui.response();
        }

        let response = ui.interact(self.available_rect, self.id, Sense::FOCUSABLE);
        if self.contains_focus {
            response.request_focus();
        }

        // Once the webview is focused, it doesn't allow focus to be moved with the keyboard to
        // other parts of the UI, so that focus can move within the web contents.
        // TODO: Release focus from the webview if keyboard focus moves past the beginning or end
        // of the web contents.
        let event_filter = EventFilter {
            tab: true,
            horizontal_arrows: true,
            vertical_arrows: true,
            escape: true,
        };
        ui.memory_mut(|mem| mem.set_focus_lock_filter(self.id, event_filter));

        response
    }
}
