/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{OnceCell, RefCell};
use std::sync::Arc;

use app_units::{AU_PER_PX, Au};
use base::WebRenderEpochToU16;
use base::id::ScrollTreeNodeId;
use embedder_traits::Cursor;
use euclid::{Point2D, SideOffsets2D, Size2D, UnknownUnit};
use fonts::GlyphStore;
use gradient::WebRenderGradient;
use range::Range as ServoRange;
use servo_geometry::MaxRect;
use style::Zero;
use style::color::{AbsoluteColor, ColorSpace};
use style::computed_values::border_image_outset::T as BorderImageOutset;
use style::computed_values::text_decoration_style::T as ComputedTextDecorationStyle;
use style::dom::OpaqueNode;
use style::properties::ComputedValues;
use style::properties::longhands::visibility::computed_value::T as Visibility;
use style::properties::style_structs::Border;
use style::values::computed::{
    BorderImageSideWidth, BorderImageWidth, BorderStyle, LengthPercentage,
    NonNegativeLengthOrNumber, NumberOrPercentage, OutlineStyle,
};
use style::values::generics::NonNegative;
use style::values::generics::rect::Rect;
use style::values::specified::text::TextDecorationLine;
use style::values::specified::ui::CursorKind;
use webrender_api::units::{DevicePixel, LayoutPixel, LayoutRect, LayoutSize};
use webrender_api::{
    self as wr, BorderDetails, BoxShadowClipMode, ClipChainId, CommonItemProperties,
    ImageRendering, NinePatchBorder, NinePatchBorderSource, units,
};
use webrender_traits::display_list::{AxesScrollSensitivity, CompositorDisplayListInfo};
use wr::units::LayoutVector2D;

use crate::context::{LayoutContext, ResolvedImage};
use crate::display_list::conversions::ToWebRender;
use crate::display_list::stacking_context::StackingContextSection;
use crate::fragment_tree::{
    BackgroundMode, BoxFragment, Fragment, FragmentFlags, FragmentTree, SpecificLayoutInfo, Tag,
    TextFragment,
};
use crate::geom::{
    LengthPercentageOrAuto, PhysicalPoint, PhysicalRect, PhysicalSides, PhysicalSize,
};
use crate::replaced::NaturalSizes;
use crate::style_ext::{BorderStyleColor, ComputedValuesExt};

mod background;
mod clip_path;
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
const INSERTION_POINT_LOGICAL_WIDTH: Au = Au(AU_PER_PX);

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
        viewport_scroll_sensitivity: AxesScrollSensitivity,
        first_reflow: bool,
    ) -> Self {
        Self {
            wr: wr::DisplayListBuilder::new(pipeline_id),
            compositor_info: CompositorDisplayListInfo::new(
                viewport_size,
                content_size,
                pipeline_id,
                epoch,
                viewport_scroll_sensitivity,
                first_reflow,
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

    /// The current [ScrollTreeNodeId] for this [DisplayListBuilder]. This is necessary in addition
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
}

impl DisplayList {
    /// Build the display list, returning true if it was contentful.
    pub fn build(
        &mut self,
        context: &LayoutContext,
        fragment_tree: &FragmentTree,
        root_stacking_context: &StackingContext,
    ) {
        #[cfg(feature = "tracing")]
        let _span = tracing::trace_span!("display_list::build", servo_profiling = true).entered();
        let mut builder = DisplayListBuilder {
            current_scroll_node_id: self.compositor_info.root_reference_frame_id,
            current_reference_frame_scroll_node_id: self.compositor_info.root_reference_frame_id,
            current_clip_chain_id: ClipChainId::INVALID,
            element_for_canvas_background: fragment_tree.canvas_background.from_element,
            context,
            display_list: self,
        };
        fragment_tree.build_display_list(&mut builder, root_stacking_context);
    }
}

impl DisplayListBuilder<'_> {
    fn wr(&mut self) -> &mut wr::DisplayListBuilder {
        &mut self.display_list.wr
    }

    fn mark_is_contentful(&mut self) {
        self.display_list.compositor_info.is_contentful = true;
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
        is_hit_test_for_scrollable_overflow: bool,
        is_collapsed_table_borders: bool,
    ) {
        match self {
            Fragment::Box(box_fragment) | Fragment::Float(box_fragment) => {
                let box_fragment = &*box_fragment.borrow();
                match box_fragment.style.get_inherited_box().visibility {
                    Visibility::Visible => BuilderForBoxFragment::new(
                        box_fragment,
                        containing_block,
                        is_hit_test_for_scrollable_overflow,
                        is_collapsed_table_borders,
                    )
                    .build(builder, section),
                    Visibility::Hidden => (),
                    Visibility::Collapse => (),
                }
            },
            Fragment::AbsoluteOrFixedPositioned(_) => {},
            Fragment::Positioning(positioning_fragment) => {
                let positioning_fragment = positioning_fragment.borrow();
                if let Some(style) = positioning_fragment.style.as_ref() {
                    let rect = positioning_fragment
                        .rect
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
            Fragment::Image(image) => {
                let image = image.borrow();
                match image.style.get_inherited_box().visibility {
                    Visibility::Visible => {
                        builder.mark_is_contentful();

                        let image_rendering = image
                            .style
                            .get_inherited_box()
                            .image_rendering
                            .to_webrender();
                        let rect = image
                            .rect
                            .translate(containing_block.origin.to_vector())
                            .to_webrender();
                        let clip = image
                            .clip
                            .translate(containing_block.origin.to_vector())
                            .to_webrender();
                        let common = builder.common_properties(clip, &image.style);

                        if let Some(image_key) = image.image_key {
                            builder.wr().push_image(
                                &common,
                                rect,
                                image_rendering,
                                wr::AlphaType::PremultipliedAlpha,
                                image_key,
                                wr::ColorF::WHITE,
                            );
                        }
                    },
                    Visibility::Hidden => (),
                    Visibility::Collapse => (),
                }
            },
            Fragment::IFrame(iframe) => {
                let iframe = iframe.borrow();
                match iframe.style.get_inherited_box().visibility {
                    Visibility::Visible => {
                        builder.mark_is_contentful();
                        let rect = iframe.rect.translate(containing_block.origin.to_vector());

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
                }
            },
            Fragment::Text(text) => {
                let text = &*text.borrow();
                match text.parent_style.get_inherited_box().visibility {
                    Visibility::Visible => {
                        self.build_display_list_for_text_fragment(text, builder, containing_block)
                    },
                    Visibility::Hidden => (),
                    Visibility::Collapse => (),
                }
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

        builder.mark_is_contentful();

        let rect = fragment.rect.translate(containing_block.origin.to_vector());
        let mut baseline_origin = rect.origin;
        baseline_origin.y += fragment.font_metrics.ascent;
        let glyphs = glyphs(
            &fragment.glyphs,
            baseline_origin,
            fragment.justification_adjustment,
            !fragment.has_selection(),
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

        if fragment
            .text_decoration_line
            .contains(TextDecorationLine::UNDERLINE)
        {
            let mut rect = rect;
            rect.origin.y += font_metrics.ascent - font_metrics.underline_offset;
            rect.size.height = Au::from_f32_px(font_metrics.underline_size.to_nearest_pixel(dppx));
            self.build_display_list_for_text_decoration(fragment, builder, &rect, &color);
        }

        if fragment
            .text_decoration_line
            .contains(TextDecorationLine::OVERLINE)
        {
            let mut rect = rect;
            rect.size.height = Au::from_f32_px(font_metrics.underline_size.to_nearest_pixel(dppx));
            self.build_display_list_for_text_decoration(fragment, builder, &rect, &color);
        }

        // TODO: This caret/text selection implementation currently does not account for vertical text
        // and RTL text properly.
        if let Some(range) = fragment.selection_range {
            let baseline_origin = rect.origin;
            if !range.is_empty() {
                let start = glyphs_advance_by_index(
                    &fragment.glyphs,
                    range.begin(),
                    baseline_origin,
                    fragment.justification_adjustment,
                );

                let end = glyphs_advance_by_index(
                    &fragment.glyphs,
                    range.end(),
                    baseline_origin,
                    fragment.justification_adjustment,
                );

                let selection_rect = LayoutRect::new(
                    Point2D::new(start.x.to_f32_px(), containing_block.min_y().to_f32_px()),
                    Point2D::new(end.x.to_f32_px(), containing_block.max_y().to_f32_px()),
                );
                if let Some(selection_color) = fragment
                    .selected_style
                    .clone_background_color()
                    .as_absolute()
                {
                    let selection_common =
                        builder.common_properties(selection_rect, &fragment.parent_style);
                    builder.wr().push_rect(
                        &selection_common,
                        selection_rect,
                        rgba(*selection_color),
                    );
                }
            } else {
                let insertion_point = glyphs_advance_by_index(
                    &fragment.glyphs,
                    range.begin(),
                    baseline_origin,
                    fragment.justification_adjustment,
                );

                let insertion_point_rect = LayoutRect::new(
                    Point2D::new(
                        insertion_point.x.to_f32_px(),
                        containing_block.min_y().to_f32_px(),
                    ),
                    Point2D::new(
                        insertion_point.x.to_f32_px() + INSERTION_POINT_LOGICAL_WIDTH.to_f32_px(),
                        containing_block.max_y().to_f32_px(),
                    ),
                );
                let insertion_point_common =
                    builder.common_properties(insertion_point_rect, &fragment.parent_style);
                builder
                    .wr()
                    .push_rect(&insertion_point_common, insertion_point_rect, rgba(color));
            }
        }

        builder.wr().push_text(
            &common,
            rect.to_webrender(),
            &glyphs,
            fragment.font_key,
            rgba(color),
            None,
        );

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
    margin_rect: OnceCell<units::LayoutRect>,
    padding_rect: OnceCell<units::LayoutRect>,
    content_rect: OnceCell<units::LayoutRect>,
    border_radius: wr::BorderRadius,
    border_edge_clip_chain_id: RefCell<Option<ClipChainId>>,
    padding_edge_clip_chain_id: RefCell<Option<ClipChainId>>,
    content_edge_clip_chain_id: RefCell<Option<ClipChainId>>,
    is_hit_test_for_scrollable_overflow: bool,
    is_collapsed_table_borders: bool,
}

impl<'a> BuilderForBoxFragment<'a> {
    fn new(
        fragment: &'a BoxFragment,
        containing_block: &'a PhysicalRect<Au>,
        is_hit_test_for_scrollable_overflow: bool,
        is_collapsed_table_borders: bool,
    ) -> Self {
        let border_rect = fragment
            .border_rect()
            .translate(containing_block.origin.to_vector());

        let webrender_border_rect = border_rect.to_webrender();
        let border_radius = {
            let resolve = |radius: &LengthPercentage, box_size: Au| {
                radius.to_used_value(box_size).to_f32_px()
            };
            let corner = |corner: &style::values::computed::BorderCornerRadius| {
                Size2D::new(
                    resolve(&corner.0.width.0, border_rect.size.width),
                    resolve(&corner.0.height.0, border_rect.size.height),
                )
            };
            let b = fragment.style.get_border();
            let mut radius = wr::BorderRadius {
                top_left: corner(&b.border_top_left_radius),
                top_right: corner(&b.border_top_right_radius),
                bottom_right: corner(&b.border_bottom_right_radius),
                bottom_left: corner(&b.border_bottom_left_radius),
            };

            normalize_radii(&webrender_border_rect, &mut radius);
            radius
        };

        Self {
            fragment,
            containing_block,
            border_rect: webrender_border_rect,
            border_radius,
            margin_rect: OnceCell::new(),
            padding_rect: OnceCell::new(),
            content_rect: OnceCell::new(),
            border_edge_clip_chain_id: RefCell::new(None),
            padding_edge_clip_chain_id: RefCell::new(None),
            content_edge_clip_chain_id: RefCell::new(None),
            is_hit_test_for_scrollable_overflow,
            is_collapsed_table_borders,
        }
    }

    fn content_rect(&self) -> &units::LayoutRect {
        self.content_rect.get_or_init(|| {
            self.fragment
                .content_rect
                .translate(self.containing_block.origin.to_vector())
                .to_webrender()
        })
    }

    fn padding_rect(&self) -> &units::LayoutRect {
        self.padding_rect.get_or_init(|| {
            self.fragment
                .padding_rect()
                .translate(self.containing_block.origin.to_vector())
                .to_webrender()
        })
    }

    fn margin_rect(&self) -> &units::LayoutRect {
        self.margin_rect.get_or_init(|| {
            self.fragment
                .margin_rect()
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

        let radii = inner_radii(self.border_radius, self.fragment.border.to_webrender());
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
            (self.fragment.border + self.fragment.padding).to_webrender(),
        );
        let maybe_clip =
            create_clip_chain(radii, *self.content_rect(), builder, force_clip_creation);
        *self.content_edge_clip_chain_id.borrow_mut() = maybe_clip;
        maybe_clip
    }

    fn build(&mut self, builder: &mut DisplayListBuilder, section: StackingContextSection) {
        if self.is_hit_test_for_scrollable_overflow {
            self.build_hit_test(builder, self.fragment.scrollable_overflow().to_webrender());
            return;
        }

        if self.is_collapsed_table_borders {
            self.build_collapsed_table_borders(builder);
            return;
        }

        if section == StackingContextSection::Outline {
            self.build_outline(builder);
            return;
        }

        self.build_hit_test(builder, self.border_rect);
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

    fn build_hit_test(&self, builder: &mut DisplayListBuilder, rect: LayoutRect) {
        let hit_info = builder.hit_info(
            &self.fragment.style,
            self.fragment.base.tag,
            Cursor::Default,
        );
        let hit_info = match hit_info {
            Some(hit_info) => hit_info,
            None => return,
        };

        let mut common = builder.common_properties(rect, &self.fragment.style);
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
        let background_color = painter.style.resolve_color(&b.background_color);
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
        let style = painter.style;
        let b = style.get_background();
        let node = self.fragment.base.tag.map(|tag| tag.node);
        // Reverse because the property is top layer first, we want to paint bottom layer first.
        for (index, image) in b.background_image.0.iter().enumerate().rev() {
            match builder.context.resolve_image(node, image) {
                None => {},
                Some(ResolvedImage::Gradient(gradient)) => {
                    let intrinsic = NaturalSizes::empty();
                    let Some(layer) =
                        &background::layout_layer(self, painter, builder, index, intrinsic)
                    else {
                        continue;
                    };

                    match gradient::build(style, gradient, layer.tile_size, builder) {
                        WebRenderGradient::Linear(linear_gradient) => builder.wr().push_gradient(
                            &layer.common,
                            layer.bounds,
                            linear_gradient,
                            layer.tile_size,
                            layer.tile_spacing,
                        ),
                        WebRenderGradient::Radial(radial_gradient) => {
                            builder.wr().push_radial_gradient(
                                &layer.common,
                                layer.bounds,
                                radial_gradient,
                                layer.tile_size,
                                layer.tile_spacing,
                            )
                        },
                        WebRenderGradient::Conic(conic_gradient) => {
                            builder.wr().push_conic_gradient(
                                &layer.common,
                                layer.bounds,
                                conic_gradient,
                                layer.tile_size,
                                layer.tile_spacing,
                            )
                        },
                    }
                },
                Some(ResolvedImage::Image(image_info)) => {
                    // FIXME: https://drafts.csswg.org/css-images-4/#the-image-resolution
                    let dppx = 1.0;
                    let intrinsic = NaturalSizes::from_width_and_height(
                        image_info.width as f32 / dppx,
                        image_info.height as f32 / dppx,
                    );
                    let Some(image_key) = image_info.key else {
                        continue;
                    };

                    if let Some(layer) =
                        background::layout_layer(self, painter, builder, index, intrinsic)
                    {
                        if layer.repeat {
                            builder.wr().push_repeating_image(
                                &layer.common,
                                layer.bounds,
                                layer.tile_size,
                                layer.tile_spacing,
                                style.clone_image_rendering().to_webrender(),
                                wr::AlphaType::PremultipliedAlpha,
                                image_key,
                                wr::ColorF::WHITE,
                            )
                        } else {
                            builder.wr().push_image(
                                &layer.common,
                                layer.bounds,
                                style.clone_image_rendering().to_webrender(),
                                wr::AlphaType::PremultipliedAlpha,
                                image_key,
                                wr::ColorF::WHITE,
                            )
                        }
                    }
                },
            }
        }
    }

    fn build_border_side(&mut self, style_color: BorderStyleColor) -> wr::BorderSide {
        wr::BorderSide {
            color: rgba(style_color.color),
            style: match style_color.style {
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

    fn build_collapsed_table_borders(&mut self, builder: &mut DisplayListBuilder) {
        let Some(SpecificLayoutInfo::TableGridWithCollapsedBorders(table_info)) =
            &self.fragment.specific_layout_info
        else {
            return;
        };
        let mut common =
            builder.common_properties(units::LayoutRect::default(), &self.fragment.style);
        let radius = wr::BorderRadius::default();
        let mut column_sum = Au::zero();
        for (x, column_size) in table_info.track_sizes.x.iter().enumerate() {
            let mut row_sum = Au::zero();
            for (y, row_size) in table_info.track_sizes.y.iter().enumerate() {
                let left_border = &table_info.collapsed_borders.x[x][y];
                let right_border = &table_info.collapsed_borders.x[x + 1][y];
                let top_border = &table_info.collapsed_borders.y[y][x];
                let bottom_border = &table_info.collapsed_borders.y[y + 1][x];
                let details = wr::BorderDetails::Normal(wr::NormalBorder {
                    left: self.build_border_side(left_border.style_color.clone()),
                    right: self.build_border_side(right_border.style_color.clone()),
                    top: self.build_border_side(top_border.style_color.clone()),
                    bottom: self.build_border_side(bottom_border.style_color.clone()),
                    radius,
                    do_aa: true,
                });
                let mut border_widths = PhysicalSides::new(
                    top_border.width,
                    right_border.width,
                    bottom_border.width,
                    left_border.width,
                );
                let left_adjustment = if x == 0 {
                    -border_widths.left / 2
                } else {
                    std::mem::take(&mut border_widths.left) / 2
                };
                let top_adjustment = if y == 0 {
                    -border_widths.top / 2
                } else {
                    std::mem::take(&mut border_widths.top) / 2
                };
                let origin =
                    PhysicalPoint::new(column_sum + left_adjustment, row_sum + top_adjustment);
                let size = PhysicalSize::new(
                    *column_size - left_adjustment + border_widths.right / 2,
                    *row_size - top_adjustment + border_widths.bottom / 2,
                );
                let border_rect = PhysicalRect::new(origin, size)
                    .translate(self.fragment.content_rect.origin.to_vector())
                    .translate(self.containing_block.origin.to_vector())
                    .to_webrender();
                common.clip_rect = border_rect;
                builder.wr().push_border(
                    &common,
                    border_rect,
                    border_widths.to_webrender(),
                    details,
                );
                row_sum += *row_size;
            }
            column_sum += *column_size;
        }
    }

    fn build_border(&mut self, builder: &mut DisplayListBuilder) {
        if self.fragment.has_collapsed_borders() {
            // Avoid painting borders for tables and table parts in collapsed-borders mode,
            // since the resulting collapsed borders are painted on their own in a special way.
            return;
        }

        let border = self.fragment.style.get_border();
        let border_widths = self.fragment.border.to_webrender();

        if border_widths == SideOffsets2D::zero() {
            return;
        }

        // `border-image` replaces an element's border entirely.
        let common = builder.common_properties(self.border_rect, &self.fragment.style);
        if self.build_border_image(builder, &common, border, border_widths) {
            return;
        }

        let current_color = self.fragment.style.get_inherited_text().clone_color();
        let style_color = BorderStyleColor::from_border(border, &current_color);
        let details = wr::BorderDetails::Normal(wr::NormalBorder {
            top: self.build_border_side(style_color.top),
            right: self.build_border_side(style_color.right),
            bottom: self.build_border_side(style_color.bottom),
            left: self.build_border_side(style_color.left),
            radius: self.border_radius,
            do_aa: true,
        });
        builder
            .wr()
            .push_border(&common, self.border_rect, border_widths, details)
    }

    /// Add a display item for image borders if necessary.
    fn build_border_image(
        &self,
        builder: &mut DisplayListBuilder,
        common: &CommonItemProperties,
        border: &Border,
        border_widths: SideOffsets2D<f32, LayoutPixel>,
    ) -> bool {
        let border_style_struct = self.fragment.style.get_border();
        let border_image_outset =
            resolve_border_image_outset(border_style_struct.border_image_outset, border_widths);
        let border_image_area = self.border_rect.to_rect().outer_rect(border_image_outset);
        let border_image_size = border_image_area.size;
        let border_image_widths = resolve_border_image_width(
            &border_style_struct.border_image_width,
            border_widths,
            border_image_size,
        );
        let border_image_repeat = &border_style_struct.border_image_repeat;
        let border_image_fill = border_style_struct.border_image_slice.fill;
        let border_image_slice = &border_style_struct.border_image_slice.offsets;

        let stops = Vec::new();
        let mut width = border_image_size.width;
        let mut height = border_image_size.height;
        let node = self.fragment.base.tag.map(|tag| tag.node);
        let source = match builder
            .context
            .resolve_image(node, &border.border_image_source)
        {
            None => return false,
            Some(ResolvedImage::Image(image_info)) => {
                let Some(key) = image_info.key else {
                    return false;
                };

                width = image_info.width as f32;
                height = image_info.height as f32;
                NinePatchBorderSource::Image(key, ImageRendering::Auto)
            },
            Some(ResolvedImage::Gradient(gradient)) => {
                match gradient::build(&self.fragment.style, gradient, border_image_size, builder) {
                    WebRenderGradient::Linear(gradient) => {
                        NinePatchBorderSource::Gradient(gradient)
                    },
                    WebRenderGradient::Radial(gradient) => {
                        NinePatchBorderSource::RadialGradient(gradient)
                    },
                    WebRenderGradient::Conic(gradient) => {
                        NinePatchBorderSource::ConicGradient(gradient)
                    },
                }
            },
        };

        let size = euclid::Size2D::new(width as i32, height as i32);
        let details = BorderDetails::NinePatch(NinePatchBorder {
            source,
            width: size.width,
            height: size.height,
            slice: resolve_border_image_slice(border_image_slice, size),
            fill: border_image_fill,
            repeat_horizontal: border_image_repeat.0.to_webrender(),
            repeat_vertical: border_image_repeat.1.to_webrender(),
        });
        builder.wr().push_border(
            common,
            border_image_area.to_box2d(),
            border_image_widths,
            details,
        );
        builder.wr().push_stops(&stops);
        true
    }

    fn build_outline(&mut self, builder: &mut DisplayListBuilder) {
        let style = &self.fragment.style;
        let outline = style.get_outline();
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
        let border_style = match outline.outline_style {
            // TODO: treating 'auto' as 'solid' is allowed by the spec,
            // but we should do something better.
            OutlineStyle::Auto => BorderStyle::Solid,
            OutlineStyle::BorderStyle(s) => s,
        };
        let side = self.build_border_side(BorderStyleColor {
            style: border_style,
            color: style.resolve_color(&outline.outline_color),
        });
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
        let common = builder.common_properties(MaxRect::max_rect(), &self.fragment.style);
        for box_shadow in box_shadows.iter().rev() {
            let (rect, clip_mode) = if box_shadow.inset {
                (*self.padding_rect(), BoxShadowClipMode::Inset)
            } else {
                (self.border_rect, BoxShadowClipMode::Outset)
            };

            builder.wr().push_box_shadow(
                &common,
                rect,
                LayoutVector2D::new(
                    box_shadow.base.horizontal.px(),
                    box_shadow.base.vertical.px(),
                ),
                rgba(self.fragment.style.resolve_color(&box_shadow.base.color)),
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
    ignore_whitespace: bool,
) -> Vec<wr::GlyphInstance> {
    use fonts_traits::ByteIndex;
    use range::Range;

    let mut glyphs = vec![];
    for run in glyph_runs {
        for glyph in run.iter_glyphs_for_byte_range(&Range::new(ByteIndex(0), run.len())) {
            if !run.is_whitespace() || !ignore_whitespace {
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

fn glyphs_advance_by_index(
    glyph_runs: &[Arc<GlyphStore>],
    index: fonts_traits::ByteIndex,
    baseline_origin: PhysicalPoint<Au>,
    justification_adjustment: Au,
) -> PhysicalPoint<Au> {
    let mut point = baseline_origin;
    for run in glyph_runs {
        let total_advance = run.advance_for_byte_range(
            &ServoRange::new(fonts::ByteIndex(0), index),
            justification_adjustment,
        );
        point.x += total_advance;
    }
    point
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

/// Resolve the WebRender border-image outset area from the style values.
fn resolve_border_image_outset(
    outset: BorderImageOutset,
    border: SideOffsets2D<f32, LayoutPixel>,
) -> SideOffsets2D<f32, LayoutPixel> {
    fn image_outset_for_side(outset: NonNegativeLengthOrNumber, border_width: f32) -> f32 {
        match outset {
            NonNegativeLengthOrNumber::Length(length) => length.px(),
            NonNegativeLengthOrNumber::Number(factor) => border_width * factor.0,
        }
    }

    SideOffsets2D::new(
        image_outset_for_side(outset.0, border.top),
        image_outset_for_side(outset.1, border.right),
        image_outset_for_side(outset.2, border.bottom),
        image_outset_for_side(outset.3, border.left),
    )
}

/// Resolve the WebRender border-image width from the style values.
fn resolve_border_image_width(
    width: &BorderImageWidth,
    border: SideOffsets2D<f32, LayoutPixel>,
    border_area: Size2D<f32, LayoutPixel>,
) -> SideOffsets2D<f32, LayoutPixel> {
    fn image_width_for_side(
        border_image_width: &BorderImageSideWidth,
        border_width: f32,
        total_length: f32,
    ) -> f32 {
        match border_image_width {
            BorderImageSideWidth::LengthPercentage(v) => {
                v.to_used_value(Au::from_f32_px(total_length)).to_f32_px()
            },
            BorderImageSideWidth::Number(x) => border_width * x.0,
            BorderImageSideWidth::Auto => border_width,
        }
    }

    SideOffsets2D::new(
        image_width_for_side(&width.0, border.top, border_area.height),
        image_width_for_side(&width.1, border.right, border_area.width),
        image_width_for_side(&width.2, border.bottom, border_area.height),
        image_width_for_side(&width.3, border.left, border_area.width),
    )
}

/// Resolve the WebRender border-image slice from the style values.
fn resolve_border_image_slice(
    border_image_slice: &Rect<NonNegative<NumberOrPercentage>>,
    size: Size2D<i32, UnknownUnit>,
) -> SideOffsets2D<i32, DevicePixel> {
    fn resolve_percentage(value: NonNegative<NumberOrPercentage>, length: i32) -> i32 {
        match value.0 {
            NumberOrPercentage::Percentage(p) => (p.0 * length as f32).round() as i32,
            NumberOrPercentage::Number(n) => n.round() as i32,
        }
    }

    SideOffsets2D::new(
        resolve_percentage(border_image_slice.0, size.height),
        resolve_percentage(border_image_slice.1, size.width),
        resolve_percentage(border_image_slice.2, size.height),
        resolve_percentage(border_image_slice.3, size.width),
    )
}

pub(super) fn normalize_radii(rect: &units::LayoutRect, radius: &mut wr::BorderRadius) {
    // Normalize radii that add up to > 100%.
    // https://www.w3.org/TR/css-backgrounds-3/#corner-overlap
    // > Let f = min(L_i/S_i), where i ∈ {top, right, bottom, left},
    // > S_i is the sum of the two corresponding radii of the corners on side i,
    // > and L_top = L_bottom = the width of the box,
    // > and L_left = L_right = the height of the box.
    // > If f < 1, then all corner radii are reduced by multiplying them by f.
    let f = (rect.width() / (radius.top_left.width + radius.top_right.width))
        .min(rect.width() / (radius.bottom_left.width + radius.bottom_right.width))
        .min(rect.height() / (radius.top_left.height + radius.bottom_left.height))
        .min(rect.height() / (radius.top_right.height + radius.bottom_right.height));
    if f < 1.0 {
        radius.top_left *= f;
        radius.top_right *= f;
        radius.bottom_right *= f;
        radius.bottom_left *= f;
    }
}

/// <https://drafts.csswg.org/css-shapes-1/#valdef-shape-box-margin-box>
/// > The corner radii of this shape are determined by the corresponding
/// > border-radius and margin values. If the ratio of border-radius/margin is 1 or more,
/// > or margin is negative or zero, then the margin box corner radius is
/// > max(border-radius + margin, 0). If the ratio of border-radius/margin is less than 1,
/// > and margin is positive, then the margin box corner radius is
/// > border-radius + margin * (1 + (ratio-1)^3).
pub(super) fn compute_margin_box_radius(
    radius: wr::BorderRadius,
    layout_rect: LayoutSize,
    fragment: &BoxFragment,
) -> wr::BorderRadius {
    let margin = fragment.style.physical_margin();
    let adjust_radius = |radius: f32, margin: f32| -> f32 {
        if margin <= 0. || (radius / margin) >= 1. {
            (radius + margin).max(0.)
        } else {
            radius + (margin * (1. + (radius / margin - 1.).powf(3.)))
        }
    };
    let compute_margin_radius = |radius: LayoutSize,
                                 layout_rect: LayoutSize,
                                 margin: Size2D<LengthPercentageOrAuto, UnknownUnit>|
     -> LayoutSize {
        let zero = LengthPercentage::zero();
        let width = margin
            .width
            .auto_is(|| &zero)
            .to_used_value(Au::from_f32_px(layout_rect.width));
        let height = margin
            .height
            .auto_is(|| &zero)
            .to_used_value(Au::from_f32_px(layout_rect.height));
        LayoutSize::new(
            adjust_radius(radius.width, width.to_f32_px()),
            adjust_radius(radius.height, height.to_f32_px()),
        )
    };
    wr::BorderRadius {
        top_left: compute_margin_radius(
            radius.top_left,
            layout_rect,
            Size2D::new(margin.left, margin.top),
        ),
        top_right: compute_margin_radius(
            radius.top_right,
            layout_rect,
            Size2D::new(margin.right, margin.top),
        ),
        bottom_left: compute_margin_radius(
            radius.bottom_left,
            layout_rect,
            Size2D::new(margin.left, margin.bottom),
        ),
        bottom_right: compute_margin_radius(
            radius.bottom_right,
            layout_rect,
            Size2D::new(margin.right, margin.bottom),
        ),
    }
}
