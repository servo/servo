/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::{self, CSSStyleDeclarationMethods};
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::cssrule::CSSRule;
use dom::element::Element;
use dom::node::{Node, window_from_node};
use dom::window::Window;
use dom_struct::dom_struct;
use parking_lot::RwLock;
use servo_url::ServoUrl;
use std::ascii::AsciiExt;
use std::sync::Arc;
use style::attr::AttrValue;
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
    CSSRule(JS<CSSRule>,
            #[ignore_heap_size_of = "Arc"]
            Arc<RwLock<PropertyDeclarationBlock>>),
}

impl CSSStyleOwner {
    // Mutate the declaration block associated to this style owner, and
    // optionally indicate if it has changed (assumed to be true).
    fn mutate_associated_block<F, R>(&self, f: F) -> R
        where F: FnOnce(&mut PropertyDeclarationBlock, &mut bool) -> R,
    {
        // TODO(emilio): This has some duplication just to avoid dummy clones.
        //
        // This is somewhat complex but the complexity is encapsulated.
        let mut changed = true;
        match *self {
            CSSStyleOwner::Element(ref el) => {
                let mut attr = el.style_attribute().borrow_mut().take();
                let result = if attr.is_some() {
                    let lock = attr.as_ref().unwrap();
                    let mut pdb = lock.write();
                    let result = f(&mut pdb, &mut changed);
                    result
                } else {
                    let mut pdb = PropertyDeclarationBlock {
                        important_count: 0,
                        declarations: vec![],
                    };
                    let result = f(&mut pdb, &mut changed);

                    // Here `changed` is somewhat silly, because we know the
                    // exact conditions under it changes.
                    changed = !pdb.declarations.is_empty();
                    if changed {
                        attr = Some(Arc::new(RwLock::new(pdb)));
                    }

                    result
                };

                if changed {
                    // Note that there's no need to remove the attribute here if
                    // the declaration block is empty[1], and if `attr` is
                    // `None` it means that it necessarily didn't change, so no
                    // need to go through all the set_attribute machinery.
                    //
                    // [1]: https://github.com/whatwg/html/issues/2306
                    if let Some(pdb) = attr {
                        let serialization = pdb.read().to_css_string();
                        el.set_attribute(&local_name!("style"),
                                         AttrValue::Declaration(serialization,
                                                                pdb));
                    }
                } else {
                    // Remember to put it back.
                    *el.style_attribute().borrow_mut() = attr;
                }

                result
            }
            CSSStyleOwner::CSSRule(ref rule, ref pdb) => {
                let result = f(&mut *pdb.write(), &mut changed);
                if changed {
                    rule.global().as_window().Document().invalidate_stylesheets();
                }
                result
            }
        }
    }

    fn with_block<F, R>(&self, f: F) -> R
        where F: FnOnce(&PropertyDeclarationBlock) -> R,
    {
        match *self {
            CSSStyleOwner::Element(ref el) => {
                match *el.style_attribute().borrow() {
                    Some(ref pdb) => f(&pdb.read()),
                    None => {
                        let pdb = PropertyDeclarationBlock {
                            important_count: 0,
                            declarations: vec![],
                        };
                        f(&pdb)
                    }
                }
            }
            CSSStyleOwner::CSSRule(_, ref pdb) => {
                f(&pdb.read())
            }
        }
    }

    fn window(&self) -> Root<Window> {
        match *self {
            CSSStyleOwner::Element(ref el) => window_from_node(&**el),
            CSSStyleOwner::CSSRule(ref rule, _) => Root::from_ref(rule.global().as_window()),
        }
    }

    fn base_url(&self) -> ServoUrl {
        match *self {
            CSSStyleOwner::Element(ref el) => window_from_node(&**el).Document().base_url(),
            CSSStyleOwner::CSSRule(ref rule, _) => {
                rule.parent_stylesheet().style_stylesheet().base_url.clone()
            }
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

        let mut string = String::new();

        self.owner.with_block(|pdb| {
            pdb.property_value_to_css(&id, &mut string).unwrap();
        });

        DOMString::from(string)
    }

    fn set_property(&self, id: PropertyId, value: DOMString, priority: DOMString) -> ErrorResult {
        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        self.owner.mutate_associated_block(|ref mut pdb, mut changed| {
            if value.is_empty() {
                // Step 4
                *changed = pdb.remove_property(&id);
                return Ok(());
            }

            // Step 5
            let importance = match &*priority {
                "" => Importance::Normal,
                p if p.eq_ignore_ascii_case("important") => Importance::Important,
                _ => {
                    *changed = false;
                    return Ok(());
                }
            };

            // Step 6
            let window = self.owner.window();
            let declarations =
                parse_one_declaration(id, &value, &self.owner.base_url(),
                                      window.css_error_reporter(),
                                      ParserContextExtraData::default());

            // Step 7
            let declarations = match declarations {
                Ok(declarations) => declarations,
                Err(_) => {
                    *changed = false;
                    return Ok(());
                }
            };

            // Step 8
            // Step 9
            // We could try to be better I guess?
            *changed = !declarations.is_empty();
            for declaration in declarations {
                // TODO(emilio): We could check it changed
                pdb.set_parsed_declaration(declaration.0, importance);
            }

            Ok(())
        })
    }
}

impl CSSStyleDeclarationMethods for CSSStyleDeclaration {
    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-length
    fn Length(&self) -> u32 {
        self.owner.with_block(|pdb| {
            pdb.declarations.len() as u32
        })
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

        self.owner.with_block(|pdb| {
            if pdb.property_priority(&id).important() {
                DOMString::from("important")
            } else {
                // Step 4
                DOMString::new()
            }
        })
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
        let id = match PropertyId::parse(property.into()) {
            Ok(id) => id,
            Err(..) => return Ok(()), // Unkwown property
        };

        // Step 4
        let importance = match &*priority {
            "" => Importance::Normal,
            p if p.eq_ignore_ascii_case("important") => Importance::Important,
            _ => return Ok(()),
        };

        self.owner.mutate_associated_block(|ref mut pdb, mut changed| {
            // Step 5 & 6
            *changed = pdb.set_importance(&id, importance);
        });

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
        self.owner.mutate_associated_block(|mut pdb, mut changed| {
            pdb.property_value_to_css(&id, &mut string).unwrap();
            *changed = pdb.remove_property(&id);
        });

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
        self.owner.with_block(|pdb| {
            pdb.declarations.get(index as usize).map(|entry| {
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
        self.owner.with_block(|pdb| {
            DOMString::from(pdb.to_css_string())
        })
    }

    // https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-csstext
    fn SetCssText(&self, value: DOMString) -> ErrorResult {
        let window = self.owner.window();

        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        self.owner.mutate_associated_block(|mut pdb, mut _changed| {
            // Step 3
            *pdb = parse_style_attribute(&value,
                                         &self.owner.base_url(),
                                         window.css_error_reporter(),
                                         ParserContextExtraData::default());
        });

        Ok(())
    }

    // https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-_camel_cased_attribute
    css_properties_accessors!(css_properties);
}
