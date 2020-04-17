/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::attr::{AttrSelectorOperation, NamespaceConstraint, ParsedAttrSelectorOperation};
use crate::bloom::{BloomFilter, BLOOM_HASH_MASK};
use crate::nth_index_cache::NthIndexCacheInner;
use crate::parser::{AncestorHashes, Combinator, Component, LocalName};
use crate::parser::{NonTSPseudoClass, Selector, SelectorImpl, SelectorIter, SelectorList};
use crate::tree::Element;
use smallvec::SmallVec;
use std::borrow::Borrow;
use std::iter;

pub use crate::context::*;

// The bloom filter for descendant CSS selectors will have a <1% false
// positive rate until it has this many selectors in it, then it will
// rapidly increase.
pub static RECOMMENDED_SELECTOR_BLOOM_FILTER_SIZE: usize = 4096;

bitflags! {
    /// Set of flags that are set on either the element or its parent (depending
    /// on the flag) if the element could potentially match a selector.
    pub struct ElementSelectorFlags: usize {
        /// When a child is added or removed from the parent, all the children
        /// must be restyled, because they may match :nth-last-child,
        /// :last-of-type, :nth-last-of-type, or :only-of-type.
        const HAS_SLOW_SELECTOR = 1 << 0;

        /// When a child is added or removed from the parent, any later
        /// children must be restyled, because they may match :nth-child,
        /// :first-of-type, or :nth-of-type.
        const HAS_SLOW_SELECTOR_LATER_SIBLINGS = 1 << 1;

        /// When a child is added or removed from the parent, the first and
        /// last children must be restyled, because they may match :first-child,
        /// :last-child, or :only-child.
        const HAS_EDGE_CHILD_SELECTOR = 1 << 2;

        /// The element has an empty selector, so when a child is appended we
        /// might need to restyle the parent completely.
        const HAS_EMPTY_SELECTOR = 1 << 3;
    }
}

impl ElementSelectorFlags {
    /// Returns the subset of flags that apply to the element.
    pub fn for_self(self) -> ElementSelectorFlags {
        self & (ElementSelectorFlags::HAS_EMPTY_SELECTOR)
    }

    /// Returns the subset of flags that apply to the parent.
    pub fn for_parent(self) -> ElementSelectorFlags {
        self & (ElementSelectorFlags::HAS_SLOW_SELECTOR |
            ElementSelectorFlags::HAS_SLOW_SELECTOR_LATER_SIBLINGS |
            ElementSelectorFlags::HAS_EDGE_CHILD_SELECTOR)
    }
}

/// Holds per-compound-selector data.
struct LocalMatchingContext<'a, 'b: 'a, Impl: SelectorImpl> {
    shared: &'a mut MatchingContext<'b, Impl>,
    matches_hover_and_active_quirk: MatchesHoverAndActiveQuirk,
}

#[inline(always)]
pub fn matches_selector_list<E>(
    selector_list: &SelectorList<E::Impl>,
    element: &E,
    context: &mut MatchingContext<E::Impl>,
) -> bool
where
    E: Element,
{
    // This is pretty much any(..) but manually inlined because the compiler
    // refuses to do so from querySelector / querySelectorAll.
    for selector in &selector_list.0 {
        let matches = matches_selector(selector, 0, None, element, context, &mut |_, _| {});

        if matches {
            return true;
        }
    }

    false
}

#[inline(always)]
fn may_match(hashes: &AncestorHashes, bf: &BloomFilter) -> bool {
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
#[derive(Clone, Copy, Eq, PartialEq)]
enum SelectorMatchingResult {
    Matched,
    NotMatchedAndRestartFromClosestLaterSibling,
    NotMatchedAndRestartFromClosestDescendant,
    NotMatchedGlobally,
}

/// Whether the :hover and :active quirk applies.
///
/// https://quirks.spec.whatwg.org/#the-active-and-hover-quirk
#[derive(Clone, Copy, Debug, PartialEq)]
enum MatchesHoverAndActiveQuirk {
    Yes,
    No,
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
pub fn matches_selector<E, F>(
    selector: &Selector<E::Impl>,
    offset: usize,
    hashes: Option<&AncestorHashes>,
    element: &E,
    context: &mut MatchingContext<E::Impl>,
    flags_setter: &mut F,
) -> bool
where
    E: Element,
    F: FnMut(&E, ElementSelectorFlags),
{
    // Use the bloom filter to fast-reject.
    if let Some(hashes) = hashes {
        if let Some(filter) = context.bloom_filter {
            if !may_match(hashes, filter) {
                return false;
            }
        }
    }

    matches_complex_selector(selector.iter_from(offset), element, context, flags_setter)
}

/// Whether a compound selector matched, and whether it was the rightmost
/// selector inside the complex selector.
pub enum CompoundSelectorMatchingResult {
    /// The selector was fully matched.
    FullyMatched,
    /// The compound selector matched, and the next combinator offset is
    /// `next_combinator_offset`.
    Matched { next_combinator_offset: usize },
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
pub fn matches_compound_selector_from<E>(
    selector: &Selector<E::Impl>,
    mut from_offset: usize,
    context: &mut MatchingContext<E::Impl>,
    element: &E,
) -> CompoundSelectorMatchingResult
where
    E: Element,
{
    if cfg!(debug_assertions) && from_offset != 0 {
        selector.combinator_at_parse_order(from_offset - 1); // This asserts.
    }

    let mut local_context = LocalMatchingContext {
        shared: context,
        matches_hover_and_active_quirk: MatchesHoverAndActiveQuirk::No,
    };

    // Find the end of the selector or the next combinator, then match
    // backwards, so that we match in the same order as
    // matches_complex_selector, which is usually faster.
    let start_offset = from_offset;
    for component in selector.iter_raw_parse_order_from(from_offset) {
        if matches!(*component, Component::Combinator(..)) {
            debug_assert_ne!(from_offset, 0, "Selector started with a combinator?");
            break;
        }

        from_offset += 1;
    }

    debug_assert!(from_offset >= 1);
    debug_assert!(from_offset <= selector.len());

    let iter = selector.iter_from(selector.len() - from_offset);
    debug_assert!(
        iter.clone().next().is_some() ||
            (from_offset != selector.len() &&
                matches!(
                    selector.combinator_at_parse_order(from_offset),
                    Combinator::SlotAssignment | Combinator::PseudoElement
                )),
        "Got the math wrong: {:?} | {:?} | {} {}",
        selector,
        selector.iter_raw_match_order().as_slice(),
        from_offset,
        start_offset
    );

    for component in iter {
        if !matches_simple_selector(component, element, &mut local_context, &mut |_, _| {}) {
            return CompoundSelectorMatchingResult::NotMatched;
        }
    }

    if from_offset != selector.len() {
        return CompoundSelectorMatchingResult::Matched {
            next_combinator_offset: from_offset,
        };
    }

    CompoundSelectorMatchingResult::FullyMatched
}

/// Matches a complex selector.
#[inline(always)]
pub fn matches_complex_selector<E, F>(
    mut iter: SelectorIter<E::Impl>,
    element: &E,
    context: &mut MatchingContext<E::Impl>,
    flags_setter: &mut F,
) -> bool
where
    E: Element,
    F: FnMut(&E, ElementSelectorFlags),
{
    // If this is the special pseudo-element mode, consume the ::pseudo-element
    // before proceeding, since the caller has already handled that part.
    if context.matching_mode() == MatchingMode::ForStatelessPseudoElement && !context.is_nested() {
        // Consume the pseudo.
        match *iter.next().unwrap() {
            Component::PseudoElement(ref pseudo) => {
                if let Some(ref f) = context.pseudo_element_matching_fn {
                    if !f(pseudo) {
                        return false;
                    }
                }
            },
            _ => {
                debug_assert!(
                    false,
                    "Used MatchingMode::ForStatelessPseudoElement \
                     in a non-pseudo selector"
                );
            },
        }

        // The only other parser-allowed Component in this sequence is a state
        // class. We just don't match in that case.
        if let Some(s) = iter.next() {
            debug_assert!(
                matches!(*s, Component::NonTSPseudoClass(..)),
                "Someone messed up pseudo-element parsing"
            );
            return false;
        }

        // Advance to the non-pseudo-element part of the selector.
        let next_sequence = iter.next_sequence().unwrap();
        debug_assert_eq!(next_sequence, Combinator::PseudoElement);
    }

    let result =
        matches_complex_selector_internal(iter, element, context, flags_setter, Rightmost::Yes);

    match result {
        SelectorMatchingResult::Matched => true,
        _ => false,
    }
}

#[inline]
fn matches_hover_and_active_quirk<Impl: SelectorImpl>(
    selector_iter: &SelectorIter<Impl>,
    context: &MatchingContext<Impl>,
    rightmost: Rightmost,
) -> MatchesHoverAndActiveQuirk {
    if context.quirks_mode() != QuirksMode::Quirks {
        return MatchesHoverAndActiveQuirk::No;
    }

    if context.is_nested() {
        return MatchesHoverAndActiveQuirk::No;
    }

    // This compound selector had a pseudo-element to the right that we
    // intentionally skipped.
    if rightmost == Rightmost::Yes &&
        context.matching_mode() == MatchingMode::ForStatelessPseudoElement
    {
        return MatchesHoverAndActiveQuirk::No;
    }

    let all_match = selector_iter.clone().all(|simple| match *simple {
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
        Component::NonTSPseudoClass(ref pseudo_class) => pseudo_class.is_active_or_hover(),
        _ => true,
    });

    if all_match {
        MatchesHoverAndActiveQuirk::Yes
    } else {
        MatchesHoverAndActiveQuirk::No
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Rightmost {
    Yes,
    No,
}

#[inline(always)]
fn next_element_for_combinator<E>(
    element: &E,
    combinator: Combinator,
    selector: &SelectorIter<E::Impl>,
    context: &MatchingContext<E::Impl>,
) -> Option<E>
where
    E: Element,
{
    match combinator {
        Combinator::NextSibling | Combinator::LaterSibling => element.prev_sibling_element(),
        Combinator::Child | Combinator::Descendant => {
            match element.parent_element() {
                Some(e) => return Some(e),
                None => {},
            }

            if !element.parent_node_is_shadow_root() {
                return None;
            }

            // https://drafts.csswg.org/css-scoping/#host-element-in-tree:
            //
            //   For the purpose of Selectors, a shadow host also appears in
            //   its shadow tree, with the contents of the shadow tree treated
            //   as its children. (In other words, the shadow host is treated as
            //   replacing the shadow root node.)
            //
            // and also:
            //
            //   When considered within its own shadow trees, the shadow host is
            //   featureless. Only the :host, :host(), and :host-context()
            //   pseudo-classes are allowed to match it.
            //
            // Since we know that the parent is a shadow root, we necessarily
            // are in a shadow tree of the host, and the next selector will only
            // match if the selector is a featureless :host selector.
            if !selector.clone().is_featureless_host_selector() {
                return None;
            }

            element.containing_shadow_host()
        },
        Combinator::Part => element.containing_shadow_host(),
        Combinator::SlotAssignment => {
            debug_assert!(element
                .assigned_slot()
                .map_or(true, |s| s.is_html_slot_element()));
            let scope = context.current_host?;
            let mut current_slot = element.assigned_slot()?;
            while current_slot.containing_shadow_host().unwrap().opaque() != scope {
                current_slot = current_slot.assigned_slot()?;
            }
            Some(current_slot)
        },
        Combinator::PseudoElement => element.pseudo_element_originating_element(),
    }
}

fn matches_complex_selector_internal<E, F>(
    mut selector_iter: SelectorIter<E::Impl>,
    element: &E,
    context: &mut MatchingContext<E::Impl>,
    flags_setter: &mut F,
    rightmost: Rightmost,
) -> SelectorMatchingResult
where
    E: Element,
    F: FnMut(&E, ElementSelectorFlags),
{
    debug!(
        "Matching complex selector {:?} for {:?}",
        selector_iter, element
    );

    let matches_compound_selector = matches_compound_selector(
        &mut selector_iter,
        element,
        context,
        flags_setter,
        rightmost,
    );

    let combinator = selector_iter.next_sequence();
    if combinator.map_or(false, |c| c.is_sibling()) {
        flags_setter(
            element,
            ElementSelectorFlags::HAS_SLOW_SELECTOR_LATER_SIBLINGS,
        );
    }

    if !matches_compound_selector {
        return SelectorMatchingResult::NotMatchedAndRestartFromClosestLaterSibling;
    }

    let combinator = match combinator {
        None => return SelectorMatchingResult::Matched,
        Some(c) => c,
    };

    let candidate_not_found = match combinator {
        Combinator::NextSibling | Combinator::LaterSibling => {
            SelectorMatchingResult::NotMatchedAndRestartFromClosestDescendant
        },
        Combinator::Child |
        Combinator::Descendant |
        Combinator::SlotAssignment |
        Combinator::Part |
        Combinator::PseudoElement => SelectorMatchingResult::NotMatchedGlobally,
    };

    let mut next_element =
        next_element_for_combinator(element, combinator, &selector_iter, &context);

    // Stop matching :visited as soon as we find a link, or a combinator for
    // something that isn't an ancestor.
    let mut visited_handling = if element.is_link() || combinator.is_sibling() {
        VisitedHandlingMode::AllLinksUnvisited
    } else {
        context.visited_handling()
    };

    loop {
        let element = match next_element {
            None => return candidate_not_found,
            Some(next_element) => next_element,
        };

        let result = context.with_visited_handling_mode(visited_handling, |context| {
            matches_complex_selector_internal(
                selector_iter.clone(),
                &element,
                context,
                flags_setter,
                Rightmost::No,
            )
        });

        match (result, combinator) {
            // Return the status immediately.
            (SelectorMatchingResult::Matched, _) |
            (SelectorMatchingResult::NotMatchedGlobally, _) |
            (_, Combinator::NextSibling) => {
                return result;
            },

            // Upgrade the failure status to
            // NotMatchedAndRestartFromClosestDescendant.
            (_, Combinator::PseudoElement) | (_, Combinator::Child) => {
                return SelectorMatchingResult::NotMatchedAndRestartFromClosestDescendant;
            },

            // If the failure status is
            // NotMatchedAndRestartFromClosestDescendant and combinator is
            // Combinator::LaterSibling, give up this Combinator::LaterSibling
            // matching and restart from the closest descendant combinator.
            (
                SelectorMatchingResult::NotMatchedAndRestartFromClosestDescendant,
                Combinator::LaterSibling,
            ) => {
                return result;
            },

            // The Combinator::Descendant combinator and the status is
            // NotMatchedAndRestartFromClosestLaterSibling or
            // NotMatchedAndRestartFromClosestDescendant, or the
            // Combinator::LaterSibling combinator and the status is
            // NotMatchedAndRestartFromClosestDescendant, we can continue to
            // matching on the next candidate element.
            _ => {},
        }

        if element.is_link() {
            visited_handling = VisitedHandlingMode::AllLinksUnvisited;
        }

        next_element = next_element_for_combinator(&element, combinator, &selector_iter, &context);
    }
}

#[inline]
fn matches_local_name<E>(element: &E, local_name: &LocalName<E::Impl>) -> bool
where
    E: Element,
{
    let name = select_name(
        element.is_html_element_in_html_document(),
        &local_name.name,
        &local_name.lower_name,
    )
    .borrow();
    element.has_local_name(name)
}

/// Determines whether the given element matches the given compound selector.
#[inline]
fn matches_compound_selector<E, F>(
    selector_iter: &mut SelectorIter<E::Impl>,
    element: &E,
    context: &mut MatchingContext<E::Impl>,
    flags_setter: &mut F,
    rightmost: Rightmost,
) -> bool
where
    E: Element,
    F: FnMut(&E, ElementSelectorFlags),
{
    let matches_hover_and_active_quirk =
        matches_hover_and_active_quirk(&selector_iter, context, rightmost);

    // Handle some common cases first.
    // We may want to get rid of this at some point if we can make the
    // generic case fast enough.
    let mut selector = selector_iter.next();
    if let Some(&Component::LocalName(ref local_name)) = selector {
        if !matches_local_name(element, local_name) {
            return false;
        }
        selector = selector_iter.next();
    }
    let class_and_id_case_sensitivity = context.classes_and_ids_case_sensitivity();
    if let Some(&Component::ID(ref id)) = selector {
        if !element.has_id(id, class_and_id_case_sensitivity) {
            return false;
        }
        selector = selector_iter.next();
    }
    while let Some(&Component::Class(ref class)) = selector {
        if !element.has_class(class, class_and_id_case_sensitivity) {
            return false;
        }
        selector = selector_iter.next();
    }
    let selector = match selector {
        Some(s) => s,
        None => return true,
    };

    let mut local_context = LocalMatchingContext {
        shared: context,
        matches_hover_and_active_quirk,
    };
    iter::once(selector)
        .chain(selector_iter)
        .all(|simple| matches_simple_selector(simple, element, &mut local_context, flags_setter))
}

/// Determines whether the given element matches the given single selector.
fn matches_simple_selector<E, F>(
    selector: &Component<E::Impl>,
    element: &E,
    context: &mut LocalMatchingContext<E::Impl>,
    flags_setter: &mut F,
) -> bool
where
    E: Element,
    F: FnMut(&E, ElementSelectorFlags),
{
    debug_assert!(context.shared.is_nested() || !context.shared.in_negation());

    match *selector {
        Component::Combinator(_) => unreachable!(),
        Component::Part(ref parts) => {
            let mut hosts = SmallVec::<[E; 4]>::new();

            let mut host = match element.containing_shadow_host() {
                Some(h) => h,
                None => return false,
            };

            let current_host = context.shared.current_host;
            if current_host != Some(host.opaque()) {
                loop {
                    let outer_host = host.containing_shadow_host();
                    if outer_host.as_ref().map(|h| h.opaque()) == current_host {
                        break;
                    }
                    let outer_host = match outer_host {
                        Some(h) => h,
                        None => return false,
                    };
                    // TODO(emilio): if worth it, we could early return if
                    // host doesn't have the exportparts attribute.
                    hosts.push(host);
                    host = outer_host;
                }
            }

            // Translate the part into the right scope.
            parts.iter().all(|part| {
                let mut part = part.clone();
                for host in hosts.iter().rev() {
                    part = match host.imported_part(&part) {
                        Some(p) => p,
                        None => return false,
                    };
                }
                element.is_part(&part)
            })
        },
        Component::Slotted(ref selector) => {
            // <slots> are never flattened tree slottables.
            !element.is_html_slot_element() &&
                context.shared.nest(|context| {
                    matches_complex_selector(selector.iter(), element, context, flags_setter)
                })
        },
        Component::PseudoElement(ref pseudo) => {
            element.match_pseudo_element(pseudo, context.shared)
        },
        Component::LocalName(ref local_name) => matches_local_name(element, local_name),
        Component::ExplicitUniversalType | Component::ExplicitAnyNamespace => true,
        Component::Namespace(_, ref url) | Component::DefaultNamespace(ref url) => {
            element.has_namespace(&url.borrow())
        },
        Component::ExplicitNoNamespace => {
            let ns = crate::parser::namespace_empty_string::<E::Impl>();
            element.has_namespace(&ns.borrow())
        },
        Component::ID(ref id) => {
            element.has_id(id, context.shared.classes_and_ids_case_sensitivity())
        },
        Component::Class(ref class) => {
            element.has_class(class, context.shared.classes_and_ids_case_sensitivity())
        },
        Component::AttributeInNoNamespaceExists {
            ref local_name,
            ref local_name_lower,
        } => {
            let is_html = element.is_html_element_in_html_document();
            element.attr_matches(
                &NamespaceConstraint::Specific(&crate::parser::namespace_empty_string::<E::Impl>()),
                select_name(is_html, local_name, local_name_lower),
                &AttrSelectorOperation::Exists,
            )
        },
        Component::AttributeInNoNamespace {
            ref local_name,
            ref value,
            operator,
            case_sensitivity,
            never_matches,
        } => {
            if never_matches {
                return false;
            }
            let is_html = element.is_html_element_in_html_document();
            element.attr_matches(
                &NamespaceConstraint::Specific(&crate::parser::namespace_empty_string::<E::Impl>()),
                local_name,
                &AttrSelectorOperation::WithValue {
                    operator: operator,
                    case_sensitivity: case_sensitivity.to_unconditional(is_html),
                    expected_value: value,
                },
            )
        },
        Component::AttributeOther(ref attr_sel) => {
            if attr_sel.never_matches {
                return false;
            }
            let is_html = element.is_html_element_in_html_document();
            let empty_string;
            let namespace = match attr_sel.namespace() {
                Some(ns) => ns,
                None => {
                    empty_string = crate::parser::namespace_empty_string::<E::Impl>();
                    NamespaceConstraint::Specific(&empty_string)
                },
            };
            element.attr_matches(
                &namespace,
                select_name(is_html, &attr_sel.local_name, &attr_sel.local_name_lower),
                &match attr_sel.operation {
                    ParsedAttrSelectorOperation::Exists => AttrSelectorOperation::Exists,
                    ParsedAttrSelectorOperation::WithValue {
                        operator,
                        case_sensitivity,
                        ref expected_value,
                    } => AttrSelectorOperation::WithValue {
                        operator: operator,
                        case_sensitivity: case_sensitivity.to_unconditional(is_html),
                        expected_value: expected_value,
                    },
                },
            )
        },
        Component::NonTSPseudoClass(ref pc) => {
            if context.matches_hover_and_active_quirk == MatchesHoverAndActiveQuirk::Yes &&
                !context.shared.is_nested() &&
                pc.is_active_or_hover() &&
                !element.is_link()
            {
                return false;
            }

            element.match_non_ts_pseudo_class(pc, &mut context.shared, flags_setter)
        },
        Component::FirstChild => matches_first_child(element, flags_setter),
        Component::LastChild => matches_last_child(element, flags_setter),
        Component::OnlyChild => {
            matches_first_child(element, flags_setter) && matches_last_child(element, flags_setter)
        },
        Component::Root => element.is_root(),
        Component::Empty => {
            flags_setter(element, ElementSelectorFlags::HAS_EMPTY_SELECTOR);
            element.is_empty()
        },
        Component::Host(ref selector) => {
            context
                .shared
                .shadow_host()
                .map_or(false, |host| host == element.opaque()) &&
                selector.as_ref().map_or(true, |selector| {
                    context.shared.nest(|context| {
                        matches_complex_selector(selector.iter(), element, context, flags_setter)
                    })
                })
        },
        Component::Scope => match context.shared.scope_element {
            Some(ref scope_element) => element.opaque() == *scope_element,
            None => element.is_root(),
        },
        Component::NthChild(a, b) => {
            matches_generic_nth_child(element, context, a, b, false, false, flags_setter)
        },
        Component::NthLastChild(a, b) => {
            matches_generic_nth_child(element, context, a, b, false, true, flags_setter)
        },
        Component::NthOfType(a, b) => {
            matches_generic_nth_child(element, context, a, b, true, false, flags_setter)
        },
        Component::NthLastOfType(a, b) => {
            matches_generic_nth_child(element, context, a, b, true, true, flags_setter)
        },
        Component::FirstOfType => {
            matches_generic_nth_child(element, context, 0, 1, true, false, flags_setter)
        },
        Component::LastOfType => {
            matches_generic_nth_child(element, context, 0, 1, true, true, flags_setter)
        },
        Component::OnlyOfType => {
            matches_generic_nth_child(element, context, 0, 1, true, false, flags_setter) &&
                matches_generic_nth_child(element, context, 0, 1, true, true, flags_setter)
        },
        Component::Is(ref list) | Component::Where(ref list) => context.shared.nest(|context| {
            for selector in &**list {
                if matches_complex_selector(selector.iter(), element, context, flags_setter) {
                    return true;
                }
            }
            false
        }),
        Component::Negation(ref negated) => context.shared.nest_for_negation(|context| {
            let mut local_context = LocalMatchingContext {
                matches_hover_and_active_quirk: MatchesHoverAndActiveQuirk::No,
                shared: context,
            };
            !negated
                .iter()
                .all(|ss| matches_simple_selector(ss, element, &mut local_context, flags_setter))
        }),
    }
}

#[inline(always)]
fn select_name<'a, T>(is_html: bool, local_name: &'a T, local_name_lower: &'a T) -> &'a T {
    if is_html {
        local_name_lower
    } else {
        local_name
    }
}

#[inline]
fn matches_generic_nth_child<E, F>(
    element: &E,
    context: &mut LocalMatchingContext<E::Impl>,
    a: i32,
    b: i32,
    is_of_type: bool,
    is_from_end: bool,
    flags_setter: &mut F,
) -> bool
where
    E: Element,
    F: FnMut(&E, ElementSelectorFlags),
{
    if element.ignores_nth_child_selectors() {
        return false;
    }

    flags_setter(
        element,
        if is_from_end {
            ElementSelectorFlags::HAS_SLOW_SELECTOR
        } else {
            ElementSelectorFlags::HAS_SLOW_SELECTOR_LATER_SIBLINGS
        },
    );

    // Grab a reference to the appropriate cache.
    let mut cache = context
        .shared
        .nth_index_cache
        .as_mut()
        .map(|c| c.get(is_of_type, is_from_end));

    // Lookup or compute the index.
    let index = if let Some(i) = cache.as_mut().and_then(|c| c.lookup(element.opaque())) {
        i
    } else {
        let i = nth_child_index(
            element,
            is_of_type,
            is_from_end,
            cache.as_mut().map(|s| &mut **s),
        );
        cache.as_mut().map(|c| c.insert(element.opaque(), i));
        i
    };
    debug_assert_eq!(
        index,
        nth_child_index(element, is_of_type, is_from_end, None),
        "invalid cache"
    );

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
fn nth_child_index<E>(
    element: &E,
    is_of_type: bool,
    is_from_end: bool,
    mut cache: Option<&mut NthIndexCacheInner>,
) -> i32
where
    E: Element,
{
    // The traversal mostly processes siblings left to right. So when we walk
    // siblings to the right when computing NthLast/NthLastOfType we're unlikely
    // to get cache hits along the way. As such, we take the hit of walking the
    // siblings to the left checking the cache in the is_from_end case (this
    // matches what Gecko does). The indices-from-the-left is handled during the
    // regular look further below.
    if let Some(ref mut c) = cache {
        if is_from_end && !c.is_empty() {
            let mut index: i32 = 1;
            let mut curr = element.clone();
            while let Some(e) = curr.prev_sibling_element() {
                curr = e;
                if !is_of_type || element.is_same_type(&curr) {
                    if let Some(i) = c.lookup(curr.opaque()) {
                        return i - index;
                    }
                    index += 1;
                }
            }
        }
    }

    let mut index: i32 = 1;
    let mut curr = element.clone();
    let next = |e: E| {
        if is_from_end {
            e.next_sibling_element()
        } else {
            e.prev_sibling_element()
        }
    };
    while let Some(e) = next(curr) {
        curr = e;
        if !is_of_type || element.is_same_type(&curr) {
            // If we're computing indices from the left, check each element in the
            // cache. We handle the indices-from-the-right case at the top of this
            // function.
            if !is_from_end {
                if let Some(i) = cache.as_mut().and_then(|c| c.lookup(curr.opaque())) {
                    return i + index;
                }
            }
            index += 1;
        }
    }

    index
}

#[inline]
fn matches_first_child<E, F>(element: &E, flags_setter: &mut F) -> bool
where
    E: Element,
    F: FnMut(&E, ElementSelectorFlags),
{
    flags_setter(element, ElementSelectorFlags::HAS_EDGE_CHILD_SELECTOR);
    element.prev_sibling_element().is_none()
}

#[inline]
fn matches_last_child<E, F>(element: &E, flags_setter: &mut F) -> bool
where
    E: Element,
    F: FnMut(&E, ElementSelectorFlags),
{
    flags_setter(element, ElementSelectorFlags::HAS_EDGE_CHILD_SELECTOR);
    element.next_sibling_element().is_none()
}
