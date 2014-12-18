/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Builds display lists from flows and fragments.
//!
//! Other browser engines sometimes call this "painting", but it is more accurately called display
//! list building, as the actual painting does not happen here—only deciding *what* we're going to
//! paint.

#![deny(unsafe_blocks)]

use block::BlockFlow;
use context::LayoutContext;
use flow::{mod, Flow, IS_ABSOLUTELY_POSITIONED, NEEDS_LAYER};
use fragment::{Fragment, SpecificFragmentInfo, IframeFragmentInfo, ImageFragmentInfo};
use fragment::ScannedTextFragmentInfo;
use list_item::ListItemFlow;
use model;
use util::{OpaqueNodeMethods, ToGfxColor};

use geom::approxeq::ApproxEq;
use geom::{Point2D, Rect, Size2D, SideOffsets2D};
use gfx::color;
use gfx::display_list::{BOX_SHADOW_INFLATION_FACTOR, BaseDisplayItem, BorderDisplayItem};
use gfx::display_list::{BorderRadii, BoxShadowDisplayItem};
use gfx::display_list::{DisplayItem, DisplayList, DisplayItemMetadata};
use gfx::display_list::{GradientDisplayItem};
use gfx::display_list::{GradientStop, ImageDisplayItem, LineDisplayItem};
use gfx::display_list::{SidewaysLeft};
use gfx::display_list::{SidewaysRight, SolidColorDisplayItem};
use gfx::display_list::{StackingContext, TextDisplayItem, Upright};
use gfx::paint_task::PaintLayer;
use servo_msg::compositor_msg::{FixedPosition, Scrollable};
use servo_msg::constellation_msg::{ConstellationChan, FrameRectMsg};
use servo_net::image::holder::ImageHolder;
use servo_util::cursor::{DefaultCursor, TextCursor, VerticalTextCursor};
use servo_util::geometry::{mod, Au, ZERO_POINT, ZERO_RECT};
use servo_util::logical_geometry::{LogicalRect, WritingMode};
use servo_util::opts;
use std::default::Default;
use std::num::FloatMath;
use style::computed::{AngleOrCorner, LengthOrPercentage, HorizontalDirection, VerticalDirection};
use style::computed::{Image, LinearGradient};
use style::computed_values::{background_attachment, background_repeat, border_style, overflow};
use style::computed_values::{position, visibility};
use style::style_structs::Border;
use style::{ComputedValues, RGBA};
use sync::Arc;
use url::Url;

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
                                                       clip_rect: &Rect<Au>);

    /// Adds the display items necessary to paint the background image of this fragment to the
    /// display list at the appropriate stacking level.
    fn build_display_list_for_background_image(&self,
                                               style: &ComputedValues,
                                               display_list: &mut DisplayList,
                                               layout_context: &LayoutContext,
                                               level: StackingLevel,
                                               absolute_bounds: &Rect<Au>,
                                               clip_rect: &Rect<Au>,
                                               image_url: &Url);

    /// Adds the display items necessary to paint the background linear gradient of this fragment
    /// to the display list at the appropriate stacking level.
    fn build_display_list_for_background_linear_gradient(&self,
                                                         display_list: &mut DisplayList,
                                                         level: StackingLevel,
                                                         absolute_bounds: &Rect<Au>,
                                                         clip_rect: &Rect<Au>,
                                                         gradient: &LinearGradient,
                                                         style: &ComputedValues);

    /// Adds the display items necessary to paint the borders of this fragment to a display list if
    /// necessary.
    fn build_display_list_for_borders_if_applicable(&self,
                                                    style: &ComputedValues,
                                                    display_list: &mut DisplayList,
                                                    abs_bounds: &Rect<Au>,
                                                    level: StackingLevel,
                                                    clip_rect: &Rect<Au>);

    /// Adds the display items necessary to paint the outline of this fragment to the display list
    /// if necessary.
    fn build_display_list_for_outline_if_applicable(&self,
                                                    style: &ComputedValues,
                                                    display_list: &mut DisplayList,
                                                    bounds: &Rect<Au>,
                                                    clip_rect: &Rect<Au>);

    /// Adds the display items necessary to paint the box shadow of this fragment to the display
    /// list if necessary.
    fn build_display_list_for_box_shadow_if_applicable(&self,
                                                       style: &ComputedValues,
                                                       list: &mut DisplayList,
                                                       layout_context: &LayoutContext,
                                                       level: StackingLevel,
                                                       absolute_bounds: &Rect<Au>,
                                                       clip_rect: &Rect<Au>);

    fn build_debug_borders_around_text_fragments(&self,
                                                 style: &ComputedValues,
                                                 display_list: &mut DisplayList,
                                                 flow_origin: Point2D<Au>,
                                                 text_fragment: &ScannedTextFragmentInfo,
                                                 clip_rect: &Rect<Au>);

    fn build_debug_borders_around_fragment(&self,
                                           display_list: &mut DisplayList,
                                           flow_origin: Point2D<Au>,
                                           clip_rect: &Rect<Au>);

    /// Adds the display items for this fragment to the given display list.
    ///
    /// Arguments:
    ///
    /// * `display_list`: The display list to add display items to.
    /// * `layout_context`: The layout context.
    /// * `dirty`: The dirty rectangle in the coordinate system of the owning flow.
    /// * `flow_origin`: Position of the origin of the owning flow wrt the display list root flow.
    /// * `clip_rect`: The rectangle to clip the display items to.
    fn build_display_list(&mut self,
                          display_list: &mut DisplayList,
                          layout_context: &LayoutContext,
                          flow_origin: Point2D<Au>,
                          background_and_border_level: BackgroundAndBorderLevel,
                          clip_rect: &Rect<Au>);

    /// Sends the size and position of this iframe fragment to the constellation. This is out of
    /// line to guide inlining.
    fn finalize_position_and_size_of_iframe(&self,
                                            iframe_fragment: &IframeFragmentInfo,
                                            offset: Point2D<Au>,
                                            layout_context: &LayoutContext);

    fn clip_rect_for_children(&self, current_clip_rect: &Rect<Au>, flow_origin: &Point2D<Au>)
                              -> Rect<Au>;

    /// Calculates the clipping rectangle for a fragment, taking the `clip` property into account
    /// per CSS 2.1 § 11.1.2.
    fn calculate_style_specified_clip(&self, parent_clip_rect: &Rect<Au>, origin: &Point2D<Au>)
                                      -> Rect<Au>;
}

fn build_border_radius(abs_bounds: &Rect<Au>, border_style: &Border) -> BorderRadii<Au> {
    // TODO(cgaebel): Support border radii even in the case of multiple border widths.
    // This is an extennsion of supporting elliptical radii. For now, all percentage
    // radii will be relative to the width.

    BorderRadii {
        top_left:     model::specified(border_style.border_top_left_radius.radius,     abs_bounds.size.width),
        top_right:    model::specified(border_style.border_top_right_radius.radius,    abs_bounds.size.width),
        bottom_right: model::specified(border_style.border_bottom_right_radius.radius, abs_bounds.size.width),
        bottom_left:  model::specified(border_style.border_bottom_left_radius.radius,  abs_bounds.size.width),
    }
}

impl FragmentDisplayListBuilding for Fragment {
    fn build_display_list_for_background_if_applicable(&self,
                                                       style: &ComputedValues,
                                                       display_list: &mut DisplayList,
                                                       layout_context: &LayoutContext,
                                                       level: StackingLevel,
                                                       absolute_bounds: &Rect<Au>,
                                                       clip_rect: &Rect<Au>) {
        // FIXME: This causes a lot of background colors to be displayed when they are clearly not
        // needed. We could use display list optimization to clean this up, but it still seems
        // inefficient. What we really want is something like "nearest ancestor element that
        // doesn't have a fragment".
        let background_color = style.resolve_color(style.get_background().background_color);
        if !background_color.alpha.approx_eq(&0.0) {
            display_list.push(DisplayItem::SolidColorClass(box SolidColorDisplayItem {
                base: BaseDisplayItem::new(*absolute_bounds,
                                           DisplayItemMetadata::new(self.node,
                                                                    style,
                                                                    DefaultCursor),
                                           *clip_rect),
                color: background_color.to_gfx_color(),
            }), level);
        }

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
                                                                       clip_rect,
                                                                       gradient,
                                                                       style)
            }
            Some(Image::Url(ref image_url)) => {
                self.build_display_list_for_background_image(style,
                                                             display_list,
                                                             layout_context,
                                                             level,
                                                             absolute_bounds,
                                                             clip_rect,
                                                             image_url)
            }
        }
    }

    fn build_display_list_for_background_image(&self,
                                               style: &ComputedValues,
                                               display_list: &mut DisplayList,
                                               layout_context: &LayoutContext,
                                               level: StackingLevel,
                                               absolute_bounds: &Rect<Au>,
                                               clip_rect: &Rect<Au>,
                                               image_url: &Url) {
        let background = style.get_background();
        let mut holder = ImageHolder::new(image_url.clone(),
                                          layout_context.shared.image_cache.clone());
        let image = match holder.get_image(self.node.to_untrusted_node_address()) {
            None => {
                // No image data at all? Do nothing.
                //
                // TODO: Add some kind of placeholder background image.
                debug!("(building display list) no background image :(");
                return
            }
            Some(image) => image,
        };
        debug!("(building display list) building background image");

        let image_width = Au::from_px(image.width as int);
        let image_height = Au::from_px(image.height as int);
        let mut bounds = *absolute_bounds;

        // Clip.
        //
        // TODO: Check the bounds to see if a clip item is actually required.
        let clip_rect = clip_rect.intersection(&bounds).unwrap_or(ZERO_RECT);

        // Use background-attachment to get the initial virtual origin
        let (virtual_origin_x, virtual_origin_y) = match background.background_attachment {
            background_attachment::scroll => {
                (absolute_bounds.origin.x, absolute_bounds.origin.y)
            }
            background_attachment::fixed => {
                (Au(0), Au(0))
            }
        };

        // Use background-position to get the offset
        let horizontal_position = model::specified(background.background_position.horizontal,
                                                   bounds.size.width - image_width);
        let vertical_position = model::specified(background.background_position.vertical,
                                                 bounds.size.height - image_height);

        let abs_x = virtual_origin_x + horizontal_position;
        let abs_y = virtual_origin_y + vertical_position;

        // Adjust origin and size based on background-repeat
        match background.background_repeat {
            background_repeat::no_repeat => {
                bounds.origin.x = abs_x;
                bounds.origin.y = abs_y;
                bounds.size.width = image_width;
                bounds.size.height = image_height;
            }
            background_repeat::repeat_x => {
                bounds.origin.y = abs_y;
                bounds.size.height = image_height;
                ImageFragmentInfo::tile_image(&mut bounds.origin.x, &mut bounds.size.width,
                                                abs_x, image.width);
            }
            background_repeat::repeat_y => {
                bounds.origin.x = abs_x;
                bounds.size.width = image_width;
                ImageFragmentInfo::tile_image(&mut bounds.origin.y, &mut bounds.size.height,
                                                abs_y, image.height);
            }
            background_repeat::repeat => {
                ImageFragmentInfo::tile_image(&mut bounds.origin.x, &mut bounds.size.width,
                                                abs_x, image.width);
                ImageFragmentInfo::tile_image(&mut bounds.origin.y, &mut bounds.size.height,
                                                abs_y, image.height);
            }
        };

        // Create the image display item.
        display_list.push(DisplayItem::ImageClass(box ImageDisplayItem {
            base: BaseDisplayItem::new(bounds,
                                       DisplayItemMetadata::new(self.node, style, DefaultCursor),
                                       clip_rect),
            image: image.clone(),
            stretch_size: Size2D(Au::from_px(image.width as int),
                                 Au::from_px(image.height as int)),
        }), level);
    }

    fn build_display_list_for_background_linear_gradient(&self,
                                                         display_list: &mut DisplayList,
                                                         level: StackingLevel,
                                                         absolute_bounds: &Rect<Au>,
                                                         clip_rect: &Rect<Au>,
                                                         gradient: &LinearGradient,
                                                         style: &ComputedValues) {
        let clip_rect = clip_rect.intersection(absolute_bounds).unwrap_or(ZERO_RECT);

        // This is the distance between the center and the ending point; i.e. half of the distance
        // between the starting point and the ending point.
        let delta = match gradient.angle_or_corner {
            AngleOrCorner::Angle(angle) => {
                Point2D(Au((angle.radians().sin() *
                             absolute_bounds.size.width.to_f64().unwrap() / 2.0) as i32),
                        Au((-angle.radians().cos() *
                             absolute_bounds.size.height.to_f64().unwrap() / 2.0) as i32))
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
                Point2D(Au(x_factor * absolute_bounds.size.width.to_i32().unwrap() / 2),
                        Au(y_factor * absolute_bounds.size.height.to_i32().unwrap() / 2))
            }
        };

        // This is the length of the gradient line.
        let length = Au((delta.x.to_f64().unwrap() * 2.0).hypot(delta.y.to_f64().unwrap() * 2.0)
                        as i32);

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
                            match gradient.stops
                                          .as_slice()
                                          .slice_from(i)
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

        let center = Point2D(absolute_bounds.origin.x + absolute_bounds.size.width / 2,
                             absolute_bounds.origin.y + absolute_bounds.size.height / 2);

        let gradient_display_item = DisplayItem::GradientClass(box GradientDisplayItem {
            base: BaseDisplayItem::new(*absolute_bounds,
                                       DisplayItemMetadata::new(self.node, style, DefaultCursor),
                                       clip_rect),
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
                                                       clip_rect: &Rect<Au>) {
        // NB: According to CSS-BACKGROUNDS, box shadows render in *reverse* order (front to back).
        for box_shadow in style.get_effects().box_shadow.iter().rev() {
            let inflation = box_shadow.spread_radius + box_shadow.blur_radius *
                BOX_SHADOW_INFLATION_FACTOR;
            let bounds =
                absolute_bounds.translate(&Point2D(box_shadow.offset_x, box_shadow.offset_y))
                               .inflate(inflation, inflation);
            list.push(DisplayItem::BoxShadowClass(box BoxShadowDisplayItem {
                base: BaseDisplayItem::new(bounds,
                                           DisplayItemMetadata::new(self.node,
                                                                    style,
                                                                    DefaultCursor),
                                           *clip_rect),
                box_bounds: *absolute_bounds,
                color: style.resolve_color(box_shadow.color).to_gfx_color(),
                offset: Point2D(box_shadow.offset_x, box_shadow.offset_y),
                blur_radius: box_shadow.blur_radius,
                spread_radius: box_shadow.spread_radius,
                inset: box_shadow.inset,
            }), level);
        }
    }

    fn build_display_list_for_borders_if_applicable(&self,
                                                    style: &ComputedValues,
                                                    display_list: &mut DisplayList,
                                                    abs_bounds: &Rect<Au>,
                                                    level: StackingLevel,
                                                    clip_rect: &Rect<Au>) {
        let border = style.logical_border_width();
        if border.is_zero() {
            return
        }

        let top_color = style.resolve_color(style.get_border().border_top_color);
        let right_color = style.resolve_color(style.get_border().border_right_color);
        let bottom_color = style.resolve_color(style.get_border().border_bottom_color);
        let left_color = style.resolve_color(style.get_border().border_left_color);

        // Append the border to the display list.
        display_list.push(DisplayItem::BorderClass(box BorderDisplayItem {
            base: BaseDisplayItem::new(*abs_bounds,
                                       DisplayItemMetadata::new(self.node, style, DefaultCursor),
                                       *clip_rect),
            border_widths: border.to_physical(style.writing_mode),
            color: SideOffsets2D::new(top_color.to_gfx_color(),
                                      right_color.to_gfx_color(),
                                      bottom_color.to_gfx_color(),
                                      left_color.to_gfx_color()),
            style: SideOffsets2D::new(style.get_border().border_top_style,
                                      style.get_border().border_right_style,
                                      style.get_border().border_bottom_style,
                                      style.get_border().border_left_style),
            radius: build_border_radius(abs_bounds, style.get_border()),
        }), level);
    }

    fn build_display_list_for_outline_if_applicable(&self,
                                                    style: &ComputedValues,
                                                    display_list: &mut DisplayList,
                                                    bounds: &Rect<Au>,
                                                    clip_rect: &Rect<Au>) {
        let width = style.get_outline().outline_width;
        if width == Au(0) {
            return
        }

        let outline_style = style.get_outline().outline_style;
        if outline_style == border_style::none {
            return
        }

        // Outlines are not accounted for in the dimensions of the border box, so adjust the
        // absolute bounds.
        let mut bounds = *bounds;
        bounds.origin.x = bounds.origin.x - width;
        bounds.origin.y = bounds.origin.y - width;
        bounds.size.width = bounds.size.width + width + width;
        bounds.size.height = bounds.size.height + width + width;

        // Append the outline to the display list.
        let color = style.resolve_color(style.get_outline().outline_color).to_gfx_color();
        display_list.outlines.push_back(DisplayItem::BorderClass(box BorderDisplayItem {
            base: BaseDisplayItem::new(bounds,
                                       DisplayItemMetadata::new(self.node, style, DefaultCursor),
                                       *clip_rect),
            border_widths: SideOffsets2D::new_all_same(width),
            color: SideOffsets2D::new_all_same(color),
            style: SideOffsets2D::new_all_same(outline_style),
            radius: Default::default(),
        }))
    }

    fn build_debug_borders_around_text_fragments(&self,
                                                 style: &ComputedValues,
                                                 display_list: &mut DisplayList,
                                                 flow_origin: Point2D<Au>,
                                                 text_fragment: &ScannedTextFragmentInfo,
                                                 clip_rect: &Rect<Au>) {
        // FIXME(#2795): Get the real container size
        let container_size = Size2D::zero();
        // Fragment position wrt to the owning flow.
        let fragment_bounds = self.border_box.to_physical(self.style.writing_mode, container_size);
        let absolute_fragment_bounds = Rect(
            fragment_bounds.origin + flow_origin,
            fragment_bounds.size);

        // Compute the text fragment bounds and draw a border surrounding them.
        display_list.content.push_back(DisplayItem::BorderClass(box BorderDisplayItem {
            base: BaseDisplayItem::new(absolute_fragment_bounds,
                                       DisplayItemMetadata::new(self.node, style, DefaultCursor),
                                       *clip_rect),
            border_widths: SideOffsets2D::new_all_same(Au::from_px(1)),
            color: SideOffsets2D::new_all_same(color::rgb(0, 0, 200)),
            style: SideOffsets2D::new_all_same(border_style::solid),
            radius: Default::default(),
        }));

        // Draw a rectangle representing the baselines.
        let ascent = text_fragment.run.ascent();
        let mut baseline = self.border_box.clone();
        baseline.start.b = baseline.start.b + ascent;
        baseline.size.block = Au(0);
        let mut baseline = baseline.to_physical(self.style.writing_mode, container_size);
        baseline.origin = baseline.origin + flow_origin;

        let line_display_item = box LineDisplayItem {
            base: BaseDisplayItem::new(baseline,
                                       DisplayItemMetadata::new(self.node, style, DefaultCursor),
                                       *clip_rect),
            color: color::rgb(0, 200, 0),
            style: border_style::dashed,
        };
        display_list.content.push_back(DisplayItem::LineClass(line_display_item));
    }

    fn build_debug_borders_around_fragment(&self,
                                           display_list: &mut DisplayList,
                                           flow_origin: Point2D<Au>,
                                           clip_rect: &Rect<Au>) {
        // FIXME(#2795): Get the real container size
        let container_size = Size2D::zero();
        // Fragment position wrt to the owning flow.
        let fragment_bounds = self.border_box.to_physical(self.style.writing_mode, container_size);
        let absolute_fragment_bounds = Rect(
            fragment_bounds.origin + flow_origin,
            fragment_bounds.size);

        // This prints a debug border around the border of this fragment.
        display_list.content.push_back(DisplayItem::BorderClass(box BorderDisplayItem {
            base: BaseDisplayItem::new(absolute_fragment_bounds,
                                       DisplayItemMetadata::new(self.node,
                                                                &*self.style,
                                                                DefaultCursor),
                                       *clip_rect),
            border_widths: SideOffsets2D::new_all_same(Au::from_px(1)),
            color: SideOffsets2D::new_all_same(color::rgb(0, 0, 200)),
            style: SideOffsets2D::new_all_same(border_style::solid),
            radius: Default::default(),
        }));
    }

    fn calculate_style_specified_clip(&self, parent_clip_rect: &Rect<Au>, origin: &Point2D<Au>)
                                      -> Rect<Au> {
        // Account for `clip` per CSS 2.1 § 11.1.2.
        let style_clip_rect = match (self.style().get_box().position,
                                     self.style().get_effects().clip) {
            (position::absolute, Some(style_clip_rect)) => style_clip_rect,
            _ => return *parent_clip_rect,
        };

        // FIXME(pcwalton, #2795): Get the real container size.
        let border_box = self.border_box.to_physical(self.style.writing_mode, Size2D::zero());
        let clip_origin = Point2D(border_box.origin.x + style_clip_rect.left,
                                  border_box.origin.y + style_clip_rect.top);
        Rect(clip_origin + *origin,
             Size2D(style_clip_rect.right.unwrap_or(border_box.size.width) - clip_origin.x,
                    style_clip_rect.bottom.unwrap_or(border_box.size.height) - clip_origin.y))
    }

    fn build_display_list(&mut self,
                          display_list: &mut DisplayList,
                          layout_context: &LayoutContext,
                          flow_origin: Point2D<Au>,
                          background_and_border_level: BackgroundAndBorderLevel,
                          clip_rect: &Rect<Au>) {
        // Compute the fragment position relative to the parent stacking context. If the fragment
        // itself establishes a stacking context, then the origin of its position will be (0, 0)
        // for the purposes of this computation.
        let stacking_relative_flow_origin = if self.establishes_stacking_context() {
            ZERO_POINT
        } else {
            flow_origin
        };
        let absolute_fragment_bounds =
            self.stacking_relative_bounds(&stacking_relative_flow_origin);

        // FIXME(#2795): Get the real container size
        let container_size = Size2D::zero();
        let rect_to_absolute = |writing_mode: WritingMode, logical_rect: LogicalRect<Au>| {
            let physical_rect = logical_rect.to_physical(writing_mode, container_size);
            Rect(physical_rect.origin + stacking_relative_flow_origin, physical_rect.size)
        };

        debug!("Fragment::build_display_list at rel={}, abs={}: {}",
               self.border_box,
               absolute_fragment_bounds,
               self);
        debug!("Fragment::build_display_list: dirty={}, flow_origin={}",
               layout_context.shared.dirty,
               flow_origin);

        if self.style().get_inheritedbox().visibility != visibility::visible {
            return
        }

        if !absolute_fragment_bounds.intersects(&layout_context.shared.dirty) {
            debug!("Fragment::build_display_list: Did not intersect...");
            return
        }

        // Calculate the clip rect. If there's nothing to render at all, don't even construct
        // display list items.
        let clip_rect = self.calculate_style_specified_clip(clip_rect,
                                                            &absolute_fragment_bounds.origin);
        if !absolute_fragment_bounds.intersects(&clip_rect) {
            return;
        }

        debug!("Fragment::build_display_list: intersected. Adding display item...");

        if self.is_primary_fragment() {
            let level =
                StackingLevel::from_background_and_border_level(background_and_border_level);

            // Add a shadow to the list, if applicable.
            match self.inline_context {
                Some(ref inline_context) => {
                    for style in inline_context.styles.iter().rev() {
                        self.build_display_list_for_box_shadow_if_applicable(
                            &**style,
                            display_list,
                            layout_context,
                            level,
                            &absolute_fragment_bounds,
                            &clip_rect);
                    }
                }
                None => {}
            }
            match self.specific {
                SpecificFragmentInfo::ScannedText(_) => {},
                _ => {
                    self.build_display_list_for_box_shadow_if_applicable(
                        &*self.style,
                        display_list,
                        layout_context,
                        level,
                        &absolute_fragment_bounds,
                        &clip_rect);
                }
            }

            // Add the background to the list, if applicable.
            match self.inline_context {
                Some(ref inline_context) => {
                    for style in inline_context.styles.iter().rev() {
                        self.build_display_list_for_background_if_applicable(
                            &**style,
                            display_list,
                            layout_context,
                            level,
                            &absolute_fragment_bounds,
                            &clip_rect);
                    }
                }
                None => {}
            }
            match self.specific {
                SpecificFragmentInfo::ScannedText(_) => {},
                _ => {
                    self.build_display_list_for_background_if_applicable(
                        &*self.style,
                        display_list,
                        layout_context,
                        level,
                        &absolute_fragment_bounds,
                        &clip_rect);
                }
            }

            // Add a border and outlines, if applicable.
            match self.inline_context {
                Some(ref inline_context) => {
                    for style in inline_context.styles.iter().rev() {
                        self.build_display_list_for_borders_if_applicable(
                            &**style,
                            display_list,
                            &absolute_fragment_bounds,
                            level,
                            &clip_rect);
                        self.build_display_list_for_outline_if_applicable(
                            &**style,
                            display_list,
                            &absolute_fragment_bounds,
                            &clip_rect);
                    }
                }
                None => {}
            }
            match self.specific {
                SpecificFragmentInfo::ScannedText(_) => {},
                _ => {
                    self.build_display_list_for_borders_if_applicable(
                        &*self.style,
                        display_list,
                        &absolute_fragment_bounds,
                        level,
                        &clip_rect);
                    self.build_display_list_for_outline_if_applicable(
                        &*self.style,
                        display_list,
                        &absolute_fragment_bounds,
                        &clip_rect);
                }
            }
        }

        let content_box = self.content_box();
        let absolute_content_box = rect_to_absolute(self.style.writing_mode, content_box);

        // Create special per-fragment-type display items.
        match self.specific {
            SpecificFragmentInfo::UnscannedText(_) => panic!("Shouldn't see unscanned fragments here."),
            SpecificFragmentInfo::TableColumn(_) => panic!("Shouldn't see table column fragments here."),
            SpecificFragmentInfo::ScannedText(ref text_fragment) => {
                // Create the text display item.
                let (orientation, cursor) = if self.style.writing_mode.is_vertical() {
                    if self.style.writing_mode.is_sideways_left() {
                        (SidewaysLeft, VerticalTextCursor)
                    } else {
                        (SidewaysRight, VerticalTextCursor)
                    }
                } else {
                    (Upright, TextCursor)
                };

                let metrics = &text_fragment.run.font_metrics;
                let baseline_origin = {
                    let mut content_box_start = content_box.start;
                    content_box_start.b = content_box_start.b + metrics.ascent;
                    content_box_start.to_physical(self.style.writing_mode, container_size)
                        + flow_origin
                };

                display_list.content.push_back(DisplayItem::TextClass(box TextDisplayItem {
                    base: BaseDisplayItem::new(absolute_content_box,
                                               DisplayItemMetadata::new(self.node,
                                                                        self.style(),
                                                                        cursor),
                                               clip_rect),
                    text_run: text_fragment.run.clone(),
                    range: text_fragment.range,
                    text_color: self.style().get_color().color.to_gfx_color(),
                    orientation: orientation,
                    baseline_origin: baseline_origin,
                }));

                // Create display items for text decoration
                {
                    let line = |maybe_color: Option<RGBA>,
                                style: &ComputedValues,
                                rect: || -> LogicalRect<Au>| {
                        match maybe_color {
                            None => {}
                            Some(color) => {
                                let bounds = rect_to_absolute(self.style.writing_mode, rect());
                                display_list.content.push_back(DisplayItem::SolidColorClass(
                                    box SolidColorDisplayItem {
                                        base: BaseDisplayItem::new(
                                                  bounds,
                                                  DisplayItemMetadata::new(self.node,
                                                                           style,
                                                                           DefaultCursor),
                                                  clip_rect),
                                        color: color.to_gfx_color(),
                                    }))
                            }
                        }
                    };

                    let text_decorations =
                        self.style().get_inheritedtext()._servo_text_decorations_in_effect;
                    line(text_decorations.underline, self.style(), || {
                        let mut rect = content_box.clone();
                        rect.start.b = rect.start.b + metrics.ascent - metrics.underline_offset;
                        rect.size.block = metrics.underline_size;
                        rect
                    });

                    line(text_decorations.overline, self.style(), || {
                        let mut rect = content_box.clone();
                        rect.size.block = metrics.underline_size;
                        rect
                    });

                    line(text_decorations.line_through, self.style(), || {
                        let mut rect = content_box.clone();
                        rect.start.b = rect.start.b + metrics.ascent - metrics.strikeout_offset;
                        rect.size.block = metrics.strikeout_size;
                        rect
                    });
                }

                if opts::get().show_debug_fragment_borders {
                    self.build_debug_borders_around_text_fragments(self.style(),
                                                                   display_list,
                                                                   flow_origin,
                                                                   &**text_fragment,
                                                                   &clip_rect);
                }
            }
            SpecificFragmentInfo::Generic | SpecificFragmentInfo::Iframe(..) | SpecificFragmentInfo::Table | SpecificFragmentInfo::TableCell |
            SpecificFragmentInfo::TableRow | SpecificFragmentInfo::TableWrapper | SpecificFragmentInfo::InlineBlock(_) |
            SpecificFragmentInfo::InlineAbsoluteHypothetical(_) => {
                if opts::get().show_debug_fragment_borders {
                    self.build_debug_borders_around_fragment(display_list,
                                                             flow_origin,
                                                             &clip_rect);
                }
            }
            SpecificFragmentInfo::Image(ref mut image_fragment) => {
                let image_ref = &mut image_fragment.image;
                match image_ref.get_image(self.node.to_untrusted_node_address()) {
                    Some(image) => {
                        debug!("(building display list) building image fragment");

                        // Place the image into the display list.
                        display_list.content.push_back(DisplayItem::ImageClass(box ImageDisplayItem {
                            base: BaseDisplayItem::new(absolute_content_box,
                                                       DisplayItemMetadata::new(self.node,
                                                                                &*self.style,
                                                                                DefaultCursor),
                                                       clip_rect),
                            image: image.clone(),
                            stretch_size: absolute_content_box.size,
                        }));
                    }
                    None => {
                        // No image data at all? Do nothing.
                        //
                        // TODO: Add some kind of placeholder image.
                        debug!("(building display list) no image :(");
                    }
                }
            }
        }

        if opts::get().show_debug_fragment_borders {
           self.build_debug_borders_around_fragment(display_list, flow_origin, &clip_rect)
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
        match self.specific {
            SpecificFragmentInfo::Iframe(ref iframe_fragment) => {
                self.finalize_position_and_size_of_iframe(&**iframe_fragment,
                                                          absolute_fragment_bounds.origin,
                                                          layout_context)
            }
            _ => {}
        }
    }

    #[inline(never)]
    fn finalize_position_and_size_of_iframe(&self,
                                            iframe_fragment: &IframeFragmentInfo,
                                            offset: Point2D<Au>,
                                            layout_context: &LayoutContext) {
        let border_padding = (self.border_padding).to_physical(self.style.writing_mode);
        let content_size = self.content_box().size.to_physical(self.style.writing_mode);
        let iframe_rect = Rect(Point2D(geometry::to_frac_px(offset.x + border_padding.left) as f32,
                                       geometry::to_frac_px(offset.y + border_padding.top) as f32),
                               Size2D(geometry::to_frac_px(content_size.width) as f32,
                                      geometry::to_frac_px(content_size.height) as f32));

        debug!("finalizing position and size of iframe for {},{}",
               iframe_fragment.pipeline_id,
               iframe_fragment.subpage_id);
        let ConstellationChan(ref chan) = layout_context.shared.constellation_chan;
        chan.send(FrameRectMsg(iframe_fragment.pipeline_id,
                               iframe_fragment.subpage_id,
                               iframe_rect));
    }

    fn clip_rect_for_children(&self, current_clip_rect: &Rect<Au>, origin: &Point2D<Au>)
                              -> Rect<Au> {
        // Don't clip if we're text.
        match self.specific {
            SpecificFragmentInfo::ScannedText(_) => return *current_clip_rect,
            _ => {}
        }

        // Account for style-specified `clip`.
        let current_clip_rect = self.calculate_style_specified_clip(current_clip_rect, origin);

        // Only clip if `overflow` tells us to.
        match self.style.get_box().overflow {
            overflow::hidden | overflow::auto | overflow::scroll => {}
            _ => return current_clip_rect,
        }

        // Create a new clip rect.
        //
        // FIXME(#2795): Get the real container size.
        let physical_rect = self.border_box.to_physical(self.style.writing_mode, Size2D::zero());
        current_clip_rect.intersection(&Rect(Point2D(physical_rect.origin.x + origin.x,
                                                     physical_rect.origin.y + origin.y),
                                             physical_rect.size)).unwrap_or(ZERO_RECT)
    }
}

pub trait BlockFlowDisplayListBuilding {
    fn build_display_list_for_block_base(&mut self,
                                         display_list: &mut DisplayList,
                                         layout_context: &LayoutContext,
                                         background_border_level: BackgroundAndBorderLevel);
    fn build_display_list_for_static_block(&mut self,
                                           display_list: Box<DisplayList>,
                                           layout_context: &LayoutContext,
                                           background_border_level: BackgroundAndBorderLevel);
    fn build_display_list_for_absolutely_positioned_block(&mut self,
                                                          display_list: Box<DisplayList>,
                                                          layout_context: &LayoutContext);
    fn build_display_list_for_floating_block(&mut self,
                                             display_list: Box<DisplayList>,
                                             layout_context: &LayoutContext);
    fn build_display_list_for_block(&mut self,
                                    display_list: Box<DisplayList>,
                                    layout_context: &LayoutContext);
    fn create_stacking_context(&self,
                               display_list: Box<DisplayList>,
                               layer: Option<Arc<PaintLayer>>)
                               -> Arc<StackingContext>;
}

impl BlockFlowDisplayListBuilding for BlockFlow {
    fn build_display_list_for_block_base(&mut self,
                                         display_list: &mut DisplayList,
                                         layout_context: &LayoutContext,
                                         background_border_level: BackgroundAndBorderLevel) {
        // Add the box that starts the block context.
        let stacking_relative_fragment_origin =
            self.base.stacking_relative_position_of_child_fragment(&self.fragment);
        self.fragment.build_display_list(display_list,
                                         layout_context,
                                         stacking_relative_fragment_origin,
                                         background_border_level,
                                         &self.base.clip_rect);

        for kid in self.base.children.iter_mut() {
            flow::mut_base(kid).display_list_building_result.add_to(display_list);
        }
    }

    fn build_display_list_for_static_block(&mut self,
                                           mut display_list: Box<DisplayList>,
                                           layout_context: &LayoutContext,
                                           background_border_level: BackgroundAndBorderLevel) {
        self.build_display_list_for_block_base(&mut *display_list,
                                               layout_context,
                                               background_border_level);

        self.base.display_list_building_result = if self.fragment.establishes_stacking_context() {
            DisplayListBuildingResult::StackingContext(self.create_stacking_context(display_list, None))
        } else {
            DisplayListBuildingResult::Normal(display_list)
        }
    }

    fn build_display_list_for_absolutely_positioned_block(&mut self,
                                                          mut display_list: Box<DisplayList>,
                                                          layout_context: &LayoutContext) {
        self.build_display_list_for_block_base(&mut *display_list,
                                               layout_context,
                                               BackgroundAndBorderLevel::RootOfStackingContext);

        if !self.base.absolute_position_info.layers_needed_for_positioned_flows &&
                !self.base.flags.contains(NEEDS_LAYER) {
            // We didn't need a layer.
            self.base.display_list_building_result =
                DisplayListBuildingResult::StackingContext(self.create_stacking_context(display_list, None));
            return
        }

        // If we got here, then we need a new layer.
        let scroll_policy = if self.is_fixed() {
            FixedPosition
        } else {
            Scrollable
        };

        let transparent = color::rgba(1.0, 1.0, 1.0, 0.0);
        let stacking_context =
            self.create_stacking_context(display_list,
                                         Some(Arc::new(PaintLayer::new(self.layer_id(0),
                                                                       transparent,
                                                                       scroll_policy))));
        self.base.display_list_building_result = DisplayListBuildingResult::StackingContext(stacking_context)
    }

    fn build_display_list_for_floating_block(&mut self,
                                             mut display_list: Box<DisplayList>,
                                             layout_context: &LayoutContext) {
        self.build_display_list_for_block_base(&mut *display_list,
                                               layout_context,
                                               BackgroundAndBorderLevel::RootOfStackingContext);
        display_list.form_float_pseudo_stacking_context();

        self.base.display_list_building_result = if self.fragment.establishes_stacking_context() {
            DisplayListBuildingResult::StackingContext(self.create_stacking_context(display_list, None))
        } else {
            DisplayListBuildingResult::Normal(display_list)
        }
    }

    fn build_display_list_for_block(&mut self,
                                    display_list: Box<DisplayList>,
                                    layout_context: &LayoutContext) {
        if self.base.flags.is_float() {
            // TODO(#2009, pcwalton): This is a pseudo-stacking context. We need to merge `z-index:
            // auto` kids into the parent stacking context, when that is supported.
            self.build_display_list_for_floating_block(display_list, layout_context)
        } else if self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) {
            self.build_display_list_for_absolutely_positioned_block(display_list, layout_context)
        } else {
            self.build_display_list_for_static_block(display_list, layout_context, BackgroundAndBorderLevel::Block)
        }
    }

    fn create_stacking_context(&self,
                               display_list: Box<DisplayList>,
                               layer: Option<Arc<PaintLayer>>)
                               -> Arc<StackingContext> {
        let bounds = Rect(self.base.stacking_relative_position,
                          self.base.overflow.size.to_physical(self.base.writing_mode));
        let z_index = self.fragment.style().get_box().z_index.number_or_zero();
        let opacity = self.fragment.style().get_effects().opacity as f32;
        Arc::new(StackingContext::new(display_list, bounds, z_index, opacity, layer))
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
        match self.marker {
            None => {}
            Some(ref mut marker) => {
                let stacking_relative_fragment_origin =
                    self.block_flow.base.stacking_relative_position_of_child_fragment(marker);
                marker.build_display_list(&mut *display_list,
                                          layout_context,
                                          stacking_relative_fragment_origin,
                                          BackgroundAndBorderLevel::Content,
                                          &self.block_flow.base.clip_rect);
            }
        }

        // Draw the rest of the block.
        self.block_flow.build_display_list_for_block(display_list, layout_context)
    }
}

// A helper data structure for gradients.
struct StopRun {
    start_offset: f32,
    end_offset: f32,
    start_index: uint,
    stop_count: uint,
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
        LengthOrPercentage::Length(Au(length)) => fmin(1.0, (length as f32) / (total_length as f32)),
        LengthOrPercentage::Percentage(percentage) => percentage as f32,
    }
}

/// "Steps" as defined by CSS 2.1 § E.2.
#[deriving(Clone, PartialEq, Show)]
pub enum StackingLevel {
    /// The border and backgrounds for the root of this stacking context: steps 1 and 2.
    BackgroundAndBorders,
    /// Borders and backgrounds for block-level descendants: step 4.
    BlockBackgroundsAndBorders,
    /// All other content.
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
