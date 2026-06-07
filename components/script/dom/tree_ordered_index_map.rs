/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, Ref};
use std::collections::hash_map::Entry;

use js::context::NoGC;
use rustc_hash::{FxBuildHasher, FxHashMap};
use script_bindings::assert::assert_in_script;
use script_bindings::cell::DomRefCell;
use script_bindings::inheritance::Castable;
use script_bindings::root::{Dom, DomRoot};
use style::Atom;

use crate::dom::Node;
use crate::dom::bindings::root::{LayoutDom, MutNullableDom};
use crate::dom::bindings::trace::HashMapTracedValues;
use crate::dom::iterators::ShadowIncluding;
use crate::dom::types::Element;

/// An entry in the [`TreeOrderedIndexMap`].
///
/// This entry has two states: resolved and unresolved. When an entry in the map refers to a
/// single element, it is resolved. When any additional entries are added or removed, the entry
/// becomes unresolved. At that point, in order to know what element an `id` or `name`
/// attribute refer to, the [`TreeOrderedIndexMap`] walks the DOM and fills out the `elements`
/// and `element` field of the entry.
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct TreeOrderedIndexMapEntry {
    /// The first element in the tree that has this key. If this is null,
    /// then it means the list needs to be regenerated after a modification
    /// of the map.
    element: MutNullableDom<Element>,
    /// An ordered list of [`Element`]s that have this key. When this is empty,
    /// it means the list needs to be regenerated after a modification of the map.
    elements: Vec<Dom<Element>>,
    /// The number of [`Element`]s that have this key. This is used in order to
    /// do an early return during generation of [`Self::elements`].
    count: usize,
}

impl TreeOrderedIndexMapEntry {
    fn new(element: &Element) -> Self {
        Self {
            element: MutNullableDom::new(Some(element)),
            elements: vec![Dom::from_ref(element)],
            count: 1,
        }
    }

    fn add(&mut self) {
        assert!(self.count >= 1);
        self.count += 1;
        self.element.clear();
        self.elements.clear()
    }

    fn remove(&mut self) -> bool {
        self.count -= 1;
        self.element.clear();
        self.elements.clear();
        self.count > 0
    }

    /// Returns true if this entry needs to be resolved. An entry will need to be
    /// resolved if it ever gains more than one element associated with it or has
    /// more than one element associated and `Self::remove()` is called.
    fn needs_resolution(&self) -> bool {
        // An empty elements array implicitly means that an entry needs resolution, because
        // `Self::add()` and `Self::remove()` will clear the array.
        self.elements.is_empty()
    }
}

#[derive(Clone, Copy, JSTraceable, MallocSizeOf)]
enum IndexType {
    Id,
    Name,
}

impl IndexType {
    fn get(&self, element: &Element) -> Option<Atom> {
        match self {
            IndexType::Id => element.get_id(),
            IndexType::Name => element.get_name(),
        }
    }
}

/// A data structure that tracks the use of an identifier type in the DOM. Currently this is
/// meant for tracking elements that share an `id` or `name` attribute.
///
/// The core problem this structure is trying to solve is that elements with the same `id` and
/// 'name' often need to be accessed in document order, yet calculating the order of elements
/// is expensive. This structure makes calculation of that ordering as cheap as possible by
/// delaying it until the last moment before it is needed.
///
/// The goal here is to mitigate the situation where many elements on a page share the same
/// identifier. Maintaining an in-order map in that case has very negative performance
/// characteristics. This map solves that problem, making the optimal case still O(1). It's
/// actually not very common that a normal page needs the elements in order during script
/// execution.
///
/// Each entry in the map holds a cached ordering, initially valid if there is only a single
/// element that has a particular `id` or `name`. When another element is added to the map
/// which shares that identifier, the cache is invalidated and count of elements is maintained.
/// Only when needing to access the ordered representation of the elements with the identifier
/// do we walk the DOM, collecting the various elements in order.
///
/// There is one unfortunate penalty we have to pay due to Servo's rooting design: layout that
/// might match elements by id *does* need elements in order. Before layout or running any
/// query selectors, we need to resolve all unresolved entries in the map. In the end though,
/// this is still just a single walk over the DOM. This is *only* run when a map entry becomes
/// invalid due to to an element sharing an id being added or removed.
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct TreeOrderedIndexMap {
    /// It is safe to use FxHash for these maps as Atoms are `string_cache` items that will
    /// have the hash computed from a u32.
    map: DomRefCell<HashMapTracedValues<Atom, TreeOrderedIndexMapEntry, FxBuildHasher>>,
    /// Whether or not this map might need to be resolved for layout or for calls to
    /// [`Self::for_each`].
    might_need_rebuild_for_layout: Cell<bool>,
    /// The [`IndexType`] that this [`TreeOrderedIndexMap`] uses, which is either
    /// the `id` or `name` attribute.
    index_type: IndexType,
}

impl TreeOrderedIndexMap {
    pub(crate) fn name() -> Self {
        Self {
            map: Default::default(),
            might_need_rebuild_for_layout: Default::default(),
            index_type: IndexType::Name,
        }
    }

    pub(crate) fn id() -> Self {
        Self {
            map: Default::default(),
            might_need_rebuild_for_layout: Default::default(),
            index_type: IndexType::Id,
        }
    }

    /// Add an entry to this map from the `key` (an `id` or `name` attribute value) to
    /// `Element`. If this is the first time this key is used, then the map will be able to map
    /// to the entry without having to resolve ordering.
    pub(crate) fn add(&self, key: &Atom, element: &Element) {
        debug_assert!(
            element.upcast::<Node>().is_in_a_document_tree() ||
                element.upcast::<Node>().is_in_a_shadow_tree()
        );
        debug_assert!(!key.is_empty());

        let mut map = self.map.borrow_mut();
        match map.entry(key.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(TreeOrderedIndexMapEntry::new(element));
            },
            Entry::Occupied(mut entry) => {
                entry.get_mut().add();
                self.might_need_rebuild_for_layout.set(true);
            },
        }
    }

    /// Remove an entry from the map with `key` (an `id` or `name` attribute value). If there
    /// are more elements that share the same key, the entry will need resoluton before use.
    /// Removal of the last element for an entry will remove it from the map.
    pub(crate) fn remove(&self, key: &Atom) {
        let mut map = self.map.borrow_mut();
        let Entry::Occupied(mut occupied_entry) = map.entry(key.clone()) else {
            unreachable!("Tried to remove unknown id or name entry: {key}");
        };
        if !occupied_entry.get_mut().remove() {
            occupied_entry.remove();
        } else {
            self.might_need_rebuild_for_layout.set(true);
        }
    }

    /// Get the first element that has this `key` from the DOM, possibly resolving the ordering
    /// of unresolved entries.
    ///
    /// Note: This should *never* be used during layout as it does rooting and unrooting.
    pub(crate) fn get(&self, no_gc: &NoGC, scope: &Node, key: &Atom) -> Option<DomRoot<Element>> {
        assert_in_script();

        let mut map = self.map.borrow_mut();
        let Entry::Occupied(mut occupied_entry) = map.entry(key.clone()) else {
            return None;
        };
        if let Some(element) = occupied_entry.get().element.get() {
            return Some(element);
        }

        Self::resolve_one(no_gc, scope, key, occupied_entry.get_mut(), self.index_type);
        if let Some(element) = occupied_entry.get().element.get() {
            return Some(element);
        }

        occupied_entry.remove();
        None
    }

    /// Get all of the entries in the map, in DOM order that share a particular `key`. If the
    /// entry for this key is unresolved, it will be resolved.
    ///
    /// Note: This should *never* be used during layout as it does rooting and unrooting.
    pub(crate) fn get_all(
        &self,
        no_gc: &NoGC,
        scope: &Node,
        key: &Atom,
    ) -> Ref<'_, [Dom<Element>]> {
        assert_in_script();

        {
            let mut map = self.map.borrow_mut();
            if let Entry::Occupied(mut occupied_entry) = map.entry(key.clone()) {
                if occupied_entry.get().needs_resolution() {
                    Self::resolve_one(no_gc, scope, key, occupied_entry.get_mut(), self.index_type);
                }
                if occupied_entry.get().elements.is_empty() {
                    occupied_entry.remove();
                }
            }
        }
        Ref::map(self.map.borrow(), |map| {
            map.get(key)
                .map(|entry| &*entry.elements)
                .unwrap_or_default()
        })
    }

    /// Run the given callback against every (key, elements) tuple in this map. This will
    /// resolve all unresolved entries in the map.
    ///
    /// Note: This should *never* be used during layout as it does rooting and unrooting.
    pub(crate) fn for_each(
        &self,
        no_gc: &NoGC,
        scope: &Node,
        mut callback: impl FnMut(&Atom, &[Dom<Element>]),
    ) {
        self.resolve_all(no_gc, scope);
        for (key, entry) in self.map.borrow().iter() {
            callback(key, &entry.elements);
        }
    }

    /// Get all of the entries in the map, in DOM order that share a particular `key`. This
    /// will not resolve any entries. It is an error to call this before an entry is resolved.
    #[expect(unsafe_code)]
    pub(crate) fn get_all_for_layout(&self, key: &Atom) -> &[LayoutDom<'_, Element>] {
        // # Safety: `Dom<Element>` should have the exact same memory layout as
        // `LayoutDom<'_, Element>` so this should be safe.
        unsafe {
            self.map
                .borrow_for_layout()
                .get(key)
                .map_or(&[], |entry| LayoutDom::to_layout_slice(&entry.elements))
        }
    }

    /// Resolve a single entry in the map that has more than one element associated with it.
    /// This will walk the DOM and gather all of the elements that have this id/name associated
    /// with them in DOM order.
    fn resolve_one(
        no_gc: &NoGC,
        scope: &Node,
        key: &Atom,
        entry: &mut TreeOrderedIndexMapEntry,
        index_type: IndexType,
    ) {
        for node in scope.traverse_preorder_non_rooting(no_gc, ShadowIncluding::No) {
            let Some(element) = node.downcast::<Element>() else {
                continue;
            };
            match index_type.get(element) {
                Some(id) if id == *key => {
                    entry.elements.push(Dom::from_ref(element));
                    entry.element.or_init(|| DomRoot::from_ref(element));

                    if entry.count == entry.elements.len() {
                        return;
                    }
                },
                _ => {},
            }
        }

        entry.count = entry.elements.len();
    }

    /// Resolve all entries in the map, meaning the next access will not need any resolution.
    /// This should be called before any layout activity that accesses the map.
    pub(crate) fn resolve_all(&self, no_gc: &NoGC, scope: &Node) {
        if !self.might_need_rebuild_for_layout.take() {
            return;
        }

        let mut map = self.map.borrow_mut();
        let mut entries_needing_rebuild: FxHashMap<_, _> = map
            .iter_mut()
            .filter(|(_, entry)| entry.needs_resolution())
            .collect();

        for node in scope.traverse_preorder_non_rooting(no_gc, ShadowIncluding::No) {
            let Some(element) = node.downcast::<Element>() else {
                continue;
            };

            let Some(index) = self.index_type.get(element) else {
                continue;
            };

            let mut complete = false;
            if let Some(entry) = entries_needing_rebuild.get_mut(&index) {
                entry.elements.push(Dom::from_ref(element));
                entry.element.or_init(|| DomRoot::from_ref(element));
                complete = entry.count == entry.elements.len();
            }

            // Stop tracking entries that are complete.
            if complete {
                entries_needing_rebuild.remove(&index);

                // When we have completed all entries, exit this method entirely.
                if entries_needing_rebuild.is_empty() {
                    return;
                }
            }
        }

        // If no elements were found for any of the entries needing resolution,
        // remove them from the map mirroring what `resolve_one` does. For all
        // other entries, update their final `count`.
        let mut keys_needing_removal = Vec::new();
        for (key, entry) in entries_needing_rebuild {
            if entry.elements.is_empty() {
                keys_needing_removal.push(key.clone());
            } else {
                entry.count = entry.elements.len();
            }
        }

        for key in keys_needing_removal {
            map.remove(&key);
        }
    }
}
