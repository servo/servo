/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::ToCss;
use dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::{self, CSSStyleDeclarationMethods};
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::element::Element;
use dom::node::{Node, NodeDamage, window_from_node};
use dom::window::Window;
use parking_lot::RwLock;
use std::ascii::AsciiExt;
use std::sync::Arc;
use string_cache::Atom;
use style::parser::ParserContextExtraData;
use style::properties::{Shorthand, Importance};
use style::properties::{is_supported_property, parse_one_declaration, parse_style_attribute};
use style::selector_impl::PseudoElement;

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
    Readonly,
}

macro_rules! css_properties(
    ( $([$getter:ident, $setter:ident, $cssprop:expr]),* ) => (
        $(
            fn $getter(&self) -> DOMString {
                self.GetPropertyValue(DOMString::from($cssprop))
            }
            fn $setter(&self, value: DOMString) -> ErrorResult {
                self.SetPropertyValue(DOMString::from($cssprop), value)
            }
        )*
    );
);

impl CSSStyleDeclaration {
    pub fn new_inherited(owner: &Element,
                         pseudo: Option<PseudoElement>,
                         modification_access: CSSModificationAccess)
                         -> CSSStyleDeclaration {
        CSSStyleDeclaration {
            reflector_: Reflector::new(),
            owner: JS::from_ref(owner),
            readonly: modification_access == CSSModificationAccess::Readonly,
            pseudo: pseudo,
        }
    }

    pub fn new(global: &Window,
               owner: &Element,
               pseudo: Option<PseudoElement>,
               modification_access: CSSModificationAccess)
               -> Root<CSSStyleDeclaration> {
        reflect_dom_object(box CSSStyleDeclaration::new_inherited(owner,
                                                                  pseudo,
                                                                  modification_access),
                           global,
                           CSSStyleDeclarationBinding::Wrap)
    }

    fn get_computed_style(&self, property: &Atom) -> Option<DOMString> {
        let node = self.owner.upcast::<Node>();
        if !node.is_in_doc() {
            // TODO: Node should be matched against the style rules of this window.
            // Firefox is currently the only browser to implement this.
            return None;
        }
        let addr = node.to_trusted_node_address();
        window_from_node(&*self.owner).resolved_style_query(addr, self.pseudo.clone(), property)
    }
}

impl CSSStyleDeclarationMethods for CSSStyleDeclaration {
    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-length
    fn Length(&self) -> u32 {
        let elem = self.owner.upcast::<Element>();
        let len = match *elem.style_attribute().borrow() {
            Some(ref declarations) => declarations.read().declarations.len(),
            None => 0,
        };
        len as u32
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-item
    fn Item(&self, index: u32) -> DOMString {
        self.IndexedGetter(index).unwrap_or_default()
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertyvalue
    fn GetPropertyValue(&self, mut property: DOMString) -> DOMString {
        let owner = &self.owner;

        // Step 1
        property.make_ascii_lowercase();
        let property = Atom::from(property);

        if self.readonly {
            // Readonly style declarations are used for getComputedStyle.
            return self.get_computed_style(&property).unwrap_or(DOMString::new());
        }

        // Step 2
        if let Some(shorthand) = Shorthand::from_name(&property) {
            let style_attribute = owner.style_attribute().borrow();
            let style_attribute = if let Some(ref style_attribute) = *style_attribute {
                style_attribute.read()
            } else {
                // shorthand.longhands() is never empty, so with no style attribute
                // step 2.2.2 would do this:
                return DOMString::new()
            };

            // Step 2.1
            let mut list = vec![];

            // Step 2.2
            for longhand in shorthand.longhands() {
                // Step 2.2.1
                let declaration = style_attribute.get(longhand);

                // Step 2.2.2 & 2.2.3
                match declaration {
                    Some(&(ref declaration, _importance)) => list.push(declaration),
                    None => return DOMString::new(),
                }
            }

            // Step 2.3
            // TODO: important is hardcoded to false because method does not implement it yet
            let serialized_value = shorthand.serialize_shorthand_value_to_string(
                list, Importance::Normal);
            return DOMString::from(serialized_value);
        }

        // Step 3 & 4
        owner.get_inline_style_declaration(&property, |d| match d {
            Some(declaration) => DOMString::from(declaration.0.value()),
            None => DOMString::new(),
        })
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertypriority
    fn GetPropertyPriority(&self, mut property: DOMString) -> DOMString {
        // Step 1
        property.make_ascii_lowercase();
        let property = Atom::from(property);

        // Step 2
        if let Some(shorthand) = Shorthand::from_name(&property) {
            // Step 2.1 & 2.2 & 2.3
            if shorthand.longhands().iter()
                                    .map(|&longhand| self.GetPropertyPriority(DOMString::from(longhand)))
                                    .all(|priority| priority == "important") {
                return DOMString::from("important");
            }
        } else {
            // Step 3
            return self.owner.get_inline_style_declaration(&property, |d| {
                if let Some(decl) = d {
                    if decl.1.important() {
                        return DOMString::from("important");
                    }
                }

                // Step 4
                DOMString::new()
            })
        }

        // Step 4
        DOMString::new()
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-setproperty
    fn SetProperty(&self,
                   mut property: DOMString,
                   value: DOMString,
                   priority: DOMString)
                   -> ErrorResult {
        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        // Step 2
        property.make_ascii_lowercase();

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
            "" => Importance::Normal,
            p if p.eq_ignore_ascii_case("important") => Importance::Important,
            _ => return Ok(()),
        };

        // Step 6
        let window = window_from_node(&*self.owner);
        let declarations =
            parse_one_declaration(&property, &value, &window.get_url(), window.css_error_reporter(),
                                  ParserContextExtraData::default());

        // Step 7
        let declarations = if let Ok(declarations) = declarations {
            declarations
        } else {
            return Ok(());
        };

        let element = self.owner.upcast::<Element>();

        // Step 8
        // Step 9
        element.update_inline_style(declarations, priority);

        let node = element.upcast::<Node>();
        node.dirty(NodeDamage::NodeStyleDamaged);
        Ok(())
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-setpropertypriority
    fn SetPropertyPriority(&self, property: DOMString, priority: DOMString) -> ErrorResult {
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
            "" => Importance::Normal,
            p if p.eq_ignore_ascii_case("important") => Importance::Important,
            _ => return Ok(()),
        };

        let element = self.owner.upcast::<Element>();

        // Step 5 & 6
        match Shorthand::from_name(&property) {
            Some(shorthand) => {
                element.set_inline_style_property_priority(shorthand.longhands(), priority)
            }
            None => element.set_inline_style_property_priority(&[&*property], priority),
        }

        let node = element.upcast::<Node>();
        node.dirty(NodeDamage::NodeStyleDamaged);
        Ok(())
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-setpropertyvalue
    fn SetPropertyValue(&self, property: DOMString, value: DOMString) -> ErrorResult {
        self.SetProperty(property, value, DOMString::new())
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-removeproperty
    fn RemoveProperty(&self, mut property: DOMString) -> Fallible<DOMString> {
        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        // Step 2
        property.make_ascii_lowercase();

        // Step 3
        let value = self.GetPropertyValue(property.clone());

        let element = self.owner.upcast::<Element>();

        match Shorthand::from_name(&property) {
            // Step 4
            Some(shorthand) => {
                for longhand in shorthand.longhands() {
                    element.remove_inline_style_property(longhand)
                }
            }
            // Step 5
            None => element.remove_inline_style_property(&property),
        }

        let node = element.upcast::<Node>();
        node.dirty(NodeDamage::NodeStyleDamaged);

        // Step 6
        Ok(value)
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-cssfloat
    fn CssFloat(&self) -> DOMString {
        self.GetPropertyValue(DOMString::from("float"))
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-cssfloat
    fn SetCssFloat(&self, value: DOMString) -> ErrorResult {
        self.SetPropertyValue(DOMString::from("float"), value)
    }

    // https://dev.w3.org/csswg/cssom/#the-cssstyledeclaration-interface
    fn IndexedGetter(&self, index: u32) -> Option<DOMString> {
        let index = index as usize;
        let elem = self.owner.upcast::<Element>();
        let style_attribute = elem.style_attribute().borrow();
        style_attribute.as_ref().and_then(|declarations| {
            declarations.read().declarations.get(index).map(|entry| {
                let (ref declaration, importance) = *entry;
                let mut css = declaration.to_css_string();
                if importance.important() {
                    css += " !important";
                }
                DOMString::from(css)
            })
        })
    }

    // https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-csstext
    fn CssText(&self) -> DOMString {
        let elem = self.owner.upcast::<Element>();
        let style_attribute = elem.style_attribute().borrow();

        if let Some(declarations) = style_attribute.as_ref() {
            DOMString::from(declarations.read().to_css_string())
        } else {
            DOMString::new()
        }
    }

    // https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-csstext
    fn SetCssText(&self, value: DOMString) -> ErrorResult {
        let window = window_from_node(self.owner.upcast::<Node>());
        let element = self.owner.upcast::<Element>();

        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        // Step 3
        let decl_block = parse_style_attribute(&value, &window.get_url(), window.css_error_reporter(),
                                               ParserContextExtraData::default());
        *element.style_attribute().borrow_mut() = if decl_block.declarations.is_empty() {
            None // Step 2
        } else {
            Some(Arc::new(RwLock::new(decl_block)))
        };
        element.sync_property_with_attrs_style();
        let node = element.upcast::<Node>();
        node.dirty(NodeDamage::NodeStyleDamaged);
        Ok(())
    }

    // https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-_camel_cased_attribute
    css_properties_accessors!(css_properties);
}
