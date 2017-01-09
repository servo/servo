/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::{self, CSSStyleDeclarationMethods};
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
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
use style::parser::ParserContextExtraData;
use style::properties::{Importance, PropertyDeclarationBlock, PropertyId, LonghandId, ShorthandId};
use style::properties::{parse_one_declaration, parse_style_attribute};
use style::selector_parser::PseudoElement;
use style_traits::ToCss;

// http://dev.w3.org/csswg/cssom/#the-cssstyledeclaration-interface
#[dom_struct]
pub struct CSSStyleDeclaration {
    reflector_: Reflector,
    owner: CSSStyleOwner,
    readonly: bool,
    pseudo: Option<PseudoElement>,
}

#[derive(HeapSizeOf, JSTraceable)]
#[must_root]
pub enum CSSStyleOwner {
    Element(JS<Element>),
    CSSRule(JS<Window>,
            #[ignore_heap_size_of = "Arc"]
            Arc<RwLock<PropertyDeclarationBlock>>),
}

impl CSSStyleOwner {
    fn style_attribute(&self) -> Option<Arc<RwLock<PropertyDeclarationBlock>>> {
        match *self {
            CSSStyleOwner::Element(ref el) => {
                if let Some(ref pdb) = *el.style_attribute().borrow() {
                    Some(pdb.clone())
                } else {
                    None
                }
            }
            CSSStyleOwner::CSSRule(_, ref pdb) => {
                Some(pdb.clone())
            }
        }
    }

    fn window(&self) -> Root<Window> {
        match *self {
            CSSStyleOwner::Element(ref el) => window_from_node(&**el),
            CSSStyleOwner::CSSRule(ref window, _) => Root::from_ref(&**window),
        }
    }

    fn flush_style(&self, pdb: &PropertyDeclarationBlock) {
        if let CSSStyleOwner::Element(ref el) = *self {
            el.set_style_attr(pdb.to_css_string());
        }
    }

    fn dirty(&self) {
        match *self {
            CSSStyleOwner::Element(ref el) =>
                el.upcast::<Node>().dirty(NodeDamage::NodeStyleDamaged),
            CSSStyleOwner::CSSRule(ref window, _) =>
                window.Document().invalidate_stylesheets(),
        }
    }
}

#[derive(PartialEq, HeapSizeOf)]
pub enum CSSModificationAccess {
    ReadWrite,
    Readonly,
}

macro_rules! css_properties(
    ( $([$getter:ident, $setter:ident, $id:expr],)* ) => (
        $(
            fn $getter(&self) -> DOMString {
                self.get_property_value($id)
            }
            fn $setter(&self, value: DOMString) -> ErrorResult {
                self.set_property($id, value, DOMString::new())
            }
        )*
    );
);

impl CSSStyleDeclaration {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(owner: CSSStyleOwner,
                         pseudo: Option<PseudoElement>,
                         modification_access: CSSModificationAccess)
                         -> CSSStyleDeclaration {
        CSSStyleDeclaration {
            reflector_: Reflector::new(),
            owner: owner,
            readonly: modification_access == CSSModificationAccess::Readonly,
            pseudo: pseudo,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(global: &Window,
               owner: CSSStyleOwner,
               pseudo: Option<PseudoElement>,
               modification_access: CSSModificationAccess)
               -> Root<CSSStyleDeclaration> {
        reflect_dom_object(box CSSStyleDeclaration::new_inherited(owner,
                                                                  pseudo,
                                                                  modification_access),
                           global,
                           CSSStyleDeclarationBinding::Wrap)
    }

    fn get_computed_style(&self, property: PropertyId) -> DOMString {
        match self.owner {
            CSSStyleOwner::CSSRule(..) =>
                panic!("get_computed_style called on CSSStyleDeclaration with a CSSRule owner"),
            CSSStyleOwner::Element(ref el) => {
                let node = el.upcast::<Node>();
                if !node.is_in_doc() {
                    // TODO: Node should be matched against the style rules of this window.
                    // Firefox is currently the only browser to implement this.
                    return DOMString::new();
                }
                let addr = node.to_trusted_node_address();
                window_from_node(node).resolved_style_query(addr, self.pseudo.clone(), property)
            }
        }
    }

    fn get_property_value(&self, id: PropertyId) -> DOMString {
        if self.readonly {
            // Readonly style declarations are used for getComputedStyle.
            return self.get_computed_style(id);
        }

        if let Some(ref lock) = self.owner.style_attribute() {
            let mut string = String::new();
            lock.read().property_value_to_css(&id, &mut string).unwrap();
            DOMString::from(string)
        } else {
            // No style attribute is like an empty style attribute: no matching declaration.
            DOMString::new()
        }
    }

    fn set_property(&self, id: PropertyId, value: DOMString, priority: DOMString) -> ErrorResult {
        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        if value.is_empty() {
            // Step 4
            let empty = {
                if let Some(ref lock) = self.owner.style_attribute() {
                    let mut style_attribute = lock.write();
                    style_attribute.remove_property(&id);
                    style_attribute.declarations.is_empty()
                } else {
                    // No style attribute is like an empty style attribute: nothing to remove.
                    return Ok(())
                }
            };
            if let (&CSSStyleOwner::Element(ref el), true) = (&self.owner, empty) {
                *el.style_attribute().borrow_mut() = None;
            }
        } else {
            // Step 5
            let importance = match &*priority {
                "" => Importance::Normal,
                p if p.eq_ignore_ascii_case("important") => Importance::Important,
                _ => return Ok(()),
            };

            // Step 6
            let window = self.owner.window();
            let declarations =
                parse_one_declaration(id, &value, &window.get_url(), window.css_error_reporter(),
                                      ParserContextExtraData::default());

            // Step 7
            let declarations = match declarations {
                Ok(declarations) => declarations,
                Err(_) => return Ok(())
            };

            // Step 8
            // Step 9
            match self.owner.style_attribute() {
                Some(ref lock) => {
                    let mut style_attribute = lock.write();
                    for declaration in declarations {
                        style_attribute.set_parsed_declaration(declaration, importance);
                    }
                    self.owner.flush_style(&style_attribute);
                }
                None => {
                    let important_count = if importance.important() {
                        declarations.len() as u32
                    } else {
                        0
                    };
                    let block = PropertyDeclarationBlock {
                        declarations: declarations.into_iter().map(|d| (d, importance)).collect(),
                        important_count: important_count,
                    };
                    if let CSSStyleOwner::Element(ref el) = self.owner {
                        el.set_style_attr(block.to_css_string());
                        *el.style_attribute().borrow_mut() = Some(Arc::new(RwLock::new(block)));
                    } else {
                        panic!("set_property called on a CSSStyleDeclaration with a non-Element owner");
                    }
                }
            }
        }

        self.owner.dirty();
        Ok(())
    }
}

impl CSSStyleDeclarationMethods for CSSStyleDeclaration {
    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-length
    fn Length(&self) -> u32 {
        self.owner.style_attribute().as_ref().map_or(0, |lock| lock.read().declarations.len() as u32)
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-item
    fn Item(&self, index: u32) -> DOMString {
        self.IndexedGetter(index).unwrap_or_default()
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertyvalue
    fn GetPropertyValue(&self, property: DOMString) -> DOMString {
        let id = if let Ok(id) = PropertyId::parse(property.into()) {
            id
        } else {
            // Unkwown property
            return DOMString::new()
        };
        self.get_property_value(id)
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertypriority
    fn GetPropertyPriority(&self, property: DOMString) -> DOMString {
        let id = if let Ok(id) = PropertyId::parse(property.into()) {
            id
        } else {
            // Unkwown property
            return DOMString::new()
        };

        if let Some(ref lock) = self.owner.style_attribute() {
            if lock.read().property_priority(&id).important() {
                DOMString::from("important")
            } else {
                // Step 4
                DOMString::new()
            }
        } else {
            // No style attribute is like an empty style attribute: no matching declaration.
            DOMString::new()
        }
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-setproperty
    fn SetProperty(&self,
                   property: DOMString,
                   value: DOMString,
                   priority: DOMString)
                   -> ErrorResult {
        // Step 3
        let id = if let Ok(id) = PropertyId::parse(property.into()) {
            id
        } else {
            // Unknown property
            return Ok(())
        };
        self.set_property(id, value, priority)
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-setpropertypriority
    fn SetPropertyPriority(&self, property: DOMString, priority: DOMString) -> ErrorResult {
        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        // Step 2 & 3
        let id = if let Ok(id) = PropertyId::parse(property.into()) {
            id
        } else {
            // Unkwown property
            return Ok(())
        };

        // Step 4
        let importance = match &*priority {
            "" => Importance::Normal,
            p if p.eq_ignore_ascii_case("important") => Importance::Important,
            _ => return Ok(()),
        };

        if let Some(ref lock) = self.owner.style_attribute() {
            let mut style_attribute = lock.write();

            // Step 5 & 6
            style_attribute.set_importance(&id, importance);

            self.owner.flush_style(&style_attribute);
            self.owner.dirty();
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

        let id = if let Ok(id) = PropertyId::parse(property.into()) {
            id
        } else {
            // Unkwown property, cannot be there to remove.
            return Ok(DOMString::new())
        };

        let mut string = String::new();
        let empty = {
            if let Some(ref lock) = self.owner.style_attribute() {
                let mut style_attribute = lock.write();
                // Step 3
                style_attribute.property_value_to_css(&id, &mut string).unwrap();

                // Step 4 & 5
                style_attribute.remove_property(&id);
                self.owner.flush_style(&style_attribute);
                style_attribute.declarations.is_empty()
            } else {
                // No style attribute is like an empty style attribute: nothing to remove.
                return Ok(DOMString::new())
            }
        };
        if let (&CSSStyleOwner::Element(ref el), true) = (&self.owner, empty) {
            *el.style_attribute().borrow_mut() = None;
        }
        self.owner.dirty();

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
        self.owner.style_attribute().as_ref().and_then(|lock| {
            lock.read().declarations.get(index as usize).map(|entry| {
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
        self.owner.style_attribute().as_ref().map_or(DOMString::new(), |lock|
            DOMString::from(lock.read().to_css_string()))
    }

    // https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-csstext
    fn SetCssText(&self, value: DOMString) -> ErrorResult {
        let window = self.owner.window();

        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        // Step 3
        let decl_block = parse_style_attribute(&value, &window.get_url(), window.css_error_reporter(),
                                               ParserContextExtraData::default());
        if let CSSStyleOwner::Element(ref el) = self.owner {
            *el.style_attribute().borrow_mut() = if decl_block.declarations.is_empty() {
                el.set_style_attr(String::new());
                None // Step 2
            } else {
                el.set_style_attr(decl_block.to_css_string());
                Some(Arc::new(RwLock::new(decl_block)))
            };
        }
        self.owner.dirty();
        Ok(())
    }

    // https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-_camel_cased_attribute
    css_properties_accessors!(css_properties);
}
