/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Servo heavily uses display lists, which are retained-mode lists of painting commands to
//! perform. Using a list instead of painting elements in immediate mode allows transforms, hit
//! testing, and invalidation to be performed using the same primitives as painting. It also allows
//! Servo to aggressively cull invisible and out-of-bounds painting elements, to reduce overdraw.
//!
//! Display items describe relatively high-level drawing operations (for example, entire borders
//! and shadows instead of lines and blur operations), to reduce the amount of allocation required.
//! They are therefore not exactly analogous to constructs like Skia pictures, which consist of
//! low-level drawing primitives.

use euclid::{SideOffsets2D, TypedRect, Vector2D};
use gfx_traits::{self, StackingContextId};
use gfx_traits::print_tree::PrintTree;
use msg::constellation_msg::PipelineId;
use net_traits::image::base::Image;
use servo_geometry::MaxRect;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::f32;
use std::fmt;
use webrender_api::{BorderRadius, BorderWidths, BoxShadowClipMode, ClipMode, ColorF};
use webrender_api::{ComplexClipRegion, ExtendMode, ExternalScrollId, FilterOp, FontInstanceKey};
use webrender_api::{GlyphInstance, GradientStop, ImageKey, ImageRendering, LayoutPoint};
use webrender_api::{LayoutRect, LayoutSize, LayoutTransform, LayoutVector2D, LineStyle};
use webrender_api::{MixBlendMode, NinePatchBorder, NormalBorder, ScrollSensitivity, Shadow};
use webrender_api::{StickyOffsetBounds, TransformStyle};

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

impl gfx_traits::DisplayList for DisplayList {
    /// Analyze the display list to figure out if this may be the first
    /// contentful paint (i.e. the display list contains items of type text,
    /// image, non-white canvas or SVG). Used by metrics.
    fn is_contentful(&self) -> bool {
        for item in &self.list {
            match item {
                &DisplayItem::Text(_) | &DisplayItem::Image(_) => return true,
                _ => (),
            }
        }

        false
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
    pub margins: SideOffsets2D<Option<f32>>,
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
    SolidColor(Box<SolidColorDisplayItem>),
    Text(Box<TextDisplayItem>),
    Image(Box<ImageDisplayItem>),
    Border(Box<BorderDisplayItem>),
    Gradient(Box<GradientDisplayItem>),
    RadialGradient(Box<RadialGradientDisplayItem>),
    Line(Box<LineDisplayItem>),
    BoxShadow(Box<BoxShadowDisplayItem>),
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
    /// The boundaries of the display item, in layer coordinates.
    pub bounds: LayoutRect,

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
        bounds: LayoutRect,
        metadata: DisplayItemMetadata,
        clip_rect: LayoutRect,
        section: DisplayListSection,
        stacking_context_id: StackingContextId,
        clipping_and_scrolling: ClippingAndScrolling,
    ) -> BaseDisplayItem {
        BaseDisplayItem {
            bounds,
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
            bounds: TypedRect::zero(),
            metadata: DisplayItemMetadata {
                node: OpaqueNode(0),
                pointing: None,
            },
            // Create a rectangle of maximal size.
            clip_rect: LayoutRect::max_rect(),
            section: DisplayListSection::Content,
            stacking_context_id: StackingContextId::root(),
            clipping_and_scrolling:
                ClippingAndScrolling::simple(ClipScrollNodeIndex::root_scroll_node()),
        }
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

    /// Mutates this clipping region to intersect with the given rectangle.
    ///
    /// TODO(pcwalton): This could more eagerly eliminate complex clipping regions, at the cost of
    /// complexity.
    #[inline]
    pub fn intersect_rect(&mut self, rect: &LayoutRect) {
        self.main = self.main.intersection(rect).unwrap_or(LayoutRect::zero())
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
    pub fn might_intersect_point(&self, point: &LayoutPoint) -> bool {
        self.main.contains(point) &&
            self.complex
                .iter()
                .all(|complex| complex.rect.contains(point))
    }

    /// Returns true if this clipping region might intersect the given rectangle and false
    /// otherwise. This is a quick, not a precise, test; it can yield false positives.
    #[inline]
    pub fn might_intersect_rect(&self, rect: &LayoutRect) -> bool {
        self.main.intersects(rect) &&
            self.complex
                .iter()
                .all(|complex| complex.rect.intersects(rect))
    }

    /// Returns true if this clipping region completely surrounds the given rect.
    #[inline]
    pub fn does_not_clip_rect(&self, rect: &LayoutRect) -> bool {
        self.main.contains(&rect.origin) && self.main.contains(&rect.bottom_right()) &&
            self.complex.iter().all(|complex| {
                complex.rect.contains(&rect.origin) && complex.rect.contains(&rect.bottom_right())
            })
    }

    /// Returns a bounding rect that surrounds this entire clipping region.
    #[inline]
    pub fn bounding_rect(&self) -> LayoutRect {
        let mut rect = self.main;
        for complex in &*self.complex {
            rect = rect.union(&complex.rect)
        }
        rect
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
            if existing_complex_region.completely_encloses(&new_complex_region) {
                *existing_complex_region = new_complex_region;
                return;
            }
            if new_complex_region.completely_encloses(existing_complex_region) {
                return;
            }
        }

        self.complex.push(new_complex_region);
    }

    /// Translates this clipping region by the given vector.
    #[inline]
    pub fn translate(&self, delta: &LayoutVector2D) -> ClippingRegion {
        ClippingRegion {
            main: self.main.translate(delta),
            complex: self.complex
                .iter()
                .map(|complex| ComplexClipRegion {
                    rect: complex.rect.translate(delta),
                    radii: complex.radii,
                    mode: complex.mode,
                })
                .collect(),
        }
    }

    #[inline]
    pub fn is_max(&self) -> bool {
        self.main == LayoutRect::max_rect() && self.complex.is_empty()
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

pub trait CompletelyEncloses {
    fn completely_encloses(&self, other: &Self) -> bool;
}

impl CompletelyEncloses for ComplexClipRegion {
    // TODO(pcwalton): This could be more aggressive by considering points that touch the inside of
    // the border radius ellipse.
    fn completely_encloses(&self, other: &Self) -> bool {
        let left = self.radii.top_left.width.max(self.radii.bottom_left.width);
        let top = self.radii.top_left.height.max(self.radii.top_right.height);
        let right = self.radii
            .top_right
            .width
            .max(self.radii.bottom_right.width);
        let bottom = self.radii
            .bottom_left
            .height
            .max(self.radii.bottom_right.height);
        let interior = LayoutRect::new(
            LayoutPoint::new(self.rect.origin.x + left, self.rect.origin.y + top),
            LayoutSize::new(
                self.rect.size.width - left - right,
                self.rect.size.height - top - bottom,
            ),
        );
        interior.origin.x <= other.rect.origin.x && interior.origin.y <= other.rect.origin.y &&
            interior.max_x() >= other.rect.max_x() && interior.max_y() >= other.rect.max_y()
    }
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

/// Paints a solid color.
#[derive(Clone, Serialize)]
pub struct SolidColorDisplayItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    /// The color.
    pub color: ColorF,
}

/// Paints text.
#[derive(Clone, Serialize)]
pub struct TextDisplayItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,
    /// A collection of (non-whitespace) glyphs to be displayed.
    pub glyphs: Vec<GlyphInstance>,
    /// Reference to the font to be used.
    pub font_key: FontInstanceKey,
    /// The color of the text.
    pub text_color: ColorF,
}

#[derive(Clone, Eq, PartialEq, Serialize)]
pub enum TextOrientation {
    Upright,
    SidewaysLeft,
    SidewaysRight,
}

/// Paints an image.
#[derive(Clone, Serialize)]
pub struct ImageDisplayItem {
    pub base: BaseDisplayItem,

    pub id: ImageKey,

    /// The dimensions to which the image display item should be stretched. If this is smaller than
    /// the bounds of this display item, then the image will be repeated in the appropriate
    /// direction to tile the entire bounds.
    pub stretch_size: LayoutSize,

    /// The amount of space to add to the right and bottom part of each tile, when the image
    /// is tiled.
    pub tile_spacing: LayoutSize,

    /// The algorithm we should use to stretch the image. See `image_rendering` in CSS-IMAGES-3 §
    /// 5.3.
    pub image_rendering: ImageRendering,
}
/// Paints an iframe.
#[derive(Clone, Serialize)]
pub struct IframeDisplayItem {
    pub base: BaseDisplayItem,
    pub iframe: PipelineId,
}

/// Paints a gradient.
#[derive(Clone, Serialize)]
pub struct Gradient {
    /// The start point of the gradient (computed during display list construction).
    pub start_point: LayoutPoint,

    /// The end point of the gradient (computed during display list construction).
    pub end_point: LayoutPoint,

    /// A list of color stops.
    pub stops: Vec<GradientStop>,

    /// Whether the gradient is repeated or clamped.
    pub extend_mode: ExtendMode,
}

#[derive(Clone, Serialize)]
pub struct GradientDisplayItem {
    /// Fields common to all display item.
    pub base: BaseDisplayItem,

    /// Contains all gradient data. Included start, end point and color stops.
    pub gradient: Gradient,

    /// The size of a single gradient tile.
    ///
    /// The gradient may fill an entire element background
    /// but it can be composed from many smaller copys of
    /// the same gradient.
    ///
    /// Without tiles, the tile will be the same size as the background.
    pub tile: LayoutSize,
    pub tile_spacing: LayoutSize,
}

/// Paints a radial gradient.
#[derive(Clone, Serialize)]
pub struct RadialGradient {
    /// The center point of the gradient.
    pub center: LayoutPoint,

    /// The radius of the gradient with an x and an y component.
    pub radius: LayoutSize,

    /// A list of color stops.
    pub stops: Vec<GradientStop>,

    /// Whether the gradient is repeated or clamped.
    pub extend_mode: ExtendMode,
}

#[derive(Clone, Serialize)]
pub struct RadialGradientDisplayItem {
    /// Fields common to all display item.
    pub base: BaseDisplayItem,

    /// Contains all gradient data.
    pub gradient: RadialGradient,

    /// The size of a single gradient tile.
    ///
    /// The gradient may fill an entire element background
    /// but it can be composed from many smaller copys of
    /// the same gradient.
    ///
    /// Without tiles, the tile will be the same size as the background.
    pub tile: LayoutSize,
    pub tile_spacing: LayoutSize,
}

/// A border that is made of linear gradient
#[derive(Clone, Serialize)]
pub struct GradientBorder {
    /// The gradient info that this border uses, border-image-source.
    pub gradient: Gradient,

    /// Outsets for the border, as per border-image-outset.
    pub outset: SideOffsets2D<f32>,
}

/// A border that is made of radial gradient
#[derive(Clone, Serialize)]
pub struct RadialGradientBorder {
    /// The gradient info that this border uses, border-image-source.
    pub gradient: RadialGradient,

    /// Outsets for the border, as per border-image-outset.
    pub outset: SideOffsets2D<f32>,
}

/// Specifies the type of border
#[derive(Clone, Serialize)]
pub enum BorderDetails {
    Normal(NormalBorder),
    Image(NinePatchBorder),
    Gradient(GradientBorder),
    RadialGradient(RadialGradientBorder),
}

/// Paints a border.
#[derive(Clone, Serialize)]
pub struct BorderDisplayItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    /// Border widths.
    pub border_widths: BorderWidths,

    /// Details for specific border type
    pub details: BorderDetails,
}

/// Paints a line segment.
#[derive(Clone, Serialize)]
pub struct LineDisplayItem {
    pub base: BaseDisplayItem,

    /// The line segment color.
    pub color: ColorF,

    /// The line segment style.
    pub style: LineStyle,
}

/// Paints a box shadow per CSS-BACKGROUNDS.
#[derive(Clone, Serialize)]
pub struct BoxShadowDisplayItem {
    /// Fields common to all display items.
    pub base: BaseDisplayItem,

    /// The dimensions of the box that we're placing a shadow around.
    pub box_bounds: LayoutRect,

    /// The offset of this shadow from the box.
    pub offset: LayoutVector2D,

    /// The color of this shadow.
    pub color: ColorF,

    /// The blur radius for this shadow.
    pub blur_radius: f32,

    /// The spread radius of this shadow.
    pub spread_radius: f32,

    /// The border radius of this shadow.
    pub border_radius: BorderRadius,

    /// How we should clip the result.
    pub clip_mode: BoxShadowClipMode,
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
            DisplayItem::SolidColor(ref solid_color) => &solid_color.base,
            DisplayItem::Text(ref text) => &text.base,
            DisplayItem::Image(ref image_item) => &image_item.base,
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

    pub fn scroll_node_index(&self) -> ClipScrollNodeIndex {
        self.base().clipping_and_scrolling.scrolling
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
                DisplayItem::SolidColor(ref solid_color) => format!(
                    "SolidColor rgba({}, {}, {}, {})",
                    solid_color.color.r,
                    solid_color.color.g,
                    solid_color.color.b,
                    solid_color.color.a
                ),
                DisplayItem::Text(_) => "Text".to_owned(),
                DisplayItem::Image(_) => "Image".to_owned(),
                DisplayItem::Border(_) => "Border".to_owned(),
                DisplayItem::Gradient(_) => "Gradient".to_owned(),
                DisplayItem::RadialGradient(_) => "RadialGradient".to_owned(),
                DisplayItem::Line(_) => "Line".to_owned(),
                DisplayItem::BoxShadow(_) => "BoxShadow".to_owned(),
                DisplayItem::PushTextShadow(_) => "PushTextShadow".to_owned(),
                DisplayItem::PopAllTextShadows(_) => "PopTextShadow".to_owned(),
                DisplayItem::Iframe(_) => "Iframe".to_owned(),
                DisplayItem::PushStackingContext(_) |
                DisplayItem::PopStackingContext(_) |
                DisplayItem::DefineClipScrollNode(_) => "".to_owned(),
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
pub type ScrollOffsetMap = HashMap<ExternalScrollId, Vector2D<f32>>;

pub trait SimpleMatrixDetection {
    fn is_identity_or_simple_translation(&self) -> bool;
}

impl SimpleMatrixDetection for LayoutTransform {
    #[inline]
    fn is_identity_or_simple_translation(&self) -> bool {
        let (_0, _1) = (0.0, 1.0);
        self.m11 == _1 && self.m12 == _0 && self.m13 == _0 && self.m14 == _0 && self.m21 == _0 &&
            self.m22 == _1 && self.m23 == _0 && self.m24 == _0 && self.m31 == _0 &&
            self.m32 == _0 && self.m33 == _1 && self.m34 == _0 && self.m44 == _1
    }
}
