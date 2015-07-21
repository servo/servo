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

use azure::azure_hl::Color;
use euclid::{Point2D, Rect, Size2D, SideOffsets2D, Matrix4};
use libc::uintptr_t;
use paint_task::PaintLayer;
use std::sync::Arc;
use style::computed_values::{border_style, filter, image_rendering, mix_blend_mode};
use text::glyph::CharIndex;
use text::TextRun;
use util::geometry::Au;
use util::mem::HeapSizeOf;
use util::linked_list::SerializableLinkedList;
use util::range::Range;

/// An opaque handle to a node. The only safe operation that can be performed on this node is to
/// compare it to another opaque handle or to another node.
///
/// Because the script task's GC does not trace layout, node data cannot be safely stored in layout
/// data structures. Also, layout code tends to be faster when the DOM is not being accessed, for
/// locality reasons. Using `OpaqueNode` enforces this invariant.
#[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf, Deserialize, Serialize)]
pub struct OpaqueNode(pub uintptr_t);

#[derive(HeapSizeOf, Deserialize, Serialize)]
/// Represents one CSS stacking context, which may or may not have a hardware layer.
pub struct StackingContext {
    /// The display items that make up this stacking context.
    pub display_list: Box<DisplayList>,

    /// The layer for this stacking context, if there is one.
    #[ignore_heap_size_of = "FIXME(njn): should measure this at some point"]
    pub layer: Option<Arc<PaintLayer>>,

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
}

/// Display items that make up a stacking context. "Steps" here refer to the steps in CSS 2.1
/// Appendix E.
///
/// TODO(pcwalton): We could reduce the size of this structure with a more "skip list"-like
/// structure, omitting several pointers and lengths.
#[derive(HeapSizeOf, Deserialize, Serialize)]
pub struct DisplayList {
    /// The border and backgrounds for the root of this stacking context: steps 1 and 2.
    pub background_and_borders: SerializableLinkedList<DisplayItem>,
    /// Borders and backgrounds for block-level descendants: step 4.
    pub block_backgrounds_and_borders: SerializableLinkedList<DisplayItem>,
    /// Floats: step 5. These are treated as pseudo-stacking contexts.
    pub floats: SerializableLinkedList<DisplayItem>,
    /// All non-positioned content.
    pub content: SerializableLinkedList<DisplayItem>,
    /// All positioned content that does not get a stacking context.
    pub positioned_content: SerializableLinkedList<DisplayItem>,
    /// Outlines: step 10.
    pub outlines: SerializableLinkedList<DisplayItem>,
    /// Child stacking contexts.
    pub children: SerializableLinkedList<Arc<StackingContext>>,
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
    pub text_run: Arc<Box<TextRun>>,

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
