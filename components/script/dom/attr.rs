/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::mem;
use std::sync::LazyLock;

use devtools_traits::AttrInfo;
use dom_struct::dom_struct;
use html5ever::{LocalName, Namespace, Prefix, local_name, ns};
use style::attr::{AttrIdentifier, AttrValue};
use style::values::GenericAtomIdent;
use stylo_atoms::Atom;

use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use crate::dom::bindings::codegen::UnionTypes::TrustedHTMLOrTrustedScriptOrTrustedScriptURLOrString as TrustedTypeOrString;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::root::{DomRoot, LayoutDom, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::trustedtypepolicyfactory::TrustedTypePolicyFactory;
use crate::script_runtime::CanGc;

// https://dom.spec.whatwg.org/#interface-attr
#[dom_struct]
pub(crate) struct Attr {
    node_: Node,
    #[no_trace]
    identifier: AttrIdentifier,
    #[no_trace]
    value: DomRefCell<AttrValue>,

    /// the element that owns this attribute.
    owner: MutNullableDom<Element>,
}

impl Attr {
    fn new_inherited(
        document: &Document,
        local_name: LocalName,
        value: AttrValue,
        name: LocalName,
        namespace: Namespace,
        prefix: Option<Prefix>,
        owner: Option<&Element>,
    ) -> Attr {
        Attr {
            node_: Node::new_inherited(document),
            identifier: AttrIdentifier {
                local_name: GenericAtomIdent(local_name),
                name: GenericAtomIdent(name),
                namespace: GenericAtomIdent(namespace),
                prefix: prefix.map(GenericAtomIdent),
            },
            value: DomRefCell::new(value),
            owner: MutNullableDom::new(owner),
        }
    }
    #[expect(clippy::too_many_arguments)]
    pub(crate) fn new(
        document: &Document,
        local_name: LocalName,
        value: AttrValue,
        name: LocalName,
        namespace: Namespace,
        prefix: Option<Prefix>,
        owner: Option<&Element>,
        can_gc: CanGc,
    ) -> DomRoot<Attr> {
        Node::reflect_node(
            Box::new(Attr::new_inherited(
                document, local_name, value, name, namespace, prefix, owner,
            )),
            document,
            can_gc,
        )
    }

    #[inline]
    pub(crate) fn name(&self) -> &LocalName {
        &self.identifier.name.0
    }

    #[inline]
    pub(crate) fn namespace(&self) -> &Namespace {
        &self.identifier.namespace.0
    }

    #[inline]
    pub(crate) fn prefix(&self) -> Option<&Prefix> {
        Some(&self.identifier.prefix.as_ref()?.0)
    }
}

impl AttrMethods<crate::DomTypeHolder> for Attr {
    /// <https://dom.spec.whatwg.org/#dom-attr-localname>
    fn LocalName(&self) -> DOMString {
        // FIXME(ajeffrey): convert directly from LocalName to DOMString
        DOMString::from(&**self.local_name())
    }

    /// <https://dom.spec.whatwg.org/#dom-attr-value>
    fn Value(&self) -> DOMString {
        // FIXME(ajeffrey): convert directly from AttrValue to DOMString
        DOMString::from(&**self.value())
    }

    /// <https://dom.spec.whatwg.org/#set-an-existing-attribute-value>
    fn SetValue(&self, value: DOMString, can_gc: CanGc) -> Fallible<()> {
        // Step 2. Otherwise:
        if let Some(owner) = self.owner() {
            // Step 2.1. Let element be attribute’s element.
            // Step 2.2. Let verifiedValue be the result of calling
            // get trusted type compliant attribute value with attribute’s local name,
            // attribute’s namespace, element, and value. [TRUSTED-TYPES]
            let value = TrustedTypePolicyFactory::get_trusted_types_compliant_attribute_value(
                owner.namespace(),
                owner.local_name(),
                self.local_name(),
                Some(self.namespace()),
                TrustedTypeOrString::String(value),
                &owner.owner_global(),
                can_gc,
            )?;
            if let Some(owner) = self.owner() {
                // Step 2.4. Change attribute to verifiedValue.
                let value = owner.parse_attribute(self.namespace(), self.local_name(), value);
                owner.change_attribute(self, value, can_gc);
            } else {
                // Step 2.3. If attribute’s element is null, then set attribute’s value to verifiedValue, and return.
                self.set_value(value);
            }
        } else {
            // Step 1. If attribute’s element is null, then set attribute’s value to value.
            self.set_value(value);
        }
        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-attr-name>
    fn Name(&self) -> DOMString {
        // FIXME(ajeffrey): convert directly from LocalName to DOMString
        DOMString::from(&**self.name())
    }

    /// <https://dom.spec.whatwg.org/#dom-attr-namespaceuri>
    fn GetNamespaceURI(&self) -> Option<DOMString> {
        match *self.namespace() {
            ns!() => None,
            ref url => Some(DOMString::from(&**url)),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-attr-prefix>
    fn GetPrefix(&self) -> Option<DOMString> {
        // FIXME(ajeffrey): convert directly from LocalName to DOMString
        self.prefix().map(|p| DOMString::from(&**p))
    }

    /// <https://dom.spec.whatwg.org/#dom-attr-ownerelement>
    fn GetOwnerElement(&self) -> Option<DomRoot<Element>> {
        self.owner()
    }

    /// <https://dom.spec.whatwg.org/#dom-attr-specified>
    fn Specified(&self) -> bool {
        true // Always returns true
    }
}

impl Attr {
    /// Used to swap the attribute's value without triggering mutation events
    pub(crate) fn swap_value(&self, value: &mut AttrValue) {
        mem::swap(&mut *self.value.borrow_mut(), value);
    }

    pub(crate) fn identifier(&self) -> &AttrIdentifier {
        &self.identifier
    }

    pub(crate) fn value(&self) -> Ref<'_, AttrValue> {
        self.value.borrow()
    }

    fn set_value(&self, value: DOMString) {
        *self.value.borrow_mut() = AttrValue::String(value.into());
    }

    pub(crate) fn local_name(&self) -> &LocalName {
        &self.identifier.local_name
    }

    /// Sets the owner element. Should be called after the attribute is added
    /// or removed from its older parent.
    pub(crate) fn set_owner(&self, owner: Option<&Element>) {
        let ns = self.namespace();
        match (self.owner(), owner) {
            (Some(old), None) => {
                // Already gone from the list of attributes of old owner.
                assert!(
                    old.get_attribute(ns, &self.identifier.local_name)
                        .as_deref() !=
                        Some(self)
                )
            },
            (Some(old), Some(new)) => assert_eq!(&*old, new),
            _ => {},
        }
        self.owner.set(owner);
    }

    pub(crate) fn owner(&self) -> Option<DomRoot<Element>> {
        self.owner.get()
    }

    pub(crate) fn summarize(&self) -> AttrInfo {
        AttrInfo {
            namespace: (**self.namespace()).to_owned(),
            name: (**self.name()).to_owned(),
            value: (**self.value()).to_owned(),
        }
    }

    pub(crate) fn qualified_name(&self) -> DOMString {
        match self.prefix() {
            Some(ref prefix) => DOMString::from(format!("{}:{}", prefix, &**self.local_name())),
            None => DOMString::from(&**self.local_name()),
        }
    }
}

pub(crate) trait AttrHelpersForLayout<'dom> {
    fn value(self) -> &'dom AttrValue;
    fn as_str(&self) -> &'dom str;
    fn to_tokens(self) -> Option<&'dom [Atom]>;
    fn local_name(self) -> &'dom LocalName;
    fn namespace(self) -> &'dom Namespace;
}

#[expect(unsafe_code)]
impl<'dom> AttrHelpersForLayout<'dom> for LayoutDom<'dom, Attr> {
    #[inline]
    fn value(self) -> &'dom AttrValue {
        unsafe { self.unsafe_get().value.borrow_for_layout() }
    }

    #[inline]
    fn as_str(&self) -> &'dom str {
        self.value()
    }

    #[inline]
    fn to_tokens(self) -> Option<&'dom [Atom]> {
        match *self.value() {
            AttrValue::TokenList(_, ref tokens) => Some(tokens),
            _ => None,
        }
    }

    #[inline]
    fn local_name(self) -> &'dom LocalName {
        &self.unsafe_get().identifier.local_name.0
    }

    #[inline]
    fn namespace(self) -> &'dom Namespace {
        &self.unsafe_get().identifier.namespace.0
    }
}

/// A helper function to check if attribute is relevant.
pub(crate) fn is_relevant_attribute(namespace: &Namespace, local_name: &LocalName) -> bool {
    // <https://svgwg.org/svg2-draft/linking.html#XLinkHrefAttribute>
    namespace == &ns!() || (namespace == &ns!(xlink) && local_name == &local_name!("href"))
}

/// A help function to check if an attribute is a boolean attribute.
pub(crate) fn is_boolean_attribute(name: &str) -> bool {
    // The full list of attributes can be found in [1]. All attributes marked as "Boolean
    // attribute" in the "Value" column are boolean attributes. Note that "hidden" is effectively
    // treated as a boolean attribute, according to WPT test "test_global_boolean_attributes" in
    // webdriver/tests/classic/get_element_attribute/get.py
    //
    // [1] <https://html.spec.whatwg.org/multipage/#attributes-3>
    static BOOLEAN_ATTRIBUTES: LazyLock<[&str; 30]> = LazyLock::new(|| {
        [
            "allowfullscreen",
            "alpha",
            "async",
            "autofocus",
            "autoplay",
            "checked",
            "controls",
            "default",
            "defer",
            "disabled",
            "formnovalidate",
            "hidden",
            "inert",
            "ismap",
            "itemscope",
            "loop",
            "multiple",
            "muted",
            "nomodule",
            "novalidate",
            "open",
            "playsinline",
            "readonly",
            "required",
            "reversed",
            "selected",
            "shadowrootclonable",
            "shadowrootcustomelementregistry",
            "shadowrootdelegatesfocus",
            "shadowrootserializable",
        ]
    });

    BOOLEAN_ATTRIBUTES
        .iter()
        .any(|&boolean_attr| boolean_attr.eq_ignore_ascii_case(name))
}
