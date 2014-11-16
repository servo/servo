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
use display_list::optimizer::DisplayListOptimizer;
use render_context::{RenderContext, ToAzureRect};
use text::glyph::CharIndex;
use text::TextRun;

use azure::azure::AzFloat;
use collections::dlist::{mod, DList};
use geom::{Point2D, Rect, SideOffsets2D, Size2D, Matrix2D};
use libc::uintptr_t;
use render_task::RenderLayer;
use script_traits::UntrustedNodeAddress;
use servo_msg::compositor_msg::LayerId;
use servo_net::image::base::Image;
use servo_util::dlist as servo_dlist;
use servo_util::geometry::{mod, Au};
use servo_util::range::Range;
use servo_util::smallvec::{SmallVec, SmallVec8};
use std::fmt;
use std::mem;
use std::slice::Items;
use style::computed_values::border_style;
use sync::Arc;

// It seems cleaner to have layout code not mention Azure directly, so let's just reexport this for
// layout to use.
pub use azure::azure_hl::GradientStop;

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

/// Display items that make up a stacking context. "Steps" here refer to the steps in CSS 2.1
/// Appendix E.
///
/// TODO(pcwalton): We could reduce the size of this structure with a more "skip list"-like
/// structure, omitting several pointers and lengths.
pub struct DisplayList {
    /// The border and backgrounds for the root of this stacking context: steps 1 and 2.
    pub background_and_borders: DList<DisplayItem>,
    /// Borders and backgrounds for block-level descendants: step 4.
    pub block_backgrounds_and_borders: DList<DisplayItem>,
    /// Floats: step 5. These are treated as pseudo-stacking contexts.
    pub floats: DList<DisplayItem>,
    /// All other content.
    pub content: DList<DisplayItem>,
    /// Child stacking contexts.
    pub children: DList<Arc<StackingContext>>,
}

impl DisplayList {
    /// Creates a new, empty display list.
    #[inline]
    pub fn new() -> DisplayList {
        DisplayList {
            background_and_borders: DList::new(),
            block_backgrounds_and_borders: DList::new(),
            floats: DList::new(),
            content: DList::new(),
            children: DList::new(),
        }
    }

    /// Appends all display items from `other` into `self`, preserving stacking order and emptying
    /// `other` in the process.
    #[inline]
    pub fn append_from(&mut self, other: &mut DisplayList) {
        servo_dlist::append_from(&mut self.background_and_borders,
                                 &mut other.background_and_borders);
        servo_dlist::append_from(&mut self.block_backgrounds_and_borders,
                                 &mut other.block_backgrounds_and_borders);
        servo_dlist::append_from(&mut self.floats, &mut other.floats);
        servo_dlist::append_from(&mut self.content, &mut other.content);
        servo_dlist::append_from(&mut self.children, &mut other.children);
    }

    /// Merges all display items from all non-float stacking levels to the `float` stacking level.
    #[inline]
    pub fn form_float_pseudo_stacking_context(&mut self) {
        servo_dlist::prepend_from(&mut self.floats, &mut self.content);
        servo_dlist::prepend_from(&mut self.floats, &mut self.block_backgrounds_and_borders);
        servo_dlist::prepend_from(&mut self.floats, &mut self.background_and_borders);
    }

    /// Returns a list of all items in this display list concatenated together. This is extremely
    /// inefficient and should only be used for debugging.
    pub fn all_display_items(&self) -> Vec<DisplayItem> {
        let mut result = Vec::new();
        for display_item in self.background_and_borders.iter() {
            result.push((*display_item).clone())
        }
        for display_item in self.block_backgrounds_and_borders.iter() {
            result.push((*display_item).clone())
        }
        for display_item in self.floats.iter() {
            result.push((*display_item).clone())
        }
        for display_item in self.content.iter() {
            result.push((*display_item).clone())
        }
        result
    }
}

/// Represents one CSS stacking context, which may or may not have a hardware layer.
pub struct StackingContext {
    /// The display items that make up this stacking context.
    pub display_list: Box<DisplayList>,
    /// The layer for this stacking context, if there is one.
    pub layer: Option<Arc<RenderLayer>>,
    /// The position and size of this stacking context.
    pub bounds: Rect<Au>,
    /// The clipping rect for this stacking context, in the coordinate system of the *parent*
    /// stacking context.
    pub clip_rect: Rect<Au>,
    /// The `z-index` for this stacking context.
    pub z_index: i32,
}

impl StackingContext {
    /// Creates a new stacking context.
    ///
    /// TODO(pcwalton): Stacking contexts should not always be clipped to their bounds, to handle
    /// overflow properly.
    #[inline]
    pub fn new(display_list: Box<DisplayList>,
               bounds: Rect<Au>,
               z_index: i32,
               layer: Option<Arc<RenderLayer>>)
               -> StackingContext {
        StackingContext {
            display_list: display_list,
            layer: layer,
            bounds: bounds,
            clip_rect: bounds,
            z_index: z_index,
        }
    }

    /// Draws the stacking context in the proper order according to the steps in CSS 2.1 § E.2.
    pub fn optimize_and_draw_into_context(&self,
                                          render_context: &mut RenderContext,
                                          tile_bounds: &Rect<AzFloat>,
                                          current_transform: &Matrix2D<AzFloat>,
                                          current_clip_stack: &mut Vec<Rect<Au>>) {
        // Optimize the display list to throw out out-of-bounds display items and so forth.
        let display_list = DisplayListOptimizer::new(tile_bounds).optimize(&*self.display_list);

        // Sort positioned children according to z-index.
        let mut positioned_children = SmallVec8::new();
        for kid in display_list.children.iter() {
            positioned_children.push((*kid).clone());
        }
        positioned_children.as_slice_mut().sort_by(|this, other| this.z_index.cmp(&other.z_index));

        // Steps 1 and 2: Borders and background for the root.
        for display_item in display_list.background_and_borders.iter() {
            display_item.draw_into_context(render_context, current_transform, current_clip_stack)
        }

        // Step 3: Positioned descendants with negative z-indices.
        for positioned_kid in positioned_children.iter() {
            if positioned_kid.z_index >= 0 {
                break
            }
            if positioned_kid.layer.is_none() {
                let new_transform =
                    current_transform.translate(positioned_kid.bounds.origin.x.to_nearest_px()
                                                    as AzFloat,
                                                positioned_kid.bounds.origin.y.to_nearest_px()
                                                    as AzFloat);
                let new_tile_rect =
                    self.compute_tile_rect_for_child_stacking_context(tile_bounds,
                                                                      &**positioned_kid);
                positioned_kid.optimize_and_draw_into_context(render_context,
                                                              &new_tile_rect,
                                                              &new_transform,
                                                              current_clip_stack);
            }
        }

        // Step 4: Block backgrounds and borders.
        for display_item in display_list.block_backgrounds_and_borders.iter() {
            display_item.draw_into_context(render_context, current_transform, current_clip_stack)
        }

        // Step 5: Floats.
        for display_item in display_list.floats.iter() {
            display_item.draw_into_context(render_context, current_transform, current_clip_stack)
        }

        // TODO(pcwalton): Step 6: Inlines that generate stacking contexts.

        // Step 7: Content.
        for display_item in display_list.content.iter() {
            display_item.draw_into_context(render_context, current_transform, current_clip_stack)
        }

        // Steps 8 and 9: Positioned descendants with nonnegative z-indices.
        for positioned_kid in positioned_children.iter() {
            if positioned_kid.z_index < 0 {
                continue
            }

            if positioned_kid.layer.is_none() {
                let new_transform =
                    current_transform.translate(positioned_kid.bounds.origin.x.to_nearest_px()
                                                    as AzFloat,
                                                positioned_kid.bounds.origin.y.to_nearest_px()
                                                    as AzFloat);
                let new_tile_rect =
                    self.compute_tile_rect_for_child_stacking_context(tile_bounds,
                                                                      &**positioned_kid);
                positioned_kid.optimize_and_draw_into_context(render_context,
                                                              &new_tile_rect,
                                                              &new_transform,
                                                              current_clip_stack);
            }
        }

        // TODO(pcwalton): Step 10: Outlines.
    }

    /// Translate the given tile rect into the coordinate system of a child stacking context.
    fn compute_tile_rect_for_child_stacking_context(&self,
                                                    tile_bounds: &Rect<AzFloat>,
                                                    child_stacking_context: &StackingContext)
                                                    -> Rect<AzFloat> {
        static ZERO_AZURE_RECT: Rect<f32> = Rect {
            origin: Point2D {
                x: 0.0,
                y: 0.0,
            },
            size: Size2D {
                width: 0.0,
                height: 0.0
            }
        };

        let child_stacking_context_bounds = child_stacking_context.bounds.to_azure_rect();
        let tile_subrect = tile_bounds.intersection(&child_stacking_context_bounds)
                                      .unwrap_or(ZERO_AZURE_RECT);
        let offset = tile_subrect.origin - child_stacking_context_bounds.origin;
        Rect(offset, tile_subrect.size)
    }

    /// Places all nodes containing the point of interest into `result`, topmost first. If
    /// `topmost_only` is true, stops after placing one node into the list. `result` must be empty
    /// upon entry to this function.
    pub fn hit_test(&self,
                    point: Point2D<Au>,
                    result: &mut Vec<UntrustedNodeAddress>,
                    topmost_only: bool) {
        fn hit_test_in_list<'a,I>(point: Point2D<Au>,
                                  result: &mut Vec<UntrustedNodeAddress>,
                                  topmost_only: bool,
                                  mut iterator: I)
                                  where I: Iterator<&'a DisplayItem> {
            for item in iterator {
                if geometry::rect_contains_point(item.base().clip_rect, point) &&
                        geometry::rect_contains_point(item.bounds(), point) {
                    result.push(item.base().node.to_untrusted_node_address());
                    if topmost_only {
                        return
                    }
                }
            }
        }

        debug_assert!(!topmost_only || result.is_empty());

        // Iterate through display items in reverse stacking order. Steps here refer to the
        // painting steps in CSS 2.1 Appendix E.
        //
        // Steps 9 and 8: Positioned descendants with nonnegative z-indices.
        for kid in self.display_list.children.iter().rev() {
            if kid.z_index < 0 {
                continue
            }
            kid.hit_test(point, result, topmost_only);
            if topmost_only && !result.is_empty() {
                return
            }
        }

        // Steps 7, 5, and 4: Content, floats, and block backgrounds and borders.
        //
        // TODO(pcwalton): Step 6: Inlines that generate stacking contexts.
        for display_list in [
            &self.display_list.content,
            &self.display_list.floats,
            &self.display_list.block_backgrounds_and_borders,
        ].iter() {
            hit_test_in_list(point, result, topmost_only, display_list.iter().rev());
            if topmost_only && !result.is_empty() {
                return
            }
        }

        // Step 3: Positioned descendants with negative z-indices.
        for kid in self.display_list.children.iter().rev() {
            if kid.z_index >= 0 {
                continue
            }
            kid.hit_test(point, result, topmost_only);
            if topmost_only && !result.is_empty() {
                return
            }
        }

        // Steps 2 and 1: Borders and background for the root.
        hit_test_in_list(point,
                         result,
                         topmost_only,
                         self.display_list.background_and_borders.iter().rev())
    }
}

/// Returns the stacking context in the given tree of stacking contexts with a specific layer ID.
pub fn find_stacking_context_with_layer_id(this: &Arc<StackingContext>, layer_id: LayerId)
                                           -> Option<Arc<StackingContext>> {
    match this.layer {
        Some(ref layer) if layer.id == layer_id => return Some((*this).clone()),
        Some(_) | None => {}
    }

    for kid in this.display_list.children.iter() {
        match find_stacking_context_with_layer_id(kid, layer_id) {
            Some(stacking_context) => return Some(stacking_context),
            None => {}
        }
    }

    None
}

/// One drawing command in the list.
#[deriving(Clone)]
pub enum DisplayItem {
    SolidColorDisplayItemClass(Box<SolidColorDisplayItem>),
    TextDisplayItemClass(Box<TextDisplayItem>),
    ImageDisplayItemClass(Box<ImageDisplayItem>),
    BorderDisplayItemClass(Box<BorderDisplayItem>),
    GradientDisplayItemClass(Box<GradientDisplayItem>),
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

    /// The rectangle to clip to.
    ///
    /// TODO(pcwalton): Eventually, to handle `border-radius`, this will (at least) need to grow
    /// the ability to describe rounded rectangles.
    pub clip_rect: Rect<Au>,
}

impl BaseDisplayItem {
    #[inline(always)]
    pub fn new(bounds: Rect<Au>, node: OpaqueNode, clip_rect: Rect<Au>) -> BaseDisplayItem {
        BaseDisplayItem {
            bounds: bounds,
            node: node,
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

/// Paints a gradient.
#[deriving(Clone)]
pub struct GradientDisplayItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    /// The start point of the gradient (computed during display list construction).
    pub start_point: Point2D<Au>,

    /// The end point of the gradient (computed during display list construction).
    pub end_point: Point2D<Au>,

    /// A list of color stops.
    pub stops: Vec<GradientStop>,
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

        render_context.draw_target.set_transform(current_transform);

        match *self {
            SolidColorDisplayItemClass(ref solid_color) => {
                render_context.draw_solid_color(&solid_color.base.bounds, solid_color.color)
            }

            TextDisplayItemClass(ref text) => {
                debug!("Drawing text at {}.", text.base.bounds);
                render_context.draw_text(&**text, current_transform);
            }

            ImageDisplayItemClass(ref image_item) => {
                debug!("Drawing image at {}.", image_item.base.bounds);

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

            GradientDisplayItemClass(ref gradient) => {
                render_context.draw_linear_gradient(&gradient.base.bounds,
                                                    &gradient.start_point,
                                                    &gradient.end_point,
                                                    gradient.stops.as_slice());
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
            GradientDisplayItemClass(ref gradient) => &gradient.base,
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
            GradientDisplayItemClass(ref mut gradient) => &mut gradient.base,
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
        write!(f, "{} @ {} ({:x})",
            match *self {
                SolidColorDisplayItemClass(_) => "SolidColor",
                TextDisplayItemClass(_) => "Text",
                ImageDisplayItemClass(_) => "Image",
                BorderDisplayItemClass(_) => "Border",
                GradientDisplayItemClass(_) => "Gradient",
                LineDisplayItemClass(_) => "Line",
                PseudoDisplayItemClass(_) => "Pseudo",
            },
            self.base().bounds,
            self.base().node.id()
        )
    }
}

pub trait OpaqueNodeMethods {
    /// Converts this node to an `UntrustedNodeAddress`. An `UntrustedNodeAddress` is just the type
    /// of node that script expects to receive in a hit test.
    fn to_untrusted_node_address(&self) -> UntrustedNodeAddress;
}


impl OpaqueNodeMethods for OpaqueNode {
    fn to_untrusted_node_address(&self) -> UntrustedNodeAddress {
        unsafe {
            let OpaqueNode(addr) = *self;
            let addr: UntrustedNodeAddress = mem::transmute(addr);
            addr
        }
    }
}

