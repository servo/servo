/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{OnceCell, RefCell};
use std::sync::Arc;

use app_units::{AU_PER_PX, Au};
use base::WebRenderEpochToU16;
use base::id::ScrollTreeNodeId;
use clip::{Clip, ClipId};
use compositing_traits::display_list::{CompositorDisplayListInfo, SpatialTreeNodeInfo};
use embedder_traits::Cursor;
use euclid::{Point2D, Scale, SideOffsets2D, Size2D, UnknownUnit, Vector2D};
use fonts::GlyphStore;
use gradient::WebRenderGradient;
use layout_api::ReflowRequest;
use net_traits::image_cache::Image as CachedImage;
use range::Range as ServoRange;
use servo_arc::Arc as ServoArc;
use servo_config::opts::DebugOptions;
use servo_geometry::MaxRect;
use style::Zero;
use style::color::{AbsoluteColor, ColorSpace};
use style::computed_values::border_image_outset::T as BorderImageOutset;
use style::computed_values::text_decoration_style::{
    T as ComputedTextDecorationStyle, T as TextDecorationStyle,
};
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
use style_traits::{CSSPixel as StyloCSSPixel, DevicePixel as StyloDevicePixel};
use webrender_api::units::{DeviceIntSize, DevicePixel, LayoutPixel, LayoutRect, LayoutSize};
use webrender_api::{
    self as wr, BorderDetails, BoxShadowClipMode, BuiltDisplayList, ClipChainId, ClipMode,
    CommonItemProperties, ComplexClipRegion, ImageRendering, NinePatchBorder,
    NinePatchBorderSource, PropertyBinding, SpatialId, SpatialTreeItemKey, units,
};
use wr::units::LayoutVector2D;

use crate::cell::ArcRefCell;
use crate::context::{ImageResolver, ResolvedImage};
pub use crate::display_list::conversions::ToWebRender;
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
mod clip;
mod conversions;
mod gradient;
mod stacking_context;

use background::BackgroundPainter;
pub use stacking_context::*;

// webrender's `ItemTag` is private.
type ItemTag = (u64, u16);
type HitInfo = Option<ItemTag>;
const INSERTION_POINT_LOGICAL_WIDTH: Au = Au(AU_PER_PX);

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

    /// The current [`ClipId`] for this [DisplayListBuilder]. This allows
    /// only passing the builder instead passing the containing
    /// [stacking_context::StackingContextContent::Fragment] as an argument to display
    /// list building functions.
    current_clip_id: ClipId,

    /// The [`wr::DisplayListBuilder`] for this Servo [`DisplayListBuilder`].
    pub webrender_display_list_builder: &'a mut wr::DisplayListBuilder,

    /// The [`CompositorDisplayListInfo`] used to collect display list items and metadata.
    pub compositor_info: &'a mut CompositorDisplayListInfo,

    /// Data about the fragments that are highlighted by the inspector, if any.
    ///
    /// This data is collected during the traversal of the fragment tree and used
    /// to paint the highlight at the very end.
    inspector_highlight: Option<InspectorHighlight>,

    /// Whether or not the `<body>` element should be painted. This is false if the root `<html>`
    /// element inherits the `<body>`'s background to paint the page canvas background.
    /// See <https://drafts.csswg.org/css-backgrounds/#body-background>.
    paint_body_background: bool,

    /// A mapping from [`ClipId`] To WebRender [`ClipChainId`] used when building this WebRender
    /// display list.
    clip_map: Vec<ClipChainId>,

    /// An [`ImageResolver`] to use during display list construction.
    image_resolver: Arc<ImageResolver>,

    /// The device pixel ratio used for this `Document`'s display list.
    device_pixel_ratio: Scale<f32, StyloCSSPixel, StyloDevicePixel>,
}

struct InspectorHighlight {
    /// The node that should be highlighted
    tag: Tag,

    /// Accumulates information about the fragments that belong to the highlighted node.
    ///
    /// This information is collected as the fragment tree is traversed to build the
    /// display list.
    state: Option<HighlightTraversalState>,
}

struct HighlightTraversalState {
    /// The smallest rectangle that fully encloses all fragments created by the highlighted
    /// dom node, if any.
    content_box: euclid::Rect<Au, StyloCSSPixel>,

    spatial_id: SpatialId,

    clip_chain_id: ClipChainId,

    /// When the highlighted fragment is a box fragment we remember the information
    /// needed to paint padding, border and margin areas.
    maybe_box_fragment: Option<ArcRefCell<BoxFragment>>,
}

impl InspectorHighlight {
    fn for_node(node: OpaqueNode) -> Self {
        Self {
            tag: Tag::new(node),
            state: None,
        }
    }
}

impl DisplayListBuilder<'_> {
    pub(crate) fn build(
        reflow_request: &ReflowRequest,
        stacking_context_tree: &mut StackingContextTree,
        fragment_tree: &FragmentTree,
        image_resolver: Arc<ImageResolver>,
        device_pixel_ratio: Scale<f32, StyloCSSPixel, StyloDevicePixel>,
        debug: &DebugOptions,
    ) -> BuiltDisplayList {
        // Build the rest of the display list which inclues all of the WebRender primitives.
        let compositor_info = &mut stacking_context_tree.compositor_info;
        compositor_info.hit_test_info.clear();

        let mut webrender_display_list_builder =
            webrender_api::DisplayListBuilder::new(compositor_info.pipeline_id);
        webrender_display_list_builder.begin();

        // `dump_serialized_display_list` doesn't actually print anything. It sets up
        // the display list for printing the serialized version when `finalize()` is called.
        // We need to call this before adding any display items so that they are printed
        // during `finalize()`.
        if debug.dump_display_list {
            webrender_display_list_builder.dump_serialized_display_list();
        }

        #[cfg(feature = "tracing")]
        let _span =
            tracing::trace_span!("DisplayListBuilder::build", servo_profiling = true).entered();
        let mut builder = DisplayListBuilder {
            current_scroll_node_id: compositor_info.root_reference_frame_id,
            current_reference_frame_scroll_node_id: compositor_info.root_reference_frame_id,
            current_clip_id: ClipId::INVALID,
            webrender_display_list_builder: &mut webrender_display_list_builder,
            compositor_info,
            inspector_highlight: reflow_request
                .highlighted_dom_node
                .map(InspectorHighlight::for_node),
            paint_body_background: true,
            clip_map: Default::default(),
            image_resolver,
            device_pixel_ratio,
        };

        builder.add_all_spatial_nodes();

        for clip in stacking_context_tree.clip_store.0.iter() {
            builder.add_clip_to_display_list(clip);
        }

        // Paint the canvas’ background (if any) before/under everything else
        stacking_context_tree
            .root_stacking_context
            .build_canvas_background_display_list(&mut builder, fragment_tree);
        stacking_context_tree
            .root_stacking_context
            .build_display_list(&mut builder);
        builder.paint_dom_inspector_highlight();

        webrender_display_list_builder.end().1
    }

    fn wr(&mut self) -> &mut wr::DisplayListBuilder {
        self.webrender_display_list_builder
    }

    fn pipeline_id(&mut self) -> wr::PipelineId {
        self.compositor_info.pipeline_id
    }

    fn mark_is_contentful(&mut self) {
        self.compositor_info.is_contentful = true;
    }

    fn spatial_id(&self, id: ScrollTreeNodeId) -> SpatialId {
        self.compositor_info.scroll_tree.webrender_id(&id)
    }

    fn clip_chain_id(&self, id: ClipId) -> ClipChainId {
        match id {
            ClipId::INVALID => ClipChainId::INVALID,
            _ => *self
                .clip_map
                .get(id.0)
                .expect("Should never try to get clip before adding it to WebRender display list"),
        }
    }

    pub(crate) fn add_all_spatial_nodes(&mut self) {
        // A count of the number of SpatialTree nodes pushed to the WebRender display
        // list. This is merely to ensure that the currently-unused SpatialTreeItemKey
        // produced for every SpatialTree node is unique.
        let mut spatial_tree_count = 0;
        let mut scroll_tree = std::mem::take(&mut self.compositor_info.scroll_tree);
        let mut mapping = Vec::with_capacity(scroll_tree.nodes.len());

        mapping.push(SpatialId::root_reference_frame(self.pipeline_id()));
        mapping.push(SpatialId::root_scroll_node(self.pipeline_id()));

        let pipeline_id = self.pipeline_id();
        let pipeline_tag = ((pipeline_id.0 as u64) << 32) | pipeline_id.1 as u64;

        for node in scroll_tree.nodes.iter().skip(2) {
            let parent_scroll_node_id = node
                .parent
                .expect("Should have already added root reference frame");
            let parent_spatial_node_id = mapping
                .get(parent_scroll_node_id.index)
                .expect("Should add spatial nodes to display list in order");

            // Produce a new SpatialTreeItemKey. This is currently unused by WebRender,
            // but has to be unique to the entire scene.
            spatial_tree_count += 1;
            let spatial_tree_item_key = SpatialTreeItemKey::new(pipeline_tag, spatial_tree_count);

            mapping.push(match &node.info {
                SpatialTreeNodeInfo::ReferenceFrame(info) => {
                    let spatial_id = self.wr().push_reference_frame(
                        info.origin,
                        *parent_spatial_node_id,
                        info.transform_style,
                        PropertyBinding::Value(info.transform),
                        info.kind,
                        spatial_tree_item_key,
                    );
                    self.wr().pop_reference_frame();
                    spatial_id
                },
                SpatialTreeNodeInfo::Scroll(info) => {
                    self.wr().define_scroll_frame(
                        *parent_spatial_node_id,
                        info.external_id,
                        info.content_rect,
                        info.clip_rect,
                        LayoutVector2D::zero(), /* external_scroll_offset */
                        0,                      /* scroll_offset_generation */
                        wr::HasScrollLinkedEffect::No,
                        spatial_tree_item_key,
                    )
                },
                SpatialTreeNodeInfo::Sticky(info) => {
                    self.wr().define_sticky_frame(
                        *parent_spatial_node_id,
                        info.frame_rect,
                        info.margins,
                        info.vertical_offset_bounds,
                        info.horizontal_offset_bounds,
                        LayoutVector2D::zero(), /* previously_applied_offset */
                        spatial_tree_item_key,
                        None, /* transform */
                    )
                },
            });
        }

        scroll_tree.update_mapping(mapping);
        self.compositor_info.scroll_tree = scroll_tree;
    }

    /// Add the given [`Clip`] to the WebRender display list and create a mapping from
    /// its [`ClipId`] to a WebRender [`ClipChainId`]. This happens:
    ///  - When WebRender display list construction starts: All clips created during the
    ///    `StackingContextTree` construction are added in one batch. These clips are used
    ///    for things such as `overflow: scroll` elements.
    ///  - When a clip is added during WebRender display list construction for individual
    ///    items. In that case, this is called by [`Self::maybe_create_clip`].
    pub(crate) fn add_clip_to_display_list(&mut self, clip: &Clip) -> ClipChainId {
        assert_eq!(
            clip.id.0,
            self.clip_map.len(),
            "Clips should be added in order"
        );

        let spatial_id = self.spatial_id(clip.parent_scroll_node_id);
        let new_clip_id = if clip.radii.is_zero() {
            self.wr().define_clip_rect(spatial_id, clip.rect)
        } else {
            self.wr().define_clip_rounded_rect(
                spatial_id,
                ComplexClipRegion {
                    rect: clip.rect,
                    radii: clip.radii,
                    mode: ClipMode::Clip,
                },
            )
        };

        // WebRender has two different ways of expressing "no clip." ClipChainId::INVALID should be
        // used for primitives, but `None` is used for stacking contexts and clip chains. We convert
        // to the `Option<ClipChainId>` representation here. Just passing Some(ClipChainId::INVALID)
        // leads to a crash.
        let parent_clip_chain_id = match self.clip_chain_id(clip.parent_clip_id) {
            ClipChainId::INVALID => None,
            parent => Some(parent),
        };
        let clip_chain_id = self
            .wr()
            .define_clip_chain(parent_clip_chain_id, [new_clip_id]);
        self.clip_map.push(clip_chain_id);
        clip_chain_id
    }

    /// Add a new clip to the WebRender display list being built. This only happens during
    /// WebRender display list building and these clips should be added after all clips
    /// from the `StackingContextTree` have already been processed.
    fn maybe_create_clip(
        &mut self,
        radii: wr::BorderRadius,
        rect: units::LayoutRect,
        force_clip_creation: bool,
    ) -> Option<ClipChainId> {
        if radii.is_zero() && !force_clip_creation {
            return None;
        }

        Some(self.add_clip_to_display_list(&Clip {
            id: ClipId(self.clip_map.len()),
            radii,
            rect,
            parent_scroll_node_id: self.current_scroll_node_id,
            parent_clip_id: self.current_clip_id,
        }))
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
            spatial_id: self.spatial_id(self.current_scroll_node_id),
            clip_chain_id: self.clip_chain_id(self.current_clip_id),
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

        let hit_test_index = self.compositor_info.add_hit_test_info(
            tag?.node.0 as u64,
            Some(cursor(inherited_ui.cursor.keyword, auto_cursor)),
            self.current_scroll_node_id,
        );
        Some((hit_test_index as u64, self.compositor_info.epoch.as_u16()))
    }

    /// Draw highlights around the node that is currently hovered in the devtools.
    fn paint_dom_inspector_highlight(&mut self) {
        let Some(highlight) = self
            .inspector_highlight
            .take()
            .and_then(|highlight| highlight.state)
        else {
            return;
        };

        const CONTENT_BOX_HIGHLIGHT_COLOR: webrender_api::ColorF = webrender_api::ColorF {
            r: 0.23,
            g: 0.7,
            b: 0.87,
            a: 0.5,
        };

        const PADDING_BOX_HIGHLIGHT_COLOR: webrender_api::ColorF = webrender_api::ColorF {
            r: 0.49,
            g: 0.3,
            b: 0.7,
            a: 0.5,
        };

        const BORDER_BOX_HIGHLIGHT_COLOR: webrender_api::ColorF = webrender_api::ColorF {
            r: 0.2,
            g: 0.2,
            b: 0.2,
            a: 0.5,
        };

        const MARGIN_BOX_HIGHLIGHT_COLOR: webrender_api::ColorF = webrender_api::ColorF {
            r: 1.,
            g: 0.93,
            b: 0.,
            a: 0.5,
        };

        // Highlight content box
        let content_box = highlight.content_box.to_webrender();
        let properties = wr::CommonItemProperties {
            clip_rect: content_box,
            spatial_id: highlight.spatial_id,
            clip_chain_id: highlight.clip_chain_id,
            flags: wr::PrimitiveFlags::default(),
        };

        self.wr()
            .push_rect(&properties, content_box, CONTENT_BOX_HIGHLIGHT_COLOR);

        // Highlight margin, border and padding
        if let Some(box_fragment) = highlight.maybe_box_fragment {
            let mut paint_highlight =
                |color: webrender_api::ColorF,
                 fragment_relative_bounds: PhysicalRect<Au>,
                 widths: webrender_api::units::LayoutSideOffsets| {
                    if widths.is_zero() {
                        return;
                    }

                    let bounds = box_fragment
                        .borrow()
                        .offset_by_containing_block(&fragment_relative_bounds)
                        .to_webrender();

                    // We paint each highlighted area as if it was a border for simplicity
                    let border_style = wr::BorderSide {
                        color,
                        style: webrender_api::BorderStyle::Solid,
                    };

                    let details = wr::BorderDetails::Normal(wr::NormalBorder {
                        top: border_style,
                        right: border_style,
                        bottom: border_style,
                        left: border_style,
                        radius: webrender_api::BorderRadius::default(),
                        do_aa: true,
                    });

                    let common = wr::CommonItemProperties {
                        clip_rect: bounds,
                        spatial_id: highlight.spatial_id,
                        clip_chain_id: highlight.clip_chain_id,
                        flags: wr::PrimitiveFlags::default(),
                    };
                    self.wr().push_border(&common, bounds, widths, details)
                };

            let box_fragment = box_fragment.borrow();
            paint_highlight(
                PADDING_BOX_HIGHLIGHT_COLOR,
                box_fragment.padding_rect(),
                box_fragment.padding.to_webrender(),
            );
            paint_highlight(
                BORDER_BOX_HIGHLIGHT_COLOR,
                box_fragment.border_rect(),
                box_fragment.border.to_webrender(),
            );
            paint_highlight(
                MARGIN_BOX_HIGHLIGHT_COLOR,
                box_fragment.margin_rect(),
                box_fragment.margin.to_webrender(),
            );
        }
    }
}

impl InspectorHighlight {
    fn register_fragment_of_highlighted_dom_node(
        &mut self,
        fragment: &Fragment,
        spatial_id: SpatialId,
        clip_chain_id: ClipChainId,
        containing_block: &PhysicalRect<Au>,
    ) {
        let state = self.state.get_or_insert(HighlightTraversalState {
            content_box: euclid::Rect::zero(),
            spatial_id,
            clip_chain_id,
            maybe_box_fragment: None,
        });

        // We expect all fragments generated by one node to be in the same scroll tree node and clip node
        debug_assert_eq!(spatial_id, state.spatial_id);
        if clip_chain_id != ClipChainId::INVALID && state.clip_chain_id != ClipChainId::INVALID {
            debug_assert_eq!(
                clip_chain_id, state.clip_chain_id,
                "Fragments of the same node must either have no clip chain or the same one"
            );
        }

        let fragment_relative_rect = match fragment {
            Fragment::Box(fragment) | Fragment::Float(fragment) => {
                state.maybe_box_fragment = Some(fragment.clone());

                fragment.borrow().content_rect
            },
            Fragment::Positioning(fragment) => fragment.borrow().rect,
            Fragment::Text(fragment) => fragment.borrow().rect,
            Fragment::Image(image_fragment) => image_fragment.borrow().rect,
            Fragment::AbsoluteOrFixedPositioned(_) => return,
            Fragment::IFrame(iframe_fragment) => iframe_fragment.borrow().rect,
        };

        state.content_box = state
            .content_box
            .union(&fragment_relative_rect.translate(containing_block.origin.to_vector()));
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
        text_decorations: &Arc<Vec<FragmentTextDecoration>>,
    ) {
        let spatial_id = builder.spatial_id(builder.current_scroll_node_id);
        let clip_chain_id = builder.clip_chain_id(builder.current_clip_id);
        if let Some(inspector_highlight) = &mut builder.inspector_highlight {
            if self.tag() == Some(inspector_highlight.tag) {
                inspector_highlight.register_fragment_of_highlighted_dom_node(
                    self,
                    spatial_id,
                    clip_chain_id,
                    containing_block,
                );
            }
        }

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
                let rect = positioning_fragment
                    .rect
                    .translate(containing_block.origin.to_vector());
                self.maybe_push_hit_test_for_style_and_tag(
                    builder,
                    &positioning_fragment.style,
                    positioning_fragment.base.tag,
                    rect,
                    Cursor::Default,
                );
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
                match text
                    .inline_styles
                    .style
                    .borrow()
                    .get_inherited_box()
                    .visibility
                {
                    Visibility::Visible => self.build_display_list_for_text_fragment(
                        text,
                        builder,
                        containing_block,
                        text_decorations,
                    ),
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

        let clip_chain_id = builder.clip_chain_id(builder.current_clip_id);
        let spatial_id = builder.spatial_id(builder.current_scroll_node_id);
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
        text_decorations: &Arc<Vec<FragmentTextDecoration>>,
    ) {
        // NB: The order of painting text components (CSS Text Decoration Module Level 3) is:
        // shadows, underline, overline, text, text-emphasis, and then line-through.

        builder.mark_is_contentful();

        let rect = fragment.rect.translate(containing_block.origin.to_vector());
        let mut baseline_origin = rect.origin;
        baseline_origin.y += fragment.font_metrics.ascent;
        let include_whitespace =
            fragment.has_selection() || text_decorations.iter().any(|item| !item.line.is_empty());

        let glyphs = glyphs(
            &fragment.glyphs,
            baseline_origin,
            fragment.justification_adjustment,
            include_whitespace,
        );
        if glyphs.is_empty() {
            return;
        }

        let parent_style = fragment.inline_styles.style.borrow();
        self.maybe_push_hit_test_for_style_and_tag(
            builder,
            &parent_style,
            fragment.base.tag,
            rect,
            Cursor::Text,
        );

        let color = parent_style.clone_color();
        let font_metrics = &fragment.font_metrics;
        let dppx = builder.device_pixel_ratio.get();
        let common = builder.common_properties(rect.to_webrender(), &parent_style);

        // Shadows. According to CSS-BACKGROUNDS, text shadows render in *reverse* order (front to
        // back).
        let shadows = &parent_style.get_inherited_text().text_shadow;
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

        for text_decoration in text_decorations.iter() {
            if text_decoration.line.contains(TextDecorationLine::UNDERLINE) {
                let mut rect = rect;
                rect.origin.y += font_metrics.ascent - font_metrics.underline_offset;
                rect.size.height =
                    Au::from_f32_px(font_metrics.underline_size.to_nearest_pixel(dppx));

                self.build_display_list_for_text_decoration(
                    &parent_style,
                    builder,
                    &rect,
                    text_decoration,
                    TextDecorationLine::UNDERLINE,
                );
            }
        }

        for text_decoration in text_decorations.iter() {
            if text_decoration.line.contains(TextDecorationLine::OVERLINE) {
                let mut rect = rect;
                rect.size.height =
                    Au::from_f32_px(font_metrics.underline_size.to_nearest_pixel(dppx));
                self.build_display_list_for_text_decoration(
                    &parent_style,
                    builder,
                    &rect,
                    text_decoration,
                    TextDecorationLine::OVERLINE,
                );
            }
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
                    .inline_styles
                    .selected
                    .borrow()
                    .clone_background_color()
                    .as_absolute()
                {
                    let selection_common = builder.common_properties(selection_rect, &parent_style);
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
                    builder.common_properties(insertion_point_rect, &parent_style);
                // TODO: The color of the caret is currently hardcoded to the text color.
                // We should be retrieving the caret color from the style properly.
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

        for text_decoration in text_decorations.iter() {
            if text_decoration
                .line
                .contains(TextDecorationLine::LINE_THROUGH)
            {
                let mut rect = rect;
                rect.origin.y += font_metrics.ascent - font_metrics.strikeout_offset;
                rect.size.height =
                    Au::from_f32_px(font_metrics.strikeout_size.to_nearest_pixel(dppx));
                self.build_display_list_for_text_decoration(
                    &parent_style,
                    builder,
                    &rect,
                    text_decoration,
                    TextDecorationLine::LINE_THROUGH,
                );
            }
        }

        if !shadows.0.is_empty() {
            builder.wr().pop_all_shadows();
        }
    }

    fn build_display_list_for_text_decoration(
        &self,
        parent_style: &ServoArc<ComputedValues>,
        builder: &mut DisplayListBuilder,
        rect: &PhysicalRect<Au>,
        text_decoration: &FragmentTextDecoration,
        line: TextDecorationLine,
    ) {
        if text_decoration.style == ComputedTextDecorationStyle::MozNone {
            return;
        }

        let mut rect = rect.to_webrender();
        let line_thickness = rect.height().ceil();

        if text_decoration.style == ComputedTextDecorationStyle::Wavy {
            rect = rect.inflate(0.0, line_thickness * 1.0);
        }

        let common_properties = builder.common_properties(rect, parent_style);
        builder.wr().push_line(
            &common_properties,
            &rect,
            line_thickness,
            wr::LineOrientation::Horizontal,
            &rgba(text_decoration.color),
            text_decoration.style.to_webrender(),
        );

        if text_decoration.style == TextDecorationStyle::Double {
            let half_height = (rect.height() / 2.0).floor().max(1.0);
            let y_offset = match line {
                TextDecorationLine::OVERLINE => -rect.height() - half_height,
                _ => rect.height() + half_height,
            };
            let rect = rect.translate(Vector2D::new(0.0, y_offset));
            let common_properties = builder.common_properties(rect, parent_style);
            builder.wr().push_line(
                &common_properties,
                &rect,
                line_thickness,
                wr::LineOrientation::Horizontal,
                &rgba(text_decoration.color),
                text_decoration.style.to_webrender(),
            );
        }
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

        let maybe_clip =
            builder.maybe_create_clip(self.border_radius, self.border_rect, force_clip_creation);
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
            builder.maybe_create_clip(radii, *self.padding_rect(), force_clip_creation);
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
            builder.maybe_create_clip(radii, *self.content_rect(), force_clip_creation);
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
        let flags = self.fragment.base.flags;

        // The root element's background is painted separately as it might inherit the `<body>`'s
        // background.
        if flags.intersects(FragmentFlags::IS_ROOT_ELEMENT) {
            return;
        }
        // If the `<body>` background was inherited by the root element, don't paint it again here.
        if !builder.paint_body_background &&
            flags.intersects(FragmentFlags::IS_BODY_ELEMENT_OF_HTML_ELEMENT_ROOT)
        {
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
                    style: &extra_background.style.borrow_mut(),
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
            match builder.image_resolver.resolve_image(node, image) {
                Err(_) => {},
                Ok(ResolvedImage::Gradient(gradient)) => {
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
                Ok(ResolvedImage::Image { image, size }) => {
                    // FIXME: https://drafts.csswg.org/css-images-4/#the-image-resolution
                    let dppx = 1.0;
                    let intrinsic =
                        NaturalSizes::from_width_and_height(size.width / dppx, size.height / dppx);
                    let layer = background::layout_layer(self, painter, builder, index, intrinsic);
                    let image_wr_key = match image {
                        CachedImage::Raster(raster_image) => raster_image.id,
                        CachedImage::Vector(vector_image) => {
                            let scale = builder.device_pixel_ratio.get();
                            let default_size: DeviceIntSize =
                                Size2D::new(size.width * scale, size.height * scale).to_i32();
                            let layer_size = layer.as_ref().map(|layer| {
                                Size2D::new(
                                    layer.tile_size.width * scale,
                                    layer.tile_size.height * scale,
                                )
                                .to_i32()
                            });

                            node.and_then(|node| {
                                let size = layer_size.unwrap_or(default_size);
                                builder.image_resolver.rasterize_vector_image(
                                    vector_image.id,
                                    size,
                                    node,
                                )
                            })
                            .and_then(|rasterized_image| rasterized_image.id)
                        },
                    };

                    let Some(image_key) = image_wr_key else {
                        continue;
                    };

                    if let Some(layer) = layer {
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
            .image_resolver
            .resolve_image(node, &border.border_image_source)
        {
            Err(_) => return false,
            Ok(ResolvedImage::Image { image, size }) => {
                let Some(image) = image.as_raster_image() else {
                    return false;
                };

                let Some(key) = image.id else {
                    return false;
                };

                width = size.width;
                height = size.height;
                NinePatchBorderSource::Image(key, ImageRendering::Auto)
            },
            Ok(ResolvedImage::Gradient(gradient)) => {
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

        // If the size of the border is zero or the size of the border image is zero, just
        // don't render anything. Zero-sized gradients cause problems in WebRender.
        if size.is_empty() || border_image_size.is_empty() {
            return true;
        }

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
    include_whitespace: bool,
) -> Vec<wr::GlyphInstance> {
    use fonts_traits::ByteIndex;
    use range::Range;

    let mut glyphs = vec![];
    for run in glyph_runs {
        for glyph in run.iter_glyphs_for_byte_range(&Range::new(ByteIndex(0), run.len())) {
            if !run.is_whitespace() || include_whitespace {
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

// TODO: The implementation here does not account for multiple glyph runs properly.
fn glyphs_advance_by_index(
    glyph_runs: &[Arc<GlyphStore>],
    index: fonts_traits::ByteIndex,
    baseline_origin: PhysicalPoint<Au>,
    justification_adjustment: Au,
) -> PhysicalPoint<Au> {
    let mut point = baseline_origin;
    let mut index = index;
    for run in glyph_runs {
        let range = ServoRange::new(fonts::ByteIndex(0), index.min(run.len()));
        index = index - range.length();
        let total_advance = run.advance_for_byte_range(&range, justification_adjustment);
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
