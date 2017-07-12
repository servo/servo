/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use attr::{AttrSelectorWithNamespace, ParsedAttrSelectorOperation, AttrSelectorOperator};
use attr::{ParsedCaseSensitivity, SELECTOR_WHITESPACE, NamespaceConstraint};
use bloom::BLOOM_HASH_MASK;
use builder::{SelectorBuilder, SpecificityAndFlags};
use cssparser::{ParseError, BasicParseError, CompactCowStr};
use cssparser::{Token, Parser as CssParser, parse_nth, ToCss, serialize_identifier, CssStringWriter};
use precomputed_hash::PrecomputedHash;
use servo_arc::ThinArc;
use sink::Push;
use smallvec::SmallVec;
use std::ascii::AsciiExt;
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
        _pseudo_class: &<Self::Impl as SelectorImpl>::NonTSPseudoClass)
        -> bool
    {
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

#[derive(Clone, Debug, PartialEq)]
pub enum SelectorParseError<'i, T> {
    PseudoElementInComplexSelector,
    NoQualifiedNameInAttributeSelector,
    TooManyCompoundSelectorComponentsInNegation,
    NegationSelectorComponentNotNamespace,
    NegationSelectorComponentNotLocalName,
    EmptySelector,
    NonSimpleSelectorInNegation,
    UnexpectedTokenInAttributeSelector,
    PseudoElementExpectedColon,
    PseudoElementExpectedIdent,
    UnsupportedPseudoClass,
    UnexpectedIdent(CompactCowStr<'i>),
    ExpectedNamespace(CompactCowStr<'i>),
    Custom(T),
}

impl<'a, T> Into<ParseError<'a, SelectorParseError<'a, T>>> for SelectorParseError<'a, T> {
    fn into(self) -> ParseError<'a, SelectorParseError<'a, T>> {
        ParseError::Custom(self)
    }
}

macro_rules! with_all_bounds {
    (
        [ $( $InSelector: tt )* ]
        [ $( $CommonBounds: tt )* ]
        [ $( $FromStr: tt )* ]
    ) => {
        fn from_cow_str<T>(cow: Cow<str>) -> T where T: $($FromStr)* {
            match cow {
                Cow::Borrowed(s) => T::from(s),
                Cow::Owned(s) => T::from(s),
            }
        }

        /// This trait allows to define the parser implementation in regards
        /// of pseudo-classes/elements
        ///
        /// NB: We need Clone so that we can derive(Clone) on struct with that
        /// are parameterized on SelectorImpl. See
        /// https://github.com/rust-lang/rust/issues/26925
        pub trait SelectorImpl: Clone + Sized + 'static {
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
    [From<String> + for<'a> From<&'a str>]
}

pub trait Parser<'i> {
    type Impl: SelectorImpl;
    type Error: 'i;

    /// Whether the name is a pseudo-element that can be specified with
    /// the single colon syntax in addition to the double-colon syntax.
    fn is_pseudo_element_allows_single_colon(name: &CompactCowStr<'i>) -> bool {
        is_css2_pseudo_element(name)
    }

    /// This function can return an "Err" pseudo-element in order to support CSS2.1
    /// pseudo-elements.
    fn parse_non_ts_pseudo_class(&self, name: CompactCowStr<'i>)
                                 -> Result<<Self::Impl as SelectorImpl>::NonTSPseudoClass,
                                           ParseError<'i, SelectorParseError<'i, Self::Error>>> {
        Err(ParseError::Custom(SelectorParseError::UnexpectedIdent(name)))
    }

    fn parse_non_ts_functional_pseudo_class<'t>
        (&self, name: CompactCowStr<'i>, _arguments: &mut CssParser<'i, 't>)
         -> Result<<Self::Impl as SelectorImpl>::NonTSPseudoClass,
                   ParseError<'i, SelectorParseError<'i, Self::Error>>>
    {
        Err(ParseError::Custom(SelectorParseError::UnexpectedIdent(name)))
    }

    fn parse_pseudo_element(&self, name: CompactCowStr<'i>)
                            -> Result<<Self::Impl as SelectorImpl>::PseudoElement,
                                      ParseError<'i, SelectorParseError<'i, Self::Error>>> {
        Err(ParseError::Custom(SelectorParseError::UnexpectedIdent(name)))
    }

    fn parse_functional_pseudo_element<'t>
        (&self, name: CompactCowStr<'i>, _arguments: &mut CssParser<'i, 't>)
         -> Result<<Self::Impl as SelectorImpl>::PseudoElement,
                   ParseError<'i, SelectorParseError<'i, Self::Error>>> {
        Err(ParseError::Custom(SelectorParseError::UnexpectedIdent(name)))
    }

    fn default_namespace(&self) -> Option<<Self::Impl as SelectorImpl>::NamespaceUrl> {
        None
    }

    fn namespace_for_prefix(&self, _prefix: &<Self::Impl as SelectorImpl>::NamespacePrefix)
                            -> Option<<Self::Impl as SelectorImpl>::NamespaceUrl> {
        None
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SelectorAndHashes<Impl: SelectorImpl> {
    pub selector: Selector<Impl>,
    pub hashes: AncestorHashes,
}

impl<Impl: SelectorImpl> SelectorAndHashes<Impl> {
    pub fn new(selector: Selector<Impl>) -> Self {
        let hashes = AncestorHashes::new(&selector);
        Self::new_with_hashes(selector, hashes)
    }

    pub fn new_with_hashes(selector: Selector<Impl>, hashes: AncestorHashes) -> Self {
        SelectorAndHashes {
            selector: selector,
            hashes: hashes,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SelectorList<Impl: SelectorImpl>(pub Vec<SelectorAndHashes<Impl>>);

impl<Impl: SelectorImpl> SelectorList<Impl> {
    /// Parse a comma-separated list of Selectors.
    /// https://drafts.csswg.org/selectors/#grouping
    ///
    /// Return the Selectors or Err if there is an invalid selector.
    pub fn parse<'i, 't, P, E>(parser: &P, input: &mut CssParser<'i, 't>)
                               -> Result<Self, ParseError<'i, SelectorParseError<'i, E>>>
    where P: Parser<'i, Impl=Impl, Error=E> {
        input.parse_comma_separated(|input| parse_selector(parser, input).map(SelectorAndHashes::new))
             .map(SelectorList)
    }

    /// Creates a SelectorList from a Vec of selectors. Used in tests.
    pub fn from_vec(v: Vec<Selector<Impl>>) -> Self {
        SelectorList(v.into_iter().map(SelectorAndHashes::new).collect())
    }
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
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct AncestorHashes {
    pub packed_hashes: [u32; 3],
}

impl AncestorHashes {
    pub fn new<Impl: SelectorImpl>(s: &Selector<Impl>) -> Self {
        Self::from_iter(s.iter())
    }

    pub fn from_iter<Impl: SelectorImpl>(iter: SelectorIter<Impl>) -> Self {
        // Compute ancestor hashes for the bloom filter.
        let mut hashes = [0u32; 4];
        let mut hash_iter = AncestorIter::new(iter)
                             .map(|x| x.ancestor_hash())
                             .filter(|x| x.is_some())
                             .map(|x| x.unwrap());
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
        where V: SelectorVisitor<Impl = Self::Impl>;
}

impl<Impl: SelectorImpl> SelectorMethods for Selector<Impl> {
    type Impl = Impl;

    fn visit<V>(&self, visitor: &mut V) -> bool
        where V: SelectorVisitor<Impl = Impl>,
    {
        let mut current = self.iter();
        let mut combinator = None;
        loop {
            if !visitor.visit_complex_selector(current.clone(), combinator) {
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
        where V: SelectorVisitor<Impl = Impl>,
    {
        use self::Component::*;
        if !visitor.visit_simple_selector(self) {
            return false;
        }

        match *self {
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
    pub fn specificity(&self) -> u32 {
        self.0.header.header.specificity()
    }

    pub fn has_pseudo_element(&self) -> bool {
        self.0.header.header.has_pseudo_element()
    }

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
    pub fn iter(&self) -> SelectorIter<Impl> {
        SelectorIter {
            iter: self.iter_raw_match_order(),
            next_combinator: None,
        }
    }

    /// Returns an iterator over this selector in matching order (right-to-left),
    /// skipping the rightmost |offset| Components.
    pub fn iter_from(&self, offset: usize) -> SelectorIter<Impl> {
        let iter = self.0.slice[offset..].iter();
        SelectorIter {
            iter: iter,
            next_combinator: None,
        }
    }

    /// Returns the combinator at index `index` (one-indexed from the right),
    /// or panics if the component is not a combinator.
    ///
    /// FIXME(bholley): Use more intuitive indexing.
    pub fn combinator_at(&self, index: usize) -> Combinator {
        match self.0.slice[index - 1] {
            Component::Combinator(c) => c,
            ref other => {
                panic!("Not a combinator: {:?}, {:?}, index: {}",
                       other, self, index)
            }
        }
    }

    /// Returns an iterator over the entire sequence of simple selectors and
    /// combinators, in matching order (from right to left).
    pub fn iter_raw_match_order(&self) -> slice::Iter<Component<Impl>> {
        self.0.slice.iter()
    }

    /// Returns an iterator over the sequence of simple selectors and
    /// combinators, in parse order (from left to right), _starting_
    /// 'offset_from_right' entries from the past-the-end sentinel on
    /// the right. So "0" panics,. "1" iterates nothing, and "len"
    /// iterates the entire sequence.
    ///
    /// FIXME(bholley): This API is rather unintuive, and should really
    /// be changed to accept an offset from the left. Same for combinator_at.
    pub fn iter_raw_parse_order_from(&self, offset_from_right: usize) -> Rev<slice::Iter<Component<Impl>>> {
        self.0.slice[..offset_from_right].iter().rev()
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
    pub fn len(&self) -> usize {
        self.0.slice.len()
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
    pub fn next_sequence(&mut self) -> Option<Combinator> {
        self.next_combinator.take()
    }

    /// Returns remaining count of the simple selectors and combinators in the Selector.
    pub fn selector_length(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, Impl: SelectorImpl> Iterator for SelectorIter<'a, Impl> {
    type Item = &'a Component<Impl>;
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
pub struct AncestorIter<'a, Impl: 'a + SelectorImpl>(SelectorIter<'a, Impl>);
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

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
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
}

impl Combinator {
    /// Returns true if this combinator is a child or descendant combinator.
    pub fn is_ancestor(&self) -> bool {
        matches!(*self, Combinator::Child |
                        Combinator::Descendant |
                        Combinator::PseudoElement)
    }

    /// Returns true if this combinator is a pseudo-element combinator.
    pub fn is_pseudo_element(&self) -> bool {
        matches!(*self, Combinator::PseudoElement)
    }

    /// Returns true if this combinator is a next- or later-sibling combinator.
    pub fn is_sibling(&self) -> bool {
        matches!(*self, Combinator::NextSibling | Combinator::LaterSibling)
    }
}

/// A CSS simple selector or combinator. We store both in the same enum for
/// optimal packing and cache performance, see [1].
///
/// [1] https://bugzilla.mozilla.org/show_bug.cgi?id=1357973
#[derive(Eq, PartialEq, Clone)]
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

    // Pseudo-classes
    //
    // CSS3 Negation only takes a simple simple selector, but we still need to
    // treat it as a compound selector because it might be a type selector which
    // we represent as a namespace and a localname.
    //
    // Note: if/when we upgrade this to CSS4, which supports combinators, we
    // need to think about how this should interact with visit_complex_selector,
    // and what the consumers of those APIs should do about the presence of
    // combinators in negation.
    Negation(Box<[Component<Impl>]>),
    FirstChild, LastChild, OnlyChild,
    Root,
    Empty,
    NthChild(i32, i32),
    NthLastChild(i32, i32),
    NthOfType(i32, i32),
    NthLastOfType(i32, i32),
    FirstOfType,
    LastOfType,
    OnlyOfType,
    NonTSPseudoClass(Impl::NonTSPseudoClass),
    PseudoElement(Impl::PseudoElement),
}

impl<Impl: SelectorImpl> Component<Impl> {
    /// Compute the ancestor hash to check against the bloom filter.
    pub fn ancestor_hash(&self) -> Option<u32> {
        match *self {
            Component::LocalName(LocalName { ref name, ref lower_name }) => {
                // Only insert the local-name into the filter if it's all lowercase.
                // Otherwise we would need to test both hashes, and our data structures
                // aren't really set up for that.
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
            Component::ID(ref id) => {
                Some(id.precomputed_hash())
            },
            Component::Class(ref class) => {
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

#[derive(Eq, PartialEq, Clone)]
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
        first.selector.to_css(dest)?;
        for selector_and_hashes in iter {
            dest.write_str(", ")?;
            selector_and_hashes.selector.to_css(dest)?;
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
        let mut combinators = self.iter_raw_match_order().rev().filter(|x| x.is_combinator());
        let compound_selectors = self.iter_raw_match_order().as_slice().split(|x| x.is_combinator()).rev();

        let mut combinators_exhausted = false;
        for compound in compound_selectors {
            debug_assert!(!combinators_exhausted);
            for item in compound.iter() {
                item.to_css(dest)?;
            }
            match combinators.next() {
                Some(c) => c.to_css(dest)?,
                None => combinators_exhausted = true,
            };
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
        }
    }
}

impl<Impl: SelectorImpl> ToCss for Component<Impl> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        use self::Component::*;
        match *self {
            Combinator(ref c) => {
                c.to_css(dest)
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
            FirstOfType => dest.write_str(":first-of-type"),
            LastOfType => dest.write_str(":last-of-type"),
            OnlyOfType => dest.write_str(":only-of-type"),
            NthChild(a, b) => write!(dest, ":nth-child({}n{:+})", a, b),
            NthLastChild(a, b) => write!(dest, ":nth-last-child({}n{:+})", a, b),
            NthOfType(a, b) => write!(dest, ":nth-of-type({}n{:+})", a, b),
            NthLastOfType(a, b) => write!(dest, ":nth-last-of-type({}n{:+})", a, b),
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
fn parse_selector<'i, 't, P, E, Impl>(
        parser: &P,
        input: &mut CssParser<'i, 't>)
        -> Result<Selector<Impl>, ParseError<'i, SelectorParseError<'i, E>>>
    where P: Parser<'i, Impl=Impl, Error=E>, Impl: SelectorImpl
{
    let mut builder = SelectorBuilder::default();

    let mut parsed_pseudo_element;
    'outer_loop: loop {
        // Parse a sequence of simple selectors.
        parsed_pseudo_element = parse_compound_selector(parser, input, &mut builder)?;
        if parsed_pseudo_element {
            break;
        }

        // Parse a combinator.
        let combinator;
        let mut any_whitespace = false;
        loop {
            let position = input.position();
            match input.next_including_whitespace() {
                Err(_e) => break 'outer_loop,
                Ok(Token::WhiteSpace(_)) => any_whitespace = true,
                Ok(Token::Delim('>')) => {
                    combinator = Combinator::Child;
                    break
                }
                Ok(Token::Delim('+')) => {
                    combinator = Combinator::NextSibling;
                    break
                }
                Ok(Token::Delim('~')) => {
                    combinator = Combinator::LaterSibling;
                    break
                }
                Ok(_) => {
                    input.reset(position);
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

    Ok(Selector(builder.build(parsed_pseudo_element)))
}

impl<Impl: SelectorImpl> Selector<Impl> {
    /// Parse a selector, without any pseudo-element.
    pub fn parse<'i, 't, P, E>(parser: &P, input: &mut CssParser<'i, 't>)
                               -> Result<Self, ParseError<'i, SelectorParseError<'i, E>>>
        where P: Parser<'i, Impl=Impl, Error=E>
    {
        let selector = parse_selector(parser, input)?;
        if selector.has_pseudo_element() {
            return Err(ParseError::Custom(SelectorParseError::PseudoElementInComplexSelector))
        }
        Ok(selector)
    }
}

/// * `Err(())`: Invalid selector, abort
/// * `Ok(false)`: Not a type selector, could be something else. `input` was not consumed.
/// * `Ok(true)`: Length 0 (`*|*`), 1 (`*|E` or `ns|*`) or 2 (`|E` or `ns|E`)
fn parse_type_selector<'i, 't, P, E, Impl, S>(parser: &P, input: &mut CssParser<'i, 't>, sink: &mut S)
                                              -> Result<bool, ParseError<'i, SelectorParseError<'i, E>>>
    where P: Parser<'i, Impl=Impl, Error=E>,
          Impl: SelectorImpl,
          S: Push<Component<Impl>>,
{
    match parse_qualified_name(parser, input, /* in_attr_selector = */ false)? {
        None => Ok(false),
        Some((namespace, local_name)) => {
            match namespace {
                QNamePrefix::ImplicitAnyNamespace => {}
                QNamePrefix::ImplicitDefaultNamespace(url) => {
                    sink.push(Component::DefaultNamespace(url))
                }
                QNamePrefix::ExplicitNamespace(prefix, url) => {
                    sink.push(Component::Namespace(prefix, url))
                }
                QNamePrefix::ExplicitNoNamespace => {
                    sink.push(Component::ExplicitNoNamespace)
                }
                QNamePrefix::ExplicitAnyNamespace => {
                    sink.push(Component::ExplicitAnyNamespace)
                }
                QNamePrefix::ImplicitNoNamespace => {
                    unreachable!()  // Not returned with in_attr_selector = false
                }
            }
            match local_name {
                Some(name) => {
                    sink.push(Component::LocalName(LocalName {
                        lower_name: from_cow_str(to_ascii_lowercase(&name)),
                        name: from_cow_str(name.into()),
                    }))
                }
                None => {
                    sink.push(Component::ExplicitUniversalType)
                }
            }
            Ok(true)
        }
    }
}

#[derive(Debug)]
enum SimpleSelectorParseResult<Impl: SelectorImpl> {
    SimpleSelector(Component<Impl>),
    PseudoElement(Impl::PseudoElement),
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

/// * `Err(())`: Invalid selector, abort
/// * `Ok(None)`: Not a simple selector, could be something else. `input` was not consumed.
/// * `Ok(Some((namespace, local_name)))`: `None` for the local name means a `*` universal selector
fn parse_qualified_name<'i, 't, P, E, Impl>
                       (parser: &P, input: &mut CssParser<'i, 't>,
                        in_attr_selector: bool)
                        -> Result<Option<(QNamePrefix<Impl>, Option<CompactCowStr<'i>>)>,
                                  ParseError<'i, SelectorParseError<'i, E>>>
    where P: Parser<'i, Impl=Impl, Error=E>, Impl: SelectorImpl
{
    let default_namespace = |local_name| {
        let namespace = match parser.default_namespace() {
            Some(url) => QNamePrefix::ImplicitDefaultNamespace(url),
            None => QNamePrefix::ImplicitAnyNamespace,
        };
        Ok(Some((namespace, local_name)))
    };

    let explicit_namespace = |input: &mut CssParser<'i, 't>, namespace| {
        match input.next_including_whitespace() {
            Ok(Token::Delim('*')) if !in_attr_selector => {
                Ok(Some((namespace, None)))
            },
            Ok(Token::Ident(local_name)) => {
                Ok(Some((namespace, Some(local_name))))
            },
            Ok(t) => Err(ParseError::Basic(BasicParseError::UnexpectedToken(t))),
            Err(e) => Err(ParseError::Basic(e)),
        }
    };

    let position = input.position();
    match input.next_including_whitespace() {
        Ok(Token::Ident(value)) => {
            let position = input.position();
            match input.next_including_whitespace() {
                Ok(Token::Delim('|')) => {
                    let prefix = from_cow_str(value.clone().into());
                    let result = parser.namespace_for_prefix(&prefix);
                    let url = result.ok_or(ParseError::Custom(
                        SelectorParseError::ExpectedNamespace(value.into())))?;
                    explicit_namespace(input, QNamePrefix::ExplicitNamespace(prefix, url))
                },
                _ => {
                    input.reset(position);
                    if in_attr_selector {
                        Ok(Some((QNamePrefix::ImplicitNoNamespace, Some(value))))
                    } else {
                        default_namespace(Some(value))
                    }
                }
            }
        },
        Ok(Token::Delim('*')) => {
            let position = input.position();
            match input.next_including_whitespace() {
                Ok(Token::Delim('|')) => {
                    explicit_namespace(input, QNamePrefix::ExplicitAnyNamespace)
                }
                result => {
                    input.reset(position);
                    if in_attr_selector {
                        match result {
                            Ok(t) => Err(ParseError::Basic(BasicParseError::UnexpectedToken(t))),
                             Err(e) => Err(ParseError::Basic(e)),
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
        _ => {
            input.reset(position);
            Ok(None)
        }
    }
}


fn parse_attribute_selector<'i, 't, P, E, Impl>(parser: &P, input: &mut CssParser<'i, 't>)
                                                -> Result<Component<Impl>,
                                                          ParseError<'i, SelectorParseError<'i, E>>>
    where P: Parser<'i, Impl=Impl, Error=E>, Impl: SelectorImpl
{
    let namespace;
    let local_name;
    match parse_qualified_name(parser, input, /* in_attr_selector = */ true)? {
        None => return Err(ParseError::Custom(SelectorParseError::NoQualifiedNameInAttributeSelector)),
        Some((_, None)) => unreachable!(),
        Some((ns, Some(ln))) => {
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

    let operator;
    let value;
    let never_matches;
    match input.next() {
        // [foo]
        Err(_) => {
            let local_name_lower = from_cow_str(to_ascii_lowercase(&local_name));
            let local_name = from_cow_str(local_name.into());
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
        Ok(Token::Delim('=')) => {
            value = input.expect_ident_or_string()?;
            never_matches = false;
            operator = AttrSelectorOperator::Equal;
        }
        // [foo~=bar]
        Ok(Token::IncludeMatch) => {
            value = input.expect_ident_or_string()?;
            never_matches = value.is_empty() || value.contains(SELECTOR_WHITESPACE);
            operator = AttrSelectorOperator::Includes;
        }
        // [foo|=bar]
        Ok(Token::DashMatch) => {
            value = input.expect_ident_or_string()?;
            never_matches = false;
            operator = AttrSelectorOperator::DashMatch;
        }
        // [foo^=bar]
        Ok(Token::PrefixMatch) => {
            value = input.expect_ident_or_string()?;
            never_matches = value.is_empty();
            operator = AttrSelectorOperator::Prefix;
        }
        // [foo*=bar]
        Ok(Token::SubstringMatch) => {
            value = input.expect_ident_or_string()?;
            never_matches = value.is_empty();
            operator = AttrSelectorOperator::Substring;
        }
        // [foo$=bar]
        Ok(Token::SuffixMatch) => {
            value = input.expect_ident_or_string()?;
            never_matches = value.is_empty();
            operator = AttrSelectorOperator::Suffix;
        }
        _ => return Err(SelectorParseError::UnexpectedTokenInAttributeSelector.into())
    }

    let mut case_sensitivity = parse_attribute_flags(input)?;

    let value = from_cow_str(value.into());
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
        local_name_lower = from_cow_str(local_name_lower_cow.into());
    }
    let local_name = from_cow_str(local_name.into());
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


fn parse_attribute_flags<'i, 't, E>(input: &mut CssParser<'i, 't>)
                                    -> Result<ParsedCaseSensitivity,
                                              ParseError<'i, SelectorParseError<'i, E>>> {
    match input.next() {
        Err(_) => {
            // Selectors spec says language-defined, but HTML says sensitive.
            Ok(ParsedCaseSensitivity::CaseSensitive)
        }
        Ok(Token::Ident(ref value)) if value.eq_ignore_ascii_case("i") => {
            Ok(ParsedCaseSensitivity::AsciiCaseInsensitive)
        }
        Ok(t) => Err(ParseError::Basic(BasicParseError::UnexpectedToken(t)))
    }
}


/// Level 3: Parse **one** simple_selector.  (Though we might insert a second
/// implied "<defaultns>|*" type selector.)
fn parse_negation<'i, 't, P, E, Impl>(parser: &P,
                                      input: &mut CssParser<'i, 't>)
                                      -> Result<Component<Impl>,
                                                ParseError<'i, SelectorParseError<'i, E>>>
    where P: Parser<'i, Impl=Impl, Error=E>, Impl: SelectorImpl
{
    // We use a sequence because a type selector may be represented as two Components.
    let mut sequence = SmallVec::<[Component<Impl>; 2]>::new();

    // Consume any leading whitespace.
    loop {
        let position = input.position();
        if !matches!(input.next_including_whitespace(), Ok(Token::WhiteSpace(_))) {
            input.reset(position);
            break
        }
    }

    // Get exactly one simple selector. The parse logic in the caller will verify
    // that there are no trailing tokens after we're done.
    if !parse_type_selector(parser, input, &mut sequence)? {
        match parse_one_simple_selector(parser, input, /* inside_negation = */ true)? {
            Some(SimpleSelectorParseResult::SimpleSelector(s)) => {
                sequence.push(s);
            },
            None => {
                return Err(ParseError::Custom(SelectorParseError::EmptySelector));
            },
            Some(SimpleSelectorParseResult::PseudoElement(_)) => {
                return Err(ParseError::Custom(SelectorParseError::NonSimpleSelectorInNegation));
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
///
/// The boolean represent whether a pseudo-element has been parsed.
fn parse_compound_selector<'i, 't, P, E, Impl>(
    parser: &P,
    input: &mut CssParser<'i, 't>,
    mut builder: &mut SelectorBuilder<Impl>)
    -> Result<bool, ParseError<'i, SelectorParseError<'i, E>>>
    where P: Parser<'i, Impl=Impl, Error=E>, Impl: SelectorImpl
{
    // Consume any leading whitespace.
    loop {
        let position = input.position();
        if !matches!(input.next_including_whitespace(), Ok(Token::WhiteSpace(_))) {
            input.reset(position);
            break
        }
    }
    let mut empty = true;
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
        match parse_one_simple_selector(parser, input, /* inside_negation = */ false)? {
            None => break,
            Some(SimpleSelectorParseResult::SimpleSelector(s)) => {
                builder.push_simple_selector(s);
                empty = false
            }
            Some(SimpleSelectorParseResult::PseudoElement(p)) => {
                // Try to parse state to its right. There are only 3 allowable
                // state selectors that can go on pseudo-elements.
                let mut state_selectors = SmallVec::<[Component<Impl>; 3]>::new();

                loop {
                    match input.next_including_whitespace() {
                        Ok(Token::Colon) => {},
                        Ok(Token::WhiteSpace(_)) | Err(_) => break,
                        _ => return Err(SelectorParseError::PseudoElementExpectedColon.into()),
                    }

                    // TODO(emilio): Functional pseudo-classes too?
                    // We don't need it for now.
                    let name = match input.next_including_whitespace() {
                        Ok(Token::Ident(name)) => name,
                        _ => return Err(SelectorParseError::PseudoElementExpectedIdent.into()),
                    };

                    let pseudo_class =
                        P::parse_non_ts_pseudo_class(parser, name)?;
                    if !p.supports_pseudo_class(&pseudo_class) {
                        return Err(SelectorParseError::UnsupportedPseudoClass.into());
                    }
                    state_selectors.push(Component::NonTSPseudoClass(pseudo_class));
                }

                if !builder.is_empty() {
                    builder.push_combinator(Combinator::PseudoElement);
                }

                builder.push_simple_selector(Component::PseudoElement(p));
                for state_selector in state_selectors.into_iter() {
                    builder.push_simple_selector(state_selector);
                }

                pseudo = true;
                empty = false;
                break
            }
        }
    }
    if empty {
        // An empty selector is invalid.
        Err(ParseError::Custom(SelectorParseError::EmptySelector))
    } else {
        Ok(pseudo)
    }
}

fn parse_functional_pseudo_class<'i, 't, P, E, Impl>(parser: &P,
                                                     input: &mut CssParser<'i, 't>,
                                                     name: CompactCowStr<'i>,
                                                     inside_negation: bool)
                                                     -> Result<Component<Impl>,
                                                               ParseError<'i, SelectorParseError<'i, E>>>
    where P: Parser<'i, Impl=Impl, Error=E>, Impl: SelectorImpl
{
    match_ignore_ascii_case! { &name,
        "nth-child" => return parse_nth_pseudo_class(input, Component::NthChild),
        "nth-of-type" => return parse_nth_pseudo_class(input, Component::NthOfType),
        "nth-last-child" => return parse_nth_pseudo_class(input, Component::NthLastChild),
        "nth-last-of-type" => return parse_nth_pseudo_class(input, Component::NthLastOfType),
        "not" => {
            if inside_negation {
                return Err(ParseError::Custom(SelectorParseError::UnexpectedIdent("not".into())));
            }
            return parse_negation(parser, input)
        },
        _ => {}
    }
    P::parse_non_ts_functional_pseudo_class(parser, name, input)
        .map(Component::NonTSPseudoClass)
}


fn parse_nth_pseudo_class<'i, 't, Impl, F, E>(input: &mut CssParser<'i, 't>, selector: F)
                                              -> Result<Component<Impl>,
                                                        ParseError<'i, SelectorParseError<'i, E>>>
where Impl: SelectorImpl, F: FnOnce(i32, i32) -> Component<Impl> {
    let (a, b) = parse_nth(input)?;
    Ok(selector(a, b))
}


/// Returns whether the name corresponds to a CSS2 pseudo-element that
/// can be specified with the single colon syntax (in addition to the
/// double-colon syntax, which can be used for all pseudo-elements).
pub fn is_css2_pseudo_element<'i>(name: &CompactCowStr<'i>) -> bool {
    // ** Do not add to this list! **
    return name.eq_ignore_ascii_case("before") ||
           name.eq_ignore_ascii_case("after") ||
           name.eq_ignore_ascii_case("first-line") ||
           name.eq_ignore_ascii_case("first-letter");
}

/// Parse a simple selector other than a type selector.
///
/// * `Err(())`: Invalid selector, abort
/// * `Ok(None)`: Not a simple selector, could be something else. `input` was not consumed.
/// * `Ok(Some(_))`: Parsed a simple selector or pseudo-element
fn parse_one_simple_selector<'i, 't, P, E, Impl>(parser: &P,
                                                 input: &mut CssParser<'i, 't>,
                                                 inside_negation: bool)
                                                 -> Result<Option<SimpleSelectorParseResult<Impl>>,
                                                           ParseError<'i, SelectorParseError<'i, E>>>
    where P: Parser<'i, Impl=Impl, Error=E>, Impl: SelectorImpl
{
    let start_position = input.position();
    match input.next_including_whitespace() {
        Ok(Token::IDHash(id)) => {
            let id = Component::ID(from_cow_str(id.into()));
            Ok(Some(SimpleSelectorParseResult::SimpleSelector(id)))
        }
        Ok(Token::Delim('.')) => {
            match input.next_including_whitespace()? {
                Token::Ident(class) => {
                    let class = Component::Class(from_cow_str(class.into()));
                    Ok(Some(SimpleSelectorParseResult::SimpleSelector(class)))
                }
                t => Err(ParseError::Basic(BasicParseError::UnexpectedToken(t))),
            }
        }
        Ok(Token::SquareBracketBlock) => {
            let attr = input.parse_nested_block(|input| parse_attribute_selector(parser, input))?;
            Ok(Some(SimpleSelectorParseResult::SimpleSelector(attr)))
        }
        Ok(Token::Colon) => {
            let (is_single_colon, next_token) = match input.next_including_whitespace()? {
                Token::Colon => (false, input.next_including_whitespace()?),
                t => (true, t),
            };
            let (name, is_functional) = match next_token {
                Token::Ident(name) => (name, false),
                Token::Function(name) => (name, true),
                t => return Err(ParseError::Basic(BasicParseError::UnexpectedToken(t))),
            };
            let is_pseudo_element = !is_single_colon ||
                P::is_pseudo_element_allows_single_colon(&name);
            if is_pseudo_element {
                let pseudo_element = if is_functional {
                    input.parse_nested_block(|input| {
                        P::parse_functional_pseudo_element(parser, name, input)
                    })?
                } else {
                    P::parse_pseudo_element(parser, name)?
                };
                Ok(Some(SimpleSelectorParseResult::PseudoElement(pseudo_element)))
            } else {
                let pseudo_class = if is_functional {
                    input.parse_nested_block(|input| {
                        parse_functional_pseudo_class(parser, input, name, inside_negation)
                    })?
                } else {
                    parse_simple_pseudo_class(parser, name)?
                };
                Ok(Some(SimpleSelectorParseResult::SimpleSelector(pseudo_class)))
            }
        }
        _ => {
            input.reset(start_position);
            Ok(None)
        }
    }
}

fn parse_simple_pseudo_class<'i, P, E, Impl>(parser: &P, name: CompactCowStr<'i>)
                                             -> Result<Component<Impl>,
                                                       ParseError<'i, SelectorParseError<'i, E>>>
    where P: Parser<'i, Impl=Impl, Error=E>, Impl: SelectorImpl
{
    (match_ignore_ascii_case! { &name,
        "first-child" => Ok(Component::FirstChild),
        "last-child"  => Ok(Component::LastChild),
        "only-child"  => Ok(Component::OnlyChild),
        "root" => Ok(Component::Root),
        "empty" => Ok(Component::Empty),
        "first-of-type" => Ok(Component::FirstOfType),
        "last-of-type"  => Ok(Component::LastOfType),
        "only-of-type"  => Ok(Component::OnlyOfType),
        _ => Err(())
    }).or_else(|()| {
        P::parse_non_ts_pseudo_class(parser, name)
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

    #[derive(PartialEq, Clone, Debug, Eq)]
    pub enum PseudoClass {
        Hover,
        Active,
        Lang(String),
    }

    #[derive(Eq, PartialEq, Clone, Debug)]
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
            where V: SelectorVisitor<Impl = Self::Impl> { true }
    }

    #[derive(Clone, PartialEq, Debug)]
    pub struct DummySelectorImpl;

    #[derive(Default)]
    pub struct DummyParser {
        default_ns: Option<DummyAtom>,
        ns_prefixes: HashMap<DummyAtom, DummyAtom>,
    }

    impl SelectorImpl for DummySelectorImpl {
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

    #[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
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
        type Error = ();

        fn parse_non_ts_pseudo_class(&self, name: CompactCowStr<'i>)
                                     -> Result<PseudoClass,
                                               ParseError<'i, SelectorParseError<'i, ()>>> {
            match_ignore_ascii_case! { &name,
                "hover" => Ok(PseudoClass::Hover),
                "active" => Ok(PseudoClass::Active),
                _ => Err(SelectorParseError::Custom(()).into())
            }
        }

        fn parse_non_ts_functional_pseudo_class<'t>(&self, name: CompactCowStr<'i>,
                                                    parser: &mut CssParser<'i, 't>)
                                                    -> Result<PseudoClass,
                                                              ParseError<'i, SelectorParseError<'i, ()>>> {
            match_ignore_ascii_case! { &name,
                "lang" => Ok(PseudoClass::Lang(parser.expect_ident_or_string()?.into_owned())),
                _ => Err(SelectorParseError::Custom(()).into())
            }
        }

        fn parse_pseudo_element(&self, name: CompactCowStr<'i>)
                                -> Result<PseudoElement,
                                          ParseError<'i, SelectorParseError<'i, ()>>> {
            match_ignore_ascii_case! { &name,
                "before" => Ok(PseudoElement::Before),
                "after" => Ok(PseudoElement::After),
                _ => Err(SelectorParseError::Custom(()).into())
            }
        }

        fn default_namespace(&self) -> Option<DummyAtom> {
            self.default_ns.clone()
        }

        fn namespace_for_prefix(&self, prefix: &DummyAtom) -> Option<DummyAtom> {
            self.ns_prefixes.get(prefix).cloned()
        }
    }

    fn parse<'i>(input: &'i str) -> Result<SelectorList<DummySelectorImpl>,
                                           ParseError<'i, SelectorParseError<'i, ()>>> {
        parse_ns(input, &DummyParser::default())
    }

    fn parse_ns<'i>(input: &'i str, parser: &DummyParser)
                    -> Result<SelectorList<DummySelectorImpl>,
                              ParseError<'i, SelectorParseError<'i, ()>>> {
        let mut parser_input = ParserInput::new(input);
        let result = SelectorList::parse(parser, &mut CssParser::new(&mut parser_input));
        if let Ok(ref selectors) = result {
            assert_eq!(selectors.0.len(), 1);
            assert_eq!(selectors.0[0].selector.to_css_string(), input);
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
        assert!(parse("").is_err()) ;
        assert!(parse(":lang(4)").is_err()) ;
        assert!(parse(":lang(en US)").is_err()) ;
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
        // https://github.com/servo/servo/issues/16020
        assert_eq!(parse("*|e"), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::ExplicitAnyNamespace,
                Component::LocalName(LocalName {
                    name: DummyAtom::from("e"),
                    lower_name: DummyAtom::from("e")
                })
            ), specificity(0, 0, 1))
        ))));
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
        assert_eq!(parse("*|*"), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(
                Component::ExplicitAnyNamespace,
                Component::ExplicitUniversalType,
            ), specificity(0, 0, 0))
        ))));
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
        assert_eq!(parse("[attr |= \"foo\"]"), Ok(SelectorList::from_vec(vec!(
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
        assert!(parse("table[rules]:not([rules = \"none\"]):not([rules = \"\"])").is_ok());
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
        assert_eq!(parse_ns(":not(*|*)", &parser), Ok(SelectorList::from_vec(vec!(
            Selector::from_vec(vec!(Component::Negation(
                vec![
                    Component::ExplicitAnyNamespace,
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
    }

    #[test]
    fn test_pseudo_iter() {
        let selector = &parse("q::before").unwrap().0[0].selector;
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
        let selector = &parse("*|*::before").unwrap().0[0].selector;
        assert!(selector.is_universal());
    }

    #[test]
    fn test_empty_pseudo_iter() {
        let selector = &parse("::before").unwrap().0[0].selector;
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
        parse(":not(:hover) ~ label").unwrap().0[0].selector.visit(&mut test_visitor);
        assert!(test_visitor.seen.contains(&":hover".into()));

        let mut test_visitor = TestVisitor { seen: vec![], };
        parse("::before:hover").unwrap().0[0].selector.visit(&mut test_visitor);
        assert!(test_visitor.seen.contains(&":hover".into()));
    }
}
