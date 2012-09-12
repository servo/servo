/** Creates CSS boxes from a DOM. */
use au = gfx::geometry;
use core::dvec::DVec;
use css::values::{CSSDisplay, DisplayBlock, DisplayInline, DisplayInlineBlock, DisplayNone};
use css::values::{Inherit, Initial, Specified};
use dom::base::{ElementData, HTMLDivElement, HTMLImageElement};
use dom::base::{Element, Text, Node, Doctype, Comment, NodeTree};
use layout::base::{Box, BoxData, GenericBox, ImageBox, TextBox, BoxTree};
use layout::base::{FlowContext, FlowContextData, BlockFlow, InlineFlow, InlineBlockFlow, RootFlow, FlowTree};
use layout::block::BlockFlowData;
use layout::inline::InlineFlowData;
use layout::root::RootFlowData;
use layout::text::TextBoxData;
use option::is_none;
use util::tree;
use servo_text::text_run::TextRun;
use servo_text::font_library::FontLibrary;

export LayoutTreeBuilder;

struct LayoutTreeBuilder {
    mut root_box: Option<@Box>,
    mut root_ctx: Option<@FlowContext>,
    mut next_bid: int,
    mut next_cid: int
}

fn LayoutTreeBuilder() -> LayoutTreeBuilder {
    LayoutTreeBuilder {
        root_box: None,
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
    fn construct_recursively(cur_node: Node, parent_ctx: @FlowContext, parent_box: @Box) {
        let style = cur_node.style();
        
        // DEBUG
        let n_str = fmt!("%?", cur_node.read(|n| copy n.kind ));
        debug!("Considering node: %?", n_str);

        // TODO: handle interactions with 'float', 'position' (CSS 2.1, Section 9.7)
        let display = match style.display_type {
            Specified(v) => match v {
                // tree ends here if 'display: none'
                DisplayNone => return, 
                _ => v
            },
            Inherit | Initial => DisplayInline // TODO: fail instead once resolve works
            //fail ~"Node should have resolved value for 'display', but was initial or inherit"
        };

        // first, create the proper box kind, based on node characteristics
        let box_data = match cur_node.create_box_data(display) {
            None => return,
            Some(data) => data
        };

        // then, figure out its proper context, possibly reorganizing.
        let next_ctx: @FlowContext = match box_data {
            /* Text box is always an inline flow. create implicit inline
            flow ctx if we aren't inside one already. */
            TextBox(*) => {
                if (parent_ctx.starts_inline_flow()) {
                    parent_ctx
                } else {
                    self.make_ctx(InlineFlow(InlineFlowData()), tree::empty())
                }
            },
            ImageBox(*) | GenericBox => {
                match display {
                    DisplayInline | DisplayInlineBlock => {
                        /* if inline, try to put into inline context,
                        making a new one if necessary */
                        if (parent_ctx.starts_inline_flow()) {
                            parent_ctx
                        } else {
                            self.make_ctx(InlineFlow(InlineFlowData()), tree::empty())
                        }
                    },
                    /* block boxes always create a new context */
                    DisplayBlock => {
                        self.make_ctx(BlockFlow(BlockFlowData()), tree::empty())
                    },
                    _ => fail fmt!("unsupported display type in box generation: %?", display)
                }
            }
        };

        // make box, add box to any context-specific list.
        let mut new_box = self.make_box(cur_node, parent_ctx, box_data);
        debug!("Assign ^box to flow: %?", next_ctx.debug_str());

        match next_ctx.kind {
            InlineFlow(d) => { d.boxes.push(new_box) }
            BlockFlow(d) => { d.box = Some(new_box) }
            _ => {} // TODO: float lists, etc.
        };

        // connect the box to its parent box
        debug!("Adding child box b%? of b%?", parent_box.id, new_box.id);
        BoxTree.add_child(parent_box, new_box);
    
        if (!next_ctx.eq(parent_ctx)) {
            debug!("Adding child flow f%? of f%?", parent_ctx.id, next_ctx.id);
            FlowTree.add_child(parent_ctx, next_ctx);
        }
        // recurse
        do NodeTree.each_child(cur_node) |child_node| {
            self.construct_recursively(child_node, next_ctx, new_box); true
        }

        // Fixup any irregularities, such as split inlines (CSS 2.1 Section 9.2.1.1)
        if (next_ctx.starts_inline_flow()) {
            let mut found_child_inline = false;
            let mut found_child_block = false;

            do FlowTree.each_child(next_ctx) |child_ctx| {
                match child_ctx.kind {
                    InlineFlow(*) | InlineBlockFlow => found_child_inline = true,
                    BlockFlow(*) => found_child_block = true,
                    _ => {}
                }; true
            }

            if found_child_block && found_child_inline {
                self.fixup_split_inline(next_ctx)
            }
        }
    }

    fn fixup_split_inline(foo: @FlowContext) {
        // TODO: finish me. 
        fail ~"TODO: handle case where an inline is split by a block"
    }

    /** entry point for box creation. Should only be 
    called on root DOM element. */
    fn construct_trees(root: Node) -> Result<@Box, ()> {
        self.root_ctx = Some(self.make_ctx(RootFlow(RootFlowData()), tree::empty()));
        self.root_box = Some(self.make_box(root, self.root_ctx.get(), GenericBox));

        self.construct_recursively(root, self.root_ctx.get(), self.root_box.get());
        return Ok(self.root_box.get())
    }

    fn make_ctx(kind : FlowContextData, tree: tree::Tree<@FlowContext>) -> @FlowContext {
        let ret = @FlowContext(self.next_ctx_id(), kind, tree);
        debug!("Created context: %s", ret.debug_str());
        ret
    }

    fn make_box(node : Node, ctx: @FlowContext, data: BoxData) -> @Box {
        let ret = @Box(self.next_box_id(), node, ctx, data);
        debug!("Created box: %s", ret.debug_str());
        ret
    }
}

trait PrivateBuilderMethods {
    fn create_box_data(display: CSSDisplay) -> Option<BoxData>;
}

impl Node : PrivateBuilderMethods {
    fn create_box_data(display: CSSDisplay) -> Option<BoxData> {
        do self.read |node| {
            match node.kind {
                ~Doctype(*) | ~Comment(*) => None,
                ~Text(string) => {
                    // TODO: clean this up. Fonts should not be created here.
                    let flib = FontLibrary();
                    let font = flib.get_test_font();
                    let run = TextRun(font, string);
                    Some(TextBox(TextBoxData(copy string, ~[move run])))
                }
                ~Element(element) => {
                    match (element.kind, display) {
                        (~HTMLImageElement({size}), _) => Some(ImageBox(size)),
//                      (_, Specified(_)) => Some(GenericBox),
                        (_, _) => Some(GenericBox) // TODO: replace this with the commented lines
//                      (_, _) => fail ~"Can't create box for Node with non-specified 'display' type"
                    }
                }
            }
        }
    }
}

