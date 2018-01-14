/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Builds display lists from flows and fragments.
//!
//! Other browser engines sometimes call this "painting", but it is more accurately called display
//! list building, as the actual painting does not happen here—only deciding *what* we're going to
//! paint.

#![deny(unsafe_code)]

use app_units::{Au, AU_PER_PX};
use block::{BlockFlow, BlockStackingContextType};
use canvas_traits::canvas::{CanvasMsg, FromLayoutMsg};
use context::LayoutContext;
use display_list::ToLayout;
use display_list::background::{compute_background_image_size, tile_image_axis};
use display_list::background::{convert_linear_gradient, convert_radial_gradient};
use display_list::webrender_helpers::ToBorderRadius;
use euclid::{Point2D, Rect, SideOffsets2D, Size2D, Transform3D, TypedRect, TypedSize2D, Vector2D};
use flex::FlexFlow;
use flow::{BaseFlow, Flow, FlowFlags};
use flow_ref::FlowRef;
use fnv::FnvHashMap;
use fragment::{CanvasFragmentSource, CoordinateSystem, Fragment, ScannedTextFragmentInfo};
use fragment::SpecificFragmentInfo;
use gfx::display_list;
use gfx::display_list::{BaseDisplayItem, BorderDetails, BorderDisplayItem, BLUR_INFLATION_FACTOR};
use gfx::display_list::{BorderRadii, BoxShadowDisplayItem, ClipScrollNode};
use gfx::display_list::{ClipScrollNodeIndex, ClipScrollNodeType, ClippingAndScrolling};
use gfx::display_list::{ClippingRegion, DisplayItem, DisplayItemMetadata, DisplayList};
use gfx::display_list::{DisplayListSection, GradientDisplayItem, IframeDisplayItem, ImageBorder};
use gfx::display_list::{ImageDisplayItem, LineDisplayItem, NormalBorder, OpaqueNode};
use gfx::display_list::{PopAllTextShadowsDisplayItem, PushTextShadowDisplayItem};
use gfx::display_list::{RadialGradientDisplayItem, SolidColorDisplayItem, StackingContext};
use gfx::display_list::{StackingContextType, StickyFrameData, TextDisplayItem, TextOrientation};
use gfx::display_list::WebRenderImageInfo;
use gfx_traits::{combine_id_with_fragment_type, FragmentType, StackingContextId};
use inline::{InlineFlow, InlineFragmentNodeFlags};
use ipc_channel::ipc;
use list_item::ListItemFlow;
use model::{self, MaybeAuto};
use msg::constellation_msg::{BrowsingContextId, PipelineId};
use net_traits::image::base::PixelFormat;
use net_traits::image_cache::UsePlaceholder;
use range::Range;
use servo_config::opts;
use servo_geometry::max_rect;
use std::{cmp, f32};
use std::default::Default;
use std::mem;
use std::sync::Arc;
use style::computed_values::background_attachment::single_value::T as BackgroundAttachment;
use style::computed_values::background_clip::single_value::T as BackgroundClip;
use style::computed_values::background_origin::single_value::T as BackgroundOrigin;
use style::computed_values::border_style::T as BorderStyle;
use style::computed_values::cursor;
use style::computed_values::image_rendering::T as ImageRendering;
use style::computed_values::overflow_x::T as StyleOverflow;
use style::computed_values::pointer_events::T as PointerEvents;
use style::computed_values::position::T as StylePosition;
use style::computed_values::visibility::T as Visibility;
use style::logical_geometry::{LogicalMargin, LogicalPoint, LogicalRect, LogicalSize, WritingMode};
use style::properties::ComputedValues;
use style::properties::longhands::border_image_repeat::computed_value::RepeatKeyword;
use style::properties::style_structs;
use style::servo::restyle_damage::ServoRestyleDamage;
use style::values::{Either, RGBA};
use style::values::computed::{Gradient, NumberOrPercentage};
use style::values::computed::effects::SimpleShadow;
use style::values::generics::background::BackgroundSize;
use style::values::generics::effects::Filter;
use style::values::generics::image::{GradientKind, Image, PaintWorklet};
use style_traits::CSSPixel;
use style_traits::ToCss;
use style_traits::cursor::Cursor;
use table_cell::CollapsedBordersForCell;
use webrender_api::{BoxShadowClipMode, ClipId, ClipMode, ColorF, ComplexClipRegion, LineStyle};
use webrender_api::{LocalClip, RepeatMode, ScrollPolicy, ScrollSensitivity, StickyOffsetBounds};

trait ResolvePercentage {
    fn resolve(&self, length: u32) -> u32;
}

impl ResolvePercentage for NumberOrPercentage {
    fn resolve(&self, length: u32) -> u32 {
        match *self {
            NumberOrPercentage::Percentage(p) => (p.0 * length as f32).round() as u32,
            NumberOrPercentage::Number(n) => n.round() as u32,
        }
    }
}

fn convert_repeat_mode(from: RepeatKeyword) -> RepeatMode {
    match from {
        RepeatKeyword::Stretch => RepeatMode::Stretch,
        RepeatKeyword::Repeat => RepeatMode::Repeat,
        RepeatKeyword::Round => RepeatMode::Round,
        RepeatKeyword::Space => RepeatMode::Space,
    }
}

fn establishes_containing_block_for_absolute(
    flags: StackingContextCollectionFlags,
    positioning: StylePosition,
) -> bool {
    !flags.contains(StackingContextCollectionFlags::NEVER_CREATES_CONTAINING_BLOCK) &&
        StylePosition::Static != positioning
}

trait RgbColor {
    fn rgb(r: u8, g: u8, b: u8) -> Self;
}

impl RgbColor for ColorF {
    fn rgb(r: u8, g: u8, b: u8) -> Self {
        ColorF {
            r: (r as f32) / (255.0 as f32),
            g: (g as f32) / (255.0 as f32),
            b: (b as f32) / (255.0 as f32),
            a: 1.0 as f32,
        }
    }
}

static THREAD_TINT_COLORS: [ColorF; 8] = [
    ColorF {
        r: 6.0 / 255.0,
        g: 153.0 / 255.0,
        b: 198.0 / 255.0,
        a: 0.7,
    },
    ColorF {
        r: 255.0 / 255.0,
        g: 212.0 / 255.0,
        b: 83.0 / 255.0,
        a: 0.7,
    },
    ColorF {
        r: 116.0 / 255.0,
        g: 29.0 / 255.0,
        b: 109.0 / 255.0,
        a: 0.7,
    },
    ColorF {
        r: 204.0 / 255.0,
        g: 158.0 / 255.0,
        b: 199.0 / 255.0,
        a: 0.7,
    },
    ColorF {
        r: 242.0 / 255.0,
        g: 46.0 / 255.0,
        b: 121.0 / 255.0,
        a: 0.7,
    },
    ColorF {
        r: 116.0 / 255.0,
        g: 203.0 / 255.0,
        b: 196.0 / 255.0,
        a: 0.7,
    },
    ColorF {
        r: 255.0 / 255.0,
        g: 249.0 / 255.0,
        b: 201.0 / 255.0,
        a: 0.7,
    },
    ColorF {
        r: 137.0 / 255.0,
        g: 196.0 / 255.0,
        b: 78.0 / 255.0,
        a: 0.7,
    },
];

fn get_cyclic<T>(arr: &[T], index: usize) -> &T {
    &arr[index % arr.len()]
}

pub struct InlineNodeBorderInfo {
    is_first_fragment_of_element: bool,
    is_last_fragment_of_element: bool,
}

#[derive(Debug)]
struct StackingContextInfo {
    children: Vec<StackingContext>,
    clip_scroll_nodes: Vec<ClipScrollNodeIndex>,
}

impl StackingContextInfo {
    fn new() -> StackingContextInfo {
        StackingContextInfo {
            children: Vec::new(),
            clip_scroll_nodes: Vec::new(),
        }
    }

    fn take_children(&mut self) -> Vec<StackingContext> {
        mem::replace(&mut self.children, Vec::new())
    }
}

pub struct StackingContextCollectionState {
    /// The PipelineId of this stacking context collection.
    pub pipeline_id: PipelineId,

    /// The root of the StackingContext tree.
    pub root_stacking_context: StackingContext,

    /// StackingContext and ClipScrollNode children for each StackingContext.
    stacking_context_info: FnvHashMap<StackingContextId, StackingContextInfo>,

    pub clip_scroll_nodes: Vec<ClipScrollNode>,

    /// The current stacking context id, used to keep track of state when building.
    /// recursively building and processing the display list.
    pub current_stacking_context_id: StackingContextId,

    /// The current stacking real context id, which doesn't include pseudo-stacking contexts.
    pub current_real_stacking_context_id: StackingContextId,

    /// The next stacking context id that we will assign to a stacking context.
    pub next_stacking_context_id: StackingContextId,

    /// The current clip and scroll info, used to keep track of state when
    /// recursively building and processing the display list.
    pub current_clipping_and_scrolling: ClippingAndScrolling,

    /// The clip and scroll info  of the first ancestor which defines a containing block.
    /// This is necessary because absolutely positioned items should be clipped
    /// by their containing block's scroll root.
    pub containing_block_clipping_and_scrolling: ClippingAndScrolling,

    /// A stack of clips used to cull display list entries that are outside the
    /// rendered region.
    pub clip_stack: Vec<Rect<Au>>,

    /// A stack of clips used to cull display list entries that are outside the
    /// rendered region, but only collected at containing block boundaries.
    pub containing_block_clip_stack: Vec<Rect<Au>>,

    /// The flow parent's content box, used to calculate sticky constraints.
    parent_stacking_relative_content_box: Rect<Au>,
}

impl StackingContextCollectionState {
    pub fn new(pipeline_id: PipelineId) -> StackingContextCollectionState {
        let root_clip_indices = ClippingAndScrolling::simple(ClipScrollNodeIndex(0));

        // This is just a dummy node to take up a slot in the array. WebRender
        // takes care of adding this root node and it can be ignored during DL conversion.
        let root_node = ClipScrollNode {
            id: Some(ClipId::root_scroll_node(pipeline_id.to_webrender())),
            parent_index: ClipScrollNodeIndex(0),
            clip: ClippingRegion::from_rect(&TypedRect::zero()),
            content_rect: Rect::zero(),
            node_type: ClipScrollNodeType::ScrollFrame(ScrollSensitivity::ScriptAndInputEvents),
        };

        StackingContextCollectionState {
            pipeline_id: pipeline_id,
            root_stacking_context: StackingContext::root(),
            stacking_context_info: FnvHashMap::default(),
            clip_scroll_nodes: vec![root_node],
            current_stacking_context_id: StackingContextId::root(),
            current_real_stacking_context_id: StackingContextId::root(),
            next_stacking_context_id: StackingContextId::root().next(),
            current_clipping_and_scrolling: root_clip_indices,
            containing_block_clipping_and_scrolling: root_clip_indices,
            clip_stack: Vec::new(),
            containing_block_clip_stack: Vec::new(),
            parent_stacking_relative_content_box: Rect::zero(),
        }
    }

    fn generate_stacking_context_id(&mut self) -> StackingContextId {
        let next_stacking_context_id = self.next_stacking_context_id.next();
        mem::replace(&mut self.next_stacking_context_id, next_stacking_context_id)
    }

    fn add_stacking_context(
        &mut self,
        parent_id: StackingContextId,
        stacking_context: StackingContext,
    ) {
        let info = self.stacking_context_info
            .entry(parent_id)
            .or_insert(StackingContextInfo::new());
        info.children.push(stacking_context);
    }

    fn add_clip_scroll_node(&mut self, clip_scroll_node: ClipScrollNode) -> ClipScrollNodeIndex {
        // We want the scroll root to be defined before any possible item that could use it,
        // so we make sure that it is added to the beginning of the parent "real" (non-pseudo)
        // stacking context. This ensures that item reordering will not result in an item using
        // the scroll root before it is defined.
        self.clip_scroll_nodes.push(clip_scroll_node);
        let index = ClipScrollNodeIndex(self.clip_scroll_nodes.len() - 1);
        let info = self.stacking_context_info
            .entry(self.current_real_stacking_context_id)
            .or_insert(StackingContextInfo::new());
        info.clip_scroll_nodes.push(index);
        index
    }
}

pub struct DisplayListBuildState<'a> {
    /// A LayoutContext reference important for creating WebRender images.
    pub layout_context: &'a LayoutContext<'a>,

    /// The root of the StackingContext tree.
    pub root_stacking_context: StackingContext,

    /// StackingContext and ClipScrollNode children for each StackingContext.
    stacking_context_info: FnvHashMap<StackingContextId, StackingContextInfo>,

    /// A vector of ClipScrollNodes which will be given ids during WebRender DL conversion.
    pub clip_scroll_nodes: Vec<ClipScrollNode>,

    /// The items in this display list.
    pub items: FnvHashMap<StackingContextId, Vec<DisplayItem>>,

    /// Whether or not we are processing an element that establishes scrolling overflow. Used
    /// to determine what ClipScrollNode to place backgrounds and borders into.
    pub processing_scrolling_overflow_element: bool,

    /// The current stacking context id, used to keep track of state when building.
    /// recursively building and processing the display list.
    pub current_stacking_context_id: StackingContextId,

    /// The current clip and scroll info, used to keep track of state when
    /// recursively building and processing the display list.
    pub current_clipping_and_scrolling: ClippingAndScrolling,

    /// Vector containing iframe sizes, used to inform the constellation about
    /// new iframe sizes
    pub iframe_sizes: Vec<(BrowsingContextId, TypedSize2D<f32, CSSPixel>)>,
}

impl<'a> DisplayListBuildState<'a> {
    pub fn new(
        layout_context: &'a LayoutContext,
        state: StackingContextCollectionState,
    ) -> DisplayListBuildState<'a> {
        let root_clip_indices = ClippingAndScrolling::simple(ClipScrollNodeIndex(0));
        DisplayListBuildState {
            layout_context: layout_context,
            root_stacking_context: state.root_stacking_context,
            items: FnvHashMap::default(),
            stacking_context_info: state.stacking_context_info,
            clip_scroll_nodes: state.clip_scroll_nodes,
            processing_scrolling_overflow_element: false,
            current_stacking_context_id: StackingContextId::root(),
            current_clipping_and_scrolling: root_clip_indices,
            iframe_sizes: Vec::new(),
        }
    }

    fn add_display_item(&mut self, display_item: DisplayItem) {
        let items = self.items
            .entry(display_item.stacking_context_id())
            .or_insert(Vec::new());
        items.push(display_item);
    }

    fn parent_clip_scroll_node_index(&self, index: ClipScrollNodeIndex) -> ClipScrollNodeIndex {
        if index.is_root_scroll_node() {
            return index;
        }

        self.clip_scroll_nodes[index.0].parent_index
    }

    fn is_background_or_border_of_clip_scroll_node(&self, section: DisplayListSection) -> bool {
        (section == DisplayListSection::BackgroundAndBorders ||
            section == DisplayListSection::BlockBackgroundsAndBorders) &&
            self.processing_scrolling_overflow_element
    }

    fn create_base_display_item(
        &self,
        bounds: &Rect<Au>,
        clip: LocalClip,
        node: OpaqueNode,
        cursor: Option<Cursor>,
        section: DisplayListSection,
    ) -> BaseDisplayItem {
        let clipping_and_scrolling = if self.is_background_or_border_of_clip_scroll_node(section) {
            ClippingAndScrolling::simple(self.parent_clip_scroll_node_index(
                self.current_clipping_and_scrolling.scrolling,
            ))
        } else {
            self.current_clipping_and_scrolling
        };

        BaseDisplayItem::new(
            &bounds,
            DisplayItemMetadata {
                node: node,
                pointing: cursor,
            },
            clip,
            section,
            self.current_stacking_context_id,
            clipping_and_scrolling,
        )
    }

    pub fn to_display_list(mut self) -> DisplayList {
        let mut list = Vec::new();
        let root_context = mem::replace(&mut self.root_stacking_context, StackingContext::root());

        self.to_display_list_for_stacking_context(&mut list, root_context);

        DisplayList {
            list: list,
            clip_scroll_nodes: self.clip_scroll_nodes,
        }
    }

    fn to_display_list_for_stacking_context(
        &mut self,
        list: &mut Vec<DisplayItem>,
        stacking_context: StackingContext,
    ) {
        let mut child_items = self.items
            .remove(&stacking_context.id)
            .unwrap_or(Vec::new());
        child_items.sort_by(|a, b| a.base().section.cmp(&b.base().section));
        child_items.reverse();

        let mut info = self.stacking_context_info
            .remove(&stacking_context.id)
            .unwrap_or_else(StackingContextInfo::new);

        info.children.sort();

        if stacking_context.context_type != StackingContextType::Real {
            list.extend(
                info.clip_scroll_nodes
                    .into_iter()
                    .map(|index| index.to_define_item()),
            );
            self.to_display_list_for_items(list, child_items, info.children);
        } else {
            let (push_item, pop_item) = stacking_context.to_display_list_items();
            list.push(push_item);
            list.extend(
                info.clip_scroll_nodes
                    .into_iter()
                    .map(|index| index.to_define_item()),
            );
            self.to_display_list_for_items(list, child_items, info.children);
            list.push(pop_item);
        }
    }

    fn to_display_list_for_items(
        &mut self,
        list: &mut Vec<DisplayItem>,
        mut child_items: Vec<DisplayItem>,
        child_stacking_contexts: Vec<StackingContext>,
    ) {
        // Properly order display items that make up a stacking context. "Steps" here
        // refer to the steps in CSS 2.1 Appendix E.
        // Steps 1 and 2: Borders and background for the root.
        while child_items.last().map_or(false, |child| {
            child.section() == DisplayListSection::BackgroundAndBorders
        }) {
            list.push(child_items.pop().unwrap());
        }

        // Step 3: Positioned descendants with negative z-indices.
        let mut child_stacking_contexts = child_stacking_contexts.into_iter().peekable();
        while child_stacking_contexts
            .peek()
            .map_or(false, |child| child.z_index < 0)
        {
            let context = child_stacking_contexts.next().unwrap();
            self.to_display_list_for_stacking_context(list, context);
        }

        // Step 4: Block backgrounds and borders.
        while child_items.last().map_or(false, |child| {
            child.section() == DisplayListSection::BlockBackgroundsAndBorders
        }) {
            list.push(child_items.pop().unwrap());
        }

        // Step 5: Floats.
        while child_stacking_contexts.peek().map_or(false, |child| {
            child.context_type == StackingContextType::PseudoFloat
        }) {
            let context = child_stacking_contexts.next().unwrap();
            self.to_display_list_for_stacking_context(list, context);
        }

        // Step 6 & 7: Content and inlines that generate stacking contexts.
        while child_items.last().map_or(false, |child| {
            child.section() == DisplayListSection::Content
        }) {
            list.push(child_items.pop().unwrap());
        }

        // Step 8 & 9: Positioned descendants with nonnegative, numeric z-indices.
        for child in child_stacking_contexts {
            self.to_display_list_for_stacking_context(list, child);
        }

        // Step 10: Outlines.
        for item in child_items.drain(..) {
            list.push(item);
        }
    }
}

/// The logical width of an insertion point: at the moment, a one-pixel-wide line.
const INSERTION_POINT_LOGICAL_WIDTH: Au = Au(1 * AU_PER_PX);

pub trait FragmentDisplayListBuilding {
    fn collect_stacking_contexts_for_blocklike_fragment(
        &mut self,
        state: &mut StackingContextCollectionState,
    ) -> bool;

    /// Adds the display items necessary to paint the background of this fragment to the display
    /// list if necessary.
    fn build_display_list_for_background_if_applicable(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        display_list_section: DisplayListSection,
        absolute_bounds: &Rect<Au>,
    );

    /// Determines where to place an element background image or gradient.
    ///
    /// Photos have their resolution as intrinsic size while gradients have
    /// no intrinsic size.
    fn compute_background_placement(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        absolute_bounds: Rect<Au>,
        intrinsic_size: Option<Size2D<Au>>,
        index: usize,
    ) -> BackgroundPlacement;

    /// Adds the display items necessary to paint a webrender image of this fragment to the
    /// appropriate section of the display list.
    fn build_display_list_for_webrender_image(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        display_list_section: DisplayListSection,
        absolute_bounds: Rect<Au>,
        webrender_image: WebRenderImageInfo,
        index: usize,
    );

    /// Calculates the webrender image for a paint worklet.
    /// Returns None if the worklet is not registered.
    /// If the worklet has missing image URLs, it passes them to the image cache for loading.
    fn get_webrender_image_for_paint_worklet(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        paint_worklet: &PaintWorklet,
        size: Size2D<Au>,
    ) -> Option<WebRenderImageInfo>;

    /// Adds the display items necessary to paint the background linear gradient of this fragment
    /// to the appropriate section of the display list.
    fn build_display_list_for_background_gradient(
        &self,
        state: &mut DisplayListBuildState,
        display_list_section: DisplayListSection,
        absolute_bounds: Rect<Au>,
        gradient: &Gradient,
        style: &ComputedValues,
        index: usize,
    );

    /// Adds the display items necessary to paint the borders of this fragment to a display list if
    /// necessary.
    fn build_display_list_for_borders_if_applicable(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        inline_node_info: Option<InlineNodeBorderInfo>,
        border_painting_mode: BorderPaintingMode,
        bounds: &Rect<Au>,
        display_list_section: DisplayListSection,
        clip: &Rect<Au>,
    );

    /// Adds the display items necessary to paint the outline of this fragment to the display list
    /// if necessary.
    fn build_display_list_for_outline_if_applicable(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        bounds: &Rect<Au>,
        clip: &Rect<Au>,
    );

    /// Adds the display items necessary to paint the box shadow of this fragment to the display
    /// list if necessary.
    fn build_display_list_for_box_shadow_if_applicable(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        display_list_section: DisplayListSection,
        absolute_bounds: &Rect<Au>,
        clip: &Rect<Au>,
    );

    /// Adds display items necessary to draw debug boxes around a scanned text fragment.
    fn build_debug_borders_around_text_fragments(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        stacking_relative_border_box: &Rect<Au>,
        stacking_relative_content_box: &Rect<Au>,
        text_fragment: &ScannedTextFragmentInfo,
        clip: &Rect<Au>,
    );

    /// Adds display items necessary to draw debug boxes around this fragment.
    fn build_debug_borders_around_fragment(
        &self,
        state: &mut DisplayListBuildState,
        stacking_relative_border_box: &Rect<Au>,
        clip: &Rect<Au>,
    );

    /// Adds the display items for this fragment to the given display list.
    ///
    /// Arguments:
    ///
    /// * `state`: The display building state, including the display list currently
    ///   under construction and other metadata useful for constructing it.
    /// * `dirty`: The dirty rectangle in the coordinate system of the owning flow.
    /// * `stacking_relative_flow_origin`: Position of the origin of the owning flow with respect
    ///   to its nearest ancestor stacking context.
    /// * `relative_containing_block_size`: The size of the containing block that
    ///   `position: relative` makes use of.
    /// * `clip`: The region to clip the display items to.
    fn build_display_list(
        &mut self,
        state: &mut DisplayListBuildState,
        stacking_relative_flow_origin: &Vector2D<Au>,
        relative_containing_block_size: &LogicalSize<Au>,
        relative_containing_block_mode: WritingMode,
        border_painting_mode: BorderPaintingMode,
        display_list_section: DisplayListSection,
        clip: &Rect<Au>,
    );

    /// Builds the display items necessary to paint the selection and/or caret for this fragment,
    /// if any.
    fn build_display_items_for_selection_if_necessary(
        &self,
        state: &mut DisplayListBuildState,
        stacking_relative_border_box: &Rect<Au>,
        display_list_section: DisplayListSection,
        clip: &Rect<Au>,
    );

    /// Creates the text display item for one text fragment. This can be called multiple times for
    /// one fragment if there are text shadows.
    ///
    /// `text_shadow` will be `Some` if this is rendering a shadow.
    fn build_display_list_for_text_fragment(
        &self,
        state: &mut DisplayListBuildState,
        text_fragment: &ScannedTextFragmentInfo,
        stacking_relative_content_box: &Rect<Au>,
        text_shadows: &[SimpleShadow],
        clip: &Rect<Au>,
    );

    /// Creates the display item for a text decoration: underline, overline, or line-through.
    fn build_display_list_for_text_decoration(
        &self,
        state: &mut DisplayListBuildState,
        color: &RGBA,
        stacking_relative_box: &LogicalRect<Au>,
        clip: &Rect<Au>,
    );

    /// A helper method that `build_display_list` calls to create per-fragment-type display items.
    fn build_fragment_type_specific_display_items(
        &mut self,
        state: &mut DisplayListBuildState,
        stacking_relative_border_box: &Rect<Au>,
        clip: &Rect<Au>,
    );

    /// Creates a stacking context for associated fragment.
    fn create_stacking_context(
        &self,
        id: StackingContextId,
        base_flow: &BaseFlow,
        scroll_policy: ScrollPolicy,
        context_type: StackingContextType,
        parent_clipping_and_scrolling: ClippingAndScrolling,
    ) -> StackingContext;

    fn unique_id(&self) -> u64;

    fn fragment_type(&self) -> FragmentType;
}

fn handle_overlapping_radii(size: &Size2D<Au>, radii: &BorderRadii<Au>) -> BorderRadii<Au> {
    // No two corners' border radii may add up to more than the length of the edge
    // between them. To prevent that, all radii are scaled down uniformly.
    fn scale_factor(radius_a: Au, radius_b: Au, edge_length: Au) -> f32 {
        let required = radius_a + radius_b;

        if required <= edge_length {
            1.0
        } else {
            edge_length.to_f32_px() / required.to_f32_px()
        }
    }

    let top_factor = scale_factor(radii.top_left.width, radii.top_right.width, size.width);
    let bottom_factor = scale_factor(
        radii.bottom_left.width,
        radii.bottom_right.width,
        size.width,
    );
    let left_factor = scale_factor(radii.top_left.height, radii.bottom_left.height, size.height);
    let right_factor = scale_factor(
        radii.top_right.height,
        radii.bottom_right.height,
        size.height,
    );
    let min_factor = top_factor
        .min(bottom_factor)
        .min(left_factor)
        .min(right_factor);
    if min_factor < 1.0 {
        radii.scale_by(min_factor)
    } else {
        *radii
    }
}

fn build_border_radius(
    abs_bounds: &Rect<Au>,
    border_style: &style_structs::Border,
) -> BorderRadii<Au> {
    // TODO(cgaebel): Support border radii even in the case of multiple border widths.
    // This is an extension of supporting elliptical radii. For now, all percentage
    // radii will be relative to the width.

    handle_overlapping_radii(
        &abs_bounds.size,
        &BorderRadii {
            top_left: model::specified_border_radius(
                border_style.border_top_left_radius,
                abs_bounds.size,
            ),
            top_right: model::specified_border_radius(
                border_style.border_top_right_radius,
                abs_bounds.size,
            ),
            bottom_right: model::specified_border_radius(
                border_style.border_bottom_right_radius,
                abs_bounds.size,
            ),
            bottom_left: model::specified_border_radius(
                border_style.border_bottom_left_radius,
                abs_bounds.size,
            ),
        },
    )
}

/// Get the border radius for the rectangle inside of a rounded border. This is useful
/// for building the clip for the content inside the border.
fn build_border_radius_for_inner_rect(
    outer_rect: &Rect<Au>,
    style: &ComputedValues,
) -> BorderRadii<Au> {
    let radii = build_border_radius(&outer_rect, style.get_border());
    if radii.is_square() {
        return radii;
    }

    // Since we are going to using the inner rectangle (outer rectangle minus
    // border width), we need to adjust to border radius so that we are smaller
    // rectangle with the same border curve.
    let border_widths = style.logical_border_width().to_physical(style.writing_mode);
    calculate_inner_border_radii(radii, border_widths)
}

fn build_inner_border_box_for_border_rect(
    border_box: &Rect<Au>,
    style: &ComputedValues,
) -> Rect<Au> {
    let border_widths = style.logical_border_width().to_physical(style.writing_mode);
    let mut inner_border_box = *border_box;
    inner_border_box.origin.x += border_widths.left;
    inner_border_box.origin.y += border_widths.top;
    inner_border_box.size.width -= border_widths.right + border_widths.left;
    inner_border_box.size.height -= border_widths.bottom + border_widths.top;
    inner_border_box
}

/// Subtract offsets from a bounding box.
///
/// As an example if the bounds are the border-box and the border
/// is provided as offsets the result will be the padding-box.
fn calculate_inner_bounds(mut bounds: Rect<Au>, offsets: SideOffsets2D<Au>) -> Rect<Au> {
    bounds.origin.x += offsets.left;
    bounds.origin.y += offsets.top;
    bounds.size.width -= offsets.horizontal();
    bounds.size.height -= offsets.vertical();
    bounds
}

fn calculate_inner_border_radii(
    mut radii: BorderRadii<Au>,
    offsets: SideOffsets2D<Au>,
) -> BorderRadii<Au> {
    radii.top_left.width = cmp::max(Au(0), radii.top_left.width - offsets.left);
    radii.bottom_left.width = cmp::max(Au(0), radii.bottom_left.width - offsets.left);

    radii.top_right.width = cmp::max(Au(0), radii.top_right.width - offsets.right);
    radii.bottom_right.width = cmp::max(Au(0), radii.bottom_right.width - offsets.right);

    radii.top_left.height = cmp::max(Au(0), radii.top_left.height - offsets.top);
    radii.top_right.height = cmp::max(Au(0), radii.top_right.height - offsets.top);

    radii.bottom_left.height = cmp::max(Au(0), radii.bottom_left.height - offsets.bottom);
    radii.bottom_right.height = cmp::max(Au(0), radii.bottom_right.height - offsets.bottom);
    radii
}

impl FragmentDisplayListBuilding for Fragment {
    fn collect_stacking_contexts_for_blocklike_fragment(
        &mut self,
        state: &mut StackingContextCollectionState,
    ) -> bool {
        match self.specific {
            SpecificFragmentInfo::InlineBlock(ref mut block_flow) => {
                let block_flow = FlowRef::deref_mut(&mut block_flow.flow_ref);
                block_flow.collect_stacking_contexts(state);
                true
            },
            SpecificFragmentInfo::InlineAbsoluteHypothetical(ref mut block_flow) => {
                let block_flow = FlowRef::deref_mut(&mut block_flow.flow_ref);
                block_flow.collect_stacking_contexts(state);
                true
            },
            SpecificFragmentInfo::InlineAbsolute(ref mut block_flow) => {
                let block_flow = FlowRef::deref_mut(&mut block_flow.flow_ref);
                block_flow.collect_stacking_contexts(state);
                true
            },
            // FIXME: In the future, if #15144 is fixed we can remove this case. See #18510.
            SpecificFragmentInfo::TruncatedFragment(ref mut info) => info.full
                .collect_stacking_contexts_for_blocklike_fragment(state),
            _ => false,
        }
    }

    fn build_display_list_for_background_if_applicable(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        display_list_section: DisplayListSection,
        absolute_bounds: &Rect<Au>,
    ) {
        // FIXME: This causes a lot of background colors to be displayed when they are clearly not
        // needed. We could use display list optimization to clean this up, but it still seems
        // inefficient. What we really want is something like "nearest ancestor element that
        // doesn't have a fragment".
        let background = style.get_background();
        let background_color = style.resolve_color(background.background_color);

        // 'background-clip' determines the area within which the background is painted.
        // http://dev.w3.org/csswg/css-backgrounds-3/#the-background-clip
        let mut bounds = *absolute_bounds;

        // This is the clip for the color (which is the last element in the bg array)
        let color_clip = get_cyclic(
            &background.background_clip.0,
            background.background_image.0.len() - 1,
        );

        // Adjust the clipping region as necessary to account for `border-radius`.
        let mut border_radii = build_border_radius(absolute_bounds, style.get_border());

        match *color_clip {
            BackgroundClip::BorderBox => {},
            BackgroundClip::PaddingBox => {
                let border = style.logical_border_width().to_physical(style.writing_mode);
                bounds = calculate_inner_bounds(bounds, border);
                border_radii = calculate_inner_border_radii(border_radii, border);
            },
            BackgroundClip::ContentBox => {
                let border_padding = self.border_padding.to_physical(style.writing_mode);
                bounds = calculate_inner_bounds(bounds, border_padding);
                border_radii = calculate_inner_border_radii(border_radii, border_padding);
            },
        }

        let clip = if !border_radii.is_square() {
            LocalClip::RoundedRect(
                bounds.to_layout(),
                ComplexClipRegion::new(
                    bounds.to_layout(),
                    border_radii.to_border_radius(),
                    ClipMode::Clip,
                ),
            )
        } else {
            LocalClip::Rect(bounds.to_layout())
        };

        let base = state.create_base_display_item(
            &bounds,
            clip,
            self.node,
            style.get_cursor(Cursor::Default),
            display_list_section,
        );
        state.add_display_item(DisplayItem::SolidColor(Box::new(SolidColorDisplayItem {
            base: base,
            color: background_color.to_layout(),
        })));

        // The background image is painted on top of the background color.
        // Implements background image, per spec:
        // http://www.w3.org/TR/CSS21/colors.html#background
        let background = style.get_background();
        for (i, background_image) in background.background_image.0.iter().enumerate().rev() {
            match *background_image {
                Either::First(_) => {},
                Either::Second(Image::Gradient(ref gradient)) => {
                    self.build_display_list_for_background_gradient(
                        state,
                        display_list_section,
                        *absolute_bounds,
                        gradient,
                        style,
                        i,
                    );
                },
                Either::Second(Image::Url(ref image_url)) => {
                    if let Some(url) = image_url.url() {
                        let webrender_image = state.layout_context.get_webrender_image_for_url(
                            self.node,
                            url.clone(),
                            UsePlaceholder::No,
                        );
                        if let Some(webrender_image) = webrender_image {
                            self.build_display_list_for_webrender_image(
                                state,
                                style,
                                display_list_section,
                                *absolute_bounds,
                                webrender_image,
                                i,
                            );
                        }
                    }
                },
                Either::Second(Image::PaintWorklet(ref paint_worklet)) => {
                    let bounding_box = self.border_box - style.logical_border_width();
                    let bounding_box_size = bounding_box.size.to_physical(style.writing_mode);
                    let background_size =
                        get_cyclic(&style.get_background().background_size.0, i).clone();
                    let size = match background_size {
                        BackgroundSize::Explicit { width, height } => Size2D::new(
                            MaybeAuto::from_style(width, bounding_box_size.width)
                                .specified_or_default(bounding_box_size.width),
                            MaybeAuto::from_style(height, bounding_box_size.height)
                                .specified_or_default(bounding_box_size.height),
                        ),
                        _ => bounding_box_size,
                    };
                    let webrender_image = self.get_webrender_image_for_paint_worklet(
                        state,
                        style,
                        paint_worklet,
                        size,
                    );
                    if let Some(webrender_image) = webrender_image {
                        self.build_display_list_for_webrender_image(
                            state,
                            style,
                            display_list_section,
                            *absolute_bounds,
                            webrender_image,
                            i,
                        );
                    }
                },
                Either::Second(Image::Rect(_)) => {
                    // TODO: Implement `-moz-image-rect`
                },
                Either::Second(Image::Element(_)) => {
                    // TODO: Implement `-moz-element`
                },
            }
        }
    }

    fn compute_background_placement(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        absolute_bounds: Rect<Au>,
        intrinsic_size: Option<Size2D<Au>>,
        index: usize,
    ) -> BackgroundPlacement {
        let bg = style.get_background();
        let bg_attachment = *get_cyclic(&bg.background_attachment.0, index);
        let bg_clip = *get_cyclic(&bg.background_clip.0, index);
        let bg_origin = *get_cyclic(&bg.background_origin.0, index);
        let bg_position_x = get_cyclic(&bg.background_position_x.0, index);
        let bg_position_y = get_cyclic(&bg.background_position_y.0, index);
        let bg_repeat = get_cyclic(&bg.background_repeat.0, index);
        let bg_size = *get_cyclic(&bg.background_size.0, index);

        let css_clip = match bg_clip {
            BackgroundClip::BorderBox => absolute_bounds,
            BackgroundClip::PaddingBox => calculate_inner_bounds(
                absolute_bounds,
                style.logical_border_width().to_physical(style.writing_mode),
            ),
            BackgroundClip::ContentBox => calculate_inner_bounds(
                absolute_bounds,
                self.border_padding.to_physical(style.writing_mode),
            ),
        };

        let mut bounds = match bg_attachment {
            BackgroundAttachment::Scroll => match bg_origin {
                BackgroundOrigin::BorderBox => absolute_bounds,
                BackgroundOrigin::PaddingBox => calculate_inner_bounds(
                    absolute_bounds,
                    style.logical_border_width().to_physical(style.writing_mode),
                ),
                BackgroundOrigin::ContentBox => calculate_inner_bounds(
                    absolute_bounds,
                    self.border_padding.to_physical(style.writing_mode),
                ),
            },
            BackgroundAttachment::Fixed => Rect::new(
                Point2D::origin(),
                // Get current viewport
                state.layout_context.shared_context().viewport_size(),
            ),
        };

        let mut tile_size = compute_background_image_size(bg_size, bounds.size, intrinsic_size);

        let mut tile_spacing = Size2D::zero();
        let own_position = bounds.size - tile_size;
        let pos_x = bg_position_x.to_used_value(own_position.width);
        let pos_y = bg_position_y.to_used_value(own_position.height);
        tile_image_axis(
            bg_repeat.0,
            &mut bounds.origin.x,
            &mut bounds.size.width,
            &mut tile_size.width,
            &mut tile_spacing.width,
            pos_x,
            css_clip.origin.x,
            css_clip.size.width,
        );
        tile_image_axis(
            bg_repeat.1,
            &mut bounds.origin.y,
            &mut bounds.size.height,
            &mut tile_size.height,
            &mut tile_spacing.height,
            pos_y,
            css_clip.origin.y,
            css_clip.size.height,
        );

        BackgroundPlacement {
            bounds,
            tile_size,
            tile_spacing,
            css_clip,
        }
    }

    fn build_display_list_for_webrender_image(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        display_list_section: DisplayListSection,
        absolute_bounds: Rect<Au>,
        webrender_image: WebRenderImageInfo,
        index: usize,
    ) {
        debug!("(building display list) building background image");

        let image = Size2D::new(
            Au::from_px(webrender_image.width as i32),
            Au::from_px(webrender_image.height as i32),
        );
        let placement =
            self.compute_background_placement(state, style, absolute_bounds, Some(image), index);

        // Create the image display item.
        let base = state.create_base_display_item(
            &placement.bounds,
            LocalClip::Rect(placement.css_clip.to_layout()),
            self.node,
            style.get_cursor(Cursor::Default),
            display_list_section,
        );

        debug!("(building display list) adding background image.");
        state.add_display_item(DisplayItem::Image(Box::new(ImageDisplayItem {
            base: base,
            webrender_image: webrender_image,
            image_data: None,
            stretch_size: placement.tile_size,
            tile_spacing: placement.tile_spacing,
            image_rendering: style.get_inheritedbox().image_rendering.clone(),
        })));
    }

    fn get_webrender_image_for_paint_worklet(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        paint_worklet: &PaintWorklet,
        size_in_au: Size2D<Au>,
    ) -> Option<WebRenderImageInfo> {
        let device_pixel_ratio = state.layout_context.style_context.device_pixel_ratio();
        let size_in_px =
            TypedSize2D::new(size_in_au.width.to_f32_px(), size_in_au.height.to_f32_px());

        // TODO: less copying.
        let name = paint_worklet.name.clone();
        let arguments = paint_worklet
            .arguments
            .iter()
            .map(|argument| argument.to_css_string())
            .collect();

        let draw_result = match state.layout_context.registered_painters.get(&name) {
            Some(painter) => {
                debug!(
                    "Drawing a paint image {}({},{}).",
                    name, size_in_px.width, size_in_px.height
                );
                let properties = painter
                    .properties()
                    .iter()
                    .filter_map(|(name, id)| id.as_shorthand().err().map(|id| (name, id)))
                    .map(|(name, id)| (name.clone(), style.computed_value_to_string(id)))
                    .collect();
                painter.draw_a_paint_image(size_in_px, device_pixel_ratio, properties, arguments)
            },
            None => {
                debug!("Worklet {} called before registration.", name);
                return None;
            },
        };

        if let Ok(draw_result) = draw_result {
            let webrender_image = WebRenderImageInfo {
                width: draw_result.width,
                height: draw_result.height,
                format: draw_result.format,
                key: draw_result.image_key,
            };

            for url in draw_result.missing_image_urls.into_iter() {
                debug!("Requesting missing image URL {}.", url);
                state.layout_context.get_webrender_image_for_url(
                    self.node,
                    url,
                    UsePlaceholder::No,
                );
            }
            Some(webrender_image)
        } else {
            None
        }
    }

    fn build_display_list_for_background_gradient(
        &self,
        state: &mut DisplayListBuildState,
        display_list_section: DisplayListSection,
        absolute_bounds: Rect<Au>,
        gradient: &Gradient,
        style: &ComputedValues,
        index: usize,
    ) {
        let placement =
            self.compute_background_placement(state, style, absolute_bounds, None, index);

        let base = state.create_base_display_item(
            &placement.bounds,
            LocalClip::Rect(placement.css_clip.to_layout()),
            self.node,
            style.get_cursor(Cursor::Default),
            display_list_section,
        );

        let display_item = match gradient.kind {
            GradientKind::Linear(angle_or_corner) => {
                let gradient = convert_linear_gradient(
                    placement.tile_size,
                    &gradient.items[..],
                    angle_or_corner,
                    gradient.repeating,
                );
                DisplayItem::Gradient(Box::new(GradientDisplayItem {
                    base: base,
                    gradient: gradient,
                    tile: placement.tile_size,
                    tile_spacing: placement.tile_spacing,
                }))
            },
            GradientKind::Radial(shape, center, _angle) => {
                let gradient = convert_radial_gradient(
                    placement.tile_size,
                    &gradient.items[..],
                    shape,
                    center,
                    gradient.repeating,
                );
                DisplayItem::RadialGradient(Box::new(RadialGradientDisplayItem {
                    base: base,
                    gradient: gradient,
                    tile: placement.tile_size,
                    tile_spacing: placement.tile_spacing,
                }))
            },
        };
        state.add_display_item(display_item);
    }

    fn build_display_list_for_box_shadow_if_applicable(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        display_list_section: DisplayListSection,
        absolute_bounds: &Rect<Au>,
        clip: &Rect<Au>,
    ) {
        // NB: According to CSS-BACKGROUNDS, box shadows render in *reverse* order (front to back).
        for box_shadow in style.get_effects().box_shadow.0.iter().rev() {
            let bounds = shadow_bounds(
                &absolute_bounds.translate(&Vector2D::new(
                    Au::from(box_shadow.base.horizontal),
                    Au::from(box_shadow.base.vertical),
                )),
                Au::from(box_shadow.base.blur),
                Au::from(box_shadow.spread),
            );

            let base = state.create_base_display_item(
                &bounds,
                LocalClip::from(clip.to_layout()),
                self.node,
                style.get_cursor(Cursor::Default),
                display_list_section,
            );
            let border_radius = build_border_radius(absolute_bounds, style.get_border());
            state.add_display_item(DisplayItem::BoxShadow(Box::new(BoxShadowDisplayItem {
                base: base,
                box_bounds: *absolute_bounds,
                color: box_shadow
                    .base
                    .color
                    .unwrap_or(style.get_color().color)
                    .to_layout(),
                offset: Vector2D::new(
                    Au::from(box_shadow.base.horizontal),
                    Au::from(box_shadow.base.vertical),
                ),
                blur_radius: Au::from(box_shadow.base.blur),
                spread_radius: Au::from(box_shadow.spread),
                border_radius,
                clip_mode: if box_shadow.inset {
                    BoxShadowClipMode::Inset
                } else {
                    BoxShadowClipMode::Outset
                },
            })));
        }
    }

    fn build_display_list_for_borders_if_applicable(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        inline_info: Option<InlineNodeBorderInfo>,
        border_painting_mode: BorderPaintingMode,
        bounds: &Rect<Au>,
        display_list_section: DisplayListSection,
        clip: &Rect<Au>,
    ) {
        let mut border = style.logical_border_width();

        if let Some(inline_info) = inline_info {
            modify_border_width_for_inline_sides(&mut border, inline_info);
        }

        match border_painting_mode {
            BorderPaintingMode::Separate => {},
            BorderPaintingMode::Collapse(collapsed_borders) => {
                collapsed_borders.adjust_border_widths_for_painting(&mut border)
            },
            BorderPaintingMode::Hidden => return,
        }
        if border.is_zero() {
            // TODO: check if image-border-outset is zero
            return;
        }

        let border_style_struct = style.get_border();
        let mut colors = SideOffsets2D::new(
            border_style_struct.border_top_color,
            border_style_struct.border_right_color,
            border_style_struct.border_bottom_color,
            border_style_struct.border_left_color,
        );
        let mut border_style = SideOffsets2D::new(
            border_style_struct.border_top_style,
            border_style_struct.border_right_style,
            border_style_struct.border_bottom_style,
            border_style_struct.border_left_style,
        );

        if let BorderPaintingMode::Collapse(collapsed_borders) = border_painting_mode {
            collapsed_borders.adjust_border_colors_and_styles_for_painting(
                &mut colors,
                &mut border_style,
                style.writing_mode,
            );
        }

        let colors = SideOffsets2D::new(
            style.resolve_color(colors.top),
            style.resolve_color(colors.right),
            style.resolve_color(colors.bottom),
            style.resolve_color(colors.left),
        );

        // If this border collapses, then we draw outside the boundaries we were given.
        let mut bounds = *bounds;
        if let BorderPaintingMode::Collapse(collapsed_borders) = border_painting_mode {
            collapsed_borders.adjust_border_bounds_for_painting(&mut bounds, style.writing_mode)
        }

        // Append the border to the display list.
        let base = state.create_base_display_item(
            &bounds,
            LocalClip::from(clip.to_layout()),
            self.node,
            style.get_cursor(Cursor::Default),
            display_list_section,
        );

        match border_style_struct.border_image_source {
            Either::First(_) => {
                state.add_display_item(DisplayItem::Border(Box::new(BorderDisplayItem {
                    base: base,
                    border_widths: border.to_physical(style.writing_mode),
                    details: BorderDetails::Normal(NormalBorder {
                        color: SideOffsets2D::new(
                            colors.top.to_layout(),
                            colors.right.to_layout(),
                            colors.bottom.to_layout(),
                            colors.left.to_layout(),
                        ),
                        style: border_style,
                        radius: build_border_radius(&bounds, border_style_struct),
                    }),
                })));
            },
            Either::Second(Image::Gradient(ref gradient)) => {
                let border_widths = border.to_physical(style.writing_mode);
                let details = match gradient.kind {
                    GradientKind::Linear(angle_or_corner) => {
                        BorderDetails::Gradient(display_list::GradientBorder {
                            gradient: convert_linear_gradient(
                                bounds.size,
                                &gradient.items[..],
                                angle_or_corner,
                                gradient.repeating,
                            ),
                            // TODO(gw): Support border-image-outset
                            outset: SideOffsets2D::zero(),
                        })
                    },
                    GradientKind::Radial(shape, center, _angle) => {
                        BorderDetails::RadialGradient(display_list::RadialGradientBorder {
                            gradient: convert_radial_gradient(
                                bounds.size,
                                &gradient.items[..],
                                shape,
                                center,
                                gradient.repeating,
                            ),
                            // TODO(gw): Support border-image-outset
                            outset: SideOffsets2D::zero(),
                        })
                    },
                };
                state.add_display_item(DisplayItem::Border(Box::new(BorderDisplayItem {
                    base,
                    border_widths,
                    details,
                })));
            },
            Either::Second(Image::PaintWorklet(ref paint_worklet)) => {
                // TODO: this size should be increased by border-image-outset
                let size = self.border_box.size.to_physical(style.writing_mode);
                let webrender_image =
                    self.get_webrender_image_for_paint_worklet(state, style, paint_worklet, size);
                if let Some(webrender_image) = webrender_image {
                    let corners = &border_style_struct.border_image_slice.offsets;

                    state.add_display_item(DisplayItem::Border(Box::new(BorderDisplayItem {
                        base: base,
                        border_widths: border.to_physical(style.writing_mode),
                        details: BorderDetails::Image(ImageBorder {
                            image: webrender_image,
                            fill: border_style_struct.border_image_slice.fill,
                            slice: SideOffsets2D::new(
                                corners.0.resolve(webrender_image.height),
                                corners.1.resolve(webrender_image.width),
                                corners.2.resolve(webrender_image.height),
                                corners.3.resolve(webrender_image.width),
                            ),
                            // TODO(gw): Support border-image-outset
                            outset: SideOffsets2D::zero(),
                            repeat_horizontal: convert_repeat_mode(
                                border_style_struct.border_image_repeat.0,
                            ),
                            repeat_vertical: convert_repeat_mode(
                                border_style_struct.border_image_repeat.1,
                            ),
                        }),
                    })));
                }
            },
            Either::Second(Image::Rect(..)) => {
                // TODO: Handle border-image with `-moz-image-rect`.
            },
            Either::Second(Image::Element(..)) => {
                // TODO: Handle border-image with `-moz-element`.
            },
            Either::Second(Image::Url(ref image_url)) => {
                if let Some(url) = image_url.url() {
                    let webrender_image = state.layout_context.get_webrender_image_for_url(
                        self.node,
                        url.clone(),
                        UsePlaceholder::No,
                    );
                    if let Some(webrender_image) = webrender_image {
                        let corners = &border_style_struct.border_image_slice.offsets;

                        state.add_display_item(DisplayItem::Border(Box::new(BorderDisplayItem {
                            base: base,
                            border_widths: border.to_physical(style.writing_mode),
                            details: BorderDetails::Image(ImageBorder {
                                image: webrender_image,
                                fill: border_style_struct.border_image_slice.fill,
                                slice: SideOffsets2D::new(
                                    corners.0.resolve(webrender_image.height),
                                    corners.1.resolve(webrender_image.width),
                                    corners.2.resolve(webrender_image.height),
                                    corners.3.resolve(webrender_image.width),
                                ),
                                // TODO(gw): Support border-image-outset
                                outset: SideOffsets2D::zero(),
                                repeat_horizontal: convert_repeat_mode(
                                    border_style_struct.border_image_repeat.0,
                                ),
                                repeat_vertical: convert_repeat_mode(
                                    border_style_struct.border_image_repeat.1,
                                ),
                            }),
                        })));
                    }
                }
            },
        }
    }

    fn build_display_list_for_outline_if_applicable(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        bounds: &Rect<Au>,
        clip: &Rect<Au>,
    ) {
        use style::values::specified::outline::OutlineStyle;

        let width = Au::from(style.get_outline().outline_width);
        if width == Au(0) {
            return;
        }

        let outline_style = match style.get_outline().outline_style {
            OutlineStyle::Auto => BorderStyle::Solid,
            OutlineStyle::Other(BorderStyle::None) => return,
            OutlineStyle::Other(border_style) => border_style,
        };

        // Outlines are not accounted for in the dimensions of the border box, so adjust the
        // absolute bounds.
        let mut bounds = *bounds;
        let offset = width + Au::from(style.get_outline().outline_offset);
        bounds.origin.x = bounds.origin.x - offset;
        bounds.origin.y = bounds.origin.y - offset;
        bounds.size.width = bounds.size.width + offset + offset;
        bounds.size.height = bounds.size.height + offset + offset;

        // Append the outline to the display list.
        let color = style
            .resolve_color(style.get_outline().outline_color)
            .to_layout();
        let base = state.create_base_display_item(
            &bounds,
            LocalClip::from(clip.to_layout()),
            self.node,
            style.get_cursor(Cursor::Default),
            DisplayListSection::Outlines,
        );
        state.add_display_item(DisplayItem::Border(Box::new(BorderDisplayItem {
            base: base,
            border_widths: SideOffsets2D::new_all_same(width),
            details: BorderDetails::Normal(NormalBorder {
                color: SideOffsets2D::new_all_same(color),
                style: SideOffsets2D::new_all_same(outline_style),
                radius: Default::default(),
            }),
        })));
    }

    fn build_debug_borders_around_text_fragments(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        stacking_relative_border_box: &Rect<Au>,
        stacking_relative_content_box: &Rect<Au>,
        text_fragment: &ScannedTextFragmentInfo,
        clip: &Rect<Au>,
    ) {
        // FIXME(pcwalton, #2795): Get the real container size.
        let container_size = Size2D::zero();

        // Compute the text fragment bounds and draw a border surrounding them.
        let base = state.create_base_display_item(
            stacking_relative_border_box,
            LocalClip::from(clip.to_layout()),
            self.node,
            style.get_cursor(Cursor::Default),
            DisplayListSection::Content,
        );
        state.add_display_item(DisplayItem::Border(Box::new(BorderDisplayItem {
            base: base,
            border_widths: SideOffsets2D::new_all_same(Au::from_px(1)),
            details: BorderDetails::Normal(NormalBorder {
                color: SideOffsets2D::new_all_same(ColorF::rgb(0, 0, 200)),
                style: SideOffsets2D::new_all_same(BorderStyle::Solid),
                radius: Default::default(),
            }),
        })));

        // Draw a rectangle representing the baselines.
        let mut baseline = LogicalRect::from_physical(
            self.style.writing_mode,
            *stacking_relative_content_box,
            container_size,
        );
        baseline.start.b = baseline.start.b + text_fragment.run.ascent();
        baseline.size.block = Au(0);
        let baseline = baseline.to_physical(self.style.writing_mode, container_size);

        let base = state.create_base_display_item(
            &baseline,
            LocalClip::from(clip.to_layout()),
            self.node,
            style.get_cursor(Cursor::Default),
            DisplayListSection::Content,
        );
        state.add_display_item(DisplayItem::Line(Box::new(LineDisplayItem {
            base: base,
            color: ColorF::rgb(0, 200, 0),
            style: LineStyle::Dashed,
        })));
    }

    fn build_debug_borders_around_fragment(
        &self,
        state: &mut DisplayListBuildState,
        stacking_relative_border_box: &Rect<Au>,
        clip: &Rect<Au>,
    ) {
        // This prints a debug border around the border of this fragment.
        let base = state.create_base_display_item(
            stacking_relative_border_box,
            LocalClip::from(clip.to_layout()),
            self.node,
            self.style.get_cursor(Cursor::Default),
            DisplayListSection::Content,
        );
        state.add_display_item(DisplayItem::Border(Box::new(BorderDisplayItem {
            base: base,
            border_widths: SideOffsets2D::new_all_same(Au::from_px(1)),
            details: BorderDetails::Normal(NormalBorder {
                color: SideOffsets2D::new_all_same(ColorF::rgb(0, 0, 200)),
                style: SideOffsets2D::new_all_same(BorderStyle::Solid),
                radius: Default::default(),
            }),
        })));
    }

    fn build_display_items_for_selection_if_necessary(
        &self,
        state: &mut DisplayListBuildState,
        stacking_relative_border_box: &Rect<Au>,
        display_list_section: DisplayListSection,
        clip: &Rect<Au>,
    ) {
        let scanned_text_fragment_info = match self.specific {
            SpecificFragmentInfo::ScannedText(ref scanned_text_fragment_info) => {
                scanned_text_fragment_info
            },
            _ => return,
        };

        // Draw a highlighted background if the text is selected.
        //
        // TODO: Allow non-text fragments to be selected too.
        if scanned_text_fragment_info.selected() {
            let style = self.selected_style();
            let background_color = style.resolve_color(style.get_background().background_color);
            let base = state.create_base_display_item(
                stacking_relative_border_box,
                LocalClip::from(clip.to_layout()),
                self.node,
                self.style.get_cursor(Cursor::Default),
                display_list_section,
            );
            state.add_display_item(DisplayItem::SolidColor(Box::new(SolidColorDisplayItem {
                base: base,
                color: background_color.to_layout(),
            })));
        }

        // Draw a caret at the insertion point.
        let insertion_point_index = match scanned_text_fragment_info.insertion_point {
            Some(insertion_point_index) => insertion_point_index,
            None => return,
        };
        let range = Range::new(
            scanned_text_fragment_info.range.begin(),
            insertion_point_index - scanned_text_fragment_info.range.begin(),
        );
        let advance = scanned_text_fragment_info.run.advance_for_range(&range);

        let insertion_point_bounds;
        let cursor;
        if !self.style.writing_mode.is_vertical() {
            insertion_point_bounds = Rect::new(
                Point2D::new(
                    stacking_relative_border_box.origin.x + advance,
                    stacking_relative_border_box.origin.y,
                ),
                Size2D::new(
                    INSERTION_POINT_LOGICAL_WIDTH,
                    stacking_relative_border_box.size.height,
                ),
            );
            cursor = Cursor::Text;
        } else {
            insertion_point_bounds = Rect::new(
                Point2D::new(
                    stacking_relative_border_box.origin.x,
                    stacking_relative_border_box.origin.y + advance,
                ),
                Size2D::new(
                    stacking_relative_border_box.size.width,
                    INSERTION_POINT_LOGICAL_WIDTH,
                ),
            );
            cursor = Cursor::VerticalText;
        };

        let base = state.create_base_display_item(
            &insertion_point_bounds,
            LocalClip::from(clip.to_layout()),
            self.node,
            self.style.get_cursor(cursor),
            display_list_section,
        );
        state.add_display_item(DisplayItem::SolidColor(Box::new(SolidColorDisplayItem {
            base: base,
            color: self.style().get_color().color.to_layout(),
        })));
    }

    fn build_display_list(
        &mut self,
        state: &mut DisplayListBuildState,
        stacking_relative_flow_origin: &Vector2D<Au>,
        relative_containing_block_size: &LogicalSize<Au>,
        relative_containing_block_mode: WritingMode,
        border_painting_mode: BorderPaintingMode,
        display_list_section: DisplayListSection,
        clip: &Rect<Au>,
    ) {
        self.restyle_damage.remove(ServoRestyleDamage::REPAINT);
        if self.style().get_inheritedbox().visibility != Visibility::Visible {
            return;
        }

        // Compute the fragment position relative to the parent stacking context. If the fragment
        // itself establishes a stacking context, then the origin of its position will be (0, 0)
        // for the purposes of this computation.
        let stacking_relative_border_box = self.stacking_relative_border_box(
            stacking_relative_flow_origin,
            relative_containing_block_size,
            relative_containing_block_mode,
            CoordinateSystem::Own,
        );

        debug!(
            "Fragment::build_display_list at rel={:?}, abs={:?}, flow origin={:?}: {:?}",
            self.border_box, stacking_relative_border_box, stacking_relative_flow_origin, self
        );

        // Check the clip rect. If there's nothing to render at all, don't even construct display
        // list items.
        let empty_rect = !clip.intersects(&stacking_relative_border_box);
        if self.is_primary_fragment() && !empty_rect {
            // Add shadows, background, borders, and outlines, if applicable.
            if let Some(ref inline_context) = self.inline_context {
                for node in inline_context.nodes.iter().rev() {
                    self.build_display_list_for_background_if_applicable(
                        state,
                        &*node.style,
                        display_list_section,
                        &stacking_relative_border_box,
                    );

                    self.build_display_list_for_box_shadow_if_applicable(
                        state,
                        &*node.style,
                        display_list_section,
                        &stacking_relative_border_box,
                        clip,
                    );

                    self.build_display_list_for_borders_if_applicable(
                        state,
                        &*node.style,
                        Some(InlineNodeBorderInfo {
                            is_first_fragment_of_element: node.flags
                                .contains(InlineFragmentNodeFlags::FIRST_FRAGMENT_OF_ELEMENT),
                            is_last_fragment_of_element: node.flags
                                .contains(InlineFragmentNodeFlags::LAST_FRAGMENT_OF_ELEMENT),
                        }),
                        border_painting_mode,
                        &stacking_relative_border_box,
                        display_list_section,
                        clip,
                    );

                    // FIXME(emilio): Why does outline not do the same width
                    // fixup as border?
                    self.build_display_list_for_outline_if_applicable(
                        state,
                        &*node.style,
                        &stacking_relative_border_box,
                        clip,
                    );
                }
            }

            if !self.is_scanned_text_fragment() {
                self.build_display_list_for_background_if_applicable(
                    state,
                    &*self.style,
                    display_list_section,
                    &stacking_relative_border_box,
                );

                self.build_display_list_for_box_shadow_if_applicable(
                    state,
                    &*self.style,
                    display_list_section,
                    &stacking_relative_border_box,
                    clip,
                );

                self.build_display_list_for_borders_if_applicable(
                    state,
                    &*self.style,
                    /* inline_node_info = */ None,
                    border_painting_mode,
                    &stacking_relative_border_box,
                    display_list_section,
                    clip,
                );

                self.build_display_list_for_outline_if_applicable(
                    state,
                    &*self.style,
                    &stacking_relative_border_box,
                    clip,
                );
            }
        }

        if self.is_primary_fragment() {
            // Paint the selection point if necessary.  Even an empty text fragment may have an
            // insertion point, so we do this even if `empty_rect` is true.
            self.build_display_items_for_selection_if_necessary(
                state,
                &stacking_relative_border_box,
                display_list_section,
                clip,
            );
        }

        if empty_rect {
            return;
        }

        debug!("Fragment::build_display_list: intersected. Adding display item...");

        // Create special per-fragment-type display items.
        self.build_fragment_type_specific_display_items(state, &stacking_relative_border_box, clip);

        if opts::get().show_debug_fragment_borders {
            self.build_debug_borders_around_fragment(state, &stacking_relative_border_box, clip)
        }
    }

    fn build_fragment_type_specific_display_items(
        &mut self,
        state: &mut DisplayListBuildState,
        stacking_relative_border_box: &Rect<Au>,
        clip: &Rect<Au>,
    ) {
        // Compute the context box position relative to the parent stacking context.
        let stacking_relative_content_box =
            self.stacking_relative_content_box(stacking_relative_border_box);

        // Adjust the clipping region as necessary to account for `border-radius`.
        let build_local_clip = |style: &ComputedValues| {
            let radii = build_border_radius_for_inner_rect(&stacking_relative_border_box, style);
            if !radii.is_square() {
                LocalClip::RoundedRect(
                    stacking_relative_border_box.to_layout(),
                    ComplexClipRegion::new(
                        stacking_relative_content_box.to_layout(),
                        radii.to_border_radius(),
                        ClipMode::Clip,
                    ),
                )
            } else {
                LocalClip::Rect(stacking_relative_border_box.to_layout())
            }
        };

        match self.specific {
            SpecificFragmentInfo::TruncatedFragment(ref truncated_fragment)
                if truncated_fragment.text_info.is_some() =>
            {
                let text_fragment = truncated_fragment.text_info.as_ref().unwrap();
                // Create the main text display item.
                self.build_display_list_for_text_fragment(
                    state,
                    &text_fragment,
                    &stacking_relative_content_box,
                    &self.style.get_inheritedtext().text_shadow.0,
                    clip,
                );

                if opts::get().show_debug_fragment_borders {
                    self.build_debug_borders_around_text_fragments(
                        state,
                        self.style(),
                        stacking_relative_border_box,
                        &stacking_relative_content_box,
                        &text_fragment,
                        clip,
                    );
                }
            }
            SpecificFragmentInfo::ScannedText(ref text_fragment) => {
                // Create the main text display item.
                self.build_display_list_for_text_fragment(
                    state,
                    &text_fragment,
                    &stacking_relative_content_box,
                    &self.style.get_inheritedtext().text_shadow.0,
                    clip,
                );

                if opts::get().show_debug_fragment_borders {
                    self.build_debug_borders_around_text_fragments(
                        state,
                        self.style(),
                        stacking_relative_border_box,
                        &stacking_relative_content_box,
                        &text_fragment,
                        clip,
                    );
                }
            },
            SpecificFragmentInfo::Generic |
            SpecificFragmentInfo::GeneratedContent(..) |
            SpecificFragmentInfo::Table |
            SpecificFragmentInfo::TableCell |
            SpecificFragmentInfo::TableRow |
            SpecificFragmentInfo::TableWrapper |
            SpecificFragmentInfo::Multicol |
            SpecificFragmentInfo::MulticolColumn |
            SpecificFragmentInfo::InlineBlock(_) |
            SpecificFragmentInfo::InlineAbsoluteHypothetical(_) |
            SpecificFragmentInfo::InlineAbsolute(_) |
            SpecificFragmentInfo::TruncatedFragment(_) |
            SpecificFragmentInfo::Svg(_) => {
                if opts::get().show_debug_fragment_borders {
                    self.build_debug_borders_around_fragment(
                        state,
                        stacking_relative_border_box,
                        clip,
                    );
                }
            },
            SpecificFragmentInfo::Iframe(ref fragment_info) => {
                if !stacking_relative_content_box.is_empty() {
                    let browsing_context_id = match fragment_info.browsing_context_id {
                        Some(browsing_context_id) => browsing_context_id,
                        None => return warn!("No browsing context id for iframe."),
                    };
                    let pipeline_id = match fragment_info.pipeline_id {
                        Some(pipeline_id) => pipeline_id,
                        None => return warn!("No pipeline id for iframe {}.", browsing_context_id),
                    };

                    let base = state.create_base_display_item(
                        &stacking_relative_content_box,
                        build_local_clip(&self.style),
                        self.node,
                        self.style.get_cursor(Cursor::Default),
                        DisplayListSection::Content,
                    );
                    let item = DisplayItem::Iframe(Box::new(IframeDisplayItem {
                        base: base,
                        iframe: pipeline_id,
                    }));

                    let size = Size2D::new(
                        item.bounds().size.width.to_f32_px(),
                        item.bounds().size.height.to_f32_px(),
                    );
                    state
                        .iframe_sizes
                        .push((browsing_context_id, TypedSize2D::from_untyped(&size)));

                    state.add_display_item(item);
                }
            },
            SpecificFragmentInfo::Image(ref mut image_fragment) => {
                // Place the image into the display list.
                if let Some(ref image) = image_fragment.image {
                    let base = state.create_base_display_item(
                        &stacking_relative_content_box,
                        build_local_clip(&self.style),
                        self.node,
                        self.style.get_cursor(Cursor::Default),
                        DisplayListSection::Content,
                    );
                    state.add_display_item(DisplayItem::Image(Box::new(ImageDisplayItem {
                        base: base,
                        webrender_image: WebRenderImageInfo::from_image(image),
                        image_data: Some(Arc::new(image.bytes.clone())),
                        stretch_size: stacking_relative_content_box.size,
                        tile_spacing: Size2D::zero(),
                        image_rendering: self.style.get_inheritedbox().image_rendering.clone(),
                    })));
                }
            },
            SpecificFragmentInfo::Canvas(ref canvas_fragment_info) => {
                let computed_width = canvas_fragment_info.dom_width.to_px();
                let computed_height = canvas_fragment_info.dom_height.to_px();

                let (image_key, format) = match canvas_fragment_info.source {
                    CanvasFragmentSource::WebGL(image_key) => (image_key, PixelFormat::BGRA8),
                    CanvasFragmentSource::Image(ref ipc_renderer) => match *ipc_renderer {
                        Some(ref ipc_renderer) => {
                            let ipc_renderer = ipc_renderer.lock().unwrap();
                            let (sender, receiver) = ipc::channel().unwrap();
                            ipc_renderer
                                .send(CanvasMsg::FromLayout(FromLayoutMsg::SendData(sender)))
                                .unwrap();
                            (receiver.recv().unwrap().image_key, PixelFormat::BGRA8)
                        },
                        None => return,
                    },
                };

                let base = state.create_base_display_item(
                    &stacking_relative_content_box,
                    build_local_clip(&self.style),
                    self.node,
                    self.style.get_cursor(Cursor::Default),
                    DisplayListSection::Content,
                );
                let display_item = DisplayItem::Image(Box::new(ImageDisplayItem {
                    base: base,
                    webrender_image: WebRenderImageInfo {
                        width: computed_width as u32,
                        height: computed_height as u32,
                        format: format,
                        key: Some(image_key),
                    },
                    image_data: None,
                    stretch_size: stacking_relative_content_box.size,
                    tile_spacing: Size2D::zero(),
                    image_rendering: ImageRendering::Auto,
                }));

                state.add_display_item(display_item);
            },
            SpecificFragmentInfo::UnscannedText(_) => {
                panic!("Shouldn't see unscanned fragments here.")
            },
            SpecificFragmentInfo::TableColumn(_) => {
                panic!("Shouldn't see table column fragments here.")
            },
        }
    }

    fn create_stacking_context(
        &self,
        id: StackingContextId,
        base_flow: &BaseFlow,
        scroll_policy: ScrollPolicy,
        context_type: StackingContextType,
        parent_clipping_and_scrolling: ClippingAndScrolling,
    ) -> StackingContext {
        let border_box = self.stacking_relative_border_box(
            &base_flow.stacking_relative_position,
            &base_flow
                .early_absolute_position_info
                .relative_containing_block_size,
            base_flow
                .early_absolute_position_info
                .relative_containing_block_mode,
            CoordinateSystem::Parent,
        );
        // First, compute the offset of our border box (including relative positioning)
        // from our flow origin, since that is what `BaseFlow::overflow` is relative to.
        let border_box_offset = border_box
            .translate(&-base_flow.stacking_relative_position)
            .origin;
        // Then, using that, compute our overflow region relative to our border box.
        let overflow = base_flow
            .overflow
            .paint
            .translate(&-border_box_offset.to_vector());

        // Create the filter pipeline.
        let effects = self.style().get_effects();
        let mut filters = effects.filter.0.clone();
        if effects.opacity != 1.0 {
            filters.push(Filter::Opacity(effects.opacity.into()))
        }

        StackingContext::new(
            id,
            context_type,
            &border_box,
            &overflow,
            self.effective_z_index(),
            filters.into(),
            self.style().get_effects().mix_blend_mode.to_layout(),
            self.transform_matrix(&border_box),
            self.style().get_used_transform_style().to_layout(),
            self.perspective_matrix(&border_box),
            scroll_policy,
            parent_clipping_and_scrolling,
        )
    }

    fn build_display_list_for_text_fragment(
        &self,
        state: &mut DisplayListBuildState,
        text_fragment: &ScannedTextFragmentInfo,
        stacking_relative_content_box: &Rect<Au>,
        text_shadows: &[SimpleShadow],
        clip: &Rect<Au>,
    ) {
        // NB: The order for painting text components (CSS Text Decoration Module Level 3) is:
        // shadows, underline, overline, text, text-emphasis, and then line-through.

        // TODO(emilio): Allow changing more properties by ::selection
        // Paint the text with the color as described in its styling.
        let text_color = if text_fragment.selected() {
            self.selected_style().get_color().color
        } else {
            self.style().get_color().color
        };

        // Determine the orientation and cursor to use.
        let (orientation, cursor) = if self.style.writing_mode.is_vertical() {
            // TODO: Distinguish between 'sideways-lr' and 'sideways-rl' writing modes in CSS
            // Writing Modes Level 4.
            (TextOrientation::SidewaysRight, Cursor::VerticalText)
        } else {
            (TextOrientation::Upright, Cursor::Text)
        };

        // Compute location of the baseline.
        //
        // FIXME(pcwalton): Get the real container size.
        let container_size = Size2D::zero();
        let metrics = &text_fragment.run.font_metrics;
        let baseline_origin = stacking_relative_content_box.origin +
            LogicalPoint::new(self.style.writing_mode, Au(0), metrics.ascent)
                .to_physical(self.style.writing_mode, container_size)
                .to_vector();

        // Base item for all text/shadows
        let base = state.create_base_display_item(
            &stacking_relative_content_box,
            LocalClip::from(clip.to_layout()),
            self.node,
            self.style().get_cursor(cursor),
            DisplayListSection::Content,
        );

        // NB: According to CSS-BACKGROUNDS, text shadows render in *reverse* order (front
        // to back).

        // Shadows
        for shadow in text_shadows.iter().rev() {
            state.add_display_item(DisplayItem::PushTextShadow(Box::new(
                PushTextShadowDisplayItem {
                    base: base.clone(),
                    blur_radius: Au::from(shadow.blur),
                    offset: Vector2D::new(Au::from(shadow.horizontal), Au::from(shadow.vertical)),
                    color: shadow
                        .color
                        .unwrap_or(self.style().get_color().color)
                        .to_layout(),
                },
            )));
        }

        // Create display items for text decorations.
        let text_decorations = self.style()
            .get_inheritedtext()
            ._servo_text_decorations_in_effect;

        let stacking_relative_content_box = LogicalRect::from_physical(
            self.style.writing_mode,
            *stacking_relative_content_box,
            container_size,
        );

        // Underline
        if text_decorations.underline {
            let mut stacking_relative_box = stacking_relative_content_box;
            stacking_relative_box.start.b =
                stacking_relative_content_box.start.b + metrics.ascent - metrics.underline_offset;
            stacking_relative_box.size.block = metrics.underline_size;
            self.build_display_list_for_text_decoration(
                state,
                &text_color,
                &stacking_relative_box,
                clip,
            );
        }

        // Overline
        if text_decorations.overline {
            let mut stacking_relative_box = stacking_relative_content_box;
            stacking_relative_box.size.block = metrics.underline_size;
            self.build_display_list_for_text_decoration(
                state,
                &text_color,
                &stacking_relative_box,
                clip,
            );
        }

        // Text
        state.add_display_item(DisplayItem::Text(Box::new(TextDisplayItem {
            base: base.clone(),
            text_run: text_fragment.run.clone(),
            range: text_fragment.range,
            text_color: text_color.to_layout(),
            orientation: orientation,
            baseline_origin: baseline_origin,
        })));

        // TODO(#17715): emit text-emphasis marks here.
        // (just push another TextDisplayItem?)

        // Line-Through
        if text_decorations.line_through {
            let mut stacking_relative_box = stacking_relative_content_box;
            stacking_relative_box.start.b =
                stacking_relative_box.start.b + metrics.ascent - metrics.strikeout_offset;
            stacking_relative_box.size.block = metrics.strikeout_size;
            self.build_display_list_for_text_decoration(
                state,
                &text_color,
                &stacking_relative_box,
                clip,
            );
        }

        // Pop all the PushTextShadows
        if !text_shadows.is_empty() {
            state.add_display_item(DisplayItem::PopAllTextShadows(Box::new(
                PopAllTextShadowsDisplayItem { base: base.clone() },
            )));
        }
    }

    fn build_display_list_for_text_decoration(
        &self,
        state: &mut DisplayListBuildState,
        color: &RGBA,
        stacking_relative_box: &LogicalRect<Au>,
        clip: &Rect<Au>,
    ) {
        // FIXME(pcwalton, #2795): Get the real container size.
        let container_size = Size2D::zero();
        let stacking_relative_box =
            stacking_relative_box.to_physical(self.style.writing_mode, container_size);
        let base = state.create_base_display_item(
            &stacking_relative_box,
            LocalClip::from(clip.to_layout()),
            self.node,
            self.style.get_cursor(Cursor::Default),
            DisplayListSection::Content,
        );

        state.add_display_item(DisplayItem::Line(Box::new(LineDisplayItem {
            base: base,
            color: color.to_layout(),
            style: LineStyle::Solid,
        })));
    }

    fn unique_id(&self) -> u64 {
        let fragment_type = self.fragment_type();
        let id = self.node.id() as usize;
        combine_id_with_fragment_type(id, fragment_type) as u64
    }

    fn fragment_type(&self) -> FragmentType {
        self.pseudo.fragment_type()
    }
}

bitflags! {
    pub struct StackingContextCollectionFlags: u8 {
        /// This flow never establishes a containing block.
        const NEVER_CREATES_CONTAINING_BLOCK = 0b001;
        /// This flow never creates a ClipScrollNode.
        const NEVER_CREATES_CLIP_SCROLL_NODE = 0b010;
        /// This flow never creates a stacking context.
        const NEVER_CREATES_STACKING_CONTEXT = 0b100;
    }
}

pub trait BlockFlowDisplayListBuilding {
    fn collect_stacking_contexts_for_block(
        &mut self,
        state: &mut StackingContextCollectionState,
        flags: StackingContextCollectionFlags,
    );

    fn transform_clip_to_coordinate_space(
        &mut self,
        state: &mut StackingContextCollectionState,
        preserved_state: &mut SavedStackingContextCollectionState,
    );
    fn setup_clipping_for_block(
        &mut self,
        state: &mut StackingContextCollectionState,
        preserved_state: &mut SavedStackingContextCollectionState,
        stacking_context_type: BlockStackingContextType,
        flags: StackingContextCollectionFlags,
    ) -> ClippingAndScrolling;
    fn setup_clip_scroll_node_for_position(
        &mut self,
        state: &mut StackingContextCollectionState,
        border_box: &Rect<Au>,
    );
    fn setup_clip_scroll_node_for_overflow(
        &mut self,
        state: &mut StackingContextCollectionState,
        border_box: &Rect<Au>,
    );
    fn setup_clip_scroll_node_for_css_clip(
        &mut self,
        state: &mut StackingContextCollectionState,
        preserved_state: &mut SavedStackingContextCollectionState,
        stacking_relative_border_box: &Rect<Au>,
    );
    fn create_pseudo_stacking_context_for_block(
        &mut self,
        parent_stacking_context_id: StackingContextId,
        parent_clip_and_scroll_info: ClippingAndScrolling,
        state: &mut StackingContextCollectionState,
    );
    fn create_real_stacking_context_for_block(
        &mut self,
        parent_stacking_context_id: StackingContextId,
        parent_clipping_and_scrolling: ClippingAndScrolling,
        state: &mut StackingContextCollectionState,
    );
    fn build_display_list_for_block(
        &mut self,
        state: &mut DisplayListBuildState,
        border_painting_mode: BorderPaintingMode,
    );

    fn block_stacking_context_type(
        &self,
        flags: StackingContextCollectionFlags,
    ) -> BlockStackingContextType;
}

/// This structure manages ensuring that modification to StackingContextCollectionState is
/// only temporary. It's useful for moving recursively down the flow tree and ensuring
/// that the state is restored for siblings. To use this structure, we must call
/// SavedStackingContextCollectionState::restore in order to restore the state.
/// TODO(mrobinson): It would be nice to use RAII here to avoid having to call restore.
pub struct SavedStackingContextCollectionState {
    stacking_context_id: StackingContextId,
    real_stacking_context_id: StackingContextId,
    clipping_and_scrolling: ClippingAndScrolling,
    containing_block_clipping_and_scrolling: ClippingAndScrolling,
    clips_pushed: usize,
    containing_block_clips_pushed: usize,
    stacking_relative_content_box: Rect<Au>,
}

impl SavedStackingContextCollectionState {
    fn new(state: &mut StackingContextCollectionState) -> SavedStackingContextCollectionState {
        SavedStackingContextCollectionState {
            stacking_context_id: state.current_stacking_context_id,
            real_stacking_context_id: state.current_real_stacking_context_id,
            clipping_and_scrolling: state.current_clipping_and_scrolling,
            containing_block_clipping_and_scrolling: state.containing_block_clipping_and_scrolling,
            clips_pushed: 0,
            containing_block_clips_pushed: 0,
            stacking_relative_content_box: state.parent_stacking_relative_content_box,
        }
    }

    fn switch_to_containing_block_clip(&mut self, state: &mut StackingContextCollectionState) {
        let clip = state
            .containing_block_clip_stack
            .last()
            .cloned()
            .unwrap_or_else(max_rect);
        state.clip_stack.push(clip);
        self.clips_pushed += 1;
    }

    fn restore(self, state: &mut StackingContextCollectionState) {
        state.current_stacking_context_id = self.stacking_context_id;
        state.current_real_stacking_context_id = self.real_stacking_context_id;
        state.current_clipping_and_scrolling = self.clipping_and_scrolling;
        state.containing_block_clipping_and_scrolling =
            self.containing_block_clipping_and_scrolling;
        state.parent_stacking_relative_content_box = self.stacking_relative_content_box;

        let truncate_length = state.clip_stack.len() - self.clips_pushed;
        state.clip_stack.truncate(truncate_length);

        let truncate_length =
            state.containing_block_clip_stack.len() - self.containing_block_clips_pushed;
        state.containing_block_clip_stack.truncate(truncate_length);
    }

    fn push_clip(
        &mut self,
        state: &mut StackingContextCollectionState,
        clip: &Rect<Au>,
        positioning: StylePosition,
    ) {
        let mut clip = *clip;
        if positioning != StylePosition::Fixed {
            if let Some(old_clip) = state.clip_stack.last() {
                clip = old_clip.intersection(&clip).unwrap_or_else(Rect::zero);
            }
        }

        state.clip_stack.push(clip);
        self.clips_pushed += 1;

        if StylePosition::Absolute == positioning {
            state.containing_block_clip_stack.push(clip);
            self.containing_block_clips_pushed += 1;
        }
    }
}

impl BlockFlowDisplayListBuilding for BlockFlow {
    fn transform_clip_to_coordinate_space(
        &mut self,
        state: &mut StackingContextCollectionState,
        preserved_state: &mut SavedStackingContextCollectionState,
    ) {
        if state.clip_stack.is_empty() {
            return;
        }
        let border_box = self.stacking_relative_border_box(CoordinateSystem::Parent);
        let transform = match self.fragment.transform_matrix(&border_box) {
            Some(transform) => transform,
            None => return,
        };

        let perspective = self.fragment
            .perspective_matrix(&border_box)
            .unwrap_or_else(Transform3D::identity);
        let transform = transform.pre_mul(&perspective).inverse();

        let origin = &border_box.origin;
        let transform_clip = |clip: &Rect<Au>| {
            if *clip == max_rect() {
                return *clip;
            }

            match transform {
                Some(transform) if transform.m13 != 0.0 || transform.m23 != 0.0 => {
                    // We cannot properly handle perspective transforms, because there may be a
                    // situation where an element is transformed from outside the clip into the
                    // clip region. Here we don't have enough information to detect when that is
                    // happening. For the moment we just punt on trying to optimize the display
                    // list for those cases.
                    max_rect()
                },
                Some(transform) => {
                    let clip = Rect::new(
                        Point2D::new(
                            (clip.origin.x - origin.x).to_f32_px(),
                            (clip.origin.y - origin.y).to_f32_px(),
                        ),
                        Size2D::new(clip.size.width.to_f32_px(), clip.size.height.to_f32_px()),
                    );

                    let clip = transform.transform_rect(&clip);

                    Rect::new(
                        Point2D::new(
                            Au::from_f32_px(clip.origin.x),
                            Au::from_f32_px(clip.origin.y),
                        ),
                        Size2D::new(
                            Au::from_f32_px(clip.size.width),
                            Au::from_f32_px(clip.size.height),
                        ),
                    )
                },
                None => Rect::zero(),
            }
        };

        if let Some(clip) = state.clip_stack.last().cloned() {
            state.clip_stack.push(transform_clip(&clip));
            preserved_state.clips_pushed += 1;
        }

        if let Some(clip) = state.containing_block_clip_stack.last().cloned() {
            state
                .containing_block_clip_stack
                .push(transform_clip(&clip));
            preserved_state.containing_block_clips_pushed += 1;
        }
    }

    fn collect_stacking_contexts_for_block(
        &mut self,
        state: &mut StackingContextCollectionState,
        flags: StackingContextCollectionFlags,
    ) {
        let mut preserved_state = SavedStackingContextCollectionState::new(state);

        let block_stacking_context_type = self.block_stacking_context_type(flags);
        self.base.stacking_context_id = match block_stacking_context_type {
            BlockStackingContextType::NonstackingContext => state.current_stacking_context_id,
            BlockStackingContextType::PseudoStackingContext |
            BlockStackingContextType::StackingContext => state.generate_stacking_context_id(),
        };
        state.current_stacking_context_id = self.base.stacking_context_id;

        if block_stacking_context_type == BlockStackingContextType::StackingContext {
            state.current_real_stacking_context_id = self.base.stacking_context_id;
        }

        // We are getting the id of the scroll root that contains us here, not the id of
        // any scroll root that we create. If we create a scroll root, its index will be
        // stored in state.current_clipping_and_scrolling. If we create a stacking context,
        // we don't want it to be contained by its own scroll root.
        let containing_clipping_and_scrolling = self.setup_clipping_for_block(
            state,
            &mut preserved_state,
            block_stacking_context_type,
            flags,
        );

        if establishes_containing_block_for_absolute(flags, self.positioning()) {
            state.containing_block_clipping_and_scrolling = state.current_clipping_and_scrolling;
        }

        match block_stacking_context_type {
            BlockStackingContextType::NonstackingContext => {
                self.base.collect_stacking_contexts_for_children(state);
            },
            BlockStackingContextType::PseudoStackingContext => {
                self.create_pseudo_stacking_context_for_block(
                    preserved_state.stacking_context_id,
                    containing_clipping_and_scrolling,
                    state,
                );
            },
            BlockStackingContextType::StackingContext => {
                self.create_real_stacking_context_for_block(
                    preserved_state.stacking_context_id,
                    containing_clipping_and_scrolling,
                    state,
                );
            },
        }

        preserved_state.restore(state);
    }

    fn setup_clipping_for_block(
        &mut self,
        state: &mut StackingContextCollectionState,
        preserved_state: &mut SavedStackingContextCollectionState,
        stacking_context_type: BlockStackingContextType,
        flags: StackingContextCollectionFlags,
    ) -> ClippingAndScrolling {
        // If this block is absolutely positioned, we should be clipped and positioned by
        // the scroll root of our nearest ancestor that establishes a containing block.
        let containing_clipping_and_scrolling = match self.positioning() {
            StylePosition::Absolute => {
                preserved_state.switch_to_containing_block_clip(state);
                state.current_clipping_and_scrolling =
                    state.containing_block_clipping_and_scrolling;
                state.containing_block_clipping_and_scrolling
            },
            StylePosition::Fixed => {
                preserved_state.push_clip(state, &max_rect(), StylePosition::Fixed);
                state.current_clipping_and_scrolling
            },
            _ => state.current_clipping_and_scrolling,
        };
        self.base.clipping_and_scrolling = Some(containing_clipping_and_scrolling);

        let stacking_relative_border_box = if self.fragment.establishes_stacking_context() {
            self.stacking_relative_border_box(CoordinateSystem::Own)
        } else {
            self.stacking_relative_border_box(CoordinateSystem::Parent)
        };

        if stacking_context_type == BlockStackingContextType::StackingContext {
            self.transform_clip_to_coordinate_space(state, preserved_state);
        }

        if !flags.contains(StackingContextCollectionFlags::NEVER_CREATES_CLIP_SCROLL_NODE) {
            self.setup_clip_scroll_node_for_position(state, &stacking_relative_border_box);
            self.setup_clip_scroll_node_for_overflow(state, &stacking_relative_border_box);
            self.setup_clip_scroll_node_for_css_clip(
                state,
                preserved_state,
                &stacking_relative_border_box,
            );
        }
        self.base.clip = state.clip_stack.last().cloned().unwrap_or_else(max_rect);

        // We keep track of our position so that any stickily positioned elements can
        // properly determine the extent of their movement relative to scrolling containers.
        if !flags.contains(StackingContextCollectionFlags::NEVER_CREATES_CONTAINING_BLOCK) {
            let border_box = if self.fragment.establishes_stacking_context() {
                stacking_relative_border_box
            } else {
                self.stacking_relative_border_box(CoordinateSystem::Own)
            };
            state.parent_stacking_relative_content_box =
                self.fragment.stacking_relative_content_box(&border_box)
        }

        match self.positioning() {
            StylePosition::Absolute | StylePosition::Relative | StylePosition::Fixed => {
                state.containing_block_clipping_and_scrolling = state.current_clipping_and_scrolling
            },
            _ => {},
        }

        containing_clipping_and_scrolling
    }

    fn setup_clip_scroll_node_for_position(
        &mut self,
        state: &mut StackingContextCollectionState,
        border_box: &Rect<Au>,
    ) {
        if self.positioning() != StylePosition::Sticky {
            return;
        }

        let sticky_position = self.sticky_position();
        if sticky_position.left == MaybeAuto::Auto && sticky_position.right == MaybeAuto::Auto &&
            sticky_position.top == MaybeAuto::Auto &&
            sticky_position.bottom == MaybeAuto::Auto
        {
            return;
        }

        // Since position: sticky elements always establish a stacking context, we will
        // have previously calculated our border box in our own coordinate system. In
        // order to properly calculate max offsets we need to compare our size and
        // position in our parent's coordinate system.
        let border_box_in_parent = self.stacking_relative_border_box(CoordinateSystem::Parent);
        let margins = self.fragment.margin.to_physical(
            self.base
                .early_absolute_position_info
                .relative_containing_block_mode,
        );

        // Position:sticky elements are always restricted based on the size and position of
        // their containing block, which for sticky items is like relative and statically
        // positioned items: just the parent block.
        let constraint_rect = state.parent_stacking_relative_content_box;

        let to_offset_bound = |constraint_edge: Au, moving_edge: Au| -> f32 {
            (constraint_edge - moving_edge).to_f32_px()
        };

        // This is the minimum negative offset and then the maximum positive offset. We just
        // specify every edge, but if the corresponding margin is None, that offset has no effect.
        let vertical_offset_bounds = StickyOffsetBounds::new(
            to_offset_bound(
                constraint_rect.min_y(),
                border_box_in_parent.min_y() - margins.top,
            ),
            to_offset_bound(constraint_rect.max_y(), border_box_in_parent.max_y()),
        );
        let horizontal_offset_bounds = StickyOffsetBounds::new(
            to_offset_bound(
                constraint_rect.min_x(),
                border_box_in_parent.min_x() - margins.left,
            ),
            to_offset_bound(constraint_rect.max_x(), border_box_in_parent.max_x()),
        );

        // The margins control which edges have sticky behavior.
        let sticky_frame_data = StickyFrameData {
            margins: SideOffsets2D::new(
                sticky_position.top.to_option().map(|v| v.to_f32_px()),
                sticky_position.right.to_option().map(|v| v.to_f32_px()),
                sticky_position.bottom.to_option().map(|v| v.to_f32_px()),
                sticky_position.left.to_option().map(|v| v.to_f32_px()),
            ),
            vertical_offset_bounds,
            horizontal_offset_bounds,
        };

        let new_clip_scroll_index = state.add_clip_scroll_node(ClipScrollNode {
            id: None,
            parent_index: self.clipping_and_scrolling().scrolling,
            clip: ClippingRegion::from_rect(border_box),
            content_rect: Rect::zero(),
            node_type: ClipScrollNodeType::StickyFrame(sticky_frame_data),
        });

        let new_clipping_and_scrolling = ClippingAndScrolling::simple(new_clip_scroll_index);
        self.base.clipping_and_scrolling = Some(new_clipping_and_scrolling);
        state.current_clipping_and_scrolling = new_clipping_and_scrolling;
    }

    fn setup_clip_scroll_node_for_overflow(
        &mut self,
        state: &mut StackingContextCollectionState,
        border_box: &Rect<Au>,
    ) {
        if !self.overflow_style_may_require_clip_scroll_node() {
            return;
        }

        let content_box = self.fragment.stacking_relative_content_box(&border_box);
        let has_scrolling_overflow = self.base.overflow.scroll.origin != Point2D::zero() ||
            self.base.overflow.scroll.size.width > content_box.size.width ||
            self.base.overflow.scroll.size.height > content_box.size.height ||
            StyleOverflow::Hidden == self.fragment.style.get_box().overflow_x ||
            StyleOverflow::Hidden == self.fragment.style.get_box().overflow_y;

        self.mark_scrolling_overflow(has_scrolling_overflow);
        if !has_scrolling_overflow {
            return;
        }

        // If we already have a scroll root for this flow, just return. This can happen
        // when fragments map to more than one flow, such as in the case of table
        // wrappers. We just accept the first scroll root in that case.
        let new_clip_scroll_node_id =
            ClipId::new(self.fragment.unique_id(), state.pipeline_id.to_webrender());

        let sensitivity = if StyleOverflow::Hidden == self.fragment.style.get_box().overflow_x &&
            StyleOverflow::Hidden == self.fragment.style.get_box().overflow_y
        {
            ScrollSensitivity::Script
        } else {
            ScrollSensitivity::ScriptAndInputEvents
        };

        let clip_rect = build_inner_border_box_for_border_rect(&border_box, &self.fragment.style);
        let mut clip = ClippingRegion::from_rect(&clip_rect);
        let radii = build_border_radius_for_inner_rect(&border_box, &self.fragment.style);
        if !radii.is_square() {
            clip.intersect_with_rounded_rect(&clip_rect, &radii)
        }

        let content_size = self.base.overflow.scroll.origin + self.base.overflow.scroll.size;
        let content_size = Size2D::new(content_size.x, content_size.y);

        let new_clip_scroll_index = state.add_clip_scroll_node(ClipScrollNode {
            id: Some(new_clip_scroll_node_id),
            parent_index: self.clipping_and_scrolling().scrolling,
            clip: clip,
            content_rect: Rect::new(content_box.origin, content_size),
            node_type: ClipScrollNodeType::ScrollFrame(sensitivity),
        });

        let new_clipping_and_scrolling = ClippingAndScrolling::simple(new_clip_scroll_index);
        self.base.clipping_and_scrolling = Some(new_clipping_and_scrolling);
        state.current_clipping_and_scrolling = new_clipping_and_scrolling;
    }

    /// Adds a scroll root for a block to take the `clip` property into account
    /// per CSS 2.1 § 11.1.2.
    fn setup_clip_scroll_node_for_css_clip(
        &mut self,
        state: &mut StackingContextCollectionState,
        preserved_state: &mut SavedStackingContextCollectionState,
        stacking_relative_border_box: &Rect<Au>,
    ) {
        // Account for `clip` per CSS 2.1 § 11.1.2.
        let style_clip_rect = match self.fragment.style().get_effects().clip {
            Either::First(style_clip_rect) => style_clip_rect,
            _ => return,
        };

        // CSS `clip` should only apply to position:absolute or positione:fixed elements.
        // CSS Masking Appendix A: "Applies to: Absolutely positioned elements."
        match self.positioning() {
            StylePosition::Absolute | StylePosition::Fixed => {},
            _ => return,
        }

        let clip_origin = Point2D::new(
            stacking_relative_border_box.origin.x +
                style_clip_rect.left.map(Au::from).unwrap_or(Au(0)),
            stacking_relative_border_box.origin.y +
                style_clip_rect.top.map(Au::from).unwrap_or(Au(0)),
        );
        let right = style_clip_rect
            .right
            .map(Au::from)
            .unwrap_or(stacking_relative_border_box.size.width);
        let bottom = style_clip_rect
            .bottom
            .map(Au::from)
            .unwrap_or(stacking_relative_border_box.size.height);
        let clip_size = Size2D::new(right - clip_origin.x, bottom - clip_origin.y);

        let clip_rect = Rect::new(clip_origin, clip_size);
        preserved_state.push_clip(state, &clip_rect, self.positioning());

        let new_index = state.add_clip_scroll_node(ClipScrollNode {
            id: None,
            parent_index: self.clipping_and_scrolling().scrolling,
            clip: ClippingRegion::from_rect(&clip_rect),
            content_rect: Rect::zero(), // content_rect isn't important for clips.
            node_type: ClipScrollNodeType::Clip,
        });

        let new_indices = ClippingAndScrolling::new(new_index, new_index);
        self.base.clipping_and_scrolling = Some(new_indices);
        state.current_clipping_and_scrolling = new_indices;
    }

    fn create_pseudo_stacking_context_for_block(
        &mut self,
        parent_stacking_context_id: StackingContextId,
        parent_clipping_and_scrolling: ClippingAndScrolling,
        state: &mut StackingContextCollectionState,
    ) {
        let creation_mode = if self.base
            .flags
            .contains(FlowFlags::IS_ABSOLUTELY_POSITIONED) ||
            self.fragment.style.get_box().position != StylePosition::Static
        {
            StackingContextType::PseudoPositioned
        } else {
            assert!(self.base.flags.is_float());
            StackingContextType::PseudoFloat
        };

        let new_context = self.fragment.create_stacking_context(
            self.base.stacking_context_id,
            &self.base,
            ScrollPolicy::Scrollable,
            creation_mode,
            parent_clipping_and_scrolling,
        );
        state.add_stacking_context(parent_stacking_context_id, new_context);

        self.base.collect_stacking_contexts_for_children(state);

        let children = state
            .stacking_context_info
            .get_mut(&self.base.stacking_context_id)
            .map(|info| info.take_children());
        if let Some(children) = children {
            for child in children {
                if child.context_type == StackingContextType::PseudoFloat {
                    state.add_stacking_context(self.base.stacking_context_id, child);
                } else {
                    state.add_stacking_context(parent_stacking_context_id, child);
                }
            }
        }
    }

    fn create_real_stacking_context_for_block(
        &mut self,
        parent_stacking_context_id: StackingContextId,
        parent_clipping_and_scrolling: ClippingAndScrolling,
        state: &mut StackingContextCollectionState,
    ) {
        let scroll_policy = if self.is_fixed() {
            ScrollPolicy::Fixed
        } else {
            ScrollPolicy::Scrollable
        };

        let stacking_context = self.fragment.create_stacking_context(
            self.base.stacking_context_id,
            &self.base,
            scroll_policy,
            StackingContextType::Real,
            parent_clipping_and_scrolling,
        );

        state.add_stacking_context(parent_stacking_context_id, stacking_context);
        self.base.collect_stacking_contexts_for_children(state);
    }

    fn build_display_list_for_block(
        &mut self,
        state: &mut DisplayListBuildState,
        border_painting_mode: BorderPaintingMode,
    ) {
        let background_border_section = if self.base.flags.is_float() {
            DisplayListSection::BackgroundAndBorders
        } else if self.base
            .flags
            .contains(FlowFlags::IS_ABSOLUTELY_POSITIONED)
        {
            if self.fragment.establishes_stacking_context() {
                DisplayListSection::BackgroundAndBorders
            } else {
                DisplayListSection::BlockBackgroundsAndBorders
            }
        } else {
            DisplayListSection::BlockBackgroundsAndBorders
        };

        state.processing_scrolling_overflow_element = self.has_scrolling_overflow();

        // Add the box that starts the block context.
        self.fragment.build_display_list(
            state,
            &self.base.stacking_relative_position,
            &self.base
                .early_absolute_position_info
                .relative_containing_block_size,
            self.base
                .early_absolute_position_info
                .relative_containing_block_mode,
            border_painting_mode,
            background_border_section,
            &self.base.clip,
        );

        self.base
            .build_display_items_for_debugging_tint(state, self.fragment.node);

        state.processing_scrolling_overflow_element = false;
    }

    #[inline]
    fn block_stacking_context_type(
        &self,
        flags: StackingContextCollectionFlags,
    ) -> BlockStackingContextType {
        if flags.contains(StackingContextCollectionFlags::NEVER_CREATES_STACKING_CONTEXT) {
            return BlockStackingContextType::NonstackingContext;
        }

        if self.fragment.establishes_stacking_context() {
            return BlockStackingContextType::StackingContext;
        }

        if self.base
            .flags
            .contains(FlowFlags::IS_ABSOLUTELY_POSITIONED)
        {
            return BlockStackingContextType::PseudoStackingContext;
        }

        if self.fragment.style.get_box().position != StylePosition::Static {
            return BlockStackingContextType::PseudoStackingContext;
        }

        if self.base.flags.is_float() {
            return BlockStackingContextType::PseudoStackingContext;
        }

        BlockStackingContextType::NonstackingContext
    }
}

pub trait InlineFlowDisplayListBuilding {
    fn collect_stacking_contexts_for_inline(&mut self, state: &mut StackingContextCollectionState);
    fn build_display_list_for_inline_fragment_at_index(
        &mut self,
        state: &mut DisplayListBuildState,
        index: usize,
    );
    fn build_display_list_for_inline(&mut self, state: &mut DisplayListBuildState);
}

impl InlineFlowDisplayListBuilding for InlineFlow {
    fn collect_stacking_contexts_for_inline(&mut self, state: &mut StackingContextCollectionState) {
        self.base.stacking_context_id = state.current_stacking_context_id;
        self.base.clipping_and_scrolling = Some(state.current_clipping_and_scrolling);
        self.base.clip = state.clip_stack.last().cloned().unwrap_or_else(max_rect);

        for fragment in self.fragments.fragments.iter_mut() {
            let previous_cb_clipping_and_scrolling = state.containing_block_clipping_and_scrolling;
            if establishes_containing_block_for_absolute(
                StackingContextCollectionFlags::empty(),
                fragment.style.get_box().position,
            ) {
                state.containing_block_clipping_and_scrolling =
                    state.current_clipping_and_scrolling;
            }

            if !fragment.collect_stacking_contexts_for_blocklike_fragment(state) {
                if fragment.establishes_stacking_context() {
                    fragment.stacking_context_id = state.generate_stacking_context_id();

                    let current_stacking_context_id = state.current_stacking_context_id;
                    let stacking_context = fragment.create_stacking_context(
                        fragment.stacking_context_id,
                        &self.base,
                        ScrollPolicy::Scrollable,
                        StackingContextType::Real,
                        state.current_clipping_and_scrolling,
                    );

                    state.add_stacking_context(current_stacking_context_id, stacking_context);
                } else {
                    fragment.stacking_context_id = state.current_stacking_context_id;
                }
            }

            state.containing_block_clipping_and_scrolling = previous_cb_clipping_and_scrolling;
        }
    }

    fn build_display_list_for_inline_fragment_at_index(
        &mut self,
        state: &mut DisplayListBuildState,
        index: usize,
    ) {
        let fragment = self.fragments.fragments.get_mut(index).unwrap();
        fragment.build_display_list(
            state,
            &self.base.stacking_relative_position,
            &self.base
                .early_absolute_position_info
                .relative_containing_block_size,
            self.base
                .early_absolute_position_info
                .relative_containing_block_mode,
            BorderPaintingMode::Separate,
            DisplayListSection::Content,
            &self.base.clip,
        );
    }

    fn build_display_list_for_inline(&mut self, state: &mut DisplayListBuildState) {
        debug!(
            "Flow: building display list for {} inline fragments",
            self.fragments.len()
        );

        // We iterate using an index here, because we want to avoid doing a doing
        // a double-borrow of self (one mutable for the method call and one immutable
        // for the self.fragments.fragment iterator itself).
        for index in 0..self.fragments.fragments.len() {
            let (establishes_stacking_context, stacking_context_id) = {
                let fragment = self.fragments.fragments.get(index).unwrap();
                (
                    self.base.stacking_context_id != fragment.stacking_context_id,
                    fragment.stacking_context_id,
                )
            };

            let parent_stacking_context_id = state.current_stacking_context_id;
            if establishes_stacking_context {
                state.current_stacking_context_id = stacking_context_id;
            }

            self.build_display_list_for_inline_fragment_at_index(state, index);

            if establishes_stacking_context {
                state.current_stacking_context_id = parent_stacking_context_id
            }
        }

        if !self.fragments.fragments.is_empty() {
            self.base
                .build_display_items_for_debugging_tint(state, self.fragments.fragments[0].node);
        }
    }
}

pub trait ListItemFlowDisplayListBuilding {
    fn build_display_list_for_list_item(&mut self, state: &mut DisplayListBuildState);
}

impl ListItemFlowDisplayListBuilding for ListItemFlow {
    fn build_display_list_for_list_item(&mut self, state: &mut DisplayListBuildState) {
        // Draw the marker, if applicable.
        for marker in &mut self.marker_fragments {
            marker.build_display_list(
                state,
                &self.block_flow.base.stacking_relative_position,
                &self.block_flow
                    .base
                    .early_absolute_position_info
                    .relative_containing_block_size,
                self.block_flow
                    .base
                    .early_absolute_position_info
                    .relative_containing_block_mode,
                BorderPaintingMode::Separate,
                DisplayListSection::Content,
                &self.block_flow.base.clip,
            );
        }

        // Draw the rest of the block.
        self.block_flow
            .build_display_list_for_block(state, BorderPaintingMode::Separate)
    }
}

pub trait FlexFlowDisplayListBuilding {
    fn build_display_list_for_flex(&mut self, state: &mut DisplayListBuildState);
}

impl FlexFlowDisplayListBuilding for FlexFlow {
    fn build_display_list_for_flex(&mut self, state: &mut DisplayListBuildState) {
        // Draw the rest of the block.
        self.as_mut_block()
            .build_display_list_for_block(state, BorderPaintingMode::Separate)
    }
}

trait BaseFlowDisplayListBuilding {
    fn build_display_items_for_debugging_tint(
        &self,
        state: &mut DisplayListBuildState,
        node: OpaqueNode,
    );
}

impl BaseFlowDisplayListBuilding for BaseFlow {
    fn build_display_items_for_debugging_tint(
        &self,
        state: &mut DisplayListBuildState,
        node: OpaqueNode,
    ) {
        if !opts::get().show_debug_parallel_layout {
            return;
        }

        let thread_id = self.thread_id;
        let stacking_context_relative_bounds = Rect::new(
            self.stacking_relative_position.to_point(),
            self.position.size.to_physical(self.writing_mode),
        );

        let mut color = THREAD_TINT_COLORS[thread_id as usize % THREAD_TINT_COLORS.len()];
        color.a = 1.0;
        let base = state.create_base_display_item(
            &stacking_context_relative_bounds.inflate(Au::from_px(2), Au::from_px(2)),
            LocalClip::from(self.clip.to_layout()),
            node,
            None,
            DisplayListSection::Content,
        );
        state.add_display_item(DisplayItem::Border(Box::new(BorderDisplayItem {
            base: base,
            border_widths: SideOffsets2D::new_all_same(Au::from_px(2)),
            details: BorderDetails::Normal(NormalBorder {
                color: SideOffsets2D::new_all_same(color),
                style: SideOffsets2D::new_all_same(BorderStyle::Solid),
                radius: BorderRadii::all_same(Au(0)),
            }),
        })));
    }
}

trait ComputedValuesCursorUtility {
    fn get_cursor(&self, default_cursor: Cursor) -> Option<Cursor>;
}

impl ComputedValuesCursorUtility for ComputedValues {
    /// Gets the cursor to use given the specific ComputedValues.  `default_cursor` specifies
    /// the cursor to use if `cursor` is `auto`. Typically, this will be `PointerCursor`, but for
    /// text display items it may be `TextCursor` or `VerticalTextCursor`.
    #[inline]
    fn get_cursor(&self, default_cursor: Cursor) -> Option<Cursor> {
        match (
            self.get_pointing().pointer_events,
            self.get_pointing().cursor,
        ) {
            (PointerEvents::None, _) => None,
            (PointerEvents::Auto, cursor::Keyword::Auto) => Some(default_cursor),
            (PointerEvents::Auto, cursor::Keyword::Cursor(cursor)) => Some(cursor),
        }
    }
}

/// Adjusts `content_rect` as necessary for the given spread, and blur so that the resulting
/// bounding rect contains all of a shadow's ink.
fn shadow_bounds(content_rect: &Rect<Au>, blur: Au, spread: Au) -> Rect<Au> {
    let inflation = spread + blur * BLUR_INFLATION_FACTOR;
    content_rect.inflate(inflation, inflation)
}

/// Adjusts borders as appropriate to account for a fragment's status as the
/// first or last fragment within the range of an element.
///
/// Specifically, this function sets border widths to zero on the sides for
/// which the fragment is not outermost.
fn modify_border_width_for_inline_sides(
    border_width: &mut LogicalMargin<Au>,
    inline_border_info: InlineNodeBorderInfo,
) {
    if !inline_border_info.is_first_fragment_of_element {
        border_width.inline_start = Au(0);
    }

    if !inline_border_info.is_last_fragment_of_element {
        border_width.inline_end = Au(0);
    }
}

/// Describes how to paint the borders.
#[derive(Clone, Copy)]
pub enum BorderPaintingMode<'a> {
    /// Paint borders separately (`border-collapse: separate`).
    Separate,
    /// Paint collapsed borders.
    Collapse(&'a CollapsedBordersForCell),
    /// Paint no borders.
    Hidden,
}

#[derive(Clone, Copy, Debug)]
pub struct BackgroundPlacement {
    /// Rendering bounds. The background will start in the uppper-left corner
    /// and fill the whole area.
    bounds: Rect<Au>,
    /// Background tile size. Some backgrounds are repeated. These are the
    /// dimensions of a single image of the background.
    tile_size: Size2D<Au>,
    /// Spacing between tiles. Some backgrounds are not repeated seamless
    /// but have seams between them like tiles in real life.
    tile_spacing: Size2D<Au>,
    /// A clip area. While the background is rendered according to all the
    /// measures above it is only shown within these bounds.
    css_clip: Rect<Au>,
}
