/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::attr::{AttrSelectorOperator, AttrSelectorWithOptionalNamespace};
use crate::attr::{NamespaceConstraint, ParsedAttrSelectorOperation, ParsedCaseSensitivity};
use crate::bloom::BLOOM_HASH_MASK;
use crate::builder::{
    relative_selector_list_specificity_and_flags, selector_list_specificity_and_flags,
    SelectorBuilder, SelectorFlags, Specificity, SpecificityAndFlags,
};
use crate::context::QuirksMode;
use crate::sink::Push;
use crate::visitor::SelectorListKind;
pub use crate::visitor::SelectorVisitor;
use bitflags::bitflags;
use cssparser::parse_nth;
use cssparser::{BasicParseError, BasicParseErrorKind, ParseError, ParseErrorKind};
use cssparser::{CowRcStr, Delimiter, SourceLocation};
use cssparser::{Parser as CssParser, ToCss, Token};
use precomputed_hash::PrecomputedHash;
use servo_arc::{HeaderWithLength, ThinArc, UniqueArc};
use smallvec::SmallVec;
use std::borrow::{Borrow, Cow};
use std::fmt::{self, Debug};
use std::iter::Rev;
use std::slice;

/// A trait that represents a pseudo-element.
pub trait PseudoElement: Sized + ToCss {
    /// The `SelectorImpl` this pseudo-element is used for.
    type Impl: SelectorImpl;

    /// Whether the pseudo-element supports a given state selector to the right
    /// of it.
    fn accepts_state_pseudo_classes(&self) -> bool {
        false
    }

    /// Whether this pseudo-element is valid after a ::slotted(..) pseudo.
    fn valid_after_slotted(&self) -> bool {
        false
    }
}

/// A trait that represents a pseudo-class.
pub trait NonTSPseudoClass: Sized + ToCss {
    /// The `SelectorImpl` this pseudo-element is used for.
    type Impl: SelectorImpl;

    /// Whether this pseudo-class is :active or :hover.
    fn is_active_or_hover(&self) -> bool;

    /// Whether this pseudo-class belongs to:
    ///
    /// https://drafts.csswg.org/selectors-4/#useraction-pseudos
    fn is_user_action_state(&self) -> bool;

    fn visit<V>(&self, _visitor: &mut V) -> bool
    where
        V: SelectorVisitor<Impl = Self::Impl>,
    {
        true
    }
}

/// Returns a Cow::Borrowed if `s` is already ASCII lowercase, and a
/// Cow::Owned if `s` had to be converted into ASCII lowercase.
fn to_ascii_lowercase(s: &str) -> Cow<str> {
    if let Some(first_uppercase) = s.bytes().position(|byte| byte >= b'A' && byte <= b'Z') {
        let mut string = s.to_owned();
        string[first_uppercase..].make_ascii_lowercase();
        string.into()
    } else {
        s.into()
    }
}

bitflags! {
    /// Flags that indicate at which point of parsing a selector are we.
    struct SelectorParsingState: u8 {
        /// Whether we should avoid adding default namespaces to selectors that
        /// aren't type or universal selectors.
        const SKIP_DEFAULT_NAMESPACE = 1 << 0;

        /// Whether we've parsed a ::slotted() pseudo-element already.
        ///
        /// If so, then we can only parse a subset of pseudo-elements, and
        /// whatever comes after them if so.
        const AFTER_SLOTTED = 1 << 1;
        /// Whether we've parsed a ::part() pseudo-element already.
        ///
        /// If so, then we can only parse a subset of pseudo-elements, and
        /// whatever comes after them if so.
        const AFTER_PART = 1 << 2;
        /// Whether we've parsed a pseudo-element (as in, an
        /// `Impl::PseudoElement` thus not accounting for `::slotted` or
        /// `::part`) already.
        ///
        /// If so, then other pseudo-elements and most other selectors are
        /// disallowed.
        const AFTER_PSEUDO_ELEMENT = 1 << 3;
        /// Whether we've parsed a non-stateful pseudo-element (again, as-in
        /// `Impl::PseudoElement`) already. If so, then other pseudo-classes are
        /// disallowed. If this flag is set, `AFTER_PSEUDO_ELEMENT` must be set
        /// as well.
        const AFTER_NON_STATEFUL_PSEUDO_ELEMENT = 1 << 4;

        /// Whether we are after any of the pseudo-like things.
        const AFTER_PSEUDO = Self::AFTER_PART.bits | Self::AFTER_SLOTTED.bits | Self::AFTER_PSEUDO_ELEMENT.bits;

        /// Whether we explicitly disallow combinators.
        const DISALLOW_COMBINATORS = 1 << 5;

        /// Whether we explicitly disallow pseudo-element-like things.
        const DISALLOW_PSEUDOS = 1 << 6;

        /// Whether we explicitly disallow relative selectors (i.e. `:has()`).
        const DISALLOW_RELATIVE_SELECTOR = 1 << 7;
    }
}

impl SelectorParsingState {
    #[inline]
    fn allows_pseudos(self) -> bool {
        // NOTE(emilio): We allow pseudos after ::part and such.
        !self.intersects(Self::AFTER_PSEUDO_ELEMENT | Self::DISALLOW_PSEUDOS)
    }

    #[inline]
    fn allows_slotted(self) -> bool {
        !self.intersects(Self::AFTER_PSEUDO | Self::DISALLOW_PSEUDOS)
    }

    #[inline]
    fn allows_part(self) -> bool {
        !self.intersects(Self::AFTER_PSEUDO | Self::DISALLOW_PSEUDOS)
    }

    // TODO(emilio): Maybe some of these should be allowed, but this gets us on
    // the safe side for now, matching previous behavior. Gotta be careful with
    // the ones like :-moz-any, which allow nested selectors but don't carry the
    // state, and so on.
    #[inline]
    fn allows_custom_functional_pseudo_classes(self) -> bool {
        !self.intersects(Self::AFTER_PSEUDO)
    }

    #[inline]
    fn allows_non_functional_pseudo_classes(self) -> bool {
        !self.intersects(Self::AFTER_SLOTTED | Self::AFTER_NON_STATEFUL_PSEUDO_ELEMENT)
    }

    #[inline]
    fn allows_tree_structural_pseudo_classes(self) -> bool {
        !self.intersects(Self::AFTER_PSEUDO)
    }

    #[inline]
    fn allows_combinators(self) -> bool {
        !self.intersects(Self::DISALLOW_COMBINATORS)
    }
}

pub type SelectorParseError<'i> = ParseError<'i, SelectorParseErrorKind<'i>>;

#[derive(Clone, Debug, PartialEq)]
pub enum SelectorParseErrorKind<'i> {
    NoQualifiedNameInAttributeSelector(Token<'i>),
    EmptySelector,
    DanglingCombinator,
    NonCompoundSelector,
    NonPseudoElementAfterSlotted,
    InvalidPseudoElementAfterSlotted,
    InvalidPseudoElementInsideWhere,
    InvalidState,
    UnexpectedTokenInAttributeSelector(Token<'i>),
    PseudoElementExpectedColon(Token<'i>),
    PseudoElementExpectedIdent(Token<'i>),
    NoIdentForPseudo(Token<'i>),
    UnsupportedPseudoClassOrElement(CowRcStr<'i>),
    UnexpectedIdent(CowRcStr<'i>),
    ExpectedNamespace(CowRcStr<'i>),
    ExpectedBarInAttr(Token<'i>),
    BadValueInAttr(Token<'i>),
    InvalidQualNameInAttr(Token<'i>),
    ExplicitNamespaceUnexpectedToken(Token<'i>),
    ClassNeedsIdent(Token<'i>),
}

macro_rules! with_all_bounds {
    (
        [ $( $InSelector: tt )* ]
        [ $( $CommonBounds: tt )* ]
        [ $( $FromStr: tt )* ]
    ) => {
        /// This trait allows to define the parser implementation in regards
        /// of pseudo-classes/elements
        ///
        /// NB: We need Clone so that we can derive(Clone) on struct with that
        /// are parameterized on SelectorImpl. See
        /// <https://github.com/rust-lang/rust/issues/26925>
        pub trait SelectorImpl: Clone + Debug + Sized + 'static {
            type ExtraMatchingData<'a>: Sized + Default;
            type AttrValue: $($InSelector)*;
            type Identifier: $($InSelector)*;
            type LocalName: $($InSelector)* + Borrow<Self::BorrowedLocalName>;
            type NamespaceUrl: $($CommonBounds)* + Default + Borrow<Self::BorrowedNamespaceUrl>;
            type NamespacePrefix: $($InSelector)* + Default;
            type BorrowedNamespaceUrl: ?Sized + Eq;
            type BorrowedLocalName: ?Sized + Eq;

            /// non tree-structural pseudo-classes
            /// (see: https://drafts.csswg.org/selectors/#structural-pseudos)
            type NonTSPseudoClass: $($CommonBounds)* + NonTSPseudoClass<Impl = Self>;

            /// pseudo-elements
            type PseudoElement: $($CommonBounds)* + PseudoElement<Impl = Self>;

            /// Whether attribute hashes should be collected for filtering
            /// purposes.
            fn should_collect_attr_hash(_name: &Self::LocalName) -> bool {
                false
            }
        }
    }
}

macro_rules! with_bounds {
    ( [ $( $CommonBounds: tt )* ] [ $( $FromStr: tt )* ]) => {
        with_all_bounds! {
            [$($CommonBounds)* + $($FromStr)* + ToCss]
            [$($CommonBounds)*]
            [$($FromStr)*]
        }
    }
}

with_bounds! {
    [Clone + Eq]
    [for<'a> From<&'a str>]
}

pub trait Parser<'i> {
    type Impl: SelectorImpl;
    type Error: 'i + From<SelectorParseErrorKind<'i>>;

    /// Whether to parse the `::slotted()` pseudo-element.
    fn parse_slotted(&self) -> bool {
        false
    }

    /// Whether to parse the `::part()` pseudo-element.
    fn parse_part(&self) -> bool {
        false
    }

    /// Whether to parse the selector list of nth-child() or nth-last-child().
    fn parse_nth_child_of(&self) -> bool {
        false
    }

    /// Whether to parse the `:where` pseudo-class.
    fn parse_is_and_where(&self) -> bool {
        false
    }

    /// Whether to parse the :has pseudo-class.
    fn parse_has(&self) -> bool {
        false
    }

    /// Whether to parse the '&' delimiter as a parent selector.
    fn parse_parent_selector(&self) -> bool {
        false
    }

    /// Whether the given function name is an alias for the `:is()` function.
    fn is_is_alias(&self, _name: &str) -> bool {
        false
    }

    /// Whether to parse the `:host` pseudo-class.
    fn parse_host(&self) -> bool {
        false
    }

    /// Whether to allow forgiving selector-list parsing.
    fn allow_forgiving_selectors(&self) -> bool {
        true
    }

    /// This function can return an "Err" pseudo-element in order to support CSS2.1
    /// pseudo-elements.
    fn parse_non_ts_pseudo_class(
        &self,
        location: SourceLocation,
        name: CowRcStr<'i>,
    ) -> Result<<Self::Impl as SelectorImpl>::NonTSPseudoClass, ParseError<'i, Self::Error>> {
        Err(
            location.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(
                name,
            )),
        )
    }

    fn parse_non_ts_functional_pseudo_class<'t>(
        &self,
        name: CowRcStr<'i>,
        arguments: &mut CssParser<'i, 't>,
    ) -> Result<<Self::Impl as SelectorImpl>::NonTSPseudoClass, ParseError<'i, Self::Error>> {
        Err(
            arguments.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(
                name,
            )),
        )
    }

    fn parse_pseudo_element(
        &self,
        location: SourceLocation,
        name: CowRcStr<'i>,
    ) -> Result<<Self::Impl as SelectorImpl>::PseudoElement, ParseError<'i, Self::Error>> {
        Err(
            location.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(
                name,
            )),
        )
    }

    fn parse_functional_pseudo_element<'t>(
        &self,
        name: CowRcStr<'i>,
        arguments: &mut CssParser<'i, 't>,
    ) -> Result<<Self::Impl as SelectorImpl>::PseudoElement, ParseError<'i, Self::Error>> {
        Err(
            arguments.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(
                name,
            )),
        )
    }

    fn default_namespace(&self) -> Option<<Self::Impl as SelectorImpl>::NamespaceUrl> {
        None
    }

    fn namespace_for_prefix(
        &self,
        _prefix: &<Self::Impl as SelectorImpl>::NamespacePrefix,
    ) -> Option<<Self::Impl as SelectorImpl>::NamespaceUrl> {
        None
    }
}

#[derive(Clone, Debug, Eq, PartialEq, ToShmem)]
#[shmem(no_bounds)]
pub struct SelectorList<Impl: SelectorImpl>(
    #[shmem(field_bound)] pub SmallVec<[Selector<Impl>; 1]>,
);

/// Whether or not we're using forgiving parsing mode
enum ForgivingParsing {
    /// Discard the entire selector list upon encountering any invalid selector.
    /// This is the default behavior for almost all of CSS.
    No,
    /// Ignore invalid selectors, potentially creating an empty selector list.
    ///
    /// This is the error recovery mode of :is() and :where()
    Yes,
}

/// Flag indicating if we're parsing relative selectors.
#[derive(Copy, Clone, PartialEq)]
enum ParseRelative {
    /// Expect selectors to start with a combinator, assuming descendant combinator if not present.
    Yes,
    /// Treat as parse error if any selector begins with a combinator.
    No,
}

impl<Impl: SelectorImpl> SelectorList<Impl> {
    /// Returns a selector list with a single `&`
    pub fn ampersand() -> Self {
        Self(smallvec::smallvec![Selector::ampersand()])
    }

    /// Parse a comma-separated list of Selectors.
    /// <https://drafts.csswg.org/selectors/#grouping>
    ///
    /// Return the Selectors or Err if there is an invalid selector.
    pub fn parse<'i, 't, P>(
        parser: &P,
        input: &mut CssParser<'i, 't>,
    ) -> Result<Self, ParseError<'i, P::Error>>
    where
        P: Parser<'i, Impl = Impl>,
    {
        Self::parse_with_state(
            parser,
            input,
            SelectorParsingState::empty(),
            ForgivingParsing::No,
            ParseRelative::No,
        )
    }

    #[inline]
    fn parse_with_state<'i, 't, P>(
        parser: &P,
        input: &mut CssParser<'i, 't>,
        state: SelectorParsingState,
        recovery: ForgivingParsing,
        parse_relative: ParseRelative,
    ) -> Result<Self, ParseError<'i, P::Error>>
    where
        P: Parser<'i, Impl = Impl>,
    {
        let mut values = SmallVec::new();
        loop {
            let selector = input.parse_until_before(Delimiter::Comma, |i| {
                parse_selector(parser, i, state, parse_relative)
            });

            let was_ok = selector.is_ok();
            match selector {
                Ok(selector) => values.push(selector),
                Err(err) => match recovery {
                    ForgivingParsing::No => return Err(err),
                    ForgivingParsing::Yes => {
                        if !parser.allow_forgiving_selectors() {
                            return Err(err);
                        }
                    },
                },
            }

            loop {
                match input.next() {
                    Err(_) => return Ok(SelectorList(values)),
                    Ok(&Token::Comma) => break,
                    Ok(_) => {
                        debug_assert!(!was_ok, "Shouldn't have got a selector if getting here");
                    },
                }
            }
        }
    }

    /// Creates a SelectorList from a Vec of selectors. Used in tests.
    #[allow(dead_code)]
    pub(crate) fn from_vec(v: Vec<Selector<Impl>>) -> Self {
        SelectorList(SmallVec::from_vec(v))
    }
}

/// Parses one compound selector suitable for nested stuff like :-moz-any, etc.
fn parse_inner_compound_selector<'i, 't, P, Impl>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
    state: SelectorParsingState,
) -> Result<Selector<Impl>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl = Impl>,
    Impl: SelectorImpl,
{
    parse_selector(
        parser,
        input,
        state | SelectorParsingState::DISALLOW_PSEUDOS | SelectorParsingState::DISALLOW_COMBINATORS,
        ParseRelative::No,
    )
}

/// Ancestor hashes for the bloom filter. We precompute these and store them
/// inline with selectors to optimize cache performance during matching.
/// This matters a lot.
///
/// We use 4 hashes, which is copied from Gecko, who copied it from WebKit.
/// Note that increasing the number of hashes here will adversely affect the
/// cache hit when fast-rejecting long lists of Rules with inline hashes.
///
/// Because the bloom filter only uses the bottom 24 bits of the hash, we pack
/// the fourth hash into the upper bits of the first three hashes in order to
/// shrink Rule (whose size matters a lot). This scheme minimizes the runtime
/// overhead of the packing for the first three hashes (we just need to mask
/// off the upper bits) at the expense of making the fourth somewhat more
/// complicated to assemble, because we often bail out before checking all the
/// hashes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AncestorHashes {
    pub packed_hashes: [u32; 3],
}

fn collect_ancestor_hashes<Impl: SelectorImpl>(
    iter: SelectorIter<Impl>,
    quirks_mode: QuirksMode,
    hashes: &mut [u32; 4],
    len: &mut usize,
) -> bool
where
    Impl::Identifier: PrecomputedHash,
    Impl::LocalName: PrecomputedHash,
    Impl::NamespaceUrl: PrecomputedHash,
{
    for component in AncestorIter::new(iter) {
        let hash = match *component {
            Component::LocalName(LocalName {
                ref name,
                ref lower_name,
            }) => {
                // Only insert the local-name into the filter if it's all
                // lowercase.  Otherwise we would need to test both hashes, and
                // our data structures aren't really set up for that.
                if name != lower_name {
                    continue;
                }
                name.precomputed_hash()
            },
            Component::DefaultNamespace(ref url) | Component::Namespace(_, ref url) => {
                url.precomputed_hash()
            },
            // In quirks mode, class and id selectors should match
            // case-insensitively, so just avoid inserting them into the filter.
            Component::ID(ref id) if quirks_mode != QuirksMode::Quirks => id.precomputed_hash(),
            Component::Class(ref class) if quirks_mode != QuirksMode::Quirks => {
                class.precomputed_hash()
            },
            Component::AttributeInNoNamespace { ref local_name, .. }
                if Impl::should_collect_attr_hash(local_name) =>
            {
                // AttributeInNoNamespace is only used when local_name ==
                // local_name_lower.
                local_name.precomputed_hash()
            },
            Component::AttributeInNoNamespaceExists {
                ref local_name,
                ref local_name_lower,
                ..
            } => {
                // Only insert the local-name into the filter if it's all
                // lowercase.  Otherwise we would need to test both hashes, and
                // our data structures aren't really set up for that.
                if local_name != local_name_lower || !Impl::should_collect_attr_hash(local_name) {
                    continue;
                }
                local_name.precomputed_hash()
            },
            Component::AttributeOther(ref selector) => {
                if selector.local_name != selector.local_name_lower ||
                    !Impl::should_collect_attr_hash(&selector.local_name)
                {
                    continue;
                }
                selector.local_name.precomputed_hash()
            },
            Component::Is(ref list) | Component::Where(ref list) => {
                // :where and :is OR their selectors, so we can't put any hash
                // in the filter if there's more than one selector, as that'd
                // exclude elements that may match one of the other selectors.
                if list.len() == 1 &&
                    !collect_ancestor_hashes(list[0].iter(), quirks_mode, hashes, len)
                {
                    return false;
                }
                continue;
            },
            _ => continue,
        };

        hashes[*len] = hash & BLOOM_HASH_MASK;
        *len += 1;
        if *len == hashes.len() {
            return false;
        }
    }
    true
}

impl AncestorHashes {
    pub fn new<Impl: SelectorImpl>(selector: &Selector<Impl>, quirks_mode: QuirksMode) -> Self
    where
        Impl::Identifier: PrecomputedHash,
        Impl::LocalName: PrecomputedHash,
        Impl::NamespaceUrl: PrecomputedHash,
    {
        // Compute ancestor hashes for the bloom filter.
        let mut hashes = [0u32; 4];
        let mut len = 0;
        collect_ancestor_hashes(selector.iter(), quirks_mode, &mut hashes, &mut len);
        debug_assert!(len <= 4);

        // Now, pack the fourth hash (if it exists) into the upper byte of each of
        // the other three hashes.
        if len == 4 {
            let fourth = hashes[3];
            hashes[0] |= (fourth & 0x000000ff) << 24;
            hashes[1] |= (fourth & 0x0000ff00) << 16;
            hashes[2] |= (fourth & 0x00ff0000) << 8;
        }

        AncestorHashes {
            packed_hashes: [hashes[0], hashes[1], hashes[2]],
        }
    }

    /// Returns the fourth hash, reassembled from parts.
    pub fn fourth_hash(&self) -> u32 {
        ((self.packed_hashes[0] & 0xff000000) >> 24) |
            ((self.packed_hashes[1] & 0xff000000) >> 16) |
            ((self.packed_hashes[2] & 0xff000000) >> 8)
    }
}

#[inline]
pub fn namespace_empty_string<Impl: SelectorImpl>() -> Impl::NamespaceUrl {
    // Rust type’s default, not default namespace
    Impl::NamespaceUrl::default()
}

/// A Selector stores a sequence of simple selectors and combinators. The
/// iterator classes allow callers to iterate at either the raw sequence level or
/// at the level of sequences of simple selectors separated by combinators. Most
/// callers want the higher-level iterator.
///
/// We store compound selectors internally right-to-left (in matching order).
/// Additionally, we invert the order of top-level compound selectors so that
/// each one matches left-to-right. This is because matching namespace, local name,
/// id, and class are all relatively cheap, whereas matching pseudo-classes might
/// be expensive (depending on the pseudo-class). Since authors tend to put the
/// pseudo-classes on the right, it's faster to start matching on the left.
///
/// This reordering doesn't change the semantics of selector matching, and we
/// handle it in to_css to make it invisible to serialization.
#[derive(Clone, Eq, PartialEq, ToShmem)]
#[shmem(no_bounds)]
pub struct Selector<Impl: SelectorImpl>(
    #[shmem(field_bound)] ThinArc<SpecificityAndFlags, Component<Impl>>,
);

impl<Impl: SelectorImpl> Selector<Impl> {
    /// See Arc::mark_as_intentionally_leaked
    pub fn mark_as_intentionally_leaked(&self) {
        self.0.with_arc(|a| a.mark_as_intentionally_leaked())
    }

    fn ampersand() -> Self {
        Self(ThinArc::from_header_and_iter(
            SpecificityAndFlags {
                specificity: 0,
                flags: SelectorFlags::HAS_PARENT,
            },
            std::iter::once(Component::ParentSelector),
        ))
    }

    #[inline]
    pub fn specificity(&self) -> u32 {
        self.0.header.header.specificity()
    }

    #[inline]
    fn flags(&self) -> SelectorFlags {
        self.0.header.header.flags
    }

    #[inline]
    pub fn has_pseudo_element(&self) -> bool {
        self.0.header.header.has_pseudo_element()
    }

    #[inline]
    pub fn has_parent_selector(&self) -> bool {
        self.0.header.header.has_parent_selector()
    }

    #[inline]
    pub fn is_slotted(&self) -> bool {
        self.0.header.header.is_slotted()
    }

    #[inline]
    pub fn is_part(&self) -> bool {
        self.0.header.header.is_part()
    }

    #[inline]
    pub fn parts(&self) -> Option<&[Impl::Identifier]> {
        if !self.is_part() {
            return None;
        }

        let mut iter = self.iter();
        if self.has_pseudo_element() {
            // Skip the pseudo-element.
            for _ in &mut iter {}

            let combinator = iter.next_sequence()?;
            debug_assert_eq!(combinator, Combinator::PseudoElement);
        }

        for component in iter {
            if let Component::Part(ref part) = *component {
                return Some(part);
            }
        }

        debug_assert!(false, "is_part() lied somehow?");
        None
    }

    #[inline]
    pub fn pseudo_element(&self) -> Option<&Impl::PseudoElement> {
        if !self.has_pseudo_element() {
            return None;
        }

        for component in self.iter() {
            if let Component::PseudoElement(ref pseudo) = *component {
                return Some(pseudo);
            }
        }

        debug_assert!(false, "has_pseudo_element lied!");
        None
    }

    /// Whether this selector (pseudo-element part excluded) matches every element.
    ///
    /// Used for "pre-computed" pseudo-elements in components/style/stylist.rs
    #[inline]
    pub fn is_universal(&self) -> bool {
        self.iter_raw_match_order().all(|c| {
            matches!(
                *c,
                Component::ExplicitUniversalType |
                    Component::ExplicitAnyNamespace |
                    Component::Combinator(Combinator::PseudoElement) |
                    Component::PseudoElement(..)
            )
        })
    }

    /// Returns an iterator over this selector in matching order (right-to-left).
    /// When a combinator is reached, the iterator will return None, and
    /// next_sequence() may be called to continue to the next sequence.
    #[inline]
    pub fn iter(&self) -> SelectorIter<Impl> {
        SelectorIter {
            iter: self.iter_raw_match_order(),
            next_combinator: None,
        }
    }

    /// Same as `iter()`, but skips `RelativeSelectorAnchor` and its associated combinator.
    #[inline]
    pub fn iter_skip_relative_selector_anchor(&self) -> SelectorIter<Impl> {
        if cfg!(debug_assertions) {
            let mut selector_iter = self.iter_raw_parse_order_from(0);
            assert!(
                matches!(
                    selector_iter.next().unwrap(),
                    Component::RelativeSelectorAnchor
                ),
                "Relative selector does not start with RelativeSelectorAnchor"
            );
            assert!(
                selector_iter.next().unwrap().is_combinator(),
                "Relative combinator does not exist"
            );
        }

        SelectorIter {
            iter: self.0.slice[..self.len() - 2].iter(),
            next_combinator: None,
        }
    }

    /// Whether this selector is a featureless :host selector, with no
    /// combinators to the left, and optionally has a pseudo-element to the
    /// right.
    #[inline]
    pub fn is_featureless_host_selector_or_pseudo_element(&self) -> bool {
        let mut iter = self.iter();
        if !self.has_pseudo_element() {
            return iter.is_featureless_host_selector();
        }

        // Skip the pseudo-element.
        for _ in &mut iter {}

        match iter.next_sequence() {
            None => return false,
            Some(combinator) => {
                debug_assert_eq!(combinator, Combinator::PseudoElement);
            },
        }

        iter.is_featureless_host_selector()
    }

    /// Returns an iterator over this selector in matching order (right-to-left),
    /// skipping the rightmost |offset| Components.
    #[inline]
    pub fn iter_from(&self, offset: usize) -> SelectorIter<Impl> {
        let iter = self.0.slice[offset..].iter();
        SelectorIter {
            iter,
            next_combinator: None,
        }
    }

    /// Returns the combinator at index `index` (zero-indexed from the right),
    /// or panics if the component is not a combinator.
    #[inline]
    pub fn combinator_at_match_order(&self, index: usize) -> Combinator {
        match self.0.slice[index] {
            Component::Combinator(c) => c,
            ref other => panic!(
                "Not a combinator: {:?}, {:?}, index: {}",
                other, self, index
            ),
        }
    }

    /// Returns an iterator over the entire sequence of simple selectors and
    /// combinators, in matching order (from right to left).
    #[inline]
    pub fn iter_raw_match_order(&self) -> slice::Iter<Component<Impl>> {
        self.0.slice.iter()
    }

    /// Returns the combinator at index `index` (zero-indexed from the left),
    /// or panics if the component is not a combinator.
    #[inline]
    pub fn combinator_at_parse_order(&self, index: usize) -> Combinator {
        match self.0.slice[self.len() - index - 1] {
            Component::Combinator(c) => c,
            ref other => panic!(
                "Not a combinator: {:?}, {:?}, index: {}",
                other, self, index
            ),
        }
    }

    /// Returns an iterator over the sequence of simple selectors and
    /// combinators, in parse order (from left to right), starting from
    /// `offset`.
    #[inline]
    pub fn iter_raw_parse_order_from(&self, offset: usize) -> Rev<slice::Iter<Component<Impl>>> {
        self.0.slice[..self.len() - offset].iter().rev()
    }

    /// Creates a Selector from a vec of Components, specified in parse order. Used in tests.
    #[allow(dead_code)]
    pub(crate) fn from_vec(
        vec: Vec<Component<Impl>>,
        specificity: u32,
        flags: SelectorFlags,
    ) -> Self {
        let mut builder = SelectorBuilder::default();
        for component in vec.into_iter() {
            if let Some(combinator) = component.as_combinator() {
                builder.push_combinator(combinator);
            } else {
                builder.push_simple_selector(component);
            }
        }
        let spec = SpecificityAndFlags { specificity, flags };
        Selector(builder.build_with_specificity_and_flags(spec))
    }

    pub fn replace_parent_selector(&self, parent: &[Selector<Impl>]) -> Self {
        // FIXME(emilio): Shouldn't allow replacing if parent has a pseudo-element selector
        // or what not.
        let flags = self.flags() - SelectorFlags::HAS_PARENT;
        let mut specificity = Specificity::from(self.specificity());
        let parent_specificity =
            Specificity::from(selector_list_specificity_and_flags(parent.iter()).specificity());

        // The specificity at this point will be wrong, we replace it by the correct one after the
        // fact.
        let specificity_and_flags = SpecificityAndFlags {
            specificity: self.specificity(),
            flags,
        };

        fn replace_parent_on_selector_list<Impl: SelectorImpl>(
            orig: &[Selector<Impl>],
            parent: &[Selector<Impl>],
            specificity: &mut Specificity,
            with_specificity: bool,
        ) -> Vec<Selector<Impl>> {
            let mut any = false;

            let result = orig
                .iter()
                .map(|s| {
                    if !s.has_parent_selector() {
                        return s.clone();
                    }
                    any = true;
                    s.replace_parent_selector(parent)
                })
                .collect();

            if !any || !with_specificity {
                return result;
            }

            *specificity += Specificity::from(
                selector_list_specificity_and_flags(result.iter()).specificity -
                    selector_list_specificity_and_flags(orig.iter()).specificity,
            );
            result
        }

        fn replace_parent_on_relative_selector_list<Impl: SelectorImpl>(
            orig: &[RelativeSelector<Impl>],
            parent: &[Selector<Impl>],
            specificity: &mut Specificity,
        ) -> Vec<RelativeSelector<Impl>> {
            let mut any = false;

            let result = orig
                .iter()
                .map(|s| {
                    if !s.selector.has_parent_selector() {
                        return s.clone();
                    }
                    any = true;
                    RelativeSelector {
                        match_hint: s.match_hint,
                        selector: s.selector.replace_parent_selector(parent),
                    }
                })
                .collect();

            if !any {
                return result;
            }

            *specificity += Specificity::from(
                relative_selector_list_specificity_and_flags(&result).specificity -
                    relative_selector_list_specificity_and_flags(orig).specificity,
            );
            result
        }

        fn replace_parent_on_selector<Impl: SelectorImpl>(
            orig: &Selector<Impl>,
            parent: &[Selector<Impl>],
            specificity: &mut Specificity,
        ) -> Selector<Impl> {
            if !orig.has_parent_selector() {
                return orig.clone();
            }
            let new_selector = orig.replace_parent_selector(parent);
            *specificity += Specificity::from(new_selector.specificity() - orig.specificity());
            new_selector
        }

        let mut items = if !self.has_parent_selector() {
            // Implicit `&` plus descendant combinator.
            let iter = self.iter_raw_match_order();
            let len = iter.len() + 2;
            specificity += parent_specificity;
            let iter = iter
                .cloned()
                .chain(std::iter::once(Component::Combinator(
                    Combinator::Descendant,
                )))
                .chain(std::iter::once(Component::Is(
                    parent.to_vec().into_boxed_slice(),
                )));
            let header = HeaderWithLength::new(specificity_and_flags, len);
            UniqueArc::from_header_and_iter_with_size(header, iter, len)
        } else {
            let iter = self.iter_raw_match_order().map(|component| {
                use self::Component::*;
                match *component {
                    LocalName(..) |
                    ID(..) |
                    Class(..) |
                    AttributeInNoNamespaceExists { .. } |
                    AttributeInNoNamespace { .. } |
                    AttributeOther(..) |
                    ExplicitUniversalType |
                    ExplicitAnyNamespace |
                    ExplicitNoNamespace |
                    DefaultNamespace(..) |
                    Namespace(..) |
                    Root |
                    Empty |
                    Scope |
                    Nth(..) |
                    NonTSPseudoClass(..) |
                    PseudoElement(..) |
                    Combinator(..) |
                    Host(None) |
                    Part(..) |
                    RelativeSelectorAnchor => component.clone(),
                    ParentSelector => {
                        specificity += parent_specificity;
                        Is(parent.to_vec().into_boxed_slice())
                    },
                    Negation(ref selectors) => {
                        Negation(
                            replace_parent_on_selector_list(
                                selectors,
                                parent,
                                &mut specificity,
                                /* with_specificity = */ true,
                            )
                            .into_boxed_slice(),
                        )
                    },
                    Is(ref selectors) => {
                        Is(replace_parent_on_selector_list(
                            selectors,
                            parent,
                            &mut specificity,
                            /* with_specificity = */ true,
                        )
                        .into_boxed_slice())
                    },
                    Where(ref selectors) => {
                        Where(
                            replace_parent_on_selector_list(
                                selectors,
                                parent,
                                &mut specificity,
                                /* with_specificity = */ false,
                            )
                            .into_boxed_slice(),
                        )
                    },
                    Has(ref selectors) => Has(replace_parent_on_relative_selector_list(
                        selectors,
                        parent,
                        &mut specificity,
                    )
                    .into_boxed_slice()),

                    Host(Some(ref selector)) => Host(Some(replace_parent_on_selector(
                        selector,
                        parent,
                        &mut specificity,
                    ))),
                    NthOf(ref data) => {
                        let selectors = replace_parent_on_selector_list(
                            data.selectors(),
                            parent,
                            &mut specificity,
                            /* with_specificity = */ true,
                        );
                        NthOf(NthOfSelectorData::new(
                            data.nth_data(),
                            selectors.into_iter(),
                        ))
                    },
                    Slotted(ref selector) => Slotted(replace_parent_on_selector(
                        selector,
                        parent,
                        &mut specificity,
                    )),
                }
            });
            let header = HeaderWithLength::new(specificity_and_flags, iter.len());
            UniqueArc::from_header_and_iter(header, iter)
        };
        items.header_mut().specificity = specificity.into();
        Selector(items.shareable_thin())
    }

    /// Returns count of simple selectors and combinators in the Selector.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.slice.len()
    }

    /// Returns the address on the heap of the ThinArc for memory reporting.
    pub fn thin_arc_heap_ptr(&self) -> *const ::std::os::raw::c_void {
        self.0.heap_ptr()
    }

    /// Traverse selector components inside `self`.
    ///
    /// Implementations of this method should call `SelectorVisitor` methods
    /// or other impls of `Visit` as appropriate based on the fields of `Self`.
    ///
    /// A return value of `false` indicates terminating the traversal.
    /// It should be propagated with an early return.
    /// On the contrary, `true` indicates that all fields of `self` have been traversed:
    ///
    /// ```rust,ignore
    /// if !visitor.visit_simple_selector(&self.some_simple_selector) {
    ///     return false;
    /// }
    /// if !self.some_component.visit(visitor) {
    ///     return false;
    /// }
    /// true
    /// ```
    pub fn visit<V>(&self, visitor: &mut V) -> bool
    where
        V: SelectorVisitor<Impl = Impl>,
    {
        let mut current = self.iter();
        let mut combinator = None;
        loop {
            if !visitor.visit_complex_selector(combinator) {
                return false;
            }

            for selector in &mut current {
                if !selector.visit(visitor) {
                    return false;
                }
            }

            combinator = current.next_sequence();
            if combinator.is_none() {
                break;
            }
        }

        true
    }
}

#[derive(Clone)]
pub struct SelectorIter<'a, Impl: 'a + SelectorImpl> {
    iter: slice::Iter<'a, Component<Impl>>,
    next_combinator: Option<Combinator>,
}

impl<'a, Impl: 'a + SelectorImpl> SelectorIter<'a, Impl> {
    /// Prepares this iterator to point to the next sequence to the left,
    /// returning the combinator if the sequence was found.
    #[inline]
    pub fn next_sequence(&mut self) -> Option<Combinator> {
        self.next_combinator.take()
    }

    /// Whether this selector is a featureless host selector, with no
    /// combinators to the left.
    #[inline]
    pub(crate) fn is_featureless_host_selector(&mut self) -> bool {
        self.selector_length() > 0 &&
            self.all(|component| component.is_host()) &&
            self.next_sequence().is_none()
    }

    #[inline]
    pub(crate) fn matches_for_stateless_pseudo_element(&mut self) -> bool {
        let first = match self.next() {
            Some(c) => c,
            // Note that this is the common path that we keep inline: the
            // pseudo-element not having anything to its right.
            None => return true,
        };
        self.matches_for_stateless_pseudo_element_internal(first)
    }

    #[inline(never)]
    fn matches_for_stateless_pseudo_element_internal(&mut self, first: &Component<Impl>) -> bool {
        if !first.matches_for_stateless_pseudo_element() {
            return false;
        }
        for component in self {
            // The only other parser-allowed Components in this sequence are
            // state pseudo-classes, or one of the other things that can contain
            // them.
            if !component.matches_for_stateless_pseudo_element() {
                return false;
            }
        }
        true
    }

    /// Returns remaining count of the simple selectors and combinators in the Selector.
    #[inline]
    pub fn selector_length(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, Impl: SelectorImpl> Iterator for SelectorIter<'a, Impl> {
    type Item = &'a Component<Impl>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(
            self.next_combinator.is_none(),
            "You should call next_sequence!"
        );
        match *self.iter.next()? {
            Component::Combinator(c) => {
                self.next_combinator = Some(c);
                None
            },
            ref x => Some(x),
        }
    }
}

impl<'a, Impl: SelectorImpl> fmt::Debug for SelectorIter<'a, Impl> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let iter = self.iter.clone().rev();
        for component in iter {
            component.to_css(f)?
        }
        Ok(())
    }
}

/// An iterator over all combinators in a selector. Does not traverse selectors within psuedoclasses.
struct CombinatorIter<'a, Impl: 'a + SelectorImpl>(SelectorIter<'a, Impl>);
impl<'a, Impl: 'a + SelectorImpl> CombinatorIter<'a, Impl> {
    fn new(inner: SelectorIter<'a, Impl>) -> Self {
        let mut result = CombinatorIter(inner);
        result.consume_non_combinators();
        result
    }

    fn consume_non_combinators(&mut self) {
        while self.0.next().is_some() {}
    }
}

impl<'a, Impl: SelectorImpl> Iterator for CombinatorIter<'a, Impl> {
    type Item = Combinator;
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.0.next_sequence();
        self.consume_non_combinators();
        result
    }
}

/// An iterator over all simple selectors belonging to ancestors.
struct AncestorIter<'a, Impl: 'a + SelectorImpl>(SelectorIter<'a, Impl>);
impl<'a, Impl: 'a + SelectorImpl> AncestorIter<'a, Impl> {
    /// Creates an AncestorIter. The passed-in iterator is assumed to point to
    /// the beginning of the child sequence, which will be skipped.
    fn new(inner: SelectorIter<'a, Impl>) -> Self {
        let mut result = AncestorIter(inner);
        result.skip_until_ancestor();
        result
    }

    /// Skips a sequence of simple selectors and all subsequent sequences until
    /// a non-pseudo-element ancestor combinator is reached.
    fn skip_until_ancestor(&mut self) {
        loop {
            while self.0.next().is_some() {}
            // If this is ever changed to stop at the "pseudo-element"
            // combinator, we will need to fix the way we compute hashes for
            // revalidation selectors.
            if self.0.next_sequence().map_or(true, |x| {
                matches!(x, Combinator::Child | Combinator::Descendant)
            }) {
                break;
            }
        }
    }
}

impl<'a, Impl: SelectorImpl> Iterator for AncestorIter<'a, Impl> {
    type Item = &'a Component<Impl>;
    fn next(&mut self) -> Option<Self::Item> {
        // Grab the next simple selector in the sequence if available.
        let next = self.0.next();
        if next.is_some() {
            return next;
        }

        // See if there are more sequences. If so, skip any non-ancestor sequences.
        if let Some(combinator) = self.0.next_sequence() {
            if !matches!(combinator, Combinator::Child | Combinator::Descendant) {
                self.skip_until_ancestor();
            }
        }

        self.0.next()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ToShmem)]
pub enum Combinator {
    Child,        //  >
    Descendant,   // space
    NextSibling,  // +
    LaterSibling, // ~
    /// A dummy combinator we use to the left of pseudo-elements.
    ///
    /// It serializes as the empty string, and acts effectively as a child
    /// combinator in most cases.  If we ever actually start using a child
    /// combinator for this, we will need to fix up the way hashes are computed
    /// for revalidation selectors.
    PseudoElement,
    /// Another combinator used for ::slotted(), which represent the jump from
    /// a node to its assigned slot.
    SlotAssignment,
    /// Another combinator used for `::part()`, which represents the jump from
    /// the part to the containing shadow host.
    Part,
}

impl Combinator {
    /// Returns true if this combinator is a child or descendant combinator.
    #[inline]
    pub fn is_ancestor(&self) -> bool {
        matches!(
            *self,
            Combinator::Child |
                Combinator::Descendant |
                Combinator::PseudoElement |
                Combinator::SlotAssignment
        )
    }

    /// Returns true if this combinator is a pseudo-element combinator.
    #[inline]
    pub fn is_pseudo_element(&self) -> bool {
        matches!(*self, Combinator::PseudoElement)
    }

    /// Returns true if this combinator is a next- or later-sibling combinator.
    #[inline]
    pub fn is_sibling(&self) -> bool {
        matches!(*self, Combinator::NextSibling | Combinator::LaterSibling)
    }
}

/// An enum for the different types of :nth- pseudoclasses
#[derive(Copy, Clone, Eq, PartialEq, ToShmem)]
#[shmem(no_bounds)]
pub enum NthType {
    Child,
    LastChild,
    OnlyChild,
    OfType,
    LastOfType,
    OnlyOfType,
}

impl NthType {
    pub fn is_only(self) -> bool {
        self == Self::OnlyChild || self == Self::OnlyOfType
    }

    pub fn is_of_type(self) -> bool {
        self == Self::OfType || self == Self::LastOfType || self == Self::OnlyOfType
    }

    pub fn is_from_end(self) -> bool {
        self == Self::LastChild || self == Self::LastOfType
    }
}

/// The properties that comprise an :nth- pseudoclass as of Selectors 3 (e.g.,
/// nth-child(An+B)).
/// https://www.w3.org/TR/selectors-3/#nth-child-pseudo
#[derive(Copy, Clone, Eq, PartialEq, ToShmem)]
#[shmem(no_bounds)]
pub struct NthSelectorData {
    pub ty: NthType,
    pub is_function: bool,
    pub a: i32,
    pub b: i32,
}

impl NthSelectorData {
    /// Returns selector data for :only-{child,of-type}
    #[inline]
    pub const fn only(of_type: bool) -> Self {
        Self {
            ty: if of_type {
                NthType::OnlyOfType
            } else {
                NthType::OnlyChild
            },
            is_function: false,
            a: 0,
            b: 1,
        }
    }

    /// Returns selector data for :first-{child,of-type}
    #[inline]
    pub const fn first(of_type: bool) -> Self {
        Self {
            ty: if of_type {
                NthType::OfType
            } else {
                NthType::Child
            },
            is_function: false,
            a: 0,
            b: 1,
        }
    }

    /// Returns selector data for :last-{child,of-type}
    #[inline]
    pub const fn last(of_type: bool) -> Self {
        Self {
            ty: if of_type {
                NthType::LastOfType
            } else {
                NthType::LastChild
            },
            is_function: false,
            a: 0,
            b: 1,
        }
    }

    /// Writes the beginning of the selector.
    #[inline]
    fn write_start<W: fmt::Write>(&self, dest: &mut W) -> fmt::Result {
        dest.write_str(match self.ty {
            NthType::Child if self.is_function => ":nth-child(",
            NthType::Child => ":first-child",
            NthType::LastChild if self.is_function => ":nth-last-child(",
            NthType::LastChild => ":last-child",
            NthType::OfType if self.is_function => ":nth-of-type(",
            NthType::OfType => ":first-of-type",
            NthType::LastOfType if self.is_function => ":nth-last-of-type(",
            NthType::LastOfType => ":last-of-type",
            NthType::OnlyChild => ":only-child",
            NthType::OnlyOfType => ":only-of-type",
        })
    }

    /// Serialize <an+b> (part of the CSS Syntax spec, but currently only used here).
    /// <https://drafts.csswg.org/css-syntax-3/#serialize-an-anb-value>
    #[inline]
    fn write_affine<W: fmt::Write>(&self, dest: &mut W) -> fmt::Result {
        match (self.a, self.b) {
            (0, 0) => dest.write_char('0'),

            (1, 0) => dest.write_char('n'),
            (-1, 0) => dest.write_str("-n"),
            (_, 0) => write!(dest, "{}n", self.a),

            (0, _) => write!(dest, "{}", self.b),
            (1, _) => write!(dest, "n{:+}", self.b),
            (-1, _) => write!(dest, "-n{:+}", self.b),
            (_, _) => write!(dest, "{}n{:+}", self.a, self.b),
        }
    }
}

/// The properties that comprise an :nth- pseudoclass as of Selectors 4 (e.g.,
/// nth-child(An+B [of S]?)).
/// https://www.w3.org/TR/selectors-4/#nth-child-pseudo
#[derive(Clone, Eq, PartialEq, ToShmem)]
#[shmem(no_bounds)]
pub struct NthOfSelectorData<Impl: SelectorImpl>(
    #[shmem(field_bound)] ThinArc<NthSelectorData, Selector<Impl>>,
);

impl<Impl: SelectorImpl> NthOfSelectorData<Impl> {
    /// Returns selector data for :nth-{,last-}{child,of-type}(An+B [of S])
    #[inline]
    pub fn new<I>(nth_data: &NthSelectorData, selectors: I) -> Self
    where
        I: Iterator<Item = Selector<Impl>> + ExactSizeIterator,
    {
        Self(ThinArc::from_header_and_iter(*nth_data, selectors))
    }

    /// Returns the An+B part of the selector
    #[inline]
    pub fn nth_data(&self) -> &NthSelectorData {
        &self.0.header.header
    }

    /// Returns the selector list part of the selector
    #[inline]
    pub fn selectors(&self) -> &[Selector<Impl>] {
        &self.0.slice
    }
}

/// Flag indicating where a given relative selector's match would be contained.
#[derive(Clone, Copy, Eq, PartialEq, ToShmem)]
pub enum RelativeSelectorMatchHint {
    /// Within this element's subtree.
    InSubtree,
    /// Within this element's direct children.
    InChild,
    /// This element's next sibling.
    InNextSibling,
    /// Within this element's next sibling's subtree.
    InNextSiblingSubtree,
    /// Within this element's subsequent siblings.
    InSibling,
    /// Across this element's subsequent siblings and their subtrees.
    InSiblingSubtree,
}

/// Storage for a relative selector.
#[derive(Clone, Eq, PartialEq, ToShmem)]
#[shmem(no_bounds)]
pub struct RelativeSelector<Impl: SelectorImpl> {
    /// Match space constraining hint.
    pub match_hint: RelativeSelectorMatchHint,
    /// The selector. Guaranteed to contain `RelativeSelectorAnchor` and the relative combinator in parse order.
    #[shmem(field_bound)]
    pub selector: Selector<Impl>,
}

bitflags! {
    /// Composition of combinators in a given selector, not traversing selectors of pseudoclasses.
    struct CombinatorComposition: u8 {
        const DESCENDANTS = 1 << 0;
        const SIBLINGS = 1 << 1;
    }
}

impl CombinatorComposition {
    fn for_relative_selector<Impl: SelectorImpl>(inner_selector: &Selector<Impl>) -> Self {
        let mut result = CombinatorComposition::empty();
        for combinator in CombinatorIter::new(inner_selector.iter_skip_relative_selector_anchor()) {
            match combinator {
                Combinator::Descendant | Combinator::Child => {
                    result.insert(Self::DESCENDANTS);
                },
                Combinator::NextSibling | Combinator::LaterSibling => {
                    result.insert(Self::SIBLINGS);
                },
                Combinator::Part | Combinator::PseudoElement | Combinator::SlotAssignment => {
                    continue
                },
            };
            if result.is_all() {
                break;
            }
        }
        return result;
    }
}

impl<Impl: SelectorImpl> RelativeSelector<Impl> {
    fn from_selector_list(selector_list: SelectorList<Impl>) -> Box<[Self]> {
        let vec: Vec<Self> = selector_list
            .0
            .into_iter()
            .map(|selector| {
                // It's more efficient to keep track of all this during the parse time, but that seems like a lot of special
                // case handling for what it's worth.
                if cfg!(debug_assertions) {
                    let relative_selector_anchor = selector.iter_raw_parse_order_from(0).next();
                    debug_assert!(
                        relative_selector_anchor.is_some(),
                        "Relative selector is empty"
                    );
                    debug_assert!(
                        matches!(
                            relative_selector_anchor.unwrap(),
                            Component::RelativeSelectorAnchor
                        ),
                        "Relative selector anchor is missing"
                    );
                }
                // Leave a hint for narrowing down the search space when we're matching.
                let match_hint = match selector.combinator_at_parse_order(1) {
                    Combinator::Descendant => RelativeSelectorMatchHint::InSubtree,
                    Combinator::Child => {
                        let composition = CombinatorComposition::for_relative_selector(&selector);
                        if composition.is_empty() || composition == CombinatorComposition::SIBLINGS
                        {
                            RelativeSelectorMatchHint::InChild
                        } else {
                            // Technically, for any composition that consists of child combinators only,
                            // the search space is depth-constrained, but it's probably not worth optimizing for.
                            RelativeSelectorMatchHint::InSubtree
                        }
                    },
                    Combinator::NextSibling => {
                        let composition = CombinatorComposition::for_relative_selector(&selector);
                        if composition.is_empty() {
                            RelativeSelectorMatchHint::InNextSibling
                        } else if composition == CombinatorComposition::SIBLINGS {
                            RelativeSelectorMatchHint::InSibling
                        } else if composition == CombinatorComposition::DESCENDANTS {
                            // Match won't cross multiple siblings.
                            RelativeSelectorMatchHint::InNextSiblingSubtree
                        } else {
                            RelativeSelectorMatchHint::InSiblingSubtree
                        }
                    },
                    Combinator::LaterSibling => {
                        let composition = CombinatorComposition::for_relative_selector(&selector);
                        if composition.is_empty() || composition == CombinatorComposition::SIBLINGS
                        {
                            RelativeSelectorMatchHint::InSibling
                        } else {
                            // Even if the match may not cross multiple siblings, we have to look until
                            // we find a match anyway.
                            RelativeSelectorMatchHint::InSiblingSubtree
                        }
                    },
                    Combinator::Part | Combinator::PseudoElement | Combinator::SlotAssignment => {
                        debug_assert!(false, "Unexpected relative combinator");
                        RelativeSelectorMatchHint::InSubtree
                    },
                };
                RelativeSelector {
                    match_hint,
                    selector,
                }
            })
            .collect();
        vec.into_boxed_slice()
    }
}

/// A CSS simple selector or combinator. We store both in the same enum for
/// optimal packing and cache performance, see [1].
///
/// [1] https://bugzilla.mozilla.org/show_bug.cgi?id=1357973
#[derive(Clone, Eq, PartialEq, ToShmem)]
#[shmem(no_bounds)]
pub enum Component<Impl: SelectorImpl> {
    LocalName(LocalName<Impl>),

    ID(#[shmem(field_bound)] Impl::Identifier),
    Class(#[shmem(field_bound)] Impl::Identifier),

    AttributeInNoNamespaceExists {
        #[shmem(field_bound)]
        local_name: Impl::LocalName,
        local_name_lower: Impl::LocalName,
    },
    // Used only when local_name is already lowercase.
    AttributeInNoNamespace {
        local_name: Impl::LocalName,
        operator: AttrSelectorOperator,
        #[shmem(field_bound)]
        value: Impl::AttrValue,
        case_sensitivity: ParsedCaseSensitivity,
    },
    // Use a Box in the less common cases with more data to keep size_of::<Component>() small.
    AttributeOther(Box<AttrSelectorWithOptionalNamespace<Impl>>),

    ExplicitUniversalType,
    ExplicitAnyNamespace,

    ExplicitNoNamespace,
    DefaultNamespace(#[shmem(field_bound)] Impl::NamespaceUrl),
    Namespace(
        #[shmem(field_bound)] Impl::NamespacePrefix,
        #[shmem(field_bound)] Impl::NamespaceUrl,
    ),

    /// Pseudo-classes
    Negation(Box<[Selector<Impl>]>),
    Root,
    Empty,
    Scope,
    ParentSelector,
    Nth(NthSelectorData),
    NthOf(NthOfSelectorData<Impl>),
    NonTSPseudoClass(#[shmem(field_bound)] Impl::NonTSPseudoClass),
    /// The ::slotted() pseudo-element:
    ///
    /// https://drafts.csswg.org/css-scoping/#slotted-pseudo
    ///
    /// The selector here is a compound selector, that is, no combinators.
    ///
    /// NOTE(emilio): This should support a list of selectors, but as of this
    /// writing no other browser does, and that allows them to put ::slotted()
    /// in the rule hash, so we do that too.
    ///
    /// See https://github.com/w3c/csswg-drafts/issues/2158
    Slotted(Selector<Impl>),
    /// The `::part` pseudo-element.
    ///   https://drafts.csswg.org/css-shadow-parts/#part
    Part(#[shmem(field_bound)] Box<[Impl::Identifier]>),
    /// The `:host` pseudo-class:
    ///
    /// https://drafts.csswg.org/css-scoping/#host-selector
    ///
    /// NOTE(emilio): This should support a list of selectors, but as of this
    /// writing no other browser does, and that allows them to put :host()
    /// in the rule hash, so we do that too.
    ///
    /// See https://github.com/w3c/csswg-drafts/issues/2158
    Host(Option<Selector<Impl>>),
    /// The `:where` pseudo-class.
    ///
    /// https://drafts.csswg.org/selectors/#zero-matches
    ///
    /// The inner argument is conceptually a SelectorList, but we move the
    /// selectors to the heap to keep Component small.
    Where(Box<[Selector<Impl>]>),
    /// The `:is` pseudo-class.
    ///
    /// https://drafts.csswg.org/selectors/#matches-pseudo
    ///
    /// Same comment as above re. the argument.
    Is(Box<[Selector<Impl>]>),
    /// The `:has` pseudo-class.
    ///
    /// https://drafts.csswg.org/selectors/#has-pseudo
    ///
    /// Same comment as above re. the argument.
    Has(Box<[RelativeSelector<Impl>]>),
    /// An implementation-dependent pseudo-element selector.
    PseudoElement(#[shmem(field_bound)] Impl::PseudoElement),

    Combinator(Combinator),

    /// Used only for relative selectors, which starts with a combinator
    /// (With an implied descendant combinator if not specified).
    ///
    /// https://drafts.csswg.org/selectors-4/#typedef-relative-selector
    RelativeSelectorAnchor,
}

impl<Impl: SelectorImpl> Component<Impl> {
    /// Returns true if this is a combinator.
    #[inline]
    pub fn is_combinator(&self) -> bool {
        matches!(*self, Component::Combinator(_))
    }

    /// Returns true if this is a :host() selector.
    #[inline]
    pub fn is_host(&self) -> bool {
        matches!(*self, Component::Host(..))
    }

    /// Returns the value as a combinator if applicable, None otherwise.
    pub fn as_combinator(&self) -> Option<Combinator> {
        match *self {
            Component::Combinator(c) => Some(c),
            _ => None,
        }
    }

    /// Whether this component is valid after a pseudo-element. Only intended
    /// for sanity-checking.
    pub fn maybe_allowed_after_pseudo_element(&self) -> bool {
        match *self {
            Component::NonTSPseudoClass(..) => true,
            Component::Negation(ref selectors) |
            Component::Is(ref selectors) |
            Component::Where(ref selectors) => selectors.iter().all(|selector| {
                selector
                    .iter_raw_match_order()
                    .all(|c| c.maybe_allowed_after_pseudo_element())
            }),
            _ => false,
        }
    }

    /// Whether a given selector should match for stateless pseudo-elements.
    ///
    /// This is a bit subtle: Only selectors that return true in
    /// `maybe_allowed_after_pseudo_element` should end up here, and
    /// `NonTSPseudoClass` never matches (as it is a stateless pseudo after
    /// all).
    fn matches_for_stateless_pseudo_element(&self) -> bool {
        debug_assert!(
            self.maybe_allowed_after_pseudo_element(),
            "Someone messed up pseudo-element parsing: {:?}",
            *self
        );
        match *self {
            Component::Negation(ref selectors) => !selectors.iter().all(|selector| {
                selector
                    .iter_raw_match_order()
                    .all(|c| c.matches_for_stateless_pseudo_element())
            }),
            Component::Is(ref selectors) | Component::Where(ref selectors) => {
                selectors.iter().any(|selector| {
                    selector
                        .iter_raw_match_order()
                        .all(|c| c.matches_for_stateless_pseudo_element())
                })
            },
            _ => false,
        }
    }

    pub fn visit<V>(&self, visitor: &mut V) -> bool
    where
        V: SelectorVisitor<Impl = Impl>,
    {
        use self::Component::*;
        if !visitor.visit_simple_selector(self) {
            return false;
        }

        match *self {
            Slotted(ref selector) => {
                if !selector.visit(visitor) {
                    return false;
                }
            },
            Host(Some(ref selector)) => {
                if !selector.visit(visitor) {
                    return false;
                }
            },
            AttributeInNoNamespaceExists {
                ref local_name,
                ref local_name_lower,
            } => {
                if !visitor.visit_attribute_selector(
                    &NamespaceConstraint::Specific(&namespace_empty_string::<Impl>()),
                    local_name,
                    local_name_lower,
                ) {
                    return false;
                }
            },
            AttributeInNoNamespace { ref local_name, .. } => {
                if !visitor.visit_attribute_selector(
                    &NamespaceConstraint::Specific(&namespace_empty_string::<Impl>()),
                    local_name,
                    local_name,
                ) {
                    return false;
                }
            },
            AttributeOther(ref attr_selector) => {
                let empty_string;
                let namespace = match attr_selector.namespace() {
                    Some(ns) => ns,
                    None => {
                        empty_string = crate::parser::namespace_empty_string::<Impl>();
                        NamespaceConstraint::Specific(&empty_string)
                    },
                };
                if !visitor.visit_attribute_selector(
                    &namespace,
                    &attr_selector.local_name,
                    &attr_selector.local_name_lower,
                ) {
                    return false;
                }
            },

            NonTSPseudoClass(ref pseudo_class) => {
                if !pseudo_class.visit(visitor) {
                    return false;
                }
            },
            Negation(ref list) | Is(ref list) | Where(ref list) => {
                let list_kind = SelectorListKind::from_component(self);
                debug_assert!(!list_kind.is_empty());
                if !visitor.visit_selector_list(list_kind, &list) {
                    return false;
                }
            },
            NthOf(ref nth_of_data) => {
                if !visitor.visit_selector_list(SelectorListKind::NTH_OF, nth_of_data.selectors()) {
                    return false;
                }
            },
            _ => {},
        }

        true
    }
}

#[derive(Clone, Eq, PartialEq, ToShmem)]
#[shmem(no_bounds)]
pub struct LocalName<Impl: SelectorImpl> {
    #[shmem(field_bound)]
    pub name: Impl::LocalName,
    pub lower_name: Impl::LocalName,
}

impl<Impl: SelectorImpl> Debug for Selector<Impl> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Selector(")?;
        self.to_css(f)?;
        write!(
            f,
            ", specificity = {:#x}, flags = {:?})",
            self.specificity(),
            self.flags()
        )
    }
}

impl<Impl: SelectorImpl> Debug for Component<Impl> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_css(f)
    }
}
impl<Impl: SelectorImpl> Debug for AttrSelectorWithOptionalNamespace<Impl> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_css(f)
    }
}
impl<Impl: SelectorImpl> Debug for LocalName<Impl> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_css(f)
    }
}

fn serialize_selector_list<'a, Impl, I, W>(iter: I, dest: &mut W) -> fmt::Result
where
    Impl: SelectorImpl,
    I: Iterator<Item = &'a Selector<Impl>>,
    W: fmt::Write,
{
    let mut first = true;
    for selector in iter {
        if !first {
            dest.write_str(", ")?;
        }
        first = false;
        selector.to_css(dest)?;
    }
    Ok(())
}

impl<Impl: SelectorImpl> ToCss for SelectorList<Impl> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        serialize_selector_list(self.0.iter(), dest)
    }
}

impl<Impl: SelectorImpl> ToCss for Selector<Impl> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        // Compound selectors invert the order of their contents, so we need to
        // undo that during serialization.
        //
        // This two-iterator strategy involves walking over the selector twice.
        // We could do something more clever, but selector serialization probably
        // isn't hot enough to justify it, and the stringification likely
        // dominates anyway.
        //
        // NB: A parse-order iterator is a Rev<>, which doesn't expose as_slice(),
        // which we need for |split|. So we split by combinators on a match-order
        // sequence and then reverse.

        let mut combinators = self
            .iter_raw_match_order()
            .rev()
            .filter_map(|x| x.as_combinator());
        let compound_selectors = self
            .iter_raw_match_order()
            .as_slice()
            .split(|x| x.is_combinator())
            .rev();

        let mut combinators_exhausted = false;
        for compound in compound_selectors {
            debug_assert!(!combinators_exhausted);

            // https://drafts.csswg.org/cssom/#serializing-selectors
            if compound.is_empty() {
                continue;
            }
            if let Component::RelativeSelectorAnchor = compound.first().unwrap() {
                debug_assert!(
                    compound.len() == 1,
                    "RelativeLeft should only be a simple selector"
                );
                combinators.next().unwrap().to_css_relative(dest)?;
                continue;
            }

            // 1. If there is only one simple selector in the compound selectors
            //    which is a universal selector, append the result of
            //    serializing the universal selector to s.
            //
            // Check if `!compound.empty()` first--this can happen if we have
            // something like `... > ::before`, because we store `>` and `::`
            // both as combinators internally.
            //
            // If we are in this case, after we have serialized the universal
            // selector, we skip Step 2 and continue with the algorithm.
            let (can_elide_namespace, first_non_namespace) = match compound[0] {
                Component::ExplicitAnyNamespace |
                Component::ExplicitNoNamespace |
                Component::Namespace(..) => (false, 1),
                Component::DefaultNamespace(..) => (true, 1),
                _ => (true, 0),
            };
            let mut perform_step_2 = true;
            let next_combinator = combinators.next();
            if first_non_namespace == compound.len() - 1 {
                match (next_combinator, &compound[first_non_namespace]) {
                    // We have to be careful here, because if there is a
                    // pseudo element "combinator" there isn't really just
                    // the one simple selector. Technically this compound
                    // selector contains the pseudo element selector as well
                    // -- Combinator::PseudoElement, just like
                    // Combinator::SlotAssignment, don't exist in the
                    // spec.
                    (Some(Combinator::PseudoElement), _) |
                    (Some(Combinator::SlotAssignment), _) => (),
                    (_, &Component::ExplicitUniversalType) => {
                        // Iterate over everything so we serialize the namespace
                        // too.
                        for simple in compound.iter() {
                            simple.to_css(dest)?;
                        }
                        // Skip step 2, which is an "otherwise".
                        perform_step_2 = false;
                    },
                    _ => (),
                }
            }

            // 2. Otherwise, for each simple selector in the compound selectors
            //    that is not a universal selector of which the namespace prefix
            //    maps to a namespace that is not the default namespace
            //    serialize the simple selector and append the result to s.
            //
            // See https://github.com/w3c/csswg-drafts/issues/1606, which is
            // proposing to change this to match up with the behavior asserted
            // in cssom/serialize-namespaced-type-selectors.html, which the
            // following code tries to match.
            if perform_step_2 {
                for simple in compound.iter() {
                    if let Component::ExplicitUniversalType = *simple {
                        // Can't have a namespace followed by a pseudo-element
                        // selector followed by a universal selector in the same
                        // compound selector, so we don't have to worry about the
                        // real namespace being in a different `compound`.
                        if can_elide_namespace {
                            continue;
                        }
                    }
                    simple.to_css(dest)?;
                }
            }

            // 3. If this is not the last part of the chain of the selector
            //    append a single SPACE (U+0020), followed by the combinator
            //    ">", "+", "~", ">>", "||", as appropriate, followed by another
            //    single SPACE (U+0020) if the combinator was not whitespace, to
            //    s.
            match next_combinator {
                Some(c) => c.to_css(dest)?,
                None => combinators_exhausted = true,
            };

            // 4. If this is the last part of the chain of the selector and
            //    there is a pseudo-element, append "::" followed by the name of
            //    the pseudo-element, to s.
            //
            // (we handle this above)
        }

        Ok(())
    }
}

impl Combinator {
    fn to_css_internal<W>(&self, dest: &mut W, prefix_space: bool) -> fmt::Result
    where
        W: fmt::Write,
    {
        if matches!(
            *self,
            Combinator::PseudoElement | Combinator::Part | Combinator::SlotAssignment
        ) {
            return Ok(());
        }
        if prefix_space {
            dest.write_char(' ')?;
        }
        match *self {
            Combinator::Child => dest.write_str("> "),
            Combinator::Descendant => Ok(()),
            Combinator::NextSibling => dest.write_str("+ "),
            Combinator::LaterSibling => dest.write_str("~ "),
            Combinator::PseudoElement | Combinator::Part | Combinator::SlotAssignment => unsafe {
                debug_unreachable!("Already handled")
            },
        }
    }

    fn to_css_relative<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        self.to_css_internal(dest, false)
    }
}

impl ToCss for Combinator {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        self.to_css_internal(dest, true)
    }
}

impl<Impl: SelectorImpl> ToCss for Component<Impl> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        use self::Component::*;

        match *self {
            Combinator(ref c) => c.to_css(dest),
            Slotted(ref selector) => {
                dest.write_str("::slotted(")?;
                selector.to_css(dest)?;
                dest.write_char(')')
            },
            Part(ref part_names) => {
                dest.write_str("::part(")?;
                for (i, name) in part_names.iter().enumerate() {
                    if i != 0 {
                        dest.write_char(' ')?;
                    }
                    name.to_css(dest)?;
                }
                dest.write_char(')')
            },
            PseudoElement(ref p) => p.to_css(dest),
            ID(ref s) => {
                dest.write_char('#')?;
                s.to_css(dest)
            },
            Class(ref s) => {
                dest.write_char('.')?;
                s.to_css(dest)
            },
            LocalName(ref s) => s.to_css(dest),
            ExplicitUniversalType => dest.write_char('*'),

            DefaultNamespace(_) => Ok(()),
            ExplicitNoNamespace => dest.write_char('|'),
            ExplicitAnyNamespace => dest.write_str("*|"),
            Namespace(ref prefix, _) => {
                prefix.to_css(dest)?;
                dest.write_char('|')
            },

            AttributeInNoNamespaceExists { ref local_name, .. } => {
                dest.write_char('[')?;
                local_name.to_css(dest)?;
                dest.write_char(']')
            },
            AttributeInNoNamespace {
                ref local_name,
                operator,
                ref value,
                case_sensitivity,
                ..
            } => {
                dest.write_char('[')?;
                local_name.to_css(dest)?;
                operator.to_css(dest)?;
                dest.write_char('"')?;
                value.to_css(dest)?;
                dest.write_char('"')?;
                match case_sensitivity {
                    ParsedCaseSensitivity::CaseSensitive |
                    ParsedCaseSensitivity::AsciiCaseInsensitiveIfInHtmlElementInHtmlDocument => {},
                    ParsedCaseSensitivity::AsciiCaseInsensitive => dest.write_str(" i")?,
                    ParsedCaseSensitivity::ExplicitCaseSensitive => dest.write_str(" s")?,
                }
                dest.write_char(']')
            },
            AttributeOther(ref attr_selector) => attr_selector.to_css(dest),

            // Pseudo-classes
            Root => dest.write_str(":root"),
            Empty => dest.write_str(":empty"),
            Scope => dest.write_str(":scope"),
            ParentSelector => dest.write_char('&'),
            Host(ref selector) => {
                dest.write_str(":host")?;
                if let Some(ref selector) = *selector {
                    dest.write_char('(')?;
                    selector.to_css(dest)?;
                    dest.write_char(')')?;
                }
                Ok(())
            },
            Nth(ref nth_data) => {
                nth_data.write_start(dest)?;
                if nth_data.is_function {
                    nth_data.write_affine(dest)?;
                    dest.write_char(')')?;
                }
                Ok(())
            },
            NthOf(ref nth_of_data) => {
                let nth_data = nth_of_data.nth_data();
                nth_data.write_start(dest)?;
                debug_assert!(
                    nth_data.is_function,
                    "A selector must be a function to hold An+B notation"
                );
                nth_data.write_affine(dest)?;
                debug_assert!(
                    matches!(nth_data.ty, NthType::Child | NthType::LastChild),
                    "Only :nth-child or :nth-last-child can be of a selector list"
                );
                debug_assert!(
                    !nth_of_data.selectors().is_empty(),
                    "The selector list should not be empty"
                );
                dest.write_str(" of ")?;
                serialize_selector_list(nth_of_data.selectors().iter(), dest)?;
                dest.write_char(')')
            },
            Is(ref list) | Where(ref list) | Negation(ref list) => {
                match *self {
                    Where(..) => dest.write_str(":where(")?,
                    Is(..) => dest.write_str(":is(")?,
                    Negation(..) => dest.write_str(":not(")?,
                    _ => unreachable!(),
                }
                serialize_selector_list(list.iter(), dest)?;
                dest.write_str(")")
            },
            Has(ref list) => {
                dest.write_str(":has(")?;
                let mut first = true;
                for RelativeSelector { ref selector, .. } in list.iter() {
                    if !first {
                        dest.write_str(", ")?;
                    }
                    first = false;
                    selector.to_css(dest)?;
                }
                dest.write_str(")")
            },
            NonTSPseudoClass(ref pseudo) => pseudo.to_css(dest),
            RelativeSelectorAnchor => Ok(()),
        }
    }
}

impl<Impl: SelectorImpl> ToCss for AttrSelectorWithOptionalNamespace<Impl> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_char('[')?;
        match self.namespace {
            Some(NamespaceConstraint::Specific((ref prefix, _))) => {
                prefix.to_css(dest)?;
                dest.write_char('|')?
            },
            Some(NamespaceConstraint::Any) => dest.write_str("*|")?,
            None => {},
        }
        self.local_name.to_css(dest)?;
        match self.operation {
            ParsedAttrSelectorOperation::Exists => {},
            ParsedAttrSelectorOperation::WithValue {
                operator,
                case_sensitivity,
                ref value,
            } => {
                operator.to_css(dest)?;
                dest.write_char('"')?;
                value.to_css(dest)?;
                dest.write_char('"')?;
                match case_sensitivity {
                    ParsedCaseSensitivity::CaseSensitive |
                    ParsedCaseSensitivity::AsciiCaseInsensitiveIfInHtmlElementInHtmlDocument => {},
                    ParsedCaseSensitivity::AsciiCaseInsensitive => dest.write_str(" i")?,
                    ParsedCaseSensitivity::ExplicitCaseSensitive => dest.write_str(" s")?,
                }
            },
        }
        dest.write_char(']')
    }
}

impl<Impl: SelectorImpl> ToCss for LocalName<Impl> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        self.name.to_css(dest)
    }
}

/// Build up a Selector.
/// selector : simple_selector_sequence [ combinator simple_selector_sequence ]* ;
///
/// `Err` means invalid selector.
fn parse_selector<'i, 't, P, Impl>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
    mut state: SelectorParsingState,
    parse_relative: ParseRelative,
) -> Result<Selector<Impl>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl = Impl>,
    Impl: SelectorImpl,
{
    let mut builder = SelectorBuilder::default();
    if parse_relative == ParseRelative::Yes {
        builder.push_simple_selector(Component::RelativeSelectorAnchor);
        // Do we see a combinator? If so, push that. Otherwise, push a descendant combinator.
        builder
            .push_combinator(parse_combinator::<P, Impl>(input).unwrap_or(Combinator::Descendant));
    }
    'outer_loop: loop {
        // Parse a sequence of simple selectors.
        let empty = parse_compound_selector(parser, &mut state, input, &mut builder)?;
        if empty {
            return Err(input.new_custom_error(if builder.has_combinators() {
                SelectorParseErrorKind::DanglingCombinator
            } else {
                SelectorParseErrorKind::EmptySelector
            }));
        }

        if state.intersects(SelectorParsingState::AFTER_PSEUDO) {
            debug_assert!(state.intersects(
                SelectorParsingState::AFTER_PSEUDO_ELEMENT |
                    SelectorParsingState::AFTER_SLOTTED |
                    SelectorParsingState::AFTER_PART
            ));
            break;
        }

        let combinator = if let Ok(c) = parse_combinator::<P, Impl>(input) {
            c
        } else {
            break 'outer_loop;
        };

        if !state.allows_combinators() {
            return Err(input.new_custom_error(SelectorParseErrorKind::InvalidState));
        }

        builder.push_combinator(combinator);
    }
    return Ok(Selector(builder.build()));
}

fn parse_combinator<'i, 't, P, Impl>(input: &mut CssParser<'i, 't>) -> Result<Combinator, ()> {
    let mut any_whitespace = false;
    loop {
        let before_this_token = input.state();
        match input.next_including_whitespace() {
            Err(_e) => return Err(()),
            Ok(&Token::WhiteSpace(_)) => any_whitespace = true,
            Ok(&Token::Delim('>')) => {
                return Ok(Combinator::Child);
            },
            Ok(&Token::Delim('+')) => {
                return Ok(Combinator::NextSibling);
            },
            Ok(&Token::Delim('~')) => {
                return Ok(Combinator::LaterSibling);
            },
            Ok(_) => {
                input.reset(&before_this_token);
                if any_whitespace {
                    return Ok(Combinator::Descendant);
                } else {
                    return Err(());
                }
            },
        }
    }
}

impl<Impl: SelectorImpl> Selector<Impl> {
    /// Parse a selector, without any pseudo-element.
    #[inline]
    pub fn parse<'i, 't, P>(
        parser: &P,
        input: &mut CssParser<'i, 't>,
    ) -> Result<Self, ParseError<'i, P::Error>>
    where
        P: Parser<'i, Impl = Impl>,
    {
        parse_selector(
            parser,
            input,
            SelectorParsingState::empty(),
            ParseRelative::No,
        )
    }
}

/// * `Err(())`: Invalid selector, abort
/// * `Ok(false)`: Not a type selector, could be something else. `input` was not consumed.
/// * `Ok(true)`: Length 0 (`*|*`), 1 (`*|E` or `ns|*`) or 2 (`|E` or `ns|E`)
fn parse_type_selector<'i, 't, P, Impl, S>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
    state: SelectorParsingState,
    sink: &mut S,
) -> Result<bool, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl = Impl>,
    Impl: SelectorImpl,
    S: Push<Component<Impl>>,
{
    match parse_qualified_name(parser, input, /* in_attr_selector = */ false) {
        Err(ParseError {
            kind: ParseErrorKind::Basic(BasicParseErrorKind::EndOfInput),
            ..
        }) |
        Ok(OptionalQName::None(_)) => Ok(false),
        Ok(OptionalQName::Some(namespace, local_name)) => {
            if state.intersects(SelectorParsingState::AFTER_PSEUDO) {
                return Err(input.new_custom_error(SelectorParseErrorKind::InvalidState));
            }
            match namespace {
                QNamePrefix::ImplicitAnyNamespace => {},
                QNamePrefix::ImplicitDefaultNamespace(url) => {
                    sink.push(Component::DefaultNamespace(url))
                },
                QNamePrefix::ExplicitNamespace(prefix, url) => {
                    sink.push(match parser.default_namespace() {
                        Some(ref default_url) if url == *default_url => {
                            Component::DefaultNamespace(url)
                        },
                        _ => Component::Namespace(prefix, url),
                    })
                },
                QNamePrefix::ExplicitNoNamespace => sink.push(Component::ExplicitNoNamespace),
                QNamePrefix::ExplicitAnyNamespace => {
                    match parser.default_namespace() {
                        // Element type selectors that have no namespace
                        // component (no namespace separator) represent elements
                        // without regard to the element's namespace (equivalent
                        // to "*|") unless a default namespace has been declared
                        // for namespaced selectors (e.g. in CSS, in the style
                        // sheet). If a default namespace has been declared,
                        // such selectors will represent only elements in the
                        // default namespace.
                        // -- Selectors § 6.1.1
                        // So we'll have this act the same as the
                        // QNamePrefix::ImplicitAnyNamespace case.
                        None => {},
                        Some(_) => sink.push(Component::ExplicitAnyNamespace),
                    }
                },
                QNamePrefix::ImplicitNoNamespace => {
                    unreachable!() // Not returned with in_attr_selector = false
                },
            }
            match local_name {
                Some(name) => sink.push(Component::LocalName(LocalName {
                    lower_name: to_ascii_lowercase(&name).as_ref().into(),
                    name: name.as_ref().into(),
                })),
                None => sink.push(Component::ExplicitUniversalType),
            }
            Ok(true)
        },
        Err(e) => Err(e),
    }
}

#[derive(Debug)]
enum SimpleSelectorParseResult<Impl: SelectorImpl> {
    SimpleSelector(Component<Impl>),
    PseudoElement(Impl::PseudoElement),
    SlottedPseudo(Selector<Impl>),
    PartPseudo(Box<[Impl::Identifier]>),
}

#[derive(Debug)]
enum QNamePrefix<Impl: SelectorImpl> {
    ImplicitNoNamespace,                          // `foo` in attr selectors
    ImplicitAnyNamespace,                         // `foo` in type selectors, without a default ns
    ImplicitDefaultNamespace(Impl::NamespaceUrl), // `foo` in type selectors, with a default ns
    ExplicitNoNamespace,                          // `|foo`
    ExplicitAnyNamespace,                         // `*|foo`
    ExplicitNamespace(Impl::NamespacePrefix, Impl::NamespaceUrl), // `prefix|foo`
}

enum OptionalQName<'i, Impl: SelectorImpl> {
    Some(QNamePrefix<Impl>, Option<CowRcStr<'i>>),
    None(Token<'i>),
}

/// * `Err(())`: Invalid selector, abort
/// * `Ok(None(token))`: Not a simple selector, could be something else. `input` was not consumed,
///                      but the token is still returned.
/// * `Ok(Some(namespace, local_name))`: `None` for the local name means a `*` universal selector
fn parse_qualified_name<'i, 't, P, Impl>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
    in_attr_selector: bool,
) -> Result<OptionalQName<'i, Impl>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl = Impl>,
    Impl: SelectorImpl,
{
    let default_namespace = |local_name| {
        let namespace = match parser.default_namespace() {
            Some(url) => QNamePrefix::ImplicitDefaultNamespace(url),
            None => QNamePrefix::ImplicitAnyNamespace,
        };
        Ok(OptionalQName::Some(namespace, local_name))
    };

    let explicit_namespace = |input: &mut CssParser<'i, 't>, namespace| {
        let location = input.current_source_location();
        match input.next_including_whitespace() {
            Ok(&Token::Delim('*')) if !in_attr_selector => Ok(OptionalQName::Some(namespace, None)),
            Ok(&Token::Ident(ref local_name)) => {
                Ok(OptionalQName::Some(namespace, Some(local_name.clone())))
            },
            Ok(t) if in_attr_selector => {
                let e = SelectorParseErrorKind::InvalidQualNameInAttr(t.clone());
                Err(location.new_custom_error(e))
            },
            Ok(t) => Err(location.new_custom_error(
                SelectorParseErrorKind::ExplicitNamespaceUnexpectedToken(t.clone()),
            )),
            Err(e) => Err(e.into()),
        }
    };

    let start = input.state();
    match input.next_including_whitespace() {
        Ok(Token::Ident(value)) => {
            let value = value.clone();
            let after_ident = input.state();
            match input.next_including_whitespace() {
                Ok(&Token::Delim('|')) => {
                    let prefix = value.as_ref().into();
                    let result = parser.namespace_for_prefix(&prefix);
                    let url = result.ok_or(
                        after_ident
                            .source_location()
                            .new_custom_error(SelectorParseErrorKind::ExpectedNamespace(value)),
                    )?;
                    explicit_namespace(input, QNamePrefix::ExplicitNamespace(prefix, url))
                },
                _ => {
                    input.reset(&after_ident);
                    if in_attr_selector {
                        Ok(OptionalQName::Some(
                            QNamePrefix::ImplicitNoNamespace,
                            Some(value),
                        ))
                    } else {
                        default_namespace(Some(value))
                    }
                },
            }
        },
        Ok(Token::Delim('*')) => {
            let after_star = input.state();
            match input.next_including_whitespace() {
                Ok(&Token::Delim('|')) => {
                    explicit_namespace(input, QNamePrefix::ExplicitAnyNamespace)
                },
                _ if !in_attr_selector => {
                    input.reset(&after_star);
                    default_namespace(None)
                },
                result => {
                    let t = result?;
                    Err(after_star
                        .source_location()
                        .new_custom_error(SelectorParseErrorKind::ExpectedBarInAttr(t.clone())))
                },
            }
        },
        Ok(Token::Delim('|')) => explicit_namespace(input, QNamePrefix::ExplicitNoNamespace),
        Ok(t) => {
            let t = t.clone();
            input.reset(&start);
            Ok(OptionalQName::None(t))
        },
        Err(e) => {
            input.reset(&start);
            Err(e.into())
        },
    }
}

fn parse_attribute_selector<'i, 't, P, Impl>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
) -> Result<Component<Impl>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl = Impl>,
    Impl: SelectorImpl,
{
    let namespace;
    let local_name;

    input.skip_whitespace();

    match parse_qualified_name(parser, input, /* in_attr_selector = */ true)? {
        OptionalQName::None(t) => {
            return Err(input.new_custom_error(
                SelectorParseErrorKind::NoQualifiedNameInAttributeSelector(t),
            ));
        },
        OptionalQName::Some(_, None) => unreachable!(),
        OptionalQName::Some(ns, Some(ln)) => {
            local_name = ln;
            namespace = match ns {
                QNamePrefix::ImplicitNoNamespace | QNamePrefix::ExplicitNoNamespace => None,
                QNamePrefix::ExplicitNamespace(prefix, url) => {
                    Some(NamespaceConstraint::Specific((prefix, url)))
                },
                QNamePrefix::ExplicitAnyNamespace => Some(NamespaceConstraint::Any),
                QNamePrefix::ImplicitAnyNamespace | QNamePrefix::ImplicitDefaultNamespace(_) => {
                    unreachable!() // Not returned with in_attr_selector = true
                },
            }
        },
    }

    let location = input.current_source_location();
    let operator = match input.next() {
        // [foo]
        Err(_) => {
            let local_name_lower = to_ascii_lowercase(&local_name).as_ref().into();
            let local_name = local_name.as_ref().into();
            if let Some(namespace) = namespace {
                return Ok(Component::AttributeOther(Box::new(
                    AttrSelectorWithOptionalNamespace {
                        namespace: Some(namespace),
                        local_name,
                        local_name_lower,
                        operation: ParsedAttrSelectorOperation::Exists,
                    },
                )));
            } else {
                return Ok(Component::AttributeInNoNamespaceExists {
                    local_name,
                    local_name_lower,
                });
            }
        },

        // [foo=bar]
        Ok(&Token::Delim('=')) => AttrSelectorOperator::Equal,
        // [foo~=bar]
        Ok(&Token::IncludeMatch) => AttrSelectorOperator::Includes,
        // [foo|=bar]
        Ok(&Token::DashMatch) => AttrSelectorOperator::DashMatch,
        // [foo^=bar]
        Ok(&Token::PrefixMatch) => AttrSelectorOperator::Prefix,
        // [foo*=bar]
        Ok(&Token::SubstringMatch) => AttrSelectorOperator::Substring,
        // [foo$=bar]
        Ok(&Token::SuffixMatch) => AttrSelectorOperator::Suffix,
        Ok(t) => {
            return Err(location.new_custom_error(
                SelectorParseErrorKind::UnexpectedTokenInAttributeSelector(t.clone()),
            ));
        },
    };

    let value = match input.expect_ident_or_string() {
        Ok(t) => t.clone(),
        Err(BasicParseError {
            kind: BasicParseErrorKind::UnexpectedToken(t),
            location,
        }) => return Err(location.new_custom_error(SelectorParseErrorKind::BadValueInAttr(t))),
        Err(e) => return Err(e.into()),
    };

    let attribute_flags = parse_attribute_flags(input)?;
    let value = value.as_ref().into();
    let local_name_lower;
    let local_name_is_ascii_lowercase;
    let case_sensitivity;
    {
        let local_name_lower_cow = to_ascii_lowercase(&local_name);
        case_sensitivity =
            attribute_flags.to_case_sensitivity(local_name_lower_cow.as_ref(), namespace.is_some());
        local_name_lower = local_name_lower_cow.as_ref().into();
        local_name_is_ascii_lowercase = matches!(local_name_lower_cow, Cow::Borrowed(..));
    }
    let local_name = local_name.as_ref().into();
    if namespace.is_some() || !local_name_is_ascii_lowercase {
        Ok(Component::AttributeOther(Box::new(
            AttrSelectorWithOptionalNamespace {
                namespace,
                local_name,
                local_name_lower,
                operation: ParsedAttrSelectorOperation::WithValue {
                    operator,
                    case_sensitivity,
                    value,
                },
            },
        )))
    } else {
        Ok(Component::AttributeInNoNamespace {
            local_name,
            operator,
            value,
            case_sensitivity,
        })
    }
}

/// An attribute selector can have 's' or 'i' as flags, or no flags at all.
enum AttributeFlags {
    // Matching should be case-sensitive ('s' flag).
    CaseSensitive,
    // Matching should be case-insensitive ('i' flag).
    AsciiCaseInsensitive,
    // No flags.  Matching behavior depends on the name of the attribute.
    CaseSensitivityDependsOnName,
}

impl AttributeFlags {
    fn to_case_sensitivity(self, local_name: &str, have_namespace: bool) -> ParsedCaseSensitivity {
        match self {
            AttributeFlags::CaseSensitive => ParsedCaseSensitivity::ExplicitCaseSensitive,
            AttributeFlags::AsciiCaseInsensitive => ParsedCaseSensitivity::AsciiCaseInsensitive,
            AttributeFlags::CaseSensitivityDependsOnName => {
                if !have_namespace &&
                    include!(concat!(
                        env!("OUT_DIR"),
                        "/ascii_case_insensitive_html_attributes.rs"
                    ))
                    .contains(local_name)
                {
                    ParsedCaseSensitivity::AsciiCaseInsensitiveIfInHtmlElementInHtmlDocument
                } else {
                    ParsedCaseSensitivity::CaseSensitive
                }
            },
        }
    }
}

fn parse_attribute_flags<'i, 't>(
    input: &mut CssParser<'i, 't>,
) -> Result<AttributeFlags, BasicParseError<'i>> {
    let location = input.current_source_location();
    let token = match input.next() {
        Ok(t) => t,
        Err(..) => {
            // Selectors spec says language-defined; HTML says it depends on the
            // exact attribute name.
            return Ok(AttributeFlags::CaseSensitivityDependsOnName);
        },
    };

    let ident = match *token {
        Token::Ident(ref i) => i,
        ref other => return Err(location.new_basic_unexpected_token_error(other.clone())),
    };

    Ok(match_ignore_ascii_case! {
        ident,
        "i" => AttributeFlags::AsciiCaseInsensitive,
        "s" => AttributeFlags::CaseSensitive,
        _ => return Err(location.new_basic_unexpected_token_error(token.clone())),
    })
}

/// Level 3: Parse **one** simple_selector.  (Though we might insert a second
/// implied "<defaultns>|*" type selector.)
fn parse_negation<'i, 't, P, Impl>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
    state: SelectorParsingState,
) -> Result<Component<Impl>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl = Impl>,
    Impl: SelectorImpl,
{
    let list = SelectorList::parse_with_state(
        parser,
        input,
        state |
            SelectorParsingState::SKIP_DEFAULT_NAMESPACE |
            SelectorParsingState::DISALLOW_PSEUDOS,
        ForgivingParsing::No,
        ParseRelative::No,
    )?;

    Ok(Component::Negation(list.0.into_vec().into_boxed_slice()))
}

/// simple_selector_sequence
/// : [ type_selector | universal ] [ HASH | class | attrib | pseudo | negation ]*
/// | [ HASH | class | attrib | pseudo | negation ]+
///
/// `Err(())` means invalid selector.
/// `Ok(true)` is an empty selector
fn parse_compound_selector<'i, 't, P, Impl>(
    parser: &P,
    state: &mut SelectorParsingState,
    input: &mut CssParser<'i, 't>,
    builder: &mut SelectorBuilder<Impl>,
) -> Result<bool, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl = Impl>,
    Impl: SelectorImpl,
{
    input.skip_whitespace();

    let mut empty = true;
    if parse_type_selector(parser, input, *state, builder)? {
        empty = false;
    }

    loop {
        let result = match parse_one_simple_selector(parser, input, *state)? {
            None => break,
            Some(result) => result,
        };

        if empty {
            if let Some(url) = parser.default_namespace() {
                // If there was no explicit type selector, but there is a
                // default namespace, there is an implicit "<defaultns>|*" type
                // selector. Except for :host() or :not() / :is() / :where(),
                // where we ignore it.
                //
                // https://drafts.csswg.org/css-scoping/#host-element-in-tree:
                //
                //     When considered within its own shadow trees, the shadow
                //     host is featureless. Only the :host, :host(), and
                //     :host-context() pseudo-classes are allowed to match it.
                //
                // https://drafts.csswg.org/selectors-4/#featureless:
                //
                //     A featureless element does not match any selector at all,
                //     except those it is explicitly defined to match. If a
                //     given selector is allowed to match a featureless element,
                //     it must do so while ignoring the default namespace.
                //
                // https://drafts.csswg.org/selectors-4/#matches
                //
                //     Default namespace declarations do not affect the compound
                //     selector representing the subject of any selector within
                //     a :is() pseudo-class, unless that compound selector
                //     contains an explicit universal selector or type selector.
                //
                //     (Similar quotes for :where() / :not())
                //
                let ignore_default_ns = state
                    .intersects(SelectorParsingState::SKIP_DEFAULT_NAMESPACE) ||
                    matches!(
                        result,
                        SimpleSelectorParseResult::SimpleSelector(Component::Host(..))
                    );
                if !ignore_default_ns {
                    builder.push_simple_selector(Component::DefaultNamespace(url));
                }
            }
        }

        empty = false;

        match result {
            SimpleSelectorParseResult::SimpleSelector(s) => {
                builder.push_simple_selector(s);
            },
            SimpleSelectorParseResult::PartPseudo(part_names) => {
                state.insert(SelectorParsingState::AFTER_PART);
                builder.push_combinator(Combinator::Part);
                builder.push_simple_selector(Component::Part(part_names));
            },
            SimpleSelectorParseResult::SlottedPseudo(selector) => {
                state.insert(SelectorParsingState::AFTER_SLOTTED);
                builder.push_combinator(Combinator::SlotAssignment);
                builder.push_simple_selector(Component::Slotted(selector));
            },
            SimpleSelectorParseResult::PseudoElement(p) => {
                state.insert(SelectorParsingState::AFTER_PSEUDO_ELEMENT);
                if !p.accepts_state_pseudo_classes() {
                    state.insert(SelectorParsingState::AFTER_NON_STATEFUL_PSEUDO_ELEMENT);
                }
                builder.push_combinator(Combinator::PseudoElement);
                builder.push_simple_selector(Component::PseudoElement(p));
            },
        }
    }
    Ok(empty)
}

fn parse_is_where<'i, 't, P, Impl>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
    state: SelectorParsingState,
    component: impl FnOnce(Box<[Selector<Impl>]>) -> Component<Impl>,
) -> Result<Component<Impl>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl = Impl>,
    Impl: SelectorImpl,
{
    debug_assert!(parser.parse_is_and_where());
    // https://drafts.csswg.org/selectors/#matches-pseudo:
    //
    //     Pseudo-elements cannot be represented by the matches-any
    //     pseudo-class; they are not valid within :is().
    //
    let inner = SelectorList::parse_with_state(
        parser,
        input,
        state |
            SelectorParsingState::SKIP_DEFAULT_NAMESPACE |
            SelectorParsingState::DISALLOW_PSEUDOS,
        ForgivingParsing::Yes,
        ParseRelative::No,
    )?;
    Ok(component(inner.0.into_vec().into_boxed_slice()))
}

fn parse_has<'i, 't, P, Impl>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
    state: SelectorParsingState,
) -> Result<Component<Impl>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl = Impl>,
    Impl: SelectorImpl,
{
    debug_assert!(parser.parse_has());
    if state.intersects(SelectorParsingState::DISALLOW_RELATIVE_SELECTOR) {
        return Err(input.new_custom_error(SelectorParseErrorKind::InvalidState));
    }
    // Nested `:has()` is disallowed, mark it as such.
    // Note: The spec defines ":has-allowed pseudo-element," but there's no
    // pseudo-element defined as such at the moment.
    // https://w3c.github.io/csswg-drafts/selectors-4/#has-allowed-pseudo-element
    let inner = SelectorList::parse_with_state(
        parser,
        input,
        state |
            SelectorParsingState::SKIP_DEFAULT_NAMESPACE |
            SelectorParsingState::DISALLOW_PSEUDOS |
            SelectorParsingState::DISALLOW_RELATIVE_SELECTOR,
        ForgivingParsing::No,
        ParseRelative::Yes,
    )?;
    Ok(Component::Has(RelativeSelector::from_selector_list(inner)))
}

fn parse_functional_pseudo_class<'i, 't, P, Impl>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
    name: CowRcStr<'i>,
    state: SelectorParsingState,
) -> Result<Component<Impl>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl = Impl>,
    Impl: SelectorImpl,
{
    match_ignore_ascii_case! { &name,
        "nth-child" => return parse_nth_pseudo_class(parser, input, state, NthType::Child),
        "nth-of-type" => return parse_nth_pseudo_class(parser, input, state, NthType::OfType),
        "nth-last-child" => return parse_nth_pseudo_class(parser, input, state, NthType::LastChild),
        "nth-last-of-type" => return parse_nth_pseudo_class(parser, input, state, NthType::LastOfType),
        "is" if parser.parse_is_and_where() => return parse_is_where(parser, input, state, Component::Is),
        "where" if parser.parse_is_and_where() => return parse_is_where(parser, input, state, Component::Where),
        "has" if parser.parse_has() => return parse_has(parser, input, state),
        "host" => {
            if !state.allows_tree_structural_pseudo_classes() {
                return Err(input.new_custom_error(SelectorParseErrorKind::InvalidState));
            }
            return Ok(Component::Host(Some(parse_inner_compound_selector(parser, input, state)?)));
        },
        "not" => {
            return parse_negation(parser, input, state)
        },
        _ => {}
    }

    if parser.parse_is_and_where() && parser.is_is_alias(&name) {
        return parse_is_where(parser, input, state, Component::Is);
    }

    if !state.allows_custom_functional_pseudo_classes() {
        return Err(input.new_custom_error(SelectorParseErrorKind::InvalidState));
    }

    P::parse_non_ts_functional_pseudo_class(parser, name, input).map(Component::NonTSPseudoClass)
}

fn parse_nth_pseudo_class<'i, 't, P, Impl>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
    state: SelectorParsingState,
    ty: NthType,
) -> Result<Component<Impl>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl = Impl>,
    Impl: SelectorImpl,
{
    if !state.allows_tree_structural_pseudo_classes() {
        return Err(input.new_custom_error(SelectorParseErrorKind::InvalidState));
    }
    let (a, b) = parse_nth(input)?;
    let nth_data = NthSelectorData {
        ty,
        is_function: true,
        a,
        b,
    };
    if !parser.parse_nth_child_of() || ty.is_of_type() {
        return Ok(Component::Nth(nth_data));
    }

    // Try to parse "of <selector-list>".
    if input.try_parse(|i| i.expect_ident_matching("of")).is_err() {
        return Ok(Component::Nth(nth_data));
    }
    // Whitespace between "of" and the selector list is optional
    // https://github.com/w3c/csswg-drafts/issues/8285
    let mut selectors = SelectorList::parse_with_state(
        parser,
        input,
        state |
            SelectorParsingState::SKIP_DEFAULT_NAMESPACE |
            SelectorParsingState::DISALLOW_PSEUDOS,
        ForgivingParsing::No,
        ParseRelative::No,
    )?;
    Ok(Component::NthOf(NthOfSelectorData::new(
        &nth_data,
        selectors.0.drain(..),
    )))
}

/// Returns whether the name corresponds to a CSS2 pseudo-element that
/// can be specified with the single colon syntax (in addition to the
/// double-colon syntax, which can be used for all pseudo-elements).
fn is_css2_pseudo_element(name: &str) -> bool {
    // ** Do not add to this list! **
    match_ignore_ascii_case! { name,
        "before" | "after" | "first-line" | "first-letter" => true,
        _ => false,
    }
}

/// Parse a simple selector other than a type selector.
///
/// * `Err(())`: Invalid selector, abort
/// * `Ok(None)`: Not a simple selector, could be something else. `input` was not consumed.
/// * `Ok(Some(_))`: Parsed a simple selector or pseudo-element
fn parse_one_simple_selector<'i, 't, P, Impl>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
    state: SelectorParsingState,
) -> Result<Option<SimpleSelectorParseResult<Impl>>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl = Impl>,
    Impl: SelectorImpl,
{
    let start = input.state();
    let token = match input.next_including_whitespace().map(|t| t.clone()) {
        Ok(t) => t,
        Err(..) => {
            input.reset(&start);
            return Ok(None);
        },
    };

    Ok(Some(match token {
        Token::IDHash(id) => {
            if state.intersects(SelectorParsingState::AFTER_PSEUDO) {
                return Err(input.new_custom_error(SelectorParseErrorKind::InvalidState));
            }
            let id = Component::ID(id.as_ref().into());
            SimpleSelectorParseResult::SimpleSelector(id)
        },
        Token::Delim(delim) if delim == '.' || (delim == '&' && parser.parse_parent_selector()) => {
            if state.intersects(SelectorParsingState::AFTER_PSEUDO) {
                return Err(input.new_custom_error(SelectorParseErrorKind::InvalidState));
            }
            let location = input.current_source_location();
            SimpleSelectorParseResult::SimpleSelector(if delim == '&' {
                Component::ParentSelector
            } else {
                let class = match *input.next_including_whitespace()? {
                    Token::Ident(ref class) => class,
                    ref t => {
                        let e = SelectorParseErrorKind::ClassNeedsIdent(t.clone());
                        return Err(location.new_custom_error(e));
                    },
                };
                Component::Class(class.as_ref().into())
            })
        },
        Token::SquareBracketBlock => {
            if state.intersects(SelectorParsingState::AFTER_PSEUDO) {
                return Err(input.new_custom_error(SelectorParseErrorKind::InvalidState));
            }
            let attr = input.parse_nested_block(|input| parse_attribute_selector(parser, input))?;
            SimpleSelectorParseResult::SimpleSelector(attr)
        },
        Token::Colon => {
            let location = input.current_source_location();
            let (is_single_colon, next_token) = match input.next_including_whitespace()?.clone() {
                Token::Colon => (false, input.next_including_whitespace()?.clone()),
                t => (true, t),
            };
            let (name, is_functional) = match next_token {
                Token::Ident(name) => (name, false),
                Token::Function(name) => (name, true),
                t => {
                    let e = SelectorParseErrorKind::PseudoElementExpectedIdent(t);
                    return Err(input.new_custom_error(e));
                },
            };
            let is_pseudo_element = !is_single_colon || is_css2_pseudo_element(&name);
            if is_pseudo_element {
                if !state.allows_pseudos() {
                    return Err(input.new_custom_error(SelectorParseErrorKind::InvalidState));
                }
                let pseudo_element = if is_functional {
                    if P::parse_part(parser) && name.eq_ignore_ascii_case("part") {
                        if !state.allows_part() {
                            return Err(
                                input.new_custom_error(SelectorParseErrorKind::InvalidState)
                            );
                        }
                        let names = input.parse_nested_block(|input| {
                            let mut result = Vec::with_capacity(1);
                            result.push(input.expect_ident()?.as_ref().into());
                            while !input.is_exhausted() {
                                result.push(input.expect_ident()?.as_ref().into());
                            }
                            Ok(result.into_boxed_slice())
                        })?;
                        return Ok(Some(SimpleSelectorParseResult::PartPseudo(names)));
                    }
                    if P::parse_slotted(parser) && name.eq_ignore_ascii_case("slotted") {
                        if !state.allows_slotted() {
                            return Err(
                                input.new_custom_error(SelectorParseErrorKind::InvalidState)
                            );
                        }
                        let selector = input.parse_nested_block(|input| {
                            parse_inner_compound_selector(parser, input, state)
                        })?;
                        return Ok(Some(SimpleSelectorParseResult::SlottedPseudo(selector)));
                    }
                    input.parse_nested_block(|input| {
                        P::parse_functional_pseudo_element(parser, name, input)
                    })?
                } else {
                    P::parse_pseudo_element(parser, location, name)?
                };

                if state.intersects(SelectorParsingState::AFTER_SLOTTED) &&
                    !pseudo_element.valid_after_slotted()
                {
                    return Err(input.new_custom_error(SelectorParseErrorKind::InvalidState));
                }
                SimpleSelectorParseResult::PseudoElement(pseudo_element)
            } else {
                let pseudo_class = if is_functional {
                    input.parse_nested_block(|input| {
                        parse_functional_pseudo_class(parser, input, name, state)
                    })?
                } else {
                    parse_simple_pseudo_class(parser, location, name, state)?
                };
                SimpleSelectorParseResult::SimpleSelector(pseudo_class)
            }
        },
        _ => {
            input.reset(&start);
            return Ok(None);
        },
    }))
}

fn parse_simple_pseudo_class<'i, P, Impl>(
    parser: &P,
    location: SourceLocation,
    name: CowRcStr<'i>,
    state: SelectorParsingState,
) -> Result<Component<Impl>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl = Impl>,
    Impl: SelectorImpl,
{
    if !state.allows_non_functional_pseudo_classes() {
        return Err(location.new_custom_error(SelectorParseErrorKind::InvalidState));
    }

    if state.allows_tree_structural_pseudo_classes() {
        match_ignore_ascii_case! { &name,
            "first-child" => return Ok(Component::Nth(NthSelectorData::first(/* of_type = */ false))),
            "last-child" => return Ok(Component::Nth(NthSelectorData::last(/* of_type = */ false))),
            "only-child" => return Ok(Component::Nth(NthSelectorData::only(/* of_type = */ false))),
            "root" => return Ok(Component::Root),
            "empty" => return Ok(Component::Empty),
            "scope" => return Ok(Component::Scope),
            "host" if P::parse_host(parser) => return Ok(Component::Host(None)),
            "first-of-type" => return Ok(Component::Nth(NthSelectorData::first(/* of_type = */ true))),
            "last-of-type" => return Ok(Component::Nth(NthSelectorData::last(/* of_type = */ true))),
            "only-of-type" => return Ok(Component::Nth(NthSelectorData::only(/* of_type = */ true))),
            _ => {},
        }
    }

    let pseudo_class = P::parse_non_ts_pseudo_class(parser, location, name)?;
    if state.intersects(SelectorParsingState::AFTER_PSEUDO_ELEMENT) &&
        !pseudo_class.is_user_action_state()
    {
        return Err(location.new_custom_error(SelectorParseErrorKind::InvalidState));
    }
    Ok(Component::NonTSPseudoClass(pseudo_class))
}

// NB: pub module in order to access the DummyParser
#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::builder::SelectorFlags;
    use crate::parser;
    use cssparser::{serialize_identifier, Parser as CssParser, ParserInput, ToCss};
    use std::collections::HashMap;
    use std::fmt;

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum PseudoClass {
        Hover,
        Active,
        Lang(String),
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum PseudoElement {
        Before,
        After,
        Highlight(String),
    }

    impl parser::PseudoElement for PseudoElement {
        type Impl = DummySelectorImpl;

        fn accepts_state_pseudo_classes(&self) -> bool {
            true
        }

        fn valid_after_slotted(&self) -> bool {
            true
        }
    }

    impl parser::NonTSPseudoClass for PseudoClass {
        type Impl = DummySelectorImpl;

        #[inline]
        fn is_active_or_hover(&self) -> bool {
            matches!(*self, PseudoClass::Active | PseudoClass::Hover)
        }

        #[inline]
        fn is_user_action_state(&self) -> bool {
            self.is_active_or_hover()
        }
    }

    impl ToCss for PseudoClass {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where
            W: fmt::Write,
        {
            match *self {
                PseudoClass::Hover => dest.write_str(":hover"),
                PseudoClass::Active => dest.write_str(":active"),
                PseudoClass::Lang(ref lang) => {
                    dest.write_str(":lang(")?;
                    serialize_identifier(lang, dest)?;
                    dest.write_char(')')
                },
            }
        }
    }

    impl ToCss for PseudoElement {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where
            W: fmt::Write,
        {
            match *self {
                PseudoElement::Before => dest.write_str("::before"),
                PseudoElement::After => dest.write_str("::after"),
                PseudoElement::Highlight(ref name) => {
                    dest.write_str("::highlight(")?;
                    serialize_identifier(&name, dest)?;
                    dest.write_char(')')
                },
            }
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct DummySelectorImpl;

    #[derive(Default)]
    pub struct DummyParser {
        default_ns: Option<DummyAtom>,
        ns_prefixes: HashMap<DummyAtom, DummyAtom>,
    }

    impl DummyParser {
        fn default_with_namespace(default_ns: DummyAtom) -> DummyParser {
            DummyParser {
                default_ns: Some(default_ns),
                ns_prefixes: Default::default(),
            }
        }
    }

    impl SelectorImpl for DummySelectorImpl {
        type ExtraMatchingData<'a> = std::marker::PhantomData<&'a ()>;
        type AttrValue = DummyAttrValue;
        type Identifier = DummyAtom;
        type LocalName = DummyAtom;
        type NamespaceUrl = DummyAtom;
        type NamespacePrefix = DummyAtom;
        type BorrowedLocalName = DummyAtom;
        type BorrowedNamespaceUrl = DummyAtom;
        type NonTSPseudoClass = PseudoClass;
        type PseudoElement = PseudoElement;
    }

    #[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
    pub struct DummyAttrValue(String);

    impl ToCss for DummyAttrValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where
            W: fmt::Write,
        {
            use std::fmt::Write;

            write!(cssparser::CssStringWriter::new(dest), "{}", &self.0)
        }
    }

    impl<'a> From<&'a str> for DummyAttrValue {
        fn from(string: &'a str) -> Self {
            Self(string.into())
        }
    }

    #[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
    pub struct DummyAtom(String);

    impl ToCss for DummyAtom {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where
            W: fmt::Write,
        {
            serialize_identifier(&self.0, dest)
        }
    }

    impl From<String> for DummyAtom {
        fn from(string: String) -> Self {
            DummyAtom(string)
        }
    }

    impl<'a> From<&'a str> for DummyAtom {
        fn from(string: &'a str) -> Self {
            DummyAtom(string.into())
        }
    }

    impl<'i> Parser<'i> for DummyParser {
        type Impl = DummySelectorImpl;
        type Error = SelectorParseErrorKind<'i>;

        fn parse_slotted(&self) -> bool {
            true
        }

        fn parse_nth_child_of(&self) -> bool {
            true
        }

        fn parse_is_and_where(&self) -> bool {
            true
        }

        fn parse_has(&self) -> bool {
            true
        }

        fn parse_parent_selector(&self) -> bool {
            true
        }

        fn parse_part(&self) -> bool {
            true
        }

        fn parse_non_ts_pseudo_class(
            &self,
            location: SourceLocation,
            name: CowRcStr<'i>,
        ) -> Result<PseudoClass, SelectorParseError<'i>> {
            match_ignore_ascii_case! { &name,
                "hover" => return Ok(PseudoClass::Hover),
                "active" => return Ok(PseudoClass::Active),
                _ => {}
            }
            Err(
                location.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(
                    name,
                )),
            )
        }

        fn parse_non_ts_functional_pseudo_class<'t>(
            &self,
            name: CowRcStr<'i>,
            parser: &mut CssParser<'i, 't>,
        ) -> Result<PseudoClass, SelectorParseError<'i>> {
            match_ignore_ascii_case! { &name,
                "lang" => {
                    let lang = parser.expect_ident_or_string()?.as_ref().to_owned();
                    return Ok(PseudoClass::Lang(lang));
                },
                _ => {}
            }
            Err(
                parser.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(
                    name,
                )),
            )
        }

        fn parse_pseudo_element(
            &self,
            location: SourceLocation,
            name: CowRcStr<'i>,
        ) -> Result<PseudoElement, SelectorParseError<'i>> {
            match_ignore_ascii_case! { &name,
                "before" => return Ok(PseudoElement::Before),
                "after" => return Ok(PseudoElement::After),
                _ => {}
            }
            Err(
                location.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(
                    name,
                )),
            )
        }

        fn parse_functional_pseudo_element<'t>(
            &self,
            name: CowRcStr<'i>,
            parser: &mut CssParser<'i, 't>,
        ) -> Result<PseudoElement, SelectorParseError<'i>> {
            match_ignore_ascii_case! {&name,
                "highlight" => return Ok(PseudoElement::Highlight(parser.expect_ident()?.as_ref().to_owned())),
                _ => {}
            }
            Err(
                parser.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(
                    name,
                )),
            )
        }

        fn default_namespace(&self) -> Option<DummyAtom> {
            self.default_ns.clone()
        }

        fn namespace_for_prefix(&self, prefix: &DummyAtom) -> Option<DummyAtom> {
            self.ns_prefixes.get(prefix).cloned()
        }
    }

    fn parse<'i>(
        input: &'i str,
    ) -> Result<SelectorList<DummySelectorImpl>, SelectorParseError<'i>> {
        parse_ns(input, &DummyParser::default())
    }

    fn parse_expected<'i, 'a>(
        input: &'i str,
        expected: Option<&'a str>,
    ) -> Result<SelectorList<DummySelectorImpl>, SelectorParseError<'i>> {
        parse_ns_expected(input, &DummyParser::default(), expected)
    }

    fn parse_ns<'i>(
        input: &'i str,
        parser: &DummyParser,
    ) -> Result<SelectorList<DummySelectorImpl>, SelectorParseError<'i>> {
        parse_ns_expected(input, parser, None)
    }

    fn parse_ns_expected<'i, 'a>(
        input: &'i str,
        parser: &DummyParser,
        expected: Option<&'a str>,
    ) -> Result<SelectorList<DummySelectorImpl>, SelectorParseError<'i>> {
        let mut parser_input = ParserInput::new(input);
        let result = SelectorList::parse(parser, &mut CssParser::new(&mut parser_input));
        if let Ok(ref selectors) = result {
            // We can't assume that the serialized parsed selector will equal
            // the input; for example, if there is no default namespace, '*|foo'
            // should serialize to 'foo'.
            assert_eq!(
                selectors.to_css_string(),
                match expected {
                    Some(x) => x,
                    None => input,
                }
            );
        }
        result
    }

    fn specificity(a: u32, b: u32, c: u32) -> u32 {
        a << 20 | b << 10 | c
    }

    #[test]
    fn test_empty() {
        let mut input = ParserInput::new(":empty");
        let list = SelectorList::parse(&DummyParser::default(), &mut CssParser::new(&mut input));
        assert!(list.is_ok());
    }

    const MATHML: &str = "http://www.w3.org/1998/Math/MathML";
    const SVG: &str = "http://www.w3.org/2000/svg";

    #[test]
    fn test_parsing() {
        assert!(parse("").is_err());
        assert!(parse(":lang(4)").is_err());
        assert!(parse(":lang(en US)").is_err());
        assert_eq!(
            parse("EeÉ"),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![Component::LocalName(LocalName {
                    name: DummyAtom::from("EeÉ"),
                    lower_name: DummyAtom::from("eeÉ"),
                })],
                specificity(0, 0, 1),
                Default::default(),
            )]))
        );
        assert_eq!(
            parse("|e"),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::ExplicitNoNamespace,
                    Component::LocalName(LocalName {
                        name: DummyAtom::from("e"),
                        lower_name: DummyAtom::from("e"),
                    }),
                ],
                specificity(0, 0, 1),
                Default::default(),
            )]))
        );
        // When the default namespace is not set, *| should be elided.
        // https://github.com/servo/servo/pull/17537
        assert_eq!(
            parse_expected("*|e", Some("e")),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![Component::LocalName(LocalName {
                    name: DummyAtom::from("e"),
                    lower_name: DummyAtom::from("e"),
                })],
                specificity(0, 0, 1),
                Default::default(),
            )]))
        );
        // When the default namespace is set, *| should _not_ be elided (as foo
        // is no longer equivalent to *|foo--the former is only for foo in the
        // default namespace).
        // https://github.com/servo/servo/issues/16020
        assert_eq!(
            parse_ns(
                "*|e",
                &DummyParser::default_with_namespace(DummyAtom::from("https://mozilla.org"))
            ),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::ExplicitAnyNamespace,
                    Component::LocalName(LocalName {
                        name: DummyAtom::from("e"),
                        lower_name: DummyAtom::from("e"),
                    }),
                ],
                specificity(0, 0, 1),
                Default::default(),
            )]))
        );
        assert_eq!(
            parse("*"),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![Component::ExplicitUniversalType],
                specificity(0, 0, 0),
                Default::default(),
            )]))
        );
        assert_eq!(
            parse("|*"),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::ExplicitNoNamespace,
                    Component::ExplicitUniversalType,
                ],
                specificity(0, 0, 0),
                Default::default(),
            )]))
        );
        assert_eq!(
            parse_expected("*|*", Some("*")),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![Component::ExplicitUniversalType],
                specificity(0, 0, 0),
                Default::default(),
            )]))
        );
        assert_eq!(
            parse_ns(
                "*|*",
                &DummyParser::default_with_namespace(DummyAtom::from("https://mozilla.org"))
            ),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::ExplicitAnyNamespace,
                    Component::ExplicitUniversalType,
                ],
                specificity(0, 0, 0),
                Default::default(),
            )]))
        );
        assert_eq!(
            parse(".foo:lang(en-US)"),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::Class(DummyAtom::from("foo")),
                    Component::NonTSPseudoClass(PseudoClass::Lang("en-US".to_owned())),
                ],
                specificity(0, 2, 0),
                Default::default(),
            )]))
        );
        assert_eq!(
            parse("#bar"),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![Component::ID(DummyAtom::from("bar"))],
                specificity(1, 0, 0),
                Default::default(),
            )]))
        );
        assert_eq!(
            parse("e.foo#bar"),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::LocalName(LocalName {
                        name: DummyAtom::from("e"),
                        lower_name: DummyAtom::from("e"),
                    }),
                    Component::Class(DummyAtom::from("foo")),
                    Component::ID(DummyAtom::from("bar")),
                ],
                specificity(1, 1, 1),
                Default::default(),
            )]))
        );
        assert_eq!(
            parse("e.foo #bar"),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::LocalName(LocalName {
                        name: DummyAtom::from("e"),
                        lower_name: DummyAtom::from("e"),
                    }),
                    Component::Class(DummyAtom::from("foo")),
                    Component::Combinator(Combinator::Descendant),
                    Component::ID(DummyAtom::from("bar")),
                ],
                specificity(1, 1, 1),
                Default::default(),
            )]))
        );
        // Default namespace does not apply to attribute selectors
        // https://github.com/mozilla/servo/pull/1652
        let mut parser = DummyParser::default();
        assert_eq!(
            parse_ns("[Foo]", &parser),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![Component::AttributeInNoNamespaceExists {
                    local_name: DummyAtom::from("Foo"),
                    local_name_lower: DummyAtom::from("foo"),
                }],
                specificity(0, 1, 0),
                Default::default(),
            )]))
        );
        assert!(parse_ns("svg|circle", &parser).is_err());
        parser
            .ns_prefixes
            .insert(DummyAtom("svg".into()), DummyAtom(SVG.into()));
        assert_eq!(
            parse_ns("svg|circle", &parser),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::Namespace(DummyAtom("svg".into()), SVG.into()),
                    Component::LocalName(LocalName {
                        name: DummyAtom::from("circle"),
                        lower_name: DummyAtom::from("circle"),
                    }),
                ],
                specificity(0, 0, 1),
                Default::default(),
            )]))
        );
        assert_eq!(
            parse_ns("svg|*", &parser),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::Namespace(DummyAtom("svg".into()), SVG.into()),
                    Component::ExplicitUniversalType,
                ],
                specificity(0, 0, 0),
                Default::default(),
            )]))
        );
        // Default namespace does not apply to attribute selectors
        // https://github.com/mozilla/servo/pull/1652
        // but it does apply to implicit type selectors
        // https://github.com/servo/rust-selectors/pull/82
        parser.default_ns = Some(MATHML.into());
        assert_eq!(
            parse_ns("[Foo]", &parser),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::DefaultNamespace(MATHML.into()),
                    Component::AttributeInNoNamespaceExists {
                        local_name: DummyAtom::from("Foo"),
                        local_name_lower: DummyAtom::from("foo"),
                    },
                ],
                specificity(0, 1, 0),
                Default::default(),
            )]))
        );
        // Default namespace does apply to type selectors
        assert_eq!(
            parse_ns("e", &parser),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::DefaultNamespace(MATHML.into()),
                    Component::LocalName(LocalName {
                        name: DummyAtom::from("e"),
                        lower_name: DummyAtom::from("e"),
                    }),
                ],
                specificity(0, 0, 1),
                Default::default(),
            )]))
        );
        assert_eq!(
            parse_ns("*", &parser),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::DefaultNamespace(MATHML.into()),
                    Component::ExplicitUniversalType,
                ],
                specificity(0, 0, 0),
                Default::default(),
            )]))
        );
        assert_eq!(
            parse_ns("*|*", &parser),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::ExplicitAnyNamespace,
                    Component::ExplicitUniversalType,
                ],
                specificity(0, 0, 0),
                Default::default(),
            )]))
        );
        // Default namespace applies to universal and type selectors inside :not and :matches,
        // but not otherwise.
        assert_eq!(
            parse_ns(":not(.cl)", &parser),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::DefaultNamespace(MATHML.into()),
                    Component::Negation(
                        vec![Selector::from_vec(
                            vec![Component::Class(DummyAtom::from("cl"))],
                            specificity(0, 1, 0),
                            Default::default(),
                        )]
                        .into_boxed_slice()
                    ),
                ],
                specificity(0, 1, 0),
                Default::default(),
            )]))
        );
        assert_eq!(
            parse_ns(":not(*)", &parser),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::DefaultNamespace(MATHML.into()),
                    Component::Negation(
                        vec![Selector::from_vec(
                            vec![
                                Component::DefaultNamespace(MATHML.into()),
                                Component::ExplicitUniversalType,
                            ],
                            specificity(0, 0, 0),
                            Default::default(),
                        )]
                        .into_boxed_slice(),
                    ),
                ],
                specificity(0, 0, 0),
                Default::default(),
            )]))
        );
        assert_eq!(
            parse_ns(":not(e)", &parser),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::DefaultNamespace(MATHML.into()),
                    Component::Negation(
                        vec![Selector::from_vec(
                            vec![
                                Component::DefaultNamespace(MATHML.into()),
                                Component::LocalName(LocalName {
                                    name: DummyAtom::from("e"),
                                    lower_name: DummyAtom::from("e"),
                                }),
                            ],
                            specificity(0, 0, 1),
                            Default::default(),
                        ),]
                        .into_boxed_slice()
                    ),
                ],
                specificity(0, 0, 1),
                Default::default(),
            )]))
        );
        assert_eq!(
            parse("[attr|=\"foo\"]"),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![Component::AttributeInNoNamespace {
                    local_name: DummyAtom::from("attr"),
                    operator: AttrSelectorOperator::DashMatch,
                    value: DummyAttrValue::from("foo"),
                    case_sensitivity: ParsedCaseSensitivity::CaseSensitive,
                }],
                specificity(0, 1, 0),
                Default::default(),
            )]))
        );
        // https://github.com/mozilla/servo/issues/1723
        assert_eq!(
            parse("::before"),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::Combinator(Combinator::PseudoElement),
                    Component::PseudoElement(PseudoElement::Before),
                ],
                specificity(0, 0, 1),
                SelectorFlags::HAS_PSEUDO,
            )]))
        );
        assert_eq!(
            parse("::before:hover"),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::Combinator(Combinator::PseudoElement),
                    Component::PseudoElement(PseudoElement::Before),
                    Component::NonTSPseudoClass(PseudoClass::Hover),
                ],
                specificity(0, 1, 1),
                SelectorFlags::HAS_PSEUDO,
            )]))
        );
        assert_eq!(
            parse("::before:hover:hover"),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::Combinator(Combinator::PseudoElement),
                    Component::PseudoElement(PseudoElement::Before),
                    Component::NonTSPseudoClass(PseudoClass::Hover),
                    Component::NonTSPseudoClass(PseudoClass::Hover),
                ],
                specificity(0, 2, 1),
                SelectorFlags::HAS_PSEUDO,
            )]))
        );
        assert!(parse("::before:hover:lang(foo)").is_err());
        assert!(parse("::before:hover .foo").is_err());
        assert!(parse("::before .foo").is_err());
        assert!(parse("::before ~ bar").is_err());
        assert!(parse("::before:active").is_ok());

        // https://github.com/servo/servo/issues/15335
        assert!(parse(":: before").is_err());
        assert_eq!(
            parse("div ::after"),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::LocalName(LocalName {
                        name: DummyAtom::from("div"),
                        lower_name: DummyAtom::from("div"),
                    }),
                    Component::Combinator(Combinator::Descendant),
                    Component::Combinator(Combinator::PseudoElement),
                    Component::PseudoElement(PseudoElement::After),
                ],
                specificity(0, 0, 2),
                SelectorFlags::HAS_PSEUDO,
            )]))
        );
        assert_eq!(
            parse("#d1 > .ok"),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::ID(DummyAtom::from("d1")),
                    Component::Combinator(Combinator::Child),
                    Component::Class(DummyAtom::from("ok")),
                ],
                (1 << 20) + (1 << 10) + (0 << 0),
                Default::default(),
            )]))
        );
        parser.default_ns = None;
        assert!(parse(":not(#provel.old)").is_ok());
        assert!(parse(":not(#provel > old)").is_ok());
        assert!(parse("table[rules]:not([rules=\"none\"]):not([rules=\"\"])").is_ok());
        // https://github.com/servo/servo/issues/16017
        assert_eq!(
            parse_ns(":not(*)", &parser),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![Component::Negation(
                    vec![Selector::from_vec(
                        vec![Component::ExplicitUniversalType],
                        specificity(0, 0, 0),
                        Default::default(),
                    )]
                    .into_boxed_slice()
                )],
                specificity(0, 0, 0),
                Default::default(),
            )]))
        );
        assert_eq!(
            parse_ns(":not(|*)", &parser),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![Component::Negation(
                    vec![Selector::from_vec(
                        vec![
                            Component::ExplicitNoNamespace,
                            Component::ExplicitUniversalType,
                        ],
                        specificity(0, 0, 0),
                        Default::default(),
                    )]
                    .into_boxed_slice(),
                )],
                specificity(0, 0, 0),
                Default::default(),
            )]))
        );
        // *| should be elided if there is no default namespace.
        // https://github.com/servo/servo/pull/17537
        assert_eq!(
            parse_ns_expected(":not(*|*)", &parser, Some(":not(*)")),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![Component::Negation(
                    vec![Selector::from_vec(
                        vec![Component::ExplicitUniversalType],
                        specificity(0, 0, 0),
                        Default::default()
                    )]
                    .into_boxed_slice()
                )],
                specificity(0, 0, 0),
                Default::default(),
            )]))
        );

        assert!(parse("::highlight(foo)").is_ok());

        assert!(parse("::slotted()").is_err());
        assert!(parse("::slotted(div)").is_ok());
        assert!(parse("::slotted(div).foo").is_err());
        assert!(parse("::slotted(div + bar)").is_err());
        assert!(parse("::slotted(div) + foo").is_err());

        assert!(parse("::part()").is_err());
        assert!(parse("::part(42)").is_err());
        assert!(parse("::part(foo bar)").is_ok());
        assert!(parse("::part(foo):hover").is_ok());
        assert!(parse("::part(foo) + bar").is_err());

        assert!(parse("div ::slotted(div)").is_ok());
        assert!(parse("div + slot::slotted(div)").is_ok());
        assert!(parse("div + slot::slotted(div.foo)").is_ok());
        assert!(parse("slot::slotted(div,foo)::first-line").is_err());
        assert!(parse("::slotted(div)::before").is_ok());
        assert!(parse("slot::slotted(div,foo)").is_err());

        assert!(parse("foo:where()").is_ok());
        assert!(parse("foo:where(div, foo, .bar baz)").is_ok());
        assert!(parse_expected("foo:where(::before)", Some("foo:where()")).is_ok());
    }

    #[test]
    fn parent_selector() {
        assert!(parse("foo &").is_ok());
        assert_eq!(
            parse("#foo &.bar"),
            Ok(SelectorList::from_vec(vec![Selector::from_vec(
                vec![
                    Component::ID(DummyAtom::from("foo")),
                    Component::Combinator(Combinator::Descendant),
                    Component::ParentSelector,
                    Component::Class(DummyAtom::from("bar")),
                ],
                (1 << 20) + (1 << 10) + (0 << 0),
                SelectorFlags::HAS_PARENT
            )]))
        );

        let parent = parse(".bar, div .baz").unwrap();
        let child = parse("#foo &.bar").unwrap();
        assert_eq!(
            SelectorList::from_vec(vec![child.0[0].replace_parent_selector(&parent.0)]),
            parse("#foo :is(.bar, div .baz).bar").unwrap()
        );

        let has_child = parse("#foo:has(&.bar)").unwrap();
        assert_eq!(
            SelectorList::from_vec(vec![has_child.0[0].replace_parent_selector(&parent.0)]),
            parse("#foo:has(:is(.bar, div .baz).bar)").unwrap()
        );

        let child = parse("#foo").unwrap();
        assert_eq!(
            SelectorList::from_vec(vec![child.0[0].replace_parent_selector(&parent.0)]),
            parse(":is(.bar, div .baz) #foo").unwrap()
        );
    }

    #[test]
    fn test_pseudo_iter() {
        let selector = &parse("q::before").unwrap().0[0];
        assert!(!selector.is_universal());
        let mut iter = selector.iter();
        assert_eq!(
            iter.next(),
            Some(&Component::PseudoElement(PseudoElement::Before))
        );
        assert_eq!(iter.next(), None);
        let combinator = iter.next_sequence();
        assert_eq!(combinator, Some(Combinator::PseudoElement));
        assert!(matches!(iter.next(), Some(&Component::LocalName(..))));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_sequence(), None);
    }

    #[test]
    fn test_universal() {
        let selector = &parse_ns(
            "*|*::before",
            &DummyParser::default_with_namespace(DummyAtom::from("https://mozilla.org")),
        )
        .unwrap()
        .0[0];
        assert!(selector.is_universal());
    }

    #[test]
    fn test_empty_pseudo_iter() {
        let selector = &parse("::before").unwrap().0[0];
        assert!(selector.is_universal());
        let mut iter = selector.iter();
        assert_eq!(
            iter.next(),
            Some(&Component::PseudoElement(PseudoElement::Before))
        );
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_sequence(), Some(Combinator::PseudoElement));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_sequence(), None);
    }

    struct TestVisitor {
        seen: Vec<String>,
    }

    impl SelectorVisitor for TestVisitor {
        type Impl = DummySelectorImpl;

        fn visit_simple_selector(&mut self, s: &Component<DummySelectorImpl>) -> bool {
            let mut dest = String::new();
            s.to_css(&mut dest).unwrap();
            self.seen.push(dest);
            true
        }
    }

    #[test]
    fn visitor() {
        let mut test_visitor = TestVisitor { seen: vec![] };
        parse(":not(:hover) ~ label").unwrap().0[0].visit(&mut test_visitor);
        assert!(test_visitor.seen.contains(&":hover".into()));

        let mut test_visitor = TestVisitor { seen: vec![] };
        parse("::before:hover").unwrap().0[0].visit(&mut test_visitor);
        assert!(test_visitor.seen.contains(&":hover".into()));
    }
}
