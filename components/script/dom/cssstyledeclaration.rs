/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast};
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::js::{JS, JSRef, OptionalRootedRootable};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::DocumentHelpers;
use dom::element::{Element, ElementHelpers};
use dom::htmlelement::HTMLElement;
use dom::node::{document_from_node, NodeDamage, Node};
use servo_util::str::DOMString;
use string_cache::Atom;
use style::{is_supported_property, longhands_from_shorthand, parse_style_attribute};
use style::PropertyDeclaration;
use url::Url;

use std::ascii::AsciiExt;

#[dom_struct]
pub struct CSSStyleDeclaration {
    reflector_: Reflector,
    owner: Option<JS<HTMLElement>>,
}

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
    pub fn new_inherited(owner: Option<JSRef<HTMLElement>>) -> CSSStyleDeclaration {
        CSSStyleDeclaration {
            reflector_: Reflector::new(),
            owner: owner.map(|owner| JS::from_rooted(owner)),
        }
    }
}

trait PrivateCSSStyleDeclarationHelpers {
    fn get_declaration(self, property: &Atom) -> Option<PropertyDeclaration>;
}

impl<'a> PrivateCSSStyleDeclarationHelpers for JSRef<'a, CSSStyleDeclaration> {
    fn get_declaration(self, property: &Atom) -> Option<PropertyDeclaration> {
        self.owner.root().and_then(|owner| {
            let element: JSRef<Element> = ElementCast::from_ref(*owner);
            element.get_inline_style_declaration(property).map(|decl| decl.clone())
        })
    }
}

impl<'a> CSSStyleDeclarationMethods for JSRef<'a, CSSStyleDeclaration> {
    fn CssText(self) -> DOMString {
        "".to_string()
    }

    fn SetCssText(self, _cssText: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Length(self) -> u32 {
        0
    }

    fn Item(self, _index: u32) -> DOMString {
        "".to_string()
    }

    //http://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertyvalue
    fn GetPropertyValue(self, property: DOMString) -> DOMString {
        // 1. Let property be property converted to ASCII lowercase.
        let property = Atom::from_slice(property.as_slice().to_ascii_lower().as_slice());

        // 2. If property is a shorthand property, then follow these substeps:
        let longhand_properties = longhands_from_shorthand(property.as_slice());
        if longhand_properties.is_some() {
            //    1. Let list be a new empty array.
            let mut list = vec!();

            //    2. For each longhand property longhand that property maps to, in canonical order,
            //       follow these substeps:
            for longhand in longhand_properties.unwrap().iter() {
                //       1. If longhand is a case-sensitive match for a property name of a
                //          CSS declaration in the declarations, let declaration be that CSS
                //          declaration, or null otherwise.
                let declaration = self.get_declaration(&Atom::from_slice(longhand.as_slice()));

                //       2. If declaration is null, return the empty string and terminate these
                //          steps.
                //XXXjdm ambiguous? this suggests that if we're missing a longhand we return nothing at all.
                if declaration.is_some() {
                    //       3. Append the declaration to list.
                    list.push(declaration.unwrap());
                }
            }

            //    3. Return the serialization of list and terminate these steps.
            return serialize_list(&list);
        }

        // 3. If property is a case-sensitive match for a property name of a CSS declaration
        //    in the declarations, return the result of invoking serialize a CSS value of that
        //    declaration and terminate these steps.
        // 4. Return the empty string.
        let declaration = self.get_declaration(&property);
        declaration.as_ref().map(|declaration| serialize_value(declaration))
                   .unwrap_or("".to_string())
    }

    fn GetPropertyPriority(self, _property: DOMString) -> DOMString {
        "".to_string()
    }

    fn SetProperty(self, _property: DOMString, _value: DOMString,
                   _priority: DOMString) -> ErrorResult {
        Ok(())
    }

    fn SetPropertyValue(self, property: DOMString, value: DOMString) -> ErrorResult {
        // 1. If the readonly flag is set, throw a NoModificationAllowedError exception
        //    and terminate these steps.
        //TODO

        // 2. Let property be property converted to ASCII lowercase.
        let property = Atom::from_slice(property.as_slice().to_ascii_lower().as_slice());

        // 3. If property is not a case-sensitive match for a supported CSS property,
        //    terminate this algorithm.
        if !is_supported_property(property.as_slice()) {
            return Ok(());
        }

        // 4. If value is the empty string, invoke removeProperty() with property as argument
        //    and terminate this algorithm.
        if value.is_empty() {
            //TODO: self.RemoveProperty(property)
            return Ok(());
        }

        // 5. Let component value list be the result of parsing value for property property.
        let mut synthesized_declaration = property.as_slice().to_string();
        synthesized_declaration.push_str(": ");
        synthesized_declaration.push_str(value.as_slice());
        //XXXjdm need page url
        let decl_block = parse_style_attribute(synthesized_declaration.as_slice(),
                                               &Url::parse("http://localhost").unwrap());

        // 6. If component value list is null terminate these steps.
        if decl_block.normal.len() == 0 {
            return Ok(());
        }

        let owner = self.owner.root();
        let element: JSRef<Element> = ElementCast::from_ref(**owner.as_ref().unwrap());

        assert!(decl_block.important.len() == 0);
        for decl in decl_block.normal.iter() {
            // 7. If property is a shorthand property, then for each longhand property
            //    longhand that property maps to, in canonical order, set the CSS
            //    declaration value longhand to the appropriate value(s) from component
            //    value list, and with the list of declarations being the declarations.

            // 8. Otherwise, set the CSS declaration value property to the
            //    value component value list, and with the list of declarations
            //    being the declarations.

            element.update_inline_style(decl.clone());
        }

        let document = document_from_node(element).root();
        let node: JSRef<Node> = NodeCast::from_ref(element);
        document.content_changed(node, NodeDamage::NodeStyleDamaged);
        Ok(())
    }

    fn SetPropertyPriority(self, _property: DOMString, _priority: DOMString) -> ErrorResult {
        Ok(())
    }

    fn RemoveProperty(self, _property: DOMString) -> Fallible<DOMString> {
        Ok("".to_string())
    }

    fn IndexedGetter(self, _index: u32, _found: &mut bool) -> DOMString {
        "".to_string()
    }
}

impl Reflectable for CSSStyleDeclaration {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
