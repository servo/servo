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

use azure::azure::AzFloat;
use collections::Deque;
use collections::dlist::{mod, DList};
use geom::{Point2D, Rect, SideOffsets2D, Size2D, Matrix2D};
use libc::uintptr_t;
use servo_net::image::base::Image;
use servo_util::dlist as servo_dlist;
use servo_util::geometry::Au;
use servo_util::range::Range;
use std::fmt;
use std::slice::Items;
use style::computed_values::border_style;
use sync::Arc;

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

/// "Steps" as defined by CSS 2.1 § E.2.
#[deriving(Clone, PartialEq, Show)]
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
    #[inline]
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
    pub positioned_descendants: Vec<(i32, DisplayList)>,
}

impl StackingContext {
    /// Creates a new empty stacking context.
    #[inline]
    fn new() -> StackingContext {
        StackingContext {
            background_and_borders: DisplayList::new(),
            block_backgrounds_and_borders: DisplayList::new(),
            floats: DisplayList::new(),
            content: DisplayList::new(),
            positioned_descendants: Vec::new(),
        }
    }

    /// Initializes a stacking context from a display list, consuming that display list in the
    /// process.
    fn init_from_list(&mut self, list: &mut DisplayList) {
        while !list.list.is_empty() {
            let mut head = DisplayList::from_list(servo_dlist::split(&mut list.list));
            match head.front().unwrap().base().level {
                BackgroundAndBordersStackingLevel => {
                    self.background_and_borders.append_from(&mut head)
                }
                BlockBackgroundsAndBordersStackingLevel => {
                    self.block_backgrounds_and_borders.append_from(&mut head)
                }
                FloatStackingLevel => self.floats.append_from(&mut head),
                ContentStackingLevel => self.content.append_from(&mut head),
                PositionedDescendantStackingLevel(z_index) => {
                    match self.positioned_descendants.iter_mut().find(|& &(z, _)| z_index == z) {
                        Some(&(_, ref mut my_list)) => {
                            my_list.append_from(&mut head);
                            continue
                        }
                        None => {}
                    }

                    self.positioned_descendants.push((z_index, head))
                }
            }
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
#[deriving(Clone, Show)]
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
    #[inline]
    pub fn new() -> DisplayList {
        DisplayList {
            list: DList::new(),
        }
    }

    /// Creates a new display list from the given list of display items.
    fn from_list(list: DList<DisplayItem>) -> DisplayList {
        DisplayList {
            list: list,
        }
    }

    /// Appends the given item to the display list.
    #[inline]
    pub fn push(&mut self, item: DisplayItem) {
        self.list.push(item);
    }

    /// Appends the items in the given display list to this one, removing them in the process.
    #[inline]
    pub fn append_from(&mut self, other: &mut DisplayList) {
        servo_dlist::append_from(&mut self.list, &mut other.list)
    }

    /// Returns the first display item in this list.
    #[inline]
    fn front(&self) -> Option<&DisplayItem> {
        self.list.front()
    }

    pub fn debug(&self) {
        for item in self.list.iter() {
            item.debug_with_level(0);
        }
    }

    /// Draws the display list into the given render context. The display list must be flattened
    /// first for correct painting.
    pub fn draw_into_context(&self,
                             render_context: &mut RenderContext,
                             current_transform: &Matrix2D<AzFloat>,
                             current_clip_stack: &mut Vec<Rect<Au>>) {
        debug!("Beginning display list.");
        for item in self.list.iter() {
            item.draw_into_context(render_context, current_transform, current_clip_stack)
        }
        debug!("Ending display list.");
    }

    /// Returns a preorder iterator over the given display list.
    #[inline]
    pub fn iter<'a>(&'a self) -> DisplayItemIterator<'a> {
        ParentDisplayItemIterator(self.list.iter())
    }

    /// Flattens a display list into a display list with a single stacking level according to the
    /// steps in CSS 2.1 § E.2.
    ///
    /// This must be called before `draw_into_context()` is for correct results.
    pub fn flatten(&mut self, resulting_level: StackingLevel) {
        // Fast paths:
        if self.list.len() == 0 {
            return
        }
        if self.list.len() == 1 {
            self.set_stacking_level(resulting_level);
            return
        }

        let mut stacking_context = StackingContext::new();
        stacking_context.init_from_list(self);
        debug_assert!(self.list.is_empty());

        // Steps 1 and 2: Borders and background for the root.
        self.append_from(&mut stacking_context.background_and_borders);

        // Sort positioned children according to z-index.
        stacking_context.positioned_descendants.sort_by(|&(z_index_a, _), &(z_index_b, _)| {
            z_index_a.cmp(&z_index_b)
        });

        // Step 3: Positioned descendants with negative z-indices.
        for &(ref mut z_index, ref mut list) in stacking_context.positioned_descendants.iter_mut() {
            if *z_index < 0 {
                self.append_from(list)
            }
        }

        // Step 4: Block backgrounds and borders.
        self.append_from(&mut stacking_context.block_backgrounds_and_borders);

        // Step 5: Floats.
        self.append_from(&mut stacking_context.floats);

        // TODO(pcwalton): Step 6: Inlines that generate stacking contexts.

        // Step 7: Content.
        self.append_from(&mut stacking_context.content);

        // Steps 8 and 9: Positioned descendants with nonnegative z-indices.
        for &(ref mut z_index, ref mut list) in stacking_context.positioned_descendants.iter_mut() {
            if *z_index >= 0 {
                self.append_from(list)
            }
        }

        // TODO(pcwalton): Step 10: Outlines.

        self.set_stacking_level(resulting_level);
    }

    /// Sets the stacking level for this display list and all its subitems.
    fn set_stacking_level(&mut self, new_level: StackingLevel) {
        for item in self.list.iter_mut() {
            item.mut_base().level = new_level;
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
    /// The boundaries of the display item, in layer coordinates.
    pub bounds: Rect<Au>,

    /// The originating DOM node.
    pub node: OpaqueNode,

    /// The stacking level in which this display item lives.
    pub level: StackingLevel,

    /// The rectangle to clip to.
    ///
    /// TODO(pcwalton): Eventually, to handle `border-radius`, this will (at least) need to grow
    /// the ability to describe rounded rectangles.
    pub clip_rect: Rect<Au>,
}

impl BaseDisplayItem {
    #[inline(always)]
    pub fn new(bounds: Rect<Au>, node: OpaqueNode, level: StackingLevel, clip_rect: Rect<Au>)
               -> BaseDisplayItem {
        BaseDisplayItem {
            bounds: bounds,
            node: node,
            level: level,
            clip_rect: clip_rect,
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
    fn draw_into_context(&self,
                         render_context: &mut RenderContext,
                         current_transform: &Matrix2D<AzFloat>,
                         current_clip_stack: &mut Vec<Rect<Au>>) {
        // This should have been flattened to the content stacking level first.
        assert!(self.base().level == ContentStackingLevel);

        // TODO(pcwalton): This will need some tweaking to deal with more complex clipping regions.
        let clip_rect = &self.base().clip_rect;
        if current_clip_stack.len() == 0 || current_clip_stack.last().unwrap() != clip_rect {
            while current_clip_stack.len() != 0 {
                render_context.draw_pop_clip();
                drop(current_clip_stack.pop());
            }
            render_context.draw_push_clip(clip_rect);
            current_clip_stack.push(*clip_rect);
        }

        match *self {
            SolidColorDisplayItemClass(ref solid_color) => {
                render_context.draw_solid_color(&solid_color.base.bounds, solid_color.color)
            }

            TextDisplayItemClass(ref text) => {
                debug!("Drawing text at {}.", text.base.bounds);
                render_context.draw_text(&**text, current_transform);
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
            PseudoDisplayItemClass(ref mut base) => &mut **base,
        }
    }

    pub fn bounds(&self) -> Rect<Au> {
        self.base().bounds
    }

    pub fn debug_with_level(&self, level: uint) {
        let mut indent = String::new();
        for _ in range(0, level) {
            indent.push_str("| ")
        }
        println!("{}+ {}", indent, self);
    }
}

impl fmt::Show for DisplayItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} @ {} ({:x}) [{}]",
            match *self {
                SolidColorDisplayItemClass(_) => "SolidColor",
                TextDisplayItemClass(_) => "Text",
                ImageDisplayItemClass(_) => "Image",
                BorderDisplayItemClass(_) => "Border",
                LineDisplayItemClass(_) => "Line",
                PseudoDisplayItemClass(_) => "Pseudo",
            },
            self.base().bounds,
            self.base().node.id(),
            self.base().level
        )
    }
}
