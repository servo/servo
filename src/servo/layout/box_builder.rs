/** Creates CSS boxes from a DOM. */
use au = gfx::geometry;
use core::dvec::DVec;
use css::styles::{SpecifiedStyle, empty_style_for_node_kind};
use css::values::{CSSDisplay, DisplayBlock, DisplayInline, DisplayInlineBlock, DisplayNone};
use css::values::{Inherit, Initial, Specified};
use dom::element::*;
use dom::node::{Comment, Doctype, Element, Text, Node, LayoutData};
use image::holder::ImageHolder;
use layout::box::*;
use layout::block::BlockFlowData;
use layout::context::LayoutContext;
use layout::flow::*;
use layout::inline::InlineFlowData;
use layout::root::RootFlowData;
use option::is_none;
use util::tree;

pub struct LayoutTreeBuilder {
    mut root_flow: Option<@FlowContext>,
    mut next_bid: int,
    mut next_cid: int
}

impl LayoutTreeBuilder {
    static pure fn new() -> LayoutTreeBuilder {
        LayoutTreeBuilder {
            root_flow: None,
            next_bid: -1,
            next_cid: -1
        }
    }
}

struct BoxCollector {
    flow: @FlowContext,
    consumer: @BoxConsumer
}

impl BoxCollector {
    static pure fn new(flow: @FlowContext, consumer: @BoxConsumer) -> BoxCollector {
        BoxCollector {
            flow: flow,
            consumer: consumer
        }
    }
}

struct BuilderContext {
    default_collector: BoxCollector,
    priv mut inline_collector: Option<BoxCollector>
}

impl BuilderContext {
    static pure fn new(collector: &BoxCollector) -> BuilderContext {
        unsafe { debug!("Creating new BuilderContext for flow: %s", collector.flow.debug_str()); }
        BuilderContext {
            default_collector: copy *collector,
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

        BuilderContext::new(&BoxCollector::new(new_flow, @BoxConsumer::new(new_flow)))
    }
        
    priv fn make_inline_collector(builder: &LayoutTreeBuilder) -> BuilderContext {
        debug!("BuilderContext: making new inline collector flow");
        let new_flow = builder.make_flow(Flow_Inline);
        let new_consumer = @BoxConsumer::new(new_flow);
        let inline_collector = BoxCollector::new(new_flow, new_consumer);

        self.inline_collector = Some(inline_collector);
        self.attach_child_flow(new_flow);

        BuilderContext::new(&inline_collector)
    }

    priv fn get_inline_collector(builder: &LayoutTreeBuilder) -> BuilderContext {
        match copy self.inline_collector {
            Some(ref collector) => BuilderContext::new(collector),
            None => self.make_inline_collector(builder)
        }
    }

    priv fn clear_inline_collector() {
        self.inline_collector = None;
    }

    fn containing_context_for_display(display: CSSDisplay,
                                      builder: &LayoutTreeBuilder) -> BuilderContext {
        match (display, self.default_collector.flow) { 
            (DisplayBlock, @RootFlow(*)) => self.create_child_flow_of_type(Flow_Block, builder),
            (DisplayBlock, @BlockFlow(*)) => {
                self.clear_inline_collector();
                self.create_child_flow_of_type(Flow_Block, builder)
            },
            (DisplayInline, @InlineFlow(*)) => self.clone(),
            (DisplayInlineBlock, @InlineFlow(*)) => self.clone(),
            (DisplayInline, @BlockFlow(*)) => self.get_inline_collector(builder),
            (DisplayInlineBlock, @BlockFlow(*)) => self.get_inline_collector(builder),
            _ => self.clone()
        }
    }
}

impl LayoutTreeBuilder {
    /* Debug-only ids */
    fn next_box_id() -> int { self.next_bid += 1; self.next_bid }
    fn next_flow_id() -> int { self.next_cid += 1; self.next_cid }

    /** Creates necessary box(es) and flow context(s) for the current DOM node,
    and recurses on its children. */
    fn construct_recursively(layout_ctx: &LayoutContext, cur_node: Node, parent_ctx: &BuilderContext) {
        let style = cur_node.style();
        // DEBUG
        debug!("Considering node: %?", fmt!("%?", cur_node.read(|n| copy n.kind )));

        // TODO: remove this once UA styles work
        // TODO: handle interactions with 'float', 'position' (CSS 2.1, Section 9.7)
        let simulated_display = match self.simulate_UA_display_rules(cur_node, &style) {
            DisplayNone => return, // tree ends here if 'display: none'
            v => v
        };

        // first, determine the box type, based on node characteristics
        let box_type = self.decide_box_type(cur_node, simulated_display);
        let this_ctx = parent_ctx.containing_context_for_display(simulated_display, &self);
        let new_box = self.make_box(layout_ctx, box_type, cur_node, this_ctx.default_collector.flow);
        this_ctx.default_collector.consumer.push_box(layout_ctx, new_box);

        // recurse on child nodes.
        for tree::each_child(&NodeTree, &cur_node) |child_node| {
            self.construct_recursively(layout_ctx, *child_node, &this_ctx);
        }

        this_ctx.default_collector.consumer.pop_box(layout_ctx, new_box);
        self.simplify_children_of_flow(layout_ctx, &this_ctx);

        // store reference to the flow context which contains any
        // boxes that correspond to child_flow.node. These boxes may
        // eventually be elided or split, but the mapping between
        // nodes and FlowContexts should not change during layout.
        for tree::each_child(&FlowTree, &this_ctx.default_collector.flow) |child_flow: &@FlowContext| {
            do (copy child_flow.d().node).iter |node| {
                assert node.has_aux();
                do node.aux |data| { data.flow = Some(*child_flow) }
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
    fn simplify_children_of_flow(_layout_ctx: &LayoutContext, parent_ctx: &BuilderContext) {
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

    fn fixup_split_inline(_foo: @FlowContext) {
        // TODO: finish me. 
        fail ~"TODO: handle case where an inline is split by a block"
    }

    priv fn simulate_UA_display_rules(node: Node, style: &SpecifiedStyle) -> CSSDisplay {
        let resolved = match style.display_type {
            Inherit | Initial => DisplayInline, // TODO: remove once resolve works
            Specified(v) => v
        };

        if (resolved == DisplayNone) { return resolved; }

        do node.read |n| {
            match n.kind {
                ~Doctype(*) | ~Comment(*) => DisplayNone,
                ~Text(*) => DisplayInline,
                ~Element(e) => match e.kind {
                    ~HTMLHeadElement(*) => DisplayNone,
                    ~HTMLScriptElement(*) => DisplayNone,
                    ~HTMLParagraphElement(*) => DisplayBlock,
                    ~HTMLDivElement(*) => DisplayBlock,
                    ~HTMLBodyElement(*) => DisplayBlock,
                    ~HTMLHeadingElement(*) => DisplayBlock,
                    ~HTMLHtmlElement(*) => DisplayBlock,
                    ~HTMLUListElement(*) => DisplayBlock,
                    ~HTMLOListElement(*) => DisplayBlock,
                    _ => resolved
                }
            }
        }
    }

    /** entry point for box creation. Should only be 
    called on root DOM element. */
    fn construct_trees(layout_ctx: &LayoutContext, root: Node) -> Result<@FlowContext, ()> {
        let new_flow = self.make_flow(Flow_Root);
        let new_consumer = @BoxConsumer::new(new_flow);
        let root_ctx = BuilderContext::new(&BoxCollector::new(new_flow, new_consumer));

        self.root_flow = Some(new_flow);
        self.construct_recursively(layout_ctx, root, &root_ctx);
        return Ok(new_flow)
    }

    fn make_flow(ty : FlowContextType) -> @FlowContext {
        let data = FlowData(self.next_flow_id());
        let ret = match ty {
            Flow_Absolute    => @AbsoluteFlow(move data),
            Flow_Block       => @BlockFlow(move data, BlockFlowData()),
            Flow_Float       => @FloatFlow(move data),
            Flow_InlineBlock => @InlineBlockFlow(move data),
            Flow_Inline      => @InlineFlow(move data, InlineFlowData()),
            Flow_Root        => @RootFlow(move data, RootFlowData()),
            Flow_Table       => @TableFlow(move data)
        };
        debug!("LayoutTreeBuilder: created flow: %s", ret.debug_str());
        ret
    }

    /**
       disambiguate between different methods here instead of inlining, since each
       case has very different complexity 
    */
    fn make_box(layout_ctx: &LayoutContext, ty: RenderBoxType, node: Node, ctx: @FlowContext) -> @RenderBox {
        let ret = match ty {
            RenderBox_Generic => self.make_generic_box(layout_ctx, node, ctx),
            RenderBox_Text    => self.make_text_box(layout_ctx, node, ctx),
            RenderBox_Image   => self.make_image_box(layout_ctx, node, ctx),
        };
        debug!("LayoutTreeBuilder: created box: %s", ret.debug_str());
        ret
    }

    fn make_generic_box(_layout_ctx: &LayoutContext, node: Node, ctx: @FlowContext) -> @RenderBox {
        @GenericBox(RenderBoxData(node, ctx, self.next_box_id()))
    }

    fn make_image_box(layout_ctx: &LayoutContext, node: Node, ctx: @FlowContext) -> @RenderBox {
        do node.read |n| {
            match n.kind {
                ~Element(ed) => match ed.kind {
                    ~HTMLImageElement(d) => {
                        // TODO: this could be written as a pattern guard, but it triggers
                        // an ICE (mozilla/rust issue #3601)
                        if d.image.is_some() {
                            let holder = ImageHolder({copy *d.image.get_ref()},
                                                     layout_ctx.image_cache);

                            @ImageBox(RenderBoxData(node, ctx, self.next_box_id()), move holder)
                        } else {
                            info!("Tried to make image box, but couldn't find image. Made generic box instead.");
                            self.make_generic_box(layout_ctx, node, ctx)
                        }
                    },
                    _ => fail ~"WAT error: why couldn't we make an image box?"
                },
                _ => fail ~"WAT error: why couldn't we make an image box?"
            }
        }

    }

    fn make_text_box(_layout_ctx: &LayoutContext, node: Node, ctx: @FlowContext) -> @RenderBox {
        do node.read |n| {
            match n.kind {
                ~Text(string) => @UnscannedTextBox(RenderBoxData(node, ctx, self.next_box_id()), copy string),
                _ => fail ~"WAT error: why couldn't we make a text box?"
            }
        }
    }

    fn decide_box_type(node: Node, display: CSSDisplay) -> RenderBoxType {
        do node.read |n| {
            match n.kind {
                ~Doctype(*) | ~Comment(*) => fail ~"Hey, doctypes and comments shouldn't get here! They are display:none!",
                ~Text(*) => RenderBox_Text,
                ~Element(element) => {
                    // FIXME: Bad copy
                    match (copy element.kind, display) {
                        (~HTMLImageElement(d), _) if d.image.is_some() => RenderBox_Image,
//                      (_, Specified(_)) => GenericBox,
                        (_, _) => RenderBox_Generic // TODO: replace this with the commented lines
//                      (_, _) => fail ~"Can't create box for Node with non-specified 'display' type"
                    }
                }
            }
        }
    }
}
