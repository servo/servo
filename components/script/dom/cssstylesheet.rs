use dom::bindings::codegen::Bindings::CSSStyleSheetBinding;
use dom::bindings::codegen::Bindings::CSSStyleSheetBinding::CSSStyleSheetMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::cssrule::CSSRule;
use dom::cssrulelist::CSSRuleList;
use dom::window::Window;
use style::servo::Stylesheet;
use std::sync::Arc;
use util::str::DOMString;

#[dom_struct]
pub struct CSSStyleSheet {
	reflector_: Reflector,
	stylesheet: Arc<Stylesheet>,
	//ownerRule: CSSRule,
	//cssRules: CSSRuleList,
}

impl CSSStyleSheet {
    #[allow(unrooted_must_root)]
    fn new_inherited(stylesheet: Arc<Stylesheet>) -> CSSStyleSheet {
        CSSStyleSheet {
        	reflector_: Reflector::new(),
            //stylesheet: Arc::new(stylesheet)
            stylesheet: stylesheet,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, stylesheet: Arc<Stylesheet>) -> Root<CSSStyleSheet> {
        reflect_dom_object(box CSSStyleSheet::new_inherited(stylesheet),
                           GlobalRef::Window(window),
                           CSSStyleSheetBinding::Wrap)
    }
}

impl CSSStyleSheetMethods for CSSStyleSheet {
	
	fn InsertRule(&self, rule: DOMString, index: u32) -> u32 {
		0
	}

	fn DeleteRule(&self, index: u32) {
		//self.stylesheet.borrow_mut().rules.remove(index);
	}

	/*fn CssRules(&self) -> Root<CSSRuleList> {

	}*/
}