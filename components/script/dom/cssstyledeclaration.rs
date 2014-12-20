/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::{mod, CSSStyleDeclarationMethods};
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast};
use dom::bindings::error::ErrorResult;
use dom::bindings::global::GlobalRef;
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

macro_rules! css_properties(
    ( $([$getter:ident, $setter:ident, $cssprop:expr]),* ) => (
        $(
            fn $getter(self) -> DOMString {
                self.GetPropertyValue($cssprop.to_string())
            }
            fn $setter(self, value: DOMString) {
                self.SetPropertyValue($cssprop.to_string(), value).unwrap();
            }
        )*
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
                           GlobalRef::Window(global),
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

    css_properties!(
        [Color, SetColor, "color"],
        [Background, SetBackground, "background"],
        [BackgroundColor, SetBackgroundColor, "background-color"],
        [BackgroundPosition, SetBackgroundPosition, "background-position"],
        [BackgroundImage, SetBackgroundImage, "background-image"],
        [BackgroundRepeat, SetBackgroundRepeat, "background-repeat"],
        [BackgroundAttachment, SetBackgroundAttachment, "background-attachment"],
        [Border, SetBorder, "border"],
        [BorderColor, SetBorderColor, "border-color"],
        [BorderRadius, SetBorderRadius, "border-radius"],
        [BorderStyle, SetBorderStyle, "border-style"],
        [BorderWidth, SetBorderWidth, "border-width"],
        [BorderBottom, SetBorderBottom, "border-bottom"],
        [BorderBottomColor, SetBorderBottomColor, "border-bottom-color"],
        [BorderBottomStyle, SetBorderBottomStyle, "border-bottom-style"],
        [BorderBottomWidth, SetBorderBottomWidth, "border-bottom-width"],
        [BorderLeft, SetBorderLeft, "border-left"],
        [BorderLeftColor, SetBorderLeftColor, "border-left-color"],
        [BorderLeftStyle, SetBorderLeftStyle, "border-left-style"],
        [BorderLeftWidth, SetBorderLeftWidth, "border-left-width"],
        [BorderRight, SetBorderRight, "border-right"],
        [BorderRightColor, SetBorderRightColor, "border-right-color"],
        [BorderRightStyle, SetBorderRightStyle, "border-right-style"],
        [BorderRightWidth, SetBorderRightWidth, "border-right-width"],
        [BorderTop, SetBorderTop, "border-top"],
        [BorderTopColor, SetBorderTopColor, "border-top-color"],
        [BorderTopStyle, SetBorderTopStyle, "border-top-style"],
        [BorderTopWidth, SetBorderTopWidth, "border-top-width"],
        [Content, SetContent, "content"],
        [Display, SetDisplay, "display"],
        [Opacity, SetOpacity, "opacity"],
        [Width, SetWidth, "width"],
        [MinWidth, SetMinWidth, "min-width"],
        [MaxWidth, SetMaxWidth, "max-width"],
        [Height, SetHeight, "height"],
        [MinHeight, SetMinHeight, "min-height"],
        [MaxHeight, SetMaxHeight, "max-height"],
        [Clear, SetClear, "clear"],
        [Direction, SetDirection, "direction"],
        [LineHeight, SetLineHeight, "line-height"],
        [VerticalAlign, SetVerticalAlign, "vertical-align"],
        [ListStyle, SetListStyle, "list-style"],
        [ListStylePosition, SetListStylePosition, "list-style-position"],
        [ListStyleType, SetListStyleType, "list-style-type"],
        [ListStyleImage, SetListStyleImage, "list-style-image"],
        [Visibility, SetVisibility, "visibility"],
        [Cursor, SetCursor, "cursor"],
        [BoxShadow, SetBoxShadow, "box-shadow"],
        [BoxSizing, SetBoxSizing, "box-sizing"],
        [Overflow, SetOverflow, "overflow"],
        [OverflowWrap, SetOverflowWrap, "overflow-wrap"],
        [TableLayout, SetTableLayout, "table-layout"],
        [EmptyCells, SetEmptyCells, "empty-cells"],
        [CaptionSide, SetCaptionSide, "caption-side"],
        [WhiteSpace, SetWhiteSpace, "white-space"],
        [WritingMode, SetWritingMode, "writing-mode"],
        [LetterSpacing, SetLetterSpacing, "letter-spacing"],
        [WordSpacing, SetWordSpacing, "word-spacing"],
        [WordWrap, SetWordWrap, "word-wrap"],
        [TextAlign, SetTextAlign, "text-align"],
        [TextDecoration, SetTextDecoration, "text-decoration"],
        [TextIndent, SetTextIndent, "text-indent"],
        [TextOrientation, SetTextOrientation, "text-orientation"],
        [TextTransform, SetTextTransform, "text-transform"],
        [Font, SetFont, "font"],
        [FontFamily, SetFontFamily, "font-family"],
        [FontSize, SetFontSize, "font-size"],
        [FontStyle, SetFontStyle, "font-style"],
        [FontVariant, SetFontVariant, "font-variant"],
        [FontWeight, SetFontWeight, "font-weight"],
        [Margin, SetMargin, "margin"],
        [MarginBottom, SetMarginBottom, "margin-bottom"],
        [MarginLeft, SetMarginLeft, "margin-left"],
        [MarginRight, SetMarginRight, "margin-right"],
        [MarginTop, SetMarginTop, "margin-top"],
        [Padding, SetPadding, "padding"],
        [PaddingBottom, SetPaddingBottom, "padding-bottom"],
        [PaddingLeft, SetPaddingLeft, "padding-left"],
        [PaddingRight, SetPaddingRight, "padding-right"],
        [PaddingTop, SetPaddingTop, "padding-top"],
        [Outline, SetOutline, "outline"],
        [Position, SetPosition, "position"],
        [Bottom, SetBottom, "bottom"],
        [Left, SetLeft, "left"],
        [Right, SetRight, "right"],
        [Top, SetTop, "top"],
        [ZIndex, SetZIndex, "z-index"]
    )
}

impl Reflectable for CSSStyleDeclaration {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
