/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSFontFaceRuleBinding;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::cssrule::{CSSRule, SpecificCSSRule};
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylesheets::FontFaceRule;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct CSSFontFaceRule<TH: TypeHolderTrait> {
    cssrule: CSSRule<TH>,
    #[ignore_malloc_size_of = "Arc"]
    fontfacerule: Arc<Locked<FontFaceRule>>,
}

impl<TH: TypeHolderTrait> CSSFontFaceRule<TH> {
    fn new_inherited(parent_stylesheet: &CSSStyleSheet<TH>, fontfacerule: Arc<Locked<FontFaceRule>>)
                     -> Self {
        CSSFontFaceRule {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            fontfacerule: fontfacerule,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window<TH>, parent_stylesheet: &CSSStyleSheet<TH>,
               fontfacerule: Arc<Locked<FontFaceRule>>) -> DomRoot<Self> {
        reflect_dom_object(Box::new(CSSFontFaceRule::new_inherited(parent_stylesheet, fontfacerule)),
                           window,
                           CSSFontFaceRuleBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> SpecificCSSRule for CSSFontFaceRule<TH> {
    fn ty(&self) -> u16 {
        use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
        CSSRuleConstants::FONT_FACE_RULE
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        self.fontfacerule.read_with(&guard).to_css_string(&guard).into()
    }
}
