/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Creates CSS boxes from a DOM tree.

use layout::aux::LayoutAuxMethods;
use layout::block::BlockFlowData;
use layout::box::{GenericRenderBoxClass, ImageRenderBox, ImageRenderBoxClass, RenderBox};
use layout::box::{RenderBoxBase, RenderBoxType, RenderBox_Generic, RenderBox_Image};
use layout::box::{RenderBox_Text, UnscannedTextRenderBox, UnscannedTextRenderBoxClass};
use layout::context::LayoutContext;
use layout::flow::{AbsoluteFlow, BlockFlow, FloatFlow, Flow_Absolute, Flow_Block, Flow_Float};
use layout::flow::{Flow_Inline, Flow_InlineBlock, Flow_Root, Flow_Table, FlowContext};
use layout::flow::{FlowContextType, FlowData, InlineBlockFlow, InlineFlow, TableFlow};
use layout::inline::{InlineFlowData, InlineLayout};
use css::node_style::StyledNode;

use newcss::values::{CSSDisplay, CSSDisplayBlock, CSSDisplayInline, CSSDisplayInlineBlock};
use newcss::values::{CSSDisplayTable, CSSDisplayInlineTable, CSSDisplayListItem};
use newcss::values::{CSSDisplayTableRowGroup, CSSDisplayTableHeaderGroup, CSSDisplayTableFooterGroup};
use newcss::values::{CSSDisplayTableRow, CSSDisplayTableColumnGroup, CSSDisplayTableColumn};
use newcss::values::{CSSDisplayTableCell, CSSDisplayTableCaption};
use newcss::values::{CSSDisplayNone};
use script::dom::element::*;
use script::dom::node::{AbstractNode, CommentNodeTypeId, DoctypeNodeTypeId};
use script::dom::node::{ElementNodeTypeId, LayoutView, TextNodeTypeId};
use servo_util::range::Range;
use servo_util::tree::{TreeNodeRef, TreeUtils};

pub struct LayoutTreeBuilder {
    root_flow: Option<FlowContext>,
    next_cid: int,
    next_bid: int,
}

pub impl LayoutTreeBuilder {
    fn new() -> LayoutTreeBuilder {
        LayoutTreeBuilder {
            root_flow: None,
            next_cid: -1,
            next_bid: -1,
        }
    }
}

// helper object for building the initial box list and making the
// mapping between DOM nodes and boxes.
struct BoxGenerator {
    flow: FlowContext,
    range_stack: ~[uint],
}

enum InlineSpacerSide {
    LogicalBefore,
    LogicalAfter,
}

priv fn simulate_UA_display_rules(node: AbstractNode<LayoutView>) -> CSSDisplay {
    // FIXME
    /*let resolved = do node.aux |nd| {
        match nd.style.display_type {
            Inherit | Initial => DisplayInline, // TODO: remove once resolve works
            Specified(v) => v
        }
    };*/

    let resolved = CSSDisplayInline;
    if resolved == CSSDisplayNone {
        return resolved;
    }

    match node.type_id() {
        DoctypeNodeTypeId | CommentNodeTypeId => CSSDisplayNone,
        TextNodeTypeId => CSSDisplayInline,
        ElementNodeTypeId(element_type_id) => {
            match element_type_id {
                HTMLHeadElementTypeId |
                HTMLScriptElementTypeId => CSSDisplayNone,
                HTMLParagraphElementTypeId |
                HTMLDivElementTypeId |
                HTMLBodyElementTypeId |
                HTMLHeadingElementTypeId |
                HTMLHtmlElementTypeId |
                HTMLUListElementTypeId |
                HTMLOListElementTypeId => CSSDisplayBlock,
                _ => resolved
            }
        }
    }
}

impl BoxGenerator {
    /* Debug ids only */

    fn new(flow: FlowContext) -> BoxGenerator {
        debug!("Creating box generator for flow: %s", flow.debug_str());
        BoxGenerator {
            flow: flow,
            range_stack: ~[]
        }
    }

    /* Whether "spacer" boxes are needed to stand in for this DOM node */
    fn inline_spacers_needed_for_node(&self, _: AbstractNode<LayoutView>) -> bool {
        return false;
    }

    // TODO: implement this, generating spacer 
    fn make_inline_spacer_for_node_side(&self,
                                        _: &LayoutContext,
                                        _: AbstractNode<LayoutView>,
                                        _: InlineSpacerSide)
                                        -> Option<RenderBox> {
        None
    }

    pub fn push_node(&mut self,
                     ctx: &LayoutContext,
                     node: AbstractNode<LayoutView>,
                     builder: &mut LayoutTreeBuilder) {
        debug!("BoxGenerator[f%d]: pushing node: %s", self.flow.id(), node.debug_str());

        // first, determine the box type, based on node characteristics
        let simulated_display = simulate_UA_display_rules(node);
        // TODO: remove this once UA styles work
        let box_type = self.decide_box_type(node, simulated_display);

        debug!("BoxGenerator[f%d]: point a", self.flow.id());

        // depending on flow, make a box for this node.
        match self.flow {
            InlineFlow(inline) => {
                let node_range_start = inline.boxes.len();
                self.range_stack.push(node_range_start);

                // if a leaf, make a box.
                if node.is_leaf() {
                    let new_box = self.make_box(ctx, box_type, node, self.flow, builder);
                    inline.boxes.push(new_box);
                } else if self.inline_spacers_needed_for_node(node) {
                    // else, maybe make a spacer for "left" margin, border, padding
                    for self.make_inline_spacer_for_node_side(ctx, node, LogicalBefore).each
                            |spacer: &RenderBox| {
                        inline.boxes.push(*spacer);
                    }
                }
                // TODO: cases for inline-block, etc.
            },
            BlockFlow(block) => {
                debug!("BoxGenerator[f%d]: point b", block.common.id);
                let new_box = self.make_box(ctx, box_type, node, self.flow, builder);

                debug!("BoxGenerator[f%d]: attaching box[b%d] to block flow (node: %s)",
                       block.common.id,
                       new_box.id(),
                       node.debug_str());

                assert!(block.box.is_none());
                block.box = Some(new_box);
            },
            _ => warn!("push_node() not implemented for flow f%d", self.flow.id()),
        }
    }

    pub fn pop_node(&mut self,
                    ctx: &LayoutContext,
                    node: AbstractNode<LayoutView>) {
        debug!("BoxGenerator[f%d]: popping node: %s", self.flow.id(), node.debug_str());

        match self.flow {
            InlineFlow(inline) => {
                let inline = &mut *inline;

                if self.inline_spacers_needed_for_node(node) {
                    // If this non-leaf box generates extra horizontal spacing, add a SpacerBox for
                    // it.
                    let result = self.make_inline_spacer_for_node_side(ctx, node, LogicalAfter);
                    for result.each |spacer| {
                        let boxes = &mut self.flow.inline().boxes;
                        boxes.push(*spacer);
                    }
                }
                let mut node_range: Range = Range::new(self.range_stack.pop(), 0);
                node_range.extend_to(inline.boxes.len());

                if node_range.length() == 0 {
                    warn!("node range length is zero?!")
                }

                debug!("BoxGenerator: adding element range=%?", node_range);
                inline.elems.add_mapping(node, &node_range);
            },
            BlockFlow(*) => assert!(self.range_stack.len() == 0),
            _ => warn!("pop_node() not implemented for flow %?", self.flow.id()),
        }
    }

    /// Disambiguate between different methods here instead of inlining, since each case has very
    /// different complexity.
    fn make_box(&mut self,
                layout_ctx: &LayoutContext,
                ty: RenderBoxType,
                node: AbstractNode<LayoutView>,
                flow_context: FlowContext,
                builder: &mut LayoutTreeBuilder)
                -> RenderBox {
        let base = RenderBoxBase::new(node, flow_context, builder.next_box_id());
        let result = match ty {
            RenderBox_Generic => GenericRenderBoxClass(@mut base),
            RenderBox_Text => UnscannedTextRenderBoxClass(@mut UnscannedTextRenderBox::new(base)),
            RenderBox_Image => self.make_image_box(layout_ctx, node, base),
        };
        debug!("BoxGenerator: created box: %s", result.debug_str());
        result
    }

    fn make_image_box(&mut self,
                      layout_ctx: &LayoutContext,
                      node: AbstractNode<LayoutView>,
                      base: RenderBoxBase)
                      -> RenderBox {
        assert!(node.is_image_element());

        do node.with_imm_image_element |image_element| {
            if image_element.image.is_some() {
                // FIXME(pcwalton): Don't copy URLs.
                let url = copy *image_element.image.get_ref();
                ImageRenderBoxClass(@mut ImageRenderBox::new(base, url, layout_ctx.image_cache))
            } else {
                info!("Tried to make image box, but couldn't find image. Made generic box \
                       instead.");
                GenericRenderBoxClass(@mut base)
            }
        }
    }

    fn decide_box_type(&self, node: AbstractNode<LayoutView>, _: CSSDisplay) -> RenderBoxType {
        if node.is_text() {
            RenderBox_Text
        } else if node.is_image_element() {
            do node.with_imm_image_element |image_element| {
                match image_element.image {
                    Some(_) => RenderBox_Image,
                    None => RenderBox_Generic,
                }
            }
        } else if node.is_element() {
            RenderBox_Generic
        } else {
            fail!(~"Hey, doctypes and comments shouldn't get here! They are display:none!")
        }
    }

}


pub impl LayoutTreeBuilder {
    /* Debug-only ids */
    fn next_flow_id(&mut self) -> int { self.next_cid += 1; self.next_cid }
    fn next_box_id(&mut self) -> int { self.next_bid += 1; self.next_bid }

    /// Creates necessary box(es) and flow context(s) for the current DOM node,
    /// and recurses on its children.
    fn construct_recursively(&mut self,
                             layout_ctx: &LayoutContext,
                             cur_node: AbstractNode<LayoutView>,
                             parent_generator: @mut BoxGenerator,
                             prev_sibling_generator: Option<@mut BoxGenerator>)
                             -> Option<@mut BoxGenerator> {
        debug!("Considering node: %s", cur_node.debug_str());

        let this_generator = match self.box_generator_for_node(cur_node, 
                                                                   parent_generator,
                                                                   prev_sibling_generator) {
            Some(gen) => gen,
            None => { return prev_sibling_generator; }
        };

        debug!("point a: %s", cur_node.debug_str());
        this_generator.push_node(layout_ctx, cur_node, self);
        debug!("point b: %s", cur_node.debug_str());

        // recurse on child nodes.
        let mut prev_generator: Option<@mut BoxGenerator> = None;
        for cur_node.each_child |child_node| {
            prev_generator = self.construct_recursively(layout_ctx, child_node, this_generator, prev_generator);
        }

        this_generator.pop_node(layout_ctx, cur_node);
        self.simplify_children_of_flow(layout_ctx, &mut this_generator.flow);

        // store reference to the flow context which contains any
        // boxes that correspond to child_flow.node. These boxes may
        // eventually be elided or split, but the mapping between
        // nodes and FlowContexts should not change during layout.
        let flow: &FlowContext = &this_generator.flow;
        for flow.each_child |child_flow| {
            do child_flow.with_base |child_node| {
                let dom_node = child_node.node;
                assert!(dom_node.has_layout_data());
                dom_node.layout_data().flow = Some(child_flow);
            }
        }
        Some(this_generator)
    }

    fn box_generator_for_node(&mut self, 
                              node: AbstractNode<LayoutView>, 
                              parent_generator: @mut BoxGenerator,
                              sibling_generator: Option<@mut BoxGenerator>)
                              -> Option<@mut BoxGenerator> {

        fn is_root(node: AbstractNode<LayoutView>) -> bool {
            match node.parent_node() {
                None => true,
                Some(_) => false
            }
        }
        let display = if (node.is_element()) {
            match node.style().display(is_root(node)) {
                CSSDisplayNone => return None, // tree ends here if 'display: none'
                // TODO(eatkinson) these are hacks so that the code doesn't crash
                // when unsupported display values are used. They should be deleted
                // as they are implemented.
                CSSDisplayListItem => CSSDisplayBlock,
                CSSDisplayTable => CSSDisplayBlock,
                CSSDisplayInlineTable => CSSDisplayInlineBlock,
                CSSDisplayTableRowGroup => CSSDisplayBlock,
                CSSDisplayTableHeaderGroup => CSSDisplayBlock,
                CSSDisplayTableFooterGroup => CSSDisplayBlock,
                CSSDisplayTableRow => CSSDisplayBlock,
                CSSDisplayTableColumnGroup => return None,
                CSSDisplayTableColumn => return None,
                CSSDisplayTableCell => CSSDisplayBlock,
                CSSDisplayTableCaption => CSSDisplayBlock,
                v => v
            }
        } else {
            match node.type_id() {

                ElementNodeTypeId(_) => CSSDisplayInline,
                TextNodeTypeId => CSSDisplayInline,
                DoctypeNodeTypeId | CommentNodeTypeId => return None,
            }
        };

        let sibling_flow: Option<FlowContext> = match sibling_generator {
            None => None,
            Some(gen) => Some(gen.flow)
        };

        let new_generator = match (display, parent_generator.flow, sibling_flow) { 
            (CSSDisplayBlock, BlockFlow(info), _) => match (info.is_root, node.parent_node()) {
                // If this is the root node, then use the root flow's
                // context. Otherwise, make a child block context.
                (true, Some(_)) => { self.create_child_generator(node, parent_generator, Flow_Block) }
                (true, None)    => { parent_generator }
                (false, _)      => {
                    self.create_child_generator(node, parent_generator, Flow_Block)
                }
            },
            // Inlines that are children of inlines are part of the same flow
            (CSSDisplayInline, InlineFlow(*), _) => parent_generator,
            (CSSDisplayInlineBlock, InlineFlow(*), _) => parent_generator,

            // Inlines that are children of blocks create new flows if their
            // previous sibling was a block.
            (CSSDisplayInline, BlockFlow(*), Some(BlockFlow(*))) |
            (CSSDisplayInlineBlock, BlockFlow(*), Some(BlockFlow(*))) => {
                self.create_child_generator(node, parent_generator, Flow_Inline)
            }

            // Inlines whose previous sibling was not a block try to use their
            // sibling's flow context.
            (CSSDisplayInline, BlockFlow(*), _) |
            (CSSDisplayInlineBlock, BlockFlow(*), _) => {
                self.create_child_generator_if_needed(node, 
                                                      parent_generator, 
                                                      sibling_generator, 
                                                      Flow_Inline)
            }

            // TODO(eatkinson): blocks that are children of inlines need
            // to split their parent flows.
            //
            // TODO(eatkinson): floats and positioned elements.
            _ => parent_generator
        };

        Some(new_generator)
    }

    fn create_child_generator(&mut self,
                              node: AbstractNode<LayoutView>,
                              parent_generator: @mut BoxGenerator,
                              ty: FlowContextType)
                              -> @mut BoxGenerator {

        let new_flow = self.make_flow(ty, node);
        parent_generator.flow.add_child(new_flow);

        @mut BoxGenerator::new(new_flow)
    }

    fn create_child_generator_if_needed(&mut self,
                                        node: AbstractNode<LayoutView>,
                                        parent_generator: @mut BoxGenerator,
                                        maybe_generator: Option<@mut BoxGenerator>,
                                        ty: FlowContextType)
                                        -> @mut BoxGenerator {

        match maybe_generator {
            None => self.create_child_generator(node, parent_generator, ty),
            Some(gen) => gen
        }
    }

    /// Fix up any irregularities such as:
    ///
    /// * split inlines (CSS 2.1 Section 9.2.1.1)
    /// * elide non-preformatted whitespace-only text boxes and their flows (CSS 2.1 Section
    ///   9.2.2.1).
    ///
    /// The latter can only be done immediately adjacent to, or at the beginning or end of a block
    /// flow. Otherwise, the whitespace might affect whitespace collapsing with adjacent text.
    fn simplify_children_of_flow(&self, _: &LayoutContext, parent_flow: &mut FlowContext) {
        match *parent_flow {
            InlineFlow(*) => {
                let mut found_child_inline = false;
                let mut found_child_block = false;

                let flow = *parent_flow;
                for flow.each_child |child_ctx: FlowContext| {
                    match child_ctx {
                        InlineFlow(*) | InlineBlockFlow(*) => found_child_inline = true,
                        BlockFlow(*) => found_child_block = true,
                        _ => {}
                    }
                }

                if found_child_block && found_child_inline {
                    self.fixup_split_inline(*parent_flow)
                }
            },
            BlockFlow(*) => {
                // FIXME: this will create refcounted cycles between the removed flow and any
                // of its RenderBox or FlowContext children, and possibly keep alive other junk

                // check first/last child for whitespace-ness
                let first_child = do parent_flow.with_base |parent_node| {
                    parent_node.first_child
                };
                for first_child.each |&first_flow| {
                    if first_flow.starts_inline_flow() {
                        // FIXME: workaround for rust#6393
                        let mut do_remove = false;
                        {
                            let boxes = &first_flow.inline().boxes;
                            if boxes.len() == 1 && boxes[0].is_whitespace_only() {
                                debug!("LayoutTreeBuilder: pruning whitespace-only first child \
                                        flow f%d from parent f%d", 
                                       first_flow.id(),
                                       parent_flow.id());
                                do_remove = true;
                            }
                        }
                        if (do_remove) { 
                            (*parent_flow).remove_child(first_flow);
                        }
                    }
                }

                let last_child = do parent_flow.with_base |parent_node| {
                    parent_node.last_child
                };
                for last_child.each |&last_flow| {
                    if last_flow.starts_inline_flow() {
                        // FIXME: workaround for rust#6393
                        let mut do_remove = false;
                        {
                            let boxes = &last_flow.inline().boxes;
                            if boxes.len() == 1 && boxes.last().is_whitespace_only() {
                                debug!("LayoutTreeBuilder: pruning whitespace-only last child \
                                        flow f%d from parent f%d", 
                                       last_flow.id(),
                                       parent_flow.id());
                                do_remove = true;
                            }
                        }
                        if (do_remove) {
                            (*parent_flow).remove_child(last_flow);
                        }
                    }
                }
            },
            _ => {}
        }
    }

    fn fixup_split_inline(&self, _: FlowContext) {
        // TODO: finish me. 
        fail!(~"TODO: handle case where an inline is split by a block")
    }

    /// Entry point for box creation. Should only be called on the root DOM element.
    fn construct_trees(&mut self, layout_ctx: &LayoutContext, root: AbstractNode<LayoutView>)
                       -> Result<FlowContext, ()> {
        let new_flow = self.make_flow(Flow_Root, root);
        let new_generator = @mut BoxGenerator::new(new_flow);

        self.root_flow = Some(new_flow);
        self.construct_recursively(layout_ctx, root, new_generator, None);
        return Ok(new_flow)
    }

    /// Creates a flow of the given type for the supplied node.
    fn make_flow(&mut self, ty: FlowContextType, node: AbstractNode<LayoutView>) -> FlowContext {
        let info = FlowData::new(self.next_flow_id(), node);
        let result = match ty {
            Flow_Absolute    => AbsoluteFlow(@mut info),
            Flow_Block       => BlockFlow(@mut BlockFlowData::new(info)),
            Flow_Float       => FloatFlow(@mut info),
            Flow_InlineBlock => InlineBlockFlow(@mut info),
            Flow_Inline      => InlineFlow(@mut InlineFlowData::new(info)),
            Flow_Root        => BlockFlow(@mut BlockFlowData::new_root(info)),
            Flow_Table       => TableFlow(@mut info),
        };
        debug!("LayoutTreeBuilder: created flow: %s", result.debug_str());
        result
    }
}
