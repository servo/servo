/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html5ever::{LocalName, local_name, ns};
use js::context::JSContext;
use servo_arc::Arc as ServoArc;
use style::attr::AttrValue;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::UnionTypes::{TrustedHTMLOrString, TrustedScriptURLOrUSVString};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::element::attributes::storage::AttrRef;
use crate::dom::element::{AttributeMutationReason, Element};
use crate::dom::node::NodeTraits;

impl Element {
    pub(crate) fn get_int_attribute(&self, local_name: &LocalName, default: i32) -> i32 {
        match self.get_attribute(local_name) {
            Some(ref attribute) => match *attribute.value() {
                AttrValue::Int(_, value) => value,
                _ => unreachable!("Expected an AttrValue::Int: implement parse_plain_attribute"),
            },
            None => default,
        }
    }

    pub(crate) fn set_atomic_attribute(
        &self,
        cx: &mut JSContext,
        local_name: &LocalName,
        value: DOMString,
    ) {
        self.set_attribute(cx, local_name, AttrValue::from_atomic(value.into()));
    }

    pub(crate) fn set_bool_attribute(
        &self,
        cx: &mut JSContext,
        local_name: &LocalName,
        value: bool,
    ) {
        if self.has_attribute(local_name) == value {
            return;
        }
        if value {
            self.set_string_attribute(cx, local_name, DOMString::new());
        } else {
            self.remove_attribute(cx, &ns!(), local_name);
        }
    }

    pub(crate) fn get_url_attribute(&self, local_name: &LocalName) -> USVString {
        let Some(value) = self.get_attribute_string_value(local_name) else {
            return Default::default();
        };
        self.owner_document()
            .encoding_parse_a_url(&value)
            .map(|parsed| USVString(parsed.into_string()))
            .unwrap_or_else(|_| USVString(value))
    }

    pub(crate) fn set_url_attribute(
        &self,
        cx: &mut JSContext,
        local_name: &LocalName,
        value: USVString,
    ) {
        self.set_attribute(cx, local_name, AttrValue::String(value.into()));
    }

    pub(crate) fn get_trusted_type_url_attribute(
        &self,
        local_name: &LocalName,
    ) -> TrustedScriptURLOrUSVString {
        let Some(value) = self.get_attribute_string_value(local_name) else {
            return TrustedScriptURLOrUSVString::USVString(USVString::default());
        };
        self.owner_document()
            .encoding_parse_a_url(&value)
            .map(|parsed| TrustedScriptURLOrUSVString::USVString(USVString(parsed.into_string())))
            .unwrap_or_else(|_| TrustedScriptURLOrUSVString::USVString(USVString(value)))
    }

    pub(crate) fn get_trusted_html_attribute(&self, local_name: &LocalName) -> TrustedHTMLOrString {
        TrustedHTMLOrString::String(self.get_string_attribute(local_name))
    }

    pub(crate) fn get_string_attribute(&self, local_name: &LocalName) -> DOMString {
        self.get_attribute_string_value(local_name)
            .map(|value| value.into())
            .unwrap_or_default()
    }

    pub(crate) fn set_string_attribute(
        &self,
        cx: &mut JSContext,
        local_name: &LocalName,
        value: DOMString,
    ) {
        self.set_attribute(cx, local_name, value.str().to_string().into());
    }

    /// Used for string attribute reflections where absence of the attribute returns `null`,
    /// e.g. `element.ariaLabel` returning `null` when the `aria-label` attribute is absent.
    pub(crate) fn get_nullable_string_attribute(
        &self,
        local_name: &LocalName,
    ) -> Option<DOMString> {
        if self.has_attribute(local_name) {
            Some(self.get_string_attribute(local_name))
        } else {
            None
        }
    }

    /// Used for string attribute reflections where setting `null`/`undefined` removes the
    /// attribute, e.g. `element.ariaLabel = null` removing the `aria-label` attribute.
    pub(crate) fn set_nullable_string_attribute(
        &self,
        cx: &mut JSContext,
        local_name: &LocalName,
        value: Option<DOMString>,
    ) {
        match value {
            Some(val) => {
                self.set_string_attribute(cx, local_name, val);
            },
            None => {
                self.remove_attribute(cx, &ns!(), local_name);
            },
        }
    }

    pub(crate) fn get_tokenlist_attribute(&self, local_name: &LocalName) -> Vec<Atom> {
        self.get_attribute(local_name)
            .map(|attribute| attribute.value().as_tokens().to_vec())
            .unwrap_or_default()
    }

    pub(crate) fn set_tokenlist_attribute(
        &self,
        cx: &mut JSContext,
        local_name: &LocalName,
        value: DOMString,
    ) {
        self.set_attribute(
            cx,
            local_name,
            AttrValue::from_serialized_tokenlist(value.into()),
        );
    }

    pub(crate) fn get_uint_attribute(&self, local_name: &LocalName, default: u32) -> u32 {
        match self.get_attribute(local_name) {
            Some(ref attribute) => match *attribute.value() {
                AttrValue::UInt(_, value) => value,
                _ => unreachable!("Expected an AttrValue::UInt: implement parse_plain_attribute"),
            },
            None => default,
        }
    }

    /// Ensure that for styles, we clone the already-parsed property declaration block.
    /// This does two things:
    /// 1. It uses the same fast-path as CSSStyleDeclaration
    /// 2. It also avoids the CSP checks when cloning (it shouldn't run any when cloning
    ///    existing valid attributes)
    fn compute_attribute_value_with_style_fast_path(&self, attr: AttrRef<'_>) -> AttrValue {
        if *attr.local_name() == local_name!("style") {
            let document = self.owner_document();

            if let AttrValue::Declaration {
                block,
                lock,
                serialization,
            } = &*attr.value()
            {
                // Even though the property declaration block inside this AttrValue will
                // be replaced, the serialization will be exactly the same, so preserve
                // that instead of re-serializing.
                let cloned_block = block.read_with(&lock.read()).clone();
                return AttrValue::Declaration {
                    block: ServoArc::new(lock.wrap(cloned_block)),
                    lock: lock.clone(),
                    serialization: serialization.clone(),
                };
            }

            if let Some(ref pdb) = *self.style_attribute().borrow() {
                let shared_lock = document.style_shared_author_lock();
                let new_pdb = pdb.read_with(&shared_lock.read()).clone();
                return AttrValue::Declaration {
                    block: ServoArc::new(shared_lock.wrap(new_pdb)),
                    lock: shared_lock.clone(),
                    // The style attribute was not set via a declaration, so try to
                    // preserve any serialization that existed before instead of
                    // re-serializing.
                    serialization: (**attr.value()).to_owned().into(),
                };
            }
        }

        attr.value().clone()
    }

    /// <https://dom.spec.whatwg.org/#concept-node-clone>
    pub(crate) fn copy_all_attributes_to_other_element(
        &self,
        cx: &mut JSContext,
        target_element: &Element,
    ) {
        // Step 2.5. For each attribute of node’s attribute list:
        for attr in self.attrs().borrow().iter() {
            // Step 2.5.1. Let copyAttribute be the result of cloning a single node given attribute, document, and null.
            let new_value = self.compute_attribute_value_with_style_fast_path(attr);
            // Step 2.5.2. Append copyAttribute to copy.
            target_element.push_new_attribute(
                cx,
                attr.local_name().clone(),
                new_value,
                attr.name().clone(),
                attr.namespace().clone(),
                attr.prefix().cloned(),
                AttributeMutationReason::ByCloning,
            );
        }
    }
}
