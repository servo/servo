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
use euclid::{Matrix4D, Point2D, Rect, Size2D};
use euclid::num::{One, Zero};
use euclid::rect::TypedRect;
use euclid::side_offsets::SideOffsets2D;
use gfx_traits::{ScrollRootId, StackingContextId};
use gfx_traits::print_tree::PrintTree;
use ipc_channel::ipc::IpcSharedMemory;
use msg::constellation_msg::PipelineId;
use net_traits::image::base::{Image, PixelFormat};
use range::Range;
use servo_geometry::max_rect;
use std::cmp::{self, Ordering};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use style::computed_values::{border_style, filter, image_rendering, mix_blend_mode};
use style_traits::cursor::Cursor;
use text::TextRun;
use text::glyph::ByteIndex;
use webrender_traits::{self, ColorF, GradientStop, ScrollPolicy, WebGLContextId};

pub use style::dom::OpaqueNode;

/// The factor that we multiply the blur radius by in order to inflate the boundaries of display
/// items that involve a blur. This ensures that the display item boundaries include all the ink.
pub static BLUR_INFLATION_FACTOR: i32 = 3;

#[derive(HeapSizeOf, Deserialize, Serialize)]
pub struct DisplayList {
    pub list: Vec<DisplayItem>,
}

impl DisplayList {
    // Returns the text index within a node for the point of interest.
    pub fn text_index(&self,
                      node: OpaqueNode,
                      client_point: &Point2D<Au>,
                      scroll_offsets: &ScrollOffsetMap)
                      -> Option<usize> {
        let mut result = Vec::new();
        let mut traversal = DisplayListTraversal::new(self);
        self.text_index_contents(node,
                                 &mut traversal,
                                 client_point,
                                 client_point,
                                 scroll_offsets,
                                 &mut result);
        result.pop()
    }

    pub fn text_index_contents<'a>(&self,
                                   node: OpaqueNode,
                                   traversal: &mut DisplayListTraversal<'a>,
                                   translated_point: &Point2D<Au>,
                                   client_point: &Point2D<Au>,
                                   scroll_offsets: &ScrollOffsetMap,
                                   result: &mut Vec<usize>) {
        while let Some(item) = traversal.next() {
            match item {
                &DisplayItem::PushStackingContext(ref stacking_context_item) => {
                    let mut point = *translated_point;
                    DisplayList::translate_point(&stacking_context_item.stacking_context,
                                                 &mut point,
                                                 client_point);
                    self.text_index_contents(node,
                                             traversal,
                                             &point,
                                             client_point,
                                             scroll_offsets,
                                             result);
                }
                &DisplayItem::PushScrollRoot(ref item) => {
                    let mut point = *translated_point;
                    DisplayList::scroll_root(&item.scroll_root,
                                             &mut point,
                                             scroll_offsets);
                    self.text_index_contents(node,
                                             traversal,
                                             &point,
                                             client_point,
                                             scroll_offsets,
                                             result);

                },
                &DisplayItem::PopStackingContext(_) => return,
                &DisplayItem::Text(ref text) => {
                    let base = item.base();
                    if base.metadata.node == node {
                        let offset = *translated_point - text.baseline_origin;
                        let index = text.text_run.range_index_of_advance(&text.range, offset.x);
                        result.push(index);
                    }
                },
                _ => {},
            }
        }
    }

    // Return all nodes containing the point of interest, bottommost first, and
    // respecting the `pointer-events` CSS property.
    pub fn hit_test(&self,
                    translated_point: &Point2D<Au>,
                    client_point: &Point2D<Au>,
                    scroll_offsets: &ScrollOffsetMap)
                    -> Vec<DisplayItemMetadata> {
        let mut result = Vec::new();
        let mut traversal = DisplayListTraversal::new(self);
        self.hit_test_contents(&mut traversal,
                               translated_point,
                               client_point,
                               scroll_offsets,
                               &mut result);
        result
    }

    pub fn hit_test_contents<'a>(&self,
                                 traversal: &mut DisplayListTraversal<'a>,
                                 translated_point: &Point2D<Au>,
                                 client_point: &Point2D<Au>,
                                 scroll_offsets: &ScrollOffsetMap,
                                 result: &mut Vec<DisplayItemMetadata>) {
        while let Some(item) = traversal.next() {
            match item {
                &DisplayItem::PushStackingContext(ref stacking_context_item) => {
                    let mut point = *translated_point;
                    DisplayList::translate_point(&stacking_context_item.stacking_context,
                                                 &mut point,
                                                 client_point);
                    self.hit_test_contents(traversal,
                                           &point,
                                           client_point,
                                           scroll_offsets,
                                           result);
                }
                &DisplayItem::PushScrollRoot(ref item) => {
                    let mut point = *translated_point;
                    DisplayList::scroll_root(&item.scroll_root,
                                             &mut point,
                                             scroll_offsets);
                    self.hit_test_contents(traversal,
                                           &point,
                                           client_point,
                                           scroll_offsets,
                                           result);
                }
                &DisplayItem::PopStackingContext(_) | &DisplayItem::PopScrollRoot(_) => return,
                _ => {
                    if let Some(meta) = item.hit_test(*translated_point) {
                        result.push(meta);
                    }
                }
            }
        }
    }

    #[inline]
    fn translate_point<'a>(stacking_context: &StackingContext,
                           translated_point: &mut Point2D<Au>,
                           client_point: &Point2D<Au>) {
        // Convert the parent translated point into stacking context local transform space if the
        // stacking context isn't fixed.  If it's fixed, we need to use the client point anyway.
        debug_assert!(stacking_context.context_type == StackingContextType::Real);
        let is_fixed = stacking_context.scroll_policy == ScrollPolicy::Fixed;
        *translated_point = if is_fixed {
            *client_point
        } else {
            let point = *translated_point - stacking_context.bounds.origin;

            match stacking_context.transform {
                Some(transform) => {
                    let inv_transform = match transform.inverse() {
                        Some(transform) => transform,
                        None => {
                            // If a transform function causes the current transformation matrix of an object
                            // to be non-invertible, the object and its content do not get displayed.
                            return;
                        }
                    };
                    let frac_point = inv_transform.transform_point(&Point2D::new(point.x.to_f32_px(),
                                                                                 point.y.to_f32_px()));
                    Point2D::new(Au::from_f32_px(frac_point.x), Au::from_f32_px(frac_point.y))
                }
                None => {
                    point
                }
            }
        };
    }

    #[inline]
    fn scroll_root<'a>(scroll_root: &ScrollRoot,
                       translated_point: &mut Point2D<Au>,
                       scroll_offsets: &ScrollOffsetMap) {
        // Adjust the translated point to account for the scroll offset if necessary.
        //
        // We don't perform this adjustment on the root stacking context because
        // the DOM-side code has already translated the point for us (e.g. in
        // `Window::hit_test_query()`) by now.
        if let Some(scroll_offset) = scroll_offsets.get(&scroll_root.id) {
            translated_point.x -= Au::from_f32_px(scroll_offset.x);
            translated_point.y -= Au::from_f32_px(scroll_offset.y);
        }
    }

    pub fn print(&self) {
        let mut print_tree = PrintTree::new("Display List".to_owned());
        self.print_with_tree(&mut print_tree);
    }

    pub fn print_with_tree(&self, print_tree: &mut PrintTree) {
        print_tree.new_level("Items".to_owned());
        for item in &self.list {
            print_tree.add_item(format!("{:?} StackingContext: {:?} ScrollRoot: {:?}",
                                        item,
                                        item.base().stacking_context_id,
                                        item.scroll_root_id()));
        }
        print_tree.end_level();
    }
}

pub struct DisplayListTraversal<'a> {
    pub display_list: &'a DisplayList,
    pub next_item_index: usize,
    pub first_item_index: usize,
    pub last_item_index: usize,
}

impl<'a> DisplayListTraversal<'a> {
    pub fn new(display_list: &'a DisplayList) -> DisplayListTraversal {
        DisplayListTraversal {
            display_list: display_list,
            next_item_index: 0,
            first_item_index: 0,
            last_item_index: display_list.list.len(),
        }
    }

    pub fn new_partial(display_list: &'a DisplayList,
                       stacking_context_id: StackingContextId,
                       start: usize,
                       end: usize)
                       -> DisplayListTraversal {
        debug_assert!(start <= end);
        debug_assert!(display_list.list.len() > start);
        debug_assert!(display_list.list.len() > end);

        let stacking_context_start = display_list.list[0..start].iter().rposition(|item|
            match item {
                &DisplayItem::PushStackingContext(ref item) =>
                    item.stacking_context.id == stacking_context_id,
                _ => false,
            }).unwrap_or(start);
        debug_assert!(stacking_context_start <= start);

        DisplayListTraversal {
            display_list: display_list,
            next_item_index: stacking_context_start,
            first_item_index: start,
            last_item_index: end + 1,
        }
    }

    pub fn previous_item_id(&self) -> usize {
        self.next_item_index - 1
    }

    pub fn skip_to_end_of_stacking_context(&mut self, id: StackingContextId) {
        self.next_item_index = self.display_list.list[self.next_item_index..].iter()
                                                                             .position(|item| {
            match item {
                &DisplayItem::PopStackingContext(ref item) => item.stacking_context_id == id,
                _ => false
            }
        }).unwrap_or(self.display_list.list.len());
        debug_assert!(self.next_item_index < self.last_item_index);
    }
}

impl<'a> Iterator for DisplayListTraversal<'a> {
    type Item = &'a DisplayItem;

    fn next(&mut self) -> Option<&'a DisplayItem> {
        while self.next_item_index < self.last_item_index {
            debug_assert!(self.next_item_index <= self.last_item_index);

            let reached_first_item = self.next_item_index >= self.first_item_index;
            let item = &self.display_list.list[self.next_item_index];

            self.next_item_index += 1;

            if reached_first_item {
                return Some(item)
            }

            // Before we reach the starting item, we only emit stacking context boundaries. This
            // is to ensure that we properly position items when we are processing a display list
            // slice that is relative to a certain stacking context.
            match item {
                &DisplayItem::PushStackingContext(_) |
                &DisplayItem::PopStackingContext(_) => return Some(item),
                _ => {}
            }
        }

        None
    }
}

/// Display list sections that make up a stacking context. Each section  here refers
/// to the steps in CSS 2.1 Appendix E.
///
#[derive(Clone, Copy, Debug, Deserialize, Eq, HeapSizeOf, Ord, PartialEq, PartialOrd, Serialize)]
pub enum DisplayListSection {
    BackgroundAndBorders,
    BlockBackgroundsAndBorders,
    Content,
    Outlines,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, HeapSizeOf, Ord, PartialEq, PartialOrd, Serialize)]
pub enum StackingContextType {
    Real,
    PseudoPositioned,
    PseudoFloat,
    PseudoScrollingArea,
}

#[derive(Clone, HeapSizeOf, Deserialize, Serialize)]
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
    pub transform: Option<Matrix4D<f32>>,

    /// The perspective matrix to be applied to children.
    pub perspective: Option<Matrix4D<f32>>,

    /// The scroll policy of this layer.
    pub scroll_policy: ScrollPolicy,

    /// The id of the parent scrolling area that contains this StackingContext.
    pub parent_scroll_id: ScrollRootId,
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
               transform: Option<Matrix4D<f32>>,
               perspective: Option<Matrix4D<f32>>,
               scroll_policy: ScrollPolicy,
               parent_scroll_id: ScrollRootId)
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
            scroll_policy: scroll_policy,
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
                             filter::T::new(Vec::new()),
                             mix_blend_mode::T::normal,
                             None,
                             None,
                             ScrollPolicy::Scrollable,
                             ScrollRootId::root())
    }

    pub fn to_display_list_items(self) -> (DisplayItem, DisplayItem) {
        let mut base_item = BaseDisplayItem::empty();
        base_item.stacking_context_id = self.id;
        base_item.scroll_root_id = self.parent_scroll_id;

        let pop_item = DisplayItem::PopStackingContext(Box::new(
            PopStackingContextItem {
                base: base_item.clone(),
                stacking_context_id: self.id,
            }
        ));

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

impl fmt::Debug for StackingContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let type_string =  if self.context_type == StackingContextType::Real {
            "StackingContext"
        } else {
            "Pseudo-StackingContext"
        };

        write!(f, "{} at {:?} with overflow {:?}: {:?}",
               type_string,
               self.bounds,
               self.overflow,
               self.id)
    }
}

/// Defines a stacking context.
#[derive(Clone, Debug, HeapSizeOf, Deserialize, Serialize)]
pub struct ScrollRoot {
    /// The unique ID of this ScrollRoot.
    pub id: ScrollRootId,

    /// The unique ID of the parent of this ScrollRoot.
    pub parent_id: ScrollRootId,

    /// The position of this scroll root's frame in the parent stacking context.
    pub clip: ClippingRegion,

    /// The rect of the contents that can be scrolled inside of the scroll root.
    pub content_rect: Rect<Au>,
}

impl ScrollRoot {
    pub fn to_push(&self) -> DisplayItem {
        DisplayItem::PushScrollRoot(box PushScrollRootItem {
            base: BaseDisplayItem::empty(),
            scroll_root: self.clone(),
        })
    }
}


/// One drawing command in the list.
#[derive(Clone, Deserialize, HeapSizeOf, Serialize)]
pub enum DisplayItem {
    SolidColor(Box<SolidColorDisplayItem>),
    Text(Box<TextDisplayItem>),
    Image(Box<ImageDisplayItem>),
    WebGL(Box<WebGLDisplayItem>),
    Border(Box<BorderDisplayItem>),
    Gradient(Box<GradientDisplayItem>),
    Line(Box<LineDisplayItem>),
    BoxShadow(Box<BoxShadowDisplayItem>),
    Iframe(Box<IframeDisplayItem>),
    PushStackingContext(Box<PushStackingContextItem>),
    PopStackingContext(Box<PopStackingContextItem>),
    PushScrollRoot(Box<PushScrollRootItem>),
    PopScrollRoot(Box<BaseDisplayItem>),
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

    /// The id of the scroll root this item belongs to.
    pub scroll_root_id: ScrollRootId,
}

impl BaseDisplayItem {
    #[inline(always)]
    pub fn new(bounds: &Rect<Au>,
               metadata: DisplayItemMetadata,
               clip: &ClippingRegion,
               section: DisplayListSection,
               stacking_context_id: StackingContextId,
               scroll_root_id: ScrollRootId)
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
            scroll_root_id: scroll_root_id,
        }
    }

    #[inline(always)]
    pub fn empty() -> BaseDisplayItem {
        BaseDisplayItem {
            bounds: TypedRect::zero(),
            metadata: DisplayItemMetadata {
                node: OpaqueNode(0),
                pointing: None,
            },
            clip: ClippingRegion::max(),
            section: DisplayListSection::Content,
            stacking_context_id: StackingContextId::root(),
            scroll_root_id: ScrollRootId::root(),
        }
    }
}

/// A clipping region for a display item. Currently, this can describe rectangles, rounded
/// rectangles (for `border-radius`), or arbitrary intersections of the two. Arbitrary transforms
/// are not supported because those are handled by the higher-level `StackingContext` abstraction.
#[derive(Clone, PartialEq, HeapSizeOf, Deserialize, Serialize)]
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
            main: max_rect(),
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

    #[inline]
    pub fn is_max(&self) -> bool {
        self.main == max_rect() && self.complex.is_empty()
    }
}

impl fmt::Debug for ClippingRegion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if *self == ClippingRegion::max() {
            write!(f, "ClippingRegion::Max")
        } else if *self == ClippingRegion::empty() {
            write!(f, "ClippingRegion::Empty")
        } else if self.main == max_rect() {
            write!(f, "ClippingRegion(Complex={:?})", self.complex)
        } else {
            write!(f, "ClippingRegion(Rect={:?}, Complex={:?})", self.main, self.complex)
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
    pub color: ColorF,
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
    pub text_color: ColorF,

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

    /// The amount of space to add to the right and bottom part of each tile, when the image
    /// is tiled.
    pub tile_spacing: Size2D<Au>,

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
pub struct Gradient {
    /// The start point of the gradient (computed during display list construction).
    pub start_point: Point2D<Au>,

    /// The end point of the gradient (computed during display list construction).
    pub end_point: Point2D<Au>,

    /// A list of color stops.
    pub stops: Vec<GradientStop>,
}

#[derive(Clone, Deserialize, HeapSizeOf, Serialize)]
pub struct GradientDisplayItem {
    /// Fields common to all display item.
    pub base: BaseDisplayItem,

    /// Contains all gradient data. Included start, end point and color stops.
    pub gradient: Gradient,
}

/// A normal border, supporting CSS border styles.
#[derive(Clone, HeapSizeOf, Deserialize, Serialize)]
pub struct NormalBorder {
    /// Border colors.
    pub color: SideOffsets2D<ColorF>,

    /// Border styles.
    pub style: SideOffsets2D<border_style::T>,

    /// Border radii.
    ///
    /// TODO(pcwalton): Elliptical radii.
    pub radius: BorderRadii<Au>,
}

/// A border that is made of image segments.
#[derive(Clone, HeapSizeOf, Deserialize, Serialize)]
pub struct ImageBorder {
    /// The image this border uses, border-image-source.
    pub image: WebRenderImageInfo,

    /// How to slice the image, as per border-image-slice.
    pub slice: SideOffsets2D<u32>,

    /// Outsets for the border, as per border-image-outset.
    pub outset: SideOffsets2D<f32>,

    /// If fill is true, draw the center patch of the image.
    pub fill: bool,

    /// How to repeat or stretch horizontal edges (border-image-repeat).
    #[ignore_heap_size_of = "WebRender traits type, and tiny"]
    pub repeat_horizontal: webrender_traits::RepeatMode,

    /// How to repeat or stretch vertical edges (border-image-repeat).
    #[ignore_heap_size_of = "WebRender traits type, and tiny"]
    pub repeat_vertical: webrender_traits::RepeatMode,
}

/// A border that is made of linear gradient
#[derive(Clone, HeapSizeOf, Deserialize, Serialize)]
pub struct GradientBorder {
    /// The gradient info that this border uses, border-image-source.
    pub gradient: Gradient,

    /// Outsets for the border, as per border-image-outset.
    pub outset: SideOffsets2D<f32>,
}

/// Specifies the type of border
#[derive(Clone, HeapSizeOf, Deserialize, Serialize)]
pub enum BorderDetails {
    Normal(NormalBorder),
    Image(ImageBorder),
    Gradient(GradientBorder),
}

/// Paints a border.
#[derive(Clone, HeapSizeOf, Deserialize, Serialize)]
pub struct BorderDisplayItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    /// Border widths.
    pub border_widths: SideOffsets2D<Au>,

    /// Details for specific border type
    pub details: BorderDetails,
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
        Size2D::new(corner.width.scale_by(s), corner.height.scale_by(s))
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
            top_left: Size2D::new(value.clone(), value.clone()),
            top_right: Size2D::new(value.clone(), value.clone()),
            bottom_right: Size2D::new(value.clone(), value.clone()),
            bottom_left: Size2D::new(value.clone(), value.clone()),
        }
    }
}

/// Paints a line segment.
#[derive(Clone, HeapSizeOf, Deserialize, Serialize)]
pub struct LineDisplayItem {
    pub base: BaseDisplayItem,

    /// The line segment color.
    pub color: ColorF,

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
    pub color: ColorF,

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

/// Defines a stacking context.
#[derive(Clone, HeapSizeOf, Deserialize, Serialize)]
pub struct PushStackingContextItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    pub stacking_context: StackingContext,
}

/// Defines a stacking context.
#[derive(Clone, HeapSizeOf, Deserialize, Serialize)]
pub struct PopStackingContextItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    pub stacking_context_id: StackingContextId,
}

/// Starts a group of items inside a particular scroll root.
#[derive(Clone, HeapSizeOf, Deserialize, Serialize)]
pub struct PushScrollRootItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    /// The scroll root that this item starts.
    pub scroll_root: ScrollRoot,
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
    pub fn base(&self) -> &BaseDisplayItem {
        match *self {
            DisplayItem::SolidColor(ref solid_color) => &solid_color.base,
            DisplayItem::Text(ref text) => &text.base,
            DisplayItem::Image(ref image_item) => &image_item.base,
            DisplayItem::WebGL(ref webgl_item) => &webgl_item.base,
            DisplayItem::Border(ref border) => &border.base,
            DisplayItem::Gradient(ref gradient) => &gradient.base,
            DisplayItem::Line(ref line) => &line.base,
            DisplayItem::BoxShadow(ref box_shadow) => &box_shadow.base,
            DisplayItem::Iframe(ref iframe) => &iframe.base,
            DisplayItem::PushStackingContext(ref stacking_context) => &stacking_context.base,
            DisplayItem::PopStackingContext(ref item) => &item.base,
            DisplayItem::PushScrollRoot(ref item) => &item.base,
            DisplayItem::PopScrollRoot(ref base) => &base,
        }
    }

    pub fn scroll_root_id(&self) -> ScrollRootId {
        self.base().scroll_root_id
    }

    pub fn stacking_context_id(&self) -> StackingContextId {
        self.base().stacking_context_id
    }

    pub fn section(&self) -> DisplayListSection {
        self.base().section
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
            DisplayItem::Border(ref border) => {
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
            DisplayItem::BoxShadow(_) => {
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
        if let DisplayItem::PushStackingContext(ref item) = *self {
            return write!(f, "PushStackingContext({:?})", item.stacking_context);
        }

        if let DisplayItem::PopStackingContext(ref item) = *self {
            return write!(f, "PopStackingContext({:?}", item.stacking_context_id);
        }

        if let DisplayItem::PushScrollRoot(ref item) = *self {
            return write!(f, "PushScrollRoot({:?}", item.scroll_root);
        }

        if let DisplayItem::PopScrollRoot(_) = *self {
            return write!(f, "PopScrollRoot");
        }

        write!(f, "{} @ {:?} {:?}",
            match *self {
                DisplayItem::SolidColor(ref solid_color) =>
                    format!("SolidColor rgba({}, {}, {}, {})",
                            solid_color.color.r,
                            solid_color.color.g,
                            solid_color.color.b,
                            solid_color.color.a),
                DisplayItem::Text(ref text) => {
                    format!("Text ({:?})",
                            &text.text_run.text[
                                text.range.begin().0 as usize..(text.range.begin().0 + text.range.length().0) as usize])
                }
                DisplayItem::Image(_) => "Image".to_owned(),
                DisplayItem::WebGL(_) => "WebGL".to_owned(),
                DisplayItem::Border(_) => "Border".to_owned(),
                DisplayItem::Gradient(_) => "Gradient".to_owned(),
                DisplayItem::Line(_) => "Line".to_owned(),
                DisplayItem::BoxShadow(_) => "BoxShadow".to_owned(),
                DisplayItem::Iframe(_) => "Iframe".to_owned(),
                DisplayItem::PushStackingContext(_) |
                DisplayItem::PopStackingContext(_) |
                DisplayItem::PushScrollRoot(_) |
                DisplayItem::PopScrollRoot(_) => "".to_owned(),
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
pub type ScrollOffsetMap = HashMap<ScrollRootId, Point2D<f32>>;


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
