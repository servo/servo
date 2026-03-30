/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html5ever::{LocalName, ns};
use js::context::JSContext;
use style::attr::AttrValue;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use crate::dom::bindings::codegen::UnionTypes::{TrustedHTMLOrString, TrustedScriptURLOrUSVString};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::element::Element;
use crate::dom::node::NodeTraits;
use crate::script_runtime::CanGc;

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
        local_name: &LocalName,
        value: DOMString,
        can_gc: CanGc,
    ) {
        self.set_attribute(local_name, AttrValue::from_atomic(value.into()), can_gc);
    }

    pub(crate) fn set_bool_attribute(&self, local_name: &LocalName, value: bool, can_gc: CanGc) {
        if self.has_attribute(local_name) == value {
            return;
        }
        if value {
            self.set_string_attribute(local_name, DOMString::new(), can_gc);
        } else {
            self.remove_attribute(&ns!(), local_name, can_gc);
        }
    }

    pub(crate) fn get_url_attribute(&self, local_name: &LocalName) -> USVString {
        let Some(attribute) = self.get_attribute(local_name) else {
            return Default::default();
        };
        let value = &**attribute.value();
        self.owner_document()
            .encoding_parse_a_url(value)
            .map(|parsed| USVString(parsed.into_string()))
            .unwrap_or_else(|_| USVString(value.to_owned()))
    }

    pub(crate) fn set_url_attribute(
        &self,
        local_name: &LocalName,
        value: USVString,
        can_gc: CanGc,
    ) {
        self.set_attribute(local_name, AttrValue::String(value.to_string()), can_gc);
    }

    pub(crate) fn get_trusted_type_url_attribute(
        &self,
        local_name: &LocalName,
    ) -> TrustedScriptURLOrUSVString {
        let Some(attribute) = self.get_attribute(local_name) else {
            return TrustedScriptURLOrUSVString::USVString(USVString::default());
        };
        let value = &**attribute.value();
        self.owner_document()
            .encoding_parse_a_url(value)
            .map(|parsed| TrustedScriptURLOrUSVString::USVString(USVString(parsed.into_string())))
            .unwrap_or_else(|_| TrustedScriptURLOrUSVString::USVString(USVString(value.to_owned())))
    }

    pub(crate) fn get_trusted_html_attribute(&self, local_name: &LocalName) -> TrustedHTMLOrString {
        TrustedHTMLOrString::String(self.get_string_attribute(local_name))
    }

    pub(crate) fn get_string_attribute(&self, local_name: &LocalName) -> DOMString {
        self.get_attribute(local_name)
            .map(|attribute| attribute.Value())
            .unwrap_or_default()
    }

    pub(crate) fn set_string_attribute(
        &self,
        local_name: &LocalName,
        value: DOMString,
        can_gc: CanGc,
    ) {
        self.set_attribute(local_name, AttrValue::String(value.into()), can_gc);
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
                self.set_string_attribute(local_name, val, CanGc::from_cx(cx));
            },
            None => {
                self.remove_attribute(&ns!(), local_name, CanGc::from_cx(cx));
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
        local_name: &LocalName,
        value: DOMString,
        can_gc: CanGc,
    ) {
        self.set_attribute(
            local_name,
            AttrValue::from_serialized_tokenlist(value.into()),
            can_gc,
        );
    }

    pub(crate) fn set_atomic_tokenlist_attribute(
        &self,
        local_name: &LocalName,
        tokens: Vec<Atom>,
        can_gc: CanGc,
    ) {
        self.set_attribute(local_name, AttrValue::from_atomic_tokens(tokens), can_gc);
    }

    pub(crate) fn set_int_attribute(&self, local_name: &LocalName, value: i32, can_gc: CanGc) {
        self.set_attribute(local_name, AttrValue::Int(value.to_string(), value), can_gc);
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

    pub(crate) fn set_uint_attribute(&self, local_name: &LocalName, value: u32, can_gc: CanGc) {
        self.set_attribute(
            local_name,
            AttrValue::UInt(value.to_string(), value),
            can_gc,
        );
    }
}
