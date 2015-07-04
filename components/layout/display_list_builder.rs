/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Builds display lists from flows and fragments.
//!
//! Other browser engines sometimes call this "painting", but it is more accurately called display
//! list building, as the actual painting does not happen here—only deciding *what* we're going to
//! paint.

#![deny(unsafe_code)]

use azure::azure_hl::Color;
use block::BlockFlow;
use context::LayoutContext;
use flow::{self, BaseFlow, Flow, IS_ABSOLUTELY_POSITIONED, NEEDS_LAYER};
use fragment::{CoordinateSystem, Fragment, IframeFragmentInfo, ImageFragmentInfo};
use fragment::{ScannedTextFragmentInfo, SpecificFragmentInfo};
use inline::InlineFlow;
use list_item::ListItemFlow;
use model::{self, MaybeAuto, ToGfxMatrix, ToAu};
use table_cell::CollapsedBordersForCell;

use euclid::{Point2D, Point3D, Rect, Size2D, SideOffsets2D};
use euclid::Matrix4;
use gfx_traits::color;
use gfx::display_list::{BLUR_INFLATION_FACTOR, BaseDisplayItem, BorderDisplayItem};
use gfx::display_list::{BorderRadii, BoxShadowClipMode, BoxShadowDisplayItem, ClippingRegion};
use gfx::display_list::{DisplayItem, DisplayList, DisplayItemMetadata};
use gfx::display_list::{GradientDisplayItem};
use gfx::display_list::{GradientStop, ImageDisplayItem, LineDisplayItem};
use gfx::display_list::{OpaqueNode, SolidColorDisplayItem};
use gfx::display_list::{StackingContext, TextDisplayItem, TextOrientation};
use gfx::paint_task::{PaintLayer, THREAD_TINT_COLORS};
use msg::compositor_msg::{ScrollPolicy, LayerId};
use msg::constellation_msg::ConstellationChan;
use msg::constellation_msg::Msg as ConstellationMsg;
use net_traits::image_cache_task::UsePlaceholder;
use png::{self, PixelsByColorType};
use std::cmp;
use std::default::Default;
use std::iter::repeat;
use std::sync::Arc;
use std::f32;
use style::computed_values::filter::Filter;
use style::computed_values::{background_attachment, background_clip, background_origin,
                             background_repeat, background_size};
use style::computed_values::{border_style, image_rendering, overflow_x, position,
                             visibility, transform, transform_style};
use style::properties::ComputedValues;
use style::properties::style_structs::Border;
use style::values::RGBA;
use style::values::computed::{Image, LinearGradient};
use style::values::computed::{LengthOrNone, LengthOrPercentage, LengthOrPercentageOrAuto};
use style::values::specified::{AngleOrCorner, HorizontalDirection, VerticalDirection};
use url::Url;
use util::cursor::Cursor;
use util::geometry::{Au, ZERO_POINT};
use util::logical_geometry::{LogicalPoint, LogicalRect, LogicalSize, WritingMode};
use util::opts;

use canvas_traits::{CanvasMsg, CanvasCommonMsg};
use std::sync::mpsc::channel;

/// A possible `PaintLayer` for an stacking context
pub enum StackingContextLayer {
    Existing(PaintLayer),
    IfCanvas(LayerId),
}

/// The results of display list building for a single flow.
pub enum DisplayListBuildingResult {
    None,
    StackingContext(Arc<StackingContext>),
    Normal(Box<DisplayList>),
}

impl DisplayListBuildingResult {
    /// Adds the display list items contained within this display list building result to the given
    /// display list, preserving stacking order. If this display list building result does not
    /// consist of an entire stacking context, it will be emptied.
    pub fn add_to(&mut self, display_list: &mut DisplayList) {
        match *self {
            DisplayListBuildingResult::None => return,
            DisplayListBuildingResult::StackingContext(ref mut stacking_context) => {
                display_list.children.push_back((*stacking_context).clone())
            }
            DisplayListBuildingResult::Normal(ref mut source_display_list) => {
                display_list.append_from(&mut **source_display_list)
            }
        }
    }
}

pub trait FragmentDisplayListBuilding {
    /// Adds the display items necessary to paint the background of this fragment to the display
    /// list if necessary.
    fn build_display_list_for_background_if_applicable(&self,
                                                       style: &ComputedValues,
                                                       display_list: &mut DisplayList,
                                                       layout_context: &LayoutContext,
                                                       level: StackingLevel,
                                                       absolute_bounds: &Rect<Au>,
                                                       clip: &ClippingRegion);

    /// Computes the background size for an image with the given background area according to the
    /// rules in CSS-BACKGROUNDS § 3.9.
    fn compute_background_image_size(&self,
                                     style: &ComputedValues,
                                     bounds: &Rect<Au>,
                                     image: &png::Image)
                                     -> Size2D<Au>;

    /// Adds the display items necessary to paint the background image of this fragment to the
    /// display list at the appropriate stacking level.
    fn build_display_list_for_background_image(&self,
                                               style: &ComputedValues,
                                               display_list: &mut DisplayList,
                                               layout_context: &LayoutContext,
                                               level: StackingLevel,
                                               absolute_bounds: &Rect<Au>,
                                               clip: &ClippingRegion,
                                               image_url: &Url);

    /// Adds the display items necessary to paint the background linear gradient of this fragment
    /// to the display list at the appropriate stacking level.
    fn build_display_list_for_background_linear_gradient(&self,
                                                         display_list: &mut DisplayList,
                                                         level: StackingLevel,
                                                         absolute_bounds: &Rect<Au>,
                                                         clip: &ClippingRegion,
                                                         gradient: &LinearGradient,
                                                         style: &ComputedValues);

    /// Adds the display items necessary to paint the borders of this fragment to a display list if
    /// necessary.
    fn build_display_list_for_borders_if_applicable(
            &self,
            style: &ComputedValues,
            border_painting_mode: BorderPaintingMode,
            display_list: &mut DisplayList,
            bounds: &Rect<Au>,
            level: StackingLevel,
            clip: &ClippingRegion);

    /// Adds the display items necessary to paint the outline of this fragment to the display list
    /// if necessary.
    fn build_display_list_for_outline_if_applicable(&self,
                                                    style: &ComputedValues,
                                                    display_list: &mut DisplayList,
                                                    bounds: &Rect<Au>,
                                                    clip: &ClippingRegion);

    /// Adds the display items necessary to paint the box shadow of this fragment to the display
    /// list if necessary.
    fn build_display_list_for_box_shadow_if_applicable(&self,
                                                       style: &ComputedValues,
                                                       list: &mut DisplayList,
                                                       layout_context: &LayoutContext,
                                                       level: StackingLevel,
                                                       absolute_bounds: &Rect<Au>,
                                                       clip: &ClippingRegion);

    /// Adds display items necessary to draw debug boxes around a scanned text fragment.
    fn build_debug_borders_around_text_fragments(&self,
                                                 style: &ComputedValues,
                                                 display_list: &mut DisplayList,
                                                 stacking_relative_border_box: &Rect<Au>,
                                                 stacking_relative_content_box: &Rect<Au>,
                                                 text_fragment: &ScannedTextFragmentInfo,
                                                 clip: &ClippingRegion);

    /// Adds display items necessary to draw debug boxes around this fragment.
    fn build_debug_borders_around_fragment(&self,
                                           display_list: &mut DisplayList,
                                           stacking_relative_border_box: &Rect<Au>,
                                           clip: &ClippingRegion);

    /// Adds the display items for this fragment to the given display list.
    ///
    /// Arguments:
    ///
    /// * `display_list`: The display list to add display items to.
    /// * `layout_context`: The layout context.
    /// * `dirty`: The dirty rectangle in the coordinate system of the owning flow.
    /// * `stacking_relative_flow_origin`: Position of the origin of the owning flow with respect
    ///   to its nearest ancestor stacking context.
    /// * `relative_containing_block_size`: The size of the containing block that
    ///   `position: relative` makes use of.
    /// * `clip`: The region to clip the display items to.
    /// * `stacking_relative_display_port`: The position and size of the display port with respect
    ///   to the nearest ancestor stacking context.
    fn build_display_list(&mut self,
                          display_list: &mut DisplayList,
                          layout_context: &LayoutContext,
                          stacking_relative_flow_origin: &Point2D<Au>,
                          relative_containing_block_size: &LogicalSize<Au>,
                          relative_containing_block_mode: WritingMode,
                          border_painting_mode: BorderPaintingMode,
                          background_and_border_level: BackgroundAndBorderLevel,
                          clip: &ClippingRegion,
                          stacking_relative_display_port: &Rect<Au>);

    /// Sends the size and position of this iframe fragment to the constellation. This is out of
    /// line to guide inlining.
    fn finalize_position_and_size_of_iframe(&self,
                                            iframe_fragment: &IframeFragmentInfo,
                                            offset: Point2D<Au>,
                                            layout_context: &LayoutContext);

    /// Returns the appropriate clipping region for descendants of this flow.
    fn clipping_region_for_children(&self,
                                    current_clip: &ClippingRegion,
                                    stacking_relative_border_box: &Rect<Au>)
                                    -> ClippingRegion;

    /// Calculates the clipping rectangle for a fragment, taking the `clip` property into account
    /// per CSS 2.1 § 11.1.2.
    fn calculate_style_specified_clip(&self,
                                      parent_clip: &ClippingRegion,
                                      stacking_relative_border_box: &Rect<Au>)
                                      -> ClippingRegion;

    /// Creates the text display item for one text fragment. This can be called multiple times for
    /// one fragment if there are text shadows.
    ///
    /// `shadow_blur_radius` will be `Some` if this is a shadow, even if the blur radius is zero.
    fn build_display_list_for_text_fragment(&self,
                                            display_list: &mut DisplayList,
                                            text_fragment: &ScannedTextFragmentInfo,
                                            text_color: RGBA,
                                            stacking_relative_content_box: &Rect<Au>,
                                            shadow_blur_radius: Option<Au>,
                                            offset: &Point2D<Au>,
                                            clip: &ClippingRegion);

    /// Creates the display item for a text decoration: underline, overline, or line-through.
    fn build_display_list_for_text_decoration(&self,
                                              display_list: &mut DisplayList,
                                              color: &RGBA,
                                              stacking_relative_box: &LogicalRect<Au>,
                                              clip: &ClippingRegion,
                                              blur_radius: Au);

    /// A helper method that `build_display_list` calls to create per-fragment-type display items.
    fn build_fragment_type_specific_display_items(&mut self,
                                                  display_list: &mut DisplayList,
                                                  stacking_relative_border_box: &Rect<Au>,
                                                  clip: &ClippingRegion);

    /// Creates a stacking context for associated fragment.
    fn create_stacking_context(&self,
                               base_flow: &BaseFlow,
                               display_list: Box<DisplayList>,
                               layout_context: &LayoutContext,
                               layer: StackingContextLayer)
                               -> Arc<StackingContext>;

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

    let top_factor = scale_factor(radii.top_left, radii.top_right, size.width);
    let bottom_factor = scale_factor(radii.bottom_left, radii.bottom_right, size.width);
    let left_factor = scale_factor(radii.top_left, radii.bottom_left, size.height);
    let right_factor = scale_factor(radii.top_right, radii.bottom_right, size.height);
    let min_factor = top_factor.min(bottom_factor).min(left_factor).min(right_factor);
    if min_factor < 1.0 {
        BorderRadii {
            top_left:     radii.top_left    .scale_by(min_factor),
            top_right:    radii.top_right   .scale_by(min_factor),
            bottom_left:  radii.bottom_left .scale_by(min_factor),
            bottom_right: radii.bottom_right.scale_by(min_factor),
        }
    } else {
        *radii
    }
}

fn build_border_radius(abs_bounds: &Rect<Au>, border_style: &Border) -> BorderRadii<Au> {
    // TODO(cgaebel): Support border radii even in the case of multiple border widths.
    // This is an extension of supporting elliptical radii. For now, all percentage
    // radii will be relative to the width.

    handle_overlapping_radii(&abs_bounds.size, &BorderRadii {
        top_left:     model::specified(border_style.border_top_left_radius,
                                       abs_bounds.size.width),
        top_right:    model::specified(border_style.border_top_right_radius,
                                       abs_bounds.size.width),
        bottom_right: model::specified(border_style.border_bottom_right_radius,
                                       abs_bounds.size.width),
        bottom_left:  model::specified(border_style.border_bottom_left_radius,
                                       abs_bounds.size.width),
    })
}

impl FragmentDisplayListBuilding for Fragment {
    fn build_display_list_for_background_if_applicable(&self,
                                                       style: &ComputedValues,
                                                       display_list: &mut DisplayList,
                                                       layout_context: &LayoutContext,
                                                       level: StackingLevel,
                                                       absolute_bounds: &Rect<Au>,
                                                       clip: &ClippingRegion) {
        // Adjust the clipping region as necessary to account for `border-radius`.
        let border_radii = build_border_radius(absolute_bounds, style.get_border());
        let mut clip = (*clip).clone();
        if !border_radii.is_square() {
            clip = clip.intersect_with_rounded_rect(absolute_bounds, &border_radii)
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

        display_list.push(DisplayItem::SolidColorClass(box SolidColorDisplayItem {
            base: BaseDisplayItem::new(bounds,
                                       DisplayItemMetadata::new(self.node,
                                                                style,
                                                                Cursor::DefaultCursor),
                                       clip.clone()),
            color: background_color.to_gfx_color(),
        }), level);

        // The background image is painted on top of the background color.
        // Implements background image, per spec:
        // http://www.w3.org/TR/CSS21/colors.html#background
        let background = style.get_background();
        match background.background_image {
            None => {}
            Some(Image::LinearGradient(ref gradient)) => {
                self.build_display_list_for_background_linear_gradient(display_list,
                                                                       level,
                                                                       absolute_bounds,
                                                                       &clip,
                                                                       gradient,
                                                                       style)
            }
            Some(Image::Url(ref image_url)) => {
                self.build_display_list_for_background_image(style,
                                                             display_list,
                                                             layout_context,
                                                             level,
                                                             absolute_bounds,
                                                             &clip,
                                                             image_url)
            }
        }
    }

    fn compute_background_image_size(&self,
                                     style: &ComputedValues,
                                     bounds: &Rect<Au>,
                                     image: &png::Image)
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
                                               style: &ComputedValues,
                                               display_list: &mut DisplayList,
                                               layout_context: &LayoutContext,
                                               level: StackingLevel,
                                               absolute_bounds: &Rect<Au>,
                                               clip: &ClippingRegion,
                                               image_url: &Url) {
        let background = style.get_background();
        let image = layout_context.get_or_request_image(image_url.clone(), UsePlaceholder::No);
        if let Some(image) = image {
            debug!("(building display list) building background image");

            // Use `background-size` to get the size.
            let mut bounds = *absolute_bounds;
            let image_size = self.compute_background_image_size(style, &bounds, &*image);

            // Clip.
            //
            // TODO: Check the bounds to see if a clip item is actually required.
            let clip = clip.clone().intersect_rect(&bounds);

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
                    let border_padding = (self.border_padding).to_physical(self.style.writing_mode);
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
            display_list.push(DisplayItem::ImageClass(box ImageDisplayItem {
                base: BaseDisplayItem::new(bounds,
                                           DisplayItemMetadata::new(self.node,
                                                                    style,
                                                                    Cursor::DefaultCursor),
                                           clip),
                image: image.clone(),
                stretch_size: Size2D::new(image_size.width, image_size.height),
                image_rendering: style.get_effects().image_rendering.clone(),
            }), level);
        }
    }

    fn build_display_list_for_background_linear_gradient(&self,
                                                         display_list: &mut DisplayList,
                                                         level: StackingLevel,
                                                         absolute_bounds: &Rect<Au>,
                                                         clip: &ClippingRegion,
                                                         gradient: &LinearGradient,
                                                         style: &ComputedValues) {
        let clip = clip.clone().intersect_rect(absolute_bounds);

        // This is the distance between the center and the ending point; i.e. half of the distance
        // between the starting point and the ending point.
        let delta = match gradient.angle_or_corner {
            AngleOrCorner::Angle(angle) => {
                Point2D::new(Au::from_f32_px(angle.radians().sin() *
                                             absolute_bounds.size.width.to_f32_px() / 2.0),
                             Au::from_f32_px(-angle.radians().cos() *
                                             absolute_bounds.size.height.to_f32_px() / 2.0))
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
            base: BaseDisplayItem::new(*absolute_bounds,
                                       DisplayItemMetadata::new(self.node,
                                                                style,
                                                                Cursor::DefaultCursor),
                                       clip),
            start_point: center - delta,
            end_point: center + delta,
            stops: stops,
        });

        display_list.push(gradient_display_item, level)
    }

    fn build_display_list_for_box_shadow_if_applicable(&self,
                                                       style: &ComputedValues,
                                                       list: &mut DisplayList,
                                                       _layout_context: &LayoutContext,
                                                       level: StackingLevel,
                                                       absolute_bounds: &Rect<Au>,
                                                       clip: &ClippingRegion) {
        // NB: According to CSS-BACKGROUNDS, box shadows render in *reverse* order (front to back).
        for box_shadow in style.get_effects().box_shadow.iter().rev() {
            let bounds = shadow_bounds(&absolute_bounds.translate(&Point2D::new(box_shadow.offset_x,
                                                                                box_shadow.offset_y)),
                                       box_shadow.blur_radius,
                                       box_shadow.spread_radius);
            list.push(DisplayItem::BoxShadowClass(box BoxShadowDisplayItem {
                base: BaseDisplayItem::new(bounds,
                                           DisplayItemMetadata::new(self.node,
                                                                    style,
                                                                    Cursor::DefaultCursor),
                                           (*clip).clone()),
                box_bounds: *absolute_bounds,
                color: style.resolve_color(box_shadow.color).to_gfx_color(),
                offset: Point2D::new(box_shadow.offset_x, box_shadow.offset_y),
                blur_radius: box_shadow.blur_radius,
                spread_radius: box_shadow.spread_radius,
                clip_mode: if box_shadow.inset {
                    BoxShadowClipMode::Inset
                } else {
                    BoxShadowClipMode::Outset
                },
            }), level);
        }
    }

    fn build_display_list_for_borders_if_applicable(
            &self,
            style: &ComputedValues,
            border_painting_mode: BorderPaintingMode,
            display_list: &mut DisplayList,
            bounds: &Rect<Au>,
            level: StackingLevel,
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
        display_list.push(DisplayItem::BorderClass(box BorderDisplayItem {
            base: BaseDisplayItem::new(bounds,
                                       DisplayItemMetadata::new(self.node,
                                                                style,
                                                                Cursor::DefaultCursor),
                                       (*clip).clone()),
            border_widths: border.to_physical(style.writing_mode),
            color: SideOffsets2D::new(colors.top.to_gfx_color(),
                                      colors.right.to_gfx_color(),
                                      colors.bottom.to_gfx_color(),
                                      colors.left.to_gfx_color()),
            style: border_style,
            radius: build_border_radius(&bounds, border_style_struct),
        }), level);
    }

    fn build_display_list_for_outline_if_applicable(&self,
                                                    style: &ComputedValues,
                                                    display_list: &mut DisplayList,
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
        display_list.outlines.push_back(DisplayItem::BorderClass(box BorderDisplayItem {
            base: BaseDisplayItem::new(bounds,
                                       DisplayItemMetadata::new(self.node,
                                                                style,
                                                                Cursor::DefaultCursor),
                                       (*clip).clone()),
            border_widths: SideOffsets2D::new_all_same(width),
            color: SideOffsets2D::new_all_same(color),
            style: SideOffsets2D::new_all_same(outline_style),
            radius: Default::default(),
        }))
    }

    fn build_debug_borders_around_text_fragments(&self,
                                                 style: &ComputedValues,
                                                 display_list: &mut DisplayList,
                                                 stacking_relative_border_box: &Rect<Au>,
                                                 stacking_relative_content_box: &Rect<Au>,
                                                 text_fragment: &ScannedTextFragmentInfo,
                                                 clip: &ClippingRegion) {
        // FIXME(pcwalton, #2795): Get the real container size.
        let container_size = Size2D::zero();

        // Compute the text fragment bounds and draw a border surrounding them.
        display_list.content.push_back(DisplayItem::BorderClass(box BorderDisplayItem {
            base: BaseDisplayItem::new(*stacking_relative_border_box,
                                       DisplayItemMetadata::new(self.node,
                                                                style,
                                                                Cursor::DefaultCursor),
                                       (*clip).clone()),
            border_widths: SideOffsets2D::new_all_same(Au::from_px(1)),
            color: SideOffsets2D::new_all_same(color::rgb(0, 0, 200)),
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

        let line_display_item = box LineDisplayItem {
            base: BaseDisplayItem::new(baseline,
                                       DisplayItemMetadata::new(self.node, style, Cursor::DefaultCursor),
                                       (*clip).clone()),
            color: color::rgb(0, 200, 0),
            style: border_style::T::dashed,
        };
        display_list.content.push_back(DisplayItem::LineClass(line_display_item));
    }

    fn build_debug_borders_around_fragment(&self,
                                           display_list: &mut DisplayList,
                                           stacking_relative_border_box: &Rect<Au>,
                                           clip: &ClippingRegion) {
        // This prints a debug border around the border of this fragment.
        display_list.content.push_back(DisplayItem::BorderClass(box BorderDisplayItem {
            base: BaseDisplayItem::new(*stacking_relative_border_box,
                                       DisplayItemMetadata::new(self.node,
                                                                &*self.style,
                                                                Cursor::DefaultCursor),
                                       (*clip).clone()),
            border_widths: SideOffsets2D::new_all_same(Au::from_px(1)),
            color: SideOffsets2D::new_all_same(color::rgb(0, 0, 200)),
            style: SideOffsets2D::new_all_same(border_style::T::solid),
            radius: Default::default(),
        }));
    }

    fn calculate_style_specified_clip(&self,
                                      parent_clip: &ClippingRegion,
                                      stacking_relative_border_box: &Rect<Au>)
                                      -> ClippingRegion {
        // Account for `clip` per CSS 2.1 § 11.1.2.
        let style_clip_rect = match (self.style().get_box().position,
                                     self.style().get_effects().clip) {
            (position::T::absolute, Some(style_clip_rect)) => style_clip_rect,
            _ => return (*parent_clip).clone(),
        };

        // FIXME(pcwalton, #2795): Get the real container size.
        let clip_origin = Point2D::new(stacking_relative_border_box.origin.x + style_clip_rect.left,
                                       stacking_relative_border_box.origin.y + style_clip_rect.top);
        let right = style_clip_rect.right.unwrap_or(stacking_relative_border_box.size.width);
        let bottom = style_clip_rect.bottom.unwrap_or(stacking_relative_border_box.size.height);
        let clip_size = Size2D::new(right - clip_origin.x, bottom - clip_origin.y);
        (*parent_clip).clone().intersect_rect(&Rect::new(clip_origin, clip_size))
    }

    fn build_display_list(&mut self,
                          display_list: &mut DisplayList,
                          layout_context: &LayoutContext,
                          stacking_relative_flow_origin: &Point2D<Au>,
                          relative_containing_block_size: &LogicalSize<Au>,
                          relative_containing_block_mode: WritingMode,
                          border_painting_mode: BorderPaintingMode,
                          background_and_border_level: BackgroundAndBorderLevel,
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

        debug!("Fragment::build_display_list at rel={:?}, abs={:?}, dirty={:?}, flow origin={:?}: \
                {:?}",
               self.border_box,
               stacking_relative_border_box,
               layout_context.shared.dirty,
               stacking_relative_flow_origin,
               self);

        if !stacking_relative_border_box.intersects(stacking_relative_display_port) {
            debug!("Fragment::build_display_list: outside display port");
            return
        }

        if !stacking_relative_border_box.intersects(&layout_context.shared.dirty) {
            debug!("Fragment::build_display_list: Did not intersect...");
            return
        }

        // Calculate the clip rect. If there's nothing to render at all, don't even construct
        // display list items.
        let clip = self.calculate_style_specified_clip(clip, &stacking_relative_border_box);
        if !clip.might_intersect_rect(&stacking_relative_border_box) {
            return;
        }

        debug!("Fragment::build_display_list: intersected. Adding display item...");

        if self.is_primary_fragment() {
            let level =
                StackingLevel::from_background_and_border_level(background_and_border_level);

            // Add shadows, background, borders, and outlines, if applicable.
            if let Some(ref inline_context) = self.inline_context {
                for node in inline_context.nodes.iter().rev() {
                    self.build_display_list_for_box_shadow_if_applicable(
                        &*node.style,
                        display_list,
                        layout_context,
                        level,
                        &stacking_relative_border_box,
                        &clip);
                    self.build_display_list_for_background_if_applicable(
                        &*node.style,
                        display_list,
                        layout_context,
                        level,
                        &stacking_relative_border_box,
                        &clip);
                    self.build_display_list_for_borders_if_applicable(
                        &*node.style,
                        border_painting_mode,
                        display_list,
                        &stacking_relative_border_box,
                        level,
                        &clip);
                    self.build_display_list_for_outline_if_applicable(
                        &*node.style,
                        display_list,
                        &stacking_relative_border_box,
                        &clip);
                }
            }
            if !self.is_scanned_text_fragment() {
                self.build_display_list_for_box_shadow_if_applicable(&*self.style,
                                                                     display_list,
                                                                     layout_context,
                                                                     level,
                                                                     &stacking_relative_border_box,
                                                                     &clip);
                self.build_display_list_for_background_if_applicable(&*self.style,
                                                                     display_list,
                                                                     layout_context,
                                                                     level,
                                                                     &stacking_relative_border_box,
                                                                     &clip);
                self.build_display_list_for_borders_if_applicable(&*self.style,
                                                                  border_painting_mode,
                                                                  display_list,
                                                                  &stacking_relative_border_box,
                                                                  level,
                                                                  &clip);
                self.build_display_list_for_outline_if_applicable(&*self.style,
                                                                  display_list,
                                                                  &stacking_relative_border_box,
                                                                  &clip);
            }
        }

        // Create special per-fragment-type display items.
        self.build_fragment_type_specific_display_items(display_list,
                                                        &stacking_relative_border_box,
                                                        &clip);

        if opts::get().show_debug_fragment_borders {
           self.build_debug_borders_around_fragment(display_list,
                                                    &stacking_relative_border_box,
                                                    &clip)
        }

        // If this is an iframe, then send its position and size up to the constellation.
        //
        // FIXME(pcwalton): Doing this during display list construction seems potentially
        // problematic if iframes are outside the area we're computing the display list for, since
        // they won't be able to reflow at all until the user scrolls to them. Perhaps we should
        // separate this into two parts: first we should send the size only to the constellation
        // once that's computed during assign-block-sizes, and second we should should send the
        // origin to the constellation here during display list construction. This should work
        // because layout for the iframe only needs to know size, and origin is only relevant if
        // the iframe is actually going to be displayed.
        if let SpecificFragmentInfo::Iframe(ref iframe_fragment) = self.specific {
            self.finalize_position_and_size_of_iframe(&**iframe_fragment,
                                                      stacking_relative_border_box.origin,
                                                      layout_context)
        }
    }

    fn build_fragment_type_specific_display_items(&mut self,
                                                  display_list: &mut DisplayList,
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
                let text_color = self.style().get_color().color;
                for text_shadow in self.style.get_effects().text_shadow.0.iter().rev() {
                    let offset = &Point2D::new(text_shadow.offset_x, text_shadow.offset_y);
                    let color = self.style().resolve_color(text_shadow.color);
                    self.build_display_list_for_text_fragment(display_list,
                                                              &**text_fragment,
                                                              color,
                                                              &stacking_relative_content_box,
                                                              Some(text_shadow.blur_radius),
                                                              offset,
                                                              clip);
                }

                // Create the main text display item.
                self.build_display_list_for_text_fragment(display_list,
                                                          &**text_fragment,
                                                          text_color,
                                                          &stacking_relative_content_box,
                                                          None,
                                                          &Point2D::new(Au(0), Au(0)),
                                                          clip);

                if opts::get().show_debug_fragment_borders {
                    self.build_debug_borders_around_text_fragments(self.style(),
                                                                   display_list,
                                                                   stacking_relative_border_box,
                                                                   &stacking_relative_content_box,
                                                                   &**text_fragment,
                                                                   clip)
                }
            }
            SpecificFragmentInfo::Generic |
            SpecificFragmentInfo::GeneratedContent(..) |
            SpecificFragmentInfo::Iframe(..) |
            SpecificFragmentInfo::Table |
            SpecificFragmentInfo::TableCell |
            SpecificFragmentInfo::TableRow |
            SpecificFragmentInfo::TableWrapper |
            SpecificFragmentInfo::InlineBlock(_) |
            SpecificFragmentInfo::InlineAbsoluteHypothetical(_) |
            SpecificFragmentInfo::InlineAbsolute(_) => {
                if opts::get().show_debug_fragment_borders {
                    self.build_debug_borders_around_fragment(display_list,
                                                             stacking_relative_border_box,
                                                             clip);
                }
            }
            SpecificFragmentInfo::Image(ref mut image_fragment) => {
                // Place the image into the display list.
                if let Some(ref image) = image_fragment.image {
                    display_list.content.push_back(DisplayItem::ImageClass(box ImageDisplayItem {
                        base: BaseDisplayItem::new(stacking_relative_content_box,
                                                   DisplayItemMetadata::new(self.node,
                                                                            &*self.style,
                                                                            Cursor::DefaultCursor),
                                                   (*clip).clone()),
                        image: image.clone(),
                        stretch_size: stacking_relative_content_box.size,
                        image_rendering: self.style.get_effects().image_rendering.clone(),
                    }));
                }
            }
            SpecificFragmentInfo::Canvas(ref canvas_fragment_info) => {
                // TODO(ecoal95): make the canvas with a renderer use the custom layer
                let width = canvas_fragment_info.replaced_image_fragment_info
                    .computed_inline_size.map_or(0, |w| w.to_px() as usize);
                let height = canvas_fragment_info.replaced_image_fragment_info
                    .computed_block_size.map_or(0, |h| h.to_px() as usize);
                let (sender, receiver) = channel::<Vec<u8>>();
                let canvas_data = match canvas_fragment_info.renderer {
                    Some(ref renderer) =>  {
                        renderer.lock().unwrap().send(CanvasMsg::Common(
                                CanvasCommonMsg::SendPixelContents(sender))).unwrap();
                        receiver.recv().unwrap()
                    },
                    None => repeat(0xFFu8).take(width * height * 4).collect(),
                };
                display_list.content.push_back(DisplayItem::ImageClass(box ImageDisplayItem{
                    base: BaseDisplayItem::new(stacking_relative_content_box,
                                               DisplayItemMetadata::new(self.node,
                                                                        &*self.style,
                                                                        Cursor::DefaultCursor),
                                               (*clip).clone()),
                    image: Arc::new(png::Image {
                        width: width as u32,
                        height: height as u32,
                        pixels: PixelsByColorType::RGBA8(canvas_data),
                    }),
                    stretch_size: stacking_relative_content_box.size,
                    image_rendering: image_rendering::T::Auto,
                }));
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
                               base_flow: &BaseFlow,
                               display_list: Box<DisplayList>,
                               layout_context: &LayoutContext,
                               layer: StackingContextLayer)
                               -> Arc<StackingContext> {
        let border_box = self.stacking_relative_border_box(&base_flow.stacking_relative_position,
                                                               &base_flow.absolute_position_info
                                                               .relative_containing_block_size,
                                                               base_flow.absolute_position_info
                                                               .relative_containing_block_mode,
                                                               CoordinateSystem::Parent);

        let mut transform = Matrix4::identity();

        if let Some(ref operations) = self.style().get_effects().transform {
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
                let matrix = match operation {
                    &transform::ComputedOperation::Rotate(ax, ay, az, theta) => {
                        let theta = 2.0f32 * f32::consts::PI - theta.radians();
                        Matrix4::create_rotation(ax, ay, az, theta)
                    }
                    &transform::ComputedOperation::Perspective(d) => {
                        Matrix4::create_perspective(d.to_f32_px())
                    }
                    &transform::ComputedOperation::Scale(sx, sy, sz) => {
                        Matrix4::create_scale(sx, sy, sz)
                    }
                    &transform::ComputedOperation::Translate(tx, ty, tz) => {
                        let tx = tx.to_au(border_box.size.width).to_f32_px();
                        let ty = ty.to_au(border_box.size.height).to_f32_px();
                        let tz = tz.to_f32_px();
                        Matrix4::create_translation(tx, ty, tz)
                    }
                    &transform::ComputedOperation::Matrix(m) => {
                        m.to_gfx_matrix()
                    }
                    &transform::ComputedOperation::Skew(sx, sy) => {
                        Matrix4::create_skew(sx, sy)
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

                let perspective_matrix = Matrix4::create_perspective(d.to_f32_px());

                pre_transform.mul(&perspective_matrix).mul(&post_transform)
            }
            LengthOrNone::None => {
                Matrix4::identity()
            }
        };

        // FIXME(pcwalton): Is this vertical-writing-direction-safe?
        let margin = self.margin.to_physical(base_flow.writing_mode);
        let overflow = base_flow.overflow.translate(&-Point2D::new(margin.left, Au(0)));

        // Create the filter pipeline.
        let effects = self.style().get_effects();
        let mut filters = effects.filter.clone();
        if effects.opacity != 1.0 {
            filters.push(Filter::Opacity(effects.opacity))
        }

        // Ensure every canvas has a layer
        let layer = match layer {
            StackingContextLayer::Existing(existing_layer) => Some(existing_layer),
            StackingContextLayer::IfCanvas(layer_id) => {
                if let SpecificFragmentInfo::Canvas(_) = self.specific {
                    Some(PaintLayer::new(layer_id, color::transparent(), ScrollPolicy::Scrollable))
                } else {
                    None
                }
            }
        };

        // If it's a canvas we must propagate the layer and the renderer to the paint
        // task
        if let SpecificFragmentInfo::Canvas(ref fragment_info) = self.specific {
            let layer_id = layer.as_ref().unwrap().id;
            layout_context.shared.canvas_layers_sender
                .send((layer_id, fragment_info.renderer.clone())).unwrap();
        }

        let transform_style = self.style().get_used_transform_style();
        let layer = layer.map(|l| Arc::new(l));

        Arc::new(StackingContext::new(display_list,
                                      &border_box,
                                      &overflow,
                                      self.style().get_box().z_index.number_or_zero(),
                                      filters,
                                      self.style().get_effects().mix_blend_mode,
                                      layer,
                                      transform,
                                      perspective,
                                      transform_style == transform_style::T::flat))
    }

    #[inline(never)]
    fn finalize_position_and_size_of_iframe(&self,
                                            iframe_fragment: &IframeFragmentInfo,
                                            offset: Point2D<Au>,
                                            layout_context: &LayoutContext) {
        let border_padding = (self.border_padding).to_physical(self.style.writing_mode);
        let content_size = self.content_box().size.to_physical(self.style.writing_mode);
        let iframe_rect = Rect::new(Point2D::new((offset.x + border_padding.left).to_f32_px(),
                                                 (offset.y + border_padding.top).to_f32_px()),
                                    Size2D::new(content_size.width.to_f32_px(),
                                                content_size.height.to_f32_px()));

        debug!("finalizing position and size of iframe for {:?},{:?}",
               iframe_fragment.pipeline_id,
               iframe_fragment.subpage_id);
        let ConstellationChan(ref chan) = layout_context.shared.constellation_chan;
        chan.send(ConstellationMsg::FrameRect(iframe_fragment.pipeline_id,
                                              iframe_fragment.subpage_id,
                                              iframe_rect)).unwrap();
    }

    fn clipping_region_for_children(&self,
                                    current_clip: &ClippingRegion,
                                    stacking_relative_border_box: &Rect<Au>)
                                    -> ClippingRegion {
        // Don't clip if we're text.
        if self.is_scanned_text_fragment() {
            return (*current_clip).clone()
        }

        // Account for style-specified `clip`.
        let mut current_clip = self.calculate_style_specified_clip(current_clip,
                                                                   stacking_relative_border_box);

        // Clip according to the values of `overflow-x` and `overflow-y`.
        //
        // TODO(pcwalton): Support scrolling.
        // FIXME(pcwalton): This may be more complex than it needs to be, since it seems to be
        // impossible with the computed value rules as they are to have `overflow-x: visible` with
        // `overflow-y: <scrolling>` or vice versa!
        match self.style.get_box().overflow_x {
            overflow_x::T::hidden | overflow_x::T::auto | overflow_x::T::scroll => {
                let mut bounds = current_clip.bounding_rect();
                let max_x = cmp::min(bounds.max_x(), stacking_relative_border_box.max_x());
                bounds.origin.x = cmp::max(bounds.origin.x, stacking_relative_border_box.origin.x);
                bounds.size.width = max_x - bounds.origin.x;
                current_clip = current_clip.intersect_rect(&bounds)
            }
            _ => {}
        }
        match self.style.get_box().overflow_y.0 {
            overflow_x::T::hidden | overflow_x::T::auto | overflow_x::T::scroll => {
                let mut bounds = current_clip.bounding_rect();
                let max_y = cmp::min(bounds.max_y(), stacking_relative_border_box.max_y());
                bounds.origin.y = cmp::max(bounds.origin.y, stacking_relative_border_box.origin.y);
                bounds.size.height = max_y - bounds.origin.y;
                current_clip = current_clip.intersect_rect(&bounds)
            }
            _ => {}
        }

        current_clip
    }

    fn build_display_list_for_text_fragment(&self,
                                            display_list: &mut DisplayList,
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
        display_list.content.push_back(DisplayItem::TextClass(box TextDisplayItem {
            base: BaseDisplayItem::new(stacking_relative_content_box,
                                       DisplayItemMetadata::new(self.node, self.style(), cursor),
                                       (*clip).clone()),
            text_run: text_fragment.run.clone(),
            range: text_fragment.range,
            text_color: text_color.to_gfx_color(),
            orientation: orientation,
            baseline_origin: baseline_origin,
            blur_radius: shadow_blur_radius.unwrap_or(Au(0)),
        }));

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
            self.build_display_list_for_text_decoration(display_list,
                                                        underline_color,
                                                        &stacking_relative_box,
                                                        clip,
                                                        shadow_blur_radius.unwrap_or(Au(0)))
        }

        if let Some(ref overline_color) = text_decorations.overline {
            let mut stacking_relative_box = stacking_relative_content_box;
            stacking_relative_box.size.block = metrics.underline_size;
            self.build_display_list_for_text_decoration(display_list,
                                                        overline_color,
                                                        &stacking_relative_box,
                                                        clip,
                                                        shadow_blur_radius.unwrap_or(Au(0)))
        }

        if let Some(ref line_through_color) = text_decorations.line_through {
            let mut stacking_relative_box = stacking_relative_content_box;
            stacking_relative_box.start.b = stacking_relative_box.start.b + metrics.ascent -
                metrics.strikeout_offset;
            stacking_relative_box.size.block = metrics.strikeout_size;
            self.build_display_list_for_text_decoration(display_list,
                                                        line_through_color,
                                                        &stacking_relative_box,
                                                        clip,
                                                        shadow_blur_radius.unwrap_or(Au(0)))
        }
    }

    fn build_display_list_for_text_decoration(&self,
                                              display_list: &mut DisplayList,
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
        display_list.content.push_back(DisplayItem::BoxShadowClass(box BoxShadowDisplayItem {
            base: BaseDisplayItem::new(shadow_bounds(&stacking_relative_box, blur_radius, Au(0)),
                                       metadata,
                                       (*clip).clone()),
            box_bounds: stacking_relative_box,
            color: color.to_gfx_color(),
            offset: ZERO_POINT,
            blur_radius: blur_radius,
            spread_radius: Au(0),
            clip_mode: BoxShadowClipMode::None,
        }))
    }
}

pub trait BlockFlowDisplayListBuilding {
    fn build_display_list_for_block_base(&mut self,
                                         display_list: &mut DisplayList,
                                         layout_context: &LayoutContext,
                                         border_painting_mode: BorderPaintingMode,
                                         background_border_level: BackgroundAndBorderLevel);
    fn build_display_list_for_static_block(&mut self,
                                           display_list: Box<DisplayList>,
                                           layout_context: &LayoutContext,
                                           border_painting_mode: BorderPaintingMode,
                                           background_border_level: BackgroundAndBorderLevel);
    fn build_display_list_for_absolutely_positioned_block(
            &mut self,
            display_list: Box<DisplayList>,
            layout_context: &LayoutContext,
            border_painting_mode: BorderPaintingMode);
    fn build_display_list_for_floating_block(&mut self,
                                             display_list: Box<DisplayList>,
                                             layout_context: &LayoutContext,
                                             border_painting_mode: BorderPaintingMode);
    fn build_display_list_for_block(&mut self,
                                    display_list: Box<DisplayList>,
                                    layout_context: &LayoutContext,
                                    border_painting_mode: BorderPaintingMode);
    fn will_get_layer(&self) -> bool;
}

impl BlockFlowDisplayListBuilding for BlockFlow {
    fn build_display_list_for_block_base(&mut self,
                                         display_list: &mut DisplayList,
                                         layout_context: &LayoutContext,
                                         border_painting_mode: BorderPaintingMode,
                                         background_border_level: BackgroundAndBorderLevel) {
        // Add the box that starts the block context.
        let clip = if self.fragment.establishes_stacking_context() {
            self.base.clip.translate(&-self.base.stacking_relative_position)
        } else {
            self.base.clip.clone()
        };
        self.fragment
            .build_display_list(display_list,
                                layout_context,
                                &self.base.stacking_relative_position,
                                &self.base.absolute_position_info.relative_containing_block_size,
                                self.base.absolute_position_info.relative_containing_block_mode,
                                border_painting_mode,
                                background_border_level,
                                &clip,
                                &self.base.stacking_relative_position_of_display_port);

        // Add children.
        for kid in self.base.children.iter_mut() {
            flow::mut_base(kid).display_list_building_result.add_to(display_list);
        }

        self.base.build_display_items_for_debugging_tint(display_list, self.fragment.node);
    }

    fn build_display_list_for_static_block(&mut self,
                                           mut display_list: Box<DisplayList>,
                                           layout_context: &LayoutContext,
                                           border_painting_mode: BorderPaintingMode,
                                           background_border_level: BackgroundAndBorderLevel) {
        self.build_display_list_for_block_base(&mut *display_list,
                                               layout_context,
                                               border_painting_mode,
                                               background_border_level);

        self.base.display_list_building_result = if self.fragment.establishes_stacking_context() {
            if self.will_get_layer() {
                // If we got here, then we need a new layer.
                let scroll_policy = if self.is_fixed() {
                    ScrollPolicy::FixedPosition
                } else {
                    ScrollPolicy::Scrollable
                };

                let paint_layer = PaintLayer::new(self.layer_id(0), color::transparent(), scroll_policy);
                let layer = StackingContextLayer::Existing(paint_layer);
                let stacking_context = self.fragment.create_stacking_context(&self.base,
                                                                             display_list,
                                                                             layout_context,
                                                                             layer);
                DisplayListBuildingResult::StackingContext(stacking_context)
            } else {
                DisplayListBuildingResult::StackingContext(
                    self.fragment.create_stacking_context(&self.base,
                                                          display_list,
                                                          layout_context,
                                                          StackingContextLayer::IfCanvas(self.layer_id(0))))
            }
        } else {
            match self.fragment.style.get_box().position {
                position::T::static_ => {}
                _ => {
                    display_list.form_pseudo_stacking_context_for_positioned_content();
                }
            }
            DisplayListBuildingResult::Normal(display_list)
        }
    }

    fn will_get_layer(&self) -> bool {
        self.base.absolute_position_info.layers_needed_for_positioned_flows ||
            self.base.flags.contains(NEEDS_LAYER)
    }

    fn build_display_list_for_absolutely_positioned_block(
            &mut self,
            mut display_list: Box<DisplayList>,
            layout_context: &LayoutContext,
            border_painting_mode: BorderPaintingMode) {
        self.build_display_list_for_block_base(&mut *display_list,
                                               layout_context,
                                               border_painting_mode,
                                               BackgroundAndBorderLevel::RootOfStackingContext);

        if !self.will_get_layer() {
            // We didn't need a layer.
            self.base.display_list_building_result =
                DisplayListBuildingResult::StackingContext(
                    self.fragment.create_stacking_context(&self.base,
                                                          display_list,
                                                          layout_context,
                                                          StackingContextLayer::IfCanvas(self.layer_id(0))));
            return
        }

        // If we got here, then we need a new layer.
        let scroll_policy = if self.is_fixed() {
            ScrollPolicy::FixedPosition
        } else {
            ScrollPolicy::Scrollable
        };

        let paint_layer = PaintLayer::new(self.layer_id(0), color::transparent(), scroll_policy);
        let stacking_context = self.fragment.create_stacking_context(&self.base,
                                                                     display_list,
                                                                     layout_context,
                                                                     StackingContextLayer::Existing(paint_layer));
        self.base.display_list_building_result =
            DisplayListBuildingResult::StackingContext(stacking_context)
    }

    fn build_display_list_for_floating_block(&mut self,
                                             mut display_list: Box<DisplayList>,
                                             layout_context: &LayoutContext,
                                             border_painting_mode: BorderPaintingMode) {
        self.build_display_list_for_block_base(&mut *display_list,
                                               layout_context,
                                               border_painting_mode,
                                               BackgroundAndBorderLevel::RootOfStackingContext);
        display_list.form_float_pseudo_stacking_context();

        self.base.display_list_building_result = if self.fragment.establishes_stacking_context() {
            DisplayListBuildingResult::StackingContext(
                self.fragment.create_stacking_context(&self.base,
                                                      display_list,
                                                      layout_context,
                                                      StackingContextLayer::IfCanvas(self.layer_id(0))))
        } else {
            DisplayListBuildingResult::Normal(display_list)
        }
    }

    fn build_display_list_for_block(&mut self,
                                    display_list: Box<DisplayList>,
                                    layout_context: &LayoutContext,
                                    border_painting_mode: BorderPaintingMode) {
        if self.base.flags.is_float() {
            // TODO(#2009, pcwalton): This is a pseudo-stacking context. We need to merge `z-index:
            // auto` kids into the parent stacking context, when that is supported.
            self.build_display_list_for_floating_block(display_list,
                                                       layout_context,
                                                       border_painting_mode);
        } else if self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) {
            self.build_display_list_for_absolutely_positioned_block(display_list,
                                                                    layout_context,
                                                                    border_painting_mode);
        } else {
            self.build_display_list_for_static_block(display_list,
                                                     layout_context,
                                                     border_painting_mode,
                                                     BackgroundAndBorderLevel::Block);
        }
    }
}

pub trait InlineFlowDisplayListBuilding {
    fn build_display_list_for_inline(&mut self, layout_context: &LayoutContext);
}

impl InlineFlowDisplayListBuilding for InlineFlow {
    fn build_display_list_for_inline(&mut self, layout_context: &LayoutContext) {
        // TODO(#228): Once we form lines and have their cached bounds, we can be smarter and
        // not recurse on a line if nothing in it can intersect the dirty region.
        debug!("Flow: building display list for {} inline fragments", self.fragments.len());

        let mut display_list = box DisplayList::new();
        let mut has_stacking_context = false;
        for fragment in self.fragments.fragments.iter_mut() {
            fragment.build_display_list(&mut *display_list,
                                        layout_context,
                                        &self.base.stacking_relative_position,
                                        &self.base
                                             .absolute_position_info
                                             .relative_containing_block_size,
                                        self.base
                                            .absolute_position_info
                                            .relative_containing_block_mode,
                                        BorderPaintingMode::Separate,
                                        BackgroundAndBorderLevel::Content,
                                        &self.base.clip,
                                        &self.base.stacking_relative_position_of_display_port);

            has_stacking_context = fragment.establishes_stacking_context();

            match fragment.specific {
                SpecificFragmentInfo::InlineBlock(ref mut block_flow) => {
                    let block_flow = &mut *block_flow.flow_ref;
                    flow::mut_base(block_flow).display_list_building_result
                                              .add_to(&mut *display_list)
                }
                SpecificFragmentInfo::InlineAbsoluteHypothetical(ref mut block_flow) => {
                    let block_flow = &mut *block_flow.flow_ref;
                    flow::mut_base(block_flow).display_list_building_result
                                              .add_to(&mut *display_list)
                }
                SpecificFragmentInfo::InlineAbsolute(ref mut block_flow) => {
                    let block_flow = &mut *block_flow.flow_ref;
                    flow::mut_base(block_flow).display_list_building_result
                                              .add_to(&mut *display_list)
                }
                _ => {}
            }
        }

        if !self.fragments.fragments.is_empty() {
            self.base.build_display_items_for_debugging_tint(&mut *display_list,
                                                             self.fragments.fragments[0].node);
        }

        // FIXME(Savago): fix Fragment::establishes_stacking_context() for absolute positioned item
        // and remove the check for filter presence. Further details on #5812.
        has_stacking_context = has_stacking_context && {
            if let SpecificFragmentInfo::Canvas(_) = self.fragments.fragments[0].specific {
                true
            } else {
                !self.fragments.fragments[0].style().get_effects().filter.is_empty()
            }
        };

        self.base.display_list_building_result = if has_stacking_context {
            DisplayListBuildingResult::StackingContext(
                self.fragments.fragments[0].create_stacking_context(&self.base,
                                                                    display_list,
                                                                    layout_context,
                                                                    StackingContextLayer::IfCanvas(self.layer_id(0))))
        } else {
            DisplayListBuildingResult::Normal(display_list)
        };

        if opts::get().validate_display_list_geometry {
            self.base.validate_display_list_geometry();
        }
    }
}

pub trait ListItemFlowDisplayListBuilding {
    fn build_display_list_for_list_item(&mut self,
                                        display_list: Box<DisplayList>,
                                        layout_context: &LayoutContext);
}

impl ListItemFlowDisplayListBuilding for ListItemFlow {
    fn build_display_list_for_list_item(&mut self,
                                        mut display_list: Box<DisplayList>,
                                        layout_context: &LayoutContext) {
        // Draw the marker, if applicable.
        if let Some(ref mut marker) = self.marker {
            marker.build_display_list(&mut *display_list,
                                      layout_context,
                                      &self.block_flow.base.stacking_relative_position,
                                      &self.block_flow
                                           .base
                                           .absolute_position_info
                                           .relative_containing_block_size,
                                      self.block_flow
                                          .base
                                          .absolute_position_info
                                          .relative_containing_block_mode,
                                      BorderPaintingMode::Separate,
                                      BackgroundAndBorderLevel::Content,
                                      &self.block_flow.base.clip,
                                      &self.block_flow
                                           .base
                                           .stacking_relative_position_of_display_port);
        }

        // Draw the rest of the block.
        self.block_flow.build_display_list_for_block(display_list,
                                                     layout_context,
                                                     BorderPaintingMode::Separate)
    }
}

trait BaseFlowDisplayListBuilding {
    fn build_display_items_for_debugging_tint(&self,
                                              display_list: &mut DisplayList,
                                              node: OpaqueNode);
}

impl BaseFlowDisplayListBuilding for BaseFlow {
    fn build_display_items_for_debugging_tint(&self,
                                              display_list: &mut DisplayList,
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
        display_list.push(DisplayItem::BorderClass(box BorderDisplayItem {
            base: BaseDisplayItem::new(stacking_context_relative_bounds.inflate(Au::from_px(2),
                                                                                Au::from_px(2)),
                                       DisplayItemMetadata {
                                           node: node,
                                           pointing: None,
                                       },
                                       self.clip.clone()),
            border_widths: SideOffsets2D::new_all_same(Au::from_px(2)),
            color: SideOffsets2D::new_all_same(color),
            style: SideOffsets2D::new_all_same(border_style::T::solid),
            radius: BorderRadii::all_same(Au(0)),
        }), StackingLevel::Content);
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

fn fmin(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}

fn position_to_offset(position: LengthOrPercentage, Au(total_length): Au) -> f32 {
    match position {
        LengthOrPercentage::Length(Au(length)) => {
            fmin(1.0, (length as f32) / (total_length as f32))
        }
        LengthOrPercentage::Percentage(percentage) => percentage as f32,
    }
}

/// "Steps" as defined by CSS 2.1 § E.2.
#[derive(Clone, PartialEq, Debug, Copy)]
pub enum StackingLevel {
    /// The border and backgrounds for the root of this stacking context: steps 1 and 2.
    BackgroundAndBorders,
    /// Borders and backgrounds for block-level descendants: step 4.
    BlockBackgroundsAndBorders,
    /// All non-positioned content.
    Content,
}

impl StackingLevel {
    #[inline]
    pub fn from_background_and_border_level(level: BackgroundAndBorderLevel) -> StackingLevel {
        match level {
            BackgroundAndBorderLevel::RootOfStackingContext => StackingLevel::BackgroundAndBorders,
            BackgroundAndBorderLevel::Block => StackingLevel::BlockBackgroundsAndBorders,
            BackgroundAndBorderLevel::Content => StackingLevel::Content,
        }
    }
}

/// Which level to place backgrounds and borders in.
pub enum BackgroundAndBorderLevel {
    RootOfStackingContext,
    Block,
    Content,
}

trait StackingContextConstruction {
    /// Adds the given display item at the specified level to this display list.
    fn push(&mut self, display_item: DisplayItem, level: StackingLevel);
}

impl StackingContextConstruction for DisplayList {
    fn push(&mut self, display_item: DisplayItem, level: StackingLevel) {
        match level {
            StackingLevel::BackgroundAndBorders => {
                self.background_and_borders.push_back(display_item)
            }
            StackingLevel::BlockBackgroundsAndBorders => {
                self.block_backgrounds_and_borders.push_back(display_item)
            }
            StackingLevel::Content => self.content.push_back(display_item),
        }
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

