/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Ref;

use devtools_traits::AttrInfo;
use html5ever::{LocalName, Namespace, Prefix};
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::root::{Dom, DomRoot};
use script_bindings::str::DOMString;
use style::attr::{AttrIdentifier, AttrValue};

use crate::dom::attr::Attr;
use crate::dom::bindings::root::ToLayout;
use crate::dom::element::Element;
use crate::dom::node::node::NodeTraits;

/// Lightweight attribute storage that avoids allocating a full DOM `Attr` node.
#[derive(MallocSizeOf)]
pub(crate) struct ContentAttributeData {
    pub identifier: AttrIdentifier,
    pub value: AttrValue,
}

impl ContentAttributeData {
    #[inline]
    pub(crate) fn local_name(&self) -> &LocalName {
        &self.identifier.local_name.0
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

    #[inline]
    pub(crate) fn value(&self) -> &AttrValue {
        &self.value
    }
}

/// A reference to an attribute value, abstracting over direct and RefCell-borrowed access.
pub(crate) enum AttrValueRef<'a> {
    /// Direct reference to a value (from [`ContentAttributeData`]).
    Direct(&'a AttrValue),
    /// Borrowed from a [`DomRefCell`] (from [`Attr`]).
    Borrowed(Ref<'a, AttrValue>),
}

impl std::ops::Deref for AttrValueRef<'_> {
    type Target = AttrValue;

    fn deref(&self) -> &AttrValue {
        match self {
            AttrValueRef::Direct(value) => value,
            AttrValueRef::Borrowed(value) => value,
        }
    }
}

/// A reference to attribute data, either from a lightweight [`ContentAttributeData`]
/// or from a full [`Attr`] DOM node. Provides the same accessor interface regardless
/// of storage form.
#[derive(Clone, Copy)]
pub(crate) enum AttrRef<'a> {
    /// Lightweight data (no DOM node allocated).
    Raw(&'a ContentAttributeData),
    /// Full Attr DOM node.
    Dom(&'a Attr),
}

impl<'a> AttrRef<'a> {
    #[inline]
    pub(crate) fn local_name(&self) -> &'a LocalName {
        match self {
            AttrRef::Raw(data) => data.local_name(),
            AttrRef::Dom(attr) => attr.local_name(),
        }
    }

    #[inline]
    pub(crate) fn name(&self) -> &'a LocalName {
        match self {
            AttrRef::Raw(data) => data.name(),
            AttrRef::Dom(attr) => attr.name(),
        }
    }

    #[inline]
    pub(crate) fn namespace(&self) -> &'a Namespace {
        match self {
            AttrRef::Raw(data) => data.namespace(),
            AttrRef::Dom(attr) => attr.namespace(),
        }
    }

    #[inline]
    pub(crate) fn prefix(&self) -> Option<&'a Prefix> {
        match self {
            AttrRef::Raw(data) => data.prefix(),
            AttrRef::Dom(attr) => attr.prefix(),
        }
    }

    #[inline]
    pub(crate) fn value(&self) -> AttrValueRef<'a> {
        match self {
            AttrRef::Raw(data) => AttrValueRef::Direct(data.value()),
            AttrRef::Dom(attr) => AttrValueRef::Borrowed(attr.value()),
        }
    }

    /// Returns the underlying `&Attr` if this is a `Dom` reference.
    /// Returns `None` for `Raw` data (no DOM node exists).
    #[inline]
    pub(crate) fn as_attr(&self) -> Option<&'a Attr> {
        match self {
            AttrRef::Dom(attr) => Some(attr),
            AttrRef::Raw(_) => None,
        }
    }

    /// Returns the attribute value as a `DOMString`, equivalent to `Attr::Value()`.
    pub(crate) fn to_dom_string(self) -> DOMString {
        DOMString::from(&**self.value())
    }

    /// Returns a summary for devtools, equivalent to `Attr::summarize()`.
    pub(crate) fn summarize(&self) -> AttrInfo {
        AttrInfo {
            namespace: (**self.namespace()).to_owned(),
            name: (**self.name()).to_owned(),
            value: (**self.value()).to_owned(),
        }
    }

    /// Returns the `AttrIdentifier` for this attribute.
    pub(crate) fn identifier(&self) -> &AttrIdentifier {
        match self {
            AttrRef::Raw(data) => &data.identifier,
            AttrRef::Dom(attr) => attr.identifier(),
        }
    }
}

/// A single attribute entry, either lightweight raw data or a full DOM Attr node.
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(MallocSizeOf)]
pub(crate) enum AttributeEntry {
    /// Lightweight data — no Attr DOM node allocated.
    Raw(ContentAttributeData),
    /// Full Attr DOM node.
    Dom(Dom<Attr>),
}

impl AttributeEntry {
    /// Get an `AttrRef` for this entry.
    #[inline]
    pub(crate) fn as_ref(&self) -> AttrRef<'_> {
        match self {
            AttributeEntry::Raw(data) => AttrRef::Raw(data),
            AttributeEntry::Dom(attr) => AttrRef::Dom(attr),
        }
    }

    /// Get the value of this attribute for layout.
    #[expect(unsafe_code)]
    #[inline]
    pub(crate) fn value_for_layout(&self) -> &AttrValue {
        match self {
            AttributeEntry::Raw(data) => &data.value,
            AttributeEntry::Dom(attr) => unsafe { attr.to_layout() }.value(),
        }
    }

    /// Get the local name of this attribute for layout.
    #[expect(unsafe_code)]
    #[inline]
    pub(crate) fn local_name_for_layout(&self) -> &LocalName {
        match self {
            AttributeEntry::Raw(data) => data.local_name(),
            AttributeEntry::Dom(attr) => unsafe { attr.to_layout() }.local_name(),
        }
    }

    /// Get the namespace of this attribute for layout.
    #[expect(unsafe_code)]
    #[inline]
    pub(crate) fn namespace_for_layout(&self) -> &Namespace {
        match self {
            AttributeEntry::Raw(data) => data.namespace(),
            AttributeEntry::Dom(attr) => unsafe { attr.to_layout() }.namespace(),
        }
    }
}

/// Storage for an element's attributes. Contains an internal `DomRefCell` so that
/// `ensure_dom` can split its borrow around `Attr::new()` allocation, preventing
/// GC-during-allocation panics from double-borrowing.
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(Default, MallocSizeOf)]
pub(crate) struct AttributeStorage(DomRefCell<Vec<AttributeEntry>>);

/// A borrowed view of attribute storage that provides convenient access
/// to attributes as `AttrRef` items.
pub(crate) struct AttributesBorrow<'a>(Ref<'a, Vec<AttributeEntry>>);

impl<'a> AttributesBorrow<'a> {
    /// Iterate over attributes as `AttrRef`.
    #[inline]
    pub(crate) fn iter(&self) -> impl Iterator<Item = AttrRef<'_>> + '_ {
        self.0.iter().map(AttributeEntry::as_ref)
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub(crate) fn first(&self) -> Option<AttrRef<'_>> {
        self.0.first().map(AttributeEntry::as_ref)
    }

    #[inline]
    pub(crate) fn get(&self, index: usize) -> Option<AttrRef<'_>> {
        self.0.get(index).map(AttributeEntry::as_ref)
    }
}

impl AttributeStorage {
    /// Borrow the attributes for read access with convenient `AttrRef` iteration.
    #[inline]
    pub(crate) fn borrow(&self) -> AttributesBorrow<'_> {
        AttributesBorrow(self.0.borrow())
    }

    /// Borrow the underlying entries for layout access (unsafe).
    #[expect(unsafe_code)]
    #[inline]
    pub(crate) unsafe fn borrow_for_layout(&self) -> &Vec<AttributeEntry> {
        unsafe { self.0.borrow_for_layout() }
    }

    /// Push raw attribute data.
    pub(crate) fn push_raw(&self, data: ContentAttributeData) {
        self.0.borrow_mut().push(AttributeEntry::Raw(data));
    }

    /// Push a Dom Attr node.
    pub(crate) fn push_dom(&self, attr: &Attr) {
        self.0
            .borrow_mut()
            .push(AttributeEntry::Dom(Dom::from_ref(attr)));
    }

    /// Remove an attribute by index, returning the entry.
    pub(crate) fn remove(&self, index: usize) -> AttributeEntry {
        self.0.borrow_mut().remove(index)
    }

    /// Set an attribute entry by index.
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn set(&self, index: usize, entry: AttributeEntry) {
        self.0.borrow_mut()[index] = entry;
    }

    /// Ensure entry at index is a Dom Attr node, materializing if needed.
    /// Returns a `DomRoot<Attr>`.
    ///
    /// This method carefully splits the mutable borrow around the `Attr::new()`
    /// allocation so that GC tracing can safely borrow the storage.
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn ensure_dom(
        &self,
        cx: &mut JSContext,
        index: usize,
        element: &Element,
    ) -> DomRoot<Attr> {
        // Fast path: already materialized.
        if let AttributeEntry::Dom(attr) = &self.0.borrow()[index] {
            return DomRoot::from_ref(&**attr);
        }

        // Extract the raw data, dropping the mutable borrow before allocating.
        let data = {
            let mut entries = self.0.borrow_mut();
            let placeholder = AttributeEntry::Raw(ContentAttributeData {
                identifier: AttrIdentifier {
                    local_name: style::values::GenericAtomIdent(html5ever::local_name!("")),
                    name: style::values::GenericAtomIdent(html5ever::local_name!("")),
                    namespace: style::values::GenericAtomIdent(html5ever::ns!()),
                    prefix: None,
                },
                value: AttrValue::String(String::new()),
            });
            let old = std::mem::replace(&mut entries[index], placeholder);
            match old {
                AttributeEntry::Raw(data) => data,
                _ => unreachable!(),
            }
        };

        let doc = element.owner_document();
        let attr = Attr::new(
            cx,
            &doc,
            data.identifier.local_name.0,
            data.value,
            data.identifier.name.0,
            data.identifier.namespace.0,
            data.identifier.prefix.map(|p| p.0),
            Some(element),
        );

        self.0.borrow_mut()[index] = AttributeEntry::Dom(Dom::from_ref(&*attr));
        attr
    }
}

// Safety: Only Dom entries contain GC-traced Dom<Attr> pointers.
// Raw entries have no pointers to trace.
#[expect(unsafe_code)]
unsafe impl crate::dom::bindings::trace::JSTraceable for AttributeStorage {
    unsafe fn trace(&self, trc: *mut js::jsapi::JSTracer) {
        for entry in self.0.borrow().iter() {
            if let AttributeEntry::Dom(attr) = entry {
                unsafe { js::rust::Trace::trace(attr, trc) };
            }
        }
    }
}
