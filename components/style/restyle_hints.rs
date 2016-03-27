/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use attr::{AttrIdentifier, AttrValue};
use element_state::*;
use selector_impl::SelectorImplExt;
use selectors::Element;
use selectors::matching::matches_compound_selector;
use selectors::parser::{AttrSelector, Combinator, CompoundSelector, NamespaceConstraint, SelectorImpl, SimpleSelector};
use std::clone::Clone;
use std::sync::Arc;
use string_cache::{Atom, Namespace};

/// When the ElementState of an element (like IN_HOVER_STATE) changes, certain
/// pseudo-classes (like :hover) may require us to restyle that element, its
/// siblings, and/or its descendants. Similarly, when various attributes of an
/// element change, we may also need to restyle things with id, class, and attribute
/// selectors. Doing this conservatively is expensive, and so we use RestyleHints to
/// short-circuit work we know is unnecessary.

bitflags! {
    flags RestyleHint: u8 {
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

/// In order to compute restyle hints, we perform a selector match against a list of partial
/// selectors whose rightmost simple selector may be sensitive to the thing being changed. We
/// do this matching twice, once for the element as it exists now and once for the element as it
/// existed at the time of the last restyle. If the results of the selector match differ, that means
/// that the given partial selector is sensitive to the change, and we compute a restyle hint
/// based on its combinator.
///
/// In order to run selector matching against the old element state, we generate a wrapper for
/// the element which claims to have the old state. This is the ElementWrapper logic below.
///
/// Gecko does this differently for element states, and passes a mask called mStateMask, which
/// indicates the states that need to be ignored during selector matching. This saves an ElementWrapper
/// allocation and an additional selector match call at the expense of additional complexity inside
/// the selector matching logic. This only works for boolean states though, so we still need to
/// take the ElementWrapper approach for attribute-dependent style. So we do it the same both ways for
/// now to reduce complexity, but it's worth measuring the performance impact (if any) of the
/// mStateMask approach.

#[derive(HeapSizeOf, Clone)]
pub struct ElementSnapshot {
    pub state: Option<ElementState>,
    pub attrs: Option<Vec<(AttrIdentifier, AttrValue)>>,
}

impl ElementSnapshot {
    pub fn new() -> ElementSnapshot {
        EMPTY_SNAPSHOT.clone()
    }

    // Gets an attribute matching |namespace| and |name|, if any. Panics if |attrs| is None.
    pub fn get_attr(&self, namespace: &Namespace, name: &Atom) -> Option<&AttrValue> {
        self.attrs.as_ref().unwrap().iter()
                  .find(|&&(ref ident, _)| ident.local_name == *name && ident.namespace == *namespace)
                  .map(|&(_, ref v)| v)
    }

    // Gets an attribute matching |name| if any, ignoring namespace. Panics if |attrs| is None.
    pub fn get_attr_ignore_ns(&self, name: &Atom) -> Option<&AttrValue> {
        self.attrs.as_ref().unwrap().iter()
                  .find(|&&(ref ident, _)| ident.local_name == *name)
                  .map(|&(_, ref v)| v)
    }
}

static EMPTY_SNAPSHOT: ElementSnapshot = ElementSnapshot { state: None, attrs: None };

struct ElementWrapper<'a, E>
    where E: Element,
          E::Impl: SelectorImplExt {
    element: E,
    snapshot: &'a ElementSnapshot,
}

impl<'a, E> ElementWrapper<'a, E>
    where E: Element,
          E::Impl: SelectorImplExt {
    pub fn new(el: E) -> ElementWrapper<'a, E> {
        ElementWrapper { element: el, snapshot: &EMPTY_SNAPSHOT }
    }

    pub fn new_with_snapshot(el: E, snapshot: &'a ElementSnapshot) -> ElementWrapper<'a, E> {
        ElementWrapper { element: el, snapshot: snapshot }
    }
}

impl<'a, E> Element for ElementWrapper<'a, E>
    where E: Element,
          E::Impl: SelectorImplExt {
    type Impl = E::Impl;

    fn match_non_ts_pseudo_class(&self,
                                 pseudo_class: <Self::Impl as SelectorImpl>::NonTSPseudoClass) -> bool {
        let flag = Self::Impl::pseudo_class_state_flag(&pseudo_class);
        if flag == ElementState::empty() {
            self.element.match_non_ts_pseudo_class(pseudo_class)
        } else {
            match self.snapshot.state {
                Some(s) => s.contains(flag),
                None => self.element.match_non_ts_pseudo_class(pseudo_class)
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
    fn get_local_name(&self) -> &Atom {
        self.element.get_local_name()
    }
    fn get_namespace<'b>(&self) -> &Namespace {
        self.element.get_namespace()
    }
    fn get_id(&self) -> Option<Atom> {
        match self.snapshot.attrs {
            Some(_) => self.snapshot.get_attr(&ns!(), &atom!("id")).map(|value| value.as_atom().clone()),
            None => self.element.get_id(),
        }
    }
    fn has_class(&self, name: &Atom) -> bool {
        match self.snapshot.attrs {
            Some(_) => self.snapshot.get_attr(&ns!(), &atom!("class"))
                                    .map_or(false, |v| { v.as_tokens().iter().any(|atom| atom == name) }),
            None => self.element.has_class(name),
        }
    }
    fn match_attr<F>(&self, attr: &AttrSelector, test: F) -> bool
                    where F: Fn(&str) -> bool {
        match self.snapshot.attrs {
            Some(_) => {
                let html = self.is_html_element_in_html_document();
                let local_name = if html { &attr.lower_name } else { &attr.name };
                match attr.namespace {
                    NamespaceConstraint::Specific(ref ns) => self.snapshot.get_attr(ns, local_name),
                    NamespaceConstraint::Any => self.snapshot.get_attr_ignore_ns(local_name),
                }.map_or(false, |v| test(v))
            },
            None => self.element.match_attr(attr, test)
        }
    }
    fn is_empty(&self) -> bool {
        self.element.is_empty()
    }
    fn is_root(&self) -> bool {
        self.element.is_root()
    }
    fn each_class<F>(&self, mut callback: F) where F: FnMut(&Atom) {
        match self.snapshot.attrs {
            Some(_) => {
                if let Some(v) = self.snapshot.get_attr(&ns!(), &atom!("class")) {
                    for c in v.as_tokens() { callback(c) }
                }
            }
            None => self.element.each_class(callback),
        }
    }
}

fn selector_to_state<Impl: SelectorImplExt>(sel: &SimpleSelector<Impl>) -> ElementState {
    match *sel {
        SimpleSelector::NonTSPseudoClass(ref pc) => Impl::pseudo_class_state_flag(pc),
        _ => ElementState::empty(),
    }
}

fn is_attr_selector<Impl: SelectorImpl>(sel: &SimpleSelector<Impl>) -> bool {
    match *sel {
        SimpleSelector::ID(_) |
        SimpleSelector::Class(_) |
        SimpleSelector::AttrExists(_) |
        SimpleSelector::AttrEqual(_, _, _) |
        SimpleSelector::AttrIncludes(_, _) |
        SimpleSelector::AttrDashMatch(_, _, _) |
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

#[derive(Debug, HeapSizeOf)]
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

// Mapping between (partial) CompoundSelectors (and the combinator to their right)
// and the states and attributes they depend on.
//
// In general, for all selectors in all applicable stylesheets of the form:
//
// |a _ b _ c _ d _ e|
//
// Where:
//   * |b| and |d| are simple selectors that depend on state (like :hover) or
//     attributes (like [attr...], .foo, or #foo).
//   * |a|, |c|, and |e| are arbitrary simple selectors that do not depend on
//     state or attributes.
//
// We generate a Dependency for both |a _ b:X _| and |a _ b:X _ c _ d:Y _|, even
// though those selectors may not appear on their own in any stylesheet. This allows
// us to quickly scan through the dependency sites of all style rules and determine the
// maximum effect that a given state or attribute change may have on the style of
// elements in the document.
#[derive(Debug, HeapSizeOf)]
struct Dependency<Impl: SelectorImplExt> {
    selector: Arc<CompoundSelector<Impl>>,
    combinator: Option<Combinator>,
    sensitivities: Sensitivities,
}

#[derive(Debug, HeapSizeOf)]
pub struct DependencySet<Impl: SelectorImplExt> {
    deps: Vec<Dependency<Impl>>,
}

impl<Impl: SelectorImplExt> DependencySet<Impl> {
    pub fn new() -> DependencySet<Impl> {
        DependencySet { deps: Vec::new() }
    }

    pub fn compute_hint<E>(&self, el: &E, snapshot: &ElementSnapshot, current_state: ElementState)
                          -> RestyleHint
                          where E: Element<Impl=Impl> + Clone {
        let state_changes = snapshot.state.map_or(ElementState::empty(), |old_state| current_state ^ old_state);
        let attrs_changed = snapshot.attrs.is_some();
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

    pub fn note_selector(&mut self, selector: Arc<CompoundSelector<Impl>>) {
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
