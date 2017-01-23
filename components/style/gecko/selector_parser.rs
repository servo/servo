/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Gecko-specific bits for selector-parsing.

use cssparser::ToCss;
use element_state::ElementState;
use gecko_bindings::structs::CSSPseudoClassType;
use selector_parser::{SelectorParser, PseudoElementCascadeType};
use selector_parser::{attr_equals_selector_is_shareable, attr_exists_selector_is_shareable};
use selectors::parser::AttrSelector;
use std::borrow::Cow;
use std::fmt;
use string_cache::{Atom, Namespace, WeakAtom, WeakNamespace};

/// A representation of a CSS pseudo-element.
///
/// In Gecko, we represent pseudo-elements as plain `Atom`s.
///
/// The boolean field represents whether this element is an anonymous box. This
/// is just for convenience, instead of recomputing it.
///
/// Also, note that the `Atom` member is always a static atom, so if space is a
/// concern, we can use the raw pointer and use the lower bit to represent it
/// without space overhead.
///
/// FIXME(emilio): we know all these atoms are static. Patches are starting to
/// pile up, but a further potential optimisation is generating bindings without
/// `-no-gen-bitfield-methods` (that was removed to compile on stable, but it no
/// longer depends on it), and using the raw *mut nsIAtom (properly asserting
/// we're a static atom).
///
/// This should allow us to avoid random FFI overhead when cloning/dropping
/// pseudos.
///
/// Also, we can further optimize PartialEq and hash comparing/hashing only the
/// atoms.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PseudoElement(Atom, bool);

impl PseudoElement {
    /// Get the pseudo-element as an atom.
    #[inline]
    pub fn as_atom(&self) -> &Atom {
        &self.0
    }

    /// Whether this pseudo-element is an anonymous box.
    #[inline]
    fn is_anon_box(&self) -> bool {
        self.1
    }

    /// Construct a pseudo-element from an `Atom`, receiving whether it is also
    /// an anonymous box, and don't check it on release builds.
    ///
    /// On debug builds we assert it's the result we expect.
    #[inline]
    pub fn from_atom_unchecked(atom: Atom, is_anon_box: bool) -> Self {
        if cfg!(debug_assertions) {
            // Do the check on debug regardless.
            match Self::from_atom(&*atom, true) {
                Some(pseudo) => {
                    assert_eq!(pseudo.is_anon_box(), is_anon_box);
                    return pseudo;
                }
                None => panic!("Unknown pseudo: {:?}", atom),
            }
        }

        PseudoElement(atom, is_anon_box)
    }

    #[inline]
    fn from_atom(atom: &WeakAtom, _in_ua: bool) -> Option<Self> {
        macro_rules! pseudo_element {
            ($pseudo_str_with_colon:expr, $atom:expr, $is_anon_box:expr) => {{
                if atom == &*$atom {
                    return Some(PseudoElement($atom, $is_anon_box));
                }
            }}
        }

        include!("generated/gecko_pseudo_element_helper.rs");

        None
    }

    /// Constructs an atom from a string of text, and whether we're in a
    /// user-agent stylesheet.
    ///
    /// If we're not in a user-agent stylesheet, we will never parse anonymous
    /// box pseudo-elements.
    ///
    /// Returns `None` if the pseudo-element is not recognised.
    #[inline]
    fn from_slice(s: &str, in_ua_stylesheet: bool) -> Option<Self> {
        use std::ascii::AsciiExt;
        macro_rules! pseudo_element {
            ($pseudo_str_with_colon:expr, $atom:expr, $is_anon_box:expr) => {{
                if !$is_anon_box || in_ua_stylesheet {
                    if s.eq_ignore_ascii_case(&$pseudo_str_with_colon[1..]) {
                        return Some(PseudoElement($atom, $is_anon_box))
                    }
                }
            }}
        }

        include!("generated/gecko_pseudo_element_helper.rs");

        None
    }
}

impl ToCss for PseudoElement {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        // FIXME: why does the atom contain one colon? Pseudo-element has two
        debug_assert!(self.0.as_slice().starts_with(&[b':' as u16]) &&
                      !self.0.as_slice().starts_with(&[b':' as u16, b':' as u16]));
        try!(dest.write_char(':'));
        write!(dest, "{}", self.0)
    }
}

bitflags! {
    flags NonTSPseudoClassFlag: u8 {
        // See NonTSPseudoClass::is_internal()
        const PSEUDO_CLASS_INTERNAL = 0x01,
    }
}

macro_rules! pseudo_class_list {
    ($(($css:expr, $name:ident, $_gecko_type:tt, $_state:tt, $_flags:tt),)*) => {
        #[doc = "Our representation of a non tree-structural pseudo-class."]
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub enum NonTSPseudoClass {
            $(
                #[doc = $css]
                $name,
            )*
        }
    }
}
include!("non_ts_pseudo_class_list.rs");

impl ToCss for NonTSPseudoClass {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        macro_rules! pseudo_class_list {
            ($(($css:expr, $name:ident, $_gecko_type:tt, $_state:tt, $_flags:tt),)*) => {
                match *self {
                    $(NonTSPseudoClass::$name => concat!(":", $css),)*
                }
            }
        }
        dest.write_str(include!("non_ts_pseudo_class_list.rs"))
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
        macro_rules! pseudo_class_list {
            ($(($_css:expr, $name:ident, $_gecko_type:tt, $_state:tt, $flags:tt),)*) => {
                match *self {
                    $(NonTSPseudoClass::$name => check_flag!($flags),)*
                }
            }
        }
        include!("non_ts_pseudo_class_list.rs")
    }

    /// Get the state flag associated with a pseudo-class, if any.
    pub fn state_flag(&self) -> ElementState {
        macro_rules! flag {
            (_) => (ElementState::empty());
            ($state:ident) => (::element_state::$state);
        }
        macro_rules! pseudo_class_list {
            ($(($_css:expr, $name:ident, $_gecko_type:tt, $state:tt, $_flags:tt),)*) => {
                match *self {
                    $(NonTSPseudoClass::$name => flag!($state),)*
                }
            }
        }
        include!("non_ts_pseudo_class_list.rs")
    }

    /// Convert NonTSPseudoClass to Gecko's CSSPseudoClassType.
    pub fn to_gecko_pseudoclasstype(&self) -> Option<CSSPseudoClassType> {
        macro_rules! gecko_type {
            (_) => (None);
            ($gecko_type:ident) =>
                (Some(::gecko_bindings::structs::CSSPseudoClassType::$gecko_type));
        }
        macro_rules! pseudo_class_list {
            ($(($_css:expr, $name:ident, $gecko_type:tt, $_state:tt, $_flags:tt),)*) => {
                match *self {
                    $(NonTSPseudoClass::$name => gecko_type!($gecko_type),)*
                }
            }
        }
        include!("non_ts_pseudo_class_list.rs")
    }
}

/// The dummy struct we use to implement our selector parsing.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectorImpl;

impl ::selectors::SelectorImpl for SelectorImpl {
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

    fn attr_exists_selector_is_shareable(attr_selector: &AttrSelector<Self>) -> bool {
        attr_exists_selector_is_shareable(attr_selector)
    }

    fn attr_equals_selector_is_shareable(attr_selector: &AttrSelector<Self>,
                                         value: &Self::AttrValue) -> bool {
        attr_equals_selector_is_shareable(attr_selector, value)
    }
}

impl<'a> ::selectors::Parser for SelectorParser<'a> {
    type Impl = SelectorImpl;

    fn parse_non_ts_pseudo_class(&self, name: Cow<str>) -> Result<NonTSPseudoClass, ()> {
        macro_rules! pseudo_class_list {
            ($(($css:expr, $name:ident, $_gecko_type:tt, $_state:tt, $_flags:tt),)*) => {
                match_ignore_ascii_case! { &name,
                    $($css => NonTSPseudoClass::$name,)*
                    _ => return Err(())
                }
            }
        }
        let pseudo_class = include!("non_ts_pseudo_class_list.rs");
        if !pseudo_class.is_internal() || self.in_user_agent_stylesheet() {
            Ok(pseudo_class)
        } else {
            Err(())
        }
    }

    fn parse_pseudo_element(&self, name: Cow<str>) -> Result<PseudoElement, ()> {
        match PseudoElement::from_slice(&name, self.in_user_agent_stylesheet()) {
            Some(pseudo) => Ok(pseudo),
            None => Err(()),
        }
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
    /// Returns the kind of cascade type that a given pseudo is going to use.
    ///
    /// In Gecko we only compute ::before and ::after eagerly. We save the rules
    /// for anonymous boxes separately, so we resolve them as precomputed
    /// pseudos.
    ///
    /// We resolve the others lazily, see `Servo_ResolvePseudoStyle`.
    pub fn pseudo_element_cascade_type(pseudo: &PseudoElement) -> PseudoElementCascadeType {
        if Self::pseudo_is_before_or_after(pseudo) {
            return PseudoElementCascadeType::Eager
        }

        if pseudo.is_anon_box() {
            return PseudoElementCascadeType::Precomputed
        }

        PseudoElementCascadeType::Lazy
    }

    #[inline]
    /// Executes a function for each pseudo-element.
    pub fn each_pseudo_element<F>(mut fun: F)
        where F: FnMut(PseudoElement),
    {
        macro_rules! pseudo_element {
            ($pseudo_str_with_colon:expr, $atom:expr, $is_anon_box:expr) => {{
                fun(PseudoElement($atom, $is_anon_box));
            }}
        }

        include!("generated/gecko_pseudo_element_helper.rs")
    }

    #[inline]
    /// Returns whether the given pseudo-element is `::before` or `::after`.
    pub fn pseudo_is_before_or_after(pseudo: &PseudoElement) -> bool {
        *pseudo.as_atom() == atom!(":before") ||
        *pseudo.as_atom() == atom!(":after")
    }

    #[inline]
    /// Returns the relevant state flag for a given non-tree-structural
    /// pseudo-class.
    pub fn pseudo_class_state_flag(pc: &NonTSPseudoClass) -> ElementState {
        pc.state_flag()
    }
}
