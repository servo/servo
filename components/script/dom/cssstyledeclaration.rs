/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::cssrule::CSSRule;
use crate::dom::element::Element;
use crate::dom::node::{document_from_node, stylesheets_owner_from_node, window_from_node, Node};
use crate::dom::window::Window;
use dom_struct::dom_struct;
use servo_arc::Arc;
use servo_url::ServoUrl;
use style::attr::AttrValue;
use style::properties::{
    parse_one_declaration_into, parse_style_attribute, SourcePropertyDeclaration,
};
use style::properties::{
    Importance, LonghandId, PropertyDeclarationBlock, PropertyId, ShorthandId,
};
use style::selector_parser::PseudoElement;
use style::shared_lock::Locked;
use style::stylesheets::{CssRuleType, Origin};
use style_traits::ParsingMode;

// http://dev.w3.org/csswg/cssom/#the-cssstyledeclaration-interface
#[dom_struct]
pub struct CSSStyleDeclaration {
    reflector_: Reflector,
    owner: CSSStyleOwner,
    readonly: bool,
    pseudo: Option<PseudoElement>,
}

#[derive(JSTraceable, MallocSizeOf)]
#[unrooted_must_root_lint::must_root]
pub enum CSSStyleOwner {
    Element(Dom<Element>),
    CSSRule(
        Dom<CSSRule>,
        #[ignore_malloc_size_of = "Arc"] Arc<Locked<PropertyDeclarationBlock>>,
    ),
}

impl CSSStyleOwner {
    // Mutate the declaration block associated to this style owner, and
    // optionally indicate if it has changed (assumed to be true).
    fn mutate_associated_block<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut PropertyDeclarationBlock, &mut bool) -> R,
    {
        // TODO(emilio): This has some duplication just to avoid dummy clones.
        //
        // This is somewhat complex but the complexity is encapsulated.
        let mut changed = true;
        match *self {
            CSSStyleOwner::Element(ref el) => {
                let document = document_from_node(&**el);
                let shared_lock = document.style_shared_lock();
                let mut attr = el.style_attribute().borrow_mut().take();
                let result = if attr.is_some() {
                    let lock = attr.as_ref().unwrap();
                    let mut guard = shared_lock.write();
                    let mut pdb = lock.write_with(&mut guard);
                    let result = f(&mut pdb, &mut changed);
                    result
                } else {
                    let mut pdb = PropertyDeclarationBlock::new();
                    let result = f(&mut pdb, &mut changed);

                    // Here `changed` is somewhat silly, because we know the
                    // exact conditions under it changes.
                    changed = !pdb.declarations().is_empty();
                    if changed {
                        attr = Some(Arc::new(shared_lock.wrap(pdb)));
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
                        let guard = shared_lock.read();
                        let mut serialization = String::new();
                        pdb.read_with(&guard).to_css(&mut serialization).unwrap();
                        el.set_attribute(
                            &local_name!("style"),
                            AttrValue::Declaration(serialization, pdb),
                        );
                    }
                } else {
                    // Remember to put it back.
                    *el.style_attribute().borrow_mut() = attr;
                }

                result
            },
            CSSStyleOwner::CSSRule(ref rule, ref pdb) => {
                let result = {
                    let mut guard = rule.shared_lock().write();
                    f(&mut *pdb.write_with(&mut guard), &mut changed)
                };
                if changed {
                    // If this is changed, see also
                    // CSSStyleRule::SetSelectorText, which does the same thing.
                    if let Some(owner) = rule.parent_stylesheet().get_owner() {
                        stylesheets_owner_from_node(owner.upcast::<Node>())
                            .invalidate_stylesheets();
                    }
                }
                result
            },
        }
    }

    fn with_block<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&PropertyDeclarationBlock) -> R,
    {
        match *self {
            CSSStyleOwner::Element(ref el) => match *el.style_attribute().borrow() {
                Some(ref pdb) => {
                    let document = document_from_node(&**el);
                    let guard = document.style_shared_lock().read();
                    f(pdb.read_with(&guard))
                },
                None => {
                    let pdb = PropertyDeclarationBlock::new();
                    f(&pdb)
                },
            },
            CSSStyleOwner::CSSRule(ref rule, ref pdb) => {
                let guard = rule.shared_lock().read();
                f(pdb.read_with(&guard))
            },
        }
    }

    fn window(&self) -> DomRoot<Window> {
        match *self {
            CSSStyleOwner::Element(ref el) => window_from_node(&**el),
            CSSStyleOwner::CSSRule(ref rule, _) => DomRoot::from_ref(rule.global().as_window()),
        }
    }

    fn base_url(&self) -> ServoUrl {
        match *self {
            CSSStyleOwner::Element(ref el) => window_from_node(&**el).Document().base_url(),
            CSSStyleOwner::CSSRule(ref rule, _) => (*rule
                .parent_stylesheet()
                .style_stylesheet()
                .contents
                .url_data
                .read())
            .clone(),
        }
    }
}

#[derive(MallocSizeOf, PartialEq)]
pub enum CSSModificationAccess {
    ReadWrite,
    Readonly,
}

macro_rules! css_properties(
    ( $([$getter:ident, $setter:ident, $id:expr],)* ) => (
        $(
            fn $getter(&self) -> DOMString {
                debug_assert!(
                    $id.enabled_for_all_content(),
                    "Someone forgot a #[Pref] annotation"
                );
                self.get_property_value($id)
            }
            fn $setter(&self, value: DOMString) -> ErrorResult {
                debug_assert!(
                    $id.enabled_for_all_content(),
                    "Someone forgot a #[Pref] annotation"
                );
                self.set_property($id, value, DOMString::new())
            }
        )*
    );
);

fn remove_property(decls: &mut PropertyDeclarationBlock, id: &PropertyId) -> bool {
    let first_declaration = decls.first_declaration_to_remove(id);
    let first_declaration = match first_declaration {
        Some(i) => i,
        None => return false,
    };
    decls.remove_property(id, first_declaration);
    true
}

impl CSSStyleDeclaration {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(
        owner: CSSStyleOwner,
        pseudo: Option<PseudoElement>,
        modification_access: CSSModificationAccess,
    ) -> CSSStyleDeclaration {
        CSSStyleDeclaration {
            reflector_: Reflector::new(),
            owner: owner,
            readonly: modification_access == CSSModificationAccess::Readonly,
            pseudo: pseudo,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        global: &Window,
        owner: CSSStyleOwner,
        pseudo: Option<PseudoElement>,
        modification_access: CSSModificationAccess,
    ) -> DomRoot<CSSStyleDeclaration> {
        reflect_dom_object(
            Box::new(CSSStyleDeclaration::new_inherited(
                owner,
                pseudo,
                modification_access,
            )),
            global,
        )
    }

    fn get_computed_style(&self, property: PropertyId) -> DOMString {
        match self.owner {
            CSSStyleOwner::CSSRule(..) => {
                panic!("get_computed_style called on CSSStyleDeclaration with a CSSRule owner")
            },
            CSSStyleOwner::Element(ref el) => {
                let node = el.upcast::<Node>();
                if !node.is_connected() {
                    return DOMString::new();
                }
                let addr = node.to_trusted_node_address();
                window_from_node(node).resolved_style_query(addr, self.pseudo.clone(), property)
            },
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

        if !id.enabled_for_all_content() {
            return Ok(());
        }

        self.owner.mutate_associated_block(|pdb, changed| {
            if value.is_empty() {
                // Step 3
                *changed = remove_property(pdb, &id);
                return Ok(());
            }

            // Step 4
            let importance = match &*priority {
                "" => Importance::Normal,
                p if p.eq_ignore_ascii_case("important") => Importance::Important,
                _ => {
                    *changed = false;
                    return Ok(());
                },
            };

            // Step 5
            let window = self.owner.window();
            let quirks_mode = window.Document().quirks_mode();
            let mut declarations = SourcePropertyDeclaration::new();
            let result = parse_one_declaration_into(
                &mut declarations,
                id,
                &value,
                Origin::Author,
                &self.owner.base_url(),
                window.css_error_reporter(),
                ParsingMode::DEFAULT,
                quirks_mode,
                CssRuleType::Style,
            );

            // Step 6
            match result {
                Ok(()) => {},
                Err(_) => {
                    *changed = false;
                    return Ok(());
                },
            }

            let mut updates = Default::default();
            *changed = pdb.prepare_for_update(&declarations, importance, &mut updates);

            if !*changed {
                return Ok(());
            }

            // Step 7
            // Step 8
            pdb.update(declarations.drain(), importance, &mut updates);

            Ok(())
        })
    }
}

impl CSSStyleDeclarationMethods for CSSStyleDeclaration {
    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-length
    fn Length(&self) -> u32 {
        self.owner.with_block(|pdb| pdb.declarations().len() as u32)
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-item
    fn Item(&self, index: u32) -> DOMString {
        self.IndexedGetter(index).unwrap_or_default()
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertyvalue
    fn GetPropertyValue(&self, property: DOMString) -> DOMString {
        let id = match PropertyId::parse_enabled_for_all_content(&property) {
            Ok(id) => id,
            Err(..) => return DOMString::new(),
        };
        self.get_property_value(id)
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertypriority
    fn GetPropertyPriority(&self, property: DOMString) -> DOMString {
        let id = match PropertyId::parse_enabled_for_all_content(&property) {
            Ok(id) => id,
            Err(..) => return DOMString::new(),
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
    fn SetProperty(
        &self,
        property: DOMString,
        value: DOMString,
        priority: DOMString,
    ) -> ErrorResult {
        // Step 3
        let id = match PropertyId::parse_enabled_for_all_content(&property) {
            Ok(id) => id,
            Err(..) => return Ok(()),
        };
        self.set_property(id, value, priority)
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-removeproperty
    fn RemoveProperty(&self, property: DOMString) -> Fallible<DOMString> {
        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        let id = match PropertyId::parse_enabled_for_all_content(&property) {
            Ok(id) => id,
            Err(..) => return Ok(DOMString::new()),
        };

        let mut string = String::new();
        self.owner.mutate_associated_block(|pdb, changed| {
            pdb.property_value_to_css(&id, &mut string).unwrap();
            *changed = remove_property(pdb, &id);
        });

        // Step 6
        Ok(DOMString::from(string))
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-cssfloat
    fn CssFloat(&self) -> DOMString {
        self.get_property_value(PropertyId::Longhand(LonghandId::Float))
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-cssfloat
    fn SetCssFloat(&self, value: DOMString) -> ErrorResult {
        self.set_property(
            PropertyId::Longhand(LonghandId::Float),
            value,
            DOMString::new(),
        )
    }

    // https://dev.w3.org/csswg/cssom/#the-cssstyledeclaration-interface
    fn IndexedGetter(&self, index: u32) -> Option<DOMString> {
        self.owner.with_block(|pdb| {
            let declaration = pdb.declarations().get(index as usize)?;
            let important = pdb.declarations_importance().get(index as usize)?;
            let mut css = String::new();
            declaration.to_css(&mut css).unwrap();
            if important {
                css += " !important";
            }
            Some(DOMString::from(css))
        })
    }

    // https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-csstext
    fn CssText(&self) -> DOMString {
        self.owner.with_block(|pdb| {
            let mut serialization = String::new();
            pdb.to_css(&mut serialization).unwrap();
            DOMString::from(serialization)
        })
    }

    // https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-csstext
    fn SetCssText(&self, value: DOMString) -> ErrorResult {
        let window = self.owner.window();

        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        let quirks_mode = window.Document().quirks_mode();
        self.owner.mutate_associated_block(|pdb, _changed| {
            // Step 3
            *pdb = parse_style_attribute(
                &value,
                &self.owner.base_url(),
                window.css_error_reporter(),
                quirks_mode,
                CssRuleType::Style,
            );
        });

        Ok(())
    }

    // https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-_camel_cased_attribute
    css_properties_accessors!(css_properties);
}
