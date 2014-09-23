/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Servo heavily uses display lists, which are retained-mode lists of rendering commands to
//! perform. Using a list instead of rendering elements in immediate mode allows transforms, hit
//! testing, and invalidation to be performed using the same primitives as painting. It also allows
//! Servo to aggressively cull invisible and out-of-bounds rendering elements, to reduce overdraw.
//! Finally, display lists allow tiles to be farmed out onto multiple CPUs and rendered in
//! parallel (although this benefit does not apply to GPU-based rendering).
//!
//! Display items describe relatively high-level drawing operations (for example, entire borders
//! and shadows instead of lines and blur operations), to reduce the amount of allocation required.
//! They are therefore not exactly analogous to constructs like Skia pictures, which consist of
//! low-level drawing primitives.

use color::Color;
use render_context::RenderContext;
use text::glyph::CharIndex;
use text::TextRun;

use collections::dlist::DList;
use collections::dlist;
use geom::{Point2D, Rect, SideOffsets2D, Size2D, Matrix2D};
use libc::uintptr_t;
use servo_net::image::base::Image;
use servo_util::geometry::Au;
use servo_util::range::Range;
use std::fmt;
use std::mem;
use std::slice::Items;
use style::computed_values::border_style;
use sync::Arc;
use std::num::Zero;
use std::ptr;

use azure::AzFloat;
use azure::scaled_font::ScaledFont;
use azure::azure_hl::ColorPattern;

pub mod optimizer;

/// An opaque handle to a node. The only safe operation that can be performed on this node is to
/// compare it to another opaque handle or to another node.
///
/// Because the script task's GC does not trace layout, node data cannot be safely stored in layout
/// data structures. Also, layout code tends to be faster when the DOM is not being accessed, for
/// locality reasons. Using `OpaqueNode` enforces this invariant.
#[deriving(Clone, PartialEq)]
pub struct OpaqueNode(pub uintptr_t);

impl OpaqueNode {
    /// Returns the address of this node, for debugging purposes.
    pub fn id(&self) -> uintptr_t {
        let OpaqueNode(pointer) = *self;
        pointer
    }
}

trait ScaledFontExtensionMethods {
    fn draw_text_into_context(&self,
                              rctx: &RenderContext,
                              run: &Box<TextRun>,
                              range: &Range<CharIndex>,
                              baseline_origin: Point2D<Au>,
                              color: Color,
                              antialias: bool);
}

impl ScaledFontExtensionMethods for ScaledFont {
    fn draw_text_into_context(&self,
                              rctx: &RenderContext,
                              run: &Box<TextRun>,
                              range: &Range<CharIndex>,
                              baseline_origin: Point2D<Au>,
                              color: Color,
                              antialias: bool) {
        use libc::types::common::c99::uint32_t;
        use azure::{struct__AzDrawOptions,
                    struct__AzGlyph,
                    struct__AzGlyphBuffer,
                    struct__AzPoint};
        use azure::azure::{AzDrawTargetFillGlyphs};

        let target = rctx.get_draw_target();
        let pattern = ColorPattern::new(color);
        let azure_pattern = pattern.azure_color_pattern;
        assert!(azure_pattern.is_not_null());

        let fields = if antialias {
            0x0200
        } else {
            0
        };

        let mut options = struct__AzDrawOptions {
            mAlpha: 1f64 as AzFloat,
            fields: fields,
        };

        let mut origin = baseline_origin.clone();
        let mut azglyphs = vec!();
        azglyphs.reserve(range.length().to_uint());

        for (glyphs, _offset, slice_range) in run.iter_slices_for_range(range) {
            for (_i, glyph) in glyphs.iter_glyphs_for_char_range(&slice_range) {
                let glyph_advance = glyph.advance();
                let glyph_offset = glyph.offset().unwrap_or(Zero::zero());

                let azglyph = struct__AzGlyph {
                    mIndex: glyph.id() as uint32_t,
                    mPosition: struct__AzPoint {
                        x: (origin.x + glyph_offset.x).to_nearest_px() as AzFloat,
                        y: (origin.y + glyph_offset.y).to_nearest_px() as AzFloat
                    }
                };
                origin = Point2D(origin.x + glyph_advance, origin.y);
                azglyphs.push(azglyph)
            };
        }

        let azglyph_buf_len = azglyphs.len();
        if azglyph_buf_len == 0 { return; } // Otherwise the Quartz backend will assert.

        let mut glyphbuf = struct__AzGlyphBuffer {
            mGlyphs: azglyphs.as_mut_ptr(),
            mNumGlyphs: azglyph_buf_len as uint32_t
        };

        unsafe {
            // TODO(Issue #64): this call needs to move into azure_hl.rs
            AzDrawTargetFillGlyphs(target.azure_draw_target,
                                   self.get_ref(),
                                   &mut glyphbuf,
                                   azure_pattern,
                                   &mut options,
                                   ptr::null_mut());
        }
    }
}

/// "Steps" as defined by CSS 2.1 § E.2.
#[deriving(Clone, PartialEq)]
pub enum StackingLevel {
    /// The border and backgrounds for the root of this stacking context: steps 1 and 2.
    BackgroundAndBordersStackingLevel,
    /// Borders and backgrounds for block-level descendants: step 4.
    BlockBackgroundsAndBordersStackingLevel,
    /// Floats: step 5. These are treated as pseudo-stacking contexts.
    FloatStackingLevel,
    /// All other content.
    ContentStackingLevel,
    /// Positioned descendant stacking contexts, along with their `z-index` levels.
    ///
    /// TODO(pcwalton): `z-index` should be the actual CSS property value in order to handle
    /// `auto`, not just an integer.
    PositionedDescendantStackingLevel(i32)
}

impl StackingLevel {
    pub fn from_background_and_border_level(level: BackgroundAndBorderLevel) -> StackingLevel {
        match level {
            RootOfStackingContextLevel => BackgroundAndBordersStackingLevel,
            BlockLevel => BlockBackgroundsAndBordersStackingLevel,
            ContentLevel => ContentStackingLevel,
        }
    }
}

struct StackingContext {
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
    /// `auto`, not just an integer.
    pub positioned_descendants: Vec<(i32, DisplayList)>,
}

impl StackingContext {
    /// Creates a stacking context from a display list.
    fn new(list: DisplayList) -> StackingContext {
        let DisplayList {
            list: list
        } = list;

        let mut stacking_context = StackingContext {
            background_and_borders: DisplayList::new(),
            block_backgrounds_and_borders: DisplayList::new(),
            floats: DisplayList::new(),
            content: DisplayList::new(),
            positioned_descendants: Vec::new(),
        };

        for item in list.into_iter() {
            match item {
                ClipDisplayItemClass(box ClipDisplayItem {
                    base: base,
                    children: sublist
                }) => {
                    let sub_stacking_context = StackingContext::new(sublist);
                    stacking_context.merge_with_clip(sub_stacking_context, &base.bounds, base.node)
                }
                item => {
                    match item.base().level {
                        BackgroundAndBordersStackingLevel => {
                            stacking_context.background_and_borders.push(item)
                        }
                        BlockBackgroundsAndBordersStackingLevel => {
                            stacking_context.block_backgrounds_and_borders.push(item)
                        }
                        FloatStackingLevel => stacking_context.floats.push(item),
                        ContentStackingLevel => stacking_context.content.push(item),
                        PositionedDescendantStackingLevel(z_index) => {
                            match stacking_context.positioned_descendants
                                                  .iter_mut()
                                                  .find(|& &(z, _)| z_index == z) {
                                Some(&(_, ref mut my_list)) => {
                                    my_list.push(item);
                                    continue
                                }
                                None => {}
                            }

                            let mut new_list = DisplayList::new();
                            new_list.list.push(item);
                            stacking_context.positioned_descendants.push((z_index, new_list))
                        }
                    }
                }
            }
        }

        stacking_context
    }

    /// Merges another stacking context into this one, with the given clipping rectangle and DOM
    /// node that supplies it.
    fn merge_with_clip(&mut self,
                       other: StackingContext,
                       clip_rect: &Rect<Au>,
                       clipping_dom_node: OpaqueNode) {
        let StackingContext {
            background_and_borders,
            block_backgrounds_and_borders,
            floats,
            content,
            positioned_descendants: positioned_descendants
        } = other;

        let push = |destination: &mut DisplayList, source: DisplayList, level| {
            if !source.is_empty() {
                let base = BaseDisplayItem::new(*clip_rect, clipping_dom_node, level);
                destination.push(ClipDisplayItemClass(box ClipDisplayItem::new(base, source)))
            }
        };

        push(&mut self.background_and_borders,
             background_and_borders,
             BackgroundAndBordersStackingLevel);
        push(&mut self.block_backgrounds_and_borders,
             block_backgrounds_and_borders,
             BlockBackgroundsAndBordersStackingLevel);
        push(&mut self.floats, floats, FloatStackingLevel);
        push(&mut self.content, content, ContentStackingLevel);

        for (z_index, list) in positioned_descendants.into_iter() {
            match self.positioned_descendants
                      .iter_mut()
                      .find(|& &(existing_z_index, _)| z_index == existing_z_index) {
                Some(&(_, ref mut existing_list)) => {
                    push(existing_list, list, PositionedDescendantStackingLevel(z_index));
                    continue
                }
                None => {}
            }

            let mut new_list = DisplayList::new();
            push(&mut new_list, list, PositionedDescendantStackingLevel(z_index));
            self.positioned_descendants.push((z_index, new_list));
        }
    }
}

/// Which level to place backgrounds and borders in.
pub enum BackgroundAndBorderLevel {
    RootOfStackingContextLevel,
    BlockLevel,
    ContentLevel,
}

/// A list of rendering operations to be performed.
#[deriving(Clone)]
pub struct DisplayList {
    pub list: DList<DisplayItem>,
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
            list: DList::new(),
        }
    }

    /// Appends the given item to the display list.
    pub fn push(&mut self, item: DisplayItem) {
        self.list.push(item)
    }

    /// Appends the given display list to this display list, consuming the other display list in
    /// the process.
    pub fn push_all_move(&mut self, other: DisplayList) {
        self.list.append(other.list)
    }

    pub fn debug(&self) {
        if log_enabled!(::log::DEBUG) {
            for item in self.list.iter() {
                item.debug_with_level(0);
            }
        }
    }

    /// Draws the display list into the given render context. The display list must be flattened
    /// first for correct painting.
    pub fn draw_into_context(&self, render_context: &mut RenderContext,
                             current_transform: &Matrix2D<AzFloat>) {
        debug!("Beginning display list.");
        for item in self.list.iter() {
            item.draw_into_context(render_context, current_transform)
        }
        debug!("Ending display list.");
    }

    /// Returns a preorder iterator over the given display list.
    pub fn iter<'a>(&'a self) -> DisplayItemIterator<'a> {
        ParentDisplayItemIterator(self.list.iter())
    }

    /// Returns true if this list is empty and false otherwise.
    fn is_empty(&self) -> bool {
        self.list.len() == 0
    }

    /// Flattens a display list into a display list with a single stacking level according to the
    /// steps in CSS 2.1 § E.2.
    ///
    /// This must be called before `draw_into_context()` is for correct results.
    pub fn flatten(self, resulting_level: StackingLevel) -> DisplayList {
        // TODO(pcwalton): Sort positioned children according to z-index.

        let mut result = DisplayList::new();
        let StackingContext {
            background_and_borders,
            block_backgrounds_and_borders,
            floats,
            content,
            positioned_descendants: mut positioned_descendants
        } = StackingContext::new(self);

        // Steps 1 and 2: Borders and background for the root.
        result.push_all_move(background_and_borders);

        // TODO(pcwalton): Sort positioned children according to z-index.

        // Step 3: Positioned descendants with negative z-indices.
        for &(ref mut z_index, ref mut list) in positioned_descendants.iter_mut() {
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
        for &(ref mut z_index, ref mut list) in positioned_descendants.iter_mut() {
            if *z_index >= 0 {
                result.push_all_move(mem::replace(list, DisplayList::new()))
            }
        }

        // TODO(pcwalton): Step 10: Outlines.

        result.set_stacking_level(resulting_level);
        result
    }

    /// Sets the stacking level for this display list and all its subitems.
    fn set_stacking_level(&mut self, new_level: StackingLevel) {
        for item in self.list.iter_mut() {
            item.mut_base().level = new_level;
            match item.mut_sublist() {
                None => {}
                Some(sublist) => sublist.set_stacking_level(new_level),
            }
        }
    }
}

/// One drawing command in the list.
#[deriving(Clone)]
pub enum DisplayItem {
    SolidColorDisplayItemClass(Box<SolidColorDisplayItem>),
    TextDisplayItemClass(Box<TextDisplayItem>),
    ImageDisplayItemClass(Box<ImageDisplayItem>),
    BorderDisplayItemClass(Box<BorderDisplayItem>),
    LineDisplayItemClass(Box<LineDisplayItem>),
    ClipDisplayItemClass(Box<ClipDisplayItem>),

    /// A pseudo-display item that exists only so that queries like `ContentBoxQuery` and
    /// `ContentBoxesQuery` can be answered.
    ///
    /// FIXME(pcwalton): This is really bogus. Those queries should not consult the display list
    /// but should instead consult the flow/box tree.
    PseudoDisplayItemClass(Box<BaseDisplayItem>),
}

/// Information common to all display items.
#[deriving(Clone)]
pub struct BaseDisplayItem {
    /// The boundaries of the display item.
    ///
    /// TODO: Which coordinate system should this use?
    pub bounds: Rect<Au>,

    /// The originating DOM node.
    pub node: OpaqueNode,

    /// The stacking level in which this display item lives.
    pub level: StackingLevel,
}

impl BaseDisplayItem {
    pub fn new(bounds: Rect<Au>, node: OpaqueNode, level: StackingLevel) -> BaseDisplayItem {
        BaseDisplayItem {
            bounds: bounds,
            node: node,
            level: level,
        }
    }
}

/// Renders a solid color.
#[deriving(Clone)]
pub struct SolidColorDisplayItem {
    pub base: BaseDisplayItem,
    pub color: Color,
}

/// Renders text.
#[deriving(Clone)]
pub struct TextDisplayItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    /// The text run.
    pub text_run: Arc<Box<TextRun>>,

    /// The range of text within the text run.
    pub range: Range<CharIndex>,

    /// The color of the text.
    pub text_color: Color,

    pub baseline_origin: Point2D<Au>,
    pub orientation: TextOrientation,
}

#[deriving(Clone, Eq, PartialEq)]
pub enum TextOrientation {
    Upright,
    SidewaysLeft,
    SidewaysRight,
}

/// Renders an image.
#[deriving(Clone)]
pub struct ImageDisplayItem {
    pub base: BaseDisplayItem,
    pub image: Arc<Box<Image>>,

    /// The dimensions to which the image display item should be stretched. If this is smaller than
    /// the bounds of this display item, then the image will be repeated in the appropriate
    /// direction to tile the entire bounds.
    pub stretch_size: Size2D<Au>,
}

/// Renders a border.
#[deriving(Clone)]
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
#[deriving(Clone)]
pub struct LineDisplayItem {
    pub base: BaseDisplayItem,

    /// The line segment color.
    pub color: Color,

    /// The line segment style.
    pub style: border_style::T
}

/// Clips a list of child display items to this display item's boundaries.
#[deriving(Clone)]
pub struct ClipDisplayItem {
    /// The base information.
    pub base: BaseDisplayItem,

    /// The child nodes.
    pub children: DisplayList,
}

impl ClipDisplayItem {
    pub fn new(base: BaseDisplayItem, children: DisplayList) -> ClipDisplayItem {
        ClipDisplayItem {
            base: base,
            children: children,
        }
    }
}

pub enum DisplayItemIterator<'a> {
    EmptyDisplayItemIterator,
    ParentDisplayItemIterator(dlist::Items<'a,DisplayItem>),
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
    fn draw_into_context(&self, render_context: &mut RenderContext,
                         current_transform: &Matrix2D<AzFloat>) {
        // This should have been flattened to the content stacking level first.
        assert!(self.base().level == ContentStackingLevel);

        match *self {
            SolidColorDisplayItemClass(ref solid_color) => {
                render_context.draw_solid_color(&solid_color.base.bounds, solid_color.color)
            }

            ClipDisplayItemClass(ref clip) => {
                render_context.draw_push_clip(&clip.base.bounds);
                for item in clip.children.iter() {
                    (*item).draw_into_context(render_context, current_transform);
                }
                render_context.draw_pop_clip();
            }

            TextDisplayItemClass(ref text) => {
                debug!("Drawing text at {}.", text.base.bounds);

                // Optimization: Don’t set a transform matrix for upright text,
                // and pass a strart point to `draw_text_into_context`.
                // For sideways text, it’s easier to do the rotation such that its center
                // (the baseline’s start point) is at (0, 0) coordinates.
                let baseline_origin = match text.orientation {
                    Upright => text.baseline_origin,
                    SidewaysLeft => {
                        let x = text.baseline_origin.x.to_nearest_px() as AzFloat;
                        let y = text.baseline_origin.y.to_nearest_px() as AzFloat;
                        render_context.draw_target.set_transform(&current_transform.mul(
                            &Matrix2D::new(
                                0., -1.,
                                1., 0.,
                                x, y
                            )
                        ));
                        Zero::zero()
                    },
                    SidewaysRight => {
                        let x = text.baseline_origin.x.to_nearest_px() as AzFloat;
                        let y = text.baseline_origin.y.to_nearest_px() as AzFloat;
                        render_context.draw_target.set_transform(&current_transform.mul(
                            &Matrix2D::new(
                                0., 1.,
                                -1., 0.,
                                x, y
                            )
                        ));
                        Zero::zero()
                    }
                };

                render_context.font_ctx.get_render_font_from_template(
                    &text.text_run.font_template,
                    text.text_run.actual_pt_size,
                    render_context.opts.render_backend
                ).borrow().draw_text_into_context(
                    render_context,
                    &*text.text_run,
                    &text.range,
                    baseline_origin,
                    text.text_color,
                    render_context.opts.enable_text_antialiasing
                );

                // Undo the transform, only when we did one.
                if text.orientation != Upright {
                    render_context.draw_target.set_transform(current_transform)
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

            PseudoDisplayItemClass(_) => {}
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
            PseudoDisplayItemClass(ref base) => &**base,
        }
    }

    pub fn mut_base<'a>(&'a mut self) -> &'a mut BaseDisplayItem {
        match *self {
            SolidColorDisplayItemClass(ref mut solid_color) => &mut solid_color.base,
            TextDisplayItemClass(ref mut text) => &mut text.base,
            ImageDisplayItemClass(ref mut image_item) => &mut image_item.base,
            BorderDisplayItemClass(ref mut border) => &mut border.base,
            LineDisplayItemClass(ref mut line) => &mut line.base,
            ClipDisplayItemClass(ref mut clip) => &mut clip.base,
            PseudoDisplayItemClass(ref mut base) => &mut **base,
        }
    }

    pub fn bounds(&self) -> Rect<Au> {
        self.base().bounds
    }

    pub fn children<'a>(&'a self) -> DisplayItemIterator<'a> {
        match *self {
            ClipDisplayItemClass(ref clip) => ParentDisplayItemIterator(clip.children.list.iter()),
            SolidColorDisplayItemClass(..) |
            TextDisplayItemClass(..) |
            ImageDisplayItemClass(..) |
            BorderDisplayItemClass(..) |
            LineDisplayItemClass(..) |
            PseudoDisplayItemClass(..) => EmptyDisplayItemIterator,
        }
    }

    /// Returns a mutable reference to the sublist contained within this display list item, if any.
    fn mut_sublist<'a>(&'a mut self) -> Option<&'a mut DisplayList> {
        match *self {
            ClipDisplayItemClass(ref mut clip) => Some(&mut clip.children),
            SolidColorDisplayItemClass(..) |
            TextDisplayItemClass(..) |
            ImageDisplayItemClass(..) |
            BorderDisplayItemClass(..) |
            LineDisplayItemClass(..) |
            PseudoDisplayItemClass(..) => None,
        }
    }

    pub fn debug_with_level(&self, level: uint) {
        let mut indent = String::new();
        for _ in range(0, level) {
            indent.push_str("| ")
        }
        debug!("{}+ {}", indent, self);
        for child in self.children() {
            child.debug_with_level(level + 1);
        }
    }
}

impl fmt::Show for DisplayItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} @ {} ({:x})",
            match *self {
                SolidColorDisplayItemClass(_) => "SolidColor",
                TextDisplayItemClass(_) => "Text",
                ImageDisplayItemClass(_) => "Image",
                BorderDisplayItemClass(_) => "Border",
                LineDisplayItemClass(_) => "Line",
                ClipDisplayItemClass(_) => "Clip",
                PseudoDisplayItemClass(_) => "Pseudo",
            },
            self.base().bounds,
            self.base().node.id(),
        )
    }
}
