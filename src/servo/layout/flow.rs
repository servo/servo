use au = gfx::geometry;
use au::au;
use dl = gfx::display_list;
use dom::node::Node;
use geom::rect::Rect;
use geom::point::Point2D;
// TODO: pub-use these
use layout::block::BlockFlowData;
use layout::context::LayoutContext;
use layout::debug::DebugMethods;
use layout::inline::InlineFlowData;
use layout::root::RootFlowData;
use util::tree;

/** Servo's experimental layout system builds a tree of FlowContexts
and RenderBoxes, and figures out positions and display attributes of
tree nodes. Positions are computed in several tree traversals driven
by fundamental data dependencies of inline and block layout.

Flows are interior nodes in the layout tree, and correspond closely to
flow contexts in the CSS specification. Flows are responsible for
positioning their child flow contexts and render boxes. Flows have
purpose-specific fields, such as auxilliary line box structs,
out-of-flow child lists, and so on.

Currently, the important types of flows are:

 * BlockFlow: a flow that establishes a block context. It has several
   child flows, each of which are positioned according to block
   formatting context rules (as if child flows CSS block boxes). Block
   flows also contain a single GenericBox to represent their rendered
   borders, padding, etc. (In the future, this render box may be
   folded into BlockFlow to save space.)

 * InlineFlow: a flow that establishes an inline context. It has a
   flat list of child boxes/flows that are subject to inline layout
   and line breaking, and structs to represent line breaks and mapping
   to CSS boxes, for the purpose of handling `getClientRects()`.

*/


struct FlowLayoutData {
    // TODO: min/pref and position are used during disjoint phases of
    // layout; maybe combine into a single enum to save space.
    mut min_width: au,
    mut pref_width: au,
    mut position: Rect<au>,
}

fn FlowLayoutData() -> FlowLayoutData {
    FlowLayoutData {
        min_width: au(0),
        pref_width: au(0),
        position : au::zero_rect(),
    }
}

/* The type of the formatting context, and data specific to each
context, such as linebox structures or float lists */ 
enum FlowContextData {
    AbsoluteFlow, 
    BlockFlow(BlockFlowData),
    FloatFlow,
    InlineBlockFlow,
    InlineFlow(InlineFlowData),
    RootFlow(RootFlowData),
    TableFlow
}

/* A particular kind of layout context. It manages the positioning of
   render boxes within the context.  */
struct FlowContext {
    kind: FlowContextData,
    data: FlowLayoutData,
    mut node: Option<Node>,
    /* reference to parent, children flow contexts */
    tree: tree::Tree<@FlowContext>,
    /* TODO (Issue #87): debug only */
    mut id: int
}


fn FlowContext(id: int, kind: FlowContextData, tree: tree::Tree<@FlowContext>) -> FlowContext {
    FlowContext {
        kind: kind,
        data: FlowLayoutData(),
        node: None,
        tree: tree,
        id: id
    }
}

impl @FlowContext : cmp::Eq {
    pure fn eq(other: &@FlowContext) -> bool { core::box::ptr_eq(self, *other) }
    pure fn ne(other: &@FlowContext) -> bool { !core::box::ptr_eq(self, *other) }
}


/* Flow context disambiguation methods: the verbose alternative to virtual methods */
impl @FlowContext {
    fn bubble_widths(ctx: &LayoutContext) {
        match self.kind {
            BlockFlow(*)  => self.bubble_widths_block(ctx),
            InlineFlow(*) => self.bubble_widths_inline(ctx),
            RootFlow(*)   => self.bubble_widths_root(ctx),
            _ => fail fmt!("Tried to bubble_widths of flow: %?", self.kind)
        }
    }

    fn assign_widths(ctx: &LayoutContext) {
        match self.kind {
            BlockFlow(*)  => self.assign_widths_block(ctx),
            InlineFlow(*) => self.assign_widths_inline(ctx),
            RootFlow(*)   => self.assign_widths_root(ctx),
            _ => fail fmt!("Tried to assign_widths of flow: %?", self.kind)
        }
    }

    fn assign_height(ctx: &LayoutContext) {
        match self.kind {
            BlockFlow(*)  => self.assign_height_block(ctx),
            InlineFlow(*) => self.assign_height_inline(ctx),
            RootFlow(*)   => self.assign_height_root(ctx),
            _ => fail fmt!("Tried to assign_height of flow: %?", self.kind)
        }
    }

    fn build_display_list_recurse(builder: &dl::DisplayListBuilder, dirty: &Rect<au>,
                                  offset: &Point2D<au>, list: &dl::DisplayList) {
        match self.kind {
            RootFlow(*) => self.build_display_list_root(builder, dirty, offset, list),
            BlockFlow(*) => self.build_display_list_block(builder, dirty, offset, list),
            InlineFlow(*) => self.build_display_list_inline(builder, dirty, offset, list),
            _ => fail fmt!("Tried to build_display_list_recurse of flow: %?", self.kind)
        }
    }
}


/* The tree holding FlowContexts */
enum FlowTree { FlowTree }

impl FlowTree : tree::ReadMethods<@FlowContext> {
    fn each_child(ctx: @FlowContext, f: fn(&&@FlowContext) -> bool) {
        tree::each_child(self, ctx, f)
    }

    fn with_tree_fields<R>(&&b: @FlowContext, f: fn(tree::Tree<@FlowContext>) -> R) -> R {
        f(b.tree)
    }
}

impl FlowTree : tree::WriteMethods<@FlowContext> {
    fn add_child(parent: @FlowContext, child: @FlowContext) {
        assert !core::box::ptr_eq(parent, child);
        tree::add_child(self, parent, child)
    }

    fn with_tree_fields<R>(&&b: @FlowContext, f: fn(tree::Tree<@FlowContext>) -> R) -> R {
        f(b.tree)
    }
}


impl @FlowContext : DebugMethods {
    fn dump() {
        self.dump_indent(0u);
    }

    /** Dumps the flow tree, for debugging, with indentation. */
    fn dump_indent(indent: uint) {
        let mut s = ~"|";
        for uint::range(0u, indent) |_i| {
            s += ~"---- ";
        }

        s += self.debug_str();
        debug!("%s", s);

        for FlowTree.each_child(self) |child| {
            child.dump_indent(indent + 1u) 
        }
    }
    
    /* TODO: we need a string builder. This is horribly inefficient */
    fn debug_str() -> ~str {
        let repr = match self.kind {
            InlineFlow(d) => {
                let mut s = d.boxes.foldl(~"InlineFlow(children=", |s, box| {
                    fmt!("%s %?", s, box.id)
                });
                s += ~")"; s
            },
            BlockFlow(d) => {
                match d.box {
                    Some(_b) => fmt!("BlockFlow(box=b%?)", d.box.get().id),
                    None => ~"BlockFlow",
                }
            },
            _ => fmt!("%?", self.kind)
        };
            
        fmt!("c%? %?", self.id, repr)
    }
}
