/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp::Ordering;
use std::sync::LazyLock;

use dom_struct::dom_struct;
use html5ever::local_name;
use servo_arc::Arc;
use servo_url::ServoUrl;
use style::attr::AttrValue;
use style::properties::{
    parse_one_declaration_into, parse_style_attribute, Importance, LonghandId,
    PropertyDeclarationBlock, PropertyId, ShorthandId, SourcePropertyDeclaration,
};
use style::selector_parser::PseudoElement;
use style::shared_lock::Locked;
use style::stylesheets::{CssRuleType, Origin, UrlExtraData};
use style_traits::ParsingMode;

use crate::dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::cssrule::CSSRule;
use crate::dom::element::Element;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

// http://dev.w3.org/csswg/cssom/#the-cssstyledeclaration-interface
#[dom_struct]
pub(crate) struct CSSStyleDeclaration {
    reflector_: Reflector,
    owner: CSSStyleOwner,
    readonly: bool,
    #[no_trace]
    pseudo: Option<PseudoElement>,
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum CSSStyleOwner {
    Element(Dom<Element>),
    CSSRule(
        Dom<CSSRule>,
        #[ignore_malloc_size_of = "Arc"]
        #[no_trace]
        Arc<Locked<PropertyDeclarationBlock>>,
    ),
}

impl CSSStyleOwner {
    // Mutate the declaration block associated to this style owner, and
    // optionally indicate if it has changed (assumed to be true).
    fn mutate_associated_block<F, R>(&self, f: F, can_gc: CanGc) -> R
    where
        F: FnOnce(&mut PropertyDeclarationBlock, &mut bool) -> R,
    {
        // TODO(emilio): This has some duplication just to avoid dummy clones.
        //
        // This is somewhat complex but the complexity is encapsulated.
        let mut changed = true;
        match *self {
            CSSStyleOwner::Element(ref el) => {
                let document = el.owner_document();
                let shared_lock = document.style_shared_lock();
                let mut attr = el.style_attribute().borrow_mut().take();
                let result = if attr.is_some() {
                    let lock = attr.as_ref().unwrap();
                    let mut guard = shared_lock.write();
                    let pdb = lock.write_with(&mut guard);
                    f(pdb, &mut changed)
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
                            can_gc,
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
                        owner.stylesheet_list_owner().invalidate_stylesheets();
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
                    let document = el.owner_document();
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
            CSSStyleOwner::Element(ref el) => el.owner_window(),
            CSSStyleOwner::CSSRule(ref rule, _) => DomRoot::from_ref(rule.global().as_window()),
        }
    }

    fn base_url(&self) -> ServoUrl {
        match *self {
            CSSStyleOwner::Element(ref el) => el.owner_document().base_url(),
            CSSStyleOwner::CSSRule(ref rule, _) => ServoUrl::from(
                rule.parent_stylesheet()
                    .style_stylesheet()
                    .contents
                    .url_data
                    .read()
                    .0
                    .clone(),
            )
            .clone(),
        }
    }
}

#[derive(MallocSizeOf, PartialEq)]
pub(crate) enum CSSModificationAccess {
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
                self.get_property_value($id, CanGc::note())
            }
            fn $setter(&self, value: DOMString) -> ErrorResult {
                debug_assert!(
                    $id.enabled_for_all_content(),
                    "Someone forgot a #[Pref] annotation"
                );
                self.set_property($id, value, DOMString::new(), CanGc::note())
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
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(
        owner: CSSStyleOwner,
        pseudo: Option<PseudoElement>,
        modification_access: CSSModificationAccess,
    ) -> CSSStyleDeclaration {
        CSSStyleDeclaration {
            reflector_: Reflector::new(),
            owner,
            readonly: modification_access == CSSModificationAccess::Readonly,
            pseudo,
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        global: &Window,
        owner: CSSStyleOwner,
        pseudo: Option<PseudoElement>,
        modification_access: CSSModificationAccess,
        can_gc: CanGc,
    ) -> DomRoot<CSSStyleDeclaration> {
        reflect_dom_object(
            Box::new(CSSStyleDeclaration::new_inherited(
                owner,
                pseudo,
                modification_access,
            )),
            global,
            can_gc,
        )
    }

    fn get_computed_style(&self, property: PropertyId, can_gc: CanGc) -> DOMString {
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
                node.owner_window()
                    .resolved_style_query(addr, self.pseudo, property, can_gc)
            },
        }
    }

    fn get_property_value(&self, id: PropertyId, can_gc: CanGc) -> DOMString {
        if self.readonly {
            // Readonly style declarations are used for getComputedStyle.
            return self.get_computed_style(id, can_gc);
        }

        let mut string = String::new();

        self.owner.with_block(|pdb| {
            pdb.property_value_to_css(&id, &mut string).unwrap();
        });

        DOMString::from(string)
    }

    fn set_property(
        &self,
        id: PropertyId,
        value: DOMString,
        priority: DOMString,
        can_gc: CanGc,
    ) -> ErrorResult {
        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        if !id.enabled_for_all_content() {
            return Ok(());
        }

        self.owner.mutate_associated_block(
            |pdb, changed| {
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
                let mut declarations = SourcePropertyDeclaration::default();
                let result = parse_one_declaration_into(
                    &mut declarations,
                    id,
                    &value,
                    Origin::Author,
                    &UrlExtraData(self.owner.base_url().get_arc()),
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
            },
            can_gc,
        )
    }
}

pub(crate) static ENABLED_LONGHAND_PROPERTIES: LazyLock<Vec<LonghandId>> = LazyLock::new(|| {
    // The 'all' shorthand contains all the enabled longhands with 2 exceptions:
    // 'direction' and 'unicode-bidi', so these must be added afterward.
    let mut enabled_longhands: Vec<LonghandId> = ShorthandId::All.longhands().collect();
    if PropertyId::NonCustom(LonghandId::Direction.into()).enabled_for_all_content() {
        enabled_longhands.push(LonghandId::Direction);
    }
    if PropertyId::NonCustom(LonghandId::UnicodeBidi.into()).enabled_for_all_content() {
        enabled_longhands.push(LonghandId::UnicodeBidi);
    }

    // Sort lexicographically, but with vendor-prefixed properties after standard ones.
    enabled_longhands.sort_unstable_by(|a, b| {
        let a = a.name();
        let b = b.name();
        let is_a_vendor_prefixed = a.starts_with('-');
        let is_b_vendor_prefixed = b.starts_with('-');
        if is_a_vendor_prefixed == is_b_vendor_prefixed {
            a.partial_cmp(b).unwrap()
        } else if is_b_vendor_prefixed {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });
    enabled_longhands
});

impl CSSStyleDeclarationMethods<crate::DomTypeHolder> for CSSStyleDeclaration {
    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-length
    fn Length(&self) -> u32 {
        if self.readonly {
            // Readonly style declarations are used for getComputedStyle.
            // TODO: include custom properties whose computed value is not the guaranteed-invalid value.
            return ENABLED_LONGHAND_PROPERTIES.len() as u32;
        }
        self.owner.with_block(|pdb| pdb.declarations().len() as u32)
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-item
    fn Item(&self, index: u32) -> DOMString {
        self.IndexedGetter(index).unwrap_or_default()
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertyvalue
    fn GetPropertyValue(&self, property: DOMString, can_gc: CanGc) -> DOMString {
        let id = match PropertyId::parse_enabled_for_all_content(&property) {
            Ok(id) => id,
            Err(..) => return DOMString::new(),
        };
        self.get_property_value(id, can_gc)
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertypriority
    fn GetPropertyPriority(&self, property: DOMString) -> DOMString {
        if self.readonly {
            // Readonly style declarations are used for getComputedStyle.
            return DOMString::new();
        }
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
        can_gc: CanGc,
    ) -> ErrorResult {
        // Step 3
        let id = match PropertyId::parse_enabled_for_all_content(&property) {
            Ok(id) => id,
            Err(..) => return Ok(()),
        };
        self.set_property(id, value, priority, can_gc)
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-removeproperty
    fn RemoveProperty(&self, property: DOMString, can_gc: CanGc) -> Fallible<DOMString> {
        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        let id = match PropertyId::parse_enabled_for_all_content(&property) {
            Ok(id) => id,
            Err(..) => return Ok(DOMString::new()),
        };

        let mut string = String::new();
        self.owner.mutate_associated_block(
            |pdb, changed| {
                pdb.property_value_to_css(&id, &mut string).unwrap();
                *changed = remove_property(pdb, &id);
            },
            can_gc,
        );

        // Step 6
        Ok(DOMString::from(string))
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-cssfloat
    fn CssFloat(&self, can_gc: CanGc) -> DOMString {
        self.get_property_value(PropertyId::NonCustom(LonghandId::Float.into()), can_gc)
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-cssfloat
    fn SetCssFloat(&self, value: DOMString, can_gc: CanGc) -> ErrorResult {
        self.set_property(
            PropertyId::NonCustom(LonghandId::Float.into()),
            value,
            DOMString::new(),
            can_gc,
        )
    }

    // https://dev.w3.org/csswg/cssom/#the-cssstyledeclaration-interface
    fn IndexedGetter(&self, index: u32) -> Option<DOMString> {
        if self.readonly {
            // Readonly style declarations are used for getComputedStyle.
            // TODO: include custom properties whose computed value is not the guaranteed-invalid value.
            let longhand = ENABLED_LONGHAND_PROPERTIES.get(index as usize)?;
            return Some(DOMString::from(longhand.name()));
        }
        self.owner.with_block(|pdb| {
            let declaration = pdb.declarations().get(index as usize)?;
            Some(DOMString::from(declaration.id().name()))
        })
    }

    // https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-csstext
    fn CssText(&self) -> DOMString {
        if self.readonly {
            // Readonly style declarations are used for getComputedStyle.
            return DOMString::new();
        }
        self.owner.with_block(|pdb| {
            let mut serialization = String::new();
            pdb.to_css(&mut serialization).unwrap();
            DOMString::from(serialization)
        })
    }

    // https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-csstext
    fn SetCssText(&self, value: DOMString, can_gc: CanGc) -> ErrorResult {
        let window = self.owner.window();

        // Step 1
        if self.readonly {
            return Err(Error::NoModificationAllowed);
        }

        let quirks_mode = window.Document().quirks_mode();
        self.owner.mutate_associated_block(
            |pdb, _changed| {
                // Step 3
                *pdb = parse_style_attribute(
                    &value,
                    &UrlExtraData(self.owner.base_url().get_arc()),
                    window.css_error_reporter(),
                    quirks_mode,
                    CssRuleType::Style,
                );
            },
            can_gc,
        );

        Ok(())
    }

    // https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-_camel_cased_attribute
    style::css_properties_accessors!(css_properties);
}
