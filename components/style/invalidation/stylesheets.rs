/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A collection of invalidations due to changes in which stylesheets affect a
//! document.

#![deny(unsafe_code)]

use crate::dom::{TDocument, TElement, TNode};
use crate::invalidation::element::element_wrapper::{ElementSnapshot, ElementWrapper};
use crate::invalidation::element::restyle_hints::RestyleHint;
use crate::media_queries::Device;
use crate::selector_parser::{SelectorImpl, Snapshot, SnapshotMap};
use crate::shared_lock::SharedRwLockReadGuard;
use crate::stylesheets::{CssRule, StylesheetInDocument};
use crate::Atom;
use crate::CaseSensitivityExt;
use crate::LocalName as SelectorLocalName;
use fxhash::FxHashSet;
use selectors::attr::CaseSensitivity;
use selectors::parser::{Component, LocalName, Selector};

/// A style sheet invalidation represents a kind of element or subtree that may
/// need to be restyled. Whether it represents a whole subtree or just a single
/// element is determined by whether the invalidation is stored in the
/// StylesheetInvalidationSet's invalid_scopes or invalid_elements table.
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

    fn matches<E>(
        &self,
        element: E,
        snapshot: Option<&Snapshot>,
        case_sensitivity: CaseSensitivity,
    ) -> bool
    where
        E: TElement,
    {
        match *self {
            Invalidation::Class(ref class) => {
                if element.has_class(class, case_sensitivity) {
                    return true;
                }

                if let Some(snapshot) = snapshot {
                    if snapshot.has_class(class, case_sensitivity) {
                        return true;
                    }
                }
            },
            Invalidation::ID(ref id) => {
                if let Some(ref element_id) = element.id() {
                    if case_sensitivity.eq_atom(element_id, id) {
                        return true;
                    }
                }

                if let Some(snapshot) = snapshot {
                    if let Some(ref old_id) = snapshot.id_attr() {
                        if case_sensitivity.eq_atom(old_id, id) {
                            return true;
                        }
                    }
                }
            },
            Invalidation::LocalName {
                ref name,
                ref lower_name,
            } => {
                // This could look at the quirks mode of the document, instead
                // of testing against both names, but it's probably not worth
                // it.
                let local_name = element.local_name();
                return *local_name == **name || *local_name == **lower_name;
            },
        }

        false
    }
}

/// A set of invalidations due to stylesheet additions.
///
/// TODO(emilio): We might be able to do the same analysis for media query
/// changes too (or even selector changes?).
#[derive(MallocSizeOf)]
pub struct StylesheetInvalidationSet {
    /// The subtrees we know we have to restyle so far.
    invalid_scopes: FxHashSet<Invalidation>,
    /// The elements we know we have to restyle so far.
    invalid_elements: FxHashSet<Invalidation>,
    /// Whether the whole document should be restyled.
    fully_invalid: bool,
}

impl StylesheetInvalidationSet {
    /// Create an empty `StylesheetInvalidationSet`.
    pub fn new() -> Self {
        Self {
            invalid_scopes: FxHashSet::default(),
            invalid_elements: FxHashSet::default(),
            fully_invalid: false,
        }
    }

    /// Mark the DOM tree styles' as fully invalid.
    pub fn invalidate_fully(&mut self) {
        debug!("StylesheetInvalidationSet::invalidate_fully");
        self.invalid_scopes.clear();
        self.invalid_elements.clear();
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

        for rule in stylesheet.effective_rules(device, guard) {
            self.collect_invalidations_for_rule(rule, guard, device);
            if self.fully_invalid {
                self.invalid_scopes.clear();
                self.invalid_elements.clear();
                break;
            }
        }

        debug!(
            " > resulting subtree invalidations: {:?}",
            self.invalid_scopes
        );
        debug!(
            " > resulting self invalidations: {:?}",
            self.invalid_elements
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

    /// Clears the invalidation set without processing.
    pub fn clear(&mut self) {
        self.invalid_scopes.clear();
        self.invalid_elements.clear();
        self.fully_invalid = false;
    }

    fn process_invalidations<E>(&self, element: E, snapshots: Option<&SnapshotMap>) -> bool
    where
        E: TElement,
    {
        debug!(
            "Stylist::process_invalidations({:?}, {:?}, {:?})",
            element, self.invalid_scopes, self.invalid_elements,
        );

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

        if self.invalid_scopes.is_empty() && self.invalid_elements.is_empty() {
            debug!("process_invalidations: empty invalidation set");
            return false;
        }

        let case_sensitivity = element
            .as_node()
            .owner_doc()
            .quirks_mode()
            .classes_and_ids_case_sensitivity();
        self.process_invalidations_in_subtree(element, snapshots, case_sensitivity)
    }

    /// Process style invalidations in a given subtree. This traverses the
    /// subtree looking for elements that match the invalidations in
    /// invalid_scopes and invalid_elements.
    ///
    /// Returns whether it invalidated at least one element's style.
    #[allow(unsafe_code)]
    fn process_invalidations_in_subtree<E>(
        &self,
        element: E,
        snapshots: Option<&SnapshotMap>,
        case_sensitivity: CaseSensitivity,
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
        for invalidation in &self.invalid_scopes {
            if invalidation.matches(element, snapshot, case_sensitivity) {
                debug!(
                    "process_invalidations_in_subtree: {:?} matched subtree {:?}",
                    element, invalidation
                );
                data.hint.insert(RestyleHint::restyle_subtree());
                return true;
            }
        }

        let mut self_invalid = false;

        if !data.hint.contains(RestyleHint::RESTYLE_SELF) {
            for invalidation in &self.invalid_elements {
                if invalidation.matches(element, snapshot, case_sensitivity) {
                    debug!(
                        "process_invalidations_in_subtree: {:?} matched self {:?}",
                        element, invalidation
                    );
                    data.hint.insert(RestyleHint::RESTYLE_SELF);
                    self_invalid = true;
                    break;
                }
            }
        }

        let mut any_children_invalid = false;

        for child in element.traversal_children() {
            let child = match child.as_element() {
                Some(e) => e,
                None => continue,
            };

            any_children_invalid |=
                self.process_invalidations_in_subtree(child, snapshots, case_sensitivity);
        }

        if any_children_invalid {
            debug!(
                "Children of {:?} changed, setting dirty descendants",
                element
            );
            unsafe { element.set_dirty_descendants() }
        }

        return self_invalid || any_children_invalid;
    }

    fn scan_component(
        component: &Component<SelectorImpl>,
        invalidation: &mut Option<Invalidation>,
    ) {
        match *component {
            Component::LocalName(LocalName {
                ref name,
                ref lower_name,
            }) => {
                if invalidation.as_ref().map_or(true, |s| !s.is_id_or_class()) {
                    *invalidation = Some(Invalidation::LocalName {
                        name: name.clone(),
                        lower_name: lower_name.clone(),
                    });
                }
            },
            Component::Class(ref class) => {
                if invalidation.as_ref().map_or(true, |s| !s.is_id()) {
                    *invalidation = Some(Invalidation::Class(class.clone()));
                }
            },
            Component::ID(ref id) => {
                if invalidation.is_none() {
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
    fn collect_invalidations(&mut self, selector: &Selector<SelectorImpl>) {
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
            self.invalid_scopes.insert(s);
        } else if let Some(s) = element_invalidation {
            debug!(" > Found element invalidation: {:?}", s);
            self.invalid_elements.insert(s);
        } else {
            // The selector was of a form that we can't handle. Any element
            // could match it, so let's just bail out.
            debug!(" > Can't handle selector, marking fully invalid");
            self.fully_invalid = true;
        }
    }

    /// Collects invalidations for a given CSS rule.
    fn collect_invalidations_for_rule(
        &mut self,
        rule: &CssRule,
        guard: &SharedRwLockReadGuard,
        device: &Device,
    ) {
        use crate::stylesheets::CssRule::*;
        debug!("StylesheetInvalidationSet::collect_invalidations_for_rule");
        debug_assert!(!self.fully_invalid, "Not worth to be here!");

        match *rule {
            Style(ref lock) => {
                let style_rule = lock.read_with(guard);
                for selector in &style_rule.selectors.0 {
                    self.collect_invalidations(selector);
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
