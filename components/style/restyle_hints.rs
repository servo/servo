/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Restyle hints: an optimization to avoid unnecessarily matching selectors.

#![deny(missing_docs)]

use Atom;
use LocalName;
use Namespace;
use context::{SharedStyleContext, ThreadLocalStyleContext};
use dom::TElement;
use element_state::*;
#[cfg(feature = "gecko")]
use gecko_bindings::structs::nsRestyleHint;
#[cfg(feature = "servo")]
use heapsize::HeapSizeOf;
use selector_map::{SelectorMap, SelectorMapEntry};
use selector_parser::{NonTSPseudoClass, PseudoElement, SelectorImpl, Snapshot, SnapshotMap, AttrValue};
use selectors::Element;
use selectors::attr::{AttrSelectorOperation, NamespaceConstraint};
use selectors::matching::{ElementSelectorFlags, MatchingContext, MatchingMode};
use selectors::matching::{RelevantLinkStatus, VisitedHandlingMode, matches_selector};
use selectors::parser::{AncestorHashes, Combinator, Component};
use selectors::parser::{Selector, SelectorAndHashes, SelectorIter, SelectorMethods};
use selectors::visitor::SelectorVisitor;
use smallvec::SmallVec;
use std::cell::Cell;
use std::clone::Clone;
use std::cmp;
use std::fmt;

/// When the ElementState of an element (like IN_HOVER_STATE) changes,
/// certain pseudo-classes (like :hover) may require us to restyle that
/// element, its siblings, and/or its descendants. Similarly, when various
/// attributes of an element change, we may also need to restyle things with
/// id, class, and attribute selectors. Doing this conservatively is
/// expensive, and so we use RestyleHints to short-circuit work we know is
/// unnecessary.
#[derive(Debug, Clone, PartialEq)]
pub struct RestyleHint {
    /// Depths at which selector matching must be re-run.
    match_under_self: RestyleDepths,

    /// Rerun selector matching on all later siblings of the element and all
    /// of their descendants.
    match_later_siblings: bool,

    /// Whether the current element and/or all descendants must be recascade.
    recascade: CascadeHint,

    /// Levels of the cascade whose rule nodes should be recomputed and
    /// replaced.
    pub replacements: RestyleReplacements,
}

bitflags! {
    /// Cascade levels that can be updated for an element by simply replacing
    /// their rule node.
    ///
    /// Note that the bit values here must be kept in sync with the Gecko
    /// nsRestyleHint values.  If you add more bits with matching values,
    /// please add assertions to assert_restyle_hints_match below.
    pub flags RestyleReplacements: u8 {
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

/// Eight bit wide bitfield representing depths of a DOM subtree's descendants,
/// used to represent which elements must have selector matching re-run on them.
///
/// The least significant bit indicates that selector matching must be re-run
/// for the element itself, the second least significant bit for the element's
/// children, the third its grandchildren, and so on.  If the most significant
/// bit it set, it indicates that that selector matching must be re-run for
/// elements at that depth and all of their descendants.
#[derive(Debug, Clone, Copy, PartialEq)]
struct RestyleDepths(u8);

impl RestyleDepths {
    /// Returns a `RestyleDepths` representing no element depths.
    fn empty() -> Self {
        RestyleDepths(0)
    }

    /// Returns a `RestyleDepths` representing the current element depth.
    fn for_self() -> Self {
        RestyleDepths(0x01)
    }

    /// Returns a `RestyleDepths` representing the depths of all descendants of
    /// the current element.
    fn for_descendants() -> Self {
        RestyleDepths(0xfe)
    }

    /// Returns a `RestyleDepths` representing the current element depth and the
    /// depths of all the current element's descendants.
    fn for_self_and_descendants() -> Self {
        RestyleDepths(0xff)
    }

    /// Returns a `RestyleDepths` representing the specified depth, where zero
    /// is the current element depth, one is its children's depths, etc.
    fn for_depth(depth: u32) -> Self {
        RestyleDepths(1u8 << cmp::min(depth, 7))
    }

    /// Returns whether this `RestyleDepths` represents the current element
    /// depth and the depths of all the current element's descendants.
    fn is_self_and_descendants(&self) -> bool {
        self.0 == 0xff
    }

    /// Returns whether this `RestyleDepths` includes any element depth.
    fn is_any(&self) -> bool {
        self.0 != 0
    }

    /// Returns whether this `RestyleDepths` includes the current element depth.
    fn has_self(&self) -> bool {
        (self.0 & 0x01) != 0
    }

    /// Returns a new `RestyleDepths` with all depth values represented by this
    /// `RestyleDepths` reduced by one.
    fn propagate(&self) -> Self {
        RestyleDepths((self.0 >> 1) | (self.0 & 0x80))
    }

    /// Returns a new `RestyleDepths` that represents the union of the depths
    /// from `self` and `other`.
    fn insert(&mut self, other: RestyleDepths) {
        self.0 |= other.0;
    }

    /// Returns whether this `RestyleDepths` includes all depths represented
    /// by `other`.
    fn contains(&self, other: RestyleDepths) -> bool {
        (self.0 & other.0) == other.0
    }
}

bitflags! {
    /// Flags representing whether the current element or its descendants
    /// must be recascaded.
    ///
    /// FIXME(bholley): This should eventually become more fine-grained.
    pub flags CascadeHint: u8 {
        /// Recascade the current element.
        const RECASCADE_SELF = 0x01,
        /// Recascade all descendant elements.
        const RECASCADE_DESCENDANTS = 0x02,
    }
}

impl CascadeHint {
    /// Creates a new `CascadeHint` indicating that the current element and all
    /// its descendants must be recascaded.
    fn subtree() -> CascadeHint {
        RECASCADE_SELF | RECASCADE_DESCENDANTS
    }

    /// Returns a new `CascadeHint` appropriate for children of the current
    /// element.
    fn propagate(&self) -> Self {
        if self.contains(RECASCADE_DESCENDANTS) {
            CascadeHint::subtree()
        } else {
            CascadeHint::empty()
        }
    }
}

/// Asserts that all RestyleReplacements have a matching nsRestyleHint value.
#[cfg(feature = "gecko")]
#[inline]
pub fn assert_restyle_hints_match() {
    use gecko_bindings::structs;

    macro_rules! check_restyle_hints {
        ( $( $a:ident => $b:ident ),*, ) => {
            if cfg!(debug_assertions) {
                let mut replacements = RestyleReplacements::all();
                $(
                    assert_eq!(structs::$a.0 as usize, $b.bits() as usize, stringify!($b));
                    replacements.remove($b);
                )*
                assert_eq!(replacements, RestyleReplacements::empty(),
                           "all RestyleReplacements bits should have an assertion");
            }
        }
    }

    check_restyle_hints! {
        nsRestyleHint_eRestyle_CSSTransitions => RESTYLE_CSS_TRANSITIONS,
        nsRestyleHint_eRestyle_CSSAnimations => RESTYLE_CSS_ANIMATIONS,
        nsRestyleHint_eRestyle_StyleAttribute => RESTYLE_STYLE_ATTRIBUTE,
        nsRestyleHint_eRestyle_StyleAttribute_Animations => RESTYLE_SMIL,
    }
}

impl RestyleHint {
    /// Creates a new, empty `RestyleHint`, which represents no restyling work
    /// needs to be done.
    #[inline]
    pub fn empty() -> Self {
        RestyleHint {
            match_under_self: RestyleDepths::empty(),
            match_later_siblings: false,
            recascade: CascadeHint::empty(),
            replacements: RestyleReplacements::empty(),
        }
    }

    /// Creates a new `RestyleHint` that indicates selector matching must be
    /// re-run on the element.
    #[inline]
    pub fn for_self() -> Self {
        RestyleHint {
            match_under_self: RestyleDepths::for_self(),
            match_later_siblings: false,
            recascade: CascadeHint::empty(),
            replacements: RestyleReplacements::empty(),
        }
    }

    /// Creates a new `RestyleHint` that indicates selector matching must be
    /// re-run on all of the element's descendants.
    #[inline]
    pub fn descendants() -> Self {
        RestyleHint {
            match_under_self: RestyleDepths::for_descendants(),
            match_later_siblings: false,
            recascade: CascadeHint::empty(),
            replacements: RestyleReplacements::empty(),
        }
    }

    /// Creates a new `RestyleHint` that indicates selector matching must be
    /// re-run on the descendants of element at the specified depth, where 0
    /// indicates the element itself, 1 is its children, 2 its grandchildren,
    /// etc.
    #[inline]
    pub fn descendants_at_depth(depth: u32) -> Self {
        RestyleHint {
            match_under_self: RestyleDepths::for_depth(depth),
            match_later_siblings: false,
            recascade: CascadeHint::empty(),
            replacements: RestyleReplacements::empty(),
        }
    }

    /// Creates a new `RestyleHint` that indicates selector matching must be
    /// re-run on all of the element's later siblings and their descendants.
    #[inline]
    pub fn later_siblings() -> Self {
        RestyleHint {
            match_under_self: RestyleDepths::empty(),
            match_later_siblings: true,
            recascade: CascadeHint::empty(),
            replacements: RestyleReplacements::empty(),
        }
    }

    /// Creates a new `RestyleHint` that indicates selector matching must be
    /// re-run on the element and all of its descendants.
    #[inline]
    pub fn subtree() -> Self {
        RestyleHint {
            match_under_self: RestyleDepths::for_self_and_descendants(),
            match_later_siblings: false,
            recascade: CascadeHint::empty(),
            replacements: RestyleReplacements::empty(),
        }
    }

    /// Creates a new `RestyleHint` that indicates selector matching must be
    /// re-run on the element, its descendants, its later siblings, and
    /// their descendants.
    #[inline]
    pub fn subtree_and_later_siblings() -> Self {
        RestyleHint {
            match_under_self: RestyleDepths::for_self_and_descendants(),
            match_later_siblings: true,
            recascade: CascadeHint::empty(),
            replacements: RestyleReplacements::empty(),
        }
    }

    /// Creates a new `RestyleHint` that indicates the specified rule node
    /// replacements must be performed on the element.
    #[inline]
    pub fn for_replacements(replacements: RestyleReplacements) -> Self {
        RestyleHint {
            match_under_self: RestyleDepths::empty(),
            match_later_siblings: false,
            recascade: CascadeHint::empty(),
            replacements: replacements,
        }
    }

    /// Creates a new `RestyleHint` that indicates the element must be
    /// recascaded.
    pub fn recascade_self() -> Self {
        RestyleHint {
            match_under_self: RestyleDepths::empty(),
            match_later_siblings: false,
            recascade: RECASCADE_SELF,
            replacements: RestyleReplacements::empty(),
        }
    }

    /// Returns whether this `RestyleHint` represents no needed restyle work.
    #[inline]
    pub fn is_empty(&self) -> bool {
        *self == RestyleHint::empty()
    }

    /// Returns whether this `RestyleHint` represents the maximum possible
    /// restyle work, and thus any `insert()` calls will have no effect.
    #[inline]
    pub fn is_maximum(&self) -> bool {
        self.match_under_self.is_self_and_descendants() &&
            self.match_later_siblings &&
            self.recascade.is_all() &&
            self.replacements.is_all()
    }

    /// Returns whether the hint specifies that some work must be performed on
    /// the current element.
    #[inline]
    pub fn affects_self(&self) -> bool {
        self.match_self() || self.has_recascade_self() || !self.replacements.is_empty()
    }

    /// Returns whether the hint specifies that the currently element must be
    /// recascaded.
    pub fn has_recascade_self(&self) -> bool {
        self.recascade.contains(RECASCADE_SELF)
    }

    /// Returns whether the hint specifies that later siblings must be restyled.
    #[inline]
    pub fn affects_later_siblings(&self) -> bool {
        self.match_later_siblings
    }

    /// Returns whether the hint specifies that an animation cascade level must
    /// be replaced.
    #[inline]
    pub fn has_animation_hint(&self) -> bool {
        self.replacements.intersects(RestyleReplacements::for_animations())
    }

    /// Returns whether the hint specifies some restyle work other than an
    /// animation cascade level replacement.
    #[inline]
    pub fn has_non_animation_hint(&self) -> bool {
        self.match_under_self.is_any() || self.match_later_siblings ||
            !self.recascade.is_empty() ||
            self.replacements.contains(RESTYLE_STYLE_ATTRIBUTE)
    }

    /// Returns whether the hint specifies that selector matching must be re-run
    /// for the element.
    #[inline]
    pub fn match_self(&self) -> bool {
        self.match_under_self.has_self()
    }

    /// Returns whether the hint specifies that some cascade levels must be
    /// replaced.
    #[inline]
    pub fn has_replacements(&self) -> bool {
        !self.replacements.is_empty()
    }

    /// Returns a new `RestyleHint` appropriate for children of the current
    /// element.
    #[inline]
    pub fn propagate_for_non_animation_restyle(&self) -> Self {
        RestyleHint {
            match_under_self: self.match_under_self.propagate(),
            match_later_siblings: false,
            recascade: self.recascade.propagate(),
            replacements: RestyleReplacements::empty(),
        }
    }

    /// Removes all of the animation-related hints.
    #[inline]
    pub fn remove_animation_hints(&mut self) {
        self.replacements.remove(RestyleReplacements::for_animations());

        // While RECASCADE_SELF is not animation-specific, we only ever add and
        // process it during traversal.  If we are here, removing animation
        // hints, then we are in an animation-only traversal, and we know that
        // any RECASCADE_SELF flag must have been set due to changes in
        // inherited values after restyling for animations, and thus we
        // want to remove it so that we don't later try to restyle the element
        // during a normal restyle.  (We could have separate
        // RECASCADE_SELF_NORMAL and RECASCADE_SELF_ANIMATIONS flags to make it
        // clear, but this isn't currently necessary.)
        self.recascade.remove(RECASCADE_SELF);
    }

    /// Removes the later siblings hint, and returns whether it was present.
    pub fn remove_later_siblings_hint(&mut self) -> bool {
        let later_siblings = self.match_later_siblings;
        self.match_later_siblings = false;
        later_siblings
    }

    /// Unions the specified `RestyleHint` into this one.
    #[inline]
    pub fn insert_from(&mut self, other: &Self) {
        self.match_under_self.insert(other.match_under_self);
        self.match_later_siblings |= other.match_later_siblings;
        self.recascade.insert(other.recascade);
        self.replacements.insert(other.replacements);
    }

    /// Unions the specified `RestyleHint` into this one.
    #[inline]
    pub fn insert(&mut self, other: Self) {
        // A later patch should make it worthwhile to have an insert() function
        // that consumes its argument.
        self.insert_from(&other)
    }

    /// Inserts the specified `CascadeHint`.
    #[inline]
    pub fn insert_cascade_hint(&mut self, cascade_hint: CascadeHint) {
        self.recascade.insert(cascade_hint);
    }

    /// Returns whether this `RestyleHint` represents at least as much restyle
    /// work as the specified one.
    #[inline]
    pub fn contains(&self, other: &Self) -> bool {
        self.match_under_self.contains(other.match_under_self) &&
        (self.match_later_siblings & other.match_later_siblings) == other.match_later_siblings &&
        self.recascade.contains(other.recascade) &&
        self.replacements.contains(other.replacements)
    }
}

impl RestyleReplacements {
    /// The replacements for the animation cascade levels.
    #[inline]
    pub fn for_animations() -> Self {
        RESTYLE_SMIL | RESTYLE_CSS_ANIMATIONS | RESTYLE_CSS_TRANSITIONS
    }
}

#[cfg(feature = "gecko")]
impl From<nsRestyleHint> for RestyleReplacements {
    fn from(raw: nsRestyleHint) -> Self {
        Self::from_bits_truncate(raw.0 as u8)
    }
}

#[cfg(feature = "gecko")]
impl From<nsRestyleHint> for RestyleHint {
    fn from(raw: nsRestyleHint) -> Self {
        use gecko_bindings::structs::nsRestyleHint_eRestyle_ForceDescendants as eRestyle_ForceDescendants;
        use gecko_bindings::structs::nsRestyleHint_eRestyle_LaterSiblings as eRestyle_LaterSiblings;
        use gecko_bindings::structs::nsRestyleHint_eRestyle_Self as eRestyle_Self;
        use gecko_bindings::structs::nsRestyleHint_eRestyle_SomeDescendants as eRestyle_SomeDescendants;
        use gecko_bindings::structs::nsRestyleHint_eRestyle_Subtree as eRestyle_Subtree;

        let mut match_under_self = RestyleDepths::empty();
        if (raw.0 & (eRestyle_Self.0 | eRestyle_Subtree.0)) != 0 {
            match_under_self.insert(RestyleDepths::for_self());
        }
        if (raw.0 & (eRestyle_Subtree.0 | eRestyle_SomeDescendants.0)) != 0 {
            match_under_self.insert(RestyleDepths::for_descendants());
        }

        let mut recascade = CascadeHint::empty();
        if (raw.0 & eRestyle_ForceDescendants.0) != 0 {
            recascade.insert(CascadeHint::subtree())
        }

        RestyleHint {
            match_under_self: match_under_self,
            match_later_siblings: (raw.0 & eRestyle_LaterSiblings.0) != 0,
            recascade: recascade,
            replacements: raw.into(),
        }
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
pub trait ElementSnapshot : Sized {
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

    /// The `xml:lang=""` or `lang=""` attribute value per this snapshot.
    fn lang_attr(&self) -> Option<AttrValue>;
}

#[derive(Clone)]
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

    /// Returns the value of the `xml:lang=""` (or, if appropriate, `lang=""`)
    /// attribute from this element's snapshot or the closest ancestor
    /// element snapshot with the attribute specified.
    fn get_lang(&self) -> Option<AttrValue> {
        let mut current = self.clone();
        loop {
            let lang = match self.snapshot() {
                Some(snapshot) if snapshot.has_attrs() => snapshot.lang_attr(),
                _ => current.element.lang_attr(),
            };
            if lang.is_some() {
                return lang;
            }
            match current.parent_element() {
                Some(parent) => current = parent,
                None => return None,
            }
        }
    }
}

impl<'a, E> fmt::Debug for ElementWrapper<'a, E>
    where E: TElement,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Ignore other fields for now, can change later if needed.
        self.element.fmt(f)
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
    type Impl = SelectorImpl;

    fn match_non_ts_pseudo_class<F>(&self,
                                    pseudo_class: &NonTSPseudoClass,
                                    context: &mut MatchingContext,
                                    relevant_link: &RelevantLinkStatus,
                                    _setter: &mut F)
                                    -> bool
        where F: FnMut(&Self, ElementSelectorFlags),
    {
        // Some pseudo-classes need special handling to evaluate them against
        // the snapshot.
        match *pseudo_class {
            #[cfg(feature = "gecko")]
            NonTSPseudoClass::MozAny(ref selectors) => {
                use selectors::matching::matches_complex_selector;
                return selectors.iter().any(|s| {
                    matches_complex_selector(s, 0, self, context, _setter)
                })
            }

            // :dir is implemented in terms of state flags, but which state flag
            // it maps to depends on the argument to :dir.  That means we can't
            // just add its state flags to the NonTSPseudoClass, because if we
            // added all of them there, and tested via intersects() here, we'd
            // get incorrect behavior for :not(:dir()) cases.
            //
            // FIXME(bz): How can I set this up so once Servo adds :dir()
            // support we don't forget to update this code?
            #[cfg(feature = "gecko")]
            NonTSPseudoClass::Dir(ref s) => {
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

            // For :link and :visited, we don't actually want to test the element
            // state directly.  Instead, we use the `relevant_link` to determine if
            // they match.
            NonTSPseudoClass::Link => {
                return relevant_link.is_unvisited(self, context);
            }
            NonTSPseudoClass::Visited => {
                return relevant_link.is_visited(self, context);
            }

            #[cfg(feature = "gecko")]
            NonTSPseudoClass::MozTableBorderNonzero => {
                if let Some(snapshot) = self.snapshot() {
                    if snapshot.has_other_pseudo_class_state() {
                        return snapshot.mIsTableBorderNonzero();
                    }
                }
            }

            #[cfg(feature = "gecko")]
            NonTSPseudoClass::MozBrowserFrame => {
                if let Some(snapshot) = self.snapshot() {
                    if snapshot.has_other_pseudo_class_state() {
                        return snapshot.mIsMozBrowserFrame();
                    }
                }
            }

            // :lang() needs to match using the closest ancestor xml:lang="" or
            // lang="" attribtue from snapshots.
            NonTSPseudoClass::Lang(ref lang_arg) => {
                return self.element.match_element_lang(Some(self.get_lang()), lang_arg);
            }

            _ => {}
        }

        let flag = pseudo_class.state_flag();
        if flag.is_empty() {
            return self.element.match_non_ts_pseudo_class(pseudo_class,
                                                          context,
                                                          relevant_link,
                                                          &mut |_, _| {})
        }
        match self.snapshot().and_then(|s| s.state()) {
            Some(snapshot_state) => snapshot_state.intersects(flag),
            None => {
                self.element.match_non_ts_pseudo_class(pseudo_class,
                                                       context,
                                                       relevant_link,
                                                       &mut |_, _| {})
            }
        }
    }

    fn match_pseudo_element(&self,
                            pseudo_element: &PseudoElement,
                            context: &mut MatchingContext)
                            -> bool
    {
        self.element.match_pseudo_element(pseudo_element, context)
    }

    fn is_link(&self) -> bool {
        let mut context = MatchingContext::new(MatchingMode::Normal, None);
        self.match_non_ts_pseudo_class(&NonTSPseudoClass::AnyLink,
                                       &mut context,
                                       &RelevantLinkStatus::default(),
                                       &mut |_, _| {})
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

    fn attr_matches(&self,
                    ns: &NamespaceConstraint<&Namespace>,
                    local_name: &LocalName,
                    operation: &AttrSelectorOperation<&AttrValue>)
                    -> bool {
        match self.snapshot() {
            Some(snapshot) if snapshot.has_attrs() => {
                snapshot.attr_matches(ns, local_name, operation)
            }
            _ => self.element.attr_matches(ns, local_name, operation)
        }
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

    fn pseudo_element_originating_element(&self) -> Option<Self> {
        self.element.closest_non_native_anonymous_ancestor()
            .map(|e| ElementWrapper::new(e, self.snapshot_map))
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

fn is_attr_based_selector(sel: &Component<SelectorImpl>) -> bool {
    match *sel {
        Component::ID(_) |
        Component::Class(_) |
        Component::AttributeInNoNamespaceExists { .. } |
        Component::AttributeInNoNamespace { .. } |
        Component::AttributeOther(_) => true,
        Component::NonTSPseudoClass(ref pc) => pc.is_attr_based(),
        _ => false,
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// The characteristics that a selector is sensitive to.
pub struct Sensitivities {
    /// The states which the selector is sensitive to.
    pub states: ElementState,
    /// Whether the selector is sensitive to attributes.
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
    /// The dependency selector.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    selector: Selector<SelectorImpl>,
    /// The offset into the selector that we should match on.
    selector_offset: usize,
    /// The ancestor hashes associated with the above selector at the given
    /// offset.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "No heap data")]
    hashes: AncestorHashes,
    /// The hint associated with this dependency.
    pub hint: RestyleHint,
    /// The sensitivities associated with this dependency.
    pub sensitivities: Sensitivities,
}

impl SelectorMapEntry for Dependency {
    fn selector(&self) -> SelectorIter<SelectorImpl> {
        self.selector.iter_from(self.selector_offset)
    }

    fn hashes(&self) -> &AncestorHashes {
        &self.hashes
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
        self.sensitivities.attrs |= is_attr_based_selector(s);
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
    /// This is for all other normal element's selectors/selector parts.
    dependencies: SelectorMap<Dependency>,
}

/// The data that we need to compute a given restyle hint.
pub enum HintComputationContext<'a, E: 'a>
    where E: TElement,
{
    /// The data we need to compute a restyle hint for the root of the
    /// traversal.
    ///
    /// We don't bother with the bloom filter there for multiple reasons:
    ///
    ///  * The root of the traversal uses to be the root of the document, so we
    ///    don't gain much using a bloom filter.
    ///
    ///  * The chances that a non-root-element root of the traversal has a
    ///    snapshot is quite low.
    Root,

    /// The data we need to compute a restyle hint for a child.
    ///
    /// This needs a full-blown style context in order to get the selector
    /// filters up-to-date, and the dom depth in order to insert into the filter
    /// properly if needed.
    Child {
        /// The thread-local context, that holds the bloom filter alive.
        local_context: &'a mut ThreadLocalStyleContext<E>,
        /// The dom depth of this element.
        dom_depth: usize,
    }
}

impl DependencySet {
    /// Adds a selector to this `DependencySet`.
    pub fn note_selector(&mut self, selector_and_hashes: &SelectorAndHashes<SelectorImpl>) {
        let mut combinator = None;
        let mut iter = selector_and_hashes.selector.iter();
        let mut index = 0;
        let mut child_combinators_seen = 0;
        let mut saw_descendant_combinator = false;

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

            // Keep track of how many child combinators we've encountered,
            // and whether we've encountered a descendant combinator at all.
            match combinator {
                Some(Combinator::Child) => child_combinators_seen += 1,
                Some(Combinator::Descendant) => saw_descendant_combinator = true,
                _ => {}
            }

            // If we found a sensitivity, add an entry in the dependency set.
            if !visitor.sensitivities.is_empty() {
                // Compute a RestyleHint given the current combinator and the
                // tracked number of child combinators and presence of a
                // descendant combinator.
                let hint = match combinator {
                    // NB: RestyleHint::subtree() and not
                    // RestyleHint::descendants() is needed to handle properly
                    // eager pseudos, otherwise we may leave a stale style on
                    // the parent.
                    Some(Combinator::PseudoElement) => RestyleHint::subtree(),
                    Some(Combinator::Child) if !saw_descendant_combinator => {
                        RestyleHint::descendants_at_depth(child_combinators_seen)
                    }
                    Some(Combinator::Child) |
                    Some(Combinator::Descendant) => RestyleHint::descendants(),
                    Some(Combinator::NextSibling) |
                    Some(Combinator::LaterSibling) => RestyleHint::later_siblings(),
                    None => RestyleHint::for_self(),
                };

                // Reuse the bloom hashes if this is the base selector. Otherwise,
                // rebuild them.
                let hashes = if sequence_start == 0 {
                    selector_and_hashes.hashes.clone()
                } else {
                    let seq_iter = selector_and_hashes.selector.iter_from(sequence_start);
                    AncestorHashes::from_iter(seq_iter)
                };

                self.dependencies.insert(Dependency {
                    sensitivities: visitor.sensitivities,
                    hint: hint,
                    selector: selector_and_hashes.selector.clone(),
                    selector_offset: sequence_start,
                    hashes: hashes,
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
        }
    }

    /// Return the total number of dependencies that this set contains.
    pub fn len(&self) -> usize {
        self.dependencies.len()
    }

    /// Clear this dependency set.
    pub fn clear(&mut self) {
        self.dependencies = SelectorMap::new();
    }

    /// Compute a restyle hint given an element and a snapshot, per the rules
    /// explained in the rest of the documentation.
    pub fn compute_hint<'a, E>(
        &self,
        el: &E,
        shared_context: &SharedStyleContext,
        hint_context: HintComputationContext<'a, E>)
        -> RestyleHint
        where E: TElement,
    {
        debug_assert!(el.has_snapshot(), "Shouldn't be here!");
        let snapshot_el =
            ElementWrapper::new(*el, shared_context.snapshot_map);
        let snapshot =
            snapshot_el.snapshot().expect("has_snapshot lied so badly");

        let state_changes = snapshot_el.state_changes();
        let attrs_changed = snapshot.has_attrs();
        if state_changes.is_empty() && !attrs_changed {
            return RestyleHint::empty();
        }

        let mut hint = RestyleHint::empty();

        // If we are sensitive to visitedness and the visited state changed, we
        // force a restyle here. Matching doesn't depend on the actual visited
        // state at all, so we can't look at matching results to decide what to
        // do for this case.
        if state_changes.intersects(IN_VISITED_OR_UNVISITED_STATE) {
            trace!(" > visitedness change, force subtree restyle");
            // We can't just return here because there may also be attribute
            // changes as well that imply additional hints.
            hint = RestyleHint::subtree();
        }

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

        let bloom_filter = match hint_context {
            HintComputationContext::Root => None,
            HintComputationContext::Child { mut local_context, dom_depth } => {
                local_context
                    .bloom_filter
                    .insert_parents_recovering(*el, dom_depth);
                local_context.bloom_filter.assert_complete(*el);
                Some(local_context.bloom_filter.filter())
            }
        };

        let lookup_element = if el.implemented_pseudo_element().is_some() {
            el.closest_non_native_anonymous_ancestor().unwrap()
        } else {
            *el
        };

        self.dependencies
            .lookup_with_additional(lookup_element, additional_id, &additional_classes, &mut |dep| {
            trace!("scanning dependency: {:?}", dep);

            if !dep.sensitivities.sensitive_to(attrs_changed,
                                               state_changes) {
                trace!(" > non-sensitive");
                return true;
            }

            if hint.contains(&dep.hint) {
                trace!(" > hint was already there");
                return true;
            }

            // NOTE(emilio): We can't use the bloom filter for snapshots, given
            // that arbitrary elements in the parent chain may have mutated
            // their id's/classes, which means that they won't be in the
            // filter, and as such we may fast-reject selectors incorrectly.
            //
            // We may be able to improve this if we record as we go down the
            // tree whether any parent had a snapshot, and whether those
            // snapshots were taken due to an element class/id change, but it's
            // not clear we _need_ it right now.
            let mut then_context =
                MatchingContext::new_for_visited(MatchingMode::Normal, None,
                                                 VisitedHandlingMode::AllLinksUnvisited);
            let matched_then =
                matches_selector(&dep.selector,
                                 dep.selector_offset,
                                 &dep.hashes,
                                 &snapshot_el,
                                 &mut then_context,
                                 &mut |_, _| {});
            let mut now_context =
                MatchingContext::new_for_visited(MatchingMode::Normal, bloom_filter,
                                                 VisitedHandlingMode::AllLinksUnvisited);
            let matches_now =
                matches_selector(&dep.selector,
                                 dep.selector_offset,
                                 &dep.hashes,
                                 el,
                                 &mut now_context,
                                 &mut |_, _| {});

            // Check for mismatches in both the match result and also the status
            // of whether a relevant link was found.
            if matched_then != matches_now ||
               then_context.relevant_link_found != now_context.relevant_link_found {
                hint.insert_from(&dep.hint);
                return !hint.is_maximum()
            }

            // If there is a relevant link, then we also matched in visited
            // mode.  Match again in this mode to ensure this also matches.
            // Note that we never actually match directly against the element's
            // true visited state at all, since that would expose us to timing
            // attacks.  The matching process only considers the relevant link
            // state and visited handling mode when deciding if visited
            // matches.  Instead, we are rematching here in case there is some
            // :visited selector whose matching result changed for some _other_
            // element state or attribute.
            if now_context.relevant_link_found &&
               dep.sensitivities.states.intersects(IN_VISITED_OR_UNVISITED_STATE) {
                then_context.visited_handling = VisitedHandlingMode::RelevantLinkVisited;
                let matched_then =
                    matches_selector(&dep.selector,
                                     dep.selector_offset,
                                     &dep.hashes,
                                     &snapshot_el,
                                     &mut then_context,
                                     &mut |_, _| {});
                now_context.visited_handling = VisitedHandlingMode::RelevantLinkVisited;
                let matches_now =
                    matches_selector(&dep.selector,
                                     dep.selector_offset,
                                     &dep.hashes,
                                     el,
                                     &mut now_context,
                                     &mut |_, _| {});
                if matched_then != matches_now {
                    hint.insert_from(&dep.hint);
                    return !hint.is_maximum()
                }
            }

            !hint.is_maximum()
        });

        debug!("Calculated restyle hint: {:?} for {:?}. (State={:?}, {} Deps)",
               hint, el, el.get_state(), self.len());

        hint
    }
}
