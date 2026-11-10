/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use accesskit::Affine;
use egui::{Rect, Sense, Widget};
use euclid::Scale;
use servo::{DeviceIndependentPixel, DevicePixel, WebView};

pub(crate) struct WebViewPanel {
    id: egui::Id,
    webview: WebView,
    scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    available_rect: Rect,
    /// Whether the [`WebView`] represented by this panel is the active webview for servoshell.
    active: bool,
    /// Whether servoshell-global focus is inside this panel's webview, regardless of whether the
    /// application itself has system focus.
    contains_focus: bool,
    /// Whether the [`accesskit::TreeId`] for this panel's webview has been previously sent to
    /// AccessKit.
    tree_id_known: bool,
}

impl WebViewPanel {
    pub fn new(
        webview: WebView,
        scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
        available_rect: Rect,
        active: bool,
        contains_focus: bool,
        tree_id_known: bool,
    ) -> Self {
        Self {
            id: egui::Id::new(webview.id()),
            webview,
            scale_factor,
            available_rect,
            active,
            contains_focus,
            tree_id_known,
        }
    }
}

impl Widget for WebViewPanel {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let tree_id = self.webview.accesskit_tree_id();

        // Always build an accesskit graft node, even if the webview isn't currently showing, so that any tree updates
        // from the webview won't cause a panic.
        if let Some(tree_id) = tree_id {
            ui.accesskit_node_builder(self.id, |node| {
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
                node.set_tree_id(tree_id);
                // Only the transform is set: AccessKit consumers exclude graft nodes from
                // the presented tree, so bounds on this node would never be read.
                node.set_transform(affine);
            });
        }

        // If:
        // - this is not the active webview, or
        // - the application doesn't have system focus, or
        // - accessibility is active, but the initial accesskit update for the webview hasn't been
        //   sent yet;
        // then don't move egui focus to this panel on this tick .
        if !self.active || !self.webview.focused() || (tree_id.is_some() && !self.tree_id_known) {
            ui.memory_mut(|mem| mem.surrender_focus(self.id));
            return ui.response();
        }

        let response = ui.interact(self.available_rect, self.id, Sense::FOCUSABLE);

        // If the application lost and regained system focus since last time this panel was drawn,
        // we need to request focus; otherwise, focus will reset to the first UI element.
        if !response.has_focus() && self.contains_focus {
            response.request_focus();
        }

        response
    }
}
