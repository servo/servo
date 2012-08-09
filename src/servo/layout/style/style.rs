#[doc="High-level interface to CSS selector matching."]

import arc::{arc, get, clone};

import dom::style::{DisplayType, DisBlock, DisInline, DisNone, Stylesheet, Unit, Auto};
import dom::base::{Element, HTMLDivElement, HTMLHeadElement, HTMLImageElement, Node, NodeKind};
import dom::base::{Text};
import util::color::{Color, rgb};
import util::color::css_colors::{white, black};
import base::{LayoutData, NTree};

type SpecifiedStyle = {mut background_color : option<Color>,
                        mut display_type : option<DisplayType>,
                        mut font_size : option<Unit>,
                        mut height : option<Unit>,
                        mut text_color : option<Color>,
                        mut width : option<Unit>
                       };

trait DefaultStyleMethods {
    fn default_color() -> Color;
    fn default_display_type() -> DisplayType;
    fn default_width() -> Unit;
    fn default_height() -> Unit;
}

/// Default styles for various attributes in case they don't get initialized from CSS selectors.
impl NodeKind : DefaultStyleMethods {
    fn default_color() -> Color {
        match self {
          Text(*) => { white() }
          Element(*) => {
            let r = rand::rng();
            rgb(r.next() as u8, r.next() as u8, r.next() as u8)
          }
        }
    }

    fn default_display_type() -> DisplayType {
        match self {
          Text(*) => { DisInline }
          Element(element) => {
            match *element.kind {
              HTMLDivElement => DisBlock,
              HTMLHeadElement => DisNone,
              HTMLImageElement(*) => DisInline,
              UnknownElement => DisInline
            }
          }
        }
    }
    
    fn default_width() -> Unit {
        Auto
    }

    fn default_height() -> Unit {
        Auto
    }
}

/**
 * Create a specified style that can be used to initialize a node before selector matching.
 *
 * Everything is initialized to none except the display style. The default value of the display
 * style is computed so that it can be used to short-circuit selector matching to avoid computing
 * style for children of display:none objects.
 */
fn empty_style_for_node_kind(kind: NodeKind) -> SpecifiedStyle {
    let display_type = kind.default_display_type();

    {mut background_color : none,
     mut display_type : some(display_type),
     mut font_size : none,
     mut height : none,
     mut text_color : none,
     mut width : none}
}

trait StylePriv {
    fn initialize_style();
}

impl Node : StylePriv {
    #[doc="
        Set a default auxiliary data so that other threads can modify it.
        
        This is, importantly, the function that creates the layout
        data for the node (the reader-auxiliary box in the RCU model)
        and populates it with the default style.

     "]
    // TODO: we should look into folding this into building the dom,
    // instead of doing a linear sweep afterwards.
    fn initialize_style() {
        let node_kind = self.read(|n| copy *n.kind);
        let the_layout_data = @LayoutData({
            mut specified_style : ~empty_style_for_node_kind(node_kind),
            mut box : none
        });

        self.set_aux(the_layout_data);
    }
}

trait StyleMethods {
    fn initialize_style_for_subtree();
    fn get_specified_style() -> SpecifiedStyle;
    fn recompute_style_for_subtree(styles : arc<Stylesheet>);
}

impl Node : StyleMethods {
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
    fn get_specified_style() -> SpecifiedStyle {
        if !self.has_aux() {
            fail ~"get_computed_style() called on a node without a style!";
        }
        return copy *self.aux(|x| copy x).specified_style;
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
