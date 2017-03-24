/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Restyle hints: an optimization to avoid unnecessarily matching selectors.

#![deny(missing_docs)]

use Atom;
use dom::TElement;
use element_state::*;
#[cfg(feature = "gecko")]
use gecko_bindings::structs::nsRestyleHint;
#[cfg(feature = "servo")]
use heapsize::HeapSizeOf;
use selector_parser::{AttrValue, NonTSPseudoClass, Snapshot, SelectorImpl};
use selectors::{Element, MatchAttr};
use selectors::matching::{ElementSelectorFlags, StyleRelations};
use selectors::matching::matches_complex_selector;
use selectors::parser::{AttrSelector, Combinator, ComplexSelector, SimpleSelector};
use std::clone::Clone;
use std::sync::Arc;

bitflags! {
    /// When the ElementState of an element (like IN_HOVER_STATE) changes,
    /// certain pseudo-classes (like :hover) may require us to restyle that
    /// element, its siblings, and/or its descendants. Similarly, when various
    /// attributes of an element change, we may also need to restyle things with
    /// id, class, and attribute selectors. Doing this conservatively is
    /// expensive, and so we use RestyleHints to short-circuit work we know is
    /// unnecessary.
    ///
    /// Note that the bit values here must be kept in sync with the Gecko
    /// nsRestyleHint values.  If you add more bits with matching values,
    /// please add assertions to assert_restyle_hints_match below.
    pub flags RestyleHint: u32 {
        /// Rerun selector matching on the element.
        const RESTYLE_SELF = 0x01,

        /// Rerun selector matching on all of the element's descendants.
        ///
        /// NB: In Gecko, we have RESTYLE_SUBTREE which is inclusive of self,
        /// but heycam isn't aware of a good reason for that.
        const RESTYLE_DESCENDANTS = 0x04,

        /// Rerun selector matching on all later siblings of the element and all
        /// of their descendants.
        const RESTYLE_LATER_SIBLINGS = 0x08,

        /// Don't re-run selector-matching on the element, only the style
        /// attribute has changed, and this change didn't have any other
        /// dependencies.
        const RESTYLE_STYLE_ATTRIBUTE = 0x40,
    }
}

/// Asserts that all RestyleHint flags have a matching nsRestyleHint value.
#[cfg(feature = "gecko")]
#[inline]
pub fn assert_restyle_hints_match() {
    use gecko_bindings::structs;

    macro_rules! check_restyle_hints {
        ( $( $a:ident => $b:ident ),*, ) => {
            if cfg!(debug_assertions) {
                let mut hints = RestyleHint::all();
                $(
                    assert_eq!(structs::$a.0 as usize, $b.bits() as usize, stringify!($b));
                    hints.remove($b);
                )*
                assert_eq!(hints, RestyleHint::empty(), "all RestyleHint bits should have an assertion");
            }
        }
    }

    check_restyle_hints! {
        nsRestyleHint_eRestyle_Self => RESTYLE_SELF,
        // Note that eRestyle_Subtree means "self and descendants", while
        // RESTYLE_DESCENDANTS means descendants only.  The From impl
        // below handles converting eRestyle_Subtree into
        // (RESTYLE_SELF | RESTYLE_DESCENDANTS).
        nsRestyleHint_eRestyle_Subtree => RESTYLE_DESCENDANTS,
        nsRestyleHint_eRestyle_LaterSiblings => RESTYLE_LATER_SIBLINGS,
        nsRestyleHint_eRestyle_StyleAttribute => RESTYLE_STYLE_ATTRIBUTE,
    }
}

impl RestyleHint {
    /// The subset hints that affect the styling of a single element during the
    /// traversal.
    pub fn for_self() -> Self {
        RESTYLE_SELF | RESTYLE_STYLE_ATTRIBUTE
    }
}

#[cfg(feature = "gecko")]
impl From<nsRestyleHint> for RestyleHint {
    fn from(raw: nsRestyleHint) -> Self {
        use std::mem;
        let raw_bits: u32 = unsafe { mem::transmute(raw) };
        // FIXME(bholley): Finish aligning the binary representations here and
        // then .expect() the result of the checked version.
        if Self::from_bits(raw_bits).is_none() {
            warn!("stylo: dropping unsupported restyle hint bits");
        }

        let mut bits = Self::from_bits_truncate(raw_bits);

        // eRestyle_Subtree converts to (RESTYLE_SELF | RESTYLE_DESCENDANTS).
        if bits.contains(RESTYLE_DESCENDANTS) {
            bits |= RESTYLE_SELF;
        }

        bits
    }
}

#[cfg(feature = "servo")]
impl HeapSizeOf for RestyleHint {
    fn heap_size_of_children(&self) -> usize { 0 }
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
pub trait ElementSnapshot : Sized + MatchAttr<Impl=SelectorImpl> {
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
    where E: TElement,
{
    element: E,
    snapshot: Option<&'a Snapshot>,
}

impl<'a, E> ElementWrapper<'a, E>
    where E: TElement,
{
    /// Trivially constructs an `ElementWrapper` without a snapshot.
    pub fn new(el: E) -> ElementWrapper<'a, E> {
        ElementWrapper { element: el, snapshot: None }
    }

    /// Trivially constructs an `ElementWrapper` with a snapshot.
    pub fn new_with_snapshot(el: E, snapshot: &'a Snapshot) -> ElementWrapper<'a, E> {
        ElementWrapper { element: el, snapshot: Some(snapshot) }
    }
}

impl<'a, E> MatchAttr for ElementWrapper<'a, E>
    where E: TElement,
{
    type Impl = SelectorImpl;

    fn match_attr_has(&self, attr: &AttrSelector<SelectorImpl>) -> bool {
        match self.snapshot {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_has(attr),
            _   => self.element.match_attr_has(attr)
        }
    }

    fn match_attr_equals(&self,
                         attr: &AttrSelector<SelectorImpl>,
                         value: &AttrValue) -> bool {
        match self.snapshot {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_equals(attr, value),
            _   => self.element.match_attr_equals(attr, value)
        }
    }

    fn match_attr_equals_ignore_ascii_case(&self,
                                           attr: &AttrSelector<SelectorImpl>,
                                           value: &AttrValue) -> bool {
        match self.snapshot {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_equals_ignore_ascii_case(attr, value),
            _   => self.element.match_attr_equals_ignore_ascii_case(attr, value)
        }
    }

    fn match_attr_includes(&self,
                           attr: &AttrSelector<SelectorImpl>,
                           value: &AttrValue) -> bool {
        match self.snapshot {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_includes(attr, value),
            _   => self.element.match_attr_includes(attr, value)
        }
    }

    fn match_attr_dash(&self,
                       attr: &AttrSelector<SelectorImpl>,
                       value: &AttrValue) -> bool {
        match self.snapshot {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_dash(attr, value),
            _   => self.element.match_attr_dash(attr, value)
        }
    }

    fn match_attr_prefix(&self,
                         attr: &AttrSelector<SelectorImpl>,
                         value: &AttrValue) -> bool {
        match self.snapshot {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_prefix(attr, value),
            _   => self.element.match_attr_prefix(attr, value)
        }
    }

    fn match_attr_substring(&self,
                            attr: &AttrSelector<SelectorImpl>,
                            value: &AttrValue) -> bool {
        match self.snapshot {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_substring(attr, value),
            _   => self.element.match_attr_substring(attr, value)
        }
    }

    fn match_attr_suffix(&self,
                         attr: &AttrSelector<SelectorImpl>,
                         value: &AttrValue) -> bool {
        match self.snapshot {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_suffix(attr, value),
            _   => self.element.match_attr_suffix(attr, value)
        }
    }
}

impl<'a, E> Element for ElementWrapper<'a, E>
    where E: TElement,
{
    fn match_non_ts_pseudo_class<F>(&self,
                                    pseudo_class: &NonTSPseudoClass,
                                    relations: &mut StyleRelations,
                                    _: &mut F)
                                    -> bool
        where F: FnMut(&Self, ElementSelectorFlags),
    {
        let flag = SelectorImpl::pseudo_class_state_flag(pseudo_class);
        if flag.is_empty() {
            return self.element.match_non_ts_pseudo_class(pseudo_class,
                                                          relations,
                                                          &mut |_, _| {})
        }
        match self.snapshot.and_then(|s| s.state()) {
            Some(snapshot_state) => snapshot_state.contains(flag),
            None => {
                self.element.match_non_ts_pseudo_class(pseudo_class,
                                                       relations,
                                                       &mut |_, _| {})
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

    fn get_local_name(&self) -> &<Self::Impl as ::selectors::SelectorImpl>::BorrowedLocalName {
        self.element.get_local_name()
    }

    fn get_namespace(&self) -> &<Self::Impl as ::selectors::SelectorImpl>::BorrowedNamespaceUrl {
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

/// Returns the union of any `ElementState` flags for components of a
/// `ComplexSelector`.
pub fn complex_selector_to_state(sel: &ComplexSelector<SelectorImpl>) -> ElementState {
    sel.compound_selector.iter().fold(ElementState::empty(), |state, s| {
        state | selector_to_state(s)
    })
}

fn selector_to_state(sel: &SimpleSelector<SelectorImpl>) -> ElementState {
    match *sel {
        SimpleSelector::NonTSPseudoClass(ref pc) => SelectorImpl::pseudo_class_state_flag(pc),
        SimpleSelector::Negation(ref negated) => {
            negated.iter().fold(ElementState::empty(), |state, s| {
                state | complex_selector_to_state(s)
            })
        }
        _ => ElementState::empty(),
    }
}

fn is_attr_selector(sel: &SimpleSelector<SelectorImpl>) -> bool {
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
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    selector: Arc<ComplexSelector<SelectorImpl>>,
    hint: RestyleHint,
    sensitivities: Sensitivities,
}

/// A set of dependencies for a given stylist.
///
/// Note that there are measurable perf wins from storing them separately
/// depending on what kind of change they affect, and its also not a big deal to
/// do it, since the dependencies are per-document.
#[derive(Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct DependencySet {
    /// Dependencies only affected by state.
    state_deps: Vec<Dependency>,
    /// Dependencies only affected by attributes.
    attr_deps: Vec<Dependency>,
    /// Dependencies affected by both.
    common_deps: Vec<Dependency>,
}

impl DependencySet {
    fn add_dependency(&mut self, dep: Dependency) {
        let affects_attrs = dep.sensitivities.attrs;
        let affects_states = !dep.sensitivities.states.is_empty();

        if affects_attrs && affects_states {
            self.common_deps.push(dep)
        } else if affects_attrs {
            self.attr_deps.push(dep)
        } else {
            self.state_deps.push(dep)
        }
    }

    /// Create an empty `DependencySet`.
    pub fn new() -> Self {
        DependencySet {
            state_deps: vec![],
            attr_deps: vec![],
            common_deps: vec![],
        }
    }

    /// Return the total number of dependencies that this set contains.
    pub fn len(&self) -> usize {
        self.common_deps.len() + self.attr_deps.len() + self.state_deps.len()
    }

    /// Create the needed dependencies that a given selector creates, and add
    /// them to the set.
    pub fn note_selector(&mut self, selector: &Arc<ComplexSelector<SelectorImpl>>) {
        let mut cur = selector;
        let mut combinator: Option<Combinator> = None;
        loop {
            let mut sensitivities = Sensitivities::new();
            for s in &cur.compound_selector {
                sensitivities.states.insert(selector_to_state(s));
                if !sensitivities.attrs {
                    sensitivities.attrs = is_attr_selector(s);
                }

                // NOTE(emilio): I haven't thought this thoroughly, but we may
                // not need to do anything for combinators inside negations.
                //
                // Or maybe we do, and need to call note_selector recursively
                // here to account for them correctly, but keep the
                // sensitivities of the parent?
                //
                // In any case, perhaps we should just drop it, see bug 1348802.
            }
            if !sensitivities.is_empty() {
                self.add_dependency(Dependency {
                    selector: cur.clone(),
                    hint: combinator_to_restyle_hint(combinator),
                    sensitivities: sensitivities,
                });
            }

            cur = match cur.next {
                Some((ref sel, comb)) => {
                    combinator = Some(comb);
                    sel
                }
                None => break,
            }
        }
    }

    /// Clear this dependency set.
    pub fn clear(&mut self) {
        self.common_deps.clear();
        self.attr_deps.clear();
        self.state_deps.clear();
    }

    /// Compute a restyle hint given an element and a snapshot, per the rules
    /// explained in the rest of the documentation.
    pub fn compute_hint<E>(&self,
                           el: &E,
                           snapshot: &Snapshot)
                           -> RestyleHint
        where E: TElement + Clone,
    {
        let current_state = el.get_state();
        let state_changes = snapshot.state()
                                    .map_or_else(ElementState::empty, |old_state| current_state ^ old_state);
        let attrs_changed = snapshot.has_attrs();

        if state_changes.is_empty() && !attrs_changed {
            return RestyleHint::empty();
        }

        let mut hint = RestyleHint::empty();
        let snapshot_el = ElementWrapper::new_with_snapshot(el.clone(), snapshot);

        Self::compute_partial_hint(&self.common_deps, el, &snapshot_el,
                                   &state_changes, attrs_changed, &mut hint);

        if !state_changes.is_empty() {
            Self::compute_partial_hint(&self.state_deps, el, &snapshot_el,
                                       &state_changes, attrs_changed, &mut hint);
        }

        if attrs_changed {
            Self::compute_partial_hint(&self.attr_deps, el, &snapshot_el,
                                       &state_changes, attrs_changed, &mut hint);
        }

        debug!("Calculated restyle hint: {:?}. (Element={:?}, State={:?}, Snapshot={:?}, {} Deps)",
               hint, el, current_state, snapshot, self.len());
        trace!("Deps: {:?}", self);

        hint
    }

    fn compute_partial_hint<E>(deps: &[Dependency],
                               element: &E,
                               snapshot: &ElementWrapper<E>,
                               state_changes: &ElementState,
                               attrs_changed: bool,
                               hint: &mut RestyleHint)
        where E: TElement,
    {
        if hint.is_all() {
            return;
        }
        for dep in deps {
            debug_assert!((!state_changes.is_empty() && !dep.sensitivities.states.is_empty()) ||
                          (attrs_changed && dep.sensitivities.attrs),
                          "Testing a known ineffective dependency?");
            if (attrs_changed || state_changes.intersects(dep.sensitivities.states)) && !hint.intersects(dep.hint) {
                // We can ignore the selector flags, since they would have already been set during
                // original matching for any element that might change its matching behavior here.
                let matched_then =
                    matches_complex_selector(&dep.selector, snapshot, None,
                                             &mut StyleRelations::empty(),
                                             &mut |_, _| {});
                let matches_now =
                    matches_complex_selector(&dep.selector, element, None,
                                             &mut StyleRelations::empty(),
                                             &mut |_, _| {});
                if matched_then != matches_now {
                    hint.insert(dep.hint);
                }
                if hint.is_all() {
                    break;
                }
            }
        }
    }
}
