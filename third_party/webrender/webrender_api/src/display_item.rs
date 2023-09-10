/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::{SideOffsets2D, Angle};
use peek_poke::PeekPoke;
use std::ops::Not;
// local imports
use crate::font;
use crate::{PipelineId, PropertyBinding};
use crate::color::ColorF;
use crate::image::{ColorDepth, ImageKey};
use crate::units::*;
use std::hash::{Hash, Hasher};

// ******************************************************************
// * NOTE: some of these structs have an "IMPLICIT" comment.        *
// * This indicates that the BuiltDisplayList will have serialized  *
// * a list of values nearby that this item consumes. The traversal *
// * iterator should handle finding these. DebugDisplayItem should  *
// * make them explicit.                                            *
// ******************************************************************

/// A tag that can be used to identify items during hit testing. If the tag
/// is missing then the item doesn't take part in hit testing at all. This
/// is composed of two numbers. In Servo, the first is an identifier while the
/// second is used to select the cursor that should be used during mouse
/// movement. In Gecko, the first is a scrollframe identifier, while the second
/// is used to store various flags that APZ needs to properly process input
/// events.
pub type ItemTag = (u64, u16);

/// An identifier used to refer to previously sent display items. Currently it
/// refers to individual display items, but this may change later.
pub type ItemKey = u16;

bitflags! {
    #[repr(C)]
    #[derive(Deserialize, MallocSizeOf, Serialize, PeekPoke)]
    pub struct PrimitiveFlags: u8 {
        /// The CSS backface-visibility property (yes, it can be really granular)
        const IS_BACKFACE_VISIBLE = 1 << 0;
        /// If set, this primitive represents a scroll bar container
        const IS_SCROLLBAR_CONTAINER = 1 << 1;
        /// If set, this primitive represents a scroll bar thumb
        const IS_SCROLLBAR_THUMB = 1 << 2;
        /// This is used as a performance hint - this primitive may be promoted to a native
        /// compositor surface under certain (implementation specific) conditions. This
        /// is typically used for large videos, and canvas elements.
        const PREFER_COMPOSITOR_SURFACE = 1 << 3;
        /// If set, this primitive can be passed directly to the compositor via its
        /// ExternalImageId, and the compositor will use the native image directly.
        /// Used as a further extension on top of PREFER_COMPOSITOR_SURFACE.
        const SUPPORTS_EXTERNAL_COMPOSITOR_SURFACE = 1 << 4;
    }
}

impl Default for PrimitiveFlags {
    fn default() -> Self {
        PrimitiveFlags::IS_BACKFACE_VISIBLE
    }
}

/// A grouping of fields a lot of display items need, just to avoid
/// repeating these over and over in this file.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct CommonItemProperties {
    /// Bounds of the display item to clip to. Many items are logically
    /// infinite, and rely on this clip_rect to define their bounds
    /// (solid colors, background-images, gradients, etc).
    pub clip_rect: LayoutRect,
    /// Additional clips
    pub clip_id: ClipId,
    /// The coordinate-space the item is in (yes, it can be really granular)
    pub spatial_id: SpatialId,
    /// Various flags describing properties of this primitive.
    pub flags: PrimitiveFlags,
}

impl CommonItemProperties {
    /// Convenience for tests.
    pub fn new(
        clip_rect: LayoutRect,
        space_and_clip: SpaceAndClipInfo,
    ) -> Self {
        Self {
            clip_rect,
            spatial_id: space_and_clip.spatial_id,
            clip_id: space_and_clip.clip_id,
            flags: PrimitiveFlags::default(),
        }
    }
}

/// Per-primitive information about the nodes in the clip tree and
/// the spatial tree that the primitive belongs to.
///
/// Note: this is a separate struct from `PrimitiveInfo` because
/// it needs indirectional mapping during the DL flattening phase,
/// turning into `ScrollNodeAndClipChain`.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct SpaceAndClipInfo {
    pub spatial_id: SpatialId,
    pub clip_id: ClipId,
}

impl SpaceAndClipInfo {
    /// Create a new space/clip info associated with the root
    /// scroll frame.
    pub fn root_scroll(pipeline_id: PipelineId) -> Self {
        SpaceAndClipInfo {
            spatial_id: SpatialId::root_scroll_node(pipeline_id),
            clip_id: ClipId::root(pipeline_id),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, PeekPoke)]
pub enum DisplayItem {
    // These are the "real content" display items
    Rectangle(RectangleDisplayItem),
    ClearRectangle(ClearRectangleDisplayItem),
    HitTest(HitTestDisplayItem),
    Text(TextDisplayItem),
    Line(LineDisplayItem),
    Border(BorderDisplayItem),
    BoxShadow(BoxShadowDisplayItem),
    PushShadow(PushShadowDisplayItem),
    Gradient(GradientDisplayItem),
    RadialGradient(RadialGradientDisplayItem),
    ConicGradient(ConicGradientDisplayItem),
    Image(ImageDisplayItem),
    RepeatingImage(RepeatingImageDisplayItem),
    YuvImage(YuvImageDisplayItem),
    BackdropFilter(BackdropFilterDisplayItem),

    // Clips
    RectClip(RectClipDisplayItem),
    RoundedRectClip(RoundedRectClipDisplayItem),
    ImageMaskClip(ImageMaskClipDisplayItem),
    Clip(ClipDisplayItem),
    ClipChain(ClipChainItem),

    // Spaces and Frames that content can be scoped under.
    ScrollFrame(ScrollFrameDisplayItem),
    StickyFrame(StickyFrameDisplayItem),
    Iframe(IframeDisplayItem),
    PushReferenceFrame(ReferenceFrameDisplayListItem),
    PushStackingContext(PushStackingContextDisplayItem),

    // These marker items indicate an array of data follows, to be used for the
    // next non-marker item.
    SetGradientStops,
    SetFilterOps,
    SetFilterData,
    SetFilterPrimitives,
    SetPoints,

    // These marker items terminate a scope introduced by a previous item.
    PopReferenceFrame,
    PopStackingContext,
    PopAllShadows,

    ReuseItems(ItemKey),
    RetainedItems(ItemKey),
}

/// This is a "complete" version of the DisplayItem, with all implicit trailing
/// arrays included, for debug serialization (captures).
#[cfg(any(feature = "serialize", feature = "deserialize"))]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
pub enum DebugDisplayItem {
    Rectangle(RectangleDisplayItem),
    ClearRectangle(ClearRectangleDisplayItem),
    HitTest(HitTestDisplayItem),
    Text(TextDisplayItem, Vec<font::GlyphInstance>),
    Line(LineDisplayItem),
    Border(BorderDisplayItem),
    BoxShadow(BoxShadowDisplayItem),
    PushShadow(PushShadowDisplayItem),
    Gradient(GradientDisplayItem),
    RadialGradient(RadialGradientDisplayItem),
    ConicGradient(ConicGradientDisplayItem),
    Image(ImageDisplayItem),
    RepeatingImage(RepeatingImageDisplayItem),
    YuvImage(YuvImageDisplayItem),
    BackdropFilter(BackdropFilterDisplayItem),

    ImageMaskClip(ImageMaskClipDisplayItem),
    RoundedRectClip(RoundedRectClipDisplayItem),
    RectClip(RectClipDisplayItem),
    Clip(ClipDisplayItem, Vec<ComplexClipRegion>),
    ClipChain(ClipChainItem, Vec<ClipId>),

    ScrollFrame(ScrollFrameDisplayItem),
    StickyFrame(StickyFrameDisplayItem),
    Iframe(IframeDisplayItem),
    PushReferenceFrame(ReferenceFrameDisplayListItem),
    PushStackingContext(PushStackingContextDisplayItem),

    SetGradientStops(Vec<GradientStop>),
    SetFilterOps(Vec<FilterOp>),
    SetFilterData(FilterData),
    SetFilterPrimitives(Vec<FilterPrimitive>),
    SetPoints(Vec<LayoutPoint>),

    PopReferenceFrame,
    PopStackingContext,
    PopAllShadows,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct ImageMaskClipDisplayItem {
    pub id: ClipId,
    pub parent_space_and_clip: SpaceAndClipInfo,
    pub image_mask: ImageMask,
    pub fill_rule: FillRule,
} // IMPLICIT points: Vec<LayoutPoint>

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct RectClipDisplayItem {
    pub id: ClipId,
    pub parent_space_and_clip: SpaceAndClipInfo,
    pub clip_rect: LayoutRect,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct RoundedRectClipDisplayItem {
    pub id: ClipId,
    pub parent_space_and_clip: SpaceAndClipInfo,
    pub clip: ComplexClipRegion,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct ClipDisplayItem {
    pub id: ClipId,
    pub parent_space_and_clip: SpaceAndClipInfo,
    pub clip_rect: LayoutRect,
} // IMPLICIT: complex_clips: Vec<ComplexClipRegion>

/// The minimum and maximum allowable offset for a sticky frame in a single dimension.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct StickyOffsetBounds {
    /// The minimum offset for this frame, typically a negative value, which specifies how
    /// far in the negative direction the sticky frame can offset its contents in this
    /// dimension.
    pub min: f32,

    /// The maximum offset for this frame, typically a positive value, which specifies how
    /// far in the positive direction the sticky frame can offset its contents in this
    /// dimension.
    pub max: f32,
}

impl StickyOffsetBounds {
    pub fn new(min: f32, max: f32) -> StickyOffsetBounds {
        StickyOffsetBounds { min, max }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct StickyFrameDisplayItem {
    pub id: SpatialId,
    pub parent_spatial_id: SpatialId,
    pub bounds: LayoutRect,

    /// The margins that should be maintained between the edge of the parent viewport and this
    /// sticky frame. A margin of None indicates that the sticky frame should not stick at all
    /// to that particular edge of the viewport.
    pub margins: SideOffsets2D<Option<f32>, LayoutPixel>,

    /// The minimum and maximum vertical offsets for this sticky frame. Ignoring these constraints,
    /// the sticky frame will continue to stick to the edge of the viewport as its original
    /// position is scrolled out of view. Constraints specify a maximum and minimum offset from the
    /// original position relative to non-sticky content within the same scrolling frame.
    pub vertical_offset_bounds: StickyOffsetBounds,

    /// The minimum and maximum horizontal offsets for this sticky frame. Ignoring these constraints,
    /// the sticky frame will continue to stick to the edge of the viewport as its original
    /// position is scrolled out of view. Constraints specify a maximum and minimum offset from the
    /// original position relative to non-sticky content within the same scrolling frame.
    pub horizontal_offset_bounds: StickyOffsetBounds,

    /// The amount of offset that has already been applied to the sticky frame. A positive y
    /// component this field means that a top-sticky item was in a scrollframe that has been
    /// scrolled down, such that the sticky item's position needed to be offset downwards by
    /// `previously_applied_offset.y`. A negative y component corresponds to the upward offset
    /// applied due to bottom-stickiness. The x-axis works analogously.
    pub previously_applied_offset: LayoutVector2D,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, PeekPoke)]
pub enum ScrollSensitivity {
    ScriptAndInputEvents,
    Script,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct ScrollFrameDisplayItem {
    /// The id of the clip this scroll frame creates
    pub clip_id: ClipId,
    /// The id of the space this scroll frame creates
    pub scroll_frame_id: SpatialId,
    /// The size of the contents this contains (so the backend knows how far it can scroll).
    // FIXME: this can *probably* just be a size? Origin seems to just get thrown out.
    pub content_rect: LayoutRect,
    pub clip_rect: LayoutRect,
    pub parent_space_and_clip: SpaceAndClipInfo,
    pub external_id: ExternalScrollId,
    pub scroll_sensitivity: ScrollSensitivity,
    /// The amount this scrollframe has already been scrolled by, in the caller.
    /// This means that all the display items that are inside the scrollframe
    /// will have their coordinates shifted by this amount, and this offset
    /// should be added to those display item coordinates in order to get a
    /// normalized value that is consistent across display lists.
    pub external_scroll_offset: LayoutVector2D,
}

/// A solid or an animating color to draw (may not actually be a rectangle due to complex clips)
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct RectangleDisplayItem {
    pub common: CommonItemProperties,
    pub bounds: LayoutRect,
    pub color: PropertyBinding<ColorF>,
}

/// Clears all colors from the area, making it possible to cut holes in the window.
/// (useful for things like the macos frosted-glass effect).
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct ClearRectangleDisplayItem {
    pub common: CommonItemProperties,
    pub bounds: LayoutRect,
}

/// A minimal hit-testable item for the parent browser's convenience, and is
/// slimmer than a RectangleDisplayItem (no color). The existence of this as a
/// distinct item also makes it easier to inspect/debug display items.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct HitTestDisplayItem {
    pub common: CommonItemProperties,
    pub tag: ItemTag,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct LineDisplayItem {
    pub common: CommonItemProperties,
    /// We need a separate rect from common.clip_rect to encode cute
    /// tricks that firefox does to make a series of text-decorations seamlessly
    /// line up -- snapping the decorations to a multiple of their period, and
    /// then clipping them to their "proper" area. This rect is that "logical"
    /// snapped area that may be clipped to the right size by the clip_rect.
    pub area: LayoutRect,
    /// Whether the rect is interpretted as vertical or horizontal
    pub orientation: LineOrientation,
    /// This could potentially be implied from area, but we currently prefer
    /// that this is the responsibility of the layout engine. Value irrelevant
    /// for non-wavy lines.
    // FIXME: this was done before we could use tagged unions in enums, but now
    // it should just be part of LineStyle::Wavy.
    pub wavy_line_thickness: f32,
    pub color: ColorF,
    pub style: LineStyle,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize, Eq, Hash, PeekPoke)]
pub enum LineOrientation {
    Vertical,
    Horizontal,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize, Eq, Hash, PeekPoke)]
pub enum LineStyle {
    Solid,
    Dotted,
    Dashed,
    Wavy,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct TextDisplayItem {
    pub common: CommonItemProperties,
    /// The area all the glyphs should be found in. Strictly speaking this isn't
    /// necessarily needed, but layout engines should already "know" this, and we
    /// use it cull and size things quickly before glyph layout is done. Currently
    /// the glyphs *can* be outside these bounds, but that should imply they
    /// can be cut off.
    // FIXME: these are currently sometimes ignored to keep some old wrench tests
    // working, but we should really just fix the tests!
    pub bounds: LayoutRect,
    pub font_key: font::FontInstanceKey,
    pub color: ColorF,
    pub glyph_options: Option<font::GlyphOptions>,
} // IMPLICIT: glyphs: Vec<font::GlyphInstance>

#[derive(Clone, Copy, Debug, Default, Deserialize, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub struct NormalBorder {
    pub left: BorderSide,
    pub right: BorderSide,
    pub top: BorderSide,
    pub bottom: BorderSide,
    pub radius: BorderRadius,
    /// Whether to apply anti-aliasing on the border corners.
    ///
    /// Note that for this to be `false` and work, this requires the borders to
    /// be solid, and no border-radius.
    pub do_aa: bool,
}

impl NormalBorder {
    fn can_disable_antialiasing(&self) -> bool {
        fn is_valid(style: BorderStyle) -> bool {
            style == BorderStyle::Solid || style == BorderStyle::None
        }

        self.radius.is_zero() &&
            is_valid(self.top.style) &&
            is_valid(self.left.style) &&
            is_valid(self.bottom.style) &&
            is_valid(self.right.style)
    }

    /// Normalizes a border so that we don't render disallowed stuff, like inset
    /// borders that are less than two pixels wide.
    #[inline]
    pub fn normalize(&mut self, widths: &LayoutSideOffsets) {
        debug_assert!(
            self.do_aa || self.can_disable_antialiasing(),
            "Unexpected disabled-antialiasing in a border, likely won't work or will be ignored"
        );

        #[inline]
        fn renders_small_border_solid(style: BorderStyle) -> bool {
            match style {
                BorderStyle::Groove |
                BorderStyle::Ridge => true,
                _ => false,
            }
        }

        let normalize_side = |side: &mut BorderSide, width: f32| {
            if renders_small_border_solid(side.style) && width < 2. {
                side.style = BorderStyle::Solid;
            }
        };

        normalize_side(&mut self.left, widths.left);
        normalize_side(&mut self.right, widths.right);
        normalize_side(&mut self.top, widths.top);
        normalize_side(&mut self.bottom, widths.bottom);
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, MallocSizeOf, PartialEq, Serialize, Deserialize, Eq, Hash, PeekPoke)]
pub enum RepeatMode {
    Stretch,
    Repeat,
    Round,
    Space,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, PeekPoke)]
pub enum NinePatchBorderSource {
    Image(ImageKey),
    Gradient(Gradient),
    RadialGradient(RadialGradient),
    ConicGradient(ConicGradient),
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct NinePatchBorder {
    /// Describes what to use as the 9-patch source image. If this is an image,
    /// it will be stretched to fill the size given by width x height.
    pub source: NinePatchBorderSource,

    /// The width of the 9-part image.
    pub width: i32,

    /// The height of the 9-part image.
    pub height: i32,

    /// Distances from each edge where the image should be sliced up. These
    /// values are in 9-part-image space (the same space as width and height),
    /// and the resulting image parts will be used to fill the corresponding
    /// parts of the border as given by the border widths. This can lead to
    /// stretching.
    /// Slices can be overlapping. In that case, the same pixels from the
    /// 9-part image will show up in multiple parts of the resulting border.
    pub slice: DeviceIntSideOffsets,

    /// Controls whether the center of the 9 patch image is rendered or
    /// ignored. The center is never rendered if the slices are overlapping.
    pub fill: bool,

    /// Determines what happens if the horizontal side parts of the 9-part
    /// image have a different size than the horizontal parts of the border.
    pub repeat_horizontal: RepeatMode,

    /// Determines what happens if the vertical side parts of the 9-part
    /// image have a different size than the vertical parts of the border.
    pub repeat_vertical: RepeatMode,

    /// The outset for the border.
    /// TODO(mrobinson): This should be removed and handled by the client.
    pub outset: LayoutSideOffsets, // TODO: what unit is this in?
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, PeekPoke)]
pub enum BorderDetails {
    Normal(NormalBorder),
    NinePatch(NinePatchBorder),
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct BorderDisplayItem {
    pub common: CommonItemProperties,
    pub bounds: LayoutRect,
    pub widths: LayoutSideOffsets,
    pub details: BorderDetails,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, PeekPoke)]
pub enum BorderRadiusKind {
    Uniform,
    NonUniform,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub struct BorderRadius {
    pub top_left: LayoutSize,
    pub top_right: LayoutSize,
    pub bottom_left: LayoutSize,
    pub bottom_right: LayoutSize,
}

impl Default for BorderRadius {
    fn default() -> Self {
        BorderRadius {
            top_left: LayoutSize::zero(),
            top_right: LayoutSize::zero(),
            bottom_left: LayoutSize::zero(),
            bottom_right: LayoutSize::zero(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub struct BorderSide {
    pub color: ColorF,
    pub style: BorderStyle,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize, Hash, Eq, PeekPoke)]
pub enum BorderStyle {
    None = 0,
    Solid = 1,
    Double = 2,
    Dotted = 3,
    Dashed = 4,
    Hidden = 5,
    Groove = 6,
    Ridge = 7,
    Inset = 8,
    Outset = 9,
}

impl BorderStyle {
    pub fn is_hidden(self) -> bool {
        self == BorderStyle::Hidden || self == BorderStyle::None
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub enum BoxShadowClipMode {
    Outset = 0,
    Inset = 1,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct BoxShadowDisplayItem {
    pub common: CommonItemProperties,
    pub box_bounds: LayoutRect,
    pub offset: LayoutVector2D,
    pub color: ColorF,
    pub blur_radius: f32,
    pub spread_radius: f32,
    pub border_radius: BorderRadius,
    pub clip_mode: BoxShadowClipMode,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct PushShadowDisplayItem {
    pub space_and_clip: SpaceAndClipInfo,
    pub shadow: Shadow,
    pub should_inflate: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct Shadow {
    pub offset: LayoutVector2D,
    pub color: ColorF,
    pub blur_radius: f32,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, Eq, MallocSizeOf, PartialEq, Serialize, Deserialize, Ord, PartialOrd, PeekPoke)]
pub enum ExtendMode {
    Clamp,
    Repeat,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct Gradient {
    pub start_point: LayoutPoint,
    pub end_point: LayoutPoint,
    pub extend_mode: ExtendMode,
} // IMPLICIT: stops: Vec<GradientStop>

impl Gradient {
    pub fn is_valid(&self) -> bool {
        self.start_point.x.is_finite() &&
            self.start_point.y.is_finite() &&
            self.end_point.x.is_finite() &&
            self.end_point.y.is_finite()
    }
}

/// The area
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct GradientDisplayItem {
    /// NOTE: common.clip_rect is the area the gradient covers
    pub common: CommonItemProperties,
    /// The area to tile the gradient over (first tile starts at origin of this rect)
    // FIXME: this should ideally just be `tile_origin` here, with the clip_rect
    // defining the bounds of the item. Needs non-trivial backend changes.
    pub bounds: LayoutRect,
    /// How big a tile of the of the gradient should be (common case: bounds.size)
    pub tile_size: LayoutSize,
    /// The space between tiles of the gradient (common case: 0)
    pub tile_spacing: LayoutSize,
    pub gradient: Gradient,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub struct GradientStop {
    pub offset: f32,
    pub color: ColorF,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct RadialGradient {
    pub center: LayoutPoint,
    pub radius: LayoutSize,
    pub start_offset: f32,
    pub end_offset: f32,
    pub extend_mode: ExtendMode,
} // IMPLICIT stops: Vec<GradientStop>

impl RadialGradient {
    pub fn is_valid(&self) -> bool {
        self.center.x.is_finite() &&
            self.center.y.is_finite() &&
            self.start_offset.is_finite() &&
            self.end_offset.is_finite()
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct ConicGradient {
    pub center: LayoutPoint,
    pub angle: f32,
    pub start_offset: f32,
    pub end_offset: f32,
    pub extend_mode: ExtendMode,
} // IMPLICIT stops: Vec<GradientStop>

impl ConicGradient {
    pub fn is_valid(&self) -> bool {
        self.center.x.is_finite() &&
            self.center.y.is_finite() &&
            self.angle.is_finite() &&
            self.start_offset.is_finite() &&
            self.end_offset.is_finite()
    }
}

/// Just an abstraction for bundling up a bunch of clips into a "super clip".
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct ClipChainItem {
    pub id: ClipChainId,
    pub parent: Option<ClipChainId>,
} // IMPLICIT clip_ids: Vec<ClipId>

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct RadialGradientDisplayItem {
    pub common: CommonItemProperties,
    /// The area to tile the gradient over (first tile starts at origin of this rect)
    // FIXME: this should ideally just be `tile_origin` here, with the clip_rect
    // defining the bounds of the item. Needs non-trivial backend changes.
    pub bounds: LayoutRect,
    pub gradient: RadialGradient,
    pub tile_size: LayoutSize,
    pub tile_spacing: LayoutSize,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct ConicGradientDisplayItem {
    pub common: CommonItemProperties,
    /// The area to tile the gradient over (first tile starts at origin of this rect)
    // FIXME: this should ideally just be `tile_origin` here, with the clip_rect
    // defining the bounds of the item. Needs non-trivial backend changes.
    pub bounds: LayoutRect,
    pub gradient: ConicGradient,
    pub tile_size: LayoutSize,
    pub tile_spacing: LayoutSize,
}

/// Renders a filtered region of its backdrop
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct BackdropFilterDisplayItem {
    pub common: CommonItemProperties,
}
// IMPLICIT: filters: Vec<FilterOp>, filter_datas: Vec<FilterData>, filter_primitives: Vec<FilterPrimitive>

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct ReferenceFrameDisplayListItem {
    pub origin: LayoutPoint,
    pub parent_spatial_id: SpatialId,
    pub reference_frame: ReferenceFrame,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, PeekPoke)]
pub enum ReferenceFrameKind {
    /// A normal transform matrix, may contain perspective (the CSS transform property)
    Transform {
        /// Optionally marks the transform as only ever having a simple 2D scale or translation,
        /// allowing for optimizations.
        is_2d_scale_translation: bool,
        /// Marks that the transform should be snapped. Used for transforms which animate in
        /// response to scrolling, eg for zooming or dynamic toolbar fixed-positioning.
        should_snap: bool,
    },
    /// A perspective transform, that optionally scrolls relative to a specific scroll node
    Perspective {
        scrolling_relative_to: Option<ExternalScrollId>,
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, PeekPoke)]
pub enum Rotation {
    Degree0,
    Degree90,
    Degree180,
    Degree270,
}

impl Rotation {
    pub fn to_matrix(
        &self,
        size: LayoutSize,
    ) -> LayoutTransform {
        let (shift_center_to_origin, angle) = match self {
            Rotation::Degree0 => {
              (LayoutTransform::translation(-size.width / 2., -size.height / 2., 0.), Angle::degrees(0.))
            },
            Rotation::Degree90 => {
              (LayoutTransform::translation(-size.height / 2., -size.width / 2., 0.), Angle::degrees(90.))
            },
            Rotation::Degree180 => {
              (LayoutTransform::translation(-size.width / 2., -size.height / 2., 0.), Angle::degrees(180.))
            },
            Rotation::Degree270 => {
              (LayoutTransform::translation(-size.height / 2., -size.width / 2., 0.), Angle::degrees(270.))
            },
        };
        let shift_origin_to_center = LayoutTransform::translation(size.width / 2., size.height / 2., 0.);

        shift_center_to_origin
            .then(&LayoutTransform::rotation(0., 0., 1.0, angle))
            .then(&shift_origin_to_center)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, PeekPoke)]
pub enum ReferenceTransformBinding {
    /// Standard reference frame which contains a precomputed transform.
    Static {
        binding: PropertyBinding<LayoutTransform>,
    },
    /// Computed reference frame which dynamically calculates the transform
    /// based on the given parameters. The reference is the content size of
    /// the parent iframe, which is affected by snapping.
    Computed {
        scale_from: Option<LayoutSize>,
        vertical_flip: bool,
        rotation: Rotation,
    },
}

impl Default for ReferenceTransformBinding {
    fn default() -> Self {
        ReferenceTransformBinding::Static {
            binding: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct ReferenceFrame {
    pub kind: ReferenceFrameKind,
    pub transform_style: TransformStyle,
    /// The transform matrix, either the perspective matrix or the transform
    /// matrix.
    pub transform: ReferenceTransformBinding,
    pub id: SpatialId,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct PushStackingContextDisplayItem {
    pub origin: LayoutPoint,
    pub spatial_id: SpatialId,
    pub prim_flags: PrimitiveFlags,
    pub stacking_context: StackingContext,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct StackingContext {
    pub transform_style: TransformStyle,
    pub mix_blend_mode: MixBlendMode,
    pub clip_id: Option<ClipId>,
    pub raster_space: RasterSpace,
    pub flags: StackingContextFlags,
}
// IMPLICIT: filters: Vec<FilterOp>, filter_datas: Vec<FilterData>, filter_primitives: Vec<FilterPrimitive>

#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, PeekPoke)]
pub enum TransformStyle {
    Flat = 0,
    Preserve3D = 1,
}

/// Configure whether the contents of a stacking context
/// should be rasterized in local space or screen space.
/// Local space rasterized pictures are typically used
/// when we want to cache the output, and performance is
/// important. Note that this is a performance hint only,
/// which WR may choose to ignore.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, MallocSizeOf, Serialize, PeekPoke)]
#[repr(u8)]
pub enum RasterSpace {
    // Rasterize in local-space, applying supplied scale to primitives.
    // Best performance, but lower quality.
    Local(f32),

    // Rasterize the picture in screen-space, including rotation / skew etc in
    // the rasterized element. Best quality, but slower performance. Note that
    // any stacking context with a perspective transform will be rasterized
    // in local-space, even if this is set.
    Screen,
}

impl RasterSpace {
    pub fn local_scale(self) -> Option<f32> {
        match self {
            RasterSpace::Local(scale) => Some(scale),
            RasterSpace::Screen => None,
        }
    }
}

impl Eq for RasterSpace {}

impl Hash for RasterSpace {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            RasterSpace::Screen => {
                0.hash(state);
            }
            RasterSpace::Local(scale) => {
                // Note: this is inconsistent with the Eq impl for -0.0 (don't care).
                1.hash(state);
                scale.to_bits().hash(state);
            }
        }
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Deserialize, MallocSizeOf, Serialize, PeekPoke)]
    pub struct StackingContextFlags: u8 {
        /// If true, this stacking context represents a backdrop root, per the CSS
        /// filter-effects specification (see https://drafts.fxtf.org/filter-effects-2/#BackdropRoot).
        const IS_BACKDROP_ROOT = 1 << 0;
        /// If true, this stacking context is a blend container than contains
        /// mix-blend-mode children (and should thus be isolated).
        const IS_BLEND_CONTAINER = 1 << 1;
    }
}

impl Default for StackingContextFlags {
    fn default() -> Self {
        StackingContextFlags::empty()
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub enum MixBlendMode {
    Normal = 0,
    Multiply = 1,
    Screen = 2,
    Overlay = 3,
    Darken = 4,
    Lighten = 5,
    ColorDodge = 6,
    ColorBurn = 7,
    HardLight = 8,
    SoftLight = 9,
    Difference = 10,
    Exclusion = 11,
    Hue = 12,
    Saturation = 13,
    Color = 14,
    Luminosity = 15,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub enum ColorSpace {
    Srgb,
    LinearRgb,
}

/// Available composite operoations for the composite filter primitive
#[repr(C)]
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub enum CompositeOperator {
    Over,
    In,
    Atop,
    Out,
    Xor,
    Lighter,
    Arithmetic([f32; 4]),
}

impl CompositeOperator {
    // This must stay in sync with the composite operator defines in cs_svg_filter.glsl
    pub fn as_int(&self) -> u32 {
        match self {
            CompositeOperator::Over => 0,
            CompositeOperator::In => 1,
            CompositeOperator::Out => 2,
            CompositeOperator::Atop => 3,
            CompositeOperator::Xor => 4,
            CompositeOperator::Lighter => 5,
            CompositeOperator::Arithmetic(..) => 6,
        }
    }
}

/// An input to a SVG filter primitive.
#[repr(C)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub enum FilterPrimitiveInput {
    /// The input is the original graphic that the filter is being applied to.
    Original,
    /// The input is the output of the previous filter primitive in the filter primitive chain.
    Previous,
    /// The input is the output of the filter primitive at the given index in the filter primitive chain.
    OutputOfPrimitiveIndex(usize),
}

impl FilterPrimitiveInput {
    /// Gets the index of the input.
    /// Returns `None` if the source graphic is the input.
    pub fn to_index(self, cur_index: usize) -> Option<usize> {
        match self {
            FilterPrimitiveInput::Previous if cur_index > 0 => Some(cur_index - 1),
            FilterPrimitiveInput::OutputOfPrimitiveIndex(index) => Some(index),
            _ => None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct BlendPrimitive {
    pub input1: FilterPrimitiveInput,
    pub input2: FilterPrimitiveInput,
    pub mode: MixBlendMode,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct FloodPrimitive {
    pub color: ColorF,
}

impl FloodPrimitive {
    pub fn sanitize(&mut self) {
        self.color.r = self.color.r.min(1.0).max(0.0);
        self.color.g = self.color.g.min(1.0).max(0.0);
        self.color.b = self.color.b.min(1.0).max(0.0);
        self.color.a = self.color.a.min(1.0).max(0.0);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct BlurPrimitive {
    pub input: FilterPrimitiveInput,
    pub width: f32,
    pub height: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct OpacityPrimitive {
    pub input: FilterPrimitiveInput,
    pub opacity: f32,
}

impl OpacityPrimitive {
    pub fn sanitize(&mut self) {
        self.opacity = self.opacity.min(1.0).max(0.0);
    }
}

/// cbindgen:derive-eq=false
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct ColorMatrixPrimitive {
    pub input: FilterPrimitiveInput,
    pub matrix: [f32; 20],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct DropShadowPrimitive {
    pub input: FilterPrimitiveInput,
    pub shadow: Shadow,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct ComponentTransferPrimitive {
    pub input: FilterPrimitiveInput,
    // Component transfer data is stored in FilterData.
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct IdentityPrimitive {
    pub input: FilterPrimitiveInput,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct OffsetPrimitive {
    pub input: FilterPrimitiveInput,
    pub offset: LayoutVector2D,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct CompositePrimitive {
    pub input1: FilterPrimitiveInput,
    pub input2: FilterPrimitiveInput,
    pub operator: CompositeOperator,
}

/// See: https://github.com/eqrion/cbindgen/issues/9
/// cbindgen:derive-eq=false
#[repr(C)]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, PeekPoke)]
pub enum FilterPrimitiveKind {
    Identity(IdentityPrimitive),
    Blend(BlendPrimitive),
    Flood(FloodPrimitive),
    Blur(BlurPrimitive),
    // TODO: Support animated opacity?
    Opacity(OpacityPrimitive),
    /// cbindgen:derive-eq=false
    ColorMatrix(ColorMatrixPrimitive),
    DropShadow(DropShadowPrimitive),
    ComponentTransfer(ComponentTransferPrimitive),
    Offset(OffsetPrimitive),
    Composite(CompositePrimitive),
}

impl Default for FilterPrimitiveKind {
    fn default() -> Self {
        FilterPrimitiveKind::Identity(IdentityPrimitive::default())
    }
}

impl FilterPrimitiveKind {
    pub fn sanitize(&mut self) {
        match self {
            FilterPrimitiveKind::Flood(flood) => flood.sanitize(),
            FilterPrimitiveKind::Opacity(opacity) => opacity.sanitize(),

            // No sanitization needed.
            FilterPrimitiveKind::Identity(..) |
            FilterPrimitiveKind::Blend(..) |
            FilterPrimitiveKind::ColorMatrix(..) |
            FilterPrimitiveKind::Offset(..) |
            FilterPrimitiveKind::Composite(..) |
            FilterPrimitiveKind::Blur(..) |
            FilterPrimitiveKind::DropShadow(..) |
            // Component transfer's filter data is sanitized separately.
            FilterPrimitiveKind::ComponentTransfer(..) => {}
        }
    }
}

/// SVG Filter Primitive.
/// See: https://github.com/eqrion/cbindgen/issues/9
/// cbindgen:derive-eq=false
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct FilterPrimitive {
    pub kind: FilterPrimitiveKind,
    pub color_space: ColorSpace,
}

impl FilterPrimitive {
    pub fn sanitize(&mut self) {
        self.kind.sanitize();
    }
}

/// CSS filter.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, PeekPoke)]
pub enum FilterOp {
    /// Filter that does no transformation of the colors, needed for
    /// debug purposes only.
    Identity,
    Blur(f32, f32),
    Brightness(f32),
    Contrast(f32),
    Grayscale(f32),
    HueRotate(f32),
    Invert(f32),
    Opacity(PropertyBinding<f32>, f32),
    Saturate(f32),
    Sepia(f32),
    DropShadow(Shadow),
    ColorMatrix([f32; 20]),
    SrgbToLinear,
    LinearToSrgb,
    ComponentTransfer,
    Flood(ColorF),
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, PeekPoke)]
pub enum ComponentTransferFuncType {
  Identity = 0,
  Table = 1,
  Discrete = 2,
  Linear = 3,
  Gamma = 4,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct FilterData {
    pub func_r_type: ComponentTransferFuncType,
    pub r_values: Vec<f32>,
    pub func_g_type: ComponentTransferFuncType,
    pub g_values: Vec<f32>,
    pub func_b_type: ComponentTransferFuncType,
    pub b_values: Vec<f32>,
    pub func_a_type: ComponentTransferFuncType,
    pub a_values: Vec<f32>,
}

fn sanitize_func_type(
    func_type: ComponentTransferFuncType,
    values: &[f32],
) -> ComponentTransferFuncType {
    if values.is_empty() {
        return ComponentTransferFuncType::Identity;
    }
    if values.len() < 2 && func_type == ComponentTransferFuncType::Linear {
        return ComponentTransferFuncType::Identity;
    }
    if values.len() < 3 && func_type == ComponentTransferFuncType::Gamma {
        return ComponentTransferFuncType::Identity;
    }
    func_type
}

fn sanitize_values(
    func_type: ComponentTransferFuncType,
    values: &[f32],
) -> bool {
    if values.len() < 2 && func_type == ComponentTransferFuncType::Linear {
        return false;
    }
    if values.len() < 3 && func_type == ComponentTransferFuncType::Gamma {
        return false;
    }
    true
}

impl FilterData {
    /// Ensure that the number of values matches up with the function type.
    pub fn sanitize(&self) -> FilterData {
        FilterData {
            func_r_type: sanitize_func_type(self.func_r_type, &self.r_values),
            r_values:
                    if sanitize_values(self.func_r_type, &self.r_values) {
                        self.r_values.clone()
                    } else {
                        Vec::new()
                    },
            func_g_type: sanitize_func_type(self.func_g_type, &self.g_values),
            g_values:
                    if sanitize_values(self.func_g_type, &self.g_values) {
                        self.g_values.clone()
                    } else {
                        Vec::new()
                    },

            func_b_type: sanitize_func_type(self.func_b_type, &self.b_values),
            b_values:
                    if sanitize_values(self.func_b_type, &self.b_values) {
                        self.b_values.clone()
                    } else {
                        Vec::new()
                    },

            func_a_type: sanitize_func_type(self.func_a_type, &self.a_values),
            a_values:
                    if sanitize_values(self.func_a_type, &self.a_values) {
                        self.a_values.clone()
                    } else {
                        Vec::new()
                    },

        }
    }

    pub fn is_identity(&self) -> bool {
        self.func_r_type == ComponentTransferFuncType::Identity &&
        self.func_g_type == ComponentTransferFuncType::Identity &&
        self.func_b_type == ComponentTransferFuncType::Identity &&
        self.func_a_type == ComponentTransferFuncType::Identity
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct IframeDisplayItem {
    pub bounds: LayoutRect,
    pub clip_rect: LayoutRect,
    pub space_and_clip: SpaceAndClipInfo,
    pub pipeline_id: PipelineId,
    pub ignore_missing_pipeline: bool,
}

/// This describes an image that fills the specified area. It stretches or shrinks
/// the image as necessary. While RepeatingImageDisplayItem could otherwise provide
/// a superset of the functionality, it has been problematic inferring the desired
/// repetition properties when snapping changes the size of the primitive.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct ImageDisplayItem {
    pub common: CommonItemProperties,
    /// The area to tile the image over (first tile starts at origin of this rect)
    // FIXME: this should ideally just be `tile_origin` here, with the clip_rect
    // defining the bounds of the item. Needs non-trivial backend changes.
    pub bounds: LayoutRect,
    pub image_key: ImageKey,
    pub image_rendering: ImageRendering,
    pub alpha_type: AlphaType,
    /// A hack used by gecko to color a simple bitmap font used for tofu glyphs
    pub color: ColorF,
}

/// This describes a background-image and its tiling. It repeats in a grid to fill
/// the specified area.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct RepeatingImageDisplayItem {
    pub common: CommonItemProperties,
    /// The area to tile the image over (first tile starts at origin of this rect)
    // FIXME: this should ideally just be `tile_origin` here, with the clip_rect
    // defining the bounds of the item. Needs non-trivial backend changes.
    pub bounds: LayoutRect,
    /// How large to make a single tile of the image (common case: bounds.size)
    pub stretch_size: LayoutSize,
    /// The space between tiles (common case: 0)
    pub tile_spacing: LayoutSize,
    pub image_key: ImageKey,
    pub image_rendering: ImageRendering,
    pub alpha_type: AlphaType,
    /// A hack used by gecko to color a simple bitmap font used for tofu glyphs
    pub color: ColorF,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub enum ImageRendering {
    Auto = 0,
    CrispEdges = 1,
    Pixelated = 2,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub enum AlphaType {
    Alpha = 0,
    PremultipliedAlpha = 1,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct YuvImageDisplayItem {
    pub common: CommonItemProperties,
    pub bounds: LayoutRect,
    pub yuv_data: YuvData,
    pub color_depth: ColorDepth,
    pub color_space: YuvColorSpace,
    pub color_range: ColorRange,
    pub image_rendering: ImageRendering,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub enum YuvColorSpace {
    Rec601 = 0,
    Rec709 = 1,
    Rec2020 = 2,
    Identity = 3, // aka RGB as per ISO/IEC 23091-2:2019
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub enum ColorRange {
    Limited = 0,
    Full = 1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, PeekPoke)]
pub enum YuvData {
    NV12(ImageKey, ImageKey), // (Y channel, CbCr interleaved channel)
    PlanarYCbCr(ImageKey, ImageKey, ImageKey), // (Y channel, Cb channel, Cr Channel)
    InterleavedYCbCr(ImageKey), // (YCbCr interleaved channel)
}

impl YuvData {
    pub fn get_format(&self) -> YuvFormat {
        match *self {
            YuvData::NV12(..) => YuvFormat::NV12,
            YuvData::PlanarYCbCr(..) => YuvFormat::PlanarYCbCr,
            YuvData::InterleavedYCbCr(..) => YuvFormat::InterleavedYCbCr,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub enum YuvFormat {
    NV12 = 0,
    PlanarYCbCr = 1,
    InterleavedYCbCr = 2,
}

impl YuvFormat {
    pub fn get_plane_num(self) -> usize {
        match self {
            YuvFormat::NV12 => 2,
            YuvFormat::PlanarYCbCr => 3,
            YuvFormat::InterleavedYCbCr => 1,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct ImageMask {
    pub image: ImageKey,
    pub rect: LayoutRect,
    pub repeat: bool,
}

impl ImageMask {
    /// Get a local clipping rect contributed by this mask.
    pub fn get_local_clip_rect(&self) -> Option<LayoutRect> {
        if self.repeat {
            None
        } else {
            Some(self.rect)
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, MallocSizeOf, PartialEq, Serialize, Deserialize, Eq, Hash, PeekPoke)]
pub enum ClipMode {
    Clip,    // Pixels inside the region are visible.
    ClipOut, // Pixels outside the region are visible.
}

impl Not for ClipMode {
    type Output = ClipMode;

    fn not(self) -> ClipMode {
        match self {
            ClipMode::Clip => ClipMode::ClipOut,
            ClipMode::ClipOut => ClipMode::Clip,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, PeekPoke)]
pub struct ComplexClipRegion {
    /// The boundaries of the rectangle.
    pub rect: LayoutRect,
    /// Border radii of this rectangle.
    pub radii: BorderRadius,
    /// Whether we are clipping inside or outside
    /// the region.
    pub mode: ClipMode,
}

impl BorderRadius {
    pub fn zero() -> BorderRadius {
        BorderRadius {
            top_left: LayoutSize::new(0.0, 0.0),
            top_right: LayoutSize::new(0.0, 0.0),
            bottom_left: LayoutSize::new(0.0, 0.0),
            bottom_right: LayoutSize::new(0.0, 0.0),
        }
    }

    pub fn uniform(radius: f32) -> BorderRadius {
        BorderRadius {
            top_left: LayoutSize::new(radius, radius),
            top_right: LayoutSize::new(radius, radius),
            bottom_left: LayoutSize::new(radius, radius),
            bottom_right: LayoutSize::new(radius, radius),
        }
    }

    pub fn uniform_size(radius: LayoutSize) -> BorderRadius {
        BorderRadius {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }

    pub fn is_uniform(&self) -> Option<f32> {
        match self.is_uniform_size() {
            Some(radius) if radius.width == radius.height => Some(radius.width),
            _ => None,
        }
    }

    pub fn is_uniform_size(&self) -> Option<LayoutSize> {
        let uniform_radius = self.top_left;
        if self.top_right == uniform_radius && self.bottom_left == uniform_radius &&
            self.bottom_right == uniform_radius
        {
            Some(uniform_radius)
        } else {
            None
        }
    }

    /// Return whether, in each corner, the radius in *either* direction is zero.
    /// This means that none of the corners are rounded.
    pub fn is_zero(&self) -> bool {
        let corner_is_zero = |corner: &LayoutSize| corner.width == 0.0 || corner.height == 0.0;
        corner_is_zero(&self.top_left) &&
        corner_is_zero(&self.top_right) &&
        corner_is_zero(&self.bottom_right) &&
        corner_is_zero(&self.bottom_left)
    }
}

impl ComplexClipRegion {
    /// Create a new complex clip region.
    pub fn new(
        rect: LayoutRect,
        radii: BorderRadius,
        mode: ClipMode,
    ) -> Self {
        ComplexClipRegion { rect, radii, mode }
    }
}

impl ComplexClipRegion {
    /// Get a local clipping rect contributed by this clip region.
    pub fn get_local_clip_rect(&self) -> Option<LayoutRect> {
        match self.mode {
            ClipMode::Clip => {
                Some(self.rect)
            }
            ClipMode::ClipOut => {
                None
            }
        }
    }
}

pub const POLYGON_CLIP_VERTEX_MAX: usize = 16;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize, Eq, Hash, PeekPoke)]
pub enum FillRule {
    Nonzero = 0x1, // Behaves as the SVG fill-rule definition for nonzero.
    Evenodd = 0x2, // Behaves as the SVG fill-rule definition for evenodd.
}

impl From<u8> for FillRule {
    fn from(fill_rule: u8) -> Self {
        match fill_rule {
            0x1 => FillRule::Nonzero,
            0x2 => FillRule::Evenodd,
            _ => panic!("Unexpected FillRule value."),
        }
    }
}

impl From<FillRule> for u8 {
    fn from(fill_rule: FillRule) -> Self {
        match fill_rule {
            FillRule::Nonzero => 0x1,
            FillRule::Evenodd => 0x2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize, PeekPoke)]
pub struct ClipChainId(pub u64, pub PipelineId);

/// A reference to a clipping node defining how an item is clipped.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, PeekPoke)]
pub enum ClipId {
    Clip(usize, PipelineId),
    ClipChain(ClipChainId),
}

const ROOT_CLIP_ID: usize = 0;

impl ClipId {
    /// Return the root clip ID - effectively doing no clipping.
    pub fn root(pipeline_id: PipelineId) -> Self {
        ClipId::Clip(ROOT_CLIP_ID, pipeline_id)
    }

    /// Return an invalid clip ID - needed in places where we carry
    /// one but need to not attempt to use it.
    pub fn invalid() -> Self {
        ClipId::Clip(!0, PipelineId::dummy())
    }

    pub fn pipeline_id(&self) -> PipelineId {
        match *self {
            ClipId::Clip(_, pipeline_id) |
            ClipId::ClipChain(ClipChainId(_, pipeline_id)) => pipeline_id,
        }
    }

    pub fn is_root(&self) -> bool {
        match *self {
            ClipId::Clip(id, _) => id == ROOT_CLIP_ID,
            ClipId::ClipChain(_) => false,
        }
    }

    pub fn is_valid(&self) -> bool {
        match *self {
            ClipId::Clip(id, _) => id != !0,
            _ => true,
        }
    }
}

/// A reference to a spatial node defining item positioning.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize, PeekPoke)]
pub struct SpatialId(pub usize, PipelineId);

const ROOT_REFERENCE_FRAME_SPATIAL_ID: usize = 0;
const ROOT_SCROLL_NODE_SPATIAL_ID: usize = 1;

impl SpatialId {
    pub fn new(spatial_node_index: usize, pipeline_id: PipelineId) -> Self {
        SpatialId(spatial_node_index, pipeline_id)
    }

    pub fn root_reference_frame(pipeline_id: PipelineId) -> Self {
        SpatialId(ROOT_REFERENCE_FRAME_SPATIAL_ID, pipeline_id)
    }

    pub fn root_scroll_node(pipeline_id: PipelineId) -> Self {
        SpatialId(ROOT_SCROLL_NODE_SPATIAL_ID, pipeline_id)
    }

    pub fn pipeline_id(&self) -> PipelineId {
        self.1
    }

    pub fn is_root_reference_frame(&self) -> bool {
        self.0 == ROOT_REFERENCE_FRAME_SPATIAL_ID
    }

    pub fn is_root_scroll_node(&self) -> bool {
        self.0 == ROOT_SCROLL_NODE_SPATIAL_ID
    }
}

/// An external identifier that uniquely identifies a scroll frame independent of its ClipId, which
/// may change from frame to frame. This should be unique within a pipeline. WebRender makes no
/// attempt to ensure uniqueness. The zero value is reserved for use by the root scroll node of
/// every pipeline, which always has an external id.
///
/// When setting display lists with the `preserve_frame_state` this id is used to preserve scroll
/// offsets between different sets of SpatialNodes which are ScrollFrames.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize, PeekPoke)]
#[repr(C)]
pub struct ExternalScrollId(pub u64, pub PipelineId);

impl ExternalScrollId {
    pub fn pipeline_id(&self) -> PipelineId {
        self.1
    }

    pub fn is_root(&self) -> bool {
        self.0 == 0
    }
}

impl DisplayItem {
    pub fn debug_name(&self) -> &'static str {
        match *self {
            DisplayItem::Border(..) => "border",
            DisplayItem::BoxShadow(..) => "box_shadow",
            DisplayItem::ClearRectangle(..) => "clear_rectangle",
            DisplayItem::HitTest(..) => "hit_test",
            DisplayItem::RectClip(..) => "rect_clip",
            DisplayItem::RoundedRectClip(..) => "rounded_rect_clip",
            DisplayItem::ImageMaskClip(..) => "image_mask_clip",
            DisplayItem::Clip(..) => "clip",
            DisplayItem::ClipChain(..) => "clip_chain",
            DisplayItem::ConicGradient(..) => "conic_gradient",
            DisplayItem::Gradient(..) => "gradient",
            DisplayItem::Iframe(..) => "iframe",
            DisplayItem::Image(..) => "image",
            DisplayItem::RepeatingImage(..) => "repeating_image",
            DisplayItem::Line(..) => "line",
            DisplayItem::PopAllShadows => "pop_all_shadows",
            DisplayItem::PopReferenceFrame => "pop_reference_frame",
            DisplayItem::PopStackingContext => "pop_stacking_context",
            DisplayItem::PushShadow(..) => "push_shadow",
            DisplayItem::PushReferenceFrame(..) => "push_reference_frame",
            DisplayItem::PushStackingContext(..) => "push_stacking_context",
            DisplayItem::SetFilterOps => "set_filter_ops",
            DisplayItem::SetFilterData => "set_filter_data",
            DisplayItem::SetFilterPrimitives => "set_filter_primitives",
            DisplayItem::SetPoints => "set_points",
            DisplayItem::RadialGradient(..) => "radial_gradient",
            DisplayItem::Rectangle(..) => "rectangle",
            DisplayItem::ScrollFrame(..) => "scroll_frame",
            DisplayItem::SetGradientStops => "set_gradient_stops",
            DisplayItem::ReuseItems(..) => "reuse_item",
            DisplayItem::RetainedItems(..) => "retained_items",
            DisplayItem::StickyFrame(..) => "sticky_frame",
            DisplayItem::Text(..) => "text",
            DisplayItem::YuvImage(..) => "yuv_image",
            DisplayItem::BackdropFilter(..) => "backdrop_filter",
        }
    }
}

macro_rules! impl_default_for_enums {
    ($($enum:ident => $init:expr ),+) => {
        $(impl Default for $enum {
            #[allow(unused_imports)]
            fn default() -> Self {
                use $enum::*;
                $init
            }
        })*
    }
}

impl_default_for_enums! {
    DisplayItem => PopStackingContext,
    ScrollSensitivity => ScriptAndInputEvents,
    LineOrientation => Vertical,
    LineStyle => Solid,
    RepeatMode => Stretch,
    NinePatchBorderSource => Image(ImageKey::default()),
    BorderDetails => Normal(NormalBorder::default()),
    BorderRadiusKind => Uniform,
    BorderStyle => None,
    BoxShadowClipMode => Outset,
    ExtendMode => Clamp,
    FilterOp => Identity,
    ComponentTransferFuncType => Identity,
    ClipMode => Clip,
    FillRule => Nonzero,
    ClipId => ClipId::invalid(),
    ReferenceFrameKind => Transform {
        is_2d_scale_translation: false,
        should_snap: false,
    },
    Rotation => Degree0,
    TransformStyle => Flat,
    RasterSpace => Local(f32::default()),
    MixBlendMode => Normal,
    ImageRendering => Auto,
    AlphaType => Alpha,
    YuvColorSpace => Rec601,
    ColorRange => Limited,
    YuvData => NV12(ImageKey::default(), ImageKey::default()),
    YuvFormat => NV12,
    FilterPrimitiveInput => Original,
    ColorSpace => Srgb,
    CompositeOperator => Over
}
