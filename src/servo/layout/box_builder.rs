/** Creates CSS boxes from a DOM. */
use au = gfx::geometry;
use core::dvec::DVec;
use css::styles::{SpecifiedStyle, empty_style_for_node_kind};
use css::values::{CSSDisplay, DisplayBlock, DisplayInline, DisplayInlineBlock, DisplayNone};
use css::values::{Inherit, Initial, Specified};
use dom::element::*;
use dom::node::{Comment, Doctype, Element, Text, Node, NodeTree, LayoutData};
use image::holder::ImageHolder;
use layout::box::*;
use layout::block::BlockFlowData;
use layout::context::LayoutContext;
use layout::flow::*;
use layout::inline::InlineFlowData;
use layout::root::RootFlowData;
use option::is_none;
use util::tree;

export LayoutTreeBuilder;

struct LayoutTreeBuilder {
    mut root_ctx: Option<@FlowContext>,
    mut next_bid: int,
    mut next_cid: int
}

fn LayoutTreeBuilder() -> LayoutTreeBuilder {
    LayoutTreeBuilder {
        root_ctx: None,
        next_bid: -1,
        next_cid: -1
    }
}

impl LayoutTreeBuilder {
    /* Debug-only ids */
    fn next_box_id() -> int { self.next_bid += 1; self.next_bid }
    fn next_ctx_id() -> int { self.next_cid += 1; self.next_cid }

    /** Creates necessary box(es) and flow context(s) for the current DOM node,
    and recurses on its children. */
    fn construct_recursively(layout_ctx: &LayoutContext, cur_node: Node, 
                             parent_ctx: @FlowContext, parent_box: Option<@RenderBox>) {

        let style = cur_node.style();
        
        // DEBUG
        let n_str = fmt!("%?", cur_node.read(|n| copy n.kind ));
        debug!("Considering node: %?", n_str);

        // TODO: remove this once UA styles work
        // TODO: handle interactions with 'float', 'position' (CSS 2.1, Section 9.7)
        let simulated_display = match self.simulate_UA_display_rules(cur_node, &style) {
            DisplayNone => return, // tree ends here if 'display: none'
            v => v
        };

        // first, determine the box type, based on node characteristics
        let box_type = self.decide_box_type(cur_node, simulated_display);

        // then, figure out its proper context, possibly reorganizing.
        let next_ctx: @FlowContext = match box_type {
            /* Text box is always an inline flow. create implicit inline
            flow ctx if we aren't inside one already. */
            RenderBox_Text => {
                if (parent_ctx.starts_inline_flow()) {
                    parent_ctx
                } else {
                    self.make_ctx(Flow_Inline)
                }
            },
            RenderBox_Image | RenderBox_Generic => {
                match simulated_display {
                    DisplayInline | DisplayInlineBlock => {
                        /* if inline, try to put into inline context,
                        making a new one if necessary */
                        if (parent_ctx.starts_inline_flow()) {
                            parent_ctx
                        } else {
                            self.make_ctx(Flow_Inline)
                        }
                    },
                    /* block boxes always create a new context */
                    DisplayBlock => {
                        self.make_ctx(Flow_Block)
                    },
                    _ => fail fmt!("unsupported display type in box generation: %?", simulated_display)
                }
            }
        };

        // store reference to the flow context which contains any boxes
        // that correspond to cur_node
        assert cur_node.has_aux();
        do cur_node.aux |data| { data.flow = Some(next_ctx) }

        // make box, add box to any context-specific list.
        let mut new_box = self.make_box(layout_ctx, box_type, cur_node, parent_ctx);
        debug!("Assign ^box to flow: %?", next_ctx.debug_str());

        match *next_ctx {
            InlineFlow(*) => {
                next_ctx.inline().boxes.push(new_box);

                if (parent_box.is_some()) {
                    let parent = parent_box.get();

                    // connect the box to its parent box
                    debug!("In inline flow f%?, set child b%? of parent b%?", 
                           next_ctx.d().id, parent.d().id, new_box.d().id);
                    RenderBoxTree.add_child(parent, new_box);
                }
            }
            BlockFlow(*) => next_ctx.block().box = Some(new_box),
            _ => {} // TODO: float lists, etc.
        };

    
        if !core::box::ptr_eq(next_ctx, parent_ctx) {
            debug!("Adding child flow f%? of f%?",
                   parent_ctx.d().id, next_ctx.d().id);
            FlowTree.add_child(parent_ctx, next_ctx);
        }
        // recurse
        // TODO: don't set parent box unless this is an inline flow?
        do NodeTree.each_child(&cur_node) |child_node| {
            self.construct_recursively(layout_ctx, *child_node, next_ctx, Some(new_box)); true
        }

        // Fixup any irregularities, such as split inlines (CSS 2.1 Section 9.2.1.1)
        if (next_ctx.starts_inline_flow()) {
            let mut found_child_inline = false;
            let mut found_child_block = false;

            do FlowTree.each_child(next_ctx) |child_ctx| {
                match *child_ctx {
                    InlineFlow(*) | InlineBlockFlow(*) => found_child_inline = true,
                    BlockFlow(*) => found_child_block = true,
                    _ => {}
                }; true
            }

            if found_child_block && found_child_inline {
                self.fixup_split_inline(next_ctx)
            }
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
                    _ => resolved
                }
            }
        }
    }

    /** entry point for box creation. Should only be 
    called on root DOM element. */
    fn construct_trees(layout_ctx: &LayoutContext, root: Node) -> Result<@FlowContext, ()> {
        self.root_ctx = Some(self.make_ctx(Flow_Root));
        self.construct_recursively(layout_ctx, root, self.root_ctx.get(), None);
        return Ok(self.root_ctx.get())
    }

    fn make_ctx(ty : FlowContextType) -> @FlowContext {
        let data = FlowData(self.next_ctx_id());
        let ret = match ty {
            Flow_Absolute    => @AbsoluteFlow(data),
            Flow_Block       => @BlockFlow(data, BlockFlowData()),
            Flow_Float       => @FloatFlow(data),
            Flow_InlineBlock => @InlineBlockFlow(data),
            Flow_Inline      => @InlineFlow(data, InlineFlowData()),
            Flow_Root        => @RootFlow(data, RootFlowData()),
            Flow_Table       => @TableFlow(data)
        };
        debug!("Created context: %s", ret.debug_str());
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
        debug!("Created box: %s", ret.debug_str());
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
                                                     layout_ctx.image_cache,
                                                     copy layout_ctx.reflow_cb);

                            @ImageBox(RenderBoxData(node, ctx, self.next_box_id()), holder)
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