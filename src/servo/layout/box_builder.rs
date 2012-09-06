#[doc="Creates CSS boxes from a DOM."]

import css::values::{DisplayType, Block, Inline, DisplayNone};
import dom::base::{ElementData, HTMLDivElement, HTMLImageElement, Element, Text, Node};
import gfx::geometry::zero_size_au;
import layout::base::{Appearance, BTree, BlockBox, Box, BoxKind, InlineBox, IntrinsicBox, NTree};
import layout::base::{TextBoxKind};
import layout::text::TextBox;
import util::tree;
import option::is_none;

export box_builder_methods;

enum ctxt = {
    // The parent node that we're scanning.
    parent_node: Node,
    // The parent box that these boxes will be added to.
    parent_box: @Box,

    //
    // The current anonymous box that we're currently appending inline nodes to.
    //
    // See CSS2 9.2.1.1.
    //

    mut anon_box: Option<@Box>
};

fn create_context(parent_node: Node, parent_box: @Box) -> ctxt {
    return ctxt({
           parent_node: parent_node,
           parent_box: parent_box,
           mut anon_box: None
    });
}

impl ctxt {
    #[doc="
     Constructs boxes for the parent's children, when the parent's 'display' attribute is 'block'.
     "]
    fn construct_boxes_for_block_children() {
        for NTree.each_child(self.parent_node) |kid| {

            // Create boxes for the child. Get its primary box.
            let kid_box = kid.construct_boxes();
            if (kid_box.is_none()) {
                again;
            }

            // Determine the child's display.
            let disp = kid.get_specified_style().display_type;
            if disp != Some(Inline) {
                self.finish_anonymous_box_if_necessary();
            }

            // Add the child's box to the current enclosing box or the current anonymous box.
            match kid.get_specified_style().display_type {
              Some(Block) => BTree.add_child(self.parent_box, kid_box.get()),
              Some(Inline) => {
                let anon_box = match self.anon_box {
                  None => {
                    //
                    // The anonymous box inherits the attributes of its parents for now, so
                    // that properties of intrinsic boxes are not spread to their parenting
                    // anonymous box.
                    //
                    // TODO: check what CSS actually specifies
                    //

                    let b = @Box(self.parent_node, InlineBox);
                    self.anon_box = Some(b);
                    b
                  }
                  Some(b) => b
                };
                BTree.add_child(anon_box, kid_box.get());
              }
              Some(DisplayNone) => {
                // Nothing to do.
              }
              _ => { //hack for now
              }
            }
        }
    }

    #[doc="
      Constructs boxes for the parent's children, when the parent's 'display'
      attribute is 'inline'.
     "]
    fn construct_boxes_for_inline_children() {
        for NTree.each_child(self.parent_node) |kid| {

            // Construct boxes for the child. Get its primary box.
            let kid_box = kid.construct_boxes();

            // Determine the child's display.
            let disp = kid.get_specified_style().display_type;
            if disp != Some(Inline) {
                // TODO
            }

            // Add the child's box to the current enclosing box.
            match kid.get_specified_style().display_type {
              Some(Block) => {
                // TODO
                #warn("TODO: non-inline display found inside inline box");
                BTree.add_child(self.parent_box, kid_box.get());
              }
              Some(Inline) => {
                BTree.add_child(self.parent_box, kid_box.get());
              }
              Some(DisplayNone) => {
                // Nothing to do.
              }
              _  => { //hack for now
              }
            }
        }
    }

    #[doc="Constructs boxes for the parent's children."]
    fn construct_boxes_for_children() {
        #debug("parent node:");
        self.parent_node.dump();

        match self.parent_node.get_specified_style().display_type {
          Some(Block) => self.construct_boxes_for_block_children(),
          Some(Inline) => self.construct_boxes_for_inline_children(),
          Some(DisplayNone) => { /* Nothing to do. */ }
          _ => { //hack for now
          }
        }

        self.finish_anonymous_box_if_necessary();
        assert is_none(self.anon_box);
    }

    #[doc="
      Flushes the anonymous box we're creating if it exists. This appends the
      anonymous box to the block.
    "]
    fn finish_anonymous_box_if_necessary() {
        match copy self.anon_box {
          None => { /* Nothing to do. */ }
          Some(b) => BTree.add_child(self.parent_box, b)
        }
        self.anon_box = None;
    }
}

trait PrivBoxBuilder {
    fn determine_box_kind() -> Option<BoxKind>;
}

impl Node : PrivBoxBuilder {
    #[doc="
      Determines the kind of box that this node needs. Also, for images, computes the intrinsic
      size.
     "]
    fn determine_box_kind() -> Option<BoxKind> {
        match self.read(|n| copy n.kind) {
            ~Text(string) => Some(TextBoxKind(@TextBox(copy string))),
            ~Element(element) => {
                match (copy *element.kind, self.get_specified_style().display_type)  {
                    (HTMLImageElement({size}), _) => Some(IntrinsicBox(@size)),
                    (_, Some(Block)) => Some(BlockBox),
                    (_, Some(Inline)) => Some(InlineBox),
                    (_, Some(DisplayNone)) => None,
                    (_, Some(_)) => Some(InlineBox),
                    (_, None) => {
                        fail ~"The specified display style should be a default instead of none"
                    }
                }
            },
            _ => fail ~"unstyleable node type encountered"
        }
    }
}

trait BoxBuilder {
    fn construct_boxes() -> Option<@Box>;
}

impl Node : BoxBuilder {
    #[doc="Creates boxes for this node. This is the entry point."]
    fn construct_boxes() -> Option<@Box> {
        match self.determine_box_kind() {
            None => None,
            Some(kind) => {
                let my_box = @Box(self, kind);
                match kind {
                    BlockBox | InlineBox => {
                        let cx = create_context(self, my_box);
                        cx.construct_boxes_for_children();
                    }
                    _ => {
                        // Nothing to do.
                    }
                }
                Some(my_box)
            }
        }
    }
}

