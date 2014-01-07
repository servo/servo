/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use css::node_style::StyledNode;
use layout::box::{Box, CannotSplit, GenericBox, IframeBox, ImageBox, ScannedTextBox, SplitDidFit};
use layout::box::{SplitDidNotFit, UnscannedTextBox};
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{FlowClass, Flow, FlowData, InlineFlowClass};
use layout::flow;
use layout::float_context::FloatContext;
use layout::util::{ElementMapping};
use layout::float_context::{PlacementInfo, FloatLeft};

use extra::container::Deque;
use extra::ringbuf::RingBuf;
use geom::{Point2D, Rect, Size2D};
use gfx::display_list::DisplayList;
use servo_util::geometry::Au;
use servo_util::range::Range;
use std::cell::Cell;
use std::u16;
use std::util;
use style::computed_values::{text_align, vertical_align};

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
struct LineBox {
    range: Range,
    bounds: Rect<Au>,
    green_zone: Size2D<Au>
}

struct LineboxScanner {
    floats: FloatContext,
    new_boxes: ~[Box],
    work_list: RingBuf<Box>,
    pending_line: LineBox,
    lines: ~[LineBox],
    cur_y: Au,
}

impl LineboxScanner {
    pub fn new(float_ctx: FloatContext) -> LineboxScanner {
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

    fn reinitialize(&mut self, float_ctx: FloatContext) {
        self.floats = float_ctx;
        self.new_boxes.truncate(0);
        self.work_list.clear();
        self.pending_line.range = Range::empty();
        self.pending_line.bounds = Rect(Point2D(Au::new(0), Au::new(0)),
                                        Size2D(Au::new(0), Au::new(0)));
        self.pending_line.green_zone = Size2D(Au::new(0), Au::new(0));
        self.lines.truncate(0);
        self.cur_y = Au::new(0);
    }

    pub fn floats_out(&mut self) -> FloatContext {
        self.floats.clone()
    }

    fn reset_scanner(&mut self, flow: &mut InlineFlow) {
        debug!("Resetting line box scanner's state for flow f{:d}.", flow.base.id);
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
        self.reset_scanner(flow);

        loop {
            // acquire the next box to lay out from work list or box list
            let cur_box = if self.work_list.is_empty() {
                if flow.boxes.is_empty() {
                    break;
                }
                let box = flow.boxes.remove(0); // FIXME: use a linkedlist
                debug!("LineboxScanner: Working with box from box list: b{}", box.debug_id());
                box
            } else {
                let box = self.work_list.pop_front().unwrap();
                debug!("LineboxScanner: Working with box from work list: b{}", box.debug_id());
                box
            };

            let box_was_appended = self.try_append_to_line(cur_box, flow);
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
        debug!("LineboxScanner: Propagating scanned lines[n={:u}] to inline flow f{:d}",
               self.lines.len(),
               flow.base.id);

        util::swap(&mut flow.boxes, &mut self.new_boxes);
        util::swap(&mut flow.lines, &mut self.lines);
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
        let box_height = new_box.box_height();
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

        let first_box_size = first_box.position.get().size;
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
            width: placement_width,
            height: first_box_size.height,
            ceiling: ceiling,
            max_width: flow.base.position.size.width,
            f_type: FloatLeft
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
                    (Some(l_box), Some(_))  => l_box.position.get().size.width,
                    (Some(l_box), None)     => l_box.position.get().size.width,
                    (None, Some(r_box))     => r_box.position.get().size.width,
                    (None, None)            => fail!("This case makes no sense.")
                };
                return (line_bounds, actual_box_width);
            }
            SplitDidNotFit(left, right) => {
                // The split didn't fit, but we might be able to
                // push it down past floats.


                debug!("LineboxScanner: case=box split and fit didn't fit; trying to push it down");
                let actual_box_width = match (left, right) {
                    (Some(l_box), Some(_))  => l_box.position.get().size.width,
                    (Some(l_box), None)     => l_box.position.get().size.width,
                    (None, Some(r_box))     => r_box.position.get().size.width,
                    (None, None)            => fail!("This case makes no sense.")
                };

                info.width = actual_box_width;
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
               in_box.position.get().size,
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

        let new_width = self.pending_line.bounds.size.width + in_box.position.get().size.width;
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
    fn push_box_to_line(&mut self, box: Box) {
        debug!("LineboxScanner: Pushing box {} to line {:u}", box.debug_id(), self.lines.len());

        if self.pending_line.range.length() == 0 {
            assert!(self.new_boxes.len() <= (u16::max_value as uint));
            self.pending_line.range.reset(self.new_boxes.len(), 0);
        }
        self.pending_line.range.extend_by(1);
        self.pending_line.bounds.size.width = self.pending_line.bounds.size.width +
            box.position.get().size.width;
        self.pending_line.bounds.size.height = Au::max(self.pending_line.bounds.size.height,
                                                       box.position.get().size.height);
        self.new_boxes.push(box);
    }
}

pub struct InlineFlow {
    /// Data common to all flows.
    base: FlowData,

    /// A vector of all inline render boxes. Several boxes may correspond to one node/element.
    boxes: ~[Box],

    // vec of ranges into boxes that represents line positions.
    // these ranges are disjoint, and are the result of inline layout.
    // also some metadata used for positioning lines
    lines: ~[LineBox],

    // vec of ranges into boxes that represent elements. These ranges
    // must be well-nested, and are only related to the content of
    // boxes (not lines). Ranges are only kept for non-leaf elements.
    elems: ElementMapping,
}

impl InlineFlow {
    pub fn new(base: FlowData) -> InlineFlow {
        InlineFlow {
            base: base,
            boxes: ~[],
            lines: ~[],
            elems: ElementMapping::new(),
        }
    }

    pub fn from_boxes(base: FlowData, boxes: ~[Box]) -> InlineFlow {
        InlineFlow {
            base: base,
            boxes: boxes,
            lines: ~[],
            elems: ElementMapping::new(),
        }
    }

    pub fn teardown(&mut self) {
        for box in self.boxes.iter() {
            box.teardown();
        }
        self.boxes = ~[];
    }

    pub fn build_display_list_inline<E:ExtraDisplayListData>(
                                     &self,
                                     builder: &DisplayListBuilder,
                                     dirty: &Rect<Au>,
                                     list: &Cell<DisplayList<E>>)
                                     -> bool {
        let abs_rect = Rect(self.base.abs_position, self.base.position.size);
        if !abs_rect.intersects(dirty) {
            return true;
        }

        // TODO(#228): Once we form line boxes and have their cached bounds, we can be smarter and
        // not recurse on a line if nothing in it can intersect the dirty region.
        debug!("Flow[{:d}]: building display list for {:u} inline boxes",
               self.base.id,
               self.boxes.len());

        for box in self.boxes.iter() {
            box.build_display_list(builder, dirty, self.base.abs_position, (&*self) as &Flow, list)
        }

        // TODO(#225): Should `inline-block` elements have flows as children of the inline flow or
        // should the flow be nested inside the box somehow?

        // For now, don't traverse the subtree rooted here
        true
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
                (-(xheight + cur_box.box_height()).scale_by(0.5), false)
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
            let box = &boxes[i];
            let size = box.position.get().size;
            box.position.set(Rect(Point2D(offset_x, box.position.get().origin.y), size));
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
            let child_base = flow::mut_base(*kid);
            num_floats += child_base.num_floats;
            child_base.floats_in = FloatContext::new(child_base.num_floats);
        }

        let mut min_width = Au::new(0);
        let mut pref_width = Au::new(0);

        for box in self.boxes.iter() {
            debug!("Flow[{:d}]: measuring {:s}", self.base.id, box.debug_str());
            let (this_minimum_width, this_preferred_width) =
                box.minimum_and_preferred_widths();
            min_width = Au::max(min_width, this_minimum_width);
            pref_width = Au::max(pref_width, this_preferred_width);
        }

        self.base.min_width = min_width;
        self.base.pref_width = pref_width;
        self.base.num_floats = num_floats;
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent context.
    fn assign_widths(&mut self, _: &mut LayoutContext) {
        // Initialize content box widths if they haven't been initialized already.
        //
        // TODO: Combine this with `LineboxScanner`'s walk in the box list, or put this into `Box`.

        debug!("InlineFlow::assign_widths: floats_in: {:?}", self.base.floats_in);

        {
            let this = &mut *self;
            for box in this.boxes.iter() {
                box.assign_width();
            }
        }

        for kid in self.base.child_iter() {
            let child_base = flow::mut_base(*kid);
            child_base.position.size.width = self.base.position.size.width;
            child_base.flags.set_inorder(self.base.flags.inorder());
            child_base.flags.propagate_text_alignment_from_parent(self.base.flags)
        }
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

    fn assign_height(&mut self, _: &mut LayoutContext) {
        debug!("assign_height_inline: assigning height for flow {}", self.base.id);

        // Divide the boxes into lines.
        //
        // TODO(#226): Get the CSS `line-height` property from the containing block's style to
        // determine minimum linebox height.
        //
        // TODO(#226): Get the CSS `line-height` property from each non-replaced inline element to
        // determine its height for computing linebox height.
        //
        // TODO(pcwalton): Cache the linebox scanner?
        debug!("assign_height_inline: floats_in: {:?}", self.base.floats_in);

        let scanner_floats = self.base.floats_in.clone();
        let mut scanner = LineboxScanner::new(scanner_floats);

        // Access the linebox scanner.
        scanner.scan_for_lines(self);

        let mut line_height_offset = Au::new(0);

        // All lines use text alignment of the flow.
        let text_align = self.base.flags.text_align();

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

                // FIXME(pcwalton): Move into `box.rs` like the rest of box-specific layout code?
                let (top_from_base, bottom_from_base, ascent) = match cur_box.specific {
                    ImageBox(ref image_box) => {
                        let mut height = image_box.image_height();

                        // TODO: margin, border, padding's top and bottom should be calculated in
                        // advance, since baseline of image is bottom margin edge.
                        let mut top;
                        let mut bottom;
                        {
                            top = cur_box.border.get().top + cur_box.padding.get().top +
                                cur_box.margin.get().top;
                            bottom = cur_box.border.get().bottom + cur_box.padding.get().bottom +
                                cur_box.margin.get().bottom;
                        }

                        let noncontent_height = top + bottom;
                        height = height + noncontent_height;

                        let position_ref = cur_box.position.mutate();
                        position_ref.ptr.size.height = height;
                        position_ref.ptr.translate(&Point2D(Au::new(0), -height));

                        let ascent = height + bottom;
                        (height, Au::new(0), ascent)
                    },
                    ScannedTextBox(ref text_box) => {
                        let range = &text_box.range;
                        let run = &text_box.run;

                        // Compute the height based on the line-height and font size
                        let text_bounds = run.get().metrics_for_range(range).bounding_box;
                        let em_size = text_bounds.size.height;
                        let line_height = cur_box.calculate_line_height(em_size);

                        // Find the top and bottom of the content area.
                        // Those are used in text-top and text-bottom value of 'vertical-align'
                        let text_ascent = text_box.run.get().font_metrics.ascent;

                        // Offset from the top of the box is 1/2 of the leading + ascent
                        let text_offset = text_ascent + (line_height - em_size).scale_by(0.5);
                        text_bounds.translate(&Point2D(cur_box.position.get().origin.x, Au(0)));

                        (text_offset, line_height - text_offset, text_ascent)
                    },
                    GenericBox | IframeBox(_) => {
                        let height = cur_box.position.get().size.height;
                        (height, Au::new(0), height)
                    },
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
                let parent_text_top = cur_box.style().Font.font_size;

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

                cur_box.position.mutate().ptr.origin.y = line.bounds.origin.y + offset;
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

                cur_box.position.mutate().ptr.origin.y = cur_box.position.get().origin.y +
                    adjust_offset;
            }

            // This is used to set the top y position of the next linebox in the next loop.
            line_height_offset = line_height_offset + topmost + bottommost -
                line.bounds.size.height;
            line.bounds.size.height = topmost + bottommost;
        } // End of `lines.each` loop.

        self.base.position.size.height =
            if self.lines.len() > 0 {
                self.lines.last().bounds.origin.y + self.lines.last().bounds.size.height
            } else {
                Au::new(0)
            };

        self.base.floats_out = scanner.floats_out()
                                      .translate(Point2D(Au::new(0),
                                                         -self.base.position.size.height));
    }

    fn collapse_margins(&mut self,
                        _: bool,
                        _: &mut bool,
                        _: &mut Au,
                        _: &mut Au,
                        collapsing: &mut Au,
                        collapsible: &mut Au) {
        *collapsing = Au::new(0);
        // Non-empty inline flows prevent collapsing between the previous margion and the next.
        if self.base.position.size.height > Au::new(0) {
            *collapsible = Au::new(0);
        }
    }

    fn debug_str(&self) -> ~str {
        ~"InlineFlow: " + self.boxes.map(|s| s.debug_str()).connect(", ")
    }
}

