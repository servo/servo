/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A collection of invalidations due to changes in which stylesheets affect a
//! document.

#![deny(unsafe_code)]

use crate::context::QuirksMode;
use crate::dom::{TDocument, TElement, TNode};
use crate::invalidation::element::element_wrapper::{ElementSnapshot, ElementWrapper};
use crate::invalidation::element::restyle_hints::RestyleHint;
use crate::media_queries::Device;
use crate::selector_map::{MaybeCaseInsensitiveHashMap, PrecomputedHashMap};
use crate::selector_parser::{SelectorImpl, Snapshot, SnapshotMap};
use crate::shared_lock::SharedRwLockReadGuard;
use crate::stylesheets::{CssRule, StylesheetInDocument};
use crate::stylesheets::{EffectiveRules, EffectiveRulesIterator};
use crate::Atom;
use crate::LocalName as SelectorLocalName;
use selectors::parser::{Component, LocalName, Selector};

/// The kind of change that happened for a given rule.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
pub enum RuleChangeKind {
    /// The rule was inserted.
    Insertion,
    /// The rule was removed.
    Removal,
    /// Some change in the rule which we don't know about, and could have made
    /// the rule change in any way.
    Generic,
    /// A change in the declarations of a style rule.
    StyleRuleDeclarations,
}

/// A style sheet invalidation represents a kind of element or subtree that may
/// need to be restyled. Whether it represents a whole subtree or just a single
/// element is determined by the given InvalidationKind in
/// StylesheetInvalidationSet's maps.
#[derive(Debug, Eq, Hash, MallocSizeOf, PartialEq)]
enum Invalidation {
    /// An element with a given id.
    ID(Atom),
    /// An element with a given class name.
    Class(Atom),
    /// An element with a given local name.
    LocalName {
        name: SelectorLocalName,
        lower_name: SelectorLocalName,
    },
}

impl Invalidation {
    fn is_id(&self) -> bool {
        matches!(*self, Invalidation::ID(..))
    }

    fn is_id_or_class(&self) -> bool {
        matches!(*self, Invalidation::ID(..) | Invalidation::Class(..))
    }
}

/// Whether we should invalidate just the element, or the whole subtree within
/// it.
#[derive(Copy, Clone, Debug, Eq, MallocSizeOf, Ord, PartialEq, PartialOrd)]
enum InvalidationKind {
    None = 0,
    Element,
    Scope,
}

impl std::ops::BitOrAssign for InvalidationKind {
    #[inline]
    fn bitor_assign(&mut self, other: Self) {
        *self = std::cmp::max(*self, other);
    }
}

impl InvalidationKind {
    #[inline]
    fn is_scope(self) -> bool {
        matches!(self, Self::Scope)
    }

    #[inline]
    fn add(&mut self, other: Option<&InvalidationKind>) {
        if let Some(other) = other {
            *self |= *other;
        }
    }
}

/// A set of invalidations due to stylesheet additions.
///
/// TODO(emilio): We might be able to do the same analysis for media query
/// changes too (or even selector changes?).
#[derive(Debug, Default, MallocSizeOf)]
pub struct StylesheetInvalidationSet {
    classes: MaybeCaseInsensitiveHashMap<Atom, InvalidationKind>,
    ids: MaybeCaseInsensitiveHashMap<Atom, InvalidationKind>,
    local_names: PrecomputedHashMap<SelectorLocalName, InvalidationKind>,
    fully_invalid: bool,
}

impl StylesheetInvalidationSet {
    /// Create an empty `StylesheetInvalidationSet`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Mark the DOM tree styles' as fully invalid.
    pub fn invalidate_fully(&mut self) {
        debug!("StylesheetInvalidationSet::invalidate_fully");
        self.clear();
        self.fully_invalid = true;
    }

    /// Analyze the given stylesheet, and collect invalidations from their
    /// rules, in order to avoid doing a full restyle when we style the document
    /// next time.
    pub fn collect_invalidations_for<S>(
        &mut self,
        device: &Device,
        stylesheet: &S,
        guard: &SharedRwLockReadGuard,
    ) where
        S: StylesheetInDocument,
    {
        debug!("StylesheetInvalidationSet::collect_invalidations_for");
        if self.fully_invalid {
            debug!(" > Fully invalid already");
            return;
        }

        if !stylesheet.enabled() || !stylesheet.is_effective_for_device(device, guard) {
            debug!(" > Stylesheet was not effective");
            return; // Nothing to do here.
        }

        let quirks_mode = stylesheet.quirks_mode(guard);
        for rule in stylesheet.effective_rules(device, guard) {
            self.collect_invalidations_for_rule(rule, guard, device, quirks_mode);
            if self.fully_invalid {
                break;
            }
        }

        debug!(" > resulting class invalidations: {:?}", self.classes);
        debug!(" > resulting id invalidations: {:?}", self.ids);
        debug!(
            " > resulting local name invalidations: {:?}",
            self.local_names
        );
        debug!(" > fully_invalid: {}", self.fully_invalid);
    }

    /// Clears the invalidation set, invalidating elements as needed if
    /// `document_element` is provided.
    ///
    /// Returns true if any invalidations ocurred.
    pub fn flush<E>(&mut self, document_element: Option<E>, snapshots: Option<&SnapshotMap>) -> bool
    where
        E: TElement,
    {
        debug!(
            "Stylist::flush({:?}, snapshots: {})",
            document_element,
            snapshots.is_some()
        );
        let have_invalidations = match document_element {
            Some(e) => self.process_invalidations(e, snapshots),
            None => false,
        };
        self.clear();
        have_invalidations
    }

    /// Returns whether there's no invalidation to process.
    pub fn is_empty(&self) -> bool {
        !self.fully_invalid &&
            self.classes.is_empty() &&
            self.ids.is_empty() &&
            self.local_names.is_empty()
    }

    fn invalidation_kind_for<E>(
        &self,
        element: E,
        snapshot: Option<&Snapshot>,
        quirks_mode: QuirksMode,
    ) -> InvalidationKind
    where
        E: TElement,
    {
        debug_assert!(!self.fully_invalid);

        let mut kind = InvalidationKind::None;

        if !self.classes.is_empty() {
            element.each_class(|c| {
                kind.add(self.classes.get(c, quirks_mode));
            });

            if kind.is_scope() {
                return kind;
            }

            if let Some(snapshot) = snapshot {
                snapshot.each_class(|c| {
                    kind.add(self.classes.get(c, quirks_mode));
                });

                if kind.is_scope() {
                    return kind;
                }
            }
        }

        if !self.ids.is_empty() {
            if let Some(ref id) = element.id() {
                kind.add(self.ids.get(id, quirks_mode));
                if kind.is_scope() {
                    return kind;
                }
            }

            if let Some(ref old_id) = snapshot.and_then(|s| s.id_attr()) {
                kind.add(self.ids.get(old_id, quirks_mode));
                if kind.is_scope() {
                    return kind;
                }
            }
        }

        if !self.local_names.is_empty() {
            kind.add(self.local_names.get(element.local_name()));
        }

        kind
    }

    /// Clears the invalidation set without processing.
    pub fn clear(&mut self) {
        self.classes.clear();
        self.ids.clear();
        self.local_names.clear();
        self.fully_invalid = false;
        debug_assert!(self.is_empty());
    }

    fn process_invalidations<E>(&self, element: E, snapshots: Option<&SnapshotMap>) -> bool
    where
        E: TElement,
    {
        debug!("Stylist::process_invalidations({:?}, {:?})", element, self);

        {
            let mut data = match element.mutate_data() {
                Some(data) => data,
                None => return false,
            };

            if self.fully_invalid {
                debug!("process_invalidations: fully_invalid({:?})", element);
                data.hint.insert(RestyleHint::restyle_subtree());
                return true;
            }
        }

        if self.is_empty() {
            debug!("process_invalidations: empty invalidation set");
            return false;
        }

        let quirks_mode = element.as_node().owner_doc().quirks_mode();
        self.process_invalidations_in_subtree(element, snapshots, quirks_mode)
    }

    /// Process style invalidations in a given subtree. This traverses the
    /// subtree looking for elements that match the invalidations in our hash
    /// map members.
    ///
    /// Returns whether it invalidated at least one element's style.
    #[allow(unsafe_code)]
    fn process_invalidations_in_subtree<E>(
        &self,
        element: E,
        snapshots: Option<&SnapshotMap>,
        quirks_mode: QuirksMode,
    ) -> bool
    where
        E: TElement,
    {
        debug!("process_invalidations_in_subtree({:?})", element);
        let mut data = match element.mutate_data() {
            Some(data) => data,
            None => return false,
        };

        if !data.has_styles() {
            return false;
        }

        if data.hint.contains_subtree() {
            debug!(
                "process_invalidations_in_subtree: {:?} was already invalid",
                element
            );
            return false;
        }

        let element_wrapper = snapshots.map(|s| ElementWrapper::new(element, s));
        let snapshot = element_wrapper.as_ref().and_then(|e| e.snapshot());

        match self.invalidation_kind_for(element, snapshot, quirks_mode) {
            InvalidationKind::None => {},
            InvalidationKind::Element => {
                debug!(
                    "process_invalidations_in_subtree: {:?} matched self",
                    element
                );
                data.hint.insert(RestyleHint::RESTYLE_SELF);
            },
            InvalidationKind::Scope => {
                debug!(
                    "process_invalidations_in_subtree: {:?} matched subtree",
                    element
                );
                data.hint.insert(RestyleHint::restyle_subtree());
                return true;
            },
        }

        let mut any_children_invalid = false;

        for child in element.traversal_children() {
            let child = match child.as_element() {
                Some(e) => e,
                None => continue,
            };

            any_children_invalid |=
                self.process_invalidations_in_subtree(child, snapshots, quirks_mode);
        }

        if any_children_invalid {
            debug!(
                "Children of {:?} changed, setting dirty descendants",
                element
            );
            unsafe { element.set_dirty_descendants() }
        }

        data.hint.contains(RestyleHint::RESTYLE_SELF) || any_children_invalid
    }

    /// TODO(emilio): Reuse the bucket stuff from selectormap? That handles
    /// :is() / :where() etc.
    fn scan_component(
        component: &Component<SelectorImpl>,
        invalidation: &mut Option<Invalidation>,
    ) {
        match *component {
            Component::LocalName(LocalName {
                ref name,
                ref lower_name,
            }) => {
                if invalidation.is_none() {
                    *invalidation = Some(Invalidation::LocalName {
                        name: name.clone(),
                        lower_name: lower_name.clone(),
                    });
                }
            },
            Component::Class(ref class) => {
                if invalidation.as_ref().map_or(true, |s| !s.is_id_or_class()) {
                    *invalidation = Some(Invalidation::Class(class.clone()));
                }
            },
            Component::ID(ref id) => {
                if invalidation.as_ref().map_or(true, |s| !s.is_id()) {
                    *invalidation = Some(Invalidation::ID(id.clone()));
                }
            },
            _ => {
                // Ignore everything else, at least for now.
            },
        }
    }

    /// Collect invalidations for a given selector.
    ///
    /// We look at the outermost local name, class, or ID selector to the left
    /// of an ancestor combinator, in order to restyle only a given subtree.
    ///
    /// If the selector has no ancestor combinator, then we do the same for
    /// the only sequence it has, but record it as an element invalidation
    /// instead of a subtree invalidation.
    ///
    /// We prefer IDs to classs, and classes to local names, on the basis
    /// that the former should be more specific than the latter. We also
    /// prefer to generate subtree invalidations for the outermost part
    /// of the selector, to reduce the amount of traversal we need to do
    /// when flushing invalidations.
    fn collect_invalidations(
        &mut self,
        selector: &Selector<SelectorImpl>,
        quirks_mode: QuirksMode,
    ) {
        debug!(
            "StylesheetInvalidationSet::collect_invalidations({:?})",
            selector
        );

        let mut element_invalidation: Option<Invalidation> = None;
        let mut subtree_invalidation: Option<Invalidation> = None;

        let mut scan_for_element_invalidation = true;
        let mut scan_for_subtree_invalidation = false;

        let mut iter = selector.iter();

        loop {
            for component in &mut iter {
                if scan_for_element_invalidation {
                    Self::scan_component(component, &mut element_invalidation);
                } else if scan_for_subtree_invalidation {
                    Self::scan_component(component, &mut subtree_invalidation);
                }
            }
            match iter.next_sequence() {
                None => break,
                Some(combinator) => {
                    scan_for_subtree_invalidation = combinator.is_ancestor();
                },
            }
            scan_for_element_invalidation = false;
        }

        if let Some(s) = subtree_invalidation {
            debug!(" > Found subtree invalidation: {:?}", s);
            if self.insert_invalidation(s, InvalidationKind::Scope, quirks_mode) {
                return;
            }
        }

        if let Some(s) = element_invalidation {
            debug!(" > Found element invalidation: {:?}", s);
            if self.insert_invalidation(s, InvalidationKind::Element, quirks_mode) {
                return;
            }
        }

        // The selector was of a form that we can't handle. Any element could
        // match it, so let's just bail out.
        debug!(" > Can't handle selector or OOMd, marking fully invalid");
        self.invalidate_fully()
    }

    fn insert_invalidation(
        &mut self,
        invalidation: Invalidation,
        kind: InvalidationKind,
        quirks_mode: QuirksMode,
    ) -> bool {
        match invalidation {
            Invalidation::Class(c) => {
                let entry = match self.classes.try_entry(c, quirks_mode) {
                    Ok(e) => e,
                    Err(..) => return false,
                };
                *entry.or_insert(InvalidationKind::None) |= kind;
            },
            Invalidation::ID(i) => {
                let entry = match self.ids.try_entry(i, quirks_mode) {
                    Ok(e) => e,
                    Err(..) => return false,
                };
                *entry.or_insert(InvalidationKind::None) |= kind;
            },
            Invalidation::LocalName { name, lower_name } => {
                let insert_lower = name != lower_name;
                let entry = match self.local_names.try_entry(name) {
                    Ok(e) => e,
                    Err(..) => return false,
                };
                *entry.or_insert(InvalidationKind::None) |= kind;
                if insert_lower {
                    let entry = match self.local_names.try_entry(lower_name) {
                        Ok(e) => e,
                        Err(..) => return false,
                    };
                    *entry.or_insert(InvalidationKind::None) |= kind;
                }
            },
        }

        true
    }

    /// Collects invalidations for a given CSS rule, if not fully invalid
    /// already.
    ///
    /// TODO(emilio): we can't check whether the rule is inside a non-effective
    /// subtree, we potentially could do that.
    pub fn rule_changed<S>(
        &mut self,
        stylesheet: &S,
        rule: &CssRule,
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        change_kind: RuleChangeKind,
    ) where
        S: StylesheetInDocument,
    {
        use crate::stylesheets::CssRule::*;

        debug!("StylesheetInvalidationSet::rule_changed");
        if self.fully_invalid {
            return;
        }

        if !stylesheet.enabled() || !stylesheet.is_effective_for_device(device, guard) {
            debug!(" > Stylesheet was not effective");
            return; // Nothing to do here.
        }

        let is_generic_change = change_kind == RuleChangeKind::Generic;

        match *rule {
            Namespace(..) => {
                // It's not clear what handling changes for this correctly would
                // look like.
            },
            CounterStyle(..) |
            Page(..) |
            Viewport(..) |
            FontFeatureValues(..) |
            FontFace(..) |
            Keyframes(..) |
            Style(..) => {
                if is_generic_change {
                    // TODO(emilio): We need to do this for selector / keyframe
                    // name / font-face changes, because we don't have the old
                    // selector / name.  If we distinguish those changes
                    // specially, then we can at least use this invalidation for
                    // style declaration changes.
                    return self.invalidate_fully();
                }

                self.collect_invalidations_for_rule(rule, guard, device, quirks_mode)
            },
            Document(..) | Import(..) | Media(..) | Supports(..) => {
                if !is_generic_change &&
                    !EffectiveRules::is_effective(guard, device, quirks_mode, rule)
                {
                    return;
                }

                let rules =
                    EffectiveRulesIterator::effective_children(device, quirks_mode, guard, rule);
                for rule in rules {
                    self.collect_invalidations_for_rule(rule, guard, device, quirks_mode);
                    if self.fully_invalid {
                        break;
                    }
                }
            },
        }
    }

    /// Collects invalidations for a given CSS rule.
    fn collect_invalidations_for_rule(
        &mut self,
        rule: &CssRule,
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
    ) {
        use crate::stylesheets::CssRule::*;
        debug!("StylesheetInvalidationSet::collect_invalidations_for_rule");
        debug_assert!(!self.fully_invalid, "Not worth to be here!");

        match *rule {
            Style(ref lock) => {
                let style_rule = lock.read_with(guard);
                for selector in &style_rule.selectors.0 {
                    self.collect_invalidations(selector, quirks_mode);
                    if self.fully_invalid {
                        return;
                    }
                }
            },
            Document(..) | Namespace(..) | Import(..) | Media(..) | Supports(..) => {
                // Do nothing, relevant nested rules are visited as part of the
                // iteration.
            },
            FontFace(..) => {
                // Do nothing, @font-face doesn't affect computed style
                // information. We'll restyle when the font face loads, if
                // needed.
            },
            Keyframes(ref lock) => {
                let keyframes_rule = lock.read_with(guard);
                if device.animation_name_may_be_referenced(&keyframes_rule.name) {
                    debug!(
                        " > Found @keyframes rule potentially referenced \
                         from the page, marking the whole tree invalid."
                    );
                    self.fully_invalid = true;
                } else {
                    // Do nothing, this animation can't affect the style of
                    // existing elements.
                }
            },
            CounterStyle(..) | Page(..) | Viewport(..) | FontFeatureValues(..) => {
                debug!(
                    " > Found unsupported rule, marking the whole subtree \
                     invalid."
                );

                // TODO(emilio): Can we do better here?
                //
                // At least in `@page`, we could check the relevant media, I
                // guess.
                self.fully_invalid = true;
            },
        }
    }
}
