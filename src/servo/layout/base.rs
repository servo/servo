#[doc="Fundamental layout structures and algorithms."]

import dom::base::{Element, ElementKind, HTMLDivElement, HTMLImageElement, node_data, node_kind};
import dom::base::{Node};
import dom::rcu;
import dom::rcu::reader_methods;
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

enum box_kind {
    bk_block,
    bk_inline,
    bk_intrinsic(@Size2D<au>),
    bk_text(@text_box)
}

class appearance {
    let mut background_image: option<@image>;
    let mut background_color: option<Color>;

    new() {
        self.background_image = none;
        self.background_color = none;
    }
}

enum box = {
    tree: tree::fields<@box>,
    node: Node,
    mut bounds: Rect<au>,
    kind: box_kind,
    appearance: appearance
};

enum layout_data = {
    mut computed_style: computed_style,
    mut box: option<@box>
};

enum ntree { ntree }
impl of tree::rd_tree_ops<Node> for ntree {
    fn each_child(node: Node, f: fn(Node) -> bool) {
        tree::each_child(self, node, f)
    }

    fn with_tree_fields<R>(&&n: Node, f: fn(tree::fields<Node>) -> R) -> R {
        n.rd { |n| f(n.tree) }
    }
}

enum btree { btree }
impl of tree::rd_tree_ops<@box> for btree {
    fn each_child(node: @box, f: fn(&&@box) -> bool) {
        tree::each_child(self, node, f)
    }

    fn with_tree_fields<R>(&&b: @box, f: fn(tree::fields<@box>) -> R) -> R {
        f(b.tree)
    }
}

impl of tree::wr_tree_ops<@box> for btree {
    fn add_child(node: @box, child: @box) {
        tree::add_child(self, node, child)
    }

    fn with_tree_fields<R>(&&b: @box, f: fn(tree::fields<@box>) -> R) -> R {
        f(b.tree)
    }
}

impl layout_methods_priv for @box {
    #[doc="Dumps the box tree, for debugging, with indentation."]
    fn dump_indent(indent: uint) {
        let mut s = "";
        for uint::range(0u, indent) {
            |_i|
            s += "    ";
        }

        s += #fmt("%?", self.kind);
        #debug["%s", s];

        for btree.each_child(self) { |kid| kid.dump_indent(indent + 1u) }
    }
}

impl layout_methods for @box {
    #[doc="The main reflow routine."]
    fn reflow(available_width: au) {
        alt self.kind {
            bk_block { self.reflow_block(available_width) }
            bk_inline { self.reflow_inline(available_width) }
            bk_intrinsic(size) { self.reflow_intrinsic(*size) }
            bk_text(subbox) { self.reflow_text(available_width, subbox) }
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

impl node_methods_priv for Node {
    #[doc="Dumps the node tree, for debugging, with indentation."]
    fn dump_indent(indent: uint) {
        let mut s = "";
        for uint::range(0u, indent) {
            |_i|
            s += "    ";
        }

        s += #fmt("%?", self.rd({ |n| copy n.kind }));
        #debug["%s", s];

        for ntree.each_child(self) { |kid| kid.dump_indent(indent + 1u) }
    }
}

impl node_methods for Node {
    #[doc="Dumps the subtree rooted at this node, for debugging."]
    fn dump() {
        self.dump_indent(0u);
    }
}

#[cfg(test)]
mod test {
    import dom::base::{ElementData, HTMLDivElement, HTMLImageElement, methods, Element, node_data};
    import dom::base::{Node, node_kind, wr_tree_ops};
    import dom::rcu::scope;
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

    fn flat_bounds(root: @box) -> [Rect<au>] {
        let mut r = [];
        for tree::each_child(btree, root) {|c|
            r += flat_bounds(c);
        }
        ret r + [copy root.bounds];
    }

    #[test]
    #[ignore(reason = "busted")]
    fn do_layout() {
        let s = scope();

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

        tree::add_child(btree, b3, b0);
        tree::add_child(btree, b3, b1);
        tree::add_child(btree, b3, b2);

        b3.reflow_block(au(100));
        let fb = flat_bounds(b3);
        #debug["fb=%?", fb];
        assert fb == [geometry::box(au(0), au(0), au(10), au(10)),   // n0
                      geometry::box(au(0), au(10), au(10), au(15)),  // n1
                      geometry::box(au(0), au(25), au(10), au(20)),  // n2
                      geometry::box(au(0), au(0), au(100), au(45))]; // n3
    }
}

