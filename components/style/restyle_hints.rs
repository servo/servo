/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use selectors::Element;
use selectors::matching::matches_compound_selector;
use selectors::parser::{AttrSelector, Combinator, CompoundSelector, SimpleSelector};
use selectors::states::*;
use std::clone::Clone;
use std::sync::Arc;
use string_cache::{Atom, Namespace};

/// When the ElementState of an element (like IN_HOVER_STATE) changes, certain
/// pseudo-classes (like :hover) may require us to restyle that element, its
/// siblings, and/or its descendants. Doing this conservatively is expensive,
/// and so we RestyleHints to short-circuit work we know is unnecessary.
///
/// NB: We should extent restyle hints to check for attribute-dependent style
/// in addition to state-dependent style (Gecko does this).


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
struct ElementWrapper<E> where E: Element {
    element: E,
    state_override: ElementState,
}

impl<'a, E> ElementWrapper<E> where E: Element {
    pub fn new(el: E) -> ElementWrapper<E> {
        ElementWrapper { element: el, state_override: ElementState::empty() }
    }

    pub fn new_with_override(el: E, state: ElementState) -> ElementWrapper<E> {
        ElementWrapper { element: el, state_override: state }
    }
}

macro_rules! overridden_state_accessors {
    ($(
        $(#[$Flag_attr: meta])*
        state $css: expr => $variant: ident / $method: ident /
        $flag: ident = $value: expr,
    )+) => { $( fn $method(&self) -> bool { self.state_override.contains($flag) } )+
    }
}

impl<E> Element for ElementWrapper<E> where E: Element {

    // Implement the state accessors on Element to use our overridden state.
    state_pseudo_classes!(overridden_state_accessors);

    fn parent_element(&self) -> Option<Self> {
        self.element.parent_element().map(|el| ElementWrapper::new(el))
    }
    fn first_child_element(&self) -> Option<Self> {
        self.element.first_child_element().map(|el| ElementWrapper::new(el))
    }
    fn last_child_element(&self) -> Option<Self> {
        self.element.last_child_element().map(|el| ElementWrapper::new(el))
    }
    fn prev_sibling_element(&self) -> Option<Self> {
        self.element.prev_sibling_element().map(|el| ElementWrapper::new(el))
    }
    fn next_sibling_element(&self) -> Option<Self> {
        self.element.next_sibling_element().map(|el| ElementWrapper::new(el))
    }
    fn is_html_element_in_html_document(&self) -> bool {
        self.element.is_html_element_in_html_document()
    }
    fn get_local_name<'b>(&'b self) -> &'b Atom {
        self.element.get_local_name()
    }
    fn get_namespace<'b>(&'b self) -> &'b Namespace {
        self.element.get_namespace()
    }
    fn get_id(&self) -> Option<Atom> {
        self.element.get_id()
    }
    fn has_class(&self, name: &Atom) -> bool {
        self.element.has_class(name)
    }
    fn match_attr<F>(&self, attr: &AttrSelector, test: F) -> bool
                    where F: Fn(&str) -> bool {
        self.element.match_attr(attr, test)
    }
    fn is_empty(&self) -> bool {
        self.element.is_empty()
    }
    fn is_root(&self) -> bool {
        self.element.is_root()
    }
    fn is_link(&self) -> bool {
        self.element.is_link()
    }
    fn is_visited_link(&self) -> bool {
        self.element.is_visited_link()
    }
    fn is_unvisited_link(&self) -> bool {
        self.element.is_unvisited_link()
    }
    fn each_class<F>(&self, callback: F) where F: FnMut(&Atom) {
        self.element.each_class(callback)
    }
}

macro_rules! gen_selector_to_state {
    ($(
        $(#[$Flag_attr: meta])*
        state $css: expr => $variant: ident / $method: ident /
        $flag: ident = $value: expr,
    )+) => {
        fn selector_to_state(sel: &SimpleSelector) -> ElementState {
            match *sel {
                $( SimpleSelector::$variant => $flag, )+
                _ => ElementState::empty(),
            }
        }
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

state_pseudo_classes!(gen_selector_to_state);

// Mapping between (partial) CompoundSelectors (and the combinator to their right)
// and the states they depend on.
//
// In general, for all selectors in all applicable stylesheets of the form:
//
// |s _ s:X _ s _ s:Y _ s|
//
// Where:
//   * Each |s| is an arbitrary simple selector.
//   * Each |s| is an arbitrary combinator (or nothing).
//   * X and Y are state-dependent pseudo-classes like :hover.
//
// We generate a StateDependency for both |s _ s:X _| and |s _ s:X _ s _ s:Y _|, even
// though those selectors may not appear on their own in any stylesheet. This allows
// us to quickly scan through the operation points of pseudo-classes and determine the
// maximum effect their associated state changes may have on the style of elements in
// the document.
#[derive(Debug)]
struct StateDependency {
    selector: Arc<CompoundSelector>,
    combinator: Option<Combinator>,
    state: ElementState,
}

#[derive(Debug)]
pub struct StateDependencySet {
    deps: Vec<StateDependency>,
}

impl StateDependencySet {
    pub fn new() -> StateDependencySet {
        StateDependencySet { deps: Vec::new() }
    }

    pub fn compute_hint<E>(&self, el: &E, current_state: ElementState, old_state: ElementState)
                          -> RestyleHint where E: Element, E: Clone {
        let mut hint = RestyleHint::empty();
        let state_changes = current_state ^ old_state;
        for dep in &self.deps {
            if state_changes.intersects(dep.state) {
                let old_el: ElementWrapper<E> = ElementWrapper::new_with_override(el.clone(), old_state);
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

    pub fn note_selector(&mut self, selector: Arc<CompoundSelector>) {
        let mut cur = selector;
        let mut combinator: Option<Combinator> = None;
        loop {
            let mut deps = ElementState::empty();
            for s in &cur.simple_selectors {
                deps.insert(selector_to_state(s));
            }
            if !deps.is_empty() {
                self.deps.push(StateDependency {
                    selector: cur.clone(),
                    combinator: combinator,
                    state: deps,
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
