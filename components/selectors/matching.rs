/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use bloom::BloomFilter;
use parser::{CaseSensitivity, Combinator, ComplexSelector, LocalName};
use parser::{SimpleSelector, Selector};
use precomputed_hash::PrecomputedHash;
use std::borrow::Borrow;
use tree::Element;

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
    /// Set of flags that are set on either the element or its parent (depending
    /// on the flag) if the element could potentially match a selector.
    pub flags ElementSelectorFlags: u8 {
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

pub fn matches<E>(selector_list: &[Selector<E::Impl>],
                  element: &E,
                  parent_bf: Option<&BloomFilter>)
                  -> bool
    where E: Element
{
    selector_list.iter().any(|selector| {
        selector.pseudo_element.is_none() &&
        matches_complex_selector(&*selector.complex_selector,
                                 element,
                                 parent_bf,
                                 &mut StyleRelations::empty(),
                                 &mut |_, _| {})
    })
}

fn may_match<E>(mut selector: &ComplexSelector<E::Impl>,
                bf: &BloomFilter)
                -> bool
    where E: Element,
{
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
                    if !bf.might_contain_hash(name.precomputed_hash()) &&
                       !bf.might_contain_hash(lower_name.precomputed_hash()) {
                       return false
                    }
                },
                SimpleSelector::Namespace(ref namespace) => {
                    if !bf.might_contain_hash(namespace.url.precomputed_hash()) {
                        return false
                    }
                },
                SimpleSelector::ID(ref id) => {
                    if !bf.might_contain_hash(id.precomputed_hash()) {
                        return false
                    }
                },
                SimpleSelector::Class(ref class) => {
                    if !bf.might_contain_hash(class.precomputed_hash()) {
                        return false
                    }
                },
                _ => {},
            }
        }
    }

    // If we haven't proven otherwise, it may match.
    true
}

/// Determines whether the given element matches the given complex selector.
pub fn matches_complex_selector<E, F>(selector: &ComplexSelector<E::Impl>,
                                      element: &E,
                                      parent_bf: Option<&BloomFilter>,
                                      relations: &mut StyleRelations,
                                      flags_setter: &mut F)
                                      -> bool
    where E: Element,
          F: FnMut(&E, ElementSelectorFlags),
{
    if let Some(filter) = parent_bf {
        if !may_match::<E>(selector, filter) {
            return false;
        }
    }

    match matches_complex_selector_internal(selector,
                                            element,
                                            relations,
                                            flags_setter) {
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

fn matches_complex_selector_internal<E, F>(selector: &ComplexSelector<E::Impl>,
                                           element: &E,
                                           relations: &mut StyleRelations,
                                           flags_setter: &mut F)
                                           -> SelectorMatchingResult
     where E: Element,
           F: FnMut(&E, ElementSelectorFlags),
{
    let matches_all_simple_selectors = selector.compound_selector.iter().all(|simple| {
        matches_simple_selector(simple, element, relations, flags_setter)
    });

    let siblings = selector.next.as_ref().map_or(false, |&(_, combinator)| {
        matches!(combinator, Combinator::NextSibling | Combinator::LaterSibling)
    });

    if siblings {
        flags_setter(element, HAS_SLOW_SELECTOR_LATER_SIBLINGS);
    }

    if !matches_all_simple_selectors {
        return SelectorMatchingResult::NotMatchedAndRestartFromClosestLaterSibling;
    }

    match selector.next {
        None => SelectorMatchingResult::Matched,
        Some((ref next_selector, combinator)) => {
            let (mut next_element, candidate_not_found) = if siblings {
                (element.prev_sibling_element(),
                 SelectorMatchingResult::NotMatchedAndRestartFromClosestDescendant)
            } else {
                (element.parent_element(),
                 SelectorMatchingResult::NotMatchedGlobally)
            };

            loop {
                let element = match next_element {
                    None => return candidate_not_found,
                    Some(next_element) => next_element,
                };
                let result = matches_complex_selector_internal(&**next_selector,
                                                               &element,
                                                               relations,
                                                               flags_setter);
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
fn matches_simple_selector<E, F>(
        selector: &SimpleSelector<E::Impl>,
        element: &E,
        relations: &mut StyleRelations,
        flags_setter: &mut F)
        -> bool
    where E: Element,
          F: FnMut(&E, ElementSelectorFlags),
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
            element.match_attr_has(attr)
        }
        SimpleSelector::AttrEqual(ref attr, ref value, case_sensitivity) => {
            match case_sensitivity {
                CaseSensitivity::CaseSensitive => element.match_attr_equals(attr, value),
                CaseSensitivity::CaseInsensitive => element.match_attr_equals_ignore_ascii_case(attr, value),
            }
        }
        SimpleSelector::AttrIncludes(ref attr, ref value) => {
            element.match_attr_includes(attr, value)
        }
        SimpleSelector::AttrDashMatch(ref attr, ref value) => {
            element.match_attr_dash(attr, value)
        }
        SimpleSelector::AttrPrefixMatch(ref attr, ref value) => {
            element.match_attr_prefix(attr, value)
        }
        SimpleSelector::AttrSubstringMatch(ref attr, ref value) => {
            element.match_attr_substring(attr, value)
        }
        SimpleSelector::AttrSuffixMatch(ref attr, ref value) => {
            element.match_attr_suffix(attr, value)
        }
        SimpleSelector::AttrIncludesNeverMatch(..) |
        SimpleSelector::AttrPrefixNeverMatch(..) |
        SimpleSelector::AttrSubstringNeverMatch(..) |
        SimpleSelector::AttrSuffixNeverMatch(..) => {
            false
        }
        SimpleSelector::NonTSPseudoClass(ref pc) => {
            relation_if!(element.match_non_ts_pseudo_class(pc, relations, flags_setter),
                         AFFECTED_BY_STATE)
        }
        SimpleSelector::FirstChild => {
            relation_if!(matches_first_child(element, flags_setter),
                         AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::LastChild => {
            relation_if!(matches_last_child(element, flags_setter),
                         AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::OnlyChild => {
            relation_if!(matches_first_child(element, flags_setter) &&
                         matches_last_child(element, flags_setter),
                         AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::Root => {
            // We never share styles with an element with no parent, so no point
            // in creating a new StyleRelation.
            element.is_root()
        }
        SimpleSelector::Empty => {
            flags_setter(element, HAS_EMPTY_SELECTOR);
            relation_if!(element.is_empty(), AFFECTED_BY_EMPTY)
        }
        SimpleSelector::NthChild(a, b) => {
            relation_if!(matches_generic_nth_child(element, a, b, false, false, flags_setter),
                         AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::NthLastChild(a, b) => {
            relation_if!(matches_generic_nth_child(element, a, b, false, true, flags_setter),
                         AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::NthOfType(a, b) => {
            relation_if!(matches_generic_nth_child(element, a, b, true, false, flags_setter),
                         AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::NthLastOfType(a, b) => {
            relation_if!(matches_generic_nth_child(element, a, b, true, true, flags_setter),
                         AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::FirstOfType => {
            relation_if!(matches_generic_nth_child(element, 0, 1, true, false, flags_setter),
                         AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::LastOfType => {
            relation_if!(matches_generic_nth_child(element, 0, 1, true, true, flags_setter),
                         AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::OnlyOfType => {
            relation_if!(matches_generic_nth_child(element, 0, 1, true, false, flags_setter) &&
                         matches_generic_nth_child(element, 0, 1, true, true, flags_setter),
                         AFFECTED_BY_CHILD_INDEX)
        }
        SimpleSelector::Negation(ref negated) => {
            !negated.iter().all(|s| {
                match matches_complex_selector_internal(s,
                                                        element,
                                                        relations,
                                                        flags_setter) {
                    SelectorMatchingResult::Matched => true,
                    _ => false,
                }
            })
        }
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
