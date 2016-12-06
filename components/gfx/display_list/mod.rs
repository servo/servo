/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Useful methods and intermediate data structures for interacting with WebRender display lists.

use app_units::Au;
use euclid::{Matrix4D, Point2D, Rect, Size2D};
use euclid::num::{One, Zero};
use euclid::rect::TypedRect;
use gfx_traits::{ScrollRootIdMethods, StackingContextIdMethods};
use gfx_traits::print_tree::PrintTree;
use msg::constellation_msg::PipelineId;
use net_traits::image::base::{Image, PixelFormat};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::f32;
use std::io::Write;
use style_traits::cursor::Cursor;
use termcolor::{self, Buffer, ColorSpec, WriteColor};
use util::geometry;
use webrender_traits::{self, AuxiliaryListsBuilder, BorderRadius, BorderSide, BorderStyle};
use webrender_traits::{BoxShadowClipMode, ColorF, DisplayItemId, DisplayListBuilder, FilterOp};
use webrender_traits::{FontKey, GlyphInstance, GradientStop, ImageRendering, MixBlendMode};
use webrender_traits::{ScrollLayerId, ScrollLayerInfo, ScrollPolicy, ServoScrollRootId};
use webrender_traits::{StackingContextId, WebGLContextId};

pub use style::dom::OpaqueNode;

/// The factor that we multiply the blur radius by in order to inflate the boundaries of display
/// items that involve a blur. This ensures that the display item boundaries include all the ink.
pub static BLUR_INFLATION_FACTOR: f32 = 3.0;

pub trait DisplayListHitTesting {
    fn hit_test_display_list(&self,
                             translated_point: &Point2D<Au>,
                             client_point: &Point2D<Au>,
                             scroll_offsets: &ScrollOffsetMap,
                             metadatas: &[DisplayItemMetadata])
                             -> Vec<DisplayItemMetadata>;
    fn hit_test_contents<'a>(&self,
                             traversal: &mut DisplayListTraversal<'a>,
                             translated_point: &Point2D<Au>,
                             client_point: &Point2D<Au>,
                             scroll_offsets: &ScrollOffsetMap,
                             metadatas: &[DisplayItemMetadata],
                             result: &mut Vec<DisplayItemMetadata>);
    fn hit_test_scroll_root<'a>(&self,
                                traversal: &mut DisplayListTraversal<'a>,
                                scroll_layer_id: ScrollLayerId,
                                translated_point: Point2D<Au>,
                                client_point: &Point2D<Au>,
                                scroll_offsets: &ScrollOffsetMap,
                                metadatas: &[DisplayItemMetadata],
                                result: &mut Vec<DisplayItemMetadata>);
    fn hit_test_stacking_context<'a>(&self,
                                     traversal: &mut DisplayListTraversal<'a>,
                                     stacking_context: &webrender_traits::StackingContext,
                                     translated_point: &Point2D<Au>,
                                     client_point: &Point2D<Au>,
                                     scroll_offsets: &ScrollOffsetMap,
                                     metadatas: &[DisplayItemMetadata],
                                     result: &mut Vec<DisplayItemMetadata>);
}

impl DisplayListHitTesting for DisplayListBuilder {
    /// Return all nodes containing the point of interest, bottommost first, and
    /// respecting the `pointer-events` CSS property.
    fn hit_test_display_list(&self,
                             translated_point: &Point2D<Au>,
                             client_point: &Point2D<Au>,
                             scroll_offsets: &ScrollOffsetMap,
                             metadatas: &[DisplayItemMetadata])
                             -> Vec<DisplayItemMetadata> {
        let mut result = Vec::new();
        let mut traversal = DisplayListTraversal::new(self);
        self.hit_test_contents(&mut traversal,
                               translated_point,
                               client_point,
                               scroll_offsets,
                               metadatas,
                               &mut result);
        result
    }

    fn hit_test_contents<'a>(&self,
                             traversal: &mut DisplayListTraversal<'a>,
                             translated_point: &Point2D<Au>,
                             client_point: &Point2D<Au>,
                             scroll_offsets: &ScrollOffsetMap,
                             metadatas: &[DisplayItemMetadata],
                             result: &mut Vec<DisplayItemMetadata>) {
        while let Some(item) = traversal.next() {
            match item.item {
                webrender_traits::SpecificDisplayItem::PushStackingContext(ref item) => {
                    self.hit_test_stacking_context(traversal,
                                                   &item.stacking_context,
                                                   translated_point,
                                                   client_point,
                                                   scroll_offsets,
                                                   metadatas,
                                                   result);
                }
                webrender_traits::SpecificDisplayItem::PushScrollLayer(ref item) => {
                    self.hit_test_scroll_root(traversal,
                                              item.id,
                                              *translated_point,
                                              client_point,
                                              scroll_offsets,
                                              metadatas,
                                              result);
                }
                webrender_traits::SpecificDisplayItem::PopStackingContext |
                webrender_traits::SpecificDisplayItem::PopScrollLayer => return,
                _ => {
                    if let Some(meta) = item.hit_test(item.id,
                                                      translated_point,
                                                      &self.auxiliary_lists_builder,
                                                      metadatas) {
                        result.push(meta);
                    }
                }
            }
        }
    }

    fn hit_test_scroll_root<'a>(&self,
                                traversal: &mut DisplayListTraversal<'a>,
                                scroll_layer_id: ScrollLayerId,
                                mut translated_point: Point2D<Au>,
                                client_point: &Point2D<Au>,
                                scroll_offsets: &ScrollOffsetMap,
                                metadatas: &[DisplayItemMetadata],
                                result: &mut Vec<DisplayItemMetadata>) {
        // Adjust the translated point to account for the scroll offset if
        // necessary.
        //
        // We don't perform this adjustment on the root stacking context because
        // the DOM-side code has already translated the point for us (e.g. in
        // `Window::hit_test_query()`) by now.
        if let ScrollLayerInfo::Scrollable(_, scroll_root_id) = scroll_layer_id.info {
            if let Some(scroll_offset) = scroll_offsets.get(&scroll_root_id) {
                translated_point.x -= Au::from_f32_px(scroll_offset.x);
                translated_point.y -= Au::from_f32_px(scroll_offset.y);
            }
        }
        self.hit_test_contents(traversal,
                               &translated_point,
                               client_point,
                               scroll_offsets,
                               metadatas,
                               result);
    }

    fn hit_test_stacking_context<'a>(&self,
                                     traversal: &mut DisplayListTraversal<'a>,
                                     stacking_context: &webrender_traits::StackingContext,
                                     translated_point: &Point2D<Au>,
                                     client_point: &Point2D<Au>,
                                     scroll_offsets: &ScrollOffsetMap,
                                     metadatas: &[DisplayItemMetadata],
                                     result: &mut Vec<DisplayItemMetadata>) {
        // Convert the parent translated point into stacking context local transform space if the
        // stacking context isn't fixed.  If it's fixed, we need to use the client point anyway.
        let is_fixed = stacking_context.scroll_policy == ScrollPolicy::Fixed;
        let translated_point = if is_fixed {
            *client_point
        } else {
            let point = geometry::au_point_to_f32_point(translated_point) -
                stacking_context.bounds.origin;
            let inv_transform = stacking_context.transform.inverse().unwrap();
            let frac_point = inv_transform.transform_point(&point);
            geometry::f32_point_to_au_point(&frac_point)
        };

        self.hit_test_contents(traversal,
                               &translated_point,
                               client_point,
                               scroll_offsets,
                               metadatas,
                               result);
    }
}

pub struct DisplayList {
    pub list: Vec<DisplayItem>,
}

pub struct DisplayListTraversal<'a> {
    pub display_list: &'a DisplayListBuilder,
    pub stacking_context_stack: Vec<StackingContextId>,
    pub next_item_index: usize,
    pub first_item_index: usize,
    pub last_item_index: usize,
}

impl<'a> DisplayListTraversal<'a> {
    pub fn new(display_list: &'a DisplayListBuilder) -> DisplayListTraversal {
        DisplayListTraversal {
            display_list: display_list,
            stacking_context_stack: vec![],
            next_item_index: 0,
            first_item_index: 0,
            last_item_index: display_list.list.len(),
        }
    }

    pub fn new_partial(display_list: &'a DisplayListBuilder,
                       stacking_context_id: StackingContextId,
                       start: usize,
                       end: usize)
                       -> DisplayListTraversal {
        debug_assert!(start <= end);
        debug_assert!(display_list.list.len() > start);
        debug_assert!(display_list.list.len() > end);

        let stacking_context_start = display_list.list[0..start].iter().rposition(|item|
            match item.item {
                webrender_traits::SpecificDisplayItem::PushStackingContext(ref item) =>
                    item.stacking_context.id == stacking_context_id,
                _ => false,
            }).unwrap_or(start);
        debug_assert!(stacking_context_start <= start);

        DisplayListTraversal {
            display_list: display_list,
            stacking_context_stack: vec![stacking_context_id],
            next_item_index: stacking_context_start,
            first_item_index: start,
            last_item_index: end + 1,
        }
    }

    pub fn previous_item_id(&self) -> usize {
        self.next_item_index - 1
    }

    pub fn skip_to_end_of_stacking_context(&mut self, id: StackingContextId) {
        let stacking_context_stack = &mut self.stacking_context_stack;
        self.next_item_index = self.display_list.list[self.next_item_index..].iter()
                                                                             .position(|item| {
            match item.item {
                webrender_traits::SpecificDisplayItem::PopStackingContext => {
                    stacking_context_stack.pop() == Some(id)
                }
                _ => false,
            }
        }).unwrap_or(self.display_list.list.len());
        debug_assert!(self.next_item_index < self.last_item_index);
    }
}

impl<'a> Iterator for DisplayListTraversal<'a> {
    type Item = &'a webrender_traits::DisplayItem;

    fn next(&mut self) -> Option<&'a webrender_traits::DisplayItem> {
        while self.next_item_index < self.last_item_index {
            debug_assert!(self.next_item_index <= self.last_item_index);

            let reached_first_item = self.next_item_index >= self.first_item_index;
            let item = &self.display_list.list[self.next_item_index];

            self.next_item_index += 1;

            match item.item {
                webrender_traits::SpecificDisplayItem::PushStackingContext(ref item) => {
                    self.stacking_context_stack.push(item.stacking_context.id)
                }
                webrender_traits::SpecificDisplayItem::PopStackingContext => {
                    self.stacking_context_stack.pop();
                }
                _ => {}
            }

            if reached_first_item {
                return Some(item)
            }

            // Before we reach the starting item, we only emit stacking context boundaries. This
            // is to ensure that we properly position items when we are processing a display list
            // slice that is relative to a certain stacking context.
            match item.item {
                webrender_traits::SpecificDisplayItem::PushStackingContext(_) |
                webrender_traits::SpecificDisplayItem::PopStackingContext => {
                    return Some(item)
                }
                _ => {}
            }
        }

        None
    }
}

/// Display list sections that make up a stacking context. Each section here refers
/// to the steps in CSS 2.1 Appendix E.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DisplayListSection {
    BackgroundAndBorders,
    BlockBackgroundsAndBorders,
    Content,
    Outlines,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum StackingContextType {
    Real,
    PseudoPositioned,
    PseudoFloat,
    PseudoScrollingArea,
}

#[derive(Clone)]
/// Represents one CSS stacking context, which may or may not have a hardware layer.
pub struct StackingContext {
    /// The ID of this StackingContext for uniquely identifying it.
    pub id: StackingContextId,

    /// The type of this StackingContext. Used for collecting and sorting.
    pub context_type: StackingContextType,

    /// The position and size of this stacking context.
    pub bounds: Rect<f32>,

    /// The overflow rect for this stacking context in its coordinate system.
    pub overflow: Rect<f32>,

    /// The `z-index` for this stacking context.
    pub z_index: i32,

    /// CSS filters to be applied to this stacking context (including opacity).
    pub filters: Vec<FilterOp>,

    /// The blend mode with which this stacking context blends with its backdrop.
    pub blend_mode: MixBlendMode,

    /// A transform to be applied to this stacking context.
    pub transform: Matrix4D<f32>,

    /// The perspective matrix to be applied to children.
    pub perspective: Matrix4D<f32>,

    /// Whether this stacking context creates a new 3d rendering context.
    pub establishes_3d_context: bool,

    /// The scroll policy of this layer.
    pub scroll_policy: ScrollPolicy,

    /// Children of this StackingContext.
    pub children: Vec<StackingContext>,

    /// The id of the parent scrolling area that contains this StackingContext.
    pub parent_scroll_id: ServoScrollRootId,
}

impl StackingContext {
    /// Creates a new stacking context.
    #[inline]
    pub fn new(id: StackingContextId,
               context_type: StackingContextType,
               bounds: &Rect<f32>,
               overflow: &Rect<f32>,
               z_index: i32,
               filters: Vec<FilterOp>,
               blend_mode: MixBlendMode,
               transform: Matrix4D<f32>,
               perspective: Matrix4D<f32>,
               establishes_3d_context: bool,
               scroll_policy: ScrollPolicy,
               parent_scroll_id: ServoScrollRootId)
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
            scroll_policy: scroll_policy,
            children: Vec::new(),
            parent_scroll_id: parent_scroll_id,
        }
    }

    #[inline]
    pub fn root() -> StackingContext {
        StackingContext::new(StackingContextId::new(0),
                             StackingContextType::Real,
                             &Rect::zero(),
                             &Rect::zero(),
                             0,
                             Vec::new(),
                             MixBlendMode::Normal,
                             Matrix4D::identity(),
                             Matrix4D::identity(),
                             true,
                             ScrollPolicy::Scrollable,
                             ServoScrollRootId::root())
    }

    pub fn add_child(&mut self, mut child: StackingContext) {
        child.update_overflow_for_all_children();
        self.children.push(child);
    }

    pub fn child_at_mut(&mut self, index: usize) -> &mut StackingContext {
        &mut self.children[index]
    }

    pub fn children(&self) -> &[StackingContext] {
        &self.children
    }

    fn update_overflow_for_all_children(&mut self) {
        for child in self.children.iter() {
            if self.context_type == StackingContextType::Real &&
               child.context_type == StackingContextType::Real {
                // This child might be transformed, so we need to take into account
                // its transformed overflow rect too, but at the correct position.
                let overflow = child.overflow_rect_in_parent_space();
                self.overflow = self.overflow.union(&overflow);
            }
        }
    }

    fn overflow_rect_in_parent_space(&self) -> Rect<f32> {
        // Transform this stacking context to get it into the same space as
        // the parent stacking context.
        //
        // TODO: Take into account 3d transforms, even though it's a fairly
        // uncommon case.
        let origin_x = self.bounds.origin.x;
        let origin_y = self.bounds.origin.y;

        Matrix4D::identity().pre_translated(origin_x, origin_y, 0.0)
                            .pre_mul(&self.transform)
                            .to_2d()
                            .transform_rect(&self.overflow)
    }

    pub fn to_display_list_items(self, ids: [DisplayItemId; 2]) -> (DisplayItem, DisplayItem) {
        let mut base_item = BaseDisplayItem::empty(ids[0]);
        base_item.stacking_context_id = self.id;
        let pop_item = DisplayItem::PopStackingContext(Box::new(
            PopStackingContextItem {
                base: base_item.clone(),
                stacking_context_id: self.id,
            }
        ));

        let mut base_item = BaseDisplayItem::empty(ids[1]);
        base_item.stacking_context_id = self.id;
        let push_item = DisplayItem::PushStackingContext(Box::new(
            PushStackingContextItem {
                base: base_item,
                stacking_context: self,
            }
        ));

        (push_item, pop_item)
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

/// Defines a stacking context.
#[derive(Clone)]
pub struct ScrollRoot {
    /// The unique ID of this ScrollRoot.
    pub id: ServoScrollRootId,

    /// The unique ID of the parent of this ScrollRoot.
    pub parent_id: ServoScrollRootId,

    /// The position of this scroll root's frame in the parent stacking context.
    pub clip: Rect<f32>,

    /// The size of the contents that can be scrolled inside of the scroll root.
    pub size: Size2D<f32>,
}

impl ScrollRoot {
    pub fn to_push(&self, id: DisplayItemId) -> DisplayItem {
        DisplayItem::PushScrollRoot(box PushScrollRootItem {
            base: BaseDisplayItem::empty(id),
            scroll_root: self.clone(),
        })
    }
}

/// One drawing command in the list.
#[derive(Clone)]
pub enum DisplayItem {
    SolidColor(Box<SolidColorDisplayItem>),
    Text(Box<TextDisplayItem>),
    Image(Box<ImageDisplayItem>),
    WebGL(Box<WebGLDisplayItem>),
    Border(Box<BorderDisplayItem>),
    Gradient(Box<GradientDisplayItem>),
    BoxShadow(Box<BoxShadowDisplayItem>),
    Iframe(Box<IframeDisplayItem>),
    PushStackingContext(Box<PushStackingContextItem>),
    PopStackingContext(Box<PopStackingContextItem>),
    PushScrollRoot(Box<PushScrollRootItem>),
    PopScrollRoot(Box<BaseDisplayItem>),
}

/// Information common to all display items.
#[derive(Clone)]
pub struct BaseDisplayItem {
    /// The boundaries of the display item, in layer coordinates.
    pub bounds: Rect<f32>,

    /// An ID that uniquely identifies this display item in the display list.
    pub id: DisplayItemId,

    /// The region to clip to.
    pub clip: ClippingRegion,

    /// The section of the display list that this item belongs to.
    pub section: DisplayListSection,

    /// The id of the stacking context this item belongs to.
    pub stacking_context_id: StackingContextId,

    /// The id of the scroll root this item belongs to.
    pub scroll_root_id: ServoScrollRootId,
}

impl BaseDisplayItem {
    #[inline]
    pub fn new(bounds: &Rect<f32>,
               id: DisplayItemId,
               clip: &ClippingRegion,
               section: DisplayListSection,
               stacking_context_id: StackingContextId,
               scroll_root_id: ServoScrollRootId)
               -> BaseDisplayItem {
        // Detect useless clipping regions here and optimize them to `ClippingRegion::max()`.
        // The painting backend may want to optimize out clipping regions and this makes it easier
        // for it to do so.
        BaseDisplayItem {
            bounds: *bounds,
            id: id,
            clip: if clip.does_not_clip_rect(bounds) {
                ClippingRegion::max()
            } else {
                (*clip).clone()
            },
            section: section,
            stacking_context_id: stacking_context_id,
            scroll_root_id: scroll_root_id,
        }
    }

    #[inline]
    pub fn empty(id: DisplayItemId) -> BaseDisplayItem {
        BaseDisplayItem {
            bounds: TypedRect::zero(),
            id: id,
            clip: ClippingRegion::max(),
            section: DisplayListSection::Content,
            stacking_context_id: StackingContextId::root(),
            scroll_root_id: ServoScrollRootId::root(),
        }
    }
}

/// A clipping region for a display item. Currently, this can describe rectangles, rounded
/// rectangles (for `border-radius`), or arbitrary intersections of the two. Arbitrary transforms
/// are not supported because those are handled by the higher-level `StackingContext` abstraction.
#[derive(Clone, PartialEq)]
pub struct ClippingRegion {
    /// The main rectangular region. This does not include any corners.
    pub main: Rect<f32>,
    /// Any complex regions.
    ///
    /// TODO(pcwalton): Atomically reference count these? Not sure if it's worth the trouble.
    /// Measure and follow up.
    pub complex: Vec<ComplexClippingRegion>,
}

/// A complex clipping region. These don't as easily admit arbitrary intersection operations, so
/// they're stored in a list over to the side. Currently a complex clipping region is just a
/// rounded rectangle, but the CSS WGs will probably make us throw more stuff in here eventually.
#[derive(Clone, PartialEq)]
pub struct ComplexClippingRegion {
    /// The boundaries of the rectangle.
    pub rect: Rect<f32>,
    /// Border radii of this rectangle.
    pub radii: BorderRadius,
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
            main: geometry::au_rect_to_f32_rect(geometry::max_rect()),
            complex: Vec::new(),
        }
    }

    /// Returns a clipping region that represents the given rectangle.
    #[inline]
    pub fn from_rect(rect: &Rect<f32>) -> ClippingRegion {
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
    pub fn intersect_rect(&mut self, rect: &Rect<f32>) {
        self.main = self.main.intersection(rect).unwrap_or(Rect::zero())
    }

    /// Returns true if this clipping region might be nonempty. This can return false positives,
    /// but never false negatives.
    #[inline]
    pub fn might_be_nonempty(&self) -> bool {
        !self.main.is_empty()
    }

    /// Returns true if this clipping region might intersect the given rectangle and false
    /// otherwise. This is a quick, not a precise, test; it can yield false positives.
    #[inline]
    pub fn might_intersect_rect(&self, rect: &Rect<f32>) -> bool {
        self.main.intersects(rect) &&
            self.complex.iter().all(|complex| complex.rect.intersects(rect))
    }

    /// Returns true if this clipping region completely surrounds the given rect.
    #[inline]
    pub fn does_not_clip_rect(&self, rect: &Rect<f32>) -> bool {
        self.main.contains(&rect.origin) && self.main.contains(&rect.bottom_right()) &&
            self.complex.iter().all(|complex| {
                complex.rect.contains(&rect.origin) && complex.rect.contains(&rect.bottom_right())
            })
    }

    /// Returns a bounding rect that surrounds this entire clipping region.
    #[inline]
    pub fn bounding_rect(&self) -> Rect<f32> {
        let mut rect = self.main;
        for complex in &*self.complex {
            rect = rect.union(&complex.rect)
        }
        rect
    }

    /// Intersects this clipping region with the given rounded rectangle.
    #[inline]
    pub fn intersect_with_rounded_rect(&mut self, rect: &Rect<f32>, radii: &BorderRadius) {
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
    pub fn translate(&self, delta: &Point2D<f32>) -> ClippingRegion {
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

    #[inline]
    pub fn is_max(&self) -> bool {
        self.main == geometry::au_rect_to_f32_rect(geometry::max_rect()) && self.complex.is_empty()
    }
}

trait ClipRegionMethods {
    /// Returns true if this clipping region might contain the given point and false otherwise.
    /// This is a quick, not a precise, test; it can yield false positives.
    fn might_intersect_point(&self, point: &Point2D<f32>, auxiliary_lists: &AuxiliaryListsBuilder)
                             -> bool;

}

impl ClipRegionMethods for webrender_traits::ClipRegion {
    #[inline]
    fn might_intersect_point(&self, point: &Point2D<f32>, auxiliary_lists: &AuxiliaryListsBuilder)
                             -> bool {
        self.main.contains(point) &&
            auxiliary_lists.complex_clip_regions(&self.complex)
                           .iter()
                           .all(|complex| complex.rect.contains(point))
    }
}

impl ComplexClippingRegion {
    // TODO(pcwalton): This could be more aggressive by considering points that touch the inside of
    // the border radius ellipse.
    fn completely_encloses(&self, other: &ComplexClippingRegion) -> bool {
        let left = f32::max(self.radii.top_left.width, self.radii.bottom_left.width);
        let top = f32::max(self.radii.top_left.height, self.radii.top_right.height);
        let right = f32::max(self.radii.top_right.width, self.radii.bottom_right.width);
        let bottom = f32::max(self.radii.bottom_left.height, self.radii.bottom_right.height);
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
#[derive(Clone, Copy)]
pub struct DisplayItemMetadata {
    /// The DOM node from which this display item originated.
    pub node: OpaqueNode,
    /// The value of the `cursor` property when the mouse hovers over this display item. If `None`,
    /// this display item is ineligible for pointer events (`pointer-events: none`).
    pub pointing: Option<Cursor>,
}

impl DisplayItemMetadata {
    #[inline]
    pub fn empty() -> DisplayItemMetadata {
        DisplayItemMetadata {
            node: OpaqueNode(0),
            pointing: None,
        }
    }
}

/// Paints a solid color.
#[derive(Clone)]
pub struct SolidColorDisplayItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    /// The color.
    pub color: ColorF,
}

/// Paints text.
#[derive(Clone)]
pub struct TextDisplayItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    /// The text run.
    pub glyphs: Vec<GlyphInstance>,

    /// The font key.
    pub font_key: FontKey,

    /// The font size.
    pub size: Au,

    /// The color of the text.
    pub text_color: ColorF,

    /// The position of the start of the baseline of this text.
    pub baseline_origin: Point2D<f32>,

    /// The orientation of the text: upright or sideways left/right.
    pub orientation: TextOrientation,

    /// The blur radius for this text. If zero, this text is not blurred.
    pub blur_radius: Au,
}

#[derive(Clone, Eq, PartialEq)]
pub enum TextOrientation {
    Upright,
    SidewaysLeft,
    SidewaysRight,
}

/// Paints an image.
#[derive(Clone)]
pub struct ImageDisplayItem {
    pub base: BaseDisplayItem,

    pub webrender_image: WebRenderImageInfo,

    /// The dimensions to which the image display item should be stretched. If this is smaller than
    /// the bounds of this display item, then the image will be repeated in the appropriate
    /// direction to tile the entire bounds.
    pub stretch_size: Size2D<f32>,

    /// The amount of space to add to the right and bottom part of each tile, when the image
    /// is tiled.
    pub tile_spacing: Size2D<f32>,

    /// The algorithm we should use to stretch the image. See `image_rendering` in CSS-IMAGES-3 §
    /// 5.3.
    pub image_rendering: ImageRendering,
}

#[derive(Clone)]
pub struct WebGLDisplayItem {
    pub base: BaseDisplayItem,
    pub context_id: WebGLContextId,
}


/// Paints an iframe.
#[derive(Clone)]
pub struct IframeDisplayItem {
    pub base: BaseDisplayItem,
    pub iframe: PipelineId,
}

/// Paints a gradient.
#[derive(Clone)]
pub struct GradientDisplayItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    /// The start point of the gradient (computed during display list construction).
    pub start_point: Point2D<f32>,

    /// The end point of the gradient (computed during display list construction).
    pub end_point: Point2D<f32>,

    /// A list of color stops.
    pub stops: Vec<GradientStop>,
}

/// Paints a border.
#[derive(Clone)]
pub struct BorderDisplayItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    /// Style for the left side.
    pub left: BorderSide,

    /// Style for the top side.
    pub top: BorderSide,

    /// Style for the right side.
    pub right: BorderSide,

    /// Style for the bottom side.
    pub bottom: BorderSide,

    /// Border radii.
    ///
    /// TODO(pcwalton): Elliptical radii.
    pub radius: BorderRadius,
}

pub trait BorderRadiusScaling {
    /// Scale the value by the specified factor.
    fn scale_by(&self, s: f32) -> Self;
}

impl BorderRadiusScaling for BorderRadius {
    fn scale_by(&self, s: f32) -> BorderRadius {
        BorderRadius {
            top_left: self.top_left.scale_by(s),
            top_right: self.top_right.scale_by(s),
            bottom_left: self.bottom_left.scale_by(s),
            bottom_right: self.bottom_right.scale_by(s),
        }
    }
}

impl BorderRadiusScaling for Size2D<f32> {
    fn scale_by(&self, s: f32) -> Size2D<f32> {
        Size2D::new(self.width * s, self.height * s)
    }
}

pub trait BorderRadiusUtils {
    /// Returns true if all the radii are zero.
    fn is_square(&self) -> bool;
    /// Returns a set of border radii that all have the given value.
    fn all_same(value: f32) -> Self;
    /// Returns a set of border radii, all zero.
    fn square() -> Self;
}

impl BorderRadiusUtils for BorderRadius {
    fn is_square(&self) -> bool {
        self.top_left.width == 0.0 && self.top_left.height == 0.0 &&
            self.top_right.width == 0.0 && self.top_right.height == 0.0 &&
            self.bottom_left.width == 0.0 && self.bottom_left.height == 0.0 &&
            self.bottom_right.width == 0.0 && self.bottom_right.height == 0.0
    }

    /// Returns a set of border radii that all have the given value.
    fn all_same(value: f32) -> BorderRadius {
        BorderRadius {
            top_left: Size2D::new(value, value),
            top_right: Size2D::new(value, value),
            bottom_right: Size2D::new(value, value),
            bottom_left: Size2D::new(value, value),
        }
    }

    /// Returns a set of border radii, all zero.
    fn square() -> BorderRadius {
        BorderRadius::all_same(0.0)
    }
}

/// Paints a box shadow per CSS-BACKGROUNDS.
#[derive(Clone)]
pub struct BoxShadowDisplayItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    /// The dimensions of the box that we're placing a shadow around.
    pub box_bounds: Rect<f32>,

    /// The offset of this shadow from the box.
    pub offset: Point2D<f32>,

    /// The color of this shadow.
    pub color: ColorF,

    /// The blur radius for this shadow.
    pub blur_radius: f32,

    /// The spread radius of this shadow.
    pub spread_radius: f32,

    /// The border radius of this shadow.
    ///
    /// TODO(pcwalton): Elliptical radii; different radii for each corner.
    pub border_radius: f32,

    /// How we should clip the result.
    pub clip_mode: BoxShadowClipMode,
}

/// Defines a stacking context.
#[derive(Clone)]
pub struct PushStackingContextItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    pub stacking_context: StackingContext,
}

/// Defines a stacking context.
#[derive(Clone)]
pub struct PopStackingContextItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    pub stacking_context_id: StackingContextId,
}

/// Starts a group of items inside a particular scroll root.
#[derive(Clone)]
pub struct PushScrollRootItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    /// The scroll root that this item starts.
    pub scroll_root: ScrollRoot,
}

impl DisplayItem {
    pub fn base(&self) -> &BaseDisplayItem {
        match *self {
            DisplayItem::SolidColor(ref solid_color) => &solid_color.base,
            DisplayItem::Text(ref text) => &text.base,
            DisplayItem::Image(ref image_item) => &image_item.base,
            DisplayItem::WebGL(ref webgl_item) => &webgl_item.base,
            DisplayItem::Border(ref border) => &border.base,
            DisplayItem::Gradient(ref gradient) => &gradient.base,
            DisplayItem::BoxShadow(ref box_shadow) => &box_shadow.base,
            DisplayItem::Iframe(ref iframe) => &iframe.base,
            DisplayItem::PushStackingContext(ref stacking_context) => &stacking_context.base,
            DisplayItem::PopStackingContext(ref item) => &item.base,
            DisplayItem::PushScrollRoot(ref item) => &item.base,
            DisplayItem::PopScrollRoot(ref base) => &base,
        }
    }

    pub fn scroll_root_id(&self) -> ServoScrollRootId {
        self.base().scroll_root_id
    }

    pub fn stacking_context_id(&self) -> StackingContextId {
        self.base().stacking_context_id
    }

    pub fn section(&self) -> DisplayListSection {
        self.base().section
    }

    pub fn bounds(&self) -> Rect<f32> {
        self.base().bounds
    }
}

trait DisplayItemHitTesting {
    fn hit_test(&self,
                id: DisplayItemId,
                point: &Point2D<Au>,
                auxiliary_lists: &AuxiliaryListsBuilder,
                metadatas: &[DisplayItemMetadata])
                -> Option<DisplayItemMetadata>;
}

impl DisplayItemHitTesting for webrender_traits::DisplayItem {
    fn hit_test(&self,
                id: DisplayItemId,
                point: &Point2D<Au>,
                auxiliary_lists: &AuxiliaryListsBuilder,
                metadatas: &[DisplayItemMetadata])
                -> Option<DisplayItemMetadata> {
        // TODO(pcwalton): Use a precise algorithm here. This will allow us to properly hit
        // test elements with `border-radius`, for example.
        let point = geometry::au_point_to_f32_point(&point);
        if !self.clip.might_intersect_point(&point, auxiliary_lists) {
            // Clipped out.
            return None;
        }
        if !self.rect.contains(&point) {
            // Can't possibly hit.
            return None;
        }
        let metadata = &metadatas[id.0 as usize];
        if metadata.pointing.is_none() {
            // `pointer-events` is `none`. Ignore this item.
            return None;
        }

        match self.item {
            webrender_traits::SpecificDisplayItem::Border(ref border) => {
                // If the point is inside the border, it didn't hit the border!
                let interior_rect = Rect::new(
                    Point2D::new(self.rect.origin.x + border.left.width,
                                 self.rect.origin.y + border.top.width),
                    Size2D::new(self.rect.size.width - border.left.width + border.right.width,
                                self.rect.size.height - border.top.width + border.bottom.width));
                if interior_rect.contains(&point) {
                    return None;
                }
            }
            webrender_traits::SpecificDisplayItem::BoxShadow(_) => {
                // Box shadows can never be hit.
                return None;
            }
            _ => {}
        }

        Some(*metadata)
    }
}

#[derive(Copy, Clone)]
pub struct WebRenderImageInfo {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
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

/// The type of the scroll offset list.
pub type ScrollOffsetMap = HashMap<ServoScrollRootId, Point2D<f32>>;

pub trait SimpleMatrixDetection {
    fn is_identity_or_simple_translation(&self) -> bool;
}

impl SimpleMatrixDetection for Matrix4D<f32> {
    #[inline]
    fn is_identity_or_simple_translation(&self) -> bool {
        let (_0, _1) = (Zero::zero(), One::one());
        self.m11 == _1 && self.m12 == _0 && self.m13 == _0 && self.m14 == _0 &&
        self.m21 == _0 && self.m22 == _1 && self.m23 == _0 && self.m24 == _0 &&
        self.m31 == _0 && self.m32 == _0 && self.m33 == _1 && self.m34 == _0 &&
        self.m44 == _1
    }
}

pub trait DisplayListPrinting {
    fn print(&self, print_tree: &mut PrintTree);
}

impl DisplayListPrinting for DisplayListBuilder {
    fn print(&self, print_tree: &mut PrintTree) {
        let traversal = DisplayListTraversal::new(self);
        for item in traversal {
            let mut buffers = vec![
                print_tree.buffer(""),
                print_tree.buffer(""),
                print_tree.buffer(""),
                print_tree.buffer(""),
            ];

            let bounds = match item.item {
                webrender_traits::SpecificDisplayItem::PushStackingContext(ref item) => {
                    item.stacking_context.bounds
                }
                _ => item.rect,
            };

            drop(buffers[1].set_color(ColorSpec::new().set_fg(Some(termcolor::Color::White))));
            drop(buffers[1].write_all(&[b'(']));
            drop(buffers[1].reset());
            drop(write!(buffers[1], "{}px", round_px(bounds.origin.x)));
            drop(buffers[1].set_color(ColorSpec::new().set_fg(Some(termcolor::Color::White))));
            drop(buffers[1].write_all(b", "));
            drop(buffers[1].reset());
            drop(write!(buffers[1], "{}px", round_px(bounds.origin.y)));
            drop(buffers[1].set_color(ColorSpec::new().set_fg(Some(termcolor::Color::White))));
            drop(buffers[1].write_all(&[b')']));
            drop(buffers[1].reset());

            drop(buffers[2].set_color(ColorSpec::new().set_fg(Some(termcolor::Color::White))));
            drop(buffers[2].write_all(&[b'(']));
            drop(buffers[2].reset());
            drop(write!(buffers[2], "{}px", round_px(bounds.size.width)));
            drop(buffers[2].set_color(ColorSpec::new().set_fg(Some(termcolor::Color::White))));
            drop(buffers[2].write_all("×".as_bytes()));
            drop(buffers[2].reset());
            drop(write!(buffers[2], "{}px", round_px(bounds.size.height)));
            drop(buffers[2].set_color(ColorSpec::new().set_fg(Some(termcolor::Color::White))));
            drop(buffers[2].write_all(&[b')']));
            drop(buffers[2].reset());

            match item.item {
                webrender_traits::SpecificDisplayItem::PushStackingContext(_) => {
                    drop(write!(buffers[0], "Stacking Context"));
                    // TODO(pcwalton): transform, perspective, mix-blend-mode, filters
                    print_tree.new_level(&buffers);
                }
                webrender_traits::SpecificDisplayItem::PopStackingContext => {
                    print_tree.end_level();
                }
                webrender_traits::SpecificDisplayItem::PushScrollLayer(_) => {
                    drop(write!(buffers[0], "Scroll Layer"));
                    print_tree.new_level(&buffers);
                }
                webrender_traits::SpecificDisplayItem::PopScrollLayer => {
                    print_tree.end_level();
                }
                webrender_traits::SpecificDisplayItem::Text(ref text) => {
                    drop(write!(buffers[0], "Text"));
                    write_swatch(&mut buffers[3], &text.color);
                    drop(buffers[3].set_color(
                            ColorSpec::new().set_fg(Some(termcolor::Color::White))));
                    drop(buffers[3].write_all(b" \""));
                    drop(buffers[3].reset());
                    for ch in self.auxiliary_lists_builder.glyph_instances(&text.glyphs)
                                                          .iter()
                                                          .map(|glyph| glyph.index + 0x1d)
                                                          .filter(|&ch| ch >= 0x20 && ch < 0x7f)
                                                          .take(40) {
                        drop(buffers[3].write_all(&[ch as u8]))
                    }
                    drop(buffers[3].set_color(
                            ColorSpec::new().set_fg(Some(termcolor::Color::White))));
                    drop(buffers[3].write_all(&[b'"']));
                    drop(buffers[3].reset());
                    print_tree.add_item(buffers);
                }
                webrender_traits::SpecificDisplayItem::Border(ref border) => {
                    drop(write!(buffers[0], "Border"));
                    let mut sides_processed = Side::empty();
                    let mut first_side = true;
                    loop {
                        let mut these_sides =
                                match SIDES.into_iter()
                                           .filter(|side| !sides_processed.contains(**side))
                                           .next() {
                            Some(side) => *side,
                            None => break,
                        };

                        if !first_side {
                            drop(buffers[3].write_all(b"; "));
                        }

                        sides_processed.insert(these_sides);

                        let border_side = these_sides.get(border);
                        for other_side in SIDES.into_iter() {
                            if !sides_processed.contains(*other_side) &&
                                    border_side == other_side.get(border) {
                                these_sides.insert(*other_side);
                                sides_processed.insert(*other_side);
                            }
                        }

                        if border_side.width == 0.0 || border_side.style == BorderStyle::None {
                            continue
                        }

                        let mut first = true;
                        if !these_sides.is_all() {
                            for &side in SIDES.into_iter() {
                                if !these_sides.contains(side) {
                                    continue
                                }
                                if first {
                                    first = false
                                } else {
                                    drop(buffers[3].write_all(b"-"));
                                }
                                if side.contains(LEFT) {
                                    drop(buffers[3].write_all(b"left"))
                                } else if side.contains(TOP) {
                                    drop(buffers[3].write_all(b"top"))
                                } else if side.contains(RIGHT) {
                                    drop(buffers[3].write_all(b"right"))
                                } else if side.contains(BOTTOM) {
                                    drop(buffers[3].write_all(b"bottom"))
                                }
                            }
                            drop(buffers[3].write_all(b": "))
                        }

                        let style = match border_side.style {
                            BorderStyle::None => continue,
                            BorderStyle::Solid => &b"solid "[..],
                            BorderStyle::Dashed => &b"dashed "[..],
                            BorderStyle::Dotted => &b"dotted "[..],
                            BorderStyle::Groove => &b"groove "[..],
                            BorderStyle::Inset => &b"inset "[..],
                            BorderStyle::Outset => &b"outset "[..],
                            BorderStyle::Double => &b"double "[..],
                            BorderStyle::Hidden => &b"hidden "[..],
                            BorderStyle::Ridge => &b"ridge "[..],
                        };
                        drop(buffers[3].write_all(style));
                        write_color(&mut buffers[3], &border_side.color);
                        drop(write!(buffers[3],
                                    " {}px",
                                    (border_side.width * 100.0).round() / 100.0));
                        first_side = false;
                    }
                    drop(buffers[3].reset());
                    print_tree.add_item(buffers);
                }
                webrender_traits::SpecificDisplayItem::Image(_) => {
                    drop(write!(buffers[0], "Image"));
                    print_tree.add_item(buffers);
                }
                webrender_traits::SpecificDisplayItem::YuvImage(_) => {
                    drop(write!(buffers[0], "YuvImage"));
                    print_tree.add_item(buffers);
                }
                webrender_traits::SpecificDisplayItem::Iframe(_) => {
                    drop(write!(buffers[0], "Iframe"));
                    print_tree.add_item(buffers);
                }
                webrender_traits::SpecificDisplayItem::Gradient(ref gradient) => {
                    drop(write!(buffers[0], "Gradient"));
                    let mut first = true;
                    for stop in self.auxiliary_lists_builder.gradient_stops(&gradient.stops) {
                        if !first {
                            drop(write!(buffers[3], ", "))
                        } else {
                            first = false
                        }

                        write_color(&mut buffers[3], &stop.color);
                        drop(write!(buffers[3], " {}px", (stop.offset * 100.0).round() / 100.0))
                    }
                    print_tree.add_item(buffers);
                }
                webrender_traits::SpecificDisplayItem::Rectangle(ref item) => {
                    drop(write!(buffers[0], "Rectangle"));
                    write_color(&mut buffers[3], &item.color);
                    print_tree.add_item(buffers);
                }
                webrender_traits::SpecificDisplayItem::WebGL(_) => {
                    drop(write!(buffers[0], "WebGL"));
                    print_tree.add_item(buffers);
                }
                webrender_traits::SpecificDisplayItem::BoxShadow(ref shadow) => {
                    drop(write!(buffers[0], "BoxShadow"));
                    drop(write!(buffers[3],
                                "{}px {}px {}px {}px ",
                                round_px(shadow.offset.x),
                                round_px(shadow.offset.y),
                                round_px(shadow.blur_radius),
                                round_px(shadow.spread_radius)));
                    write_color(&mut buffers[3], &shadow.color);
                    print_tree.add_item(buffers);
                }
            }
        }

        // Sets the foreground color of the buffer to the closest terminal color to the one
        // supplied.
        fn write_swatch(buffer: &mut Buffer, color: &ColorF) {
            drop(buffer.set_color(ColorSpec::new().set_fg(Some(COLORS.iter()
                                                                     .fold(COLORS[0].clone(),
                                                                           |(a_color, a),
                                                                            &(ref b_color, b)| {
                let (ar, ag, ab) = (a.r - color.r, a.g - color.g, a.b - color.b);
                let (br, bg, bb) = (b.r - color.r, b.g - color.g, b.b - color.b);
                if ar * ar + ag * ag + ab * ab <= br * br + bg * bg + bb * bb {
                    (a_color, a)
                } else {
                    ((*b_color).clone(), b)
                }
            }).0))));

            drop(buffer.write_all("●".as_bytes()));
            drop(buffer.reset());

            static COLORS: [(termcolor::Color, ColorF); 8] = [
                (termcolor::Color::White,   ColorF { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }),
                (termcolor::Color::Blue,    ColorF { r: 0.0, g: 0.0, b: 1.0, a: 1.0 }),
                (termcolor::Color::Green,   ColorF { r: 0.0, g: 1.0, b: 0.0, a: 1.0 }),
                (termcolor::Color::Red,     ColorF { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }),
                (termcolor::Color::Cyan,    ColorF { r: 0.0, g: 1.0, b: 1.0, a: 1.0 }),
                (termcolor::Color::Magenta, ColorF { r: 1.0, g: 0.0, b: 1.0, a: 1.0 }),
                (termcolor::Color::Yellow,  ColorF { r: 1.0, g: 1.0, b: 0.0, a: 1.0 }),
                (termcolor::Color::White,   ColorF { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }),
            ];
        }

        fn write_color(buffer: &mut Buffer, color: &ColorF) {
            write_swatch(buffer, color);
            drop(write!(buffer,
                        " #{:02x}{:02x}{:02x}",
                        f32::round(color.r * 255.0) as u32,
                        f32::round(color.g * 255.0) as u32,
                        f32::round(color.b * 255.0) as u32));
            drop(buffer.reset())
        }

        fn round_px(value: f32) -> f32 {
            (value * 100.0).round() / 100.0
        }
    }
}

bitflags! {
    flags Side: u8 {
        const LEFT = 0x01,
        const TOP = 0x02,
        const RIGHT = 0x04,
        const BOTTOM = 0x08,
    }
}

impl Side {
    fn get(self, border: &webrender_traits::BorderDisplayItem) -> BorderSide {
        if self.contains(LEFT) {
            border.left
        } else if self.contains(TOP) {
            border.top
        } else if self.contains(RIGHT) {
            border.right
        } else {
            border.bottom
        }
    }
}

static SIDES: [Side; 4] = [LEFT, TOP, RIGHT, BOTTOM];

