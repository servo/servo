/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//use dom::cssrule::CSSRule;
use dom::bindings::codegen::Bindings::CSSRuleBinding;
use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::window::Window;
use util::str::DOMString;


#[dom_struct]
pub struct CSSRule {
    reflector_: Reflector,
    /*const STYLE_RULE: u16 = 1;
    const unsigned short CHARSET_RULE= 2; // historical
    const unsigned short IMPORT_RULE = 3;
    const unsigned short MEDIA_RULE = 4;
    const unsigned short FONT_FACE_RULE = 5;
    const unsigned short PAGE_RULE = 6;
    const unsigned short MARGIN_RULE = 9;
    const unsigned short NAMESPACE_RULE = 10;*/
    type_: u16,
    //cssText: DOMString,
}

impl CSSRule {
    #[allow(unrooted_must_root)]
    fn new_inherited(type_: u16) -> CSSRule {
        CSSRule {
            reflector_: Reflector::new(),
            type_: type_
            //cssText: cssText
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, type_: u16) -> Root<CSSRule> {
        reflect_dom_object(box CSSRule::new_inherited(type_),
                           GlobalRef::Window(window),
                           CSSRuleBinding::Wrap)
    }
}

impl CSSRuleMethods for CSSRule {
    // https://drafts.csswg.org/cssom/#dom-cssrule-type
    fn Type_(&self) -> u16 {
        
        if self.eq(&self){
           1}
        else if self.eq(&self){
           3}
        else if self.eq(&self){
           4}
        else if self.eq(&self){
           5}
        else if self.eq(&self){
           6}
        else if self.eq(&self){
           8}
        else if self.eq(&self){
           10}
        else{
           7}
    }
}
