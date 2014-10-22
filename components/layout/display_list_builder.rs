/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Builds display lists from flows and fragments.
//!
//! Other browser engines sometimes call this "painting", but it is more accurately called display
//! list building, as the actual painting does not happen here—only deciding *what* we're going to
//! paint.

#![deny(unsafe_block)]

use block::BlockFlow;
use context::LayoutContext;
use flow::{mod, Flow};
use fragment::{Fragment, GenericFragment, IframeFragment, IframeFragmentInfo, ImageFragment};
use fragment::{ImageFragmentInfo, InlineAbsoluteHypotheticalFragment, InlineBlockFragment};
use fragment::{InputFragment, ScannedTextFragment, ScannedTextFragmentInfo, TableFragment};
use fragment::{TableCellFragment, TableColumnFragment, TableRowFragment, TableWrapperFragment};
use fragment::{UnscannedTextFragment};
use model;
use util::{OpaqueNodeMethods, ToGfxColor};

use collections::dlist::DList;
use geom::approxeq::ApproxEq;
use geom::{Point2D, Rect, Size2D, SideOffsets2D};
use gfx::color;
use gfx::display_list::{BackgroundAndBorderLevel, BaseDisplayItem, BorderDisplayItem};
use gfx::display_list::{BorderDisplayItemClass, ContentStackingLevel, DisplayList};
use gfx::display_list::{FloatStackingLevel, ImageDisplayItem, ImageDisplayItemClass};
use gfx::display_list::{LineDisplayItem, LineDisplayItemClass, PositionedDescendantStackingLevel};
use gfx::display_list::{PseudoDisplayItemClass, RootOfStackingContextLevel, SidewaysLeft};
use gfx::display_list::{SidewaysRight, SolidColorDisplayItem, SolidColorDisplayItemClass};
use gfx::display_list::{StackingLevel, TextDisplayItem, TextDisplayItemClass, Upright};
use gfx::render_task::RenderLayer;
use servo_msg::compositor_msg::{FixedPosition, Scrollable};
use servo_msg::constellation_msg::{ConstellationChan, FrameRectMsg};
use servo_net::image::holder::ImageHolder;
use servo_util::geometry::{mod, Au, ZERO_RECT};
use servo_util::logical_geometry::{LogicalRect, WritingMode};
use servo_util::opts;
use std::mem;
use style::{ComputedValues, RGBA};
use style::computed_values::{background_attachment, background_repeat, border_style, overflow};
use style::computed_values::{visibility};
use sync::Arc;

pub trait FragmentDisplayListBuilding {
    /// Adds the display items necessary to paint the background of this fragment to the display
    /// list if necessary.
    fn build_display_list_for_background_if_applicable(&self,
                                                       style: &ComputedValues,
                                                       list: &mut DisplayList,
                                                       layout_context: &LayoutContext,
                                                       level: StackingLevel,
                                                       absolute_bounds: &Rect<Au>,
                                                       clip_rect: &Rect<Au>);

    /// Adds the display items necessary to paint the borders of this fragment to a display list if
    /// necessary.
    fn build_display_list_for_borders_if_applicable(&self,
                                                    style: &ComputedValues,
                                                    list: &mut DisplayList,
                                                    abs_bounds: &Rect<Au>,
                                                    level: StackingLevel,
                                                    clip_rect: &Rect<Au>);

    fn build_debug_borders_around_text_fragments(&self,
                                                 display_list: &mut DisplayList,
                                                 flow_origin: Point2D<Au>,
                                                 text_fragment: &ScannedTextFragmentInfo,
                                                 clip_rect: &Rect<Au>);

    fn build_debug_borders_around_fragment(&self,
                                           display_list: &mut DisplayList,
                                           flow_origin: Point2D<Au>,
                                           clip_rect: &Rect<Au>);

    /// Adds the display items for this fragment to the given stacking context.
    ///
    /// Arguments:
    ///
    /// * `display_list`: The unflattened display list to add display items to.
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

    fn clip_rect_for_children(&self, current_clip_rect: Rect<Au>, flow_origin: Point2D<Au>)
                              -> Rect<Au>;
}

impl FragmentDisplayListBuilding for Fragment {
    fn build_display_list_for_background_if_applicable(&self,
                                                       style: &ComputedValues,
                                                       list: &mut DisplayList,
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
            let display_item = box SolidColorDisplayItem {
                base: BaseDisplayItem::new(*absolute_bounds, self.node, level, *clip_rect),
                color: background_color.to_gfx_color(),
            };

            list.push(SolidColorDisplayItemClass(display_item))
        }

        // The background image is painted on top of the background color.
        // Implements background image, per spec:
        // http://www.w3.org/TR/CSS21/colors.html#background
        let background = style.get_background();
        let image_url = match background.background_image {
            None => return,
            Some(ref image_url) => image_url,
        };

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
        let image_display_item = ImageDisplayItemClass(box ImageDisplayItem {
            base: BaseDisplayItem::new(bounds, self.node, level, clip_rect),
            image: image.clone(),
            stretch_size: Size2D(Au::from_px(image.width as int),
                                 Au::from_px(image.height as int)),
        });
        list.push(image_display_item)
    }

    /// Adds the display items necessary to paint the borders of this fragment to a display list if
    /// necessary.
    fn build_display_list_for_borders_if_applicable(&self,
                                                    style: &ComputedValues,
                                                    list: &mut DisplayList,
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
        let border_display_item = box BorderDisplayItem {
            base: BaseDisplayItem::new(*abs_bounds, self.node, level, *clip_rect),
            border: border.to_physical(style.writing_mode),
            color: SideOffsets2D::new(top_color.to_gfx_color(),
                                      right_color.to_gfx_color(),
                                      bottom_color.to_gfx_color(),
                                      left_color.to_gfx_color()),
            style: SideOffsets2D::new(style.get_border().border_top_style,
                                      style.get_border().border_right_style,
                                      style.get_border().border_bottom_style,
                                      style.get_border().border_left_style)
        };

        list.push(BorderDisplayItemClass(border_display_item))
    }

    fn build_debug_borders_around_text_fragments(&self,
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
        let border_display_item = box BorderDisplayItem {
            base: BaseDisplayItem::new(absolute_fragment_bounds,
                                       self.node,
                                       ContentStackingLevel,
                                       *clip_rect),
            border: SideOffsets2D::new_all_same(Au::from_px(1)),
            color: SideOffsets2D::new_all_same(color::rgb(0, 0, 200)),
            style: SideOffsets2D::new_all_same(border_style::solid)
        };
        display_list.push(BorderDisplayItemClass(border_display_item));

        // Draw a rectangle representing the baselines.
        let ascent = text_fragment.run.ascent();
        let mut baseline = self.border_box.clone();
        baseline.start.b = baseline.start.b + ascent;
        baseline.size.block = Au(0);
        let mut baseline = baseline.to_physical(self.style.writing_mode, container_size);
        baseline.origin = baseline.origin + flow_origin;

        let line_display_item = box LineDisplayItem {
            base: BaseDisplayItem::new(baseline, self.node, ContentStackingLevel, *clip_rect),
            color: color::rgb(0, 200, 0),
            style: border_style::dashed,
        };
        display_list.push(LineDisplayItemClass(line_display_item));
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
        let border_display_item = box BorderDisplayItem {
            base: BaseDisplayItem::new(absolute_fragment_bounds,
                                       self.node,
                                       ContentStackingLevel,
                                       *clip_rect),
            border: SideOffsets2D::new_all_same(Au::from_px(1)),
            color: SideOffsets2D::new_all_same(color::rgb(0, 0, 200)),
            style: SideOffsets2D::new_all_same(border_style::solid)
        };
        display_list.push(BorderDisplayItemClass(border_display_item))
    }

    fn build_display_list(&mut self,
                          display_list: &mut DisplayList,
                          layout_context: &LayoutContext,
                          flow_origin: Point2D<Au>,
                          background_and_border_level: BackgroundAndBorderLevel,
                          clip_rect: &Rect<Au>) {
        // FIXME(#2795): Get the real container size
        let container_size = Size2D::zero();
        let rect_to_absolute = |writing_mode: WritingMode, logical_rect: LogicalRect<Au>| {
            let physical_rect = logical_rect.to_physical(writing_mode, container_size);
            Rect(physical_rect.origin + flow_origin, physical_rect.size)
        };
        // Fragment position wrt to the owning flow.
        let absolute_fragment_bounds = rect_to_absolute(self.style.writing_mode, self.border_box);
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

        debug!("Fragment::build_display_list: intersected. Adding display item...");

        if self.is_primary_fragment() {
            let level =
                StackingLevel::from_background_and_border_level(background_and_border_level);

            // Add a pseudo-display item for content box queries. This is a very bogus thing to do.
            let base_display_item = box BaseDisplayItem::new(absolute_fragment_bounds,
                                                             self.node,
                                                             level,
                                                             *clip_rect);
            display_list.push(PseudoDisplayItemClass(base_display_item));

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
                            clip_rect);
                    }
                }
                None => {}
            }
            match self.specific {
                ScannedTextFragment(_) => {},
                _ => {
                    self.build_display_list_for_background_if_applicable(
                        &*self.style,
                        display_list,
                        layout_context,
                        level,
                        &absolute_fragment_bounds,
                        clip_rect);
                }
            }

            // Add a border, if applicable.
            //
            // TODO: Outlines.
            match self.inline_context {
                Some(ref inline_context) => {
                    for style in inline_context.styles.iter().rev() {
                        self.build_display_list_for_borders_if_applicable(
                            &**style,
                            display_list,
                            &absolute_fragment_bounds,
                            level,
                            clip_rect);
                    }
                }
                None => {}
            }
            match self.specific {
                ScannedTextFragment(_) => {},
                _ => {
                    self.build_display_list_for_borders_if_applicable(
                        &*self.style,
                        display_list,
                        &absolute_fragment_bounds,
                        level,
                        clip_rect);
                }
            }
        }

        let content_box = self.content_box();
        let absolute_content_box = rect_to_absolute(self.style.writing_mode, content_box);

        // Create special per-fragment-type display items.
        match self.specific {
            UnscannedTextFragment(_) => fail!("Shouldn't see unscanned fragments here."),
            TableColumnFragment(_) => fail!("Shouldn't see table column fragments here."),
            ScannedTextFragment(ref text_fragment) => {
                // Create the text display item.
                let orientation = if self.style.writing_mode.is_vertical() {
                    if self.style.writing_mode.is_sideways_left() {
                        SidewaysLeft
                    } else {
                        SidewaysRight
                    }
                } else {
                    Upright
                };

                let metrics = &text_fragment.run.font_metrics;
                let baseline_origin ={
                    let mut tmp = content_box.start;
                    tmp.b = tmp.b + metrics.ascent;
                    tmp.to_physical(self.style.writing_mode, container_size) + flow_origin
                };

                let text_display_item = box TextDisplayItem {
                    base: BaseDisplayItem::new(absolute_content_box,
                                               self.node,
                                               ContentStackingLevel,
                                               *clip_rect),
                    text_run: text_fragment.run.clone(),
                    range: text_fragment.range,
                    text_color: self.style().get_color().color.to_gfx_color(),
                    orientation: orientation,
                    baseline_origin: baseline_origin,
                };
                display_list.push(TextDisplayItemClass(text_display_item));

                // Create display items for text decoration
                {
                    let line = |maybe_color: Option<RGBA>, rect: || -> LogicalRect<Au>| {
                        match maybe_color {
                            None => {},
                            Some(color) => {
                                display_list.push(SolidColorDisplayItemClass(
                                                     box SolidColorDisplayItem {
                                                        base: BaseDisplayItem::new(
                                                                  rect_to_absolute(
                                                                      self.style.writing_mode,
                                                                      rect()),
                                                               self.node,
                                                               ContentStackingLevel,
                                                               *clip_rect),
                                                        color: color.to_gfx_color(),
                                                     }));
                            }
                        }
                    };

                    let text_decorations =
                        self.style().get_inheritedtext()._servo_text_decorations_in_effect;
                    line(text_decorations.underline, || {
                        let mut rect = content_box.clone();
                        rect.start.b = rect.start.b + metrics.ascent - metrics.underline_offset;
                        rect.size.block = metrics.underline_size;
                        rect
                    });

                    line(text_decorations.overline, || {
                        let mut rect = content_box.clone();
                        rect.size.block = metrics.underline_size;
                        rect
                    });

                    line(text_decorations.line_through, || {
                        let mut rect = content_box.clone();
                        rect.start.b = rect.start.b + metrics.ascent - metrics.strikeout_offset;
                        rect.size.block = metrics.strikeout_size;
                        rect
                    });
                }

                if opts::get().show_debug_fragment_borders {
                    self.build_debug_borders_around_text_fragments(display_list,
                                                                   flow_origin,
                                                                   text_fragment,
                                                                   clip_rect);
                }
            }
            GenericFragment | IframeFragment(..) | TableFragment | TableCellFragment |
            TableRowFragment | TableWrapperFragment | InlineBlockFragment(_) | InputFragment |
            InlineAbsoluteHypotheticalFragment(_) => {
                if opts::get().show_debug_fragment_borders {
                    self.build_debug_borders_around_fragment(display_list,
                                                             flow_origin,
                                                             clip_rect);
                }
            }
            ImageFragment(ref mut image_fragment) => {
                let image_ref = &mut image_fragment.image;
                match image_ref.get_image(self.node.to_untrusted_node_address()) {
                    Some(image) => {
                        debug!("(building display list) building image fragment");

                        // Place the image into the display list.
                        let image_display_item = box ImageDisplayItem {
                            base: BaseDisplayItem::new(absolute_content_box,
                                                       self.node,
                                                       ContentStackingLevel,
                                                       *clip_rect),
                            image: image.clone(),
                            stretch_size: absolute_content_box.size,
                        };

                        display_list.push(ImageDisplayItemClass(image_display_item))
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
           self.build_debug_borders_around_fragment(display_list, flow_origin, clip_rect)
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
            IframeFragment(ref iframe_fragment) => {
                self.finalize_position_and_size_of_iframe(iframe_fragment,
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

        debug!("finalizing position and size of iframe for {:?},{:?}",
               iframe_fragment.pipeline_id,
               iframe_fragment.subpage_id);
        let ConstellationChan(ref chan) = layout_context.shared.constellation_chan;
        chan.send(FrameRectMsg(iframe_fragment.pipeline_id,
                               iframe_fragment.subpage_id,
                               iframe_rect));
    }

    fn clip_rect_for_children(&self, current_clip_rect: Rect<Au>, flow_origin: Point2D<Au>)
                              -> Rect<Au> {
        // Don't clip if we're text.
        match self.specific {
            ScannedTextFragment(_) => return current_clip_rect,
            _ => {}
        }

        // Only clip if `overflow` tells us to.
        match self.style.get_box().overflow {
            overflow::hidden | overflow::auto | overflow::scroll => {}
            _ => return current_clip_rect,
        }

        // Create a new clip rect.
        //
        // FIXME(#2795): Get the real container size.
        let physical_rect = self.border_box.to_physical(self.style.writing_mode, Size2D::zero());
        current_clip_rect.intersection(&Rect(physical_rect.origin + flow_origin,
                                             physical_rect.size)).unwrap_or(ZERO_RECT)
    }
}

pub trait BlockFlowDisplayListBuilding {
    fn build_display_list_for_block(&mut self,
                                    layout_context: &LayoutContext,
                                    background_border_level: BackgroundAndBorderLevel);
    fn build_display_list_for_absolutely_positioned_block(&mut self,
                                                          layout_context: &LayoutContext);
    fn build_display_list_for_floating_block(&mut self, layout_context: &LayoutContext);
}

impl BlockFlowDisplayListBuilding for BlockFlow {
    fn build_display_list_for_block(&mut self,
                                    layout_context: &LayoutContext,
                                    background_border_level: BackgroundAndBorderLevel) {
        let relative_offset =
            self.fragment.relative_position(&self.base
                                                 .absolute_position_info
                                                 .relative_containing_block_size);

        // Add the box that starts the block context.
        let mut display_list = DisplayList::new();
        self.fragment.build_display_list(&mut display_list,
                                         layout_context,
                                         self.base.abs_position.add_size(
                                             &relative_offset.to_physical(self.base.writing_mode)),
                                         background_border_level,
                                         &self.base.clip_rect);

        let mut child_layers = DList::new();
        for kid in self.base.child_iter() {
            if kid.is_absolutely_positioned() {
                // All absolute flows will be handled by their containing block.
                continue
            }

            display_list.push_all_move(mem::replace(&mut flow::mut_base(kid).display_list,
                                                    DisplayList::new()));
            child_layers.append(mem::replace(&mut flow::mut_base(kid).layers, DList::new()))
        }

        // Process absolute descendant links.
        for abs_descendant_link in self.base.abs_descendants.iter() {
            // TODO(pradeep): Send in our absolute position directly.
            display_list.push_all_move(mem::replace(
                    &mut flow::mut_base(abs_descendant_link).display_list,
                    DisplayList::new()));
            child_layers.append(mem::replace(&mut flow::mut_base(abs_descendant_link).layers,
                                             DList::new()));
        }

        self.base.display_list = display_list;
        self.base.layers = child_layers
    }

    fn build_display_list_for_absolutely_positioned_block(&mut self,
                                                          layout_context: &LayoutContext) {
        self.build_display_list_for_block(layout_context, RootOfStackingContextLevel);

        if !self.base.absolute_position_info.layers_needed_for_positioned_flows &&
                !self.base.flags.needs_layer() {
            // We didn't need a layer.
            let z_index = self.fragment.style().get_box().z_index.number_or_zero();
            let level = PositionedDescendantStackingLevel(z_index);
            self.base.display_list = mem::replace(&mut self.base.display_list,
                                                  DisplayList::new()).flatten(level);
            return
        }

        // If we got here, then we need a new layer.
        let layer_rect = self.base.position.union(&self.base.overflow);
        let size = Size2D(layer_rect.size.inline.to_nearest_px() as uint,
                          layer_rect.size.block.to_nearest_px() as uint);
        let origin = Point2D(self.base.abs_position.x.to_nearest_px() as uint,
                             self.base.abs_position.y.to_nearest_px() as uint);

        let scroll_policy = if self.is_fixed() {
            FixedPosition
        } else {
            Scrollable
        };
        let display_list = mem::replace(&mut self.base.display_list, DisplayList::new());
        let new_layer = RenderLayer {
            id: self.layer_id(0),
            display_list: Arc::new(display_list.flatten(ContentStackingLevel)),
            position: Rect(origin, size),
            background_color: color::rgba(1.0, 1.0, 1.0, 0.0),
            scroll_policy: scroll_policy,
        };
        self.base.layers.push(new_layer)
    }

    fn build_display_list_for_floating_block(&mut self, layout_context: &LayoutContext) {
        self.build_display_list_for_block(layout_context, RootOfStackingContextLevel);
        self.base.display_list = mem::replace(&mut self.base.display_list,
                                              DisplayList::new()).flatten(FloatStackingLevel)
    }
}
