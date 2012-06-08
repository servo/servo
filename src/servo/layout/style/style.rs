#[doc="High-level interface to CSS selector matching."]

import dom::style::{display_type, di_block, di_inline, di_none, 
                    stylesheet};
import dom::base::{element, es_div, es_head, es_img, nk_element, nk_text};
import dom::base::{node};
import dom::base::node_kind;
import dom::rcu::reader_methods;
import /*layout::*/base::*; // FIXME: resolve bug requires *
import matching::matching_methods;

type computed_style = {mut display : display_type,
                       mut back_color : uint};

#[doc="Returns the default style for the given node kind."]
fn default_style_for_node_kind(kind: node_kind) -> computed_style {
    alt kind {
      nk_text(*) {
        {mut display: di_inline, 
         mut back_color : 256u*256u*256u-1u}
      }
      nk_element(element) {
        let r = rand::rng();
        let rand_color = 256u*256u*((r.next() & (255 as u32)) as uint)
            + 256u*((r.next() & (255 as u32)) as uint)
            + ((r.next() & (255 as u32)) as uint);

        alt *element.subclass {
          es_div { {mut display : di_block,
                    mut back_color : rand_color} }
          es_head { {mut display : di_none, mut back_color : rand_color} }
          es_img(*) { {mut display : di_inline, mut back_color : rand_color} }
          es_unknown { {mut display : di_inline, mut back_color : 
                            rand_color} }
        }
      }
    }
}

impl style_priv for node {
    #[doc="
        Performs CSS selector matching on a node.
        
        This is, importantly, the function that creates the layout data for
        the node (the reader-auxiliary box in the RCU model) and populates it
        with the computed style.
    "]
    fn recompute_style(styles : stylesheet) {
        let style = self.match_css_style(styles);

        #debug("recomputing style; parent node:");

        let the_layout_data = @layout_data({
            mut computed_style: style,
            mut box: none
        });

        #debug("layout data: %?", the_layout_data);

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
        ret copy self.aux({ |x| copy x }).computed_style;
    }

    #[doc="
        Performs CSS selector matching on a subtree.

        This is, importantly, the function that creates the layout data for
        the node (the reader-auxiliary box in the RCU model) and populates it
        with the computed style.

        TODO: compute the style of multiple nodes in parallel.
    "]
    fn recompute_style_for_subtree(styles : stylesheet) {
        self.recompute_style(styles);
        for ntree.each_child(self) {
            |kid|
            kid.recompute_style_for_subtree(styles);
        }
    }
}

