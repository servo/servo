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
use newcss::values::CSSBorderStyle;
use render_context::RenderContext;
use text::SendableTextRun;

use std::cast::transmute_region;
use geom::{Point2D, Rect, Size2D, SideOffsets2D};
use servo_net::image::base::Image;
use servo_util::range::Range;
use extra::arc::Arc;

use newcss::values::{CSSTextDecorationUnderline, CSSTextDecorationOverline, CSSTextDecorationLineThrough};

/// A list of rendering operations to be performed.
pub struct DisplayList<E> {
    list: ~[DisplayItem<E>]
}

impl<E> DisplayList<E> {
    /// Creates a new display list.
    pub fn new() -> DisplayList<E> {
        DisplayList {
            list: ~[]
        }
    }

    /// Appends the given item to the display list.
    pub fn append_item(&mut self, item: DisplayItem<E>) {
        // FIXME(Issue #150): crashes
        //debug!("Adding display item %u: %?", self.len(), item);
        self.list.push(item)
    }

    /// Draws the display list into the given render context.
    pub fn draw_into_context(&self, render_context: &RenderContext) {
        debug!("Beginning display list.");
        for item in self.list.iter() {
            // FIXME(Issue #150): crashes
            //debug!("drawing %?", *item);
            item.draw_into_context(render_context)
        }
        debug!("Ending display list.")
    }
}

/// One drawing command in the list.
pub enum DisplayItem<E> {
    SolidColorDisplayItemClass(~SolidColorDisplayItem<E>),
    TextDisplayItemClass(~TextDisplayItem<E>),
    ImageDisplayItemClass(~ImageDisplayItem<E>),
    BorderDisplayItemClass(~BorderDisplayItem<E>),
}

/// Information common to all display items.
pub struct BaseDisplayItem<E> {
    /// The boundaries of the display item.
    ///
    /// TODO: Which coordinate system should this use?
    bounds: Rect<Au>,

    /// Extra data: either the originating flow (for hit testing) or nothing (for rendering).
    extra: E,
}

/// Renders a solid color.
pub struct SolidColorDisplayItem<E> {
    base: BaseDisplayItem<E>,
    color: Color,
}

/// Renders text.
pub struct TextDisplayItem<E> {
    base: BaseDisplayItem<E>,
    text_run: ~SendableTextRun,
    range: Range,
    color: Color,
}

/// Renders an image.
pub struct ImageDisplayItem<E> {
    base: BaseDisplayItem<E>,
    image: Arc<~Image>,
}

/// Renders a border.
pub struct BorderDisplayItem<E> {
    base: BaseDisplayItem<E>,

    /// The border widths
    border: SideOffsets2D<Au>,

    /// The border colors.
    color: SideOffsets2D<Color>,

    /// The border styles.
    style: SideOffsets2D<CSSBorderStyle>
}

impl<E> DisplayItem<E> {
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

                let width = text.base.bounds.size.width;
                let underline_size = font.metrics.underline_size;
                let underline_offset = font.metrics.underline_offset;
                let strikeout_size = font.metrics.strikeout_size;
                let strikeout_offset = font.metrics.strikeout_offset;

                match new_run.decoration {
                    CSSTextDecorationUnderline => {
                        let underline_y = baseline_origin.y - underline_offset;
                        let underline_bounds = Rect(Point2D(baseline_origin.x, underline_y),
                                                    Size2D(width, underline_size));
                        render_context.draw_solid_color(&underline_bounds, text.color);
                    },
                    CSSTextDecorationOverline => {
                        let overline_bounds = Rect(Point2D(baseline_origin.x, origin.y),
                                                   Size2D(width, underline_size));
                        render_context.draw_solid_color(&overline_bounds, text.color);
                    },
                    CSSTextDecorationLineThrough => {
                        let strikeout_y = baseline_origin.y - strikeout_offset;
                        let strikeout_bounds = Rect(Point2D(baseline_origin.x, strikeout_y),
                                                    Size2D(width, strikeout_size));
                        render_context.draw_solid_color(&strikeout_bounds, text.color);
                    },
                    _ => ()
                }
            }

            ImageDisplayItemClass(ref image_item) => {
                debug!("Drawing image at %?.", image_item.base.bounds);

                render_context.draw_image(image_item.base.bounds, image_item.image.clone())
            }

            BorderDisplayItemClass(ref border) => {
                render_context.draw_border(&border.base.bounds,
                                           border.border,
                                           border.color,
                                           border.style)
            }
        }
    }

    pub fn base<'a>(&'a self) -> &'a BaseDisplayItem<E> {
        // FIXME(tkuehn): Workaround for Rust region bug.
        unsafe {
            match *self {
                SolidColorDisplayItemClass(ref solid_color) => transmute_region(&solid_color.base),
                TextDisplayItemClass(ref text) => transmute_region(&text.base),
                ImageDisplayItemClass(ref image_item) => transmute_region(&image_item.base),
                BorderDisplayItemClass(ref border) => transmute_region(&border.base)
            }
        }
    }

    pub fn bounds(&self) -> Rect<Au> {
        self.base().bounds
    }
}

