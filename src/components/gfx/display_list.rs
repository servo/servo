/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Servo heavily uses display lists, which are retained-mode lists of rendering commands to
/// perform. Using a list instead of rendering elements in immediate mode allows transforms, hit
/// testing, and invalidation to be performed using the same primitives as painting. It also allows
/// Servo to aggressively cull invisible and out-of-bounds rendering elements, to reduce overdraw.
/// Finally, display lists allow tiles to be farmed out onto multiple CPUs and rendered in
/// parallel (although this benefit does not apply to GPU-based rendering).
///
/// Display items describe relatively high-level drawing operations (for example, entire borders
/// and shadows instead of lines and blur operations), to reduce the amount of allocation required.
/// They are therefore not exactly analogous to constructs like Skia pictures, which consist of
/// low-level drawing primitives.

use color::Color;
use geometry::Au;
use render_context::RenderContext;
use text::SendableTextRun;

use geom::{Point2D, Rect, Size2D};
use servo_net::image::base::Image;
use servo_util::range::Range;
use std::arc::ARC;
use std::arc;

/// A list of rendering operations to be performed.
pub struct DisplayList {
    priv list: ~[DisplayItem]
}

impl DisplayList {
    /// Creates a new display list.
    pub fn new() -> DisplayList {
        DisplayList {
            list: ~[]
        }
    }

    /// Appends the given item to the display list.
    pub fn append_item(&mut self, item: DisplayItem) {
        // FIXME(Issue #150): crashes
        //debug!("Adding display item %u: %?", self.len(), item);
        self.list.push(item)
    }

    /// Draws the display list into the given render context.
    pub fn draw_into_context(&self, render_context: &RenderContext) {
        debug!("Beginning display list.");
        for self.list.each |item| {
            // FIXME(Issue #150): crashes
            //debug!("drawing %?", *item);
            item.draw_into_context(render_context)
        }
        debug!("Ending display list.")
    }
}

/// One drawing command in the list.
pub enum DisplayItem {
    SolidColorDisplayItemClass(~SolidColorDisplayItem),
    TextDisplayItemClass(~TextDisplayItem),
    ImageDisplayItemClass(~ImageDisplayItem),
    BorderDisplayItemClass(~BorderDisplayItem),
}

/// Information common to all display items.
pub struct BaseDisplayItem {
    /// The boundaries of the display item.
    ///
    /// TODO: Which coordinate system should this use?
    bounds: Rect<Au>,
}

/// Renders a solid color.
pub struct SolidColorDisplayItem {
    base: BaseDisplayItem,
    color: Color,
}

/// Renders text.
pub struct TextDisplayItem {
    base: BaseDisplayItem,
    text_run: ~SendableTextRun,
    range: Range,
    color: Color,
}

/// Renders an image.
pub struct ImageDisplayItem {
    base: BaseDisplayItem,
    image: ARC<~Image>,
}

/// Renders a border.
pub struct BorderDisplayItem {
    base: BaseDisplayItem,
    /// The width of the border.
    width: Au,
    /// The color of the border.
    color: Color,
}

impl DisplayItem {
    /// Renders this display item into the given render context.
    fn draw_into_context(&self, render_context: &RenderContext) {
        match *self {
            SolidColorDisplayItemClass(ref solid_color) => {
                render_context.draw_solid_color(&solid_color.base.bounds, solid_color.color)
            }

            TextDisplayItemClass(ref text) => {
                debug!("Drawing text at %?.", text.base.bounds);

                // FIXME(pcwalton): Allocating? Why?
                let new_run = @text.text_run.deserialize(render_context.font_ctx);

                let font = new_run.font;
                let origin = text.base.bounds.origin;
                let baseline_origin = Point2D(origin.x, origin.y + font.metrics.ascent);

                font.draw_text_into_context(render_context,
                                            new_run,
                                            &text.range,
                                            baseline_origin,
                                            text.color);

                if new_run.underline {
                    // TODO(eatkinson): Use the font metrics to properly position the underline
                    // bar.
                    let width = text.base.bounds.size.width;
                    let underline_size = font.metrics.underline_size;
                    let underline_bounds = Rect(Point2D(baseline_origin.x, baseline_origin.y),
                                                Size2D(width, underline_size));
                    render_context.draw_solid_color(&underline_bounds, text.color);
                }
            }

            ImageDisplayItemClass(ref image_item) => {
                debug!("Drawing image at %?.", image_item.base.bounds);

                render_context.draw_image(image_item.base.bounds, image_item.image.clone())
            }

            BorderDisplayItemClass(ref border) => {
                render_context.draw_border(&border.base.bounds, border.width, border.color)
            }
        }
    }
}

