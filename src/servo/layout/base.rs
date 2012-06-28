#[doc="Fundamental layout structures and algorithms."]

import dom::base::{Element, ElementKind, HTMLDivElement, HTMLImageElement, Node, NodeData};
import dom::base::{NodeKind};
import dom::rcu;
import dom::rcu::ReaderMethods;
import gfx::geometry;
import gfx::geometry::{au, zero_size_au};
import geom::point::Point2D;
import geom::rect::Rect;
import geom::size::Size2D;
import image::base::image;
import layout::block::block_layout_methods;
import layout::inline::inline_layout_methods;
import layout::style::style::*;
import layout::text::*;
import util::tree;
import util::color::Color;

enum BoxKind {
    BlockBox,
    InlineBox,
    IntrinsicBox(@Size2D<au>),
    TextBox(@text_box)
}

class Appearance {
    let mut background_image: option<@image>;
    let mut background_color: option<Color>;

    new() {
        self.background_image = none;
        self.background_color = none;
    }
}

class Box {
    let tree: tree::Tree<@Box>;
    let node: Node;
    let kind: BoxKind;
    let mut bounds: Rect<au>;
    let appearance: Appearance;

    new(node: Node, kind: BoxKind) {
        self.tree = tree::empty();
        self.node = node;
        self.kind = kind;
        self.bounds = geometry::zero_rect_au();
        self.appearance = Appearance();
    }
}

enum LayoutData = {
    mut computed_style: ~computed_style,
    mut box: option<@Box>
};

// FIXME: This is way too complex! Why do these have to have dummy receivers? --pcw

enum NTree { NTree }
impl NodeTreeReadMethods of tree::ReadMethods<Node> for NTree {
    fn each_child(node: Node, f: fn(Node) -> bool) {
        tree::each_child(self, node, f)
    }

    fn with_tree_fields<R>(&&n: Node, f: fn(tree::Tree<Node>) -> R) -> R {
        n.read { |n| f(n.tree) }
    }
}

enum BTree { BTree }
impl BoxTreeReadMethods of tree::ReadMethods<@Box> for BTree {
    fn each_child(node: @Box, f: fn(&&@Box) -> bool) {
        tree::each_child(self, node, f)
    }

    fn with_tree_fields<R>(&&b: @Box, f: fn(tree::Tree<@Box>) -> R) -> R {
        f(b.tree)
    }
}

impl BoxTreeWriteMethods of tree::WriteMethods<@Box> for BTree {
    fn add_child(node: @Box, child: @Box) {
        tree::add_child(self, node, child)
    }

    fn with_tree_fields<R>(&&b: @Box, f: fn(tree::Tree<@Box>) -> R) -> R {
        f(b.tree)
    }
}

impl layout_methods_priv for @Box {
    #[doc="Dumps the box tree, for debugging, with indentation."]
    fn dump_indent(indent: uint) {
        let mut s = "";
        for uint::range(0u, indent) {
            |_i|
            s += "    ";
        }

        s += #fmt("%?", self.kind);
        #debug["%s", s];

        for BTree.each_child(self) { |kid| kid.dump_indent(indent + 1u) }
    }
}

impl layout_methods for @Box {
    #[doc="The main reflow routine."]
    fn reflow(available_width: au) {
        alt self.kind {
            BlockBox            { self.reflow_block(available_width)        }
            InlineBox           { self.reflow_inline(available_width)       }
            IntrinsicBox(size)  { self.reflow_intrinsic(*size)              }
            TextBox(subbox)     { self.reflow_text(available_width, subbox) }
        }
    }

    #[doc="The trivial reflow routine for instrinsically-sized frames."]
    fn reflow_intrinsic(size: Size2D<au>) {
        self.bounds.size = copy size;

        #debug["reflow_intrinsic size=%?", copy self.bounds];
    }

    #[doc="Dumps the box tree, for debugging."]
    fn dump() {
        self.dump_indent(0u);
    }
}

// Debugging

impl PrivateNodeMethods for Node {
    #[doc="Dumps the node tree, for debugging, with indentation."]
    fn dump_indent(indent: uint) {
        let mut s = "";
        for uint::range(0u, indent) {
            |_i|
            s += "    ";
        }

        s += #fmt("%?", self.read({ |n| copy n.kind }));
        #debug["%s", s];

        for NTree.each_child(self) { |kid| kid.dump_indent(indent + 1u) }
    }
}

impl NodeMethods for Node {
    #[doc="Dumps the subtree rooted at this node, for debugging."]
    fn dump() {
        self.dump_indent(0u);
    }
}

#[cfg(test)]
mod test {
    import dom::base::{Element, ElementData, HTMLDivElement, HTMLImageElement, Node, NodeKind};
    import dom::base::{NodeScope, TreeWriteMethods};
    import dom::rcu::Scope;
    import box_builder::{box_builder_methods};

    /*
    use sdl;
    import sdl::video;

    fn with_screen(f: fn(*sdl::surface)) {
        let screen = video::set_video_mode(
            320, 200, 32,
            [video::hwsurface], [video::doublebuf]);
        assert screen != ptr::null();

        f(screen);

        video::free_surface(screen);
    }
    */

    fn flat_bounds(root: @Box) -> [Rect<au>] {
        let mut r = [];
        for tree::each_child(BTree, root) {|c|
            r += flat_bounds(c);
        }
        ret r + [copy root.bounds];
    }

    #[test]
    #[ignore(reason = "busted")]
    fn do_layout() {
        let s = Scope();

        fn mk_img(size: Size2D<au>) -> ~ElementKind {
            ~HTMLImageElement({mut size: size})
        }

        let n0 = s.new_node(Element(ElementData("img", mk_img(Size2D(au(10),au(10))))));
        let n1 = s.new_node(Element(ElementData("img", mk_img(Size2D(au(10),au(10))))));
        let n2 = s.new_node(Element(ElementData("img", mk_img(Size2D(au(10),au(20))))));
        let n3 = s.new_node(Element(ElementData("div", ~HTMLDivElement)));

        tree::add_child(s, n3, n0);
        tree::add_child(s, n3, n1);
        tree::add_child(s, n3, n2);

        let b0 = n0.construct_boxes();
        let b1 = n1.construct_boxes();
        let b2 = n2.construct_boxes();
        let b3 = n3.construct_boxes();

        tree::add_child(BTree, b3, b0);
        tree::add_child(BTree, b3, b1);
        tree::add_child(BTree, b3, b2);

        b3.reflow_block(au(100));
        let fb = flat_bounds(b3);
        #debug["fb=%?", fb];
        assert fb == [geometry::box(au(0), au(0), au(10), au(10)),   // n0
                      geometry::box(au(0), au(10), au(10), au(15)),  // n1
                      geometry::box(au(0), au(25), au(10), au(20)),  // n2
                      geometry::box(au(0), au(0), au(100), au(45))]; // n3
    }
}

