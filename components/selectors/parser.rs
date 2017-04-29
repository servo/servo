/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use arcslice::ArcSlice;
use cssparser::{Token, Parser as CssParser, parse_nth, ToCss, serialize_identifier, CssStringWriter};
use precomputed_hash::PrecomputedHash;
use smallvec::SmallVec;
use std::ascii::AsciiExt;
use std::borrow::{Borrow, Cow};
use std::cmp;
use std::fmt::{self, Display, Debug, Write};
use std::hash::Hash;
use std::iter::Rev;
use std::ops::Add;
use std::slice;
use tree::SELECTOR_WHITESPACE;
use visitor::SelectorVisitor;

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

        fn from_ascii_lowercase<T>(s: &str) -> T where T: $($FromStr)* {
            if let Some(first_uppercase) = s.bytes().position(|byte| byte >= b'A' && byte <= b'Z') {
                let mut string = s.to_owned();
                string[first_uppercase..].make_ascii_lowercase();
                T::from(string)
            } else {
                T::from(s)
            }
        }

        /// This trait allows to define the parser implementation in regards
        /// of pseudo-classes/elements
        ///
        /// NB: We need Clone so that we can derive(Clone) on struct with that
        /// are parameterized on SelectorImpl. See
        /// https://github.com/rust-lang/rust/issues/26925
        pub trait SelectorImpl: Clone + Sized {
            type AttrValue: $($InSelector)*;
            type Identifier: $($InSelector)* + PrecomputedHash;
            type ClassName: $($InSelector)* + PrecomputedHash;
            type LocalName: $($InSelector)* + Borrow<Self::BorrowedLocalName> + PrecomputedHash;
            type NamespaceUrl: $($CommonBounds)* + Default + Borrow<Self::BorrowedNamespaceUrl> + PrecomputedHash;
            type NamespacePrefix: $($InSelector)* + Default;
            type BorrowedNamespaceUrl: ?Sized + Eq;
            type BorrowedLocalName: ?Sized + Eq + Hash;

            /// non tree-structural pseudo-classes
            /// (see: https://drafts.csswg.org/selectors/#structural-pseudos)
            type NonTSPseudoClass: $($CommonBounds)* + Sized + ToCss + SelectorMethods<Impl = Self>;

            /// pseudo-elements
            type PseudoElement: $($CommonBounds)* + Sized + ToCss;
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
    [Clone + Eq + Hash]
    [From<String> + for<'a> From<&'a str>]
}

pub trait Parser {
    type Impl: SelectorImpl;

    /// This function can return an "Err" pseudo-element in order to support CSS2.1
    /// pseudo-elements.
    fn parse_non_ts_pseudo_class(&self, _name: Cow<str>)
                                 -> Result<<Self::Impl as SelectorImpl>::NonTSPseudoClass, ()> {
        Err(())
    }

    fn parse_non_ts_functional_pseudo_class
        (&self, _name: Cow<str>, _arguments: &mut CssParser)
        -> Result<<Self::Impl as SelectorImpl>::NonTSPseudoClass, ()>
    {
        Err(())
    }

    fn parse_pseudo_element(&self, _name: Cow<str>)
                            -> Result<<Self::Impl as SelectorImpl>::PseudoElement, ()> {
        Err(())
    }

    fn default_namespace(&self) -> Option<<Self::Impl as SelectorImpl>::NamespaceUrl> {
        None
    }

    fn namespace_for_prefix(&self, _prefix: &<Self::Impl as SelectorImpl>::NamespacePrefix)
                            -> Option<<Self::Impl as SelectorImpl>::NamespaceUrl> {
        None
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct SelectorList<Impl: SelectorImpl>(pub Vec<Selector<Impl>>);

impl<Impl: SelectorImpl> SelectorList<Impl> {
    /// Parse a comma-separated list of Selectors.
    /// https://drafts.csswg.org/selectors/#grouping
    ///
    /// Return the Selectors or Err if there is an invalid selector.
    pub fn parse<P>(parser: &P, input: &mut CssParser) -> Result<Self, ()>
    where P: Parser<Impl=Impl> {
        input.parse_comma_separated(|input| parse_selector(parser, input))
             .map(SelectorList)
    }
}

/// Copied from Gecko, where it was noted to be unmeasured.
const NUM_ANCESTOR_HASHES: usize = 4;

/// The cores parts of a selector used for matching. This exists to make that
/// information accessibly separately from the specificity and pseudo-element
/// information that lives on |Selector| proper. We may want to refactor things
/// and move that information elsewhere, at which point we could rename this
/// to |Selector|.
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct SelectorInner<Impl: SelectorImpl> {
    /// The selector data.
    pub complex: ComplexSelector<Impl>,
    /// Ancestor hashes for the bloom filter. We precompute these and store
    /// them inline to optimize cache performance during selector matching.
    /// This matters a lot.
    pub ancestor_hashes: [u32; NUM_ANCESTOR_HASHES],
}

impl<Impl: SelectorImpl> SelectorInner<Impl> {
    pub fn new(c: ComplexSelector<Impl>) -> Self {
        let mut hashes = [0; NUM_ANCESTOR_HASHES];
        {
            // Compute ancestor hashes for the bloom filter.
            let mut hash_iter = c.iter_ancestors()
                                 .map(|x| x.ancestor_hash())
                                 .filter(|x| x.is_some())
                                 .map(|x| x.unwrap());
            for i in 0..NUM_ANCESTOR_HASHES {
                hashes[i] = match hash_iter.next() {
                    Some(x) => x,
                    None => break,
                }
            }
        }

        SelectorInner {
            complex: c,
            ancestor_hashes: hashes,
        }
    }

    /// Creates a clone of this selector with everything to the left of
    /// (and including) the rightmost ancestor combinator removed. So
    /// the selector |span foo > bar + baz| will become |bar + baz|.
    /// This is used for revalidation selectors in servo.
    ///
    /// The bloom filter hashes are copied, even though they correspond to
    /// parts of the selector that have been stripped out, because they are
    /// still useful for fast-rejecting the reduced selectors.
    pub fn slice_to_first_ancestor_combinator(&self) -> Self {
        let maybe_pos = self.complex.iter_raw()
                            .position(|s| s.as_combinator()
                                           .map_or(false, |c| c.is_ancestor()));
        match maybe_pos {
            None => self.clone(),
            Some(index) => SelectorInner {
                complex: self.complex.slice_to(index),
                ancestor_hashes: self.ancestor_hashes.clone(),
            },
        }
    }

    /// Creates a SelectorInner from a Vec of Components. Used in tests.
    pub fn from_vec(vec: Vec<Component<Impl>>) -> Self {
        let complex = ComplexSelector::from_vec(vec);
        Self::new(complex)
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Selector<Impl: SelectorImpl> {
    pub inner: SelectorInner<Impl>,
    pub pseudo_element: Option<Impl::PseudoElement>,
    pub specificity: u32,
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
        self.inner.complex.visit(visitor)
    }
}

impl<Impl: SelectorImpl> SelectorMethods for ComplexSelector<Impl> {
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
            },
            AttrExists(ref selector) |
            AttrEqual(ref selector, _, _) |
            AttrIncludes(ref selector, _) |
            AttrDashMatch(ref selector, _) |
            AttrPrefixMatch(ref selector, _) |
            AttrSubstringMatch(ref selector, _) |
            AttrSuffixMatch(ref selector, _) => {
                if !visitor.visit_attribute_selector(selector) {
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

/// A ComplexSelectors stores a sequence of simple selectors and combinators. The
/// iterator classes allow callers to iterate at either the raw sequence level or
/// at the level of sequences of simple selectors separated by combinators. Most
/// callers want the higher-level iterator.
///
/// We store selectors internally left-to-right (in parsing order), but the
/// canonical iteration order is right-to-left (selector matching order). The
/// iterators abstract over these details.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ComplexSelector<Impl: SelectorImpl>(ArcSlice<Component<Impl>>);

impl<Impl: SelectorImpl> ComplexSelector<Impl> {
    /// Returns an iterator over the next sequence of simple selectors. When
    /// a combinator is reached, the iterator will return None, and
    /// next_sequence() may be called to continue to the next sequence.
    pub fn iter(&self) -> SelectorIter<Impl> {
        SelectorIter {
            iter: self.iter_raw(),
            next_combinator: None,
        }
    }

    /// Returns an iterator over the entire sequence of simple selectors and combinators,
    /// from right to left.
    pub fn iter_raw(&self) -> Rev<slice::Iter<Component<Impl>>> {
        self.iter_raw_rev().rev()
    }

    /// Returns an iterator over the entire sequence of simple selectors and combinators,
    /// from left to right.
    pub fn iter_raw_rev(&self) -> slice::Iter<Component<Impl>> {
        self.0.iter()
    }

    /// Returns an iterator over ancestor simple selectors. All combinators and
    /// non-ancestor simple selectors will be skipped.
    pub fn iter_ancestors(&self) -> AncestorIter<Impl> {
        AncestorIter::new(self.iter())
    }

    /// Returns a ComplexSelector identical to |self| but with the rightmost |index|
    /// entries removed.
    pub fn slice_from(&self, index: usize) -> Self {
        // Note that we convert the slice_from to slice_to because selectors are
        // stored left-to-right but logical order is right-to-left.
        ComplexSelector(self.0.clone().slice_to(self.0.len() - index))
    }

    /// Returns a ComplexSelector identical to |self| but with the leftmost
    /// |len() - index| entries removed.
    pub fn slice_to(&self, index: usize) -> Self {
        // Note that we convert the slice_to to slice_from because selectors are
        // stored left-to-right but logical order is right-to-left.
        ComplexSelector(self.0.clone().slice_from(self.0.len() - index))
    }

    /// Creates a ComplexSelector from a vec of Components. Used in tests.
    pub fn from_vec(vec: Vec<Component<Impl>>) -> Self {
        ComplexSelector(ArcSlice::new(vec.into_boxed_slice()))
    }
}

#[derive(Clone)]
pub struct SelectorIter<'a, Impl: 'a + SelectorImpl> {
    iter: Rev<slice::Iter<'a, Component<Impl>>>,
    next_combinator: Option<Combinator>,
}

impl<'a, Impl: 'a + SelectorImpl> SelectorIter<'a, Impl> {
    /// Prepares this iterator to point to the next sequence to the left,
    /// returning the combinator if the sequence was found.
    pub fn next_sequence(&mut self) -> Option<Combinator> {
        self.next_combinator.take()
    }
}

impl<'a, Impl: SelectorImpl> Iterator for SelectorIter<'a, Impl> {
    type Item = &'a Component<Impl>;
    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.next_combinator.is_none(), "Should call take_combinator!");
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

    /// Skips a sequence of simple selectors and all subsequent sequences until an
    /// ancestor combinator is reached.
    fn skip_until_ancestor(&mut self) {
        loop {
            while let Some(_) = self.0.next() {}
            if self.0.next_sequence().map_or(true, |x| x.is_ancestor()) {
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
            if !combinator.is_ancestor() {
                self.skip_until_ancestor();
            }
        }

        self.0.next()
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub enum Combinator {
    Child,  //  >
    Descendant,  // space
    NextSibling,  // +
    LaterSibling,  // ~
}

impl Combinator {
    /// Returns true if this combinator is a child or descendant combinator.
    pub fn is_ancestor(&self) -> bool {
        matches!(*self, Combinator::Child | Combinator::Descendant)
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
#[derive(Eq, PartialEq, Clone, Hash)]
pub enum Component<Impl: SelectorImpl> {
    Combinator(Combinator),
    ID(Impl::Identifier),
    Class(Impl::ClassName),
    LocalName(LocalName<Impl>),
    Namespace(Namespace<Impl>),

    // Attribute selectors
    AttrExists(AttrSelector<Impl>),  // [foo]
    AttrEqual(AttrSelector<Impl>, Impl::AttrValue, CaseSensitivity),  // [foo=bar]
    AttrIncludes(AttrSelector<Impl>, Impl::AttrValue),  // [foo~=bar]
    AttrDashMatch(AttrSelector<Impl>, Impl::AttrValue), // [foo|=bar]
    AttrPrefixMatch(AttrSelector<Impl>, Impl::AttrValue),  // [foo^=bar]
    AttrSubstringMatch(AttrSelector<Impl>, Impl::AttrValue),  // [foo*=bar]
    AttrSuffixMatch(AttrSelector<Impl>, Impl::AttrValue),  // [foo$=bar]

    AttrIncludesNeverMatch(AttrSelector<Impl>, Impl::AttrValue),  // empty value or with whitespace
    AttrPrefixNeverMatch(AttrSelector<Impl>, Impl::AttrValue),  // empty value
    AttrSubstringNeverMatch(AttrSelector<Impl>, Impl::AttrValue),  // empty value
    AttrSuffixNeverMatch(AttrSelector<Impl>, Impl::AttrValue),  // empty value

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
    // ...
}

impl<Impl: SelectorImpl> Component<Impl> {
    /// Compute the ancestor hash to check against the bloom filter.
    fn ancestor_hash(&self) -> Option<u32> {
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
            Component::Namespace(ref namespace) => {
                Some(namespace.url.precomputed_hash())
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

#[derive(Eq, PartialEq, Clone, Hash, Copy, Debug)]
pub enum CaseSensitivity {
    CaseSensitive,  // Selectors spec says language-defined, but HTML says sensitive.
    CaseInsensitive,
}


#[derive(Eq, PartialEq, Clone, Hash)]
pub struct LocalName<Impl: SelectorImpl> {
    pub name: Impl::LocalName,
    pub lower_name: Impl::LocalName,
}

#[derive(Eq, PartialEq, Clone, Hash)]
pub struct AttrSelector<Impl: SelectorImpl> {
    pub name: Impl::LocalName,
    pub lower_name: Impl::LocalName,
    pub namespace: NamespaceConstraint<Impl>,
}

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub enum NamespaceConstraint<Impl: SelectorImpl> {
    Any,
    Specific(Namespace<Impl>),
}

/// FIXME(SimonSapin): should Hash only hash the URL? What is it used for?
#[derive(Eq, PartialEq, Clone, Hash)]
pub struct Namespace<Impl: SelectorImpl> {
    pub prefix: Option<Impl::NamespacePrefix>,
    pub url: Impl::NamespaceUrl,
}

impl<Impl: SelectorImpl> Default for Namespace<Impl> {
    fn default() -> Self {
        Namespace {
            prefix: None,
            url: Impl::NamespaceUrl::default(),  // empty string
        }
    }
}


impl<Impl: SelectorImpl> Debug for Selector<Impl> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Selector(")?;
        self.to_css(f)?;
        write!(f, ", specificity = 0x{:x})", self.specificity)
    }
}

impl<Impl: SelectorImpl> Debug for SelectorInner<Impl> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.complex.to_css(f) }
}
impl<Impl: SelectorImpl> Debug for ComplexSelector<Impl> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.to_css(f) }
}
impl<Impl: SelectorImpl> Debug for Component<Impl> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.to_css(f) }
}
impl<Impl: SelectorImpl> Debug for AttrSelector<Impl> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.to_css(f) }
}
impl<Impl: SelectorImpl> Debug for Namespace<Impl> {
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
        self.inner.complex.to_css(dest)?;
        if let Some(ref pseudo) = self.pseudo_element {
            pseudo.to_css(dest)?;
        }
        Ok(())
    }
}

impl<Impl: SelectorImpl> ToCss for ComplexSelector<Impl> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
       for item in self.iter_raw_rev() {
           item.to_css(dest)?;
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
            ID(ref s) => {
                dest.write_char('#')?;
                display_to_css_identifier(s, dest)
            }
            Class(ref s) => {
                dest.write_char('.')?;
                display_to_css_identifier(s, dest)
            }
            LocalName(ref s) => s.to_css(dest),
            Namespace(ref ns) => ns.to_css(dest),

            // Attribute selectors
            AttrExists(ref a) => {
                dest.write_char('[')?;
                a.to_css(dest)?;
                dest.write_char(']')
            }
            AttrEqual(ref a, ref v, case) => {
                attr_selector_to_css(a, " = ", v, match case {
                    CaseSensitivity::CaseSensitive => None,
                    CaseSensitivity::CaseInsensitive => Some(" i"),
                }, dest)
            }
            AttrDashMatch(ref a, ref v) => attr_selector_to_css(a, " |= ", v, None, dest),
            AttrIncludesNeverMatch(ref a, ref v) |
            AttrIncludes(ref a, ref v) => attr_selector_to_css(a, " ~= ", v, None, dest),
            AttrPrefixNeverMatch(ref a, ref v) |
            AttrPrefixMatch(ref a, ref v) => attr_selector_to_css(a, " ^= ", v, None, dest),
            AttrSubstringNeverMatch(ref a, ref v) |
            AttrSubstringMatch(ref a, ref v) => attr_selector_to_css(a, " *= ", v, None, dest),
            AttrSuffixNeverMatch(ref a, ref v) |
            AttrSuffixMatch(ref a, ref v) => attr_selector_to_css(a, " $= ", v, None, dest),

            // Pseudo-classes
            Negation(ref arg) => {
                dest.write_str(":not(")?;
                debug_assert!(arg.len() <= 1 || (arg.len() == 2 && matches!(arg[0], Component::Namespace(_))));
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

impl<Impl: SelectorImpl> ToCss for AttrSelector<Impl> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if let NamespaceConstraint::Specific(ref ns) = self.namespace {
            ns.to_css(dest)?;
        }
        display_to_css_identifier(&self.name, dest)
    }
}

impl<Impl: SelectorImpl> ToCss for Namespace<Impl> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if let Some(ref prefix) = self.prefix {
            display_to_css_identifier(prefix, dest)?;
            dest.write_char('|')?;
        }
        Ok(())
    }
}

impl<Impl: SelectorImpl> ToCss for LocalName<Impl> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        display_to_css_identifier(&self.name, dest)
    }
}

fn attr_selector_to_css<Impl, W>(attr: &AttrSelector<Impl>,
                                 operator: &str,
                                 value: &Impl::AttrValue,
                                 modifier: Option<&str>,
                                 dest: &mut W)
                                 -> fmt::Result
where Impl: SelectorImpl, W: fmt::Write
{
    dest.write_char('[')?;
    attr.to_css(dest)?;
    dest.write_str(operator)?;
    dest.write_char('"')?;
    write!(CssStringWriter::new(dest), "{}", value)?;
    dest.write_char('"')?;
    if let Some(m) = modifier {
        dest.write_str(m)?;
    }
    dest.write_char(']')
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

const MAX_10BIT: u32 = (1u32 << 10) - 1;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct Specificity {
    id_selectors: u32,
    class_like_selectors: u32,
    element_selectors: u32,
}

impl Add for Specificity {
    type Output = Specificity;

    fn add(self, rhs: Specificity) -> Specificity {
        Specificity {
            id_selectors: self.id_selectors + rhs.id_selectors,
            class_like_selectors:
                self.class_like_selectors + rhs.class_like_selectors,
            element_selectors:
                self.element_selectors + rhs.element_selectors,
        }
    }
}

impl Default for Specificity {
    fn default() -> Specificity {
        Specificity {
            id_selectors: 0,
            class_like_selectors: 0,
            element_selectors: 0,
        }
    }
}

impl From<u32> for Specificity {
    fn from(value: u32) -> Specificity {
        assert!(value <= MAX_10BIT << 20 | MAX_10BIT << 10 | MAX_10BIT);
        Specificity {
            id_selectors: value >> 20,
            class_like_selectors: (value >> 10) & MAX_10BIT,
            element_selectors: value & MAX_10BIT,
        }
    }
}

impl From<Specificity> for u32 {
    fn from(specificity: Specificity) -> u32 {
        cmp::min(specificity.id_selectors, MAX_10BIT) << 20
        | cmp::min(specificity.class_like_selectors, MAX_10BIT) << 10
        | cmp::min(specificity.element_selectors, MAX_10BIT)
    }
}

fn specificity<Impl>(complex_selector: &ComplexSelector<Impl>,
                     pseudo_element: Option<&Impl::PseudoElement>)
                     -> u32
                     where Impl: SelectorImpl {
    let mut specificity = complex_selector_specificity(complex_selector);
    if pseudo_element.is_some() {
        specificity.element_selectors += 1;
    }
    specificity.into()
}

fn complex_selector_specificity<Impl>(selector: &ComplexSelector<Impl>)
                                      -> Specificity
                                      where Impl: SelectorImpl {
    fn simple_selector_specificity<Impl>(simple_selector: &Component<Impl>,
                                         specificity: &mut Specificity)
                                         where Impl: SelectorImpl {
        match *simple_selector {
            Component::Combinator(..) => unreachable!(),
            Component::LocalName(..) =>
                specificity.element_selectors += 1,
            Component::ID(..) =>
                specificity.id_selectors += 1,
            Component::Class(..) |
            Component::AttrExists(..) |
            Component::AttrEqual(..) |
            Component::AttrIncludes(..) |
            Component::AttrDashMatch(..) |
            Component::AttrPrefixMatch(..) |
            Component::AttrSubstringMatch(..) |
            Component::AttrSuffixMatch(..) |

            Component::AttrIncludesNeverMatch(..) |
            Component::AttrPrefixNeverMatch(..) |
            Component::AttrSubstringNeverMatch(..) |
            Component::AttrSuffixNeverMatch(..) |

            Component::FirstChild | Component::LastChild |
            Component::OnlyChild | Component::Root |
            Component::Empty |
            Component::NthChild(..) |
            Component::NthLastChild(..) |
            Component::NthOfType(..) |
            Component::NthLastOfType(..) |
            Component::FirstOfType | Component::LastOfType |
            Component::OnlyOfType |
            Component::NonTSPseudoClass(..) =>
                specificity.class_like_selectors += 1,

            Component::Namespace(..) => (),
            Component::Negation(ref negated) => {
                for ss in negated.iter() {
                    simple_selector_specificity(&ss, specificity);
                }
            }
        }
    }


    let mut specificity = Default::default();
    let mut iter = selector.iter();
    loop {
        for simple_selector in &mut iter {
            simple_selector_specificity(&simple_selector, &mut specificity);
        }
        if iter.next_sequence().is_none() {
            break;
        }
    }
    specificity
}

/// Build up a Selector.
/// selector : simple_selector_sequence [ combinator simple_selector_sequence ]* ;
///
/// `Err` means invalid selector.
fn parse_selector<P, Impl>(parser: &P, input: &mut CssParser) -> Result<Selector<Impl>, ()>
    where P: Parser<Impl=Impl>, Impl: SelectorImpl
{
    let (complex, pseudo_element) =
        parse_complex_selector_and_pseudo_element(parser, input)?;
    Ok(Selector {
        specificity: specificity(&complex, pseudo_element.as_ref()),
        inner: SelectorInner::new(complex),
        pseudo_element: pseudo_element,
    })
}

/// We use a SmallVec for parsing to avoid extra reallocs compared to using a Vec
/// directly. When parsing is done, we convert the SmallVec into a Vec (which is
/// free if the vec has already spilled to the heap, and more cache-friendly if
/// it hasn't), and then steal the buffer of that vec into a boxed slice.
///
/// If we parse N <= 4 entries, we save no reallocations.
/// If we parse 4 < N <= 8 entries, we save one reallocation.
/// If we parse N > 8 entries, we save two reallocations.
type ParseVec<Impl> = SmallVec<[Component<Impl>; 8]>;

fn parse_complex_selector_and_pseudo_element<P, Impl>(
        parser: &P,
        input: &mut CssParser)
        -> Result<(ComplexSelector<Impl>, Option<Impl::PseudoElement>), ()>
    where P: Parser<Impl=Impl>, Impl: SelectorImpl
{
    let mut sequence = ParseVec::new();
    let mut pseudo_element;
    'outer_loop: loop {
        // Parse a sequence of simple selectors.
        pseudo_element = parse_compound_selector(parser, input, &mut sequence,
                                                 /* inside_negation = */ false)?;
        if pseudo_element.is_some() {
            break;
        }

        // Parse a combinator.
        let combinator;
        let mut any_whitespace = false;
        loop {
            let position = input.position();
            match input.next_including_whitespace() {
                Err(()) => break 'outer_loop,
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
        sequence.push(Component::Combinator(combinator));
    }

    let complex = ComplexSelector(ArcSlice::new(sequence.into_vec().into_boxed_slice()));
    Ok((complex, pseudo_element))
}

impl<Impl: SelectorImpl> ComplexSelector<Impl> {
    /// Parse a complex selector.
    pub fn parse<P>(parser: &P, input: &mut CssParser) -> Result<Self, ()>
        where P: Parser<Impl=Impl>
    {
        let (complex, pseudo_element) =
            parse_complex_selector_and_pseudo_element(parser, input)?;
        if pseudo_element.is_some() {
            return Err(())
        }
        Ok(complex)
    }
}

/// * `Err(())`: Invalid selector, abort
/// * `Ok(None)`: Not a type selector, could be something else. `input` was not consumed.
/// * `Ok(Some(vec))`: Length 0 (`*|*`), 1 (`*|E` or `ns|*`) or 2 (`|E` or `ns|E`)
fn parse_type_selector<P, Impl>(parser: &P, input: &mut CssParser, sequence: &mut ParseVec<Impl>)
                       -> Result<bool, ()>
    where P: Parser<Impl=Impl>, Impl: SelectorImpl
{
    match parse_qualified_name(parser, input, /* in_attr_selector = */ false)? {
        None => Ok(false),
        Some((namespace, local_name)) => {
            match namespace {
                NamespaceConstraint::Specific(ns) => {
                    sequence.push(Component::Namespace(ns))
                },
                NamespaceConstraint::Any => (),
            }
            match local_name {
                Some(name) => {
                    sequence.push(Component::LocalName(LocalName {
                        lower_name: from_ascii_lowercase(&name),
                        name: from_cow_str(name),
                    }))
                }
                None => (),
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

/// * `Err(())`: Invalid selector, abort
/// * `Ok(None)`: Not a simple selector, could be something else. `input` was not consumed.
/// * `Ok(Some((namespace, local_name)))`: `None` for the local name means a `*` universal selector
fn parse_qualified_name<'i, 't, P, Impl>
                       (parser: &P, input: &mut CssParser<'i, 't>,
                        in_attr_selector: bool)
                        -> Result<Option<(NamespaceConstraint<Impl>, Option<Cow<'i, str>>)>, ()>
    where P: Parser<Impl=Impl>, Impl: SelectorImpl
{
    let default_namespace = |local_name| {
        let namespace = match parser.default_namespace() {
            Some(url) => NamespaceConstraint::Specific(Namespace {
                prefix: None,
                url: url
            }),
            None => NamespaceConstraint::Any,
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
            _ => Err(()),
        }
    };

    let position = input.position();
    match input.next_including_whitespace() {
        Ok(Token::Ident(value)) => {
            let position = input.position();
            match input.next_including_whitespace() {
                Ok(Token::Delim('|')) => {
                    let prefix = from_cow_str(value);
                    let result = parser.namespace_for_prefix(&prefix);
                    let url = result.ok_or(())?;
                    explicit_namespace(input, NamespaceConstraint::Specific(Namespace {
                        prefix: Some(prefix),
                        url: url
                    }))
                },
                _ => {
                    input.reset(position);
                    if in_attr_selector {
                        Ok(Some((NamespaceConstraint::Specific(Default::default()), Some(value))))
                    } else {
                        default_namespace(Some(value))
                    }
                }
            }
        },
        Ok(Token::Delim('*')) => {
            let position = input.position();
            match input.next_including_whitespace() {
                Ok(Token::Delim('|')) => explicit_namespace(input, NamespaceConstraint::Any),
                _ => {
                    input.reset(position);
                    if in_attr_selector {
                        Err(())
                    } else {
                        default_namespace(None)
                    }
                },
            }
        },
        Ok(Token::Delim('|')) => {
            explicit_namespace(input, NamespaceConstraint::Specific(Default::default()))
        }
        _ => {
            input.reset(position);
            Ok(None)
        }
    }
}


fn parse_attribute_selector<P, Impl>(parser: &P, input: &mut CssParser)
                                     -> Result<Component<Impl>, ()>
    where P: Parser<Impl=Impl>, Impl: SelectorImpl
{
    let attr = match parse_qualified_name(parser, input, /* in_attr_selector = */ true)? {
        None => return Err(()),
        Some((_, None)) => unreachable!(),
        Some((namespace, Some(local_name))) => AttrSelector {
            namespace: namespace,
            lower_name: from_ascii_lowercase(&local_name),
            name: from_cow_str(local_name),
        },
    };

    match input.next() {
        // [foo]
        Err(()) => Ok(Component::AttrExists(attr)),

        // [foo=bar]
        Ok(Token::Delim('=')) => {
            let value = input.expect_ident_or_string()?;
            let flags = parse_attribute_flags(input)?;
            Ok(Component::AttrEqual(attr, from_cow_str(value), flags))
        }
        // [foo~=bar]
        Ok(Token::IncludeMatch) => {
            let value = input.expect_ident_or_string()?;
            if value.is_empty() || value.contains(SELECTOR_WHITESPACE) {
                Ok(Component::AttrIncludesNeverMatch(attr, from_cow_str(value)))
            } else {
                Ok(Component::AttrIncludes(attr, from_cow_str(value)))
            }
        }
        // [foo|=bar]
        Ok(Token::DashMatch) => {
            let value = input.expect_ident_or_string()?;
            Ok(Component::AttrDashMatch(attr, from_cow_str(value)))
        }
        // [foo^=bar]
        Ok(Token::PrefixMatch) => {
            let value = input.expect_ident_or_string()?;
            if value.is_empty() {
                Ok(Component::AttrPrefixNeverMatch(attr, from_cow_str(value)))
            } else {
                Ok(Component::AttrPrefixMatch(attr, from_cow_str(value)))
            }
        }
        // [foo*=bar]
        Ok(Token::SubstringMatch) => {
            let value = input.expect_ident_or_string()?;
            if value.is_empty() {
                Ok(Component::AttrSubstringNeverMatch(attr, from_cow_str(value)))
            } else {
                Ok(Component::AttrSubstringMatch(attr, from_cow_str(value)))
            }
        }
        // [foo$=bar]
        Ok(Token::SuffixMatch) => {
            let value = input.expect_ident_or_string()?;
            if value.is_empty() {
                Ok(Component::AttrSuffixNeverMatch(attr, from_cow_str(value)))
            } else {
                Ok(Component::AttrSuffixMatch(attr, from_cow_str(value)))
            }
        }
        _ => Err(())
    }
}


fn parse_attribute_flags(input: &mut CssParser) -> Result<CaseSensitivity, ()> {
    match input.next() {
        Err(()) => Ok(CaseSensitivity::CaseSensitive),
        Ok(Token::Ident(ref value)) if value.eq_ignore_ascii_case("i") => {
            Ok(CaseSensitivity::CaseInsensitive)
        }
        _ => Err(())
    }
}


/// Level 3: Parse **one** simple_selector.  (Though we might insert a second
/// implied "<defaultns>|*" type selector.)
fn parse_negation<P, Impl>(parser: &P,
                           input: &mut CssParser)
                           -> Result<Component<Impl>, ()>
    where P: Parser<Impl=Impl>, Impl: SelectorImpl
{
    let mut v = ParseVec::new();
    parse_compound_selector(parser, input, &mut v, /* inside_negation = */ true)?;

    let allow = v.len() <= 1 ||
        (v.len() == 2 && matches!(v[0], Component::Namespace(_)) &&
         matches!(v[1], Component::LocalName(_)));

    if allow {
        Ok(Component::Negation(v.into_vec().into_boxed_slice()))
    } else {
        Err(())
    }
}

/// simple_selector_sequence
/// : [ type_selector | universal ] [ HASH | class | attrib | pseudo | negation ]*
/// | [ HASH | class | attrib | pseudo | negation ]+
///
/// `Err(())` means invalid selector
fn parse_compound_selector<P, Impl>(
    parser: &P,
    input: &mut CssParser,
    mut sequence: &mut ParseVec<Impl>,
    inside_negation: bool)
    -> Result<Option<Impl::PseudoElement>, ()>
    where P: Parser<Impl=Impl>, Impl: SelectorImpl
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
    if !parse_type_selector(parser, input, &mut sequence)? {
        if let Some(url) = parser.default_namespace() {
            // If there was no explicit type selector, but there is a
            // default namespace, there is an implicit "<defaultns>|*" type
            // selector.
            //
            // Note that this doesn't apply to :not() and :matches() per spec.
            if !inside_negation {
                sequence.push(Component::Namespace(Namespace {
                    prefix: None,
                    url: url
                }));
            }
        }
    } else {
        empty = false;
    }

    let mut pseudo_element = None;
    loop {
        match parse_one_simple_selector(parser, input, inside_negation)? {
            None => break,
            Some(SimpleSelectorParseResult::SimpleSelector(s)) => {
                sequence.push(s);
                empty = false
            }
            Some(SimpleSelectorParseResult::PseudoElement(p)) => {
                pseudo_element = Some(p);
                empty = false;
                break
            }
        }
    }
    if empty {
        // An empty selector is invalid.
        Err(())
    } else {
        Ok(pseudo_element)
    }
}

fn parse_functional_pseudo_class<P, Impl>(parser: &P,
                                          input: &mut CssParser,
                                          name: Cow<str>,
                                          inside_negation: bool)
                                          -> Result<Component<Impl>, ()>
    where P: Parser<Impl=Impl>, Impl: SelectorImpl
{
    match_ignore_ascii_case! { &name,
        "nth-child" => return parse_nth_pseudo_class(input, Component::NthChild),
        "nth-of-type" => return parse_nth_pseudo_class(input, Component::NthOfType),
        "nth-last-child" => return parse_nth_pseudo_class(input, Component::NthLastChild),
        "nth-last-of-type" => return parse_nth_pseudo_class(input, Component::NthLastOfType),
        "not" => {
            if inside_negation {
                return Err(())
            }
            return parse_negation(parser, input)
        },
        _ => {}
    }
    P::parse_non_ts_functional_pseudo_class(parser, name, input)
        .map(Component::NonTSPseudoClass)
}


fn parse_nth_pseudo_class<Impl, F>(input: &mut CssParser, selector: F)
                                   -> Result<Component<Impl>, ()>
where Impl: SelectorImpl, F: FnOnce(i32, i32) -> Component<Impl> {
    let (a, b) = parse_nth(input)?;
    Ok(selector(a, b))
}


/// Parse a simple selector other than a type selector.
///
/// * `Err(())`: Invalid selector, abort
/// * `Ok(None)`: Not a simple selector, could be something else. `input` was not consumed.
/// * `Ok(Some(_))`: Parsed a simple selector or pseudo-element
fn parse_one_simple_selector<P, Impl>(parser: &P,
                                      input: &mut CssParser,
                                      inside_negation: bool)
                                      -> Result<Option<SimpleSelectorParseResult<Impl>>, ()>
    where P: Parser<Impl=Impl>, Impl: SelectorImpl
{
    let start_position = input.position();
    match input.next_including_whitespace() {
        Ok(Token::IDHash(id)) => {
            let id = Component::ID(from_cow_str(id));
            Ok(Some(SimpleSelectorParseResult::SimpleSelector(id)))
        }
        Ok(Token::Delim('.')) => {
            match input.next_including_whitespace() {
                Ok(Token::Ident(class)) => {
                    let class = Component::Class(from_cow_str(class));
                    Ok(Some(SimpleSelectorParseResult::SimpleSelector(class)))
                }
                _ => Err(()),
            }
        }
        Ok(Token::SquareBracketBlock) => {
            let attr = input.parse_nested_block(|input| parse_attribute_selector(parser, input))?;
            Ok(Some(SimpleSelectorParseResult::SimpleSelector(attr)))
        }
        Ok(Token::Colon) => {
            match input.next_including_whitespace() {
                Ok(Token::Ident(name)) => {
                    // Supported CSS 2.1 pseudo-elements only.
                    // ** Do not add to this list! **
                    if name.eq_ignore_ascii_case("before") ||
                       name.eq_ignore_ascii_case("after") ||
                       name.eq_ignore_ascii_case("first-line") ||
                       name.eq_ignore_ascii_case("first-letter") {
                        let pseudo_element = P::parse_pseudo_element(parser, name)?;
                        Ok(Some(SimpleSelectorParseResult::PseudoElement(pseudo_element)))
                    } else {
                        let pseudo_class = parse_simple_pseudo_class(parser, name)?;
                        Ok(Some(SimpleSelectorParseResult::SimpleSelector(pseudo_class)))
                    }
                }
                Ok(Token::Function(name)) => {
                    let pseudo = input.parse_nested_block(|input| {
                        parse_functional_pseudo_class(parser, input, name, inside_negation)
                    })?;
                    Ok(Some(SimpleSelectorParseResult::SimpleSelector(pseudo)))
                }
                Ok(Token::Colon) => {
                    match input.next_including_whitespace() {
                        Ok(Token::Ident(name)) => {
                            let pseudo = P::parse_pseudo_element(parser, name)?;
                            Ok(Some(SimpleSelectorParseResult::PseudoElement(pseudo)))
                        }
                        _ => Err(())
                    }
                }
                _ => Err(())
            }
        }
        _ => {
            input.reset(start_position);
            Ok(None)
        }
    }
}

fn parse_simple_pseudo_class<P, Impl>(parser: &P, name: Cow<str>) -> Result<Component<Impl>, ()>
    where P: Parser<Impl=Impl>, Impl: SelectorImpl
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
    use cssparser::{Parser as CssParser, ToCss, serialize_identifier};
    use std::borrow::Cow;
    use std::collections::HashMap;
    use std::fmt;
    use super::*;

    #[derive(PartialEq, Clone, Debug, Hash, Eq)]
    pub enum PseudoClass {
        Hover,
        Lang(String),
    }

    #[derive(Eq, PartialEq, Clone, Debug, Hash)]
    pub enum PseudoElement {
        Before,
        After,
    }

    impl ToCss for PseudoClass {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                PseudoClass::Hover => dest.write_str(":hover"),
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
    }

    #[derive(Default, Debug, Hash, Clone, PartialEq, Eq)]
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

    impl Parser for DummyParser {
        type Impl = DummySelectorImpl;

        fn parse_non_ts_pseudo_class(&self, name: Cow<str>)
                                     -> Result<PseudoClass, ()> {
            match_ignore_ascii_case! { &name,
                "hover" => Ok(PseudoClass::Hover),
                _ => Err(())
            }
        }

        fn parse_non_ts_functional_pseudo_class(&self, name: Cow<str>,
                                                parser: &mut CssParser)
                                                -> Result<PseudoClass, ()> {
            match_ignore_ascii_case! { &name,
                "lang" => Ok(PseudoClass::Lang(try!(parser.expect_ident_or_string()).into_owned())),
                _ => Err(())
            }
        }

        fn parse_pseudo_element(&self, name: Cow<str>)
                                -> Result<PseudoElement, ()> {
            match_ignore_ascii_case! { &name,
                "before" => Ok(PseudoElement::Before),
                "after" => Ok(PseudoElement::After),
                _ => Err(())
            }
        }

        fn default_namespace(&self) -> Option<DummyAtom> {
            self.default_ns.clone()
        }

        fn namespace_for_prefix(&self, prefix: &DummyAtom) -> Option<DummyAtom> {
            self.ns_prefixes.get(prefix).cloned()
        }
    }

    fn parse(input: &str) -> Result<SelectorList<DummySelectorImpl>, ()> {
        parse_ns(input, &DummyParser::default())
    }

    fn parse_ns(input: &str, parser: &DummyParser)
                -> Result<SelectorList<DummySelectorImpl>, ()> {
        let result = SelectorList::parse(parser, &mut CssParser::new(input));
        if let Ok(ref selectors) = result {
            assert_eq!(selectors.0.len(), 1);
            assert_eq!(selectors.0[0].to_css_string(), input);
        }
        result
    }

    fn specificity(a: u32, b: u32, c: u32) -> u32 {
        a << 20 | b << 10 | c
    }

    #[test]
    fn test_empty() {
        let list = SelectorList::parse(&DummyParser::default(), &mut CssParser::new(":empty"));
        assert!(list.is_ok());
    }

    const MATHML: &'static str = "http://www.w3.org/1998/Math/MathML";
    const SVG: &'static str = "http://www.w3.org/2000/svg";

    #[test]
    fn test_parsing() {
        assert_eq!(parse(""), Err(())) ;
        assert_eq!(parse(":lang(4)"), Err(())) ;
        assert_eq!(parse(":lang(en US)"), Err(())) ;
        assert_eq!(parse("EeÉ"), Ok(SelectorList(vec!(Selector {
            inner: SelectorInner::from_vec(vec!(Component::LocalName(LocalName {
                    name: DummyAtom::from("EeÉ"),
                    lower_name: DummyAtom::from("eeÉ") })),
            ),
            pseudo_element: None,
            specificity: specificity(0, 0, 1),
        }))));
        assert_eq!(parse(".foo:lang(en-US)"), Ok(SelectorList(vec!(Selector {
            inner: SelectorInner::from_vec(vec![
                    Component::Class(DummyAtom::from("foo")),
                    Component::NonTSPseudoClass(PseudoClass::Lang("en-US".to_owned()))
            ]),
            pseudo_element: None,
            specificity: specificity(0, 2, 0),
        }))));
        assert_eq!(parse("#bar"), Ok(SelectorList(vec!(Selector {
            inner: SelectorInner::from_vec(vec!(Component::ID(DummyAtom::from("bar")))),
            pseudo_element: None,
            specificity: specificity(1, 0, 0),
        }))));
        assert_eq!(parse("e.foo#bar"), Ok(SelectorList(vec!(Selector {
            inner: SelectorInner::from_vec(vec!(Component::LocalName(LocalName {
                                            name: DummyAtom::from("e"),
                                            lower_name: DummyAtom::from("e") }),
                                       Component::Class(DummyAtom::from("foo")),
                                       Component::ID(DummyAtom::from("bar")))),
            pseudo_element: None,
            specificity: specificity(1, 1, 1),
        }))));
        assert_eq!(parse("e.foo #bar"), Ok(SelectorList(vec!(Selector {
            inner: SelectorInner::from_vec(vec!(
                           Component::LocalName(LocalName {
                               name: DummyAtom::from("e"),
                               lower_name: DummyAtom::from("e")
                           }),
                           Component::Class(DummyAtom::from("foo")),
                           Component::Combinator(Combinator::Descendant),
                           Component::ID(DummyAtom::from("bar")),
                       )),
            pseudo_element: None,
            specificity: specificity(1, 1, 1),
        }))));
        // Default namespace does not apply to attribute selectors
        // https://github.com/mozilla/servo/pull/1652
        let mut parser = DummyParser::default();
        assert_eq!(parse_ns("[Foo]", &parser), Ok(SelectorList(vec!(Selector {
            inner: SelectorInner::from_vec(vec!(
                Component::AttrExists(AttrSelector {
                    name: DummyAtom::from("Foo"),
                    lower_name: DummyAtom::from("foo"),
                    namespace: NamespaceConstraint::Specific(Namespace {
                        prefix: None,
                        url: "".into(),
                    }) }))),
            pseudo_element: None,
            specificity: specificity(0, 1, 0),
        }))));
        assert_eq!(parse_ns("svg|circle", &parser), Err(()));
        parser.ns_prefixes.insert(DummyAtom("svg".into()), DummyAtom(SVG.into()));
        assert_eq!(parse_ns("svg|circle", &parser), Ok(SelectorList(vec![Selector {
            inner: SelectorInner::from_vec(
                vec![
                    Component::Namespace(Namespace {
                        prefix: Some(DummyAtom("svg".into())),
                        url: SVG.into(),
                    }),
                    Component::LocalName(LocalName {
                        name: DummyAtom::from("circle"),
                        lower_name: DummyAtom::from("circle"),
                    })
                ]),
            pseudo_element: None,
            specificity: specificity(0, 0, 1),
        }])));
        // Default namespace does not apply to attribute selectors
        // https://github.com/mozilla/servo/pull/1652
        // but it does apply to implicit type selectors
        // https://github.com/servo/rust-selectors/pull/82
        parser.default_ns = Some(MATHML.into());
        assert_eq!(parse_ns("[Foo]", &parser), Ok(SelectorList(vec!(Selector {
            inner: SelectorInner::from_vec(
                vec![
                    Component::Namespace(Namespace {
                        prefix: None,
                        url: MATHML.into(),
                    }),
                    Component::AttrExists(AttrSelector {
                        name: DummyAtom::from("Foo"),
                        lower_name: DummyAtom::from("foo"),
                        namespace: NamespaceConstraint::Specific(Namespace {
                            prefix: None,
                            url: "".into(),
                        }),
                    }),
                ]),
            pseudo_element: None,
            specificity: specificity(0, 1, 0),
        }))));
        // Default namespace does apply to type selectors
        assert_eq!(parse_ns("e", &parser), Ok(SelectorList(vec!(Selector {
            inner: SelectorInner::from_vec(
                vec!(
                    Component::Namespace(Namespace {
                        prefix: None,
                        url: MATHML.into(),
                    }),
                    Component::LocalName(LocalName {
                        name: DummyAtom::from("e"),
                        lower_name: DummyAtom::from("e") }),
                )),
            pseudo_element: None,
            specificity: specificity(0, 0, 1),
        }))));
        assert_eq!(parse("[attr |= \"foo\"]"), Ok(SelectorList(vec![Selector {
            inner: SelectorInner::from_vec(
                vec![
                    Component::AttrDashMatch(AttrSelector {
                        name: DummyAtom::from("attr"),
                        lower_name: DummyAtom::from("attr"),
                        namespace: NamespaceConstraint::Specific(Namespace {
                            prefix: None,
                            url: "".into(),
                        }),
                    }, DummyAtom::from("foo"))
                ]),
            pseudo_element: None,
            specificity: specificity(0, 1, 0),
        }])));
        // https://github.com/mozilla/servo/issues/1723
        assert_eq!(parse("::before"), Ok(SelectorList(vec!(Selector {
            inner: SelectorInner::from_vec(vec![]),
            pseudo_element: Some(PseudoElement::Before),
            specificity: specificity(0, 0, 1),
        }))));
        // https://github.com/servo/servo/issues/15335
        assert_eq!(parse(":: before"), Err(()));
        assert_eq!(parse("div ::after"), Ok(SelectorList(vec!(Selector {
            inner: SelectorInner::from_vec(
                vec![
                     Component::LocalName(LocalName {
                        name: DummyAtom::from("div"),
                        lower_name: DummyAtom::from("div") }),
                    Component::Combinator(Combinator::Descendant),
                ]),
            pseudo_element: Some(PseudoElement::After),
            specificity: specificity(0, 0, 2),
        }))));
        assert_eq!(parse("#d1 > .ok"), Ok(SelectorList(vec![Selector {
            inner: SelectorInner::from_vec(
                vec![
                    Component::ID(DummyAtom::from("d1")),
                    Component::Combinator(Combinator::Child),
                    Component::Class(DummyAtom::from("ok")),
                ]),
            pseudo_element: None,
            specificity: (1 << 20) + (1 << 10) + (0 << 0),
        }])));
        parser.default_ns = None;
        assert_eq!(parse(":not(#provel.old)"), Err(()));
        assert_eq!(parse(":not(#provel > old)"), Err(()));
        assert!(parse("table[rules]:not([rules = \"none\"]):not([rules = \"\"])").is_ok());
        assert_eq!(parse(":not(#provel)"), Ok(SelectorList(vec!(Selector {
            inner: SelectorInner::from_vec(vec!(Component::Negation(
                vec![
                    Component::ID(DummyAtom::from("provel")),
                ].into_boxed_slice()
            ))),
            pseudo_element: None,
            specificity: specificity(1, 0, 0),
        }))));
        assert_eq!(parse_ns(":not(svg|circle)", &parser), Ok(SelectorList(vec!(Selector {
            inner: SelectorInner::from_vec(vec!(Component::Negation(
                vec![
                    Component::Namespace(Namespace {
                        prefix: Some(DummyAtom("svg".into())),
                        url: SVG.into(),
                    }),
                    Component::LocalName(LocalName {
                        name: DummyAtom::from("circle"),
                        lower_name: DummyAtom::from("circle")
                    }),
                ].into_boxed_slice()
            ))),
            pseudo_element: None,
            specificity: specificity(0, 0, 1),
        }))));
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
    }
}
