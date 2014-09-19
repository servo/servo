/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSS2PropertiesBinding;
use dom::bindings::codegen::Bindings::CSS2PropertiesBinding::CSS2PropertiesMethods;
use dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use dom::bindings::codegen::InheritTypes::CSSStyleDeclarationCast;
use dom::bindings::global;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::cssstyledeclaration::CSSStyleDeclaration;
use dom::htmlelement::HTMLElement;
use dom::window::Window;
use servo_util::str::DOMString;

#[dom_struct]
pub struct CSS2Properties {
    cssstyledeclaration: CSSStyleDeclaration,
}

macro_rules! css_getter(
    ( $idlattr:ident, $cssprop:expr ) => (
        fn $idlattr(self) -> DOMString {
            let decl: JSRef<CSSStyleDeclaration> = CSSStyleDeclarationCast::from_ref(self);
            decl.GetPropertyValue($cssprop.to_string())
        }
    );
)

macro_rules! css_setter(
    ( $fnname:ident, $cssprop:expr ) => (
        fn $fnname(self, value: DOMString) {
            let decl: JSRef<CSSStyleDeclaration> = CSSStyleDeclarationCast::from_ref(self);
            decl.SetPropertyValue($cssprop.to_string(), value).unwrap();
        }
    );
)

impl CSS2Properties {
    fn new_inherited(owner: JSRef<HTMLElement>) -> CSS2Properties {
        CSS2Properties {
            cssstyledeclaration: CSSStyleDeclaration::new_inherited(Some(owner)),
        }
    }

    pub fn new(global: JSRef<Window>, owner: JSRef<HTMLElement>) -> Temporary<CSS2Properties> {
        reflect_dom_object(box CSS2Properties::new_inherited(owner),
                           global::Window(global),
                           CSS2PropertiesBinding::Wrap)
    }
}

impl<'a> CSS2PropertiesMethods for JSRef<'a, CSS2Properties> {
    css_getter!(Color, "color")
    css_setter!(SetColor, "color")

    css_getter!(Background, "background")
    css_setter!(SetBackground, "background")

    css_getter!(BackgroundColor, "background-color")
    css_setter!(SetBackgroundColor, "background-color")

    css_getter!(BackgroundPosition, "background-position")
    css_setter!(SetBackgroundPosition, "background-position")

    css_getter!(BackgroundImage, "background-image")
    css_setter!(SetBackgroundImage, "background-image")

    css_getter!(BackgroundRepeat, "background-repeat")
    css_setter!(SetBackgroundRepeat, "background-repeat")

    css_getter!(BackgroundAttachment, "background-attachment")
    css_setter!(SetBackgroundAttachment, "background-attachment")

    css_getter!(Display, "display")
    css_setter!(SetDisplay, "display")

    css_getter!(Width, "width")
    css_setter!(SetWidth, "width")

    css_getter!(Height, "height")
    css_setter!(SetHeight, "height")

    fn IndexedGetter(self, index: u32, found: &mut bool) -> DOMString {
        let decl: JSRef<CSSStyleDeclaration> = CSSStyleDeclarationCast::from_ref(self);
        decl.IndexedGetter(index, found)
    }
}

impl Reflectable for CSS2Properties {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.cssstyledeclaration.reflector()
    }
}
