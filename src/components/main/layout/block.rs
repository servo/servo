/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS block layout.

use layout::box_::Box;
use layout::construct::FlowConstructor;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::floats::{ClearBoth, ClearLeft, ClearRight, FloatKind, FloatLeft, Floats};
use layout::floats::{PlacementInfo};
use layout::flow::{AssignHeightsFinished, AssignHeightsNeedsWidthOfFlow, AssignHeightsResult};
use layout::flow::{BaseFlow, BlockFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use layout::flow;
use layout::model::{MaybeAuto, Specified, Auto, specified_or_none, specified};
use layout::parallel::UnsafeFlow;
use layout::parallel;
use layout::wrapper::ThreadSafeLayoutNode;

use std::cell::RefCell;
use std::util;
use geom::{Point2D, Rect, SideOffsets2D, Size2D};
use gfx::display_list::{DisplayList, DisplayListCollection};
use servo_util::geometry::Au;
use servo_util::geometry;
use style::computed_values::{clear, display, float, overflow};

/// Information specific to floated blocks.
pub struct FloatedBlockInfo {
    containing_width: Au,

    /// Offset relative to where the parent tried to position this flow
    rel_pos: Point2D<Au>,

    /// Index into the box list for inline floats
    index: Option<uint>,

    /// Left or right?
    float_kind: FloatKind,
}

impl FloatedBlockInfo {
    pub fn new(float_kind: FloatKind) -> FloatedBlockInfo {
        FloatedBlockInfo {
            containing_width: Au(0),
            rel_pos: Point2D(Au(0), Au(0)),
            index: None,
            float_kind: float_kind,
        }
    }
}

/// Transient information used when assigning heights.
pub struct AssignHeightsState {
    /// The current Y position.
    cur_y: Au,
    /// The top offset to the content edge of the box.
    top_offset: Au,
    /// The bottom offset to the content edge of the box.
    bottom_offset: Au,
    /// The left offset to the content edge of the box.
    left_offset: Au,
    /// The amount of margin that we can potentially collapse with.
    collapsible: Au,
    /// How much to move up by to get to the beginning of the current child flow.
    ///
    /// Example: if the previous sibling's margin-bottom is 20px and our margin-top is 12px, the
    /// collapsed margin will be 20px. Since cur_y will be at the bottom margin edge of the
    /// previous sibling, we have to move up by 12px to get to our top margin edge. So,
    /// `collapsing` will be set to 12px.
    collapsing: Au,
    /// The top margin value.
    margin_top: Au,
    /// The bottom margin value.
    margin_bottom: Au,
    /// The child flow we need to process next, if any. This is used to resume where we left off.
    resume_point: Option<UnsafeFlow>,
    /// Whether the top margin is collapsible.
    top_margin_collapsible: bool,
    /// Whether the bottom margin is collapsible.
    bottom_margin_collapsible: bool,
    /// Whether we processing the first element in the flow.
    first_in_flow: bool,
    /// Whether we found an in-order child (i.e. one that was impacted by floats).
    found_inorder: bool,
}

impl AssignHeightsState {
    #[inline]
    fn new() -> AssignHeightsState {
        AssignHeightsState {
            cur_y: Au::new(0),
            top_offset: Au::new(0),
            bottom_offset: Au::new(0),
            left_offset: Au::new(0),
            collapsible: Au::new(0),
            collapsing: Au::new(0),
            margin_top: Au::new(0),
            margin_bottom: Au::new(0),
            resume_point: None,
            top_margin_collapsible: false,
            bottom_margin_collapsible: false,
            first_in_flow: true,
            found_inorder: false,
        }
    }
}

/// A block formatting context.
pub struct BlockFlow {
    /// Data common to all flows.
    base: BaseFlow,

    /// The associated box.
    box_: Option<Box>,

    /// Additional floating flow members.
    float: Option<~FloatedBlockInfo>,

    state: Option<~AssignHeightsState>,

    //TODO: is_fixed and is_root should be bit fields to conserve memory.
    /// Whether this block flow is the root flow.
    is_root: bool,

    is_fixed: bool,
}

impl BlockFlow {
    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode,
                     is_fixed: bool)
                     -> BlockFlow {
        BlockFlow {
            base: BaseFlow::new((*node).clone()),
            box_: Some(Box::new(constructor, node)),
            is_root: false,
            is_fixed: is_fixed,
            float: None,
            state: None,
        }
    }

    pub fn float_from_node(constructor: &mut FlowConstructor,
                           node: &ThreadSafeLayoutNode,
                           float_kind: FloatKind)
                           -> BlockFlow {
        BlockFlow {
            base: BaseFlow::new((*node).clone()),
            box_: Some(Box::new(constructor, node)),
            is_root: false,
            is_fixed: false,
            float: Some(~FloatedBlockInfo::new(float_kind)),
            state: None,
        }
    }

    pub fn new_root(base: BaseFlow) -> BlockFlow {
        BlockFlow {
            base: base,
            box_: None,
            is_root: true,
            is_fixed: false,
            float: None,
            state: None,
        }
    }

    pub fn new_float(base: BaseFlow, float_kind: FloatKind) -> BlockFlow {
        BlockFlow {
            base: base,
            box_: None,
            is_root: false,
            is_fixed: false,
            float: Some(~FloatedBlockInfo::new(float_kind)),
            state: None,
        }
    }

    pub fn is_float(&self) -> bool {
        self.float.is_some()
    }

    pub fn teardown(&mut self) {
        for box_ in self.box_.iter() {
            box_.teardown();
        }
        self.box_ = None;
        self.float = None;
    }

    /// Computes left and right margins and width based on CSS 2.1 section 10.3.3.
    /// Requires borders and padding to already be computed.
    fn compute_horiz(&self,
                     width: MaybeAuto,
                     left_margin: MaybeAuto,
                     right_margin: MaybeAuto,
                     available_width: Au)
                     -> (Au, Au, Au) {
        // If width is not 'auto', and width + margins > available_width, all
        // 'auto' margins are treated as 0.
        let (left_margin, right_margin) = match width {
            Auto => (left_margin, right_margin),
            Specified(width) => {
                let left = left_margin.specified_or_zero();
                let right = right_margin.specified_or_zero();

                if((left + right + width) > available_width) {
                    (Specified(left), Specified(right))
                } else {
                    (left_margin, right_margin)
                }
            }
        };

        // Invariant: left_margin_Au + width_Au + right_margin_Au == available_width
        let (left_margin_Au, width_Au, right_margin_Au) = match (left_margin, width, right_margin) {
            // If all have a computed value other than 'auto', the system is
            // over-constrained and we need to discard a margin.
            // If direction is ltr, ignore the specified right margin and
            // solve for it.
            // If it is rtl, ignore the specified left margin.
            // FIXME(eatkinson): this assumes the direction is ltr
            (Specified(margin_l), Specified(width), Specified(_margin_r)) =>
                (margin_l, width, available_width - (margin_l + width )),

            // If exactly one value is 'auto', solve for it
            (Auto, Specified(width), Specified(margin_r)) =>
                (available_width - (width + margin_r), width, margin_r),
            (Specified(margin_l), Auto, Specified(margin_r)) =>
                (margin_l, available_width - (margin_l + margin_r), margin_r),
            (Specified(margin_l), Specified(width), Auto) =>
                (margin_l, width, available_width - (margin_l + width)),

            // If width is set to 'auto', any other 'auto' value becomes '0',
            // and width is solved for
            (Auto, Auto, Specified(margin_r)) =>
                (Au::new(0), available_width - margin_r, margin_r),
            (Specified(margin_l), Auto, Auto) =>
                (margin_l, available_width - margin_l, Au::new(0)),
            (Auto, Auto, Auto) =>
                (Au::new(0), available_width, Au::new(0)),

            // If left and right margins are auto, they become equal
            (Auto, Specified(width), Auto) => {
                let margin = (available_width - width).scale_by(0.5);
                (margin, width, margin)
            }

        };
        // Return values in same order as params.
        (width_Au, left_margin_Au, right_margin_Au)
    }

    // Return (content width, left margin, right, margin)
    fn compute_block_margins(&self, box_: &Box, remaining_width: Au, available_width: Au)
                             -> (Au, Au, Au) {
        let style = box_.style();

        let (width, maybe_margin_left, maybe_margin_right) =
            (MaybeAuto::from_style(style.Box.get().width, remaining_width),
             MaybeAuto::from_style(style.Margin.get().margin_left, remaining_width),
             MaybeAuto::from_style(style.Margin.get().margin_right, remaining_width));

        let (width, margin_left, margin_right) = self.compute_horiz(width,
                                                                    maybe_margin_left,
                                                                    maybe_margin_right,
                                                                    available_width);

        // CSS Section 10.4: Minimum and Maximum widths
        // If the tentative used width is greater than 'max-width', width should be recalculated,
        // but this time using the computed value of 'max-width' as the computed value for 'width'.
        let (width, margin_left, margin_right) = {
            match specified_or_none(style.Box.get().max_width, remaining_width) {
                Some(value) if value < width => self.compute_horiz(Specified(value),
                                                                   maybe_margin_left,
                                                                   maybe_margin_right,
                                                                   available_width),
                _ => (width, margin_left, margin_right)
            }
        };

        // If the resulting width is smaller than 'min-width', width should be recalculated,
        // but this time using the value of 'min-width' as the computed value for 'width'.
        let (width, margin_left, margin_right) = {
            let computed_min_width = specified(style.Box.get().min_width, remaining_width);
            if computed_min_width > width {
                self.compute_horiz(Specified(computed_min_width),
                                   maybe_margin_left,
                                   maybe_margin_right,
                                   available_width)
            } else {
                (width, margin_left, margin_right)
            }
        };

        return (width, margin_left, margin_right);
    }

    // CSS Section 10.3.5
    fn compute_float_margins(&self, box_: &Box, remaining_width: Au) -> (Au, Au, Au) {
        let style = box_.style();
        let margin_left = MaybeAuto::from_style(style.Margin.get().margin_left,
                                                remaining_width).specified_or_zero();
        let margin_right = MaybeAuto::from_style(style.Margin.get().margin_right,
                                                 remaining_width).specified_or_zero();
        let shrink_to_fit = geometry::min(self.base.pref_width,
                                          geometry::max(self.base.min_width, remaining_width));
        let width = MaybeAuto::from_style(style.Box.get().width,
                                          remaining_width).specified_or_default(shrink_to_fit);
        debug!("assign_widths_float -- width: {}", width);
        return (width, margin_left, margin_right);
    }

    /// Assumes that the float has already had its height assigned.
    fn place_float(&mut self) {
        let mut height = Au(0);
        let mut clearance = Au(0);
        let mut full_noncontent_width = Au(0);
        let mut margin_height = Au(0);

        for box_ in self.box_.iter() {
            height = box_.border_box.get().size.height;
            clearance = match box_.clear() {
                None => Au(0),
                Some(clear) => self.base.floats.clearance(clear),
            };

            let noncontent_width = box_.padding.get().left + box_.padding.get().right +
                box_.border.get().left + box_.border.get().right;

            full_noncontent_width = noncontent_width + box_.margin.get().left +
                box_.margin.get().right;
            margin_height = box_.margin.get().top + box_.margin.get().bottom;
        }

        let info = PlacementInfo {
            size: Size2D(self.base.position.size.width + full_noncontent_width,
                         height + margin_height),
            ceiling: clearance,
            max_width: self.float.get_ref().containing_width,
            kind: self.float.get_ref().float_kind,
        };

        // Place the float and return the `Floats` back to the parent flow.
        // After, grab the position and use that to set our position.
        self.base.floats.add_float(&info);

        self.float.get_mut_ref().rel_pos = self.base.floats.last_float_pos().unwrap();
    }

    fn assign_height_float(&mut self, ctx: &mut LayoutContext) -> AssignHeightsResult {
        // Now that we've determined our height, propagate that out.
        let mut cur_y = Au(0);
        let mut top_offset = Au(0);

        for box_ in self.box_.iter() {
            top_offset = box_.margin.get().top + box_.border.get().top + box_.padding.get().top;
            cur_y = cur_y + top_offset;
        }

        // Whether we found an in-order child (i.e. one that was impacted by floats).
        let mut found_inorder = false;

        let mut floats = Floats::new();
        for kid in self.base.child_iter() {
            {
                let kid_base = flow::mut_base(kid);
                kid_base.position.origin.y = cur_y;
            }

            let result = perform_delayed_width_assignment_for_child_if_necessary(
                kid,
                self.box_.as_ref().unwrap(),
                &mut floats);
            assert!(result == AssignHeightsFinished);

            // Now perform in-order subtraversals if necessary.
            let (result, inorder) = kid.process_inorder_child_if_necessary(ctx, floats);
            assert!(result == AssignHeightsFinished);
            found_inorder = found_inorder || inorder;

            let kid_base = flow::mut_base(kid);
            cur_y = cur_y + kid_base.position.size.height;

            floats = kid_base.floats.clone();
        }

        // Floats establish a block formatting context, so we discard the output floats here.
        drop(floats);

        let mut height = cur_y - top_offset;

        let mut noncontent_height;
        let box_ = self.box_.as_ref().unwrap();
        let mut position = box_.border_box.get();

        // The associated box is the border box of this flow.
        position.origin.y = box_.margin.get().top;

        noncontent_height = box_.padding.get().top + box_.padding.get().bottom +
            box_.border.get().top + box_.border.get().bottom;

        //TODO(eatkinson): compute heights properly using the 'height' property.
        let height_prop = MaybeAuto::from_style(box_.style().Box.get().height,
                                                Au::new(0)).specified_or_zero();

        height = geometry::max(height, height_prop) + noncontent_height;
        debug!("assign_height_float -- height: {}", height);

        position.size.height = height;
        box_.border_box.set(position);

        AssignHeightsFinished
    }

    pub fn build_display_list_block<E:ExtraDisplayListData>(
                                    &mut self,
                                    builder: &DisplayListBuilder,
                                    container_block_size: &Size2D<Au>,
                                    dirty: &Rect<Au>,
                                    mut index: uint,
                                    lists: &RefCell<DisplayListCollection<E>>)
                                    -> uint {
        if self.is_float() {
            self.build_display_list_float(builder, container_block_size, dirty, index, lists);
            return index;
        }

        if self.is_fixed {
            lists.with_mut(|lists| {
                index = lists.lists.len();
                lists.add_list(DisplayList::<E>::new());
            });
        }

        let abs_rect = Rect(self.base.abs_position, self.base.position.size);
        if !abs_rect.intersects(dirty) {
            return index;
        }

        debug!("build_display_list_block: adding display element");

        let rel_offset = match self.box_ {
            Some(ref box_) => {
                box_.relative_position(container_block_size)
            },
            None => {
                Point2D {
                    x: Au::new(0),
                    y: Au::new(0),
                }
            }
        };

        // add box that starts block context
        for box_ in self.box_.iter() {
            box_.build_display_list(builder, dirty, self.base.abs_position + rel_offset, (&*self) as &Flow, index, lists);
        }
        // TODO: handle any out-of-flow elements
        let this_position = self.base.abs_position;

        for child in self.base.child_iter() {
            let child_base = flow::mut_base(child);
            child_base.abs_position = this_position + child_base.position.origin + rel_offset;
        }

        index
    }

    pub fn build_display_list_float<E:ExtraDisplayListData>(
                                    &mut self,
                                    builder: &DisplayListBuilder,
                                    container_block_size: &Size2D<Au>,
                                    dirty: &Rect<Au>,
                                    index: uint,
                                    lists: &RefCell<DisplayListCollection<E>>)
                                    -> bool {
        let abs_rect = Rect(self.base.abs_position, self.base.position.size);
        if !abs_rect.intersects(dirty) {
            return true;
        }

        // position:relative
        let rel_offset = match self.box_ {
            Some(ref box_) => {
                box_.relative_position(container_block_size)
            },
            None => {
                Point2D {
                    x: Au::new(0),
                    y: Au::new(0),
                }
            }
        };


        let offset = self.base.abs_position + self.float.get_ref().rel_pos + rel_offset;
        // add box that starts block context
        for box_ in self.box_.iter() {
            box_.build_display_list(builder, dirty, offset, (&*self) as &Flow, index, lists);
        }


        // TODO: handle any out-of-flow elements

        // go deeper into the flow tree
        for child in self.base.child_iter() {
            let child_base = flow::mut_base(child);
            child_base.abs_position = offset + child_base.position.origin + rel_offset;
        }

        false
    }

}

impl Flow for BlockFlow {
    fn class(&self) -> FlowClass {
        BlockFlowClass
    }

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        self
    }

    /// Returns the direction that this flow clears floats in, if any.
    fn float_clearance(&self) -> clear::T {
        self.box_.as_ref().unwrap().style().Box.get().clear
    }

    fn is_block_formatting_context(&self, only_impactable_by_floats: bool) -> bool {
        let style = self.box_.as_ref().unwrap().style();
        if style.Box.get().float != float::none {
            return !only_impactable_by_floats
        }
        if style.Box.get().overflow != overflow::visible {
            return true
        }
        match style.Box.get().display {
            display::table_cell | display::table_caption | display::inline_block => true,
            _ => false,
        }
    }

    /* Recursively (bottom-up) determine the context's preferred and
    minimum widths.  When called on this context, all child contexts
    have had their min/pref widths set. This function must decide
    min/pref widths based on child context widths and dimensions of
    any boxes it is responsible for flowing.  */

    /// Recursively (bottom-up) determine the context's preferred and minimum widths. When called
    /// on this context, all child contexts have had their min/pref widths set. This function must
    /// decide min/pref widths based on child context widths and dimensions of any boxes it is
    /// responsible for flowing.
    ///
    /// TODO(pradeep): Absolute contexts.
    /// TODO(pcwalton): Inline-blocks.
    fn bubble_widths(&mut self, _: &mut LayoutContext) {
        let mut min_width = Au::new(0);
        let mut pref_width = Au::new(0);
        let (mut has_left_floated_descendants, mut has_right_floated_descendants) = (false, false);

        // Find the maximum width from child block contexts.
        for kid in self.base.child_iter() {
            let kid_base = flow::mut_base(kid);
            min_width = geometry::max(min_width, kid_base.min_width);
            pref_width = geometry::max(pref_width, kid_base.pref_width);

            has_left_floated_descendants = has_left_floated_descendants ||
                kid_base.flags_info.flags.has_left_floated_descendants();
            has_right_floated_descendants = has_right_floated_descendants ||
                kid_base.flags_info.flags.has_right_floated_descendants();
        }

        // If not an anonymous block context, add in block box's widths.
        // These widths will not include child elements, just padding, etc.
        for box_ in self.box_.iter() {
            {
                let style = box_.style();

                match style.Box.get().float {
                    float::none => {
                        self.base
                            .flags_info
                            .flags
                            .set_has_left_floated_descendants(has_left_floated_descendants);
                        self.base
                            .flags_info
                            .flags
                            .set_has_right_floated_descendants(has_right_floated_descendants);
                    }
                    float::left => {
                        self.base.flags_info.flags.set_has_left_floated_descendants(true);
                        self.base
                            .flags_info
                            .flags
                            .set_has_right_floated_descendants(has_right_floated_descendants);
                    }
                    float::right => {
                        self.base.flags_info.flags.set_has_right_floated_descendants(true);
                        self.base
                            .flags_info
                            .flags
                            .set_has_left_floated_descendants(has_left_floated_descendants);
                    }
                }

                // Can compute border width here since it doesn't depend on anything.
                //
                // TODO(pcwalton): Don't do this; just have later passes consult the style
                // directly.
                box_.compute_borders(style);
            }

            let (this_minimum_width, this_preferred_width) = box_.minimum_and_preferred_widths();
            min_width = min_width + this_minimum_width;
            pref_width = pref_width + this_preferred_width;
        }

        self.base.min_width = min_width;
        self.base.pref_width = pref_width;
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent context.
    ///
    /// Dual boxes consume some width first, and the remainder is assigned to all child (block)
    /// contexts.
    fn assign_widths(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_widths({}): assigning width for flow",
               if self.is_float() {
                   "float"
               } else {
                   "block"
               });

        if self.is_root {
            debug!("Setting root position");
            self.base.position.origin = Au::zero_point();
            self.base.position.size.width = ctx.screen_size.width;

            self.base.floats = Floats::new();
            self.base.flags_info.flags.set_impacted_by_left_floats(false);
            self.base.flags_info.flags.set_impacted_by_right_floats(false);
        }

        // The position was set to the containing block by the flow's parent.
        let mut remaining_width = self.base.position.size.width;
        let mut x_offset = Au::new(0);

        // Parent usually sets this, but block formatting contexts are never inorder
        if self.is_block_formatting_context(false) {
            self.base.flags_info.flags.set_impacted_by_left_floats(false);
            self.base.flags_info.flags.set_impacted_by_right_floats(false);
        }

        if self.is_float() {
            self.float.get_mut_ref().containing_width = remaining_width;
        }

        for box_ in self.box_.iter() {
            let style = box_.style();

            // The text alignment of a block flow is the text alignment of its box's style.
            self.base.flags_info.flags.set_text_align(style.InheritedText.get().text_align);

            box_.assign_width(remaining_width);
            // Can compute padding here since we know containing block width.
            box_.compute_padding(style, remaining_width);

            // Margins are 0 right now so base.noncontent_width() is just borders + padding.
            let available_width = remaining_width - box_.noncontent_width();

            // Top and bottom margins for blocks are 0 if auto.
            let margin_top = MaybeAuto::from_style(style.Margin.get().margin_top,
                                                   remaining_width).specified_or_zero();
            let margin_bottom = MaybeAuto::from_style(style.Margin.get().margin_bottom,
                                                      remaining_width).specified_or_zero();

            let (width, margin_left, margin_right) = if self.is_float() {
                self.compute_float_margins(box_, remaining_width)
            } else {
                self.compute_block_margins(box_, remaining_width, available_width)
            };

            box_.margin.set(SideOffsets2D::new(margin_top,
                                               margin_right,
                                               margin_bottom,
                                               margin_left));

            let screen_size = ctx.screen_size;
            let (x, w) = box_.get_x_coord_and_new_width_if_fixed(screen_size.width,
                                                                 screen_size.height,
                                                                 width,
                                                                 box_.content_left(),
                                                                 self.is_fixed);

            x_offset = x;
            remaining_width = w;

            // The associated box is the border box of this flow.
            let mut position_ref = box_.border_box.borrow_mut();
            if self.is_fixed {
                position_ref.get().origin.x = x_offset + box_.margin.get().left;
                x_offset = x_offset + box_.padding.get().left;
            } else {
                position_ref.get().origin.x = box_.margin.get().left;
            }
            let padding_and_borders = box_.padding.get().left + box_.padding.get().right +
                box_.border.get().left + box_.border.get().right;
            position_ref.get().size.width = remaining_width + padding_and_borders;
        }

        if self.is_float() {
            self.base.position.size.width = remaining_width;
        }

        // FIXME(ksh8281): avoid copy
        let flags_info = self.base.flags_info.clone();

        // Keep track of whether floats could impact each child.
        let mut left_floats_impact_child = self.base.flags_info.flags.impacted_by_left_floats();
        let mut right_floats_impact_child = self.base.flags_info.flags.impacted_by_right_floats();

        for kid in self.base.child_iter() {
            {
                assert!(kid.is_block_flow() || kid.is_inline_flow());

                let child_base = flow::mut_base(kid);
                child_base.position.origin.x = x_offset;
                child_base.position.size.width = remaining_width;
            }

            match kid.float_clearance() {
                clear::none => {}
                clear::left => left_floats_impact_child = false,
                clear::right => right_floats_impact_child = false,
                clear::both => {
                    left_floats_impact_child = false;
                    right_floats_impact_child = false;
                }
            }

            {
                let kid_base = flow::mut_base(kid);
                kid_base.flags_info.flags.set_impacted_by_left_floats(left_floats_impact_child);
                kid_base.flags_info.flags.set_impacted_by_right_floats(right_floats_impact_child);
            }

            // If the kid establishes a block formatting context, we can't assign its widths yet.
            if (left_floats_impact_child || right_floats_impact_child) &&
                    kid.is_block_formatting_context(true) {
                let kid_base = flow::mut_base(kid);
                kid_base.flags_info.flags.set_assign_widths_delayed(true);
            }

            let kid_base = flow::mut_base(kid);
            left_floats_impact_child = left_floats_impact_child ||
                kid_base.flags_info.flags.has_left_floated_descendants();
            right_floats_impact_child = right_floats_impact_child ||
                kid_base.flags_info.flags.has_right_floated_descendants();

            // Per CSS 2.1 § 16.3.1, text decoration propagates to all children in flow.
            //
            // TODO(pcwalton): When we have out-of-flow children, don't unconditionally propagate.
            // TODO(pcwalton): Do this in the cascade instead.

            kid_base.flags_info.propagate_text_decoration_from_parent(&flags_info);
            kid_base.flags_info.propagate_text_alignment_from_parent(&flags_info)
        }
    }

    fn process_inorder_child_if_necessary(&mut self,
                                          layout_context: &mut LayoutContext,
                                          floats: Floats)
                                          -> (AssignHeightsResult, bool) {
        if self.is_float() {
            self.base.floats = floats;
            self.place_float();
            return (AssignHeightsFinished, true)
        }

        let impacted = self.base.flags_info.flags.impacted_by_floats();
        if impacted {
            self.base.floats = floats;
            return (self.assign_height(layout_context), true)
        }

        (AssignHeightsFinished, false)
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) -> AssignHeightsResult {
        //assign height for box
        for box_ in self.box_.iter() {
            box_.assign_height();
        }

        if self.is_float() {
            debug!("assign_height_float: assigning height for float");
            return self.assign_height_float(ctx)
        }

        let mut state;
        match util::replace(&mut self.state, None) {
            Some(~existing_state) => state = existing_state,
            None => {
                state = AssignHeightsState::new();

                for box_ in self.box_.iter() {
                    state.top_offset = box_.margin.get().top + box_.border.get().top +
                        box_.padding.get().top;
                    state.cur_y = state.cur_y + state.top_offset;
                    state.bottom_offset = box_.margin.get().bottom + box_.border.get().bottom +
                        box_.padding.get().bottom;
                    state.left_offset = box_.content_left();
                }

                self.base.floats.translate(Point2D(-state.left_offset, -state.top_offset));

                for box_ in self.box_.iter() {
                    if !self.is_root && box_.border.get().top == Au(0) &&
                            box_.padding.get().top == Au(0) {
                        state.collapsible = box_.margin.get().top;
                        state.top_margin_collapsible = true;
                    }
                    if !self.is_root && box_.border.get().bottom == Au(0) &&
                            box_.padding.get().bottom == Au(0) {
                        state.bottom_margin_collapsible = true;
                    }
                    state.margin_top = box_.margin.get().top;
                    state.margin_bottom = box_.margin.get().bottom;
                }
            }
        };

        // At this point, cur_y is at the content edge of the flow's box.
        let mut result = AssignHeightsFinished;
        let mut found_inorder = false;
        let mut floats = self.base.floats.clone();
        for kid in self.base.child_iter() {
            match state.resume_point {
                None => {}
                Some(resume_point) if
                        resume_point == parallel::mut_borrowed_flow_to_unsafe_flow(kid) => {
                    state.resume_point = None;
                }
                Some(_) => continue,
            }

            // If the child establishes a block formatting context, perform its width assignment
            // now.
            result = perform_delayed_width_assignment_for_child_if_necessary(kid,
                                                                             self.box_
                                                                                 .as_ref()
                                                                                 .unwrap(),
                                                                             &floats);
            if result != AssignHeightsFinished {
                state.resume_point = Some(parallel::mut_borrowed_flow_to_unsafe_flow(kid));
                break
            }

            // At this point, cur_y is at bottom margin edge of previous kid.
            kid.collapse_margins(state.top_margin_collapsible,
                                 &mut state.first_in_flow,
                                 &mut state.margin_top,
                                 &mut state.top_offset,
                                 &mut state.collapsing,
                                 &mut state.collapsible);
            state.cur_y = state.cur_y - state.collapsing;

            let in_flow = kid.in_flow();
            if in_flow {
                // Take clearance into account. This must be done here (as opposed to within the
                // kid's own `assign_heights` call) because the kid might have been laid out
                // already.
                let clearance = match kid.float_clearance() {
                    clear::none => Au::new(0),
                    clear::left => floats.clearance(ClearLeft),
                    clear::right => floats.clearance(ClearRight),
                    clear::both => floats.clearance(ClearBoth),
                };
                if clearance != Au::new(0) {
                    state.cur_y = state.cur_y + clearance;
                    floats.translate(Point2D(Au::new(0), -clearance));
                }
            }

            // At this point, after moving up by `collapsing`, cur_y is at the top margin edge of
            // kid.
            //
            // Now perform in-order subtraversals if necessary.
            //
            // Floats for blocks work like this:
            //  * self.floats_in -> child[0].floats_in
            //  * visit child[0]
            //  * child[i-1].floats_out -> child[i].floats_in
            //  * visit child[i]
            //  * repeat until all children are visited.
            //  * last_child.floats_out -> self.floats_out (done at the end of this function)
            let (this_result, inorder) = kid.process_inorder_child_if_necessary(ctx,
                                                                                floats.clone());
            result = this_result;
            found_inorder = found_inorder || inorder;
            if result != AssignHeightsFinished {
                state.resume_point = Some(parallel::mut_borrowed_flow_to_unsafe_flow(kid));
                break
            }

            let kid_base = flow::mut_base(kid);
            kid_base.position.origin.y = state.cur_y;
            state.cur_y = state.cur_y + kid_base.position.size.height;

            // At this point, cur_y is at the bottom margin edge of kid.
            floats = kid_base.floats.clone();
        }

        // Save our floats, and stop now if we were interrupted.
        self.base.floats = floats;
        if result != AssignHeightsFinished {
            self.state = Some(~state);
            return result
        }

        // The bottom margin collapses with its last in-flow block-level child's bottom margin
        // if the parent has no bottom boder, no bottom padding.
        state.collapsing = if state.bottom_margin_collapsible {
            if state.margin_bottom < state.collapsible {
                state.margin_bottom = state.collapsible;
            }
            state.collapsible
        } else {
            Au::new(0)
        };

        // TODO: A box's own margins collapse if the 'min-height' property is zero, and it has neither
        // top or bottom borders nor top or bottom padding, and it has a 'height' of either 0 or 'auto',
        // and it does not contain a line box, and all of its in-flow children's margins (if any) collapse.

        let screen_height = ctx.screen_size.height;

        let mut height = if self.is_root {
            // FIXME(pcwalton): The max is taken here so that you can scroll the page, but this is
            // not correct behavior according to CSS 2.1 § 10.5. Instead I think we should treat
            // the root element as having `overflow: scroll` and use the layers-based scrolling
            // infrastructure to make it scrollable.
            Au::max(screen_height, state.cur_y)
        } else {
            // (cur_y - collapsing) will get you the bottom content edge
            // top_offset will be at top content edge
            // hence, height = content height
            state.cur_y - state.top_offset - state.collapsing
        };

        for box_ in self.box_.iter() {
            let style = box_.style();

            // At this point, `height` is the height of the containing block, so passing `height`
            // as the second argument here effectively makes percentages relative to the containing
            // block per CSS 2.1 § 10.5.
            // TODO: We need to pass in the correct containing block height
            // for absolutely positioned elems
            height = match MaybeAuto::from_style(style.Box.get().height, height) {
                Auto => height,
                Specified(value) => value
            };
        }

        // Here, height is content height of box_

        let mut noncontent_height = Au::new(0);
        for box_ in self.box_.iter() {
            let mut position = box_.border_box.get();
            let mut margin = box_.margin.get();

            // The associated box is the border box of this flow.
            // Margin after collapse
            margin.top = state.margin_top;
            margin.bottom = state.margin_bottom;

            noncontent_height = box_.padding.get().top + box_.padding.get().bottom +
                box_.border.get().top + box_.border.get().bottom;

            let (y, h) = box_.get_y_coord_and_new_height_if_fixed(screen_height,
                                                                  height,
                                                                  margin.top,
                                                                  self.is_fixed);

            position.origin.y = y;
            height = h;

            if self.is_fixed {
                for kid in self.base.child_iter() {
                    let child_node = flow::mut_base(kid);
                    child_node.position.origin.y = position.origin.y + state.top_offset;
                }
            }

            position.size.height = if self.is_fixed {
                height
            } else {
                // Border box height
                height + noncontent_height
            };

            noncontent_height = noncontent_height + margin.top + margin.bottom;

            box_.border_box.set(position);
            box_.margin.set(margin);
        }

        self.base.position.size.height = if self.is_fixed {
            height
        } else {
            // Height of margin box
            height + noncontent_height
        };

        if found_inorder {
            let extra_height = height - (state.cur_y - state.top_offset) + state.bottom_offset;
            self.base.floats.translate(Point2D(state.left_offset, -extra_height));
        }

        AssignHeightsFinished
    }

    // CSS Section 8.3.1 - Collapsing Margins
    fn collapse_margins(&mut self,
                        top_margin_collapsible: bool,
                        first_in_flow: &mut bool,
                        margin_top: &mut Au,
                        top_offset: &mut Au,
                        collapsing: &mut Au,
                        collapsible: &mut Au) {
        if self.is_float() {
            // Margins between a floated box and any other box do not collapse.
            *collapsing = Au::new(0);
            return;
        }

        for box_ in self.box_.iter() {
            // The top margin collapses with its first in-flow block-level child's
            // top margin if the parent has no top border, no top padding.
            if *first_in_flow && top_margin_collapsible {
                // If top-margin of parent is less than top-margin of its first child,
                // the parent box goes down until its top is aligned with the child.
                if *margin_top < box_.margin.get().top {
                    // TODO: The position of child floats should be updated and this
                    // would influence clearance as well. See #725
                    let extra_margin = box_.margin.get().top - *margin_top;
                    *top_offset = *top_offset + extra_margin;
                    *margin_top = box_.margin.get().top;
                }
            }
            // The bottom margin of an in-flow block-level element collapses
            // with the top margin of its next in-flow block-level sibling.
            *collapsing = geometry::min(box_.margin.get().top, *collapsible);
            *collapsible = box_.margin.get().bottom;
        }

        *first_in_flow = false;
    }

    fn in_flow(&self) -> bool {
        !self.is_float()
    }

    fn mark_as_root(&mut self) {
        self.is_root = true
    }

    fn debug_str(&self) -> ~str {
        let impacted = if self.base.flags_info.flags.impacted_by_floats() {
            "(impacted)"
        } else {
            ""
        };

        let txt = if self.is_float() {
            format!("FloatFlow{}: ", impacted)
        } else if self.is_root {
            format!("RootFlow{}: ", impacted)
        } else {
            format!("BlockFlow{}: ", impacted)
        };

        txt.append(match self.box_ {
            Some(ref rb) => rb.debug_str(),
            None => ~"",
        })
    }
}

fn perform_delayed_width_assignment_for_child_if_necessary(kid: &mut Flow,
                                                           parent_box: &Box,
                                                           floats: &Floats)
                                                           -> AssignHeightsResult {
    {
        let kid_base = flow::mut_base(kid);
        if !kid_base.flags_info.flags.assign_widths_delayed() {
            return AssignHeightsFinished
        }

        // This is a block formatting context impacted by floats. We can now assign its width.
        //
        // FIXME(pcwalton): Setting the height to 1 is a hack; can we do better?
        let placement_info = PlacementInfo {
            size: Size2D(kid_base.min_width, Au::new(1)),
            ceiling: Au::new(0),
            max_width: parent_box.border_box.get().size.width,
            kind: FloatLeft,
        };
        let placement_box = floats.place_between_floats(&placement_info);
        kid_base.position.origin.x = parent_box.content_left();
        kid_base.position.size.width = placement_box.size.width;

        kid_base.flags_info.flags.set_assign_widths_delayed(false);
    }

    let unsafe_flow = parallel::mut_borrowed_flow_to_unsafe_flow(kid);
    AssignHeightsNeedsWidthOfFlow(unsafe_flow)
}

