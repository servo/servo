#[doc="High-level interface to CSS selector matching."]

import dom::style::{display_type, di_block, di_inline, di_none, stylesheet};
import dom::base::{HTMLDivElement, HTMLHeadElement, HTMLImageElement, Element, Text, node};
import dom::base::node_kind;
import dom::rcu::reader_methods;
import layout::base::*; // FIXME: resolve bug requires *
import matching::matching_methods;
import util::color::{Color, rgb};
import util::color::css_colors::{white, black};

type computed_style = {mut display : display_type,
                       mut back_color : Color};

#[doc="Returns the default style for the given node kind."]
fn default_style_for_node_kind(kind: node_kind) -> computed_style {
    alt kind {
      Text(*) {
        {mut display: di_inline, 
         mut back_color: white()}
      }
      Element(element) {
        let r = rand::rng();
        let rand_color = rgb(r.next() as u8, r.next() as u8, r.next() as u8);

        alt *element.kind {
          HTMLDivElement      { {mut display: di_block,  mut back_color: rand_color} }
          HTMLHeadElement     { {mut display: di_none,   mut back_color: rand_color} }
          HTMLImageElement(*) { {mut display: di_inline, mut back_color: rand_color} }
          UnknownElement      { {mut display: di_inline, mut back_color: rand_color} }
        }
      }
    }
}

impl style_priv for node {
    #[doc="
        Performs CSS selector matching on a node.
        
        This is, importantly, the function that creates the layout data for the node (the reader-
        auxiliary box in the RCU model) and populates it with the computed style.
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
        Returns the computed style for the given node. If CSS selector matching has not yet been
        performed, fails.

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

        This is, importantly, the function that creates the layout data for the node (the reader-
        auxiliary box in the RCU model) and populates it with the computed style.

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

