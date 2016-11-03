/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Builds display lists from flows and fragments.
//!
//! Other browser engines sometimes call this "painting", but it is more accurately called display
//! list building, as the actual painting does not happen here—only deciding *what* we're going to
//! paint.

#![deny(unsafe_code)]

use app_units::{AU_PER_PX, Au};
use block::{BlockFlow, BlockStackingContextType};
use canvas_traits::{CanvasData, CanvasMsg, FromLayoutMsg};
use context::SharedLayoutContext;
use euclid::{Matrix4D, Point2D, Radians, Rect, SideOffsets2D, Size2D};
use euclid::point::TypedPoint2D;
use flex::FlexFlow;
use flow::{BaseFlow, Flow, IS_ABSOLUTELY_POSITIONED};
use flow_ref::FlowRef;
use fragment::{CoordinateSystem, Fragment, ImageFragmentInfo, ScannedTextFragmentInfo};
use fragment::SpecificFragmentInfo;
use gfx::display_list::{BLUR_INFLATION_FACTOR, BaseDisplayItem, BorderDisplayItem};
use gfx::display_list::{BorderRadii, BoxShadowClipMode, BoxShadowDisplayItem, ClippingRegion};
use gfx::display_list::{DisplayItem, DisplayItemMetadata, DisplayList, DisplayListSection};
use gfx::display_list::{GradientDisplayItem, IframeDisplayItem, ImageDisplayItem};
use gfx::display_list::{LineDisplayItem, OpaqueNode};
use gfx::display_list::{SolidColorDisplayItem, ScrollRoot, StackingContext, StackingContextType};
use gfx::display_list::{TextDisplayItem, TextOrientation, WebGLDisplayItem, WebRenderImageInfo};
use gfx_traits::{ScrollPolicy, ScrollRootId, StackingContextId};
use inline::{FIRST_FRAGMENT_OF_ELEMENT, InlineFlow, LAST_FRAGMENT_OF_ELEMENT};
use ipc_channel::ipc;
use list_item::ListItemFlow;
use model::{self, MaybeAuto, ToGfxMatrix};
use net_traits::image::base::PixelFormat;
use net_traits::image_cache_thread::UsePlaceholder;
use range::Range;
use std::{cmp, f32};
use std::collections::HashMap;
use std::default::Default;
use std::mem;
use std::sync::Arc;
use style::computed_values::{background_attachment, background_clip, background_origin};
use style::computed_values::{background_repeat, background_size, border_style};
use style::computed_values::{cursor, image_rendering, overflow_x};
use style::computed_values::{pointer_events, position, transform, transform_style, visibility};
use style::computed_values::_servo_overflow_clip_box as overflow_clip_box;
use style::computed_values::filter::Filter;
use style::computed_values::text_shadow::TextShadow;
use style::logical_geometry::{LogicalPoint, LogicalRect, LogicalSize, WritingMode};
use style::properties::{self, ServoComputedValues};
use style::properties::style_structs;
use style::servo::restyle_damage::REPAINT;
use style::values::{self, Either, RGBA, computed};
use style::values::computed::{Gradient, GradientKind, LengthOrPercentage, LengthOrPercentageOrAuto};
use style::values::specified::{AngleOrCorner, HorizontalDirection, VerticalDirection};
use style_traits::cursor::Cursor;
use table_cell::CollapsedBordersForCell;
use url::Url;
use util::opts;
use webrender_traits::{ColorF, GradientStop};

trait RgbColor {
    fn rgb(r: u8, g: u8, b: u8) -> Self;
}

impl RgbColor for ColorF {
    fn rgb(r: u8, g: u8, b: u8) -> Self {
        ColorF {
            r: (r as f32) / (255.0 as f32),
            g: (g as f32) / (255.0 as f32),
            b: (b as f32) / (255.0 as f32),
            a: 1.0 as f32
        }
    }
}

static THREAD_TINT_COLORS: [ColorF; 8] = [
    ColorF { r: 6.0 / 255.0, g: 153.0 / 255.0, b: 198.0 / 255.0, a: 0.7 },
    ColorF { r: 255.0 / 255.0, g: 212.0 / 255.0, b: 83.0 / 255.0, a: 0.7 },
    ColorF { r: 116.0 / 255.0, g: 29.0 / 255.0, b: 109.0 / 255.0, a: 0.7 },
    ColorF { r: 204.0 / 255.0, g: 158.0 / 255.0, b: 199.0 / 255.0, a: 0.7 },
    ColorF { r: 242.0 / 255.0, g: 46.0 / 255.0, b: 121.0 / 255.0, a: 0.7 },
    ColorF { r: 116.0 / 255.0, g: 203.0 / 255.0, b: 196.0 / 255.0, a: 0.7 },
    ColorF { r: 255.0 / 255.0, g: 249.0 / 255.0, b: 201.0 / 255.0, a: 0.7 },
    ColorF { r: 137.0 / 255.0, g: 196.0 / 255.0, b: 78.0 / 255.0, a: 0.7 },
];

fn get_cyclic<T>(arr: &[T], index: usize) -> &T {
    &arr[index % arr.len()]
}

pub struct DisplayListBuildState<'a> {
    pub shared_layout_context: &'a SharedLayoutContext,
    pub root_stacking_context: StackingContext,
    pub items: HashMap<StackingContextId, Vec<DisplayItem>>,
    pub scroll_roots: HashMap<ScrollRootId, ScrollRoot>,
    pub stacking_context_id_stack: Vec<StackingContextId>,
    pub scroll_root_id_stack: Vec<ScrollRootId>,
}

impl<'a> DisplayListBuildState<'a> {
    pub fn new(shared_layout_context: &'a SharedLayoutContext,
               stacking_context_id: StackingContextId)
               -> DisplayListBuildState<'a> {
        DisplayListBuildState {
            shared_layout_context: shared_layout_context,
            root_stacking_context: StackingContext::root(),
            items: HashMap::new(),
            scroll_roots: HashMap::new(),
            stacking_context_id_stack: vec!(stacking_context_id),
            scroll_root_id_stack: vec!(ScrollRootId::root()),
        }
    }

    fn add_display_item(&mut self, display_item: DisplayItem) {
        let items = self.items.entry(display_item.stacking_context_id()).or_insert(Vec::new());
        items.push(display_item);
    }

    fn add_scroll_root(&mut self, scroll_root: ScrollRoot) {
        debug_assert!(!self.scroll_roots.contains_key(&scroll_root.id));
        self.scroll_roots.insert(scroll_root.id, scroll_root);
    }

    pub fn stacking_context_id(&self) -> StackingContextId {
        self.stacking_context_id_stack.last().unwrap().clone()
    }

    pub fn push_stacking_context_id(&mut self, stacking_context_id: StackingContextId) {
        self.stacking_context_id_stack.push(stacking_context_id);
    }

    pub fn pop_stacking_context_id(&mut self) {
        assert!(!self.stacking_context_id_stack.is_empty());
        self.stacking_context_id_stack.pop();
    }

    pub fn scroll_root_id(&self) -> ScrollRootId {
        self.scroll_root_id_stack.last().unwrap().clone()
    }

    pub fn parent_scroll_root_id(&self) -> ScrollRootId {
        debug_assert!(self.scroll_root_id_stack.len() > 1);
        self.scroll_root_id_stack[self.scroll_root_id_stack.len() - 2]
    }

    pub fn push_scroll_root_id(&mut self, id: ScrollRootId) {
        self.scroll_root_id_stack.push(id);
    }

    pub fn pop_scroll_root_id(&mut self) {
        self.scroll_root_id_stack.pop();
        assert!(!self.scroll_root_id_stack.is_empty());
    }

    fn create_base_display_item(&self,
                                bounds: &Rect<Au>,
                                clip: &ClippingRegion,
                                node: OpaqueNode,
                                cursor: Option<Cursor>,
                                section: DisplayListSection)
                                -> BaseDisplayItem {
        BaseDisplayItem::new(&bounds,
                             DisplayItemMetadata {
                                 node: node,
                                 pointing: cursor,
                             },
                             &clip,
                             section,
                             self.stacking_context_id(),
                             self.scroll_root_id())
    }

    pub fn to_display_list(mut self) -> DisplayList {
        let mut scroll_root_stack = Vec::new();
        scroll_root_stack.push(ScrollRootId::root());

        let mut list = Vec::new();
        let root_context = mem::replace(&mut self.root_stacking_context, StackingContext::root());

        self.to_display_list_for_stacking_context(&mut list, root_context, &mut scroll_root_stack);

        DisplayList {
            list: list,
        }
    }

    fn to_display_list_for_stacking_context(&mut self,
                                            list: &mut Vec<DisplayItem>,
                                            mut stacking_context: StackingContext,
                                            scroll_root_stack: &mut Vec<ScrollRootId>) {
        let mut child_items = self.items.remove(&stacking_context.id).unwrap_or(Vec::new());
        child_items.sort_by(|a, b| a.base().section.cmp(&b.base().section));
        child_items.reverse();

        let mut child_stacking_contexts =
            mem::replace(&mut stacking_context.children, Vec::new());
        child_stacking_contexts.sort();

        let real_stacking_context = stacking_context.context_type == StackingContextType::Real;
        if !real_stacking_context {
            self.to_display_list_for_items(list,
                                           child_items,
                                           child_stacking_contexts,
                                           scroll_root_stack);
            return;
        }

        let mut scroll_root_stack = Vec::new();
        scroll_root_stack.push(stacking_context.parent_scroll_id);

        let (push_item, pop_item) = stacking_context.to_display_list_items();
        list.push(push_item);
        self.to_display_list_for_items(list,
                                       child_items,
                                       child_stacking_contexts,
                                       &mut scroll_root_stack);
        list.push(pop_item);
    }

    fn to_display_list_for_items(&mut self,
                                 list: &mut Vec<DisplayItem>,
                                 mut child_items: Vec<DisplayItem>,
                                 child_stacking_contexts: Vec<StackingContext>,
                                 scroll_root_stack: &mut Vec<ScrollRootId>) {
        // Properly order display items that make up a stacking context. "Steps" here
        // refer to the steps in CSS 2.1 Appendix E.
        // Steps 1 and 2: Borders and background for the root.
        while child_items.last().map_or(false,
             |child| child.section() == DisplayListSection::BackgroundAndBorders) {
            let item = child_items.pop().unwrap();
            self.switch_scroll_roots(list, item.scroll_root_id(), scroll_root_stack);
            list.push(item);
        }

        // Step 3: Positioned descendants with negative z-indices.
        let mut child_stacking_contexts = child_stacking_contexts.into_iter().peekable();
        while child_stacking_contexts.peek().map_or(false, |child| child.z_index < 0) {
            let context = child_stacking_contexts.next().unwrap();
            self.switch_scroll_roots(list, context.parent_scroll_id, scroll_root_stack);
            self.to_display_list_for_stacking_context(list, context, scroll_root_stack);
        }

        // Step 4: Block backgrounds and borders.
        while child_items.last().map_or(false,
             |child| child.section() == DisplayListSection::BlockBackgroundsAndBorders) {
            let item = child_items.pop().unwrap();
            self.switch_scroll_roots(list, item.scroll_root_id(), scroll_root_stack);
            list.push(item);
        }

        // Step 5: Floats.
        while child_stacking_contexts.peek().map_or(false,
            |child| child.context_type == StackingContextType::PseudoFloat) {
            let context = child_stacking_contexts.next().unwrap();
            self.switch_scroll_roots(list, context.parent_scroll_id, scroll_root_stack);
            self.to_display_list_for_stacking_context(list, context, scroll_root_stack);
        }

        // Step 6 & 7: Content and inlines that generate stacking contexts.
        while child_items.last().map_or(false,
             |child| child.section() == DisplayListSection::Content) {
            let item = child_items.pop().unwrap();
            self.switch_scroll_roots(list, item.scroll_root_id(), scroll_root_stack);
            list.push(item);
        }

        // Step 8 & 9: Positioned descendants with nonnegative, numeric z-indices.
        for child in child_stacking_contexts {
            self.switch_scroll_roots(list, child.parent_scroll_id, scroll_root_stack);
            self.to_display_list_for_stacking_context(list, child, scroll_root_stack);
        }

        // Step 10: Outlines.
        for item in child_items.drain(..) {
            self.switch_scroll_roots(list, item.scroll_root_id(), scroll_root_stack);
            list.push(item);
        }

        for _ in scroll_root_stack.drain(1..) {
            list.push(DisplayItem::PopScrollRoot(Box::new(BaseDisplayItem::empty())));
        }
    }

    fn switch_scroll_roots(&self,
                           list: &mut Vec<DisplayItem>,
                           new_id: ScrollRootId,
                           scroll_root_stack: &mut Vec<ScrollRootId>) {
        if new_id == *scroll_root_stack.last().unwrap() {
            return;
        }

        if new_id == *scroll_root_stack.first().unwrap() {
            for _ in scroll_root_stack.drain(1..) {
                list.push(DisplayItem::PopScrollRoot(Box::new(BaseDisplayItem::empty())));
            }
            return;
        }

        // We never want to reach the root of the ScrollRoot tree without encountering the
        // containing scroll root of this StackingContext. If this does happen we've tried to
        // switch to a ScrollRoot that does not contain our current stacking context or isn't
        // itself a direct child of our current stacking context. This should never happen
        // due to CSS stacking rules.
        debug_assert!(new_id != ScrollRootId::root());

        let scroll_root = self.scroll_roots.get(&new_id).unwrap();
        self.switch_scroll_roots(list, scroll_root.parent_id, scroll_root_stack);

        scroll_root_stack.push(new_id);
        list.push(scroll_root.to_push());
    }
}

/// The logical width of an insertion point: at the moment, a one-pixel-wide line.
const INSERTION_POINT_LOGICAL_WIDTH: Au = Au(1 * AU_PER_PX);

// TODO(gw): The transforms spec says that perspective length must
// be positive. However, there is some confusion between the spec
// and browser implementations as to handling the case of 0 for the
// perspective value. Until the spec bug is resolved, at least ensure
// that a provided perspective value of <= 0.0 doesn't cause panics
// and behaves as it does in other browsers.
// See https://lists.w3.org/Archives/Public/www-style/2016Jan/0020.html for more details.
#[inline]
fn create_perspective_matrix(d: Au) -> Matrix4D<f32> {
    let d = d.to_f32_px();
    if d <= 0.0 {
        Matrix4D::identity()
    } else {
        Matrix4D::create_perspective(d)
    }
}

pub trait FragmentDisplayListBuilding {
    /// Adds the display items necessary to paint the background of this fragment to the display
    /// list if necessary.
    fn build_display_list_for_background_if_applicable(&self,
                                                       state: &mut DisplayListBuildState,
                                                       style: &ServoComputedValues,
                                                       display_list_section: DisplayListSection,
                                                       absolute_bounds: &Rect<Au>,
                                                       clip: &ClippingRegion);

    /// Computes the background size for an image with the given background area according to the
    /// rules in CSS-BACKGROUNDS § 3.9.
    fn compute_background_image_size(&self,
                                     style: &ServoComputedValues,
                                     bounds: &Rect<Au>,
                                     image: &WebRenderImageInfo, index: usize)
                                     -> Size2D<Au>;

    /// Adds the display items necessary to paint the background image of this fragment to the
    /// appropriate section of the display list.
    fn build_display_list_for_background_image(&self,
                                               state: &mut DisplayListBuildState,
                                               style: &ServoComputedValues,
                                               display_list_section: DisplayListSection,
                                               absolute_bounds: &Rect<Au>,
                                               clip: &ClippingRegion,
                                               image_url: &Url,
                                               background_index: usize);

    /// Adds the display items necessary to paint the background linear gradient of this fragment
    /// to the appropriate section of the display list.
    fn build_display_list_for_background_gradient(&self,
                                                  state: &mut DisplayListBuildState,
                                                  display_list_section: DisplayListSection,
                                                  absolute_bounds: &Rect<Au>,
                                                  clip: &ClippingRegion,
                                                  gradient: &Gradient,
                                                  style: &ServoComputedValues);

    /// Adds the display items necessary to paint the borders of this fragment to a display list if
    /// necessary.
    fn build_display_list_for_borders_if_applicable(
            &self,
            state: &mut DisplayListBuildState,
            style: &ServoComputedValues,
            border_painting_mode: BorderPaintingMode,
            bounds: &Rect<Au>,
            display_list_section: DisplayListSection,
            clip: &ClippingRegion);

    /// Adds the display items necessary to paint the outline of this fragment to the display list
    /// if necessary.
    fn build_display_list_for_outline_if_applicable(&self,
                                                    state: &mut DisplayListBuildState,
                                                    style: &ServoComputedValues,
                                                    bounds: &Rect<Au>,
                                                    clip: &ClippingRegion);

    /// Adds the display items necessary to paint the box shadow of this fragment to the display
    /// list if necessary.
    fn build_display_list_for_box_shadow_if_applicable(&self,
                                                       state: &mut DisplayListBuildState,
                                                       style: &ServoComputedValues,
                                                       display_list_section: DisplayListSection,
                                                       absolute_bounds: &Rect<Au>,
                                                       clip: &ClippingRegion);

    /// Adds display items necessary to draw debug boxes around a scanned text fragment.
    fn build_debug_borders_around_text_fragments(&self,
                                                 state: &mut DisplayListBuildState,
                                                 style: &ServoComputedValues,
                                                 stacking_relative_border_box: &Rect<Au>,
                                                 stacking_relative_content_box: &Rect<Au>,
                                                 text_fragment: &ScannedTextFragmentInfo,
                                                 clip: &ClippingRegion);

    /// Adds display items necessary to draw debug boxes around this fragment.
    fn build_debug_borders_around_fragment(&self,
                                           state: &mut DisplayListBuildState,
                                           stacking_relative_border_box: &Rect<Au>,
                                           clip: &ClippingRegion);

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
    fn build_display_list(&mut self,
                          state: &mut DisplayListBuildState,
                          stacking_relative_flow_origin: &Point2D<Au>,
                          relative_containing_block_size: &LogicalSize<Au>,
                          relative_containing_block_mode: WritingMode,
                          border_painting_mode: BorderPaintingMode,
                          display_list_section: DisplayListSection,
                          clip: &ClippingRegion);

    /// Adjusts the clipping region for all descendants of this fragment as appropriate.
    fn adjust_clipping_region_for_children(&self,
                                           current_clip: &mut ClippingRegion,
                                           stacking_relative_border_box: &Rect<Au>);

    /// Adjusts the clipping rectangle for a fragment to take the `clip` property into account
    /// per CSS 2.1 § 11.1.2.
    fn adjust_clip_for_style(&self,
                             parent_clip: &mut ClippingRegion,
                             stacking_relative_border_box: &Rect<Au>);

    /// Builds the display items necessary to paint the selection and/or caret for this fragment,
    /// if any.
    fn build_display_items_for_selection_if_necessary(&self,
                                                      state: &mut DisplayListBuildState,
                                                      stacking_relative_border_box: &Rect<Au>,
                                                      display_list_section: DisplayListSection,
                                                      clip: &ClippingRegion);

    /// Creates the text display item for one text fragment. This can be called multiple times for
    /// one fragment if there are text shadows.
    ///
    /// `text_shadow` will be `Some` if this is rendering a shadow.
    fn build_display_list_for_text_fragment(&self,
                                            state: &mut DisplayListBuildState,
                                            text_fragment: &ScannedTextFragmentInfo,
                                            stacking_relative_content_box: &Rect<Au>,
                                            text_shadow: Option<&TextShadow>,
                                            clip: &ClippingRegion);

    /// Creates the display item for a text decoration: underline, overline, or line-through.
    fn build_display_list_for_text_decoration(&self,
                                              state: &mut DisplayListBuildState,
                                              color: &RGBA,
                                              stacking_relative_box: &LogicalRect<Au>,
                                              clip: &ClippingRegion,
                                              blur_radius: Au);

    /// A helper method that `build_display_list` calls to create per-fragment-type display items.
    fn build_fragment_type_specific_display_items(&mut self,
                                                  state: &mut DisplayListBuildState,
                                                  stacking_relative_border_box: &Rect<Au>,
                                                  clip: &ClippingRegion);

    /// Creates a stacking context for associated fragment.
    fn create_stacking_context(&self,
                               id: StackingContextId,
                               base_flow: &BaseFlow,
                               scroll_policy: ScrollPolicy,
                               mode: StackingContextCreationMode,
                               parent_scroll_id: ScrollRootId)
                               -> StackingContext;

    /// Returns the 4D matrix representing this fragment's transform.
    fn transform_matrix(&self, stacking_relative_border_box: &Rect<Au>) -> Matrix4D<f32>;
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
    let bottom_factor = scale_factor(radii.bottom_left.width, radii.bottom_right.width, size.width);
    let left_factor = scale_factor(radii.top_left.height, radii.bottom_left.height, size.height);
    let right_factor = scale_factor(radii.top_right.height, radii.bottom_right.height, size.height);
    let min_factor = top_factor.min(bottom_factor).min(left_factor).min(right_factor);
    if min_factor < 1.0 {
        radii.scale_by(min_factor)
    } else {
        *radii
    }
}

fn build_border_radius(abs_bounds: &Rect<Au>, border_style: &style_structs::Border) -> BorderRadii<Au> {
    // TODO(cgaebel): Support border radii even in the case of multiple border widths.
    // This is an extension of supporting elliptical radii. For now, all percentage
    // radii will be relative to the width.

    handle_overlapping_radii(&abs_bounds.size, &BorderRadii {
        top_left:     model::specified_border_radius(border_style.border_top_left_radius,
                                                     abs_bounds.size.width),
        top_right:    model::specified_border_radius(border_style.border_top_right_radius,
                                                     abs_bounds.size.width),
        bottom_right: model::specified_border_radius(border_style.border_bottom_right_radius,
                                                     abs_bounds.size.width),
        bottom_left:  model::specified_border_radius(border_style.border_bottom_left_radius,
                                                     abs_bounds.size.width),
    })
}

impl FragmentDisplayListBuilding for Fragment {
    fn build_display_list_for_background_if_applicable(&self,
                                                       state: &mut DisplayListBuildState,
                                                       style: &ServoComputedValues,
                                                       display_list_section: DisplayListSection,
                                                       absolute_bounds: &Rect<Au>,
                                                       clip: &ClippingRegion) {
        // Adjust the clipping region as necessary to account for `border-radius`.
        let border_radii = build_border_radius(absolute_bounds, style.get_border());
        let mut clip = (*clip).clone();
        if !border_radii.is_square() {
            clip.intersect_with_rounded_rect(absolute_bounds, &border_radii)
        }
        let background = style.get_background();

        // FIXME: This causes a lot of background colors to be displayed when they are clearly not
        // needed. We could use display list optimization to clean this up, but it still seems
        // inefficient. What we really want is something like "nearest ancestor element that
        // doesn't have a fragment".
        let background_color = style.resolve_color(background.background_color);

        // 'background-clip' determines the area within which the background is painted.
        // http://dev.w3.org/csswg/css-backgrounds-3/#the-background-clip
        let mut bounds = *absolute_bounds;

        // This is the clip for the color (which is the last element in the bg array)
        let color_clip = get_cyclic(&background.background_clip.0,
                                    background.background_image.0.len() - 1);

        match *color_clip {
            background_clip::single_value::T::border_box => {}
            background_clip::single_value::T::padding_box => {
                let border = style.logical_border_width().to_physical(style.writing_mode);
                bounds.origin.x = bounds.origin.x + border.left;
                bounds.origin.y = bounds.origin.y + border.top;
                bounds.size.width = bounds.size.width - border.horizontal();
                bounds.size.height = bounds.size.height - border.vertical();
            }
            background_clip::single_value::T::content_box => {
                let border_padding = self.border_padding.to_physical(style.writing_mode);
                bounds.origin.x = bounds.origin.x + border_padding.left;
                bounds.origin.y = bounds.origin.y + border_padding.top;
                bounds.size.width = bounds.size.width - border_padding.horizontal();
                bounds.size.height = bounds.size.height - border_padding.vertical();
            }
        }

        let base = state.create_base_display_item(&bounds,
                                                  &clip,
                                                  self.node,
                                                  style.get_cursor(Cursor::Default),
                                                  display_list_section);
        state.add_display_item(
            DisplayItem::SolidColor(box SolidColorDisplayItem {
                base: base,
                color: background_color.to_gfx_color(),
            }));

        // The background image is painted on top of the background color.
        // Implements background image, per spec:
        // http://www.w3.org/TR/CSS21/colors.html#background
        let background = style.get_background();
        for (i, background_image) in background.background_image.0.iter().enumerate().rev() {
            match background_image.0 {
                None => {}
                Some(computed::Image::Gradient(ref gradient)) => {
                    // FIXME: Radial gradients aren't implemented yet.
                    if let GradientKind::Linear(_) = gradient.gradient_kind {
                        self.build_display_list_for_background_gradient(state,
                                                                        display_list_section,
                                                                        &bounds,
                                                                        &clip,
                                                                        gradient,
                                                                        style);
                    }
                }
                Some(computed::Image::Url(ref image_url, ref _extra_data)) => {
                    self.build_display_list_for_background_image(state,
                                                                 style,
                                                                 display_list_section,
                                                                 &bounds,
                                                                 &clip,
                                                                 image_url,
                                                                 i);
                }
            }
        }
    }

    fn compute_background_image_size(&self,
                                     style: &ServoComputedValues,
                                     bounds: &Rect<Au>,
                                     image: &WebRenderImageInfo,
                                     index: usize)
                                     -> Size2D<Au> {
        // If `image_aspect_ratio` < `bounds_aspect_ratio`, the image is tall; otherwise, it is
        // wide.
        let image_aspect_ratio = (image.width as f64) / (image.height as f64);
        let bounds_aspect_ratio = bounds.size.width.to_f64_px() / bounds.size.height.to_f64_px();
        let intrinsic_size = Size2D::new(Au::from_px(image.width as i32),
                                         Au::from_px(image.height as i32));
        let background_size = get_cyclic(&style.get_background().background_size.0, index).clone();
        match (background_size, image_aspect_ratio < bounds_aspect_ratio) {
            (background_size::single_value::T::Contain, false) |
            (background_size::single_value::T::Cover, true) => {
                Size2D::new(bounds.size.width,
                            Au::from_f64_px(bounds.size.width.to_f64_px() / image_aspect_ratio))
            }

            (background_size::single_value::T::Contain, true) |
            (background_size::single_value::T::Cover, false) => {
                Size2D::new(Au::from_f64_px(bounds.size.height.to_f64_px() * image_aspect_ratio),
                            bounds.size.height)
            }

            (background_size::single_value::T::Explicit(background_size::single_value
                                                                       ::ExplicitSize {
                width,
                height: LengthOrPercentageOrAuto::Auto,
            }), _) => {
                let width = MaybeAuto::from_style(width, bounds.size.width)
                                      .specified_or_default(intrinsic_size.width);
                Size2D::new(width, Au::from_f64_px(width.to_f64_px() / image_aspect_ratio))
            }

            (background_size::single_value::T::Explicit(background_size::single_value
                                                                       ::ExplicitSize {
                width: LengthOrPercentageOrAuto::Auto,
                height
            }), _) => {
                let height = MaybeAuto::from_style(height, bounds.size.height)
                                       .specified_or_default(intrinsic_size.height);
                Size2D::new(Au::from_f64_px(height.to_f64_px() * image_aspect_ratio), height)
            }

            (background_size::single_value::T::Explicit(background_size::single_value
                                                                       ::ExplicitSize {
                width,
                height
            }), _) => {
                Size2D::new(MaybeAuto::from_style(width, bounds.size.width)
                                 .specified_or_default(intrinsic_size.width),
                       MaybeAuto::from_style(height, bounds.size.height)
                                 .specified_or_default(intrinsic_size.height))
            }
        }
    }

    fn build_display_list_for_background_image(&self,
                                               state: &mut DisplayListBuildState,
                                               style: &ServoComputedValues,
                                               display_list_section: DisplayListSection,
                                               absolute_bounds: &Rect<Au>,
                                               clip: &ClippingRegion,
                                               image_url: &Url,
                                               index: usize) {
        let background = style.get_background();
        let webrender_image = state.shared_layout_context
                                   .get_webrender_image_for_url(image_url,
                                                                UsePlaceholder::No);

        if let Some(webrender_image) = webrender_image {
            debug!("(building display list) building background image");

            // Use `background-size` to get the size.
            let mut bounds = *absolute_bounds;
            let image_size = self.compute_background_image_size(style, &bounds,
                                                                &webrender_image, index);

            // Clip.
            //
            // TODO: Check the bounds to see if a clip item is actually required.
            let mut clip = clip.clone();
            clip.intersect_rect(&bounds);

            // Background image should be positioned on the padding box basis.
            let border = style.logical_border_width().to_physical(style.writing_mode);

            // Use 'background-origin' to get the origin value.
            let origin = get_cyclic(&background.background_origin.0, index);
            let (mut origin_x, mut origin_y) = match *origin {
                background_origin::single_value::T::padding_box => {
                    (Au(0), Au(0))
                }
                background_origin::single_value::T::border_box => {
                    (-border.left, -border.top)
                }
                background_origin::single_value::T::content_box => {
                    let border_padding = self.border_padding.to_physical(self.style.writing_mode);
                    (border_padding.left - border.left, border_padding.top - border.top)
                }
            };

            // Use `background-attachment` to get the initial virtual origin
            let attachment = get_cyclic(&background.background_attachment.0, index);
            let (virtual_origin_x, virtual_origin_y) = match *attachment {
                background_attachment::single_value::T::scroll => {
                    (absolute_bounds.origin.x, absolute_bounds.origin.y)
                }
                background_attachment::single_value::T::fixed => {
                    // If the ‘background-attachment’ value for this image is ‘fixed’, then
                    // 'background-origin' has no effect.
                    origin_x = Au(0);
                    origin_y = Au(0);
                    (Au(0), Au(0))
                }
            };

            let position = *get_cyclic(&background.background_position.0, index);
            // Use `background-position` to get the offset.
            let horizontal_position = model::specified(position.horizontal,
                                                       bounds.size.width - image_size.width);
            let vertical_position = model::specified(position.vertical,
                                                     bounds.size.height - image_size.height);

            // The anchor position for this background, based on both the background-attachment
            // and background-position properties.
            let anchor_origin_x = border.left + virtual_origin_x + origin_x + horizontal_position;
            let anchor_origin_y = border.top + virtual_origin_y + origin_y + vertical_position;

            let mut tile_spacing = Size2D::zero();
            let mut stretch_size = image_size;

            // Adjust origin and size based on background-repeat
            match *get_cyclic(&background.background_repeat.0, index) {
                background_repeat::single_value::T::no_repeat => {
                    bounds.origin.x = anchor_origin_x;
                    bounds.origin.y = anchor_origin_y;
                    bounds.size.width = image_size.width;
                    bounds.size.height = image_size.height;
                }
                background_repeat::single_value::T::repeat_x => {
                    bounds.origin.y = anchor_origin_y;
                    bounds.size.height = image_size.height;
                    ImageFragmentInfo::tile_image(&mut bounds.origin.x,
                                                  &mut bounds.size.width,
                                                  anchor_origin_x,
                                                  image_size.width);
                }
                background_repeat::single_value::T::repeat_y => {
                    bounds.origin.x = anchor_origin_x;
                    bounds.size.width = image_size.width;
                    ImageFragmentInfo::tile_image(&mut bounds.origin.y,
                                                  &mut bounds.size.height,
                                                  anchor_origin_y,
                                                  image_size.height);
                }
                background_repeat::single_value::T::repeat => {
                    ImageFragmentInfo::tile_image(&mut bounds.origin.x,
                                                  &mut bounds.size.width,
                                                  anchor_origin_x,
                                                  image_size.width);
                    ImageFragmentInfo::tile_image(&mut bounds.origin.y,
                                                  &mut bounds.size.height,
                                                  anchor_origin_y,
                                                  image_size.height);
                }
                background_repeat::single_value::T::space => {
                    ImageFragmentInfo::tile_image_spaced(&mut bounds.origin.x,
                                                         &mut bounds.size.width,
                                                         &mut tile_spacing.width,
                                                         anchor_origin_x,
                                                         image_size.width);
                    ImageFragmentInfo::tile_image_spaced(&mut bounds.origin.y,
                                                         &mut bounds.size.height,
                                                         &mut tile_spacing.height,
                                                         anchor_origin_y,
                                                         image_size.height);

                }
                background_repeat::single_value::T::round => {
                    ImageFragmentInfo::tile_image_round(&mut bounds.origin.x,
                                                        &mut bounds.size.width,
                                                        anchor_origin_x,
                                                        &mut stretch_size.width);
                    ImageFragmentInfo::tile_image_round(&mut bounds.origin.y,
                                                        &mut bounds.size.height,
                                                        anchor_origin_y,
                                                        &mut stretch_size.height);
                }
            };

            // Create the image display item.
            let base = state.create_base_display_item(&bounds,
                                                    &clip,
                                                    self.node,
                                                    style.get_cursor(Cursor::Default),
                                                    display_list_section);
            state.add_display_item(DisplayItem::Image(box ImageDisplayItem {
              base: base,
              webrender_image: webrender_image,
              image_data: None,
              stretch_size: stretch_size,
              tile_spacing: tile_spacing,
              image_rendering: style.get_inheritedbox().image_rendering.clone(),
            }));

        }
    }

    fn build_display_list_for_background_gradient(&self,
                                                  state: &mut DisplayListBuildState,
                                                  display_list_section: DisplayListSection,
                                                  absolute_bounds: &Rect<Au>,
                                                  clip: &ClippingRegion,
                                                  gradient: &Gradient,
                                                  style: &ServoComputedValues) {
        let mut clip = clip.clone();
        clip.intersect_rect(absolute_bounds);


        // FIXME: Repeating gradients aren't implemented yet.
        if gradient.repeating {
          return;
        }
        let angle = if let GradientKind::Linear(angle_or_corner) = gradient.gradient_kind {
            match angle_or_corner {
                AngleOrCorner::Angle(angle) => angle.radians(),
                AngleOrCorner::Corner(horizontal, vertical) => {
                    // This the angle for one of the diagonals of the box. Our angle
                    // will either be this one, this one + PI, or one of the other
                    // two perpendicular angles.
                    let atan = (absolute_bounds.size.height.to_f32_px() /
                                absolute_bounds.size.width.to_f32_px()).atan();
                    match (horizontal, vertical) {
                        (HorizontalDirection::Right, VerticalDirection::Bottom)
                            => f32::consts::PI - atan,
                        (HorizontalDirection::Left, VerticalDirection::Bottom)
                            => f32::consts::PI + atan,
                        (HorizontalDirection::Right, VerticalDirection::Top)
                            => atan,
                        (HorizontalDirection::Left, VerticalDirection::Top)
                            => -atan,
                    }
                }
            }
        } else {
            // FIXME: Radial gradients aren't implemented yet.
            return;
        };

        // Get correct gradient line length, based on:
        // https://drafts.csswg.org/css-images-3/#linear-gradients
        let dir = Point2D::new(angle.sin(), -angle.cos());

        let line_length = (dir.x * absolute_bounds.size.width.to_f32_px()).abs() +
                          (dir.y * absolute_bounds.size.height.to_f32_px()).abs();

        let inv_dir_length = 1.0 / (dir.x * dir.x + dir.y * dir.y).sqrt();

        // This is the vector between the center and the ending point; i.e. half
        // of the distance between the starting point and the ending point.
        let delta = Point2D::new(Au::from_f32_px(dir.x * inv_dir_length * line_length / 2.0),
                                 Au::from_f32_px(dir.y * inv_dir_length * line_length / 2.0));

        // This is the length of the gradient line.
        let length = Au::from_f32_px(
            (delta.x.to_f32_px() * 2.0).hypot(delta.y.to_f32_px() * 2.0));

        // Determine the position of each stop per CSS-IMAGES § 3.4.
        //
        // FIXME(#3908, pcwalton): Make sure later stops can't be behind earlier stops.
        let mut stops = Vec::with_capacity(gradient.stops.len());
        let mut stop_run = None;
        for (i, stop) in gradient.stops.iter().enumerate() {
            let offset = match stop.position {
                None => {
                    if stop_run.is_none() {
                        // Initialize a new stop run.
                        let start_offset = if i == 0 {
                            0.0
                        } else {
                            // `unwrap()` here should never fail because this is the beginning of
                            // a stop run, which is always bounded by a length or percentage.
                            position_to_offset(gradient.stops[i - 1].position.unwrap(), length)
                        };
                        let (end_index, end_offset) =
                            match gradient.stops[i..]
                                          .iter()
                                          .enumerate()
                                          .find(|&(_, ref stop)| stop.position.is_some()) {
                                None => (gradient.stops.len() - 1, 1.0),
                                Some((end_index, end_stop)) => {
                                    // `unwrap()` here should never fail because this is the end of
                                    // a stop run, which is always bounded by a length or
                                    // percentage.
                                    (end_index,
                                     position_to_offset(end_stop.position.unwrap(), length))
                                }
                            };
                        stop_run = Some(StopRun {
                            start_offset: start_offset,
                            end_offset: end_offset,
                            start_index: i,
                            stop_count: end_index - i,
                        })
                    }

                    let stop_run = stop_run.unwrap();
                    let stop_run_length = stop_run.end_offset - stop_run.start_offset;
                    if stop_run.stop_count == 0 {
                        stop_run.end_offset
                    } else {
                        stop_run.start_offset +
                            stop_run_length * (i - stop_run.start_index) as f32 /
                                (stop_run.stop_count as f32)
                    }
                }
                Some(position) => {
                    stop_run = None;
                    position_to_offset(position, length)
                }
            };
            stops.push(GradientStop {
                offset: offset,
                color: style.resolve_color(stop.color).to_gfx_color()
            })
        }

        let center = Point2D::new(absolute_bounds.origin.x + absolute_bounds.size.width / 2,
                                  absolute_bounds.origin.y + absolute_bounds.size.height / 2);

        let base = state.create_base_display_item(absolute_bounds,
                                                  &clip,
                                                  self.node,
                                                  style.get_cursor(Cursor::Default),
                                                  display_list_section);
        let gradient_display_item = DisplayItem::Gradient(box GradientDisplayItem {
            base: base,
            start_point: center - delta,
            end_point: center + delta,
            stops: stops,
        });

        state.add_display_item(gradient_display_item);
    }

    fn build_display_list_for_box_shadow_if_applicable(&self,
                                                       state: &mut DisplayListBuildState,
                                                       style: &ServoComputedValues,
                                                       display_list_section: DisplayListSection,
                                                       absolute_bounds: &Rect<Au>,
                                                       clip: &ClippingRegion) {
        // NB: According to CSS-BACKGROUNDS, box shadows render in *reverse* order (front to back).
        for box_shadow in style.get_effects().box_shadow.0.iter().rev() {
            let bounds =
                shadow_bounds(&absolute_bounds.translate(&Point2D::new(box_shadow.offset_x,
                                                                       box_shadow.offset_y)),
                              box_shadow.blur_radius,
                              box_shadow.spread_radius);

            // TODO(pcwalton): Multiple border radii; elliptical border radii.
            let base = state.create_base_display_item(&bounds,
                                                      &clip,
                                                      self.node,
                                                      style.get_cursor(Cursor::Default),
                                                      display_list_section);
            state.add_display_item(DisplayItem::BoxShadow(box BoxShadowDisplayItem {
                base: base,
                box_bounds: *absolute_bounds,
                color: style.resolve_color(box_shadow.color).to_gfx_color(),
                offset: Point2D::new(box_shadow.offset_x, box_shadow.offset_y),
                blur_radius: box_shadow.blur_radius,
                spread_radius: box_shadow.spread_radius,
                border_radius: model::specified_border_radius(style.get_border()
                                                                   .border_top_left_radius,
                                                              absolute_bounds.size.width).width,
                clip_mode: if box_shadow.inset {
                    BoxShadowClipMode::Inset
                } else {
                    BoxShadowClipMode::Outset
                },
            }));
        }
    }

    fn build_display_list_for_borders_if_applicable(
            &self,
            state: &mut DisplayListBuildState,
            style: &ServoComputedValues,
            border_painting_mode: BorderPaintingMode,
            bounds: &Rect<Au>,
            display_list_section: DisplayListSection,
            clip: &ClippingRegion) {
        let mut border = style.logical_border_width();

        match border_painting_mode {
            BorderPaintingMode::Separate => {}
            BorderPaintingMode::Collapse(collapsed_borders) => {
                collapsed_borders.adjust_border_widths_for_painting(&mut border)
            }
            BorderPaintingMode::Hidden => return,
        }
        if border.is_zero() {
            return
        }

        let border_style_struct = style.get_border();
        let mut colors = SideOffsets2D::new(border_style_struct.border_top_color,
                                            border_style_struct.border_right_color,
                                            border_style_struct.border_bottom_color,
                                            border_style_struct.border_left_color);
        let mut border_style = SideOffsets2D::new(border_style_struct.border_top_style,
                                                  border_style_struct.border_right_style,
                                                  border_style_struct.border_bottom_style,
                                                  border_style_struct.border_left_style);
        if let BorderPaintingMode::Collapse(collapsed_borders) = border_painting_mode {
            collapsed_borders.adjust_border_colors_and_styles_for_painting(&mut colors,
                                                                           &mut border_style,
                                                                           style.writing_mode);
        }

        let colors = SideOffsets2D::new(style.resolve_color(colors.top),
                                        style.resolve_color(colors.right),
                                        style.resolve_color(colors.bottom),
                                        style.resolve_color(colors.left));

        // If this border collapses, then we draw outside the boundaries we were given.
        let mut bounds = *bounds;
        if let BorderPaintingMode::Collapse(collapsed_borders) = border_painting_mode {
            collapsed_borders.adjust_border_bounds_for_painting(&mut bounds, style.writing_mode)
        }

        // Append the border to the display list.
        let base = state.create_base_display_item(&bounds,
                                                  &clip,
                                                  self.node,
                                                  style.get_cursor(Cursor::Default),
                                                  display_list_section);
        state.add_display_item(DisplayItem::Border(box BorderDisplayItem {
            base: base,
            border_widths: border.to_physical(style.writing_mode),
            color: SideOffsets2D::new(colors.top.to_gfx_color(),
                                      colors.right.to_gfx_color(),
                                      colors.bottom.to_gfx_color(),
                                      colors.left.to_gfx_color()),
            style: border_style,
            radius: build_border_radius(&bounds, border_style_struct),
        }));
    }

    fn build_display_list_for_outline_if_applicable(&self,
                                                    state: &mut DisplayListBuildState,
                                                    style: &ServoComputedValues,
                                                    bounds: &Rect<Au>,
                                                    clip: &ClippingRegion) {
        let width = style.get_outline().outline_width;
        if width == Au(0) {
            return
        }

        let outline_style = style.get_outline().outline_style;
        if outline_style == border_style::T::none {
            return
        }

        // Outlines are not accounted for in the dimensions of the border box, so adjust the
        // absolute bounds.
        let mut bounds = *bounds;
        let offset = width + style.get_outline().outline_offset;
        bounds.origin.x = bounds.origin.x - offset;
        bounds.origin.y = bounds.origin.y - offset;
        bounds.size.width = bounds.size.width + offset + offset;
        bounds.size.height = bounds.size.height + offset + offset;

        // Append the outline to the display list.
        let color = style.resolve_color(style.get_outline().outline_color).to_gfx_color();
        let base = state.create_base_display_item(&bounds,
                                                  &clip,
                                                  self.node,
                                                  style.get_cursor(Cursor::Default),
                                                  DisplayListSection::Outlines);
        state.add_display_item(DisplayItem::Border(box BorderDisplayItem {
            base: base,
            border_widths: SideOffsets2D::new_all_same(width),
            color: SideOffsets2D::new_all_same(color),
            style: SideOffsets2D::new_all_same(outline_style),
            radius: Default::default(),
        }));
    }

    fn build_debug_borders_around_text_fragments(&self,
                                                 state: &mut DisplayListBuildState,
                                                 style: &ServoComputedValues,
                                                 stacking_relative_border_box: &Rect<Au>,
                                                 stacking_relative_content_box: &Rect<Au>,
                                                 text_fragment: &ScannedTextFragmentInfo,
                                                 clip: &ClippingRegion) {
        // FIXME(pcwalton, #2795): Get the real container size.
        let container_size = Size2D::zero();

        // Compute the text fragment bounds and draw a border surrounding them.
        let base = state.create_base_display_item(stacking_relative_border_box,
                                                  clip,
                                                  self.node,
                                                  style.get_cursor(Cursor::Default),
                                                  DisplayListSection::Content);
        state.add_display_item(DisplayItem::Border(box BorderDisplayItem {
            base: base,
            border_widths: SideOffsets2D::new_all_same(Au::from_px(1)),
            color: SideOffsets2D::new_all_same(ColorF::rgb(0, 0, 200)),
            style: SideOffsets2D::new_all_same(border_style::T::solid),
            radius: Default::default(),
        }));

        // Draw a rectangle representing the baselines.
        let mut baseline = LogicalRect::from_physical(self.style.writing_mode,
                                                      *stacking_relative_content_box,
                                                      container_size);
        baseline.start.b = baseline.start.b + text_fragment.run.ascent();
        baseline.size.block = Au(0);
        let baseline = baseline.to_physical(self.style.writing_mode, container_size);

        let base = state.create_base_display_item(&baseline,
                                                  clip,
                                                  self.node,
                                                  style.get_cursor(Cursor::Default),
                                                  DisplayListSection::Content);
        state.add_display_item(DisplayItem::Line(box LineDisplayItem {
            base: base,
            color: ColorF::rgb(0, 200, 0),
            style: border_style::T::dashed,
        }));
    }

    fn build_debug_borders_around_fragment(&self,
                                           state: &mut DisplayListBuildState,
                                           stacking_relative_border_box: &Rect<Au>,
                                           clip: &ClippingRegion) {
        // This prints a debug border around the border of this fragment.
        let base = state.create_base_display_item(stacking_relative_border_box,
                                                  clip,
                                                  self.node,
                                                  self.style.get_cursor(Cursor::Default),
                                                  DisplayListSection::Content);
        state.add_display_item(DisplayItem::Border(box BorderDisplayItem {
            base: base,
            border_widths: SideOffsets2D::new_all_same(Au::from_px(1)),
            color: SideOffsets2D::new_all_same(ColorF::rgb(0, 0, 200)),
            style: SideOffsets2D::new_all_same(border_style::T::solid),
            radius: Default::default(),
        }));
    }

    fn adjust_clip_for_style(&self,
                             parent_clip: &mut ClippingRegion,
                             stacking_relative_border_box: &Rect<Au>) {
        // Account for `clip` per CSS 2.1 § 11.1.2.
        let style_clip_rect = match (self.style().get_box().position,
                                     self.style().get_effects().clip.0) {
            (position::T::absolute, Some(style_clip_rect)) => style_clip_rect,
            _ => return,
        };

        // FIXME(pcwalton, #2795): Get the real container size.
        let clip_origin = Point2D::new(stacking_relative_border_box.origin.x + style_clip_rect.left,
                                       stacking_relative_border_box.origin.y + style_clip_rect.top);
        let right = style_clip_rect.right.unwrap_or(stacking_relative_border_box.size.width);
        let bottom = style_clip_rect.bottom.unwrap_or(stacking_relative_border_box.size.height);
        let clip_size = Size2D::new(right - clip_origin.x, bottom - clip_origin.y);
        parent_clip.intersect_rect(&Rect::new(clip_origin, clip_size))
    }

    fn build_display_items_for_selection_if_necessary(&self,
                                                      state: &mut DisplayListBuildState,
                                                      stacking_relative_border_box: &Rect<Au>,
                                                      display_list_section: DisplayListSection,
                                                      clip: &ClippingRegion) {
        let scanned_text_fragment_info = match self.specific {
            SpecificFragmentInfo::ScannedText(ref scanned_text_fragment_info) => {
                scanned_text_fragment_info
            }
            _ => return,
        };

        // Draw a highlighted background if the text is selected.
        //
        // TODO: Allow non-text fragments to be selected too.
        if scanned_text_fragment_info.selected() {
            let style = self.selected_style();
            let background_color = style.resolve_color(style.get_background().background_color);
            let base = state.create_base_display_item(stacking_relative_border_box,
                                                      &clip,
                                                      self.node,
                                                      self.style.get_cursor(Cursor::Default),
                                                      display_list_section);
            state.add_display_item(
                DisplayItem::SolidColor(box SolidColorDisplayItem {
                    base: base,
                    color: background_color.to_gfx_color(),
            }));
        }

        // Draw a caret at the insertion point.
        let insertion_point_index = match scanned_text_fragment_info.insertion_point {
            Some(insertion_point_index) => insertion_point_index,
            None => return,
        };
        let range = Range::new(scanned_text_fragment_info.range.begin(),
                               insertion_point_index - scanned_text_fragment_info.range.begin());
        let advance = scanned_text_fragment_info.run.advance_for_range(&range);

        let insertion_point_bounds;
        let cursor;
        if !self.style.writing_mode.is_vertical() {
            insertion_point_bounds =
                Rect::new(Point2D::new(stacking_relative_border_box.origin.x + advance,
                                       stacking_relative_border_box.origin.y),
                          Size2D::new(INSERTION_POINT_LOGICAL_WIDTH,
                                      stacking_relative_border_box.size.height));
            cursor = Cursor::Text;
        } else {
            insertion_point_bounds =
                Rect::new(Point2D::new(stacking_relative_border_box.origin.x,
                                       stacking_relative_border_box.origin.y + advance),
                          Size2D::new(stacking_relative_border_box.size.width,
                                      INSERTION_POINT_LOGICAL_WIDTH));
            cursor = Cursor::VerticalText;
        };

        let base = state.create_base_display_item(&insertion_point_bounds,
                                                  &clip,
                                                  self.node,
                                                  self.style.get_cursor(cursor),
                                                  display_list_section);
        state.add_display_item(DisplayItem::SolidColor(box SolidColorDisplayItem {
            base: base,
            color: self.style().get_color().color.to_gfx_color(),
        }));
    }

    fn build_display_list(&mut self,
                          state: &mut DisplayListBuildState,
                          stacking_relative_flow_origin: &Point2D<Au>,
                          relative_containing_block_size: &LogicalSize<Au>,
                          relative_containing_block_mode: WritingMode,
                          border_painting_mode: BorderPaintingMode,
                          display_list_section: DisplayListSection,
                          clip: &ClippingRegion) {
        self.restyle_damage.remove(REPAINT);
        if self.style().get_inheritedbox().visibility != visibility::T::visible {
            return
        }

        // Compute the fragment position relative to the parent stacking context. If the fragment
        // itself establishes a stacking context, then the origin of its position will be (0, 0)
        // for the purposes of this computation.
        let stacking_relative_border_box =
            self.stacking_relative_border_box(stacking_relative_flow_origin,
                                              relative_containing_block_size,
                                              relative_containing_block_mode,
                                              CoordinateSystem::Own);

        debug!("Fragment::build_display_list at rel={:?}, abs={:?}, flow origin={:?}: {:?}",
               self.border_box,
               stacking_relative_border_box,
               stacking_relative_flow_origin,
               self);

        // Check the clip rect. If there's nothing to render at all, don't even construct display
        // list items.
        let empty_rect = !clip.might_intersect_rect(&stacking_relative_border_box);
        if self.is_primary_fragment() && !empty_rect {
            // Add shadows, background, borders, and outlines, if applicable.
            if let Some(ref inline_context) = self.inline_context {
                for node in inline_context.nodes.iter().rev() {
                    self.build_display_list_for_background_if_applicable(
                        state,
                        &*node.style,
                        display_list_section,
                        &stacking_relative_border_box,
                        clip);
                    self.build_display_list_for_box_shadow_if_applicable(
                        state,
                        &*node.style,
                        display_list_section,
                        &stacking_relative_border_box,
                        clip);

                    let mut style = node.style.clone();
                    properties::modify_border_style_for_inline_sides(
                        &mut style,
                        node.flags.contains(FIRST_FRAGMENT_OF_ELEMENT),
                        node.flags.contains(LAST_FRAGMENT_OF_ELEMENT));
                    self.build_display_list_for_borders_if_applicable(
                        state,
                        &*style,
                        border_painting_mode,
                        &stacking_relative_border_box,
                        display_list_section,
                        clip);

                    self.build_display_list_for_outline_if_applicable(
                        state,
                        &*node.style,
                        &stacking_relative_border_box,
                        clip);
                }
            }

            if !self.is_scanned_text_fragment() {
                self.build_display_list_for_background_if_applicable(state,
                                                                     &*self.style,
                                                                     display_list_section,
                                                                     &stacking_relative_border_box,
                                                                     clip);
                self.build_display_list_for_box_shadow_if_applicable(state,
                                                                     &*self.style,
                                                                     display_list_section,
                                                                     &stacking_relative_border_box,
                                                                     clip);
                self.build_display_list_for_borders_if_applicable(state,
                                                                  &*self.style,
                                                                  border_painting_mode,
                                                                  &stacking_relative_border_box,
                                                                  display_list_section,
                                                                  clip);
                self.build_display_list_for_outline_if_applicable(state,
                                                                  &*self.style,
                                                                  &stacking_relative_border_box,
                                                                  clip);
            }
        }

        if self.is_primary_fragment() {
            // Paint the selection point if necessary.  Even an empty text fragment may have an
            // insertion point, so we do this even if `empty_rect` is true.
            self.build_display_items_for_selection_if_necessary(state,
                                                                &stacking_relative_border_box,
                                                                display_list_section,
                                                                clip);
        }

        if empty_rect {
            return
        }

        debug!("Fragment::build_display_list: intersected. Adding display item...");

        // Create special per-fragment-type display items.
        self.build_fragment_type_specific_display_items(state,
                                                        &stacking_relative_border_box,
                                                        clip);

        if opts::get().show_debug_fragment_borders {
           self.build_debug_borders_around_fragment(state, &stacking_relative_border_box, clip)
        }
    }

    fn build_fragment_type_specific_display_items(&mut self,
                                                  state: &mut DisplayListBuildState,
                                                  stacking_relative_border_box: &Rect<Au>,
                                                  clip: &ClippingRegion) {
        // Compute the context box position relative to the parent stacking context.
        let stacking_relative_content_box =
            self.stacking_relative_content_box(stacking_relative_border_box);

        match self.specific {
            SpecificFragmentInfo::ScannedText(ref text_fragment) => {
                // Create items for shadows.
                //
                // NB: According to CSS-BACKGROUNDS, text shadows render in *reverse* order (front
                // to back).

                for text_shadow in self.style.get_inheritedtext().text_shadow.0.iter().rev() {
                    self.build_display_list_for_text_fragment(state,
                                                              &**text_fragment,
                                                              &stacking_relative_content_box,
                                                              Some(text_shadow),
                                                              clip);
                }

                // Create the main text display item.
                self.build_display_list_for_text_fragment(state,
                                                          &**text_fragment,
                                                          &stacking_relative_content_box,
                                                          None,
                                                          clip);

                if opts::get().show_debug_fragment_borders {
                    self.build_debug_borders_around_text_fragments(state,
                                                                   self.style(),
                                                                   stacking_relative_border_box,
                                                                   &stacking_relative_content_box,
                                                                   &**text_fragment,
                                                                   clip);
                }
            }
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
            SpecificFragmentInfo::Svg(_) => {
                if opts::get().show_debug_fragment_borders {
                    self.build_debug_borders_around_fragment(state,
                                                             stacking_relative_border_box,
                                                             clip);
                }
            }
            SpecificFragmentInfo::Iframe(ref fragment_info) => {
                if !stacking_relative_content_box.is_empty() {
                    let base = state.create_base_display_item(
                        &stacking_relative_content_box,
                        clip,
                        self.node,
                        self.style.get_cursor(Cursor::Default),
                        DisplayListSection::Content);
                    let item = DisplayItem::Iframe(box IframeDisplayItem {
                        base: base,
                        iframe: fragment_info.pipeline_id,
                    });

                    state.add_display_item(item);
                }
            }
            SpecificFragmentInfo::Image(ref mut image_fragment) => {
                // Place the image into the display list.
                if let Some(ref image) = image_fragment.image {
                    let base = state.create_base_display_item(
                        &stacking_relative_content_box,
                        clip,
                        self.node,
                        self.style.get_cursor(Cursor::Default),
                        DisplayListSection::Content);
                    state.add_display_item(DisplayItem::Image(box ImageDisplayItem {
                        base: base,
                        webrender_image: WebRenderImageInfo::from_image(image),
                        image_data: Some(Arc::new(image.bytes.clone())),
                        stretch_size: stacking_relative_content_box.size,
                        tile_spacing: Size2D::zero(),
                        image_rendering: self.style.get_inheritedbox().image_rendering.clone(),
                    }));
                }
            }
            SpecificFragmentInfo::Canvas(ref canvas_fragment_info) => {
                let width = canvas_fragment_info.replaced_image_fragment_info
                    .computed_inline_size.map_or(0, |w| w.to_px() as usize);
                let height = canvas_fragment_info.replaced_image_fragment_info
                    .computed_block_size.map_or(0, |h| h.to_px() as usize);
                if width > 0 && height > 0 {
                    let computed_width = canvas_fragment_info.canvas_inline_size().to_px();
                    let computed_height = canvas_fragment_info.canvas_block_size().to_px();

                    let canvas_data = match canvas_fragment_info.ipc_renderer {
                        Some(ref ipc_renderer) => {
                            let ipc_renderer = ipc_renderer.lock().unwrap();
                            let (sender, receiver) = ipc::channel().unwrap();
                            ipc_renderer.send(CanvasMsg::FromLayout(
                                FromLayoutMsg::SendData(sender))).unwrap();
                            receiver.recv().unwrap()
                        },
                        None => return,
                    };

                    let base = state.create_base_display_item(
                        &stacking_relative_content_box,
                        clip,
                        self.node,
                        self.style.get_cursor(Cursor::Default),
                        DisplayListSection::Content);
                    let display_item = match canvas_data {
                        CanvasData::Image(canvas_data) => {
                            DisplayItem::Image(box ImageDisplayItem {
                                base: base,
                                webrender_image: WebRenderImageInfo {
                                    width: computed_width as u32,
                                    height: computed_height as u32,
                                    format: PixelFormat::RGBA8,
                                    key: Some(canvas_data.image_key),
                                },
                                image_data: None,
                                stretch_size: stacking_relative_content_box.size,
                                tile_spacing: Size2D::zero(),
                                image_rendering: image_rendering::T::auto,
                            })
                        }
                        CanvasData::WebGL(context_id) => {
                            DisplayItem::WebGL(box WebGLDisplayItem {
                                base: base,
                                context_id: context_id,
                            })
                        }
                    };

                    state.add_display_item(display_item);
                }
            }
            SpecificFragmentInfo::UnscannedText(_) => {
                panic!("Shouldn't see unscanned fragments here.")
            }
            SpecificFragmentInfo::TableColumn(_) => {
                panic!("Shouldn't see table column fragments here.")
            }
        }
    }

    fn create_stacking_context(&self,
                               id: StackingContextId,
                               base_flow: &BaseFlow,
                               scroll_policy: ScrollPolicy,
                               mode: StackingContextCreationMode,
                               parent_scroll_id: ScrollRootId)
                               -> StackingContext {
        let border_box =
             self.stacking_relative_border_box(&base_flow.stacking_relative_position,
                                               &base_flow.early_absolute_position_info
                                                         .relative_containing_block_size,
                                               base_flow.early_absolute_position_info
                                                          .relative_containing_block_mode,
                                               CoordinateSystem::Parent);
        // First, compute the offset of our border box (including relative positioning)
        // from our flow origin, since that is what `BaseFlow::overflow` is relative to.
        let border_box_offset =
            border_box.translate(&-base_flow.stacking_relative_position).origin;
        // Then, using that, compute our overflow region relative to our border box.
        let overflow = base_flow.overflow.paint.translate(&-border_box_offset);

        let transform = self.transform_matrix(&border_box);
        let perspective = match self.style().get_effects().perspective {
            Either::First(length) => {
                let perspective_origin = self.style().get_effects().perspective_origin;
                let perspective_origin =
                    Point2D::new(model::specified(perspective_origin.horizontal,
                                                  border_box.size.width).to_f32_px(),
                                 model::specified(perspective_origin.vertical,
                                                  border_box.size.height).to_f32_px());

                let pre_transform = Matrix4D::create_translation(perspective_origin.x,
                                                                 perspective_origin.y,
                                                                 0.0);
                let post_transform = Matrix4D::create_translation(-perspective_origin.x,
                                                                  -perspective_origin.y,
                                                                  0.0);

                let perspective_matrix = create_perspective_matrix(length);

                pre_transform.pre_mul(&perspective_matrix).pre_mul(&post_transform)
            }
            Either::Second(values::None_) => {
                Matrix4D::identity()
            }
        };

        // Create the filter pipeline.
        let effects = self.style().get_effects();
        let mut filters = effects.filter.clone();
        if effects.opacity != 1.0 {
            filters.push(Filter::Opacity(effects.opacity))
        }

        let transform_style = self.style().get_used_transform_style();
        let establishes_3d_context = transform_style == transform_style::T::flat;

        let context_type = match mode {
            StackingContextCreationMode::PseudoFloat => StackingContextType::PseudoFloat,
            StackingContextCreationMode::PseudoPositioned => StackingContextType::PseudoPositioned,
            _ => StackingContextType::Real,
        };

        StackingContext::new(id,
                             context_type,
                             &border_box,
                             &overflow,
                             self.effective_z_index(),
                             filters,
                             self.style().get_effects().mix_blend_mode,
                             transform,
                             perspective,
                             establishes_3d_context,
                             scroll_policy,
                             parent_scroll_id)
    }

    fn adjust_clipping_region_for_children(&self,
                                           current_clip: &mut ClippingRegion,
                                           stacking_relative_border_box: &Rect<Au>) {
        // Don't clip if we're text.
        if self.is_scanned_text_fragment() {
            return
        }

        let overflow_x = self.style.get_box().overflow_x;
        let overflow_y = self.style.get_box().overflow_y.0;
        if overflow_x == overflow_x::T::visible && overflow_y == overflow_x::T::visible {
            return
        }

        let overflow_clip_rect_owner;
        let overflow_clip_rect = match self.style.get_box()._servo_overflow_clip_box {
            overflow_clip_box::T::padding_box => {
                // FIXME(SimonSapin): should be the padding box, not border box.
                stacking_relative_border_box
            }
            overflow_clip_box::T::content_box => {
                overflow_clip_rect_owner =
                    self.stacking_relative_content_box(stacking_relative_border_box);
                &overflow_clip_rect_owner
            }
        };

        // Clip according to the values of `overflow-x` and `overflow-y`.
        //
        // FIXME(pcwalton): This may be more complex than it needs to be, since it seems to be
        // impossible with the computed value rules as they are to have `overflow-x: visible`
        // with `overflow-y: <scrolling>` or vice versa!
        if let overflow_x::T::hidden = self.style.get_box().overflow_x {
            let mut bounds = current_clip.bounding_rect();
            let max_x = cmp::min(bounds.max_x(), overflow_clip_rect.max_x());
            bounds.origin.x = cmp::max(bounds.origin.x, overflow_clip_rect.origin.x);
            bounds.size.width = max_x - bounds.origin.x;
            current_clip.intersect_rect(&bounds)
        }
        if let overflow_x::T::hidden = self.style.get_box().overflow_y.0 {
            let mut bounds = current_clip.bounding_rect();
            let max_y = cmp::min(bounds.max_y(), overflow_clip_rect.max_y());
            bounds.origin.y = cmp::max(bounds.origin.y, overflow_clip_rect.origin.y);
            bounds.size.height = max_y - bounds.origin.y;
            current_clip.intersect_rect(&bounds)
        }

        let border_radii = build_border_radius(stacking_relative_border_box,
                                               self.style.get_border());
        if !border_radii.is_square() {
            current_clip.intersect_with_rounded_rect(stacking_relative_border_box,
                                                     &border_radii)
        }
    }

    fn build_display_list_for_text_fragment(&self,
                                            state: &mut DisplayListBuildState,
                                            text_fragment: &ScannedTextFragmentInfo,
                                            stacking_relative_content_box: &Rect<Au>,
                                            text_shadow: Option<&TextShadow>,
                                            clip: &ClippingRegion) {
        // TODO(emilio): Allow changing more properties by ::selection
        let text_color = if let Some(shadow) = text_shadow {
            // If we're painting a shadow, paint the text the same color as the shadow.
            self.style().resolve_color(shadow.color)
        } else if text_fragment.selected() {
            // Otherwise, paint the text with the color as described in its styling.
            self.selected_style().get_color().color
        } else {
            self.style().get_color().color
        };
        let offset = text_shadow.map(|s| Point2D::new(s.offset_x, s.offset_y)).unwrap_or_else(Point2D::zero);
        let shadow_blur_radius = text_shadow.map(|s| s.blur_radius).unwrap_or(Au(0));

        // Determine the orientation and cursor to use.
        let (orientation, cursor) = if self.style.writing_mode.is_vertical() {
            if self.style.writing_mode.is_sideways_left() {
                (TextOrientation::SidewaysLeft, Cursor::VerticalText)
            } else {
                (TextOrientation::SidewaysRight, Cursor::VerticalText)
            }
        } else {
            (TextOrientation::Upright, Cursor::Text)
        };

        // Compute location of the baseline.
        //
        // FIXME(pcwalton): Get the real container size.
        let container_size = Size2D::zero();
        let metrics = &text_fragment.run.font_metrics;
        let stacking_relative_content_box = stacking_relative_content_box.translate(&offset);
        let baseline_origin = stacking_relative_content_box.origin +
            LogicalPoint::new(self.style.writing_mode,
                              Au(0),
                              metrics.ascent).to_physical(self.style.writing_mode,
                                                          container_size);

        // Create the text display item.
        let base = state.create_base_display_item(&stacking_relative_content_box,
                                                  clip,
                                                  self.node,
                                                  self.style().get_cursor(cursor),
                                                  DisplayListSection::Content);
        state.add_display_item(DisplayItem::Text(box TextDisplayItem {
            base: base,
            text_run: text_fragment.run.clone(),
            range: text_fragment.range,
            text_color: text_color.to_gfx_color(),
            orientation: orientation,
            baseline_origin: baseline_origin,
            blur_radius: shadow_blur_radius,
        }));

        // Create display items for text decorations.
        let mut text_decorations = self.style()
                                       .get_inheritedtext()
                                       ._servo_text_decorations_in_effect;
        // Note that the text decoration colors are always the same as the text color.
        text_decorations.underline = text_decorations.underline.map(|_| text_color);
        text_decorations.overline = text_decorations.overline.map(|_| text_color);
        text_decorations.line_through = text_decorations.line_through.map(|_| text_color);

        let stacking_relative_content_box =
            LogicalRect::from_physical(self.style.writing_mode,
                                       stacking_relative_content_box,
                                       container_size);
        if let Some(ref underline_color) = text_decorations.underline {
            let mut stacking_relative_box = stacking_relative_content_box;
            stacking_relative_box.start.b = stacking_relative_content_box.start.b +
                metrics.ascent - metrics.underline_offset;
            stacking_relative_box.size.block = metrics.underline_size;
            self.build_display_list_for_text_decoration(state,
                                                        underline_color,
                                                        &stacking_relative_box,
                                                        clip,
                                                        shadow_blur_radius);
        }

        if let Some(ref overline_color) = text_decorations.overline {
            let mut stacking_relative_box = stacking_relative_content_box;
            stacking_relative_box.size.block = metrics.underline_size;
            self.build_display_list_for_text_decoration(state,
                                                        overline_color,
                                                        &stacking_relative_box,
                                                        clip,
                                                        shadow_blur_radius);
        }

        if let Some(ref line_through_color) = text_decorations.line_through {
            let mut stacking_relative_box = stacking_relative_content_box;
            stacking_relative_box.start.b = stacking_relative_box.start.b + metrics.ascent -
                metrics.strikeout_offset;
            stacking_relative_box.size.block = metrics.strikeout_size;
            self.build_display_list_for_text_decoration(state,
                                                        line_through_color,
                                                        &stacking_relative_box,
                                                        clip,
                                                        shadow_blur_radius);
        }
    }

    fn build_display_list_for_text_decoration(&self,
                                              state: &mut DisplayListBuildState,
                                              color: &RGBA,
                                              stacking_relative_box: &LogicalRect<Au>,
                                              clip: &ClippingRegion,
                                              blur_radius: Au) {
        // Perhaps surprisingly, text decorations are box shadows. This is because they may need
        // to have blur in the case of `text-shadow`, and this doesn't hurt performance because box
        // shadows are optimized into essentially solid colors if there is no need for the blur.
        //
        // FIXME(pcwalton, #2795): Get the real container size.
        let container_size = Size2D::zero();
        let stacking_relative_box = stacking_relative_box.to_physical(self.style.writing_mode,
                                                                      container_size);
        let base = state.create_base_display_item(
            &shadow_bounds(&stacking_relative_box, blur_radius, Au(0)),
            clip,
            self.node,
            self.style.get_cursor(Cursor::Default),
            DisplayListSection::Content);
        state.add_display_item(DisplayItem::BoxShadow(box BoxShadowDisplayItem {
            base: base,
            box_bounds: stacking_relative_box,
            color: color.to_gfx_color(),
            offset: Point2D::zero(),
            blur_radius: blur_radius,
            spread_radius: Au(0),
            border_radius: Au(0),
            clip_mode: BoxShadowClipMode::None,
        }));
    }

    fn transform_matrix(&self, stacking_relative_border_box: &Rect<Au>) -> Matrix4D<f32> {
        let mut transform = Matrix4D::identity();
        let operations = match self.style.get_effects().transform.0 {
            None => return transform,
            Some(ref operations) => operations,
        };

        let transform_origin = &self.style.get_effects().transform_origin;
        let transform_origin_x = model::specified(transform_origin.horizontal,
                                                  stacking_relative_border_box.size
                                                                              .width).to_f32_px();
        let transform_origin_y = model::specified(transform_origin.vertical,
                                                  stacking_relative_border_box.size
                                                                              .height).to_f32_px();
        let transform_origin_z = transform_origin.depth.to_f32_px();

        let pre_transform = Matrix4D::create_translation(transform_origin_x,
                                                         transform_origin_y,
                                                         transform_origin_z);
        let post_transform = Matrix4D::create_translation(-transform_origin_x,
                                                          -transform_origin_y,
                                                          -transform_origin_z);

        for operation in operations {
            let matrix = match *operation {
                transform::ComputedOperation::Rotate(ax, ay, az, theta) => {
                    let theta = 2.0f32 * f32::consts::PI - theta.radians();
                    Matrix4D::create_rotation(ax, ay, az, Radians::new(theta))
                }
                transform::ComputedOperation::Perspective(d) => {
                    create_perspective_matrix(d)
                }
                transform::ComputedOperation::Scale(sx, sy, sz) => {
                    Matrix4D::create_scale(sx, sy, sz)
                }
                transform::ComputedOperation::Translate(tx, ty, tz) => {
                    let tx =
                        model::specified(tx, stacking_relative_border_box.size.width).to_f32_px();
                    let ty =
                        model::specified(ty, stacking_relative_border_box.size.height).to_f32_px();
                    let tz = tz.to_f32_px();
                    Matrix4D::create_translation(tx, ty, tz)
                }
                transform::ComputedOperation::Matrix(m) => {
                    m.to_gfx_matrix()
                }
                transform::ComputedOperation::Skew(theta_x, theta_y) => {
                    Matrix4D::create_skew(Radians::new(theta_x.radians()),
                                          Radians::new(theta_y.radians()))
                }
            };

            transform = transform.pre_mul(&matrix);
        }

        pre_transform.pre_mul(&transform).pre_mul(&post_transform)
    }
}

pub trait BlockFlowDisplayListBuilding {
    fn collect_stacking_contexts_for_block(&mut self,
                                           parent: &mut StackingContext,
                                           parent_scroll_root_id: ScrollRootId);
    fn build_display_list_for_block(&mut self,
                                    state: &mut DisplayListBuildState,
                                    border_painting_mode: BorderPaintingMode);

    /// Changes this block's clipping region from its parent's coordinate system to its own
    /// coordinate system if necessary (i.e. if this block is a stacking context).
    ///
    /// The clipping region is initially in each block's parent's coordinate system because the
    /// parent of each block does not have enough information to determine what the child's
    /// coordinate system is on its own. Specifically, if the child is absolutely positioned, the
    /// parent does not know where the child's absolute position is at the time it assigns clipping
    /// regions, because flows compute their own absolute positions.
    fn switch_coordinate_system_if_necessary(&mut self);
}

impl BlockFlowDisplayListBuilding for BlockFlow {
    fn collect_stacking_contexts_for_block(&mut self,
                                           parent: &mut StackingContext,
                                           parent_scroll_root_id: ScrollRootId) {
        let scroll_root_id = if self.has_scrolling_overflow() {
            ScrollRootId::new_of_type(self.fragment.node.id() as usize,
                                      self.fragment.fragment_type())
        } else {
            parent_scroll_root_id
        };
        self.base.scroll_root_id = scroll_root_id;

        let block_stacking_context_type = self.block_stacking_context_type();
        if block_stacking_context_type == BlockStackingContextType::NonstackingContext {
            self.base.stacking_context_id = parent.id;
            self.base.collect_stacking_contexts_for_children(parent, scroll_root_id);
            return;
        }

        let stacking_context_id = StackingContextId::new_of_type(self.fragment.node.id() as usize,
                                                                 self.fragment.fragment_type());
        self.base.stacking_context_id = stacking_context_id;

        if block_stacking_context_type == BlockStackingContextType::PseudoStackingContext {
            let creation_mode = if self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) ||
                                   self.fragment.style.get_box().position != position::T::static_ {
                StackingContextCreationMode::PseudoPositioned
            } else {
                assert!(self.base.flags.is_float());
                StackingContextCreationMode::PseudoFloat
            };

            let mut new_context = self.fragment.create_stacking_context(stacking_context_id,
                                                                        &self.base,
                                                                        ScrollPolicy::Scrollable,
                                                                        creation_mode,
                                                                        parent_scroll_root_id);
            self.base.collect_stacking_contexts_for_children(&mut new_context, scroll_root_id);
            let new_children: Vec<StackingContext> = new_context.children.drain(..).collect();

            let mut non_floating_children = Vec::new();
            for child in new_children {
                if child.context_type == StackingContextType::PseudoFloat {
                    new_context.children.push(child);
                } else {
                    non_floating_children.push(child);
                }
            }

            parent.add_child(new_context);
            parent.children.append(&mut non_floating_children);
            return;
        }

        let scroll_policy = if self.is_fixed() {
            ScrollPolicy::FixedPosition
        } else {
            ScrollPolicy::Scrollable
        };

        let mut stacking_context = self.fragment.create_stacking_context(
            stacking_context_id,
            &self.base,
            scroll_policy,
            StackingContextCreationMode::Normal,
            parent_scroll_root_id);
        self.base.collect_stacking_contexts_for_children(&mut stacking_context, scroll_root_id);
        parent.add_child(stacking_context);
    }

    fn build_display_list_for_block(&mut self,
                                    state: &mut DisplayListBuildState,
                                    border_painting_mode: BorderPaintingMode) {
        let establishes_stacking_context = self.fragment.establishes_stacking_context();
        let background_border_section = if self.base.flags.is_float() {
            DisplayListSection::BackgroundAndBorders
        } else if self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) {
            if establishes_stacking_context {
                DisplayListSection::BackgroundAndBorders
            } else {
                DisplayListSection::BlockBackgroundsAndBorders
            }
        } else {
            DisplayListSection::BlockBackgroundsAndBorders
        };

        if self.has_scrolling_overflow() {
            let size = self.base.overflow.scroll.size;
            let mut clip = self.fragment.stacking_relative_border_box(
                &self.base.stacking_relative_position,
                &self.base.early_absolute_position_info.relative_containing_block_size,
                self.base.early_absolute_position_info.relative_containing_block_mode,
                CoordinateSystem::Parent);
            if establishes_stacking_context {
                clip = Rect::new(TypedPoint2D::zero(), clip.size);
            }

            let parent_id = state.parent_scroll_root_id();
            state.add_scroll_root(
                ScrollRoot {
                    id: self.base.scroll_root_id,
                    parent_id: parent_id,
                    clip: clip,
                    size: size,
                }
            );
        }

        // Add the box that starts the block context.
        self.fragment
            .build_display_list(state,
                                &self.base.stacking_relative_position,
                                &self.base
                                     .early_absolute_position_info
                                     .relative_containing_block_size,
                                self.base
                                    .early_absolute_position_info
                                    .relative_containing_block_mode,
                                border_painting_mode,
                                background_border_section,
                                &self.base.clip);

        self.base.build_display_items_for_debugging_tint(state, self.fragment.node);
    }

    fn switch_coordinate_system_if_necessary(&mut self) {
        // Avoid overflows!
        if self.base.clip.is_max() {
            return
        }

        if !self.fragment.establishes_stacking_context() {
            return
        }

        let stacking_relative_border_box =
            self.fragment.stacking_relative_border_box(&self.base.stacking_relative_position,
                                                       &self.base
                                                            .early_absolute_position_info
                                                            .relative_containing_block_size,
                                                       self.base
                                                           .early_absolute_position_info
                                                           .relative_containing_block_mode,
                                                       CoordinateSystem::Parent);
        self.base.clip = self.base.clip.translate(&-stacking_relative_border_box.origin);

        // Account for `transform`, if applicable.
        if self.fragment.style.get_effects().transform.0.is_none() {
            return
        }
        let transform = match self.fragment
                                  .transform_matrix(&stacking_relative_border_box)
                                  .inverse() {
            Some(transform) => transform,
            None => {
                // Singular matrix. Ignore it.
                return
            }
        };

        // FIXME(pcwalton): This is inaccurate: not all transforms are 2D, and not all clips are
        // axis-aligned.
        let bounding_rect = self.base.clip.bounding_rect();
        let bounding_rect = Rect::new(Point2D::new(bounding_rect.origin.x.to_f32_px(),
                                                   bounding_rect.origin.y.to_f32_px()),
                                      Size2D::new(bounding_rect.size.width.to_f32_px(),
                                                  bounding_rect.size.height.to_f32_px()));
        let clip_rect = transform.to_2d().transform_rect(&bounding_rect);
        let clip_rect = Rect::new(Point2D::new(Au::from_f32_px(clip_rect.origin.x),
                                               Au::from_f32_px(clip_rect.origin.y)),
                                  Size2D::new(Au::from_f32_px(clip_rect.size.width),
                                              Au::from_f32_px(clip_rect.size.height)));
        self.base.clip = ClippingRegion::from_rect(&clip_rect)
    }
}

pub trait InlineFlowDisplayListBuilding {
    fn collect_stacking_contexts_for_inline(&mut self,
                                            parent: &mut StackingContext,
                                            parent_scroll_root_id: ScrollRootId);
    fn build_display_list_for_inline_fragment_at_index(&mut self,
                                                       state: &mut DisplayListBuildState,
                                                       index: usize);
    fn build_display_list_for_inline(&mut self, state: &mut DisplayListBuildState);
}

impl InlineFlowDisplayListBuilding for InlineFlow {
    fn collect_stacking_contexts_for_inline(&mut self,
                                            parent: &mut StackingContext,
                                            parent_scroll_root_id: ScrollRootId) {
        self.base.stacking_context_id = parent.id;
        self.base.scroll_root_id = parent_scroll_root_id;

        for mut fragment in self.fragments.fragments.iter_mut() {
            match fragment.specific {
                SpecificFragmentInfo::InlineBlock(ref mut block_flow) => {
                    let block_flow = FlowRef::deref_mut(&mut block_flow.flow_ref);
                    block_flow.collect_stacking_contexts(parent, parent_scroll_root_id);
                }
                SpecificFragmentInfo::InlineAbsoluteHypothetical(ref mut block_flow) => {
                    let block_flow = FlowRef::deref_mut(&mut block_flow.flow_ref);
                    block_flow.collect_stacking_contexts(parent, parent_scroll_root_id);
                }
                _ if fragment.establishes_stacking_context() => {
                    fragment.stacking_context_id =
                        StackingContextId::new_of_type(fragment.fragment_id(),
                                                       fragment.fragment_type());
                    parent.add_child(fragment.create_stacking_context(
                        fragment.stacking_context_id,
                        &self.base,
                        ScrollPolicy::Scrollable,
                        StackingContextCreationMode::Normal,
                        parent_scroll_root_id));
                }
                _ => fragment.stacking_context_id = parent.id,
            }
        }
    }

    fn build_display_list_for_inline_fragment_at_index(&mut self,
                                                       state: &mut DisplayListBuildState,
                                                       index: usize) {
        let fragment = self.fragments.fragments.get_mut(index).unwrap();
        fragment.build_display_list(state,
                                    &self.base.stacking_relative_position,
                                    &self.base
                                         .early_absolute_position_info
                                         .relative_containing_block_size,
                                    self.base
                                        .early_absolute_position_info
                                        .relative_containing_block_mode,
                                    BorderPaintingMode::Separate,
                                    DisplayListSection::Content,
                                    &self.base.clip);
    }

    fn build_display_list_for_inline(&mut self, state: &mut DisplayListBuildState) {
        // TODO(#228): Once we form lines and have their cached bounds, we can be smarter and
        // not recurse on a line if nothing in it can intersect the dirty region.
        debug!("Flow: building display list for {} inline fragments", self.fragments.len());

        // We iterate using an index here, because we want to avoid doing a doing
        // a double-borrow of self (one mutable for the method call and one immutable
        // for the self.fragments.fragment iterator itself).
        for index in 0..self.fragments.fragments.len() {
            let (establishes_stacking_context, stacking_context_id) = {
                let fragment = self.fragments.fragments.get(index).unwrap();
                (self.base.stacking_context_id != fragment.stacking_context_id,
                 fragment.stacking_context_id)
            };

            if establishes_stacking_context {
                state.push_stacking_context_id(stacking_context_id);
            }

            self.build_display_list_for_inline_fragment_at_index(state, index);

            if establishes_stacking_context {
                state.pop_stacking_context_id();
            }
        }

        if !self.fragments.fragments.is_empty() {
            self.base.build_display_items_for_debugging_tint(state,
                                                             self.fragments.fragments[0].node);
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
            marker.build_display_list(state,
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
                                      &self.block_flow.base.clip);
        }

        // Draw the rest of the block.
        self.block_flow.build_display_list_for_block(state, BorderPaintingMode::Separate)
    }
}

pub trait FlexFlowDisplayListBuilding {
    fn build_display_list_for_flex(&mut self, state: &mut DisplayListBuildState);
}

impl FlexFlowDisplayListBuilding for FlexFlow {
    fn build_display_list_for_flex(&mut self, state: &mut DisplayListBuildState) {
        // Draw the rest of the block.
        self.as_mut_block().build_display_list_for_block(state, BorderPaintingMode::Separate)
    }
}

trait BaseFlowDisplayListBuilding {
    fn build_display_items_for_debugging_tint(&self,
                                              state: &mut DisplayListBuildState,
                                              node: OpaqueNode);
}

impl BaseFlowDisplayListBuilding for BaseFlow {
    fn build_display_items_for_debugging_tint(&self,
                                              state: &mut DisplayListBuildState,
                                              node: OpaqueNode) {
        if !opts::get().show_debug_parallel_layout {
            return
        }

        let thread_id = self.thread_id;
        let stacking_context_relative_bounds =
            Rect::new(self.stacking_relative_position,
                      self.position.size.to_physical(self.writing_mode));

        let mut color = THREAD_TINT_COLORS[thread_id as usize % THREAD_TINT_COLORS.len()];
        color.a = 1.0;
        let base = state.create_base_display_item(
            &stacking_context_relative_bounds.inflate(Au::from_px(2), Au::from_px(2)),
            &self.clip,
            node,
            None,
            DisplayListSection::Content);
        state.add_display_item(DisplayItem::Border(box BorderDisplayItem {
            base: base,
            border_widths: SideOffsets2D::new_all_same(Au::from_px(2)),
            color: SideOffsets2D::new_all_same(color),
            style: SideOffsets2D::new_all_same(border_style::T::solid),
            radius: BorderRadii::all_same(Au(0)),
        }));
    }
}

trait ServoComputedValuesCursorUtility {
    fn get_cursor(&self, default_cursor: Cursor) -> Option<Cursor>;
}

impl ServoComputedValuesCursorUtility for ServoComputedValues {
    /// Gets the cursor to use given the specific ServoComputedValues.  `default_cursor` specifies
    /// the cursor to use if `cursor` is `auto`. Typically, this will be `PointerCursor`, but for
    /// text display items it may be `TextCursor` or `VerticalTextCursor`.
    #[inline]
    fn get_cursor(&self, default_cursor: Cursor) -> Option<Cursor> {
        match (self.get_pointing().pointer_events, self.get_pointing().cursor) {
            (pointer_events::T::none, _) => None,
            (pointer_events::T::auto, cursor::T::AutoCursor) => Some(default_cursor),
            (pointer_events::T::auto, cursor::T::SpecifiedCursor(cursor)) => Some(cursor),
        }
    }
}

// A helper data structure for gradients.
#[derive(Copy, Clone)]
struct StopRun {
    start_offset: f32,
    end_offset: f32,
    start_index: usize,
    stop_count: usize,
}

fn position_to_offset(position: LengthOrPercentage, Au(total_length): Au) -> f32 {
    match position {
        LengthOrPercentage::Length(Au(length)) => {
            (1.0f32).min(length as f32 / total_length as f32)
        }
        LengthOrPercentage::Percentage(percentage) => percentage as f32,
        LengthOrPercentage::Calc(calc) =>
            (1.0f32).min(calc.percentage() + (calc.length().0 as f32) / (total_length as f32)),
    }
}

/// Adjusts `content_rect` as necessary for the given spread, and blur so that the resulting
/// bounding rect contains all of a shadow's ink.
fn shadow_bounds(content_rect: &Rect<Au>, blur_radius: Au, spread_radius: Au) -> Rect<Au> {
    let inflation = spread_radius + blur_radius * BLUR_INFLATION_FACTOR;
    content_rect.inflate(inflation, inflation)
}

/// Allows a CSS color to be converted into a graphics color.
pub trait ToGfxColor {
    /// Converts a CSS color to a graphics color.
    fn to_gfx_color(&self) -> ColorF;
}

impl ToGfxColor for RGBA {
    fn to_gfx_color(&self) -> ColorF {
        ColorF::new(self.red, self.green, self.blue, self.alpha)
    }
}

/// Describes how to paint the borders.
#[derive(Copy, Clone)]
pub enum BorderPaintingMode<'a> {
    /// Paint borders separately (`border-collapse: separate`).
    Separate,
    /// Paint collapsed borders.
    Collapse(&'a CollapsedBordersForCell),
    /// Paint no borders.
    Hidden,
}

#[derive(Copy, Clone, PartialEq)]
pub enum StackingContextCreationMode {
    Normal,
    PseudoPositioned,
    PseudoFloat,
}

