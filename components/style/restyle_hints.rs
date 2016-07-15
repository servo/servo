/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Restyle hints: an optimization to avoid unnecessarily matching selectors.

use element_state::*;
use selector_impl::{ElementExt, SelectorImplExt, TheSelectorImpl, AttrString};
use selectors::matching::matches_compound_selector;
use selectors::parser::{AttrSelector, Combinator, CompoundSelector, SelectorImpl, SimpleSelector};
use selectors::{Element, MatchAttr};
use std::clone::Clone;
use std::sync::Arc;
use string_cache::{Atom, BorrowedAtom, BorrowedNamespace};

/// When the ElementState of an element (like IN_HOVER_STATE) changes, certain
/// pseudo-classes (like :hover) may require us to restyle that element, its
/// siblings, and/or its descendants. Similarly, when various attributes of an
/// element change, we may also need to restyle things with id, class, and
/// attribute selectors. Doing this conservatively is expensive, and so we use
/// RestyleHints to short-circuit work we know is unnecessary.
bitflags! {
    pub flags RestyleHint: u8 {
        #[doc = "Rerun selector matching on the element."]
        const RESTYLE_SELF = 0x01,
        #[doc = "Rerun selector matching on all of the element's descendants."]
        // NB: In Gecko, we have RESTYLE_SUBTREE which is inclusive of self, but heycam isn't aware
        // of a good reason for that.
        const RESTYLE_DESCENDANTS = 0x02,
        #[doc = "Rerun selector matching on all later siblings of the element and all of their descendants."]
        const RESTYLE_LATER_SIBLINGS = 0x04,
    }
}

/// In order to compute restyle hints, we perform a selector match against a
/// list of partial selectors whose rightmost simple selector may be sensitive
/// to the thing being changed. We do this matching twice, once for the element
/// as it exists now and once for the element as it existed at the time of the
/// last restyle. If the results of the selector match differ, that means that
/// the given partial selector is sensitive to the change, and we compute a
/// restyle hint based on its combinator.
///
/// In order to run selector matching against the old element state, we generate
/// a wrapper for the element which claims to have the old state. This is the
/// ElementWrapper logic below.
///
/// Gecko does this differently for element states, and passes a mask called
/// mStateMask, which indicates the states that need to be ignored during
/// selector matching. This saves an ElementWrapper allocation and an additional
/// selector match call at the expense of additional complexity inside the
/// selector matching logic. This only works for boolean states though, so we
/// still need to take the ElementWrapper approach for attribute-dependent
/// style. So we do it the same both ways for now to reduce complexity, but it's
/// worth measuring the performance impact (if any) of the mStateMask approach.
pub trait ElementSnapshot : Sized + MatchAttr {
    /// The state of the snapshot, if any.
    fn state(&self) -> Option<ElementState>;

    /// If this snapshot contains attribute information.
    fn has_attrs(&self) -> bool;

    /// The ID attribute per this snapshot. Should only be called if
    /// `has_attrs()` returns true.
    fn id_attr(&self) -> Option<Atom>;

    /// Whether this snapshot contains the class `name`. Should only be called
    /// if `has_attrs()` returns true.
    fn has_class(&self, name: &Atom) -> bool;

    /// A callback that should be called for each class of the snapshot. Should
    /// only be called if `has_attrs()` returns true.
    fn each_class<F>(&self, F)
        where F: FnMut(&Atom);
}

struct ElementWrapper<'a, E>
    where E: ElementExt
{
    element: E,
    snapshot: Option<&'a E::Snapshot>,
}

impl<'a, E> ElementWrapper<'a, E>
    where E: ElementExt
{
    pub fn new(el: E) -> ElementWrapper<'a, E> {
        ElementWrapper { element: el, snapshot: None }
    }

    pub fn new_with_snapshot(el: E, snapshot: &'a E::Snapshot) -> ElementWrapper<'a, E> {
        ElementWrapper { element: el, snapshot: Some(snapshot) }
    }
}

impl<'a, E> MatchAttr for ElementWrapper<'a, E>
    where E: ElementExt<AttrString=AttrString>,
{
    type AttrString = E::AttrString;

    fn match_attr_has(&self, attr: &AttrSelector) -> bool {
        match self.snapshot {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_has(attr),
            _   => self.element.match_attr_has(attr)
        }
    }

    fn match_attr_equals(&self,
                         attr: &AttrSelector,
                         value: &Self::AttrString) -> bool {
        match self.snapshot {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_equals(attr, value),
            _   => self.element.match_attr_equals(attr, value)
        }
    }

    fn match_attr_equals_ignore_ascii_case(&self,
                                           attr: &AttrSelector,
                                           value: &Self::AttrString) -> bool {
        match self.snapshot {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_equals_ignore_ascii_case(attr, value),
            _   => self.element.match_attr_equals_ignore_ascii_case(attr, value)
        }
    }

    fn match_attr_includes(&self,
                           attr: &AttrSelector,
                           value: &Self::AttrString) -> bool {
        match self.snapshot {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_includes(attr, value),
            _   => self.element.match_attr_includes(attr, value)
        }
    }

    fn match_attr_dash(&self,
                       attr: &AttrSelector,
                       value: &Self::AttrString) -> bool {
        match self.snapshot {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_dash(attr, value),
            _   => self.element.match_attr_dash(attr, value)
        }
    }

    fn match_attr_prefix(&self,
                         attr: &AttrSelector,
                         value: &Self::AttrString) -> bool {
        match self.snapshot {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_prefix(attr, value),
            _   => self.element.match_attr_prefix(attr, value)
        }
    }

    fn match_attr_substring(&self,
                            attr: &AttrSelector,
                            value: &Self::AttrString) -> bool {
        match self.snapshot {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_substring(attr, value),
            _   => self.element.match_attr_substring(attr, value)
        }
    }

    fn match_attr_suffix(&self,
                         attr: &AttrSelector,
                         value: &Self::AttrString) -> bool {
        match self.snapshot {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_suffix(attr, value),
            _   => self.element.match_attr_suffix(attr, value)
        }
    }
}

impl<'a, E> Element for ElementWrapper<'a, E>
    where E: ElementExt<AttrString=AttrString>,
          E::Impl: SelectorImplExt<AttrString=AttrString> {
    type Impl = E::Impl;

    fn match_non_ts_pseudo_class(&self,
                                 pseudo_class: <Self::Impl as SelectorImpl>::NonTSPseudoClass) -> bool {
        let flag = Self::Impl::pseudo_class_state_flag(&pseudo_class);
        if flag == ElementState::empty() {
            self.element.match_non_ts_pseudo_class(pseudo_class)
        } else {
            match self.snapshot.and_then(|s| s.state()) {
                Some(snapshot_state) => snapshot_state.contains(flag),
                _   => self.element.match_non_ts_pseudo_class(pseudo_class)
            }
        }
    }

    fn parent_element(&self) -> Option<Self> {
        self.element.parent_element().map(ElementWrapper::new)
    }

    fn first_child_element(&self) -> Option<Self> {
        self.element.first_child_element().map(ElementWrapper::new)
    }

    fn last_child_element(&self) -> Option<Self> {
        self.element.last_child_element().map(ElementWrapper::new)
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        self.element.prev_sibling_element().map(ElementWrapper::new)
    }

    fn next_sibling_element(&self) -> Option<Self> {
        self.element.next_sibling_element().map(ElementWrapper::new)
    }

    fn is_html_element_in_html_document(&self) -> bool {
        self.element.is_html_element_in_html_document()
    }

    fn get_local_name(&self) -> BorrowedAtom {
        self.element.get_local_name()
    }

    fn get_namespace(&self) -> BorrowedNamespace {
        self.element.get_namespace()
    }

    fn get_id(&self) -> Option<Atom> {
        match self.snapshot {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.id_attr(),
            _   => self.element.get_id()
        }
    }

    fn has_class(&self, name: &Atom) -> bool {
        match self.snapshot {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.has_class(name),
            _   => self.element.has_class(name)
        }
    }

    fn is_empty(&self) -> bool {
        self.element.is_empty()
    }

    fn is_root(&self) -> bool {
        self.element.is_root()
    }

    fn each_class<F>(&self, callback: F)
        where F: FnMut(&Atom) {
        match self.snapshot {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.each_class(callback),
            _   => self.element.each_class(callback)
        }
    }
}

fn selector_to_state(sel: &SimpleSelector<TheSelectorImpl>) -> ElementState {
    match *sel {
        SimpleSelector::NonTSPseudoClass(ref pc) => TheSelectorImpl::pseudo_class_state_flag(pc),
        _ => ElementState::empty(),
    }
}

fn is_attr_selector(sel: &SimpleSelector<TheSelectorImpl>) -> bool {
    match *sel {
        SimpleSelector::ID(_) |
        SimpleSelector::Class(_) |
        SimpleSelector::AttrExists(_) |
        SimpleSelector::AttrEqual(_, _, _) |
        SimpleSelector::AttrIncludes(_, _) |
        SimpleSelector::AttrDashMatch(_, _) |
        SimpleSelector::AttrPrefixMatch(_, _) |
        SimpleSelector::AttrSubstringMatch(_, _) |
        SimpleSelector::AttrSuffixMatch(_, _) => true,
        _ => false,
    }
}

fn combinator_to_restyle_hint(combinator: Option<Combinator>) -> RestyleHint {
    match combinator {
        None => RESTYLE_SELF,
        Some(c) => match c {
            Combinator::Child => RESTYLE_DESCENDANTS,
            Combinator::Descendant => RESTYLE_DESCENDANTS,
            Combinator::NextSibling => RESTYLE_LATER_SIBLINGS,
            Combinator::LaterSibling => RESTYLE_LATER_SIBLINGS,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
struct Sensitivities {
    pub states: ElementState,
    pub attrs: bool,
}

impl Sensitivities {
    fn is_empty(&self) -> bool {
        self.states.is_empty() && !self.attrs
    }

    fn new() -> Sensitivities {
        Sensitivities {
            states: ElementState::empty(),
            attrs: false,
        }
    }
}

/// Mapping between (partial) CompoundSelectors (and the combinator to their
/// right) and the states and attributes they depend on.
///
/// In general, for all selectors in all applicable stylesheets of the form:
///
/// |a _ b _ c _ d _ e|
///
/// Where:
///   * |b| and |d| are simple selectors that depend on state (like :hover) or
///     attributes (like [attr...], .foo, or #foo).
///   * |a|, |c|, and |e| are arbitrary simple selectors that do not depend on
///     state or attributes.
///
/// We generate a Dependency for both |a _ b:X _| and |a _ b:X _ c _ d:Y _|,
/// even though those selectors may not appear on their own in any stylesheet.
/// This allows us to quickly scan through the dependency sites of all style
/// rules and determine the maximum effect that a given state or attribute
/// change may have on the style of elements in the document.
#[derive(Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
struct Dependency {
    selector: Arc<CompoundSelector<TheSelectorImpl>>,
    combinator: Option<Combinator>,
    sensitivities: Sensitivities,
}

#[derive(Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct DependencySet {
    deps: Vec<Dependency>,
}

impl DependencySet {
    pub fn new() -> Self {
        DependencySet { deps: Vec::new() }
    }

    pub fn note_selector(&mut self, selector: Arc<CompoundSelector<TheSelectorImpl>>) {
        let mut cur = selector;
        let mut combinator: Option<Combinator> = None;
        loop {
            let mut sensitivities = Sensitivities::new();
            for s in &cur.simple_selectors {
                sensitivities.states.insert(selector_to_state(s));
                if !sensitivities.attrs {
                    sensitivities.attrs = is_attr_selector(s);
                }
            }
            if !sensitivities.is_empty() {
                self.deps.push(Dependency {
                    selector: cur.clone(),
                    combinator: combinator,
                    sensitivities: sensitivities,
                });
            }

            cur = match cur.next {
                Some((ref sel, comb)) => {
                    combinator = Some(comb);
                    sel.clone()
                }
                None => break,
            }
        }
    }

    pub fn clear(&mut self) {
        self.deps.clear();
    }
}

impl DependencySet {
    pub fn compute_hint<E>(&self, el: &E,
                           snapshot: &E::Snapshot,
                           current_state: ElementState)
                           -> RestyleHint
    where E: ElementExt + Clone
    {
        let state_changes = snapshot.state().map_or_else(ElementState::empty, |old_state| current_state ^ old_state);
        let attrs_changed = snapshot.has_attrs();
        let mut hint = RestyleHint::empty();
        for dep in &self.deps {
            if state_changes.intersects(dep.sensitivities.states) || (attrs_changed && dep.sensitivities.attrs) {
                let old_el: ElementWrapper<E> = ElementWrapper::new_with_snapshot(el.clone(), snapshot);
                let matched_then = matches_compound_selector(&*dep.selector, &old_el, None, &mut false);
                let matches_now = matches_compound_selector(&*dep.selector, el, None, &mut false);
                if matched_then != matches_now {
                    hint.insert(combinator_to_restyle_hint(dep.combinator));
                    if hint.is_all() {
                        break
                    }
                }
            }
        }
        hint
    }
}
