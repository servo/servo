/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Builds display lists from flows and fragments.
//!
//! Other browser engines sometimes call this "painting", but it is more accurately called display
//! list building, as the actual painting does not happen here—only deciding *what* we're going to
//! paint.

use app_units::{Au, AU_PER_PX};
use canvas_traits::canvas::{CanvasMsg, FromLayoutMsg};
use crate::block::BlockFlow;
use crate::context::LayoutContext;
use crate::display_list::background::{self, get_cyclic};
use crate::display_list::border;
use crate::display_list::gradient;
use crate::display_list::items::{BaseDisplayItem, ClipScrollNode, BLUR_INFLATION_FACTOR};
use crate::display_list::items::{ClipScrollNodeIndex, ClipScrollNodeType, ClippingAndScrolling};
use crate::display_list::items::{ClippingRegion, DisplayItem, DisplayItemMetadata, DisplayList};
use crate::display_list::items::{CommonDisplayItem, DisplayListSection};
use crate::display_list::items::{IframeDisplayItem, OpaqueNode};
use crate::display_list::items::{PopAllTextShadowsDisplayItem, PushTextShadowDisplayItem};
use crate::display_list::items::{StackingContext, StackingContextType, StickyFrameData};
use crate::display_list::items::{TextOrientation, WebRenderImageInfo};
use crate::display_list::ToLayout;
use crate::flex::FlexFlow;
use crate::flow::{BaseFlow, Flow, FlowFlags};
use crate::flow_ref::FlowRef;
use crate::fragment::SpecificFragmentInfo;
use crate::fragment::{CanvasFragmentSource, CoordinateSystem, Fragment, ScannedTextFragmentInfo};
use crate::inline::{InlineFlow, InlineFragmentNodeFlags};
use crate::list_item::ListItemFlow;
use crate::model::MaybeAuto;
use crate::table_cell::CollapsedBordersForCell;
use euclid::{rect, Point2D, Rect, SideOffsets2D, Size2D, TypedRect, TypedSize2D, Vector2D};
use fnv::FnvHashMap;
use gfx::text::glyph::ByteIndex;
use gfx::text::TextRun;
use gfx_traits::{combine_id_with_fragment_type, FragmentType, StackingContextId};
use ipc_channel::ipc;
use msg::constellation_msg::{BrowsingContextId, PipelineId};
use net_traits::image_cache::UsePlaceholder;
use range::Range;
use servo_config::opts;
use servo_geometry::MaxRect;
use std::default::Default;
use std::f32;
use std::mem;
use std::sync::Arc;
use style::computed_values::border_style::T as BorderStyle;
use style::computed_values::overflow_x::T as StyleOverflow;
use style::computed_values::pointer_events::T as PointerEvents;
use style::computed_values::position::T as StylePosition;
use style::computed_values::visibility::T as Visibility;
use style::logical_geometry::{LogicalMargin, LogicalPoint, LogicalRect};
use style::properties::{style_structs, ComputedValues};
use style::servo::restyle_damage::ServoRestyleDamage;
use style::values::computed::effects::SimpleShadow;
use style::values::computed::image::Image as ComputedImage;
use style::values::computed::Gradient;
use style::values::generics::background::BackgroundSize;
use style::values::generics::image::{GradientKind, Image, PaintWorklet};
use style::values::generics::ui::Cursor;
use style::values::{Either, RGBA};
use style_traits::cursor::CursorKind;
use style_traits::CSSPixel;
use style_traits::ToCss;
use webrender_api::{self, BorderDetails, BorderRadius, BorderSide, BoxShadowClipMode, ColorF};
use webrender_api::{ExternalScrollId, FilterOp, GlyphInstance, ImageRendering, LayoutRect};
use webrender_api::{LayoutSize, LayoutTransform, LayoutVector2D, LineStyle, NinePatchBorder};
use webrender_api::{NinePatchBorderSource, NormalBorder, ScrollSensitivity, StickyOffsetBounds};

fn establishes_containing_block_for_absolute(
    flags: StackingContextCollectionFlags,
    positioning: StylePosition,
    established_reference_frame: bool,
) -> bool {
    if established_reference_frame {
        return true;
    }

    !flags.contains(StackingContextCollectionFlags::POSITION_NEVER_CREATES_CONTAINING_BLOCK) &&
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

pub struct InlineNodeBorderInfo {
    is_first_fragment_of_element: bool,
    is_last_fragment_of_element: bool,
}

#[derive(Debug)]
struct StackingContextInfo {
    children: Vec<StackingContext>,
    clip_scroll_nodes: Vec<ClipScrollNodeIndex>,
    real_stacking_context_id: StackingContextId,
}

impl StackingContextInfo {
    fn new(real_stacking_context_id: StackingContextId) -> StackingContextInfo {
        StackingContextInfo {
            children: Vec::new(),
            clip_scroll_nodes: Vec::new(),
            real_stacking_context_id,
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

    /// The current reference frame ClipScrollNodeIndex.
    pub current_real_stacking_context_id: StackingContextId,

    /// The next stacking context id that we will assign to a stacking context.
    pub next_stacking_context_id: StackingContextId,

    /// The current reference frame id. This is used to assign items to the parent
    /// reference frame when we encounter a fixed position stacking context.
    pub current_parent_reference_frame_id: ClipScrollNodeIndex,

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
        let root_clip_indices =
            ClippingAndScrolling::simple(ClipScrollNodeIndex::root_scroll_node());

        let mut stacking_context_info = FnvHashMap::default();
        stacking_context_info.insert(
            StackingContextId::root(),
            StackingContextInfo::new(StackingContextId::root()),
        );

        // We add two empty nodes to represent the WebRender root reference frame and
        // root scroll nodes. WebRender adds these automatically and we add them here
        // so that the ids in the array match up with the ones we assign during display
        // list building. We ignore these two nodes during conversion to WebRender
        // display lists.
        let clip_scroll_nodes = vec![ClipScrollNode::placeholder(), ClipScrollNode::placeholder()];

        StackingContextCollectionState {
            pipeline_id: pipeline_id,
            root_stacking_context: StackingContext::root(),
            stacking_context_info,
            clip_scroll_nodes,
            current_stacking_context_id: StackingContextId::root(),
            current_real_stacking_context_id: StackingContextId::root(),
            next_stacking_context_id: StackingContextId::root().next(),
            current_parent_reference_frame_id: ClipScrollNodeIndex::root_reference_frame(),
            current_clipping_and_scrolling: root_clip_indices,
            containing_block_clipping_and_scrolling: root_clip_indices,
            clip_stack: Vec::new(),
            containing_block_clip_stack: Vec::new(),
            parent_stacking_relative_content_box: Rect::zero(),
        }
    }

    fn allocate_stacking_context_info(
        &mut self,
        stacking_context_type: StackingContextType,
    ) -> StackingContextId {
        let next_stacking_context_id = self.next_stacking_context_id.next();
        let allocated_id =
            mem::replace(&mut self.next_stacking_context_id, next_stacking_context_id);

        let real_stacking_context_id = match stacking_context_type {
            StackingContextType::Real => allocated_id,
            _ => self.current_real_stacking_context_id,
        };

        self.stacking_context_info.insert(
            allocated_id,
            StackingContextInfo::new(real_stacking_context_id),
        );

        allocated_id
    }

    fn add_stacking_context(
        &mut self,
        parent_id: StackingContextId,
        stacking_context: StackingContext,
    ) {
        self.stacking_context_info
            .get_mut(&parent_id)
            .unwrap()
            .children
            .push(stacking_context);
    }

    fn add_clip_scroll_node(&mut self, clip_scroll_node: ClipScrollNode) -> ClipScrollNodeIndex {
        let is_placeholder = clip_scroll_node.is_placeholder();

        self.clip_scroll_nodes.push(clip_scroll_node);
        let index = ClipScrollNodeIndex::new(self.clip_scroll_nodes.len() - 1);

        // If this node is a placeholder node (currently just reference frames), then don't add
        // it to the stacking context list. Placeholder nodes are created automatically by
        // WebRender and we don't want to explicitly create them in the display list. The node
        // is just there to take up a spot in the global list of ClipScrollNodes.
        if !is_placeholder {
            // We want the scroll root to be defined before any possible item that could use it,
            // so we make sure that it is added to the beginning of the parent "real" (non-pseudo)
            // stacking context. This ensures that item reordering will not result in an item using
            // the scroll root before it is defined.
            self.stacking_context_info
                .get_mut(&self.current_real_stacking_context_id)
                .unwrap()
                .clip_scroll_nodes
                .push(index);
        }

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

    /// Stores text runs to answer text queries used to place a cursor inside text.
    pub indexable_text: IndexableText,
}

impl<'a> DisplayListBuildState<'a> {
    pub fn new(
        layout_context: &'a LayoutContext,
        state: StackingContextCollectionState,
    ) -> DisplayListBuildState<'a> {
        DisplayListBuildState {
            layout_context: layout_context,
            root_stacking_context: state.root_stacking_context,
            items: FnvHashMap::default(),
            stacking_context_info: state.stacking_context_info,
            clip_scroll_nodes: state.clip_scroll_nodes,
            processing_scrolling_overflow_element: false,
            current_stacking_context_id: StackingContextId::root(),
            current_clipping_and_scrolling: ClippingAndScrolling::simple(
                ClipScrollNodeIndex::root_scroll_node(),
            ),
            iframe_sizes: Vec::new(),
            indexable_text: IndexableText::default(),
        }
    }

    fn add_display_item(&mut self, display_item: DisplayItem) {
        let items = self
            .items
            .entry(display_item.stacking_context_id())
            .or_insert(Vec::new());
        items.push(display_item);
    }

    fn add_image_item(&mut self, base: BaseDisplayItem, item: webrender_api::ImageDisplayItem) {
        if item.stretch_size == LayoutSize::zero() {
            return;
        }
        self.add_display_item(DisplayItem::Image(CommonDisplayItem::new(base, item)))
    }

    fn parent_clip_scroll_node_index(&self, index: ClipScrollNodeIndex) -> ClipScrollNodeIndex {
        if index.is_root_scroll_node() {
            return index;
        }

        self.clip_scroll_nodes[index.to_index()].parent_index
    }

    fn is_background_or_border_of_clip_scroll_node(&self, section: DisplayListSection) -> bool {
        (section == DisplayListSection::BackgroundAndBorders ||
            section == DisplayListSection::BlockBackgroundsAndBorders) &&
            self.processing_scrolling_overflow_element
    }

    fn create_base_display_item(
        &self,
        bounds: Rect<Au>,
        clip_rect: Rect<Au>,
        node: OpaqueNode,
        cursor: Option<CursorKind>,
        section: DisplayListSection,
    ) -> BaseDisplayItem {
        let clipping_and_scrolling = if self.is_background_or_border_of_clip_scroll_node(section) {
            ClippingAndScrolling::simple(
                self.parent_clip_scroll_node_index(self.current_clipping_and_scrolling.scrolling),
            )
        } else {
            self.current_clipping_and_scrolling
        };

        BaseDisplayItem::new(
            bounds.to_layout(),
            DisplayItemMetadata {
                node,
                // Store cursor id in display list.
                pointing: cursor.map(|x| x as u16),
            },
            clip_rect.to_layout(),
            section,
            self.current_stacking_context_id,
            clipping_and_scrolling,
        )
    }

    fn add_late_clip_node(&mut self, rect: LayoutRect, radii: BorderRadius) -> ClipScrollNodeIndex {
        let mut clip = ClippingRegion::from_rect(rect);
        clip.intersect_with_rounded_rect(rect, radii);

        let node = ClipScrollNode {
            parent_index: self.current_clipping_and_scrolling.scrolling,
            clip,
            content_rect: LayoutRect::zero(), // content_rect isn't important for clips.
            node_type: ClipScrollNodeType::Clip,
        };

        // We want the scroll root to be defined before any possible item that could use it,
        // so we make sure that it is added to the beginning of the parent "real" (non-pseudo)
        // stacking context. This ensures that item reordering will not result in an item using
        // the scroll root before it is defined.
        self.clip_scroll_nodes.push(node);
        let index = ClipScrollNodeIndex::new(self.clip_scroll_nodes.len() - 1);
        let real_stacking_context_id =
            self.stacking_context_info[&self.current_stacking_context_id].real_stacking_context_id;
        self.stacking_context_info
            .get_mut(&real_stacking_context_id)
            .unwrap()
            .clip_scroll_nodes
            .push(index);

        index
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
        let mut child_items = self
            .items
            .remove(&stacking_context.id)
            .unwrap_or(Vec::new());
        child_items.sort_by(|a, b| a.base().section.cmp(&b.base().section));
        child_items.reverse();

        let mut info = self
            .stacking_context_info
            .remove(&stacking_context.id)
            .unwrap();

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

    fn clipping_and_scrolling_scope<R, F: FnOnce(&mut Self) -> R>(&mut self, function: F) -> R {
        let previous_clipping_and_scrolling = self.current_clipping_and_scrolling;
        let ret = function(self);
        self.current_clipping_and_scrolling = previous_clipping_and_scrolling;
        ret
    }
}

/// The logical width of an insertion point: at the moment, a one-pixel-wide line.
const INSERTION_POINT_LOGICAL_WIDTH: Au = Au(1 * AU_PER_PX);

pub trait FragmentDisplayListBuilding {
    fn collect_stacking_contexts_for_blocklike_fragment(
        &mut self,
        state: &mut StackingContextCollectionState,
    ) -> bool;

    fn create_stacking_context_for_inline_block(
        &mut self,
        base: &BaseFlow,
        state: &mut StackingContextCollectionState,
    ) -> bool;

    /// Adds the display items necessary to paint the background of this fragment to the display
    /// list if necessary.
    fn build_display_list_for_background_if_applicable(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        display_list_section: DisplayListSection,
        absolute_bounds: Rect<Au>,
    );

    /// Same as build_display_list_for_background_if_applicable, but lets you
    /// override the actual background used
    fn build_display_list_for_background_if_applicable_with_background(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        background: &style_structs::Background,
        background_color: RGBA,
        display_list_section: DisplayListSection,
        absolute_bounds: Rect<Au>,
    );

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
        bounds: Rect<Au>,
        display_list_section: DisplayListSection,
        clip: Rect<Au>,
    );

    /// Add display item for image border.
    ///
    /// Returns `Some` if the addition was successful.
    fn build_display_list_for_border_image(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        base: BaseDisplayItem,
        bounds: Rect<Au>,
        image: &ComputedImage,
        border_width: SideOffsets2D<Au>,
    ) -> Option<()>;

    /// Adds the display items necessary to paint the outline of this fragment to the display list
    /// if necessary.
    fn build_display_list_for_outline_if_applicable(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        bounds: Rect<Au>,
        clip: Rect<Au>,
    );

    /// Adds the display items necessary to paint the box shadow of this fragment to the display
    /// list if necessary.
    fn build_display_list_for_box_shadow_if_applicable(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        display_list_section: DisplayListSection,
        absolute_bounds: Rect<Au>,
        clip: Rect<Au>,
    );

    /// Adds display items necessary to draw debug boxes around a scanned text fragment.
    fn build_debug_borders_around_text_fragments(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        stacking_relative_border_box: Rect<Au>,
        stacking_relative_content_box: Rect<Au>,
        text_fragment: &ScannedTextFragmentInfo,
        clip: Rect<Au>,
    );

    /// Adds display items necessary to draw debug boxes around this fragment.
    fn build_debug_borders_around_fragment(
        &self,
        state: &mut DisplayListBuildState,
        stacking_relative_border_box: Rect<Au>,
        clip: Rect<Au>,
    );

    /// Adds the display items for this fragment to the given display list.
    ///
    /// Arguments:
    ///
    /// * `state`: The display building state, including the display list currently
    ///   under construction and other metadata useful for constructing it.
    /// * `dirty`: The dirty rectangle in the coordinate system of the owning flow.
    /// * `clip`: The region to clip the display items to.
    /// * `overflow_content_size`: The size of content associated with this fragment
    ///      that must have overflow handling applied to it. For a scrollable block
    ///      flow, it is expected that this is the size of the child boxes.
    fn build_display_list(
        &mut self,
        state: &mut DisplayListBuildState,
        stacking_relative_border_box: Rect<Au>,
        border_painting_mode: BorderPaintingMode,
        display_list_section: DisplayListSection,
        clip: Rect<Au>,
        overflow_content_size: Option<Size2D<Au>>,
    );

    /// build_display_list, but don't update the restyle damage
    ///
    /// Must be paired with a self.restyle_damage.remove(REPAINT) somewhere
    fn build_display_list_no_damage(
        &self,
        state: &mut DisplayListBuildState,
        stacking_relative_border_box: Rect<Au>,
        border_painting_mode: BorderPaintingMode,
        display_list_section: DisplayListSection,
        clip: Rect<Au>,
        overflow_content_size: Option<Size2D<Au>>,
    );

    /// Builds the display items necessary to paint the selection and/or caret for this fragment,
    /// if any.
    fn build_display_items_for_selection_if_necessary(
        &self,
        state: &mut DisplayListBuildState,
        stacking_relative_border_box: Rect<Au>,
        display_list_section: DisplayListSection,
        clip: Rect<Au>,
    );

    /// Creates the text display item for one text fragment. This can be called multiple times for
    /// one fragment if there are text shadows.
    ///
    /// `text_shadow` will be `Some` if this is rendering a shadow.
    fn build_display_list_for_text_fragment(
        &self,
        state: &mut DisplayListBuildState,
        text_fragment: &ScannedTextFragmentInfo,
        stacking_relative_content_box: Rect<Au>,
        text_shadows: &[SimpleShadow],
        clip: Rect<Au>,
    );

    /// Creates the display item for a text decoration: underline, overline, or line-through.
    fn build_display_list_for_text_decoration(
        &self,
        state: &mut DisplayListBuildState,
        color: &RGBA,
        stacking_relative_box: &LogicalRect<Au>,
        clip: Rect<Au>,
    );

    /// A helper method that `build_display_list` calls to create per-fragment-type display items.
    fn build_fragment_type_specific_display_items(
        &self,
        state: &mut DisplayListBuildState,
        stacking_relative_border_box: Rect<Au>,
        clip: Rect<Au>,
    );

    /// Creates a stacking context for associated fragment.
    fn create_stacking_context(
        &self,
        id: StackingContextId,
        base_flow: &BaseFlow,
        context_type: StackingContextType,
        established_reference_frame: Option<ClipScrollNodeIndex>,
        parent_clipping_and_scrolling: ClippingAndScrolling,
    ) -> StackingContext;

    fn unique_id(&self) -> u64;

    fn fragment_type(&self) -> FragmentType;
}

/// Get the border radius for the rectangle inside of a rounded border. This is useful
/// for building the clip for the content inside the border.
fn build_border_radius_for_inner_rect(
    outer_rect: Rect<Au>,
    style: &ComputedValues,
) -> BorderRadius {
    let radii = border::radii(outer_rect, style.get_border());
    if radii.is_zero() {
        return radii;
    }

    // Since we are going to using the inner rectangle (outer rectangle minus
    // border width), we need to adjust to border radius so that we are smaller
    // rectangle with the same border curve.
    let border_widths = style.logical_border_width().to_physical(style.writing_mode);
    border::inner_radii(radii, border_widths)
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
            SpecificFragmentInfo::TruncatedFragment(ref mut info) => info
                .full
                .collect_stacking_contexts_for_blocklike_fragment(state),
            _ => false,
        }
    }

    fn create_stacking_context_for_inline_block(
        &mut self,
        base: &BaseFlow,
        state: &mut StackingContextCollectionState,
    ) -> bool {
        self.stacking_context_id = state.allocate_stacking_context_info(StackingContextType::Real);

        let established_reference_frame = if self.can_establish_reference_frame() {
            // WebRender currently creates reference frames automatically, so just add
            // a placeholder node to allocate a ClipScrollNodeIndex for this reference frame.
            self.established_reference_frame =
                Some(state.add_clip_scroll_node(ClipScrollNode::placeholder()));
            self.established_reference_frame
        } else {
            None
        };

        let current_stacking_context_id = state.current_stacking_context_id;
        let stacking_context = self.create_stacking_context(
            self.stacking_context_id,
            &base,
            StackingContextType::Real,
            established_reference_frame,
            state.current_clipping_and_scrolling,
        );
        state.add_stacking_context(current_stacking_context_id, stacking_context);
        true
    }

    fn build_display_list_for_background_if_applicable(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        display_list_section: DisplayListSection,
        absolute_bounds: Rect<Au>,
    ) {
        let background = style.get_background();
        let background_color = style.resolve_color(background.background_color);
        // XXXManishearth the below method should ideally use an iterator over
        // backgrounds
        self.build_display_list_for_background_if_applicable_with_background(
            state,
            style,
            background,
            background_color,
            display_list_section,
            absolute_bounds,
        )
    }

    fn build_display_list_for_background_if_applicable_with_background(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        background: &style_structs::Background,
        background_color: RGBA,
        display_list_section: DisplayListSection,
        absolute_bounds: Rect<Au>,
    ) {
        // FIXME: This causes a lot of background colors to be displayed when they are clearly not
        // needed. We could use display list optimization to clean this up, but it still seems
        // inefficient. What we really want is something like "nearest ancestor element that
        // doesn't have a fragment".

        // Quote from CSS Backgrounds and Borders Module Level 3:
        //
        // > The background color is clipped according to the background-clip value associated
        // > with the bottom-most background image layer.
        let last_background_image_index = background.background_image.0.len() - 1;
        let color_clip = *get_cyclic(&background.background_clip.0, last_background_image_index);
        let (bounds, border_radii) = background::clip(
            color_clip,
            absolute_bounds,
            style.logical_border_width().to_physical(style.writing_mode),
            self.border_padding.to_physical(self.style.writing_mode),
            border::radii(absolute_bounds, style.get_border()),
        );

        state.clipping_and_scrolling_scope(|state| {
            if !border_radii.is_zero() {
                let clip_id = state.add_late_clip_node(bounds.to_layout(), border_radii);
                state.current_clipping_and_scrolling = ClippingAndScrolling::simple(clip_id);
            }

            let base = state.create_base_display_item(
                bounds,
                bounds,
                self.node,
                style.get_cursor(CursorKind::Default),
                display_list_section,
            );
            state.add_display_item(DisplayItem::Rectangle(CommonDisplayItem::new(
                base,
                webrender_api::RectangleDisplayItem {
                    color: background_color.to_layout(),
                },
            )));
        });

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
                        absolute_bounds,
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
                                absolute_bounds,
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
                            MaybeAuto::from_style(width.0, bounding_box_size.width)
                                .specified_or_default(bounding_box_size.width),
                            MaybeAuto::from_style(height.0, bounding_box_size.height)
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
                            absolute_bounds,
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
        if webrender_image.key.is_none() {
            return;
        }

        let image = Size2D::new(
            Au::from_px(webrender_image.width as i32),
            Au::from_px(webrender_image.height as i32),
        );
        let placement = background::placement(
            style.get_background(),
            state.layout_context.shared_context().viewport_size(),
            absolute_bounds,
            Some(image),
            style.logical_border_width().to_physical(style.writing_mode),
            self.border_padding.to_physical(self.style.writing_mode),
            border::radii(absolute_bounds, style.get_border()),
            index,
        );

        state.clipping_and_scrolling_scope(|state| {
            if !placement.clip_radii.is_zero() {
                let clip_id =
                    state.add_late_clip_node(placement.clip_rect.to_layout(), placement.clip_radii);
                state.current_clipping_and_scrolling = ClippingAndScrolling::simple(clip_id);
            }

            // Create the image display item.
            let base = state.create_base_display_item(
                placement.bounds,
                placement.clip_rect,
                self.node,
                style.get_cursor(CursorKind::Default),
                display_list_section,
            );

            debug!("(building display list) adding background image.");
            state.add_image_item(
                base,
                webrender_api::ImageDisplayItem {
                    image_key: webrender_image.key.unwrap(),
                    stretch_size: placement.tile_size.to_layout(),
                    tile_spacing: placement.tile_spacing.to_layout(),
                    image_rendering: style.get_inherited_box().image_rendering.to_layout(),
                    alpha_type: webrender_api::AlphaType::PremultipliedAlpha,
                    color: webrender_api::ColorF::WHITE,
                },
            );
        });
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
        let placement = background::placement(
            style.get_background(),
            state.layout_context.shared_context().viewport_size(),
            absolute_bounds,
            None,
            style.logical_border_width().to_physical(style.writing_mode),
            self.border_padding.to_physical(self.style.writing_mode),
            border::radii(absolute_bounds, style.get_border()),
            index,
        );

        state.clipping_and_scrolling_scope(|state| {
            if !placement.clip_radii.is_zero() {
                let clip_id =
                    state.add_late_clip_node(placement.clip_rect.to_layout(), placement.clip_radii);
                state.current_clipping_and_scrolling = ClippingAndScrolling::simple(clip_id);
            }

            let base = state.create_base_display_item(
                placement.bounds,
                placement.clip_rect,
                self.node,
                style.get_cursor(CursorKind::Default),
                display_list_section,
            );

            let display_item = match gradient.kind {
                GradientKind::Linear(angle_or_corner) => {
                    let (gradient, stops) = gradient::linear(
                        style,
                        placement.tile_size,
                        &gradient.items[..],
                        angle_or_corner,
                        gradient.repeating,
                    );
                    let item = webrender_api::GradientDisplayItem {
                        gradient,
                        tile_size: placement.tile_size.to_layout(),
                        tile_spacing: placement.tile_spacing.to_layout(),
                    };
                    DisplayItem::Gradient(CommonDisplayItem::with_data(base, item, stops))
                },
                GradientKind::Radial(shape, center, _angle) => {
                    let (gradient, stops) = gradient::radial(
                        style,
                        placement.tile_size,
                        &gradient.items[..],
                        shape,
                        center,
                        gradient.repeating,
                    );
                    let item = webrender_api::RadialGradientDisplayItem {
                        gradient,
                        tile_size: placement.tile_size.to_layout(),
                        tile_spacing: placement.tile_spacing.to_layout(),
                    };
                    DisplayItem::RadialGradient(CommonDisplayItem::with_data(base, item, stops))
                },
            };
            state.add_display_item(display_item);
        });
    }

    fn build_display_list_for_box_shadow_if_applicable(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        display_list_section: DisplayListSection,
        absolute_bounds: Rect<Au>,
        clip: Rect<Au>,
    ) {
        // NB: According to CSS-BACKGROUNDS, box shadows render in *reverse* order (front to back).
        for box_shadow in style.get_effects().box_shadow.0.iter().rev() {
            let bounds = shadow_bounds(
                absolute_bounds.translate(&Vector2D::new(
                    Au::from(box_shadow.base.horizontal),
                    Au::from(box_shadow.base.vertical),
                )),
                Au::from(box_shadow.base.blur),
                Au::from(box_shadow.spread),
            );

            let base = state.create_base_display_item(
                bounds,
                clip,
                self.node,
                style.get_cursor(CursorKind::Default),
                display_list_section,
            );
            let border_radius = border::radii(absolute_bounds, style.get_border());
            state.add_display_item(DisplayItem::BoxShadow(CommonDisplayItem::new(
                base,
                webrender_api::BoxShadowDisplayItem {
                    box_bounds: absolute_bounds.to_layout(),
                    color: style.resolve_color(box_shadow.base.color).to_layout(),
                    offset: LayoutVector2D::new(
                        box_shadow.base.horizontal.px(),
                        box_shadow.base.vertical.px(),
                    ),
                    blur_radius: box_shadow.base.blur.px(),
                    spread_radius: box_shadow.spread.px(),
                    border_radius: border_radius,
                    clip_mode: if box_shadow.inset {
                        BoxShadowClipMode::Inset
                    } else {
                        BoxShadowClipMode::Outset
                    },
                },
            )));
        }
    }

    fn build_display_list_for_borders_if_applicable(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        inline_info: Option<InlineNodeBorderInfo>,
        border_painting_mode: BorderPaintingMode,
        mut bounds: Rect<Au>,
        display_list_section: DisplayListSection,
        clip: Rect<Au>,
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

        // If this border collapses, then we draw outside the boundaries we were given.
        if let BorderPaintingMode::Collapse(collapsed_borders) = border_painting_mode {
            collapsed_borders.adjust_border_bounds_for_painting(&mut bounds, style.writing_mode)
        }

        // Append the border to the display list.
        let base = state.create_base_display_item(
            bounds,
            clip,
            self.node,
            style.get_cursor(CursorKind::Default),
            display_list_section,
        );

        let border_radius = border::radii(bounds, border_style_struct);
        let border_widths = border.to_physical(style.writing_mode);

        if let Either::Second(ref image) = border_style_struct.border_image_source {
            if self
                .build_display_list_for_border_image(
                    state,
                    style,
                    base.clone(),
                    bounds,
                    image,
                    border_widths,
                )
                .is_some()
            {
                return;
            }
            // Fallback to rendering a solid border.
        }
        if border_widths == SideOffsets2D::zero() {
            return;
        }
        let details = BorderDetails::Normal(NormalBorder {
            left: BorderSide {
                color: style.resolve_color(colors.left).to_layout(),
                style: border_style.left.to_layout(),
            },
            right: BorderSide {
                color: style.resolve_color(colors.right).to_layout(),
                style: border_style.right.to_layout(),
            },
            top: BorderSide {
                color: style.resolve_color(colors.top).to_layout(),
                style: border_style.top.to_layout(),
            },
            bottom: BorderSide {
                color: style.resolve_color(colors.bottom).to_layout(),
                style: border_style.bottom.to_layout(),
            },
            radius: border_radius,
            do_aa: true,
        });
        state.add_display_item(DisplayItem::Border(CommonDisplayItem::with_data(
            base,
            webrender_api::BorderDisplayItem {
                widths: border_widths.to_layout(),
                details,
            },
            Vec::new(),
        )));
    }

    fn build_display_list_for_border_image(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        base: BaseDisplayItem,
        bounds: Rect<Au>,
        image: &ComputedImage,
        border_width: SideOffsets2D<Au>,
    ) -> Option<()> {
        let border_style_struct = style.get_border();
        let border_image_outset =
            border::image_outset(border_style_struct.border_image_outset, border_width);
        let border_image_area = bounds.outer_rect(border_image_outset).size;
        let border_image_width = border::image_width(
            &border_style_struct.border_image_width,
            border_width.to_layout(),
            border_image_area,
        );
        let border_image_repeat = &border_style_struct.border_image_repeat;
        let border_image_fill = border_style_struct.border_image_slice.fill;
        let border_image_slice = &border_style_struct.border_image_slice.offsets;

        let mut stops = Vec::new();
        let mut width = border_image_area.width.to_px() as u32;
        let mut height = border_image_area.height.to_px() as u32;
        let source = match image {
            Image::Url(ref image_url) => {
                let url = image_url.url()?;
                let image = state.layout_context.get_webrender_image_for_url(
                    self.node,
                    url.clone(),
                    UsePlaceholder::No,
                )?;
                width = image.width;
                height = image.height;
                NinePatchBorderSource::Image(image.key?)
            },
            Image::PaintWorklet(ref paint_worklet) => {
                let image = self.get_webrender_image_for_paint_worklet(
                    state,
                    style,
                    paint_worklet,
                    border_image_area,
                )?;
                width = image.width;
                height = image.height;
                NinePatchBorderSource::Image(image.key?)
            },
            Image::Gradient(ref gradient) => match gradient.kind {
                GradientKind::Linear(angle_or_corner) => {
                    let (wr_gradient, linear_stops) = gradient::linear(
                        style,
                        border_image_area,
                        &gradient.items[..],
                        angle_or_corner,
                        gradient.repeating,
                    );
                    stops = linear_stops;
                    NinePatchBorderSource::Gradient(wr_gradient)
                },
                GradientKind::Radial(shape, center, _angle) => {
                    let (wr_gradient, radial_stops) = gradient::radial(
                        style,
                        border_image_area,
                        &gradient.items[..],
                        shape,
                        center,
                        gradient.repeating,
                    );
                    stops = radial_stops;
                    NinePatchBorderSource::RadialGradient(wr_gradient)
                },
            },
            _ => return None,
        };

        let details = BorderDetails::NinePatch(NinePatchBorder {
            source,
            width: width as i32,
            height: height as i32,
            slice: border::image_slice(border_image_slice, width as i32, height as i32),
            fill: border_image_fill,
            repeat_horizontal: border_image_repeat.0.to_layout(),
            repeat_vertical: border_image_repeat.1.to_layout(),
            outset: SideOffsets2D::new(
                border_image_outset.top.to_f32_px(),
                border_image_outset.right.to_f32_px(),
                border_image_outset.bottom.to_f32_px(),
                border_image_outset.left.to_f32_px(),
            ),
        });
        state.add_display_item(DisplayItem::Border(CommonDisplayItem::with_data(
            base,
            webrender_api::BorderDisplayItem {
                widths: border_image_width,
                details,
            },
            stops,
        )));
        Some(())
    }

    fn build_display_list_for_outline_if_applicable(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        mut bounds: Rect<Au>,
        clip: Rect<Au>,
    ) {
        use style::values::specified::outline::OutlineStyle;

        let width = Au::from(style.get_outline().outline_width);
        if width == Au(0) {
            return;
        }

        let outline_style = match style.get_outline().outline_style {
            OutlineStyle::Auto => BorderStyle::Solid,
            // FIXME(emilio): I don't think this border-style check is
            // necessary, since border-style: none implies an outline-width of
            // zero at computed value time.
            OutlineStyle::BorderStyle(BorderStyle::None) => return,
            OutlineStyle::BorderStyle(s) => s,
        };

        // Outlines are not accounted for in the dimensions of the border box, so adjust the
        // absolute bounds.
        let offset = width + Au::from(style.get_outline().outline_offset);
        bounds = bounds.inflate(offset, offset);

        // Append the outline to the display list.
        let color = style
            .resolve_color(style.get_outline().outline_color)
            .to_layout();
        let base = state.create_base_display_item(
            bounds,
            clip,
            self.node,
            style.get_cursor(CursorKind::Default),
            DisplayListSection::Outlines,
        );
        state.add_display_item(DisplayItem::Border(CommonDisplayItem::with_data(
            base,
            webrender_api::BorderDisplayItem {
                widths: SideOffsets2D::new_all_same(width).to_layout(),
                details: BorderDetails::Normal(border::simple(color, outline_style.to_layout())),
            },
            Vec::new(),
        )));
    }

    fn build_debug_borders_around_text_fragments(
        &self,
        state: &mut DisplayListBuildState,
        style: &ComputedValues,
        stacking_relative_border_box: Rect<Au>,
        stacking_relative_content_box: Rect<Au>,
        text_fragment: &ScannedTextFragmentInfo,
        clip: Rect<Au>,
    ) {
        // FIXME(pcwalton, #2795): Get the real container size.
        let container_size = Size2D::zero();

        // Compute the text fragment bounds and draw a border surrounding them.
        let base = state.create_base_display_item(
            stacking_relative_border_box,
            clip,
            self.node,
            style.get_cursor(CursorKind::Default),
            DisplayListSection::Content,
        );
        state.add_display_item(DisplayItem::Border(CommonDisplayItem::with_data(
            base,
            webrender_api::BorderDisplayItem {
                widths: SideOffsets2D::new_all_same(Au::from_px(1)).to_layout(),
                details: BorderDetails::Normal(border::simple(
                    ColorF::rgb(0, 0, 200),
                    webrender_api::BorderStyle::Solid,
                )),
            },
            Vec::new(),
        )));

        // Draw a rectangle representing the baselines.
        let mut baseline = LogicalRect::from_physical(
            self.style.writing_mode,
            stacking_relative_content_box,
            container_size,
        );
        baseline.start.b = baseline.start.b + text_fragment.run.ascent();
        baseline.size.block = Au(0);
        let baseline = baseline.to_physical(self.style.writing_mode, container_size);

        let base = state.create_base_display_item(
            baseline,
            clip,
            self.node,
            style.get_cursor(CursorKind::Default),
            DisplayListSection::Content,
        );
        // TODO(gw): Use a better estimate for wavy line thickness.
        let wavy_line_thickness = (0.33 * base.bounds.size.height).ceil();
        state.add_display_item(DisplayItem::Line(CommonDisplayItem::new(
            base,
            webrender_api::LineDisplayItem {
                orientation: webrender_api::LineOrientation::Horizontal,
                wavy_line_thickness,
                color: ColorF::rgb(0, 200, 0),
                style: LineStyle::Dashed,
            },
        )));
    }

    fn build_debug_borders_around_fragment(
        &self,
        state: &mut DisplayListBuildState,
        stacking_relative_border_box: Rect<Au>,
        clip: Rect<Au>,
    ) {
        // This prints a debug border around the border of this fragment.
        let base = state.create_base_display_item(
            stacking_relative_border_box,
            clip,
            self.node,
            self.style.get_cursor(CursorKind::Default),
            DisplayListSection::Content,
        );
        state.add_display_item(DisplayItem::Border(CommonDisplayItem::with_data(
            base,
            webrender_api::BorderDisplayItem {
                widths: SideOffsets2D::new_all_same(Au::from_px(1)).to_layout(),
                details: BorderDetails::Normal(border::simple(
                    ColorF::rgb(0, 0, 200),
                    webrender_api::BorderStyle::Solid,
                )),
            },
            Vec::new(),
        )));
    }

    fn build_display_items_for_selection_if_necessary(
        &self,
        state: &mut DisplayListBuildState,
        stacking_relative_border_box: Rect<Au>,
        display_list_section: DisplayListSection,
        clip: Rect<Au>,
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
                clip,
                self.node,
                self.style.get_cursor(CursorKind::Default),
                display_list_section,
            );
            state.add_display_item(DisplayItem::Rectangle(CommonDisplayItem::new(
                base,
                webrender_api::RectangleDisplayItem {
                    color: background_color.to_layout(),
                },
            )));
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
            insertion_point_bounds = rect(
                stacking_relative_border_box.origin.x + advance,
                stacking_relative_border_box.origin.y,
                INSERTION_POINT_LOGICAL_WIDTH,
                stacking_relative_border_box.size.height,
            );
            cursor = CursorKind::Text;
        } else {
            insertion_point_bounds = rect(
                stacking_relative_border_box.origin.x,
                stacking_relative_border_box.origin.y + advance,
                stacking_relative_border_box.size.width,
                INSERTION_POINT_LOGICAL_WIDTH,
            );
            cursor = CursorKind::VerticalText;
        };

        let base = state.create_base_display_item(
            insertion_point_bounds,
            clip,
            self.node,
            self.style.get_cursor(cursor),
            display_list_section,
        );
        state.add_display_item(DisplayItem::Rectangle(CommonDisplayItem::new(
            base,
            webrender_api::RectangleDisplayItem {
                color: self.style().get_color().color.to_layout(),
            },
        )));
    }

    fn build_display_list(
        &mut self,
        state: &mut DisplayListBuildState,
        stacking_relative_border_box: Rect<Au>,
        border_painting_mode: BorderPaintingMode,
        display_list_section: DisplayListSection,
        clip: Rect<Au>,
        overflow_content_size: Option<Size2D<Au>>,
    ) {
        let previous_clipping_and_scrolling = state.current_clipping_and_scrolling;
        if let Some(index) = self.established_reference_frame {
            state.current_clipping_and_scrolling = ClippingAndScrolling::simple(index);
        }

        self.restyle_damage.remove(ServoRestyleDamage::REPAINT);
        self.build_display_list_no_damage(
            state,
            stacking_relative_border_box,
            border_painting_mode,
            display_list_section,
            clip,
            overflow_content_size,
        );

        state.current_clipping_and_scrolling = previous_clipping_and_scrolling;
    }

    fn build_display_list_no_damage(
        &self,
        state: &mut DisplayListBuildState,
        stacking_relative_border_box: Rect<Au>,
        border_painting_mode: BorderPaintingMode,
        display_list_section: DisplayListSection,
        clip: Rect<Au>,
        overflow_content_size: Option<Size2D<Au>>,
    ) {
        if self.style().get_inherited_box().visibility != Visibility::Visible {
            return;
        }

        debug!(
            "Fragment::build_display_list at rel={:?}, abs={:?}: {:?}",
            self.border_box, stacking_relative_border_box, self
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
                        stacking_relative_border_box,
                    );

                    self.build_display_list_for_box_shadow_if_applicable(
                        state,
                        &*node.style,
                        display_list_section,
                        stacking_relative_border_box,
                        clip,
                    );

                    self.build_display_list_for_borders_if_applicable(
                        state,
                        &*node.style,
                        Some(InlineNodeBorderInfo {
                            is_first_fragment_of_element: node
                                .flags
                                .contains(InlineFragmentNodeFlags::FIRST_FRAGMENT_OF_ELEMENT),
                            is_last_fragment_of_element: node
                                .flags
                                .contains(InlineFragmentNodeFlags::LAST_FRAGMENT_OF_ELEMENT),
                        }),
                        border_painting_mode,
                        stacking_relative_border_box,
                        display_list_section,
                        clip,
                    );

                    // FIXME(emilio): Why does outline not do the same width
                    // fixup as border?
                    self.build_display_list_for_outline_if_applicable(
                        state,
                        &*node.style,
                        stacking_relative_border_box,
                        clip,
                    );
                }
            }

            if !self.is_scanned_text_fragment() {
                self.build_display_list_for_background_if_applicable(
                    state,
                    &*self.style,
                    display_list_section,
                    stacking_relative_border_box,
                );

                self.build_display_list_for_box_shadow_if_applicable(
                    state,
                    &*self.style,
                    display_list_section,
                    stacking_relative_border_box,
                    clip,
                );

                self.build_display_list_for_borders_if_applicable(
                    state,
                    &*self.style,
                    /* inline_node_info = */ None,
                    border_painting_mode,
                    stacking_relative_border_box,
                    display_list_section,
                    clip,
                );

                self.build_display_list_for_outline_if_applicable(
                    state,
                    &*self.style,
                    stacking_relative_border_box,
                    clip,
                );
            }
        }

        if self.is_primary_fragment() {
            // Paint the selection point if necessary.  Even an empty text fragment may have an
            // insertion point, so we do this even if `empty_rect` is true.
            self.build_display_items_for_selection_if_necessary(
                state,
                stacking_relative_border_box,
                display_list_section,
                clip,
            );
        }

        if empty_rect {
            return;
        }

        debug!("Fragment::build_display_list: intersected. Adding display item...");

        if let Some(content_size) = overflow_content_size {
            // Create a transparent rectangle for hit-testing purposes that exists in front
            // of this fragment's background but behind its content. This ensures that any
            // hit tests inside the content box but not on actual content target the current
            // scrollable ancestor.
            let content_size = TypedRect::from_size(content_size);
            let base = state.create_base_display_item(
                content_size,
                content_size,
                self.node,
                self.style
                    .get_cursor(CursorKind::Default)
                    .or_else(|| Some(CursorKind::Default)),
                DisplayListSection::Content,
            );
            state.add_display_item(DisplayItem::Rectangle(CommonDisplayItem::new(
                base,
                webrender_api::RectangleDisplayItem {
                    color: ColorF::TRANSPARENT,
                },
            )));
        }

        // Create special per-fragment-type display items.
        state.clipping_and_scrolling_scope(|state| {
            self.build_fragment_type_specific_display_items(
                state,
                stacking_relative_border_box,
                clip,
            );
        });

        if opts::get().show_debug_fragment_borders {
            self.build_debug_borders_around_fragment(state, stacking_relative_border_box, clip)
        }
    }

    fn build_fragment_type_specific_display_items(
        &self,
        state: &mut DisplayListBuildState,
        stacking_relative_border_box: Rect<Au>,
        clip: Rect<Au>,
    ) {
        // Compute the context box position relative to the parent stacking context.
        let stacking_relative_content_box =
            self.stacking_relative_content_box(stacking_relative_border_box);

        let create_base_display_item = |state: &mut DisplayListBuildState| {
            // Adjust the clipping region as necessary to account for `border-radius`.
            let radii =
                build_border_radius_for_inner_rect(stacking_relative_border_box, &self.style);

            if !radii.is_zero() {
                let clip_id =
                    state.add_late_clip_node(stacking_relative_border_box.to_layout(), radii);
                state.current_clipping_and_scrolling = ClippingAndScrolling::simple(clip_id);
            }

            state.create_base_display_item(
                stacking_relative_content_box,
                stacking_relative_border_box,
                self.node,
                self.style.get_cursor(CursorKind::Default),
                DisplayListSection::Content,
            )
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
                    stacking_relative_content_box,
                    &self.style.get_inherited_text().text_shadow.0,
                    clip,
                );

                if opts::get().show_debug_fragment_borders {
                    self.build_debug_borders_around_text_fragments(
                        state,
                        self.style(),
                        stacking_relative_border_box,
                        stacking_relative_content_box,
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
                    stacking_relative_content_box,
                    &self.style.get_inherited_text().text_shadow.0,
                    clip,
                );

                if opts::get().show_debug_fragment_borders {
                    self.build_debug_borders_around_text_fragments(
                        state,
                        self.style(),
                        stacking_relative_border_box,
                        stacking_relative_content_box,
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

                    let base = create_base_display_item(state);
                    let item = DisplayItem::Iframe(Box::new(IframeDisplayItem {
                        base,
                        iframe: pipeline_id,
                    }));

                    let size = Size2D::new(item.bounds().size.width, item.bounds().size.height);
                    state
                        .iframe_sizes
                        .push((browsing_context_id, TypedSize2D::from_untyped(&size)));

                    state.add_display_item(item);
                }
            },
            SpecificFragmentInfo::Image(ref image_fragment) => {
                // Place the image into the display list.
                if let Some(ref image) = image_fragment.image {
                    if let Some(id) = image.id {
                        let base = create_base_display_item(state);
                        state.add_image_item(
                            base,
                            webrender_api::ImageDisplayItem {
                                image_key: id,
                                stretch_size: stacking_relative_content_box.size.to_layout(),
                                tile_spacing: LayoutSize::zero(),
                                image_rendering: self
                                    .style
                                    .get_inherited_box()
                                    .image_rendering
                                    .to_layout(),
                                alpha_type: webrender_api::AlphaType::PremultipliedAlpha,
                                color: webrender_api::ColorF::WHITE,
                            },
                        );
                    }
                }
            },
            SpecificFragmentInfo::Media(ref fragment_info) => {
                if let Some((ref image_key, _, _)) = fragment_info.current_frame {
                    let base = create_base_display_item(state);
                    state.add_image_item(
                        base,
                        webrender_api::ImageDisplayItem {
                            image_key: *image_key,
                            stretch_size: stacking_relative_border_box.size.to_layout(),
                            tile_spacing: LayoutSize::zero(),
                            image_rendering: ImageRendering::Auto,
                            alpha_type: webrender_api::AlphaType::PremultipliedAlpha,
                            color: webrender_api::ColorF::WHITE,
                        },
                    );
                }
            },
            SpecificFragmentInfo::Canvas(ref canvas_fragment_info) => {
                let image_key = match canvas_fragment_info.source {
                    CanvasFragmentSource::WebGL(image_key) => image_key,
                    CanvasFragmentSource::Image(ref ipc_renderer) => match *ipc_renderer {
                        Some(ref ipc_renderer) => {
                            let ipc_renderer = ipc_renderer.lock().unwrap();
                            let (sender, receiver) = ipc::channel().unwrap();
                            ipc_renderer
                                .send(CanvasMsg::FromLayout(
                                    FromLayoutMsg::SendData(sender),
                                    canvas_fragment_info.canvas_id.clone(),
                                ))
                                .unwrap();
                            receiver.recv().unwrap().image_key
                        },
                        None => return,
                    },
                };

                let base = create_base_display_item(state);
                let display_item = webrender_api::ImageDisplayItem {
                    image_key,
                    stretch_size: stacking_relative_content_box.size.to_layout(),
                    tile_spacing: LayoutSize::zero(),
                    image_rendering: ImageRendering::Auto,
                    alpha_type: webrender_api::AlphaType::PremultipliedAlpha,
                    color: webrender_api::ColorF::WHITE,
                };

                state.add_image_item(base, display_item);
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
        context_type: StackingContextType,
        established_reference_frame: Option<ClipScrollNodeIndex>,
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
        let mut filters: Vec<FilterOp> = effects.filter.0.iter().map(ToLayout::to_layout).collect();
        if effects.opacity != 1.0 {
            filters.push(FilterOp::Opacity(effects.opacity.into(), effects.opacity));
        }

        StackingContext::new(
            id,
            context_type,
            border_box.to_layout(),
            overflow.to_layout(),
            self.effective_z_index(),
            filters,
            self.style().get_effects().mix_blend_mode.to_layout(),
            self.transform_matrix(&border_box),
            self.style().get_used_transform_style().to_layout(),
            self.perspective_matrix(&border_box),
            parent_clipping_and_scrolling,
            established_reference_frame,
        )
    }

    fn build_display_list_for_text_fragment(
        &self,
        state: &mut DisplayListBuildState,
        text_fragment: &ScannedTextFragmentInfo,
        stacking_relative_content_box: Rect<Au>,
        text_shadows: &[SimpleShadow],
        clip: Rect<Au>,
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
        let (_orientation, cursor) = if self.style.writing_mode.is_vertical() {
            // TODO: Distinguish between 'sideways-lr' and 'sideways-rl' writing modes in CSS
            // Writing Modes Level 4.
            (TextOrientation::SidewaysRight, CursorKind::VerticalText)
        } else {
            (TextOrientation::Upright, CursorKind::Text)
        };

        // Compute location of the baseline.
        //
        // FIXME(pcwalton): Get the real container size.
        let container_size = Size2D::zero();
        let metrics = &text_fragment.run.font_metrics;
        let baseline_origin = stacking_relative_content_box.origin + LogicalPoint::new(
            self.style.writing_mode,
            Au(0),
            metrics.ascent,
        )
        .to_physical(self.style.writing_mode, container_size)
        .to_vector();

        // Base item for all text/shadows
        let base = state.create_base_display_item(
            stacking_relative_content_box,
            clip,
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
                    shadow: webrender_api::Shadow {
                        offset: LayoutVector2D::new(shadow.horizontal.px(), shadow.vertical.px()),
                        color: self.style.resolve_color(shadow.color).to_layout(),
                        blur_radius: shadow.blur.px(),
                    },
                },
            )));
        }

        // Create display items for text decorations.
        let text_decorations = self.style().get_inherited_text().text_decorations_in_effect;

        let logical_stacking_relative_content_box = LogicalRect::from_physical(
            self.style.writing_mode,
            stacking_relative_content_box,
            container_size,
        );

        // Underline
        if text_decorations.underline {
            let mut stacking_relative_box = logical_stacking_relative_content_box;
            stacking_relative_box.start.b = logical_stacking_relative_content_box.start.b +
                metrics.ascent -
                metrics.underline_offset;
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
            let mut stacking_relative_box = logical_stacking_relative_content_box;
            stacking_relative_box.size.block = metrics.underline_size;
            self.build_display_list_for_text_decoration(
                state,
                &text_color,
                &stacking_relative_box,
                clip,
            );
        }

        // Text
        let glyphs = convert_text_run_to_glyphs(
            text_fragment.run.clone(),
            text_fragment.range,
            baseline_origin,
        );
        if !glyphs.is_empty() {
            let indexable_text = IndexableTextItem {
                origin: stacking_relative_content_box.origin,
                text_run: text_fragment.run.clone(),
                range: text_fragment.range,
                baseline_origin,
            };
            state.indexable_text.insert(self.node, indexable_text);

            state.add_display_item(DisplayItem::Text(CommonDisplayItem::with_data(
                base.clone(),
                webrender_api::TextDisplayItem {
                    font_key: text_fragment.run.font_key,
                    color: text_color.to_layout(),
                    glyph_options: None,
                },
                glyphs,
            )));
        }

        // TODO(#17715): emit text-emphasis marks here.
        // (just push another TextDisplayItem?)

        // Line-Through
        if text_decorations.line_through {
            let mut stacking_relative_box = logical_stacking_relative_content_box;
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
                PopAllTextShadowsDisplayItem { base },
            )));
        }
    }

    fn build_display_list_for_text_decoration(
        &self,
        state: &mut DisplayListBuildState,
        color: &RGBA,
        stacking_relative_box: &LogicalRect<Au>,
        clip: Rect<Au>,
    ) {
        // FIXME(pcwalton, #2795): Get the real container size.
        let container_size = Size2D::zero();
        let stacking_relative_box =
            stacking_relative_box.to_physical(self.style.writing_mode, container_size);
        let base = state.create_base_display_item(
            stacking_relative_box,
            clip,
            self.node,
            self.style.get_cursor(CursorKind::Default),
            DisplayListSection::Content,
        );

        // TODO(gw): Use a better estimate for wavy line thickness.
        let wavy_line_thickness = (0.33 * base.bounds.size.height).ceil();
        state.add_display_item(DisplayItem::Line(CommonDisplayItem::new(
            base,
            webrender_api::LineDisplayItem {
                orientation: webrender_api::LineOrientation::Horizontal,
                wavy_line_thickness,
                color: color.to_layout(),
                style: LineStyle::Solid,
            },
        )));
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
        const POSITION_NEVER_CREATES_CONTAINING_BLOCK = 0b001;
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
        stacking_context_type: Option<StackingContextType>,
        established_reference_frame: Option<ClipScrollNodeIndex>,
        flags: StackingContextCollectionFlags,
    ) -> ClippingAndScrolling;
    fn setup_clip_scroll_node_for_position(
        &mut self,
        state: &mut StackingContextCollectionState,
        border_box: Rect<Au>,
    );
    fn setup_clip_scroll_node_for_overflow(
        &mut self,
        state: &mut StackingContextCollectionState,
        border_box: Rect<Au>,
    );
    fn setup_clip_scroll_node_for_css_clip(
        &mut self,
        state: &mut StackingContextCollectionState,
        preserved_state: &mut SavedStackingContextCollectionState,
        stacking_relative_border_box: Rect<Au>,
    );
    fn create_pseudo_stacking_context_for_block(
        &mut self,
        stacking_context_type: StackingContextType,
        parent_stacking_context_id: StackingContextId,
        parent_clip_and_scroll_info: ClippingAndScrolling,
        state: &mut StackingContextCollectionState,
    );
    fn create_real_stacking_context_for_block(
        &mut self,
        parent_stacking_context_id: StackingContextId,
        parent_clipping_and_scrolling: ClippingAndScrolling,
        established_reference_frame: Option<ClipScrollNodeIndex>,
        state: &mut StackingContextCollectionState,
    );
    fn build_display_list_for_block(
        &mut self,
        state: &mut DisplayListBuildState,
        border_painting_mode: BorderPaintingMode,
    );
    fn build_display_list_for_block_no_damage(
        &self,
        state: &mut DisplayListBuildState,
        border_painting_mode: BorderPaintingMode,
    );
    fn build_display_list_for_background_if_applicable_with_background(
        &self,
        state: &mut DisplayListBuildState,
        background: &style_structs::Background,
        background_color: RGBA,
    );

    fn stacking_context_type(
        &self,
        flags: StackingContextCollectionFlags,
    ) -> Option<StackingContextType>;

    fn is_reference_frame(&self, context_type: Option<StackingContextType>) -> bool;
}

/// This structure manages ensuring that modification to StackingContextCollectionState is
/// only temporary. It's useful for moving recursively down the flow tree and ensuring
/// that the state is restored for siblings. To use this structure, we must call
/// SavedStackingContextCollectionState::restore in order to restore the state.
/// TODO(mrobinson): It would be nice to use RAII here to avoid having to call restore.
pub struct SavedStackingContextCollectionState {
    stacking_context_id: StackingContextId,
    real_stacking_context_id: StackingContextId,
    parent_reference_frame_id: ClipScrollNodeIndex,
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
            parent_reference_frame_id: state.current_parent_reference_frame_id,
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
            .unwrap_or_else(MaxRect::max_rect);
        state.clip_stack.push(clip);
        self.clips_pushed += 1;
    }

    fn restore(self, state: &mut StackingContextCollectionState) {
        state.current_stacking_context_id = self.stacking_context_id;
        state.current_real_stacking_context_id = self.real_stacking_context_id;
        state.current_parent_reference_frame_id = self.parent_reference_frame_id;
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
        mut clip: Rect<Au>,
        positioning: StylePosition,
    ) {
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

        let perspective = self
            .fragment
            .perspective_matrix(&border_box)
            .unwrap_or(LayoutTransform::identity());
        let transform = transform.pre_mul(&perspective).inverse();

        let origin = border_box.origin;
        let transform_clip = |clip: Rect<Au>| {
            if clip == Rect::max_rect() {
                return clip;
            }

            match transform {
                Some(transform) if transform.m13 != 0.0 || transform.m23 != 0.0 => {
                    // We cannot properly handle perspective transforms, because there may be a
                    // situation where an element is transformed from outside the clip into the
                    // clip region. Here we don't have enough information to detect when that is
                    // happening. For the moment we just punt on trying to optimize the display
                    // list for those cases.
                    Rect::max_rect()
                },
                Some(transform) => {
                    let clip = rect(
                        (clip.origin.x - origin.x).to_f32_px(),
                        (clip.origin.y - origin.y).to_f32_px(),
                        clip.size.width.to_f32_px(),
                        clip.size.height.to_f32_px(),
                    );

                    let clip = transform.transform_rect(&clip).unwrap();

                    rect(
                        Au::from_f32_px(clip.origin.x),
                        Au::from_f32_px(clip.origin.y),
                        Au::from_f32_px(clip.size.width),
                        Au::from_f32_px(clip.size.height),
                    )
                },
                None => Rect::zero(),
            }
        };

        if let Some(clip) = state.clip_stack.last().cloned() {
            state.clip_stack.push(transform_clip(clip));
            preserved_state.clips_pushed += 1;
        }

        if let Some(clip) = state.containing_block_clip_stack.last().cloned() {
            state.containing_block_clip_stack.push(transform_clip(clip));
            preserved_state.containing_block_clips_pushed += 1;
        }
    }

    /// Returns true if this fragment may establish a reference frame and this block
    /// creates a stacking context. Both are necessary in order to establish a reference
    /// frame.
    fn is_reference_frame(&self, context_type: Option<StackingContextType>) -> bool {
        match context_type {
            Some(StackingContextType::Real) => self.fragment.can_establish_reference_frame(),
            _ => false,
        }
    }

    fn collect_stacking_contexts_for_block(
        &mut self,
        state: &mut StackingContextCollectionState,
        flags: StackingContextCollectionFlags,
    ) {
        let mut preserved_state = SavedStackingContextCollectionState::new(state);

        let stacking_context_type = self.stacking_context_type(flags);
        self.base.stacking_context_id = match stacking_context_type {
            None => state.current_stacking_context_id,
            Some(sc_type) => state.allocate_stacking_context_info(sc_type),
        };
        state.current_stacking_context_id = self.base.stacking_context_id;

        if stacking_context_type == Some(StackingContextType::Real) {
            state.current_real_stacking_context_id = self.base.stacking_context_id;
        }

        let established_reference_frame = if self.is_reference_frame(stacking_context_type) {
            // WebRender currently creates reference frames automatically, so just add
            // a placeholder node to allocate a ClipScrollNodeIndex for this reference frame.
            Some(state.add_clip_scroll_node(ClipScrollNode::placeholder()))
        } else {
            None
        };

        // We are getting the id of the scroll root that contains us here, not the id of
        // any scroll root that we create. If we create a scroll root, its index will be
        // stored in state.current_clipping_and_scrolling. If we create a stacking context,
        // we don't want it to be contained by its own scroll root.
        let containing_clipping_and_scrolling = self.setup_clipping_for_block(
            state,
            &mut preserved_state,
            stacking_context_type,
            established_reference_frame,
            flags,
        );

        if establishes_containing_block_for_absolute(
            flags,
            self.positioning(),
            established_reference_frame.is_some(),
        ) {
            state.containing_block_clipping_and_scrolling = state.current_clipping_and_scrolling;
        }

        match stacking_context_type {
            None => self.base.collect_stacking_contexts_for_children(state),
            Some(StackingContextType::Real) => {
                self.create_real_stacking_context_for_block(
                    preserved_state.stacking_context_id,
                    containing_clipping_and_scrolling,
                    established_reference_frame,
                    state,
                );
            },
            Some(stacking_context_type) => {
                self.create_pseudo_stacking_context_for_block(
                    stacking_context_type,
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
        stacking_context_type: Option<StackingContextType>,
        established_reference_frame: Option<ClipScrollNodeIndex>,
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
                // If we are a fixed positioned stacking context, we want to be scrolled by
                // our reference frame instead of the clip scroll node that we are inside.
                preserved_state.push_clip(state, Rect::max_rect(), StylePosition::Fixed);
                state.current_clipping_and_scrolling.scrolling =
                    state.current_parent_reference_frame_id;
                state.current_clipping_and_scrolling
            },
            _ => state.current_clipping_and_scrolling,
        };
        self.base.clipping_and_scrolling = Some(containing_clipping_and_scrolling);

        if let Some(reference_frame_index) = established_reference_frame {
            let clipping_and_scrolling = ClippingAndScrolling::simple(reference_frame_index);
            state.current_clipping_and_scrolling = clipping_and_scrolling;
            self.base.clipping_and_scrolling = Some(clipping_and_scrolling);
        }

        let stacking_relative_border_box = if self.fragment.establishes_stacking_context() {
            self.stacking_relative_border_box(CoordinateSystem::Own)
        } else {
            self.stacking_relative_border_box(CoordinateSystem::Parent)
        };

        if stacking_context_type == Some(StackingContextType::Real) {
            self.transform_clip_to_coordinate_space(state, preserved_state);
        }

        if !flags.contains(StackingContextCollectionFlags::NEVER_CREATES_CLIP_SCROLL_NODE) {
            self.setup_clip_scroll_node_for_position(state, stacking_relative_border_box);
            self.setup_clip_scroll_node_for_overflow(state, stacking_relative_border_box);
            self.setup_clip_scroll_node_for_css_clip(
                state,
                preserved_state,
                stacking_relative_border_box,
            );
        }
        self.base.clip = state
            .clip_stack
            .last()
            .cloned()
            .unwrap_or_else(Rect::max_rect);

        // We keep track of our position so that any stickily positioned elements can
        // properly determine the extent of their movement relative to scrolling containers.
        if !flags.contains(StackingContextCollectionFlags::POSITION_NEVER_CREATES_CONTAINING_BLOCK)
        {
            let border_box = if self.fragment.establishes_stacking_context() {
                stacking_relative_border_box
            } else {
                self.stacking_relative_border_box(CoordinateSystem::Own)
            };
            state.parent_stacking_relative_content_box =
                self.fragment.stacking_relative_content_box(border_box)
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
        border_box: Rect<Au>,
    ) {
        if self.positioning() != StylePosition::Sticky {
            return;
        }

        let sticky_position = self.sticky_position();
        if sticky_position.left == MaybeAuto::Auto &&
            sticky_position.right == MaybeAuto::Auto &&
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
            parent_index: self.clipping_and_scrolling().scrolling,
            clip: ClippingRegion::from_rect(border_box.to_layout()),
            content_rect: LayoutRect::zero(),
            node_type: ClipScrollNodeType::StickyFrame(sticky_frame_data),
        });

        let new_clipping_and_scrolling = ClippingAndScrolling::simple(new_clip_scroll_index);
        self.base.clipping_and_scrolling = Some(new_clipping_and_scrolling);
        state.current_clipping_and_scrolling = new_clipping_and_scrolling;
    }

    fn setup_clip_scroll_node_for_overflow(
        &mut self,
        state: &mut StackingContextCollectionState,
        border_box: Rect<Au>,
    ) {
        if !self.overflow_style_may_require_clip_scroll_node() {
            return;
        }

        let content_box = self.fragment.stacking_relative_content_box(border_box);
        let has_scrolling_overflow = self.base.overflow.scroll.origin != Point2D::zero() ||
            self.base.overflow.scroll.size.width > content_box.size.width ||
            self.base.overflow.scroll.size.height > content_box.size.height ||
            StyleOverflow::Hidden == self.fragment.style.get_box().overflow_x ||
            StyleOverflow::Hidden == self.fragment.style.get_box().overflow_y;

        self.mark_scrolling_overflow(has_scrolling_overflow);
        if !has_scrolling_overflow {
            return;
        }

        let sensitivity = if StyleOverflow::Hidden == self.fragment.style.get_box().overflow_x &&
            StyleOverflow::Hidden == self.fragment.style.get_box().overflow_y
        {
            ScrollSensitivity::Script
        } else {
            ScrollSensitivity::ScriptAndInputEvents
        };

        let border_widths = self
            .fragment
            .style
            .logical_border_width()
            .to_physical(self.fragment.style.writing_mode);
        let clip_rect = border_box.inner_rect(border_widths);

        let mut clip = ClippingRegion::from_rect(clip_rect.to_layout());
        let radii = build_border_radius_for_inner_rect(border_box, &self.fragment.style);
        if !radii.is_zero() {
            clip.intersect_with_rounded_rect(clip_rect.to_layout(), radii)
        }

        let content_size = self.base.overflow.scroll.origin + self.base.overflow.scroll.size;
        let content_size = Size2D::new(content_size.x, content_size.y);

        let external_id =
            ExternalScrollId(self.fragment.unique_id(), state.pipeline_id.to_webrender());
        let new_clip_scroll_index = state.add_clip_scroll_node(ClipScrollNode {
            parent_index: self.clipping_and_scrolling().scrolling,
            clip: clip,
            content_rect: Rect::new(content_box.origin, content_size).to_layout(),
            node_type: ClipScrollNodeType::ScrollFrame(sensitivity, external_id),
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
        stacking_relative_border_box: Rect<Au>,
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
        preserved_state.push_clip(state, clip_rect, self.positioning());

        let new_index = state.add_clip_scroll_node(ClipScrollNode {
            parent_index: self.clipping_and_scrolling().scrolling,
            clip: ClippingRegion::from_rect(clip_rect.to_layout()),
            content_rect: LayoutRect::zero(), // content_rect isn't important for clips.
            node_type: ClipScrollNodeType::Clip,
        });

        let new_indices = ClippingAndScrolling::new(new_index, new_index);
        self.base.clipping_and_scrolling = Some(new_indices);
        state.current_clipping_and_scrolling = new_indices;
    }

    fn create_pseudo_stacking_context_for_block(
        &mut self,
        stacking_context_type: StackingContextType,
        parent_stacking_context_id: StackingContextId,
        parent_clipping_and_scrolling: ClippingAndScrolling,
        state: &mut StackingContextCollectionState,
    ) {
        let new_context = self.fragment.create_stacking_context(
            self.base.stacking_context_id,
            &self.base,
            stacking_context_type,
            None,
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
        established_reference_frame: Option<ClipScrollNodeIndex>,
        state: &mut StackingContextCollectionState,
    ) {
        let stacking_context = self.fragment.create_stacking_context(
            self.base.stacking_context_id,
            &self.base,
            StackingContextType::Real,
            established_reference_frame,
            parent_clipping_and_scrolling,
        );

        state.add_stacking_context(parent_stacking_context_id, stacking_context);
        self.base.collect_stacking_contexts_for_children(state);
    }

    fn build_display_list_for_block_no_damage(
        &self,
        state: &mut DisplayListBuildState,
        border_painting_mode: BorderPaintingMode,
    ) {
        let background_border_section = self.background_border_section();

        state.processing_scrolling_overflow_element = self.has_scrolling_overflow();

        let content_size = if state.processing_scrolling_overflow_element {
            let content_size = self.base.overflow.scroll.origin + self.base.overflow.scroll.size;
            Some(Size2D::new(content_size.x, content_size.y))
        } else {
            None
        };

        let stacking_relative_border_box = self
            .base
            .stacking_relative_border_box_for_display_list(&self.fragment);
        // Add the box that starts the block context.
        self.fragment.build_display_list_no_damage(
            state,
            stacking_relative_border_box,
            border_painting_mode,
            background_border_section,
            self.base.clip,
            content_size,
        );

        self.base
            .build_display_items_for_debugging_tint(state, self.fragment.node);

        state.processing_scrolling_overflow_element = false;
    }

    fn build_display_list_for_block(
        &mut self,
        state: &mut DisplayListBuildState,
        border_painting_mode: BorderPaintingMode,
    ) {
        self.fragment
            .restyle_damage
            .remove(ServoRestyleDamage::REPAINT);
        self.build_display_list_for_block_no_damage(state, border_painting_mode);
    }

    fn build_display_list_for_background_if_applicable_with_background(
        &self,
        state: &mut DisplayListBuildState,
        background: &style_structs::Background,
        background_color: RGBA,
    ) {
        let stacking_relative_border_box = self
            .base
            .stacking_relative_border_box_for_display_list(&self.fragment);
        let background_border_section = self.background_border_section();

        self.fragment
            .build_display_list_for_background_if_applicable_with_background(
                state,
                self.fragment.style(),
                background,
                background_color,
                background_border_section,
                stacking_relative_border_box,
            )
    }

    #[inline]
    fn stacking_context_type(
        &self,
        flags: StackingContextCollectionFlags,
    ) -> Option<StackingContextType> {
        if flags.contains(StackingContextCollectionFlags::NEVER_CREATES_STACKING_CONTEXT) {
            return None;
        }

        if self.fragment.establishes_stacking_context() {
            return Some(StackingContextType::Real);
        }

        if self
            .base
            .flags
            .contains(FlowFlags::IS_ABSOLUTELY_POSITIONED)
        {
            return Some(StackingContextType::PseudoPositioned);
        }

        if self.fragment.style.get_box().position != StylePosition::Static {
            return Some(StackingContextType::PseudoPositioned);
        }

        if self.base.flags.is_float() {
            return Some(StackingContextType::PseudoFloat);
        }

        None
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
        self.base.clip = state
            .clip_stack
            .last()
            .cloned()
            .unwrap_or_else(Rect::max_rect);

        let previous_cb_clipping_and_scrolling = state.containing_block_clipping_and_scrolling;

        for fragment in self.fragments.fragments.iter_mut() {
            state.containing_block_clipping_and_scrolling = previous_cb_clipping_and_scrolling;

            if establishes_containing_block_for_absolute(
                StackingContextCollectionFlags::empty(),
                fragment.style.get_box().position,
                false,
            ) {
                state.containing_block_clipping_and_scrolling =
                    state.current_clipping_and_scrolling;
            }

            // We clear this here, but it might be set again if we create a stacking context for
            // this fragment.
            fragment.established_reference_frame = None;

            if !fragment.collect_stacking_contexts_for_blocklike_fragment(state) {
                if !fragment.establishes_stacking_context() {
                    fragment.stacking_context_id = state.current_stacking_context_id;
                } else {
                    fragment.create_stacking_context_for_inline_block(&self.base, state);
                }
            }

            // Reset the containing block clipping and scrolling before each loop iteration,
            // so we don't pollute subsequent fragments.
            state.containing_block_clipping_and_scrolling = previous_cb_clipping_and_scrolling;
        }
    }

    fn build_display_list_for_inline_fragment_at_index(
        &mut self,
        state: &mut DisplayListBuildState,
        index: usize,
    ) {
        let fragment = self.fragments.fragments.get_mut(index).unwrap();
        let stacking_relative_border_box = self
            .base
            .stacking_relative_border_box_for_display_list(fragment);
        fragment.build_display_list(
            state,
            stacking_relative_border_box,
            BorderPaintingMode::Separate,
            DisplayListSection::Content,
            self.base.clip,
            None,
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
            let stacking_relative_border_box = self
                .block_flow
                .base
                .stacking_relative_border_box_for_display_list(marker);
            marker.build_display_list(
                state,
                stacking_relative_border_box,
                BorderPaintingMode::Separate,
                DisplayListSection::Content,
                self.block_flow.base.clip,
                None,
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
            stacking_context_relative_bounds.inflate(Au::from_px(2), Au::from_px(2)),
            self.clip,
            node,
            None,
            DisplayListSection::Content,
        );
        state.add_display_item(DisplayItem::Border(CommonDisplayItem::with_data(
            base,
            webrender_api::BorderDisplayItem {
                widths: SideOffsets2D::new_all_same(Au::from_px(2)).to_layout(),
                details: BorderDetails::Normal(border::simple(
                    color,
                    webrender_api::BorderStyle::Solid,
                )),
            },
            Vec::new(),
        )));
    }
}

trait ComputedValuesCursorUtility {
    fn get_cursor(&self, default_cursor: CursorKind) -> Option<CursorKind>;
}

impl ComputedValuesCursorUtility for ComputedValues {
    /// Gets the cursor to use given the specific ComputedValues.  `default_cursor` specifies
    /// the cursor to use if `cursor` is `auto`. Typically, this will be `PointerCursor`, but for
    /// text display items it may be `TextCursor` or `VerticalTextCursor`.
    #[inline]
    fn get_cursor(&self, default_cursor: CursorKind) -> Option<CursorKind> {
        match (
            self.get_inherited_ui().pointer_events,
            &self.get_inherited_ui().cursor,
        ) {
            (PointerEvents::None, _) => None,
            (
                PointerEvents::Auto,
                &Cursor {
                    keyword: CursorKind::Auto,
                    ..
                },
            ) => Some(default_cursor),
            (PointerEvents::Auto, &Cursor { keyword, .. }) => Some(keyword),
        }
    }
}

/// Adjusts `content_rect` as necessary for the given spread, and blur so that the resulting
/// bounding rect contains all of a shadow's ink.
fn shadow_bounds(content_rect: Rect<Au>, blur: Au, spread: Au) -> Rect<Au> {
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

fn convert_text_run_to_glyphs(
    text_run: Arc<TextRun>,
    range: Range<ByteIndex>,
    mut origin: Point2D<Au>,
) -> Vec<GlyphInstance> {
    let mut glyphs = vec![];

    for slice in text_run.natural_word_slices_in_visual_order(&range) {
        for glyph in slice.glyphs.iter_glyphs_for_byte_range(&slice.range) {
            let glyph_advance = if glyph.char_is_space() {
                glyph.advance() + text_run.extra_word_spacing
            } else {
                glyph.advance()
            };
            if !slice.glyphs.is_whitespace() {
                let glyph_offset = glyph.offset().unwrap_or(Point2D::zero());
                let point = origin + glyph_offset.to_vector();
                let glyph = GlyphInstance {
                    index: glyph.id(),
                    point: point.to_layout(),
                };
                glyphs.push(glyph);
            }
            origin.x += glyph_advance;
        }
    }
    return glyphs;
}

pub struct IndexableTextItem {
    /// The placement of the text item on the plane.
    pub origin: Point2D<Au>,
    /// The text run.
    pub text_run: Arc<TextRun>,
    /// The range of text within the text run.
    pub range: Range<ByteIndex>,
    /// The position of the start of the baseline of this text.
    pub baseline_origin: Point2D<Au>,
}

#[derive(Default)]
pub struct IndexableText {
    inner: FnvHashMap<OpaqueNode, Vec<IndexableTextItem>>,
}

impl IndexableText {
    fn insert(&mut self, node: OpaqueNode, item: IndexableTextItem) {
        let entries = self.inner.entry(node).or_insert(Vec::new());
        entries.push(item);
    }

    pub fn get(&self, node: OpaqueNode) -> Option<&[IndexableTextItem]> {
        self.inner.get(&node).map(|x| x.as_slice())
    }

    // Returns the text index within a node for the point of interest.
    pub fn text_index(&self, node: OpaqueNode, point_in_item: Point2D<Au>) -> Option<usize> {
        let item = self.inner.get(&node)?;
        // TODO(#20020): access all elements
        let point = point_in_item + item[0].origin.to_vector();
        let offset = point - item[0].baseline_origin;
        Some(
            item[0]
                .text_run
                .range_index_of_advance(&item[0].range, offset.x),
        )
    }
}
