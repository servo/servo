#[doc="High-level interface to CSS selector matching."]

import dom::base::{nk_div, nk_img, node, node_kind};
import dom::rcu::reader_methods;
import /*layout::*/base::*; // FIXME: resolve bug requires *

enum computed_style = {
    mut display: display
};

enum display {
    di_block,
    di_inline
}

#[doc="Returns the default style for the given node kind."]
fn default_style_for_node_kind(kind : node_kind) -> computed_style {
    alt kind {
        nk_div      { computed_style({ mut display: di_block })  }
        nk_img(*)   { computed_style({ mut display: di_inline }) }
    }
}

impl style_priv for node {
    #[doc="
        Performs CSS selector matching on a node.
        
        This is, importantly, the function that creates the layout data for
        the node (the reader-auxiliary box in the RCU model) and populates it
        with the computed style.
    "]
    fn recompute_style() {
        let default_style: computed_style =
            default_style_for_node_kind(self.rd { |n| n.kind });
        let the_layout_data = @layout_data({
            mut computed_style: default_style,
            mut box: none
        });
        self.set_aux(the_layout_data);
    }
}

impl style_methods for node {
    #[doc="
        Returns the computed style for the given node. If CSS selector matching
        has not yet been performed, fails.

        TODO: Return a safe reference; don't copy.
    "]
    fn get_computed_style() -> computed_style {
        if !self.has_aux() {
            fail "get_computed_style() called on a node without a style!";
        }
        ret self.aux({ |x| x }).computed_style;
    }

    #[doc="
        Performs CSS selector matching on a subtree.

        This is, importantly, the function that creates the layout data for
        the node (the reader-auxiliary box in the RCU model) and populates it
        with the computed style.
    "]
    fn recompute_style_for_subtree() {
        self.recompute_style();
        for ntree.each_child(self) {
            |kid|
            kid.recompute_style_for_subtree();
        }
    }
}

