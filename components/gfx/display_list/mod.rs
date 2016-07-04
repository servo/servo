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
use azure::azure_hl::Color;
use euclid::approxeq::ApproxEq;
use euclid::num::Zero;
use euclid::rect::TypedRect;
use euclid::side_offsets::SideOffsets2D;
use euclid::{Matrix2D, Matrix4D, Point2D, Rect, Size2D};
use fnv::FnvHasher;
use gfx_traits::print_tree::PrintTree;
use gfx_traits::{LayerId, ScrollPolicy, StackingContextId};
use ipc_channel::ipc::IpcSharedMemory;
use msg::constellation_msg::PipelineId;
use net_traits::image::base::{Image, PixelFormat};
use paint_context::PaintContext;
use range::Range;
use serde::de::{self, Deserialize, Deserializer, MapVisitor, Visitor};
use serde::ser::impls::MapIteratorVisitor;
use serde::ser::{Serialize, Serializer};
use std::cmp::{self, Ordering};
use std::collections::HashMap;
use std::fmt;
use std::hash::{BuildHasherDefault, Hash};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use style::computed_values::{border_style, filter, image_rendering, mix_blend_mode};
use style_traits::cursor::Cursor;
use text::TextRun;
use text::glyph::ByteIndex;
use util::geometry::{self, MAX_RECT, ScreenPx};
use webrender_traits::{self, WebGLContextId};

pub use style::dom::OpaqueNode;

// It seems cleaner to have layout code not mention Azure directly, so let's just reexport this for
// layout to use.
pub use azure::azure_hl::GradientStop;

/// The factor that we multiply the blur radius by in order to inflate the boundaries of display
/// items that involve a blur. This ensures that the display item boundaries include all the ink.
pub static BLUR_INFLATION_FACTOR: i32 = 3;

/// LayerInfo is used to store PaintLayer metadata during DisplayList construction.
/// It is also used for tracking LayerIds when creating layers to preserve ordering when
/// layered DisplayItems should render underneath unlayered DisplayItems.
#[derive(Clone, Copy, HeapSizeOf, Deserialize, Serialize)]
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

    /// The color of the background in this layer. Used for unpainted content.
    pub background_color: Color,
}

impl LayerInfo {
    pub fn new(id: LayerId,
               scroll_policy: ScrollPolicy,
               subpage_pipeline_id: Option<PipelineId>,
               background_color: Color)
               -> LayerInfo {
        LayerInfo {
            layer_id: id,
            scroll_policy: scroll_policy,
            subpage_pipeline_id: subpage_pipeline_id,
            next_layer_id: id.companion_layer_id(),
            background_color: background_color,
        }
    }
}

pub struct DisplayListTraversal<'a> {
    pub display_list: &'a DisplayList,
    pub current_item_index: usize,
    pub last_item_index: usize,
}

impl<'a> DisplayListTraversal<'a> {
    fn can_draw_item_at_index(&self, index: usize) -> bool {
       index <= self.last_item_index && index < self.display_list.list.len()
    }

    pub fn advance(&mut self, context: &StackingContext) -> Option<&'a DisplayItem> {
        if !self.can_draw_item_at_index(self.current_item_index) {
            return None
        }
        if self.display_list.list[self.current_item_index].base().stacking_context_id != context.id {
            return None
        }

        self.current_item_index += 1;
        Some(&self.display_list.list[self.current_item_index - 1])
    }

    fn current_item_offset(&self) -> u32 {
        self.display_list.get_offset_for_item(&self.display_list.list[self.current_item_index])
    }

    pub fn skip_past_stacking_context(&mut self, stacking_context: &StackingContext) {
        let next_stacking_context_offset =
            self.display_list.offsets[&stacking_context.id].outlines + 1;
        while self.can_draw_item_at_index(self.current_item_index + 1) &&
              self.current_item_offset() < next_stacking_context_offset {
            self.current_item_index += 1;
        }
    }
}

#[derive(HeapSizeOf, Deserialize, Serialize)]
pub struct StackingContextOffsets {
    pub start: u32,
    pub block_backgrounds_and_borders: u32,
    pub content: u32,
    pub outlines: u32,
}

/// A FNV-based hash map. This is not serializable by `serde` by default, so we provide an
/// implementation ourselves.
pub struct FnvHashMap<K, V>(pub HashMap<K, V, BuildHasherDefault<FnvHasher>>);

impl<K, V> Deref for FnvHashMap<K, V> {
    type Target = HashMap<K, V, BuildHasherDefault<FnvHasher>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V> DerefMut for FnvHashMap<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<K, V> Serialize for FnvHashMap<K, V> where K: Eq + Hash + Serialize, V: Serialize {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        serializer.serialize_map(MapIteratorVisitor::new(self.iter(), Some(self.len())))
    }
}

impl<K, V> Deserialize for FnvHashMap<K, V> where K: Eq + Hash + Deserialize, V: Deserialize {
    #[inline]
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
        deserializer.deserialize_map(FnvHashMapVisitor::new())
    }
}
/// A visitor that produces a map.
pub struct FnvHashMapVisitor<K, V> {
    marker: PhantomData<FnvHashMap<K, V>>,
}

impl<K, V> FnvHashMapVisitor<K, V> {
    /// Construct a `FnvHashMapVisitor<T>`.
    pub fn new() -> Self {
        FnvHashMapVisitor {
            marker: PhantomData,
        }
    }
}

impl<K, V> Visitor for FnvHashMapVisitor<K, V> where K: Eq + Hash + Deserialize, V: Deserialize {
    type Value = FnvHashMap<K, V>;

    #[inline]
    fn visit_unit<E>(&mut self) -> Result<FnvHashMap<K, V>, E> where E: de::Error {
        Ok(FnvHashMap(HashMap::with_hasher(Default::default())))
    }

    #[inline]
    fn visit_map<Visitor>(&mut self, mut visitor: Visitor)
                          -> Result<FnvHashMap<K, V>, Visitor::Error>
                          where Visitor: MapVisitor {
        let mut values = FnvHashMap(HashMap::with_hasher(Default::default()));
        while let Some((key, value)) = try!(visitor.visit()) {
            HashMap::insert(&mut values, key, value);
        }
        try!(visitor.end());
        Ok(values)
    }
}

#[derive(HeapSizeOf, Deserialize, Serialize)]
pub struct DisplayList {
    pub list: Vec<DisplayItem>,
    pub offsets: FnvHashMap<StackingContextId, StackingContextOffsets>,
    pub root_stacking_context: StackingContext,
}

impl DisplayList {
    pub fn new(mut root_stacking_context: StackingContext,
               items: Vec<DisplayItem>)
               -> DisplayList {
        let mut offsets = FnvHashMap(HashMap::with_hasher(Default::default()));
        DisplayList::sort_and_count_stacking_contexts(&mut root_stacking_context, &mut offsets, 0);

        let mut display_list = DisplayList {
            list: items,
            offsets: offsets,
            root_stacking_context: root_stacking_context,
        };
        display_list.sort();
        display_list
    }

    pub fn get_offset_for_item(&self, item: &DisplayItem) -> u32 {
        let offsets = &self.offsets[&item.base().stacking_context_id];
        match item.base().section {
            DisplayListSection::BackgroundAndBorders => offsets.start,
            DisplayListSection::BlockBackgroundsAndBorders =>
                offsets.block_backgrounds_and_borders,
            DisplayListSection::Content => offsets.content,
            DisplayListSection::Outlines => offsets.outlines,
        }
    }

    fn sort(&mut self) {
        let mut list = mem::replace(&mut self.list, Vec::new());

        list.sort_by(|a, b| {
            if a.base().stacking_context_id == b.base().stacking_context_id {
                return a.base().section.cmp(&b.base().section);
            }
            self.get_offset_for_item(a).cmp(&self.get_offset_for_item(b))
        });

        mem::replace(&mut self.list, list);
    }

    pub fn print(&self) {
        let mut print_tree = PrintTree::new("Display List".to_owned());
        self.print_with_tree(&mut print_tree);
    }

    fn sort_and_count_stacking_contexts(
            stacking_context: &mut StackingContext,
            offsets: &mut HashMap<StackingContextId,
                                  StackingContextOffsets,
                                  BuildHasherDefault<FnvHasher>>,
            mut current_offset: u32)
            -> u32 {
        stacking_context.children.sort();

        let start_offset = current_offset;
        let mut block_backgrounds_and_borders_offset = None;
        let mut content_offset = None;

        for child in stacking_context.children.iter_mut() {
            if child.z_index >= 0 {
                if block_backgrounds_and_borders_offset.is_none() {
                    current_offset += 1;
                    block_backgrounds_and_borders_offset = Some(current_offset);
                }

                if child.context_type != StackingContextType::PseudoFloat &&
                        content_offset.is_none() {
                    current_offset += 1;
                    content_offset = Some(current_offset);
                }
            }

            current_offset += 1;
            current_offset =
                DisplayList::sort_and_count_stacking_contexts(child, offsets, current_offset);
        }

        let block_backgrounds_and_borders_offset =
            block_backgrounds_and_borders_offset.unwrap_or_else(|| {
                current_offset += 1;
                current_offset
        });

        let content_offset = content_offset.unwrap_or_else(|| {
            current_offset += 1;
            current_offset
        });

        current_offset += 1;

        offsets.insert(
            stacking_context.id,
            StackingContextOffsets {
                start: start_offset,
                block_backgrounds_and_borders: block_backgrounds_and_borders_offset,
                content: content_offset,
                outlines: current_offset,
           });

        current_offset + 1
    }

    pub fn print_with_tree(&self, print_tree: &mut PrintTree) {
        print_tree.new_level("Items".to_owned());
        for item in &self.list {
            print_tree.add_item(format!("{:?} StackingContext: {:?}",
                                        item,
                                        item.base().stacking_context_id));
        }
        print_tree.end_level();

        print_tree.new_level("Stacking Contexts".to_owned());
        self.root_stacking_context.print_with_tree(print_tree);
        print_tree.end_level();
    }

    /// Draws a single DisplayItem into the given PaintContext.
    pub fn draw_item_at_index_into_context(&self,
                                           paint_context: &mut PaintContext,
                                           transform: &Matrix4D<f32>,
                                           index: usize) {
        let old_transform = paint_context.draw_target.get_transform();
        paint_context.draw_target.set_transform(
            &Matrix2D::new(transform.m11, transform.m12,
                           transform.m21, transform.m22,
                           transform.m41, transform.m42));

        let item = &self.list[index];
        item.draw_into_context(paint_context);

        paint_context.draw_target.set_transform(&old_transform);
    }

    pub fn find_stacking_context<'a>(&'a self,
                                     stacking_context_id: StackingContextId)
                                     -> Option<&'a StackingContext> {
        fn find_stacking_context_in_stacking_context<'a>(stacking_context: &'a StackingContext,
                                                         stacking_context_id: StackingContextId)
                                                          -> Option<&'a StackingContext> {
            if stacking_context.id == stacking_context_id {
                return Some(stacking_context);
            }

            for kid in stacking_context.children.iter() {
                let result = find_stacking_context_in_stacking_context(kid, stacking_context_id);
                if result.is_some() {
                    return result;
                }
            }

            None
        }
        find_stacking_context_in_stacking_context(&self.root_stacking_context,
                                                  stacking_context_id)
    }

    /// Draws the DisplayList in order.
    pub fn draw_into_context<'a>(&self,
                                 paint_context: &mut PaintContext,
                                 transform: &Matrix4D<f32>,
                                 stacking_context_id: StackingContextId,
                                 start: usize,
                                 end: usize) {
        let stacking_context = self.find_stacking_context(stacking_context_id).unwrap();
        let mut traversal = DisplayListTraversal {
            display_list: self,
            current_item_index: start,
            last_item_index: end,
        };
        self.draw_stacking_context(stacking_context, &mut traversal, paint_context, transform);
    }

    fn draw_stacking_context_contents<'a>(&'a self,
                                          stacking_context: &StackingContext,
                                          traversal: &mut DisplayListTraversal<'a>,
                                          paint_context: &mut PaintContext,
                                          transform: &Matrix4D<f32>,
                                          tile_rect: Option<Rect<Au>>) {
        for child in stacking_context.children.iter() {
            while let Some(item) = traversal.advance(stacking_context) {
                if item.intersects_rect_in_parent_context(tile_rect) {
                    item.draw_into_context(paint_context);
                }
            }

            if child.intersects_rect_in_parent_context(tile_rect) {
                self.draw_stacking_context(child, traversal, paint_context, &transform);
            } else {
                traversal.skip_past_stacking_context(child);
            }
        }

        while let Some(item) = traversal.advance(stacking_context) {
            if item.intersects_rect_in_parent_context(tile_rect) {
                item.draw_into_context(paint_context);
            }
        }
    }


    fn draw_stacking_context<'a>(&'a self,
                                 stacking_context: &StackingContext,
                                 traversal: &mut DisplayListTraversal<'a>,
                                 paint_context: &mut PaintContext,
                                 transform: &Matrix4D<f32>) {
        if stacking_context.context_type != StackingContextType::Real {
            self.draw_stacking_context_contents(stacking_context,
                                                traversal,
                                                paint_context,
                                                transform,
                                                None);
            return;
        }

        let draw_target = paint_context.get_or_create_temporary_draw_target(
            &stacking_context.filters,
            stacking_context.blend_mode);

        // If a layer is being used, the transform for this layer
        // will be handled by the compositor.
        let old_transform = paint_context.draw_target.get_transform();
        let transform = match stacking_context.layer_info {
            Some(..) => *transform,
            None => {
                let pixels_per_px = paint_context.screen_pixels_per_px();
                let origin = &stacking_context.bounds.origin;
                transform.translate(
                    origin.x.to_nearest_pixel(pixels_per_px.get()) as AzFloat,
                    origin.y.to_nearest_pixel(pixels_per_px.get()) as AzFloat,
                    0.0).mul(&stacking_context.transform)
            }
        };

        {
            let mut paint_subcontext = PaintContext {
                draw_target: draw_target.clone(),
                font_context: &mut *paint_context.font_context,
                page_rect: paint_context.page_rect,
                screen_rect: paint_context.screen_rect,
                clip_rect: Some(stacking_context.overflow),
                transient_clip: None,
                layer_kind: paint_context.layer_kind,
            };

            // Set up our clip rect and transform.
            paint_subcontext.draw_target.set_transform(
                &Matrix2D::new(transform.m11, transform.m12,
                               transform.m21, transform.m22,
                               transform.m41, transform.m42));
            paint_subcontext.push_clip_if_applicable();

            self.draw_stacking_context_contents(
                stacking_context,
                traversal,
                &mut paint_subcontext,
                &transform,
                Some(transformed_tile_rect(paint_context.screen_rect, &transform)));

            paint_subcontext.remove_transient_clip_if_applicable();
            paint_subcontext.pop_clip_if_applicable();
        }

        draw_target.set_transform(&old_transform);
        paint_context.draw_temporary_draw_target_if_necessary(
            &draw_target, &stacking_context.filters, stacking_context.blend_mode);
    }

    /// Return all nodes containing the point of interest, bottommost first,
    /// and respecting the `pointer-events` CSS property.
    pub fn hit_test(&self, point: &Point2D<Au>, scroll_offsets: &ScrollOffsetMap)
                    -> Vec<DisplayItemMetadata> {
        let mut traversal = DisplayListTraversal {
            display_list: self,
            current_item_index: 0,
            last_item_index: self.list.len() - 1,
        };
        let mut result = Vec::new();
        self.root_stacking_context.hit_test(&mut traversal, point, scroll_offsets, &mut result);
        result
    }
}

fn transformed_tile_rect(tile_rect: TypedRect<ScreenPx, usize>, transform: &Matrix4D<f32>) -> Rect<Au> {
    // Invert the current transform, then use this to back transform
    // the tile rect (placed at the origin) into the space of this
    // stacking context.
    let inverse_transform = transform.invert();
    let inverse_transform_2d = Matrix2D::new(inverse_transform.m11, inverse_transform.m12,
                                             inverse_transform.m21, inverse_transform.m22,
                                             inverse_transform.m41, inverse_transform.m42);
    let tile_size = Size2D::new(tile_rect.as_f32().size.width, tile_rect.as_f32().size.height);
    let tile_rect = Rect::new(Point2D::zero(), tile_size).to_untyped();
    geometry::f32_rect_to_au_rect(inverse_transform_2d.transform_rect(&tile_rect))
}


/// Display list sections that make up a stacking context. Each section  here refers
/// to the steps in CSS 2.1 Appendix E.
///
#[derive(Clone, Copy, Debug, Deserialize, Eq, HeapSizeOf, Ord, PartialEq, PartialOrd, RustcEncodable, Serialize)]
pub enum DisplayListSection {
    BackgroundAndBorders,
    BlockBackgroundsAndBorders,
    Content,
    Outlines,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, HeapSizeOf, Ord, PartialEq, PartialOrd, RustcEncodable, Serialize)]
pub enum StackingContextType {
    Real,
    PseudoPositioned,
    PseudoFloat,
}

#[derive(HeapSizeOf, Deserialize, Serialize)]
/// Represents one CSS stacking context, which may or may not have a hardware layer.
pub struct StackingContext {
    /// The ID of this StackingContext for uniquely identifying it.
    pub id: StackingContextId,

    /// The type of this StackingContext. Used for collecting and sorting.
    pub context_type: StackingContextType,

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
    pub transform: Matrix4D<f32>,

    /// The perspective matrix to be applied to children.
    pub perspective: Matrix4D<f32>,

    /// Whether this stacking context creates a new 3d rendering context.
    pub establishes_3d_context: bool,

    /// Whether this stacking context scrolls its overflow area.
    pub scrolls_overflow_area: bool,

    /// The layer info for this stacking context, if there is any.
    pub layer_info: Option<LayerInfo>,

    /// Children of this StackingContext.
    pub children: Vec<Box<StackingContext>>,
}

impl StackingContext {
    /// Creates a new stacking context.
    #[inline]
    pub fn new(id: StackingContextId,
               context_type: StackingContextType,
               bounds: &Rect<Au>,
               overflow: &Rect<Au>,
               z_index: i32,
               filters: filter::T,
               blend_mode: mix_blend_mode::T,
               transform: Matrix4D<f32>,
               perspective: Matrix4D<f32>,
               establishes_3d_context: bool,
               scrolls_overflow_area: bool,
               layer_info: Option<LayerInfo>)
               -> StackingContext {
        StackingContext {
            id: id,
            context_type: context_type,
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
            children: Vec::new(),
        }
    }

    pub fn hit_test<'a>(&self,
                        traversal: &mut DisplayListTraversal<'a>,
                        point: &Point2D<Au>,
                        scroll_offsets: &ScrollOffsetMap,
                        result: &mut Vec<DisplayItemMetadata>) {
        // Convert the point into stacking context local transform space.
        let mut point = if self.context_type == StackingContextType::Real {
            let point = *point - self.bounds.origin;
            let inv_transform = self.transform.invert();
            let frac_point = inv_transform.transform_point(&Point2D::new(point.x.to_f32_px(),
                                                                         point.y.to_f32_px()));
            Point2D::new(Au::from_f32_px(frac_point.x), Au::from_f32_px(frac_point.y))
        } else {
            *point
        };

        // Adjust the point to account for the scroll offset if necessary. This can only happen
        // when WebRender is in use.
        //
        // We don't perform this adjustment on the root stacking context because the DOM-side code
        // has already translated the point for us (e.g. in `Document::handle_mouse_move_event()`)
        // by now.
        if self.id != StackingContextId::root() {
            if let Some(scroll_offset) = scroll_offsets.get(&self.id) {
                point.x -= Au::from_f32_px(scroll_offset.x);
                point.y -= Au::from_f32_px(scroll_offset.y);
            }
        }

        for child in self.children.iter() {
            while let Some(item) = traversal.advance(self) {
                if let Some(meta) = item.hit_test(point) {
                    result.push(meta);
                }
            }
            child.hit_test(traversal, &point, scroll_offsets, result);
        }

        while let Some(item) = traversal.advance(self) {
            if let Some(meta) = item.hit_test(point) {
                result.push(meta);
            }
        }
    }

    pub fn print_with_tree(&self, print_tree: &mut PrintTree) {
        print_tree.new_level(format!("{:?}", self));
        for kid in self.children.iter() {
            kid.print_with_tree(print_tree);
        }
        print_tree.end_level();
    }

    pub fn intersects_rect_in_parent_context(&self, rect: Option<Rect<Au>>) -> bool {
        // We only do intersection checks for real stacking contexts, since
        // pseudo stacking contexts might not have proper position information.
        if self.context_type != StackingContextType::Real {
            return true;
        }

        let rect = match rect {
            Some(ref rect) => rect,
            None => return true,
        };

        // Transform this stacking context to get it into the same space as
        // the parent stacking context.
        let origin_x = self.bounds.origin.x.to_f32_px();
        let origin_y = self.bounds.origin.y.to_f32_px();

        let transform = Matrix4D::identity().translate(origin_x,
                                                       origin_y,
                                                       0.0)
                                           .mul(&self.transform);
        let transform_2d = Matrix2D::new(transform.m11, transform.m12,
                                         transform.m21, transform.m22,
                                         transform.m41, transform.m42);

        let overflow = geometry::au_rect_to_f32_rect(self.overflow);
        let overflow = transform_2d.transform_rect(&overflow);
        let overflow = geometry::f32_rect_to_au_rect(overflow);

        rect.intersects(&overflow)
    }
}

impl Ord for StackingContext {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.z_index != 0 || other.z_index != 0 {
            return self.z_index.cmp(&other.z_index);
        }

        match (self.context_type, other.context_type) {
            (StackingContextType::PseudoFloat, StackingContextType::PseudoFloat) => Ordering::Equal,
            (StackingContextType::PseudoFloat, _) => Ordering::Less,
            (_, StackingContextType::PseudoFloat) => Ordering::Greater,
            (_, _) => Ordering::Equal,
        }
    }
}

impl PartialOrd for StackingContext {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for StackingContext {}
impl PartialEq for StackingContext {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl fmt::Debug for StackingContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let type_string = if self.layer_info.is_some() {
            "Layered StackingContext"
        } else if self.context_type == StackingContextType::Real {
            "StackingContext"
        } else {
            "Pseudo-StackingContext"
        };

        let scrollable_string = if self.scrolls_overflow_area {
            " (scrolls overflow area)"
        } else {
            ""
        };

        write!(f, "{}{} at {:?} with overflow {:?}: {:?}",
               type_string,
               scrollable_string,
               self.bounds,
               self.overflow,
               self.id)
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
    LayeredItemClass(Box<LayeredItem>),
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

    /// The section of the display list that this item belongs to.
    pub section: DisplayListSection,

    /// The id of the stacking context this item belongs to.
    pub stacking_context_id: StackingContextId,
}

impl BaseDisplayItem {
    #[inline(always)]
    pub fn new(bounds: &Rect<Au>,
               metadata: DisplayItemMetadata,
               clip: &ClippingRegion,
               section: DisplayListSection,
               stacking_context_id: StackingContextId)
               -> BaseDisplayItem {
        // Detect useless clipping regions here and optimize them to `ClippingRegion::max()`.
        // The painting backend may want to optimize out clipping regions and this makes it easier
        // for it to do so.
        BaseDisplayItem {
            bounds: *bounds,
            metadata: metadata,
            clip: if clip.does_not_clip_rect(&bounds) {
                ClippingRegion::max()
            } else {
                (*clip).clone()
            },
            section: section,
            stacking_context_id: stacking_context_id,
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

    /// Mutates this clipping region to intersect with the given rectangle.
    ///
    /// TODO(pcwalton): This could more eagerly eliminate complex clipping regions, at the cost of
    /// complexity.
    #[inline]
    pub fn intersect_rect(&mut self, rect: &Rect<Au>) {
        self.main = self.main.intersection(rect).unwrap_or(Rect::zero())
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
    pub fn intersect_with_rounded_rect(&mut self, rect: &Rect<Au>, radii: &BorderRadii<Au>) {
        let new_complex_region = ComplexClippingRegion {
            rect: *rect,
            radii: *radii,
        };

        // FIXME(pcwalton): This is O(n²) worst case for disjoint clipping regions. Is that OK?
        // They're slow anyway…
        //
        // Possibly relevant if we want to do better:
        //
        //     http://www.inrg.csie.ntu.edu.tw/algorithm2014/presentation/D&C%20Lee-84.pdf
        for existing_complex_region in &mut self.complex {
            if existing_complex_region.completely_encloses(&new_complex_region) {
                *existing_complex_region = new_complex_region;
                return
            }
            if new_complex_region.completely_encloses(existing_complex_region) {
                return
            }
        }

        self.complex.push(ComplexClippingRegion {
            rect: *rect,
            radii: *radii,
        });
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

impl ComplexClippingRegion {
    // TODO(pcwalton): This could be more aggressive by considering points that touch the inside of
    // the border radius ellipse.
    fn completely_encloses(&self, other: &ComplexClippingRegion) -> bool {
        let left = cmp::max(self.radii.top_left.width, self.radii.bottom_left.width);
        let top = cmp::max(self.radii.top_left.height, self.radii.top_right.height);
        let right = cmp::max(self.radii.top_right.width, self.radii.bottom_right.width);
        let bottom = cmp::max(self.radii.bottom_left.height, self.radii.bottom_right.height);
        let interior = Rect::new(Point2D::new(self.rect.origin.x + left, self.rect.origin.y + top),
                                 Size2D::new(self.rect.size.width - left - right,
                                             self.rect.size.height - top - bottom));
        interior.origin.x <= other.rect.origin.x && interior.origin.y <= other.rect.origin.y &&
            interior.max_x() >= other.rect.max_x() && interior.max_y() >= other.rect.max_y()
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
    pub range: Range<ByteIndex>,

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

    pub webrender_image: WebRenderImageInfo,

    #[ignore_heap_size_of = "Because it is non-owning"]
    pub image_data: Option<Arc<IpcSharedMemory>>,

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
    pub layer_info: LayerInfo,
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
                paint_context.draw_image(
                    &image_item.base.bounds,
                    &image_item.stretch_size,
                    &image_item.webrender_image,
                    &image_item.image_data
                               .as_ref()
                               .expect("Non-WR painting needs image data!")[..],
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

            DisplayItem::LayeredItemClass(ref item) => item.item.draw_into_context(paint_context),

            DisplayItem::IframeClass(..) => {}
        }
    }

    pub fn intersects_rect_in_parent_context(&self, rect: Option<Rect<Au>>) -> bool {
        let rect = match rect {
            Some(ref rect) => rect,
            None => return true,
        };

        if !rect.intersects(&self.bounds()) {
            return false;
        }

        self.base().clip.might_intersect_rect(&rect)
    }

    pub fn base(&self) -> &BaseDisplayItem {
        match *self {
            DisplayItem::SolidColorClass(ref solid_color) => &solid_color.base,
            DisplayItem::TextClass(ref text) => &text.base,
            DisplayItem::ImageClass(ref image_item) => &image_item.base,
            DisplayItem::WebGLClass(ref webgl_item) => &webgl_item.base,
            DisplayItem::BorderClass(ref border) => &border.base,
            DisplayItem::GradientClass(ref gradient) => &gradient.base,
            DisplayItem::LineClass(ref line) => &line.base,
            DisplayItem::BoxShadowClass(ref box_shadow) => &box_shadow.base,
            DisplayItem::LayeredItemClass(ref layered_item) => layered_item.item.base(),
            DisplayItem::IframeClass(ref iframe) => &iframe.base,
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

    fn hit_test(&self, point: Point2D<Au>) -> Option<DisplayItemMetadata> {
        // TODO(pcwalton): Use a precise algorithm here. This will allow us to properly hit
        // test elements with `border-radius`, for example.
        let base_item = self.base();
        if !base_item.clip.might_intersect_point(&point) {
            // Clipped out.
            return None;
        }
        if !self.bounds().contains(&point) {
            // Can't possibly hit.
            return None;
        }
        if base_item.metadata.pointing.is_none() {
            // `pointer-events` is `none`. Ignore this item.
            return None;
        }

        match *self {
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
                    return None;
                }
            }
            DisplayItem::BoxShadowClass(_) => {
                // Box shadows can never be hit.
                return None;
            }
            _ => {}
        }

        Some(base_item.metadata)
    }
}

impl fmt::Debug for DisplayItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} @ {:?} {:?}",
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
                DisplayItem::LayeredItemClass(ref layered_item) =>
                    format!("LayeredItem({:?})", layered_item.item),
                DisplayItem::IframeClass(_) => "Iframe".to_owned(),
            },
            self.bounds(),
            self.base().clip
        )
    }
}

#[derive(Copy, Clone, HeapSizeOf, Deserialize, Serialize)]
pub struct WebRenderImageInfo {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    #[ignore_heap_size_of = "WebRender traits type, and tiny"]
    pub key: Option<webrender_traits::ImageKey>,
}

impl WebRenderImageInfo {
    #[inline]
    pub fn from_image(image: &Image) -> WebRenderImageInfo {
        WebRenderImageInfo {
            width: image.width,
            height: image.height,
            format: image.format,
            key: image.id,
        }
    }
}

/// The type of the scroll offset list. This is only populated if WebRender is in use.
pub type ScrollOffsetMap = HashMap<StackingContextId, Point2D<f32>>;

