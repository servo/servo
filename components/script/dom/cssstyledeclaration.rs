/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::{mod, CSSStyleDeclarationMethods};
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast};
use dom::bindings::error::ErrorResult;
use dom::bindings::global;
use dom::bindings::js::{JS, JSRef, OptionalRootedRootable, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::document::DocumentHelpers;
use dom::element::{Element, ElementHelpers};
use dom::htmlelement::HTMLElement;
use dom::node::{window_from_node, document_from_node, NodeDamage, Node};
use dom::window::Window;
use servo_util::str::DOMString;
use string_cache::Atom;
use style::{is_supported_property, longhands_from_shorthand, parse_style_attribute};
use style::PropertyDeclaration;

use std::ascii::AsciiExt;

#[dom_struct]
pub struct CSSStyleDeclaration {
    reflector_: Reflector,
    owner: JS<HTMLElement>,
}

macro_rules! css_getter(
    ( $idlattr:ident, $cssprop:expr ) => (
        fn $idlattr(self) -> DOMString {
            self.GetPropertyValue($cssprop.to_string())
        }
    );
)

macro_rules! css_setter(
    ( $fnname:ident, $cssprop:expr ) => (
        fn $fnname(self, value: DOMString) {
            self.SetPropertyValue($cssprop.to_string(), value).unwrap();
        }
    );
)

fn serialize_list(list: &Vec<PropertyDeclaration>) -> DOMString {
    let mut result = String::new();
    for declaration in list.iter() {
        result.push_str(serialize_value(declaration).as_slice());
        result.push_str(" ");
    }
    result
}

fn serialize_value(declaration: &PropertyDeclaration) -> DOMString {
    declaration.value()
}

impl CSSStyleDeclaration {
    pub fn new_inherited(owner: JSRef<HTMLElement>) -> CSSStyleDeclaration {
        CSSStyleDeclaration {
            reflector_: Reflector::new(),
            owner: JS::from_rooted(owner),
        }
    }

    pub fn new(global: JSRef<Window>, owner: JSRef<HTMLElement>) -> Temporary<CSSStyleDeclaration> {
        reflect_dom_object(box CSSStyleDeclaration::new_inherited(owner),
                           global::Window(global),
                           CSSStyleDeclarationBinding::Wrap)
    }
}

trait PrivateCSSStyleDeclarationHelpers {
    fn get_declaration(self, property: &Atom) -> Option<PropertyDeclaration>;
}

impl<'a> PrivateCSSStyleDeclarationHelpers for JSRef<'a, CSSStyleDeclaration> {
    fn get_declaration(self, property: &Atom) -> Option<PropertyDeclaration> {
        let owner = self.owner.root();
        let element: JSRef<Element> = ElementCast::from_ref(*owner);
        element.get_inline_style_declaration(property).map(|decl| decl.clone())
    }
}

impl<'a> CSSStyleDeclarationMethods for JSRef<'a, CSSStyleDeclaration> {
    fn Length(self) -> u32 {
        let owner = self.owner.root();
        let elem: JSRef<Element> = ElementCast::from_ref(*owner);
        let len = match *elem.style_attribute().borrow() {
            Some(ref declarations) => declarations.normal.len() + declarations.important.len(),
            None => 0
        };
        len as u32
    }

    fn Item(self, index: u32) -> DOMString {
        let owner = self.owner.root();
        let elem: JSRef<Element> = ElementCast::from_ref(*owner);
        let style_attribute = elem.style_attribute().borrow();
        let result = style_attribute.as_ref().and_then(|declarations| {
            if index as uint > declarations.normal.len() {
                declarations.important
                            .get(index as uint - declarations.normal.len())
                            .map(|decl| format!("{} !important", decl))
            } else {
                declarations.normal
                            .get(index as uint)
                            .map(|decl| format!("{}", decl))
            }
        });

        result.unwrap_or("".to_string())
    }

    // http://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertyvalue
    fn GetPropertyValue(self, property: DOMString) -> DOMString {
        // Step 1
        let property = Atom::from_slice(property.as_slice().to_ascii_lower().as_slice());

        // Step 2
        let longhand_properties = longhands_from_shorthand(property.as_slice());
        if let Some(longhand_properties) = longhand_properties {
            // Step 2.1
            let mut list = vec!();

            // Step 2.2
            for longhand in longhand_properties.iter() {
                // Step 2.2.1
                let declaration = self.get_declaration(&Atom::from_slice(longhand.as_slice()));

                // Step 2.2.2 & 2.2.3
                match declaration {
                    Some(declaration) => list.push(declaration),
                    None => return "".to_string(),
                }
            }

            // Step 2.3
            return serialize_list(&list);
        }

        // Step 3 & 4
        if let Some(ref declaration) = self.get_declaration(&property) {
            serialize_value(declaration)
        } else {
            "".to_string()
        }
    }

    // http://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-setproperty
    fn SetProperty(self, property: DOMString, value: DOMString,
                   priority: DOMString) -> ErrorResult {
        //TODO: disallow modifications if readonly flag is set

        // Step 2
        let property = property.as_slice().to_ascii_lower();

        // Step 3
        if !is_supported_property(property.as_slice()) {
            return Ok(());
        }

        // Step 4
        if value.is_empty() {
            self.RemoveProperty(property);
            return Ok(());
        }

        // Step 5
        let priority = priority.as_slice().to_ascii_lower();
        if priority.as_slice() != "!important" && !priority.is_empty() {
            return Ok(());
        }

        // Step 6
        let mut synthesized_declaration = String::from_str(property.as_slice());
        synthesized_declaration.push_str(": ");
        synthesized_declaration.push_str(value.as_slice());

        let owner = self.owner.root();
        let window = window_from_node(*owner).root();
        let page = window.page();
        let decl_block = parse_style_attribute(synthesized_declaration.as_slice(),
                                               &page.get_url());

        // Step 7
        if decl_block.normal.len() == 0 {
            return Ok(());
        }

        let owner = self.owner.root();
        let element: JSRef<Element> = ElementCast::from_ref(*owner);

        // Step 8
        for decl in decl_block.normal.iter() {
            // Step 9
            element.update_inline_style(decl.clone(), !priority.is_empty());
        }

        let document = document_from_node(element).root();
        let node: JSRef<Node> = NodeCast::from_ref(element);
        document.content_changed(node, NodeDamage::NodeStyleDamaged);
        Ok(())
    }

    // http://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-setpropertyvalue
    fn SetPropertyValue(self, property: DOMString, value: DOMString) -> ErrorResult {
        self.SetProperty(property, value, "".to_string())
    }

    // http://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-removeproperty
    fn RemoveProperty(self, property: DOMString) -> DOMString {
        //TODO: disallow modifications if readonly flag is set

        // Step 2
        let property = property.as_slice().to_ascii_lower();

        // Step 3
        let value = self.GetPropertyValue(property.clone());

        let longhand_properties = longhands_from_shorthand(property.as_slice());
        match longhand_properties {
            Some(longhands) => {
                // Step 4
                for longhand in longhands.iter() {
                    self.RemoveProperty(longhand.clone());
                }
            }

            None => {
                // Step 5
                let owner = self.owner.root();
                let elem: JSRef<Element> = ElementCast::from_ref(*owner);
                elem.remove_inline_style_property(property)
            }
        }

        // Step 6
        value
    }

    // http://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-cssfloat
    fn CssFloat(self) -> DOMString {
        self.GetPropertyValue("float".to_string())
    }

    // http://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-cssfloat
    fn SetCssFloat(self, value: DOMString) -> ErrorResult {
        self.SetPropertyValue("float".to_string(), value)
    }

    fn IndexedGetter(self, index: u32, found: &mut bool) -> DOMString {
        let rval = self.Item(index);
        *found = index < self.Length();
        rval
    }

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
}

impl Reflectable for CSSStyleDeclaration {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
