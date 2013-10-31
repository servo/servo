/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Creates CSS boxes from a DOM tree.

use layout::block::BlockFlow;
use layout::float::FloatFlow;
use layout::box::{GenericRenderBox, GenericRenderBoxClass, ImageRenderBox, ImageRenderBoxClass};
use layout::box::{RenderBox, RenderBoxBase, RenderBoxClass, RenderBoxUtils, TextRenderBoxClass};
use layout::box::{UnscannedTextRenderBox, UnscannedTextRenderBoxClass};
use layout::context::LayoutContext;
use layout::float_context::FloatType;
use layout::flow::{AbsoluteFlow, BlockFlowClass, FloatFlowClass, FlowContext, FlowData};
use layout::flow::{ImmutableFlowUtils, InlineBlockFlow, InlineBlockFlowClass, InlineFlowClass};
use layout::flow::{MutableFlowUtils, TableFlow};
use layout::flow;
use layout::inline::{InlineFlow};
use layout::text::TextRunScanner;
use css::node_style::StyledNode;

use style::computed_values::display;
use style::computed_values::float;
use layout::float_context::{FloatLeft, FloatRight};
use script::dom::node::{AbstractNode, CommentNodeTypeId, DoctypeNodeTypeId};
use script::dom::node::{ElementNodeTypeId, LayoutView, TextNodeTypeId, DocumentNodeTypeId};
use script::dom::node::DocumentFragmentNodeTypeId;
use servo_util::range::Range;
use servo_util::tree::{TreeNodeRef, TreeNode};
use std::cast;
use std::cell::Cell;

enum FlowType {
    AbsoluteFlowType,
    BlockFlowType,
    FloatFlowType(FloatType),
    InlineBlockFlowType,
    InlineFlowType,
    RootFlowType,
    TableFlowType,
}

pub struct LayoutTreeBuilder {
    next_cid: int,
    next_bid: int,
}

impl LayoutTreeBuilder {
    pub fn new() -> LayoutTreeBuilder {
        LayoutTreeBuilder {
            next_cid: -1,
            next_bid: -1,
        }
    }
}

// helper object for building the initial box list and making the
// mapping between DOM nodes and boxes.
struct BoxGenerator<'self> {
    flow: &'self mut FlowContext,
    range_stack: @mut ~[uint],
}

enum InlineSpacerSide {
    LogicalBefore,
    LogicalAfter,
}

impl<'self> BoxGenerator<'self> {
    /* Debug ids only */

    fn new(flow: &'self mut FlowContext) -> BoxGenerator<'self> {
        debug!("Creating box generator for flow: {:s}", flow.debug_str());
        BoxGenerator {
            flow: flow,
            range_stack: @mut ~[]
        }
    }

    fn with_clone<R>(&mut self, cb: &fn(BoxGenerator<'self>) -> R) -> R {
        // FIXME(pcwalton): This is a hack; it can be done safely with linearity.
        unsafe {
            let gen = BoxGenerator {
                flow: cast::transmute_copy(&self.flow),
                range_stack: self.range_stack
            };
            cb(gen)
        }
    }

    /* Whether "spacer" boxes are needed to stand in for this DOM node */
    fn inline_spacers_needed_for_node(_: AbstractNode<LayoutView>) -> bool {
        return false;
    }

    // TODO: implement this, generating spacer 
    fn make_inline_spacer_for_node_side(_: &LayoutContext,
                                        _: AbstractNode<LayoutView>,
                                        _: InlineSpacerSide)
                                        -> Option<@RenderBox> {
        None
    }

    pub fn push_node(&mut self,
                     ctx: &LayoutContext,
                     node: AbstractNode<LayoutView>,
                     builder: &mut LayoutTreeBuilder) {
        debug!("BoxGenerator[f{:d}]: pushing node: {:s}", flow::base(self.flow).id, node.debug_str());

        // TODO: remove this once UA styles work
        let box_type = self.decide_box_type(node);

        debug!("BoxGenerator[f{:d}]: point a", flow::base(self.flow).id);

        let range_stack = &mut self.range_stack;
        // depending on flow, make a box for this node.
        match self.flow.class() {
            InlineFlowClass => {
                let inline = self.flow.as_inline();
                let node_range_start = inline.boxes.len();
                range_stack.push(node_range_start);

                // if a leaf, make a box.
                if node.is_leaf() {
                    let new_box = BoxGenerator::make_box(ctx, box_type, node, builder);
                    inline.boxes.push(new_box);
                } else if BoxGenerator::inline_spacers_needed_for_node(node) {
                    // else, maybe make a spacer for "left" margin, border, padding
                    let inline_spacer = BoxGenerator::make_inline_spacer_for_node_side(ctx, node, LogicalBefore);
                    for spacer in inline_spacer.iter() {
                        inline.boxes.push(*spacer);
                    }
                }
                // TODO: cases for inline-block, etc.
            },
            BlockFlowClass => {
                let block = self.flow.as_block();
                debug!("BoxGenerator[f{:d}]: point b", block.base.id);
                let new_box = BoxGenerator::make_box(ctx, box_type, node, builder);

                debug!("BoxGenerator[f{:d}]: attaching box[b{:d}] to block flow (node: {:s})",
                       block.base.id,
                       new_box.base().id(),
                       node.debug_str());

                assert!(block.box.is_none());
                block.box = Some(new_box);
            }
            FloatFlowClass => {
                let float = self.flow.as_float();
                debug!("BoxGenerator[f{:d}]: point b", float.base.id);

                let new_box = BoxGenerator::make_box(ctx, box_type, node, builder);

                debug!("BoxGenerator[f{:d}]: attaching box[b{:d}] to float flow (node: {:s})",
                        float.base.id,
                        new_box.base().id(),
                        node.debug_str());

                assert!(float.box.is_none() && float.index.is_none());
                float.box = Some(new_box);
            }
            _ => warn!("push_node() not implemented for flow f{:d}", flow::base(self.flow).id),
        }
    }

    pub fn pop_node(&mut self, ctx: &LayoutContext, node: AbstractNode<LayoutView>) {
        debug!("BoxGenerator[f{:d}]: popping node: {:s}", flow::base(self.flow).id, node.debug_str());

        match self.flow.class() {
            InlineFlowClass => {
                let inline = self.flow.as_inline();
                let inline = &mut *inline;

                if BoxGenerator::inline_spacers_needed_for_node(node) {
                    // If this non-leaf box generates extra horizontal spacing, add a SpacerBox for
                    // it.
                    let result = BoxGenerator::make_inline_spacer_for_node_side(ctx, node, LogicalAfter);
                    for spacer in result.iter() {
                        let boxes = &mut inline.boxes;
                        boxes.push(*spacer);
                    }
                }
                let mut node_range: Range = Range::new(self.range_stack.pop(), 0);
                node_range.extend_to(inline.boxes.len());

                if node_range.length() == 0 {
                    warn!("node range length is zero?!")
                }

                debug!("BoxGenerator: adding element range={}", node_range);
                inline.elems.add_mapping(node, &node_range);
            },
            BlockFlowClass => assert!(self.range_stack.len() == 0),
            FloatFlowClass => assert!(self.range_stack.len() == 0),
            _ => warn!("pop_node() not implemented for flow {:?}", flow::base(self.flow).id),
        }
    }

    /// Disambiguate between different methods here instead of inlining, since each case has very
    /// different complexity.
    fn make_box(layout_ctx: &LayoutContext,
                ty: RenderBoxClass,
                node: AbstractNode<LayoutView>,
                builder: &mut LayoutTreeBuilder)
                -> @RenderBox {
        let base = RenderBoxBase::new(node, builder.next_box_id());
        let result = match ty {
            GenericRenderBoxClass => @GenericRenderBox::new(base) as @RenderBox,
            TextRenderBoxClass | UnscannedTextRenderBoxClass => {
                @UnscannedTextRenderBox::new(base) as @RenderBox
            }
            ImageRenderBoxClass => BoxGenerator::make_image_box(layout_ctx, node, base),
        };
        debug!("BoxGenerator: created box: {:s}", result.debug_str());
        result
    }

    fn make_image_box(layout_ctx: &LayoutContext,
                      node: AbstractNode<LayoutView>,
                      base: RenderBoxBase)
                      -> @RenderBox {
        assert!(node.is_image_element());

        do node.with_imm_image_element |image_element| {
            if image_element.image.is_some() {
                // FIXME(pcwalton): Don't copy URLs.
                let url = (*image_element.image.get_ref()).clone();
                @ImageRenderBox::new(base.clone(), url, layout_ctx.image_cache) as @RenderBox
            } else {
                info!("Tried to make image box, but couldn't find image. Made generic box \
                       instead.");
                @GenericRenderBox::new(base.clone()) as @RenderBox
            }
        }
    }

    fn decide_box_type(&self, node: AbstractNode<LayoutView>) -> RenderBoxClass {
        if node.is_text() {
            TextRenderBoxClass
        } else if node.is_image_element() {
            do node.with_imm_image_element |image_element| {
                match image_element.image {
                    Some(_) => ImageRenderBoxClass,
                    None => GenericRenderBoxClass,
                }
            }
        } else if node.is_element() {
            GenericRenderBoxClass
        } else {
            fail!("Hey, doctypes and comments shouldn't get here! They are display:none!")
        }
    }

}

enum BoxGenResult<'self> {
    NoGenerator,
    ParentGenerator,
    SiblingGenerator,
    NewGenerator(BoxGenerator<'self>),
    /// Start a new generator, but also switch the parent out for the
    /// grandparent, ending the parent generator.
    ReparentingGenerator(BoxGenerator<'self>),
    Mixed(BoxGenerator<'self>, ~BoxGenResult<'self>),
}

/// Determines whether the result of child box construction needs to reparent
/// or not. Reparenting is needed when a block flow is a child of an inline;
/// in that case, we need to let the level up the stack no to end the parent
/// genertor and continue with the grandparent.
enum BoxConstructResult<'self> {
    Normal(Option<BoxGenerator<'self>>),
    Reparent(BoxGenerator<'self>),
}

impl LayoutTreeBuilder {
    /* Debug-only ids */
    pub fn next_flow_id(&mut self) -> int { self.next_cid += 1; self.next_cid }
    pub fn next_box_id(&mut self) -> int { self.next_bid += 1; self.next_bid }

    /// Creates necessary box(es) and flow context(s) for the current DOM node,
    /// and recurses on its children.
    pub fn construct_recursively<'a>(
                                 &mut self,
                                 layout_ctx: &LayoutContext,
                                 cur_node: AbstractNode<LayoutView>,
                                 mut grandparent_generator: Option<BoxGenerator<'a>>,
                                 mut parent_generator: BoxGenerator<'a>,
                                 mut prev_sibling_generator: Option<BoxGenerator<'a>>)
                                 -> BoxConstructResult<'a> {
        debug!("Considering node: {:s}", cur_node.debug_str());
        let box_gen_result = {
            let grandparent_gen_ref = match grandparent_generator {
                Some(ref mut generator) => Some(generator),
                None => None,
            };
            let sibling_gen_ref = match prev_sibling_generator {
                Some(ref mut generator) => Some(generator),
                None => None,
            };
            self.box_generator_for_node(cur_node, grandparent_gen_ref, &mut parent_generator, sibling_gen_ref)
        };

        let mut reparent = false;

        debug!("result from generator_for_node: {:?}", &box_gen_result);
        // Skip over nodes that don't belong in the flow tree
        let (this_generator, next_generator) = match box_gen_result {
            NoGenerator => return Normal(prev_sibling_generator),
            ParentGenerator => {
                do parent_generator.with_clone |clone| {
                    (clone, None)
                }
            }
            SiblingGenerator => (prev_sibling_generator.take_unwrap(), None),
            NewGenerator(gen) => (gen, None),
            ReparentingGenerator(gen) => {
                reparent = true;
                (gen, None)
            }
            Mixed(gen, next_gen) => (gen, Some(match *next_gen {
                ParentGenerator => {
                    do parent_generator.with_clone |clone| {
                        clone
                    }
                }
                SiblingGenerator => prev_sibling_generator.take_unwrap(),
                _ => fail!("Unexpect BoxGenResult")
            }))
        };

        let mut this_generator = this_generator;

        debug!("point a: {:s}", cur_node.debug_str());
        this_generator.push_node(layout_ctx, cur_node, self);
        debug!("point b: {:s}", cur_node.debug_str());

        // recurse on child nodes.
        let prev_gen_cell = Cell::new(Normal(None));
        for child_node in cur_node.children() {
            do parent_generator.with_clone |grandparent_clone| {
                let grandparent_clone_cell = Cell::new(Some(grandparent_clone));
                do this_generator.with_clone |parent_clone| {
                    match prev_gen_cell.take() {
                        Normal(prev_gen) => {
                            let prev_generator = self.construct_recursively(layout_ctx,
                                                                            child_node,
                                                                            grandparent_clone_cell.take(),
                                                                            parent_clone,
                                                                            prev_gen);
                            prev_gen_cell.put_back(prev_generator);
                        }
                        Reparent(prev_gen) => {
                            let prev_generator = self.construct_recursively(layout_ctx,
                                                                            child_node,
                                                                            None,
                                                                            grandparent_clone_cell.take().unwrap(),
                                                                            Some(prev_gen));
                            prev_gen_cell.put_back(prev_generator);
                        } 
                    }
                }
            }
        }

        this_generator.pop_node(layout_ctx, cur_node);
        self.simplify_children_of_flow(layout_ctx, this_generator.flow);

        match next_generator {
            Some(n_gen) => Normal(Some(n_gen)),
            None => {
                if reparent {
                    Reparent(this_generator)
                } else {
                    Normal(Some(this_generator))
                }
            }
        }
    }

    pub fn box_generator_for_node<'a>(&mut self,
                                      node: AbstractNode<LayoutView>,
                                      grandparent_generator: Option<&mut BoxGenerator<'a>>,
                                      parent_generator: &mut BoxGenerator<'a>,
                                      mut sibling_generator: Option<&mut BoxGenerator<'a>>)
                                      -> BoxGenResult<'a> {
        let display = match node.type_id() {
            ElementNodeTypeId(_) => match node.style().Box.display {
                display::none => return NoGenerator,
                display => display,
            },
            TextNodeTypeId => display::inline,
            DocumentNodeTypeId(_) |
            DoctypeNodeTypeId |
            DocumentFragmentNodeTypeId |
            CommentNodeTypeId => return NoGenerator,
        };

        // FIXME(pcwalton): Unsafe.
        let sibling_flow: Option<&mut FlowContext> = sibling_generator.as_mut().map(|gen| {
            unsafe {
                cast::transmute_copy(&gen.flow)
            }
        });

        let is_float = if (node.is_element()) {
            match node.style().Box.float {
                float::none => None,
                float::left => Some(FloatLeft),
                float::right => Some(FloatRight)
            }
        } else {
            None
        };

        let sibling_flow_class = match sibling_flow {
            None => None,
            Some(flow) => Some(flow.class()),
        };
        
        let new_generator = match (display, parent_generator.flow.class(), sibling_flow_class) {
            // Floats
            (display::block, BlockFlowClass, _) |
            (display::block, FloatFlowClass, _) if is_float.is_some() => {
                self.create_child_generator(node,
                                            parent_generator,
                                            FloatFlowType(is_float.unwrap()))
            }
            // If we're placing a float after an inline, append the float to the inline flow,
            // then continue building from the inline flow in case there are more inlines
            // afterward.
            (display::block, _, Some(InlineFlowClass)) if is_float.is_some() => {
                let float_type = FloatFlowType(is_float.unwrap());
                let float_generator = self.create_child_generator(node,
                                                                  sibling_generator.unwrap(),
                                                                  float_type);
                return Mixed(float_generator, ~SiblingGenerator);
            }
            // This is a catch-all case for when:
            // a) sibling_flow is None
            // b) sibling_flow is a BlockFlow
            (display::block, InlineFlowClass, _) if is_float.is_some() => {
                self.create_child_generator(node,
                                            parent_generator,
                                            FloatFlowType(is_float.unwrap()))
            }

            (display::block, BlockFlowClass, _) => {
                match (parent_generator.flow.as_block().is_root, node.parent_node()) {
                    // If this is the root node, then use the root flow's
                    // context. Otherwise, make a child block context.
                    (true, Some(parent)) if !parent.is_document() => {
                        self.create_child_generator(node, parent_generator, BlockFlowType)
                    }
                    (true, None) | (true, Some(_)) => return ParentGenerator,
                    (false, _) => {
                        self.create_child_generator(node, parent_generator, BlockFlowType)
                    }
                }
            }

            (display::block, FloatFlowClass, _) => {
                self.create_child_generator(node, parent_generator, BlockFlowType)
            }

            // Inlines that are children of inlines are part of the same flow
            (display::inline, InlineFlowClass, _) => return ParentGenerator,
            (display::inline_block, InlineFlowClass, _) => return ParentGenerator,

            // Inlines that are children of blocks create new flows if their
            // previous sibling was a block.
            (display::inline, BlockFlowClass, Some(BlockFlowClass)) |
            (display::inline_block, BlockFlowClass, Some(BlockFlowClass)) => {
                self.create_child_generator(node, parent_generator, InlineFlowType)
            }

            // The first two cases should only be hit when a FloatFlow
            // is the first child of a BlockFlow. Other times, we will
            (display::inline, _, Some(FloatFlowClass)) |
            (display::inline_block, _, Some(FloatFlowClass)) |
            (display::inline, FloatFlowClass, _) |
            (display::inline_block, FloatFlowClass, _) => {
                self.create_child_generator(node, parent_generator, InlineFlowType)
            }

            // Inlines whose previous sibling was not a block try to use their
            // sibling's flow context.
            (display::inline, BlockFlowClass, _) |
            (display::inline_block, BlockFlowClass, _) => {
                return match sibling_generator {
                    None => NewGenerator(self.create_child_generator(node, 
                                                                     parent_generator, 
                                                                     InlineFlowType)),
                    Some(*) => SiblingGenerator
                }
            }

            // blocks that are children of inlines need to split their parent
            // flows.
            (display::block, InlineFlowClass, _) => {
                match grandparent_generator {
                    None => fail!("expected to have a grandparent block flow"),
                    Some(grandparent_gen) => {
                        assert!(grandparent_gen.flow.is_block_like());

                        let block_gen = self.create_child_generator(node,
                                                                    grandparent_gen,
                                                                    BlockFlowType);
                        return ReparentingGenerator(block_gen);
                    }
                }
            }

            _ => return ParentGenerator
        };

        NewGenerator(new_generator)
    }

    pub fn create_child_generator<'a>(
                                  &mut self,
                                  node: AbstractNode<LayoutView>,
                                  parent_generator: &mut BoxGenerator<'a>,
                                  ty: FlowType)
                                  -> BoxGenerator<'a> {
        let new_flow = self.make_flow(ty, node);
        parent_generator.flow.add_new_child(new_flow);
        let flow_ref = flow::last_child(parent_generator.flow).unwrap();
        BoxGenerator::new(*flow_ref)
    }

    /// Fix up any irregularities such as:
    ///
    /// * split inlines (CSS 2.1 Section 9.2.1.1)
    /// * elide non-preformatted whitespace-only text boxes and their flows (CSS 2.1 Section
    ///   9.2.2.1).
    ///
    /// The latter can only be done immediately adjacent to, or at the beginning or end of a block
    /// flow. Otherwise, the whitespace might affect whitespace collapsing with adjacent text.
    pub fn simplify_children_of_flow(&self, ctx: &LayoutContext, parent_flow: &mut FlowContext) {
        match parent_flow.class() {
            InlineFlowClass => {
                let mut found_child_inline = false;
                let mut found_child_block = false;

                for child_ctx in flow::child_iter(parent_flow) {
                    match child_ctx.class() {
                        InlineFlowClass | InlineBlockFlowClass => found_child_inline = true,
                        BlockFlowClass => found_child_block = true,
                        _ => {}
                    }
                }

                if found_child_block && found_child_inline {
                    self.fixup_split_inline(parent_flow)
                }
            }
            BlockFlowClass | FloatFlowClass => {
                // check first/last child for whitespace-ness
                let mut do_remove = false;
                let p_id = flow::base(parent_flow).id;
                do parent_flow.with_first_child |mut first_child| {
                    for first_flow in first_child.mut_iter() {
                        if first_flow.starts_inline_flow() {
                            // FIXME: workaround for rust#6393
                            {
                                let first_inline_flow = first_flow.as_inline();
                                let boxes = &first_inline_flow.boxes;
                                if boxes.len() == 1 {
                                    let first_box = boxes[0];   // FIXME(pcwalton): Rust bug
                                    if first_box.is_whitespace_only() {
                                        debug!("LayoutTreeBuilder: pruning whitespace-only first \
                                                child flow f{:d} from parent f{:d}",
                                               first_inline_flow.base.id,
                                               p_id);
                                        do_remove = true;
                                    }
                                }
                            }
                        }
                    }
                }
                if (do_remove) { 
                    parent_flow.remove_first();
                }


                do_remove = false;
                let p_id = flow::base(parent_flow).id;
                do parent_flow.with_last_child |mut last_child| {
                    for last_flow in last_child.mut_iter() {
                        if last_flow.starts_inline_flow() {
                            // FIXME: workaround for rust#6393
                            {
                                let last_inline_flow = last_flow.as_inline();
                                let boxes = &last_inline_flow.boxes;
                                if boxes.len() == 1 && boxes.last().is_whitespace_only() {
                                    let last_box = boxes.last();    // FIXME(pcwalton): Rust bug
                                    if last_box.is_whitespace_only() {
                                        debug!("LayoutTreeBuilder: pruning whitespace-only last \
                                                child flow f{:d} from parent f{:d}",
                                               last_inline_flow.base.id,
                                               p_id);
                                        do_remove = true;
                                    }
                                }
                            }
                        }
                    }
                }
                if (do_remove) {
                    parent_flow.remove_last();
                }

                // Issue 543: We only need to do this if there are inline child
                // flows, but there's not a quick way to check at the moment.
                for child_flow in flow::child_iter(parent_flow) {
                    match child_flow.class() {
                        InlineFlowClass | InlineBlockFlowClass => {
                            let mut scanner = TextRunScanner::new();
                            scanner.scan_for_runs(ctx, *child_flow);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    pub fn fixup_split_inline(&self, _: &mut FlowContext) {
        // TODO: finish me. 
        fail!(~"TODO: handle case where an inline is split by a block")
    }

    /// Entry point for box creation. Should only be called on the root DOM element.
    pub fn construct_trees(&mut self, layout_ctx: &LayoutContext, root: AbstractNode<LayoutView>)
                       -> Result<~FlowContext:, ()> {
        debug!("Constructing flow tree for DOM: ");
        debug!("{:?}", root.dump());

        let mut new_flow = self.make_flow(RootFlowType, root);
        {
            let new_generator = BoxGenerator::new(new_flow);
            self.construct_recursively(layout_ctx, root, None, new_generator, None);
        }
        return Ok(new_flow)
    }

    /// Creates a flow of the given type for the supplied node.
    pub fn make_flow(&mut self, flow_type: FlowType, node: AbstractNode<LayoutView>)
                     -> ~FlowContext: {
        let info = FlowData::new(self.next_flow_id(), node);
        let result = match flow_type {
            AbsoluteFlowType        => ~AbsoluteFlow::new(info) as ~FlowContext:,
            BlockFlowType           => ~BlockFlow::new(info) as ~FlowContext:,
            FloatFlowType(f_type)   => ~FloatFlow::new(info, f_type) as ~FlowContext:,
            InlineBlockFlowType     => ~InlineBlockFlow::new(info) as ~FlowContext:,
            InlineFlowType          => ~InlineFlow::new(info) as ~FlowContext:,
            RootFlowType            => ~BlockFlow::new_root(info) as ~FlowContext:,
            TableFlowType           => ~TableFlow::new(info) as ~FlowContext:,
        };
        debug!("LayoutTreeBuilder: created flow: {:s}", result.debug_str());
        result
    }
}
