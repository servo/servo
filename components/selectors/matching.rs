/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use attr::{ParsedAttrSelectorOperation, AttrSelectorOperation, NamespaceConstraint};
use bloom::{BLOOM_HASH_MASK, BloomFilter};
use parser::{AncestorHashes, Combinator, Component, LocalName};
use parser::{Selector, SelectorImpl, SelectorIter, SelectorList};
use std::borrow::Borrow;
use tree::Element;

pub use context::*;

// The bloom filter for descendant CSS selectors will have a <1% false
// positive rate until it has this many selectors in it, then it will
// rapidly increase.
pub static RECOMMENDED_SELECTOR_BLOOM_FILTER_SIZE: usize = 4096;

bitflags! {
    /// Set of flags that are set on either the element or its parent (depending
    /// on the flag) if the element could potentially match a selector.
    pub flags ElementSelectorFlags: usize {
        /// When a child is added or removed from the parent, all the children
        /// must be restyled, because they may match :nth-last-child,
        /// :last-of-type, :nth-last-of-type, or :only-of-type.
        const HAS_SLOW_SELECTOR = 1 << 0,

        /// When a child is added or removed from the parent, any later
        /// children must be restyled, because they may match :nth-child,
        /// :first-of-type, or :nth-of-type.
        const HAS_SLOW_SELECTOR_LATER_SIBLINGS = 1 << 1,

        /// When a child is added or removed from the parent, the first and
        /// last children must be restyled, because they may match :first-child,
        /// :last-child, or :only-child.
        const HAS_EDGE_CHILD_SELECTOR = 1 << 2,

        /// The element has an empty selector, so when a child is appended we
        /// might need to restyle the parent completely.
        const HAS_EMPTY_SELECTOR = 1 << 3,
    }
}

impl ElementSelectorFlags {
    /// Returns the subset of flags that apply to the element.
    pub fn for_self(self) -> ElementSelectorFlags {
        self & (HAS_EMPTY_SELECTOR)
    }

    /// Returns the subset of flags that apply to the parent.
    pub fn for_parent(self) -> ElementSelectorFlags {
        self & (HAS_SLOW_SELECTOR | HAS_SLOW_SELECTOR_LATER_SIBLINGS | HAS_EDGE_CHILD_SELECTOR)
    }
}

/// Holds per-selector data alongside a pointer to MatchingContext.
pub struct LocalMatchingContext<'a, 'b: 'a, Impl: SelectorImpl> {
    /// Shared `MatchingContext`.
    pub shared: &'a mut MatchingContext<'b>,
    /// A reference to the base selector we're matching against.
    pub selector: &'a Selector<Impl>,
    /// The offset of the current compound selector being matched, kept up to
    /// date by the callees when the iterator is advanced. This, in conjunction
    /// with the selector reference above, allows callees to synthesize an
    /// iterator for the current compound selector on-demand. This is necessary
    /// because the primary iterator may already have been advanced partway
    /// through the current compound selector, and the callee may need the whole
    /// thing.
    offset: usize,
    /// The level of nesting for the selector being matched.
    pub nesting_level: usize,
    /// Holds a bool flag to see whether :active and :hover quirk should try to
    /// match or not. This flag can only be true in the case PseudoElements are
    /// encountered when matching mode is ForStatelessPseudoElement.
    pub hover_active_quirk_disabled: bool,
}

impl<'a, 'b, Impl> LocalMatchingContext<'a, 'b, Impl>
    where Impl: SelectorImpl
{
    /// Constructs a new `LocalMatchingContext`.
    pub fn new(shared: &'a mut MatchingContext<'b>,
               selector: &'a Selector<Impl>) -> Self {
        Self {
            shared: shared,
            selector: selector,
            offset: 0,
            nesting_level: 0,
            // We flip this off once third sequence is reached.
            hover_active_quirk_disabled: selector.has_pseudo_element(),
        }
    }

    /// Updates offset of Selector to show new compound selector.
    /// To be able to correctly re-synthesize main SelectorIter.
    pub fn note_next_sequence(&mut self, selector_iter: &SelectorIter<Impl>) {
        if let QuirksMode::Quirks = self.shared.quirks_mode() {
            if self.selector.has_pseudo_element() && self.offset != 0 {
                // This is the _second_ call to note_next_sequence,
                // which means we've moved past the compound
                // selector adjacent to the pseudo-element.
                self.hover_active_quirk_disabled = false;
            }

            self.offset = self.selector.len() - selector_iter.selector_length();
        }
    }

    /// Returns true if current compound selector matches :active and :hover quirk.
    /// https://quirks.spec.whatwg.org/#the-active-and-hover-quirk
    pub fn active_hover_quirk_matches(&mut self) -> bool {
        if self.shared.quirks_mode() != QuirksMode::Quirks {
            return false;
        }

        // Don't allow it in recursive selectors such as :not and :-moz-any.
        if self.nesting_level != 0 {
            return false;
        }

        if self.hover_active_quirk_disabled {
            return false;
        }

        let mut iter = if self.offset == 0 {
            self.selector.iter()
        } else {
            self.selector.iter_from(self.offset)
        };

        return iter.all(|simple| {
            match *simple {
                Component::LocalName(_) |
                Component::AttributeInNoNamespaceExists { .. } |
                Component::AttributeInNoNamespace { .. } |
                Component::AttributeOther(_) |
                Component::ID(_) |
                Component::Class(_) |
                Component::PseudoElement(_) |
                Component::Negation(_) |
                Component::FirstChild |
                Component::LastChild |
                Component::OnlyChild |
                Component::Empty |
                Component::NthChild(_, _) |
                Component::NthLastChild(_, _) |
                Component::NthOfType(_, _) |
                Component::NthLastOfType(_, _) |
                Component::FirstOfType |
                Component::LastOfType |
                Component::OnlyOfType => false,
                Component::NonTSPseudoClass(ref pseudo_class) => {
                    Impl::is_active_or_hover(pseudo_class)
                },
                _ => true,
            }
        });
    }
}

pub fn matches_selector_list<E>(selector_list: &SelectorList<E::Impl>,
                                element: &E,
                                context: &mut MatchingContext)
                                -> bool
    where E: Element
{
    selector_list.0.iter().any(|selector| {
        matches_selector(selector,
                         0,
                         None,
                         element,
                         context,
                         &mut |_, _| {})
    })
}

#[inline(always)]
fn may_match<E>(hashes: &AncestorHashes,
                bf: &BloomFilter)
                -> bool
    where E: Element,
{
    // Check the first three hashes. Note that we can check for zero before
    // masking off the high bits, since if any of the first three hashes is
    // zero the fourth will be as well. We also take care to avoid the
    // special-case complexity of the fourth hash until we actually reach it,
    // because we usually don't.
    //
    // To be clear: this is all extremely hot.
    for i in 0..3 {
        let packed = hashes.packed_hashes[i];
        if packed == 0 {
            // No more hashes left - unable to fast-reject.
            return true;
        }

        if !bf.might_contain_hash(packed & BLOOM_HASH_MASK) {
            // Hooray! We fast-rejected on this hash.
            return false;
        }
    }

    // Now do the slighty-more-complex work of synthesizing the fourth hash,
    // and check it against the filter if it exists.
    let fourth = hashes.fourth_hash();
    fourth == 0 || bf.might_contain_hash(fourth)
}

/// Tracks whether we are currently looking for relevant links for a given
/// complex selector. A "relevant link" is the element being matched if it is a
/// link or the nearest ancestor link.
///
/// `matches_complex_selector` creates a new instance of this for each complex
/// selector we try to match for an element. This is done because `is_visited`
/// and `is_unvisited` are based on relevant link state of only the current
/// complex selector being matched (not the global relevant link status for all
/// selectors in `MatchingContext`).
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum RelevantLinkStatus {
    /// Looking for a possible relevant link.  This is the initial mode when
    /// matching a selector.
    Looking,
    /// Not looking for a relevant link.  We transition to this mode if we
    /// encounter a sibiling combinator (since only ancestor combinators are
    /// allowed for this purpose).
    NotLooking,
    /// Found a relevant link for the element being matched.
    Found,
}

impl Default for RelevantLinkStatus {
    fn default() -> Self {
        RelevantLinkStatus::NotLooking
    }
}

impl RelevantLinkStatus {
    /// If we found the relevant link for this element, record that in the
    /// overall matching context for the element as a whole and stop looking for
    /// addtional links.
    fn examine_potential_link<E>(&self, element: &E, context: &mut MatchingContext)
                                 -> RelevantLinkStatus
        where E: Element,
    {
        // If a relevant link was previously found, we no longer want to look
        // for links.  Only the nearest ancestor link is considered relevant.
        if *self != RelevantLinkStatus::Looking {
            return RelevantLinkStatus::NotLooking
        }

        if !element.is_link() {
            return *self
        }

        // We found a relevant link. Record this in the `MatchingContext`,
        // where we track whether one was found for _any_ selector (meaning
        // this field might already be true from a previous selector).
        context.relevant_link_found = true;
        // Also return `Found` to update the relevant link status for _this_
        // specific selector's matching process.
        RelevantLinkStatus::Found
    }

    /// Returns whether an element is considered visited for the purposes of
    /// matching.  This is true only if the element is a link, an relevant link
    /// exists for the element, and the visited handling mode is set to accept
    /// relevant links as visited.
    pub fn is_visited<E>(&self, element: &E, context: &MatchingContext) -> bool
        where E: Element,
    {
        if !element.is_link() {
            return false
        }

        if context.visited_handling == VisitedHandlingMode::AllLinksVisitedAndUnvisited {
            return true;
        }

        // Non-relevant links are always unvisited.
        if *self != RelevantLinkStatus::Found {
            return false
        }

        context.visited_handling == VisitedHandlingMode::RelevantLinkVisited
    }

    /// Returns whether an element is considered unvisited for the purposes of
    /// matching.  Assuming the element is a link, this is always true for
    /// non-relevant links, since only relevant links can potentially be treated
    /// as visited.  If this is a relevant link, then is it unvisited if the
    /// visited handling mode is set to treat all links as unvisted (including
    /// relevant links).
    pub fn is_unvisited<E>(&self, element: &E, context: &MatchingContext) -> bool
        where E: Element,
    {
        if !element.is_link() {
            return false
        }

        if context.visited_handling == VisitedHandlingMode::AllLinksVisitedAndUnvisited {
            return true;
        }

        // Non-relevant links are always unvisited.
        if *self != RelevantLinkStatus::Found {
            return true
        }

        context.visited_handling == VisitedHandlingMode::AllLinksUnvisited
    }
}

/// A result of selector matching, includes 3 failure types,
///
///   NotMatchedAndRestartFromClosestLaterSibling
///   NotMatchedAndRestartFromClosestDescendant
///   NotMatchedGlobally
///
/// When NotMatchedGlobally appears, stop selector matching completely since
/// the succeeding selectors never matches.
/// It is raised when
///   Child combinator cannot find the candidate element.
///   Descendant combinator cannot find the candidate element.
///
/// When NotMatchedAndRestartFromClosestDescendant appears, the selector
/// matching does backtracking and restarts from the closest Descendant
/// combinator.
/// It is raised when
///   NextSibling combinator cannot find the candidate element.
///   LaterSibling combinator cannot find the candidate element.
///   Child combinator doesn't match on the found element.
///
/// When NotMatchedAndRestartFromClosestLaterSibling appears, the selector
/// matching does backtracking and restarts from the closest LaterSibling
/// combinator.
/// It is raised when
///   NextSibling combinator doesn't match on the found element.
///
/// For example, when the selector "d1 d2 a" is provided and we cannot *find*
/// an appropriate ancestor element for "d1", this selector matching raises
/// NotMatchedGlobally since even if "d2" is moved to more upper element, the
/// candidates for "d1" becomes less than before and d1 .
///
/// The next example is siblings. When the selector "b1 + b2 ~ d1 a" is
/// provided and we cannot *find* an appropriate brother element for b1,
/// the selector matching raises NotMatchedAndRestartFromClosestDescendant.
/// The selectors ("b1 + b2 ~") doesn't match and matching restart from "d1".
///
/// The additional example is child and sibling. When the selector
/// "b1 + c1 > b2 ~ d1 a" is provided and the selector "b1" doesn't match on
/// the element, this "b1" raises NotMatchedAndRestartFromClosestLaterSibling.
/// However since the selector "c1" raises
/// NotMatchedAndRestartFromClosestDescendant. So the selector
/// "b1 + c1 > b2 ~ " doesn't match and restart matching from "d1".
#[derive(PartialEq, Eq, Copy, Clone)]
enum SelectorMatchingResult {
    Matched,
    NotMatchedAndRestartFromClosestLaterSibling,
    NotMatchedAndRestartFromClosestDescendant,
    NotMatchedGlobally,
}

/// Matches a selector, fast-rejecting against a bloom filter.
///
/// We accept an offset to allow consumers to represent and match against
/// partial selectors (indexed from the right). We use this API design, rather
/// than having the callers pass a SelectorIter, because creating a SelectorIter
/// requires dereferencing the selector to get the length, which adds an
/// unncessary cache miss for cases when we can fast-reject with AncestorHashes
/// (which the caller can store inline with the selector pointer).
#[inline(always)]
pub fn matches_selector<E, F>(selector: &Selector<E::Impl>,
                              offset: usize,
                              hashes: Option<&AncestorHashes>,
                              element: &E,
                              context: &mut MatchingContext,
                              flags_setter: &mut F)
                              -> bool
    where E: Element,
          F: FnMut(&E, ElementSelectorFlags),
{
    // Use the bloom filter to fast-reject.
    if let Some(hashes) = hashes {
        if let Some(filter) = context.bloom_filter {
            if !may_match::<E>(hashes, filter) {
                return false;
            }
        }
    }

    let mut local_context = LocalMatchingContext::new(context, selector);
    let iter = if offset == 0 {
        selector.iter()
    } else {
        selector.iter_from(offset)
    };
    matches_complex_selector(iter, element, &mut local_context, flags_setter)
}

/// Whether a compound selector matched, and whether it was the rightmost
/// selector inside the complex selector.
pub enum CompoundSelectorMatchingResult {
    /// The compound selector matched, and the next combinator offset is
    /// `next_combinator_offset`.
    ///
    /// If the next combinator offset is zero, it means that it's the rightmost
    /// selector.
    Matched { next_combinator_offset: usize, },
    /// The selector didn't match.
    NotMatched,
}

/// Matches a compound selector belonging to `selector`, starting at offset
/// `from_offset`, matching left to right.
///
/// Requires that `from_offset` points to a `Combinator`.
///
/// NOTE(emilio): This doesn't allow to match in the leftmost sequence of the
/// complex selector, but it happens to be the case we don't need it.
pub fn matches_compound_selector<E>(
    selector: &Selector<E::Impl>,
    mut from_offset: usize,
    context: &mut MatchingContext,
    element: &E,
) -> CompoundSelectorMatchingResult
where
    E: Element
{
    if cfg!(debug_assertions) {
        selector.combinator_at(from_offset); // This asserts.
    }

    let mut local_context = LocalMatchingContext::new(context, selector);
    for component in selector.iter_raw_parse_order_from(from_offset - 1) {
        if matches!(*component, Component::Combinator(..)) {
            return CompoundSelectorMatchingResult::Matched {
                next_combinator_offset: from_offset - 1,
            }
        }

        if !matches_simple_selector(
            component,
            element,
            &mut local_context,
            &RelevantLinkStatus::NotLooking,
            &mut |_, _| {}) {
            return CompoundSelectorMatchingResult::NotMatched;
        }

        from_offset -= 1;
    }

    return CompoundSelectorMatchingResult::Matched {
        next_combinator_offset: 0,
    }
}

/// Matches a complex selector.
pub fn matches_complex_selector<E, F>(mut iter: SelectorIter<E::Impl>,
                                      element: &E,
                                      mut context: &mut LocalMatchingContext<E::Impl>,
                                      flags_setter: &mut F)
                                      -> bool
    where E: Element,
          F: FnMut(&E, ElementSelectorFlags),
{
    if cfg!(debug_assertions) {
        if context.nesting_level == 0 &&
            context.shared.matching_mode == MatchingMode::ForStatelessPseudoElement {
            assert!(iter.clone().any(|c| {
                matches!(*c, Component::PseudoElement(..))
            }));
        }
    }

    // If this is the special pseudo-element mode, consume the ::pseudo-element
    // before proceeding, since the caller has already handled that part.
    if context.nesting_level == 0 &&
        context.shared.matching_mode == MatchingMode::ForStatelessPseudoElement {
        // Consume the pseudo.
        let pseudo = iter.next().unwrap();
        debug_assert!(matches!(*pseudo, Component::PseudoElement(..)),
                      "Used MatchingMode::ForStatelessPseudoElement in a non-pseudo selector");

        // The only other parser-allowed Component in this sequence is a state
        // class. We just don't match in that case.
        if let Some(s) = iter.next() {
            debug_assert!(matches!(*s, Component::NonTSPseudoClass(..)),
                          "Someone messed up pseudo-element parsing");
            return false;
        }

        // Advance to the non-pseudo-element part of the selector, and inform the context.
        if iter.next_sequence().is_none() {
            return true;
        }
        context.note_next_sequence(&mut iter);
    }

    match matches_complex_selector_internal(iter,
                                            element,
                                            context,
                                            &mut RelevantLinkStatus::Looking,
                                            flags_setter) {
        SelectorMatchingResult::Matched => true,
        _ => false
    }
}

fn matches_complex_selector_internal<E, F>(mut selector_iter: SelectorIter<E::Impl>,
                                           element: &E,
                                           context: &mut LocalMatchingContext<E::Impl>,
                                           relevant_link: &mut RelevantLinkStatus,
                                           flags_setter: &mut F)
                                           -> SelectorMatchingResult
     where E: Element,
           F: FnMut(&E, ElementSelectorFlags),
{
    *relevant_link = relevant_link.examine_potential_link(element, &mut context.shared);

    let matches_all_simple_selectors = selector_iter.all(|simple| {
        matches_simple_selector(simple, element, context, &relevant_link, flags_setter)
    });

    debug!("Matching for {:?}, simple selector {:?}, relevant link {:?}",
           element, selector_iter, relevant_link);

    let combinator = selector_iter.next_sequence();
    // Inform the context that the we've advanced to the next compound selector.
    context.note_next_sequence(&mut selector_iter);
    let siblings = combinator.map_or(false, |c| c.is_sibling());
    if siblings {
        flags_setter(element, HAS_SLOW_SELECTOR_LATER_SIBLINGS);
    }

    if !matches_all_simple_selectors {
        return SelectorMatchingResult::NotMatchedAndRestartFromClosestLaterSibling;
    }

    match combinator {
        None => SelectorMatchingResult::Matched,
        Some(c) => {
            let (mut next_element, candidate_not_found) = match c {
                Combinator::NextSibling | Combinator::LaterSibling => {
                    // Only ancestor combinators are allowed while looking for
                    // relevant links, so switch to not looking.
                    *relevant_link = RelevantLinkStatus::NotLooking;
                    (element.prev_sibling_element(),
                     SelectorMatchingResult::NotMatchedAndRestartFromClosestDescendant)
                }
                Combinator::Child | Combinator::Descendant => {
                    if element.blocks_ancestor_combinators() {
                        (None, SelectorMatchingResult::NotMatchedGlobally)
                    } else {
                        (element.parent_element(),
                         SelectorMatchingResult::NotMatchedGlobally)
                    }
                }
                Combinator::PseudoElement => {
                    (element.pseudo_element_originating_element(),
                     SelectorMatchingResult::NotMatchedGlobally)
                }
            };

            loop {
                let element = match next_element {
                    None => return candidate_not_found,
                    Some(next_element) => next_element,
                };
                let result = matches_complex_selector_internal(selector_iter.clone(),
                                                               &element,
                                                               context,
                                                               relevant_link,
                                                               flags_setter);
                match (result, c) {
                    // Return the status immediately.
                    (SelectorMatchingResult::Matched, _) => return result,
                    (SelectorMatchingResult::NotMatchedGlobally, _) => return result,

                    // Upgrade the failure status to
                    // NotMatchedAndRestartFromClosestDescendant.
                    (_, Combinator::PseudoElement) |
                    (_, Combinator::Child) => return SelectorMatchingResult::NotMatchedAndRestartFromClosestDescendant,

                    // Return the status directly.
                    (_, Combinator::NextSibling) => return result,

                    // If the failure status is NotMatchedAndRestartFromClosestDescendant
                    // and combinator is Combinator::LaterSibling, give up this Combinator::LaterSibling matching
                    // and restart from the closest descendant combinator.
                    (SelectorMatchingResult::NotMatchedAndRestartFromClosestDescendant, Combinator::LaterSibling)
                        => return result,

                    // The Combinator::Descendant combinator and the status is
                    // NotMatchedAndRestartFromClosestLaterSibling or
                    // NotMatchedAndRestartFromClosestDescendant,
                    // or the Combinator::LaterSibling combinator and the status is
                    // NotMatchedAndRestartFromClosestDescendant
                    // can continue to matching on the next candidate element.
                    _ => {},
                }
                next_element = if siblings {
                    element.prev_sibling_element()
                } else {
                    element.parent_element()
                };
            }
        }
    }
}

/// Determines whether the given element matches the given single selector.
#[inline]
fn matches_simple_selector<E, F>(
        selector: &Component<E::Impl>,
        element: &E,
        context: &mut LocalMatchingContext<E::Impl>,
        relevant_link: &RelevantLinkStatus,
        flags_setter: &mut F)
        -> bool
    where E: Element,
          F: FnMut(&E, ElementSelectorFlags),
{
    match *selector {
        Component::Combinator(_) => unreachable!(),
        Component::PseudoElement(ref pseudo) => {
            element.match_pseudo_element(pseudo, context.shared)
        }
        Component::LocalName(LocalName { ref name, ref lower_name }) => {
            let is_html = element.is_html_element_in_html_document();
            element.get_local_name() == select_name(is_html, name, lower_name).borrow()
        }
        Component::ExplicitUniversalType |
        Component::ExplicitAnyNamespace => {
            true
        }
        Component::Namespace(_, ref url) |
        Component::DefaultNamespace(ref url) => {
            element.get_namespace() == url.borrow()
        }
        Component::ExplicitNoNamespace => {
            let ns = ::parser::namespace_empty_string::<E::Impl>();
            element.get_namespace() == ns.borrow()
        }
        Component::ID(ref id) => {
            element.has_id(id, context.shared.classes_and_ids_case_sensitivity())
        }
        Component::Class(ref class) => {
            element.has_class(class, context.shared.classes_and_ids_case_sensitivity())
        }
        Component::AttributeInNoNamespaceExists { ref local_name, ref local_name_lower } => {
            let is_html = element.is_html_element_in_html_document();
            element.attr_matches(
                &NamespaceConstraint::Specific(&::parser::namespace_empty_string::<E::Impl>()),
                select_name(is_html, local_name, local_name_lower),
                &AttrSelectorOperation::Exists
            )
        }
        Component::AttributeInNoNamespace {
            ref local_name,
            ref local_name_lower,
            ref value,
            operator,
            case_sensitivity,
            never_matches,
        } => {
            if never_matches {
                return false
            }
            let is_html = element.is_html_element_in_html_document();
            element.attr_matches(
                &NamespaceConstraint::Specific(&::parser::namespace_empty_string::<E::Impl>()),
                select_name(is_html, local_name, local_name_lower),
                &AttrSelectorOperation::WithValue {
                    operator: operator,
                    case_sensitivity: case_sensitivity.to_unconditional(is_html),
                    expected_value: value,
                }
            )
        }
        Component::AttributeOther(ref attr_sel) => {
            if attr_sel.never_matches {
                return false
            }
            let is_html = element.is_html_element_in_html_document();
            element.attr_matches(
                &attr_sel.namespace(),
                select_name(is_html, &attr_sel.local_name, &attr_sel.local_name_lower),
                &match attr_sel.operation {
                    ParsedAttrSelectorOperation::Exists => AttrSelectorOperation::Exists,
                    ParsedAttrSelectorOperation::WithValue {
                        operator,
                        case_sensitivity,
                        ref expected_value,
                    } => {
                        AttrSelectorOperation::WithValue {
                            operator: operator,
                            case_sensitivity: case_sensitivity.to_unconditional(is_html),
                            expected_value: expected_value,
                        }
                    }
                }
            )
        }
        Component::NonTSPseudoClass(ref pc) => {
            element.match_non_ts_pseudo_class(pc, context, relevant_link, flags_setter)
        }
        Component::FirstChild => {
            matches_first_child(element, flags_setter)
        }
        Component::LastChild => {
            matches_last_child(element, flags_setter)
        }
        Component::OnlyChild => {
            matches_first_child(element, flags_setter) &&
            matches_last_child(element, flags_setter)
        }
        Component::Root => {
            element.is_root()
        }
        Component::Empty => {
            flags_setter(element, HAS_EMPTY_SELECTOR);
            element.is_empty()
        }
        Component::NthChild(a, b) => {
            matches_generic_nth_child(element, a, b, false, false, flags_setter)
        }
        Component::NthLastChild(a, b) => {
            matches_generic_nth_child(element, a, b, false, true, flags_setter)
        }
        Component::NthOfType(a, b) => {
            matches_generic_nth_child(element, a, b, true, false, flags_setter)
        }
        Component::NthLastOfType(a, b) => {
            matches_generic_nth_child(element, a, b, true, true, flags_setter)
        }
        Component::FirstOfType => {
            matches_generic_nth_child(element, 0, 1, true, false, flags_setter)
        }
        Component::LastOfType => {
            matches_generic_nth_child(element, 0, 1, true, true, flags_setter)
        }
        Component::OnlyOfType => {
            matches_generic_nth_child(element, 0, 1, true, false, flags_setter) &&
            matches_generic_nth_child(element, 0, 1, true, true, flags_setter)
        }
        Component::Negation(ref negated) => {
            context.nesting_level += 1;
            let result = !negated.iter().all(|ss| {
                matches_simple_selector(ss, element, context,
                                        relevant_link, flags_setter)
            });
            context.nesting_level -= 1;
            result
        }
    }
}

fn select_name<'a, T>(is_html: bool, local_name: &'a T, local_name_lower: &'a T) -> &'a T {
    if is_html {
        local_name_lower
    } else {
        local_name
    }
}

#[inline]
fn matches_generic_nth_child<E, F>(element: &E,
                                   a: i32,
                                   b: i32,
                                   is_of_type: bool,
                                   is_from_end: bool,
                                   flags_setter: &mut F)
                                   -> bool
    where E: Element,
          F: FnMut(&E, ElementSelectorFlags),
{
    flags_setter(element, if is_from_end {
        HAS_SLOW_SELECTOR
    } else {
        HAS_SLOW_SELECTOR_LATER_SIBLINGS
    });

    let mut index: i32 = 1;
    let mut next_sibling = if is_from_end {
        element.next_sibling_element()
    } else {
        element.prev_sibling_element()
    };

    loop {
        let sibling = match next_sibling {
            None => break,
            Some(next_sibling) => next_sibling
        };

        if is_of_type {
            if element.get_local_name() == sibling.get_local_name() &&
                element.get_namespace() == sibling.get_namespace() {
                index += 1;
            }
        } else {
          index += 1;
        }
        next_sibling = if is_from_end {
            sibling.next_sibling_element()
        } else {
            sibling.prev_sibling_element()
        };
    }

    // Is there a non-negative integer n such that An+B=index?
    match index.checked_sub(b) {
        None => false,
        Some(an) => match an.checked_div(a) {
            Some(n) => n >= 0 && a * n == an,
            None /* a == 0 */ => an == 0,
        },
    }
}

#[inline]
fn matches_first_child<E, F>(element: &E, flags_setter: &mut F) -> bool
    where E: Element,
          F: FnMut(&E, ElementSelectorFlags),
{
    flags_setter(element, HAS_EDGE_CHILD_SELECTOR);
    element.prev_sibling_element().is_none()
}

#[inline]
fn matches_last_child<E, F>(element: &E, flags_setter: &mut F) -> bool
    where E: Element,
          F: FnMut(&E, ElementSelectorFlags),
{
    flags_setter(element, HAS_EDGE_CHILD_SELECTOR);
    element.next_sibling_element().is_none()
}
