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

use geom::{Point2D, Rect, SideOffsets2D, Size2D};
use libc::uintptr_t;
use servo_net::image::base::Image;
use servo_util::geometry::Au;
use servo_util::range::Range;
use servo_util::smallvec::{SmallVec, SmallVec0, SmallVecIterator};
use std::mem;
use std::slice::Items;
use style::computed_values::border_style;
use sync::Arc;

/// An opaque handle to a node. The only safe operation that can be performed on this node is to
/// compare it to another opaque handle or to another node.
///
/// Because the script task's GC does not trace layout, node data cannot be safely stored in layout
/// data structures. Also, layout code tends to be faster when the DOM is not being accessed, for
/// locality reasons. Using `OpaqueNode` enforces this invariant.
#[deriving(Clone, Eq)]
pub struct OpaqueNode(pub uintptr_t);

impl OpaqueNode {
    /// Returns the address of this node, for debugging purposes.
    pub fn id(&self) -> uintptr_t {
        let OpaqueNode(pointer) = *self;
        pointer
    }
}

/// A stacking context. See CSS 2.1 § E.2. "Steps" below refer to steps in that section of the
/// specification.
///
/// TODO(pcwalton): Outlines.
pub struct StackingContext {
    /// The border and backgrounds for the root of this stacking context: steps 1 and 2.
    pub background_and_borders: DisplayList,
    /// Borders and backgrounds for block-level descendants: step 4.
    pub block_backgrounds_and_borders: DisplayList,
    /// Floats: step 5. These are treated as pseudo-stacking contexts.
    pub floats: DisplayList,
    /// All other content.
    pub content: DisplayList,

    /// Positioned descendant stacking contexts, along with their `z-index` levels.
    ///
    /// TODO(pcwalton): `z-index` should be the actual CSS property value in order to handle
    /// `auto`, not just an integer. In this case we should store an actual stacking context, not
    /// a flattened display list.
    pub positioned_descendants: SmallVec0<(int, DisplayList)>,
}

impl StackingContext {
    pub fn new() -> StackingContext {
        StackingContext {
            background_and_borders: DisplayList::new(),
            block_backgrounds_and_borders: DisplayList::new(),
            floats: DisplayList::new(),
            content: DisplayList::new(),
            positioned_descendants: SmallVec0::new(),
        }
    }

    pub fn list_for_background_and_border_level<'a>(
                                                &'a mut self,
                                                level: BackgroundAndBorderLevel)
                                                -> &'a mut DisplayList {
        match level {
            RootOfStackingContextLevel => &mut self.background_and_borders,
            BlockLevel => &mut self.block_backgrounds_and_borders,
            ContentLevel => &mut self.content,
        }
    }

    /// Flattens a stacking context into a display list according to the steps in CSS 2.1 § E.2.
    pub fn flatten(self) -> DisplayList {
        // Steps 1 and 2: Borders and background for the root.
        let StackingContext {
            background_and_borders: mut result,
            block_backgrounds_and_borders,
            floats,
            content,
            positioned_descendants: mut positioned_descendants
        } = self;

        // TODO(pcwalton): Sort positioned children according to z-index.

        // Step 3: Positioned descendants with negative z-indices.
        for &(ref mut z_index, ref mut list) in positioned_descendants.mut_iter() {
            if *z_index < 0 {
                result.push_all_move(mem::replace(list, DisplayList::new()))
            }
        }

        // Step 4: Block backgrounds and borders.
        result.push_all_move(block_backgrounds_and_borders);

        // Step 5: Floats.
        result.push_all_move(floats);

        // TODO(pcwalton): Step 6: Inlines that generate stacking contexts.

        // Step 7: Content.
        result.push_all_move(content);

        // Steps 8 and 9: Positioned descendants with nonnegative z-indices.
        for &(ref mut z_index, ref mut list) in positioned_descendants.mut_iter() {
            if *z_index >= 0 {
                result.push_all_move(mem::replace(list, DisplayList::new()))
            }
        }

        // TODO(pcwalton): Step 10: Outlines.

        result
    }
}

/// Which level to place backgrounds and borders in.
pub enum BackgroundAndBorderLevel {
    RootOfStackingContextLevel,
    BlockLevel,
    ContentLevel,
}

/// A list of rendering operations to be performed.
pub struct DisplayList {
    pub list: SmallVec0<DisplayItem>,
}

pub enum DisplayListIterator<'a> {
    EmptyDisplayListIterator,
    ParentDisplayListIterator(Items<'a,DisplayList>),
}

impl<'a> Iterator<&'a DisplayList> for DisplayListIterator<'a> {
    #[inline]
    fn next(&mut self) -> Option<&'a DisplayList> {
        match *self {
            EmptyDisplayListIterator => None,
            ParentDisplayListIterator(ref mut subiterator) => subiterator.next(),
        }
    }
}

impl DisplayList {
    /// Creates a new display list.
    pub fn new() -> DisplayList {
        DisplayList {
            list: SmallVec0::new(),
        }
    }

    fn dump(&self) {
        for item in self.list.iter() {
            item.debug_with_level(0);
        }
    }

    /// Appends the given item to the display list.
    pub fn push(&mut self, item: DisplayItem) {
        self.list.push(item)
    }

    /// Appends the given display list to this display list, consuming the other display list in
    /// the process.
    pub fn push_all_move(&mut self, other: DisplayList) {
        self.list.push_all_move(other.list)
    }

    /// Draws the display list into the given render context.
    pub fn draw_into_context(&self, render_context: &mut RenderContext) {
        debug!("Beginning display list.");
        for item in self.list.iter() {
            item.draw_into_context(render_context)
        }
        debug!("Ending display list.");
    }

    /// Returns a preorder iterator over the given display list.
    pub fn iter<'a>(&'a self) -> DisplayItemIterator<'a> {
        ParentDisplayItemIterator(self.list.iter())
    }
}

/// One drawing command in the list.
pub enum DisplayItem {
    SolidColorDisplayItemClass(~SolidColorDisplayItem),
    TextDisplayItemClass(~TextDisplayItem),
    ImageDisplayItemClass(~ImageDisplayItem),
    BorderDisplayItemClass(~BorderDisplayItem),
    LineDisplayItemClass(~LineDisplayItem),
    ClipDisplayItemClass(~ClipDisplayItem)
}

/// Information common to all display items.
pub struct BaseDisplayItem {
    /// The boundaries of the display item.
    ///
    /// TODO: Which coordinate system should this use?
    pub bounds: Rect<Au>,

    /// The originating DOM node.
    pub node: OpaqueNode,
}

/// Renders a solid color.
pub struct SolidColorDisplayItem {
    pub base: BaseDisplayItem,
    pub color: Color,
}

/// Renders text.
pub struct TextDisplayItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    /// The text run.
    pub text_run: Arc<~TextRun>,

    /// The range of text within the text run.
    pub range: Range,

    /// The color of the text.
    pub text_color: Color,

    /// A bitfield of flags for text display items.
    pub flags: TextDisplayItemFlags,

    /// The color of text-decorations
    pub underline_color: Color,
    pub overline_color: Color,
    pub line_through_color: Color,
}

/// Flags for text display items.
pub struct TextDisplayItemFlags(pub u8);

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
pub struct ImageDisplayItem {
    pub base: BaseDisplayItem,
    pub image: Arc<~Image>,

    /// The dimensions to which the image display item should be stretched. If this is smaller than
    /// the bounds of this display item, then the image will be repeated in the appropriate
    /// direction to tile the entire bounds.
    pub stretch_size: Size2D<Au>,
}

/// Renders a border.
pub struct BorderDisplayItem {
    pub base: BaseDisplayItem,

    /// The border widths
    pub border: SideOffsets2D<Au>,

    /// The border colors.
    pub color: SideOffsets2D<Color>,

    /// The border styles.
    pub style: SideOffsets2D<border_style::T>
}

/// Renders a line segment.
pub struct LineDisplayItem {
    pub base: BaseDisplayItem,

    /// The line segment color.
    pub color: Color,

    /// The line segment style.
    pub style: border_style::T
}

pub struct ClipDisplayItem {
    pub base: BaseDisplayItem,
    pub child_list: SmallVec0<DisplayItem>,
    pub need_clip: bool
}

pub enum DisplayItemIterator<'a> {
    EmptyDisplayItemIterator,
    ParentDisplayItemIterator(SmallVecIterator<'a,DisplayItem>),
}

impl<'a> Iterator<&'a DisplayItem> for DisplayItemIterator<'a> {
    #[inline]
    fn next(&mut self) -> Option<&'a DisplayItem> {
        match *self {
            EmptyDisplayItemIterator => None,
            ParentDisplayItemIterator(ref mut subiterator) => subiterator.next(),
        }
    }
}

impl DisplayItem {
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
                let text_run = text.text_run.clone();
                let font = render_context.font_ctx.get_font_by_descriptor(&text_run.font_descriptor).unwrap();

                let font_metrics = {
                    font.borrow().metrics.clone()
                };
                let origin = text.base.bounds.origin;
                let baseline_origin = Point2D(origin.x, origin.y + font_metrics.ascent);
                {
                    font.borrow_mut().draw_text_into_context(render_context,
                                                             &*text.text_run,
                                                             &text.range,
                                                             baseline_origin,
                                                             text.text_color);
                }
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

                let mut y_offset = Au(0);
                while y_offset < image_item.base.bounds.size.height {
                    let mut x_offset = Au(0);
                    while x_offset < image_item.base.bounds.size.width {
                        let mut bounds = image_item.base.bounds;
                        bounds.origin.x = bounds.origin.x + x_offset;
                        bounds.origin.y = bounds.origin.y + y_offset;
                        bounds.size = image_item.stretch_size;

                        render_context.draw_image(bounds, image_item.image.clone());

                        x_offset = x_offset + image_item.stretch_size.width;
                    }

                    y_offset = y_offset + image_item.stretch_size.height;
                }
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

    pub fn base<'a>(&'a self) -> &'a BaseDisplayItem {
        match *self {
            SolidColorDisplayItemClass(ref solid_color) => &solid_color.base,
            TextDisplayItemClass(ref text) => &text.base,
            ImageDisplayItemClass(ref image_item) => &image_item.base,
            BorderDisplayItemClass(ref border) => &border.base,
            LineDisplayItemClass(ref line) => &line.base,
            ClipDisplayItemClass(ref clip) => &clip.base,
        }
    }

    pub fn bounds(&self) -> Rect<Au> {
        self.base().bounds
    }

    pub fn children<'a>(&'a self) -> DisplayItemIterator<'a> {
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

