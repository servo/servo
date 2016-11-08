/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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
use servo_atoms::Atom;
use std::ascii::AsciiExt;
use std::sync::Arc;
use style::parser::ParserContextExtraData;
use style::properties::{Shorthand, Importance, PropertyDeclarationBlock};
use style::properties::{is_supported_property, parse_one_declaration, parse_style_attribute};
use style::selector_impl::PseudoElement;
use style_traits::ToCss;

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
            Some(ref lock) => lock.read().declarations.len(),
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
        if self.readonly {
            // Readonly style declarations are used for getComputedStyle.
            property.make_ascii_lowercase();
            let property = Atom::from(property);
            return self.get_computed_style(&property).unwrap_or(DOMString::new());
        }

        let style_attribute = self.owner.style_attribute().borrow();
        let style_attribute = if let Some(ref lock) = *style_attribute {
            lock.read()
        } else {
            // No style attribute is like an empty style attribute: no matching declaration.
            return DOMString::new()
        };

        let mut string = String::new();
        style_attribute.property_value_to_css(&property, &mut string).unwrap();
        DOMString::from(string)
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertypriority
    fn GetPropertyPriority(&self, property: DOMString) -> DOMString {
        let style_attribute = self.owner.style_attribute().borrow();
        let style_attribute = if let Some(ref lock) = *style_attribute {
            lock.read()
        } else {
            // No style attribute is like an empty style attribute: no matching declaration.
            return DOMString::new()
        };

        if style_attribute.property_priority(&property).important() {
            DOMString::from("important")
        } else {
            // Step 4
            DOMString::new()
        }
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-setproperty
    fn SetProperty(&self,
                   property: DOMString,
                   value: DOMString,
                   priority: DOMString)
                   -> ErrorResult {
        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        // Step 3
        if !is_supported_property(&property) {
            return Ok(());
        }

        let mut style_attribute = self.owner.style_attribute().borrow_mut();

        if value.is_empty() {
            // Step 4
            let empty;
            {
                let mut style_attribute = if let Some(ref lock) = *style_attribute {
                    lock.write()
                } else {
                    // No style attribute is like an empty style attribute: nothing to remove.
                    return Ok(())
                };

                style_attribute.remove_property(&property);
                empty = style_attribute.declarations.is_empty()
            }
            if empty {
                *style_attribute = None;
            }
        } else {
            // Step 5
            let importance = match &*priority {
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

            // Step 8
            // Step 9
            match *style_attribute {
                Some(ref lock) => {
                    let mut style_attribute = lock.write();
                    for declaration in declarations {
                        style_attribute.set_parsed_declaration(declaration, importance);
                    }
                    self.owner.set_style_attr(style_attribute.to_css_string());
                }
                ref mut option @ None => {
                    let important_count = if importance.important() {
                        declarations.len() as u32
                    } else {
                        0
                    };
                    let block = PropertyDeclarationBlock {
                        declarations: declarations.into_iter().map(|d| (d, importance)).collect(),
                        important_count: important_count,
                    };
                    self.owner.set_style_attr(block.to_css_string());
                    *option = Some(Arc::new(RwLock::new(block)));
                }
            }
        }

        let node = self.owner.upcast::<Node>();
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
        let importance = match &*priority {
            "" => Importance::Normal,
            p if p.eq_ignore_ascii_case("important") => Importance::Important,
            _ => return Ok(()),
        };

        let style_attribute = self.owner.style_attribute().borrow();
        if let Some(ref lock) = *style_attribute {
            let mut style_attribute = lock.write();

            // Step 5 & 6
            match Shorthand::from_name(&property) {
                Some(shorthand) => style_attribute.set_importance(shorthand.longhands(), importance),
                None => style_attribute.set_importance(&[&*property], importance),
            }

            self.owner.set_style_attr(style_attribute.to_css_string());
            let node = self.owner.upcast::<Node>();
            node.dirty(NodeDamage::NodeStyleDamaged);
        }
        Ok(())
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-setpropertyvalue
    fn SetPropertyValue(&self, property: DOMString, value: DOMString) -> ErrorResult {
        self.SetProperty(property, value, DOMString::new())
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-removeproperty
    fn RemoveProperty(&self, property: DOMString) -> Fallible<DOMString> {
        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        let mut style_attribute = self.owner.style_attribute().borrow_mut();
        let mut string = String::new();
        let empty;
        {
            let mut style_attribute = if let Some(ref lock) = *style_attribute {
                lock.write()
            } else {
                // No style attribute is like an empty style attribute: nothing to remove.
                return Ok(DOMString::new())
            };

            // Step 3
            style_attribute.property_value_to_css(&property, &mut string).unwrap();

            // Step 4 & 5
            style_attribute.remove_property(&property);
            self.owner.set_style_attr(style_attribute.to_css_string());
            empty = style_attribute.declarations.is_empty()
        }
        if empty {
            *style_attribute = None;
        }

        let node = self.owner.upcast::<Node>();
        node.dirty(NodeDamage::NodeStyleDamaged);

        // Step 6
        Ok(DOMString::from(string))
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
        style_attribute.as_ref().and_then(|lock| {
            lock.read().declarations.get(index).map(|entry| {
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

        if let Some(lock) = style_attribute.as_ref() {
            DOMString::from(lock.read().to_css_string())
        } else {
            DOMString::new()
        }
    }

    // https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-csstext
    fn SetCssText(&self, value: DOMString) -> ErrorResult {
        let window = window_from_node(self.owner.upcast::<Node>());

        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        // Step 3
        let decl_block = parse_style_attribute(&value, &window.get_url(), window.css_error_reporter(),
                                               ParserContextExtraData::default());
        *self.owner.style_attribute().borrow_mut() = if decl_block.declarations.is_empty() {
            self.owner.set_style_attr(String::new());
            None // Step 2
        } else {
            self.owner.set_style_attr(decl_block.to_css_string());
            Some(Arc::new(RwLock::new(decl_block)))
        };
        let node = self.owner.upcast::<Node>();
        node.dirty(NodeDamage::NodeStyleDamaged);
        Ok(())
    }

    // https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-_camel_cased_attribute
    css_properties_accessors!(css_properties);
}
