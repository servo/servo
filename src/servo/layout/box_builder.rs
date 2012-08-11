#[doc="Creates CSS boxes from a DOM."]

import dom::base::{ElementData, HTMLDivElement, HTMLImageElement, Element, Text, Node};
import dom::style::{DisplayType, DisBlock, DisInline, DisNone};
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

    mut anon_box: option<@Box>
};

fn create_context(parent_node: Node, parent_box: @Box) -> ctxt {
    return ctxt({
           parent_node: parent_node,
           parent_box: parent_box,
           mut anon_box: none
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

            // Determine the child's display.
            let disp = kid.get_specified_style().display_type;
            if disp != some(DisInline) {
                self.finish_anonymous_box_if_necessary();
            }

            // Add the child's box to the current enclosing box or the current anonymous box.
            match kid.get_specified_style().display_type {
              some(DisBlock) => BTree.add_child(self.parent_box, kid_box),
              some(DisInline) => {
                let anon_box = match self.anon_box {
                  none => {
                    //
                    // The anonymous box inherits the attributes of its parents for now, so
                    // that properties of intrinsic boxes are not spread to their parenting
                    // anonymous box.
                    //
                    // TODO: check what CSS actually specifies
                    //

                    let b = @Box(self.parent_node, InlineBox);
                    self.anon_box = some(b);
                    b
                  }
                  some(b) => b
                };
                BTree.add_child(anon_box, kid_box);
              }
              some(DisNone) => {
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
            if disp != some(DisInline) {
                // TODO
            }

            // Add the child's box to the current enclosing box.
            match kid.get_specified_style().display_type {
              some(DisBlock) => {
                // TODO
                #warn("TODO: non-inline display found inside inline box");
                BTree.add_child(self.parent_box, kid_box);
              }
              some(DisInline) => {
                BTree.add_child(self.parent_box, kid_box);
              }
              some(DisNone) => {
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
          some(DisBlock) => self.construct_boxes_for_block_children(),
          some(DisInline) => self.construct_boxes_for_inline_children(),
          some(DisNone) => { /* Nothing to do. */ }
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
          none => { /* Nothing to do. */ }
          some(b) => BTree.add_child(self.parent_box, b)
        }
        self.anon_box = none;
    }
}

trait PrivBoxBuilder {
    fn determine_box_kind() -> BoxKind;
}

impl Node : PrivBoxBuilder {
    #[doc="
      Determines the kind of box that this node needs. Also, for images, computes the intrinsic
      size.
     "]
    fn determine_box_kind() -> BoxKind {
        match self.read(|n| copy n.kind) {
            ~Text(string) => TextBoxKind(@TextBox(copy string)),
            ~Element(element) => {
                match (copy *element.kind, self.get_specified_style().display_type)  {
                    (HTMLImageElement({size}), _) => IntrinsicBox(@size),
                    (_, some(DisBlock)) => BlockBox,
                    (_, some(DisInline)) => InlineBox,
                    (_, some(DisNone)) => {
                        // TODO: don't have a box here at all?
                        IntrinsicBox(@zero_size_au())
                    }
                    (_, none) => {
                        fail ~"The specified display style should be a default instead of none"
                    }
                }
            }
        }
    }
}

trait BoxBuilder {
    fn construct_boxes() -> @Box;
}

impl Node : BoxBuilder {
    #[doc="Creates boxes for this node. This is the entry point."]
    fn construct_boxes() -> @Box {
        let box_kind = self.determine_box_kind();
        let my_box = @Box(self, box_kind);
        match box_kind {
          BlockBox | InlineBox => {
            let cx = create_context(self, my_box);
            cx.construct_boxes_for_children();
          }
          _ => {
            // Nothing to do.
          }
        }
        return my_box;
    }
}

