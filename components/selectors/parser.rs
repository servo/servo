/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use attr::{AttrSelectorWithNamespace, ParsedAttrSelectorOperation, AttrSelectorOperator};
use attr::{ParsedCaseSensitivity, SELECTOR_WHITESPACE, NamespaceConstraint};
use bloom::BLOOM_HASH_MASK;
use builder::{SelectorBuilder, SpecificityAndFlags};
use context::QuirksMode;
use cssparser::{ParseError, ParseErrorKind, BasicParseError, BasicParseErrorKind};
use cssparser::{SourceLocation, CowRcStr, Delimiter};
use cssparser::{Token, Parser as CssParser, parse_nth, ToCss, serialize_identifier, CssStringWriter};
use precomputed_hash::PrecomputedHash;
use servo_arc::ThinArc;
use sink::Push;
use smallvec::SmallVec;
#[allow(unused_imports)] use std::ascii::AsciiExt;
use std::borrow::{Borrow, Cow};
use std::fmt::{self, Display, Debug, Write};
use std::iter::Rev;
use std::slice;
use visitor::SelectorVisitor;

/// A trait that represents a pseudo-element.
pub trait PseudoElement : Sized + ToCss {
    /// The `SelectorImpl` this pseudo-element is used for.
    type Impl: SelectorImpl;

    /// Whether the pseudo-element supports a given state selector to the right
    /// of it.
    fn supports_pseudo_class(
        &self,
        _pseudo_class: &<Self::Impl as SelectorImpl>::NonTSPseudoClass,
    ) -> bool {
        false
    }
}

fn to_ascii_lowercase(s: &str) -> Cow<str> {
    if let Some(first_uppercase) = s.bytes().position(|byte| byte >= b'A' && byte <= b'Z') {
        let mut string = s.to_owned();
        string[first_uppercase..].make_ascii_lowercase();
        string.into()
    } else {
        s.into()
    }
}

pub type SelectorParseError<'i> = ParseError<'i, SelectorParseErrorKind<'i>>;

#[derive(Clone, Debug, PartialEq)]
pub enum SelectorParseErrorKind<'i> {
    PseudoElementInComplexSelector,
    NoQualifiedNameInAttributeSelector(Token<'i>),
    EmptySelector,
    DanglingCombinator,
    NonSimpleSelectorInNegation,
    NonCompoundSelector,
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
    EmptyNegation,
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
        pub trait SelectorImpl: Clone + Sized + 'static {
            type ExtraMatchingData: Sized + Default + 'static;
            type AttrValue: $($InSelector)*;
            type Identifier: $($InSelector)* + PrecomputedHash;
            type ClassName: $($InSelector)* + PrecomputedHash;
            type LocalName: $($InSelector)* + Borrow<Self::BorrowedLocalName> + PrecomputedHash;
            type NamespaceUrl: $($CommonBounds)* + Default + Borrow<Self::BorrowedNamespaceUrl> + PrecomputedHash;
            type NamespacePrefix: $($InSelector)* + Default;
            type BorrowedNamespaceUrl: ?Sized + Eq;
            type BorrowedLocalName: ?Sized + Eq;

            /// non tree-structural pseudo-classes
            /// (see: https://drafts.csswg.org/selectors/#structural-pseudos)
            type NonTSPseudoClass: $($CommonBounds)* + Sized + ToCss + SelectorMethods<Impl = Self>;

            /// pseudo-elements
            type PseudoElement: $($CommonBounds)* + PseudoElement<Impl = Self>;

            /// Returns whether the given pseudo class is :active or :hover.
            #[inline]
            fn is_active_or_hover(pseudo_class: &Self::NonTSPseudoClass) -> bool;
        }
    }
}

macro_rules! with_bounds {
    ( [ $( $CommonBounds: tt )* ] [ $( $FromStr: tt )* ]) => {
        with_all_bounds! {
            [$($CommonBounds)* + $($FromStr)* + Display]
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

    /// Whether the name is a pseudo-element that can be specified with
    /// the single colon syntax in addition to the double-colon syntax.
    fn pseudo_element_allows_single_colon(name: &str) -> bool {
        is_css2_pseudo_element(name)
    }

    /// Whether to parse the `::slotted()` pseudo-element.
    fn parse_slotted(&self) -> bool {
        false
    }

    /// This function can return an "Err" pseudo-element in order to support CSS2.1
    /// pseudo-elements.
    fn parse_non_ts_pseudo_class(
        &self,
        location: SourceLocation,
        name: CowRcStr<'i>,
    ) -> Result<<Self::Impl as SelectorImpl>::NonTSPseudoClass, ParseError<'i, Self::Error>> {
        Err(location.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name)))
    }

    fn parse_non_ts_functional_pseudo_class<'t>(
        &self,
        name: CowRcStr<'i>,
        arguments: &mut CssParser<'i, 't>,
    ) -> Result<<Self::Impl as SelectorImpl>::NonTSPseudoClass, ParseError<'i, Self::Error>> {
        Err(arguments.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name)))
    }

    fn parse_pseudo_element(
        &self,
        location: SourceLocation,
        name: CowRcStr<'i>,
    ) -> Result<<Self::Impl as SelectorImpl>::PseudoElement, ParseError<'i, Self::Error>> {
        Err(location.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name)))
    }

    fn parse_functional_pseudo_element<'t>(
        &self,
        name: CowRcStr<'i>,
        arguments: &mut CssParser<'i, 't>,
    ) -> Result<<Self::Impl as SelectorImpl>::PseudoElement, ParseError<'i, Self::Error>> {
        Err(arguments.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name)))
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectorList<Impl: SelectorImpl>(pub SmallVec<[Selector<Impl>; 1]>);

impl<Impl: SelectorImpl> SelectorList<Impl> {
    /// Parse a comma-separated list of Selectors.
    /// <https://drafts.csswg.org/selectors/#grouping>
    ///
    /// Return the Selectors or Err if there is an invalid selector.
    pub fn parse<'i, 't, P>(
        parser: &P,
        input: &mut CssParser<'i, 't>,
    ) -> Result<Self, ParseError<'i, P::Error>>
    where
        P: Parser<'i, Impl=Impl>,
    {
        let mut values = SmallVec::new();
        loop {
            values.push(input.parse_until_before(Delimiter::Comma, |input| parse_selector(parser, input))?);
            match input.next() {
                Err(_) => return Ok(SelectorList(values)),
                Ok(&Token::Comma) => continue,
                Ok(_) => unreachable!(),
            }
        }
    }

    /// Creates a SelectorList from a Vec of selectors. Used in tests.
    pub fn from_vec(v: Vec<Selector<Impl>>) -> Self {
        SelectorList(SmallVec::from_vec(v))
    }
}

/// Parses one compound selector suitable for nested stuff like ::-moz-any, etc.
fn parse_inner_compound_selector<'i, 't, P, Impl>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
) -> Result<Selector<Impl>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl=Impl>,
    Impl: SelectorImpl,
{
    let location = input.current_source_location();
    let selector = Selector::parse(parser, input)?;
    // Ensure they're actually all compound selectors.
    if selector.iter_raw_match_order().any(|s| s.is_combinator()) {
        return Err(location.new_custom_error(
            SelectorParseErrorKind::NonCompoundSelector
        ))
    }

    Ok(selector)
}

/// Parse a comma separated list of compound selectors.
pub fn parse_compound_selector_list<'i, 't, P, Impl>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
) -> Result<Box<[Selector<Impl>]>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl=Impl>,
    Impl: SelectorImpl,
{
    input.parse_comma_separated(|input| {
        parse_inner_compound_selector(parser, input)
    }).map(|selectors| selectors.into_boxed_slice())
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
#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq)]
pub struct AncestorHashes {
    pub packed_hashes: [u32; 3],
}

impl AncestorHashes {
    pub fn new<Impl: SelectorImpl>(
        selector: &Selector<Impl>,
        quirks_mode: QuirksMode,
    ) -> Self {
        Self::from_iter(selector.iter(), quirks_mode)
    }

    fn from_iter<Impl: SelectorImpl>(
        iter: SelectorIter<Impl>,
        quirks_mode: QuirksMode,
    ) -> Self {
        // Compute ancestor hashes for the bloom filter.
        let mut hashes = [0u32; 4];
        let mut hash_iter = AncestorIter::new(iter)
                             .filter_map(|x| x.ancestor_hash(quirks_mode));
        for i in 0..4 {
            hashes[i] = match hash_iter.next() {
                Some(x) => x & BLOOM_HASH_MASK,
                None => break,
            }
        }

        // Now, pack the fourth hash (if it exists) into the upper byte of each of
        // the other three hashes.
        let fourth = hashes[3];
        if fourth != 0 {
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

pub trait SelectorMethods {
    type Impl: SelectorImpl;

    fn visit<V>(&self, visitor: &mut V) -> bool
    where
        V: SelectorVisitor<Impl = Self::Impl>;
}

impl<Impl: SelectorImpl> SelectorMethods for Selector<Impl> {
    type Impl = Impl;

    fn visit<V>(&self, visitor: &mut V) -> bool
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

impl<Impl: SelectorImpl> SelectorMethods for Component<Impl> {
    type Impl = Impl;

    fn visit<V>(&self, visitor: &mut V) -> bool
    where
        V: SelectorVisitor<Impl = Impl>,
    {
        use self::Component::*;
        if !visitor.visit_simple_selector(self) {
            return false;
        }

        match *self {
            Slotted(ref selectors) => {
                for selector in selectors.iter() {
                    if !selector.visit(visitor) {
                        return false;
                    }
                }
            }
            Negation(ref negated) => {
                for component in negated.iter() {
                    if !component.visit(visitor) {
                        return false;
                    }
                }
            }

            AttributeInNoNamespaceExists { ref local_name, ref local_name_lower } => {
                if !visitor.visit_attribute_selector(
                    &NamespaceConstraint::Specific(&namespace_empty_string::<Impl>()),
                    local_name,
                    local_name_lower,
                ) {
                    return false;
                }
            }
            AttributeInNoNamespace { ref local_name, ref local_name_lower, never_matches, .. }
            if !never_matches => {
                if !visitor.visit_attribute_selector(
                    &NamespaceConstraint::Specific(&namespace_empty_string::<Impl>()),
                    local_name,
                    local_name_lower,
                ) {
                    return false;
                }
            }
            AttributeOther(ref attr_selector) if !attr_selector.never_matches => {
                if !visitor.visit_attribute_selector(
                    &attr_selector.namespace(),
                    &attr_selector.local_name,
                    &attr_selector.local_name_lower,
                ) {
                    return false;
                }
            }

            NonTSPseudoClass(ref pseudo_class) => {
                if !pseudo_class.visit(visitor) {
                    return false;
                }
            },
            _ => {}
        }

        true
    }
}

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
#[derive(Clone, Eq, PartialEq)]
pub struct Selector<Impl: SelectorImpl>(ThinArc<SpecificityAndFlags, Component<Impl>>);

impl<Impl: SelectorImpl> Selector<Impl> {
    #[inline]
    pub fn specificity(&self) -> u32 {
        self.0.header.header.specificity()
    }

    #[inline]
    pub fn has_pseudo_element(&self) -> bool {
        self.0.header.header.has_pseudo_element()
    }

    #[inline]
    pub fn is_slotted(&self) -> bool {
        self.0.header.header.is_slotted()
    }

    #[inline]
    pub fn pseudo_element(&self) -> Option<&Impl::PseudoElement> {
        if !self.has_pseudo_element() {
            return None
        }

        for component in self.iter() {
            if let Component::PseudoElement(ref pseudo) = *component {
                return Some(pseudo)
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
        self.iter_raw_match_order().all(|c| matches!(*c,
            Component::ExplicitUniversalType |
            Component::ExplicitAnyNamespace |
            Component::Combinator(Combinator::PseudoElement) |
            Component::PseudoElement(..)
        ))
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

    /// Returns an iterator over this selector in matching order (right-to-left),
    /// skipping the rightmost |offset| Components.
    #[inline]
    pub fn iter_from(&self, offset: usize) -> SelectorIter<Impl> {
        let iter = self.0.slice[offset..].iter();
        SelectorIter {
            iter: iter,
            next_combinator: None,
        }
    }

    /// Returns the combinator at index `index` (zero-indexed from the right),
    /// or panics if the component is not a combinator.
    #[inline]
    pub fn combinator_at_match_order(&self, index: usize) -> Combinator {
        match self.0.slice[index] {
            Component::Combinator(c) => c,
            ref other => {
                panic!("Not a combinator: {:?}, {:?}, index: {}",
                       other, self, index)
            }
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
            ref other => {
                panic!("Not a combinator: {:?}, {:?}, index: {}",
                       other, self, index)
            }
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
    pub fn from_vec(vec: Vec<Component<Impl>>, specificity_and_flags: u32) -> Self {
        let mut builder = SelectorBuilder::default();
        for component in vec.into_iter() {
            if let Some(combinator) = component.as_combinator() {
                builder.push_combinator(combinator);
            } else {
                builder.push_simple_selector(component);
            }
        }
        let spec = SpecificityAndFlags(specificity_and_flags);
        Selector(builder.build_with_specificity_and_flags(spec))
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
        debug_assert!(self.next_combinator.is_none(),
                      "You should call next_sequence!");
        match self.iter.next() {
            None => None,
            Some(&Component::Combinator(c)) => {
                self.next_combinator = Some(c);
                None
            },
            Some(x) => Some(x),
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
            if self.0.next_sequence().map_or(true, |x| matches!(x, Combinator::Child | Combinator::Descendant)) {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Combinator {
    Child,  //  >
    Descendant,  // space
    NextSibling,  // +
    LaterSibling,  // ~
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
}

impl Combinator {
    /// Returns true if this combinator is a child or descendant combinator.
    #[inline]
    pub fn is_ancestor(&self) -> bool {
        matches!(*self,
                 Combinator::Child |
                 Combinator::Descendant |
                 Combinator::PseudoElement |
                 Combinator::SlotAssignment)
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

/// A CSS simple selector or combinator. We store both in the same enum for
/// optimal packing and cache performance, see [1].
///
/// [1] https://bugzilla.mozilla.org/show_bug.cgi?id=1357973
#[derive(Clone, Eq, PartialEq)]
pub enum Component<Impl: SelectorImpl> {
    Combinator(Combinator),

    ExplicitAnyNamespace,
    ExplicitNoNamespace,
    DefaultNamespace(Impl::NamespaceUrl),
    Namespace(Impl::NamespacePrefix, Impl::NamespaceUrl),

    ExplicitUniversalType,
    LocalName(LocalName<Impl>),

    ID(Impl::Identifier),
    Class(Impl::ClassName),

    AttributeInNoNamespaceExists {
        local_name: Impl::LocalName,
        local_name_lower: Impl::LocalName,
    },
    AttributeInNoNamespace {
        local_name: Impl::LocalName,
        local_name_lower: Impl::LocalName,
        operator: AttrSelectorOperator,
        value: Impl::AttrValue,
        case_sensitivity: ParsedCaseSensitivity,
        never_matches: bool,
    },
    // Use a Box in the less common cases with more data to keep size_of::<Component>() small.
    AttributeOther(Box<AttrSelectorWithNamespace<Impl>>),

    /// Pseudo-classes
    ///
    /// CSS3 Negation only takes a simple simple selector, but we still need to
    /// treat it as a compound selector because it might be a type selector
    /// which we represent as a namespace and a localname.
    ///
    /// Note: if/when we upgrade this to CSS4, which supports combinators, we
    /// need to think about how this should interact with
    /// visit_complex_selector, and what the consumers of those APIs should do
    /// about the presence of combinators in negation.
    Negation(Box<[Component<Impl>]>),
    FirstChild, LastChild, OnlyChild,
    Root,
    Empty,
    Scope,
    NthChild(i32, i32),
    NthLastChild(i32, i32),
    NthOfType(i32, i32),
    NthLastOfType(i32, i32),
    FirstOfType,
    LastOfType,
    OnlyOfType,
    NonTSPseudoClass(Impl::NonTSPseudoClass),
    /// The ::slotted() pseudo-element (which isn't actually a pseudo-element,
    /// and probably should be a pseudo-class):
    ///
    /// https://drafts.csswg.org/css-scoping/#slotted-pseudo
    ///
    /// The selector here is a compound selector, that is, no combinators.
    ///
    /// NOTE(emilio): This should support a list of selectors, but as of this
    /// writing no other browser does, and that allows them to put ::slotted()
    /// in the rule hash, so we do that too.
    Slotted(Selector<Impl>),
    PseudoElement(Impl::PseudoElement),
}

impl<Impl: SelectorImpl> Component<Impl> {
    /// Compute the ancestor hash to check against the bloom filter.
    fn ancestor_hash(&self, quirks_mode: QuirksMode) -> Option<u32> {
        match *self {
            Component::LocalName(LocalName { ref name, ref lower_name }) => {
                // Only insert the local-name into the filter if it's all
                // lowercase.  Otherwise we would need to test both hashes, and
                // our data structures aren't really set up for that.
                if name == lower_name {
                    Some(name.precomputed_hash())
                } else {
                    None
                }
            },
            Component::DefaultNamespace(ref url) |
            Component::Namespace(_, ref url) => {
                Some(url.precomputed_hash())
            },
            // In quirks mode, class and id selectors should match
            // case-insensitively, so just avoid inserting them into the filter.
            Component::ID(ref id) if quirks_mode != QuirksMode::Quirks => {
                Some(id.precomputed_hash())
            },
            Component::Class(ref class) if quirks_mode != QuirksMode::Quirks => {
                Some(class.precomputed_hash())
            },
            _ => None,
        }
    }

    /// Returns true if this is a combinator.
    pub fn is_combinator(&self) -> bool {
        matches!(*self, Component::Combinator(_))
    }

    /// Returns the value as a combinator if applicable, None otherwise.
    pub fn as_combinator(&self) -> Option<Combinator> {
        match *self {
            Component::Combinator(c) => Some(c),
            _ => None,
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct LocalName<Impl: SelectorImpl> {
    pub name: Impl::LocalName,
    pub lower_name: Impl::LocalName,
}

impl<Impl: SelectorImpl> Debug for Selector<Impl> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Selector(")?;
        self.to_css(f)?;
        write!(f, ", specificity = 0x{:x})", self.specificity())
    }
}

impl<Impl: SelectorImpl> Debug for Component<Impl> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.to_css(f) }
}
impl<Impl: SelectorImpl> Debug for AttrSelectorWithNamespace<Impl> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.to_css(f) }
}
impl<Impl: SelectorImpl> Debug for LocalName<Impl> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.to_css(f) }
}

impl<Impl: SelectorImpl> ToCss for SelectorList<Impl> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let mut iter = self.0.iter();
        let first = iter.next()
            .expect("Empty SelectorList, should contain at least one selector");
        first.to_css(dest)?;
        for selector in iter {
            dest.write_str(", ")?;
            selector.to_css(dest)?;
        }
        Ok(())
    }
}

impl<Impl: SelectorImpl> ToCss for Selector<Impl> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
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

        let mut combinators = self.iter_raw_match_order().rev().filter(|x| x.is_combinator()).peekable();
        let compound_selectors = self.iter_raw_match_order().as_slice().split(|x| x.is_combinator()).rev();

        let mut combinators_exhausted = false;
        for compound in compound_selectors {
            debug_assert!(!combinators_exhausted);

            // https://drafts.csswg.org/cssom/#serializing-selectors

            if !compound.is_empty() {
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
                let (can_elide_namespace, first_non_namespace) = match &compound[0] {
                    &Component::ExplicitAnyNamespace |
                    &Component::ExplicitNoNamespace |
                    &Component::Namespace(_, _) => (false, 1),
                    &Component::DefaultNamespace(_) => (true, 1),
                    _ => (true, 0),
                };
                let mut perform_step_2 = true;
                if first_non_namespace == compound.len() - 1 {
                    match (combinators.peek(), &compound[first_non_namespace]) {
                        // We have to be careful here, because if there is a
                        // pseudo element "combinator" there isn't really just
                        // the one simple selector. Technically this compound
                        // selector contains the pseudo element selector as well
                        // -- Combinator::PseudoElement, just like
                        // Combinator::SlotAssignment, don't exist in the
                        // spec.
                        (Some(&&Component::Combinator(Combinator::PseudoElement)), _) |
                        (Some(&&Component::Combinator(Combinator::SlotAssignment)), _) => (),
                        (_, &Component::ExplicitUniversalType) => {
                            // Iterate over everything so we serialize the namespace
                            // too.
                            for simple in compound.iter() {
                                simple.to_css(dest)?;
                            }
                            // Skip step 2, which is an "otherwise".
                            perform_step_2 = false;
                        }
                        (_, _) => (),
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
                                continue
                            }
                        }
                        simple.to_css(dest)?;
                    }
                }
            }

            // 3. If this is not the last part of the chain of the selector
            //    append a single SPACE (U+0020), followed by the combinator
            //    ">", "+", "~", ">>", "||", as appropriate, followed by another
            //    single SPACE (U+0020) if the combinator was not whitespace, to
            //    s.
            match combinators.next() {
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

impl ToCss for Combinator {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            Combinator::Child => dest.write_str(" > "),
            Combinator::Descendant => dest.write_str(" "),
            Combinator::NextSibling => dest.write_str(" + "),
            Combinator::LaterSibling => dest.write_str(" ~ "),
            Combinator::PseudoElement => Ok(()),
            Combinator::SlotAssignment => Ok(()),
        }
    }
}

impl<Impl: SelectorImpl> ToCss for Component<Impl> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        use self::Component::*;

        /// Serialize <an+b> values (part of the CSS Syntax spec, but currently only used here).
        /// <https://drafts.csswg.org/css-syntax-3/#serialize-an-anb-value>
        fn write_affine<W>(dest: &mut W, a: i32, b: i32) -> fmt::Result where W: fmt::Write {
            match (a, b) {
                (0, 0) => dest.write_char('0'),

                (1, 0) => dest.write_char('n'),
                (_, 0) => write!(dest, "{}n", a),

                (0, _) => write!(dest, "{}", b),
                (1, _) => write!(dest, "n{:+}", b),
                (-1, _) => write!(dest, "-n{:+}", b),
                (_, _) => write!(dest, "{}n{:+}", a, b),
            }
        }

        match *self {
            Combinator(ref c) => {
                c.to_css(dest)
            }
            Slotted(ref selector) => {
                dest.write_str("::slotted(")?;
                selector.to_css(dest)?;
                dest.write_char(')')
            }
            PseudoElement(ref p) => {
                p.to_css(dest)
            }
            ID(ref s) => {
                dest.write_char('#')?;
                display_to_css_identifier(s, dest)
            }
            Class(ref s) => {
                dest.write_char('.')?;
                display_to_css_identifier(s, dest)
            }
            LocalName(ref s) => s.to_css(dest),
            ExplicitUniversalType => dest.write_char('*'),

            DefaultNamespace(_) => Ok(()),
            ExplicitNoNamespace => dest.write_char('|'),
            ExplicitAnyNamespace => dest.write_str("*|"),
            Namespace(ref prefix, _) => {
                display_to_css_identifier(prefix, dest)?;
                dest.write_char('|')
            }

            AttributeInNoNamespaceExists { ref local_name, .. } => {
                dest.write_char('[')?;
                display_to_css_identifier(local_name, dest)?;
                dest.write_char(']')
            }
            AttributeInNoNamespace { ref local_name, operator, ref value, case_sensitivity, .. } => {
                dest.write_char('[')?;
                display_to_css_identifier(local_name, dest)?;
                operator.to_css(dest)?;
                dest.write_char('"')?;
                write!(CssStringWriter::new(dest), "{}", value)?;
                dest.write_char('"')?;
                match case_sensitivity {
                    ParsedCaseSensitivity::CaseSensitive |
                    ParsedCaseSensitivity::AsciiCaseInsensitiveIfInHtmlElementInHtmlDocument => {},
                    ParsedCaseSensitivity::AsciiCaseInsensitive => dest.write_str(" i")?,
                }
                dest.write_char(']')
            }
            AttributeOther(ref attr_selector) => attr_selector.to_css(dest),

            // Pseudo-classes
            Negation(ref arg) => {
                dest.write_str(":not(")?;
                for component in arg.iter() {
                    component.to_css(dest)?;
                }
                dest.write_str(")")
            }

            FirstChild => dest.write_str(":first-child"),
            LastChild => dest.write_str(":last-child"),
            OnlyChild => dest.write_str(":only-child"),
            Root => dest.write_str(":root"),
            Empty => dest.write_str(":empty"),
            Scope => dest.write_str(":scope"),
            FirstOfType => dest.write_str(":first-of-type"),
            LastOfType => dest.write_str(":last-of-type"),
            OnlyOfType => dest.write_str(":only-of-type"),
            NthChild(a, b) | NthLastChild(a, b) | NthOfType(a, b) | NthLastOfType(a, b) => {
                match *self {
                    NthChild(_, _) => dest.write_str(":nth-child(")?,
                    NthLastChild(_, _) => dest.write_str(":nth-last-child(")?,
                    NthOfType(_, _) => dest.write_str(":nth-of-type(")?,
                    NthLastOfType(_, _) => dest.write_str(":nth-last-of-type(")?,
                    _ => unreachable!(),
                }
                write_affine(dest, a, b)?;
                dest.write_char(')')
            }
            NonTSPseudoClass(ref pseudo) => pseudo.to_css(dest),
        }
    }
}

impl<Impl: SelectorImpl> ToCss for AttrSelectorWithNamespace<Impl> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_char('[')?;
        match self.namespace {
            NamespaceConstraint::Specific((ref prefix, _)) => {
                display_to_css_identifier(prefix, dest)?;
                dest.write_char('|')?
            }
            NamespaceConstraint::Any => {
                dest.write_str("*|")?
            }
        }
        display_to_css_identifier(&self.local_name, dest)?;
        match self.operation {
            ParsedAttrSelectorOperation::Exists => {},
            ParsedAttrSelectorOperation::WithValue {
                operator, case_sensitivity, ref expected_value
            } => {
                operator.to_css(dest)?;
                dest.write_char('"')?;
                write!(CssStringWriter::new(dest), "{}", expected_value)?;
                dest.write_char('"')?;
                match case_sensitivity {
                    ParsedCaseSensitivity::CaseSensitive |
                    ParsedCaseSensitivity::AsciiCaseInsensitiveIfInHtmlElementInHtmlDocument => {},
                    ParsedCaseSensitivity::AsciiCaseInsensitive => dest.write_str(" i")?,
                }
            },
        }
        dest.write_char(']')
    }
}

impl<Impl: SelectorImpl> ToCss for LocalName<Impl> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        display_to_css_identifier(&self.name, dest)
    }
}

/// Serialize the output of Display as a CSS identifier
fn display_to_css_identifier<T: Display, W: fmt::Write>(x: &T, dest: &mut W) -> fmt::Result {
    // FIXME(SimonSapin): it is possible to avoid this heap allocation
    // by creating a stream adapter like cssparser::CssStringWriter
    // that holds and writes to `&mut W` and itself implements `fmt::Write`.
    //
    // I haven’t done this yet because it would require somewhat complex and fragile state machine
    // to support in `fmt::Write::write_char` cases that,
    // in `serialize_identifier` (which has the full value as a `&str` slice),
    // can be expressed as
    // `string.starts_with("--")`, `string == "-"`, `string.starts_with("-")`, etc.
    //
    // And I don’t even know if this would be a performance win: jemalloc is good at what it does
    // and the state machine might be slower than `serialize_identifier` as currently written.
    let string = x.to_string();

    serialize_identifier(&string, dest)
}

/// Build up a Selector.
/// selector : simple_selector_sequence [ combinator simple_selector_sequence ]* ;
///
/// `Err` means invalid selector.
fn parse_selector<'i, 't, P, Impl>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
) -> Result<Selector<Impl>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl=Impl>,
    Impl: SelectorImpl,
{
    let mut builder = SelectorBuilder::default();

    let mut has_pseudo_element;
    let mut slotted;
    'outer_loop: loop {
        // Parse a sequence of simple selectors.
        match parse_compound_selector(parser, input, &mut builder)? {
            Some((has_pseudo, slot)) => {
                has_pseudo_element = has_pseudo;
                slotted = slot;
            }
            None => {
                return Err(input.new_custom_error(if builder.has_combinators() {
                    SelectorParseErrorKind::DanglingCombinator
                } else {
                    SelectorParseErrorKind::EmptySelector
                }))
            }
        };

        if has_pseudo_element || slotted {
            break;
        }

        // Parse a combinator.
        let combinator;
        let mut any_whitespace = false;
        loop {
            let before_this_token = input.state();
            match input.next_including_whitespace() {
                Err(_e) => break 'outer_loop,
                Ok(&Token::WhiteSpace(_)) => any_whitespace = true,
                Ok(&Token::Delim('>')) => {
                    combinator = Combinator::Child;
                    break
                }
                Ok(&Token::Delim('+')) => {
                    combinator = Combinator::NextSibling;
                    break
                }
                Ok(&Token::Delim('~')) => {
                    combinator = Combinator::LaterSibling;
                    break
                }
                Ok(_) => {
                    input.reset(&before_this_token);
                    if any_whitespace {
                        combinator = Combinator::Descendant;
                        break
                    } else {
                        break 'outer_loop
                    }
                }
            }
        }
        builder.push_combinator(combinator);
    }

    Ok(Selector(builder.build(has_pseudo_element, slotted)))
}

impl<Impl: SelectorImpl> Selector<Impl> {
    /// Parse a selector, without any pseudo-element.
    pub fn parse<'i, 't, P>(
        parser: &P,
        input: &mut CssParser<'i, 't>,
    ) -> Result<Self, ParseError<'i, P::Error>>
    where
        P: Parser<'i, Impl=Impl>
    {
        let selector = parse_selector(parser, input)?;
        if selector.has_pseudo_element() {
            return Err(input.new_custom_error(SelectorParseErrorKind::PseudoElementInComplexSelector))
        }
        Ok(selector)
    }
}

/// * `Err(())`: Invalid selector, abort
/// * `Ok(false)`: Not a type selector, could be something else. `input` was not consumed.
/// * `Ok(true)`: Length 0 (`*|*`), 1 (`*|E` or `ns|*`) or 2 (`|E` or `ns|E`)
fn parse_type_selector<'i, 't, P, Impl, S>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
    sink: &mut S,
) -> Result<bool, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl=Impl>,
    Impl: SelectorImpl,
    S: Push<Component<Impl>>,
{
    match parse_qualified_name(parser, input, /* in_attr_selector = */ false) {
        Err(ParseError { kind: ParseErrorKind::Basic(BasicParseErrorKind::EndOfInput), .. }) |
        Ok(OptionalQName::None(_)) => Ok(false),
        Ok(OptionalQName::Some(namespace, local_name)) => {
            match namespace {
                QNamePrefix::ImplicitAnyNamespace => {}
                QNamePrefix::ImplicitDefaultNamespace(url) => {
                    sink.push(Component::DefaultNamespace(url))
                }
                QNamePrefix::ExplicitNamespace(prefix, url) => {
                    sink.push(match parser.default_namespace() {
                        Some(ref default_url) if url == *default_url => Component::DefaultNamespace(url),
                        _ => Component::Namespace(prefix, url),
                    })
                }
                QNamePrefix::ExplicitNoNamespace => {
                    sink.push(Component::ExplicitNoNamespace)
                }
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
                }
                QNamePrefix::ImplicitNoNamespace => {
                    unreachable!()  // Not returned with in_attr_selector = false
                }
            }
            match local_name {
                Some(name) => {
                    sink.push(Component::LocalName(LocalName {
                        lower_name: to_ascii_lowercase(&name).as_ref().into(),
                        name: name.as_ref().into(),
                    }))
                }
                None => {
                    sink.push(Component::ExplicitUniversalType)
                }
            }
            Ok(true)
        }
        Err(e) => Err(e)
    }
}

#[derive(Debug)]
enum SimpleSelectorParseResult<Impl: SelectorImpl> {
    SimpleSelector(Component<Impl>),
    PseudoElement(Impl::PseudoElement),
    SlottedPseudo(Selector<Impl>),
}

#[derive(Debug)]
enum QNamePrefix<Impl: SelectorImpl> {
    ImplicitNoNamespace, // `foo` in attr selectors
    ImplicitAnyNamespace, // `foo` in type selectors, without a default ns
    ImplicitDefaultNamespace(Impl::NamespaceUrl),  // `foo` in type selectors, with a default ns
    ExplicitNoNamespace,  // `|foo`
    ExplicitAnyNamespace,  // `*|foo`
    ExplicitNamespace(Impl::NamespacePrefix, Impl::NamespaceUrl),  // `prefix|foo`
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
    P: Parser<'i, Impl=Impl>,
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
            Ok(&Token::Delim('*')) if !in_attr_selector => {
                Ok(OptionalQName::Some(namespace, None))
            }
            Ok(&Token::Ident(ref local_name)) => {
                Ok(OptionalQName::Some(namespace, Some(local_name.clone())))
            }
            Ok(t) if in_attr_selector => {
                Err(location.new_custom_error(
                    SelectorParseErrorKind::InvalidQualNameInAttr(t.clone())
                ))
            }
            Ok(t) => {
                Err(location.new_custom_error(
                    SelectorParseErrorKind::ExplicitNamespaceUnexpectedToken(t.clone())
                ))
            }
            Err(e) => Err(e.into()),
        }
    };

    let start = input.state();
    // FIXME: remove clone() when lifetimes are non-lexical
    match input.next_including_whitespace().map(|t| t.clone()) {
        Ok(Token::Ident(value)) => {
            let after_ident = input.state();
            match input.next_including_whitespace() {
                Ok(&Token::Delim('|')) => {
                    let prefix = value.as_ref().into();
                    let result = parser.namespace_for_prefix(&prefix);
                    let url = result.ok_or(after_ident.source_location().new_custom_error(
                        SelectorParseErrorKind::ExpectedNamespace(value)))?;
                    explicit_namespace(input, QNamePrefix::ExplicitNamespace(prefix, url))
                },
                _ => {
                    input.reset(&after_ident);
                    if in_attr_selector {
                        Ok(OptionalQName::Some(QNamePrefix::ImplicitNoNamespace, Some(value)))
                    } else {
                        default_namespace(Some(value))
                    }
                }
            }
        },
        Ok(Token::Delim('*')) => {
            let after_star = input.state();
            // FIXME: remove clone() when lifetimes are non-lexical
            match input.next_including_whitespace().map(|t| t.clone()) {
                Ok(Token::Delim('|')) => {
                    explicit_namespace(input, QNamePrefix::ExplicitAnyNamespace)
                }
                result => {
                    input.reset(&after_star);
                    if in_attr_selector {
                        match result {
                            Ok(t) => Err(after_star.source_location().new_custom_error(
                                SelectorParseErrorKind::ExpectedBarInAttr(t)
                            )),
                            Err(e) => Err(e.into()),
                        }
                    } else {
                        default_namespace(None)
                    }
                },
            }
        },
        Ok(Token::Delim('|')) => {
            explicit_namespace(input, QNamePrefix::ExplicitNoNamespace)
        }
        Ok(t) => {
            input.reset(&start);
            Ok(OptionalQName::None(t))
        }
        Err(e) => {
            input.reset(&start);
            Err(e.into())
        }
    }
}

fn parse_attribute_selector<'i, 't, P, Impl>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
) -> Result<Component<Impl>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl=Impl>,
    Impl: SelectorImpl,
{
    let namespace;
    let local_name;

    input.skip_whitespace();

    match parse_qualified_name(parser, input, /* in_attr_selector = */ true)? {
        OptionalQName::None(t) => {
            return Err(input.new_custom_error(
                SelectorParseErrorKind::NoQualifiedNameInAttributeSelector(t)
            ))
        }
        OptionalQName::Some(_, None) => unreachable!(),
        OptionalQName::Some(ns, Some(ln)) => {
            local_name = ln;
            namespace = match ns {
                QNamePrefix::ImplicitNoNamespace |
                QNamePrefix::ExplicitNoNamespace => {
                    None
                }
                QNamePrefix::ExplicitNamespace(prefix, url) => {
                    Some(NamespaceConstraint::Specific((prefix, url)))
                }
                QNamePrefix::ExplicitAnyNamespace => {
                    Some(NamespaceConstraint::Any)
                }
                QNamePrefix::ImplicitAnyNamespace |
                QNamePrefix::ImplicitDefaultNamespace(_) => {
                    unreachable!()  // Not returned with in_attr_selector = true
                }
            }
        }
    }

    let location = input.current_source_location();
    let operator = match input.next() {
        // [foo]
        Err(_) => {
            let local_name_lower = to_ascii_lowercase(&local_name).as_ref().into();
            let local_name = local_name.as_ref().into();
            if let Some(namespace) = namespace {
                return Ok(Component::AttributeOther(Box::new(AttrSelectorWithNamespace {
                    namespace: namespace,
                    local_name: local_name,
                    local_name_lower: local_name_lower,
                    operation: ParsedAttrSelectorOperation::Exists,
                    never_matches: false,
                })))
            } else {
                return Ok(Component::AttributeInNoNamespaceExists {
                    local_name: local_name,
                    local_name_lower: local_name_lower,
                })
            }
        }

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
        Ok(t) => return Err(location.new_custom_error(
            SelectorParseErrorKind::UnexpectedTokenInAttributeSelector(t.clone())
        ))
    };

    let value = match input.expect_ident_or_string() {
        Ok(t) => t.clone(),
        Err(BasicParseError { kind: BasicParseErrorKind::UnexpectedToken(t), location }) => {
            return Err(location.new_custom_error(SelectorParseErrorKind::BadValueInAttr(t)))
        }
        Err(e) => return Err(e.into()),
    };
    let never_matches = match operator {
        AttrSelectorOperator::Equal |
        AttrSelectorOperator::DashMatch => false,

        AttrSelectorOperator::Includes => {
            value.is_empty() || value.contains(SELECTOR_WHITESPACE)
        }

        AttrSelectorOperator::Prefix |
        AttrSelectorOperator::Substring |
        AttrSelectorOperator::Suffix => value.is_empty()
    };

    let mut case_sensitivity = parse_attribute_flags(input)?;

    let value = value.as_ref().into();
    let local_name_lower;
    {
        let local_name_lower_cow = to_ascii_lowercase(&local_name);
        if let ParsedCaseSensitivity::CaseSensitive = case_sensitivity {
            if namespace.is_none() &&
                include!(concat!(env!("OUT_DIR"), "/ascii_case_insensitive_html_attributes.rs"))
                .contains(&*local_name_lower_cow)
            {
                case_sensitivity =
                    ParsedCaseSensitivity::AsciiCaseInsensitiveIfInHtmlElementInHtmlDocument
            }
        }
        local_name_lower = local_name_lower_cow.as_ref().into();
    }
    let local_name = local_name.as_ref().into();
    if let Some(namespace) = namespace {
        Ok(Component::AttributeOther(Box::new(AttrSelectorWithNamespace {
            namespace: namespace,
            local_name: local_name,
            local_name_lower: local_name_lower,
            never_matches: never_matches,
            operation: ParsedAttrSelectorOperation::WithValue {
                operator: operator,
                case_sensitivity: case_sensitivity,
                expected_value: value,
            }
        })))
    } else {
        Ok(Component::AttributeInNoNamespace {
            local_name: local_name,
            local_name_lower: local_name_lower,
            operator: operator,
            value: value,
            case_sensitivity: case_sensitivity,
            never_matches: never_matches,
        })
    }
}


fn parse_attribute_flags<'i, 't>(
    input: &mut CssParser<'i, 't>,
) -> Result<ParsedCaseSensitivity, BasicParseError<'i>> {
    let location = input.current_source_location();
    match input.next() {
        Err(_) => {
            // Selectors spec says language-defined, but HTML says sensitive.
            Ok(ParsedCaseSensitivity::CaseSensitive)
        }
        Ok(&Token::Ident(ref value)) if value.eq_ignore_ascii_case("i") => {
            Ok(ParsedCaseSensitivity::AsciiCaseInsensitive)
        }
        Ok(t) => Err(location.new_basic_unexpected_token_error(t.clone()))
    }
}


/// Level 3: Parse **one** simple_selector.  (Though we might insert a second
/// implied "<defaultns>|*" type selector.)
fn parse_negation<'i, 't, P, Impl>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
) -> Result<Component<Impl>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl=Impl>,
    Impl: SelectorImpl,
{
    // We use a sequence because a type selector may be represented as two Components.
    let mut sequence = SmallVec::<[Component<Impl>; 2]>::new();

    input.skip_whitespace();

    // Get exactly one simple selector. The parse logic in the caller will verify
    // that there are no trailing tokens after we're done.
    let is_type_sel = match parse_type_selector(parser, input, &mut sequence) {
        Ok(result) => result,
        Err(ParseError { kind: ParseErrorKind::Basic(BasicParseErrorKind::EndOfInput), .. }) => {
            return Err(input.new_custom_error(SelectorParseErrorKind::EmptyNegation))
        }
        Err(e) => return Err(e.into()),
    };
    if !is_type_sel {
        match parse_one_simple_selector(parser, input, /* inside_negation = */ true)? {
            Some(SimpleSelectorParseResult::SimpleSelector(s)) => {
                sequence.push(s);
            },
            None => {
                return Err(input.new_custom_error(SelectorParseErrorKind::EmptyNegation));
            },
            Some(SimpleSelectorParseResult::PseudoElement(_)) |
            Some(SimpleSelectorParseResult::SlottedPseudo(_)) => {
                return Err(input.new_custom_error(SelectorParseErrorKind::NonSimpleSelectorInNegation));
            }
        }
    }

    // Success.
    Ok(Component::Negation(sequence.into_vec().into_boxed_slice()))
}

/// simple_selector_sequence
/// : [ type_selector | universal ] [ HASH | class | attrib | pseudo | negation ]*
/// | [ HASH | class | attrib | pseudo | negation ]+
///
/// `Err(())` means invalid selector.
/// `Ok(None)` is an empty selector
///
/// The booleans represent whether a pseudo-element has been parsed, and whether
/// ::slotted() has been parsed, respectively.
fn parse_compound_selector<'i, 't, P, Impl>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
    builder: &mut SelectorBuilder<Impl>,
) -> Result<Option<(bool, bool)>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl=Impl>,
    Impl: SelectorImpl,
{
    input.skip_whitespace();

    let mut empty = true;
    let mut slot = false;
    if !parse_type_selector(parser, input, builder)? {
        if let Some(url) = parser.default_namespace() {
            // If there was no explicit type selector, but there is a
            // default namespace, there is an implicit "<defaultns>|*" type
            // selector.
            builder.push_simple_selector(Component::DefaultNamespace(url))
        }
    } else {
        empty = false;
    }

    let mut pseudo = false;
    loop {
        let parse_result =
            match parse_one_simple_selector(parser, input, /* inside_negation = */ false)? {
                None => break,
                Some(result) => result,
            };

        match parse_result {
            SimpleSelectorParseResult::SimpleSelector(s) => {
                builder.push_simple_selector(s);
                empty = false
            }
            SimpleSelectorParseResult::PseudoElement(p) => {
                // Try to parse state to its right. There are only 3 allowable
                // state selectors that can go on pseudo-elements.
                let mut state_selectors = SmallVec::<[Component<Impl>; 3]>::new();

                loop {
                    let location = input.current_source_location();
                    match input.next_including_whitespace() {
                        Ok(&Token::Colon) => {},
                        Ok(&Token::WhiteSpace(_)) | Err(_) => break,
                        Ok(t) =>
                            return Err(location.new_custom_error(
                                SelectorParseErrorKind::PseudoElementExpectedColon(t.clone())
                            )),
                    }

                    let location = input.current_source_location();
                    // TODO(emilio): Functional pseudo-classes too?
                    // We don't need it for now.
                    let name = match input.next_including_whitespace()? {
                        &Token::Ident(ref name) => name.clone(),
                        t => return Err(location.new_custom_error(
                            SelectorParseErrorKind::NoIdentForPseudo(t.clone())
                        )),
                    };

                    let pseudo_class =
                        P::parse_non_ts_pseudo_class(parser, location, name.clone())?;
                    if !p.supports_pseudo_class(&pseudo_class) {
                        return Err(input.new_custom_error(
                            SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name)
                        ));
                    }
                    state_selectors.push(Component::NonTSPseudoClass(pseudo_class));
                }

                if !builder.is_empty() {
                    builder.push_combinator(Combinator::PseudoElement);
                }

                builder.push_simple_selector(Component::PseudoElement(p));
                for state_selector in state_selectors.drain() {
                    builder.push_simple_selector(state_selector);
                }

                pseudo = true;
                empty = false;
                break
            }
            SimpleSelectorParseResult::SlottedPseudo(selector) => {
                empty = false;
                slot = true;
                if !builder.is_empty() {
                    builder.push_combinator(Combinator::SlotAssignment);
                }
                builder.push_simple_selector(Component::Slotted(selector));
                // FIXME(emilio): ::slotted() should support ::before and
                // ::after after it, so we shouldn't break, but we shouldn't
                // push more type selectors either.
                break;
            }
        }
    }
    if empty {
        // An empty selector is invalid.
        Ok(None)
    } else {
        Ok(Some((pseudo, slot)))
    }
}

fn parse_functional_pseudo_class<'i, 't, P, Impl>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
    name: CowRcStr<'i>,
    inside_negation: bool,
) -> Result<Component<Impl>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl=Impl>,
    Impl: SelectorImpl,
{
    match_ignore_ascii_case! { &name,
        "nth-child" => return Ok(parse_nth_pseudo_class(input, Component::NthChild)?),
        "nth-of-type" => return Ok(parse_nth_pseudo_class(input, Component::NthOfType)?),
        "nth-last-child" => return Ok(parse_nth_pseudo_class(input, Component::NthLastChild)?),
        "nth-last-of-type" => return Ok(parse_nth_pseudo_class(input, Component::NthLastOfType)?),
        "not" => {
            if inside_negation {
                return Err(input.new_custom_error(
                    SelectorParseErrorKind::UnexpectedIdent("not".into())
                ));
            }
            return parse_negation(parser, input)
        },
        _ => {}
    }
    P::parse_non_ts_functional_pseudo_class(parser, name, input)
        .map(Component::NonTSPseudoClass)
}


fn parse_nth_pseudo_class<'i, 't, Impl, F>(
    input: &mut CssParser<'i, 't>,
    selector: F,
) -> Result<Component<Impl>, BasicParseError<'i>>
where
    Impl: SelectorImpl,
    F: FnOnce(i32, i32) -> Component<Impl>,
{
    let (a, b) = parse_nth(input)?;
    Ok(selector(a, b))
}


/// Returns whether the name corresponds to a CSS2 pseudo-element that
/// can be specified with the single colon syntax (in addition to the
/// double-colon syntax, which can be used for all pseudo-elements).
pub fn is_css2_pseudo_element(name: &str) -> bool {
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
    inside_negation: bool,
) -> Result<Option<SimpleSelectorParseResult<Impl>>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl=Impl>,
    Impl: SelectorImpl,
{
    let start = input.state();
    // FIXME: remove clone() when lifetimes are non-lexical
    match input.next_including_whitespace().map(|t| t.clone()) {
        Ok(Token::IDHash(id)) => {
            let id = Component::ID(id.as_ref().into());
            Ok(Some(SimpleSelectorParseResult::SimpleSelector(id)))
        }
        Ok(Token::Delim('.')) => {
            let location = input.current_source_location();
            match *input.next_including_whitespace()? {
                Token::Ident(ref class) => {
                    let class = Component::Class(class.as_ref().into());
                    Ok(Some(SimpleSelectorParseResult::SimpleSelector(class)))
                }
                ref t => Err(location.new_custom_error(
                    SelectorParseErrorKind::ClassNeedsIdent(t.clone())
                )),
            }
        }
        Ok(Token::SquareBracketBlock) => {
            let attr = input.parse_nested_block(|input| parse_attribute_selector(parser, input))?;
            Ok(Some(SimpleSelectorParseResult::SimpleSelector(attr)))
        }
        Ok(Token::Colon) => {
            let location = input.current_source_location();
            let (is_single_colon, next_token) = match input.next_including_whitespace()?.clone() {
                Token::Colon => (false, input.next_including_whitespace()?.clone()),
                t => (true, t),
            };
            let (name, is_functional) = match next_token {
                Token::Ident(name) => (name, false),
                Token::Function(name) => (name, true),
                t => return Err(input.new_custom_error(
                    SelectorParseErrorKind::PseudoElementExpectedIdent(t)
                )),
            };
            let is_pseudo_element = !is_single_colon ||
                P::pseudo_element_allows_single_colon(&name);
            if is_pseudo_element {
                let parse_result = if is_functional {
                    if P::parse_slotted(parser) && name.eq_ignore_ascii_case("slotted") {
                        SimpleSelectorParseResult::SlottedPseudo(
                            input.parse_nested_block(|input| {
                                parse_inner_compound_selector(
                                    parser,
                                    input,
                                )
                            })?
                        )
                    } else {
                        SimpleSelectorParseResult::PseudoElement(
                            input.parse_nested_block(|input| {
                                P::parse_functional_pseudo_element(
                                    parser,
                                    name,
                                    input,
                                )
                            })?
                        )
                    }
                } else {
                    SimpleSelectorParseResult::PseudoElement(
                        P::parse_pseudo_element(parser, location, name)?
                    )
                };
                Ok(Some(parse_result))
            } else {
                let pseudo_class = if is_functional {
                    input.parse_nested_block(|input| {
                        parse_functional_pseudo_class(parser, input, name, inside_negation)
                    })?
                } else {
                    parse_simple_pseudo_class(parser, location, name)?
                };
                Ok(Some(SimpleSelectorParseResult::SimpleSelector(pseudo_class)))
            }
        }
        _ => {
            input.reset(&start);
            Ok(None)
        }
    }
}

fn parse_simple_pseudo_class<'i, P, Impl>(
    parser: &P,
    location: SourceLocation,
    name: CowRcStr<'i>,
) -> Result<Component<Impl>, ParseError<'i, P::Error>>
where
    P: Parser<'i, Impl=Impl>,
    Impl: SelectorImpl
{
    (match_ignore_ascii_case! { &name,
        "first-child" => Ok(Component::FirstChild),
        "last-child"  => Ok(Component::LastChild),
        "only-child"  => Ok(Component::OnlyChild),
        "root" => Ok(Component::Root),
        "empty" => Ok(Component::Empty),
        "scope" => Ok(Component::Scope),
        "first-of-type" => Ok(Component::FirstOfType),
        "last-of-type"  => Ok(Component::LastOfType),
        "only-of-type"  => Ok(Component::OnlyOfType),
        _ => Err(())
    }).or_else(|()| {
        P::parse_non_ts_pseudo_class(parser, location, name)
            .map(Component::NonTSPseudoClass)
    })
}

// NB: pub module in order to access the DummyParser
#[cfg(test)]
pub mod tests {
    use builder::HAS_PSEUDO_BIT;
    use cssparser::{Parser as CssParser, ToCss, serialize_identifier, ParserInput};
    use parser;
    use std::collections::HashMap;
    use std::fmt;
    use super::*;

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
    }

    impl parser::PseudoElement for PseudoElement {
        type Impl = DummySelectorImpl;

        fn supports_pseudo_class(&self, pc: &PseudoClass) -> bool {
            match *pc {
                PseudoClass::Hover => true,
                PseudoClass::Active |
                PseudoClass::Lang(..) => false,
            }
        }
    }

    impl ToCss for PseudoClass {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                PseudoClass::Hover => dest.write_str(":hover"),
                PseudoClass::Active => dest.write_str(":active"),
                PseudoClass::Lang(ref lang) => {
                    dest.write_str(":lang(")?;
                    serialize_identifier(lang, dest)?;
                    dest.write_char(')')
                }
            }
        }
    }

    impl ToCss for PseudoElement {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                PseudoElement::Before => dest.write_str("::before"),
                PseudoElement::After => dest.write_str("::after"),
            }
        }
    }

    impl SelectorMethods for PseudoClass {
        type Impl = DummySelectorImpl;

        fn visit<V>(&self, _visitor: &mut V) -> bool
        where
            V: SelectorVisitor<Impl = Self::Impl>,
        {
            true
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
        type ExtraMatchingData = ();
        type AttrValue = DummyAtom;
        type Identifier = DummyAtom;
        type ClassName = DummyAtom;
        type LocalName = DummyAtom;
        type NamespaceUrl = DummyAtom;
        type NamespacePrefix = DummyAtom;
        type BorrowedLocalName = DummyAtom;
        type BorrowedNamespaceUrl = DummyAtom;
        type NonTSPseudoClass = PseudoClass;
        type PseudoElement = PseudoElement;

        #[inline]
        fn is_active_or_hover(pseudo_class: &Self::NonTSPseudoClass) -> bool {
            matches!(*pseudo_class, PseudoClass::Active |
                                    PseudoClass::Hover)
        }
    }

    #[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
    pub struct DummyAtom(String);

    impl fmt::Display for DummyAtom {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            <String as fmt::Display>::fmt(&self.0, fmt)
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

    impl PrecomputedHash for DummyAtom {
        fn precomputed_hash(&self) -> u32 {
            return 0
        }
    }

    impl<'i> Parser<'i> for DummyParser {
        type Impl = DummySelectorImpl;
        type Error = SelectorParseErrorKind<'i>;

        fn parse_slotted(&self) -> bool {
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
            Err(location.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name)))
        }

        fn parse_non_ts_functional_pseudo_class<'t>(
            &self,
            name: CowRcStr<'i>,
            parser: &mut CssParser<'i, 't>,
        ) -> Result<PseudoClass, SelectorParseError<'i>> {
            match_ignore_ascii_case! { &name,
                "lang" => return Ok(PseudoClass::Lang(parser.expect_ident_or_string()?.as_ref().to_owned())),
                _ => {}
            }
            Err(parser.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name)))
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
            Err(location.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name)))
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
        expected: Option<&'a str>
    ) -> Result<SelectorList<DummySelectorImpl>, SelectorParseError<'i>> {
        let mut parser_input = ParserInput::new(input);
        let result = SelectorList::parse(parser, &mut CssParser::new(&mut parser_input));
        if let Ok(ref selectors) = result {
            assert_eq!(selectors.0.len(), 1);
            // We can't assume that the serialized parsed selector will equal
            // the input; for example, if there is no default namespace, '*|foo'
            // should serialize to 'foo'.
            assert_eq!(
                selectors.0[0].to_css_string(),
                match expected {
                    Some(x) => x,
                    None => input
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

    const MATHML: &'static str = "http://www.w3.org/1998/Math/MathML";
    const SVG: &'static str = "http://www.w3.org/2000/svg";

    #[test]
    fn test_parsing() {
        assert!(parse("").is_err());
        assert!(parse(":lang(4)").is_err());
        assert!(parse(":lang(en US)").is_err());
        assert_eq!(parse("EeÉ"), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::LocalName(LocalName {
                    name: DummyAtom::from("EeÉ"),
                    lower_name: DummyAtom::from("eeÉ") })
            ), specificity(0, 0, 1))
        ))));
        assert_eq!(parse("|e"), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::ExplicitNoNamespace,
                Component::LocalName(LocalName {
                    name: DummyAtom::from("e"),
                    lower_name: DummyAtom::from("e")
                })), specificity(0, 0, 1))
        ))));
        // When the default namespace is not set, *| should be elided.
        // https://github.com/servo/servo/pull/17537
        assert_eq!(parse_expected("*|e", Some("e")), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::LocalName(LocalName {
                    name: DummyAtom::from("e"),
                    lower_name: DummyAtom::from("e")
                })
            ), specificity(0, 0, 1))
        ))));
        // When the default namespace is set, *| should _not_ be elided (as foo
        // is no longer equivalent to *|foo--the former is only for foo in the
        // default namespace).
        // https://github.com/servo/servo/issues/16020
        assert_eq!(
            parse_ns(
                "*|e",
                &DummyParser::default_with_namespace(DummyAtom::from("https://mozilla.org"))
            ),
            Ok(SelectorList::from_vec(vec!(
                Selector::from_vec(vec!(
                    Component::ExplicitAnyNamespace,
                    Component::LocalName(LocalName {
                        name: DummyAtom::from("e"),
                        lower_name: DummyAtom::from("e")
                    })
                ), specificity(0, 0, 1)))))
        );
        assert_eq!(parse("*"), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::ExplicitUniversalType,
            ), specificity(0, 0, 0))
        ))));
        assert_eq!(parse("|*"), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::ExplicitNoNamespace,
                Component::ExplicitUniversalType,
            ), specificity(0, 0, 0))
        ))));
        assert_eq!(parse_expected("*|*", Some("*")), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::ExplicitUniversalType,
            ), specificity(0, 0, 0))
        ))));
        assert_eq!(
            parse_ns(
                "*|*",
                &DummyParser::default_with_namespace(DummyAtom::from("https://mozilla.org"))
            ),
            Ok(SelectorList::from_vec(vec!(
                Selector::from_vec(vec!(
                    Component::ExplicitAnyNamespace,
                    Component::ExplicitUniversalType,
                ), specificity(0, 0, 0)))))
        );
        assert_eq!(parse(".foo:lang(en-US)"), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                    Component::Class(DummyAtom::from("foo")),
                    Component::NonTSPseudoClass(PseudoClass::Lang("en-US".to_owned()))
            ), specificity(0, 2, 0))
        ))));
        assert_eq!(parse("#bar"), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::ID(DummyAtom::from("bar"))
            ), specificity(1, 0, 0))
        ))));
        assert_eq!(parse("e.foo#bar"), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::LocalName(LocalName {
                    name: DummyAtom::from("e"),
                    lower_name: DummyAtom::from("e")
                }),
                Component::Class(DummyAtom::from("foo")),
                Component::ID(DummyAtom::from("bar"))
            ), specificity(1, 1, 1))
        ))));
        assert_eq!(parse("e.foo #bar"), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
               Component::LocalName(LocalName {
                   name: DummyAtom::from("e"),
                   lower_name: DummyAtom::from("e")
               }),
               Component::Class(DummyAtom::from("foo")),
               Component::Combinator(Combinator::Descendant),
               Component::ID(DummyAtom::from("bar")),
            ), specificity(1, 1, 1))
        ))));
        // Default namespace does not apply to attribute selectors
        // https://github.com/mozilla/servo/pull/1652
        let mut parser = DummyParser::default();
        assert_eq!(parse_ns("[Foo]", &parser), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::AttributeInNoNamespaceExists {
                    local_name: DummyAtom::from("Foo"),
                    local_name_lower: DummyAtom::from("foo"),
                }
            ), specificity(0, 1, 0))
        ))));
        assert!(parse_ns("svg|circle", &parser).is_err());
        parser.ns_prefixes.insert(DummyAtom("svg".into()), DummyAtom(SVG.into()));
        assert_eq!(parse_ns("svg|circle", &parser), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::Namespace(DummyAtom("svg".into()), SVG.into()),
                Component::LocalName(LocalName {
                    name: DummyAtom::from("circle"),
                    lower_name: DummyAtom::from("circle"),
                })
            ), specificity(0, 0, 1))
        ))));
        assert_eq!(parse_ns("svg|*", &parser), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::Namespace(DummyAtom("svg".into()), SVG.into()),
                Component::ExplicitUniversalType,
            ), specificity(0, 0, 0))
        ))));
        // Default namespace does not apply to attribute selectors
        // https://github.com/mozilla/servo/pull/1652
        // but it does apply to implicit type selectors
        // https://github.com/servo/rust-selectors/pull/82
        parser.default_ns = Some(MATHML.into());
        assert_eq!(parse_ns("[Foo]", &parser), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::DefaultNamespace(MATHML.into()),
                Component::AttributeInNoNamespaceExists {
                    local_name: DummyAtom::from("Foo"),
                    local_name_lower: DummyAtom::from("foo"),
                },
            ), specificity(0, 1, 0))
        ))));
        // Default namespace does apply to type selectors
        assert_eq!(parse_ns("e", &parser), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::DefaultNamespace(MATHML.into()),
                Component::LocalName(LocalName {
                    name: DummyAtom::from("e"),
                    lower_name: DummyAtom::from("e") }),
            ), specificity(0, 0, 1))
        ))));
        assert_eq!(parse_ns("*", &parser), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::DefaultNamespace(MATHML.into()),
                Component::ExplicitUniversalType,
            ), specificity(0, 0, 0))
        ))));
        assert_eq!(parse_ns("*|*", &parser), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::ExplicitAnyNamespace,
                Component::ExplicitUniversalType,
            ), specificity(0, 0, 0))
        ))));
        // Default namespace applies to universal and type selectors inside :not and :matches,
        // but not otherwise.
        assert_eq!(parse_ns(":not(.cl)", &parser), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::DefaultNamespace(MATHML.into()),
                Component::Negation(vec![
                    Component::Class(DummyAtom::from("cl"))
                ].into_boxed_slice()),
            ), specificity(0, 1, 0))
        ))));
        assert_eq!(parse_ns(":not(*)", &parser), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::DefaultNamespace(MATHML.into()),
                Component::Negation(vec![
                    Component::DefaultNamespace(MATHML.into()),
                    Component::ExplicitUniversalType,
                ].into_boxed_slice()),
            ), specificity(0, 0, 0))
        ))));
        assert_eq!(parse_ns(":not(e)", &parser), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::DefaultNamespace(MATHML.into()),
                Component::Negation(vec![
                    Component::DefaultNamespace(MATHML.into()),
                    Component::LocalName(LocalName {
                        name: DummyAtom::from("e"),
                        lower_name: DummyAtom::from("e")
                    }),
                ].into_boxed_slice())
            ), specificity(0, 0, 1))
        ))));
        assert_eq!(parse("[attr|=\"foo\"]"), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::AttributeInNoNamespace {
                    local_name: DummyAtom::from("attr"),
                    local_name_lower: DummyAtom::from("attr"),
                    operator: AttrSelectorOperator::DashMatch,
                    value: DummyAtom::from("foo"),
                    never_matches: false,
                    case_sensitivity: ParsedCaseSensitivity::CaseSensitive,
                }
            ), specificity(0, 1, 0))
        ))));
        // https://github.com/mozilla/servo/issues/1723
        assert_eq!(parse("::before"), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::PseudoElement(PseudoElement::Before),
            ), specificity(0, 0, 1) | HAS_PSEUDO_BIT)
        ))));
        assert_eq!(parse("::before:hover"), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::PseudoElement(PseudoElement::Before),
                Component::NonTSPseudoClass(PseudoClass::Hover),
            ), specificity(0, 1, 1) | HAS_PSEUDO_BIT)
        ))));
        assert_eq!(parse("::before:hover:hover"), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::PseudoElement(PseudoElement::Before),
                Component::NonTSPseudoClass(PseudoClass::Hover),
                Component::NonTSPseudoClass(PseudoClass::Hover),
            ), specificity(0, 2, 1) | HAS_PSEUDO_BIT)
        ))));
        assert!(parse("::before:hover:active").is_err());
        assert!(parse("::before:hover .foo").is_err());
        assert!(parse("::before .foo").is_err());
        assert!(parse("::before ~ bar").is_err());
        assert!(parse("::before:active").is_err());

        // https://github.com/servo/servo/issues/15335
        assert!(parse(":: before").is_err());
        assert_eq!(parse("div ::after"), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                 Component::LocalName(LocalName {
                    name: DummyAtom::from("div"),
                    lower_name: DummyAtom::from("div") }),
                Component::Combinator(Combinator::Descendant),
                Component::Combinator(Combinator::PseudoElement),
                Component::PseudoElement(PseudoElement::After),
            ), specificity(0, 0, 2) | HAS_PSEUDO_BIT)
        ))));
        assert_eq!(parse("#d1 > .ok"), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::ID(DummyAtom::from("d1")),
                Component::Combinator(Combinator::Child),
                Component::Class(DummyAtom::from("ok")),
            ), (1 << 20) + (1 << 10) + (0 << 0))
        ))));
        parser.default_ns = None;
        assert!(parse(":not(#provel.old)").is_err());
        assert!(parse(":not(#provel > old)").is_err());
        assert!(parse("table[rules]:not([rules=\"none\"]):not([rules=\"\"])").is_ok());
        assert_eq!(parse(":not(#provel)"), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(Component::Negation(vec!(
                    Component::ID(DummyAtom::from("provel")),
                ).into_boxed_slice()
            )), specificity(1, 0, 0))
        ))));
        assert_eq!(parse_ns(":not(svg|circle)", &parser), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(Component::Negation(
                vec![
                    Component::Namespace(DummyAtom("svg".into()), SVG.into()),
                    Component::LocalName(LocalName {
                        name: DummyAtom::from("circle"),
                        lower_name: DummyAtom::from("circle")
                    }),
                ].into_boxed_slice()
            )), specificity(0, 0, 1))
        ))));
        // https://github.com/servo/servo/issues/16017
        assert_eq!(parse_ns(":not(*)", &parser), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(Component::Negation(
                vec![
                    Component::ExplicitUniversalType,
                ].into_boxed_slice()
            )), specificity(0, 0, 0))
        ))));
        assert_eq!(parse_ns(":not(|*)", &parser), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(Component::Negation(
                vec![
                    Component::ExplicitNoNamespace,
                    Component::ExplicitUniversalType,
                ].into_boxed_slice()
            )), specificity(0, 0, 0))
        ))));
        // *| should be elided if there is no default namespace.
        // https://github.com/servo/servo/pull/17537
        assert_eq!(parse_ns_expected(":not(*|*)", &parser, Some(":not(*)")), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(Component::Negation(
                vec![
                    Component::ExplicitUniversalType,
                ].into_boxed_slice()
            )), specificity(0, 0, 0))
        ))));
        assert_eq!(parse_ns(":not(svg|*)", &parser), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(Component::Negation(
                vec![
                    Component::Namespace(DummyAtom("svg".into()), SVG.into()),
                    Component::ExplicitUniversalType,
                ].into_boxed_slice()
            )), specificity(0, 0, 0))
        ))));

        assert!(parse("::slotted()").is_err());
        assert!(parse("::slotted(div)").is_ok());
        assert!(parse("::slotted(div).foo").is_err());
        assert!(parse("::slotted(div + bar)").is_err());
        assert!(parse("::slotted(div) + foo").is_err());
        assert!(parse("div ::slotted(div)").is_ok());
        assert!(parse("div + slot::slotted(div)").is_ok());
        assert!(parse("div + slot::slotted(div.foo)").is_ok());
        assert!(parse("slot::slotted(div,foo)::first-line").is_err());
        // TODO
        assert!(parse("::slotted(div)::before").is_err());
        assert!(parse("slot::slotted(div,foo)").is_err());
    }

    #[test]
    fn test_pseudo_iter() {
        let selector = &parse("q::before").unwrap().0[0];
        assert!(!selector.is_universal());
        let mut iter = selector.iter();
        assert_eq!(iter.next(), Some(&Component::PseudoElement(PseudoElement::Before)));
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
            &DummyParser::default_with_namespace(DummyAtom::from("https://mozilla.org"))
        ).unwrap().0[0];
        assert!(selector.is_universal());
    }

    #[test]
    fn test_empty_pseudo_iter() {
        let selector = &parse("::before").unwrap().0[0];
        assert!(selector.is_universal());
        let mut iter = selector.iter();
        assert_eq!(iter.next(), Some(&Component::PseudoElement(PseudoElement::Before)));
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
        let mut test_visitor = TestVisitor { seen: vec![], };
        parse(":not(:hover) ~ label").unwrap().0[0].visit(&mut test_visitor);
        assert!(test_visitor.seen.contains(&":hover".into()));

        let mut test_visitor = TestVisitor { seen: vec![], };
        parse("::before:hover").unwrap().0[0].visit(&mut test_visitor);
        assert!(test_visitor.seen.contains(&":hover".into()));
    }
}
