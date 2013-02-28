/** Creates CSS boxes from a DOM. */

use dom::element::*;
use dom::node::{AbstractNode, Comment, CommentNodeTypeId, Doctype, DoctypeNodeTypeId, Element};
use dom::node::{ElementNodeTypeId, Node, Text, TextNodeTypeId};
use dom;
use layout::block::BlockFlowData;
use layout::box::*;
use layout::context::LayoutContext;
use layout::debug::{BoxedDebugMethods, DebugMethods};
use layout::flow::*;
use layout::inline::InlineFlowData;
use layout::root::RootFlowData;
use util::tree;

use core::dvec::DVec;
use gfx::image::holder::ImageHolder;
use gfx::util::range::Range;
use newcss::values::{CSSDisplay, CSSDisplayBlock, CSSDisplayInline, CSSDisplayInlineBlock};
use newcss::values::{CSSDisplayNone, Inherit, Specified};

pub struct LayoutTreeBuilder {
    mut root_flow: Option<@FlowContext>,
    mut next_bid: int,
    mut next_cid: int
}

pub impl LayoutTreeBuilder {
    static pure fn new() -> LayoutTreeBuilder {
        LayoutTreeBuilder {
            root_flow: None,
            next_bid: -1,
            next_cid: -1
        }
    }
}

// helper object for building the initial box list and making the
// mapping between DOM nodes and boxes.
struct BoxGenerator {
    flow: @FlowContext,
    range_stack: DVec<uint>,
}

enum InlineSpacerSide {
    LogicalBefore,
    LogicalAfter,
}

priv fn simulate_UA_display_rules(node: AbstractNode) -> CSSDisplay {
    // FIXME
    /*let resolved = do node.aux |nd| {
        match nd.style.display_type {
            Inherit | Initial => DisplayInline, // TODO: remove once resolve works
            Specified(v) => v
        }
    };*/

    let resolved = CSSDisplayInline;
    if (resolved == CSSDisplayNone) { return resolved; }

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
    static pure fn new(flow: @FlowContext) -> BoxGenerator {
        unsafe { debug!("Creating box generator for flow: %s", flow.debug_str()); }
        BoxGenerator {
            flow: flow,
            range_stack: DVec()
        }
    }

    /* Whether "spacer" boxes are needed to stand in for this DOM node */
    pure fn inline_spacers_needed_for_node(_: AbstractNode) -> bool {
        return false;
    }

    // TODO: implement this, generating spacer 
    fn make_inline_spacer_for_node_side(_: &LayoutContext,
                                        _: AbstractNode,
                                        _: InlineSpacerSide)
                                     -> Option<@RenderBox> {
        None
    }

    pub fn push_node(ctx: &LayoutContext, builder: &LayoutTreeBuilder, node: AbstractNode) {
        debug!("BoxGenerator[f%d]: pushing node: %s", self.flow.d().id, node.debug_str());

        // first, determine the box type, based on node characteristics
        let simulated_display = simulate_UA_display_rules(node);
        // TODO: remove this once UA styles work
        let box_type = builder.decide_box_type(node, simulated_display);

        debug!("BoxGenerator[f%d]: point a", self.flow.d().id);

        // depending on flow, make a box for this node.
        match self.flow {
            @InlineFlow(*) => {
                let node_range_start = match self.flow {
                    @InlineFlow(*) => self.flow.inline().boxes.len(),
                    _ => 0
                };
                self.range_stack.push(node_range_start);

                // if a leaf, make a box.
                if node.is_leaf() {
                    let new_box = builder.make_box(ctx, box_type, node, self.flow);
                    self.flow.inline().boxes.push(new_box);
                } else if self.inline_spacers_needed_for_node(node) {
                    // else, maybe make a spacer for "left" margin, border, padding
                    do self.make_inline_spacer_for_node_side(ctx, node, LogicalBefore).iter
                            |spacer: &@RenderBox| {
                        self.flow.inline().boxes.push(*spacer);
                    }
                }
                // TODO: cases for inline-block, etc.
            },
            @BlockFlow(*) => {
                debug!("BoxGenerator[f%d]: point b", self.flow.d().id);
                let new_box = builder.make_box(ctx, box_type, node, self.flow);
                debug!("BoxGenerator[f%d]: attaching box[b%d] to block flow (node: %s)",
                       self.flow.d().id, new_box.d().id, node.debug_str());

                assert self.flow.block().box.is_none();
                self.flow.block().box = Some(new_box);
            },
            @RootFlow(*) => {
                debug!("BoxGenerator[f%d]: point c", self.flow.d().id);
                let new_box = builder.make_box(ctx, box_type, node, self.flow);
                debug!("BoxGenerator[f%d]: (node is: %s)", self.flow.d().id, node.debug_str());
                debug!("BoxGenerator[f%d]: attaching box[b%d] to root flow (node: %s)",
                       self.flow.d().id, new_box.d().id, node.debug_str());

                assert self.flow.root().box.is_none();
                self.flow.root().box = Some(new_box);
            },
            _ => { warn!("push_node() not implemented for flow f%d", self.flow.d().id) }
        }
    }

    pub fn pop_node(ctx: &LayoutContext, _builder: &LayoutTreeBuilder, node: AbstractNode) {
        debug!("BoxGenerator[f%d]: popping node: %s", self.flow.d().id, node.debug_str());

        match self.flow {
            @InlineFlow(*) => {
                if self.inline_spacers_needed_for_node(node) {
                    // if this non-leaf box generates extra horizontal
                    // spacing, add a SpacerBox for it.
                    do self.make_inline_spacer_for_node_side(ctx, node, LogicalAfter).iter |spacer: &@RenderBox| {
                        self.flow.inline().boxes.push(*spacer);
                    }
                }
                let mut node_range: Range = Range::new(self.range_stack.pop(), 0);
                node_range.extend_to(self.flow.inline().boxes.len());
                assert node_range.length() > 0;

                debug!("BoxGenerator: adding element range=%?", node_range);
                self.flow.inline().elems.add_mapping(node, &const node_range);
            },
            @BlockFlow(*) | @RootFlow(*) => {
                assert self.range_stack.len() == 0;
            },
            _ => { warn!("pop_node() not implemented for flow %?", self.flow.d().id) }
        }
    }
}

struct BuilderContext {
    default_collector: @BoxGenerator,
    priv mut inline_collector: Option<@BoxGenerator>
}

impl BuilderContext {
    static pure fn new(collector: @BoxGenerator) -> BuilderContext {
        unsafe { debug!("Creating new BuilderContext for flow: %s", collector.flow.debug_str()); }
        BuilderContext {
            default_collector: collector,
            inline_collector: None,
        }
    }

    fn clone() -> BuilderContext {
        debug!("BuilderContext: cloning context");
        copy self
    }
    
    priv fn attach_child_flow(child: @FlowContext) {
        debug!("BuilderContext: Adding child flow f%? of f%?",
               self.default_collector.flow.d().id, child.d().id);
        tree::add_child(&FlowTree, self.default_collector.flow, child);
    }
    
    priv fn create_child_flow_of_type(flow_type: FlowContextType,
                                      builder: &LayoutTreeBuilder) -> BuilderContext {
        let new_flow = builder.make_flow(flow_type);
        self.attach_child_flow(new_flow);

        BuilderContext::new(@BoxGenerator::new(new_flow))
    }
        
    priv fn make_inline_collector(builder: &LayoutTreeBuilder) -> BuilderContext {
        debug!("BuilderContext: making new inline collector flow");
        let new_flow = builder.make_flow(Flow_Inline);
        let new_generator = @BoxGenerator::new(new_flow);

        self.inline_collector = Some(new_generator);
        self.attach_child_flow(new_flow);

        BuilderContext::new(new_generator)
    }

    priv fn get_inline_collector(builder: &LayoutTreeBuilder) -> BuilderContext {
        match copy self.inline_collector {
            Some(collector) => BuilderContext::new(collector),
            None => self.make_inline_collector(builder)
        }
    }

    priv fn clear_inline_collector() {
        self.inline_collector = None;
    }

    // returns a context for the current node, or None if the document subtree rooted
    // by the node should not generate a layout tree. For example, nodes with style 'display:none'
    // should just not generate any flows or boxes.
    fn containing_context_for_node(node: AbstractNode,
                                   builder: &LayoutTreeBuilder)
                                -> Option<BuilderContext> {
        // TODO: remove this once UA styles work
        // TODO: handle interactions with 'float', 'position' (CSS 2.1, Section 9.7)
        let simulated_display = match simulate_UA_display_rules(node) {
            CSSDisplayNone => return None, // tree ends here if 'display: none'
            v => v
        };

        let containing_context = match (simulated_display, self.default_collector.flow) { 
            (CSSDisplayBlock, @RootFlow(*)) => {
                // If this is the root node, then use the root flow's
                // context. Otherwise, make a child block context.
                match node.parent_node() {
                    Some(_) => { self.create_child_flow_of_type(Flow_Block, builder) }
                    None => { self.clone() },
                }
            },
            (CSSDisplayBlock, @BlockFlow(*)) => {
                self.clear_inline_collector();
                self.create_child_flow_of_type(Flow_Block, builder)
            },
            (CSSDisplayInline, @InlineFlow(*)) => self.clone(),
            (CSSDisplayInlineBlock, @InlineFlow(*)) => self.clone(),
            (CSSDisplayInline, @BlockFlow(*)) => self.get_inline_collector(builder),
            (CSSDisplayInlineBlock, @BlockFlow(*)) => self.get_inline_collector(builder),
            _ => self.clone()
        };

        Some(containing_context)
    }
}

impl LayoutTreeBuilder {
    /* Debug-only ids */
    fn next_box_id(&self) -> int { self.next_bid += 1; self.next_bid }
    fn next_flow_id(&self) -> int { self.next_cid += 1; self.next_cid }

    /// Creates necessary box(es) and flow context(s) for the current DOM node,
    /// and recurses on its children.
    fn construct_recursively(&self,
                             layout_ctx: &LayoutContext,
                             cur_node: AbstractNode,
                             parent_ctx: &BuilderContext) {
        debug!("Considering node: %s", cur_node.debug_str());

        let this_ctx = match parent_ctx.containing_context_for_node(cur_node, self) {
            Some(ctx) => ctx,
            None => { return; } // no context because of display: none. Stop building subtree. 
        };
        debug!("point a: %s", cur_node.debug_str());
        this_ctx.default_collector.push_node(layout_ctx, self, cur_node);
        debug!("point b: %s", cur_node.debug_str());

        // recurse on child nodes.
        for cur_node.each_child |child_node| {
            self.construct_recursively(layout_ctx, child_node, &this_ctx);
        }

        this_ctx.default_collector.pop_node(layout_ctx, self, cur_node);
        self.simplify_children_of_flow(layout_ctx, &this_ctx);

        // store reference to the flow context which contains any
        // boxes that correspond to child_flow.node. These boxes may
        // eventually be elided or split, but the mapping between
        // nodes and FlowContexts should not change during layout.
        for tree::each_child(&FlowTree, &this_ctx.default_collector.flow) |child_flow: &@FlowContext| {
            do (copy child_flow.d().node).iter |node| {
                assert node.has_layout_data();
                node.layout_data().flow = Some(*child_flow);
            }
        }
    }

    // Fixup any irregularities such as:
    //
    // * split inlines (CSS 2.1 Section 9.2.1.1)
    // * elide non-preformatted whitespace-only text boxes and their
    //   flows (CSS 2.1 Section 9.2.2.1).
    //
    // The latter can only be done immediately adjacent to, or at the
    // beginning or end of a block flow. Otherwise, the whitespace
    // might affect whitespace collapsing with adjacent text.
    fn simplify_children_of_flow(&self, _: &LayoutContext, parent_ctx: &BuilderContext) {
        match *parent_ctx.default_collector.flow {
            InlineFlow(*) => {
                let mut found_child_inline = false;
                let mut found_child_block = false;

                for tree::each_child(&FlowTree, &parent_ctx.default_collector.flow) |child_ctx: &@FlowContext| {
                    match **child_ctx {
                        InlineFlow(*) | InlineBlockFlow(*) => found_child_inline = true,
                        BlockFlow(*) => found_child_block = true,
                        _ => {}
                    }
                }

                if found_child_block && found_child_inline {
                    self.fixup_split_inline(parent_ctx.default_collector.flow)
                }
            },
            BlockFlow(*) => {
                // FIXME: this will create refcounted cycles between the removed flow and any
                // of its RenderBox or FlowContext children, and possibly keep alive other junk
                let parent_flow = parent_ctx.default_collector.flow;
                // check first/last child for whitespace-ness
                do tree::first_child(&FlowTree, &parent_flow).iter |first_flow: &@FlowContext| {
                    if first_flow.starts_inline_flow() {
                        let boxes = &first_flow.inline().boxes;
                        if boxes.len() == 1 && boxes[0].is_whitespace_only() {
                            debug!("LayoutTreeBuilder: pruning whitespace-only first child flow f%d from parent f%d", 
                                   first_flow.d().id, parent_flow.d().id);
                            tree::remove_child(&FlowTree, parent_flow, *first_flow);
                        }
                    }
                }
                do tree::last_child(&FlowTree, &parent_flow).iter |last_flow: &@FlowContext| {
                    if last_flow.starts_inline_flow() {
                        let boxes = &last_flow.inline().boxes;
                        if boxes.len() == 1 && boxes.last().is_whitespace_only() {
                            debug!("LayoutTreeBuilder: pruning whitespace-only last child flow f%d from parent f%d", 
                                   last_flow.d().id, parent_flow.d().id);
                            tree::remove_child(&FlowTree, parent_flow, *last_flow);
                        }
                    }
                }
            },
            _ => {}
        }
    }

    fn fixup_split_inline(&self, _: @FlowContext) {
        // TODO: finish me. 
        fail!(~"TODO: handle case where an inline is split by a block")
    }

    /** entry point for box creation. Should only be 
    called on root DOM element. */
    fn construct_trees(&self, layout_ctx: &LayoutContext, root: AbstractNode)
                    -> Result<@FlowContext, ()> {
        let new_flow = self.make_flow(Flow_Root);
        let new_generator = @BoxGenerator::new(new_flow);
        let root_ctx = BuilderContext::new(new_generator);

        self.root_flow = Some(new_flow);
        self.construct_recursively(layout_ctx, root, &root_ctx);
        return Ok(new_flow)
    }

    fn make_flow(&self, ty: FlowContextType) -> @FlowContext {
        let data = FlowData(self.next_flow_id());
        let ret = match ty {
            Flow_Absolute    => @AbsoluteFlow(data),
            Flow_Block       => @BlockFlow(data, BlockFlowData()),
            Flow_Float       => @FloatFlow(data),
            Flow_InlineBlock => @InlineBlockFlow(data),
            Flow_Inline      => @InlineFlow(data, InlineFlowData()),
            Flow_Root        => @RootFlow(data, RootFlowData()),
            Flow_Table       => @TableFlow(data)
        };
        debug!("LayoutTreeBuilder: created flow: %s", ret.debug_str());
        ret
    }

    /**
       disambiguate between different methods here instead of inlining, since each
       case has very different complexity 
    */
    fn make_box(&self,
                layout_ctx: &LayoutContext,
                ty: RenderBoxType,
                node: AbstractNode,
                ctx: @FlowContext)
             -> @RenderBox {
        let ret = match ty {
            RenderBox_Generic => self.make_generic_box(layout_ctx, node, ctx),
            RenderBox_Text    => self.make_text_box(layout_ctx, node, ctx),
            RenderBox_Image   => self.make_image_box(layout_ctx, node, ctx),
        };
        debug!("LayoutTreeBuilder: created box: %s", ret.debug_str());
        ret
    }

    fn make_generic_box(&self,
                        _: &LayoutContext,
                        node: AbstractNode,
                        ctx: @FlowContext)
                     -> @RenderBox {
        @GenericBox(RenderBoxData(copy node, ctx, self.next_box_id()))
    }

    fn make_image_box(&self,
                      layout_ctx: &LayoutContext,
                      node: AbstractNode,
                      ctx: @FlowContext)
                   -> @RenderBox {
        if !node.is_image_element() {
            fail!(~"WAT error: why couldn't we make an image box?");
        }

        do node.with_imm_image_element |image_element| {
            if image_element.image.is_some() {
                let holder = ImageHolder::new(copy *image_element.image.get_ref(),
                                              layout_ctx.image_cache);
                @ImageBox(RenderBoxData(node, ctx, self.next_box_id()), holder)
            } else {
                info!("Tried to make image box, but couldn't find image. Made generic box instead.");
                self.make_generic_box(layout_ctx, node, ctx)
            }
        }
    }

    fn make_text_box(&self,
                     _: &LayoutContext,
                     node: AbstractNode,
                     ctx: @FlowContext)
                  -> @RenderBox {
        if !node.is_text() {
            fail!(~"WAT error: why couldn't we make a text box?");
        }

        // FIXME: Don't copy text. I guess it should be atomically reference counted?
        do node.with_imm_text |text_node| {
            let string = text_node.text.to_str();
            @UnscannedTextBox(RenderBoxData(node, ctx, self.next_box_id()), string)
        }
    }

    fn decide_box_type(&self, node: AbstractNode, display: CSSDisplay) -> RenderBoxType {
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
