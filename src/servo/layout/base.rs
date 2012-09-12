/* Fundamental layout structures and algorithms. */

use au = gfx::geometry;
use core::dvec::DVec;
use core::to_str::ToStr;
use core::rand;
use css::styles::SpecifiedStyle;
use css::values::{BoxSizing, Length, Px, CSSDisplay};
use dom::base::{Element, ElementKind, HTMLDivElement, HTMLImageElement};
use dom::base::{Node, NodeData, NodeKind, NodeTree};
use dom::rcu;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::geometry::au;
use image::{Image, ImageHolder};
use layout::block::BlockFlowData;
use layout::inline::InlineFlowData;
use layout::root::RootFlowData;
use layout::text::TextBoxData;
use servo_text::text_run::TextRun;
use std::arc::{ARC, clone};
use std::net::url::Url;
use task::spawn;
use util::color::Color;
use util::tree;
use vec::{push, push_all};

/* The type of the formatting context, and data specific to each
context, such as lineboxes or float lists */ 
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
   layout boxes within the context.

   Flow contexts form a tree that is induced by the structure of the
   box tree. Each context is responsible for laying out one or more
   boxes, according to the flow type. The number of flow contexts should
   be much fewer than the number of boxes. The context maintains a vector
   of its constituent boxes in their document order.
*/
struct FlowContext {
    kind: FlowContextData,
    data: FlowLayoutData,
    /* reference to parent, children flow contexts */
    tree: tree::Tree<@FlowContext>,
    /* TODO: debug only */
    mut id: int
}

fn FlowContext(id: int, kind: FlowContextData, tree: tree::Tree<@FlowContext>) -> FlowContext {
    FlowContext {
        kind: kind,
        data: FlowLayoutData(),
        tree: tree,
        id: id
    }
}

impl @FlowContext : cmp::Eq {
    pure fn eq(&&other: @FlowContext) -> bool { box::ptr_eq(self, other) }
    pure fn ne(&&other: @FlowContext) -> bool { !box::ptr_eq(self, other) }
}

/* A box's kind influences how its styles are interpreted during
   layout.  For example, replaced content such as images are resized
   differently than tables, text, or other content.

   It also holds data specific to different box types, such as text.
*/
enum BoxData {
    GenericBox,
    ImageBox(Size2D<au>),
    TextBox(TextBoxData)
}

struct Box {
    /* references to children, parent */
    tree : tree::Tree<@Box>,
    /* originating DOM node */
    node : Node,
    /* reference to containing flow context, which this box
       participates in */
    ctx  : @FlowContext,
    /* results of flow computation */
    data : BoxLayoutData,
    /* kind tag and kind-specific data */
    kind : BoxData,
    /* TODO: debug only */
    mut id: int
}

fn Box(id: int, node: Node, ctx: @FlowContext, kind: BoxData) -> Box {
    Box {
        /* will be set when box is parented */
        tree : tree::empty(),
        node : node,
        ctx  : ctx,
        data : BoxLayoutData(),
        kind : kind,
        id : id
    }
}

impl @Box {
    pure fn is_replaced() -> bool {
        match self.kind {
            ImageBox(*) => true, // TODO: form elements, etc
            _ => false
        }
    }

    pure fn get_min_width() -> au {
        match self.kind {
            // TODO: this should account for min/pref widths of the
            // box element in isolation. That includes
            // border/margin/padding but not child widths. The block
            // FlowContext will combine the width of this element and
            // that of its children to arrive at the context width.
            GenericBox => au(0),
            // TODO: If image isn't available, consult Node
            // attrs, etc. to determine intrinsic dimensions. These
            // dimensions are not defined by CSS 2.1, but are defined
            // by the HTML5 spec in Section 4.8.1
            ImageBox(size) => size.width,
            // TODO: account for line breaks, etc. The run should know
            // how to compute its own min and pref widths, and should
            // probably cache them.
            TextBox(d) => d.runs.foldl(au(0), |sum, run| {
                au::max(sum, run.min_break_width())
            })
        }
    }

    pure fn get_pref_width() -> au {
        match self.kind {
            // TODO: this should account for min/pref widths of the
            // box element in isolation. That includes
            // border/margin/padding but not child widths. The block
            // FlowContext will combine the width of this element and
            // that of its children to arrive at the context width.
            GenericBox => au(0),
            // TODO: If image isn't available, consult Node
            // attrs, etc. to determine intrinsic dimensions. These
            // dimensions are not defined by CSS 2.1, but are defined
            // by the HTML5 spec in Section 4.8.1
            ImageBox(size) => size.width,
            // TODO: account for line breaks, etc. The run should know
            // how to compute its own min and pref widths, and should
            // probably cache them.
            TextBox(d) => d.runs.foldl(au(0), |sum, run| {
                au::max(sum, run.size().width)
            })
        }
    }

    /* Returns the amount of left, right "fringe" used by this
    box. This should be based on margin, border, padding, width. */
    fn get_used_width() -> (au, au) {
        // TODO: this should actually do some computation!
        // See CSS 2.1, Section 10.3, 10.4.

        (au(0), au(0))
    }
    
    /* Returns the amount of left, right "fringe" used by this
    box. This should be based on margin, border, padding, width. */
    fn get_used_height() -> (au, au) {
        // TODO: this should actually do some computation!
        // See CSS 2.1, Section 10.5, 10.6.

        (au(0), au(0))
    }

    // This will be very unhappy if it is getting run in parallel with
    // anything trying to read the background image
    fn get_image() -> Option<ARC<~Image>> {
        let mut image = None;

        // Do a dance where we swap the ImageHolder out before we can
        // get the image out of it because we can't match against it
        // because holder.get_image() is not pure.
        if (self.data.background_image).is_some() {
            let mut temp = None;
            temp <-> self.data.background_image;
            let holder <- option::unwrap(temp);
            image = holder.get_image();
            self.data.background_image = Some(holder);
        }

        image
    }
}

struct FlowLayoutData {
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

struct BoxLayoutData {
    mut min_width: au,
    mut pref_width: au,
    mut position: Rect<au>,

    mut font_size: Length,
    mut background_image: Option<ImageHolder>,
}

fn BoxLayoutData() -> BoxLayoutData {
    BoxLayoutData {
        min_width: au(0),
        pref_width: au(0),
        position : au::zero_rect(),

        font_size : Px(0.0),
        background_image : None,
    }
}

// FIXME: Why do these have to be redefined for each node type?

/* The tree holding boxes */
enum BoxTree { BoxTree }

impl BoxTree : tree::ReadMethods<@Box> {
    fn each_child(node: @Box, f: fn(&&@Box) -> bool) {
        tree::each_child(self, node, f)
    }

    fn with_tree_fields<R>(&&b: @Box, f: fn(tree::Tree<@Box>) -> R) -> R {
        f(b.tree)
    }
}

impl BoxTree : tree::WriteMethods<@Box> {
    fn add_child(parent: @Box, child: @Box) {
        assert !box::ptr_eq(parent, child);
        tree::add_child(self, parent, child)
    }

    fn with_tree_fields<R>(&&b: @Box, f: fn(tree::Tree<@Box>) -> R) -> R {
        f(b.tree)
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
        assert !box::ptr_eq(parent, child);
        tree::add_child(self, parent, child)
    }

    fn with_tree_fields<R>(&&b: @FlowContext, f: fn(tree::Tree<@FlowContext>) -> R) -> R {
        f(b.tree)
    }
}

impl @FlowContext {
    fn bubble_widths() {
        match self.kind {
            BlockFlow(*)  => self.bubble_widths_block(),
            InlineFlow(*) => self.bubble_widths_inline(),
            RootFlow(*)   => self.bubble_widths_root(),
            _ => fail fmt!("Tried to bubble_widths of flow: %?", self.kind)
        }
    }

    fn assign_widths() {
        match self.kind {
            BlockFlow(*)  => self.assign_widths_block(),
            InlineFlow(*) => self.assign_widths_inline(),
            RootFlow(*)   => self.assign_widths_root(),
            _ => fail fmt!("Tried to assign_widths of flow: %?", self.kind)
        }
    }

    fn assign_height() {
        match self.kind {
            BlockFlow(*)  => self.assign_height_block(),
            InlineFlow(*) => self.assign_height_inline(),
            RootFlow(*)   => self.assign_height_root(),
            _ => fail fmt!("Tried to assign_height of flow: %?", self.kind)
        }
    }
}

// Debugging

trait DebugMethods {
    fn dump();
    fn dump_indent(ident: uint);
    fn debug_str() -> ~str;
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
                    Some(b) => fmt!("BlockFlow(box=b%?)", d.box.get().id),
                    None => ~"BlockFlow",
                }
            },
            _ => fmt!("%?", self.kind)
        };
            
        fmt!("c%? %?", self.id, repr)
    }
}

impl Node : DebugMethods {
    /* Dumps the subtree rooted at this node, for debugging. */
    fn dump() {
        self.dump_indent(0u);
    }
    /* Dumps the node tree, for debugging, with indentation. */
    fn dump_indent(indent: uint) {
        let mut s = ~"";
        for uint::range(0u, indent) |_i| {
            s += ~"    ";
        }

        s += self.debug_str();
        debug!("%s", s);

        for NodeTree.each_child(self) |kid| {
            kid.dump_indent(indent + 1u) 
        }
    }

    fn debug_str() -> ~str {
        fmt!("%?", self.read(|n| copy n.kind ))
    }
}

impl @Box : DebugMethods {
    fn dump() {
        self.dump_indent(0u);
    }

    /* Dumps the node tree, for debugging, with indentation. */
    fn dump_indent(indent: uint) {
        let mut s = ~"";
        for uint::range(0u, indent) |_i| {
            s += ~"    ";
        }

        s += self.debug_str();
        debug!("%s", s);

        for BoxTree.each_child(self) |kid| {
            kid.dump_indent(indent + 1u) 
        }
    }

    fn debug_str() -> ~str {
        let repr = match self.kind {
            GenericBox(*) => ~"GenericBox",
            ImageBox(*) => ~"ImageBox",
            TextBox(d) => {
                let mut s = d.runs.foldl(~"TextBox(runs=", |s, run| {
                    fmt!("%s  \"%s\"", s, run.text)
                });
                s += ~")"; s
            }
        };

        fmt!("box b%?: %?", self.id, repr)
    }
}

#[cfg(test)]
mod test {
    use dom::base::{Element, ElementData, HTMLDivElement, HTMLImageElement, Node, NodeKind};
    use dom::base::{NodeScope};
    use dom::rcu::Scope;

    /*
    use sdl;
    use sdl::video;

    fn with_screen(f: fn(*sdl::surface)) {
        let screen = video::set_video_mode(
            320, 200, 32,
            ~[video::hwsurface], ~[video::doublebuf]);
        assert screen != ptr::null();

        f(screen);

        video::free_surface(screen);
    }
    */

    fn flat_bounds(root: @Box) -> ~[Rect<au>] {
        let mut r = ~[];
        for tree::each_child(BoxTree, root) |c| {
            push_all(r, flat_bounds(c));
        }

        push(r, copy root.data.position);

        return r;
    }

    // TODO: redo tests here, but probably is part of box_builder.rs
}

