/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Servo heavily uses display lists, which are retained-mode lists of painting commands to
//! perform. Using a list instead of painting elements in immediate mode allows transforms, hit
//! testing, and invalidation to be performed using the same primitives as painting. It also allows
//! Servo to aggressively cull invisible and out-of-bounds painting elements, to reduce overdraw.
//!
//! Display items describe relatively high-level drawing operations (for example, entire borders
//! and shadows instead of lines and blur operations), to reduce the amount of allocation required.
//! They are therefore not exactly analogous to constructs like Skia pictures, which consist of
//! low-level drawing primitives.

use euclid::{SideOffsets2D, Vector2D};
use gfx_traits::print_tree::PrintTree;
use gfx_traits::{self, StackingContextId};
use msg::constellation_msg::PipelineId;
use net_traits::image::base::Image;
use servo_geometry::MaxRect;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::f32;
use std::fmt;
use style::computed_values::_servo_top_layer::T as InTopLayer;
use webrender_api as wr;
use webrender_api::units::{LayoutPixel, LayoutPoint, LayoutRect, LayoutSize, LayoutTransform};
use webrender_api::{
    BorderRadius, ClipId, ClipMode, CommonItemProperties, ComplexClipRegion, ExternalScrollId,
    FilterOp, GlyphInstance, GradientStop, ImageKey, MixBlendMode, PrimitiveFlags,
    ScrollSensitivity, Shadow, SpatialId, StickyOffsetBounds, TransformStyle,
};

pub use style::dom::OpaqueNode;

/// The factor that we multiply the blur radius by in order to inflate the boundaries of display
/// items that involve a blur. This ensures that the display item boundaries include all the ink.
pub static BLUR_INFLATION_FACTOR: i32 = 3;

/// An index into the vector of ClipScrollNodes. During WebRender conversion these nodes
/// are given ClipIds.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct ClipScrollNodeIndex(usize);

impl ClipScrollNodeIndex {
    pub fn root_scroll_node() -> ClipScrollNodeIndex {
        ClipScrollNodeIndex(1)
    }

    pub fn root_reference_frame() -> ClipScrollNodeIndex {
        ClipScrollNodeIndex(0)
    }

    pub fn new(index: usize) -> ClipScrollNodeIndex {
        assert_ne!(index, 0, "Use the root_reference_frame constructor");
        assert_ne!(index, 1, "Use the root_scroll_node constructor");
        ClipScrollNodeIndex(index)
    }

    pub fn is_root_scroll_node(&self) -> bool {
        *self == Self::root_scroll_node()
    }

    pub fn to_define_item(&self) -> DisplayItem {
        DisplayItem::DefineClipScrollNode(Box::new(DefineClipScrollNodeItem {
            base: BaseDisplayItem::empty(),
            node_index: *self,
        }))
    }

    pub fn to_index(self) -> usize {
        self.0
    }
}

/// A set of indices into the clip scroll node vector for a given item.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct ClippingAndScrolling {
    pub scrolling: ClipScrollNodeIndex,
    pub clipping: Option<ClipScrollNodeIndex>,
}

impl ClippingAndScrolling {
    pub fn simple(scrolling: ClipScrollNodeIndex) -> ClippingAndScrolling {
        ClippingAndScrolling {
            scrolling,
            clipping: None,
        }
    }

    pub fn new(scrolling: ClipScrollNodeIndex, clipping: ClipScrollNodeIndex) -> Self {
        ClippingAndScrolling {
            scrolling,
            clipping: Some(clipping),
        }
    }
}

#[derive(Serialize)]
pub struct DisplayList {
    pub list: Vec<DisplayItem>,
    pub clip_scroll_nodes: Vec<ClipScrollNode>,
}

impl DisplayList {
    /// Return the bounds of this display list based on the dimensions of the root
    /// stacking context.
    pub fn bounds(&self) -> LayoutRect {
        match self.list.get(0) {
            Some(&DisplayItem::PushStackingContext(ref item)) => item.stacking_context.bounds,
            Some(_) => unreachable!("Root element of display list not stacking context."),
            None => LayoutRect::zero(),
        }
    }

    pub fn print(&self) {
        let mut print_tree = PrintTree::new("Display List".to_owned());
        self.print_with_tree(&mut print_tree);
    }

    pub fn print_with_tree(&self, print_tree: &mut PrintTree) {
        print_tree.new_level("ClipScrollNodes".to_owned());
        for node in &self.clip_scroll_nodes {
            print_tree.add_item(format!("{:?}", node));
        }
        print_tree.end_level();

        print_tree.new_level("Items".to_owned());
        for item in &self.list {
            print_tree.add_item(format!(
                "{:?} StackingContext: {:?} {:?}",
                item,
                item.base().stacking_context_id,
                item.clipping_and_scrolling()
            ));
        }
        print_tree.end_level();
    }
}

/// Display list sections that make up a stacking context. Each section  here refers
/// to the steps in CSS 2.1 Appendix E.
///
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum DisplayListSection {
    BackgroundAndBorders,
    BlockBackgroundsAndBorders,
    Content,
    Outlines,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum StackingContextType {
    Real,
    PseudoPositioned,
    PseudoFloat,
}

#[derive(Clone, Serialize)]
/// Represents one CSS stacking context, which may or may not have a hardware layer.
pub struct StackingContext {
    /// The ID of this StackingContext for uniquely identifying it.
    pub id: StackingContextId,

    /// The type of this StackingContext. Used for collecting and sorting.
    pub context_type: StackingContextType,

    /// The position and size of this stacking context.
    pub bounds: LayoutRect,

    /// The overflow rect for this stacking context in its coordinate system.
    pub overflow: LayoutRect,

    /// The `z-index` for this stacking context.
    pub z_index: i32,

    /// Whether this is the top layer.
    pub in_top_layer: InTopLayer,

    /// CSS filters to be applied to this stacking context (including opacity).
    pub filters: Vec<FilterOp>,

    /// The blend mode with which this stacking context blends with its backdrop.
    pub mix_blend_mode: MixBlendMode,

    /// A transform to be applied to this stacking context.
    pub transform: Option<LayoutTransform>,

    /// The transform style of this stacking context.
    pub transform_style: TransformStyle,

    /// The perspective matrix to be applied to children.
    pub perspective: Option<LayoutTransform>,

    /// The clip and scroll info for this StackingContext.
    pub parent_clipping_and_scrolling: ClippingAndScrolling,

    /// The index of the reference frame that this stacking context estalishes.
    pub established_reference_frame: Option<ClipScrollNodeIndex>,
}

impl StackingContext {
    /// Creates a new stacking context.
    #[inline]
    pub fn new(
        id: StackingContextId,
        context_type: StackingContextType,
        bounds: LayoutRect,
        overflow: LayoutRect,
        z_index: i32,
        in_top_layer: InTopLayer,
        filters: Vec<FilterOp>,
        mix_blend_mode: MixBlendMode,
        transform: Option<LayoutTransform>,
        transform_style: TransformStyle,
        perspective: Option<LayoutTransform>,
        parent_clipping_and_scrolling: ClippingAndScrolling,
        established_reference_frame: Option<ClipScrollNodeIndex>,
    ) -> StackingContext {
        StackingContext {
            id,
            context_type,
            bounds,
            overflow,
            z_index,
            in_top_layer,
            filters,
            mix_blend_mode,
            transform,
            transform_style,
            perspective,
            parent_clipping_and_scrolling,
            established_reference_frame,
        }
    }

    #[inline]
    pub fn root() -> StackingContext {
        StackingContext::new(
            StackingContextId::root(),
            StackingContextType::Real,
            LayoutRect::zero(),
            LayoutRect::zero(),
            0,
            InTopLayer::None,
            vec![],
            MixBlendMode::Normal,
            None,
            TransformStyle::Flat,
            None,
            ClippingAndScrolling::simple(ClipScrollNodeIndex::root_scroll_node()),
            None,
        )
    }

    pub fn to_display_list_items(self) -> (DisplayItem, DisplayItem) {
        let mut base_item = BaseDisplayItem::empty();
        base_item.stacking_context_id = self.id;
        base_item.clipping_and_scrolling = self.parent_clipping_and_scrolling;

        let pop_item = DisplayItem::PopStackingContext(Box::new(PopStackingContextItem {
            base: base_item.clone(),
            stacking_context_id: self.id,
        }));

        let push_item = DisplayItem::PushStackingContext(Box::new(PushStackingContextItem {
            base: base_item,
            stacking_context: self,
        }));

        (push_item, pop_item)
    }
}

impl Ord for StackingContext {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.in_top_layer == InTopLayer::Top {
            if other.in_top_layer == InTopLayer::Top {
                return Ordering::Equal;
            } else {
                return Ordering::Greater;
            }
        } else if other.in_top_layer == InTopLayer::Top {
            return Ordering::Less;
        }

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
        let type_string = if self.context_type == StackingContextType::Real {
            "StackingContext"
        } else {
            "Pseudo-StackingContext"
        };

        write!(
            f,
            "{} at {:?} with overflow {:?}: {:?}",
            type_string, self.bounds, self.overflow, self.id
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StickyFrameData {
    pub margins: SideOffsets2D<Option<f32>, LayoutPixel>,
    pub vertical_offset_bounds: StickyOffsetBounds,
    pub horizontal_offset_bounds: StickyOffsetBounds,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum ClipScrollNodeType {
    Placeholder,
    ScrollFrame(ScrollSensitivity, ExternalScrollId),
    StickyFrame(StickyFrameData),
    Clip,
}

/// Defines a clip scroll node.
#[derive(Clone, Debug, Serialize)]
pub struct ClipScrollNode {
    /// The index of the parent of this ClipScrollNode.
    pub parent_index: ClipScrollNodeIndex,

    /// The position of this scroll root's frame in the parent stacking context.
    pub clip: ClippingRegion,

    /// The rect of the contents that can be scrolled inside of the scroll root.
    pub content_rect: LayoutRect,

    /// The type of this ClipScrollNode.
    pub node_type: ClipScrollNodeType,
}

impl ClipScrollNode {
    pub fn placeholder() -> ClipScrollNode {
        ClipScrollNode {
            parent_index: ClipScrollNodeIndex(0),
            clip: ClippingRegion::from_rect(LayoutRect::zero()),
            content_rect: LayoutRect::zero(),
            node_type: ClipScrollNodeType::Placeholder,
        }
    }

    pub fn is_placeholder(&self) -> bool {
        self.node_type == ClipScrollNodeType::Placeholder
    }
}

/// One drawing command in the list.
#[derive(Clone, Serialize)]
pub enum DisplayItem {
    Rectangle(Box<CommonDisplayItem<wr::RectangleDisplayItem>>),
    Text(Box<CommonDisplayItem<wr::TextDisplayItem, Vec<GlyphInstance>>>),
    Image(Box<CommonDisplayItem<wr::ImageDisplayItem>>),
    RepeatingImage(Box<CommonDisplayItem<wr::RepeatingImageDisplayItem>>),
    Border(Box<CommonDisplayItem<wr::BorderDisplayItem, Vec<GradientStop>>>),
    Gradient(Box<CommonDisplayItem<wr::GradientDisplayItem, Vec<GradientStop>>>),
    RadialGradient(Box<CommonDisplayItem<wr::RadialGradientDisplayItem, Vec<GradientStop>>>),
    Line(Box<CommonDisplayItem<wr::LineDisplayItem>>),
    BoxShadow(Box<CommonDisplayItem<wr::BoxShadowDisplayItem>>),
    PushTextShadow(Box<PushTextShadowDisplayItem>),
    PopAllTextShadows(Box<PopAllTextShadowsDisplayItem>),
    Iframe(Box<IframeDisplayItem>),
    PushStackingContext(Box<PushStackingContextItem>),
    PopStackingContext(Box<PopStackingContextItem>),
    DefineClipScrollNode(Box<DefineClipScrollNodeItem>),
}

/// Information common to all display items.
#[derive(Clone, Serialize)]
pub struct BaseDisplayItem {
    /// Metadata attached to this display item.
    pub metadata: DisplayItemMetadata,

    /// The clip rectangle to use for this item.
    pub clip_rect: LayoutRect,

    /// The section of the display list that this item belongs to.
    pub section: DisplayListSection,

    /// The id of the stacking context this item belongs to.
    pub stacking_context_id: StackingContextId,

    /// The clip and scroll info for this item.
    pub clipping_and_scrolling: ClippingAndScrolling,
}

impl BaseDisplayItem {
    #[inline(always)]
    pub fn new(
        metadata: DisplayItemMetadata,
        clip_rect: LayoutRect,
        section: DisplayListSection,
        stacking_context_id: StackingContextId,
        clipping_and_scrolling: ClippingAndScrolling,
    ) -> BaseDisplayItem {
        BaseDisplayItem {
            metadata,
            clip_rect,
            section,
            stacking_context_id,
            clipping_and_scrolling,
        }
    }

    #[inline(always)]
    pub fn empty() -> BaseDisplayItem {
        BaseDisplayItem {
            metadata: DisplayItemMetadata {
                node: OpaqueNode(0),
                pointing: None,
            },
            // Create a rectangle of maximal size.
            clip_rect: LayoutRect::max_rect(),
            section: DisplayListSection::Content,
            stacking_context_id: StackingContextId::root(),
            clipping_and_scrolling: ClippingAndScrolling::simple(
                ClipScrollNodeIndex::root_scroll_node(),
            ),
        }
    }
}

pub fn empty_common_item_properties() -> CommonItemProperties {
    CommonItemProperties {
        clip_rect: LayoutRect::max_rect(),
        clip_id: ClipId::root(wr::PipelineId::dummy()),
        spatial_id: SpatialId::root_scroll_node(wr::PipelineId::dummy()),
        hit_info: None,
        flags: PrimitiveFlags::empty(),
        item_key: None,
    }
}

/// A clipping region for a display item. Currently, this can describe rectangles, rounded
/// rectangles (for `border-radius`), or arbitrary intersections of the two. Arbitrary transforms
/// are not supported because those are handled by the higher-level `StackingContext` abstraction.
#[derive(Clone, PartialEq, Serialize)]
pub struct ClippingRegion {
    /// The main rectangular region. This does not include any corners.
    pub main: LayoutRect,
    /// Any complex regions.
    ///
    /// TODO(pcwalton): Atomically reference count these? Not sure if it's worth the trouble.
    /// Measure and follow up.
    pub complex: Vec<ComplexClipRegion>,
}

impl ClippingRegion {
    /// Returns an empty clipping region that, if set, will result in no pixels being visible.
    #[inline]
    pub fn empty() -> ClippingRegion {
        ClippingRegion {
            main: LayoutRect::zero(),
            complex: Vec::new(),
        }
    }

    /// Returns an all-encompassing clipping region that clips no pixels out.
    #[inline]
    pub fn max() -> ClippingRegion {
        ClippingRegion {
            main: LayoutRect::max_rect(),
            complex: Vec::new(),
        }
    }

    /// Returns a clipping region that represents the given rectangle.
    #[inline]
    pub fn from_rect(rect: LayoutRect) -> ClippingRegion {
        ClippingRegion {
            main: rect,
            complex: Vec::new(),
        }
    }

    /// Intersects this clipping region with the given rounded rectangle.
    #[inline]
    pub fn intersect_with_rounded_rect(&mut self, rect: LayoutRect, radii: BorderRadius) {
        let new_complex_region = ComplexClipRegion {
            rect,
            radii,
            mode: ClipMode::Clip,
        };

        // FIXME(pcwalton): This is O(n²) worst case for disjoint clipping regions. Is that OK?
        // They're slow anyway…
        //
        // Possibly relevant if we want to do better:
        //
        //     http://www.inrg.csie.ntu.edu.tw/algorithm2014/presentation/D&C%20Lee-84.pdf
        for existing_complex_region in &mut self.complex {
            if completely_encloses(&existing_complex_region, &new_complex_region) {
                *existing_complex_region = new_complex_region;
                return;
            }
            if completely_encloses(&new_complex_region, &existing_complex_region) {
                return;
            }
        }

        self.complex.push(new_complex_region);
    }
}

impl fmt::Debug for ClippingRegion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if *self == ClippingRegion::max() {
            write!(f, "ClippingRegion::Max")
        } else if *self == ClippingRegion::empty() {
            write!(f, "ClippingRegion::Empty")
        } else if self.main == LayoutRect::max_rect() {
            write!(f, "ClippingRegion(Complex={:?})", self.complex)
        } else {
            write!(
                f,
                "ClippingRegion(Rect={:?}, Complex={:?})",
                self.main, self.complex
            )
        }
    }
}

// TODO(pcwalton): This could be more aggressive by considering points that touch the inside of
// the border radius ellipse.
fn completely_encloses(this: &ComplexClipRegion, other: &ComplexClipRegion) -> bool {
    let left = this.radii.top_left.width.max(this.radii.bottom_left.width);
    let top = this.radii.top_left.height.max(this.radii.top_right.height);
    let right = this
        .radii
        .top_right
        .width
        .max(this.radii.bottom_right.width);
    let bottom = this
        .radii
        .bottom_left
        .height
        .max(this.radii.bottom_right.height);
    let interior = LayoutRect::new(
        LayoutPoint::new(this.rect.origin.x + left, this.rect.origin.y + top),
        LayoutSize::new(
            this.rect.size.width - left - right,
            this.rect.size.height - top - bottom,
        ),
    );
    interior.origin.x <= other.rect.origin.x &&
        interior.origin.y <= other.rect.origin.y &&
        interior.max_x() >= other.rect.max_x() &&
        interior.max_y() >= other.rect.max_y()
}

/// Metadata attached to each display item. This is useful for performing auxiliary threads with
/// the display list involving hit testing: finding the originating DOM node and determining the
/// cursor to use when the element is hovered over.
#[derive(Clone, Copy, Serialize)]
pub struct DisplayItemMetadata {
    /// The DOM node from which this display item originated.
    pub node: OpaqueNode,
    /// The value of the `cursor` property when the mouse hovers over this display item. If `None`,
    /// this display item is ineligible for pointer events (`pointer-events: none`).
    pub pointing: Option<u16>,
}

#[derive(Clone, Eq, PartialEq, Serialize)]
pub enum TextOrientation {
    Upright,
    SidewaysLeft,
    SidewaysRight,
}

/// Paints an iframe.
#[derive(Clone, Serialize)]
pub struct IframeDisplayItem {
    pub base: BaseDisplayItem,
    pub iframe: PipelineId,
    pub bounds: LayoutRect,
}

#[derive(Clone, Serialize)]
pub struct CommonDisplayItem<T, U = ()> {
    pub base: BaseDisplayItem,
    pub item: T,
    pub data: U,
}

impl<T> CommonDisplayItem<T> {
    pub fn new(base: BaseDisplayItem, item: T) -> Box<CommonDisplayItem<T>> {
        Box::new(CommonDisplayItem {
            base,
            item,
            data: (),
        })
    }
}

impl<T, U> CommonDisplayItem<T, U> {
    pub fn with_data(base: BaseDisplayItem, item: T, data: U) -> Box<CommonDisplayItem<T, U>> {
        Box::new(CommonDisplayItem { base, item, data })
    }
}

/// Defines a text shadow that affects all items until the paired PopTextShadow.
#[derive(Clone, Serialize)]
pub struct PushTextShadowDisplayItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    pub shadow: Shadow,
}

/// Defines a text shadow that affects all items until the next PopTextShadow.
#[derive(Clone, Serialize)]
pub struct PopAllTextShadowsDisplayItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,
}

/// Defines a stacking context.
#[derive(Clone, Serialize)]
pub struct PushStackingContextItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    pub stacking_context: StackingContext,
}

/// Defines a stacking context.
#[derive(Clone, Serialize)]
pub struct PopStackingContextItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    pub stacking_context_id: StackingContextId,
}

/// Starts a group of items inside a particular scroll root.
#[derive(Clone, Serialize)]
pub struct DefineClipScrollNodeItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    /// The scroll root that this item starts.
    pub node_index: ClipScrollNodeIndex,
}

impl DisplayItem {
    pub fn base(&self) -> &BaseDisplayItem {
        match *self {
            DisplayItem::Rectangle(ref rect) => &rect.base,
            DisplayItem::Text(ref text) => &text.base,
            DisplayItem::Image(ref image_item) => &image_item.base,
            DisplayItem::RepeatingImage(ref image_item) => &image_item.base,
            DisplayItem::Border(ref border) => &border.base,
            DisplayItem::Gradient(ref gradient) => &gradient.base,
            DisplayItem::RadialGradient(ref gradient) => &gradient.base,
            DisplayItem::Line(ref line) => &line.base,
            DisplayItem::BoxShadow(ref box_shadow) => &box_shadow.base,
            DisplayItem::PushTextShadow(ref push_text_shadow) => &push_text_shadow.base,
            DisplayItem::PopAllTextShadows(ref pop_text_shadow) => &pop_text_shadow.base,
            DisplayItem::Iframe(ref iframe) => &iframe.base,
            DisplayItem::PushStackingContext(ref stacking_context) => &stacking_context.base,
            DisplayItem::PopStackingContext(ref item) => &item.base,
            DisplayItem::DefineClipScrollNode(ref item) => &item.base,
        }
    }

    pub fn clipping_and_scrolling(&self) -> ClippingAndScrolling {
        self.base().clipping_and_scrolling
    }

    pub fn stacking_context_id(&self) -> StackingContextId {
        self.base().stacking_context_id
    }

    pub fn section(&self) -> DisplayListSection {
        self.base().section
    }

    pub fn bounds(&self) -> LayoutRect {
        match *self {
            DisplayItem::Rectangle(ref item) => item.item.common.clip_rect,
            DisplayItem::Text(ref item) => item.item.bounds,
            DisplayItem::Image(ref item) => item.item.bounds,
            DisplayItem::RepeatingImage(ref item) => item.item.bounds,
            DisplayItem::Border(ref item) => item.item.bounds,
            DisplayItem::Gradient(ref item) => item.item.bounds,
            DisplayItem::RadialGradient(ref item) => item.item.bounds,
            DisplayItem::Line(ref item) => item.item.area,
            DisplayItem::BoxShadow(ref item) => item.item.box_bounds,
            DisplayItem::PushTextShadow(_) => LayoutRect::zero(),
            DisplayItem::PopAllTextShadows(_) => LayoutRect::zero(),
            DisplayItem::Iframe(ref item) => item.bounds,
            DisplayItem::PushStackingContext(ref item) => item.stacking_context.bounds,
            DisplayItem::PopStackingContext(_) => LayoutRect::zero(),
            DisplayItem::DefineClipScrollNode(_) => LayoutRect::zero(),
        }
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

        if let DisplayItem::DefineClipScrollNode(ref item) = *self {
            return write!(f, "DefineClipScrollNode({:?}", item.node_index);
        }

        write!(
            f,
            "{} @ {:?} {:?}",
            match *self {
                DisplayItem::Rectangle(_) => "Rectangle",
                DisplayItem::Text(_) => "Text",
                DisplayItem::Image(_) => "Image",
                DisplayItem::RepeatingImage(_) => "RepeatingImage",
                DisplayItem::Border(_) => "Border",
                DisplayItem::Gradient(_) => "Gradient",
                DisplayItem::RadialGradient(_) => "RadialGradient",
                DisplayItem::Line(_) => "Line",
                DisplayItem::BoxShadow(_) => "BoxShadow",
                DisplayItem::PushTextShadow(_) => "PushTextShadow",
                DisplayItem::PopAllTextShadows(_) => "PopTextShadow",
                DisplayItem::Iframe(_) => "Iframe",
                DisplayItem::PushStackingContext(_) |
                DisplayItem::PopStackingContext(_) |
                DisplayItem::DefineClipScrollNode(_) => "",
            },
            self.bounds(),
            self.base().clip_rect
        )
    }
}

#[derive(Clone, Copy, Serialize)]
pub struct WebRenderImageInfo {
    pub width: u32,
    pub height: u32,
    pub key: Option<ImageKey>,
}

impl WebRenderImageInfo {
    #[inline]
    pub fn from_image(image: &Image) -> WebRenderImageInfo {
        WebRenderImageInfo {
            width: image.width,
            height: image.height,
            key: image.id,
        }
    }
}

/// The type of the scroll offset list. This is only populated if WebRender is in use.
pub type ScrollOffsetMap = HashMap<ExternalScrollId, Vector2D<f32, LayoutPixel>>;
