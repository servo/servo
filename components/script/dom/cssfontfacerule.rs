/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylesheets::{CssRuleType, FontFaceRule};

use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cssrule::{CSSRule, SpecificCSSRule};
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CSSFontFaceRule {
    cssrule: CSSRule,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    fontfacerule: Arc<Locked<FontFaceRule>>,
}

impl CSSFontFaceRule {
    fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        fontfacerule: Arc<Locked<FontFaceRule>>,
    ) -> CSSFontFaceRule {
        CSSFontFaceRule {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            fontfacerule,
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        fontfacerule: Arc<Locked<FontFaceRule>>,
    ) -> DomRoot<CSSFontFaceRule> {
        reflect_dom_object(
            Box::new(CSSFontFaceRule::new_inherited(
                parent_stylesheet,
                fontfacerule,
            )),
            window,
            CanGc::note(),
        )
    }
}

impl SpecificCSSRule for CSSFontFaceRule {
    fn ty(&self) -> CssRuleType {
        CssRuleType::FontFace
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        self.fontfacerule
            .read_with(&guard)
            .to_css_string(&guard)
            .into()
    }
}
