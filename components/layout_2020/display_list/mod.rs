/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{OnceCell, RefCell};
use std::sync::Arc;

use app_units::Au;
use base::id::BrowsingContextId;
use base::WebRenderEpochToU16;
use embedder_traits::Cursor;
use euclid::{Point2D, SideOffsets2D, Size2D};
use fnv::FnvHashMap;
use fonts::GlyphStore;
use net_traits::image_cache::UsePlaceholder;
use servo_geometry::MaxRect;
use style::color::{AbsoluteColor, ColorSpace};
use style::computed_values::text_decoration_style::T as ComputedTextDecorationStyle;
use style::dom::OpaqueNode;
use style::properties::longhands::visibility::computed_value::T as Visibility;
use style::properties::ComputedValues;
use style::values::computed::{BorderStyle, Color, Length, LengthPercentage, OutlineStyle};
use style::values::specified::text::TextDecorationLine;
use style::values::specified::ui::CursorKind;
use style_traits::CSSPixel;
use webrender_api::{self as wr, units, BoxShadowClipMode, ClipChainId};
use webrender_traits::display_list::{
    CompositorDisplayListInfo, ScrollSensitivity, ScrollTreeNodeId,
};
use wr::units::LayoutVector2D;

use crate::context::LayoutContext;
use crate::display_list::conversions::ToWebRender;
use crate::display_list::stacking_context::StackingContextSection;
use crate::fragment_tree::{
    BackgroundMode, BoxFragment, Fragment, FragmentFlags, FragmentTree, Tag, TextFragment,
};
use crate::geom::{PhysicalPoint, PhysicalRect};
use crate::replaced::IntrinsicSizes;
use crate::style_ext::ComputedValuesExt;

mod background;
mod conversions;
mod gradient;
mod stacking_context;

use background::BackgroundPainter;
pub use stacking_context::*;

#[derive(Clone, Copy)]
pub struct WebRenderImageInfo {
    pub width: u32,
    pub height: u32,
    pub key: Option<wr::ImageKey>,
}

// webrender's `ItemTag` is private.
type ItemTag = (u64, u16);
type HitInfo = Option<ItemTag>;

/// Where the information that's used to build display lists is stored. This
/// includes both a [wr::DisplayListBuilder] for building up WebRender-specific
/// display list information and a [CompositorDisplayListInfo] used to store
/// information used by the compositor, such as a compositor-side scroll tree.
pub struct DisplayList {
    /// The [wr::DisplayListBuilder] used to collect display list items.
    pub wr: wr::DisplayListBuilder,

    /// The information about the WebRender display list that the compositor
    /// consumes. This curerntly contains the out-of-band hit testing information
    /// data structure that the compositor uses to map hit tests to information
    /// about the item hit.
    pub compositor_info: CompositorDisplayListInfo,

    /// A count of the number of SpatialTree nodes pushed to the WebRender display
    /// list. This is merely to ensure that the currently-unused SpatialTreeItemKey
    /// produced for every SpatialTree node is unique.
    pub spatial_tree_count: u64,
}

impl DisplayList {
    /// Create a new [DisplayList] given the dimensions of the layout and the WebRender
    /// pipeline id.
    pub fn new(
        viewport_size: units::LayoutSize,
        content_size: units::LayoutSize,
        pipeline_id: wr::PipelineId,
        epoch: wr::Epoch,
        root_scroll_sensitivity: ScrollSensitivity,
    ) -> Self {
        Self {
            wr: wr::DisplayListBuilder::new(pipeline_id),
            compositor_info: CompositorDisplayListInfo::new(
                viewport_size,
                content_size,
                pipeline_id,
                epoch,
                root_scroll_sensitivity,
            ),
            spatial_tree_count: 0,
        }
    }

    pub fn define_clip_chain<I>(&mut self, parent: ClipChainId, clips: I) -> ClipChainId
    where
        I: IntoIterator<Item = wr::ClipId>,
        I::IntoIter: ExactSizeIterator + Clone,
    {
        // WebRender has two different ways of expressing "no clip." ClipChainId::INVALID should be
        // used for primitives, but `None` is used for stacking contexts and clip chains. We convert
        // to the `Option<ClipChainId>` representation here. Just passing Some(ClipChainId::INVALID)
        // leads to a crash.
        let parent = match parent {
            ClipChainId::INVALID => None,
            parent => Some(parent),
        };
        self.wr.define_clip_chain(parent, clips)
    }
}

pub(crate) struct DisplayListBuilder<'a> {
    /// The current [ScrollTreeNodeId] for this [DisplayListBuilder]. This
    /// allows only passing the builder instead passing the containing
    /// [stacking_context::StackingContextContent::Fragment] as an argument to display
    /// list building functions.
    current_scroll_node_id: ScrollTreeNodeId,

    /// The current [ScrollNodeTreeId] for this [DisplayListBuilder]. This is necessary in addition
    /// to the [Self::current_scroll_node_id], because some pieces of fragments as backgrounds with
    /// `background-attachment: fixed` need to not scroll while the rest of the fragment does.
    current_reference_frame_scroll_node_id: ScrollTreeNodeId,

    /// The current [wr::ClipId] for this [DisplayListBuilder]. This allows
    /// only passing the builder instead passing the containing
    /// [stacking_context::StackingContextContent::Fragment] as an argument to display
    /// list building functions.
    current_clip_chain_id: ClipChainId,

    /// The [OpaqueNode] handle to the node used to paint the page background
    /// if the background was a canvas.
    element_for_canvas_background: OpaqueNode,

    /// A [LayoutContext] used to get information about the device pixel ratio
    /// and get handles to WebRender images.
    pub context: &'a LayoutContext<'a>,

    /// The [DisplayList] used to collect display list items and metadata.
    pub display_list: &'a mut DisplayList,

    /// A recording of the sizes of iframes encountered when building this
    /// display list. This information is forwarded to layout for the
    /// iframe so that its layout knows how large the initial containing block /
    /// viewport is.
    iframe_sizes: FnvHashMap<BrowsingContextId, Size2D<f32, CSSPixel>>,

    /// Contentful paint i.e. whether the display list contains items of type
    /// text, image, non-white canvas or SVG). Used by metrics.
    /// See <https://w3c.github.io/paint-timing/#first-contentful-paint>.
    is_contentful: bool,
}

impl DisplayList {
    pub fn build(
        &mut self,
        context: &LayoutContext,
        fragment_tree: &FragmentTree,
        root_stacking_context: &StackingContext,
    ) -> (FnvHashMap<BrowsingContextId, Size2D<f32, CSSPixel>>, bool) {
        let mut builder = DisplayListBuilder {
            current_scroll_node_id: self.compositor_info.root_reference_frame_id,
            current_reference_frame_scroll_node_id: self.compositor_info.root_reference_frame_id,
            current_clip_chain_id: ClipChainId::INVALID,
            element_for_canvas_background: fragment_tree.canvas_background.from_element,
            is_contentful: false,
            context,
            display_list: self,
            iframe_sizes: FnvHashMap::default(),
        };
        fragment_tree.build_display_list(&mut builder, root_stacking_context);
        (builder.iframe_sizes, builder.is_contentful)
    }
}

impl<'a> DisplayListBuilder<'a> {
    fn wr(&mut self) -> &mut wr::DisplayListBuilder {
        &mut self.display_list.wr
    }

    fn common_properties(
        &self,
        clip_rect: units::LayoutRect,
        style: &ComputedValues,
    ) -> wr::CommonItemProperties {
        // TODO(mrobinson): We should take advantage of this field to pass hit testing
        // information. This will allow us to avoid creating hit testing display items
        // for fragments that paint their entire border rectangle.
        wr::CommonItemProperties {
            clip_rect,
            spatial_id: self.current_scroll_node_id.spatial_id,
            clip_chain_id: self.current_clip_chain_id,
            flags: style.get_webrender_primitive_flags(),
        }
    }

    fn hit_info(
        &mut self,
        style: &ComputedValues,
        tag: Option<Tag>,
        auto_cursor: Cursor,
    ) -> HitInfo {
        use style::computed_values::pointer_events::T as PointerEvents;

        let inherited_ui = style.get_inherited_ui();
        if inherited_ui.pointer_events == PointerEvents::None {
            return None;
        }

        let hit_test_index = self.display_list.compositor_info.add_hit_test_info(
            tag?.node.0 as u64,
            Some(cursor(inherited_ui.cursor.keyword, auto_cursor)),
            self.current_scroll_node_id,
        );
        Some((
            hit_test_index as u64,
            self.display_list.compositor_info.epoch.as_u16(),
        ))
    }
}

impl Fragment {
    pub(crate) fn build_display_list(
        &self,
        builder: &mut DisplayListBuilder,
        containing_block: &PhysicalRect<Au>,
        section: StackingContextSection,
    ) {
        match self {
            Fragment::Box(b) | Fragment::Float(b) => match b.style.get_inherited_box().visibility {
                Visibility::Visible => {
                    BuilderForBoxFragment::new(b, containing_block).build(builder, section)
                },
                Visibility::Hidden => (),
                Visibility::Collapse => (),
            },
            Fragment::AbsoluteOrFixedPositioned(_) => {},
            Fragment::Positioning(positioning_fragment) => {
                if let Some(style) = positioning_fragment.style.as_ref() {
                    let rect = positioning_fragment
                        .rect
                        .to_physical(style.writing_mode, containing_block)
                        .translate(containing_block.origin.to_vector());
                    self.maybe_push_hit_test_for_style_and_tag(
                        builder,
                        style,
                        positioning_fragment.base.tag,
                        rect,
                        Cursor::Default,
                    );
                }
            },
            Fragment::Image(i) => match i.style.get_inherited_box().visibility {
                Visibility::Visible => {
                    builder.is_contentful = true;
                    let rect = i
                        .rect
                        .to_physical(i.style.writing_mode, containing_block)
                        .translate(containing_block.origin.to_vector());

                    let common = builder.common_properties(rect.to_webrender(), &i.style);
                    builder.wr().push_image(
                        &common,
                        rect.to_webrender(),
                        image_rendering(i.style.get_inherited_box().image_rendering),
                        wr::AlphaType::PremultipliedAlpha,
                        i.image_key,
                        wr::ColorF::WHITE,
                    );
                },
                Visibility::Hidden => (),
                Visibility::Collapse => (),
            },
            Fragment::IFrame(iframe) => match iframe.style.get_inherited_box().visibility {
                Visibility::Visible => {
                    builder.is_contentful = true;
                    let rect = iframe
                        .rect
                        .to_physical(iframe.style.writing_mode, containing_block)
                        .translate(containing_block.origin.to_vector());

                    builder.iframe_sizes.insert(
                        iframe.browsing_context_id,
                        Size2D::new(rect.size.width.to_f32_px(), rect.size.height.to_f32_px()),
                    );

                    let common = builder.common_properties(rect.to_webrender(), &iframe.style);
                    builder.wr().push_iframe(
                        rect.to_webrender(),
                        common.clip_rect,
                        &wr::SpaceAndClipInfo {
                            spatial_id: common.spatial_id,
                            clip_chain_id: common.clip_chain_id,
                        },
                        iframe.pipeline_id.into(),
                        true,
                    );
                },
                Visibility::Hidden => (),
                Visibility::Collapse => (),
            },
            Fragment::Text(t) => match t.parent_style.get_inherited_box().visibility {
                Visibility::Visible => {
                    self.build_display_list_for_text_fragment(t, builder, containing_block)
                },
                Visibility::Hidden => (),
                Visibility::Collapse => (),
            },
        }
    }

    fn maybe_push_hit_test_for_style_and_tag(
        &self,
        builder: &mut DisplayListBuilder,
        style: &ComputedValues,
        tag: Option<Tag>,
        rect: PhysicalRect<Au>,
        cursor: Cursor,
    ) {
        let hit_info = builder.hit_info(style, tag, cursor);
        let hit_info = match hit_info {
            Some(hit_info) => hit_info,
            None => return,
        };

        let clip_chain_id = builder.current_clip_chain_id;
        let spatial_id = builder.current_scroll_node_id.spatial_id;
        builder.wr().push_hit_test(
            rect.to_webrender(),
            clip_chain_id,
            spatial_id,
            style.get_webrender_primitive_flags(),
            hit_info,
        );
    }

    fn build_display_list_for_text_fragment(
        &self,
        fragment: &TextFragment,
        builder: &mut DisplayListBuilder,
        containing_block: &PhysicalRect<Au>,
    ) {
        // NB: The order of painting text components (CSS Text Decoration Module Level 3) is:
        // shadows, underline, overline, text, text-emphasis, and then line-through.

        builder.is_contentful = true;

        let rect = fragment
            .rect
            .to_physical(fragment.parent_style.writing_mode, containing_block)
            .translate(containing_block.origin.to_vector());
        let mut baseline_origin = rect.origin;
        baseline_origin.y += fragment.font_metrics.ascent;
        let glyphs = glyphs(
            &fragment.glyphs,
            baseline_origin,
            fragment.justification_adjustment,
        );
        if glyphs.is_empty() {
            return;
        }

        self.maybe_push_hit_test_for_style_and_tag(
            builder,
            &fragment.parent_style,
            fragment.base.tag,
            rect,
            Cursor::Text,
        );

        let color = fragment.parent_style.clone_color();
        let font_metrics = &fragment.font_metrics;
        let dppx = builder.context.style_context.device_pixel_ratio().get();
        let common = builder.common_properties(rect.to_webrender(), &fragment.parent_style);

        // Shadows. According to CSS-BACKGROUNDS, text shadows render in *reverse* order (front to
        // back).
        let shadows = &fragment.parent_style.get_inherited_text().text_shadow;
        for shadow in shadows.0.iter().rev() {
            builder.wr().push_shadow(
                &wr::SpaceAndClipInfo {
                    spatial_id: common.spatial_id,
                    clip_chain_id: common.clip_chain_id,
                },
                wr::Shadow {
                    offset: LayoutVector2D::new(shadow.horizontal.px(), shadow.vertical.px()),
                    color: rgba(shadow.color.resolve_to_absolute(&color)),
                    blur_radius: shadow.blur.px(),
                },
                true, /* should_inflate */
            );
        }

        // Underline.
        if fragment
            .text_decoration_line
            .contains(TextDecorationLine::UNDERLINE)
        {
            let mut rect = rect;
            rect.origin.y += font_metrics.ascent - font_metrics.underline_offset;
            rect.size.height = Au::from_f32_px(font_metrics.underline_size.to_nearest_pixel(dppx));
            self.build_display_list_for_text_decoration(fragment, builder, &rect, &color);
        }

        // Overline.
        if fragment
            .text_decoration_line
            .contains(TextDecorationLine::OVERLINE)
        {
            let mut rect = rect;
            rect.size.height = Au::from_f32_px(font_metrics.underline_size.to_nearest_pixel(dppx));
            self.build_display_list_for_text_decoration(fragment, builder, &rect, &color);
        }

        // Text.
        builder.wr().push_text(
            &common,
            rect.to_webrender(),
            &glyphs,
            fragment.font_key,
            rgba(color),
            None,
        );

        // Line-through.
        if fragment
            .text_decoration_line
            .contains(TextDecorationLine::LINE_THROUGH)
        {
            let mut rect = rect;
            rect.origin.y += font_metrics.ascent - font_metrics.strikeout_offset;
            rect.size.height = Au::from_f32_px(font_metrics.strikeout_size.to_nearest_pixel(dppx));
            self.build_display_list_for_text_decoration(fragment, builder, &rect, &color);
        }

        if !shadows.0.is_empty() {
            builder.wr().pop_all_shadows();
        }
    }

    fn build_display_list_for_text_decoration(
        &self,
        fragment: &TextFragment,
        builder: &mut DisplayListBuilder,
        rect: &PhysicalRect<Au>,
        color: &AbsoluteColor,
    ) {
        let rect = rect.to_webrender();
        let wavy_line_thickness = (0.33 * rect.size().height).ceil();
        let text_decoration_color = fragment
            .parent_style
            .clone_text_decoration_color()
            .resolve_to_absolute(color);
        let text_decoration_style = fragment.parent_style.clone_text_decoration_style();
        if text_decoration_style == ComputedTextDecorationStyle::MozNone {
            return;
        }
        builder.display_list.wr.push_line(
            &builder.common_properties(rect, &fragment.parent_style),
            &rect,
            wavy_line_thickness,
            wr::LineOrientation::Horizontal,
            &rgba(text_decoration_color),
            text_decoration_style.to_webrender(),
        );
        // XXX(ferjm) support text-decoration-style: double
    }
}

struct BuilderForBoxFragment<'a> {
    fragment: &'a BoxFragment,
    containing_block: &'a PhysicalRect<Au>,
    border_rect: units::LayoutRect,
    padding_rect: OnceCell<units::LayoutRect>,
    content_rect: OnceCell<units::LayoutRect>,
    border_radius: wr::BorderRadius,
    border_edge_clip_chain_id: RefCell<Option<ClipChainId>>,
    padding_edge_clip_chain_id: RefCell<Option<ClipChainId>>,
    content_edge_clip_chain_id: RefCell<Option<ClipChainId>>,
}

impl<'a> BuilderForBoxFragment<'a> {
    fn new(fragment: &'a BoxFragment, containing_block: &'a PhysicalRect<Au>) -> Self {
        let border_rect: units::LayoutRect = fragment
            .border_rect()
            .to_physical(fragment.style.writing_mode, containing_block)
            .translate(containing_block.origin.to_vector())
            .to_webrender();

        let border_radius = {
            let resolve = |radius: &LengthPercentage, box_size: f32| {
                radius.percentage_relative_to(Length::new(box_size)).px()
            };
            let corner = |corner: &style::values::computed::BorderCornerRadius| {
                Size2D::new(
                    resolve(&corner.0.width.0, border_rect.size().width),
                    resolve(&corner.0.height.0, border_rect.size().height),
                )
            };
            let b = fragment.style.get_border();
            let mut radius = wr::BorderRadius {
                top_left: corner(&b.border_top_left_radius),
                top_right: corner(&b.border_top_right_radius),
                bottom_right: corner(&b.border_bottom_right_radius),
                bottom_left: corner(&b.border_bottom_left_radius),
            };
            // Normalize radii that add up to > 100%.
            // https://www.w3.org/TR/css-backgrounds-3/#corner-overlap
            // > Let f = min(L_i/S_i), where i ∈ {top, right, bottom, left},
            // > S_i is the sum of the two corresponding radii of the corners on side i,
            // > and L_top = L_bottom = the width of the box,
            // > and L_left = L_right = the height of the box.
            // > If f < 1, then all corner radii are reduced by multiplying them by f.
            let f = (border_rect.width() / (radius.top_left.width + radius.top_right.width))
                .min(border_rect.width() / (radius.bottom_left.width + radius.bottom_right.width))
                .min(border_rect.height() / (radius.top_left.height + radius.bottom_left.height))
                .min(border_rect.height() / (radius.top_right.height + radius.bottom_right.height));
            if f < 1.0 {
                radius.top_left *= f;
                radius.top_right *= f;
                radius.bottom_right *= f;
                radius.bottom_left *= f;
            }
            radius
        };

        Self {
            fragment,
            containing_block,
            border_rect,
            border_radius,
            padding_rect: OnceCell::new(),
            content_rect: OnceCell::new(),
            border_edge_clip_chain_id: RefCell::new(None),
            padding_edge_clip_chain_id: RefCell::new(None),
            content_edge_clip_chain_id: RefCell::new(None),
        }
    }

    fn content_rect(&self) -> &units::LayoutRect {
        self.content_rect.get_or_init(|| {
            self.fragment
                .content_rect
                .to_physical(self.fragment.style.writing_mode, self.containing_block)
                .translate(self.containing_block.origin.to_vector())
                .to_webrender()
        })
    }

    fn padding_rect(&self) -> &units::LayoutRect {
        self.padding_rect.get_or_init(|| {
            self.fragment
                .padding_rect()
                .to_physical(self.fragment.style.writing_mode, self.containing_block)
                .translate(self.containing_block.origin.to_vector())
                .to_webrender()
        })
    }

    fn border_edge_clip(
        &self,
        builder: &mut DisplayListBuilder,
        force_clip_creation: bool,
    ) -> Option<ClipChainId> {
        if let Some(clip) = *self.border_edge_clip_chain_id.borrow() {
            return Some(clip);
        }

        let maybe_clip = create_clip_chain(
            self.border_radius,
            self.border_rect,
            builder,
            force_clip_creation,
        );
        *self.border_edge_clip_chain_id.borrow_mut() = maybe_clip;
        maybe_clip
    }

    fn padding_edge_clip(
        &self,
        builder: &mut DisplayListBuilder,
        force_clip_creation: bool,
    ) -> Option<ClipChainId> {
        if let Some(clip) = *self.padding_edge_clip_chain_id.borrow() {
            return Some(clip);
        }

        let radii = inner_radii(
            self.border_radius,
            self.fragment
                .border
                .to_physical(self.fragment.style.writing_mode)
                .to_webrender(),
        );
        let maybe_clip =
            create_clip_chain(radii, *self.padding_rect(), builder, force_clip_creation);
        *self.padding_edge_clip_chain_id.borrow_mut() = maybe_clip;
        maybe_clip
    }

    fn content_edge_clip(
        &self,
        builder: &mut DisplayListBuilder,
        force_clip_creation: bool,
    ) -> Option<ClipChainId> {
        if let Some(clip) = *self.content_edge_clip_chain_id.borrow() {
            return Some(clip);
        }

        let radii = inner_radii(
            self.border_radius,
            (self.fragment.border + self.fragment.padding)
                .to_physical(self.fragment.style.writing_mode)
                .to_webrender(),
        );
        let maybe_clip =
            create_clip_chain(radii, *self.content_rect(), builder, force_clip_creation);
        *self.content_edge_clip_chain_id.borrow_mut() = maybe_clip;
        maybe_clip
    }

    fn build(&mut self, builder: &mut DisplayListBuilder, section: StackingContextSection) {
        if section == StackingContextSection::Outline {
            self.build_outline(builder);
        } else {
            self.build_hit_test(builder);
            if self
                .fragment
                .base
                .flags
                .contains(FragmentFlags::DO_NOT_PAINT)
            {
                return;
            }
            self.build_background(builder);
            self.build_box_shadow(builder);
            self.build_border(builder);
        }
    }

    fn build_hit_test(&self, builder: &mut DisplayListBuilder) {
        let hit_info = builder.hit_info(
            &self.fragment.style,
            self.fragment.base.tag,
            Cursor::Default,
        );
        let hit_info = match hit_info {
            Some(hit_info) => hit_info,
            None => return,
        };

        let mut common = builder.common_properties(self.border_rect, &self.fragment.style);
        if let Some(clip_chain_id) = self.border_edge_clip(builder, false) {
            common.clip_chain_id = clip_chain_id;
        }
        builder.wr().push_hit_test(
            common.clip_rect,
            common.clip_chain_id,
            common.spatial_id,
            common.flags,
            hit_info,
        );
    }

    fn build_background_for_painter(
        &mut self,
        builder: &mut DisplayListBuilder,
        painter: &BackgroundPainter,
    ) {
        let b = painter.style.get_background();
        let background_color = painter.style.resolve_color(b.background_color.clone());
        if background_color.alpha > 0.0 {
            // https://drafts.csswg.org/css-backgrounds/#background-color
            // “The background color is clipped according to the background-clip
            //  value associated with the bottom-most background image layer.”
            let layer_index = b.background_image.0.len() - 1;
            let bounds = painter.painting_area(self, builder, layer_index);
            let common = painter.common_properties(self, builder, layer_index, bounds);
            builder
                .wr()
                .push_rect(&common, bounds, rgba(background_color))
        }

        self.build_background_image(builder, painter);
    }

    fn build_background(&mut self, builder: &mut DisplayListBuilder) {
        if self
            .fragment
            .base
            .is_for_node(builder.element_for_canvas_background)
        {
            // This background is already painted for the canvas, don’t paint it again here.
            return;
        }

        // If this BoxFragment does not paint a background, do nothing.
        if let BackgroundMode::None = self.fragment.background_mode {
            return;
        }

        // Paint all extra backgrounds for this BoxFragment. These are painted first, as that's
        // the order that they are expected to be painted for table cells (where this feature
        // is used).
        if let BackgroundMode::Extra(ref extra_backgrounds) = self.fragment.background_mode {
            for extra_background in extra_backgrounds {
                let positioning_area = extra_background.rect;
                let painter = BackgroundPainter {
                    style: &extra_background.style,
                    painting_area_override: None,
                    positioning_area_override: Some(
                        positioning_area
                            .to_physical(self.fragment.style.writing_mode, self.containing_block)
                            .translate(self.containing_block.origin.to_vector())
                            .to_webrender(),
                    ),
                };
                self.build_background_for_painter(builder, &painter);
            }
        }

        let painter = BackgroundPainter {
            style: &self.fragment.style,
            painting_area_override: None,
            positioning_area_override: None,
        };
        self.build_background_for_painter(builder, &painter);
    }

    fn build_background_image(
        &mut self,
        builder: &mut DisplayListBuilder,
        painter: &BackgroundPainter,
    ) {
        use style::values::computed::image::Image;
        let style = painter.style;
        let b = style.get_background();
        // Reverse because the property is top layer first, we want to paint bottom layer first.
        for (index, image) in b.background_image.0.iter().enumerate().rev() {
            match image {
                Image::None => {},
                Image::Gradient(ref gradient) => {
                    let intrinsic = IntrinsicSizes::empty();
                    if let Some(layer) =
                        &background::layout_layer(self, painter, builder, index, intrinsic)
                    {
                        gradient::build(style, gradient, layer, builder)
                    }
                },
                Image::Url(ref image_url) => {
                    // FIXME: images won’t always have in intrinsic width or
                    // height when support for SVG is added, or a WebRender
                    // `ImageKey`, for that matter.
                    //
                    // FIXME: It feels like this should take into account the pseudo
                    // element and not just the node.
                    let node = match self.fragment.base.tag {
                        Some(tag) => tag.node,
                        None => continue,
                    };
                    let image_url = match image_url.url() {
                        Some(url) => url.clone(),
                        None => continue,
                    };
                    let (width, height, key) = match builder.context.get_webrender_image_for_url(
                        node,
                        image_url.into(),
                        UsePlaceholder::No,
                    ) {
                        Some(WebRenderImageInfo {
                            width,
                            height,
                            key: Some(key),
                        }) => (width, height, key),
                        _ => continue,
                    };

                    // FIXME: https://drafts.csswg.org/css-images-4/#the-image-resolution
                    let dppx = 1.0;
                    let intrinsic = IntrinsicSizes::from_width_and_height(
                        width as f32 / dppx,
                        height as f32 / dppx,
                    );

                    if let Some(layer) =
                        background::layout_layer(self, painter, builder, index, intrinsic)
                    {
                        let image_rendering = image_rendering(style.clone_image_rendering());
                        if layer.repeat {
                            builder.wr().push_repeating_image(
                                &layer.common,
                                layer.bounds,
                                layer.tile_size,
                                layer.tile_spacing,
                                image_rendering,
                                wr::AlphaType::PremultipliedAlpha,
                                key,
                                wr::ColorF::WHITE,
                            )
                        } else {
                            builder.wr().push_image(
                                &layer.common,
                                layer.bounds,
                                image_rendering,
                                wr::AlphaType::PremultipliedAlpha,
                                key,
                                wr::ColorF::WHITE,
                            )
                        }
                    }
                },
                Image::PaintWorklet(_) => {
                    // TODO: Add support for PaintWorklet rendering.
                },
                Image::ImageSet(..) | Image::CrossFade(..) => {
                    unreachable!("Shouldn't be parsed on Servo for now")
                },
            }
        }
    }

    fn build_border_side(&mut self, style: BorderStyle, color: Color) -> wr::BorderSide {
        wr::BorderSide {
            color: rgba(self.fragment.style.resolve_color(color)),
            style: match style {
                BorderStyle::None => wr::BorderStyle::None,
                BorderStyle::Solid => wr::BorderStyle::Solid,
                BorderStyle::Double => wr::BorderStyle::Double,
                BorderStyle::Dotted => wr::BorderStyle::Dotted,
                BorderStyle::Dashed => wr::BorderStyle::Dashed,
                BorderStyle::Hidden => wr::BorderStyle::Hidden,
                BorderStyle::Groove => wr::BorderStyle::Groove,
                BorderStyle::Ridge => wr::BorderStyle::Ridge,
                BorderStyle::Inset => wr::BorderStyle::Inset,
                BorderStyle::Outset => wr::BorderStyle::Outset,
            },
        }
    }

    fn build_border(&mut self, builder: &mut DisplayListBuilder) {
        let border = self.fragment.style.get_border();
        let border_widths = self
            .fragment
            .border
            .to_physical(self.fragment.style.writing_mode)
            .to_webrender();

        if border_widths == SideOffsets2D::zero() {
            return;
        }

        let common = builder.common_properties(self.border_rect, &self.fragment.style);
        let details = wr::BorderDetails::Normal(wr::NormalBorder {
            top: self.build_border_side(border.border_top_style, border.border_top_color.clone()),
            right: self
                .build_border_side(border.border_right_style, border.border_right_color.clone()),
            bottom: self.build_border_side(
                border.border_bottom_style,
                border.border_bottom_color.clone(),
            ),
            left: self
                .build_border_side(border.border_left_style, border.border_left_color.clone()),
            radius: self.border_radius,
            do_aa: true,
        });
        builder
            .wr()
            .push_border(&common, self.border_rect, border_widths, details)
    }

    fn build_outline(&mut self, builder: &mut DisplayListBuilder) {
        let outline = self.fragment.style.get_outline();
        let width = outline.outline_width.to_f32_px();
        if width == 0.0 {
            return;
        }
        let offset = outline
            .outline_offset
            .px()
            .max(-self.border_rect.width() / 2.0)
            .max(-self.border_rect.height() / 2.0) +
            width;
        let outline_rect = self.border_rect.inflate(offset, offset);
        let common = builder.common_properties(outline_rect, &self.fragment.style);
        let widths = SideOffsets2D::new_all_same(width);
        let style = match outline.outline_style {
            // TODO: treating 'auto' as 'solid' is allowed by the spec,
            // but we should do something better.
            OutlineStyle::Auto => BorderStyle::Solid,
            OutlineStyle::BorderStyle(s) => s,
        };
        let side = self.build_border_side(style, outline.outline_color.clone());
        let details = wr::BorderDetails::Normal(wr::NormalBorder {
            top: side,
            right: side,
            bottom: side,
            left: side,
            radius: offset_radii(self.border_radius, offset),
            do_aa: true,
        });
        builder
            .wr()
            .push_border(&common, outline_rect, widths, details)
    }

    fn build_box_shadow(&self, builder: &mut DisplayListBuilder<'_>) {
        let box_shadows = &self.fragment.style.get_effects().box_shadow.0;
        if box_shadows.is_empty() {
            return;
        }

        // NB: According to CSS-BACKGROUNDS, box shadows render in *reverse* order (front to back).
        let border_rect = self.border_rect;
        let common = builder.common_properties(MaxRect::max_rect(), &self.fragment.style);
        for box_shadow in box_shadows.iter().rev() {
            let clip_mode = if box_shadow.inset {
                BoxShadowClipMode::Inset
            } else {
                BoxShadowClipMode::Outset
            };

            builder.wr().push_box_shadow(
                &common,
                border_rect,
                LayoutVector2D::new(
                    box_shadow.base.horizontal.px(),
                    box_shadow.base.vertical.px(),
                ),
                rgba(
                    self.fragment
                        .style
                        .resolve_color(box_shadow.base.color.clone()),
                ),
                box_shadow.base.blur.px(),
                box_shadow.spread.px(),
                self.border_radius,
                clip_mode,
            );
        }
    }
}

fn rgba(color: AbsoluteColor) -> wr::ColorF {
    let rgba = color.to_color_space(ColorSpace::Srgb);
    wr::ColorF::new(
        rgba.components.0.clamp(0.0, 1.0),
        rgba.components.1.clamp(0.0, 1.0),
        rgba.components.2.clamp(0.0, 1.0),
        rgba.alpha,
    )
}

fn glyphs(
    glyph_runs: &[Arc<GlyphStore>],
    mut baseline_origin: PhysicalPoint<Au>,
    justification_adjustment: Au,
) -> Vec<wr::GlyphInstance> {
    use fonts_traits::ByteIndex;
    use range::Range;

    let mut glyphs = vec![];
    for run in glyph_runs {
        for glyph in run.iter_glyphs_for_byte_range(&Range::new(ByteIndex(0), run.len())) {
            if !run.is_whitespace() {
                let glyph_offset = glyph.offset().unwrap_or(Point2D::zero());
                let point = units::LayoutPoint::new(
                    baseline_origin.x.to_f32_px() + glyph_offset.x.to_f32_px(),
                    baseline_origin.y.to_f32_px() + glyph_offset.y.to_f32_px(),
                );
                let glyph = wr::GlyphInstance {
                    index: glyph.id(),
                    point,
                };
                glyphs.push(glyph);
            }

            if glyph.char_is_word_separator() {
                baseline_origin.x += justification_adjustment;
            }
            baseline_origin.x += glyph.advance();
        }
    }
    glyphs
}

fn cursor(kind: CursorKind, auto_cursor: Cursor) -> Cursor {
    match kind {
        CursorKind::Auto => auto_cursor,
        CursorKind::None => Cursor::None,
        CursorKind::Default => Cursor::Default,
        CursorKind::Pointer => Cursor::Pointer,
        CursorKind::ContextMenu => Cursor::ContextMenu,
        CursorKind::Help => Cursor::Help,
        CursorKind::Progress => Cursor::Progress,
        CursorKind::Wait => Cursor::Wait,
        CursorKind::Cell => Cursor::Cell,
        CursorKind::Crosshair => Cursor::Crosshair,
        CursorKind::Text => Cursor::Text,
        CursorKind::VerticalText => Cursor::VerticalText,
        CursorKind::Alias => Cursor::Alias,
        CursorKind::Copy => Cursor::Copy,
        CursorKind::Move => Cursor::Move,
        CursorKind::NoDrop => Cursor::NoDrop,
        CursorKind::NotAllowed => Cursor::NotAllowed,
        CursorKind::Grab => Cursor::Grab,
        CursorKind::Grabbing => Cursor::Grabbing,
        CursorKind::EResize => Cursor::EResize,
        CursorKind::NResize => Cursor::NResize,
        CursorKind::NeResize => Cursor::NeResize,
        CursorKind::NwResize => Cursor::NwResize,
        CursorKind::SResize => Cursor::SResize,
        CursorKind::SeResize => Cursor::SeResize,
        CursorKind::SwResize => Cursor::SwResize,
        CursorKind::WResize => Cursor::WResize,
        CursorKind::EwResize => Cursor::EwResize,
        CursorKind::NsResize => Cursor::NsResize,
        CursorKind::NeswResize => Cursor::NeswResize,
        CursorKind::NwseResize => Cursor::NwseResize,
        CursorKind::ColResize => Cursor::ColResize,
        CursorKind::RowResize => Cursor::RowResize,
        CursorKind::AllScroll => Cursor::AllScroll,
        CursorKind::ZoomIn => Cursor::ZoomIn,
        CursorKind::ZoomOut => Cursor::ZoomOut,
    }
}

fn image_rendering(ir: style::computed_values::image_rendering::T) -> wr::ImageRendering {
    use style::computed_values::image_rendering::T as ImageRendering;
    match ir {
        ImageRendering::Auto => wr::ImageRendering::Auto,
        ImageRendering::CrispEdges => wr::ImageRendering::CrispEdges,
        ImageRendering::Pixelated => wr::ImageRendering::Pixelated,
    }
}

/// Radii for the padding edge or content edge
fn inner_radii(mut radii: wr::BorderRadius, insets: units::LayoutSideOffsets) -> wr::BorderRadius {
    assert!(insets.left >= 0.0, "left inset must not be negative");
    radii.top_left.width -= insets.left;
    radii.bottom_left.width -= insets.left;

    assert!(insets.right >= 0.0, "left inset must not be negative");
    radii.top_right.width -= insets.right;
    radii.bottom_right.width -= insets.right;

    assert!(insets.top >= 0.0, "top inset must not be negative");
    radii.top_left.height -= insets.top;
    radii.top_right.height -= insets.top;

    assert!(insets.bottom >= 0.0, "bottom inset must not be negative");
    radii.bottom_left.height -= insets.bottom;
    radii.bottom_right.height -= insets.bottom;
    radii
}

fn offset_radii(mut radii: wr::BorderRadius, offset: f32) -> wr::BorderRadius {
    if offset == 0.0 {
        return radii;
    }
    if offset < 0.0 {
        return inner_radii(radii, units::LayoutSideOffsets::new_all_same(-offset));
    }
    let expand = |radius: &mut f32| {
        // Expand the radius by the specified amount, but keeping sharp corners.
        // TODO: this behavior is not continuous, it's being discussed in the CSSWG:
        // https://github.com/w3c/csswg-drafts/issues/7103
        if *radius > 0.0 {
            *radius += offset;
        }
    };
    expand(&mut radii.top_left.width);
    expand(&mut radii.top_left.height);
    expand(&mut radii.top_right.width);
    expand(&mut radii.top_right.height);
    expand(&mut radii.bottom_right.width);
    expand(&mut radii.bottom_right.height);
    expand(&mut radii.bottom_left.width);
    expand(&mut radii.bottom_left.height);
    radii
}

fn create_clip_chain(
    radii: wr::BorderRadius,
    rect: units::LayoutRect,
    builder: &mut DisplayListBuilder,
    force_clip_creation: bool,
) -> Option<ClipChainId> {
    if radii.is_zero() && !force_clip_creation {
        return None;
    }

    let spatial_id = builder.current_scroll_node_id.spatial_id;
    let parent_clip_chain_id = builder.current_clip_chain_id;
    let new_clip_id = if radii.is_zero() {
        builder.wr().define_clip_rect(spatial_id, rect)
    } else {
        builder.wr().define_clip_rounded_rect(
            spatial_id,
            wr::ComplexClipRegion {
                rect,
                radii,
                mode: wr::ClipMode::Clip,
            },
        )
    };

    Some(
        builder
            .display_list
            .define_clip_chain(parent_clip_chain_id, [new_clip_id]),
    )
}
