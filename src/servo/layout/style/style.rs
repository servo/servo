#[doc="High-level interface to CSS selector matching."]

import arc::{arc, get, clone};

import dom::style::{DisplayType, DisBlock, DisInline, DisNone, Stylesheet};
import dom::base::{Element, HTMLDivElement, HTMLHeadElement, HTMLImageElement, Node, NodeKind};
import dom::base::{Text};
import dom::rcu::ReaderMethods;
import layout::base::*; // FIXME: resolve bug requires *
import matching::matching_methods;
import util::color::{Color, rgb};
import util::color::css_colors::{white, black};

type computed_style = {mut display : DisplayType, mut back_color : Color};

#[doc="Returns the default style for the given node kind."]
fn default_style_for_node_kind(kind: NodeKind) -> computed_style {
    alt kind {
      Text(*) {
        {mut display: DisInline, mut back_color: white()}
      }
      Element(element) {
        let r = rand::rng();
        let rand_color = rgb(r.next() as u8, r.next() as u8, r.next() as u8);

        alt *element.kind {
          HTMLDivElement      { {mut display: DisBlock,  mut back_color: rand_color} }
          HTMLHeadElement     { {mut display: DisNone,   mut back_color: rand_color} }
          HTMLImageElement(*) { {mut display: DisInline, mut back_color: rand_color} }
          UnknownElement      { {mut display: DisInline, mut back_color: rand_color} }
        }
      }
    }
}

impl style_priv for Node {
    #[doc="Set a default auxilliary data so that other threads can modify it.
        
        This is, importantly, the function that creates the layout data for the node (the reader-
        auxiliary box in the RCU model) and populates it with the default style.
     "]
    fn initialize_style() {
        let node_kind = self.read(|n| copy *n.kind);
        let the_layout_data = @LayoutData({
            mut computed_style : ~default_style_for_node_kind(node_kind),
            mut box : none
        });

        self.set_aux(the_layout_data);
    }
}

impl style_methods for Node {
    #[doc="Sequentially initialize the nodes' auxilliary data so they can be updated in parallel."]
    fn initialize_style_for_subtree() {
        self.initialize_style();
        
        for NTree.each_child(self) |kid| {
            kid.initialize_style_for_subtree();
        }
    }
    
    #[doc="
        Returns the computed style for the given node. If CSS selector matching has not yet been
        performed, fails.

        TODO: Return a safe reference; don't copy.
    "]
    fn get_computed_style() -> computed_style {
        if !self.has_aux() {
            fail "get_computed_style() called on a node without a style!";
        }
        ret copy *self.aux(|x| copy x).computed_style;
    }

    #[doc="
        Performs CSS selector matching on a subtree.

        This is, importantly, the function that updates the layout data for the node (the reader-
        auxiliary box in the RCU model) with the computed style.
    "]
    fn recompute_style_for_subtree(styles : arc<Stylesheet>) {
        listen(|ack_chan| {
            let mut i = 0u;
            
            // Compute the styles of each of our children in parallel
            for NTree.each_child(self) |kid| {
                i = i + 1u;
                let new_styles = clone(&styles);
                
                task::spawn(|| {
                    kid.recompute_style_for_subtree(new_styles); 
                    ack_chan.send(());
                })
            }

            self.match_css_style(*get(&styles));
            
            // Make sure we have finished updating the tree before returning
            while i > 0 {
                ack_chan.recv();
                i = i - 1u;
            }
        })
    }
}
