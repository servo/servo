/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use bloom::BloomFilter;
use parser::{CaseSensitivity, Combinator, ComplexSelector, Component, LocalName};
use parser::{Selector, SelectorInner, SelectorIter};
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
    pub flags StyleRelations: usize {
        /// Whether this element is affected by an ID selector.
        const AFFECTED_BY_ID_SELECTOR = 1 << 0,
        /// Whether this element has a style attribute. Computed
        /// externally.
        const AFFECTED_BY_STYLE_ATTRIBUTE = 1 << 1,
        /// Whether this element is affected by presentational hints. This is
        /// computed externally (that is, in Servo).
        const AFFECTED_BY_PRESENTATIONAL_HINTS = 1 << 2,
        /// Whether this element has pseudo-element styles. Computed externally.
        const AFFECTED_BY_PSEUDO_ELEMENTS = 1 << 3,
    }
}

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

pub fn matches_selector_list<E>(selector_list: &[Selector<E::Impl>],
                                element: &E,
                                parent_bf: Option<&BloomFilter>)
                                -> bool
    where E: Element
{
    selector_list.iter().any(|selector| {
        selector.pseudo_element.is_none() &&
        matches_selector(&selector.inner,
                         element,
                         parent_bf,
                         &mut StyleRelations::empty(),
                         &mut |_, _| {})
    })
}

fn may_match<E>(sel: &SelectorInner<E::Impl>,
                bf: &BloomFilter)
                -> bool
    where E: Element,
{
    // Check against the list of precomputed hashes.
    for hash in sel.ancestor_hashes.iter() {
        // If we hit the 0 sentinel hash, that means the rest are zero as well.
        if *hash == 0 {
            break;
        }

        if !bf.might_contain_hash(*hash) {
            return false;
        }
    }

    true
}

/// Determines whether the given element matches the given complex selector.
pub fn matches_selector<E, F>(selector: &SelectorInner<E::Impl>,
                              element: &E,
                              parent_bf: Option<&BloomFilter>,
                              relations: &mut StyleRelations,
                              flags_setter: &mut F)
                              -> bool
    where E: Element,
          F: FnMut(&E, ElementSelectorFlags),
{
    // Use the bloom filter to fast-reject.
    if let Some(filter) = parent_bf {
        if !may_match::<E>(selector, filter) {
            return false;
        }
    }

    // Match the selector.
    matches_complex_selector(&selector.complex, element, relations, flags_setter)
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

/// Matches a complex selector.
pub fn matches_complex_selector<E, F>(selector: &ComplexSelector<E::Impl>,
                                      element: &E,
                                      relations: &mut StyleRelations,
                                      flags_setter: &mut F)
                                      -> bool
     where E: Element,
           F: FnMut(&E, ElementSelectorFlags),
{
    match matches_complex_selector_internal(selector.iter(),
                                            element,
                                            relations,
                                            flags_setter) {
        SelectorMatchingResult::Matched => true,
        _ => false
    }
}

fn matches_complex_selector_internal<E, F>(mut selector_iter: SelectorIter<E::Impl>,
                                           element: &E,
                                           relations: &mut StyleRelations,
                                           flags_setter: &mut F)
                                           -> SelectorMatchingResult
     where E: Element,
           F: FnMut(&E, ElementSelectorFlags),
{
    let matches_all_simple_selectors = selector_iter.all(|simple| {
        matches_simple_selector(simple, element, relations, flags_setter)
    });

    let combinator = selector_iter.next_sequence();
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
                let result = matches_complex_selector_internal(selector_iter.clone(),
                                                               &element,
                                                               relations,
                                                               flags_setter);
                match (result, c) {
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
        selector: &Component<E::Impl>,
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
        Component::Combinator(_) => unreachable!(),
        Component::LocalName(LocalName { ref name, ref lower_name }) => {
            let name = if element.is_html_element_in_html_document() { lower_name } else { name };
            element.get_local_name() == name.borrow()
        }
        Component::Namespace(ref namespace) => {
            element.get_namespace() == namespace.url.borrow()
        }
        // TODO: case-sensitivity depends on the document type and quirks mode
        Component::ID(ref id) => {
            relation_if!(element.get_id().map_or(false, |attr| attr == *id),
                         AFFECTED_BY_ID_SELECTOR)
        }
        Component::Class(ref class) => {
            element.has_class(class)
        }
        Component::AttrExists(ref attr) => {
            element.match_attr_has(attr)
        }
        Component::AttrEqual(ref attr, ref value, case_sensitivity) => {
            match case_sensitivity {
                CaseSensitivity::CaseSensitive => element.match_attr_equals(attr, value),
                CaseSensitivity::CaseInsensitive => element.match_attr_equals_ignore_ascii_case(attr, value),
            }
        }
        Component::AttrIncludes(ref attr, ref value) => {
            element.match_attr_includes(attr, value)
        }
        Component::AttrDashMatch(ref attr, ref value) => {
            element.match_attr_dash(attr, value)
        }
        Component::AttrPrefixMatch(ref attr, ref value) => {
            element.match_attr_prefix(attr, value)
        }
        Component::AttrSubstringMatch(ref attr, ref value) => {
            element.match_attr_substring(attr, value)
        }
        Component::AttrSuffixMatch(ref attr, ref value) => {
            element.match_attr_suffix(attr, value)
        }
        Component::AttrIncludesNeverMatch(..) |
        Component::AttrPrefixNeverMatch(..) |
        Component::AttrSubstringNeverMatch(..) |
        Component::AttrSuffixNeverMatch(..) => {
            false
        }
        Component::NonTSPseudoClass(ref pc) => {
            element.match_non_ts_pseudo_class(pc, relations, flags_setter)
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
            // We never share styles with an element with no parent, so no point
            // in creating a new StyleRelation.
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
            !negated.iter().all(|s| {
                match matches_complex_selector_internal(s.iter(),
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
