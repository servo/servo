/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Token, Parser as CssParser, parse_nth, ToCss, serialize_identifier, CssStringWriter};
use precomputed_hash::PrecomputedHash;
use std::ascii::AsciiExt;
use std::borrow::{Borrow, Cow};
use std::cmp;
use std::fmt::{self, Display, Debug, Write};
use std::hash::Hash;
use std::ops::Add;
use std::sync::Arc;
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
        pub trait SelectorImpl: Sized {
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

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Selector<Impl: SelectorImpl> {
    pub complex_selector: Arc<ComplexSelector<Impl>>,
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
        self.complex_selector.visit(visitor)
    }
}

impl<Impl: SelectorImpl> SelectorMethods for Arc<ComplexSelector<Impl>> {
    type Impl = Impl;

    fn visit<V>(&self, visitor: &mut V) -> bool
        where V: SelectorVisitor<Impl = Impl>,
    {
        let mut current = self;
        let mut combinator = None;
        loop {
            if !visitor.visit_complex_selector(current, combinator) {
                return false;
            }

            for selector in &current.compound_selector {
                if !selector.visit(visitor) {
                    return false;
                }
            }

            match current.next {
                Some((ref next, next_combinator)) => {
                    current = next;
                    combinator = Some(next_combinator);
                }
                None => break,
            }
        }

        true
    }
}

impl<Impl: SelectorImpl> SelectorMethods for SimpleSelector<Impl> {
    type Impl = Impl;

    fn visit<V>(&self, visitor: &mut V) -> bool
        where V: SelectorVisitor<Impl = Impl>,
    {
        use self::SimpleSelector::*;
        if !visitor.visit_simple_selector(self) {
            return false;
        }

        match *self {
            Negation(ref negated) => {
                for selector in negated {
                    if !selector.visit(visitor) {
                        return false;
                    }
                }
            }
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

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ComplexSelector<Impl: SelectorImpl> {
    pub compound_selector: Vec<SimpleSelector<Impl>>,
    pub next: Option<(Arc<ComplexSelector<Impl>>, Combinator)>,  // c.next is left of c
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub enum Combinator {
    Child,  //  >
    Descendant,  // space
    NextSibling,  // +
    LaterSibling,  // ~
}

#[derive(Eq, PartialEq, Clone, Hash)]
pub enum SimpleSelector<Impl: SelectorImpl> {
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
    Negation(Vec<Arc<ComplexSelector<Impl>>>),
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

impl<Impl: SelectorImpl> Debug for ComplexSelector<Impl> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.to_css(f) }
}
impl<Impl: SelectorImpl> Debug for SimpleSelector<Impl> {
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
        self.complex_selector.to_css(dest)?;
        if let Some(ref pseudo) = self.pseudo_element {
            pseudo.to_css(dest)?;
        }
        Ok(())
    }
}

impl<Impl: SelectorImpl> ToCss for ComplexSelector<Impl> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if let Some((ref next, ref combinator)) = self.next {
            next.to_css(dest)?;
            combinator.to_css(dest)?;
        }
        for simple in &self.compound_selector {
            simple.to_css(dest)?;
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

impl<Impl: SelectorImpl> ToCss for SimpleSelector<Impl> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        use self::SimpleSelector::*;
        match *self {
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
            Negation(ref args) => {
                dest.write_str(":not(")?;
                let mut args = args.iter();
                let first = args.next().unwrap();
                first.to_css(dest)?;
                for arg in args {
                    dest.write_str(", ")?;
                    arg.to_css(dest)?;
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

fn complex_selector_specificity<Impl>(mut selector: &ComplexSelector<Impl>)
                                      -> Specificity
                                      where Impl: SelectorImpl {
    fn compound_selector_specificity<Impl>(compound_selector: &[SimpleSelector<Impl>],
                                           specificity: &mut Specificity)
                                           where Impl: SelectorImpl {
        for simple_selector in compound_selector.iter() {
            match *simple_selector {
                SimpleSelector::LocalName(..) =>
                    specificity.element_selectors += 1,
                SimpleSelector::ID(..) =>
                    specificity.id_selectors += 1,
                SimpleSelector::Class(..) |
                SimpleSelector::AttrExists(..) |
                SimpleSelector::AttrEqual(..) |
                SimpleSelector::AttrIncludes(..) |
                SimpleSelector::AttrDashMatch(..) |
                SimpleSelector::AttrPrefixMatch(..) |
                SimpleSelector::AttrSubstringMatch(..) |
                SimpleSelector::AttrSuffixMatch(..) |

                SimpleSelector::AttrIncludesNeverMatch(..) |
                SimpleSelector::AttrPrefixNeverMatch(..) |
                SimpleSelector::AttrSubstringNeverMatch(..) |
                SimpleSelector::AttrSuffixNeverMatch(..) |

                SimpleSelector::FirstChild | SimpleSelector::LastChild |
                SimpleSelector::OnlyChild | SimpleSelector::Root |
                SimpleSelector::Empty |
                SimpleSelector::NthChild(..) |
                SimpleSelector::NthLastChild(..) |
                SimpleSelector::NthOfType(..) |
                SimpleSelector::NthLastOfType(..) |
                SimpleSelector::FirstOfType | SimpleSelector::LastOfType |
                SimpleSelector::OnlyOfType |
                SimpleSelector::NonTSPseudoClass(..) =>
                    specificity.class_like_selectors += 1,

                SimpleSelector::Namespace(..) => (),
                SimpleSelector::Negation(ref negated) => {
                    let max =
                        negated.iter().map(|s| complex_selector_specificity(&s))
                               .max().unwrap();
                    *specificity = *specificity + max;
                }
            }
        }
    }

    let mut specificity = Default::default();
    compound_selector_specificity(&selector.compound_selector,
                              &mut specificity);
    loop {
        match selector.next {
            None => break,
            Some((ref next_selector, _)) => {
                selector = &**next_selector;
                compound_selector_specificity(&selector.compound_selector,
                                          &mut specificity)
            }
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
        complex_selector: Arc::new(complex),
        pseudo_element: pseudo_element,
    })
}

fn parse_complex_selector_and_pseudo_element<P, Impl>(
        parser: &P,
        input: &mut CssParser)
        -> Result<(ComplexSelector<Impl>, Option<Impl::PseudoElement>), ()>
    where P: Parser<Impl=Impl>, Impl: SelectorImpl
{
    let (first, mut pseudo_element) = parse_compound_selector(parser, input)?;
    let mut complex = ComplexSelector { compound_selector: first, next: None };

    'outer_loop: while pseudo_element.is_none() {
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
        let (compound_selector, pseudo) = parse_compound_selector(parser, input)?;
        complex = ComplexSelector {
            compound_selector: compound_selector,
            next: Some((Arc::new(complex), combinator))
        };
        pseudo_element = pseudo;
    }

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
fn parse_type_selector<P, Impl>(parser: &P, input: &mut CssParser)
                       -> Result<Option<Vec<SimpleSelector<Impl>>>, ()>
    where P: Parser<Impl=Impl>, Impl: SelectorImpl
{
    match parse_qualified_name(parser, input, /* in_attr_selector = */ false)? {
        None => Ok(None),
        Some((namespace, local_name)) => {
            let mut compound_selector = vec!();
            match namespace {
                NamespaceConstraint::Specific(ns) => {
                    compound_selector.push(SimpleSelector::Namespace(ns))
                },
                NamespaceConstraint::Any => (),
            }
            match local_name {
                Some(name) => {
                    compound_selector.push(SimpleSelector::LocalName(LocalName {
                        lower_name: from_ascii_lowercase(&name),
                        name: from_cow_str(name),
                    }))
                }
                None => (),
            }
            Ok(Some(compound_selector))
        }
    }
}

#[derive(Debug)]
enum SimpleSelectorParseResult<Impl: SelectorImpl> {
    SimpleSelector(SimpleSelector<Impl>),
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
                                     -> Result<SimpleSelector<Impl>, ()>
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
        Err(()) => Ok(SimpleSelector::AttrExists(attr)),

        // [foo=bar]
        Ok(Token::Delim('=')) => {
            let value = input.expect_ident_or_string()?;
            let flags = parse_attribute_flags(input)?;
            Ok(SimpleSelector::AttrEqual(attr, from_cow_str(value), flags))
        }
        // [foo~=bar]
        Ok(Token::IncludeMatch) => {
            let value = input.expect_ident_or_string()?;
            if value.is_empty() || value.contains(SELECTOR_WHITESPACE) {
                Ok(SimpleSelector::AttrIncludesNeverMatch(attr, from_cow_str(value)))
            } else {
                Ok(SimpleSelector::AttrIncludes(attr, from_cow_str(value)))
            }
        }
        // [foo|=bar]
        Ok(Token::DashMatch) => {
            let value = input.expect_ident_or_string()?;
            Ok(SimpleSelector::AttrDashMatch(attr, from_cow_str(value)))
        }
        // [foo^=bar]
        Ok(Token::PrefixMatch) => {
            let value = input.expect_ident_or_string()?;
            if value.is_empty() {
                Ok(SimpleSelector::AttrPrefixNeverMatch(attr, from_cow_str(value)))
            } else {
                Ok(SimpleSelector::AttrPrefixMatch(attr, from_cow_str(value)))
            }
        }
        // [foo*=bar]
        Ok(Token::SubstringMatch) => {
            let value = input.expect_ident_or_string()?;
            if value.is_empty() {
                Ok(SimpleSelector::AttrSubstringNeverMatch(attr, from_cow_str(value)))
            } else {
                Ok(SimpleSelector::AttrSubstringMatch(attr, from_cow_str(value)))
            }
        }
        // [foo$=bar]
        Ok(Token::SuffixMatch) => {
            let value = input.expect_ident_or_string()?;
            if value.is_empty() {
                Ok(SimpleSelector::AttrSuffixNeverMatch(attr, from_cow_str(value)))
            } else {
                Ok(SimpleSelector::AttrSuffixMatch(attr, from_cow_str(value)))
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
                           -> Result<SimpleSelector<Impl>, ()>
    where P: Parser<Impl=Impl>, Impl: SelectorImpl
{
    input.parse_comma_separated(|input| ComplexSelector::parse(parser, input).map(Arc::new))
         .map(SimpleSelector::Negation)
}

/// simple_selector_sequence
/// : [ type_selector | universal ] [ HASH | class | attrib | pseudo | negation ]*
/// | [ HASH | class | attrib | pseudo | negation ]+
///
/// `Err(())` means invalid selector
fn parse_compound_selector<P, Impl>(
    parser: &P,
    input: &mut CssParser)
    -> Result<(Vec<SimpleSelector<Impl>>, Option<Impl::PseudoElement>), ()>
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
    let mut compound_selector = match parse_type_selector(parser, input)? {
        None => {
            match parser.default_namespace() {
                // If there was no explicit type selector, but there is a
                // default namespace, there is an implicit "<defaultns>|*" type
                // selector.
                Some(url) => vec![SimpleSelector::Namespace(Namespace {
                    prefix: None,
                    url: url
                })],
                None => vec![],
            }
        }
        Some(s) => { empty = false; s }
    };

    let mut pseudo_element = None;
    loop {
        match parse_one_simple_selector(parser, input, /* inside_negation = */ false)? {
            None => break,
            Some(SimpleSelectorParseResult::SimpleSelector(s)) => {
                compound_selector.push(s);
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
        Ok((compound_selector, pseudo_element))
    }
}

fn parse_functional_pseudo_class<P, Impl>(parser: &P,
                                          input: &mut CssParser,
                                          name: Cow<str>,
                                          inside_negation: bool)
                                          -> Result<SimpleSelector<Impl>, ()>
    where P: Parser<Impl=Impl>, Impl: SelectorImpl
{
    match_ignore_ascii_case! { &name,
        "nth-child" => return parse_nth_pseudo_class(input, SimpleSelector::NthChild),
        "nth-of-type" => return parse_nth_pseudo_class(input, SimpleSelector::NthOfType),
        "nth-last-child" => return parse_nth_pseudo_class(input, SimpleSelector::NthLastChild),
        "nth-last-of-type" => return parse_nth_pseudo_class(input, SimpleSelector::NthLastOfType),
        "not" => {
            if inside_negation {
                return Err(())
            }
            return parse_negation(parser, input)
        },
        _ => {}
    }
    P::parse_non_ts_functional_pseudo_class(parser, name, input)
        .map(SimpleSelector::NonTSPseudoClass)
}


fn parse_nth_pseudo_class<Impl, F>(input: &mut CssParser, selector: F)
                                   -> Result<SimpleSelector<Impl>, ()>
where Impl: SelectorImpl, F: FnOnce(i32, i32) -> SimpleSelector<Impl> {
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
            let id = SimpleSelector::ID(from_cow_str(id));
            Ok(Some(SimpleSelectorParseResult::SimpleSelector(id)))
        }
        Ok(Token::Delim('.')) => {
            match input.next_including_whitespace() {
                Ok(Token::Ident(class)) => {
                    let class = SimpleSelector::Class(from_cow_str(class));
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

fn parse_simple_pseudo_class<P, Impl>(parser: &P, name: Cow<str>) -> Result<SimpleSelector<Impl>, ()>
    where P: Parser<Impl=Impl>, Impl: SelectorImpl
{
    (match_ignore_ascii_case! { &name,
        "first-child" => Ok(SimpleSelector::FirstChild),
        "last-child"  => Ok(SimpleSelector::LastChild),
        "only-child"  => Ok(SimpleSelector::OnlyChild),
        "root" => Ok(SimpleSelector::Root),
        "empty" => Ok(SimpleSelector::Empty),
        "first-of-type" => Ok(SimpleSelector::FirstOfType),
        "last-of-type"  => Ok(SimpleSelector::LastOfType),
        "only-of-type"  => Ok(SimpleSelector::OnlyOfType),
        _ => Err(())
    }).or_else(|()| {
        P::parse_non_ts_pseudo_class(parser, name)
            .map(SimpleSelector::NonTSPseudoClass)
    })
}

// NB: pub module in order to access the DummyParser
#[cfg(test)]
pub mod tests {
    use cssparser::{Parser as CssParser, ToCss, serialize_identifier};
    use std::borrow::Cow;
    use std::collections::HashMap;
    use std::fmt;
    use std::sync::Arc;
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

        fn visit<V>(&self, visitor: &mut V) -> bool
            where V: SelectorVisitor<Impl = Self::Impl> { true }
    }

    #[derive(PartialEq, Debug)]
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
            complex_selector: Arc::new(ComplexSelector {
                compound_selector: vec!(SimpleSelector::LocalName(LocalName {
                    name: DummyAtom::from("EeÉ"),
                    lower_name: DummyAtom::from("eeÉ") })),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(0, 0, 1),
        }))));
        assert_eq!(parse(".foo:lang(en-US)"), Ok(SelectorList(vec!(Selector {
            complex_selector: Arc::new(ComplexSelector {
                compound_selector: vec![
                    SimpleSelector::Class(DummyAtom::from("foo")),
                    SimpleSelector::NonTSPseudoClass(PseudoClass::Lang("en-US".to_owned()))
                ],
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(0, 2, 0),
        }))));
        assert_eq!(parse("#bar"), Ok(SelectorList(vec!(Selector {
            complex_selector: Arc::new(ComplexSelector {
                compound_selector: vec!(SimpleSelector::ID(DummyAtom::from("bar"))),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(1, 0, 0),
        }))));
        assert_eq!(parse("e.foo#bar"), Ok(SelectorList(vec!(Selector {
            complex_selector: Arc::new(ComplexSelector {
                compound_selector: vec!(SimpleSelector::LocalName(LocalName {
                                            name: DummyAtom::from("e"),
                                            lower_name: DummyAtom::from("e") }),
                                       SimpleSelector::Class(DummyAtom::from("foo")),
                                       SimpleSelector::ID(DummyAtom::from("bar"))),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(1, 1, 1),
        }))));
        assert_eq!(parse("e.foo #bar"), Ok(SelectorList(vec!(Selector {
            complex_selector: Arc::new(ComplexSelector {
                compound_selector: vec!(SimpleSelector::ID(DummyAtom::from("bar"))),
                next: Some((Arc::new(ComplexSelector {
                    compound_selector: vec!(SimpleSelector::LocalName(LocalName {
                                                name: DummyAtom::from("e"),
                                                lower_name: DummyAtom::from("e") }),
                                           SimpleSelector::Class(DummyAtom::from("foo"))),
                    next: None,
                }), Combinator::Descendant)),
            }),
            pseudo_element: None,
            specificity: specificity(1, 1, 1),
        }))));
        // Default namespace does not apply to attribute selectors
        // https://github.com/mozilla/servo/pull/1652
        let mut parser = DummyParser::default();
        assert_eq!(parse_ns("[Foo]", &parser), Ok(SelectorList(vec!(Selector {
            complex_selector: Arc::new(ComplexSelector {
                compound_selector: vec!(SimpleSelector::AttrExists(AttrSelector {
                    name: DummyAtom::from("Foo"),
                    lower_name: DummyAtom::from("foo"),
                    namespace: NamespaceConstraint::Specific(Namespace {
                        prefix: None,
                        url: "".into(),
                    }),
                })),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(0, 1, 0),
        }))));
        assert_eq!(parse_ns("svg|circle", &parser), Err(()));
        parser.ns_prefixes.insert(DummyAtom("svg".into()), DummyAtom(SVG.into()));
        assert_eq!(parse_ns("svg|circle", &parser), Ok(SelectorList(vec![Selector {
            complex_selector: Arc::new(ComplexSelector {
                compound_selector: vec![
                    SimpleSelector::Namespace(Namespace {
                        prefix: Some(DummyAtom("svg".into())),
                        url: SVG.into(),
                    }),
                    SimpleSelector::LocalName(LocalName {
                        name: DummyAtom::from("circle"),
                        lower_name: DummyAtom::from("circle"),
                    })
                ],
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(0, 0, 1),
        }])));
        // Default namespace does not apply to attribute selectors
        // https://github.com/mozilla/servo/pull/1652
        // but it does apply to implicit type selectors
        // https://github.com/servo/rust-selectors/pull/82
        parser.default_ns = Some(MATHML.into());
        assert_eq!(parse_ns("[Foo]", &parser), Ok(SelectorList(vec!(Selector {
            complex_selector: Arc::new(ComplexSelector {
                compound_selector: vec![
                    SimpleSelector::Namespace(Namespace {
                        prefix: None,
                        url: MATHML.into(),
                    }),
                    SimpleSelector::AttrExists(AttrSelector {
                        name: DummyAtom::from("Foo"),
                        lower_name: DummyAtom::from("foo"),
                        namespace: NamespaceConstraint::Specific(Namespace {
                            prefix: None,
                            url: "".into(),
                        }),
                    }),
                ],
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(0, 1, 0),
        }))));
        // Default namespace does apply to type selectors
        assert_eq!(parse_ns("e", &parser), Ok(SelectorList(vec!(Selector {
            complex_selector: Arc::new(ComplexSelector {
                compound_selector: vec!(
                    SimpleSelector::Namespace(Namespace {
                        prefix: None,
                        url: MATHML.into(),
                    }),
                    SimpleSelector::LocalName(LocalName {
                        name: DummyAtom::from("e"),
                        lower_name: DummyAtom::from("e") }),
                ),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(0, 0, 1),
        }))));
        assert_eq!(parse("[attr |= \"foo\"]"), Ok(SelectorList(vec![Selector {
            complex_selector: Arc::new(ComplexSelector {
                compound_selector: vec![
                    SimpleSelector::AttrDashMatch(AttrSelector {
                        name: DummyAtom::from("attr"),
                        lower_name: DummyAtom::from("attr"),
                        namespace: NamespaceConstraint::Specific(Namespace {
                            prefix: None,
                            url: "".into(),
                        }),
                    }, DummyAtom::from("foo"))
                ],
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(0, 1, 0),
        }])));
        // https://github.com/mozilla/servo/issues/1723
        assert_eq!(parse("::before"), Ok(SelectorList(vec!(Selector {
            complex_selector: Arc::new(ComplexSelector {
                compound_selector: vec!(),
                next: None,
            }),
            pseudo_element: Some(PseudoElement::Before),
            specificity: specificity(0, 0, 1),
        }))));
        // https://github.com/servo/servo/issues/15335
        assert_eq!(parse(":: before"), Err(()));
        assert_eq!(parse("div ::after"), Ok(SelectorList(vec!(Selector {
            complex_selector: Arc::new(ComplexSelector {
                compound_selector: vec!(),
                next: Some((Arc::new(ComplexSelector {
                    compound_selector: vec!(SimpleSelector::LocalName(LocalName {
                        name: DummyAtom::from("div"),
                        lower_name: DummyAtom::from("div") })),
                    next: None,
                }), Combinator::Descendant)),
            }),
            pseudo_element: Some(PseudoElement::After),
            specificity: specificity(0, 0, 2),
        }))));
        assert_eq!(parse("#d1 > .ok"), Ok(SelectorList(vec![Selector {
            complex_selector: Arc::new(ComplexSelector {
                compound_selector: vec![
                    SimpleSelector::Class(DummyAtom::from("ok")),
                ],
                next: Some((Arc::new(ComplexSelector {
                    compound_selector: vec![
                        SimpleSelector::ID(DummyAtom::from("d1")),
                    ],
                    next: None,
                }), Combinator::Child)),
            }),
            pseudo_element: None,
            specificity: (1 << 20) + (1 << 10) + (0 << 0),
        }])));
        assert_eq!(parse(":not(.babybel, #provel.old)"), Ok(SelectorList(vec!(Selector {
            complex_selector: Arc::new(ComplexSelector {
                compound_selector: vec!(SimpleSelector::Negation(
                    vec!(
                        Arc::new(ComplexSelector {
                            compound_selector: vec!(SimpleSelector::Class(DummyAtom::from("babybel"))),
                            next: None
                        }),
                        Arc::new(ComplexSelector {
                            compound_selector: vec!(
                                SimpleSelector::ID(DummyAtom::from("provel")),
                                SimpleSelector::Class(DummyAtom::from("old")),
                            ),
                            next: None
                        }),
                    )
                )),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(1, 1, 0),
        }))));
    }
}
