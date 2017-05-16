/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Gecko-specific bits for selector-parsing.

use cssparser::{Parser, ToCss};
use element_state::{IN_ACTIVE_STATE, IN_FOCUS_STATE, IN_HOVER_STATE};
use element_state::ElementState;
use gecko_bindings::structs::CSSPseudoClassType;
use selector_parser::{SelectorParser, PseudoElementCascadeType};
use selectors::parser::{ComplexSelector, SelectorMethods};
use selectors::visitor::SelectorVisitor;
use std::borrow::Cow;
use std::fmt;
use string_cache::{Atom, Namespace, WeakAtom, WeakNamespace};

pub use gecko::pseudo_element::{PseudoElement, EAGER_PSEUDOS, EAGER_PSEUDO_COUNT};
pub use gecko::snapshot::SnapshotMap;

bitflags! {
    flags NonTSPseudoClassFlag: u8 {
        // See NonTSPseudoClass::is_internal()
        const PSEUDO_CLASS_INTERNAL = 0x01,
    }
}

macro_rules! pseudo_class_name {
    (bare: [$(($css:expr, $name:ident, $gecko_type:tt, $state:tt, $flags:tt),)*],
     string: [$(($s_css:expr, $s_name:ident, $s_gecko_type:tt, $s_state:tt, $s_flags:tt),)*]) => {
        #[doc = "Our representation of a non tree-structural pseudo-class."]
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub enum NonTSPseudoClass {
            $(
                #[doc = $css]
                $name,
            )*
            $(
                #[doc = $s_css]
                $s_name(Box<[u16]>),
            )*
            /// The non-standard `:-moz-any` pseudo-class.
            ///
            /// TODO(emilio): We disallow combinators and pseudos here, so we
            /// should use SimpleSelector instead
            MozAny(Box<[ComplexSelector<SelectorImpl>]>),
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
                        write!(dest, ":{}(", $s_css)?;
                        {
                            // FIXME(emilio): Avoid the extra allocation!
                            let mut css = CssStringWriter::new(dest);

                            // Discount the null char in the end from the
                            // string.
                            css.write_str(&String::from_utf16(&s[..s.len() - 1]).unwrap())?;
                        }
                        return dest.write_str(")")
                    }, )*
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

impl SelectorMethods for NonTSPseudoClass {
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
    /// A pseudo-class is internal if it can only be used inside
    /// user agent style sheets.
    pub fn is_internal(&self) -> bool {
        macro_rules! check_flag {
            (_) => (false);
            ($flags:expr) => ($flags.contains(PSEUDO_CLASS_INTERNAL));
        }
        macro_rules! pseudo_class_check_internal {
            (bare: [$(($css:expr, $name:ident, $gecko_type:tt, $state:tt, $flags:tt),)*],
            string: [$(($s_css:expr, $s_name:ident, $s_gecko_type:tt, $s_state:tt, $s_flags:tt),)*]) => {
                match *self {
                    $(NonTSPseudoClass::$name => check_flag!($flags),)*
                    $(NonTSPseudoClass::$s_name(..) => check_flag!($s_flags),)*
                    NonTSPseudoClass::MozAny(_) => false,
                }
            }
        }
        apply_non_ts_list!(pseudo_class_check_internal)
    }

    /// https://drafts.csswg.org/selectors-4/#useraction-pseudos
    ///
    /// We intentionally skip the link-related ones.
    fn is_safe_user_action_state(&self) -> bool {
        matches!(*self, NonTSPseudoClass::Hover |
                        NonTSPseudoClass::Active |
                        NonTSPseudoClass::Focus)
    }

    /// Get the state flag associated with a pseudo-class, if any.
    pub fn state_flag(&self) -> ElementState {
        macro_rules! flag {
            (_) => (ElementState::empty());
            ($state:ident) => (::element_state::$state);
        }
        macro_rules! pseudo_class_state {
            (bare: [$(($css:expr, $name:ident, $gecko_type:tt, $state:tt, $flags:tt),)*],
            string: [$(($s_css:expr, $s_name:ident, $s_gecko_type:tt, $s_state:tt, $s_flags:tt),)*]) => {
                match *self {
                    $(NonTSPseudoClass::$name => flag!($state),)*
                    $(NonTSPseudoClass::$s_name(..) => flag!($s_state),)*
                    NonTSPseudoClass::MozAny(..) => ElementState::empty(),
                }
            }
        }
        apply_non_ts_list!(pseudo_class_state)
    }

    /// Returns true if the given pseudoclass should trigger style sharing cache revalidation.
    pub fn needs_cache_revalidation(&self) -> bool {
        self.state_flag().is_empty() &&
        !matches!(*self, NonTSPseudoClass::MozAny(_))
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
                    NonTSPseudoClass::MozAny(_) => gecko_type!(any),
                }
            }
        }
        apply_non_ts_list!(pseudo_class_geckotype)
    }
}

/// The dummy struct we use to implement our selector parsing.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SelectorImpl;

/// Some subset of pseudo-elements in Gecko are sensitive to some state
/// selectors.
///
/// We store the sensitive states in this struct in order to properly handle
/// these.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PseudoElementSelector {
    pseudo: PseudoElement,
    state: ElementState,
}

impl PseudoElementSelector {
    /// Returns the pseudo-element this selector represents.
    pub fn pseudo_element(&self) -> &PseudoElement {
        &self.pseudo
    }

    /// Returns the pseudo-element selector state.
    pub fn state(&self) -> ElementState {
        self.state
    }
}

impl ToCss for PseudoElementSelector {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        if cfg!(debug_assertions) {
            let mut state = self.state;
            state.remove(IN_HOVER_STATE | IN_ACTIVE_STATE | IN_FOCUS_STATE);
            assert_eq!(state, ElementState::empty(),
                       "Unhandled pseudo-element state selector?");
        }

        self.pseudo.to_css(dest)?;

        if self.state.contains(IN_HOVER_STATE) {
            dest.write_str(":hover")?
        }

        if self.state.contains(IN_ACTIVE_STATE) {
            dest.write_str(":active")?
        }

        if self.state.contains(IN_FOCUS_STATE) {
            dest.write_str(":focus")?
        }

        Ok(())
    }
}

impl ::selectors::SelectorImpl for SelectorImpl {
    type AttrValue = Atom;
    type Identifier = Atom;
    type ClassName = Atom;
    type LocalName = Atom;
    type NamespacePrefix = Atom;
    type NamespaceUrl = Namespace;
    type BorrowedNamespaceUrl = WeakNamespace;
    type BorrowedLocalName = WeakAtom;

    type PseudoElementSelector = PseudoElementSelector;
    type NonTSPseudoClass = NonTSPseudoClass;
}

impl<'a> ::selectors::Parser for SelectorParser<'a> {
    type Impl = SelectorImpl;

    fn parse_non_ts_pseudo_class(&self, name: Cow<str>) -> Result<NonTSPseudoClass, ()> {
        macro_rules! pseudo_class_parse {
            (bare: [$(($css:expr, $name:ident, $gecko_type:tt, $state:tt, $flags:tt),)*],
             string: [$(($s_css:expr, $s_name:ident, $s_gecko_type:tt, $s_state:tt, $s_flags:tt),)*]) => {
                match_ignore_ascii_case! { &name,
                    $($css => NonTSPseudoClass::$name,)*
                    _ => return Err(())
                }
            }
        }
        let pseudo_class = apply_non_ts_list!(pseudo_class_parse);
        if !pseudo_class.is_internal() || self.in_user_agent_stylesheet() {
            Ok(pseudo_class)
        } else {
            Err(())
        }
    }

    fn parse_non_ts_functional_pseudo_class(&self,
                                            name: Cow<str>,
                                            parser: &mut Parser)
                                            -> Result<NonTSPseudoClass, ()> {
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
                    "-moz-any" => {
                        let selectors = parser.parse_comma_separated(|input| {
                            ComplexSelector::parse(self, input)
                        })?;
                        // Selectors inside `:-moz-any` may not include combinators.
                        if selectors.iter().flat_map(|x| x.iter_raw()).any(|s| s.is_combinator()) {
                            return Err(())
                        }
                        NonTSPseudoClass::MozAny(selectors.into_boxed_slice())
                    }
                    _ => return Err(())
                }
            }
        }
        let pseudo_class = apply_non_ts_list!(pseudo_class_string_parse);
        if !pseudo_class.is_internal() || self.in_user_agent_stylesheet() {
            Ok(pseudo_class)
        } else {
            Err(())
        }
    }

    fn parse_pseudo_element(&self, name: Cow<str>, input: &mut Parser) -> Result<PseudoElementSelector, ()> {
        let pseudo =
            match PseudoElement::from_slice(&name, self.in_user_agent_stylesheet()) {
                Some(pseudo) => pseudo,
                None => return Err(()),
            };

        let state = if pseudo.supports_user_action_state() {
            input.try(|input| {
                let mut state = ElementState::empty();

                while !input.is_exhausted() {
                    input.expect_colon()?;
                    let ident = input.expect_ident()?;
                    let pseudo_class = self.parse_non_ts_pseudo_class(ident)?;

                    if !pseudo_class.is_safe_user_action_state() {
                        return Err(())
                    }
                    state.insert(pseudo_class.state_flag());
                }

                Ok(state)
            }).ok()
        } else {
            None
        };

        Ok(PseudoElementSelector {
            pseudo: pseudo,
            state: state.unwrap_or(ElementState::empty()),
        })
    }

    fn default_namespace(&self) -> Option<Namespace> {
        self.namespaces.default.clone()
    }

    fn namespace_for_prefix(&self, prefix: &Atom) -> Option<Namespace> {
        self.namespaces.prefixes.get(prefix).cloned()
    }
}

impl SelectorImpl {
    #[inline]
    /// Legacy alias for PseudoElement::cascade_type.
    pub fn pseudo_element_cascade_type(pseudo: &PseudoElement) -> PseudoElementCascadeType {
        pseudo.cascade_type()
    }

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


    #[inline]
    /// Executes a function for each pseudo-element.
    pub fn each_pseudo_element<F>(fun: F)
        where F: FnMut(PseudoElement),
    {
        PseudoElement::each(fun)
    }

    #[inline]
    /// Returns the relevant state flag for a given non-tree-structural
    /// pseudo-class.
    pub fn pseudo_class_state_flag(pc: &NonTSPseudoClass) -> ElementState {
        pc.state_flag()
    }
}
