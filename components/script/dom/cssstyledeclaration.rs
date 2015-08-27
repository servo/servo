/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::{self, CSSStyleDeclarationMethods};
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast};
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::document::DocumentHelpers;
use dom::element::{ElementHelpers, StylePriority, Element};
use dom::node::{window_from_node, document_from_node, NodeDamage, NodeHelpers};
use dom::window::{Window, WindowHelpers};
use selectors::parser::PseudoElement;
use string_cache::Atom;
use style::properties::PropertyDeclaration;
use style::properties::{is_supported_property, longhands_from_shorthand, parse_one_declaration};
use util::str::DOMString;

use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::Ref;

// http://dev.w3.org/csswg/cssom/#the-cssstyledeclaration-interface
#[dom_struct]
pub struct CSSStyleDeclaration {
    reflector_: Reflector,
    owner: JS<Element>,
    readonly: bool,
    pseudo: Option<PseudoElement>,
}

#[derive(PartialEq, HeapSizeOf)]
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
            fn $setter(self, value: DOMString) -> ErrorResult {
                self.SetPropertyValue($cssprop.to_owned(), value)
            }
        )*
    );
);

fn serialize_list(list: &[Ref<PropertyDeclaration>]) -> DOMString {
    list.iter().fold(String::new(), |accum, ref declaration| {
        accum + &declaration.value() + " "
    })
}

impl CSSStyleDeclaration {
    pub fn new_inherited(owner: &Element,
                         pseudo: Option<PseudoElement>,
                         modification_access: CSSModificationAccess) -> CSSStyleDeclaration {
        CSSStyleDeclaration {
            reflector_: Reflector::new(),
            owner: JS::from_ref(owner),
            pseudo: pseudo,
            readonly: modification_access == CSSModificationAccess::Readonly,
        }
    }

    pub fn new(global: &Window, owner: &Element,
               pseudo: Option<PseudoElement>,
               modification_access: CSSModificationAccess) -> Root<CSSStyleDeclaration> {
        reflect_dom_object(box CSSStyleDeclaration::new_inherited(owner, pseudo, modification_access),
                           GlobalRef::Window(global),
                           CSSStyleDeclarationBinding::Wrap)
    }
}

impl CSSStyleDeclaration {
    fn get_computed_style(&self, property: &Atom) -> Option<DOMString> {
        let owner = self.owner.root();
        let node = NodeCast::from_ref(owner.r());
        if !node.is_in_doc() {
            // TODO: Node should be matched against the style rules of this window.
            // Firefox is currently the only browser to implement this.
            return None;
        }
        let addr = node.to_trusted_node_address();
        window_from_node(owner.r()).resolved_style_query(addr, self.pseudo.clone(), property)
    }
}

impl<'a> CSSStyleDeclarationMethods for &'a CSSStyleDeclaration {
    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-length
    fn Length(self) -> u32 {
        let owner = self.owner.root();
        let elem = ElementCast::from_ref(owner.r());
        let len = match *elem.style_attribute().borrow() {
            Some(ref declarations) => declarations.normal.len() + declarations.important.len(),
            None => 0
        };
        len as u32
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-item
    fn Item(self, index: u32) -> DOMString {
        let index = index as usize;
        let owner = self.owner.root();
        let elem = ElementCast::from_ref(owner.r());
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

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertyvalue
    fn GetPropertyValue(self, property: DOMString) -> DOMString {
        let owner = self.owner.root();

        // Step 1
        let property = Atom::from_slice(&property.to_ascii_lowercase());

        if self.readonly {
            // Readonly style declarations are used for getComputedStyle.
            return self.get_computed_style(&property).unwrap_or("".to_owned());
        }

        // Step 2
        let longhand_properties = longhands_from_shorthand(&property);
        if let Some(longhand_properties) = longhand_properties {
            // Step 2.1
            let mut list = vec!();

            // Step 2.2
            for longhand in &*longhand_properties {
                // Step 2.2.1
                let declaration = owner.get_inline_style_declaration(&Atom::from_slice(&longhand));

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
        // FIXME: redundant let binding https://github.com/rust-lang/rust/issues/22252
        let result = match owner.get_inline_style_declaration(&property) {
            Some(declaration) => declaration.value(),
            None => "".to_owned(),
        };
        result
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertypriority
    fn GetPropertyPriority(self, property: DOMString) -> DOMString {
        // Step 1
        let property = Atom::from_slice(&property.to_ascii_lowercase());

        // Step 2
        let longhand_properties = longhands_from_shorthand(&property);
        if let Some(longhand_properties) = longhand_properties {
            // Step 2.1 & 2.2 & 2.3
            if longhand_properties.iter()
                                  .map(|&longhand| self.GetPropertyPriority(longhand.to_owned()))
                                  .all(|priority| priority == "important") {

                return "important".to_owned();
            }
        // Step 3
        } else {
            // FIXME: extra let binding https://github.com/rust-lang/rust/issues/22323
            let owner = self.owner.root();
            if owner.get_important_inline_style_declaration(&property).is_some() {
                return "important".to_owned();
            }
        }

        // Step 4
        "".to_owned()
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-setproperty
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
        let priority = match &*priority {
            "" => StylePriority::Normal,
            p if p.eq_ignore_ascii_case("important") => StylePriority::Important,
            _ => return Ok(()),
        };

        // Step 6
        let owner = self.owner.root();
        let window = window_from_node(owner.r());
        let declarations = parse_one_declaration(&property, &value, &window.r().get_url());

        // Step 7
        let declarations = if let Ok(declarations) = declarations {
            declarations
        } else {
            return Ok(());
        };

        let owner = self.owner.root();
        let element = ElementCast::from_ref(owner.r());

        // Step 8
        for decl in declarations {
            // Step 9
            element.update_inline_style(decl, priority);
        }

        let document = document_from_node(element);
        let node = NodeCast::from_ref(element);
        document.r().content_changed(node, NodeDamage::NodeStyleDamaged);
        Ok(())
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-setpropertypriority
    fn SetPropertyPriority(self, property: DOMString, priority: DOMString) -> ErrorResult {
        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        // Step 2 & 3
        if !is_supported_property(&property) {
            return Ok(());
        }

        // Step 4
        let priority = match &*priority {
            "" => StylePriority::Normal,
            p if p.eq_ignore_ascii_case("important") => StylePriority::Important,
            _ => return Ok(()),
        };

        let owner = self.owner.root();
        let element = ElementCast::from_ref(owner.r());

        // Step 5 & 6
        match longhands_from_shorthand(&property) {
            Some(properties) => element.set_inline_style_property_priority(properties, priority),
            None => element.set_inline_style_property_priority(&[&*property], priority)
        }

        let document = document_from_node(element);
        let node = NodeCast::from_ref(element);
        document.r().content_changed(node, NodeDamage::NodeStyleDamaged);
        Ok(())
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-setpropertyvalue
    fn SetPropertyValue(self, property: DOMString, value: DOMString) -> ErrorResult {
        self.SetProperty(property, value, "".to_owned())
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-removeproperty
    fn RemoveProperty(self, property: DOMString) -> Fallible<DOMString> {
        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        // Step 2
        let property = property.to_ascii_lowercase();

        // Step 3
        let value = self.GetPropertyValue(property.clone());

        let owner = self.owner.root();
        let elem = ElementCast::from_ref(owner.r());

        match longhands_from_shorthand(&property) {
            // Step 4
            Some(longhands) => {
                for longhand in &*longhands {
                    elem.remove_inline_style_property(longhand)
                }
            }
            // Step 5
            None => elem.remove_inline_style_property(&property)
        }

        // Step 6
        Ok(value)
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-cssfloat
    fn CssFloat(self) -> DOMString {
        self.GetPropertyValue("float".to_owned())
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-cssfloat
    fn SetCssFloat(self, value: DOMString) -> ErrorResult {
        self.SetPropertyValue("float".to_owned(), value)
    }

    // https://dev.w3.org/csswg/cssom/#the-cssstyledeclaration-interface
    fn IndexedGetter(self, index: u32, found: &mut bool) -> DOMString {
        let rval = self.Item(index);
        *found = index < self.Length();
        rval
    }

    css_properties_accessors!(css_properties);
}
