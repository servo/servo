/**
 * High-level interface to CSS selector matching.
 */
use std::arc::{ARC, get, clone};

use core::dvec::DVec;
use css::values::*;
use css::values::Stylesheet;
use dom::element::{HTMLDivElement, HTMLHeadElement, HTMLImageElement, UnknownElement, HTMLScriptElement};
use dom::node::{Comment, Doctype, Element, Text,
                Node, NodeKind, NodeTree, LayoutData};
use util::color::{Color, rgb};
use util::color::css_colors::{white, black};
use layout::context::LayoutContext;

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

    /* TODO: this belongs in the UA stylesheet */
    fn default_display_type() -> CSSDisplay {
        match self {
          Text(*) => DisplayInline,
          Element(element) => {
            match *element.kind {
              HTMLDivElement => DisplayBlock,
              HTMLHeadElement => DisplayNone,
              HTMLImageElement(*) => DisplayInline,
              HTMLScriptElement => DisplayNone,
              _ => DisplayInline,
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

trait StyleMethods {
    fn initialize_layout_data() -> Option<@LayoutData>;

    fn style() -> SpecifiedStyle;
    fn initialize_style_for_subtree(ctx: &LayoutContext, refs: &DVec<@LayoutData>);
    fn recompute_style_for_subtree(ctx: &LayoutContext, styles : ARC<Stylesheet>);
}

impl Node : StyleMethods {
    /** If none exists, creates empty layout data for the node (the reader-auxiliary
     * box in the RCU model) and populates it with an empty style object.
     */
    fn initialize_layout_data() -> Option<@LayoutData> {
        match self.has_aux() {
            false => {
                let node_kind = self.read(|n| copy *n.kind);
                let data = @LayoutData({
                    mut style : ~empty_style_for_node_kind(node_kind),
                    mut flow  : None
                });
                self.set_aux(data); Some(data)
            },
            true => None
        }
    }
        
    /** 
     * Returns the computed style for the given node. If CSS selector
     * matching has not yet been performed, fails.
     */
    fn style() -> SpecifiedStyle {
        if !self.has_aux() {
            fail ~"get_style() called on a node without a style!";
        }
        // TODO: return a safe reference; don't copy!
        return copy *self.aux(|x| copy x).style;
    }

    /**
     * Initializes layout data and styles for a Node tree, if any nodes do not have
     * this data already. Append created layout data to the task's GC roots.
     */
    fn initialize_style_for_subtree(_ctx: &LayoutContext, refs: &DVec<@LayoutData>) {
        do self.traverse_preorder |n| {
            match n.initialize_layout_data() {
                Some(r) => refs.push(r),
                None => {}
            }
        }
    }

    /**
     * Performs CSS selector matching on a subtree.

     * This is, importantly, the function that updates the layout data for
     * the node (the reader-auxiliary box in the RCU model) with the
     * computed style.
     */
    fn recompute_style_for_subtree(ctx: &LayoutContext, styles : ARC<Stylesheet>) {
        let mut i = 0u;
        
        // Compute the styles of each of our children in parallel
        for NodeTree.each_child(self) |kid| {
            i = i + 1u;
            let new_styles = clone(&styles);
            
            kid.recompute_style_for_subtree(ctx, new_styles); 
        }

        self.match_css_style(*get(&styles));
    }
}
