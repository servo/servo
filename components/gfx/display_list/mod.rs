/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Servo heavily uses display lists, which are retained-mode lists of painting commands to
//! perform. Using a list instead of painting elements in immediate mode allows transforms, hit
//! testing, and invalidation to be performed using the same primitives as painting. It also allows
//! Servo to aggressively cull invisible and out-of-bounds painting elements, to reduce overdraw.
//! Finally, display lists allow tiles to be farmed out onto multiple CPUs and painted in parallel
//! (although this benefit does not apply to GPU-based painting).
//!
//! Display items describe relatively high-level drawing operations (for example, entire borders
//! and shadows instead of lines and blur operations), to reduce the amount of allocation required.
//! They are therefore not exactly analogous to constructs like Skia pictures, which consist of
//! low-level drawing primitives.

#![deny(unsafe_code)]

use display_list::optimizer::DisplayListOptimizer;
use paint_context::PaintContext;
use self::DisplayItem::*;
use self::DisplayItemIterator::*;
use text::TextRun;
use text::glyph::CharIndex;

use azure::azure::AzFloat;
use azure::azure_hl::Color;
use euclid::approxeq::ApproxEq;
use euclid::num::Zero;
use euclid::{Point2D, Rect, SideOffsets2D, Size2D, Matrix2D, Matrix4};
use gfx_traits::color;
use libc::uintptr_t;
use msg::compositor_msg::{LayerId, LayerKind, ScrollPolicy};
use net_traits::image::base::Image;
use paint_task::PaintLayer;
use smallvec::SmallVec;
use std::collections::linked_list::{self, LinkedList};
use std::fmt;
use std::slice::Iter;
use std::sync::Arc;
use style::computed_values::{border_style, cursor, filter, image_rendering, mix_blend_mode};
use style::computed_values::{pointer_events};
use style::properties::ComputedValues;
use util::cursor::Cursor;
use util::geometry::{self, Au, MAX_RECT, ZERO_RECT};
use util::linked_list::prepend_from;
use util::mem::HeapSizeOf;
use util::opts;
use util::range::Range;

// It seems cleaner to have layout code not mention Azure directly, so let's just reexport this for
// layout to use.
pub use azure::azure_hl::GradientStop;

pub mod optimizer;

/// The factor that we multiply the blur radius by in order to inflate the boundaries of display
/// items that involve a blur. This ensures that the display item boundaries include all the ink.
pub static BLUR_INFLATION_FACTOR: i32 = 3;

const MIN_INDENTATION_LENGTH: usize = 4;

/// An opaque handle to a node. The only safe operation that can be performed on this node is to
/// compare it to another opaque handle or to another node.
///
/// Because the script task's GC does not trace layout, node data cannot be safely stored in layout
/// data structures. Also, layout code tends to be faster when the DOM is not being accessed, for
/// locality reasons. Using `OpaqueNode` enforces this invariant.
#[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf, Hash, Eq, Deserialize, Serialize)]
pub struct OpaqueNode(pub uintptr_t);

impl OpaqueNode {
    /// Returns the address of this node, for debugging purposes.
    #[inline]
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
#[derive(HeapSizeOf, Deserialize, Serialize)]
pub struct DisplayList {
    /// The border and backgrounds for the root of this stacking context: steps 1 and 2.
    pub background_and_borders: LinkedList<DisplayItem>,
    /// Borders and backgrounds for block-level descendants: step 4.
    pub block_backgrounds_and_borders: LinkedList<DisplayItem>,
    /// Floats: step 5. These are treated as pseudo-stacking contexts.
    pub floats: LinkedList<DisplayItem>,
    /// All non-positioned content.
    pub content: LinkedList<DisplayItem>,
    /// All positioned content that does not get a stacking context.
    pub positioned_content: LinkedList<DisplayItem>,
    /// Outlines: step 10.
    pub outlines: LinkedList<DisplayItem>,
    /// Child stacking contexts.
    pub children: LinkedList<Arc<StackingContext>>,
    /// Child PaintLayers that will be rendered on top of everything else.
    pub layered_children: LinkedList<PaintLayer>,
}

impl DisplayList {
    /// Creates a new, empty display list.
    #[inline]
    pub fn new() -> DisplayList {
        DisplayList {
            background_and_borders: LinkedList::new(),
            block_backgrounds_and_borders: LinkedList::new(),
            floats: LinkedList::new(),
            content: LinkedList::new(),
            positioned_content: LinkedList::new(),
            outlines: LinkedList::new(),
            children: LinkedList::new(),
            layered_children: LinkedList::new(),
        }
    }

    /// Sort all children by their z-index and split layered children into their own
    /// section of the display list.
    /// TODO(mrobinson): This should properly handle unlayered children that are on
    /// top of layered children.
    #[inline]
    pub fn sort_and_layerize_children(&mut self) {
        let mut children: SmallVec<[Arc<StackingContext>; 8]> = SmallVec::new();
        while let Some(stacking_context) = self.children.pop_front() {
            children.push(stacking_context);
        }
        children.sort_by(|this, other| this.z_index.cmp(&other.z_index));

        for stacking_context in children.into_iter() {
            match stacking_context.layer_id {
                Some(layer_id) => self.layered_children.push_back(
                    PaintLayer::new(layer_id, color::transparent(), stacking_context)),
                None => self.children.push_back(stacking_context),
            }
        }
    }

    /// Appends all display items from `other` into `self`, preserving stacking order and emptying
    /// `other` in the process.
    #[inline]
    pub fn append_from(&mut self, other: &mut DisplayList) {
        self.background_and_borders.append(&mut other.background_and_borders);
        self.block_backgrounds_and_borders.append(&mut other.block_backgrounds_and_borders);
        self.floats.append(&mut other.floats);
        self.content.append(&mut other.content);
        self.positioned_content.append(&mut other.positioned_content);
        self.outlines.append(&mut other.outlines);
        self.children.append(&mut other.children);
        self.layered_children.append(&mut other.layered_children);
    }

    /// Merges all display items from all non-float stacking levels to the `float` stacking level.
    #[inline]
    pub fn form_float_pseudo_stacking_context(&mut self) {
        prepend_from(&mut self.floats, &mut self.outlines);
        prepend_from(&mut self.floats, &mut self.positioned_content);
        prepend_from(&mut self.floats, &mut self.content);
        prepend_from(&mut self.floats, &mut self.block_backgrounds_and_borders);
        prepend_from(&mut self.floats, &mut self.background_and_borders);
    }

    /// Merges all display items from all non-positioned-content stacking levels to the
    /// positioned-content stacking level.
    #[inline]
    pub fn form_pseudo_stacking_context_for_positioned_content(&mut self) {
        prepend_from(&mut self.positioned_content, &mut self.outlines);
        prepend_from(&mut self.positioned_content, &mut self.content);
        prepend_from(&mut self.positioned_content, &mut self.floats);
        prepend_from(&mut self.positioned_content, &mut self.block_backgrounds_and_borders);
        prepend_from(&mut self.positioned_content, &mut self.background_and_borders);
    }

    /// Returns a list of all items in this display list concatenated together. This is extremely
    /// inefficient and should only be used for debugging.
    pub fn all_display_items(&self) -> Vec<DisplayItem> {
        let mut result = Vec::new();
        for display_item in &self.background_and_borders {
            result.push((*display_item).clone())
        }
        for display_item in &self.block_backgrounds_and_borders {
            result.push((*display_item).clone())
        }
        for display_item in &self.floats {
            result.push((*display_item).clone())
        }
        for display_item in &self.content {
            result.push((*display_item).clone())
        }
        for display_item in &self.positioned_content {
            result.push((*display_item).clone())
        }
        for display_item in &self.outlines {
            result.push((*display_item).clone())
        }
        result
    }

    // Print the display list. Only makes sense to call it after performing reflow.
    pub fn print_items(&self, indentation: String) {
        // Closures are so nice!
        let doit = |items: &Vec<DisplayItem>| {
            for item in items {
                match *item {
                    DisplayItem::SolidColorClass(ref solid_color) => {
                        println!("{} SolidColor({},{},{},{}). {:?}",
                                 indentation,
                                 solid_color.color.r,
                                 solid_color.color.g,
                                 solid_color.color.b,
                                 solid_color.color.a,
                                 solid_color.base.bounds)
                    }
                    DisplayItem::TextClass(ref text) => {
                        println!("{} Text. {:?}", indentation, text.base.bounds)
                    }
                    DisplayItem::ImageClass(ref image) => {
                        println!("{} Image. {:?}", indentation, image.base.bounds)
                    }
                    DisplayItem::BorderClass(ref border) => {
                        println!("{} Border. {:?}", indentation, border.base.bounds)
                    }
                    DisplayItem::GradientClass(ref gradient) => {
                        println!("{} Gradient. {:?}", indentation, gradient.base.bounds)
                    }
                    DisplayItem::LineClass(ref line) => {
                        println!("{} Line. {:?}", indentation, line.base.bounds)
                    }
                    DisplayItem::BoxShadowClass(ref box_shadow) => {
                        println!("{} Box_shadow. {:?}", indentation, box_shadow.base.bounds)
                    }
                }
            }
            println!("\n");
        };

        doit(&(self.all_display_items()));
        if !self.children.is_empty() {
            println!("{} Children stacking contexts list length: {}",
                     indentation,
                     self.children.len());
            for stacking_context in &self.children {
                stacking_context.print(indentation.clone() +
                                       &indentation[0..MIN_INDENTATION_LENGTH]);
            }
        }
        if !self.layered_children.is_empty() {
            println!("{} Child layers list length: {}",
                     indentation,
                     self.layered_children.len());
            for paint_layer in &self.layered_children {
                paint_layer.stacking_context.print(indentation.clone() +
                                                   &indentation[0..MIN_INDENTATION_LENGTH]);
            }
        }
    }
}

#[derive(HeapSizeOf, Deserialize, Serialize)]
/// Represents one CSS stacking context, which may or may not have a hardware layer.
pub struct StackingContext {
    /// The display items that make up this stacking context.
    pub display_list: Box<DisplayList>,

    /// The position and size of this stacking context.
    pub bounds: Rect<Au>,

    /// The overflow rect for this stacking context in its coordinate system.
    pub overflow: Rect<Au>,

    /// The `z-index` for this stacking context.
    pub z_index: i32,

    /// CSS filters to be applied to this stacking context (including opacity).
    pub filters: filter::T,

    /// The blend mode with which this stacking context blends with its backdrop.
    pub blend_mode: mix_blend_mode::T,

    /// A transform to be applied to this stacking context.
    pub transform: Matrix4,

    /// The perspective matrix to be applied to children.
    pub perspective: Matrix4,

    /// Whether this stacking context creates a new 3d rendering context.
    pub establishes_3d_context: bool,

    /// Whether this stacking context scrolls its overflow area.
    pub scrolls_overflow_area: bool,

    /// The scrolling policy of this stacking context, if it is promoted
    /// to a layer.
    pub scroll_policy: ScrollPolicy,

    /// The layer id for this stacking context, if there is one.
    pub layer_id: Option<LayerId>,
}

impl StackingContext {
    /// Creates a new stacking context.
    #[inline]
    pub fn new(mut display_list: Box<DisplayList>,
               bounds: &Rect<Au>,
               overflow: &Rect<Au>,
               z_index: i32,
               filters: filter::T,
               blend_mode: mix_blend_mode::T,
               transform: Matrix4,
               perspective: Matrix4,
               establishes_3d_context: bool,
               scrolls_overflow_area: bool,
               scroll_policy: ScrollPolicy,
               layer_id: Option<LayerId>)
               -> StackingContext {
        display_list.sort_and_layerize_children();
        StackingContext {
            display_list: display_list,
            bounds: *bounds,
            overflow: *overflow,
            z_index: z_index,
            filters: filters,
            blend_mode: blend_mode,
            transform: transform,
            perspective: perspective,
            establishes_3d_context: establishes_3d_context,
            scrolls_overflow_area: scrolls_overflow_area,
            scroll_policy: scroll_policy,
            layer_id: layer_id,
        }
    }

    /// Draws the stacking context in the proper order according to the steps in CSS 2.1 § E.2.
    pub fn draw_into_context(&self,
                             display_list: &DisplayList,
                             paint_context: &mut PaintContext,
                             transform: &Matrix4,
                             clip_rect: Option<&Rect<Au>>) {
        let temporary_draw_target =
            paint_context.get_or_create_temporary_draw_target(&self.filters, self.blend_mode);
        {
            let mut paint_subcontext = PaintContext {
                draw_target: temporary_draw_target.clone(),
                font_context: &mut *paint_context.font_context,
                page_rect: paint_context.page_rect,
                screen_rect: paint_context.screen_rect,
                clip_rect: clip_rect.map(|clip_rect| *clip_rect),
                transient_clip: None,
                layer_kind: paint_context.layer_kind,
            };

            if opts::get().dump_display_list_optimized {
                println!("**** optimized display list. Tile bounds: {:?}", paint_context.page_rect);
                display_list.print_items("####".to_owned());
            }

            // Set up our clip rect and transform.
            let old_transform = paint_subcontext.draw_target.get_transform();
            let xform_2d = Matrix2D::new(transform.m11, transform.m12,
                                         transform.m21, transform.m22,
                                         transform.m41, transform.m42);
            paint_subcontext.draw_target.set_transform(&xform_2d);
            paint_subcontext.push_clip_if_applicable();

            // Steps 1 and 2: Borders and background for the root.
            for display_item in &display_list.background_and_borders {
                display_item.draw_into_context(&mut paint_subcontext)
            }

            // Step 3: Positioned descendants with negative z-indices.
            for positioned_kid in &display_list.children {
                if positioned_kid.z_index >= 0 {
                    break
                }
                let new_transform =
                    transform.translate(positioned_kid.bounds
                                                      .origin
                                                      .x
                                                      .to_nearest_px() as AzFloat,
                                        positioned_kid.bounds
                                                      .origin
                                                      .y
                                                      .to_nearest_px() as AzFloat,
                                        0.0);
                positioned_kid.optimize_and_draw_into_context(&mut paint_subcontext,
                                                              &new_transform,
                                                              Some(&positioned_kid.overflow))
            }

            // Step 4: Block backgrounds and borders.
            for display_item in &display_list.block_backgrounds_and_borders {
                display_item.draw_into_context(&mut paint_subcontext)
            }

            // Step 5: Floats.
            for display_item in &display_list.floats {
                display_item.draw_into_context(&mut paint_subcontext)
            }

            // TODO(pcwalton): Step 6: Inlines that generate stacking contexts.

            // Step 7: Content.
            for display_item in &display_list.content {
                display_item.draw_into_context(&mut paint_subcontext)
            }

            // Step 8: Positioned descendants with `z-index: auto`.
            for display_item in &display_list.positioned_content {
                display_item.draw_into_context(&mut paint_subcontext)
            }

            // Step 9: Positioned descendants with nonnegative, numeric z-indices.
            for positioned_kid in &self.display_list.children {
                if positioned_kid.z_index < 0 {
                    continue
                }
                let new_transform =
                    transform.translate(positioned_kid.bounds
                                                      .origin
                                                      .x
                                                      .to_nearest_px() as AzFloat,
                                        positioned_kid.bounds
                                                      .origin
                                                      .y
                                                      .to_nearest_px() as AzFloat,
                                        0.0);
                positioned_kid.optimize_and_draw_into_context(&mut paint_subcontext,
                                                              &new_transform,
                                                              Some(&positioned_kid.overflow))
            }

            // Step 10: Outlines.
            for display_item in &display_list.outlines {
                display_item.draw_into_context(&mut paint_subcontext)
            }

            // Undo our clipping and transform.
            paint_subcontext.remove_transient_clip_if_applicable();
            paint_subcontext.pop_clip_if_applicable();
            paint_subcontext.draw_target.set_transform(&old_transform)
        }

        paint_context.draw_temporary_draw_target_if_necessary(&temporary_draw_target,
                                                              &self.filters,
                                                              self.blend_mode)

    }

    /// Optionally optimize and then draws the stacking context.
    pub fn optimize_and_draw_into_context(&self,
                                          paint_context: &mut PaintContext,
                                          transform: &Matrix4,
                                          clip_rect: Option<&Rect<Au>>) {

        // If a layer is being used, the transform for this layer
        // will be handled by the compositor.
        let transform = match self.layer_id {
            Some(..) => *transform,
            None => transform.mul(&self.transform),
        };

        // TODO(gw): This is a hack to avoid running the DL optimizer
        // on 3d transformed tiles. We should have a better solution
        // than just disabling the opts here.
        if paint_context.layer_kind == LayerKind::HasTransform {
            self.draw_into_context(&self.display_list,
                                   paint_context,
                                   &transform,
                                   clip_rect);

        } else {
            // Invert the current transform, then use this to back transform
            // the tile rect (placed at the origin) into the space of this
            // stacking context.
            let inverse_transform = transform.invert();
            let inverse_transform_2d = Matrix2D::new(inverse_transform.m11, inverse_transform.m12,
                                                     inverse_transform.m21, inverse_transform.m22,
                                                     inverse_transform.m41, inverse_transform.m42);

            let tile_size = Size2D::new(paint_context.screen_rect.size.width as f32,
                                        paint_context.screen_rect.size.height as f32);
            let tile_rect = Rect::new(Point2D::zero(), tile_size);
            let tile_rect = inverse_transform_2d.transform_rect(&tile_rect);

            // Optimize the display list to throw out out-of-bounds display items and so forth.
            let display_list = DisplayListOptimizer::new(&tile_rect).optimize(&*self.display_list);

            self.draw_into_context(&display_list,
                                   paint_context,
                                   &transform,
                                   clip_rect);
        }
    }

    /// Places all nodes containing the point of interest into `result`, topmost first. Respects
    /// the `pointer-events` CSS property If `topmost_only` is true, stops after placing one node
    /// into the list. `result` must be empty upon entry to this function.
    pub fn hit_test(&self,
                    mut point: Point2D<Au>,
                    result: &mut Vec<DisplayItemMetadata>,
                    topmost_only: bool) {
        fn hit_test_in_list<'a, I>(point: Point2D<Au>,
                                   result: &mut Vec<DisplayItemMetadata>,
                                   topmost_only: bool,
                                   iterator: I)
                                   where I: Iterator<Item=&'a DisplayItem> {
            for item in iterator {
                // TODO(pcwalton): Use a precise algorithm here. This will allow us to properly hit
                // test elements with `border-radius`, for example.
                if !item.base().clip.might_intersect_point(&point) {
                    // Clipped out.
                    continue
                }
                if !geometry::rect_contains_point(item.bounds(), point) {
                    // Can't possibly hit.
                    continue
                }
                if item.base().metadata.pointing.is_none() {
                    // `pointer-events` is `none`. Ignore this item.
                    continue
                }
                match *item {
                    DisplayItem::BorderClass(ref border) => {
                        // If the point is inside the border, it didn't hit the border!
                        let interior_rect =
                            Rect::new(
                                Point2D::new(border.base.bounds.origin.x +
                                             border.border_widths.left,
                                             border.base.bounds.origin.y +
                                             border.border_widths.top),
                                Size2D::new(border.base.bounds.size.width -
                                                (border.border_widths.left +
                                                 border.border_widths.right),
                                            border.base.bounds.size.height -
                                                (border.border_widths.top +
                                                 border.border_widths.bottom)));
                        if geometry::rect_contains_point(interior_rect, point) {
                            continue
                        }
                    }
                    _ => {}
                }

                // We found a hit!
                result.push(item.base().metadata);
                if topmost_only {
                    return
                }
            }
        }

        // Convert the point into stacking context local space
        point = point - self.bounds.origin;

        debug_assert!(!topmost_only || result.is_empty());
        let inv_transform = self.transform.invert();
        let frac_point = inv_transform.transform_point(&Point2D::new(point.x.to_f32_px(),
                                                                     point.y.to_f32_px()));
        point = Point2D::new(Au::from_f32_px(frac_point.x), Au::from_f32_px(frac_point.y));

        // Layers are positioned on top of this layer should get a shot at the hit test first.
        for layer in self.display_list.layered_children.iter().rev() {
            layer.stacking_context.hit_test(point, result, topmost_only);
            if topmost_only && !result.is_empty() {
                return
            }
        }

        // Iterate through display items in reverse stacking order. Steps here refer to the
        // painting steps in CSS 2.1 Appendix E.
        //
        // Step 10: Outlines.
        hit_test_in_list(point, result, topmost_only, self.display_list.outlines.iter().rev());
        if topmost_only && !result.is_empty() {
            return
        }

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

        // Steps 8, 7, 5, and 4: Positioned content, content, floats, and block backgrounds and
        // borders.
        //
        // TODO(pcwalton): Step 6: Inlines that generate stacking contexts.
        for display_list in &[
            &self.display_list.positioned_content,
            &self.display_list.content,
            &self.display_list.floats,
            &self.display_list.block_backgrounds_and_borders,
        ] {
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

    pub fn print(&self, mut indentation: String) {
        // We cover the case of an empty string.
        if indentation.is_empty() {
            indentation = "####".to_owned();
        }

        // We grow the indentation by 4 characters if needed.
        // I wish to push it all as a slice, but it won't work if the string is a single char.
        while indentation.len() < MIN_INDENTATION_LENGTH {
            let c = indentation.char_at(0);
            indentation.push(c);
        }

        println!("{:?} Stacking context at {:?} with overflow {:?}:",
                 indentation,
                 self.bounds,
                 self.overflow);

        self.display_list.print_items(indentation);
    }
}

/// Returns the stacking context in the given tree of stacking contexts with a specific layer ID.
pub fn find_stacking_context_with_layer_id(this: &Arc<StackingContext>, layer_id: LayerId)
                                           -> Option<Arc<StackingContext>> {
    for kid in &this.display_list.layered_children {
        if let Some(stacking_context) = kid.find_stacking_context_with_layer_id(layer_id) {
            return Some(stacking_context);
        }
    }

    for kid in &this.display_list.children {
        if let Some(stacking_context) = find_stacking_context_with_layer_id(kid, layer_id) {
            return Some(stacking_context);
        }
    }

    None
}

/// One drawing command in the list.
#[derive(Clone, Deserialize, HeapSizeOf, Serialize)]
pub enum DisplayItem {
    SolidColorClass(Box<SolidColorDisplayItem>),
    TextClass(Box<TextDisplayItem>),
    ImageClass(Box<ImageDisplayItem>),
    BorderClass(Box<BorderDisplayItem>),
    GradientClass(Box<GradientDisplayItem>),
    LineClass(Box<LineDisplayItem>),
    BoxShadowClass(Box<BoxShadowDisplayItem>),
}

/// Information common to all display items.
#[derive(Clone, Deserialize, HeapSizeOf, Serialize)]
pub struct BaseDisplayItem {
    /// The boundaries of the display item, in layer coordinates.
    pub bounds: Rect<Au>,

    /// Metadata attached to this display item.
    pub metadata: DisplayItemMetadata,

    /// The region to clip to.
    pub clip: ClippingRegion,
}

impl BaseDisplayItem {
    #[inline(always)]
    pub fn new(bounds: Rect<Au>, metadata: DisplayItemMetadata, clip: ClippingRegion)
               -> BaseDisplayItem {
        BaseDisplayItem {
            bounds: bounds,
            metadata: metadata,
            clip: clip,
        }
    }
}


/// A clipping region for a display item. Currently, this can describe rectangles, rounded
/// rectangles (for `border-radius`), or arbitrary intersections of the two. Arbitrary transforms
/// are not supported because those are handled by the higher-level `StackingContext` abstraction.
#[derive(Clone, PartialEq, Debug, HeapSizeOf, Deserialize, Serialize)]
pub struct ClippingRegion {
    /// The main rectangular region. This does not include any corners.
    pub main: Rect<Au>,
    /// Any complex regions.
    ///
    /// TODO(pcwalton): Atomically reference count these? Not sure if it's worth the trouble.
    /// Measure and follow up.
    pub complex: Vec<ComplexClippingRegion>,
}

/// A complex clipping region. These don't as easily admit arbitrary intersection operations, so
/// they're stored in a list over to the side. Currently a complex clipping region is just a
/// rounded rectangle, but the CSS WGs will probably make us throw more stuff in here eventually.
#[derive(Clone, PartialEq, Debug, HeapSizeOf, Deserialize, Serialize)]
pub struct ComplexClippingRegion {
    /// The boundaries of the rectangle.
    pub rect: Rect<Au>,
    /// Border radii of this rectangle.
    pub radii: BorderRadii<Au>,
}

impl ClippingRegion {
    /// Returns an empty clipping region that, if set, will result in no pixels being visible.
    #[inline]
    pub fn empty() -> ClippingRegion {
        ClippingRegion {
            main: ZERO_RECT,
            complex: Vec::new(),
        }
    }

    /// Returns an all-encompassing clipping region that clips no pixels out.
    #[inline]
    pub fn max() -> ClippingRegion {
        ClippingRegion {
            main: MAX_RECT,
            complex: Vec::new(),
        }
    }

    /// Returns a clipping region that represents the given rectangle.
    #[inline]
    pub fn from_rect(rect: &Rect<Au>) -> ClippingRegion {
        ClippingRegion {
            main: *rect,
            complex: Vec::new(),
        }
    }

    /// Returns the intersection of this clipping region and the given rectangle.
    ///
    /// TODO(pcwalton): This could more eagerly eliminate complex clipping regions, at the cost of
    /// complexity.
    #[inline]
    pub fn intersect_rect(self, rect: &Rect<Au>) -> ClippingRegion {
        ClippingRegion {
            main: self.main.intersection(rect).unwrap_or(ZERO_RECT),
            complex: self.complex,
        }
    }

    /// Returns true if this clipping region might be nonempty. This can return false positives,
    /// but never false negatives.
    #[inline]
    pub fn might_be_nonempty(&self) -> bool {
        !self.main.is_empty()
    }

    /// Returns true if this clipping region might contain the given point and false otherwise.
    /// This is a quick, not a precise, test; it can yield false positives.
    #[inline]
    pub fn might_intersect_point(&self, point: &Point2D<Au>) -> bool {
        geometry::rect_contains_point(self.main, *point) &&
            self.complex.iter().all(|complex| geometry::rect_contains_point(complex.rect, *point))
    }

    /// Returns true if this clipping region might intersect the given rectangle and false
    /// otherwise. This is a quick, not a precise, test; it can yield false positives.
    #[inline]
    pub fn might_intersect_rect(&self, rect: &Rect<Au>) -> bool {
        self.main.intersects(rect) &&
            self.complex.iter().all(|complex| complex.rect.intersects(rect))
    }


    /// Returns a bounding rect that surrounds this entire clipping region.
    #[inline]
    pub fn bounding_rect(&self) -> Rect<Au> {
        let mut rect = self.main;
        for complex in &*self.complex {
            rect = rect.union(&complex.rect)
        }
        rect
    }

    /// Intersects this clipping region with the given rounded rectangle.
    #[inline]
    pub fn intersect_with_rounded_rect(mut self, rect: &Rect<Au>, radii: &BorderRadii<Au>)
                                       -> ClippingRegion {
        self.complex.push(ComplexClippingRegion {
            rect: *rect,
            radii: *radii,
        });
        self
    }

    /// Translates this clipping region by the given vector.
    #[inline]
    pub fn translate(&self, delta: &Point2D<Au>) -> ClippingRegion {
        ClippingRegion {
            main: self.main.translate(delta),
            complex: self.complex.iter().map(|complex| {
                ComplexClippingRegion {
                    rect: complex.rect.translate(delta),
                    radii: complex.radii,
                }
            }).collect(),
        }
    }
}


/// Metadata attached to each display item. This is useful for performing auxiliary tasks with
/// the display list involving hit testing: finding the originating DOM node and determining the
/// cursor to use when the element is hovered over.
#[derive(Clone, Copy, HeapSizeOf, Deserialize, Serialize)]
pub struct DisplayItemMetadata {
    /// The DOM node from which this display item originated.
    pub node: OpaqueNode,
    /// The value of the `cursor` property when the mouse hovers over this display item. If `None`,
    /// this display item is ineligible for pointer events (`pointer-events: none`).
    pub pointing: Option<Cursor>,
}

impl DisplayItemMetadata {
    /// Creates a new set of display metadata for a display item constributed by a DOM node.
    /// `default_cursor` specifies the cursor to use if `cursor` is `auto`. Typically, this will
    /// be `PointerCursor`, but for text display items it may be `TextCursor` or
    /// `VerticalTextCursor`.
    #[inline]
    pub fn new(node: OpaqueNode, style: &ComputedValues, default_cursor: Cursor)
               -> DisplayItemMetadata {
        DisplayItemMetadata {
            node: node,
            pointing: match (style.get_pointing().pointer_events, style.get_pointing().cursor) {
                (pointer_events::T::none, _) => None,
                (pointer_events::T::auto, cursor::T::AutoCursor) => Some(default_cursor),
                (pointer_events::T::auto, cursor::T::SpecifiedCursor(cursor)) => Some(cursor),
            },
        }
    }
}

/// Paints a solid color.
#[derive(Clone, HeapSizeOf, Deserialize, Serialize)]
pub struct SolidColorDisplayItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    /// The color.
    pub color: Color,
}

/// Paints text.
#[derive(Clone, HeapSizeOf, Deserialize, Serialize)]
pub struct TextDisplayItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    /// The text run.
    #[ignore_heap_size_of = "Because it is non-owning"]
    pub text_run: Arc<TextRun>,

    /// The range of text within the text run.
    pub range: Range<CharIndex>,

    /// The color of the text.
    pub text_color: Color,

    /// The position of the start of the baseline of this text.
    pub baseline_origin: Point2D<Au>,

    /// The orientation of the text: upright or sideways left/right.
    pub orientation: TextOrientation,

    /// The blur radius for this text. If zero, this text is not blurred.
    pub blur_radius: Au,
}

#[derive(Clone, Eq, PartialEq, HeapSizeOf, Deserialize, Serialize)]
pub enum TextOrientation {
    Upright,
    SidewaysLeft,
    SidewaysRight,
}

/// Paints an image.
#[derive(Clone, HeapSizeOf, Deserialize, Serialize)]
pub struct ImageDisplayItem {
    pub base: BaseDisplayItem,
    #[ignore_heap_size_of = "Because it is non-owning"]
    pub image: Arc<Image>,

    /// The dimensions to which the image display item should be stretched. If this is smaller than
    /// the bounds of this display item, then the image will be repeated in the appropriate
    /// direction to tile the entire bounds.
    pub stretch_size: Size2D<Au>,

    /// The algorithm we should use to stretch the image. See `image_rendering` in CSS-IMAGES-3 §
    /// 5.3.
    pub image_rendering: image_rendering::T,
}


/// Paints a gradient.
#[derive(Clone, Deserialize, Serialize)]
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


impl HeapSizeOf for GradientDisplayItem {
    fn heap_size_of_children(&self) -> usize {
        use libc::c_void;
        use util::mem::heap_size_of;

        // We can't measure `stops` via Vec's HeapSizeOf implementation because GradientStop isn't
        // defined in this module, and we don't want to import GradientStop into util::mem where
        // the HeapSizeOf trait is defined. So we measure the elements directly.
        self.base.heap_size_of_children() +
            heap_size_of(self.stops.as_ptr() as *const c_void)
    }
}


/// Paints a border.
#[derive(Clone, HeapSizeOf, Deserialize, Serialize)]
pub struct BorderDisplayItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    /// Border widths.
    pub border_widths: SideOffsets2D<Au>,

    /// Border colors.
    pub color: SideOffsets2D<Color>,

    /// Border styles.
    pub style: SideOffsets2D<border_style::T>,

    /// Border radii.
    ///
    /// TODO(pcwalton): Elliptical radii.
    pub radius: BorderRadii<Au>,
}

/// Information about the border radii.
///
/// TODO(pcwalton): Elliptical radii.
#[derive(Clone, Default, PartialEq, Debug, Copy, HeapSizeOf, Deserialize, Serialize)]
pub struct BorderRadii<T> {
    pub top_left: T,
    pub top_right: T,
    pub bottom_right: T,
    pub bottom_left: T,
}

impl<T> BorderRadii<T> where T: PartialEq + Zero {
    /// Returns true if all the radii are zero.
    pub fn is_square(&self) -> bool {
        let zero = Zero::zero();
        self.top_left == zero && self.top_right == zero && self.bottom_right == zero &&
            self.bottom_left == zero
    }
}

impl<T> BorderRadii<T> where T: PartialEq + Zero + Clone {
    /// Returns a set of border radii that all have the given value.
    pub fn all_same(value: T) -> BorderRadii<T> {
        BorderRadii {
            top_left: value.clone(),
            top_right: value.clone(),
            bottom_right: value.clone(),
            bottom_left: value.clone(),
        }
    }
}

/// Paints a line segment.
#[derive(Clone, HeapSizeOf, Deserialize, Serialize)]
pub struct LineDisplayItem {
    pub base: BaseDisplayItem,

    /// The line segment color.
    pub color: Color,

    /// The line segment style.
    pub style: border_style::T
}

/// Paints a box shadow per CSS-BACKGROUNDS.
#[derive(Clone, HeapSizeOf, Deserialize, Serialize)]
pub struct BoxShadowDisplayItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    /// The dimensions of the box that we're placing a shadow around.
    pub box_bounds: Rect<Au>,

    /// The offset of this shadow from the box.
    pub offset: Point2D<Au>,

    /// The color of this shadow.
    pub color: Color,

    /// The blur radius for this shadow.
    pub blur_radius: Au,

    /// The spread radius of this shadow.
    pub spread_radius: Au,

    /// How we should clip the result.
    pub clip_mode: BoxShadowClipMode,
}

/// How a box shadow should be clipped.
#[derive(Clone, Copy, Debug, PartialEq, HeapSizeOf, Deserialize, Serialize)]
pub enum BoxShadowClipMode {
    /// No special clipping should occur. This is used for (shadowed) text decorations.
    None,
    /// The area inside `box_bounds` should be clipped out. Corresponds to the normal CSS
    /// `box-shadow`.
    Outset,
    /// The area outside `box_bounds` should be clipped out. Corresponds to the `inset` flag on CSS
    /// `box-shadow`.
    Inset,
}

pub enum DisplayItemIterator<'a> {
    Empty,
    Parent(linked_list::Iter<'a, DisplayItem>),
}

impl<'a> Iterator for DisplayItemIterator<'a> {
    type Item = &'a DisplayItem;
    #[inline]
    fn next(&mut self) -> Option<&'a DisplayItem> {
        match *self {
            DisplayItemIterator::Empty => None,
            DisplayItemIterator::Parent(ref mut subiterator) => subiterator.next(),
        }
    }
}

impl DisplayItem {
    /// Paints this display item into the given painting context.
    fn draw_into_context(&self, paint_context: &mut PaintContext) {
        let this_clip = &self.base().clip;
        match paint_context.transient_clip {
            Some(ref transient_clip) if transient_clip == this_clip => {}
            Some(_) | None => paint_context.push_transient_clip((*this_clip).clone()),
        }

        match *self {
            DisplayItem::SolidColorClass(ref solid_color) => {
                if !solid_color.color.a.approx_eq(&0.0) {
                    paint_context.draw_solid_color(&solid_color.base.bounds, solid_color.color)
                }
            }

            DisplayItem::TextClass(ref text) => {
                debug!("Drawing text at {:?}.", text.base.bounds);
                paint_context.draw_text(&**text);
            }

            DisplayItem::ImageClass(ref image_item) => {
                debug!("Drawing image at {:?}.", image_item.base.bounds);
                paint_context.draw_image(&image_item.base.bounds,
                                         &image_item.stretch_size,
                                         image_item.image.clone(),
                                         image_item.image_rendering.clone());
            }

            DisplayItem::BorderClass(ref border) => {
                paint_context.draw_border(&border.base.bounds,
                                          &border.border_widths,
                                          &border.radius,
                                          &border.color,
                                          &border.style)
            }

            DisplayItem::GradientClass(ref gradient) => {
                paint_context.draw_linear_gradient(&gradient.base.bounds,
                                                   &gradient.start_point,
                                                   &gradient.end_point,
                                                   &gradient.stops);
            }

            DisplayItem::LineClass(ref line) => {
                paint_context.draw_line(&line.base.bounds, line.color, line.style)
            }

            DisplayItem::BoxShadowClass(ref box_shadow) => {
                paint_context.draw_box_shadow(&box_shadow.box_bounds,
                                              &box_shadow.offset,
                                              box_shadow.color,
                                              box_shadow.blur_radius,
                                              box_shadow.spread_radius,
                                              box_shadow.clip_mode)
            }
        }
    }

    pub fn base(&self) -> &BaseDisplayItem {
        match *self {
            DisplayItem::SolidColorClass(ref solid_color) => &solid_color.base,
            DisplayItem::TextClass(ref text) => &text.base,
            DisplayItem::ImageClass(ref image_item) => &image_item.base,
            DisplayItem::BorderClass(ref border) => &border.base,
            DisplayItem::GradientClass(ref gradient) => &gradient.base,
            DisplayItem::LineClass(ref line) => &line.base,
            DisplayItem::BoxShadowClass(ref box_shadow) => &box_shadow.base,
        }
    }

    pub fn mut_base(&mut self) -> &mut BaseDisplayItem {
        match *self {
            DisplayItem::SolidColorClass(ref mut solid_color) => &mut solid_color.base,
            DisplayItem::TextClass(ref mut text) => &mut text.base,
            DisplayItem::ImageClass(ref mut image_item) => &mut image_item.base,
            DisplayItem::BorderClass(ref mut border) => &mut border.base,
            DisplayItem::GradientClass(ref mut gradient) => &mut gradient.base,
            DisplayItem::LineClass(ref mut line) => &mut line.base,
            DisplayItem::BoxShadowClass(ref mut box_shadow) => &mut box_shadow.base,
        }
    }

    pub fn bounds(&self) -> Rect<Au> {
        self.base().bounds
    }

    pub fn debug_with_level(&self, level: u32) {
        let mut indent = String::new();
        for _ in 0..level {
            indent.push_str("| ")
        }
        println!("{}+ {:?}", indent, self);
    }
}

impl fmt::Debug for DisplayItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} @ {:?} ({:x})",
            match *self {
                DisplayItem::SolidColorClass(_) => "SolidColor",
                DisplayItem::TextClass(_) => "Text",
                DisplayItem::ImageClass(_) => "Image",
                DisplayItem::BorderClass(_) => "Border",
                DisplayItem::GradientClass(_) => "Gradient",
                DisplayItem::LineClass(_) => "Line",
                DisplayItem::BoxShadowClass(_) => "BoxShadow",
            },
            self.base().bounds,
            self.base().metadata.node.id()
        )
    }
}
