/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Ref;
use std::mem::MaybeUninit;
use std::ops::Deref;

use html5ever::{LocalName, Namespace, Prefix, ns};
use itertools::Either;
use js::context::JSContext;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use script_bindings::cell::DomRefCell;
use script_bindings::root::{Dom, DomRoot};
use script_bindings::str::DOMString;
use servo_arc::{Arc, HeaderSlice, UniqueArc};
use style::attr::{AttrIdentifier, AttrValue};
use style::properties::PropertyDeclarationBlock;
use style::shared_lock::Locked;
use style::values::GenericAtomIdent;

use crate::dom::attr::Attr;
use crate::dom::bindings::root::{LayoutDom, ToLayout};
use crate::dom::element::Element;
use crate::dom::node::node::NodeTraits;

/// The empty namespace, so that [`AttrName::namespace`] can hand out a reference
/// for unqualified attributes without storing one per attribute.
static EMPTY_NAMESPACE: Namespace = ns!();

/// The name of a content attribute.
#[derive(Clone, MallocSizeOf)]
pub(crate) enum AttrName {
    /// Common case: `name == local_name`, empty namespace, no prefix.
    Unqualified(LocalName),
    /// Namespaced and/or prefixed.
    Qualified(Box<AttrIdentifier>),
}

impl AttrName {
    /// Build a name from the four parts of an [`AttrIdentifier`], picking the
    /// compact representation when they allow it.
    pub(crate) fn new(
        local_name: LocalName,
        name: LocalName,
        namespace: Namespace,
        prefix: Option<Prefix>,
    ) -> Self {
        if prefix.is_none() && name == local_name && namespace == ns!() {
            AttrName::Unqualified(local_name)
        } else {
            AttrName::Qualified(Box::new(AttrIdentifier {
                local_name: GenericAtomIdent(local_name),
                name: GenericAtomIdent(name),
                namespace: GenericAtomIdent(namespace),
                prefix: prefix.map(GenericAtomIdent),
            }))
        }
    }

    #[inline]
    pub(crate) fn local_name(&self) -> &LocalName {
        match self {
            AttrName::Unqualified(local_name) => local_name,
            AttrName::Qualified(identifier) => &identifier.local_name.0,
        }
    }

    #[inline]
    pub(crate) fn name(&self) -> &LocalName {
        match self {
            AttrName::Unqualified(local_name) => local_name,
            AttrName::Qualified(identifier) => &identifier.name.0,
        }
    }

    #[inline]
    pub(crate) fn namespace(&self) -> &Namespace {
        match self {
            AttrName::Unqualified(_) => &EMPTY_NAMESPACE,
            AttrName::Qualified(identifier) => &identifier.namespace.0,
        }
    }

    #[inline]
    pub(crate) fn prefix(&self) -> Option<&Prefix> {
        match self {
            AttrName::Unqualified(_) => None,
            AttrName::Qualified(identifier) => Some(&identifier.prefix.as_ref()?.0),
        }
    }

    /// Decompose into the parts `Attr::new` takes.
    pub(crate) fn into_parts(self) -> (LocalName, LocalName, Namespace, Option<Prefix>) {
        match self {
            AttrName::Unqualified(local_name) => (local_name.clone(), local_name, ns!(), None),
            AttrName::Qualified(identifier) => {
                let AttrIdentifier {
                    local_name,
                    name,
                    namespace,
                    prefix,
                } = *identifier;
                (local_name.0, name.0, namespace.0, prefix.map(|p| p.0))
            },
        }
    }

    /// Materialize a stylo [`AttrIdentifier`].
    pub(crate) fn as_identifier(&self) -> AttrIdentifier {
        match self {
            AttrName::Unqualified(local_name) => AttrIdentifier {
                local_name: GenericAtomIdent(local_name.clone()),
                name: GenericAtomIdent(local_name.clone()),
                namespace: GenericAtomIdent(ns!()),
                prefix: None,
            },
            AttrName::Qualified(identifier) => (**identifier).clone(),
        }
    }
}

/// Lightweight attribute storage that avoids allocating a full DOM `Attr` node.
#[derive(MallocSizeOf)]
pub(crate) struct ContentAttributeData {
    pub identifier: AttrName,
    pub value: AttrValue,
}

impl ContentAttributeData {
    #[inline]
    pub(crate) fn local_name(&self) -> &LocalName {
        self.identifier.local_name()
    }

    #[inline]
    pub(crate) fn name(&self) -> &LocalName {
        self.identifier.name()
    }

    #[inline]
    pub(crate) fn namespace(&self) -> &Namespace {
        self.identifier.namespace()
    }

    #[inline]
    pub(crate) fn prefix(&self) -> Option<&Prefix> {
        self.identifier.prefix()
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

impl Deref for AttrValueRef<'_> {
    type Target = AttrValue;

    fn deref(&self) -> &AttrValue {
        match self {
            AttrValueRef::Direct(value) => value,
            AttrValueRef::Borrowed(value) => value,
        }
    }
}

impl AsRef<str> for AttrValueRef<'_> {
    fn as_ref(&self) -> &str {
        self.deref()
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

    /// Returns a summary for devtools.
    #[cfg(feature = "devtools")]
    pub(crate) fn summarize(&self) -> devtools_traits::AttrInfo {
        devtools_traits::AttrInfo {
            namespace: (**self.namespace()).to_owned(),
            name: (**self.name()).to_owned(),
            value: (**self.value()).to_owned(),
        }
    }

    /// Materializes the stylo `AttrIdentifier` for this attribute.
    pub(crate) fn as_identifier(&self) -> AttrIdentifier {
        match self {
            AttrRef::Raw(data) => data.identifier.as_identifier(),
            AttrRef::Dom(attr) => attr.identifier().clone(),
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
            AttributeEntry::Dom(attr) => unsafe {
                let value: LayoutDom<'_, _> = attr.to_layout();
                value
            }
            .value(),
        }
    }

    /// Get the local name of this attribute for layout.
    #[expect(unsafe_code)]
    #[inline]
    pub(crate) fn local_name_for_layout(&self) -> &LocalName {
        match self {
            AttributeEntry::Raw(data) => data.local_name(),
            AttributeEntry::Dom(attr) => unsafe {
                let value: LayoutDom<'_, _> = attr.to_layout();
                value
            }
            .local_name(),
        }
    }

    /// Get the namespace of this attribute for layout.
    #[expect(unsafe_code)]
    #[inline]
    pub(crate) fn namespace_for_layout(&self) -> &Namespace {
        match self {
            AttributeEntry::Raw(data) => data.namespace(),
            AttributeEntry::Dom(attr) => unsafe {
                let value: LayoutDom<'_, _> = attr.to_layout();
                value
            }
            .namespace(),
        }
    }
}

#[derive(Default)]
struct AttributeStorageHeader {
    /// The number of stored attributes in the [`AttributeStorage`]. This is different
    /// than the [`HeaderSlice::len`], which represents the capacity.
    length: usize,
    /// Presentational hints, that is, CSS declarations that are automatically implied
    /// by the attributes of the element.
    /// <https://drafts.csswg.org/css-cascade-5/#preshint>
    /// <https://html.spec.whatwg.org/multipage/#presentational-hints>
    presentational_hints: Option<Arc<Locked<PropertyDeclarationBlock>>>,
}

impl MallocSizeOf for AttributeStorageHeader {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        if let Some(ref presentational_hints) = self.presentational_hints {
            presentational_hints.size_of(ops)
        } else {
            0
        }
    }
}

type AttributeStorageInner =
    Option<UniqueArc<HeaderSlice<AttributeStorageHeader, MaybeUninit<AttributeEntry>>>>;

/// Storage for an element's attributes. Contains an internal `DomRefCell` so that
/// `ensure_dom` can split its borrow around `Attr::new()` allocation, preventing
/// GC-during-allocation panics from double-borrowing.
/// The attributes are stored in a `HeaderSlice` so that we can also store the
/// presentational hints derived from the attributes in the header.
/// This is a compromise in order to avoid increasing the size of `Element`,
/// while having a minimal impact on performance.
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(Default)]
pub(crate) struct AttributeStorage(DomRefCell<AttributeStorageInner>);

impl MallocSizeOf for AttributeStorage {
    #[expect(unsafe_code)]
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let attributes = self.0.borrow();
        let Some(attributes) = &*attributes else {
            return 0;
        };
        let capacity = attributes.len();
        let mut n = std::mem::size_of::<AttributeStorageHeader>() + attributes.header.size_of(ops);
        n += capacity * std::mem::size_of::<MaybeUninit<AttributeEntry>>();
        for entry in &attributes.slice()[..attributes.header.length] {
            unsafe {
                n += entry.assume_init_ref().size_of(ops);
            }
        }
        n
    }
}

/// A borrowed view of attribute storage that provides convenient access
/// to attributes as `AttrRef` items. To be used in script.
pub(crate) struct AttributesBorrow<'a>(Ref<'a, AttributeStorageInner>);

impl<'a> AttributesBorrow<'a> {
    /// Iterate over attributes as `AttrRef`.
    #[inline]
    #[expect(unsafe_code)]
    pub(crate) fn iter(&self) -> impl Iterator<Item = AttrRef<'_>> + '_ {
        if let Some(attributes) = &*self.0 {
            let iterator = attributes.slice()[..self.len()]
                .iter()
                .map(|entry| unsafe { entry.assume_init_ref() }.as_ref());
            Either::Left(iterator)
        } else {
            Either::Right(std::iter::empty())
        }
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        let Some(attributes) = &*self.0 else {
            return 0;
        };
        attributes.header.length
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub(crate) fn first(&self) -> Option<AttrRef<'_>> {
        self.get(0)
    }

    #[inline]
    #[expect(unsafe_code)]
    pub(crate) fn get(&self, index: usize) -> Option<AttrRef<'_>> {
        if index >= self.len() {
            return None;
        }
        self.0
            .as_ref()
            .unwrap()
            .slice()
            .get(index)
            .map(|entry| unsafe { entry.assume_init_ref() }.as_ref())
    }
}

/// A borrowed view of attribute storage that provides convenient access
/// to attributes as `&AttributeEntry` items. To be used in layout.
pub(crate) struct AttributesRef<'a>(&'a AttributeStorageInner);

impl<'a> AttributesRef<'a> {
    #[inline]
    #[expect(unsafe_code)]
    pub(crate) fn iter(&self) -> std::slice::Iter<'a, AttributeEntry> {
        let Some(attributes) = self.0 else {
            return std::slice::Iter::default();
        };
        let slice = &attributes.slice()[..self.len()];
        // TODO: Once MSRV is raised to 1.93, use `slice.assume_init_ref()`.
        unsafe { &*(slice as *const [MaybeUninit<AttributeEntry>] as *const [AttributeEntry]) }
            .iter()
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        let Some(attributes) = self.0 else {
            return 0;
        };
        attributes.header.length
    }

    #[inline]
    pub(crate) fn presentational_hints(&self) -> Option<&'a Arc<Locked<PropertyDeclarationBlock>>> {
        self.0.as_ref()?.header.presentational_hints.as_ref()
    }
}

#[inline]
fn assert_valid_index(index: usize, len: usize) {
    assert!(index < len, "Index {index} isn't smaller than length {len}");
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
    pub(crate) unsafe fn borrow_for_layout<'a>(&'a self) -> AttributesRef<'a> {
        AttributesRef(unsafe { self.0.borrow_for_layout() })
    }

    /// Reserve room for at least `additional` more attributes. Unlike `reserve()`, this
    /// will not deliberately over-allocate to speculatively avoid frequent allocations.
    #[inline]
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn reserve_exact(&self, additional: usize) {
        let mut attributes = self.0.borrow_mut();
        let Some(attributes) = &mut *attributes else {
            *attributes = Some(UniqueArc::from_header_and_iter_with_size(
                AttributeStorageHeader::default(),
                std::iter::repeat_with(MaybeUninit::uninit).take(additional),
                additional,
            ));
            return;
        };
        let current_length = attributes.header.length;
        let current_capacity = attributes.len();
        let needed_capacity = current_length.saturating_add(additional);
        if needed_capacity > isize::MAX as usize {
            panic!("Capacity overflow");
        }

        // Fast path if we already have enough capacity.
        if needed_capacity <= current_capacity {
            return;
        }

        // Replace the UniqueArc with a new one with more capacity, filled with
        // the data taken from the old one.
        let mut new_attributes = UniqueArc::from_header_and_iter_with_size(
            AttributeStorageHeader {
                presentational_hints: attributes.header.presentational_hints.take(),
                length: 0,
            },
            std::iter::repeat_with(MaybeUninit::uninit).take(needed_capacity),
            needed_capacity,
        );
        new_attributes.slice_mut()[0..current_length]
            .swap_with_slice(&mut attributes.slice_mut()[0..current_length]);
        std::mem::swap(
            &mut new_attributes.header.length,
            &mut attributes.header.length,
        );
        *attributes = new_attributes;
    }

    /// Reserve room for at least `additional` more attributes. The collection may reserve
    /// more space to speculatively avoid frequent reallocations.
    pub(crate) fn reserve(&self, additional: usize) {
        let mut borrow = self.0.borrow_mut();
        let Some(attributes) = &mut *borrow else {
            *borrow = Some(UniqueArc::from_header_and_iter_with_size(
                AttributeStorageHeader::default(),
                std::iter::repeat_with(MaybeUninit::uninit).take(additional),
                additional,
            ));
            return;
        };
        let current_length = attributes.header.length;
        let current_capacity = attributes.len();
        std::mem::drop(borrow);
        let needed_capacity = current_length.saturating_add(additional);

        // Fast path if we already have enough capacity.
        if needed_capacity <= current_capacity {
            return;
        }

        // Doubling the capacity provides exponential growth, so that `push()` can have
        // an amortized cost of O(1). The doubling cannot overflow `usize` because the
        // capacity doesn't exceed `isize::MAX`.
        let new_capacity = needed_capacity.max(current_capacity * 2);
        self.reserve_exact(new_capacity - current_length);
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn push(&self, entry: AttributeEntry) {
        self.reserve(1);
        let mut attributes = self.0.borrow_mut();
        let attributes = attributes.as_mut().unwrap();
        let current_length = attributes.header.length;
        attributes.slice_mut()[current_length].write(entry);
        attributes.header.length = current_length + 1;
    }

    /// Push raw attribute data.
    pub(crate) fn push_raw(&self, data: ContentAttributeData) {
        self.push(AttributeEntry::Raw(data));
    }

    /// Push a Dom Attr node.
    pub(crate) fn push_dom(&self, attr: &Attr) {
        self.push(AttributeEntry::Dom(Dom::from_ref(attr)));
    }

    /// Remove an attribute by index, returning the entry.
    #[expect(unsafe_code)]
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn remove(&self, index: usize) -> AttributeEntry {
        let mut attributes = self.0.borrow_mut();
        let Some(attributes) = &mut *attributes else {
            assert_valid_index(index, 0);
            unreachable!("The above should panic with the right message");
        };
        let current_length = attributes.header.length;
        assert_valid_index(index, current_length);
        let slice = attributes.slice_mut();
        let slice_ptr = slice.as_mut_ptr();
        let removed;
        unsafe {
            removed = slice[index].assume_init_read();
            std::ptr::copy(
                slice_ptr.add(index + 1),
                slice_ptr.add(index),
                current_length - index - 1,
            );
        }
        attributes.header.length -= 1;
        removed
    }

    /// Set an attribute entry by index.
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    #[expect(unsafe_code)]
    pub(crate) fn set(&self, index: usize, entry: AttributeEntry) {
        let mut attributes = self.0.borrow_mut();
        let Some(attributes) = &mut *attributes else {
            assert_valid_index(index, 0);
            unreachable!("The above should panic with the right message");
        };
        assert_valid_index(index, attributes.header.length);
        let slot = &mut attributes.slice_mut()[index];
        unsafe {
            slot.assume_init_drop();
        }
        slot.write(entry);
    }

    /// Ensure entry at index is a Dom Attr node, materializing if needed.
    /// Returns a `DomRoot<Attr>`.
    ///
    /// This method carefully splits the mutable borrow around the `Attr::new()`
    /// allocation so that GC tracing can safely borrow the storage.
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    #[expect(unsafe_code)]
    pub(crate) fn ensure_dom(
        &self,
        cx: &mut JSContext,
        index: usize,
        element: &Element,
    ) -> DomRoot<Attr> {
        let mut attributes = self.0.borrow_mut();
        let Some(attributes) = &mut *attributes else {
            assert_valid_index(index, 0);
            unreachable!("The above should panic with the right message");
        };
        assert_valid_index(index, attributes.header.length);

        // Fast path: already materialized.
        if let AttributeEntry::Dom(attr) = unsafe { attributes.slice()[index].assume_init_ref() } {
            return DomRoot::from_ref(&**attr);
        }

        // Extract the raw data, dropping the mutable borrow before allocating.
        let data = {
            let entries = attributes.slice_mut();
            let entry = unsafe { entries[index].assume_init_mut() };
            let placeholder = AttributeEntry::Raw(ContentAttributeData {
                identifier: AttrName::Unqualified(html5ever::local_name!("")),
                value: AttrValue::String(String::new()),
            });
            let old = std::mem::replace(entry, placeholder);
            match old {
                AttributeEntry::Raw(data) => data,
                _ => unreachable!(),
            }
        };

        let doc = element.owner_document();
        let (local_name, name, namespace, prefix) = data.identifier.into_parts();
        let attr = Attr::new(
            cx,
            &doc,
            local_name,
            data.value,
            name,
            namespace,
            prefix,
            Some(element),
        );

        let slot = &mut attributes.slice_mut()[index];
        unsafe {
            slot.assume_init_drop();
        }
        slot.write(AttributeEntry::Dom(Dom::from_ref(&*attr)));
        attr
    }

    #[expect(unsafe_code)]
    pub(crate) fn set_presentational_hints(
        &self,
        presentational_hints: Option<Arc<Locked<PropertyDeclarationBlock>>>,
    ) {
        unsafe { self.0.borrow_mut_for_layout() }
            .get_or_insert_with(|| {
                UniqueArc::from_header_and_iter(
                    AttributeStorageHeader::default(),
                    std::iter::empty::<MaybeUninit<AttributeEntry>>(),
                )
            })
            .header
            .presentational_hints = presentational_hints;
    }
}

impl Drop for AttributeStorage {
    #[expect(unsafe_code)]
    fn drop(&mut self) {
        let mut attributes = self.0.borrow_mut();
        let Some(attributes) = &mut *attributes else {
            return;
        };
        let length = attributes.header.length;
        for entry in &mut attributes.slice_mut()[..length] {
            unsafe {
                entry.assume_init_drop();
            }
        }
    }
}

// Safety: Only Dom entries contain GC-traced Dom<Attr> pointers.
// Raw entries have no pointers to trace.
#[expect(unsafe_code)]
unsafe impl crate::dom::bindings::trace::JSTraceable for AttributeStorage {
    unsafe fn trace(&self, trc: *mut js::jsapi::JSTracer) {
        let attributes = self.0.borrow();
        let Some(attributes) = &*attributes else {
            return;
        };
        for entry in attributes.slice()[..attributes.header.length].iter() {
            if let AttributeEntry::Dom(attr) = unsafe { entry.assume_init_ref() } {
                unsafe { js::rust::Trace::trace(attr, trc) };
            }
        }
    }
}
