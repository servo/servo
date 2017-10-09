/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use attr::CaseSensitivity;
use bloom::BloomFilter;
use nth_index_cache::NthIndexCache;
use tree::OpaqueElement;

/// What kind of selector matching mode we should use.
///
/// There are two modes of selector matching. The difference is only noticeable
/// in presence of pseudo-elements.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MatchingMode {
    /// Don't ignore any pseudo-element selectors.
    Normal,

    /// Ignores any stateless pseudo-element selectors in the rightmost sequence
    /// of simple selectors.
    ///
    /// This is useful, for example, to match against ::before when you aren't a
    /// pseudo-element yourself.
    ///
    /// For example, in presence of `::before:hover`, it would never match, but
    /// `::before` would be ignored as in "matching".
    ///
    /// It's required for all the selectors you match using this mode to have a
    /// pseudo-element.
    ForStatelessPseudoElement,
}

/// The mode to use when matching unvisited and visited links.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VisitedHandlingMode {
    /// All links are matched as if they are unvisted.
    AllLinksUnvisited,
    /// All links are matched as if they are visited and unvisited (both :link
    /// and :visited match).
    ///
    /// This is intended to be used from invalidation code, to be conservative
    /// about whether we need to restyle a link.
    AllLinksVisitedAndUnvisited,
    /// A element's "relevant link" is the element being matched if it is a link
    /// or the nearest ancestor link. The relevant link is matched as though it
    /// is visited, and all other links are matched as if they are unvisited.
    RelevantLinkVisited,
}

/// Which quirks mode is this document in.
///
/// See: https://quirks.spec.whatwg.org/
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum QuirksMode {
    /// Quirks mode.
    Quirks,
    /// Limited quirks mode.
    LimitedQuirks,
    /// No quirks mode.
    NoQuirks,
}

impl QuirksMode {
    #[inline]
    pub fn classes_and_ids_case_sensitivity(self) -> CaseSensitivity {
        match self {
            QuirksMode::NoQuirks |
            QuirksMode::LimitedQuirks => CaseSensitivity::CaseSensitive,
            QuirksMode::Quirks => CaseSensitivity::AsciiCaseInsensitive,
        }
    }
}

/// Data associated with the matching process for a element.  This context is
/// used across many selectors for an element, so it's not appropriate for
/// transient data that applies to only a single selector.
pub struct MatchingContext<'a> {
    /// Input with the matching mode we should use when matching selectors.
    pub matching_mode: MatchingMode,
    /// Input with the bloom filter used to fast-reject selectors.
    pub bloom_filter: Option<&'a BloomFilter>,
    /// An optional cache to speed up nth-index-like selectors.
    pub nth_index_cache: Option<&'a mut NthIndexCache>,
    /// Input that controls how matching for links is handled.
    pub visited_handling: VisitedHandlingMode,
    /// Output that records whether we encountered a "relevant link" while
    /// matching _any_ selector for this element. (This differs from
    /// `RelevantLinkStatus` which tracks the status for the _current_ selector
    /// only.)
    pub relevant_link_found: bool,

    /// The element which is going to match :scope pseudo-class. It can be
    /// either one :scope element, or the scoping element.
    ///
    /// Note that, although in theory there can be multiple :scope elements,
    /// in current specs, at most one is specified, and when there is one,
    /// scoping element is not relevant anymore, so we use a single method for
    /// them.
    ///
    /// When this is None, :scope will match the root element.
    ///
    /// See https://drafts.csswg.org/selectors-4/#scope-pseudo
    pub scope_element: Option<OpaqueElement>,

    quirks_mode: QuirksMode,
    classes_and_ids_case_sensitivity: CaseSensitivity,
}

impl<'a> MatchingContext<'a> {
    /// Constructs a new `MatchingContext`.
    pub fn new(
        matching_mode: MatchingMode,
        bloom_filter: Option<&'a BloomFilter>,
        nth_index_cache: Option<&'a mut NthIndexCache>,
        quirks_mode: QuirksMode,
    ) -> Self {
        Self::new_for_visited(
            matching_mode,
            bloom_filter,
            nth_index_cache,
            VisitedHandlingMode::AllLinksUnvisited,
            quirks_mode
        )
    }

    /// Constructs a new `MatchingContext` for use in visited matching.
    pub fn new_for_visited(
        matching_mode: MatchingMode,
        bloom_filter: Option<&'a BloomFilter>,
        nth_index_cache: Option<&'a mut NthIndexCache>,
        visited_handling: VisitedHandlingMode,
        quirks_mode: QuirksMode,
    ) -> Self {
        Self {
            matching_mode,
            bloom_filter,
            visited_handling,
            nth_index_cache,
            quirks_mode,
            relevant_link_found: false,
            classes_and_ids_case_sensitivity: quirks_mode.classes_and_ids_case_sensitivity(),
            scope_element: None,
        }
    }

    /// The quirks mode of the document.
    #[inline]
    pub fn quirks_mode(&self) -> QuirksMode {
        self.quirks_mode
    }

    /// The case-sensitivity for class and ID selectors
    #[inline]
    pub fn classes_and_ids_case_sensitivity(&self) -> CaseSensitivity {
        self.classes_and_ids_case_sensitivity
    }
}
