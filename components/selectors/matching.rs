/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use bloom::BloomFilter;
use parser::{CaseSensitivity, Combinator, ComplexSelector, LocalName};
use parser::{SimpleSelector, Selector, SelectorImpl};
use std::borrow::Borrow;
use tree::Element;

/// The reason why we're doing selector matching.
///
/// If this is for styling, this will include the flags in the parent element.
///
/// This is done because Servo doesn't need those flags at all when it's not
/// styling (e.g., when you're doing document.querySelector). For example, a
/// slow selector in an API like querySelector doesn't imply that the parent
/// could match it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MatchingReason {
    ForStyling,
    Other,
}

impl MatchingReason {
    #[inline]
    fn for_styling(&self) -> bool {
        *self == MatchingReason::ForStyling
    }
}

// The bloom filter for descendant CSS selectors will have a <1% false
// positive rate until it has this many selectors in it, then it will
// rapidly increase.
pub static RECOMMENDED_SELECTOR_BLOOM_FILTER_SIZE: usize = 4096;

bitflags! {
    /// Set of flags that determine the different kind of elements affected by
    /// the selector matching process.
    ///
    /// This is used to implement efficient sharing.
    pub flags StyleRelations: u16 {
        /// Whether this element has matched any rule that is determined by a
        /// sibling (when using the `+` or `~` combinators).
        const AFFECTED_BY_SIBLINGS = 1 << 0,

        /// Whether this element has matched any rule whose matching is
        /// determined by its position in the tree (i.e., first-child,
        /// nth-child, etc.).
        const AFFECTED_BY_CHILD_INDEX = 1 << 1,

        /// Whether this flag is affected by any state (i.e., non
        /// tree-structural pseudo-class).
        const AFFECTED_BY_STATE = 1 << 2,

        /// Whether this element is affected by an ID selector.
        const AFFECTED_BY_ID_SELECTOR = 1 << 3,

        /// Whether this element is affected by a non-common style-affecting
        /// attribute.
        const AFFECTED_BY_NON_COMMON_STYLE_AFFECTING_ATTRIBUTE_SELECTOR = 1 << 4,

        /// Whether this element matches the :empty pseudo class.
        const AFFECTED_BY_EMPTY = 1 << 5,

        /// Whether this element has a style attribute. Computed
        /// externally.
        const AFFECTED_BY_STYLE_ATTRIBUTE = 1 << 6,

        /// Whether this element is affected by presentational hints. This is
        /// computed externally (that is, in Servo).
        const AFFECTED_BY_PRESENTATIONAL_HINTS = 1 << 7,

        /// Whether this element has pseudo-element styles. Computed externally.
        const AFFECTED_BY_PSEUDO_ELEMENTS = 1 << 8,

        /// Whether this element has effective animation styles. Computed
        /// externally.
        const AFFECTED_BY_ANIMATIONS = 1 << 9,

        /// Whether this element has effective transition styles. Computed
        /// externally.
        const AFFECTED_BY_TRANSITIONS = 1 << 10,
    }
}

bitflags! {
    /// Set of flags that are set on the parent depending on whether a child
    /// could potentially match a selector.
    ///
    /// These setters, in the case of Servo, must be atomic, due to the parallel
    /// traversal.
    pub flags ElementFlags: u8 {
        /// When a child is added or removed from this element, all the children
        /// must be restyled, because they may match :nth-last-child,
        /// :last-of-type, :nth-last-of-type, or :only-of-type.
        const HAS_SLOW_SELECTOR = 1 << 0,

        /// When a child is added or removed from this element, any later
        /// children must be restyled, because they may match :nth-child,
        /// :first-of-type, or :nth-of-type.
        const HAS_SLOW_SELECTOR_LATER_SIBLINGS = 1 << 1,

        /// When a child is added or removed from this element, the first and
        /// last children must be restyled, because they may match :first-child,
        /// :last-child, or :only-child.
        const HAS_EDGE_CHILD_SELECTOR = 1 << 2,

        /// The element has an empty selector, so when a child is appended we
        /// might need to restyle the parent completely.
        const HAS_EMPTY_SELECTOR = 1 << 3,
    }
}

pub fn matches<E>(selector_list: &[Selector<E::Impl>],
                  element: &E,
                  parent_bf: Option<&BloomFilter>,
                  reason: MatchingReason)
                  -> bool
    where E: Element
{
    selector_list.iter().any(|selector| {
        selector.pseudo_element.is_none() &&
        matches_complex_selector(&*selector.complex_selector, element, parent_bf, &mut StyleRelations::empty(), reason)
    })
}

/// Determines whether the given element matches the given complex selector.
///
/// NB: If you add support for any new kinds of selectors to this routine, be sure to set
/// `shareable` to false unless you are willing to update the style sharing logic. Otherwise things
/// will almost certainly break as elements will start mistakenly sharing styles. (See
/// `can_share_style_with` in `servo/components/style/matching.rs`.)
pub fn matches_complex_selector<E>(selector: &ComplexSelector<E::Impl>,
                                   element: &E,
                                   parent_bf: Option<&BloomFilter>,
                                   relations: &mut StyleRelations,
                                   reason: MatchingReason)
                                   -> bool
    where E: Element
{
    match matches_complex_selector_internal(selector, element, parent_bf, relations, reason) {
        SelectorMatchingResult::Matched => {
            match selector.next {
                Some((_, Combinator::NextSibling)) |
                Some((_, Combinator::LaterSibling)) => *relations |= AFFECTED_BY_SIBLINGS,
                _ => {}
            }

            true
        }
        _ => false
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

/// Quickly figures out whether or not the complex selector is worth doing more
/// work on. If the simple selectors don't match, or there's a child selector
/// that does not appear in the bloom parent bloom filter, we can exit early.
fn can_fast_reject<E>(mut selector: &ComplexSelector<E::Impl>,
                      element: &E,
                      parent_bf: Option<&BloomFilter>,
                      relations: &mut StyleRelations,
                      reason: MatchingReason)
                      -> Option<SelectorMatchingResult>
    where E: Element
{
    if !selector.compound_selector.iter().all(|simple_selector| {
      matches_simple_selector(simple_selector, element, parent_bf, relations, reason) }) {
        return Some(SelectorMatchingResult::NotMatchedAndRestartFromClosestLaterSibling);
    }

    let bf: &BloomFilter = match parent_bf {
        None => return None,
        Some(ref bf) => bf,
    };

    // See if the bloom filter can exclude any of the descendant selectors, and
    // reject if we can.
    loop {
         match selector.next {
             None => break,
             Some((ref cs, Combinator::Descendant)) => selector = &**cs,
             Some((ref cs, _)) => {
                 selector = &**cs;
                 continue;
             }
         };

        for ss in selector.compound_selector.iter() {
            match *ss {
                SimpleSelector::LocalName(LocalName { ref name, ref lower_name })  => {
                    if !bf.might_contain(name) &&
                       !bf.might_contain(lower_name) {
                        return Some(SelectorMatchingResult::NotMatchedGlobally);
                    }
                },
                SimpleSelector::Namespace(ref namespace) => {
                    if !bf.might_contain(&namespace.url) {
                        return Some(SelectorMatchingResult::NotMatchedGlobally);
                    }
                },
                SimpleSelector::ID(ref id) => {
                    if !bf.might_contain(id) {
                        return Some(SelectorMatchingResult::NotMatchedGlobally);
                    }
                },
                SimpleSelector::Class(ref class) => {
                    if !bf.might_contain(class) {
                        return Some(SelectorMatchingResult::NotMatchedGlobally);
                    }
                },
                _ => {},
            }
        }
    }

    // Can't fast reject.
    None
}

fn matches_complex_selector_internal<E>(selector: &ComplexSelector<E::Impl>,
                                         element: &E,
                                         parent_bf: Option<&BloomFilter>,
                                         relations: &mut StyleRelations,
                                         reason: MatchingReason)
                                         -> SelectorMatchingResult
     where E: Element
{
    if let Some(result) = can_fast_reject(selector, element, parent_bf, relations, reason) {
        return result;
    }

    match selector.next {
        None => SelectorMatchingResult::Matched,
        Some((ref next_selector, combinator)) => {
            let (siblings, candidate_not_found) = match combinator {
                Combinator::Child => (false, SelectorMatchingResult::NotMatchedGlobally),
                Combinator::Descendant => (false, SelectorMatchingResult::NotMatchedGlobally),
                Combinator::NextSibling => (true, SelectorMatchingResult::NotMatchedAndRestartFromClosestDescendant),
                Combinator::LaterSibling => (true, SelectorMatchingResult::NotMatchedAndRestartFromClosestDescendant),
            };
            let mut next_element = if siblings {
                element.prev_sibling_element()
            } else {
                element.parent_element()
            };
            loop {
                let element = match next_element {
                    None => return candidate_not_found,
                    Some(next_element) => next_element,
                };
                let result = matches_complex_selector_internal(&**next_selector,
                                                                &element,
                                                                parent_bf,
                                                                relations,
                                                                reason);
                match (result, combinator) {
                    // Return the status immediately.
                    (SelectorMatchingResult::Matched, _) => return result,
                    (SelectorMatchingResult::NotMatchedGlobally, _) => return result,

                    // Upgrade the failure status to
                    // NotMatchedAndRestartFromClosestDescendant.
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
fn matches_simple_selector<E>(
        selector: &SimpleSelector<E::Impl>,
        element: &E,
        parent_bf: Option<&BloomFilter>,
        relations: &mut StyleRelations,
        reason: MatchingReason)
        -> bool
    where E: Element
{
    macro_rules! relation_if {
        ($ex:expr, $flag:ident) => {
            if $ex {
                *relations |= $flag;
                true
            } else {
                false
            }
        }
    }

    match *selector {
        SimpleSelector::LocalName(LocalName { ref name, ref lower_name }) => {
            let name = if element.is_html_element_in_html_document() { lower_name } else { name };
            element.get_local_name() == name.borrow()
        }
        SimpleSelector::Namespace(ref namespace) => {
            element.get_namespace() == namespace.url.borrow()
        }
        // TODO: case-sensitivity depends on the document type and quirks mode
        SimpleSelector::ID(ref id) => {
            relation_if!(element.get_id().map_or(false, |attr| attr == *id),
                         AFFECTED_BY_ID_SELECTOR)
        }
        SimpleSelector::Class(ref class) => {
            element.has_class(class)
        }
        SimpleSelector::AttrExists(ref attr) => {
            let matches = element.match_attr_has(attr);

            if matches && !E::Impl::attr_exists_selector_is_shareable(attr) {
                *relations |= AFFECTED_BY_NON_COMMON_STYLE_AFFECTING_ATTRIBUTE_SELECTOR;
            }

            matches
        }
        SimpleSelector::AttrEqual(ref attr, ref value, case_sensitivity) => {
            let matches = match case_sensitivity {
                CaseSensitivity::CaseSensitive => element.match_attr_equals(attr, value),
                CaseSensitivity::CaseInsensitive => element.match_attr_equals_ignore_ascii_case(attr, value),
            };

            if matches && !E::Impl::attr_equals_selector_is_shareable(attr, value) {
                *relations |= AFFECTED_BY_NON_COMMON_STYLE_AFFECTING_ATTRIBUTE_SELECTOR;
            }

            matches
        }
        SimpleSelector::AttrIncludes(ref attr, ref value) => {
            relation_if!(element.match_attr_includes(attr, value),
                         AFFECTED_BY_NON_COMMON_STYLE_AFFECTING_ATTRIBUTE_SELECTOR)
        }
        SimpleSelector::AttrDashMatch(ref attr, ref value) => {
            relation_if!(element.match_attr_dash(attr, value),
                         AFFECTED_BY_NON_COMMON_STYLE_AFFECTING_ATTRIBUTE_SELECTOR)
        }
        SimpleSelector::AttrPrefixMatch(ref attr, ref value) => {
            relation_if!(element.match_attr_prefix(attr, value),
                         AFFECTED_BY_NON_COMMON_STYLE_AFFECTING_ATTRIBUTE_SELECTOR)
        }
        SimpleSelector::AttrSubstringMatch(ref attr, ref value) => {
            relation_if!(element.match_attr_substring(attr, value),
                         AFFECTED_BY_NON_COMMON_STYLE_AFFECTING_ATTRIBUTE_SELECTOR)
        }
        SimpleSelector::AttrSuffixMatch(ref attr, ref value) => {
            relation_if!(element.match_attr_suffix(attr, value),
                         AFFECTED_BY_NON_COMMON_STYLE_AFFECTING_ATTRIBUTE_SELECTOR)
        }
        SimpleSelector::AttrIncludesNeverMatch(..) |
        SimpleSelector::AttrPrefixNeverMatch(..) |
        SimpleSelector::AttrSubstringNeverMatch(..) |
        SimpleSelector::AttrSuffixNeverMatch(..) => {
            false
        }
        SimpleSelector::NonTSPseudoClass(ref pc) => {
            relation_if!(element.match_non_ts_pseudo_class(pc),
                         AFFECTED_BY_STATE)
        }
        SimpleSelector::FirstChild => {
            relation_if!(matches_first_child(element, reason), AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::LastChild => {
            relation_if!(matches_last_child(element, reason), AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::OnlyChild => {
            relation_if!(matches_first_child(element, reason) &&
                         matches_last_child(element, reason), AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::Root => {
            // We never share styles with an element with no parent, so no point
            // in creating a new StyleRelation.
            element.is_root()
        }
        SimpleSelector::Empty => {
            if reason.for_styling() {
                element.insert_flags(HAS_EMPTY_SELECTOR);
            }
            relation_if!(element.is_empty(), AFFECTED_BY_EMPTY)
        }
        SimpleSelector::NthChild(a, b) => {
            relation_if!(matches_generic_nth_child(element, a, b, false, false, reason),
                         AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::NthLastChild(a, b) => {
            relation_if!(matches_generic_nth_child(element, a, b, false, true, reason),
                         AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::NthOfType(a, b) => {
            relation_if!(matches_generic_nth_child(element, a, b, true, false, reason),
                         AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::NthLastOfType(a, b) => {
            relation_if!(matches_generic_nth_child(element, a, b, true, true, reason),
                         AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::FirstOfType => {
            relation_if!(matches_generic_nth_child(element, 0, 1, true, false, reason),
                         AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::LastOfType => {
            relation_if!(matches_generic_nth_child(element, 0, 1, true, true, reason),
                         AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::OnlyOfType => {
            relation_if!(matches_generic_nth_child(element, 0, 1, true, false, reason) &&
                         matches_generic_nth_child(element, 0, 1, true, true, reason),
                         AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::Negation(ref negated) => {
            !negated.iter().all(|s| {
                matches_complex_selector(s, element, parent_bf, relations, reason)
            })
        }
    }
}

#[inline]
fn matches_generic_nth_child<E>(element: &E,
                                a: i32,
                                b: i32,
                                is_of_type: bool,
                                is_from_end: bool,
                                reason: MatchingReason) -> bool
    where E: Element
{
    // Selectors Level 4 changed from Level 3:
    // This can match without a parent element:
    // https://drafts.csswg.org/selectors-4/#child-index

    if reason.for_styling() {
        if let Some(parent) = element.parent_element() {
            parent.insert_flags(if is_from_end {
                HAS_SLOW_SELECTOR
            } else {
                HAS_SLOW_SELECTOR_LATER_SIBLINGS
            });
        }
    }

    let mut index = 1;
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

    if a == 0 {
        b == index
    } else {
        (index - b) / a >= 0 &&
        (index - b) % a == 0
    }
}

#[inline]
fn matches_first_child<E>(element: &E, reason: MatchingReason) -> bool where E: Element {
    // Selectors Level 4 changed from Level 3:
    // This can match without a parent element:
    // https://drafts.csswg.org/selectors-4/#child-index
    if reason.for_styling() {
        if let Some(parent) = element.parent_element() {
            parent.insert_flags(HAS_EDGE_CHILD_SELECTOR);
        }
    }
    element.prev_sibling_element().is_none()
}

#[inline]
fn matches_last_child<E>(element: &E, reason: MatchingReason) -> bool where E: Element {
    // Selectors Level 4 changed from Level 3:
    // This can match without a parent element:
    // https://drafts.csswg.org/selectors-4/#child-index
    if reason.for_styling() {
        if let Some(parent) = element.parent_element() {
            parent.insert_flags(HAS_EDGE_CHILD_SELECTOR);
        }
    }

    element.next_sibling_element().is_none()
}
