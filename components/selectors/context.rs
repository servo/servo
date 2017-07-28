/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use attr::CaseSensitivity;
use bloom::BloomFilter;

/// What kind of selector matching mode we should use.
///
/// There are two modes of selector matching. The difference is only noticeable
/// in presence of pseudo-elements.
#[derive(Debug, PartialEq, Copy, Clone)]
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
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
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
#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug, HeapSizeOf)]
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
#[derive(Clone)]
pub struct MatchingContext<'a> {
    /// Input with the matching mode we should use when matching selectors.
    pub matching_mode: MatchingMode,
    /// Input with the bloom filter used to fast-reject selectors.
    pub bloom_filter: Option<&'a BloomFilter>,
    /// Input that controls how matching for links is handled.
    pub visited_handling: VisitedHandlingMode,
    /// Output that records whether we encountered a "relevant link" while
    /// matching _any_ selector for this element. (This differs from
    /// `RelevantLinkStatus` which tracks the status for the _current_ selector
    /// only.)
    pub relevant_link_found: bool,

    quirks_mode: QuirksMode,
    classes_and_ids_case_sensitivity: CaseSensitivity,
}

impl<'a> MatchingContext<'a> {
    /// Constructs a new `MatchingContext`.
    pub fn new(matching_mode: MatchingMode,
               bloom_filter: Option<&'a BloomFilter>,
               quirks_mode: QuirksMode)
               -> Self
    {
        Self {
            matching_mode: matching_mode,
            bloom_filter: bloom_filter,
            visited_handling: VisitedHandlingMode::AllLinksUnvisited,
            relevant_link_found: false,
            quirks_mode: quirks_mode,
            classes_and_ids_case_sensitivity: quirks_mode.classes_and_ids_case_sensitivity(),
        }
    }

    /// Constructs a new `MatchingContext` for use in visited matching.
    pub fn new_for_visited(matching_mode: MatchingMode,
                           bloom_filter: Option<&'a BloomFilter>,
                           visited_handling: VisitedHandlingMode,
                           quirks_mode: QuirksMode)
                           -> Self
    {
        Self {
            matching_mode: matching_mode,
            bloom_filter: bloom_filter,
            visited_handling: visited_handling,
            relevant_link_found: false,
            quirks_mode: quirks_mode,
            classes_and_ids_case_sensitivity: quirks_mode.classes_and_ids_case_sensitivity(),
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
