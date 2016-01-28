/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::{self, CSSStyleDeclarationMethods};
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::element::{Element, StylePriority};
use dom::node::{Node, NodeDamage, document_from_node, window_from_node};
use dom::window::Window;
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::Ref;
use string_cache::Atom;
use style::error_reporting::ParseErrorReporter;
use style::properties::{PropertyDeclaration, Shorthand};
use style::properties::{is_supported_property, parse_one_declaration};
use style::selector_impl::PseudoElement;
use util::str::{DOMString, str_join};

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

fn serialize_shorthand(shorthand: Shorthand, declarations: &[Ref<PropertyDeclaration>]) -> String {
    // https://drafts.csswg.org/css-variables/#variables-in-shorthands
    if let Some(css) = declarations[0].with_variables_from_shorthand(shorthand) {
        if declarations[1..]
               .iter()
               .all(|d| d.with_variables_from_shorthand(shorthand) == Some(css)) {
            css.to_owned()
        } else {
            String::new()
        }
    } else {
        if declarations.iter().any(|d| d.with_variables()) {
            String::new()
        } else {
            let str_iter = declarations.iter().map(|d| d.value());
            // FIXME: this needs property-specific code, which probably should be in style/
            // "as appropriate according to the grammar of shorthand "
            // https://drafts.csswg.org/cssom/#serialize-a-css-value
            str_join(str_iter, " ")
        }
    }
}

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
                           GlobalRef::Window(global),
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
            Some(ref declarations) => declarations.normal.len() + declarations.important.len(),
            None => 0,
        };
        len as u32
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-item
    fn Item(&self, index: u32) -> DOMString {
        let index = index as usize;
        let elem = self.owner.upcast::<Element>();
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

        result.map_or(DOMString::new(), DOMString::from)
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertyvalue
    fn GetPropertyValue(&self, mut property: DOMString) -> DOMString {
        let owner = &self.owner;

        // Step 1
        property.make_ascii_lowercase();
        let property = Atom::from(&*property);

        if self.readonly {
            // Readonly style declarations are used for getComputedStyle.
            return self.get_computed_style(&property).unwrap_or(DOMString::new());
        }

        // Step 2
        if let Some(shorthand) = Shorthand::from_name(&property) {
            // Step 2.1
            let mut list = vec![];

            // Step 2.2
            for longhand in shorthand.longhands() {
                // Step 2.2.1
                let declaration = owner.get_inline_style_declaration(&Atom::from(*longhand));

                // Step 2.2.2 & 2.2.3
                match declaration {
                    Some(declaration) => list.push(declaration),
                    None => return DOMString::new(),
                }
            }

            // Step 2.3
            return DOMString::from(serialize_shorthand(shorthand, &list));
        }

        // Step 3 & 4
        match owner.get_inline_style_declaration(&property) {
            Some(declaration) => DOMString::from(declaration.value()),
            None => DOMString::new(),
        }
    }

    // https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertypriority
    fn GetPropertyPriority(&self, mut property: DOMString) -> DOMString {
        // Step 1
        property.make_ascii_lowercase();
        let property = Atom::from(&*property);

        // Step 2
        if let Some(shorthand) = Shorthand::from_name(&property) {
            // Step 2.1 & 2.2 & 2.3
            if shorthand.longhands().iter()
                                    .map(|&longhand| self.GetPropertyPriority(DOMString::from(longhand)))
                                    .all(|priority| priority == "important") {

                return DOMString::from("important");
            }
        // Step 3
        } else {
            if self.owner.get_important_inline_style_declaration(&property).is_some() {
                return DOMString::from("important");
            }
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
            "" => StylePriority::Normal,
            p if p.eq_ignore_ascii_case("important") => StylePriority::Important,
            _ => return Ok(()),
        };

        // Step 6
        let window = window_from_node(&*self.owner);
        let declarations = parse_one_declaration(&property, &value, &window.get_url(), window.css_error_reporter());

        // Step 7
        let declarations = if let Ok(declarations) = declarations {
            declarations
        } else {
            return Ok(());
        };

        let element = self.owner.upcast::<Element>();

        // Step 8
        for decl in declarations {
            // Step 9
            element.update_inline_style(decl, priority);
        }

        let document = document_from_node(element);
        let node = element.upcast();
        document.content_changed(node, NodeDamage::NodeStyleDamaged);
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
            "" => StylePriority::Normal,
            p if p.eq_ignore_ascii_case("important") => StylePriority::Important,
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

        let document = document_from_node(element);
        let node = element.upcast();
        document.content_changed(node, NodeDamage::NodeStyleDamaged);
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

        let elem = self.owner.upcast::<Element>();

        match Shorthand::from_name(&property) {
            // Step 4
            Some(shorthand) => {
                for longhand in shorthand.longhands() {
                    elem.remove_inline_style_property(longhand)
                }
            }
            // Step 5
            None => elem.remove_inline_style_property(&property),
        }

        let document = document_from_node(elem);
        let node = elem.upcast();
        document.content_changed(node, NodeDamage::NodeStyleDamaged);

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
    fn IndexedGetter(&self, index: u32, found: &mut bool) -> DOMString {
        let rval = self.Item(index);
        *found = index < self.Length();
        rval
    }

    // https://drafts.csswg.org/cssom/#cssstyledeclaration
    css_properties_accessors!(css_properties);
}
