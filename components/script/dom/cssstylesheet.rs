use dom::bindings::codegen::Bindings::CSSStyleSheetBinding;
use dom::bindings::codegen::Bindings::CSSStyleSheetBinding::CSSStyleSheetMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::cssrulelist::CSSRuleList;
use dom::window::Window;
use style::servo::Stylesheet;
use std::sync::Arc;

#[dom_struct]
pub struct CSSStyleSheet {
        reflector_: Reflector,
	stylesheet: Arc<Stylesheet>,
	cssrules: Root<CSSRuleList>,
}

impl CSSStyleSheet {
    #[allow(unrooted_must_root)]
    fn new_inherited(stylesheet: Arc<Stylesheet>, cssrules: Root<CSSRuleList>) -> CSSStyleSheet {
        CSSStyleSheet {
            reflector_: Reflector::new(),
            stylesheet: stylesheet,
            cssrules: cssrules, // cssrule: CSSRule::new(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, stylesheet: Arc<Stylesheet>, cssrules: Root<CSSRuleList>) -> Root<CSSStyleSheet> {
        reflect_dom_object(box CSSStyleSheet::new_inherited(stylesheet, cssrules),
                           GlobalRef::Window(window),
                           CSSStyleSheetBinding::Wrap)
    }
}

impl CSSStyleSheetMethods for CSSStyleSheet {
	fn CssRules(&self) -> Root<CSSRuleList>  {
	     /*if self.disabled() == false {
                 panic!(DOMErrorName::SecurityError)
             }
             else{*/
                self.cssrules.clone()
             //}    
	}
}
