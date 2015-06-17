/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::{self, CSSStyleDeclarationMethods};
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast};
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, JSRef, OptionalRootable, Rootable, Temporary};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::document::DocumentHelpers;
use dom::element::{Element, ElementHelpers, StylePriority};
use dom::htmlelement::HTMLElement;
use dom::node::{window_from_node, document_from_node, NodeDamage, Node};
use dom::window::{Window, WindowHelpers};
use util::str::DOMString;
use string_cache::Atom;
use style::properties::{is_supported_property, longhands_from_shorthand, parse_style_attribute};
use style::properties::PropertyDeclaration;

use std::ascii::AsciiExt;
use std::borrow::ToOwned;

// http://dev.w3.org/csswg/cssom/#the-cssstyledeclaration-interface
#[dom_struct]
pub struct CSSStyleDeclaration {
    reflector_: Reflector,
    owner: JS<HTMLElement>,
    readonly: bool,
}

#[derive(PartialEq)]
pub enum CSSModificationAccess {
    ReadWrite,
    Readonly
}

macro_rules! css_properties(
    ( $([$getter:ident, $setter:ident, $cssprop:expr]),* ) => (
        $(
            fn $getter(self) -> DOMString {
                self.GetPropertyValue($cssprop.to_owned())
            }
            fn $setter(self, value: DOMString) {
                self.SetPropertyValue($cssprop.to_owned(), value).unwrap();
            }
        )*
    );
);

fn serialize_list(list: &Vec<PropertyDeclaration>) -> DOMString {
    list.iter().fold(String::new(), |accum, ref declaration| {
        accum + &declaration.value() + " "
    })
}

impl CSSStyleDeclaration {
    pub fn new_inherited(owner: JSRef<HTMLElement>,
                         modification_access: CSSModificationAccess) -> CSSStyleDeclaration {
        CSSStyleDeclaration {
            reflector_: Reflector::new(),
            owner: JS::from_rooted(owner),
            readonly: modification_access == CSSModificationAccess::Readonly,
        }
    }

    pub fn new(global: JSRef<Window>, owner: JSRef<HTMLElement>,
               modification_access: CSSModificationAccess) -> Temporary<CSSStyleDeclaration> {
        reflect_dom_object(box CSSStyleDeclaration::new_inherited(owner, modification_access),
                           GlobalRef::Window(global),
                           CSSStyleDeclarationBinding::Wrap)
    }
}

trait PrivateCSSStyleDeclarationHelpers {
    fn get_declaration(self, property: &Atom) -> Option<PropertyDeclaration>;
    fn get_important_declaration(self, property: &Atom) -> Option<PropertyDeclaration>;
}

impl<'a> PrivateCSSStyleDeclarationHelpers for JSRef<'a, CSSStyleDeclaration> {
    fn get_declaration(self, property: &Atom) -> Option<PropertyDeclaration> {
        let owner = self.owner.root();
        let element: JSRef<Element> = ElementCast::from_ref(owner.r());
        element.get_inline_style_declaration(property).map(|decl| decl.clone())
    }

    fn get_important_declaration(self, property: &Atom) -> Option<PropertyDeclaration> {
        let owner = self.owner.root();
        let element: JSRef<Element> = ElementCast::from_ref(owner.r());
        element.get_important_inline_style_declaration(property).map(|decl| decl.clone())
    }
}

impl<'a> CSSStyleDeclarationMethods for JSRef<'a, CSSStyleDeclaration> {
    // http://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-length
    fn Length(self) -> u32 {
        let owner = self.owner.root();
        let elem: JSRef<Element> = ElementCast::from_ref(owner.r());
        let len = match *elem.style_attribute().borrow() {
            Some(ref declarations) => declarations.normal.len() + declarations.important.len(),
            None => 0
        };
        len as u32
    }

    // http://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-item
    fn Item(self, index: u32) -> DOMString {
        let index = index as usize;
        let owner = self.owner.root();
        let elem: JSRef<Element> = ElementCast::from_ref(owner.r());
        let style_attribute = elem.style_attribute().borrow();
        let result = style_attribute.as_ref().and_then(|declarations| {
            if index > declarations.normal.len() {
                declarations.important
                            .get(index - declarations.normal.len())
                            .map(|decl| format!("{:?} !important", decl))
            } else {
                declarations.normal
                            .get(index)
                            .map(|decl| format!("{:?}", decl))
            }
        });

        result.unwrap_or("".to_owned())
    }

    // http://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertyvalue
    fn GetPropertyValue(self, property: DOMString) -> DOMString {
        // Step 1
        let property = Atom::from_slice(&property.to_ascii_lowercase());

        // Step 2
        let longhand_properties = longhands_from_shorthand(&property);
        if let Some(longhand_properties) = longhand_properties {
            // Step 2.1
            let mut list = vec!();

            // Step 2.2
            for longhand in longhand_properties.iter() {
                // Step 2.2.1
                let declaration = self.get_declaration(&Atom::from_slice(&longhand));

                // Step 2.2.2 & 2.2.3
                match declaration {
                    Some(declaration) => list.push(declaration),
                    None => return "".to_owned(),
                }
            }

            // Step 2.3
            return serialize_list(&list);
        }

        // Step 3 & 4
        if let Some(ref declaration) = self.get_declaration(&property) {
            declaration.value()
        } else {
            "".to_owned()
        }
    }

    // http://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertypriority
    fn GetPropertyPriority(self, property: DOMString) -> DOMString {
        // Step 1
        let property = Atom::from_slice(&property.to_ascii_lowercase());

        // Step 2
        let longhand_properties = longhands_from_shorthand(&property);
        if let Some(longhand_properties) = longhand_properties {
            // Step 2.1 & 2.2 & 2.3
            if longhand_properties.iter()
                                  .map(|longhand| self.GetPropertyPriority(longhand.clone()))
                                  .all(|priority| priority == "important") {

                return "important".to_owned();
            }
        // Step 3
        } else if self.get_important_declaration(&property).is_some() {
            return "important".to_owned();
        }

        // Step 4
        "".to_owned()
    }

    // http://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-setproperty
    fn SetProperty(self, property: DOMString, value: DOMString,
                   priority: DOMString) -> ErrorResult {
        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        // Step 2
        let property = property.to_ascii_lowercase();

        // Step 3
        if !is_supported_property(&property) {
            return Ok(());
        }

        // Step 4
        if value.is_empty() {
            return self.RemoveProperty(property).map(|_| ());
        }

        // Step 5
        let priority = priority.to_ascii_lowercase();
        if priority != "!important" && !priority.is_empty() {
            return Ok(());
        }

        // Step 6
        let mut synthesized_declaration = property;
        synthesized_declaration.push_str(": ");
        synthesized_declaration.push_str(&value);

        let owner = self.owner.root();
        let window = window_from_node(owner.r()).root();
        let decl_block = parse_style_attribute(&synthesized_declaration, &window.r().get_url());

        // Step 7
        if decl_block.normal.len() == 0 {
            return Ok(());
        }

        let owner = self.owner.root();
        let element: JSRef<Element> = ElementCast::from_ref(owner.r());

        // Step 8
        for decl in decl_block.normal.iter() {
            // Step 9
            let style_priority = if priority.is_empty() {
                StylePriority::Normal
            } else {
                StylePriority::Important
            };
            element.update_inline_style(decl.clone(), style_priority);
        }

        let document = document_from_node(element).root();
        let node: JSRef<Node> = NodeCast::from_ref(element);
        document.r().content_changed(node, NodeDamage::NodeStyleDamaged);
        Ok(())
    }

    // http://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-setpropertypriority
    fn SetPropertyPriority(self, property: DOMString, priority: DOMString) -> ErrorResult {
        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        // Step 2
        let property = property.to_ascii_lowercase();

        // Step 3
        if !is_supported_property(&property) {
            return Ok(());
        }

        // Step 4
        let priority = priority.to_ascii_lowercase();
        if priority != "important" && !priority.is_empty() {
            return Ok(());
        }

        let owner = self.owner.root();
        let window = window_from_node(owner.r()).root();
        let decl_block = parse_style_attribute(&property, &window.r().get_url());
        let element: JSRef<Element> = ElementCast::from_ref(owner.r());

        // Step 5
        for decl in decl_block.normal.iter() {
            // Step 6
            let style_priority = if priority.is_empty() {
                StylePriority::Normal
            } else {
                StylePriority::Important
            };
            element.update_inline_style(decl.clone(), style_priority);
        }

        let document = document_from_node(element).root();
        let node: JSRef<Node> = NodeCast::from_ref(element);
        document.r().content_changed(node, NodeDamage::NodeStyleDamaged);
        Ok(())
    }

    // http://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-setpropertyvalue
    fn SetPropertyValue(self, property: DOMString, value: DOMString) -> ErrorResult {
        self.SetProperty(property, value, "".to_owned())
    }

    // http://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-removeproperty
    fn RemoveProperty(self, property: DOMString) -> Fallible<DOMString> {
        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        // Step 2
        let property = property.to_ascii_lowercase();

        // Step 3
        let value = self.GetPropertyValue(property.clone());

        let longhand_properties = longhands_from_shorthand(&property);
        match longhand_properties {
            Some(longhands) => {
                // Step 4
                for longhand in longhands.iter() {
                    try!(self.RemoveProperty(longhand.clone()));
                }
            }

            None => {
                // Step 5
                let owner = self.owner.root();
                let elem: JSRef<Element> = ElementCast::from_ref(owner.r());
                elem.remove_inline_style_property(property)
            }
        }

        // Step 6
        Ok(value)
    }

    // http://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-cssfloat
    fn CssFloat(self) -> DOMString {
        self.GetPropertyValue("float".to_owned())
    }

    // http://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-cssfloat
    fn SetCssFloat(self, value: DOMString) -> ErrorResult {
        self.SetPropertyValue("float".to_owned(), value)
    }

    // http://dev.w3.org/csswg/cssom/#the-cssstyledeclaration-interface
    fn IndexedGetter(self, index: u32, found: &mut bool) -> DOMString {
        let rval = self.Item(index);
        *found = index < self.Length();
        rval
    }

    css_properties_accessors!(css_properties);
}
