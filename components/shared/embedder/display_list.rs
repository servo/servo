/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An embedder-facing snapshot of the display list produced by Servo's layout engine.
//!
//! When `layout_display_list_capture_enabled` is set, layout records a projection of
//! the content items it paints (text runs, solid color fills, images and iframes)
//! alongside the WebRender display list. Snapshots are composed across the frame tree
//! and delivered via `WebViewDelegate::notify_display_list`, for embedder-side text
//! extraction or native rendering.

use serde::{Deserialize, Serialize};
use servo_base::Epoch;
use servo_base::id::PipelineId;
use webrender_api::ColorF;
use webrender_api::units::{LayoutRect, LayoutSize, LayoutVector2D};

/// The coordinate space of a captured [`DisplayListItem`]'s rectangle, and how it
/// responds to root viewport scrolling.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DisplayListItemSpace {
    /// Positioned relative to the document origin; moves when the root viewport
    /// scrolls. Subtract [`DisplayList::scroll_offset`] to get viewport coordinates.
    Document,
    /// Anchored to the viewport (e.g. `position: fixed`); does not move when the
    /// root viewport scrolls. Rectangle is already in viewport coordinates.
    Viewport,
}

/// What a captured [`DisplayListItem`] paints.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum DisplayListItemContent {
    /// A run of text.
    Text {
        /// The source text of this run, with leading and trailing collapsible
        /// whitespace removed.
        text: String,
        /// The CSS `color` used to paint this text.
        color: ColorF,
    },
    /// A solid color fill, like an element's `background-color`.
    SolidColor {
        /// The fill color.
        color: ColorF,
    },
    /// A raster or vector image.
    Image,
    /// The viewport of a nested browsing context (`<iframe>`).
    ///
    /// In delivered snapshots, this item is immediately followed by the named
    /// pipeline's own items, positioned and clipped within this item's rectangle,
    /// whenever that pipeline has a captured snapshot.
    Iframe {
        /// The pipeline of the document displayed inside the iframe.
        pipeline_id: PipelineId,
    },
}

/// A single item captured from a layout [`DisplayList`]: a projection of what
/// layout pushes into the WebRender display list, for embedders to build their own
/// rendering interpretation of the page.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DisplayListItem {
    /// The bounding rectangle in CSS pixels.
    ///
    /// The rectangle is resolved through the spatial tree: CSS transforms, the scroll
    /// offsets of every ancestor scroll frame, and sticky positioning offsets are all
    /// applied. Transformed content is reported as the axis-aligned bounding box of
    /// the transformed rectangle. The rectangle is also clipped by the item's
    /// accumulated clip chain (`overflow` clips, scroll ports and `clip-path` bounds),
    /// so content hidden or partially hidden when painted is culled or reduced here as
    /// well.
    pub rect: LayoutRect,

    /// The coordinate space of [`Self::rect`].
    pub space: DisplayListItemSpace,

    /// What this item paints.
    pub content: DisplayListItemContent,
}

/// A snapshot of the display list layout produced for one pipeline (document), or,
/// via `WebViewDelegate::notify_display_list`, a `WebView`'s entire frame tree with
/// subframe items spliced into their `Iframe` items.
///
/// Items are in paint order (back to front). Scroll offsets are the offsets at capture
/// time: display lists are rebuilt on layout changes but not on (asynchronous) scrolls,
/// so embedders that track scrolling between snapshots should combine
/// [`DisplayListItemSpace`] with the live root scroll offset reported by the `WebView`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DisplayList {
    /// The pipeline this snapshot was captured from. For composed snapshots, the
    /// `WebView`'s root pipeline.
    pub pipeline_id: PipelineId,

    /// All captured display items, in paint order.
    pub items: Vec<DisplayListItem>,

    /// The layout epoch of [`Self::pipeline_id`] when this display list was captured.
    pub epoch: Epoch,

    /// The scroll offset of the root viewport, in CSS pixels, at capture time.
    pub scroll_offset: LayoutVector2D,

    /// The size of the root viewport in CSS pixels at capture time.
    pub viewport_size: LayoutSize,

    /// The size of the document's scrollable content in CSS pixels at capture time.
    pub content_size: LayoutSize,
}
