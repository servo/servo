/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use css::node_style::StyledNode;
use layout::box_::{Box, CannotSplit, GenericBox, IframeBox, ImageBox, InlineInfo, ScannedTextBox};
use layout::box_::{SplitDidFit, SplitDidNotFit, TableBox, TableCellBox, TableColumnBox};
use layout::box_::{TableRowBox, TableWrapperBox, UnscannedTextBox};
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, DisplayListBuildingInfo};
use layout::floats::{FloatLeft, Floats, PlacementInfo};
use layout::flow::{BaseFlow, FlowClass, Flow, InlineFlowClass};
use layout::flow;
use layout::model::IntrinsicWidths;
use layout::util::ElementMapping;
use layout::wrapper::ThreadSafeLayoutNode;

use collections::{Deque, RingBuf};
use geom::{Point2D, Rect, Size2D};
use gfx::display_list::{ContentLevel, StackingContext};
use servo_util::geometry::Au;
use servo_util::geometry;
use servo_util::range::Range;
use std::mem;
use std::u16;
use style::computed_values::{text_align, vertical_align, white_space};

/// Lineboxes are represented as offsets into the child list, rather than
/// as an object that "owns" boxes. Choosing a different set of line
/// breaks requires a new list of offsets, and possibly some splitting and
/// merging of TextBoxes.
///
/// A similar list will keep track of the mapping between CSS boxes and
/// the corresponding boxes in the inline flow.
///
/// After line breaks are determined, render boxes in the inline flow may
/// overlap visually. For example, in the case of nested inline CSS boxes,
/// outer inlines must be at least as large as the inner inlines, for
/// purposes of drawing noninherited things like backgrounds, borders,
/// outlines.
///
/// N.B. roc has an alternative design where the list instead consists of
/// things like "start outer box, text, start inner box, text, end inner
/// box, text, end outer box, text". This seems a little complicated to
/// serve as the starting point, but the current design doesn't make it
/// hard to try out that alternative.
///
/// Line boxes also contain some metadata used during line breaking. The
/// green zone is the area that the line can expand to before it collides
/// with a float or a horizontal wall of the containing block. The top
/// left corner of the green zone is the same as that of the line, but
/// the green zone can be taller and wider than the line itself.
pub struct LineBox {
    pub range: Range,
    pub bounds: Rect<Au>,
    pub green_zone: Size2D<Au>
}

struct LineboxScanner {
    pub floats: Floats,
    pub new_boxes: ~[Box],
    pub work_list: RingBuf<Box>,
    pub pending_line: LineBox,
    pub lines: ~[LineBox],
    pub cur_y: Au,
}

impl LineboxScanner {
    pub fn new(float_ctx: Floats) -> LineboxScanner {
        LineboxScanner {
            floats: float_ctx,
            new_boxes: ~[],
            work_list: RingBuf::new(),
            pending_line: LineBox {
                range: Range::empty(),
                bounds: Rect(Point2D(Au::new(0), Au::new(0)), Size2D(Au::new(0), Au::new(0))),
                green_zone: Size2D(Au::new(0), Au::new(0))
            },
            lines: ~[],
            cur_y: Au::new(0)
        }
    }

    pub fn floats(&mut self) -> Floats {
        self.floats.clone()
    }

    fn reset_scanner(&mut self) {
        debug!("Resetting line box scanner's state for flow.");
        self.lines = ~[];
        self.new_boxes = ~[];
        self.cur_y = Au::new(0);
        self.reset_linebox();
    }

    fn reset_linebox(&mut self) {
        self.pending_line.range.reset(0,0);
        self.pending_line.bounds = Rect(Point2D(Au::new(0), self.cur_y), Size2D(Au::new(0), Au::new(0)));
        self.pending_line.green_zone = Size2D(Au::new(0), Au::new(0))
    }

    pub fn scan_for_lines(&mut self, flow: &mut InlineFlow) {
        self.reset_scanner();

        loop {
            // acquire the next box to lay out from work list or box list
            let cur_box = if self.work_list.is_empty() {
                if flow.boxes.is_empty() {
                    break;
                }
                let box_ = flow.boxes.remove(0).unwrap(); // FIXME: use a linkedlist
                debug!("LineboxScanner: Working with box from box list: b{}", box_.debug_id());
                box_
            } else {
                let box_ = self.work_list.pop_front().unwrap();
                debug!("LineboxScanner: Working with box from work list: b{}", box_.debug_id());
                box_
            };

            let box_was_appended = match cur_box.white_space() {
                white_space::normal => self.try_append_to_line(cur_box, flow),
                white_space::pre => self.try_append_to_line_by_new_line(cur_box),
            };

            if !box_was_appended {
                debug!("LineboxScanner: Box wasn't appended, because line {:u} was full.",
                        self.lines.len());
                self.flush_current_line();
            } else {
                debug!("LineboxScanner: appended a box to line {:u}", self.lines.len());
            }
        }

        if self.pending_line.range.length() > 0 {
            debug!("LineboxScanner: Partially full linebox {:u} left at end of scanning.",
                    self.lines.len());
            self.flush_current_line();
        }

        flow.elems.repair_for_box_changes(flow.boxes, self.new_boxes);

        self.swap_out_results(flow);
    }

    fn swap_out_results(&mut self, flow: &mut InlineFlow) {
        debug!("LineboxScanner: Propagating scanned lines[n={:u}] to inline flow",
               self.lines.len());

        mem::swap(&mut flow.boxes, &mut self.new_boxes);
        mem::swap(&mut flow.lines, &mut self.lines);
    }

    fn flush_current_line(&mut self) {
        debug!("LineboxScanner: Flushing line {:u}: {:?}",
               self.lines.len(), self.pending_line);

        // clear line and add line mapping
        debug!("LineboxScanner: Saving information for flushed line {:u}.", self.lines.len());
        self.lines.push(self.pending_line);
        self.cur_y = self.pending_line.bounds.origin.y + self.pending_line.bounds.size.height;
        self.reset_linebox();
    }

    // FIXME(eatkinson): this assumes that the tallest box in the line determines the line height
    // This might not be the case with some weird text fonts.
    fn new_height_for_line(&self, new_box: &Box) -> Au {
        let box_height = new_box.content_height();
        if box_height > self.pending_line.bounds.size.height {
            box_height
        } else {
            self.pending_line.bounds.size.height
        }
    }

    /// Computes the position of a line that has only the provided box. Returns the bounding rect
    /// of the line's green zone (whose origin coincides with the line's origin) and the actual
    /// width of the first box after splitting.
    fn initial_line_placement(&self, first_box: &Box, ceiling: Au, flow: &mut InlineFlow)
                              -> (Rect<Au>, Au) {
        debug!("LineboxScanner: Trying to place first box of line {}", self.lines.len());

        let first_box_size = first_box.border_box.borrow().size;
        let splittable = first_box.can_split();
        debug!("LineboxScanner: box size: {}, splittable: {}", first_box_size, splittable);
        let line_is_empty: bool = self.pending_line.range.length() == 0;

        // Initally, pretend a splittable box has 0 width.
        // We will move it later if it has nonzero width
        // and that causes problems.
        let placement_width = if splittable {
            Au::new(0)
        } else {
            first_box_size.width
        };

        let mut info = PlacementInfo {
            size: Size2D(placement_width, first_box_size.height),
            ceiling: ceiling,
            max_width: flow.base.position.size.width,
            kind: FloatLeft,
        };

        let line_bounds = self.floats.place_between_floats(&info);

        debug!("LineboxScanner: found position for line: {} using placement_info: {:?}",
               line_bounds,
               info);

        // Simple case: if the box fits, then we can stop here
        if line_bounds.size.width > first_box_size.width {
            debug!("LineboxScanner: case=box fits");
            return (line_bounds, first_box_size.width);
        }

        // If not, but we can't split the box, then we'll place
        // the line here and it will overflow.
        if !splittable {
            debug!("LineboxScanner: case=line doesn't fit, but is unsplittable");
            return (line_bounds, first_box_size.width);
        }

        // Otherwise, try and split the box
        // FIXME(eatkinson): calling split_to_width here seems excessive and expensive.
        // We should find a better abstraction or merge it with the call in
        // try_append_to_line.
        match first_box.split_to_width(line_bounds.size.width, line_is_empty) {
            CannotSplit => {
                error!("LineboxScanner: Tried to split unsplittable render box! {:s}",
                        first_box.debug_str());
                return (line_bounds, first_box_size.width);
            }
            SplitDidFit(left, right) => {

                debug!("LineboxScanner: case=box split and fit");
                let actual_box_width = match (left, right) {
                    (Some(l_box), Some(_))  => l_box.border_box.borrow().size.width,
                    (Some(l_box), None)     => l_box.border_box.borrow().size.width,
                    (None, Some(r_box))     => r_box.border_box.borrow().size.width,
                    (None, None)            => fail!("This case makes no sense.")
                };
                return (line_bounds, actual_box_width);
            }
            SplitDidNotFit(left, right) => {
                // The split didn't fit, but we might be able to
                // push it down past floats.


                debug!("LineboxScanner: case=box split and fit didn't fit; trying to push it down");
                let actual_box_width = match (left, right) {
                    (Some(l_box), Some(_))  => l_box.border_box.borrow().size.width,
                    (Some(l_box), None)     => l_box.border_box.borrow().size.width,
                    (None, Some(r_box))     => r_box.border_box.borrow().size.width,
                    (None, None)            => fail!("This case makes no sense.")
                };

                info.size.width = actual_box_width;
                let new_bounds = self.floats.place_between_floats(&info);

                debug!("LineboxScanner: case=new line position: {}", new_bounds);
                return (new_bounds, actual_box_width);
            }
        }

    }

    /// Performs float collision avoidance. This is called when adding a box is going to increase
    /// the height, and because of that we will collide with some floats.
    ///
    /// We have two options here:
    /// 1) Move the entire line so that it doesn't collide any more.
    /// 2) Break the line and put the new box on the next line.
    ///
    /// The problem with option 1 is that we might move the line and then wind up breaking anyway,
    /// which violates the standard.
    /// But option 2 is going to look weird sometimes.
    ///
    /// So we'll try to move the line whenever we can, but break if we have to.
    ///
    /// Returns false if and only if we should break the line.
    fn avoid_floats(&mut self,
                    in_box: Box,
                    flow: &mut InlineFlow,
                    new_height: Au,
                    line_is_empty: bool)
                    -> bool {
        debug!("LineboxScanner: entering float collision avoider!");

        // First predict where the next line is going to be.
        let this_line_y = self.pending_line.bounds.origin.y;
        let (next_line, first_box_width) = self.initial_line_placement(&in_box, this_line_y, flow);
        let next_green_zone = next_line.size;

        let new_width = self.pending_line.bounds.size.width + first_box_width;

        // Now, see if everything can fit at the new location.
        if next_green_zone.width >= new_width && next_green_zone.height >= new_height {
            debug!("LineboxScanner: case=adding box collides vertically with floats: moving line");

            self.pending_line.bounds.origin = next_line.origin;
            self.pending_line.green_zone = next_green_zone;

            assert!(!line_is_empty, "Non-terminating line breaking");
            self.work_list.push_front(in_box);
            return true
        }

        debug!("LineboxScanner: case=adding box collides vertically with floats: breaking line");
        self.work_list.push_front(in_box);
        false
    }

    fn try_append_to_line_by_new_line(&mut self, in_box: Box) -> bool {
        if in_box.new_line_pos.len() == 0 {
            // In case of box does not include new-line character
            self.push_box_to_line(in_box);
            true
        } else {
            // In case of box includes new-line character
            match in_box.split_by_new_line() {
                SplitDidFit(left, right) => {
                    match (left, right) {
                        (Some(left_box), Some(right_box)) => {
                            self.push_box_to_line(left_box);
                            self.work_list.push_front(right_box);
                        }
                        (Some(left_box), None) => {
                            self.push_box_to_line(left_box);
                        }
                        _ => error!("LineboxScanner: This split case makes no sense!"),
                    }
                }
                _ => {}
            }
            false
        }
    }

    /// Tries to append the given box to the line, splitting it if necessary. Returns false only if
    /// we should break the line.
    fn try_append_to_line(&mut self, in_box: Box, flow: &mut InlineFlow) -> bool {
        let line_is_empty = self.pending_line.range.length() == 0;
        if line_is_empty {
            let (line_bounds, _) = self.initial_line_placement(&in_box, self.cur_y, flow);
            self.pending_line.bounds.origin = line_bounds.origin;
            self.pending_line.green_zone = line_bounds.size;
        }

        debug!("LineboxScanner: Trying to append box to line {:u} (box size: {}, green zone: \
                {}): {:s}",
               self.lines.len(),
               in_box.border_box.borrow().size,
               self.pending_line.green_zone,
               in_box.debug_str());

        let green_zone = self.pending_line.green_zone;

        // NB: At this point, if `green_zone.width < self.pending_line.bounds.size.width` or
        // `green_zone.height < self.pending_line.bounds.size.height`, then we committed a line
        // that overlaps with floats.

        let new_height = self.new_height_for_line(&in_box);
        if new_height > green_zone.height {
            // Uh-oh. Float collision imminent. Enter the float collision avoider
            return self.avoid_floats(in_box, flow, new_height, line_is_empty)
        }

        // If we're not going to overflow the green zone vertically, we might still do so
        // horizontally. We'll try to place the whole box on this line and break somewhere if it
        // doesn't fit.

        let new_width = self.pending_line.bounds.size.width + in_box.border_box.borrow().size.width;
        if new_width <= green_zone.width {
            debug!("LineboxScanner: case=box fits without splitting");
            self.push_box_to_line(in_box);
            return true
        }

        if !in_box.can_split() {
            // TODO(eatkinson, issue #224): Signal that horizontal overflow happened?
            if line_is_empty {
                debug!("LineboxScanner: case=box can't split and line {:u} is empty, so \
                        overflowing.",
                        self.lines.len());
                self.push_box_to_line(in_box);
                return true
            }
        }

        let available_width = green_zone.width - self.pending_line.bounds.size.width;
        let split = in_box.split_to_width(available_width, line_is_empty);
        let (left, right) = match (split, line_is_empty) {
            (CannotSplit, _) => {
                debug!("LineboxScanner: Tried to split unsplittable render box! {:s}",
                        in_box.debug_str());
                self.work_list.push_front(in_box);
                return false
            }
            (SplitDidNotFit(_, _), false) => {
                debug!("LineboxScanner: case=split box didn't fit, not appending and deferring \
                        original box.");
                self.work_list.push_front(in_box);
                return false
            }
            (SplitDidFit(left, right), _) => {
                debug!("LineboxScanner: case=split box did fit; deferring remainder box.");
                (left, right)
                // Fall through to push boxes to the line.
            }
            (SplitDidNotFit(left, right), true) => {
                // TODO(eatkinson, issue #224): Signal that horizontal overflow happened?
                debug!("LineboxScanner: case=split box didn't fit and line {:u} is empty, so \
                        overflowing and deferring remainder box.",
                        self.lines.len());
                (left, right)
                // Fall though to push boxes to the line.
            }
        };

        match (left, right) {
            (Some(left_box), Some(right_box)) => {
                self.push_box_to_line(left_box);
                self.work_list.push_front(right_box);
            }
            (Some(left_box), None) => self.push_box_to_line(left_box),
            (None, Some(right_box)) => self.push_box_to_line(right_box),
            (None, None) => error!("LineboxScanner: This split case makes no sense!"),
        }

        true
    }

    // An unconditional push
    fn push_box_to_line(&mut self, box_: Box) {
        debug!("LineboxScanner: Pushing box {} to line {:u}", box_.debug_id(), self.lines.len());

        if self.pending_line.range.length() == 0 {
            assert!(self.new_boxes.len() <= (u16::MAX as uint));
            self.pending_line.range.reset(self.new_boxes.len(), 0);
        }
        self.pending_line.range.extend_by(1);
        self.pending_line.bounds.size.width = self.pending_line.bounds.size.width +
            box_.border_box.borrow().size.width;
        self.pending_line.bounds.size.height = Au::max(self.pending_line.bounds.size.height,
                                                       box_.border_box.borrow().size.height);
        self.new_boxes.push(box_);
    }
}

pub struct InlineFlow {
    /// Data common to all flows.
    pub base: BaseFlow,

    /// A vector of all inline render boxes. Several boxes may correspond to one node/element.
    pub boxes: ~[Box],

    // vec of ranges into boxes that represents line positions.
    // these ranges are disjoint, and are the result of inline layout.
    // also some metadata used for positioning lines
    pub lines: ~[LineBox],

    // vec of ranges into boxes that represent elements. These ranges
    // must be well-nested, and are only related to the content of
    // boxes (not lines). Ranges are only kept for non-leaf elements.
    pub elems: ElementMapping,
}

impl InlineFlow {
    pub fn from_boxes(node: ThreadSafeLayoutNode, boxes: ~[Box]) -> InlineFlow {
        InlineFlow {
            base: BaseFlow::new(node),
            boxes: boxes,
            lines: ~[],
            elems: ElementMapping::new(),
        }
    }

    pub fn teardown(&mut self) {
        for box_ in self.boxes.iter() {
            box_.teardown();
        }
        self.boxes = ~[];
    }

    pub fn build_display_list_inline(&self,
                                     stacking_context: &mut StackingContext,
                                     builder: &DisplayListBuilder,
                                     info: &DisplayListBuildingInfo) {
        let abs_rect = Rect(self.base.abs_position, self.base.position.size);
        if !abs_rect.intersects(&builder.dirty) {
            return
        }

        // TODO(#228): Once we form line boxes and have their cached bounds, we can be smarter and
        // not recurse on a line if nothing in it can intersect the dirty region.
        debug!("Flow: building display list for {:u} inline boxes", self.boxes.len());

        for box_ in self.boxes.iter() {
            let rel_offset = box_.relative_position(&info.relative_containing_block_size);
            box_.build_display_list(stacking_context,
                                    builder,
                                    info,
                                    self.base.abs_position + rel_offset,
                                    (&*self) as &Flow,
                                    ContentLevel);
        }

        // TODO(#225): Should `inline-block` elements have flows as children of the inline flow or
        // should the flow be nested inside the box somehow?

        // For now, don't traverse the subtree rooted here.
    }

    /// Returns the relative offset from the baseline for this box, taking into account the value
    /// of the CSS `vertical-align` property.
    ///
    /// The extra boolean is set if and only if `biggest_top` and/or `biggest_bottom` were updated.
    /// That is, if the box has a `top` or `bottom` value, true is returned.
    fn relative_offset_from_baseline(cur_box: &Box,
                                     ascent: Au,
                                     parent_text_top: Au,
                                     parent_text_bottom: Au,
                                     top_from_base: &mut Au,
                                     bottom_from_base: &mut Au,
                                     biggest_top: &mut Au,
                                     biggest_bottom: &mut Au)
                                     -> (Au, bool) {
        match cur_box.vertical_align() {
            vertical_align::baseline => (-ascent, false),
            vertical_align::middle => {
                // TODO: x-height value should be used from font info.
                let xheight = Au::new(0);
                (-(xheight + cur_box.content_height()).scale_by(0.5), false)
            },
            vertical_align::sub => {
                // TODO: The proper position for subscripts should be used.
                // Lower the baseline to the proper position for subscripts
                let sub_offset = Au::new(0);
                (sub_offset - ascent, false)
            },
            vertical_align::super_ => {
                // TODO: The proper position for superscripts should be used.
                // Raise the baseline to the proper position for superscripts
                let super_offset = Au::new(0);
                (-super_offset - ascent, false)
            },
            vertical_align::text_top => {
                let box_height = *top_from_base + *bottom_from_base;
                let prev_bottom_from_base = *bottom_from_base;
                *top_from_base = parent_text_top;
                *bottom_from_base = box_height - *top_from_base;
                (*bottom_from_base - prev_bottom_from_base - ascent, false)
            },
            vertical_align::text_bottom => {
                let box_height = *top_from_base + *bottom_from_base;
                let prev_bottom_from_base = *bottom_from_base;
                *bottom_from_base = parent_text_bottom;
                *top_from_base = box_height - *bottom_from_base;
                (*bottom_from_base - prev_bottom_from_base - ascent, false)
            },
            vertical_align::top => {
                if *biggest_top < (*top_from_base + *bottom_from_base) {
                    *biggest_top = *top_from_base + *bottom_from_base;
                }
                let offset_top = *top_from_base - ascent;
                (offset_top, true)
            },
            vertical_align::bottom => {
                if *biggest_bottom < (*top_from_base + *bottom_from_base) {
                    *biggest_bottom = *top_from_base + *bottom_from_base;
                }
                let offset_bottom = -(*bottom_from_base + ascent);
                (offset_bottom, true)
            },
            vertical_align::Length(length) => (-(length + ascent), false),
            vertical_align::Percentage(p) => {
                let pt_size = cur_box.font_style().pt_size;
                let line_height = cur_box.calculate_line_height(Au::from_pt(pt_size));
                let percent_offset = line_height.scale_by(p);
                (-(percent_offset + ascent), false)
            }
        }
    }

    /// Sets box X positions based on alignment for one line.
    fn set_horizontal_box_positions(boxes: &[Box], line: &LineBox, linebox_align: text_align::T) {
        // Figure out how much width we have.
        let slack_width = Au::max(Au(0), line.green_zone.width - line.bounds.size.width);

        // Set the box x positions based on that alignment.
        let mut offset_x = line.bounds.origin.x;
        offset_x = offset_x + match linebox_align {
            // So sorry, but justified text is more complicated than shuffling linebox
            // coordinates.
            //
            // TODO(burg, issue #213): Implement `text-align: justify`.
            text_align::left | text_align::justify => Au(0),
            text_align::center => slack_width.scale_by(0.5),
            text_align::right => slack_width,
        };

        for i in line.range.eachi() {
            let box_ = &boxes[i];
            let mut border_box = box_.border_box.borrow_mut();
            let size = border_box.size;
            *border_box = Rect(Point2D(offset_x, border_box.origin.y), size);
            offset_x = offset_x + size.width;
        }
    }
}

impl Flow for InlineFlow {
    fn class(&self) -> FlowClass {
        InlineFlowClass
    }

    fn as_immutable_inline<'a>(&'a self) -> &'a InlineFlow {
        self
    }

    fn as_inline<'a>(&'a mut self) -> &'a mut InlineFlow {
        self
    }

    fn bubble_widths(&mut self, _: &mut LayoutContext) {
        let mut num_floats = 0;

        for kid in self.base.child_iter() {
            let child_base = flow::mut_base(kid);
            num_floats += child_base.num_floats;
            child_base.floats = Floats::new();
        }

        let mut intrinsic_widths = IntrinsicWidths::new();
        for box_ in self.boxes.iter() {
            debug!("Flow: measuring {:s}", box_.debug_str());
            box_.compute_borders(box_.style());

            let box_intrinsic_widths = box_.intrinsic_widths();
            intrinsic_widths.minimum_width = geometry::max(intrinsic_widths.minimum_width,
                                                           box_intrinsic_widths.minimum_width);
            intrinsic_widths.preferred_width = geometry::max(intrinsic_widths.preferred_width,
                                                             box_intrinsic_widths.preferred_width);
        }

        self.base.intrinsic_widths = intrinsic_widths;
        self.base.num_floats = num_floats;
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent context.
    fn assign_widths(&mut self, _: &mut LayoutContext) {
        // Initialize content box widths if they haven't been initialized already.
        //
        // TODO: Combine this with `LineboxScanner`'s walk in the box list, or put this into `Box`.

        debug!("InlineFlow::assign_widths: floats in: {:?}", self.base.floats);

        {
            let this = &mut *self;
            for box_ in this.boxes.iter() {
                box_.assign_replaced_width_if_necessary(self.base.position.size.width);
            }
        }

        assert!(self.base.children.len() == 0,
                "InlineFlow: should not have children flows in the current layout implementation.");

        // There are no child contexts, so stop here.

        // TODO(Issue #225): once there are 'inline-block' elements, this won't be
        // true.  In that case, set the InlineBlockBox's width to the
        // shrink-to-fit width, perform inline flow, and set the block
        // flow context's width as the assigned width of the
        // 'inline-block' box that created this flow before recursing.
    }

    fn assign_height_inorder(&mut self, ctx: &mut LayoutContext) {
        for kid in self.base.child_iter() {
            kid.assign_height_inorder(ctx);
        }
        self.assign_height(ctx);
    }

    /// Calculate and set the height of this Flow.
    ///
    /// CSS Section 10.6.1
    fn assign_height(&mut self, _: &mut LayoutContext) {
        debug!("assign_height_inline: assigning height for flow");

        // Divide the boxes into lines.
        //
        // TODO(#226): Get the CSS `line-height` property from the containing block's style to
        // determine minimum linebox height.
        //
        // TODO(#226): Get the CSS `line-height` property from each non-replaced inline element to
        // determine its height for computing linebox height.
        //
        // TODO(pcwalton): Cache the linebox scanner?
        debug!("assign_height_inline: floats in: {:?}", self.base.floats);

        // assign height for inline boxes
        for box_ in self.boxes.iter() {
            box_.assign_replaced_height_if_necessary();
        }
        let scanner_floats = self.base.floats.clone();
        let mut scanner = LineboxScanner::new(scanner_floats);

        // Access the linebox scanner.
        scanner.scan_for_lines(self);
        let mut line_height_offset = Au::new(0);

        // All lines use text alignment of the flow.
        let text_align = self.base.flags_info.flags.text_align();

        // Now, go through each line and lay out the boxes inside.
        for line in self.lines.mut_iter() {
            // Lay out boxes horizontally.
            InlineFlow::set_horizontal_box_positions(self.boxes, line, text_align);

            // Set the top y position of the current linebox.
            // `line_height_offset` is updated at the end of the previous loop.
            line.bounds.origin.y = line.bounds.origin.y + line_height_offset;

            // Calculate the distance from baseline to the top and bottom of the linebox.
            let (mut topmost, mut bottommost) = (Au(0), Au(0));
            // Calculate the biggest height among boxes with 'top' and 'bottom' values
            // respectively.
            let (mut biggest_top, mut biggest_bottom) = (Au(0), Au(0));

            for box_i in line.range.eachi() {
                let cur_box = &self.boxes[box_i];
                let top = cur_box.noncontent_top();

                // FIXME(pcwalton): Move into `box.rs` like the rest of box-specific layout code?
                let (top_from_base, bottom_from_base, ascent) = match cur_box.specific {
                    ImageBox(_) => {
                        let mut height = cur_box.content_height();

                        // TODO: margin, border, padding's top and bottom should be calculated in
                        // advance, since baseline of image is bottom margin edge.
                        let bottom = cur_box.noncontent_bottom();
                        let noncontent_height = top + bottom;
                        height = height + noncontent_height;

                        let ascent = height + bottom;
                        (height, Au::new(0), ascent)
                    },
                    ScannedTextBox(ref text_box) => {
                        let range = &text_box.range;
                        let run = &text_box.run;

                        // Compute the height based on the line-height and font size
                        let text_bounds = run.metrics_for_range(range).bounding_box;
                        let em_size = text_bounds.size.height;
                        let line_height = cur_box.calculate_line_height(em_size);

                        // Find the top and bottom of the content area.
                        // Those are used in text-top and text-bottom value of 'vertical-align'
                        let text_ascent = text_box.run.font_metrics.ascent;

                        // Offset from the top of the box is 1/2 of the leading + ascent
                        let text_offset = text_ascent + (line_height - em_size).scale_by(0.5);
                        text_bounds.translate(&Point2D(cur_box.border_box.borrow().origin.x, Au(0)));

                        (text_offset, line_height - text_offset, text_ascent)
                    },
                    GenericBox | IframeBox(_) | TableBox | TableCellBox | TableRowBox |
                    TableWrapperBox => {
                        let height = cur_box.border_box.borrow().size.height;
                        (height, Au::new(0), height)
                    },
                    TableColumnBox(_) => fail!("Table column boxes do not have height"),
                    UnscannedTextBox(_) => {
                        fail!("Unscanned text boxes should have been scanned by now.")
                    }
                };

                let mut top_from_base = top_from_base;
                let mut bottom_from_base = bottom_from_base;

                // To calculate text-top and text-bottom value of 'vertical-align',
                //  we should find the top and bottom of the content area of parent box.
                // The content area is defined in:
                //      http://www.w3.org/TR/CSS2/visudet.html#inline-non-replaced
                //
                // TODO: We should extract em-box info from the font size of the parent and
                // calculate the distances from the baseline to the top and the bottom of the
                // parent's content area.

                // We should calculate the distance from baseline to the top of parent's content
                // area. But for now we assume it's the font size.
                //
                // The spec does not state which font to use. Previous versions of the code used
                // the parent's font; this code uses the current font.
                let parent_text_top = cur_box.style().Font.get().font_size;

                // We should calculate the distance from baseline to the bottom of the parent's
                // content area. But for now we assume it's zero.
                let parent_text_bottom = Au::new(0);

                // Calculate a relative offset from the baseline.
                //
                // The no-update flag decides whether `biggest_top` and `biggest_bottom` are
                // updated or not. That is, if the box has a `top` or `bottom` value,
                // `no_update_flag` becomes true.
                let (offset, no_update_flag) =
                    InlineFlow::relative_offset_from_baseline(cur_box,
                                                              ascent,
                                                              parent_text_top,
                                                              parent_text_bottom,
                                                              &mut top_from_base,
                                                              &mut bottom_from_base,
                                                              &mut biggest_top,
                                                              &mut biggest_bottom);

                // If the current box has 'top' or 'bottom' value, no_update_flag is true.
                // Otherwise, topmost and bottomost are updated.
                if !no_update_flag && top_from_base > topmost {
                    topmost = top_from_base;
                }
                if !no_update_flag && bottom_from_base > bottommost {
                    bottommost = bottom_from_base;
                }

                cur_box.border_box.borrow_mut().origin.y = line.bounds.origin.y + offset + top;
            }

            // Calculate the distance from baseline to the top of the biggest box with 'bottom'
            // value. Then, if necessary, update the topmost.
            let topmost_of_bottom = biggest_bottom - bottommost;
            if topmost_of_bottom > topmost {
                topmost = topmost_of_bottom;
            }

            // Calculate the distance from baseline to the bottom of the biggest box with 'top'
            // value. Then, if necessary, update the bottommost.
            let bottommost_of_top = biggest_top - topmost;
            if bottommost_of_top > bottommost {
                bottommost = bottommost_of_top;
            }

            // Now, the baseline offset from the top of linebox is set as topmost.
            let baseline_offset = topmost;

            // All boxes' y position is updated following the new baseline offset.
            for box_i in line.range.eachi() {
                let cur_box = &self.boxes[box_i];
                let adjust_offset = match cur_box.vertical_align() {
                    vertical_align::top => Au::new(0),
                    vertical_align::bottom => baseline_offset + bottommost,
                    _ => baseline_offset,
                };

                let mut border_box = cur_box.border_box.borrow_mut();
                border_box.origin.y = border_box.origin.y + adjust_offset;

                let mut info = cur_box.inline_info.borrow_mut();
                if info.is_none() {
                    *info = Some(InlineInfo::new());
                }
                match &mut *info {
                    &Some(ref mut info) => {
                        // TODO (ksh8281) compute vertical-align, line-height
                        info.baseline = line.bounds.origin.y + baseline_offset;
                    },
                    &None => {}
                }
            }

            // This is used to set the top y position of the next linebox in the next loop.
            line_height_offset = line_height_offset + topmost + bottommost -
                line.bounds.size.height;
            line.bounds.size.height = topmost + bottommost;
        } // End of `lines.each` loop.

        self.base.position.size.height =
            if self.lines.len() > 0 {
                self.lines.last().get_ref().bounds.origin.y + self.lines.last().get_ref().bounds.size.height
            } else {
                Au::new(0)
            };

        self.base.floats = scanner.floats();
        self.base.floats.translate(Point2D(Au::new(0), -self.base.position.size.height));
    }

    fn debug_str(&self) -> ~str {
        let val: ~[~str] = self.boxes.iter().map(|s| s.debug_str()).collect();
        let toprint = val.connect(",");
        format!("InlineFlow: {}", toprint)
    }
}
