/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::js::JSRef;
use dom::bindings::utils::{Reflectable, Reflector};
use servo_util::str::DOMString;
use string_cache::atom::Atom;
use std::ascii::AsciiExt;

#[dom_struct]
pub struct CSSStyleDeclaration {
    reflector_: Reflector,
}

fn get_longhands_from_shorthand(shorthand: &Atom) -> Vec<Atom> {
    match shorthand.as_slice() {
        "background" =>
            vec!(Atom::from_slice("background-color"), Atom::from_slice("background-position"),
                 Atom::from_slice("background-attachment"), Atom::from_slice("background-image"),
                 Atom::from_slice("background-repeat")),
        _ => vec!(),
    }
}

type Declaration = int;

fn serialize_list(property: String, list: Vec<Declaration>) -> DOMString {
    let mut result = property;
    result.push_str(": ");
    for declaration in list.iter() {
        result.push_str(serialize_declaration(declaration).as_slice());
    }
    result
}

fn serialize_declaration(_declaration: &Declaration) -> DOMString {
    "".to_string()
}

fn get_declaration(_property: &Atom) -> Option<Declaration> {
    None
}

impl CSSStyleDeclaration {
    pub fn new_inherited() -> CSSStyleDeclaration {
        CSSStyleDeclaration {
            reflector_: Reflector::new()
        }
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
        let longhand_properties = get_longhands_from_shorthand(&property);
        if !longhand_properties.is_empty() {
            //    1. Let list be a new empty array.
            let mut list = vec!();

            //    2. For each longhand property longhand that property maps to, in canonical order,
            //       follow these substeps:
            for longhand in longhand_properties.iter() {
                //       1. If longhand is a case-sensitive match for a property name of a
                //          CSS declaration in the declarations, let declaration be that CSS
                //          declaration, or null otherwise.
                let declaration = get_declaration(longhand);

                //       2. If declaration is null, return the empty string and terminate these
                //          steps.
                if declaration.is_none() {
                    return "".to_string();
                }

                //       3. Append the declaration to list.
                list.push(declaration.unwrap());
            }

            //    3. Return the serialization of list and terminate these steps.
            return serialize_list(property.as_slice().to_string(), list);
        }

        // 3. If property is a case-sensitive match for a property name of a CSS declaration
        //    in the declarations, return the result of invoking serialize a CSS value of that
        //    declaration and terminate these steps.
        // 4. Return the empty string.
        let declaration = get_declaration(&property);
        declaration.as_ref().map(|declaration| serialize_declaration(declaration))
                   .unwrap_or("".to_string())
    }

    fn GetPropertyPriority(self, _property: DOMString) -> DOMString {
        "".to_string()
    }

    fn SetProperty(self, _property: DOMString, _value: DOMString,
                   _priority: DOMString) -> ErrorResult {
        Ok(())
    }

    fn SetPropertyValue(self, _property: DOMString, _value: DOMString) -> ErrorResult {
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
