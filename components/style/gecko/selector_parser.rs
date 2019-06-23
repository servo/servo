/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Gecko-specific bits for selector-parsing.

use crate::element_state::{DocumentState, ElementState};
use crate::gecko_bindings::structs::{self, RawServoSelectorList};
use crate::gecko_bindings::sugar::ownership::{HasBoxFFI, HasFFI, HasSimpleFFI};
use crate::invalidation::element::document_state::InvalidationMatchingData;
use crate::selector_parser::{Direction, SelectorParser};
use crate::str::starts_with_ignore_ascii_case;
use crate::string_cache::{Atom, Namespace, WeakAtom, WeakNamespace};
use crate::values::serialize_atom_identifier;
use cssparser::{BasicParseError, BasicParseErrorKind, Parser};
use cssparser::{CowRcStr, SourceLocation, ToCss, Token};
use selectors::parser::{self as selector_parser, Selector};
use selectors::parser::{SelectorParseErrorKind, Visit};
use selectors::visitor::SelectorVisitor;
use selectors::SelectorList;
use std::fmt;
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss as ToCss_};
use thin_slice::ThinBoxedSlice;

pub use crate::gecko::pseudo_element::{
    PseudoElement, EAGER_PSEUDOS, EAGER_PSEUDO_COUNT, PSEUDO_COUNT,
};
pub use crate::gecko::snapshot::SnapshotMap;

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

/// The type used to store the language argument to the `:lang` pseudo-class.
pub type Lang = Atom;

macro_rules! pseudo_class_name {
    ([$(($css:expr, $name:ident, $gecko_type:tt, $state:tt, $flags:tt),)*]) => {
        /// Our representation of a non tree-structural pseudo-class.
        #[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq, ToShmem)]
        pub enum NonTSPseudoClass {
            $(
                #[doc = $css]
                $name,
            )*
            /// The `:lang` pseudo-class.
            Lang(Lang),
            /// The `:dir` pseudo-class.
            Dir(Direction),
            /// The non-standard `:-moz-any` pseudo-class.
            ///
            /// TODO(emilio): We disallow combinators and pseudos here, so we
            /// should use SimpleSelector instead
            MozAny(ThinBoxedSlice<Selector<SelectorImpl>>),
            /// The non-standard `:-moz-locale-dir` pseudo-class.
            MozLocaleDir(Direction),
        }
    }
}
apply_non_ts_list!(pseudo_class_name);

impl ToCss for NonTSPseudoClass {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        macro_rules! pseudo_class_serialize {
            ([$(($css:expr, $name:ident, $gecko_type:tt, $state:tt, $flags:tt),)*]) => {
                match *self {
                    $(NonTSPseudoClass::$name => concat!(":", $css),)*
                    NonTSPseudoClass::Lang(ref s) => {
                        dest.write_str(":lang(")?;
                        serialize_atom_identifier(s, dest)?;
                        return dest.write_char(')');
                    },
                    NonTSPseudoClass::MozLocaleDir(ref dir) => {
                        dest.write_str(":-moz-locale-dir(")?;
                        dir.to_css(&mut CssWriter::new(dest))?;
                        return dest.write_char(')')
                    },
                    NonTSPseudoClass::Dir(ref dir) => {
                        dest.write_str(":dir(")?;
                        dir.to_css(&mut CssWriter::new(dest))?;
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
                        return dest.write_char(')')
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
    where
        V: SelectorVisitor<Impl = Self::Impl>,
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
    /// Parses the name and returns a non-ts-pseudo-class if succeeds.
    /// None otherwise. It doesn't check whether the pseudo-class is enabled
    /// in a particular state.
    pub fn parse_non_functional(name: &str) -> Option<Self> {
        macro_rules! pseudo_class_parse {
            ([$(($css:expr, $name:ident, $gecko_type:tt, $state:tt, $flags:tt),)*]) => {
                match_ignore_ascii_case! { &name,
                    $($css => Some(NonTSPseudoClass::$name),)*
                    _ => None,
                }
            }
        }
        apply_non_ts_list!(pseudo_class_parse)
    }

    /// Returns true if this pseudo-class has any of the given flags set.
    fn has_any_flag(&self, flags: NonTSPseudoClassFlag) -> bool {
        macro_rules! check_flag {
            (_) => {
                false
            };
            ($flags:ident) => {
                NonTSPseudoClassFlag::$flags.intersects(flags)
            };
        }
        macro_rules! pseudo_class_check_is_enabled_in {
            ([$(($css:expr, $name:ident, $gecko_type:tt, $state:tt, $flags:tt),)*]) => {
                match *self {
                    $(NonTSPseudoClass::$name => check_flag!($flags),)*
                    NonTSPseudoClass::MozLocaleDir(_) |
                    NonTSPseudoClass::Lang(_) |
                    NonTSPseudoClass::Dir(_) |
                    NonTSPseudoClass::MozAny(_) => false,
                }
            }
        }
        apply_non_ts_list!(pseudo_class_check_is_enabled_in)
    }

    /// Returns whether the pseudo-class is enabled in content sheets.
    fn is_enabled_in_content(&self) -> bool {
        use crate::gecko_bindings::structs::mozilla;
        match *self {
            // For pseudo-classes with pref, the availability in content
            // depends on the pref.
            NonTSPseudoClass::Fullscreen => unsafe {
                mozilla::StaticPrefs_sVarCache_full_screen_api_unprefix_enabled
            },
            // Otherwise, a pseudo-class is enabled in content when it
            // doesn't have any enabled flag.
            _ => !self
                .has_any_flag(NonTSPseudoClassFlag::PSEUDO_CLASS_ENABLED_IN_UA_SHEETS_AND_CHROME),
        }
    }

    /// Get the state flag associated with a pseudo-class, if any.
    pub fn state_flag(&self) -> ElementState {
        macro_rules! flag {
            (_) => {
                ElementState::empty()
            };
            ($state:ident) => {
                ElementState::$state
            };
        }
        macro_rules! pseudo_class_state {
            ([$(($css:expr, $name:ident, $gecko_type:tt, $state:tt, $flags:tt),)*]) => {
                match *self {
                    $(NonTSPseudoClass::$name => flag!($state),)*
                    NonTSPseudoClass::Dir(..) |
                    NonTSPseudoClass::MozLocaleDir(..) |
                    NonTSPseudoClass::Lang(..) |
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

    /// Returns true if the evaluation of the pseudo-class depends on the
    /// element's attributes.
    pub fn is_attr_based(&self) -> bool {
        matches!(
            *self,
            NonTSPseudoClass::MozTableBorderNonzero |
                NonTSPseudoClass::MozBrowserFrame |
                NonTSPseudoClass::Lang(..)
        )
    }
}

impl ::selectors::parser::NonTSPseudoClass for NonTSPseudoClass {
    type Impl = SelectorImpl;

    #[inline]
    fn is_active_or_hover(&self) -> bool {
        matches!(*self, NonTSPseudoClass::Active | NonTSPseudoClass::Hover)
    }

    /// We intentionally skip the link-related ones.
    #[inline]
    fn is_user_action_state(&self) -> bool {
        matches!(
            *self,
            NonTSPseudoClass::Hover | NonTSPseudoClass::Active | NonTSPseudoClass::Focus
        )
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
    type PartName = Atom;
    type LocalName = Atom;
    type NamespacePrefix = Atom;
    type NamespaceUrl = Namespace;
    type BorrowedNamespaceUrl = WeakNamespace;
    type BorrowedLocalName = WeakAtom;

    type PseudoElement = PseudoElement;
    type NonTSPseudoClass = NonTSPseudoClass;
}

impl<'a> SelectorParser<'a> {
    fn is_pseudo_class_enabled(&self, pseudo_class: &NonTSPseudoClass) -> bool {
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

    fn is_pseudo_element_enabled(&self, pseudo_element: &PseudoElement) -> bool {
        if pseudo_element.enabled_in_content() {
            return true;
        }

        if self.in_user_agent_stylesheet() && pseudo_element.enabled_in_ua_sheets() {
            return true;
        }

        if self.chrome_rules_enabled() && pseudo_element.enabled_in_chrome() {
            return true;
        }

        return false;
    }
}

impl<'a, 'i> ::selectors::Parser<'i> for SelectorParser<'a> {
    type Impl = SelectorImpl;
    type Error = StyleParseErrorKind<'i>;

    #[inline]
    fn parse_slotted(&self) -> bool {
        true
    }

    #[inline]
    fn parse_host(&self) -> bool {
        true
    }

    #[inline]
    fn parse_part(&self) -> bool {
        self.chrome_rules_enabled() ||
            unsafe { structs::StaticPrefs_sVarCache_layout_css_shadow_parts_enabled }
    }

    fn parse_non_ts_pseudo_class(
        &self,
        location: SourceLocation,
        name: CowRcStr<'i>,
    ) -> Result<NonTSPseudoClass, ParseError<'i>> {
        if let Some(pseudo_class) = NonTSPseudoClass::parse_non_functional(&name) {
            if self.is_pseudo_class_enabled(&pseudo_class) {
                return Ok(pseudo_class);
            }
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
        parser: &mut Parser<'i, 't>,
    ) -> Result<NonTSPseudoClass, ParseError<'i>> {
        let pseudo_class = match_ignore_ascii_case! { &name,
            "lang" => {
                let name = parser.expect_ident_or_string()?;
                NonTSPseudoClass::Lang(Atom::from(name.as_ref()))
            },
            "-moz-locale-dir" => {
                NonTSPseudoClass::MozLocaleDir(Direction::parse(parser)?)
            },
            "dir" => {
                NonTSPseudoClass::Dir(Direction::parse(parser)?)
            },
            "-moz-any" => {
                NonTSPseudoClass::MozAny(
                    selector_parser::parse_compound_selector_list(
                        self,
                        parser,
                    )?.into()
                )
            }
            _ => return Err(parser.new_custom_error(
                SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name.clone())
            ))
        };
        if self.is_pseudo_class_enabled(&pseudo_class) {
            Ok(pseudo_class)
        } else {
            Err(
                parser.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(
                    name,
                )),
            )
        }
    }

    fn parse_pseudo_element(
        &self,
        location: SourceLocation,
        name: CowRcStr<'i>,
    ) -> Result<PseudoElement, ParseError<'i>> {
        if let Some(pseudo) = PseudoElement::from_slice(&name) {
            if self.is_pseudo_element_enabled(&pseudo) {
                return Ok(pseudo);
            }
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
        parser: &mut Parser<'i, 't>,
    ) -> Result<PseudoElement, ParseError<'i>> {
        if starts_with_ignore_ascii_case(&name, "-moz-tree-") {
            // Tree pseudo-elements can have zero or more arguments, separated
            // by either comma or space.
            let mut args = Vec::new();
            loop {
                let location = parser.current_source_location();
                match parser.next() {
                    Ok(&Token::Ident(ref ident)) => args.push(Atom::from(ident.as_ref())),
                    Ok(&Token::Comma) => {},
                    Ok(t) => return Err(location.new_unexpected_token_error(t.clone())),
                    Err(BasicParseError {
                        kind: BasicParseErrorKind::EndOfInput,
                        ..
                    }) => break,
                    _ => unreachable!("Parser::next() shouldn't return any other error"),
                }
            }
            let args = args.into_boxed_slice();
            if let Some(pseudo) = PseudoElement::tree_pseudo_element(&name, args) {
                if self.is_pseudo_element_enabled(&pseudo) {
                    return Ok(pseudo);
                }
            }
        }
        Err(
            parser.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(
                name,
            )),
        )
    }

    fn default_namespace(&self) -> Option<Namespace> {
        self.namespaces.default.as_ref().map(|ns| ns.clone())
    }

    fn namespace_for_prefix(&self, prefix: &Atom) -> Option<Namespace> {
        self.namespaces.prefixes.get(prefix).cloned()
    }
}

impl SelectorImpl {
    /// A helper to traverse each eagerly cascaded pseudo-element, executing
    /// `fun` on it.
    #[inline]
    pub fn each_eagerly_cascaded_pseudo_element<F>(mut fun: F)
    where
        F: FnMut(PseudoElement),
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
