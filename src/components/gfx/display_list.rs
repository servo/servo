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
use render_context::RenderContext;
use text::TextRun;

use extra::arc::Arc;
use geom::{Point2D, Rect, Size2D, SideOffsets2D};
use servo_net::image::base::Image;
use servo_util::geometry::Au;
use servo_util::range::Range;
use std::cast::transmute_region;
use std::vec::VecIterator;
use style::computed_values::border_style;

pub struct DisplayListCollection<E> {
    lists: ~[DisplayList<E>]
}

impl<E> DisplayListCollection<E> {
    pub fn new() -> DisplayListCollection<E> {
        DisplayListCollection {
            lists: ~[]
        }
    }

    pub fn iter<'a>(&'a self) -> DisplayListIterator<'a,E> {
        ParentDisplayListIterator(self.lists.iter())
    }

    pub fn add_list(&mut self, list: DisplayList<E>) {
        self.lists.push(list);
    }

    pub fn draw_lists_into_context(&self, render_context: &mut RenderContext) {
        for list in self.lists.iter() {
            list.draw_into_context(render_context);
        }
        debug!("{:?}", self.dump());
    }

    fn dump(&self) {
        let mut index = 0;
        for list in self.lists.iter() {
            debug!("dumping display list {:d}:", index);
            list.dump();
            index = index + 1;
        }
    }
}

/// A list of rendering operations to be performed.
pub struct DisplayList<E> {
    list: ~[DisplayItem<E>]
}

pub enum DisplayListIterator<'a,E> {
    EmptyDisplayListIterator,
    ParentDisplayListIterator(VecIterator<'a,DisplayList<E>>),
}

impl<'a,E> Iterator<&'a DisplayList<E>> for DisplayListIterator<'a,E> {
    #[inline]
    fn next(&mut self) -> Option<&'a DisplayList<E>> {
        match *self {
            EmptyDisplayListIterator => None,
            ParentDisplayListIterator(ref mut subiterator) => subiterator.next(),
        }
    }
}

impl<E> DisplayList<E> {
    /// Creates a new display list.
    pub fn new() -> DisplayList<E> {
        DisplayList {
            list: ~[]
        }
    }

    fn dump(&self) {
        for item in self.list.iter() {
            item.debug_with_level(0);
        }
    }

    /// Appends the given item to the display list.
    pub fn append_item(&mut self, item: DisplayItem<E>) {
        // FIXME(Issue #150): crashes
        //debug!("Adding display item {:u}: {}", self.len(), item);
        self.list.push(item)
    }

    /// Draws the display list into the given render context.
    pub fn draw_into_context(&self, render_context: &mut RenderContext) {
        debug!("Beginning display list.");
        for item in self.list.iter() {
            // FIXME(Issue #150): crashes
            //debug!("drawing {}", *item);
            item.draw_into_context(render_context)
        }
        debug!("Ending display list.");
    }

    /// Returns a preorder iterator over the given display list.
    pub fn iter<'a>(&'a self) -> DisplayItemIterator<'a,E> {
        ParentDisplayItemIterator(self.list.iter())
    }
}

/// One drawing command in the list.
pub enum DisplayItem<E> {
    SolidColorDisplayItemClass(~SolidColorDisplayItem<E>),
    TextDisplayItemClass(~TextDisplayItem<E>),
    ImageDisplayItemClass(~ImageDisplayItem<E>),
    BorderDisplayItemClass(~BorderDisplayItem<E>),
    LineDisplayItemClass(~LineDisplayItem<E>),
    ClipDisplayItemClass(~ClipDisplayItem<E>)
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
    /// Fields common to all display items.
    base: BaseDisplayItem<E>,

    /// The text run.
    text_run: Arc<~TextRun>,

    /// The range of text within the text run.
    range: Range,

    /// The color of the text.
    text_color: Color,

    /// A bitfield of flags for text display items.
    flags: TextDisplayItemFlags,

    /// The color of text-decorations
    underline_color: Color,
    overline_color: Color,
    line_through_color: Color,
}

/// Flags for text display items.
pub struct TextDisplayItemFlags(u8);

impl TextDisplayItemFlags {
    pub fn new() -> TextDisplayItemFlags {
        TextDisplayItemFlags(0)
    }
}

// Whether underlining is forced on.
bitfield!(TextDisplayItemFlags, override_underline, set_override_underline, 0x01)
// Whether overlining is forced on.
bitfield!(TextDisplayItemFlags, override_overline, set_override_overline, 0x02)
// Whether line-through is forced on.
bitfield!(TextDisplayItemFlags, override_line_through, set_override_line_through, 0x04)

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
    style: SideOffsets2D<border_style::T>
}

/// Renders a line segment
pub struct LineDisplayItem<E> {
    base: BaseDisplayItem<E>,

    /// The line segment color.
    color: Color,

    /// The line segemnt style.
    style: border_style::T
}

pub struct ClipDisplayItem<E> {
    base: BaseDisplayItem<E>,
    child_list: ~[DisplayItem<E>],
    need_clip: bool
}

pub enum DisplayItemIterator<'a,E> {
    EmptyDisplayItemIterator,
    ParentDisplayItemIterator(VecIterator<'a,DisplayItem<E>>),
}

impl<'a,E> Iterator<&'a DisplayItem<E>> for DisplayItemIterator<'a,E> {
    #[inline]
    fn next(&mut self) -> Option<&'a DisplayItem<E>> {
        match *self {
            EmptyDisplayItemIterator => None,
            ParentDisplayItemIterator(ref mut subiterator) => subiterator.next(),
        }
    }
}

impl<E> DisplayItem<E> {
    /// Renders this display item into the given render context.
    fn draw_into_context(&self, render_context: &mut RenderContext) {
        match *self {
            SolidColorDisplayItemClass(ref solid_color) => {
                render_context.draw_solid_color(&solid_color.base.bounds, solid_color.color)
            }

            ClipDisplayItemClass(ref clip) => {
                if clip.need_clip {
                    render_context.draw_push_clip(&clip.base.bounds);
                }
                for item in clip.child_list.iter() {
                    (*item).draw_into_context(render_context);
                }
                if clip.need_clip {
                    render_context.draw_pop_clip();
                }
            }

            TextDisplayItemClass(ref text) => {
                debug!("Drawing text at {:?}.", text.base.bounds);

                // FIXME(pcwalton): Allocating? Why?
                let text_run = text.text_run.get();
                let font = render_context.font_ctx.get_font_by_descriptor(&text_run.font_descriptor).unwrap();

                let font_metrics = font.borrow().with(|font| {
                    font.metrics.clone()
                });
                let origin = text.base.bounds.origin;
                let baseline_origin = Point2D(origin.x, origin.y + font_metrics.ascent);
                font.borrow().with_mut(|font| {
                    font.draw_text_into_context(render_context,
                                                text.text_run.get(),
                                                &text.range,
                                                baseline_origin,
                                                text.text_color);
                });
                let width = text.base.bounds.size.width;
                let underline_size = font_metrics.underline_size;
                let underline_offset = font_metrics.underline_offset;
                let strikeout_size = font_metrics.strikeout_size;
                let strikeout_offset = font_metrics.strikeout_offset;

                if text_run.decoration.underline || text.flags.override_underline() {
                    let underline_y = baseline_origin.y - underline_offset;
                    let underline_bounds = Rect(Point2D(baseline_origin.x, underline_y),
                                                Size2D(width, underline_size));
                    render_context.draw_solid_color(&underline_bounds, text.underline_color);
                }
                if text_run.decoration.overline || text.flags.override_overline() {
                    let overline_bounds = Rect(Point2D(baseline_origin.x, origin.y),
                                               Size2D(width, underline_size));
                    render_context.draw_solid_color(&overline_bounds, text.overline_color);
                }
                if text_run.decoration.line_through || text.flags.override_line_through() {
                    let strikeout_y = baseline_origin.y - strikeout_offset;
                    let strikeout_bounds = Rect(Point2D(baseline_origin.x, strikeout_y),
                                                Size2D(width, strikeout_size));
                    render_context.draw_solid_color(&strikeout_bounds, text.line_through_color);
                }
            }

            ImageDisplayItemClass(ref image_item) => {
                debug!("Drawing image at {:?}.", image_item.base.bounds);

                render_context.draw_image(image_item.base.bounds, image_item.image.clone())
            }

            BorderDisplayItemClass(ref border) => {
                render_context.draw_border(&border.base.bounds,
                                           border.border,
                                           border.color,
                                           border.style)
            }

            LineDisplayItemClass(ref line) => {
                render_context.draw_line(&line.base.bounds,
                                          line.color,
                                          line.style)
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
                BorderDisplayItemClass(ref border) => transmute_region(&border.base),
                LineDisplayItemClass(ref line) => transmute_region(&line.base),
                ClipDisplayItemClass(ref clip) => transmute_region(&clip.base),
            }
        }
    }

    pub fn bounds(&self) -> Rect<Au> {
        self.base().bounds
    }

    pub fn children<'a>(&'a self) -> DisplayItemIterator<'a,E> {
        match *self {
            ClipDisplayItemClass(ref clip) => ParentDisplayItemIterator(clip.child_list.iter()),
            SolidColorDisplayItemClass(..) |
            TextDisplayItemClass(..) |
            ImageDisplayItemClass(..) |
            BorderDisplayItemClass(..) |
            LineDisplayItemClass(..) => EmptyDisplayItemIterator,
        }
    }

    pub fn debug_with_level(&self, level: uint) {
            let mut indent = ~"";
            for _ in range(0, level) {
                indent.push_str("| ")
            }
            debug!("{}+ {}", indent, self.debug_str());
            for child in self.children() {
                child.debug_with_level(level + 1);
            }
    }

    pub fn debug_str(&self) -> ~str {
        let class = match *self {
            SolidColorDisplayItemClass(_) => "SolidColor",
            TextDisplayItemClass(_) => "Text",
            ImageDisplayItemClass(_) => "Image",
            BorderDisplayItemClass(_) => "Border",
            LineDisplayItemClass(_) => "Line",
            ClipDisplayItemClass(_) => "Clip",
        };
        format!("{} @ {:?}", class, self.base().bounds)
    }
}

