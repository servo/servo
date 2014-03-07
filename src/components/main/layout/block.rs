/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS block formatting contexts.
//!
//! Terminology Note:
//! As per the CSS Spec, the term 'absolute positioning' here refers to
//! elements with position = 'absolute' or 'fixed'.
//! The term 'positioned element' refers to elements with position =
//! 'relative', 'absolute', or 'fixed'.
//!
//! CB: Containing Block of the current flow.

use layout::box_::{Box, ImageBox, ScannedTextBox};
use layout::construct::FlowConstructor;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::floats::{FloatKind, Floats, PlacementInfo};
use layout::flow::{BaseFlow, BlockFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use layout::flow::{mut_base, PreorderFlowTraversal, PostorderFlowTraversal, MutableFlowUtils};
use layout::flow;
use layout::model::{MaybeAuto, Specified, Auto, specified_or_none, specified};
use layout::wrapper::ThreadSafeLayoutNode;
use style::computed_values::{position};

use std::cell::RefCell;
use geom::{Point2D, Rect, Size2D};
use gfx::display_list::{DisplayListCollection, DisplayList};
use servo_util::geometry::Au;
use servo_util::geometry;
use servo_util::smallvec::{SmallVec, SmallVec0};

/// Information specific to floated blocks.
pub struct FloatedBlockInfo {
    containing_width: Au,

    /// Offset relative to where the parent tried to position this flow
    rel_pos: Point2D<Au>,

    /// Index into the box list for inline floats
    index: Option<uint>,

    /// Number of floated children
    floated_children: uint,

    /// Left or right?
    float_kind: FloatKind,
}

impl FloatedBlockInfo {
    pub fn new(float_kind: FloatKind) -> FloatedBlockInfo {
        FloatedBlockInfo {
            containing_width: Au(0),
            rel_pos: Point2D(Au(0), Au(0)),
            index: None,
            floated_children: 0,
            float_kind: float_kind,
        }
    }
}

/// The solutions for the heights-and-margins constraint equation.
struct HeightConstraintSolution {
    top: Au,
    bottom: Au,
    height: Au,
    margin_top: Au,
    margin_bottom: Au
}

impl HeightConstraintSolution {
    fn new(top: Au, bottom: Au, height: Au, margin_top: Au, margin_bottom: Au)
           -> HeightConstraintSolution {
        HeightConstraintSolution {
            top: top,
            bottom: bottom,
            height: height,
            margin_top: margin_top,
            margin_bottom: margin_bottom,
        }
    }

    /// Solve the vertical constraint equation for absolute non-replaced elements.
    ///
    /// CSS Section 10.6.4
    /// Constraint equation:
    /// top + bottom + height + margin-top + margin-bottom
    /// = absolute containing block height - (vertical padding and border)
    /// [aka available_height]
    ///
    /// Return the solution for the equation.
    fn solve_vertical_constraints_abs_nonreplaced(height: MaybeAuto,
                                                  top_margin: MaybeAuto,
                                                  bottom_margin: MaybeAuto,
                                                  top: MaybeAuto,
                                                  bottom: MaybeAuto,
                                                  content_height: Au,
                                                  available_height: Au,
                                                  static_y_offset: Au)
                                               -> HeightConstraintSolution {
        // Distance from the top edge of the Absolute Containing Block to the
        // top margin edge of a hypothetical box that would have been the
        // first box of the element.
        let static_position_top = static_y_offset;

        let (top, bottom, height, margin_top, margin_bottom) = match (top, bottom, height) {
            (Auto, Auto, Auto) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let top = static_position_top;
                // Now it is the same situation as top Specified and bottom
                // and height Auto.

                let height = content_height;
                let sum = top + height + margin_top + margin_bottom;
                (top, available_height - sum, height, margin_top, margin_bottom)
            }
            (Specified(top), Specified(bottom), Specified(height)) => {
                match (top_margin, bottom_margin) {
                    (Auto, Auto) => {
                        let total_margin_val = (available_height - top - bottom - height);
                        (top, bottom, height,
                         total_margin_val.scale_by(0.5),
                         total_margin_val.scale_by(0.5))
                    }
                    (Specified(margin_top), Auto) => {
                        let sum = top + bottom + height + margin_top;
                        (top, bottom, height, margin_top, available_height - sum)
                    }
                    (Auto, Specified(margin_bottom)) => {
                        let sum = top + bottom + height + margin_bottom;
                        (top, bottom, height, available_height - sum, margin_bottom)
                    }
                    (Specified(margin_top), Specified(margin_bottom)) => {
                        // Values are over-constrained. Ignore value for 'bottom'.
                        let sum = top + height + margin_top + margin_bottom;
                        (top, available_height - sum, height, margin_top, margin_bottom)
                    }
                }
            }

            // For the rest of the cases, auto values for margin are set to 0

            // If only one is Auto, solve for it
            (Auto, Specified(bottom), Specified(height)) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let sum = bottom + height + margin_top + margin_bottom;
                (available_height - sum, bottom, height, margin_top, margin_bottom)
            }
            (Specified(top), Auto, Specified(height)) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let sum = top + height + margin_top + margin_bottom;
                (top, available_height - sum, height, margin_top, margin_bottom)
            }
            (Specified(top), Specified(bottom), Auto) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let sum = top + bottom + margin_top + margin_bottom;
                (top, bottom, available_height - sum, margin_top, margin_bottom)
            }

            // If height is auto, then height is content height. Solve for the
            // non-auto value.
            (Specified(top), Auto, Auto) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let height = content_height;
                let sum = top + height + margin_top + margin_bottom;
                (top, available_height - sum, height, margin_top, margin_bottom)
            }
            (Auto, Specified(bottom), Auto) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let height = content_height;
                let sum = bottom + height + margin_top + margin_bottom;
                (available_height - sum, bottom, height, margin_top, margin_bottom)
            }

            (Auto, Auto, Specified(height)) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let top = static_position_top;
                let sum = top + height + margin_top + margin_bottom;
                (top, available_height - sum, height, margin_top, margin_bottom)
            }
        };
        HeightConstraintSolution::new(top, bottom, height, margin_top, margin_bottom)
    }

    /// Solve the vertical constraint equation for absolute replaced elements.
    ///
    /// Assumption: The used value for height has already been calculated.
    ///
    /// CSS Section 10.6.5
    /// Constraint equation:
    /// top + bottom + height + margin-top + margin-bottom
    /// = absolute containing block height - (vertical padding and border)
    /// [aka available_height]
    ///
    /// Return the solution for the equation.
    fn solve_vertical_constraints_abs_replaced(height: Au,
                                               top_margin: MaybeAuto,
                                               bottom_margin: MaybeAuto,
                                               top: MaybeAuto,
                                               bottom: MaybeAuto,
                                               _: Au,
                                               available_height: Au,
                                               static_y_offset: Au)
                                               -> HeightConstraintSolution {
        // Distance from the top edge of the Absolute Containing Block to the
        // top margin edge of a hypothetical box that would have been the
        // first box of the element.
        let static_position_top = static_y_offset;

        let (top, bottom, height, margin_top, margin_bottom) = match (top, bottom) {
            (Auto, Auto) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let top = static_position_top;
                let sum = top + height + margin_top + margin_bottom;
                (top, available_height - sum, height, margin_top, margin_bottom)
            }
            (Specified(top), Specified(bottom)) => {
                match (top_margin, bottom_margin) {
                    (Auto, Auto) => {
                        let total_margin_val = (available_height - top - bottom - height);
                        (top, bottom, height,
                         total_margin_val.scale_by(0.5),
                         total_margin_val.scale_by(0.5))
                    }
                    (Specified(margin_top), Auto) => {
                        let sum = top + bottom + height + margin_top;
                        (top, bottom, height, margin_top, available_height - sum)
                    }
                    (Auto, Specified(margin_bottom)) => {
                        let sum = top + bottom + height + margin_bottom;
                        (top, bottom, height, available_height - sum, margin_bottom)
                    }
                    (Specified(margin_top), Specified(margin_bottom)) => {
                        // Values are over-constrained. Ignore value for 'bottom'.
                        let sum = top + height + margin_top + margin_bottom;
                        (top, available_height - sum, height, margin_top, margin_bottom)
                    }
                }
            }

            // If only one is Auto, solve for it
            (Auto, Specified(bottom)) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let sum = bottom + height + margin_top + margin_bottom;
                (available_height - sum, bottom, height, margin_top, margin_bottom)
            }
            (Specified(top), Auto) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let sum = top + height + margin_top + margin_bottom;
                (top, available_height - sum, height, margin_top, margin_bottom)
            }
        };
        HeightConstraintSolution::new(top, bottom, height, margin_top, margin_bottom)
    }
}

/// The real assign-heights traversal for flows with position 'absolute'.
///
/// This is a traversal of an Absolute Flow tree.
/// - Relatively positioned flows and the Root flow start new Absolute flow trees.
/// - The kids of a flow in this tree will be the flows for which it is the
/// absolute Containing Block.
/// - Thus, leaf nodes and inner non-root nodes are all Absolute Flows.
///
/// A Flow tree can have several Absolute Flow trees (depending on the number
/// of relatively positioned flows it has).
///
/// Note that flows with position 'fixed' just form a flat list as they all
/// have the Root flow as their CB.
struct AbsoluteAssignHeightsTraversal<'a>(&'a mut LayoutContext);

impl<'a> PreorderFlowTraversal for AbsoluteAssignHeightsTraversal<'a> {
    #[inline]
    fn process(&mut self, flow: &mut Flow) -> bool {
        let block_flow = flow.as_block();

        // The root of the absolute flow tree is definitely not absolutely
        // positioned. Nothing to process here.
        if block_flow.is_root_of_absolute_flow_tree() {
            return true;
        }

        block_flow.calculate_abs_height_and_margins(**self);
        true
    }
}

/// The store-overflow traversal particular to absolute flows.
///
/// Propagate overflow up the Absolute flow tree and update overflow up to and
/// not including the root of the Absolute flow tree.
/// After that, it is up to the normal store-overflow traversal to propagate
/// it further up.
struct AbsoluteStoreOverflowTraversal<'a>{
    layout_context: &'a mut LayoutContext,
}

impl<'a> PostorderFlowTraversal for AbsoluteStoreOverflowTraversal<'a> {
    #[inline]
    fn process(&mut self, flow: &mut Flow) -> bool {
        // This will be taken care of by the normal store-overflow traversal.
        if flow.is_root_of_absolute_flow_tree() {
            return true;
        }

        flow.store_overflow(self.layout_context);
        true
    }
}

// A block formatting context.
pub struct BlockFlow {
    /// Data common to all flows.
    base: BaseFlow,

    /// The associated box.
    box_: Option<Box>,

    /// TODO: is_root should be a bit field to conserve memory.
    /// Whether this block flow is the root flow.
    is_root: bool,

    /// Static y offset of an absolute flow from its CB.
    static_y_offset: Au,

    /// Additional floating flow members.
    float: Option<~FloatedBlockInfo>
}

impl BlockFlow {
    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode)
                     -> BlockFlow {
        BlockFlow {
            base: BaseFlow::new((*node).clone()),
            box_: Some(Box::new(constructor, node)),
            is_root: false,
            static_y_offset: Au::new(0),
            float: None
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
            static_y_offset: Au::new(0),
            float: Some(~FloatedBlockInfo::new(float_kind))
        }
    }

    fn width_computer(&mut self) -> ~WidthAndMarginsComputer {
        if self.is_absolutely_positioned() {
            if self.is_replaced_content() {
                ~AbsoluteReplaced as ~WidthAndMarginsComputer
            } else {
                ~AbsoluteNonReplaced as ~WidthAndMarginsComputer
            }
        } else if self.is_float() {
            if self.is_replaced_content() {
                ~FloatReplaced as ~WidthAndMarginsComputer
            } else {
                ~FloatNonReplaced as ~WidthAndMarginsComputer
            }
        } else {
            if self.is_replaced_content() {
                ~BlockReplaced as ~WidthAndMarginsComputer
            } else {
                ~BlockNonReplaced as ~WidthAndMarginsComputer
            }
        }
    }

    /// Return this flow's box.
    fn box_<'a>(&'a mut self) -> &'a mut Box {
        match self.box_ {
            Some(ref mut box_) => box_,
            None => fail!("BlockFlow: no principal box found")
        }
    }

    /// Return the static x offset from the appropriate Containing Block for this flow.
    pub fn static_x_offset(&self) -> Au {
        if self.is_fixed() {
            self.base.fixed_static_x_offset
        } else {
            self.base.absolute_static_x_offset
        }
    }

    /// Return the size of the Containing Block for this flow.
    ///
    /// Right now, this only gets the Containing Block size for absolutely
    /// positioned elements.
    /// Note: Assume this is called in a top-down traversal, so it is ok to
    /// reference the CB.
    pub fn containing_block_size(&mut self, viewport_size: Size2D<Au>) -> Size2D<Au> {
        assert!(self.is_absolutely_positioned());
        if self.is_fixed() {
            // Initial containing block is the CB for the root
            viewport_size
        } else {
            let cb = self.base.absolute_cb.resolve().unwrap();
            cb.generated_cb_size()
        }
    }

    /// Traverse the Absolute flow tree in preorder.
    ///
    /// Traverse all your direct absolute descendants, who will then traverse
    /// their direct absolute descendants.
    /// Also, set the static y offsets for each descendant (using the value
    /// which was bubbled up during normal assign-height).
    ///
    /// Return true if the traversal is to continue or false to stop.
    fn traverse_preorder_absolute_flows<T:PreorderFlowTraversal>(&mut self,
                                                                 traversal: &mut T)
                                                                 -> bool {
        let flow = self as &mut Flow;
        if traversal.should_prune(flow) {
            return true
        }

        if !traversal.process(flow) {
            return false
        }

        let cb_top_edge_offset = flow.generated_cb_position().y;
        let mut descendant_offset_iter = mut_base(flow).abs_descendants.iter_with_offset();
        // Pass in the respective static y offset for each descendant.
        for (ref mut descendant_link, ref y_offset) in descendant_offset_iter {
            match descendant_link.resolve() {
                Some(flow) => {
                    let block = flow.as_block();
                    // The stored y_offset is wrt to the flow box.
                    // Translate it to the CB (which is the padding box).
                    block.static_y_offset = **y_offset - cb_top_edge_offset;
                    if !block.traverse_preorder_absolute_flows(traversal) {
                        return false
                    }
                }
                None => fail!("empty Rawlink to a descendant")
            }
        }

        true
    }

    /// Traverse the Absolute flow tree in postorder.
    ///
    /// Return true if the traversal is to continue or false to stop.
    fn traverse_postorder_absolute_flows<T:PostorderFlowTraversal>(&mut self,
                                                                   traversal: &mut T)
                                                                   -> bool {
        let flow = self as &mut Flow;
        if traversal.should_prune(flow) {
            return true
        }

        for descendant_link in mut_base(flow).abs_descendants.iter() {
            match descendant_link.resolve() {
                Some(abs_flow) => {
                    let block = abs_flow.as_block();
                    if !block.traverse_postorder_absolute_flows(traversal) {
                        return false
                    }
                }
                None => fail!("empty Rawlink to a descendant")
            }
        }

        traversal.process(flow)
    }

    /// Return true if this has a replaced box.
    ///
    /// The only two types of replaced boxes currently are text boxes and
    /// image boxes.
    fn is_replaced_content(&self) -> bool {
        match self.box_ {
            Some(ref box_) => {
                match box_.specific {
                    ScannedTextBox(_) | ImageBox(_) => true,
                    _ => false,
                }
            }
            None => false,
        }
    }

    pub fn teardown(&mut self) {
        for box_ in self.box_.iter() {
            box_.teardown();
        }
        self.box_ = None;
        self.float = None;
    }

    /// Return shrink-to-fit width.
    ///
    /// This is where we use the preferred widths and minimum widths
    /// calculated in the bubble-widths traversal.
    fn get_shrink_to_fit_width(&self, available_width: Au) -> Au {
        geometry::min(self.base.pref_width,
                      geometry::max(self.base.min_width, available_width))
    }

    /// Collect and update static y-offsets bubbled up by kids.
    ///
    /// This would essentially give us offsets of all absolutely positioned
    /// direct descendants and all fixed descendants, in tree order.
    ///
    /// Assume that this is called in a bottom-up traversal (specifically, the
    /// assign-height traversal). So, kids have their flow origin already set.
    /// In the case of absolute flow kids, they have their hypothetical box
    /// position already set.
    fn collect_static_y_offsets_from_kids(&mut self) {
        let mut abs_descendant_y_offsets = SmallVec0::new();
        let mut fixed_descendant_y_offsets = SmallVec0::new();

        for kid in self.base.child_iter() {
            let mut gives_abs_offsets = true;
            if kid.is_block_like() {
                let kid_block = kid.as_block();
                if kid_block.is_fixed() {
                    // It won't contribute any offsets for position 'absolute'
                    // descendants because it would be the CB for them.
                    gives_abs_offsets = false;
                    // Add the offset for the current fixed flow too.
                    fixed_descendant_y_offsets.push(kid_block.get_hypothetical_top_edge());
                } else if kid_block.is_absolutely_positioned() {
                    // It won't contribute any offsets for descendants because it
                    // would be the CB for them.
                    gives_abs_offsets = false;
                    // Give the offset for the current absolute flow alone.
                    abs_descendant_y_offsets.push(kid_block.get_hypothetical_top_edge());
                } else if kid_block.is_positioned() {
                    // It won't contribute any offsets because it would be the CB
                    // for the descendants.
                    gives_abs_offsets = false;
                }
            }

            if gives_abs_offsets {
                let kid_base = flow::mut_base(kid);
                // Consume all the static y-offsets bubbled up by kid.
                for y_offset in kid_base.abs_descendants.static_y_offsets.move_iter() {
                    // The offsets are wrt the kid flow box. Translate them to current flow.
                    y_offset = y_offset + kid_base.position.origin.y;
                    abs_descendant_y_offsets.push(y_offset);
                }
            }

            // Get all the fixed offsets.
            let kid_base = flow::mut_base(kid);
            // Consume all the static y-offsets bubbled up by kid.
            for y_offset in kid_base.fixed_descendants.static_y_offsets.move_iter() {
                // The offsets are wrt the kid flow box. Translate them to current flow.
                y_offset = y_offset + kid_base.position.origin.y;
                fixed_descendant_y_offsets.push(y_offset);
            }
        }
        self.base.abs_descendants.static_y_offsets = abs_descendant_y_offsets;
        self.base.fixed_descendants.static_y_offsets = fixed_descendant_y_offsets;
    }

    /// Assign height for current flow.
    ///
    /// + Collapse margins for flow's children and set in-flow child flows'
    /// y-coordinates now that we know their heights.
    /// + Calculate and set the height of the current flow.
    /// + Calculate height, vertical margins, and y-coordinate for the flow's
    /// box. Ideally, this should be calculated using CSS Section 10.6.7
    ///
    /// For absolute flows, store the calculated content height for the flow.
    /// Defer the calculation of the other values till a later traversal.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_height_block_base(&mut self, ctx: &mut LayoutContext, inorder: bool) {
        let mut cur_y = Au::new(0);
        let mut clearance = Au::new(0);
        // Offset to content edge of box_
        let mut top_offset = Au::new(0);
        let mut bottom_offset = Au::new(0);
        let mut left_offset = Au::new(0);

        for box_ in self.box_.iter() {
            // Note: Ignoring clearance for absolute flows as of now.
            if !self.is_absolutely_positioned() {
                clearance = match box_.clear() {
                    None => Au::new(0),
                    Some(clear) => {
                        self.base.floats.clearance(clear)
                    }
                };
            }

            top_offset = clearance + box_.margin.get().top + box_.border.get().top +
                box_.padding.get().top;
            cur_y = cur_y + top_offset;
            bottom_offset = box_.margin.get().bottom + box_.border.get().bottom +
                box_.padding.get().bottom;
            left_offset = box_.offset();
        }

        // Note: Ignoring floats for absolute flow as of now.
        if inorder && !self.is_absolutely_positioned() {
            // Floats for blocks work like this:
            // self.floats -> child[0].floats
            // visit child[0]
            // child[i-1].floats -> child[i].floats
            // visit child[i]
            // repeat until all children are visited.
            // last_child.floats -> self.floats (done at the end of this method)
            self.base.floats.translate(Point2D(-left_offset, -top_offset));
            let mut floats = self.base.floats.clone();
            for kid in self.base.child_iter() {
                flow::mut_base(kid).floats = floats;
                kid.assign_height_inorder(ctx);
                floats = flow::mut_base(kid).floats.clone();
            }
            self.base.floats = floats;
        }

        // The amount of margin that we can potentially collapse with
        let mut collapsible = Au::new(0);
        // How much to move up by to get to the beginning of
        // current kid flow.
        // Example: if previous sibling's margin-bottom is 20px and your
        // margin-top is 12px, the collapsed margin will be 20px. Since cur_y
        // will be at the bottom margin edge of the previous sibling, we have
        // to move up by 12px to get to our top margin edge. So, `collapsing`
        // will be set to 12px
        let mut collapsing = Au::new(0);
        let mut margin_top = Au::new(0);
        let mut margin_bottom = Au::new(0);
        let mut top_margin_collapsible = false;
        let mut bottom_margin_collapsible = false;
        let mut first_in_flow = true;
        // Margins for an absolutely positioned element do not collapse with
        // its children.
        if !self.is_absolutely_positioned() {
            for box_ in self.box_.iter() {
                if !self.is_root() && box_.border.get().top == Au(0)
                    && box_.padding.get().top == Au(0) {

                    collapsible = box_.margin.get().top;
                    top_margin_collapsible = true;
                }
                if !self.is_root() && box_.border.get().bottom == Au(0) &&
                    box_.padding.get().bottom == Au(0) {
                    bottom_margin_collapsible = true;
                }
                margin_top = box_.margin.get().top;
                margin_bottom = box_.margin.get().bottom;
            }
        }

        // At this point, cur_y is at the content edge of the flow's box_
        for kid in self.base.child_iter() {
            // At this point, cur_y is at bottom margin edge of previous kid

            if kid.is_absolutely_positioned() {
                // Assume that the `hypothetical box` for an absolute flow
                // starts immediately after the bottom margin edge of the
                // previous flow.
                kid.as_block().base.position.origin.y = cur_y;
                // Skip the collapsing for absolute flow kids and continue
                // with the next flow.
            } else {
                kid.collapse_margins(top_margin_collapsible,
                                     &mut first_in_flow,
                                     &mut margin_top,
                                     &mut top_offset,
                                     &mut collapsing,
                                     &mut collapsible);
                let child_node = flow::mut_base(kid);
                cur_y = cur_y - collapsing;
                // At this point, after moving up by `collapsing`, cur_y is at the
                // top margin edge of kid
                child_node.position.origin.y = cur_y;
                cur_y = cur_y + child_node.position.size.height;
                // At this point, cur_y is at the bottom margin edge of kid
            }
        }

        self.collect_static_y_offsets_from_kids();

        // The bottom margin collapses with its last in-flow block-level child's bottom margin
        // if the parent has no bottom border, no bottom padding.
        // The bottom margin for an absolutely positioned element does not
        // collapse even with its children.
        collapsing = if bottom_margin_collapsible && !self.is_absolutely_positioned() {
            if margin_bottom < collapsible {
                margin_bottom = collapsible;
            }
            collapsible
        } else {
            Au::new(0)
        };

        // TODO: A box's own margins collapse if the 'min-height' property is zero, and it has neither
        // top or bottom borders nor top or bottom padding, and it has a 'height' of either 0 or 'auto',
        // and it does not contain a line box, and all of its in-flow children's margins (if any) collapse.

        let screen_height = ctx.screen_size.height;

        let mut height = if self.is_root() {
            // FIXME(pcwalton): The max is taken here so that you can scroll the page, but this is
            // not correct behavior according to CSS 2.1 § 10.5. Instead I think we should treat
            // the root element as having `overflow: scroll` and use the layers-based scrolling
            // infrastructure to make it scrollable.
            Au::max(screen_height, cur_y)
        } else {
            // (cur_y - collapsing) will get you the the bottom margin-edge of
            // the bottom-most child.
            // top_offset: top margin-edge of the topmost child.
            // hence, height = content height
            cur_y - top_offset - collapsing
        };

        if self.is_absolutely_positioned() {
            // Store the content height for use in calculating the absolute
            // flow's dimensions later.
            for box_ in self.box_.iter() {
                let mut temp_position = box_.border_box.get();
                temp_position.size.height = height;
                box_.border_box.set(temp_position);
            }
            return;
        }

        for box_ in self.box_.iter() {
            let style = box_.style();

            // At this point, `height` is the height of the containing block, so passing `height`
            // as the second argument here effectively makes percentages relative to the containing
            // block per CSS 2.1 § 10.5.
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
            margin.top = margin_top;
            margin.bottom = margin_bottom;

            noncontent_height = box_.padding.get().top + box_.padding.get().bottom +
                box_.border.get().top + box_.border.get().bottom;

            position.origin.y = clearance + margin.top;
            // Border box height
            position.size.height = height + noncontent_height;

            noncontent_height = noncontent_height + clearance + margin.top + margin.bottom;

            box_.border_box.set(position);
            box_.margin.set(margin);
        }

        // Height of margin box + clearance
        self.base.position.size.height = height + noncontent_height;

        if inorder {
            let extra_height = height - (cur_y - top_offset) + bottom_offset;
            self.base.floats.translate(Point2D(left_offset, -extra_height));
        }

        if self.is_root_of_absolute_flow_tree() {
            // Assign heights for all flows in this Absolute flow tree.
            // This is preorder because the height of an absolute flow may depend on
            // the height of its CB, which may also be an absolute flow.
            self.traverse_preorder_absolute_flows(&mut AbsoluteAssignHeightsTraversal(ctx));
            // Store overflow for all absolute descendants.
            self.traverse_postorder_absolute_flows(&mut AbsoluteStoreOverflowTraversal {
                layout_context: ctx,
            });
        }
        if self.is_root() {
            self.assign_height_store_overflow_fixed_flows(ctx);
        }
    }

    /// Assign height for all fixed descendants.
    ///
    /// A flat iteration over all fixed descendants, passing their respective
    /// static y offsets.
    /// Also, store overflow immediately because nothing else depends on a
    /// fixed flow's height.
    fn assign_height_store_overflow_fixed_flows(&mut self, ctx: &mut LayoutContext) {
        assert!(self.is_root());
        let mut descendant_offset_iter = self.base.fixed_descendants.iter_with_offset();
        // Pass in the respective static y offset for each descendant.
        for (ref mut descendant_link, ref y_offset) in descendant_offset_iter {
            match descendant_link.resolve() {
                Some(fixed_flow) => {
                    {
                        let block = fixed_flow.as_block();
                        // The stored y_offset is wrt to the flow box (which
                        // will is also the CB, so it is the correct final value).
                        block.static_y_offset = **y_offset;
                        block.calculate_abs_height_and_margins(ctx);
                    }
                    fixed_flow.store_overflow(ctx);
                }
                None => fail!("empty Rawlink to a descendant")
            }
        }
    }

    /// Add placement information about current float flow for use by the parent.
    ///
    /// Also, use information given by parent about other floats to find out
    /// our relative position.
    ///
    /// This does not give any information about any float descendants because
    /// they do not affect elements outside of the subtree rooted at this
    /// float.
    ///
    /// This function is called on a kid flow by a parent.
    /// Therefore, assign_height_float was already called on this kid flow by
    /// the traversal function. So, the values used are well-defined.
    fn assign_height_float_inorder(&mut self) {
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

    /// Assign height for current flow.
    ///
    /// + Set in-flow child flows' y-coordinates now that we know their
    /// heights. This _doesn't_ do any margin collapsing for its children.
    /// + Calculate height and y-coordinate for the flow's box. Ideally, this
    /// should be calculated using CSS Section 10.6.7
    ///
    /// It does not calculate the height of the flow itself.
    fn assign_height_float(&mut self, ctx: &mut LayoutContext) {
        // Now that we've determined our height, propagate that out.
        let has_inorder_children = self.base.num_floats > 0;
        if has_inorder_children {
            let mut floats = Floats::new();
            for kid in self.base.child_iter() {
                flow::mut_base(kid).floats = floats;
                kid.assign_height_inorder(ctx);
                floats = flow::mut_base(kid).floats.clone();
            }

            // Floats establish a block formatting context, so we discard the output floats here.
            drop(floats);
        }
        let mut cur_y = Au(0);
        let mut top_offset = Au(0);

        for box_ in self.box_.iter() {
            top_offset = box_.margin.get().top + box_.border.get().top + box_.padding.get().top;
            cur_y = cur_y + top_offset;
        }

        // cur_y is now at the top content edge

        for kid in self.base.child_iter() {
            let child_base = flow::mut_base(kid);
            child_base.position.origin.y = cur_y;
            // cur_y is now at the bottom margin edge of kid
            cur_y = cur_y + child_base.position.size.height;
        }

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
    }

    /// Add display items for current block.
    ///
    /// Set the absolute position for children after doing any offsetting for
    /// position: relative.
    pub fn build_display_list_block<E:ExtraDisplayListData>(
                                    &mut self,
                                    builder: &DisplayListBuilder,
                                    container_block_size: &Size2D<Au>,
                                    absolute_cb_abs_position: Point2D<Au>,
                                    dirty: &Rect<Au>,
                                    index: uint,
                                    lists: &RefCell<DisplayListCollection<E>>)
                                    -> uint {

        if self.is_float() {
            self.build_display_list_float(builder, container_block_size, dirty, index, lists);
            return index;
        } else if self.is_absolutely_positioned() {
            return self.build_display_list_abs(builder, container_block_size,
                                        absolute_cb_abs_position,
                                        dirty, index, lists);
        }

        // FIXME: Shouldn't this be the abs_rect _after_ relative positioning?
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
            box_.build_display_list(builder, dirty, self.base.abs_position + rel_offset,
                                    (&*self) as &Flow, index, lists);
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

    /// Calculate and set the height, offsets, etc. for absolutely positioned flow.
    ///
    /// The layout for its in-flow children has been done during normal layout.
    /// This is just the calculation of:
    /// + height for the flow
    /// + y-coordinate of the flow wrt its Containing Block.
    /// + height, vertical margins, and y-coordinate for the flow's box.
    fn calculate_abs_height_and_margins(&mut self, ctx: &LayoutContext) {
        let containing_block_height = self.containing_block_size(ctx.screen_size).height;
        let static_y_offset = self.static_y_offset;

        for box_ in self.box_.iter() {
            // This is the stored content height value from assign-height
            let content_height = box_.border_box.get().size.height;

            let style = box_.style();

            let height_used_val = MaybeAuto::from_style(style.Box.get().height, containing_block_height);

            // Non-auto margin-top and margin-bottom values have already been
            // calculated during assign-width.
            let margin = box_.margin.get();
            let margin_top = match MaybeAuto::from_style(style.Margin.get().margin_top, Au(0)) {
                Auto => Auto,
                _ => Specified(margin.top)
            };
            let margin_bottom = match MaybeAuto::from_style(style.Margin.get().margin_bottom, Au(0)) {
                Auto => Auto,
                _ => Specified(margin.bottom)
            };

            let (top, bottom) =
                (MaybeAuto::from_style(style.PositionOffsets.get().top, containing_block_height),
                 MaybeAuto::from_style(style.PositionOffsets.get().bottom, containing_block_height));
            let available_height = containing_block_height - box_.noncontent_height();

            let solution = if self.is_replaced_content() {
                // Calculate used value of height just like we do for inline replaced elements.
                // TODO: Pass in the containing block height when Box's
                // assign-height can handle it correctly.
                box_.assign_replaced_height_if_necessary();
                // TODO: Right now, this content height value includes the
                // margin because of erroneous height calculation in Box_.
                // Check this when that has been fixed.
                let height_used_val = box_.border_box.get().size.height;
                HeightConstraintSolution::solve_vertical_constraints_abs_replaced(height_used_val,
                                                                                  margin_top,
                                                                                  margin_bottom,
                                                                                  top,
                                                                                  bottom,
                                                                                  content_height,
                                                                                  available_height,
                                                                                  static_y_offset)
            } else {
                HeightConstraintSolution::solve_vertical_constraints_abs_nonreplaced(
                    height_used_val,
                    margin_top,
                    margin_bottom,
                    top,
                    bottom,
                    content_height,
                    available_height,
                    static_y_offset)
            };

            let mut margin = box_.margin.get();
            margin.top = solution.margin_top;
            margin.bottom = solution.margin_bottom;
            box_.margin.set(margin);

            let mut position = box_.border_box.get();
            position.origin.y = box_.margin.get().top;
            // Border box height
            let border_and_padding = box_.noncontent_height();
            position.size.height = solution.height + border_and_padding;
            box_.border_box.set(position);

            self.base.position.origin.y = solution.top;
            self.base.position.size.height = solution.height + border_and_padding
                + solution.margin_top + solution.margin_bottom;
        }
    }

    /// Add display items for Absolutely Positioned flow.
    pub fn build_display_list_abs<E:ExtraDisplayListData>(
                                 &mut self,
                                 builder: &DisplayListBuilder,
                                 _: &Size2D<Au>,
                                 absolute_cb_abs_position: Point2D<Au>,
                                 dirty: &Rect<Au>,
                                 mut index: uint,
                                 lists: &RefCell<DisplayListCollection<E>>)
                                 -> uint {
        let flow_origin = if self.is_fixed() {
            // The viewport is initially at (0, 0).
            self.base.position.origin
        } else {
            // Absolute position of Containing Block + position of absolute flow
            // wrt Containing Block
            absolute_cb_abs_position + self.base.position.origin
        };

        if self.is_fixed() {
            lists.with_mut(|lists| {
                index = lists.lists.len();
                lists.add_list(DisplayList::<E>::new());
            });
        }

        // Set the absolute position, which will be passed down later as part
        // of containing block details for absolute descendants.
        self.base.abs_position = flow_origin;
        let abs_rect = Rect(flow_origin, self.base.position.size);
        if !abs_rect.intersects(dirty) {
            return index;
        }

        for box_ in self.box_.iter() {
            box_.build_display_list(builder, dirty, flow_origin, (&*self) as &Flow, index, lists);
        }

        // Go deeper into the flow tree.
        for child in self.base.child_iter() {
            let child_base = flow::mut_base(child);
            child_base.abs_position = flow_origin + child_base.position.origin;
        }

        index
    }

    /// Return the top outer edge of the Hypothetical Box for an absolute flow.
    ///
    /// This is wrt its parent flow box.
    ///
    /// During normal layout assign-height, the absolute flow's position is
    /// roughly set to its static position (the position it would have had in
    /// the normal flow).
    fn get_hypothetical_top_edge(&self) -> Au {
        self.base.position.origin.y
    }
}

impl Flow for BlockFlow {
    fn class(&self) -> FlowClass {
        BlockFlowClass
    }

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        self
    }

    fn is_store_overflow_delayed(&mut self) -> bool {
        self.is_absolutely_positioned()
    }

    /* Recursively (bottom-up) determine the context's preferred and
    minimum widths.  When called on this context, all child contexts
    have had their min/pref widths set. This function must decide
    min/pref widths based on child context widths and dimensions of
    any boxes it is responsible for flowing.  */

    /* TODO: absolute contexts */
    /* TODO: inline-blocks */
    fn bubble_widths(&mut self, _: &mut LayoutContext) {
        let mut min_width = Au::new(0);
        let mut pref_width = Au::new(0);
        let mut num_floats = 0;

        /* find max width from child block contexts */
        for child_ctx in self.base.child_iter() {
            assert!(child_ctx.is_block_flow() || child_ctx.is_inline_flow());

            let child_base = flow::mut_base(child_ctx);
            min_width = geometry::max(min_width, child_base.min_width);
            pref_width = geometry::max(pref_width, child_base.pref_width);
            num_floats = num_floats + child_base.num_floats;
        }

        if self.is_float() {
            self.base.num_floats = 1;
            self.float.get_mut_ref().floated_children = num_floats;
        } else {
            self.base.num_floats = num_floats;
        }

        /* if not an anonymous block context, add in block box's widths.
           these widths will not include child elements, just padding etc. */
        for box_ in self.box_.iter() {
            {
                // Can compute border width here since it doesn't depend on anything.
                box_.compute_borders(box_.style())
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

        if self.is_root() {
            debug!("Setting root position");
            self.base.position.origin = Au::zero_point();
            self.base.position.size.width = ctx.screen_size.width;
            self.base.floats = Floats::new();
            // Root element is not floated
            self.base.flags_info.flags.set_inorder(false);
        }

        // The position was set to the containing block by the flow's parent.
        let containing_block_width = self.base.position.size.width;
        let mut left_content_edge = Au::new(0);
        let mut content_width = containing_block_width;

        if self.is_float() {
            self.float.get_mut_ref().containing_width = containing_block_width;

            // Parent usually sets this, but floats are never inorder
            self.base.flags_info.flags.set_inorder(false);
        }

        let width_computer = self.width_computer();
        width_computer.compute_used_width(self, ctx, containing_block_width);

        for box_ in self.box_.iter() {
            // Move in from the left border edge
            left_content_edge = box_.border_box.get().origin.x
                + box_.padding.get().left + box_.border.get().left;
            let padding_and_borders = box_.padding.get().left + box_.padding.get().right +
                box_.border.get().left + box_.border.get().right;
            content_width = box_.border_box.get().size.width - padding_and_borders;
        }

        if self.is_float() {
            self.base.position.size.width = content_width;
        }

        let has_inorder_children = if self.is_float() {
            self.base.num_floats > 0
        } else {
            self.base.flags_info.flags.inorder() || self.base.num_floats > 0
        };

        let kid_abs_cb_x_offset;
        if self.is_positioned() {
            match self.box_ {
                Some(ref box_) => {
                    // Pass yourself as a new Containing Block
                    // The static x offset for any immediate kid flows will be the
                    // left padding
                    kid_abs_cb_x_offset = box_.padding.get().left;
                }
                None => fail!("BlockFlow: no principal box found"),
            }
        } else {
            // For kids, the left margin edge will be at our left content edge.
            // The current static offset is at our left margin
            // edge. So move in to the left content edge.
            kid_abs_cb_x_offset = self.base.absolute_static_x_offset + left_content_edge;
        }
        let kid_fixed_cb_x_offset = self.base.fixed_static_x_offset + left_content_edge;

        // FIXME(ksh8281): avoid copy
        let flags_info = self.base.flags_info.clone();
        for kid in self.base.child_iter() {
            assert!(kid.is_block_flow() || kid.is_inline_flow());

            if kid.is_block_flow() {
                let kid_block = kid.as_block();
                kid_block.base.absolute_static_x_offset = kid_abs_cb_x_offset;
                kid_block.base.fixed_static_x_offset = kid_fixed_cb_x_offset;
            }
            let child_base = flow::mut_base(kid);
            // Left margin edge of kid flow is at our left content edge
            child_base.position.origin.x = left_content_edge;
            // Width of kid flow is our content width
            child_base.position.size.width = content_width;
            child_base.flags_info.flags.set_inorder(has_inorder_children);

            if !child_base.flags_info.flags.inorder() {
                child_base.floats = Floats::new();
            }

            // Per CSS 2.1 § 16.3.1, text decoration propagates to all children in flow.
            //
            // TODO(pcwalton): When we have out-of-flow children, don't unconditionally propagate.

            child_base.flags_info.propagate_text_decoration_from_parent(&flags_info);
            child_base.flags_info.propagate_text_alignment_from_parent(&flags_info)
        }
    }

    /// This is called on kid flows by a parent.
    ///
    /// Hence, we can assume that assign_height has already been called on the
    /// kid (because of the bottom-up traversal).
    fn assign_height_inorder(&mut self, ctx: &mut LayoutContext) {
        if self.is_float() {
            debug!("assign_height_inorder_float: assigning height for float");
            self.assign_height_float_inorder();
        } else {
            debug!("assign_height_inorder: assigning height for block");
            self.assign_height_block_base(ctx, true);
        }
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        // Assign height for box if it is an image box.
        for box_ in self.box_.iter() {
            box_.assign_replaced_height_if_necessary();
        }

        if self.is_float() {
            debug!("assign_height_float: assigning height for float");
            self.assign_height_float(ctx);
        } else {
            debug!("assign_height: assigning height for block");
            // This is the only case in which a block flow can start an inorder
            // subtraversal.
            if self.is_root() && self.base.num_floats > 0 {
                self.assign_height_inorder(ctx);
                return;
            }
            self.assign_height_block_base(ctx, false);
        }
    }

    // CSS Section 8.3.1 - Collapsing Margins
    // `self`: the Flow whose margins we want to collapse.
    // `collapsing`: value to be set by this function. This tells us how much
    // of the top margin has collapsed with a previous margin.
    // `collapsible`: Potential collapsible margin at the bottom of this flow's box.
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

    fn mark_as_root(&mut self) {
        self.is_root = true
    }

    /// Return true if store overflow is delayed for this flow.
    ///
    /// Currently happens only for absolutely positioned flows.
    fn is_store_overflow_delayed(&mut self) -> bool {
        self.is_absolutely_positioned()
    }

    fn is_root(&self) -> bool {
        self.is_root
    }

    fn is_float(&self) -> bool {
        self.float.is_some()
    }

    /// The 'position' property of this flow.
    fn positioning(&self) -> position::T {
        match self.box_ {
            Some(ref box_) => {
                box_.style.get().Box.get().position
            }
            None => fail!("BlockFlow does not have a box_")
        }
    }

    /// Return true if this is the root of an Absolute flow tree.
    ///
    /// It has to be either relatively positioned or the Root flow.
    fn is_root_of_absolute_flow_tree(&self) -> bool {
        self.is_relatively_positioned() || self.is_root()
    }

    /// Return the dimensions of the CB generated _by_ this flow for absolute descendants.
    ///
    /// For Blocks, this will be the padding box.
    fn generated_cb_size(&self) -> Size2D<Au> {
        match self.box_ {
            Some(ref box_) => {
                box_.padding_box_size()
            }
            None => fail!("Containing Block must have a box")
        }
    }

    /// Return position of the CB generated by this flow from the start of this flow.
    fn generated_cb_position(&self) -> Point2D<Au> {
        match self.box_ {
            Some(ref box_) => {
                // Border box y coordinate + border top
                box_.border_box.get().origin + Point2D(box_.border.get().left, box_.border.get().top)}
            None => fail!("Containing Block must have a box")
        }
    }

    fn debug_str(&self) -> ~str {
        let txt = if self.is_float() {
            ~"FloatFlow: "
        } else if self.is_root() {
            ~"RootFlow: "
        } else {
            ~"BlockFlow: "
        };
        txt.append(match self.box_ {
            Some(ref rb) => rb.debug_str(),
            None => ~"",
        })
    }
}

/// The inputs for the widths-and-margins constraint equation.
struct WidthConstraintInput {
    computed_width: MaybeAuto,
    left_margin: MaybeAuto,
    right_margin: MaybeAuto,
    left: MaybeAuto,
    right: MaybeAuto,
    available_width: Au,
    static_x_offset: Au,
}

impl WidthConstraintInput {
    fn new(computed_width: MaybeAuto,
           left_margin: MaybeAuto,
           right_margin: MaybeAuto,
           left: MaybeAuto,
           right: MaybeAuto,
           available_width: Au,
           static_x_offset: Au)
           -> WidthConstraintInput {
        WidthConstraintInput {
            computed_width: computed_width,
            left_margin: left_margin,
            right_margin: right_margin,
            left: left,
            right: right,
            available_width: available_width,
            static_x_offset: static_x_offset,
        }
    }
}

/// The solutions for the widths-and-margins constraint equation.
struct WidthConstraintSolution {
    left: Au,
    right: Au,
    width: Au,
    margin_left: Au,
    margin_right: Au
}

impl WidthConstraintSolution {
    fn new(width: Au, margin_left: Au, margin_right: Au) -> WidthConstraintSolution {
        WidthConstraintSolution {
            left: Au(0),
            right: Au(0),
            width: width,
            margin_left: margin_left,
            margin_right: margin_right,
        }
    }

    fn for_absolute_flow(left: Au,
                         right: Au,
                         width: Au,
                         margin_left: Au,
                         margin_right: Au)
                         -> WidthConstraintSolution {
        WidthConstraintSolution {
            left: left,
            right: right,
            width: width,
            margin_left: margin_left,
            margin_right: margin_right,
        }
    }
}

// Trait to encapsulate the Width and Margin calculation.
//
// CSS Section 10.3
trait WidthAndMarginsComputer {
    /// Compute the inputs for the Width constraint equation.
    ///
    /// This is called only once to compute the initial inputs. For
    /// calculation involving min-width and max-width, we don't need to
    /// recompute these.
    fn compute_width_constraint_inputs(&self,
                                       block: &mut BlockFlow,
                                       parent_flow_width: Au,
                                       ctx: &mut LayoutContext)
                       -> WidthConstraintInput {
        let containing_block_width = self.containing_block_width(block, parent_flow_width, ctx);
        let computed_width = self.initial_computed_width(block, parent_flow_width, ctx);
        for box_ in block.box_.iter() {
            let style = box_.style();

            // The text alignment of a block flow is the text alignment of its box's style.
            block.base.flags_info.flags.set_text_align(style.InheritedText.get().text_align);

            box_.compute_padding(style, containing_block_width);

            // We calculate and set margin-top and margin-bottom here
            // because CSS 2.1 defines % on this wrt CB *width*.
            box_.compute_margin_top_bottom(containing_block_width);

            let (margin_left, margin_right) =
                (MaybeAuto::from_style(style.Margin.get().margin_left, containing_block_width),
                 MaybeAuto::from_style(style.Margin.get().margin_right, containing_block_width));

            let (left, right) =
                (MaybeAuto::from_style(style.PositionOffsets.get().left, containing_block_width),
                 MaybeAuto::from_style(style.PositionOffsets.get().right, containing_block_width));
            let available_width = containing_block_width - box_.noncontent_width();
            return WidthConstraintInput::new(computed_width,
                                             margin_left,
                                             margin_right,
                                             left,
                                             right,
                                             available_width,
                                             block.static_x_offset());
        }
        fail!("Block doesn't have a principal box")
    }

    /// Set the used values for width and margins got from the relevant constraint equation.
    ///
    /// This is called only once.
    ///
    /// Set:
    /// + used values for content width, left margin, and right margin for this flow's box.
    /// + x-coordinate of this flow's box.
    /// + x-coordinate of the flow wrt its Containing Block (if this is an absolute flow).
    fn set_width_constraint_solutions(&self,
                                      block: &mut BlockFlow,
                                      solution: WidthConstraintSolution) {
        let box_ = block.box_();
        let mut margin = box_.margin.get();
        margin.left = solution.margin_left;
        margin.right = solution.margin_right;
        box_.margin.set(margin);

        // The associated box is the border box of this flow.
        let mut position_ref = box_.border_box.borrow_mut();
        // Left border edge.
        position_ref.get().origin.x = box_.margin.get().left;

        // Border box width
        position_ref.get().size.width = solution.width + box_.noncontent_width();
    }

    /// Set the x coordinate of the given flow if it is absolutely positioned.
    fn set_flow_x_coord_if_necessary(&self, _: &mut BlockFlow, _: WidthConstraintSolution) {}

    /// Solve the width and margins constraints for this block flow.
    fn solve_width_constraints(&self,
                               block: &mut BlockFlow,
                               input: WidthConstraintInput)
                               -> WidthConstraintSolution;

    fn initial_computed_width(&self,
                              block: &mut BlockFlow,
                              parent_flow_width: Au,
                              ctx: &mut LayoutContext)
                              -> MaybeAuto {
        MaybeAuto::from_style(block.box_().style().Box.get().width,
                              self.containing_block_width(block, parent_flow_width, ctx))
    }

    fn containing_block_width(&self,
                              _: &mut BlockFlow,
                              parent_flow_width: Au,
                              _: &mut LayoutContext)
                              -> Au {
        parent_flow_width
    }

    /// Compute the used value of width, taking care of min-width and max-width.
    ///
    /// CSS Section 10.4: Minimum and Maximum widths
    fn compute_used_width(&self,
                          block: &mut BlockFlow,
                          ctx: &mut LayoutContext,
                          parent_flow_width: Au) {
        let mut input = self.compute_width_constraint_inputs(block, parent_flow_width, ctx);

        let containing_block_width = self.containing_block_width(block, parent_flow_width, ctx);

        let mut solution = self.solve_width_constraints(block, input);

        // If the tentative used width is greater than 'max-width', width should be recalculated,
        // but this time using the computed value of 'max-width' as the computed value for 'width'.
        match specified_or_none(block.box_().style().Box.get().max_width, containing_block_width) {
            Some(max_width) if max_width < solution.width => {
                input.computed_width = Specified(max_width);
                solution = self.solve_width_constraints(block, input);
            }
            _ => {}
        }

        // If the resulting width is smaller than 'min-width', width should be recalculated,
        // but this time using the value of 'min-width' as the computed value for 'width'.
        let computed_min_width = specified(block.box_().style().Box.get().min_width,
                                           containing_block_width);
        if computed_min_width > solution.width {
            input.computed_width = Specified(computed_min_width);
            solution = self.solve_width_constraints(block, input);
        }

        self.set_width_constraint_solutions(block, solution);
        self.set_flow_x_coord_if_necessary(block, solution);
    }

    /// Computes left and right margins and width.
    ///
    /// This is used by both replaced and non-replaced Blocks.
    ///
    /// CSS 2.1 Section 10.3.3.
    /// Constraint Equation: margin-left + margin-right + width = available_width
    /// where available_width = CB width - (horizontal border + padding)
    fn solve_block_width_constraints(&self,
                                     _: &mut BlockFlow,
                                     input: WidthConstraintInput)
                                     -> WidthConstraintSolution {
        let (computed_width, left_margin, right_margin, available_width) = (input.computed_width,
                                                                            input.left_margin,
                                                                            input.right_margin,
                                                                            input.available_width);

        // If width is not 'auto', and width + margins > available_width, all
        // 'auto' margins are treated as 0.
        let (left_margin, right_margin) = match computed_width {
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

        // Invariant: left_margin + width + right_margin == available_width
        let (left_margin, width, right_margin) = match (left_margin, computed_width, right_margin) {
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
        WidthConstraintSolution::new(width, left_margin, right_margin)
    }
}

/// The different types of Blocks.
///
/// They mainly differ in the way width and heights and margins are calculated
/// for them.
struct AbsoluteNonReplaced;
struct AbsoluteReplaced;
struct BlockNonReplaced;
struct BlockReplaced;
struct FloatNonReplaced;
struct FloatReplaced;

impl WidthAndMarginsComputer for AbsoluteNonReplaced {
    /// Solve the horizontal constraint equation for absolute non-replaced elements.
    ///
    /// CSS Section 10.3.7
    /// Constraint equation:
    /// left + right + width + margin-left + margin-right
    /// = absolute containing block width - (horizontal padding and border)
    /// [aka available_width]
    ///
    /// Return the solution for the equation.
    fn solve_width_constraints(&self,
                               block: &mut BlockFlow,
                               input: WidthConstraintInput)
                               -> WidthConstraintSolution {
        let WidthConstraintInput {
            computed_width,
            left_margin,
            right_margin,
            left,
            right,
            available_width,
            static_x_offset,
        } = input;

        // TODO: Check for direction of parent flow (NOT Containing Block)
        // when right-to-left is implemented.
        // Assume direction is 'ltr' for now

        // Distance from the left edge of the Absolute Containing Block to the
        // left margin edge of a hypothetical box that would have been the
        // first box of the element.
        let static_position_left = static_x_offset;

        let (left, right, width, margin_left, margin_right) = match (left, right, computed_width) {
            (Auto, Auto, Auto) => {
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                let left = static_position_left;
                // Now it is the same situation as left Specified and right
                // and width Auto.

                // Set right to zero to calculate width
                let width = block.get_shrink_to_fit_width(
                    available_width - (left + margin_l + margin_r));
                let sum = left + width + margin_l + margin_r;
                (left, available_width - sum, width, margin_l, margin_r)
            }
            (Specified(left), Specified(right), Specified(width)) => {
                match (left_margin, right_margin) {
                    (Auto, Auto) => {
                        let total_margin_val = (available_width - left - right - width);
                        if total_margin_val < Au(0) {
                            // margin-left becomes 0 because direction is 'ltr'.
                            // TODO: Handle 'rtl' when it is implemented.
                            (left, right, width, Au(0), total_margin_val)
                        } else {
                            // Equal margins
                            (left, right, width,
                             total_margin_val.scale_by(0.5),
                             total_margin_val.scale_by(0.5))
                        }
                    }
                    (Specified(margin_l), Auto) => {
                        let sum = left + right + width + margin_l;
                        (left, right, width, margin_l, available_width - sum)
                    }
                    (Auto, Specified(margin_r)) => {
                        let sum = left + right + width + margin_r;
                        (left, right, width, available_width - sum, margin_r)
                    }
                    (Specified(margin_l), Specified(margin_r)) => {
                        // Values are over-constrained.
                        // Ignore value for 'right' cos direction is 'ltr'.
                        // TODO: Handle 'rtl' when it is implemented.
                        let sum = left + width + margin_l + margin_r;
                        (left, available_width - sum, width, margin_l, margin_r)
                    }
                }
            }
            // For the rest of the cases, auto values for margin are set to 0

            // If only one is Auto, solve for it
            (Auto, Specified(right), Specified(width)) => {
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                let sum = right + width + margin_l + margin_r;
                (available_width - sum, right, width, margin_l, margin_r)
            }
            (Specified(left), Auto, Specified(width)) => {
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                let sum = left + width + margin_l + margin_r;
                (left, available_width - sum, width, margin_l, margin_r)
            }
            (Specified(left), Specified(right), Auto) => {
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                let sum = left + right + margin_l + margin_r;
                (left, right, available_width - sum, margin_l, margin_r)
            }

            // If width is auto, then width is shrink-to-fit. Solve for the
            // non-auto value.
            (Specified(left), Auto, Auto) => {
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                // Set right to zero to calculate width
                let width = block.get_shrink_to_fit_width(
                    available_width - (left + margin_l + margin_r));
                let sum = left + width + margin_l + margin_r;
                (left, available_width - sum, width, margin_l, margin_r)
            }
            (Auto, Specified(right), Auto) => {
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                // Set left to zero to calculate width
                let width = block.get_shrink_to_fit_width(
                    available_width - (right + margin_l + margin_r));
                let sum = right + width + margin_l + margin_r;
                (available_width - sum, right, width, margin_l, margin_r)
            }

            (Auto, Auto, Specified(width)) => {
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                // Setting 'left' to static position because direction is 'ltr'.
                // TODO: Handle 'rtl' when it is implemented.
                let left = static_position_left;
                let sum = left + width + margin_l + margin_r;
                (left, available_width - sum, width, margin_l, margin_r)
            }
        };
        WidthConstraintSolution::for_absolute_flow(left, right, width, margin_left, margin_right)
    }

    fn containing_block_width(&self, block: &mut BlockFlow, _: Au, ctx: &mut LayoutContext) -> Au {
        block.containing_block_size(ctx.screen_size).width
    }

    fn set_flow_x_coord_if_necessary(&self, block: &mut BlockFlow, solution: WidthConstraintSolution) {
        // Set the x-coordinate of the absolute flow wrt to its containing block.
        block.base.position.origin.x = solution.left;
    }
}

impl WidthAndMarginsComputer for AbsoluteReplaced {
    /// Solve the horizontal constraint equation for absolute replaced elements.
    ///
    /// `static_x_offset`: total offset of current flow's hypothetical
    /// position (static position) from its actual Containing Block.
    ///
    /// CSS Section 10.3.8
    /// Constraint equation:
    /// left + right + width + margin-left + margin-right
    /// = absolute containing block width - (horizontal padding and border)
    /// [aka available_width]
    ///
    /// Return the solution for the equation.
    fn solve_width_constraints(&self,
                               _: &mut BlockFlow,
                               input: WidthConstraintInput)
                               -> WidthConstraintSolution {
        let WidthConstraintInput {
            computed_width,
            left_margin,
            right_margin,
            left,
            right,
            available_width,
            static_x_offset,
        } = input;
        // TODO: Check for direction of static-position Containing Block (aka
        // parent flow, _not_ the actual Containing Block) when right-to-left
        // is implemented
        // Assume direction is 'ltr' for now
        // TODO: Handle all the cases for 'rtl' direction.

        let width = match computed_width {
            Specified(w) => w,
            _ => fail!("{} {}",
                       "The used value for width for absolute replaced flow",
                       "should have already been calculated by now.")
        };

        // Distance from the left edge of the Absolute Containing Block to the
        // left margin edge of a hypothetical box that would have been the
        // first box of the element.
        let static_position_left = static_x_offset;

        let (left, right, width, margin_left, margin_right) = match (left, right) {
            (Auto, Auto) => {
                let left = static_position_left;
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                let sum = left + width + margin_l + margin_r;
                (left, available_width - sum, width, margin_l, margin_r)
            }
            // If only one is Auto, solve for it
            (Auto, Specified(right)) => {
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                let sum = right + width + margin_l + margin_r;
                (available_width - sum, right, width, margin_l, margin_r)
            }
            (Specified(left), Auto) => {
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                let sum = left + width + margin_l + margin_r;
                (left, available_width - sum, width, margin_l, margin_r)
            }
            (Specified(left), Specified(right)) => {
                match (left_margin, right_margin) {
                    (Auto, Auto) => {
                        let total_margin_val = (available_width - left - right - width);
                        if total_margin_val < Au(0) {
                            // margin-left becomes 0 because direction is 'ltr'.
                            (left, right, width, Au(0), total_margin_val)
                        } else {
                            // Equal margins
                            (left, right, width,
                             total_margin_val.scale_by(0.5),
                             total_margin_val.scale_by(0.5))
                        }
                    }
                    (Specified(margin_l), Auto) => {
                        let sum = left + right + width + margin_l;
                        (left, right, width, margin_l, available_width - sum)
                    }
                    (Auto, Specified(margin_r)) => {
                        let sum = left + right + width + margin_r;
                        (left, right, width, available_width - sum, margin_r)
                    }
                    (Specified(margin_l), Specified(margin_r)) => {
                        // Values are over-constrained.
                        // Ignore value for 'right' cos direction is 'ltr'.
                        let sum = left + width + margin_l + margin_r;
                        (left, available_width - sum, width, margin_l, margin_r)
                    }
                }
            }
        };
        WidthConstraintSolution::for_absolute_flow(left, right, width, margin_left, margin_right)
    }

    /// Calculate used value of width just like we do for inline replaced elements.
    fn initial_computed_width(&self,
                              block: &mut BlockFlow,
                              _: Au,
                              ctx: &mut LayoutContext)
                              -> MaybeAuto {
        let containing_block_width = block.containing_block_size(ctx.screen_size).width;
        let box_ = block.box_();
        box_.assign_replaced_width_if_necessary(containing_block_width);
        // For replaced absolute flow, the rest of the constraint solving will
        // take width to be specified as the value computed here.
        Specified(box_.content_width())
    }

    fn containing_block_width(&self, block: &mut BlockFlow, _: Au, ctx: &mut LayoutContext) -> Au {
        block.containing_block_size(ctx.screen_size).width
    }

    fn set_flow_x_coord_if_necessary(&self, block: &mut BlockFlow, solution: WidthConstraintSolution) {
        // Set the x-coordinate of the absolute flow wrt to its containing block.
        block.base.position.origin.x = solution.left;
    }
}

impl WidthAndMarginsComputer for BlockNonReplaced {
    /// Compute left and right margins and width.
    fn solve_width_constraints(&self,
                               block: &mut BlockFlow,
                               input: WidthConstraintInput)
                               -> WidthConstraintSolution {
        self.solve_block_width_constraints(block, input)
    }
}

impl WidthAndMarginsComputer for BlockReplaced {
    /// Compute left and right margins and width.
    ///
    /// Width has already been calculated. We now calculate the margins just
    /// like for non-replaced blocks.
    fn solve_width_constraints(&self,
                               block: &mut BlockFlow,
                               input: WidthConstraintInput)
                               -> WidthConstraintSolution {
        match input.computed_width {
            Specified(_) => {},
            Auto => fail!("BlockReplaced: width should have been computed by now")
        };
        self.solve_block_width_constraints(block, input)
    }

    /// Calculate used value of width just like we do for inline replaced elements.
    fn initial_computed_width(&self,
                              block: &mut BlockFlow,
                              parent_flow_width: Au,
                              _: &mut LayoutContext)
                              -> MaybeAuto {
        let box_ = block.box_();
        box_.assign_replaced_width_if_necessary(parent_flow_width);
        // For replaced block flow, the rest of the constraint solving will
        // take width to be specified as the value computed here.
        Specified(box_.content_width())
    }

}

impl WidthAndMarginsComputer for FloatNonReplaced {
    /// CSS Section 10.3.5
    ///
    /// If width is computed as 'auto', the used value is the 'shrink-to-fit' width.
    fn solve_width_constraints(&self,
                               block: &mut BlockFlow,
                               input: WidthConstraintInput)
                               -> WidthConstraintSolution {
        let (computed_width, left_margin, right_margin, available_width) = (input.computed_width,
                                                                            input.left_margin,
                                                                            input.right_margin,
                                                                            input.available_width);
        let margin_left = left_margin.specified_or_zero();
        let margin_right = right_margin.specified_or_zero();
        let available_width_float = available_width - margin_left - margin_right;
        let shrink_to_fit = block.get_shrink_to_fit_width(available_width_float);
        let width = computed_width.specified_or_default(shrink_to_fit);
        debug!("assign_widths_float -- width: {}", width);
        WidthConstraintSolution::new(width, margin_left, margin_right)
    }
}

impl WidthAndMarginsComputer for FloatReplaced {
    /// CSS Section 10.3.5
    ///
    /// If width is computed as 'auto', the used value is the 'shrink-to-fit' width.
    fn solve_width_constraints(&self,
                               _: &mut BlockFlow,
                               input: WidthConstraintInput)
                               -> WidthConstraintSolution {
        let (computed_width, left_margin, right_margin) = (input.computed_width,
                                                           input.left_margin,
                                                           input.right_margin);
        let margin_left = left_margin.specified_or_zero();
        let margin_right = right_margin.specified_or_zero();
        let width = match computed_width {
            Specified(w) => w,
            Auto => fail!("FloatReplaced: width should have been computed by now")
        };
        debug!("assign_widths_float -- width: {}", width);
        WidthConstraintSolution::new(width, margin_left, margin_right)
    }

    /// Calculate used value of width just like we do for inline replaced elements.
    fn initial_computed_width(&self,
                              block: &mut BlockFlow,
                              parent_flow_width: Au,
                              _: &mut LayoutContext)
                              -> MaybeAuto {
        let box_ = block.box_();
        box_.assign_replaced_width_if_necessary(parent_flow_width);
        // For replaced block flow, the rest of the constraint solving will
        // take width to be specified as the value computed here.
        Specified(box_.content_width())
    }
}
