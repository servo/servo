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
use azure::azure_hl::Color;
use block::{BlockFlow, BlockStackingContextType};
use canvas_traits::{CanvasMsg, CanvasPixelData, CanvasData, FromLayoutMsg};
use context::LayoutContext;
use euclid::num::Zero;
use euclid::{Matrix4, Point2D, Point3D, Rect, SideOffsets2D, Size2D};
use flex::FlexFlow;
use flow::{BaseFlow, Flow, IS_ABSOLUTELY_POSITIONED};
use flow_ref;
use fragment::{CoordinateSystem, Fragment, HAS_LAYER, ImageFragmentInfo, ScannedTextFragmentInfo};
use fragment::{SpecificFragmentInfo};
use gfx::display_list::{BLUR_INFLATION_FACTOR, BaseDisplayItem, BorderDisplayItem};
use gfx::display_list::{BorderRadii, BoxShadowClipMode, BoxShadowDisplayItem, ClippingRegion};
use gfx::display_list::{DisplayItem, DisplayItemMetadata, DisplayListSection};
use gfx::display_list::{GradientDisplayItem};
use gfx::display_list::{GradientStop, IframeDisplayItem, ImageDisplayItem, WebGLDisplayItem, LayeredItem, LayerInfo};
use gfx::display_list::{LineDisplayItem, OpaqueNode, SolidColorDisplayItem};
use gfx::display_list::{StackingContext, StackingContextId, StackingContextType};
use gfx::display_list::{TextDisplayItem, TextOrientation, DisplayListEntry, WebRenderImageInfo};
use gfx::paint_thread::THREAD_TINT_COLORS;
use gfx::text::glyph::CharIndex;
use gfx_traits::{color, ScrollPolicy};
use inline::{FIRST_FRAGMENT_OF_ELEMENT, InlineFlow, LAST_FRAGMENT_OF_ELEMENT};
use ipc_channel::ipc::{self, IpcSharedMemory};
use list_item::ListItemFlow;
use model::{self, MaybeAuto, ToGfxMatrix};
use net_traits::image::base::PixelFormat;
use net_traits::image_cache_thread::UsePlaceholder;
use range::Range;
use std::default::Default;
use std::sync::Arc;
use std::{cmp, f32};
use style::computed_values::filter::Filter;
use style::computed_values::{_servo_overflow_clip_box as overflow_clip_box};
use style::computed_values::{background_attachment, background_clip, background_origin};
use style::computed_values::{background_repeat, background_size};
use style::computed_values::{border_style, image_rendering, overflow_x, position};
use style::computed_values::{transform, transform_style, visibility};
use style::logical_geometry::{LogicalPoint, LogicalRect, LogicalSize, WritingMode};
use style::properties::style_structs::Border;
use style::properties::{self, ComputedValues, ServoComputedValues};
use style::values::RGBA;
use style::values::computed;
use style::values::computed::{LengthOrNone, LengthOrPercentage, LengthOrPercentageOrAuto, LinearGradient};
use style::values::specified::{AngleOrCorner, HorizontalDirection, VerticalDirection};
use style_traits::cursor::Cursor;
use table_cell::CollapsedBordersForCell;
use url::Url;
use util::opts;

pub struct DisplayListBuildState<'a> {
    pub layout_context: &'a LayoutContext<'a>,
    pub items: Vec<DisplayListEntry>,
    pub stacking_context_id_stack: Vec<StackingContextId>,
}

impl<'a> DisplayListBuildState<'a> {
    pub fn new(layout_context: &'a LayoutContext, stacking_context_id: StackingContextId)
               -> DisplayListBuildState<'a> {
        DisplayListBuildState {
            layout_context: layout_context,
            items: Vec::new(),
            stacking_context_id_stack: vec!(stacking_context_id),
        }
    }

    fn add_display_item(&mut self, display_item: DisplayItem, section: DisplayListSection) {
        let stacking_context_id = self.stacking_context_id();
        self.items.push(
            DisplayListEntry {
                stacking_context_id: stacking_context_id,
                section: section,
                item: display_item
            });
    }

    fn stacking_context_id(&self) -> StackingContextId {
        self.stacking_context_id_stack.last().unwrap().clone()
    }

    pub fn push_stacking_context_id(&mut self, stacking_context_id: StackingContextId) {
        self.stacking_context_id_stack.push(stacking_context_id);
    }

    pub fn pop_stacking_context_id(&mut self) {
        self.stacking_context_id_stack.pop();
        assert!(!self.stacking_context_id_stack.is_empty());
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
fn create_perspective_matrix(d: Au) -> Matrix4 {
    let d = d.to_f32_px();
    if d <= 0.0 {
        Matrix4::identity()
    } else {
        Matrix4::create_perspective(d)
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
                                     image: &WebRenderImageInfo)
                                     -> Size2D<Au>;

    /// Adds the display items necessary to paint the background image of this fragment to the
    /// appropriate section of the display list.
    fn build_display_list_for_background_image(&self,
                                               state: &mut DisplayListBuildState,
                                               style: &ServoComputedValues,
                                               display_list_section: DisplayListSection,
                                               absolute_bounds: &Rect<Au>,
                                               clip: &ClippingRegion,
                                               image_url: &Url);

    /// Adds the display items necessary to paint the background linear gradient of this fragment
    /// to the appropriate section of the display list.
    fn build_display_list_for_background_linear_gradient(&self,
                                                         state: &mut DisplayListBuildState,
                                                         display_list_section: DisplayListSection,
                                                         absolute_bounds: &Rect<Au>,
                                                         clip: &ClippingRegion,
                                                         gradient: &LinearGradient,
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
    /// * `stacking_relative_display_port`: The position and size of the display port with respect
    ///   to the nearest ancestor stacking context.
    fn build_display_list(&mut self,
                          state: &mut DisplayListBuildState,
                          stacking_relative_flow_origin: &Point2D<Au>,
                          relative_containing_block_size: &LogicalSize<Au>,
                          relative_containing_block_mode: WritingMode,
                          border_painting_mode: BorderPaintingMode,
                          display_list_section: DisplayListSection,
                          clip: &ClippingRegion,
                          stacking_relative_display_port: &Rect<Au>);

    /// Adjusts the clipping region for descendants of this fragment as appropriate.
    fn adjust_clipping_region_for_children(&self,
                                           current_clip: &mut ClippingRegion,
                                           stacking_relative_border_box: &Rect<Au>,
                                           is_absolutely_positioned: bool);

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
    /// `shadow_blur_radius` will be `Some` if this is a shadow, even if the blur radius is zero.
    fn build_display_list_for_text_fragment(&self,
                                            state: &mut DisplayListBuildState,
                                            text_fragment: &ScannedTextFragmentInfo,
                                            text_color: RGBA,
                                            stacking_relative_content_box: &Rect<Au>,
                                            shadow_blur_radius: Option<Au>,
                                            offset: &Point2D<Au>,
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
                               mode: StackingContextCreationMode)
                               -> Box<StackingContext>;
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

fn build_border_radius(abs_bounds: &Rect<Au>, border_style: &Border) -> BorderRadii<Au> {
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

        // FIXME: This causes a lot of background colors to be displayed when they are clearly not
        // needed. We could use display list optimization to clean this up, but it still seems
        // inefficient. What we really want is something like "nearest ancestor element that
        // doesn't have a fragment".
        let background_color = style.resolve_color(style.get_background().background_color);

        // 'background-clip' determines the area within which the background is painted.
        // http://dev.w3.org/csswg/css-backgrounds-3/#the-background-clip
        let mut bounds = *absolute_bounds;

        match style.get_background().background_clip {
            background_clip::T::border_box => {}
            background_clip::T::padding_box => {
                let border = style.logical_border_width().to_physical(style.writing_mode);
                bounds.origin.x = bounds.origin.x + border.left;
                bounds.origin.y = bounds.origin.y + border.top;
                bounds.size.width = bounds.size.width - border.horizontal();
                bounds.size.height = bounds.size.height - border.vertical();
            }
            background_clip::T::content_box => {
                let border_padding = self.border_padding.to_physical(style.writing_mode);
                bounds.origin.x = bounds.origin.x + border_padding.left;
                bounds.origin.y = bounds.origin.y + border_padding.top;
                bounds.size.width = bounds.size.width - border_padding.horizontal();
                bounds.size.height = bounds.size.height - border_padding.vertical();
            }
        }

        state.add_display_item(
            DisplayItem::SolidColorClass(box SolidColorDisplayItem {
                base: BaseDisplayItem::new(&bounds,
                                           DisplayItemMetadata::new(self.node,
                                                                    style,
                                                                    Cursor::DefaultCursor),
                                           &clip),
                color: background_color.to_gfx_color(),
            }), display_list_section);

        // The background image is painted on top of the background color.
        // Implements background image, per spec:
        // http://www.w3.org/TR/CSS21/colors.html#background
        let background = style.get_background();
        match background.background_image.0 {
            None => {}
            Some(computed::Image::LinearGradient(ref gradient)) => {
                self.build_display_list_for_background_linear_gradient(state,
                                                                       display_list_section,
                                                                       &bounds,
                                                                       &clip,
                                                                       gradient,
                                                                       style);
            }
            Some(computed::Image::Url(ref image_url)) => {
                self.build_display_list_for_background_image(state,
                                                             style,
                                                             display_list_section,
                                                             &bounds,
                                                             &clip,
                                                             image_url);
            }
        }
    }

    fn compute_background_image_size(&self,
                                     style: &ServoComputedValues,
                                     bounds: &Rect<Au>,
                                     image: &WebRenderImageInfo)
                                     -> Size2D<Au> {
        // If `image_aspect_ratio` < `bounds_aspect_ratio`, the image is tall; otherwise, it is
        // wide.
        let image_aspect_ratio = (image.width as f64) / (image.height as f64);
        let bounds_aspect_ratio = bounds.size.width.to_f64_px() / bounds.size.height.to_f64_px();
        let intrinsic_size = Size2D::new(Au::from_px(image.width as i32),
                                         Au::from_px(image.height as i32));
        match (style.get_background().background_size.clone(),
               image_aspect_ratio < bounds_aspect_ratio) {
            (background_size::T::Contain, false) | (background_size::T::Cover, true) => {
                Size2D::new(bounds.size.width,
                            Au::from_f64_px(bounds.size.width.to_f64_px() / image_aspect_ratio))
            }

            (background_size::T::Contain, true) | (background_size::T::Cover, false) => {
                Size2D::new(Au::from_f64_px(bounds.size.height.to_f64_px() * image_aspect_ratio),
                            bounds.size.height)
            }

            (background_size::T::Explicit(background_size::ExplicitSize {
                width,
                height: LengthOrPercentageOrAuto::Auto,
            }), _) => {
                let width = MaybeAuto::from_style(width, bounds.size.width)
                                      .specified_or_default(intrinsic_size.width);
                Size2D::new(width, Au::from_f64_px(width.to_f64_px() / image_aspect_ratio))
            }

            (background_size::T::Explicit(background_size::ExplicitSize {
                width: LengthOrPercentageOrAuto::Auto,
                height
            }), _) => {
                let height = MaybeAuto::from_style(height, bounds.size.height)
                                       .specified_or_default(intrinsic_size.height);
                Size2D::new(Au::from_f64_px(height.to_f64_px() * image_aspect_ratio), height)
            }

            (background_size::T::Explicit(background_size::ExplicitSize {
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
                                               image_url: &Url) {
        let background = style.get_background();
        let fetch_image_data_as_well = !opts::get().use_webrender;
        let webrender_image =
            state.layout_context.get_webrender_image_for_url(image_url,
                                                             UsePlaceholder::No,
                                                             fetch_image_data_as_well);
        if let Some((webrender_image, image_data)) = webrender_image {
            debug!("(building display list) building background image");

            // Use `background-size` to get the size.
            let mut bounds = *absolute_bounds;
            let image_size = self.compute_background_image_size(style, &bounds, &webrender_image);

            // Clip.
            //
            // TODO: Check the bounds to see if a clip item is actually required.
            let mut clip = clip.clone();
            clip.intersect_rect(&bounds);

            // Background image should be positioned on the padding box basis.
            let border = style.logical_border_width().to_physical(style.writing_mode);

            // Use 'background-origin' to get the origin value.
            let (mut origin_x, mut origin_y) = match background.background_origin {
                background_origin::T::padding_box => {
                    (Au(0), Au(0))
                }
                background_origin::T::border_box => {
                    (-border.left, -border.top)
                }
                background_origin::T::content_box => {
                    let border_padding = self.border_padding.to_physical(self.style.writing_mode);
                    (border_padding.left - border.left, border_padding.top - border.top)
                }
            };

            // Use `background-attachment` to get the initial virtual origin
            let (virtual_origin_x, virtual_origin_y) = match background.background_attachment {
                background_attachment::T::scroll => {
                    (absolute_bounds.origin.x, absolute_bounds.origin.y)
                }
                background_attachment::T::fixed => {
                    // If the ‘background-attachment’ value for this image is ‘fixed’, then
                    // 'background-origin' has no effect.
                    origin_x = Au(0);
                    origin_y = Au(0);
                    (Au(0), Au(0))
                }
            };

            // Use `background-position` to get the offset.
            let horizontal_position = model::specified(background.background_position.horizontal,
                                                       bounds.size.width - image_size.width);
            let vertical_position = model::specified(background.background_position.vertical,
                                                     bounds.size.height - image_size.height);

            let abs_x = border.left + virtual_origin_x + horizontal_position + origin_x;
            let abs_y = border.top + virtual_origin_y + vertical_position + origin_y;

            // Adjust origin and size based on background-repeat
            match background.background_repeat {
                background_repeat::T::no_repeat => {
                    bounds.origin.x = abs_x;
                    bounds.origin.y = abs_y;
                    bounds.size.width = image_size.width;
                    bounds.size.height = image_size.height;
                }
                background_repeat::T::repeat_x => {
                    bounds.origin.y = abs_y;
                    bounds.size.height = image_size.height;
                    ImageFragmentInfo::tile_image(&mut bounds.origin.x,
                                                  &mut bounds.size.width,
                                                  abs_x,
                                                  image_size.width.to_nearest_px() as u32);
                }
                background_repeat::T::repeat_y => {
                    bounds.origin.x = abs_x;
                    bounds.size.width = image_size.width;
                    ImageFragmentInfo::tile_image(&mut bounds.origin.y,
                                                  &mut bounds.size.height,
                                                  abs_y,
                                                  image_size.height.to_nearest_px() as u32);
                }
                background_repeat::T::repeat => {
                    ImageFragmentInfo::tile_image(&mut bounds.origin.x,
                                                  &mut bounds.size.width,
                                                  abs_x,
                                                  image_size.width.to_nearest_px() as u32);
                    ImageFragmentInfo::tile_image(&mut bounds.origin.y,
                                                  &mut bounds.size.height,
                                                  abs_y,
                                                  image_size.height.to_nearest_px() as u32);
                }
            };

            // Create the image display item.
            state.add_display_item(DisplayItem::ImageClass(box ImageDisplayItem {
                base: BaseDisplayItem::new(&bounds,
                                           DisplayItemMetadata::new(self.node,
                                                                    style,
                                                                    Cursor::DefaultCursor),
                                           &clip),
                webrender_image: webrender_image,
                image_data: image_data.map(Arc::new),
                stretch_size: Size2D::new(image_size.width, image_size.height),
                image_rendering: style.get_inheritedbox().image_rendering.clone(),
            }), display_list_section);
        }
    }

    fn build_display_list_for_background_linear_gradient(&self,
                                                         state: &mut DisplayListBuildState,
                                                         display_list_section: DisplayListSection,
                                                         absolute_bounds: &Rect<Au>,
                                                         clip: &ClippingRegion,
                                                         gradient: &LinearGradient,
                                                         style: &ServoComputedValues) {
        let mut clip = clip.clone();
        clip.intersect_rect(absolute_bounds);

        // This is the distance between the center and the ending point; i.e. half of the distance
        // between the starting point and the ending point.
        let delta = match gradient.angle_or_corner {
            AngleOrCorner::Angle(angle) => {
                // Get correct gradient line length, based on:
                // https://drafts.csswg.org/css-images-3/#linear-gradients
                let dir = Point2D::new(angle.radians().sin(), -angle.radians().cos());

                let line_length = (dir.x * absolute_bounds.size.width.to_f32_px()).abs() +
                                  (dir.y * absolute_bounds.size.height.to_f32_px()).abs();

                let inv_dir_length = 1.0 / (dir.x * dir.x + dir.y * dir.y).sqrt();

                Point2D::new(Au::from_f32_px(dir.x * inv_dir_length * line_length / 2.0),
                             Au::from_f32_px(dir.y * inv_dir_length * line_length / 2.0))
            }
            AngleOrCorner::Corner(horizontal, vertical) => {
                let x_factor = match horizontal {
                    HorizontalDirection::Left => -1,
                    HorizontalDirection::Right => 1,
                };
                let y_factor = match vertical {
                    VerticalDirection::Top => -1,
                    VerticalDirection::Bottom => 1,
                };
                Point2D::new(absolute_bounds.size.width * x_factor / 2,
                             absolute_bounds.size.height * y_factor / 2)
            }
        };

        // This is the length of the gradient line.
        let length = Au::from_f32_px(
            (delta.x.to_f32_px() * 2.0).hypot(delta.y.to_f32_px() * 2.0));

        // Determine the position of each stop per CSS-IMAGES § 3.4.
        //
        // FIXME(#3908, pcwalton): Make sure later stops can't be behind earlier stops.
        let (mut stops, mut stop_run) = (Vec::new(), None);
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

        let gradient_display_item = DisplayItem::GradientClass(box GradientDisplayItem {
            base: BaseDisplayItem::new(absolute_bounds,
                                       DisplayItemMetadata::new(self.node,
                                                                style,
                                                                Cursor::DefaultCursor),
                                       &clip),
            start_point: center - delta,
            end_point: center + delta,
            stops: stops,
        });

        state.add_display_item(gradient_display_item, display_list_section);
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
            state.add_display_item(DisplayItem::BoxShadowClass(box BoxShadowDisplayItem {
                base: BaseDisplayItem::new(&bounds,
                                           DisplayItemMetadata::new(self.node,
                                                                    style,
                                                                    Cursor::DefaultCursor),
                                           clip),
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
            }), display_list_section);
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
        state.add_display_item(DisplayItem::BorderClass(box BorderDisplayItem {
            base: BaseDisplayItem::new(&bounds,
                                       DisplayItemMetadata::new(self.node,
                                                                style,
                                                                Cursor::DefaultCursor),
                                       clip),
            border_widths: border.to_physical(style.writing_mode),
            color: SideOffsets2D::new(colors.top.to_gfx_color(),
                                      colors.right.to_gfx_color(),
                                      colors.bottom.to_gfx_color(),
                                      colors.left.to_gfx_color()),
            style: border_style,
            radius: build_border_radius(&bounds, border_style_struct),
        }), display_list_section);
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
        state.add_display_item(DisplayItem::BorderClass(box BorderDisplayItem {
            base: BaseDisplayItem::new(&bounds,
                                       DisplayItemMetadata::new(self.node,
                                                                style,
                                                                Cursor::DefaultCursor),
                                       clip),
            border_widths: SideOffsets2D::new_all_same(width),
            color: SideOffsets2D::new_all_same(color),
            style: SideOffsets2D::new_all_same(outline_style),
            radius: Default::default(),
        }), DisplayListSection::Outlines);
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
        state.add_display_item(DisplayItem::BorderClass(box BorderDisplayItem {
            base: BaseDisplayItem::new(stacking_relative_border_box,
                                       DisplayItemMetadata::new(self.node,
                                                                style,
                                                                Cursor::DefaultCursor),
                                       clip),
            border_widths: SideOffsets2D::new_all_same(Au::from_px(1)),
            color: SideOffsets2D::new_all_same(color::rgb(0, 0, 200)),
            style: SideOffsets2D::new_all_same(border_style::T::solid),
            radius: Default::default(),
        }), DisplayListSection::Content);

        // Draw a rectangle representing the baselines.
        let mut baseline = LogicalRect::from_physical(self.style.writing_mode,
                                                      *stacking_relative_content_box,
                                                      container_size);
        baseline.start.b = baseline.start.b + text_fragment.run.ascent();
        baseline.size.block = Au(0);
        let baseline = baseline.to_physical(self.style.writing_mode, container_size);

        state.add_display_item(DisplayItem::LineClass(box LineDisplayItem {
            base: BaseDisplayItem::new(&baseline,
                                       DisplayItemMetadata::new(self.node,
                                                                style,
                                                                Cursor::DefaultCursor),
                                       clip),
            color: color::rgb(0, 200, 0),
            style: border_style::T::dashed,
        }), DisplayListSection::Content);
    }

    fn build_debug_borders_around_fragment(&self,
                                           state: &mut DisplayListBuildState,
                                           stacking_relative_border_box: &Rect<Au>,
                                           clip: &ClippingRegion) {
        // This prints a debug border around the border of this fragment.
        state.add_display_item(DisplayItem::BorderClass(box BorderDisplayItem {
            base: BaseDisplayItem::new(stacking_relative_border_box,
                                       DisplayItemMetadata::new(self.node,
                                                                &*self.style,
                                                                Cursor::DefaultCursor),
                                       clip),
            border_widths: SideOffsets2D::new_all_same(Au::from_px(1)),
            color: SideOffsets2D::new_all_same(color::rgb(0, 0, 200)),
            style: SideOffsets2D::new_all_same(border_style::T::solid),
            radius: Default::default(),
        }), DisplayListSection::Content);
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
            state.add_display_item(
                DisplayItem::SolidColorClass(box SolidColorDisplayItem {
                    base: BaseDisplayItem::new(stacking_relative_border_box,
                                               DisplayItemMetadata::new(self.node,
                                                                        &*self.style,
                                                                        Cursor::DefaultCursor),
                                               &clip),
                    color: background_color.to_gfx_color(),
            }), display_list_section);
        }

        // Draw a caret at the insertion point.
        let insertion_point_index = match scanned_text_fragment_info.insertion_point {
            Some(insertion_point_index) => insertion_point_index,
            None => return,
        };
        let range = Range::new(CharIndex(0), insertion_point_index);
        let advance = scanned_text_fragment_info.run.advance_for_range(&range);

        let insertion_point_bounds;
        let cursor;
        if !self.style.writing_mode.is_vertical() {
            insertion_point_bounds =
                Rect::new(Point2D::new(stacking_relative_border_box.origin.x + advance,
                                       stacking_relative_border_box.origin.y),
                          Size2D::new(INSERTION_POINT_LOGICAL_WIDTH,
                                      stacking_relative_border_box.size.height));
            cursor = Cursor::TextCursor;
        } else {
            insertion_point_bounds =
                Rect::new(Point2D::new(stacking_relative_border_box.origin.x,
                                       stacking_relative_border_box.origin.y + advance),
                          Size2D::new(stacking_relative_border_box.size.width,
                                      INSERTION_POINT_LOGICAL_WIDTH));
            cursor = Cursor::VerticalTextCursor;
        };

        state.add_display_item(DisplayItem::SolidColorClass(box SolidColorDisplayItem {
            base: BaseDisplayItem::new(&insertion_point_bounds,
                                       DisplayItemMetadata::new(self.node, &*self.style, cursor),
                                       &clip),
            color: self.style().get_color().color.to_gfx_color(),
        }), display_list_section);
    }

    fn build_display_list(&mut self,
                          state: &mut DisplayListBuildState,
                          stacking_relative_flow_origin: &Point2D<Au>,
                          relative_containing_block_size: &LogicalSize<Au>,
                          relative_containing_block_mode: WritingMode,
                          border_painting_mode: BorderPaintingMode,
                          display_list_section: DisplayListSection,
                          clip: &ClippingRegion,
                          stacking_relative_display_port: &Rect<Au>) {
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

        // webrender deals with all culling via aabb
        if !opts::get().use_webrender {
            if !stacking_relative_border_box.intersects(stacking_relative_display_port) {
                debug!("Fragment::build_display_list: outside display port");
                return
            }
        }

        // Calculate the clip rect. If there's nothing to render at all, don't even construct
        // display list items.
        let mut clip = (*clip).clone();
        self.adjust_clip_for_style(&mut clip, &stacking_relative_border_box);
        if !clip.might_intersect_rect(&stacking_relative_border_box) {
            return;
        }

        debug!("Fragment::build_display_list: intersected. Adding display item...");

        if self.is_primary_fragment() {
            // Add shadows, background, borders, and outlines, if applicable.
            if let Some(ref inline_context) = self.inline_context {
                for node in inline_context.nodes.iter().rev() {
                    self.build_display_list_for_background_if_applicable(
                        state,
                        &*node.style,
                        display_list_section,
                        &stacking_relative_border_box,
                        &clip);
                    self.build_display_list_for_box_shadow_if_applicable(
                        state,
                        &*node.style,
                        display_list_section,
                        &stacking_relative_border_box,
                        &clip);

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
                        &clip);

                    self.build_display_list_for_outline_if_applicable(
                        state,
                        &*node.style,
                        &stacking_relative_border_box,
                        &clip);
                }
            }

            if !self.is_scanned_text_fragment() {
                self.build_display_list_for_background_if_applicable(state,
                                                                     &*self.style,
                                                                     display_list_section,
                                                                     &stacking_relative_border_box,
                                                                     &clip);
                self.build_display_list_for_box_shadow_if_applicable(state,
                                                                     &*self.style,
                                                                     display_list_section,
                                                                     &stacking_relative_border_box,
                                                                     &clip);
                self.build_display_list_for_borders_if_applicable(state,
                                                                  &*self.style,
                                                                  border_painting_mode,
                                                                  &stacking_relative_border_box,
                                                                  display_list_section,
                                                                  &clip);
                self.build_display_list_for_outline_if_applicable(state,
                                                                  &*self.style,
                                                                  &stacking_relative_border_box,
                                                                  &clip);
            }

            // Paint the selection point if necessary.
            self.build_display_items_for_selection_if_necessary(state,
                                                                &stacking_relative_border_box,
                                                                display_list_section,
                                                                &clip);
        }

        // Create special per-fragment-type display items.
        self.build_fragment_type_specific_display_items(state,
                                                        &stacking_relative_border_box,
                                                        &clip);

        if opts::get().show_debug_fragment_borders {
           self.build_debug_borders_around_fragment(state,
                                                    &stacking_relative_border_box,
                                                    &clip);
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

                // TODO(emilio): Allow changing more properties by ::selection
                let text_color = if text_fragment.selected() {
                    self.selected_style().get_color().color
                } else {
                    self.style().get_color().color
                };

                for text_shadow in self.style.get_inheritedtext().text_shadow.0.iter().rev() {
                    let offset = &Point2D::new(text_shadow.offset_x, text_shadow.offset_y);
                    let color = self.style().resolve_color(text_shadow.color);
                    self.build_display_list_for_text_fragment(state,
                                                              &**text_fragment,
                                                              color,
                                                              &stacking_relative_content_box,
                                                              Some(text_shadow.blur_radius),
                                                              offset,
                                                              clip);
                }

                // Create the main text display item.
                self.build_display_list_for_text_fragment(state,
                                                          &**text_fragment,
                                                          text_color,
                                                          &stacking_relative_content_box,
                                                          None,
                                                          &Point2D::new(Au(0), Au(0)),
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
            SpecificFragmentInfo::InlineAbsolute(_) => {
                if opts::get().show_debug_fragment_borders {
                    self.build_debug_borders_around_fragment(state,
                                                             stacking_relative_border_box,
                                                             clip);
                }
            }
            SpecificFragmentInfo::Iframe(ref fragment_info) => {
                if !stacking_relative_content_box.is_empty() {
                    let item = DisplayItem::IframeClass(box IframeDisplayItem {
                        base: BaseDisplayItem::new(
                            &stacking_relative_content_box,
                            DisplayItemMetadata::new(self.node,
                                                     &*self.style,
                                                     Cursor::DefaultCursor),
                            clip),
                        iframe: fragment_info.pipeline_id,
                    });

                    if opts::get().use_webrender {
                        state.add_display_item(item, DisplayListSection::Content);
                    } else {
                        state.add_display_item(DisplayItem::LayeredItemClass(box LayeredItem {
                            item: item,
                            layer_info: LayerInfo::new(self.layer_id(),
                                                       ScrollPolicy::Scrollable,
                                                       Some(fragment_info.pipeline_id),
                                                       color::transparent()),
                        }), DisplayListSection::Content);
                    }
                }
            }
            SpecificFragmentInfo::Image(ref mut image_fragment) => {
                // Place the image into the display list.
                if let Some(ref image) = image_fragment.image {
                    state.add_display_item(DisplayItem::ImageClass(box ImageDisplayItem {
                        base: BaseDisplayItem::new(&stacking_relative_content_box,
                                                   DisplayItemMetadata::new(self.node,
                                                                            &*self.style,
                                                                            Cursor::DefaultCursor),
                                                   clip),
                        webrender_image: WebRenderImageInfo::from_image(image),
                        image_data: Some(Arc::new(image.bytes.clone())),
                        stretch_size: stacking_relative_content_box.size,
                        image_rendering: self.style.get_inheritedbox().image_rendering.clone(),
                    }), DisplayListSection::Content);
                }
            }
            SpecificFragmentInfo::Canvas(ref canvas_fragment_info) => {
                let width = canvas_fragment_info.replaced_image_fragment_info
                    .computed_inline_size.map_or(0, |w| w.to_px() as usize);
                let height = canvas_fragment_info.replaced_image_fragment_info
                    .computed_block_size.map_or(0, |h| h.to_px() as usize);
                if width > 0 && height > 0 {
                    let layer_id = self.layer_id();
                    let canvas_data = match canvas_fragment_info.ipc_renderer {
                        Some(ref ipc_renderer) => {
                            let ipc_renderer = ipc_renderer.lock().unwrap();
                            let (sender, receiver) = ipc::channel().unwrap();
                            ipc_renderer.send(CanvasMsg::FromLayout(
                                FromLayoutMsg::SendData(sender))).unwrap();
                            receiver.recv().unwrap()
                        },
                        None => CanvasData::Pixels(CanvasPixelData {
                            image_data: IpcSharedMemory::from_byte(0xFFu8, width * height * 4),
                            image_key: None,
                        }),
                    };

                    let display_item = match canvas_data {
                        CanvasData::Pixels(canvas_data) => {
                            DisplayItem::ImageClass(box ImageDisplayItem {
                                base: BaseDisplayItem::new(&stacking_relative_content_box,
                                                           DisplayItemMetadata::new(self.node,
                                                                                    &*self.style,
                                                                                    Cursor::DefaultCursor),
                                                           clip),
                                image_data: Some(Arc::new(canvas_data.image_data)),
                                webrender_image: WebRenderImageInfo {
                                    width: width as u32,
                                    height: height as u32,
                                    format: PixelFormat::RGBA8,
                                    key: canvas_data.image_key,
                                },
                                stretch_size: stacking_relative_content_box.size,
                                image_rendering: image_rendering::T::Auto,
                            })
                        }
                        CanvasData::WebGL(context_id) => {
                            DisplayItem::WebGLClass(box WebGLDisplayItem {
                                base: BaseDisplayItem::new(&stacking_relative_content_box,
                                                           DisplayItemMetadata::new(self.node,
                                                                                    &*self.style,
                                                                                    Cursor::DefaultCursor),
                                                           clip),
                                context_id: context_id,
                            })
                        }
                    };

                    if opts::get().use_webrender {
                        state.add_display_item(display_item, DisplayListSection::Content);
                    } else {
                        state.add_display_item(DisplayItem::LayeredItemClass(box LayeredItem {
                            item: display_item,
                            layer_info: LayerInfo::new(layer_id,
                                                       ScrollPolicy::Scrollable,
                                                       None,
                                                       color::transparent()),
                        }), DisplayListSection::Content);
                    }
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
                               mode: StackingContextCreationMode)
                               -> Box<StackingContext> {
        let border_box = match mode {
            StackingContextCreationMode::InnerScrollWrapper => {
                Rect::new(Point2D::zero(), base_flow.overflow.scroll.size)
            }
            _ => {
                self.stacking_relative_border_box(&base_flow.stacking_relative_position,
                                                  &base_flow.early_absolute_position_info
                                                            .relative_containing_block_size,
                                                  base_flow.early_absolute_position_info
                                                           .relative_containing_block_mode,
                                                  CoordinateSystem::Parent)
            }
        };
        let overflow = match mode {
            StackingContextCreationMode::InnerScrollWrapper |
            StackingContextCreationMode::OuterScrollWrapper => {
                Rect::new(Point2D::zero(), border_box.size)
            }
            _ => {
                // First, compute the offset of our border box (including relative positioning)
                // from our flow origin, since that is what `BaseFlow::overflow` is relative to.
                let border_box_offset =
                    border_box.translate(&-base_flow.stacking_relative_position).origin;
                // Then, using that, compute our overflow region relative to our border box.
                base_flow.overflow.paint.translate(&-border_box_offset)
            }
        };

        let mut transform = Matrix4::identity();
        if let Some(ref operations) = self.style().get_effects().transform.0 {
            let transform_origin = self.style().get_effects().transform_origin;
            let transform_origin =
                Point3D::new(model::specified(transform_origin.horizontal,
                                              border_box.size.width).to_f32_px(),
                             model::specified(transform_origin.vertical,
                                              border_box.size.height).to_f32_px(),
                             transform_origin.depth.to_f32_px());

            let pre_transform = Matrix4::create_translation(transform_origin.x,
                                                            transform_origin.y,
                                                            transform_origin.z);
            let post_transform = Matrix4::create_translation(-transform_origin.x,
                                                             -transform_origin.y,
                                                             -transform_origin.z);

            for operation in operations {
                let matrix = match *operation {
                    transform::ComputedOperation::Rotate(ax, ay, az, theta) => {
                        let theta = 2.0f32 * f32::consts::PI - theta.radians();
                        Matrix4::create_rotation(ax, ay, az, theta)
                    }
                    transform::ComputedOperation::Perspective(d) => {
                        create_perspective_matrix(d)
                    }
                    transform::ComputedOperation::Scale(sx, sy, sz) => {
                        Matrix4::create_scale(sx, sy, sz)
                    }
                    transform::ComputedOperation::Translate(tx, ty, tz) => {
                        let tx = model::specified(tx, border_box.size.width).to_f32_px();
                        let ty = model::specified(ty, border_box.size.height).to_f32_px();
                        let tz = tz.to_f32_px();
                        Matrix4::create_translation(tx, ty, tz)
                    }
                    transform::ComputedOperation::Matrix(m) => {
                        m.to_gfx_matrix()
                    }
                    transform::ComputedOperation::Skew(theta_x, theta_y) => {
                        Matrix4::create_skew(theta_x.radians(), theta_y.radians())
                    }
                };

                transform = transform.mul(&matrix);
            }

            transform = pre_transform.mul(&transform).mul(&post_transform);
        }

        let perspective = match self.style().get_effects().perspective {
            LengthOrNone::Length(d) => {
                let perspective_origin = self.style().get_effects().perspective_origin;
                let perspective_origin =
                    Point2D::new(model::specified(perspective_origin.horizontal,
                                                  border_box.size.width).to_f32_px(),
                                 model::specified(perspective_origin.vertical,
                                                  border_box.size.height).to_f32_px());

                let pre_transform = Matrix4::create_translation(perspective_origin.x,
                                                                perspective_origin.y,
                                                                0.0);
                let post_transform = Matrix4::create_translation(-perspective_origin.x,
                                                                 -perspective_origin.y,
                                                                 0.0);

                let perspective_matrix = create_perspective_matrix(d);

                pre_transform.mul(&perspective_matrix).mul(&post_transform)
            }
            LengthOrNone::None => {
                Matrix4::identity()
            }
        };

        // Create the filter pipeline.
        let effects = self.style().get_effects();
        let mut filters = effects.filter.clone();
        if effects.opacity != 1.0 {
            filters.push(Filter::Opacity(effects.opacity))
        }

        // There are two situations that need layers: when the fragment has the HAS_LAYER
        // flag and when we are building a layer tree for overflow scrolling.
        let layer_info = if mode == StackingContextCreationMode::InnerScrollWrapper {
            Some(LayerInfo::new(self.layer_id_for_overflow_scroll(),
                                scroll_policy,
                                None,
                                color::transparent()))
        } else if self.flags.contains(HAS_LAYER) {
            Some(LayerInfo::new(self.layer_id(), scroll_policy, None, color::transparent()))
        } else {
            None
        };

        let scrolls_overflow_area = mode == StackingContextCreationMode::OuterScrollWrapper;
        let transform_style = self.style().get_used_transform_style();
        let establishes_3d_context = scrolls_overflow_area ||
            transform_style == transform_style::T::flat;

        let context_type = match mode {
            StackingContextCreationMode::PseudoFloat => StackingContextType::PseudoFloat,
            StackingContextCreationMode::PseudoPositioned => StackingContextType::PseudoPositioned,
            _ => StackingContextType::Real,
        };

        Box::new(StackingContext::new(id,
                                      context_type,
                                      &border_box,
                                      &overflow,
                                      self.effective_z_index(),
                                      filters,
                                      self.style().get_effects().mix_blend_mode,
                                      transform,
                                      perspective,
                                      establishes_3d_context,
                                      scrolls_overflow_area,
                                      layer_info))
    }

    fn adjust_clipping_region_for_children(&self,
                                           current_clip: &mut ClippingRegion,
                                           stacking_relative_border_box: &Rect<Au>,
                                           _is_absolutely_positioned: bool) {
        // Don't clip if we're text.
        if self.is_scanned_text_fragment() {
            return
        }

        // Account for style-specified `clip`.
        self.adjust_clip_for_style(current_clip, stacking_relative_border_box);

        let overflow_x = self.style.get_box().overflow_x;
        let overflow_y = self.style.get_box().overflow_y.0;

        if let (overflow_x::T::visible, overflow_x::T::visible) = (overflow_x, overflow_y) {
            return
        }

        let tmp;
        let overflow_clip_rect = match self.style.get_box()._servo_overflow_clip_box {
            overflow_clip_box::T::padding_box => {
                // FIXME(SimonSapin): should be the padding box, not border box.
                stacking_relative_border_box
            }
            overflow_clip_box::T::content_box => {
                tmp = self.stacking_relative_content_box(stacking_relative_border_box);
                &tmp
            }
        };

        // Clip according to the values of `overflow-x` and `overflow-y`.
        //
        // FIXME(pcwalton): This may be more complex than it needs to be, since it seems to be
        // impossible with the computed value rules as they are to have `overflow-x: visible` with
        // `overflow-y: <scrolling>` or vice versa!
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
            current_clip.intersect_with_rounded_rect(stacking_relative_border_box, &border_radii)
        }
    }

    fn build_display_list_for_text_fragment(&self,
                                            state: &mut DisplayListBuildState,
                                            text_fragment: &ScannedTextFragmentInfo,
                                            text_color: RGBA,
                                            stacking_relative_content_box: &Rect<Au>,
                                            shadow_blur_radius: Option<Au>,
                                            offset: &Point2D<Au>,
                                            clip: &ClippingRegion) {
        // Determine the orientation and cursor to use.
        let (orientation, cursor) = if self.style.writing_mode.is_vertical() {
            if self.style.writing_mode.is_sideways_left() {
                (TextOrientation::SidewaysLeft, Cursor::VerticalTextCursor)
            } else {
                (TextOrientation::SidewaysRight, Cursor::VerticalTextCursor)
            }
        } else {
            (TextOrientation::Upright, Cursor::TextCursor)
        };

        // Compute location of the baseline.
        //
        // FIXME(pcwalton): Get the real container size.
        let container_size = Size2D::zero();
        let metrics = &text_fragment.run.font_metrics;
        let stacking_relative_content_box = stacking_relative_content_box.translate(offset);
        let baseline_origin = stacking_relative_content_box.origin +
            LogicalPoint::new(self.style.writing_mode,
                              Au(0),
                              metrics.ascent).to_physical(self.style.writing_mode,
                                                          container_size);

        // Create the text display item.
        state.add_display_item(DisplayItem::TextClass(box TextDisplayItem {
            base: BaseDisplayItem::new(&stacking_relative_content_box,
                                       DisplayItemMetadata::new(self.node, self.style(), cursor),
                                       clip),
            text_run: text_fragment.run.clone(),
            range: text_fragment.range,
            text_color: text_color.to_gfx_color(),
            orientation: orientation,
            baseline_origin: baseline_origin,
            blur_radius: shadow_blur_radius.unwrap_or(Au(0)),
        }), DisplayListSection::Content);

        // Create display items for text decorations.
        let mut text_decorations = self.style()
                                       .get_inheritedtext()
                                       ._servo_text_decorations_in_effect;
        if shadow_blur_radius.is_some() {
            // If we're painting a shadow, paint the decorations the same color as the shadow.
            text_decorations.underline = text_decorations.underline.map(|_| text_color);
            text_decorations.overline = text_decorations.overline.map(|_| text_color);
            text_decorations.line_through = text_decorations.line_through.map(|_| text_color);
        }

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
                                                        shadow_blur_radius.unwrap_or(Au(0)));
        }

        if let Some(ref overline_color) = text_decorations.overline {
            let mut stacking_relative_box = stacking_relative_content_box;
            stacking_relative_box.size.block = metrics.underline_size;
            self.build_display_list_for_text_decoration(state,
                                                        overline_color,
                                                        &stacking_relative_box,
                                                        clip,
                                                        shadow_blur_radius.unwrap_or(Au(0)));
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
                                                        shadow_blur_radius.unwrap_or(Au(0)));
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
        let metadata = DisplayItemMetadata::new(self.node, &*self.style, Cursor::DefaultCursor);
        state.add_display_item(DisplayItem::BoxShadowClass(box BoxShadowDisplayItem {
            base: BaseDisplayItem::new(&shadow_bounds(&stacking_relative_box, blur_radius, Au(0)),
                                       metadata,
                                       clip),
            box_bounds: stacking_relative_box,
            color: color.to_gfx_color(),
            offset: Point2D::zero(),
            blur_radius: blur_radius,
            spread_radius: Au(0),
            border_radius: Au(0),
            clip_mode: BoxShadowClipMode::None,
        }), DisplayListSection::Content);
    }
}

pub trait BlockFlowDisplayListBuilding {
    fn collect_stacking_contexts_for_block(&mut self,
                                           parent_id: StackingContextId,
                                           contexts: &mut Vec<Box<StackingContext>>)
                                           -> StackingContextId;
    fn build_display_list_for_block(&mut self,
                                    state: &mut DisplayListBuildState,
                                    border_painting_mode: BorderPaintingMode);
}

impl BlockFlowDisplayListBuilding for BlockFlow {
    fn collect_stacking_contexts_for_block(&mut self,
                                           parent_id: StackingContextId,
                                           contexts: &mut Vec<Box<StackingContext>>)
                                           -> StackingContextId {
        let block_stacking_context_type = self.block_stacking_context_type();
        if block_stacking_context_type == BlockStackingContextType::NonstackingContext {
            self.base.stacking_context_id = parent_id;
            self.base.collect_stacking_contexts_for_children(parent_id, contexts);
            return parent_id;
        }

        let stacking_context_id =
            StackingContextId::new_of_type(self.fragment.node.id() as usize,
                                           self.fragment.fragment_type());
        self.base.stacking_context_id = stacking_context_id;

        let inner_stacking_context_id = if self.has_scrolling_overflow() {
            StackingContextId::new_of_type(self.base.flow_id(), self.fragment.fragment_type())
        } else {
            stacking_context_id
        };

        let mut child_contexts = Vec::new();
        self.base.collect_stacking_contexts_for_children(inner_stacking_context_id,
                                                         &mut child_contexts);

        if block_stacking_context_type == BlockStackingContextType::PseudoStackingContext {
            let creation_mode = if self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) ||
                                   self.fragment.style.get_box().position != position::T::static_ {
                StackingContextCreationMode::PseudoPositioned
            } else {
                assert!(self.base.flags.is_float());
                StackingContextCreationMode::PseudoFloat
            };

            let stacking_context_index = contexts.len();
            contexts.push(self.fragment.create_stacking_context(stacking_context_id,
                                                                &self.base,
                                                                ScrollPolicy::Scrollable,
                                                                creation_mode));

            let mut floating = vec![];
            for child_context in child_contexts.into_iter() {
                if child_context.context_type == StackingContextType::PseudoFloat {
                    // Floating.
                    floating.push(child_context)
                } else {
                    // Positioned.
                    contexts.push(child_context)
                }
            }

            contexts[stacking_context_index].children = floating;
            return stacking_context_id;
        }

        let scroll_policy = if self.is_fixed() {
            ScrollPolicy::FixedPosition
        } else {
            ScrollPolicy::Scrollable
        };

        let stacking_context = if self.has_scrolling_overflow() {
            let mut inner_stacking_context = self.fragment.create_stacking_context(
                inner_stacking_context_id,
                &self.base,
                scroll_policy,
                StackingContextCreationMode::InnerScrollWrapper);
            inner_stacking_context.children = child_contexts;

            let mut outer_stacking_context = self.fragment.create_stacking_context(
                stacking_context_id,
                &self.base,
                scroll_policy,
                StackingContextCreationMode::OuterScrollWrapper);
            outer_stacking_context.children.push(inner_stacking_context);
            outer_stacking_context
        } else {
            let mut stacking_context = self.fragment.create_stacking_context(
                stacking_context_id,
                &self.base,
                scroll_policy,
                StackingContextCreationMode::Normal);
            stacking_context.children = child_contexts;
            stacking_context
        };

        contexts.push(stacking_context);
        stacking_context_id
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

        // Add the box that starts the block context.
        let translated_clip = if establishes_stacking_context {
            Some(self.base.clip.translate(&-self.base.stacking_relative_position))
        } else {
            None
        };
        let clip = match translated_clip {
            Some(ref translated_clip) => translated_clip,
            None => &self.base.clip,
        };

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
                                clip,
                                &self.base.stacking_relative_position_of_display_port);

        self.base.build_display_items_for_debugging_tint(state, self.fragment.node);
    }
}

pub trait InlineFlowDisplayListBuilding {
    fn collect_stacking_contexts_for_inline(&mut self,
                                            parent_id: StackingContextId,
                                            contexts: &mut Vec<Box<StackingContext>>)
                                            -> StackingContextId;
    fn build_display_list_for_inline_fragment_at_index(&mut self,
                                                       state: &mut DisplayListBuildState,
                                                       index: usize);
    fn build_display_list_for_inline(&mut self, state: &mut DisplayListBuildState);
}

impl InlineFlowDisplayListBuilding for InlineFlow {
    fn collect_stacking_contexts_for_inline(&mut self,
                                            parent_id: StackingContextId,
                                            contexts: &mut Vec<Box<StackingContext>>)
                                            -> StackingContextId {
        self.base.stacking_context_id = parent_id;

        for mut fragment in self.fragments.fragments.iter_mut() {
            match fragment.specific {
                SpecificFragmentInfo::InlineBlock(ref mut block_flow) => {
                    let block_flow = flow_ref::deref_mut(&mut block_flow.flow_ref);
                    block_flow.collect_stacking_contexts(parent_id, contexts);
                }
                SpecificFragmentInfo::InlineAbsoluteHypothetical(ref mut block_flow) => {
                    let block_flow = flow_ref::deref_mut(&mut block_flow.flow_ref);
                    block_flow.collect_stacking_contexts(parent_id, contexts);
                }
                _ if fragment.establishes_stacking_context() => {
                    fragment.stacking_context_id =
                        StackingContextId::new_of_type(fragment.fragment_id(),
                                                       fragment.fragment_type());
                    contexts.push(fragment.create_stacking_context(
                        fragment.stacking_context_id,
                        &self.base,
                        ScrollPolicy::Scrollable,
                        StackingContextCreationMode::Normal));
                }
                _ => fragment.stacking_context_id = parent_id,
            }
        }
        parent_id
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
                                    &self.base.clip,
                                    &self.base.stacking_relative_position_of_display_port);
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
                                      &self.block_flow.base.clip,
                                      &self.block_flow
                                           .base
                                           .stacking_relative_position_of_display_port);
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
        state.add_display_item(DisplayItem::BorderClass(box BorderDisplayItem {
            base: BaseDisplayItem::new(&stacking_context_relative_bounds.inflate(Au::from_px(2),
                                                                                Au::from_px(2)),
                                       DisplayItemMetadata {
                                           node: node,
                                           pointing: None,
                                       },
                                       &self.clip),
            border_widths: SideOffsets2D::new_all_same(Au::from_px(2)),
            color: SideOffsets2D::new_all_same(color),
            style: SideOffsets2D::new_all_same(border_style::T::solid),
            radius: BorderRadii::all_same(Au(0)),
        }), DisplayListSection::Content);
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
    fn to_gfx_color(&self) -> Color;
}

impl ToGfxColor for RGBA {
    fn to_gfx_color(&self) -> Color {
        color::rgba(self.red, self.green, self.blue, self.alpha)
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
    OuterScrollWrapper,
    InnerScrollWrapper,
    PseudoPositioned,
    PseudoFloat,
}
