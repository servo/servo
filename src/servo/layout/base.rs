#[doc="Fundamental layout structures and algorithms."]

import dom::base::{Element, ElementKind, HTMLDivElement, HTMLImageElement, Node, NodeData};
import dom::base::{NodeKind};
import dom::rcu;
import dom::style::Unit;
import gfx::geometry;
import gfx::geometry::{au, zero_size_au};
import geom::point::Point2D;
import geom::rect::Rect;
import geom::size::Size2D;
import image::base::Image;
import util::tree;
import util::color::Color;
import text::TextBox;
import traverse::extended_full_traversal;
import style::style::{SpecifiedStyle};
import vec::{push, push_all};
import std::net::url::url;
import resource::image_cache_task;
import image_cache_task::ImageCacheTask;
import core::to_str::ToStr;
import std::arc::{arc, clone};
import task::spawn;

enum BoxKind {
    BlockBox,
    InlineBox,
    IntrinsicBox(@Size2D<au>),
    TextBoxKind(@TextBox)
}

struct Appearance {
    let mut background_image: option<ImageHolder>;
    let mut background_color: Color;
    let mut width: Unit;
    let mut height: Unit;

    new(kind: NodeKind) {
        self.background_image = none;
        self.background_color = kind.default_color();
        self.width = kind.default_width();
        self.height = kind.default_height();
    }

    // This will be very unhappy if it is getting run in parallel with
    // anything trying to read the background image
    fn get_image() -> option<~arc<~Image>> {
        let mut image = none;

        // Do a dance where we swap the ImageHolder out before we can
        // get the image out of it because we can't match against it
        // because holder.get_image() is not pure.
        if (self.background_image).is_some() {
            let mut temp = none;
            temp <-> self.background_image;
            let holder <- option::unwrap(temp);
            image = holder.get_image();
            self.background_image = some(holder);
        }

        return image;
    }
}

struct Box {
    let tree: tree::Tree<@Box>;
    let node: Node;
    let kind: BoxKind;
    let mut bounds: Rect<au>;
    let appearance: Appearance;

    new(node: Node, kind: BoxKind) {
        self.appearance = node.read(|n| Appearance(*n.kind));
        self.tree = tree::empty();
        self.node = node;
        self.kind = kind;
        self.bounds = geometry::zero_rect_au();
    }
}

#[doc="A struct to store image data.  The image will be loaded once,
 the first time it is requested, and an arc will be stored.  Clones of
 this arc are given out on demand."]
struct ImageHolder {
    // Invariant: at least one of url and image is not none, except
    // occasionally while get_image is being called
    let mut url : option<url>;
    let mut image : option<arc<~Image>>;
    let image_cache_task: ImageCacheTask;
    let reflow: fn~();

    new(-url : url, image_cache_task: ImageCacheTask, reflow: fn~()) {
        self.url = some(copy url);
        self.image = none;
        self.image_cache_task = image_cache_task;
        self.reflow = copy reflow;

        // Tell the image cache we're going to be interested in this url
        // FIXME: These two messages must be sent to prep an image for use
        // but they are intended to be spread out in time. Ideally prefetch
        // should be done as early as possible and decode only once we
        // are sure that the image will be used.
        image_cache_task.send(image_cache_task::Prefetch(copy url));
        image_cache_task.send(image_cache_task::Decode(move url));
    }

    // This function should not be called by two tasks at the same time
    fn get_image() -> option<~arc<~Image>> {
        // If this is the first time we've called this function, load
        // the image and store it for the future
        if self.image.is_none() {
            assert self.url.is_some();

            let mut temp = none;
            temp <-> self.url;
            let url = option::unwrap(temp);

            let response_port = port();
            self.image_cache_task.send(image_cache_task::GetImage(copy url, response_port.chan()));
            self.image = match response_port.recv() {
              image_cache_task::ImageReady(image) => some(clone(&image)),
              image_cache_task::ImageNotReady => {
                // Need to reflow when the image is available
                let image_cache_task = self.image_cache_task;
                let reflow = copy self.reflow;
                do spawn |copy url, move reflow| {
                    let response_port = port();
                    image_cache_task.send(image_cache_task::WaitForImage(copy url, response_port.chan()));
                    match response_port.recv() {
                      image_cache_task::ImageReady(*) => reflow(),
                      image_cache_task::ImageNotReady => fail /*not possible*/,
                      image_cache_task::ImageFailed => ()
                    }
                }
                none
              }
              image_cache_task::ImageFailed => {
                #info("image was not ready for %s", url.to_str());
                // FIXME: Need to schedule another layout when the image is ready
                none
              }
            };
        }

        if self.image.is_some() {
            // Temporarily swap out the arc of the image so we can clone
            // it without breaking purity, then put it back and return the
            // clone.  This is not threadsafe.
            let mut temp = none;
            temp <-> self.image;
            let im_arc = option::unwrap(temp);
            self.image = some(clone(&im_arc));

            return some(~im_arc);
        } else {
            return none;
        }
    }
}

enum LayoutData = {
    mut specified_style: ~SpecifiedStyle,
    mut box: option<@Box>
};

// FIXME: This is way too complex! Why do these have to have dummy receivers? --pcw

enum NTree { NTree }
impl NTree : tree::ReadMethods<Node> {
    fn each_child(node: Node, f: fn(Node) -> bool) {
        tree::each_child(self, node, f)
    }

    fn with_tree_fields<R>(&&n: Node, f: fn(tree::Tree<Node>) -> R) -> R {
        n.read(|n| f(n.tree))
    }
}

enum BTree { BTree }

impl BTree : tree::ReadMethods<@Box> {
    fn each_child(node: @Box, f: fn(&&@Box) -> bool) {
        tree::each_child(self, node, f)
    }

    fn with_tree_fields<R>(&&b: @Box, f: fn(tree::Tree<@Box>) -> R) -> R {
        f(b.tree)
    }
}

impl BTree : tree::WriteMethods<@Box> {
    fn add_child(node: @Box, child: @Box) {
        tree::add_child(self, node, child)
    }

    fn with_tree_fields<R>(&&b: @Box, f: fn(tree::Tree<@Box>) -> R) -> R {
        f(b.tree)
    }
}

impl @Box {
    #[doc="The main reflow routine."]
    fn reflow() {
        match self.kind {
            BlockBox => self.reflow_block(),
            InlineBox => self.reflow_inline(),
            IntrinsicBox(size) => self.reflow_intrinsic(*size),
            TextBoxKind(subbox) => self.reflow_text(subbox)
        }
    }

    #[doc="Dumps the box tree, for debugging, with indentation."]
    fn dump_indent(indent: uint) {
        let mut s = ~"";
        for uint::range(0u, indent) |_i| {
            s += ~"    ";
        }

        s += #fmt("%?", self.kind);
        #debug["%s", s];

        for BTree.each_child(self) |kid| {
            kid.dump_indent(indent + 1u) 
        }
    }
}

#[doc = "
     Set your width to the maximum available width and return the
     maximum available width any children can use.  Currently children
     are just given the same available width.
"]
fn give_kids_width(+available_width : au, box : @Box) -> au {
    // TODO: give smaller available widths if the width of the
    // containing box is constrained
    match box.kind {
        BlockBox => box.bounds.size.width = available_width,
        InlineBox | IntrinsicBox(*) | TextBoxKind(*) => { }
    }

    available_width
}

#[doc="Wrapper around reflow so it can be passed to traverse"]
fn reflow_wrapper(b : @Box) {
    b.reflow();
}

impl @Box {
    #[doc="
           Run a parallel traversal over the layout tree rooted at
           this box.  On the top-down traversal give each box the
           available width determined by their parent and on the
           bottom-up traversal reflow each box based on their
           attributes and their children's sizes.
    "]
    fn reflow_subtree(available_width : au) {
        extended_full_traversal(self, available_width, give_kids_width, reflow_wrapper);
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

trait PrivateNodeMethods{
    fn dump_indent(ident: uint);
}

impl Node : PrivateNodeMethods {
    #[doc="Dumps the node tree, for debugging, with indentation."]
    fn dump_indent(indent: uint) {
        let mut s = ~"";
        for uint::range(0u, indent) |_i| {
            s += ~"    ";
        }

        s += #fmt("%?", self.read(|n| copy n.kind ));
        #debug["%s", s];

        for NTree.each_child(self) |kid| {
            kid.dump_indent(indent + 1u) 
        }
    }
}

trait NodeMethods {
    fn dump();
}

impl Node : NodeMethods {
    #[doc="Dumps the subtree rooted at this node, for debugging."]
    fn dump() {
        self.dump_indent(0u);
    }
}

#[cfg(test)]
mod test {
    import dom::base::{Element, ElementData, HTMLDivElement, HTMLImageElement, Node, NodeKind};
    import dom::base::{NodeScope};
    import dom::rcu::Scope;

    /*
    use sdl;
    import sdl::video;

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
        for tree::each_child(BTree, root) |c| {
            push_all(r, flat_bounds(c));
        }

        push(r, copy root.bounds);

        return r;
    }

    #[test]
    #[ignore(reason = "busted")]
    fn do_layout() {
        let s = Scope();

        fn mk_img(size: Size2D<au>) -> ~ElementKind {
            ~HTMLImageElement({mut size: size})
        }

        let n0 = s.new_node(Element(ElementData(~"img", mk_img(Size2D(au(10),au(10))))));
        let n1 = s.new_node(Element(ElementData(~"img", mk_img(Size2D(au(10),au(10))))));
        let n2 = s.new_node(Element(ElementData(~"img", mk_img(Size2D(au(10),au(20))))));
        let n3 = s.new_node(Element(ElementData(~"div", ~HTMLDivElement)));

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

        b3.reflow_subtree(au(100));
        let fb = flat_bounds(b3);
        #debug["fb=%?", fb];
        assert fb == ~[geometry::box(au(0), au(0), au(10), au(10)),   // n0
                       geometry::box(au(0), au(10), au(10), au(15)),  // n1
                       geometry::box(au(0), au(25), au(10), au(20)),  // n2
                       geometry::box(au(0), au(0), au(100), au(45))]; // n3
    }
}

