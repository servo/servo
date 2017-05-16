/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Restyle hints: an optimization to avoid unnecessarily matching selectors.

#![deny(missing_docs)]

use Atom;
use dom::TElement;
use element_state::*;
use fnv::FnvHashMap;
#[cfg(feature = "gecko")]
use gecko_bindings::structs::nsRestyleHint;
#[cfg(feature = "servo")]
use heapsize::HeapSizeOf;
use selector_parser::{AttrValue, NonTSPseudoClass, PseudoElement, SelectorImpl, Snapshot, SnapshotMap};
use selectors::{Element, MatchAttr};
use selectors::matching::{ElementSelectorFlags, StyleRelations};
use selectors::matching::matches_selector;
use selectors::parser::{AttrSelector, Combinator, Component, Selector};
use selectors::parser::{SelectorInner, SelectorMethods};
use selectors::visitor::SelectorVisitor;
use smallvec::SmallVec;
use std::borrow::Borrow;
use std::cell::Cell;
use std::clone::Clone;
use stylist::SelectorMap;

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

        /// Replace the style data coming from CSS transitions without updating
        /// any other style data. This hint is only processed in animation-only
        /// traversal which is prior to normal traversal.
        const RESTYLE_CSS_TRANSITIONS = 0x10,

        /// Replace the style data coming from CSS animations without updating
        /// any other style data. This hint is only processed in animation-only
        /// traversal which is prior to normal traversal.
        const RESTYLE_CSS_ANIMATIONS = 0x20,

        /// Don't re-run selector-matching on the element, only the style
        /// attribute has changed, and this change didn't have any other
        /// dependencies.
        const RESTYLE_STYLE_ATTRIBUTE = 0x40,

        /// Replace the style data coming from SMIL animations without updating
        /// any other style data. This hint is only processed in animation-only
        /// traversal which is prior to normal traversal.
        const RESTYLE_SMIL = 0x80,
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
        nsRestyleHint_eRestyle_CSSTransitions => RESTYLE_CSS_TRANSITIONS,
        nsRestyleHint_eRestyle_CSSAnimations => RESTYLE_CSS_ANIMATIONS,
        nsRestyleHint_eRestyle_StyleAttribute => RESTYLE_STYLE_ATTRIBUTE,
        nsRestyleHint_eRestyle_StyleAttribute_Animations => RESTYLE_SMIL,
    }
}

impl RestyleHint {
    /// The subset hints that affect the styling of a single element during the
    /// traversal.
    #[inline]
    pub fn for_self() -> Self {
        RESTYLE_SELF | RESTYLE_STYLE_ATTRIBUTE | Self::for_animations()
    }

    /// The subset hints that are used for animation restyle.
    #[inline]
    pub fn for_animations() -> Self {
        RESTYLE_SMIL | RESTYLE_CSS_ANIMATIONS | RESTYLE_CSS_TRANSITIONS
    }
}

#[cfg(feature = "gecko")]
impl From<nsRestyleHint> for RestyleHint {
    fn from(raw: nsRestyleHint) -> Self {
        let raw_bits: u32 = raw.0;

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
    cached_snapshot: Cell<Option<&'a Snapshot>>,
    snapshot_map: &'a SnapshotMap,
}

impl<'a, E> ElementWrapper<'a, E>
    where E: TElement,
{
    /// Trivially constructs an `ElementWrapper`.
    fn new(el: E, snapshot_map: &'a SnapshotMap) -> Self {
        ElementWrapper {
            element: el,
            cached_snapshot: Cell::new(None),
            snapshot_map: snapshot_map,
        }
    }

    /// Gets the snapshot associated with this element, if any.
    fn snapshot(&self) -> Option<&'a Snapshot> {
        if !self.element.has_snapshot() {
            return None;
        }

        if let Some(s) = self.cached_snapshot.get() {
            return Some(s);
        }

        let snapshot = self.snapshot_map.get(&self.element);
        debug_assert!(snapshot.is_some(), "has_snapshot lied!");

        self.cached_snapshot.set(snapshot);

        snapshot
    }

    fn state_changes(&self) -> ElementState {
        let snapshot = match self.snapshot() {
            Some(s) => s,
            None => return ElementState::empty(),
        };

        match snapshot.state() {
            Some(state) => state ^ self.element.get_state(),
            None => ElementState::empty(),
        }
    }
}

impl<'a, E> MatchAttr for ElementWrapper<'a, E>
    where E: TElement,
{
    type Impl = SelectorImpl;

    fn match_attr_has(&self, attr: &AttrSelector<SelectorImpl>) -> bool {
        match self.snapshot() {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_has(attr),
            _   => self.element.match_attr_has(attr)
        }
    }

    fn match_attr_equals(&self,
                         attr: &AttrSelector<SelectorImpl>,
                         value: &AttrValue) -> bool {
        match self.snapshot() {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_equals(attr, value),
            _   => self.element.match_attr_equals(attr, value)
        }
    }

    fn match_attr_equals_ignore_ascii_case(&self,
                                           attr: &AttrSelector<SelectorImpl>,
                                           value: &AttrValue) -> bool {
        match self.snapshot() {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_equals_ignore_ascii_case(attr, value),
            _   => self.element.match_attr_equals_ignore_ascii_case(attr, value)
        }
    }

    fn match_attr_includes(&self,
                           attr: &AttrSelector<SelectorImpl>,
                           value: &AttrValue) -> bool {
        match self.snapshot() {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_includes(attr, value),
            _   => self.element.match_attr_includes(attr, value)
        }
    }

    fn match_attr_dash(&self,
                       attr: &AttrSelector<SelectorImpl>,
                       value: &AttrValue) -> bool {
        match self.snapshot() {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_dash(attr, value),
            _   => self.element.match_attr_dash(attr, value)
        }
    }

    fn match_attr_prefix(&self,
                         attr: &AttrSelector<SelectorImpl>,
                         value: &AttrValue) -> bool {
        match self.snapshot() {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_prefix(attr, value),
            _   => self.element.match_attr_prefix(attr, value)
        }
    }

    fn match_attr_substring(&self,
                            attr: &AttrSelector<SelectorImpl>,
                            value: &AttrValue) -> bool {
        match self.snapshot() {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_substring(attr, value),
            _   => self.element.match_attr_substring(attr, value)
        }
    }

    fn match_attr_suffix(&self,
                         attr: &AttrSelector<SelectorImpl>,
                         value: &AttrValue) -> bool {
        match self.snapshot() {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.match_attr_suffix(attr, value),
            _   => self.element.match_attr_suffix(attr, value)
        }
    }
}

#[cfg(feature = "gecko")]
fn dir_selector_to_state(s: &[u16]) -> ElementState {
    // Jump through some hoops to deal with our Box<[u16]> thing.
    const LTR: [u16; 4] = [b'l' as u16, b't' as u16, b'r' as u16, 0];
    const RTL: [u16; 4] = [b'r' as u16, b't' as u16, b'l' as u16, 0];
    if LTR == *s {
        IN_LTR_STATE
    } else if RTL == *s {
        IN_RTL_STATE
    } else {
        // :dir(something-random) is a valid selector, but shouldn't
        // match anything.
        ElementState::empty()
    }
}

impl<'a, E> Element for ElementWrapper<'a, E>
    where E: TElement,
{
    fn match_non_ts_pseudo_class<F>(&self,
                                    pseudo_class: &NonTSPseudoClass,
                                    relations: &mut StyleRelations,
                                    _setter: &mut F)
                                    -> bool
        where F: FnMut(&Self, ElementSelectorFlags),
    {
        // :moz-any is quite special, because we need to keep matching as a
        // snapshot.
        #[cfg(feature = "gecko")]
        {
            use selectors::matching::matches_complex_selector;
            if let NonTSPseudoClass::MozAny(ref selectors) = *pseudo_class {
                return selectors.iter().any(|s| {
                    matches_complex_selector(s, self, relations, _setter)
                })
            }
        }

        // :dir needs special handling.  It's implemented in terms of state
        // flags, but which state flag it maps to depends on the argument to
        // :dir.  That means we can't just add its state flags to the
        // NonTSPseudoClass, because if we added all of them there, and tested
        // via intersects() here, we'd get incorrect behavior for :not(:dir())
        // cases.
        //
        // FIXME(bz): How can I set this up so once Servo adds :dir() support we
        // don't forget to update this code?
        #[cfg(feature = "gecko")]
        {
            if let NonTSPseudoClass::Dir(ref s) = *pseudo_class {
                let selector_flag = dir_selector_to_state(s);
                if selector_flag.is_empty() {
                    // :dir() with some random argument; does not match.
                    return false;
                }
                let state = match self.snapshot().and_then(|s| s.state()) {
                    Some(snapshot_state) => snapshot_state,
                    None => self.element.get_state(),
                };
                return state.contains(selector_flag);
            }
        }

        let flag = pseudo_class.state_flag();
        if flag.is_empty() {
            return self.element.match_non_ts_pseudo_class(pseudo_class,
                                                          relations,
                                                          &mut |_, _| {})
        }
        match self.snapshot().and_then(|s| s.state()) {
            Some(snapshot_state) => snapshot_state.intersects(flag),
            None => {
                self.element.match_non_ts_pseudo_class(pseudo_class,
                                                       relations,
                                                       &mut |_, _| {})
            }
        }
    }

    fn parent_element(&self) -> Option<Self> {
        self.element.parent_element()
            .map(|e| ElementWrapper::new(e, self.snapshot_map))
    }

    fn first_child_element(&self) -> Option<Self> {
        self.element.first_child_element()
            .map(|e| ElementWrapper::new(e, self.snapshot_map))
    }

    fn last_child_element(&self) -> Option<Self> {
        self.element.last_child_element()
            .map(|e| ElementWrapper::new(e, self.snapshot_map))
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        self.element.prev_sibling_element()
            .map(|e| ElementWrapper::new(e, self.snapshot_map))
    }

    fn next_sibling_element(&self) -> Option<Self> {
        self.element.next_sibling_element()
            .map(|e| ElementWrapper::new(e, self.snapshot_map))
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
        match self.snapshot() {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.id_attr(),
            _   => self.element.get_id()
        }
    }

    fn has_class(&self, name: &Atom) -> bool {
        match self.snapshot() {
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
        match self.snapshot() {
            Some(snapshot) if snapshot.has_attrs()
                => snapshot.each_class(callback),
            _   => self.element.each_class(callback)
        }
    }
}

fn selector_to_state(sel: &Component<SelectorImpl>) -> ElementState {
    match *sel {
        // FIXME(bz): How can I set this up so once Servo adds :dir() support we
        // don't forget to update this code?
        #[cfg(feature = "gecko")]
        Component::NonTSPseudoClass(NonTSPseudoClass::Dir(ref s)) => dir_selector_to_state(s),
        Component::NonTSPseudoClass(ref pc) => pc.state_flag(),
        _ => ElementState::empty(),
    }
}

fn is_attr_selector(sel: &Component<SelectorImpl>) -> bool {
    match *sel {
        Component::ID(_) |
        Component::Class(_) |
        Component::AttrExists(_) |
        Component::AttrEqual(_, _, _) |
        Component::AttrIncludes(_, _) |
        Component::AttrDashMatch(_, _) |
        Component::AttrPrefixMatch(_, _) |
        Component::AttrSubstringMatch(_, _) |
        Component::AttrSuffixMatch(_, _) => true,
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

#[derive(Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// The aspects of an selector which are sensitive.
pub struct Sensitivities {
    /// The states which are sensitive.
    pub states: ElementState,
    /// Whether attributes are sensitive.
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

    fn sensitive_to(&self, attrs: bool, states: ElementState) -> bool {
        (attrs && self.attrs) || self.states.intersects(states)
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
#[derive(Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Dependency {
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    selector: SelectorInner<SelectorImpl>,
    /// The hint associated with this dependency.
    pub hint: RestyleHint,
    /// The sensitivities associated with this dependency.
    pub sensitivities: Sensitivities,
}

impl Borrow<SelectorInner<SelectorImpl>> for Dependency {
    fn borrow(&self) -> &SelectorInner<SelectorImpl> {
        &self.selector
    }
}

/// A similar version of the above, but for pseudo-elements, which only care
/// about the full selector, and need it in order to properly track
/// pseudo-element selector state.
///
/// NOTE(emilio): We could add a `hint` and `sensitivities` field to the
/// `PseudoElementDependency` and stop posting `RESTYLE_DESCENDANTS`s hints if
/// we visited all the pseudo-elements of an element unconditionally as part of
/// the traversal.
///
/// That would allow us to stop posting `RESTYLE_DESCENDANTS` hints for dumb
/// selectors, and storing pseudo dependencies in the element dependency map.
///
/// That would allow us to avoid restyling the element itself when a selector
/// has only changed a pseudo-element's style, too.
///
/// There's no good way to do that right now though, and I think for the
/// foreseeable future we may just want to optimize that `RESTYLE_DESCENDANTS`
/// to become a `RESTYLE_PSEUDO_ELEMENTS` or something like that, in order to at
/// least not restyle the whole subtree.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
struct PseudoElementDependency {
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "defined in selectors")]
    selector: Selector<SelectorImpl>,
}

impl Borrow<SelectorInner<SelectorImpl>> for PseudoElementDependency {
    fn borrow(&self) -> &SelectorInner<SelectorImpl> {
        &self.selector.inner
    }
}

/// The following visitor visits all the simple selectors for a given complex
/// selector, taking care of :not and :any combinators, collecting whether any
/// of them is sensitive to attribute or state changes.
struct SensitivitiesVisitor {
    sensitivities: Sensitivities,
}

impl SelectorVisitor for SensitivitiesVisitor {
    type Impl = SelectorImpl;
    fn visit_simple_selector(&mut self, s: &Component<SelectorImpl>) -> bool {
        self.sensitivities.states.insert(selector_to_state(s));
        self.sensitivities.attrs |= is_attr_selector(s);
        true
    }
}

/// A set of dependencies for a given stylist.
///
/// Note that we can have many dependencies, often more than the total number
/// of selectors given that we can get multiple partial selectors for a given
/// selector. As such, we want all the usual optimizations, including the
/// SelectorMap and the bloom filter.
#[derive(Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct DependencySet {
    /// A map used for pseudo-element's dependencies.
    ///
    /// Note that pseudo-elements are somewhat special, because some of them in
    /// Gecko track state, and also because they don't do selector-matching as
    /// normal, but against their parent element.
    pseudo_dependencies: FnvHashMap<PseudoElement, SelectorMap<PseudoElementDependency>>,

    /// This is for all other normal element's selectors/selector parts.
    dependencies: SelectorMap<Dependency>,
}

impl DependencySet {
    /// Adds a selector to this `DependencySet`.
    pub fn note_selector(&mut self, selector: &Selector<SelectorImpl>) {
        let mut combinator = None;
        let mut iter = selector.inner.complex.iter();
        let mut index = 0;

        loop {
            let sequence_start = index;
            let mut visitor = SensitivitiesVisitor {
                sensitivities: Sensitivities::new()
            };

            // Visit all the simple selectors in this sequence.
            //
            // Note that this works because we can't have combinators nested
            // inside simple selectors (i.e. in :not() or :-moz-any()). If we
            // ever support that we'll need to visit complex selectors as well.
            for ss in &mut iter {
                ss.visit(&mut visitor);
                index += 1; // Account for the simple selector.
            }


            let pseudo_selector_is_state_dependent =
                sequence_start == 0 &&
                selector.pseudo_element.as_ref().map_or(false, |pseudo_selector| {
                    !pseudo_selector.state().is_empty()
                });

            if pseudo_selector_is_state_dependent {
                let pseudo_selector = selector.pseudo_element.as_ref().unwrap();
                self.pseudo_dependencies
                    .entry(pseudo_selector.pseudo_element().clone())
                    .or_insert_with(SelectorMap::new)
                    .insert(PseudoElementDependency {
                        selector: selector.clone(),
                    });
            }

            // If we found a sensitivity, add an entry in the dependency set.
            if !visitor.sensitivities.is_empty() {
                let mut hint = combinator_to_restyle_hint(combinator);

                if sequence_start == 0 && selector.pseudo_element.is_some() {
                    // FIXME(emilio): Be more granular about this. See the
                    // comment in `PseudoElementDependency` about how could this
                    // be modified in order to be more efficient and restyle
                    // less.
                    hint |= RESTYLE_DESCENDANTS;
                }

                let dep_selector = if sequence_start == 0 {
                    // Reuse the bloom hashes if this is the base selector.
                    selector.inner.clone()
                } else {
                    SelectorInner::new(selector.inner.complex.slice_from(sequence_start))
                };

                self.dependencies.insert(Dependency {
                    sensitivities: visitor.sensitivities,
                    hint: hint,
                    selector: dep_selector,
                });
            }

            combinator = iter.next_sequence();
            if combinator.is_none() {
                break;
            }

            index += 1; // Account for the combinator.
        }
    }

    /// Create an empty `DependencySet`.
    pub fn new() -> Self {
        DependencySet {
            dependencies: SelectorMap::new(),
            pseudo_dependencies: FnvHashMap::default(),
        }
    }

    /// Return the total number of dependencies that this set contains.
    pub fn len(&self) -> usize {
        self.dependencies.len() +
            self.pseudo_dependencies.values().fold(0, |acc, val| acc + val.len())
    }

    /// Clear this dependency set.
    pub fn clear(&mut self) {
        self.dependencies = SelectorMap::new();
        self.pseudo_dependencies.clear()
    }

    fn compute_pseudo_hint<E>(
        &self,
        pseudo: &E,
        pseudo_element: PseudoElement,
        snapshots: &SnapshotMap)
        -> RestyleHint
        where E: TElement,
    {
        debug!("compute_pseudo_hint: {:?}, {:?}", pseudo, pseudo_element);
        debug_assert!(pseudo.has_snapshot());

        let map = match self.pseudo_dependencies.get(&pseudo_element) {
            Some(map) => map,
            None => return RestyleHint::empty(),
        };

        // Only pseudo-element's state is relevant.
        let pseudo_state_changes =
            ElementWrapper::new(*pseudo, snapshots).state_changes();

        debug!("pseudo_state_changes: {:?}", pseudo_state_changes);
        if pseudo_state_changes.is_empty() {
            return RestyleHint::empty();
        }

        let selector_matching_target =
            pseudo.closest_non_native_anonymous_ancestor().unwrap();

        // Note that we rely on that, if the originating element changes, it'll
        // post a restyle hint that would make us redo selector matching, so we
        // don't need to care about that.
        //
        // If that ever changes, we'd need to share more code with
        // `compute_element_hint`.
        let mut hint = RestyleHint::empty();
        map.lookup(selector_matching_target, &mut |dep| {
            // If the selector didn't match before, it either doesn't match now
            // either (or it doesn't matter because our parent posted a restyle
            // for us above).
            if !matches_selector(&dep.selector.inner, &selector_matching_target,
                                 None, &mut StyleRelations::empty(),
                                 &mut |_, _| {}) {
                return true;
            }

            let pseudo_selector = dep.selector.pseudo_element.as_ref().unwrap();
            debug_assert!(!pseudo_selector.state().is_empty());

            if pseudo_selector.state().intersects(pseudo_state_changes) {
                hint = RESTYLE_SELF;
                return false;
            }

            true
        });

        hint
    }

    fn compute_element_hint<E>(
        &self,
        el: &E,
        snapshots: &SnapshotMap)
        -> RestyleHint
        where E: TElement,
    {
        debug_assert!(el.has_snapshot(), "Shouldn't be here!");

        let snapshot_el = ElementWrapper::new(el.clone(), snapshots);
        let snapshot =
            snapshot_el.snapshot().expect("has_snapshot lied so badly");

        let state_changes = snapshot_el.state_changes();
        let attrs_changed = snapshot.has_attrs();
        if state_changes.is_empty() && !attrs_changed {
            return RestyleHint::empty();
        }

        let mut hint = RestyleHint::empty();

        // Compute whether the snapshot has any different id or class attributes
        // from the element. If it does, we need to pass those to the lookup, so
        // that we get all the possible applicable selectors from the rulehash.
        let mut additional_id = None;
        let mut additional_classes = SmallVec::<[Atom; 8]>::new();
        if attrs_changed {
            let id = snapshot.id_attr();
            if id.is_some() && id != el.get_id() {
                additional_id = id;
            }

            snapshot.each_class(|c| {
                if !el.has_class(c) {
                    additional_classes.push(c.clone())
                }
            });
        }

        self.dependencies
            .lookup_with_additional(*el, additional_id, &additional_classes, &mut |dep| {
            trace!("scanning dependency: {:?}", dep);
            if !dep.sensitivities.sensitive_to(attrs_changed,
                                               state_changes) {
                trace!(" > non-sensitive");
                return true;
            }

            if hint.contains(dep.hint) {
                trace!(" > hint was already there");
                return true;
            }

            // We can ignore the selector flags, since they would have already
            // been set during original matching for any element that might
            // change its matching behavior here.
            let matched_then =
                matches_selector(&dep.selector, &snapshot_el, None,
                                 &mut StyleRelations::empty(),
                                 &mut |_, _| {});
            let matches_now =
                matches_selector(&dep.selector, el, None,
                                 &mut StyleRelations::empty(),
                                 &mut |_, _| {});
            if matched_then != matches_now {
                hint.insert(dep.hint);
            }

            !hint.is_all()
        });

        debug!("Calculated restyle hint: {:?} for {:?}. (State={:?}, {} Deps)",
               hint, el, el.get_state(), self.len());

        hint
    }


    /// Compute a restyle hint given an element and a snapshot, per the rules
    /// explained in the rest of the documentation.
    pub fn compute_hint<E>(&self,
                           el: &E,
                           snapshots: &SnapshotMap)
                           -> RestyleHint
        where E: TElement + Clone,
    {
        debug!("DependencySet::compute_hint({:?})", el);
        if let Some(pseudo) = el.implemented_pseudo_element() {
            return self.compute_pseudo_hint(el, pseudo, snapshots);
        }

        self.compute_element_hint(el, snapshots)
    }
}
