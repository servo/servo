use dom::node::Node;
use newcss::complete::CompleteSelectResults;
use std::cell::Cell;

pub trait NodeUtil {
    fn get_css_select_results() -> &self/CompleteSelectResults;
    fn set_css_select_results(decl : CompleteSelectResults);
}

impl NodeUtil for Node {
    /** 
     * Provides the computed style for the given node. If CSS selector
     * Returns the style results for the given node. If CSS selector
     * matching has not yet been performed, fails.
     * FIXME: This isn't completely memory safe since the style is
     * stored in a box that can be overwritten
     */
    fn get_css_select_results() -> &self/CompleteSelectResults {
        if !self.has_aux() {
            fail!(~"style() called on a node without aux data!");
        }
        unsafe { &*self.aux( |x| {
            match x.style {
                Some(ref style) => ptr::to_unsafe_ptr(style),
                None => fail!(~"style() called on node without a style!")
            }
        })}
    }

    /**
    Update the computed style of an HTML element with a style specified by CSS.
    */
    fn set_css_select_results(decl : CompleteSelectResults) {
        let decl = Cell(move decl);
        do self.aux |data| {
            data.style = Some(decl.take())
        }
    }
}
