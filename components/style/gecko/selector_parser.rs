/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Gecko-specific bits for selector-parsing.

use cssparser::{BasicParseError, BasicParseErrorKind, Parser, ToCss, Token, CowRcStr, SourceLocation};
use element_state::{DocumentState, ElementState};
use gecko_bindings::structs::{self, CSSPseudoClassType};
use gecko_bindings::structs::RawServoSelectorList;
use gecko_bindings::sugar::ownership::{HasBoxFFI, HasFFI, HasSimpleFFI};
use invalidation::element::document_state::InvalidationMatchingData;
use selector_parser::{Direction, SelectorParser};
use selectors::SelectorList;
use selectors::parser::{self as selector_parser, Selector, Visit, SelectorParseErrorKind};
use selectors::visitor::SelectorVisitor;
use std::fmt;
use string_cache::{Atom, Namespace, WeakAtom, WeakNamespace};
use style_traits::{ParseError, StyleParseErrorKind, ToCss as ToCss_};

pub use gecko::pseudo_element::{PseudoElement, EAGER_PSEUDOS, EAGER_PSEUDO_COUNT, PSEUDO_COUNT};
pub use gecko::snapshot::SnapshotMap;

bitflags! {
    // See NonTSPseudoClass::is_enabled_in()
    struct NonTSPseudoClassFlag: u8 {
        const PSEUDO_CLASS_ENABLED_IN_UA_SHEETS = 1 << 0;
        const PSEUDO_CLASS_ENABLED_IN_CHROME = 1 << 1;
        const PSEUDO_CLASS_ENABLED_IN_UA_SHEETS_AND_CHROME =
            NonTSPseudoClassFlag::PSEUDO_CLASS_ENABLED_IN_UA_SHEETS.bits |
            NonTSPseudoClassFlag::PSEUDO_CLASS_ENABLED_IN_CHROME.bits;
    }
}

/// The type used for storing pseudo-class string arguments.
pub type PseudoClassStringArg = Box<[u16]>;

macro_rules! pseudo_class_name {
    (bare: [$(($css:expr, $name:ident, $gecko_type:tt, $state:tt, $flags:tt),)*],
     string: [$(($s_css:expr, $s_name:ident, $s_gecko_type:tt, $s_state:tt, $s_flags:tt),)*]) => {
        /// Our representation of a non tree-structural pseudo-class.
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub enum NonTSPseudoClass {
            $(
                #[doc = $css]
                $name,
            )*
            $(
                #[doc = $s_css]
                $s_name(PseudoClassStringArg),
            )*
            /// The `:dir` pseudo-class.
            Dir(Box<Direction>),
            /// The non-standard `:-moz-any` pseudo-class.
            ///
            /// TODO(emilio): We disallow combinators and pseudos here, so we
            /// should use SimpleSelector instead
            MozAny(Box<[Selector<SelectorImpl>]>),
            /// The non-standard `:-moz-locale-dir` pseudo-class.
            MozLocaleDir(Box<Direction>),
        }
    }
}
apply_non_ts_list!(pseudo_class_name);

impl ToCss for NonTSPseudoClass {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        use cssparser::CssStringWriter;
        use fmt::Write;
        macro_rules! pseudo_class_serialize {
            (bare: [$(($css:expr, $name:ident, $gecko_type:tt, $state:tt, $flags:tt),)*],
             string: [$(($s_css:expr, $s_name:ident, $s_gecko_type:tt, $s_state:tt, $s_flags:tt),)*]) => {
                match *self {
                    $(NonTSPseudoClass::$name => concat!(":", $css),)*
                    $(NonTSPseudoClass::$s_name(ref s) => {
                        dest.write_str(concat!(":", $s_css, "("))?;
                        {
                            // FIXME(emilio): Avoid the extra allocation!
                            let mut css = CssStringWriter::new(dest);

                            // Discount the null char in the end from the
                            // string.
                            css.write_str(&String::from_utf16(&s[..s.len() - 1]).unwrap())?;
                        }
                        return dest.write_str(")")
                    }, )*
                    NonTSPseudoClass::MozLocaleDir(ref dir) => {
                        dest.write_str(":-moz-locale-dir(")?;
                        dir.to_css(dest)?;
                        return dest.write_char(')')
                    },
                    NonTSPseudoClass::Dir(ref dir) => {
                        dest.write_str(":dir(")?;
                        dir.to_css(dest)?;
                        return dest.write_char(')')
                    },
                    NonTSPseudoClass::MozAny(ref selectors) => {
                        dest.write_str(":-moz-any(")?;
                        let mut iter = selectors.iter();
                        let first = iter.next().expect(":-moz-any must have at least 1 selector");
                        first.to_css(dest)?;
                        for selector in iter {
                            dest.write_str(", ")?;
                            selector.to_css(dest)?;
                        }
                        return dest.write_str(")")
                    }
                }
            }
        }
        let ser = apply_non_ts_list!(pseudo_class_serialize);
        dest.write_str(ser)
    }
}

impl Visit for NonTSPseudoClass {
    type Impl = SelectorImpl;

    fn visit<V>(&self, visitor: &mut V) -> bool
        where V: SelectorVisitor<Impl = Self::Impl>,
    {
        if let NonTSPseudoClass::MozAny(ref selectors) = *self {
            for selector in selectors.iter() {
                if !selector.visit(visitor) {
                    return false;
                }
            }
        }

        true
    }
}


impl NonTSPseudoClass {
    /// Returns true if this pseudo-class has any of the given flags set.
    fn has_any_flag(&self, flags: NonTSPseudoClassFlag) -> bool {
        macro_rules! check_flag {
            (_) => (false);
            ($flags:ident) => (NonTSPseudoClassFlag::$flags.intersects(flags));
        }
        macro_rules! pseudo_class_check_is_enabled_in {
            (bare: [$(($css:expr, $name:ident, $gecko_type:tt, $state:tt, $flags:tt),)*],
            string: [$(($s_css:expr, $s_name:ident, $s_gecko_type:tt, $s_state:tt, $s_flags:tt),)*]) => {
                match *self {
                    $(NonTSPseudoClass::$name => check_flag!($flags),)*
                    $(NonTSPseudoClass::$s_name(..) => check_flag!($s_flags),)*
                    // TODO(emilio): Maybe -moz-locale-dir shouldn't be
                    // content-exposed.
                    NonTSPseudoClass::MozLocaleDir(_) |
                    NonTSPseudoClass::Dir(_) |
                    NonTSPseudoClass::MozAny(_) => false,
                }
            }
        }
        apply_non_ts_list!(pseudo_class_check_is_enabled_in)
    }

    /// Returns whether the pseudo-class is enabled in content sheets.
    fn is_enabled_in_content(&self) -> bool {
        use gecko_bindings::structs::mozilla;
        match self {
            // For pseudo-classes with pref, the availability in content
            // depends on the pref.
            &NonTSPseudoClass::Fullscreen =>
                unsafe { mozilla::StylePrefs_sUnprefixedFullscreenApiEnabled },
            // Otherwise, a pseudo-class is enabled in content when it
            // doesn't have any enabled flag.
            _ => !self.has_any_flag(NonTSPseudoClassFlag::PSEUDO_CLASS_ENABLED_IN_UA_SHEETS_AND_CHROME),
        }
    }

    /// <https://drafts.csswg.org/selectors-4/#useraction-pseudos>
    ///
    /// We intentionally skip the link-related ones.
    pub fn is_safe_user_action_state(&self) -> bool {
        matches!(*self, NonTSPseudoClass::Hover |
                        NonTSPseudoClass::Active |
                        NonTSPseudoClass::Focus)
    }

    /// Get the state flag associated with a pseudo-class, if any.
    pub fn state_flag(&self) -> ElementState {
        macro_rules! flag {
            (_) => (ElementState::empty());
            ($state:ident) => (ElementState::$state);
        }
        macro_rules! pseudo_class_state {
            (bare: [$(($css:expr, $name:ident, $gecko_type:tt, $state:tt, $flags:tt),)*],
             string: [$(($s_css:expr, $s_name:ident, $s_gecko_type:tt, $s_state:tt, $s_flags:tt),)*]) => {
                match *self {
                    $(NonTSPseudoClass::$name => flag!($state),)*
                    $(NonTSPseudoClass::$s_name(..) => flag!($s_state),)*
                    NonTSPseudoClass::Dir(..) |
                    NonTSPseudoClass::MozLocaleDir(..) |
                    NonTSPseudoClass::MozAny(..) => ElementState::empty(),
                }
            }
        }
        apply_non_ts_list!(pseudo_class_state)
    }

    /// Get the document state flag associated with a pseudo-class, if any.
    pub fn document_state_flag(&self) -> DocumentState {
        match *self {
            NonTSPseudoClass::MozLocaleDir(..) => DocumentState::NS_DOCUMENT_STATE_RTL_LOCALE,
            NonTSPseudoClass::MozWindowInactive => DocumentState::NS_DOCUMENT_STATE_WINDOW_INACTIVE,
            _ => DocumentState::empty(),
        }
    }

    /// Returns true if the given pseudoclass should trigger style sharing cache
    /// revalidation.
    pub fn needs_cache_revalidation(&self) -> bool {
        self.state_flag().is_empty() &&
        !matches!(*self,
                  // :-moz-any is handled by the revalidation visitor walking
                  // the things inside it; it does not need to cause
                  // revalidation on its own.
                  NonTSPseudoClass::MozAny(_) |
                  // :dir() depends on state only, but doesn't use state_flag
                  // because its semantics don't quite match.  Nevertheless, it
                  // doesn't need cache revalidation, because we already compare
                  // states for elements and candidates.
                  NonTSPseudoClass::Dir(_) |
                  // :-moz-is-html only depends on the state of the document and
                  // the namespace of the element; the former is invariant
                  // across all the elements involved and the latter is already
                  // checked for by our caching precondtions.
                  NonTSPseudoClass::MozIsHTML |
                  // :-moz-placeholder is parsed but never matches.
                  NonTSPseudoClass::MozPlaceholder |
                  // :-moz-locale-dir and :-moz-window-inactive depend only on
                  // the state of the document, which is invariant across all
                  // the elements involved in a given style cache.
                  NonTSPseudoClass::MozLocaleDir(_) |
                  NonTSPseudoClass::MozWindowInactive |
                  // Similar for the document themes.
                  NonTSPseudoClass::MozLWTheme |
                  NonTSPseudoClass::MozLWThemeBrightText |
                  NonTSPseudoClass::MozLWThemeDarkText
        )
    }

    /// Convert NonTSPseudoClass to Gecko's CSSPseudoClassType.
    pub fn to_gecko_pseudoclasstype(&self) -> Option<CSSPseudoClassType> {
        macro_rules! gecko_type {
            (_) => (None);
            ($gecko_type:ident) =>
                (Some(::gecko_bindings::structs::CSSPseudoClassType::$gecko_type));
        }
        macro_rules! pseudo_class_geckotype {
            (bare: [$(($css:expr, $name:ident, $gecko_type:tt, $state:tt, $flags:tt),)*],
             string: [$(($s_css:expr, $s_name:ident, $s_gecko_type:tt, $s_state:tt, $s_flags:tt),)*]) => {
                match *self {
                    $(NonTSPseudoClass::$name => gecko_type!($gecko_type),)*
                    $(NonTSPseudoClass::$s_name(..) => gecko_type!($s_gecko_type),)*
                    NonTSPseudoClass::MozLocaleDir(_) => gecko_type!(mozLocaleDir),
                    NonTSPseudoClass::Dir(_) => gecko_type!(dir),
                    NonTSPseudoClass::MozAny(_) => gecko_type!(any),
                }
            }
        }
        apply_non_ts_list!(pseudo_class_geckotype)
    }

    /// Returns true if the evaluation of the pseudo-class depends on the
    /// element's attributes.
    pub fn is_attr_based(&self) -> bool {
        matches!(*self,
                 NonTSPseudoClass::MozTableBorderNonzero |
                 NonTSPseudoClass::MozBrowserFrame |
                 NonTSPseudoClass::Lang(..))
    }
}

/// The dummy struct we use to implement our selector parsing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectorImpl;

impl ::selectors::SelectorImpl for SelectorImpl {
    type ExtraMatchingData = InvalidationMatchingData;
    type AttrValue = Atom;
    type Identifier = Atom;
    type ClassName = Atom;
    type LocalName = Atom;
    type NamespacePrefix = Atom;
    type NamespaceUrl = Namespace;
    type BorrowedNamespaceUrl = WeakNamespace;
    type BorrowedLocalName = WeakAtom;

    type PseudoElement = PseudoElement;
    type NonTSPseudoClass = NonTSPseudoClass;

    #[inline]
    fn is_active_or_hover(pseudo_class: &Self::NonTSPseudoClass) -> bool {
        matches!(*pseudo_class, NonTSPseudoClass::Active |
                                NonTSPseudoClass::Hover)
    }
}

impl<'a> SelectorParser<'a> {
    fn is_pseudo_class_enabled(
        &self,
        pseudo_class: &NonTSPseudoClass,
    ) -> bool {
        if pseudo_class.is_enabled_in_content() {
            return true;
        }

        if self.in_user_agent_stylesheet() &&
           pseudo_class.has_any_flag(NonTSPseudoClassFlag::PSEUDO_CLASS_ENABLED_IN_UA_SHEETS)
        {
            return true;
        }

        if self.chrome_rules_enabled() &&
           pseudo_class.has_any_flag(NonTSPseudoClassFlag::PSEUDO_CLASS_ENABLED_IN_CHROME)
        {
            return true;
        }

        return false;
    }
}

impl<'a, 'i> ::selectors::Parser<'i> for SelectorParser<'a> {
    type Impl = SelectorImpl;
    type Error = StyleParseErrorKind<'i>;

    fn parse_slotted(&self) -> bool {
        // NOTE(emilio): Slot assignment and such works per-document, but
        // getting a document around here is not trivial, and it's not worth
        // anyway to handle this in a per-doc basis.
        unsafe { structs::nsContentUtils_sIsShadowDOMEnabled }
    }

    fn pseudo_element_allows_single_colon(name: &str) -> bool {
        // FIXME: -moz-tree check should probably be ascii-case-insensitive.
        ::selectors::parser::is_css2_pseudo_element(name) ||
            name.starts_with("-moz-tree-")
    }

    fn parse_non_ts_pseudo_class(
        &self,
        location: SourceLocation,
        name: CowRcStr<'i>,
    ) -> Result<NonTSPseudoClass, ParseError<'i>> {
        macro_rules! pseudo_class_parse {
            (bare: [$(($css:expr, $name:ident, $gecko_type:tt, $state:tt, $flags:tt),)*],
             string: [$(($s_css:expr, $s_name:ident, $s_gecko_type:tt, $s_state:tt, $s_flags:tt),)*]) => {
                match_ignore_ascii_case! { &name,
                    $($css => NonTSPseudoClass::$name,)*
                    _ => return Err(location.new_custom_error(
                        SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name.clone())
                    ))
                }
            }
        }
        let pseudo_class = apply_non_ts_list!(pseudo_class_parse);
        if self.is_pseudo_class_enabled(&pseudo_class) {
            Ok(pseudo_class)
        } else {
            Err(location.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name)))
        }
    }

    fn parse_non_ts_functional_pseudo_class<'t>(
        &self,
        name: CowRcStr<'i>,
        parser: &mut Parser<'i, 't>,
    ) -> Result<NonTSPseudoClass, ParseError<'i>> {
        macro_rules! pseudo_class_string_parse {
            (bare: [$(($css:expr, $name:ident, $gecko_type:tt, $state:tt, $flags:tt),)*],
             string: [$(($s_css:expr, $s_name:ident, $s_gecko_type:tt, $s_state:tt, $s_flags:tt),)*]) => {
                match_ignore_ascii_case! { &name,
                    $($s_css => {
                        let name = parser.expect_ident_or_string()?;
                        // convert to null terminated utf16 string
                        // since that's what Gecko deals with
                        let utf16: Vec<u16> = name.encode_utf16().chain(Some(0u16)).collect();
                        NonTSPseudoClass::$s_name(utf16.into_boxed_slice())
                    }, )*
                    "-moz-locale-dir" => {
                        NonTSPseudoClass::MozLocaleDir(
                            Box::new(Direction::parse(parser)?)
                        )
                    },
                    "dir" => {
                        NonTSPseudoClass::Dir(
                            Box::new(Direction::parse(parser)?)
                        )
                    },
                    "-moz-any" => {
                        NonTSPseudoClass::MozAny(
                            selector_parser::parse_compound_selector_list(
                                self,
                                parser,
                            )?
                        )
                    }
                    _ => return Err(parser.new_custom_error(
                        SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name.clone())
                    ))
                }
            }
        }
        let pseudo_class = apply_non_ts_list!(pseudo_class_string_parse);
        if self.is_pseudo_class_enabled(&pseudo_class) {
            Ok(pseudo_class)
        } else {
            Err(parser.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name)))
        }
    }

    fn parse_pseudo_element(
        &self,
        location: SourceLocation,
        name: CowRcStr<'i>,
    ) -> Result<PseudoElement, ParseError<'i>> {
        PseudoElement::from_slice(&name, self.in_user_agent_stylesheet())
            .or_else(|| {
                // FIXME: -moz-tree check should probably be
                // ascii-case-insensitive.
                if name.starts_with("-moz-tree-") {
                    PseudoElement::tree_pseudo_element(&name, Box::new([]))
                } else {
                    None
                }
            })
            .ok_or(location.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name.clone())))
    }

    fn parse_functional_pseudo_element<'t>(
        &self,
        name: CowRcStr<'i>,
        parser: &mut Parser<'i, 't>,
    ) -> Result<PseudoElement, ParseError<'i>> {
        // FIXME: -moz-tree check should probably be ascii-case-insensitive.
        if name.starts_with("-moz-tree-") {
            // Tree pseudo-elements can have zero or more arguments, separated
            // by either comma or space.
            let mut args = Vec::new();
            loop {
                let location = parser.current_source_location();
                match parser.next() {
                    Ok(&Token::Ident(ref ident)) => args.push(Atom::from(ident.as_ref())),
                    Ok(&Token::Comma) => {},
                    Ok(t) => return Err(location.new_unexpected_token_error(t.clone())),
                    Err(BasicParseError { kind: BasicParseErrorKind::EndOfInput, .. }) => break,
                    _ => unreachable!("Parser::next() shouldn't return any other error"),
                }
            }
            let args = args.into_boxed_slice();
            if let Some(pseudo) = PseudoElement::tree_pseudo_element(&name, args) {
                return Ok(pseudo);
            }
        }
        Err(parser.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name.clone())))
    }

    fn default_namespace(&self) -> Option<Namespace> {
        self.namespaces.default.clone().as_ref().map(|&(ref ns, _)| ns.clone())
    }

    fn namespace_for_prefix(&self, prefix: &Atom) -> Option<Namespace> {
        self.namespaces.prefixes.get(prefix).map(|&(ref ns, _)| ns.clone())
    }
}

impl SelectorImpl {
    /// A helper to traverse each eagerly cascaded pseudo-element, executing
    /// `fun` on it.
    #[inline]
    pub fn each_eagerly_cascaded_pseudo_element<F>(mut fun: F)
        where F: FnMut(PseudoElement),
    {
        for pseudo in &EAGER_PSEUDOS {
            fun(pseudo.clone())
        }
    }
}

unsafe impl HasFFI for SelectorList<SelectorImpl> {
    type FFIType = RawServoSelectorList;
}
unsafe impl HasSimpleFFI for SelectorList<SelectorImpl> {}
unsafe impl HasBoxFFI for SelectorList<SelectorImpl> {}
