#[doc="High-level interface to CSS selector matching."]

use std::arc::{ARC, get, clone};

use css::values::*;
use css::values::Stylesheet;
use dom::base::{HTMLDivElement, HTMLHeadElement, HTMLImageElement, UnknownElement, HTMLScriptElement};
use dom::base::{Comment, Doctype, Element, Node, NodeKind, Text};
use util::color::{Color, rgb};
use util::color::css_colors::{white, black};
use layout::base::{LayoutData, NTree};

type SpecifiedStyle = {mut background_color : CSSValue<CSSBackgroundColor>,
                        mut display_type : CSSValue<CSSDisplay>,
                        mut font_size : CSSValue<CSSFontSize>,
                        mut height : CSSValue<BoxSizing>,
                        mut text_color : CSSValue<CSSColor>,
                        mut width : CSSValue<BoxSizing>
                       };

trait DefaultStyleMethods {
    fn default_color() -> Color;
    fn default_display_type() -> CSSDisplay;
    fn default_width() -> BoxSizing;
    fn default_height() -> BoxSizing;
}

/// Default styles for various attributes in case they don't get initialized from CSS selectors.
impl NodeKind : DefaultStyleMethods {
    fn default_color() -> Color {
        match self {
          Text(*) => white(),
          Element(*) => white(),
            _ => fail ~"unstyleable node type encountered"
        }
    }

    fn default_display_type() -> CSSDisplay {
        match self {
          Text(*) => { DisplayInline }
          Element(element) => {
            match *element.kind {
              HTMLDivElement => DisplayBlock,
              HTMLHeadElement => DisplayNone,
              HTMLImageElement(*) => DisplayInline,
              HTMLScriptElement => DisplayNone,
              UnknownElement => DisplayInline,
            }
          },
          Comment(*) | Doctype(*) => DisplayNone
        }
    }
    
    fn default_width() -> BoxSizing {
        BoxAuto
    }

    fn default_height() -> BoxSizing {
        BoxAuto
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

    {mut background_color : Initial,
     mut display_type : Specified(display_type),
     mut font_size : Initial,
     mut height : Initial,
     mut text_color : Initial,
     mut width : Initial}
}

trait StylePriv {
    fn initialize_style() -> ~[@LayoutData];
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
    fn initialize_style() -> ~[@LayoutData] {
        if !self.has_aux() {
            let node_kind = self.read(|n| copy *n.kind);
            let the_layout_data = @LayoutData({
                mut specified_style : ~empty_style_for_node_kind(node_kind),
                mut box : None
            });

            self.set_aux(the_layout_data);

            ~[the_layout_data]
        } else {
            ~[]
        }
    }
}

trait StyleMethods {
    fn initialize_style_for_subtree() -> ~[@LayoutData];
    fn get_specified_style() -> SpecifiedStyle;
    fn recompute_style_for_subtree(styles : ARC<Stylesheet>);
}

impl Node : StyleMethods {
    #[doc="Sequentially initialize the nodes' auxilliary data so they can be updated in parallel."]
    fn initialize_style_for_subtree() -> ~[@LayoutData] {
        let mut handles = self.initialize_style();
        
        for NTree.each_child(self) |kid| {
            handles += kid.initialize_style_for_subtree();
        }

        return handles;
    }
    
    #[doc="
        Returns the computed style for the given node. If CSS selector matching has not yet been
        performed, fails.

        TODO: Return a safe reference; don't copy.
    "]
    fn get_specified_style() -> SpecifiedStyle {
        if !self.has_aux() {
            fail ~"get_specified_style() called on a node without a style!";
        }
        return copy *self.aux(|x| copy x).specified_style;
    }

    #[doc="
        Performs CSS selector matching on a subtree.

        This is, importantly, the function that updates the layout data for the node (the reader-
        auxiliary box in the RCU model) with the computed style.
    "]
    fn recompute_style_for_subtree(styles : ARC<Stylesheet>) {
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
