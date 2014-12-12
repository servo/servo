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
            cssstyledeclaration: CSSStyleDeclaration::new_inherited(owner),
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

    css_getter!(Border, "border")
    css_setter!(SetBorder, "border")

    css_getter!(BorderColor, "border-color")
    css_setter!(SetBorderColor, "border-color")

    css_getter!(BorderStyle, "border-style")
    css_setter!(SetBorderStyle, "border-style")

    css_getter!(BorderWidth, "border-width")
    css_setter!(SetBorderWidth, "border-width")

    css_getter!(BorderBottom, "border-bottom")
    css_setter!(SetBorderBottom, "border-bottom")

    css_getter!(BorderBottomColor, "border-bottom-color")
    css_setter!(SetBorderBottomColor, "border-bottom-color")

    css_getter!(BorderBottomStyle, "border-bottom-style")
    css_setter!(SetBorderBottomStyle, "border-bottom-style")

    css_getter!(BorderBottomWidth, "border-bottom-width")
    css_setter!(SetBorderBottomWidth, "border-bottom-width")

    css_getter!(BorderLeft, "border-left")
    css_setter!(SetBorderLeft, "border-left")

    css_getter!(BorderLeftColor, "border-left-color")
    css_setter!(SetBorderLeftColor, "border-left-color")

    css_getter!(BorderLeftStyle, "border-left-style")
    css_setter!(SetBorderLeftStyle, "border-left-style")

    css_getter!(BorderLeftWidth, "border-left-width")
    css_setter!(SetBorderLeftWidth, "border-left-width")

    css_getter!(BorderRight, "border-right")
    css_setter!(SetBorderRight, "border-right")

    css_getter!(BorderRightColor, "border-right-color")
    css_setter!(SetBorderRightColor, "border-right-color")

    css_getter!(BorderRightStyle, "border-right-style")
    css_setter!(SetBorderRightStyle, "border-right-style")

    css_getter!(BorderRightWidth, "border-right-width")
    css_setter!(SetBorderRightWidth, "border-right-width")

    css_getter!(BorderTop, "border-top")
    css_setter!(SetBorderTop, "border-top")

    css_getter!(BorderTopColor, "border-top-color")
    css_setter!(SetBorderTopColor, "border-top-color")

    css_getter!(BorderTopStyle, "border-top-style")
    css_setter!(SetBorderTopStyle, "border-top-style")

    css_getter!(BorderTopWidth, "border-top-width")
    css_setter!(SetBorderTopWidth, "border-top-width")

    css_getter!(Content, "content")
    css_setter!(SetContent, "content")

    css_getter!(Display, "display")
    css_setter!(SetDisplay, "display")

    css_getter!(Width, "width")
    css_setter!(SetWidth, "width")

    css_getter!(MinWidth, "min-width")
    css_setter!(SetMinWidth, "min-width")

    css_getter!(MaxWidth, "max-width")
    css_setter!(SetMaxWidth, "max-width")

    css_getter!(Height, "height")
    css_setter!(SetHeight, "height")

    css_getter!(MinHeight, "min-height")
    css_setter!(SetMinHeight, "min-height")

    css_getter!(MaxHeight, "max-height")
    css_setter!(SetMaxHeight, "max-height")

    css_getter!(Clear, "clear")
    css_setter!(SetClear, "clear")

    css_getter!(Direction, "direction")
    css_setter!(SetDirection, "direction")

    css_getter!(LineHeight, "line-height")
    css_setter!(SetLineHeight, "line-height")

    css_getter!(VerticalAlign, "vertical-align")
    css_setter!(SetVerticalAlign, "vertical-align")

    css_getter!(Visibility, "visibility")
    css_setter!(SetVisibility, "visibility")

    css_getter!(Overflow, "overflow")
    css_setter!(SetOverflow, "overflow")

    css_getter!(TableLayout, "table-layout")
    css_setter!(SetTableLayout, "table-layout")

    css_getter!(WhiteSpace, "white-space")
    css_setter!(SetWhiteSpace, "white-space")

    css_getter!(WritingMode, "writing-mode")
    css_setter!(SetWritingMode, "writing-mode")

    css_getter!(TextAlign, "text-align")
    css_setter!(SetTextAlign, "text-align")

    css_getter!(TextDecoration, "text-decoration")
    css_setter!(SetTextDecoration, "text-decoration")

    css_getter!(TextOrientation, "text-orientation")
    css_setter!(SetTextOrientation, "text-orientation")

    css_getter!(Font, "font")
    css_setter!(SetFont, "font")

    css_getter!(FontFamily, "font-family")
    css_setter!(SetFontFamily, "font-family")

    css_getter!(FontSize, "font-size")
    css_setter!(SetFontSize, "font-size")

    css_getter!(FontStyle, "font-style")
    css_setter!(SetFontStyle, "font-style")

    css_getter!(FontVariant, "font-variant")
    css_setter!(SetFontVariant, "font-variant")

    css_getter!(FontWeight, "font-weight")
    css_setter!(SetFontWeight, "font-weight")

    css_getter!(Margin, "margin")
    css_setter!(SetMargin, "margin")

    css_getter!(MarginBottom, "margin-bottom")
    css_setter!(SetMarginBottom, "margin-bottom")

    css_getter!(MarginLeft, "margin-left")
    css_setter!(SetMarginLeft, "margin-left")

    css_getter!(MarginRight, "margin-right")
    css_setter!(SetMarginRight, "margin-right")

    css_getter!(MarginTop, "margin-top")
    css_setter!(SetMarginTop, "margin-top")

    css_getter!(Padding, "padding")
    css_setter!(SetPadding, "padding")

    css_getter!(PaddingBottom, "padding-bottom")
    css_setter!(SetPaddingBottom, "padding-bottom")

    css_getter!(PaddingLeft, "padding-left")
    css_setter!(SetPaddingLeft, "padding-left")

    css_getter!(PaddingRight, "padding-right")
    css_setter!(SetPaddingRight, "padding-right")

    css_getter!(PaddingTop, "padding-top")
    css_setter!(SetPaddingTop, "padding-top")

    css_getter!(Position, "position")
    css_setter!(SetPosition, "position")

    css_getter!(Top, "top")
    css_setter!(SetTop, "top")

    css_getter!(Bottom, "bottom")
    css_setter!(SetBottom, "bottom")

    css_getter!(Left, "left")
    css_setter!(SetLeft, "left")

    css_getter!(Right, "right")
    css_setter!(SetRight, "right")

    css_getter!(ZIndex, "z-index")
    css_setter!(SetZIndex, "z-index")

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
