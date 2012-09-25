/* Fundamental layout structures and algorithms. */

use arc = std::arc;
use arc::ARC;
use au = gfx::geometry;
use au::au;
use core::dvec::DVec;
use core::to_str::ToStr;
use core::rand;
use css::styles::SpecifiedStyle;
use css::values::{BoxSizing, Length, Px, CSSDisplay, Specified, BgColor, BgColorTransparent};
use dl = gfx::display_list;
use dom::element::{ElementKind, HTMLDivElement, HTMLImageElement};
use dom::node::{Element, Node, NodeData, NodeKind, NodeTree};
use dom::rcu;
use geom::rect::Rect;
use geom::size::Size2D;
use geom::point::Point2D;
use image::{Image, ImageHolder};
use layout::context::LayoutContext;
use layout::debug::DebugMethods;
use layout::flow::FlowContext;
use layout::text::TextBoxData;
use servo_text::text_run::TextRun;
use std::net::url::Url;
use task::spawn;
use util::color::Color;
use util::tree;
use vec::{push, push_all};

/** 
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

*/


/* A box's kind influences how its styles are interpreted during
   layout.  For example, replaced content such as images are resized
   differently than tables, text, or other content.

   It also holds data specific to different box types, such as text.
*/

struct BoxLayoutData {
    mut position: Rect<au>,
    mut font_size: Length,
}

/* TODO: this should eventually be just 'position', and
   merged into the base RenderBox struct */
fn BoxLayoutData() -> BoxLayoutData {
    BoxLayoutData {
        position : au::zero_rect(),
        font_size : Px(0.0),
    }
}

enum BoxData {
    GenericBox,
    ImageBox(ImageHolder),
    TextBox(TextBoxData)
}

struct RenderBox {
    /* references to children, parent inline flow boxes  */
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
    /* TODO (Issue #87): debug only */
    mut id: int
}

fn RenderBox(id: int, node: Node, ctx: @FlowContext, kind: BoxData) -> RenderBox {
    RenderBox {
        /* will be set if box is parented */
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

    /** In general, these functions are transitively impure because they
     * may cause glyphs to be allocated. For now, it's impure because of 
     * holder.get_image()
    */
    fn get_min_width() -> au {
        match self.kind {
            // TODO: this should account for min/pref widths of the
            // box element in isolation. That includes
            // border/margin/padding but not child widths. The block
            // FlowContext will combine the width of this element and
            // that of its children to arrive at the context width.
            GenericBox => au(0),
            // TODO: consult CSS 'width', margin, border.
            // TODO: If image isn't available, consult 'width'.
            ImageBox(i) => au::from_px(i.get_size().get_default(Size2D(0,0)).width),
            TextBox(d) => d.runs.foldl(au(0), |sum, run| {
                au::max(sum, run.min_break_width())
            })
        }
    }

    fn get_pref_width() -> au {
        match self.kind {
            // TODO: this should account for min/pref widths of the
            // box element in isolation. That includes
            // border/margin/padding but not child widths. The block
            // FlowContext will combine the width of this element and
            // that of its children to arrive at the context width.
            GenericBox => au(0),
            ImageBox(i) => au::from_px(i.get_size().get_default(Size2D(0,0)).width),
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

    /* The box formed by the content edge, as defined in CSS 2.1 Section 8.1.
       Coordinates are relative to the owning flow. */
    pure fn content_box() -> Rect<au> {
        match self.kind {
            ImageBox(i) => {
                let size = i.size();
                Rect {
                    origin: copy self.data.position.origin,
                    size:   Size2D(au::from_px(size.width),
                                   au::from_px(size.height))
                }
            },
            GenericBox(*) => {
                copy self.data.position
                /* FIXME: The following hits an ICE for whatever reason

                let origin = self.data.position.origin;
                let size   = self.data.position.size;
                let (offset_left, offset_right) = self.get_used_width();
                let (offset_top, offset_bottom) = self.get_used_height();

                Rect {
                    origin: Point2D(origin.x + offset_left, origin.y + offset_top),
                    size:   Size2D(size.width - (offset_left + offset_right),
                                   size.height - (offset_top + offset_bottom))
                }*/
            },
            TextBox(*) => {
                copy self.data.position
            }
        }
    }

    /* The box formed by the border edge, as defined in CSS 2.1 Section 8.1.
       Coordinates are relative to the owning flow. */
    fn border_box() -> Rect<au> {
        // TODO: actually compute content_box + padding + border
        self.content_box()
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
            ImageBox(i) => {
                match i.get_image() {
                    Some(image) => list.push(~dl::Image(bounds, image)),
                    /* No image data at all? Okay, add some fallback content instead. */
                    None => {
                        // TODO: shouldn't need to unbox CSSValue by now
                        let boxed_color = self.node.style().background_color;
                        let color = match boxed_color {
                            Specified(BgColor(c)) => c,
                            Specified(BgColorTransparent) | _ => util::color::rgba(0,0,0,0.0)
                        };
                        list.push(~dl::SolidColor(bounds, color.red, color.green, color.blue));
                    }
                }
            }
        }
    }
}

/**
 * The tree holding render box relations. These are only defined for
 * nested CSS boxes that are nested in an otherwise inline flow
 * context.
*/
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
        assert !core::box::ptr_eq(parent, child);
        tree::add_child(self, parent, child)
    }

    fn with_tree_fields<R>(&&b: @RenderBox, f: fn(tree::Tree<@RenderBox>) -> R) -> R {
        f(b.tree)
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
    use dom::element::{ElementData, HTMLDivElement, HTMLImageElement};
    use dom::node::{Element, NodeScope, Node, NodeKind};
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

