/* Fundamental layout structures and algorithms. */

use arc = std::arc;
use arc::ARC;
use au = gfx::geometry;
use au::au;
use core::dvec::DVec;
use core::to_str::ToStr;
use core::rand;
use css::styles::SpecifiedStyle;
use css::values::{BoxSizing, Length, Px, CSSDisplay, Specified, BgColor, BgTransparent};
use dl = gfx::display_list;
use dom::element::{ElementKind, HTMLDivElement, HTMLImageElement};
use dom::base::{Element, Node, NodeData, NodeKind, NodeTree};
use dom::rcu;
use geom::rect::Rect;
use geom::size::Size2D;
use geom::point::Point2D;
use image::{Image, ImageHolder};
use layout::block::BlockFlowData;
use layout::context::LayoutContext;
use layout::debug::DebugMethods;
use layout::inline::InlineFlowData;
use layout::root::RootFlowData;
use layout::text::TextBoxData;
use servo_text::text_run::TextRun;
use std::net::url::Url;
use task::spawn;
use util::color::Color;
use util::tree;
use vec::{push, push_all};


/** Servo's experimental layout system builds a tree of FlowContexts
and RenderBoxes, and figures out positions and display attributes of
tree nodes. Positions are computed in several tree traversals driven
by fundamental data dependencies of inline and block layout.

Render boxes (`struct RenderBox`) are the leafs of the layout
tree. They cannot position themselves. In general, render boxes do not
have a simple correspondence with CSS boxes as in the specification:

 * Several render boxes may correspond to the same CSS box or DOM
   node. For example, a CSS text box broken across two lines is
   represented by two render boxes.

 * Some CSS boxes are not created at all, such as some anonymous block
   boxes induced by inline boxes with block-level sibling boxes. In
   that case, Servo uses an InlineFlow with BlockFlow siblings; the
   InlineFlow is block-level, but not a block container. It is
   positioned as if it were a block box, but its children are
   positioned according to inline flow.

Fundamental box types include:

 * GenericBox: an empty box that contributes only borders, margins,
padding, backgrounds. It is analogous to a CSS nonreplaced content box.

 * ImageBox: a box that represents a (replaced content) image and its
   accompanying borders, shadows, etc.

 * TextBox: a box representing a single run of text with a distinct
   style. A TextBox may be split into two or more render boxes across
   line breaks. Several TextBoxes may correspond to a single DOM text
   node. Split text boxes are implemented by referring to subsets of a
   master TextRun object.


Flows (`struct FlowContext`) are interior nodes in the layout tree,
and correspond closely to flow contexts in the CSS
specification. Flows are responsible for positioning their child flow
contexts and render boxes. Flows have purpose-specific fields, such as
auxilliary line box structs, out-of-flow child lists, and so on.

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
    pure fn eq(other: &@FlowContext) -> bool { box::ptr_eq(self, *other) }
    pure fn ne(other: &@FlowContext) -> bool { !box::ptr_eq(self, *other) }
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
        assert !box::ptr_eq(parent, child);
        tree::add_child(self, parent, child)
    }

    fn with_tree_fields<R>(&&b: @FlowContext, f: fn(tree::Tree<@FlowContext>) -> R) -> R {
        f(b.tree)
    }
}


/* A box's kind influences how its styles are interpreted during
   layout.  For example, replaced content such as images are resized
   differently than tables, text, or other content.

   It also holds data specific to different box types, such as text.
*/

struct BoxLayoutData {
    mut position: Rect<au>,
    mut font_size: Length,
    mut background_image: Option<ImageHolder>,
}

fn BoxLayoutData() -> BoxLayoutData {
    BoxLayoutData {
        position : au::zero_rect(),
        font_size : Px(0.0),
        background_image : None,
    }
}

enum BoxData {
    GenericBox,
    ImageBox(Size2D<au>),
    TextBox(TextBoxData)
}

struct RenderBox {
    /* references to children, parent */
    tree : tree::Tree<@RenderBox>,
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

fn RenderBox(id: int, node: Node, ctx: @FlowContext, kind: BoxData) -> RenderBox {
    RenderBox {
        /* will be set when box is parented */
        tree : tree::empty(),
        node : node,
        ctx  : ctx,
        data : BoxLayoutData(),
        kind : kind,
        id : id
    }
}

impl @RenderBox {
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
            // TODO: consult CSS 'width', margin, border.
            // TODO: If image isn't available, consult Node
            // attrs, etc. to determine intrinsic dimensions. These
            // dimensions are not defined by CSS 2.1, but are defined
            // by the HTML5 spec in Section 4.8.1
            ImageBox(size) => size.width,
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

    // TODO: to implement stacking contexts correctly, we need to
    // create a set of display lists, one per each layer of a stacking
    // context. (CSS 2.1, Section 9.9.1). Each box is passed the list
    // set representing the box's stacking context. When asked to
    // construct its constituent display items, each box puts its
    // DisplayItems into the correct stack layer (according to CSS 2.1
    // Appendix E).  and then builder flattens the list at the end.

    /* Methods for building a display list. This is a good candidate
       for a function pointer as the number of boxes explodes.

    # Arguments

    * `builder` - the display list builder which manages the coordinate system and options.
    * `dirty` - Dirty rectangle, in the coordinate system of the owning flow (self.ctx)
    * `origin` - Total offset from display list root flow to this box's owning flow
    * `list` - List to which items should be appended
    */
    fn build_display_list(_builder: &dl::DisplayListBuilder, dirty: &Rect<au>, 
                          offset: &Point2D<au>, list: &dl::DisplayList) {
        if !self.data.position.intersects(dirty) {
            return;
        }

        let bounds : Rect<au> = Rect(self.data.position.origin.add(offset),
                                     copy self.data.position.size);

        match self.kind {
            TextBox(d) => {
                let mut runs = d.runs;
                // TODO: don't paint background for text boxes
                list.push(~dl::SolidColor(bounds, 255u8, 255u8, 255u8));
                
                let mut bounds = bounds;
                for uint::range(0, runs.len()) |i| {
                    bounds.size.height = runs[i].size().height;
                    let glyph_run = make_glyph_run(&runs[i]);
                    list.push(~dl::Glyphs(bounds, glyph_run));
                    bounds.origin.y = bounds.origin.y.add(bounds.size.height);
                }
                return;

                pure fn make_glyph_run(text_run: &TextRun) -> dl::GlyphRun {
                    dl::GlyphRun {
                        glyphs: copy text_run.glyphs
                    }
                }
            },
            // TODO: items for background, border, outline
            GenericBox(*) => { },
            ImageBox(*) => {
                match self.get_image() {
                    Some(image) => list.push(~dl::Image(bounds, image)),
                    /* No image data at all? Okay, add some fallback content instead. */
                    None => {
                        // TODO: shouldn't need to unbox CSSValue by now
                        let boxed_color = self.node.style().background_color;
                        let color = match boxed_color {
                            Specified(BgColor(c)) => c,
                            Specified(BgTransparent) | _ => util::color::rgba(0,0,0,0.0)
                        };
                        list.push(~dl::SolidColor(bounds, color.red, color.green, color.blue));
                    }
                }
            }
        }
    }
}

// FIXME: Why do these have to be redefined for each node type?

/* The tree holding render box relations. (This should only be used 
for painting nested inlines, AFAIK-- everything else depends on Flow tree)  */
enum RenderBoxTree { RenderBoxTree }

impl RenderBoxTree : tree::ReadMethods<@RenderBox> {
    fn each_child(node: @RenderBox, f: fn(&&@RenderBox) -> bool) {
        tree::each_child(self, node, f)
    }

    fn with_tree_fields<R>(&&b: @RenderBox, f: fn(tree::Tree<@RenderBox>) -> R) -> R {
        f(b.tree)
    }
}

impl RenderBoxTree : tree::WriteMethods<@RenderBox> {
    fn add_child(parent: @RenderBox, child: @RenderBox) {
        assert !box::ptr_eq(parent, child);
        tree::add_child(self, parent, child)
    }

    fn with_tree_fields<R>(&&b: @RenderBox, f: fn(tree::Tree<@RenderBox>) -> R) -> R {
        f(b.tree)
    }
}

// Debugging

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

impl @RenderBox : DebugMethods {
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

        for RenderBoxTree.each_child(self) |kid| {
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

    fn flat_bounds(root: @RenderBox) -> ~[Rect<au>] {
        let mut r = ~[];
        for tree::each_child(RenderBoxTree, root) |c| {
            push_all(r, flat_bounds(c));
        }

        push(r, copy root.data.position);

        return r;
    }

    // TODO: redo tests here, but probably is part of box_builder.rs
}

