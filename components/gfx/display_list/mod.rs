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

use app_units::Au;
use azure::azure::AzFloat;
use azure::azure_hl::{Color, DrawTarget};
use display_list::optimizer::DisplayListOptimizer;
use euclid::approxeq::ApproxEq;
use euclid::num::Zero;
use euclid::{Matrix2D, Matrix4, Point2D, Rect, SideOffsets2D, Size2D};
use gfx_traits::{color, LayerId, LayerKind, ScrollPolicy};
use heapsize::HeapSizeOf;
use msg::constellation_msg::PipelineId;
use net_traits::image::base::Image;
use paint_context::PaintContext;
use paint_thread::{PaintLayerContents, PaintLayer};
use range::Range;
use self::DisplayItem::*;
use smallvec::SmallVec;
use std::cmp::Ordering;
use std::collections::linked_list::{self, LinkedList};
use std::fmt;
use std::mem;
use std::sync::Arc;
use style::computed_values::{border_style, cursor, filter, image_rendering, mix_blend_mode};
use style::computed_values::{pointer_events};
use style::properties::ComputedValues;
use style_traits::cursor::Cursor;
use text::TextRun;
use text::glyph::CharIndex;
use util::geometry::MAX_RECT;
use util::linked_list::prepend_from;
use util::opts;
use util::print_tree::PrintTree;
use webrender_traits::WebGLContextId;

pub use style::dom::OpaqueNode;

// It seems cleaner to have layout code not mention Azure directly, so let's just reexport this for
// layout to use.
pub use azure::azure_hl::GradientStop;

pub mod optimizer;

/// The factor that we multiply the blur radius by in order to inflate the boundaries of display
/// items that involve a blur. This ensures that the display item boundaries include all the ink.
pub static BLUR_INFLATION_FACTOR: i32 = 3;

/// LayerInfo is used to store PaintLayer metadata during DisplayList construction.
/// It is also used for tracking LayerIds when creating layers to preserve ordering when
/// layered DisplayItems should render underneath unlayered DisplayItems.
#[derive(Clone, Copy, Debug, HeapSizeOf, Deserialize, Serialize)]
pub struct LayerInfo {
    /// The base LayerId of this layer.
    pub layer_id: LayerId,

    /// The scroll policy of this layer.
    pub scroll_policy: ScrollPolicy,

    /// The subpage that this layer represents, if there is one.
    pub subpage_pipeline_id: Option<PipelineId>,

    /// The id for the next layer in the sequence. This is used for synthesizing
    /// layers for content that needs to be displayed on top of this layer.
    pub next_layer_id: LayerId,
}

impl LayerInfo {
    pub fn new(id: LayerId,
               scroll_policy: ScrollPolicy,
               subpage_pipeline_id: Option<PipelineId>)
               -> LayerInfo {
        LayerInfo {
            layer_id: id,
            scroll_policy: scroll_policy,
            subpage_pipeline_id: subpage_pipeline_id,
            next_layer_id: id.companion_layer_id(),
        }
    }

    fn next(&mut self) -> LayerInfo {
        let new_layer_info = LayerInfo::new(self.next_layer_id, self.scroll_policy, None);
        self.next_layer_id = self.next_layer_id.companion_layer_id();
        new_layer_info
    }

    fn next_with_scroll_policy(&mut self, scroll_policy: ScrollPolicy) -> LayerInfo {
        let mut new_layer_info = self.next();
        new_layer_info.scroll_policy = scroll_policy;
        new_layer_info
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
    /// Child PaintLayers that will be rendered on top of everything else.
    pub layered_children: LinkedList<Arc<PaintLayer>>,
    /// Information about child layers.
    pub layer_info: LinkedList<LayerInfo>,
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
            layered_children: LinkedList::new(),
            layer_info: LinkedList::new(),
        }
    }

    /// Adds the given display item at the specified section of this display list.
    pub fn add_to_section(&mut self, display_item: DisplayItem, section: DisplayListSection) {
        self.get_section_mut(section).push_back(display_item);
    }

    /// Creates a new display list which contains a single stacking context.
    #[inline]
    pub fn new_with_stacking_context(stacking_context: Arc<StackingContext>) -> Box<DisplayList> {
        let mut display_list = box DisplayList::new();
        display_list.positioned_content.push_back(
            DisplayItem::StackingContextClass(stacking_context));
        display_list
    }

    /// Appends all display items from `other` into `self`, preserving stacking order and emptying
    /// `other` in the process.
    #[inline]
    pub fn append_from(&mut self, other: &mut Option<Box<DisplayList>>) {
        if let Some(mut other) = other.take() {
            self.background_and_borders.append(&mut other.background_and_borders);
            self.block_backgrounds_and_borders.append(&mut other.block_backgrounds_and_borders);
            self.floats.append(&mut other.floats);
            self.content.append(&mut other.content);
            self.positioned_content.append(&mut other.positioned_content);
            self.outlines.append(&mut other.outlines);
            self.layered_children.append(&mut other.layered_children);
            self.layer_info.append(&mut other.layer_info);
        }
    }

    /// Merges all display items from all non-float stacking levels to the `float` stacking level.
    /// From E.2.5 at http://www.w3.org/TR/CSS21/zindex.html. We do not include positioned content
    /// and stacking contexts in the pseudo-stacking-context.
    #[inline]
    pub fn form_float_pseudo_stacking_context(&mut self) {
        prepend_from(&mut self.floats, &mut self.outlines);
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
    pub fn flatten(&self) -> Vec<DisplayItem> {
        let mut result = Vec::new();
        fn flatten_item(result: &mut Vec<DisplayItem>, item: &DisplayItem) {
            match item {
                &DisplayItem::StackingContextClass(ref stacking_context) =>
                    result.extend(stacking_context.display_list.flatten().into_iter()),
                _ => result.push((*item).clone()),
            }
        }

        for display_item in &self.background_and_borders {
            flatten_item(&mut result, display_item);
        }
        for display_item in &self.block_backgrounds_and_borders {
            flatten_item(&mut result, display_item);
        }
        for display_item in &self.floats {
            flatten_item(&mut result, display_item);
        }
        for display_item in &self.content {
            flatten_item(&mut result, display_item);
        }
        for display_item in &self.positioned_content {
            flatten_item(&mut result, display_item);
        }
        for display_item in &self.outlines {
            flatten_item(&mut result, display_item);
        }
        result
    }

    pub fn print(&self, title: String) {
        let mut print_tree = PrintTree::new(title);
        self.print_with_tree(&mut print_tree);
    }

    pub fn print_with_tree(&self, print_tree: &mut PrintTree) {
        fn print_display_list_section(print_tree: &mut PrintTree,
                                      items: &LinkedList<DisplayItem>,
                                      title: &str) {
            if items.is_empty() {
                return;
            }

            print_tree.new_level(title.to_owned());
            for item in items {
                match item {
                    &DisplayItem::StackingContextClass(ref stacking_context) =>
                        stacking_context.print_with_tree(print_tree),
                    _ => print_tree.add_item(format!("{:?}", item)),
                }
            }
            print_tree.end_level();
        }

        print_display_list_section(print_tree,
                                   &self.background_and_borders,
                                   "Backgrounds and Borders");
        print_display_list_section(print_tree,
                                   &self.block_backgrounds_and_borders,
                                   "Block Backgrounds and Borders");
        print_display_list_section(print_tree, &self.floats, "Floats");
        print_display_list_section(print_tree, &self.content, "Content");
        print_display_list_section(print_tree, &self.positioned_content, "Positioned Content");
        print_display_list_section(print_tree, &self.outlines, "Outlines");

        if !self.layered_children.is_empty() {
            print_tree.new_level("Layers".to_owned());
            for paint_layer in &self.layered_children {
                match paint_layer.contents {
                    PaintLayerContents::StackingContext(ref stacking_context) =>
                        stacking_context.print_with_tree(print_tree),
                    PaintLayerContents::DisplayList(ref display_list) => {
                        print_tree.new_level(format!("DisplayList Layer with bounds {:?}:",
                                                     display_list.calculate_bounding_rect()));
                        display_list.print_with_tree(print_tree);
                        print_tree.end_level();
                    }
                }
            }
            print_tree.end_level();
        }
    }

    /// Draws the DisplayList in stacking context order according to the steps in CSS 2.1 § E.2.
    pub fn draw_into_context(&self,
                             draw_target: &DrawTarget,
                             paint_context: &mut PaintContext,
                             transform: &Matrix4,
                             clip_rect: Option<&Rect<Au>>) {
        let mut paint_subcontext = PaintContext {
            draw_target: draw_target.clone(),
            font_context: &mut *paint_context.font_context,
            page_rect: paint_context.page_rect,
            screen_rect: paint_context.screen_rect,
            clip_rect: clip_rect.map(|clip_rect| *clip_rect),
            transient_clip: None,
            layer_kind: paint_context.layer_kind,
        };

        if opts::get().dump_display_list_optimized {
            self.print(format!("Optimized display list. Tile bounds: {:?}",
                                paint_context.page_rect));
        }

        // Set up our clip rect and transform.
        let old_transform = paint_subcontext.draw_target.get_transform();
        let xform_2d = Matrix2D::new(transform.m11, transform.m12,
                                     transform.m21, transform.m22,
                                     transform.m41, transform.m42);
        paint_subcontext.draw_target.set_transform(&xform_2d);
        paint_subcontext.push_clip_if_applicable();

        // Steps 1 and 2: Borders and background for the root.
        for display_item in &self.background_and_borders {
            display_item.draw_into_context(transform, &mut paint_subcontext)
        }

        // Step 3: Positioned descendants with negative z-indices.
        for positioned_kid in &self.positioned_content {
            if let &DisplayItem::StackingContextClass(ref stacking_context) = positioned_kid {
                if stacking_context.z_index < 0 {
                    positioned_kid.draw_into_context(transform, &mut paint_subcontext);
                }
            }
        }

        // Step 4: Block backgrounds and borders.
        for display_item in &self.block_backgrounds_and_borders {
            display_item.draw_into_context(transform, &mut paint_subcontext)
        }

        // Step 5: Floats.
        for display_item in &self.floats {
            display_item.draw_into_context(transform, &mut paint_subcontext)
        }

        // TODO(pcwalton): Step 6: Inlines that generate stacking contexts.

        // Step 7: Content.
        for display_item in &self.content {
            display_item.draw_into_context(transform, &mut paint_subcontext)
        }

        // Step 8 & 9: Positioned descendants with nonnegative, numeric z-indices.
        for positioned_kid in &self.positioned_content {
            if let &DisplayItem::StackingContextClass(ref stacking_context) = positioned_kid {
                if stacking_context.z_index < 0 {
                    continue;
                }
            }
            positioned_kid.draw_into_context(transform, &mut paint_subcontext);
        }

        // Step 10: Outlines.
        for display_item in &self.outlines {
            display_item.draw_into_context(transform, &mut paint_subcontext)
        }

        // Undo our clipping and transform.
        paint_subcontext.remove_transient_clip_if_applicable();
        paint_subcontext.pop_clip_if_applicable();
        paint_subcontext.draw_target.set_transform(&old_transform)
    }

    pub fn hit_test(&self,
                    point: Point2D<Au>,
                    result: &mut Vec<DisplayItemMetadata>,
                    topmost_only: bool) {
        fn hit_test_item(point: Point2D<Au>,
                         result: &mut Vec<DisplayItemMetadata>,
                         item: &DisplayItem) {
                let base_item = match item.base() {
                    Some(base) => base,
                    None => return,
                };

                // TODO(pcwalton): Use a precise algorithm here. This will allow us to properly hit
                // test elements with `border-radius`, for example.
                if !base_item.clip.might_intersect_point(&point) {
                    // Clipped out.
                    return;
                }
                if !item.bounds().contains(&point) {
                    // Can't possibly hit.
                    return;
                }
                if base_item.metadata.pointing.is_none() {
                    // `pointer-events` is `none`. Ignore this item.
                    return;
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
                        if interior_rect.contains(&point) {
                            return;
                        }
                    }
                    DisplayItem::BoxShadowClass(_) => {
                        // Box shadows can never be hit.
                        return
                    }
                    _ => {}
                }

                // We found a hit!
                result.push(base_item.metadata);
            }

        fn hit_test_in_list<'a, I>(point: Point2D<Au>,
                                   result: &mut Vec<DisplayItemMetadata>,
                                   topmost_only: bool,
                                   iterator: I)
                                   where I: Iterator<Item=&'a DisplayItem> {
            for item in iterator {
                hit_test_item(point, result, item);
                if topmost_only && !result.is_empty() {
                    return;
                }
            }
        }

        // Layers that are positioned on top of this layer should get a shot at the hit test first.
        for layer in self.layered_children.iter().rev() {
            match layer.contents {
                PaintLayerContents::StackingContext(ref stacking_context) =>
                    stacking_context.hit_test(point, result, topmost_only),
                PaintLayerContents::DisplayList(ref display_list) =>
                    display_list.hit_test(point, result, topmost_only),
            }

            if topmost_only && !result.is_empty() {
                return
            }
        }

        // Iterate through display items in reverse stacking order. Steps here refer to the
        // painting steps in CSS 2.1 Appendix E.
        //
        // Step 10: Outlines.
        hit_test_in_list(point, result, topmost_only, self.outlines.iter().rev());
        if topmost_only && !result.is_empty() {
            return
        }

        // Steps 9 and 8: Positioned descendants with nonnegative z-indices.
        for kid in self.positioned_content.iter().rev() {
            if let &DisplayItem::StackingContextClass(ref stacking_context) = kid {
                if stacking_context.z_index < 0 {
                    continue
                }
                stacking_context.hit_test(point, result, topmost_only);
            } else {
                hit_test_item(point, result, kid);
            }

            if topmost_only && !result.is_empty() {
                return
            }
        }

        // Steps 8, 7, 5, and 4: Positioned content, content, floats, and block backgrounds and
        // borders.
        //
        // TODO(pcwalton): Step 6: Inlines that generate stacking contexts.
        for display_list in &[
            &self.content,
            &self.floats,
            &self.block_backgrounds_and_borders,
        ] {
            hit_test_in_list(point, result, topmost_only, display_list.iter().rev());
            if topmost_only && !result.is_empty() {
                return
            }
        }

        for kid in self.positioned_content.iter().rev() {
            if let &DisplayItem::StackingContextClass(ref stacking_context) = kid {
                if stacking_context.z_index >= 0 {
                    continue
                }
                stacking_context.hit_test(point, result, topmost_only);
                if topmost_only && !result.is_empty() {
                    return
                }
            }
        }

        // Steps 2 and 1: Borders and background for the root.
        hit_test_in_list(point,
                         result,
                         topmost_only,
                         self.background_and_borders.iter().rev())

    }

    /// Returns the PaintLayer in the given DisplayList with a specific layer ID.
    pub fn find_layer_with_layer_id(&self, layer_id: LayerId) -> Option<Arc<PaintLayer>> {
        for kid in &self.layered_children {
            if let Some(paint_layer) = PaintLayer::find_layer_with_layer_id(&kid, layer_id) {
                return Some(paint_layer);
            }
        }

        for item in &self.positioned_content {
            if let &DisplayItem::StackingContextClass(ref stacking_context) = item {
                if let Some(paint_layer)
                        = stacking_context.display_list.find_layer_with_layer_id(layer_id) {
                    return Some(paint_layer);
                }
            }
        }

        None
    }

    /// Calculate the union of all the bounds of all of the items in this display list.
    /// This is an expensive operation, so it shouldn't be done unless absolutely necessary
    /// and, if possible, the result should be cached.
    pub fn calculate_bounding_rect(&self) -> Rect<Au> {
        fn union_all_items(list: &LinkedList<DisplayItem>, mut bounds: Rect<Au>) -> Rect<Au> {
            for item in list {
                bounds = bounds.union(&item.bounds());
            }
            bounds
        };

        let mut bounds = Rect::zero();
        bounds = union_all_items(&self.background_and_borders, bounds);
        bounds = union_all_items(&self.block_backgrounds_and_borders, bounds);
        bounds = union_all_items(&self.floats, bounds);
        bounds = union_all_items(&self.content, bounds);
        bounds = union_all_items(&self.positioned_content, bounds);
        bounds = union_all_items(&self.outlines, bounds);
        bounds
    }

    #[inline]
    fn get_section_mut(&mut self, section: DisplayListSection) -> &mut LinkedList<DisplayItem> {
        match section {
            DisplayListSection::BackgroundAndBorders => &mut self.background_and_borders,
            DisplayListSection::BlockBackgroundsAndBorders =>
                &mut self.block_backgrounds_and_borders,
            DisplayListSection::Floats => &mut self.floats,
            DisplayListSection::Content => &mut self.content,
            DisplayListSection::PositionedContent => &mut self.positioned_content,
            DisplayListSection::Outlines => &mut self.outlines,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DisplayListSection {
    BackgroundAndBorders,
    BlockBackgroundsAndBorders,
    Floats,
    Content,
    PositionedContent,
    Outlines,
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

    /// The layer info for this stacking context, if there is any.
    pub layer_info: Option<LayerInfo>,

    /// The LayerId of the last child layer of this stacking context.
    pub last_child_layer_info: Option<LayerInfo>,
}

impl StackingContext {
    /// Creates a new stacking context.
    #[inline]
    pub fn new(display_list: Box<DisplayList>,
               bounds: &Rect<Au>,
               overflow: &Rect<Au>,
               z_index: i32,
               filters: filter::T,
               blend_mode: mix_blend_mode::T,
               transform: Matrix4,
               perspective: Matrix4,
               establishes_3d_context: bool,
               scrolls_overflow_area: bool,
               layer_info: Option<LayerInfo>)
               -> StackingContext {
        let mut stacking_context = StackingContext {
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
            layer_info: layer_info,
            last_child_layer_info: None,
        };
        // webrender doesn't care about layers in the display list - it's handled internally.
        if !opts::get().use_webrender {
            StackingContextLayerCreator::add_layers_to_preserve_drawing_order(&mut stacking_context);
        }
        stacking_context
    }

    /// Draws the stacking context in the proper order according to the steps in CSS 2.1 § E.2.
    pub fn draw_into_context(&self,
                             display_list: &DisplayList,
                             paint_context: &mut PaintContext,
                             transform: &Matrix4,
                             clip_rect: Option<&Rect<Au>>) {
        let temporary_draw_target =
            paint_context.get_or_create_temporary_draw_target(&self.filters, self.blend_mode);

        display_list.draw_into_context(&temporary_draw_target,
                                       paint_context,
                                       transform,
                                       clip_rect);

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
        let transform = match self.layer_info {
            Some(..) => *transform,
            None => transform.mul(&self.transform),
        };

        // TODO(gw): This is a hack to avoid running the DL optimizer
        // on 3d transformed tiles. We should have a better solution
        // than just disabling the opts here.
        if paint_context.layer_kind == LayerKind::HasTransform ||
           opts::get().use_webrender {      // webrender takes care of all culling via aabb tree!
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

            let tile_size = Size2D::new(paint_context.screen_rect.as_f32().size.width,
                                        paint_context.screen_rect.as_f32().size.height);
            let tile_rect = Rect::new(Point2D::zero(), tile_size).to_untyped();
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
                    point: Point2D<Au>,
                    result: &mut Vec<DisplayItemMetadata>,
                    topmost_only: bool) {
        // Convert the point into stacking context local space
        let point = point - self.bounds.origin;

        debug_assert!(!topmost_only || result.is_empty());
        let inv_transform = self.transform.invert();
        let frac_point = inv_transform.transform_point(&Point2D::new(point.x.to_f32_px(),
                                                                     point.y.to_f32_px()));
        let point = Point2D::new(Au::from_f32_px(frac_point.x), Au::from_f32_px(frac_point.y));
        self.display_list.hit_test(point, result, topmost_only)
    }

    pub fn print(&self, title: String) {
        let mut print_tree = PrintTree::new(title);
        self.print_with_tree(&mut print_tree);
    }

    fn print_with_tree(&self, print_tree: &mut PrintTree) {
        if self.layer_info.is_some() {
            print_tree.new_level(format!("Layered StackingContext at {:?} with overflow {:?}:",
                                         self.bounds,
                                         self.overflow));
        } else {
            print_tree.new_level(format!("StackingContext at {:?} with overflow {:?}:",
                                         self.bounds,
                                         self.overflow));
        }
        self.display_list.print_with_tree(print_tree);
        print_tree.end_level();
    }

    fn scroll_policy(&self) -> ScrollPolicy {
        match self.layer_info {
            Some(ref layer_info) => layer_info.scroll_policy,
            None => ScrollPolicy::Scrollable,
        }
    }

    fn get_layer_info(&mut self, layer_id: LayerId) -> &mut LayerInfo {
        for layer_info in self.display_list.layer_info.iter_mut() {
            if layer_info.layer_id == layer_id {
                return layer_info;
            }
        }

        panic!("Could not find LayerInfo with id: {:?}", layer_id);
    }
}

struct StackingContextLayerCreator {
    display_list_for_next_layer: Option<DisplayList>,
    next_layer_info: Option<LayerInfo>,
    building_ordering_layer: bool,
    last_child_layer_info: Option<LayerInfo>,
}

impl StackingContextLayerCreator {
    fn new() -> StackingContextLayerCreator {
        // webrender doesn't care about layers in the display list - it's handled internally.
        debug_assert!(!opts::get().use_webrender);

        StackingContextLayerCreator {
            display_list_for_next_layer: None,
            next_layer_info: None,
            building_ordering_layer: false,
            last_child_layer_info: None,
        }
    }

    #[inline]
    fn add_layers_to_preserve_drawing_order(stacking_context: &mut StackingContext) {
        let mut state = StackingContextLayerCreator::new();

        // First we need to sort positioned content by z-index, so we can paint
        // it in order and also so that we can detect situations where unlayered
        // content should be on top of layered content.
        let positioned_content = mem::replace(&mut stacking_context.display_list.positioned_content,
                                              LinkedList::new());
        let mut sorted_positioned_content: SmallVec<[DisplayItem; 8]> = SmallVec::new();
        sorted_positioned_content.extend(positioned_content.into_iter());
        sorted_positioned_content.sort_by(|this, other| this.compare_zindex(other));

        // It's important here that we process all elements in paint order, so we can detect
        // situations where layers are needed to maintain paint order.
        state.layerize_display_list_section(DisplayListSection::BackgroundAndBorders,
                                            stacking_context);

        let mut remaining_positioned_content: SmallVec<[DisplayItem; 8]> = SmallVec::new();
        for item in sorted_positioned_content.into_iter() {
            if !item.has_negative_z_index() {
                remaining_positioned_content.push(item);
            } else {
                state.add_display_item(item, DisplayListSection::PositionedContent, stacking_context);
            }
        }

        state.layerize_display_list_section(DisplayListSection::BlockBackgroundsAndBorders,
                                            stacking_context);
        state.layerize_display_list_section(DisplayListSection::Floats, stacking_context);
        state.layerize_display_list_section(DisplayListSection::Content, stacking_context);

        for item in remaining_positioned_content.into_iter() {
            assert!(!item.has_negative_z_index());
            state.add_display_item(item, DisplayListSection::PositionedContent, stacking_context);
        }

        state.layerize_display_list_section(DisplayListSection::Outlines, stacking_context);

        state.finish_building_current_layer(stacking_context);
        stacking_context.last_child_layer_info = state.find_last_child_layer_info(stacking_context);
    }

    #[inline]
    fn layerize_display_list_section(&mut self,
                                     section: DisplayListSection,
                                     stacking_context: &mut StackingContext) {
        let section_list = stacking_context.display_list.get_section_mut(section).split_off(0);
        for item in section_list.into_iter() {
            self.add_display_item(item, section, stacking_context);
        }
    }

    #[inline]
    fn all_following_children_need_layers(&self) -> bool {
        self.next_layer_info.is_some()
    }

    #[inline]
    fn display_item_needs_layer(&mut self, item: &DisplayItem) -> bool {
        match *item {
            LayeredItemClass(_) => true,
            StackingContextClass(ref stacking_context) =>
                stacking_context.layer_info.is_some() || self.all_following_children_need_layers(),
            _ => self.all_following_children_need_layers(),
        }
    }

    #[inline]
    fn prepare_ordering_layer(&mut self,
                              stacking_context: &mut StackingContext) {
        if self.building_ordering_layer {
            assert!(self.next_layer_info.is_some());
            return;
        }

        let next_layer_info = Some(stacking_context
                                   .get_layer_info(self.next_layer_info.unwrap().layer_id)
                                   .next_with_scroll_policy(ScrollPolicy::Scrollable));
        self.finish_building_current_layer(stacking_context);
        self.next_layer_info = next_layer_info;

        self.building_ordering_layer = true;
    }

    fn add_display_item(&mut self,
                        item: DisplayItem,
                        section: DisplayListSection,
                        parent_stacking_context: &mut StackingContext) {
        if !self.display_item_needs_layer(&item) {
            if let DisplayItem::StackingContextClass(ref stacking_context) = item {
                // This StackingContext has a layered child somewhere in its children.
                // We need to give all new StackingContexts their own layer, so that they
                // draw on top of this layered child.
                if let Some(layer_info) = stacking_context.last_child_layer_info {
                    self.last_child_layer_info = stacking_context.last_child_layer_info;
                    self.building_ordering_layer = true;
                    self.next_layer_info =
                        Some(layer_info.clone().next_with_scroll_policy(ScrollPolicy::Scrollable));
                }
            }

            parent_stacking_context.display_list.add_to_section(item, section);
            return;
        }

        if let StackingContextClass(ref stacking_context) = item {
            // There is a bit of subtlety here. If this item is a stacking context,
            // yet doesn't have a layer assigned this code will fall through. This means that
            // stacking contexts that are promoted to layers will share layers with sibling
            // display items.
            let layer_info = stacking_context.layer_info.clone();
            if let Some(mut layer_info) = layer_info {
                self.finish_building_current_layer(parent_stacking_context);

                // We have started processing layered stacking contexts, so any stacking context that
                // we process from now on needs its own layer to ensure proper rendering order.
                self.building_ordering_layer = true;
                self.next_layer_info =
                    Some(layer_info.next_with_scroll_policy(parent_stacking_context.scroll_policy()));

                parent_stacking_context.display_list.layered_children.push_back(
                    Arc::new(PaintLayer::new_with_stacking_context(layer_info,
                                                                   stacking_context.clone(),
                                                                   color::transparent())));
                return;
            }
        }

        if let LayeredItemClass(item) = item {
            if let Some(ref next_layer_info) = self.next_layer_info {
                if item.layer_id == next_layer_info.layer_id && !self.building_ordering_layer {
                    return;
                }
            }

            self.finish_building_current_layer(parent_stacking_context);
            self.building_ordering_layer = false;
            self.next_layer_info =
                Some(parent_stacking_context.get_layer_info(item.layer_id).clone());
            self.add_display_item_to_display_list(item.item, section);
            return;
        }

        self.prepare_ordering_layer(parent_stacking_context);
        self.add_display_item_to_display_list(item, section);
    }

    fn add_display_item_to_display_list(&mut self,
                                        item: DisplayItem,
                                        section: DisplayListSection) {
        if self.display_list_for_next_layer.is_none() {
            self.display_list_for_next_layer = Some(DisplayList::new());
        }

        if let Some(ref mut display_list) = self.display_list_for_next_layer {
            display_list.add_to_section(item, section);
        }
    }

    fn find_last_child_layer_info(self,
                                  stacking_context: &mut StackingContext)
                                  -> Option<LayerInfo> {
        if let Some(layer) = stacking_context.display_list.layered_children.back() {
            return Some(LayerInfo::new(layer.id, ScrollPolicy::Scrollable, None));
        }

        self.last_child_layer_info
    }

    #[inline]
    fn finish_building_current_layer(&mut self, stacking_context: &mut StackingContext) {
        if let Some(display_list) = self.display_list_for_next_layer.take() {
            let layer_info = self.next_layer_info.take().unwrap();
            stacking_context.display_list.layered_children.push_back(
                Arc::new(PaintLayer::new_with_display_list(layer_info, display_list)));
        }
    }
}

/// One drawing command in the list.
#[derive(Clone, Deserialize, HeapSizeOf, Serialize)]
pub enum DisplayItem {
    SolidColorClass(Box<SolidColorDisplayItem>),
    TextClass(Box<TextDisplayItem>),
    ImageClass(Box<ImageDisplayItem>),
    WebGLClass(Box<WebGLDisplayItem>),
    BorderClass(Box<BorderDisplayItem>),
    GradientClass(Box<GradientDisplayItem>),
    LineClass(Box<LineDisplayItem>),
    BoxShadowClass(Box<BoxShadowDisplayItem>),
    StackingContextClass(Arc<StackingContext>),
    LayeredItemClass(Box<LayeredItem>),
    NoopClass(Box<BaseDisplayItem>),
    IframeClass(Box<IframeDisplayItem>),
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
    pub fn new(bounds: &Rect<Au>, metadata: DisplayItemMetadata, clip: &ClippingRegion)
               -> BaseDisplayItem {
        // Detect useless clipping regions here and optimize them to `ClippingRegion::max()`.
        // The painting backend may want to optimize out clipping regions and this makes it easier
        // for it to do so.
        BaseDisplayItem {
            bounds: *bounds,
            metadata: metadata,
            clip: if clip.does_not_clip_rect(bounds) {
                ClippingRegion::max()
            } else {
                (*clip).clone()
            }
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
            main: Rect::zero(),
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
            main: self.main.intersection(rect).unwrap_or(Rect::zero()),
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
        self.main.contains(point) &&
            self.complex.iter().all(|complex| complex.rect.contains(point))
    }

    /// Returns true if this clipping region might intersect the given rectangle and false
    /// otherwise. This is a quick, not a precise, test; it can yield false positives.
    #[inline]
    pub fn might_intersect_rect(&self, rect: &Rect<Au>) -> bool {
        self.main.intersects(rect) &&
            self.complex.iter().all(|complex| complex.rect.intersects(rect))
    }

    /// Returns true if this clipping region completely surrounds the given rect.
    #[inline]
    pub fn does_not_clip_rect(&self, rect: &Rect<Au>) -> bool {
        self.main.contains(&rect.origin) && self.main.contains(&rect.bottom_right()) &&
            self.complex.iter().all(|complex| {
                complex.rect.contains(&rect.origin) && complex.rect.contains(&rect.bottom_right())
            })
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


/// Metadata attached to each display item. This is useful for performing auxiliary threads with
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

#[derive(Clone, HeapSizeOf, Deserialize, Serialize)]
pub struct WebGLDisplayItem {
    pub base: BaseDisplayItem,
    #[ignore_heap_size_of = "Defined in webrender_traits"]
    pub context_id: WebGLContextId,
}


/// Paints an iframe.
#[derive(Clone, HeapSizeOf, Deserialize, Serialize)]
pub struct IframeDisplayItem {
    pub base: BaseDisplayItem,
    pub iframe: PipelineId,
}

/// Paints a gradient.
#[derive(Clone, Deserialize, HeapSizeOf, Serialize)]
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
#[derive(Clone, PartialEq, Debug, Copy, HeapSizeOf, Deserialize, Serialize)]
pub struct BorderRadii<T> {
    pub top_left: Size2D<T>,
    pub top_right: Size2D<T>,
    pub bottom_right: Size2D<T>,
    pub bottom_left: Size2D<T>,
}

impl<T> Default for BorderRadii<T> where T: Default, T: Clone {
    fn default() -> Self {
        let top_left = Size2D::new(Default::default(),
                                   Default::default());
        let top_right = Size2D::new(Default::default(),
                                    Default::default());
        let bottom_left = Size2D::new(Default::default(),
                                      Default::default());
        let bottom_right = Size2D::new(Default::default(),
                                       Default::default());
        BorderRadii { top_left: top_left,
                      top_right: top_right,
                      bottom_left: bottom_left,
                      bottom_right: bottom_right }
    }
}

impl BorderRadii<Au> {
    // Scale the border radii by the specified factor
    pub fn scale_by(&self, s: f32) -> BorderRadii<Au> {
        BorderRadii { top_left: BorderRadii::scale_corner_by(self.top_left, s),
                      top_right: BorderRadii::scale_corner_by(self.top_right, s),
                      bottom_left: BorderRadii::scale_corner_by(self.bottom_left, s),
                      bottom_right: BorderRadii::scale_corner_by(self.bottom_right, s) }
    }

    // Scale the border corner radius by the specified factor
    pub fn scale_corner_by(corner: Size2D<Au>, s: f32) -> Size2D<Au> {
        Size2D { width: corner.width.scale_by(s), height: corner.height.scale_by(s) }
    }
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
            top_left: Size2D { width: value.clone(), height: value.clone() },
            top_right: Size2D { width: value.clone(), height: value.clone() },
            bottom_right: Size2D { width: value.clone(), height: value.clone() },
            bottom_left: Size2D { width: value.clone(), height: value.clone() },
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

    /// The border radius of this shadow.
    ///
    /// TODO(pcwalton): Elliptical radii; different radii for each corner.
    pub border_radius: Au,

    /// How we should clip the result.
    pub clip_mode: BoxShadowClipMode,
}

/// Contains an item that should get its own layer during layer creation.
#[derive(Clone, HeapSizeOf, Deserialize, Serialize)]
pub struct LayeredItem {
    /// Fields common to all display items.
    pub item: DisplayItem,

    /// The id of the layer this item belongs to.
    pub layer_id: LayerId,
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
    fn draw_into_context(&self, transform: &Matrix4, paint_context: &mut PaintContext) {
        if let Some(base) = self.base() {
            let this_clip = &base.clip;
            match paint_context.transient_clip {
                Some(ref transient_clip) if transient_clip == this_clip => {}
                Some(_) | None => paint_context.push_transient_clip((*this_clip).clone()),
            }
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

            DisplayItem::WebGLClass(_) => {
                panic!("Shouldn't be here, WebGL display items are created just with webrender");
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
                                              box_shadow.clip_mode);
            }

            DisplayItem::StackingContextClass(ref stacking_context) => {
                let pixels_per_px = paint_context.screen_pixels_per_px();
                let new_transform =
                    transform.translate(stacking_context.bounds
                                                        .origin
                                                        .x
                                                        .to_nearest_pixel(pixels_per_px.get()) as AzFloat,
                                        stacking_context.bounds
                                                        .origin
                                                        .y
                                                        .to_nearest_pixel(pixels_per_px.get()) as AzFloat,
                                        0.0);
                stacking_context.optimize_and_draw_into_context(paint_context,
                                                                &new_transform,
                                                                Some(&stacking_context.overflow))

            }

            DisplayItem::LayeredItemClass(_) => panic!("Found layered item during drawing."),

            DisplayItem::NoopClass(_) => { }
            DisplayItem::IframeClass(..) => {}
        }
    }

    pub fn base(&self) -> Option<&BaseDisplayItem> {
        match *self {
            DisplayItem::SolidColorClass(ref solid_color) => Some(&solid_color.base),
            DisplayItem::TextClass(ref text) => Some(&text.base),
            DisplayItem::ImageClass(ref image_item) => Some(&image_item.base),
            DisplayItem::WebGLClass(ref webgl_item) => Some(&webgl_item.base),
            DisplayItem::BorderClass(ref border) => Some(&border.base),
            DisplayItem::GradientClass(ref gradient) => Some(&gradient.base),
            DisplayItem::LineClass(ref line) => Some(&line.base),
            DisplayItem::BoxShadowClass(ref box_shadow) => Some(&box_shadow.base),
            DisplayItem::LayeredItemClass(ref layered_item) => layered_item.item.base(),
            DisplayItem::NoopClass(ref base_item) => Some(base_item),
            DisplayItem::StackingContextClass(_) => None,
            DisplayItem::IframeClass(ref iframe) => Some(&iframe.base),
        }
    }

    pub fn bounds(&self) -> Rect<Au> {
        match *self {
            DisplayItem::StackingContextClass(ref stacking_context) => stacking_context.bounds,
            _ => self.base().unwrap().bounds,
        }
    }

    pub fn debug_with_level(&self, level: u32) {
        let mut indent = String::new();
        for _ in 0..level {
            indent.push_str("| ")
        }
        println!("{}+ {:?}", indent, self);
    }

    fn compare_zindex(&self, other: &DisplayItem) -> Ordering {
        match (self, other) {
            (&DisplayItem::StackingContextClass(ref this),
             &DisplayItem::StackingContextClass(ref other)) => this.z_index.cmp(&other.z_index),
            (&DisplayItem::StackingContextClass(ref this), _) => this.z_index.cmp(&0),
            (_, &DisplayItem::StackingContextClass(ref other)) => 0.cmp(&other.z_index),
            (_, _) => Ordering::Equal,
        }
    }

    fn has_negative_z_index(&self) -> bool {
        if let &DisplayItem::StackingContextClass(ref stacking_context) = self {
            stacking_context.z_index < 0
        } else {
            false
        }
    }
}

impl fmt::Debug for DisplayItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} @ {:?}",
            match *self {
                DisplayItem::SolidColorClass(ref solid_color) =>
                    format!("SolidColor rgba({}, {}, {}, {})",
                            solid_color.color.r,
                            solid_color.color.g,
                            solid_color.color.b,
                            solid_color.color.a),
                DisplayItem::TextClass(_) => "Text".to_owned(),
                DisplayItem::ImageClass(_) => "Image".to_owned(),
                DisplayItem::WebGLClass(_) => "WebGL".to_owned(),
                DisplayItem::BorderClass(_) => "Border".to_owned(),
                DisplayItem::GradientClass(_) => "Gradient".to_owned(),
                DisplayItem::LineClass(_) => "Line".to_owned(),
                DisplayItem::BoxShadowClass(_) => "BoxShadow".to_owned(),
                DisplayItem::StackingContextClass(_) => "StackingContext".to_owned(),
                DisplayItem::LayeredItemClass(ref layered_item) =>
                    format!("LayeredItem({:?})", layered_item.item),
                DisplayItem::NoopClass(_) => "Noop".to_owned(),
                DisplayItem::IframeClass(_) => "Iframe".to_owned(),
            },
            self.bounds(),
        )
    }
}
