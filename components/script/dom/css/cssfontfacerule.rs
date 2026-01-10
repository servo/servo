/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylesheets::{CssRuleType, FontFaceRule};

use super::cssrule::{CSSRule, SpecificCSSRule};
use super::cssstylesheet::CSSStyleSheet;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CSSFontFaceRule {
    cssrule: CSSRule,
    #[ignore_malloc_size_of = "Stylo"]
    #[no_trace]
    fontfacerule: RefCell<Arc<Locked<FontFaceRule>>>,
}

impl CSSFontFaceRule {
    fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        fontfacerule: Arc<Locked<FontFaceRule>>,
    ) -> CSSFontFaceRule {
        CSSFontFaceRule {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            fontfacerule: RefCell::new(fontfacerule),
        }
    }

    pub(crate) fn new(
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        fontfacerule: Arc<Locked<FontFaceRule>>,
        can_gc: CanGc,
    ) -> DomRoot<CSSFontFaceRule> {
        reflect_dom_object(
            Box::new(CSSFontFaceRule::new_inherited(
                parent_stylesheet,
                fontfacerule,
            )),
            window,
            can_gc,
        )
    }

    pub(crate) fn update_rule(&self, fontfacerule: Arc<Locked<FontFaceRule>>) {
        *self.fontfacerule.borrow_mut() = fontfacerule;
    }
}

impl SpecificCSSRule for CSSFontFaceRule {
    fn ty(&self) -> CssRuleType {
        CssRuleType::FontFace
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        self.fontfacerule
            .borrow()
            .read_with(&guard)
            .to_css_string(&guard)
            .into()
    }
}
