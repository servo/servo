/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A collection of invalidations due to changes in which stylesheets affect a
//! document.

#![deny(unsafe_code)]

use Atom;
use dom::{TElement, TNode};
use fnv::FnvHashSet;
use invalidation::element::restyle_hints::RestyleHint;
use selector_parser::SelectorImpl;
use selectors::attr::CaseSensitivity;
use selectors::parser::{Component, Selector};
use shared_lock::SharedRwLockReadGuard;
use stylesheets::{CssRule, Stylesheet};
use stylist::Stylist;

/// An invalidation scope represents a kind of subtree that may need to be
/// restyled.
#[derive(Debug, Hash, Eq, PartialEq)]
enum InvalidationScope {
    /// All the descendants of an element with a given id.
    ID(Atom),
    /// All the descendants of an element with a given class name.
    Class(Atom),
}

impl InvalidationScope {
    fn is_id(&self) -> bool {
        matches!(*self, InvalidationScope::ID(..))
    }

    fn matches<E>(&self, element: E) -> bool
        where E: TElement,
    {
        match *self {
            InvalidationScope::Class(ref class) => {
                element.has_class(class, CaseSensitivity::CaseSensitive)
            }
            InvalidationScope::ID(ref id) => {
                match element.get_id() {
                    Some(element_id) => element_id == *id,
                    None => false,
                }
            }
        }
    }
}

/// A set of invalidations due to stylesheet additions.
///
/// TODO(emilio): We might be able to do the same analysis for removals and
/// media query changes too?
pub struct StylesheetInvalidationSet {
    /// The style scopes we know we have to restyle so far.
    invalid_scopes: FnvHashSet<InvalidationScope>,
    /// Whether the whole document should be invalid.
    fully_invalid: bool,
}

impl StylesheetInvalidationSet {
    /// Create an empty `StylesheetInvalidationSet`.
    pub fn new() -> Self {
        Self {
            invalid_scopes: FnvHashSet::default(),
            fully_invalid: false,
        }
    }

    /// Mark the DOM tree styles' as fully invalid.
    pub fn invalidate_fully(&mut self) {
        debug!("StylesheetInvalidationSet::invalidate_fully");
        self.invalid_scopes.clear();
        self.fully_invalid = true;
    }

    /// Analyze the given stylesheet, and collect invalidations from their
    /// rules, in order to avoid doing a full restyle when we style the document
    /// next time.
    pub fn collect_invalidations_for(
        &mut self,
        stylist: &Stylist,
        stylesheet: &Stylesheet,
        guard: &SharedRwLockReadGuard)
    {
        debug!("StylesheetInvalidationSet::collect_invalidations_for");
        if self.fully_invalid {
            debug!(" > Fully invalid already");
            return;
        }

        if stylesheet.disabled() ||
           !stylesheet.is_effective_for_device(stylist.device(), guard) {
            debug!(" > Stylesheet was not effective");
            return; // Nothing to do here.
        }

        for rule in stylesheet.effective_rules(stylist.device(), guard) {
            self.collect_invalidations_for_rule(rule, guard);
            if self.fully_invalid {
                self.invalid_scopes.clear();
                break;
            }
        }

        debug!(" > resulting invalidations: {:?}", self.invalid_scopes);
        debug!(" > fully_invalid: {}", self.fully_invalid);
    }

    /// Clears the invalidation set, invalidating elements as needed if
    /// `document_element` is provided.
    pub fn flush<E>(&mut self, document_element: Option<E>)
        where E: TElement,
    {
        if let Some(e) = document_element {
            self.process_invalidations(e);
        }
        self.invalid_scopes.clear();
        self.fully_invalid = false;
    }

    fn process_invalidations<E>(&self, element: E) -> bool
        where E: TElement,
    {
        {
            let mut data = match element.mutate_data() {
                Some(data) => data,
                None => return false,
            };

            if self.fully_invalid {
                debug!("process_invalidations: fully_invalid({:?})",
                       element);
                data.restyle.hint.insert(RestyleHint::restyle_subtree());
                return true;
            }
        }

        if self.invalid_scopes.is_empty() {
            debug!("process_invalidations: empty invalidation set");
            return false;
        }

        self.process_invalidations_in_subtree(element)
    }

    /// Process style invalidations in a given subtree, that is, look for all
    /// the relevant scopes in the subtree, and mark as dirty only the relevant
    /// ones.
    ///
    /// Returns whether it invalidated at least one element's style.
    #[allow(unsafe_code)]
    fn process_invalidations_in_subtree<E>(&self, element: E) -> bool
        where E: TElement,
    {
        let mut data = match element.mutate_data() {
            Some(data) => data,
            None => return false,
        };

        if !data.has_styles() {
            return false;
        }

        if data.restyle.hint.contains_subtree() {
            debug!("process_invalidations_in_subtree: {:?} was already invalid",
                   element);
            return false;
        }

        for scope in &self.invalid_scopes {
            if scope.matches(element) {
                debug!("process_invalidations_in_subtree: {:?} matched {:?}",
                       element, scope);
                data.restyle.hint.insert(RestyleHint::restyle_subtree());
                return true;
            }
        }


        let mut any_children_invalid = false;

        for child in element.as_node().traversal_children() {
            let child = match child.as_element() {
                Some(e) => e,
                None => continue,
            };

            any_children_invalid |= self.process_invalidations_in_subtree(child);
        }

        if any_children_invalid {
            debug!("Children of {:?} changed, setting dirty descendants",
                   element);
            unsafe { element.set_dirty_descendants() }
        }

        return any_children_invalid
    }

    fn scan_component(
        component: &Component<SelectorImpl>,
        scope: &mut Option<InvalidationScope>)
    {
        match *component {
            Component::Class(ref class) => {
                if scope.as_ref().map_or(true, |s| !s.is_id()) {
                    *scope = Some(InvalidationScope::Class(class.clone()));
                }
            }
            Component::ID(ref id) => {
                if scope.is_none() {
                    *scope = Some(InvalidationScope::ID(id.clone()));
                }
            }
            _ => {
                // Ignore everything else, at least for now.
            }
        }
    }

    /// Collect a style scopes for a given selector.
    ///
    /// We look at the outermost class or id selector to the left of an ancestor
    /// combinator, in order to restyle only a given subtree.
    ///
    /// We prefer id scopes to class scopes, and outermost scopes to innermost
    /// scopes (to reduce the amount of traversal we need to do).
    fn collect_scopes(&mut self, selector: &Selector<SelectorImpl>) {
        debug!("StylesheetInvalidationSet::collect_scopes({:?})", selector);

        let mut scope: Option<InvalidationScope> = None;

        let mut scan = true;
        let mut iter = selector.iter();

        loop {
            for component in &mut iter {
                if scan {
                    Self::scan_component(component, &mut scope);
                }
            }
            match iter.next_sequence() {
                None => break,
                Some(combinator) => {
                    scan = combinator.is_ancestor();
                }
            }
        }

        match scope {
            Some(s) => {
                debug!(" > Found scope: {:?}", s);
                self.invalid_scopes.insert(s);
            }
            None => {
                debug!(" > Scope not found");

                // If we didn't find a scope, any element could match this, so
                // let's just bail out.
                self.fully_invalid = true;
            }
        }
    }

    /// Collects invalidations for a given CSS rule.
    fn collect_invalidations_for_rule(
        &mut self,
        rule: &CssRule,
        guard: &SharedRwLockReadGuard)
    {
        use stylesheets::CssRule::*;
        debug!("StylesheetInvalidationSet::collect_invalidations_for_rule");
        debug_assert!(!self.fully_invalid, "Not worth to be here!");

        match *rule {
            Style(ref lock) => {
                let style_rule = lock.read_with(guard);
                for selector_and_hashes in &style_rule.selectors.0 {
                    self.collect_scopes(&selector_and_hashes.selector);
                    if self.fully_invalid {
                        return;
                    }
                }
            }
            Document(..) |
            Namespace(..) |
            Import(..) |
            Media(..) |
            Supports(..) => {
                // Do nothing, relevant nested rules are visited as part of the
                // iteration.
            }
            FontFace(..) |
            CounterStyle(..) |
            Keyframes(..) |
            Page(..) |
            Viewport(..) => {
                debug!(" > Found unsupported rule, marking the whole subtree \
                       invalid.");

                // TODO(emilio): Can we do better here?
                //
                // At least in `@page`, we could check the relevant media, I
                // guess.
                self.fully_invalid = true;
            }
        }
    }
}
