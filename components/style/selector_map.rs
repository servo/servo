/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A data structure to efficiently index structs containing selectors by local
//! name, ids and hash.

use {Atom, LocalName};
use applicable_declarations::ApplicableDeclarationBlock;
use context::QuirksMode;
use dom::TElement;
use fnv::FnvHashMap;
use pdqsort::sort_by;
use rule_tree::CascadeLevel;
use selector_parser::SelectorImpl;
use selectors::matching::{matches_selector, MatchingContext, ElementSelectorFlags};
use selectors::parser::{AncestorHashes, Component, Combinator, SelectorAndHashes, SelectorIter};
use selectors::parser::LocalName as LocalNameSelector;
use smallvec::VecLike;
use std::collections::HashMap;
use std::collections::hash_map;
use std::hash::Hash;
use stylist::Rule;

/// A trait to abstract over a given selector map entry.
pub trait SelectorMapEntry : Sized + Clone {
    /// Gets the selector we should use to index in the selector map.
    fn selector(&self) -> SelectorIter<SelectorImpl>;

    /// Gets the ancestor hashes associated with the selector.
    fn hashes(&self) -> &AncestorHashes;
}

impl SelectorMapEntry for SelectorAndHashes<SelectorImpl> {
    fn selector(&self) -> SelectorIter<SelectorImpl> {
        self.selector.iter()
    }

    fn hashes(&self) -> &AncestorHashes {
        &self.hashes
    }
}

/// Map element data to selector-providing objects for which the last simple
/// selector starts with them.
///
/// e.g.,
/// "p > img" would go into the set of selectors corresponding to the
/// element "img"
/// "a .foo .bar.baz" would go into the set of selectors corresponding to
/// the class "bar"
///
/// Because we match selectors right-to-left (i.e., moving up the tree
/// from an element), we need to compare the last simple selector in the
/// selector with the element.
///
/// So, if an element has ID "id1" and classes "foo" and "bar", then all
/// the rules it matches will have their last simple selector starting
/// either with "#id1" or with ".foo" or with ".bar".
///
/// Hence, the union of the rules keyed on each of element's classes, ID,
/// element name, etc. will contain the Selectors that actually match that
/// element.
///
/// TODO: Tune the initial capacity of the HashMap
#[derive(Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct SelectorMap<T: SelectorMapEntry> {
    /// A hash from an ID to rules which contain that ID selector.
    pub id_hash: MaybeCaseInsensitiveHashMap<Atom, Vec<T>>,
    /// A hash from a class name to rules which contain that class selector.
    pub class_hash: MaybeCaseInsensitiveHashMap<Atom, Vec<T>>,
    /// A hash from local name to rules which contain that local name selector.
    pub local_name_hash: FnvHashMap<LocalName, Vec<T>>,
    /// Rules that don't have ID, class, or element selectors.
    pub other: Vec<T>,
    /// The number of entries in this map.
    pub count: usize,
}

#[inline]
fn sort_by_key<T, F: Fn(&T) -> K, K: Ord>(v: &mut [T], f: F) {
    sort_by(v, |a, b| f(a).cmp(&f(b)))
}

impl<T: SelectorMapEntry> SelectorMap<T> {
    /// Trivially constructs an empty `SelectorMap`.
    pub fn new() -> Self {
        SelectorMap {
            id_hash: MaybeCaseInsensitiveHashMap::new(),
            class_hash: MaybeCaseInsensitiveHashMap::new(),
            local_name_hash: HashMap::default(),
            other: Vec::new(),
            count: 0,
        }
    }

    /// Returns whether there are any entries in the map.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.count
    }
}

impl SelectorMap<Rule> {
    /// Append to `rule_list` all Rules in `self` that match element.
    ///
    /// Extract matching rules as per element's ID, classes, tag name, etc..
    /// Sort the Rules at the end to maintain cascading order.
    pub fn get_all_matching_rules<E, V, F>(&self,
                                           element: &E,
                                           rule_hash_target: &E,
                                           matching_rules_list: &mut V,
                                           context: &mut MatchingContext,
                                           quirks_mode: QuirksMode,
                                           flags_setter: &mut F,
                                           cascade_level: CascadeLevel)
        where E: TElement,
              V: VecLike<ApplicableDeclarationBlock>,
              F: FnMut(&E, ElementSelectorFlags),
    {
        if self.is_empty() {
            return
        }

        // At the end, we're going to sort the rules that we added, so remember where we began.
        let init_len = matching_rules_list.len();
        if let Some(id) = rule_hash_target.get_id() {
            if let Some(rules) = self.id_hash.get(&id, quirks_mode) {
                SelectorMap::get_matching_rules(element,
                                                rules,
                                                matching_rules_list,
                                                context,
                                                flags_setter,
                                                cascade_level)
            }
        }

        rule_hash_target.each_class(|class| {
            if let Some(rules) = self.class_hash.get(&class, quirks_mode) {
                SelectorMap::get_matching_rules(element,
                                                rules,
                                                matching_rules_list,
                                                context,
                                                flags_setter,
                                                cascade_level)
            }
        });

        if let Some(rules) = self.local_name_hash.get(rule_hash_target.get_local_name()) {
            SelectorMap::get_matching_rules(element,
                                            rules,
                                            matching_rules_list,
                                            context,
                                            flags_setter,
                                            cascade_level)
        }

        SelectorMap::get_matching_rules(element,
                                        &self.other,
                                        matching_rules_list,
                                        context,
                                        flags_setter,
                                        cascade_level);

        // Sort only the rules we just added.
        sort_by_key(&mut matching_rules_list[init_len..],
                    |block| (block.specificity, block.source_order()));
    }

    /// Append to `rule_list` all universal Rules (rules with selector `*|*`) in
    /// `self` sorted by specificity and source order.
    pub fn get_universal_rules(&self,
                               cascade_level: CascadeLevel)
                               -> Vec<ApplicableDeclarationBlock> {
        debug_assert!(!cascade_level.is_important());
        if self.is_empty() {
            return vec![];
        }

        let mut rules_list = vec![];
        for rule in self.other.iter() {
            if rule.selector.is_universal() {
                rules_list.push(rule.to_applicable_declaration_block(cascade_level))
            }
        }

        sort_by_key(&mut rules_list,
                    |block| (block.specificity, block.source_order()));

        rules_list
    }

    /// Adds rules in `rules` that match `element` to the `matching_rules` list.
    fn get_matching_rules<E, V, F>(element: &E,
                                   rules: &[Rule],
                                   matching_rules: &mut V,
                                   context: &mut MatchingContext,
                                   flags_setter: &mut F,
                                   cascade_level: CascadeLevel)
        where E: TElement,
              V: VecLike<ApplicableDeclarationBlock>,
              F: FnMut(&E, ElementSelectorFlags),
    {
        for rule in rules {
            if matches_selector(&rule.selector,
                                0,
                                &rule.hashes,
                                element,
                                context,
                                flags_setter) {
                matching_rules.push(
                    rule.to_applicable_declaration_block(cascade_level));
            }
        }
    }
}

impl<T: SelectorMapEntry> SelectorMap<T> {
    /// Inserts into the correct hash, trying id, class, and localname.
    pub fn insert(&mut self, entry: T, quirks_mode: QuirksMode) {
        self.count += 1;

        if let Some(id_name) = get_id_name(entry.selector()) {
            self.id_hash.entry(id_name, quirks_mode).or_insert_with(Vec::new).push(entry);
            return;
        }

        if let Some(class_name) = get_class_name(entry.selector()) {
            self.class_hash.entry(class_name, quirks_mode).or_insert_with(Vec::new).push(entry);
            return;
        }

        if let Some(LocalNameSelector { name, lower_name }) = get_local_name(entry.selector()) {
            // If the local name in the selector isn't lowercase, insert it into
            // the rule hash twice. This means that, during lookup, we can always
            // find the rules based on the local name of the element, regardless
            // of whether it's an html element in an html document (in which case
            // we match against lower_name) or not (in which case we match against
            // name).
            //
            // In the case of a non-html-element-in-html-document with a
            // lowercase localname and a non-lowercase selector, the rulehash
            // lookup may produce superfluous selectors, but the subsequent
            // selector matching work will filter them out.
            if name != lower_name {
                find_push(&mut self.local_name_hash, lower_name, entry.clone());
            }
            find_push(&mut self.local_name_hash, name, entry);

            return;
        }

        self.other.push(entry);
    }

    /// Looks up entries by id, class, local name, and other (in order).
    ///
    /// Each entry is passed to the callback, which returns true to continue
    /// iterating entries, or false to terminate the lookup.
    ///
    /// Returns false if the callback ever returns false.
    ///
    /// FIXME(bholley) This overlaps with SelectorMap<Rule>::get_all_matching_rules,
    /// but that function is extremely hot and I'd rather not rearrange it.
    #[inline]
    pub fn lookup<E, F>(&self, element: E, quirks_mode: QuirksMode, f: &mut F) -> bool
        where E: TElement,
              F: FnMut(&T) -> bool
    {
        // Id.
        if let Some(id) = element.get_id() {
            if let Some(v) = self.id_hash.get(&id, quirks_mode) {
                for entry in v.iter() {
                    if !f(&entry) {
                        return false;
                    }
                }
            }
        }

        // Class.
        let mut done = false;
        element.each_class(|class| {
            if !done {
                if let Some(v) = self.class_hash.get(class, quirks_mode) {
                    for entry in v.iter() {
                        if !f(&entry) {
                            done = true;
                            return;
                        }
                    }
                }
            }
        });
        if done {
            return false;
        }

        // Local name.
        if let Some(v) = self.local_name_hash.get(element.get_local_name()) {
            for entry in v.iter() {
                if !f(&entry) {
                    return false;
                }
            }
        }

        // Other.
        for entry in self.other.iter() {
            if !f(&entry) {
                return false;
            }
        }

        true
    }

    /// Performs a normal lookup, and also looks up entries for the passed-in
    /// id and classes.
    ///
    /// Each entry is passed to the callback, which returns true to continue
    /// iterating entries, or false to terminate the lookup.
    ///
    /// Returns false if the callback ever returns false.
    #[inline]
    pub fn lookup_with_additional<E, F>(&self,
                                        element: E,
                                        quirks_mode: QuirksMode,
                                        additional_id: Option<Atom>,
                                        additional_classes: &[Atom],
                                        f: &mut F)
                                        -> bool
        where E: TElement,
              F: FnMut(&T) -> bool
    {
        // Do the normal lookup.
        if !self.lookup(element, quirks_mode, f) {
            return false;
        }

        // Check the additional id.
        if let Some(id) = additional_id {
            if let Some(v) = self.id_hash.get(&id, quirks_mode) {
                for entry in v.iter() {
                    if !f(&entry) {
                        return false;
                    }
                }
            }
        }

        // Check the additional classes.
        for class in additional_classes {
            if let Some(v) = self.class_hash.get(class, quirks_mode) {
                for entry in v.iter() {
                    if !f(&entry) {
                        return false;
                    }
                }
            }
        }

        true
    }
}

/// Searches the selector from right to left, beginning to the left of the
/// ::pseudo-element (if any), and ending at the first combinator.
///
/// The first non-None value returned from |f| is returned.
///
/// Effectively, pseudo-elements are ignored, given only state pseudo-classes
/// may appear before them.
#[inline(always)]
fn find_from_right<F, R>(mut iter: SelectorIter<SelectorImpl>,
                         mut f: F)
                         -> Option<R>
    where F: FnMut(&Component<SelectorImpl>) -> Option<R>,
{
    for ss in &mut iter {
        if let Some(r) = f(ss) {
            return Some(r)
        }
    }

    if iter.next_sequence() == Some(Combinator::PseudoElement) {
        for ss in &mut iter {
            if let Some(r) = f(ss) {
                return Some(r)
            }
        }
    }

    None
}

/// Retrieve the first ID name in the selector, or None otherwise.
#[inline(always)]
pub fn get_id_name(iter: SelectorIter<SelectorImpl>)
                   -> Option<Atom> {
    find_from_right(iter, |ss| {
        // TODO(pradeep): Implement case-sensitivity based on the
        // document type and quirks mode.
        if let Component::ID(ref id) = *ss {
            return Some(id.clone());
        }
        None
    })
}

/// Retrieve the FIRST class name in the selector, or None otherwise.
#[inline(always)]
pub fn get_class_name(iter: SelectorIter<SelectorImpl>)
                      -> Option<Atom> {
    find_from_right(iter, |ss| {
        // TODO(pradeep): Implement case-sensitivity based on the
        // document type and quirks mode.
        if let Component::Class(ref class) = *ss {
            return Some(class.clone());
        }
        None
    })
}

/// Retrieve the name if it is a type selector, or None otherwise.
#[inline(always)]
pub fn get_local_name(iter: SelectorIter<SelectorImpl>)
                      -> Option<LocalNameSelector<SelectorImpl>> {
    find_from_right(iter, |ss| {
        if let Component::LocalName(ref n) = *ss {
            return Some(LocalNameSelector {
                name: n.name.clone(),
                lower_name: n.lower_name.clone(),
            })
        }
        None
    })
}

#[inline]
fn find_push<Str: Eq + Hash, V>(map: &mut FnvHashMap<Str, Vec<V>>,
                                key: Str,
                                value: V) {
    map.entry(key).or_insert_with(Vec::new).push(value)
}

/// Wrapper for FnvHashMap that does ASCII-case-insensitive lookup in quirks mode.
#[derive(Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct MaybeCaseInsensitiveHashMap<K: Hash + Eq, V>(FnvHashMap<K, V>);

impl<V> MaybeCaseInsensitiveHashMap<Atom, V> {
    /// Empty map
    pub fn new() -> Self {
        MaybeCaseInsensitiveHashMap(FnvHashMap::default())
    }

    /// HashMap::entry
    pub fn entry(&mut self, mut key: Atom, quirks_mode: QuirksMode) -> hash_map::Entry<Atom, V> {
        if quirks_mode == QuirksMode::Quirks {
            key = key.to_ascii_lowercase()
        }
        self.0.entry(key)
    }

    /// HashMap::get
    pub fn get(&self, key: &Atom, quirks_mode: QuirksMode) -> Option<&V> {
        if quirks_mode == QuirksMode::Quirks {
            self.0.get(&key.to_ascii_lowercase())
        } else {
            self.0.get(key)
        }
    }
}
