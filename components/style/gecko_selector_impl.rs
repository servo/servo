/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use element_state::ElementState;
use selector_impl::PseudoElementCascadeType;
use selector_impl::{attr_exists_selector_is_shareable, attr_equals_selector_is_shareable};
use selectors::parser::{ParserContext, SelectorImpl, AttrSelector};
use string_cache::{Atom, WeakAtom, Namespace, WeakNamespace};
use stylesheets::Stylesheet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeckoSelectorImpl;

/// NOTE: The boolean field represents whether this element is an anonymous box.
///
/// This is just for convenience, instead of recomputing it. Also, note that
/// Atom is always a static atom, so if space is a concern, we can use the
/// raw pointer and use the lower bit to represent it without space overhead.
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
    #[inline]
    fn as_atom(&self) -> &Atom {
        &self.0
    }

    #[inline]
    fn is_anon_box(&self) -> bool {
        self.1
    }

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
    fn from_atom(atom: &WeakAtom, in_ua: bool) -> Option<Self> {
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NonTSPseudoClass {
    AnyLink,
    Link,
    Visited,
    Active,
    Focus,
    Hover,
    Enabled,
    Disabled,
    Checked,
    Indeterminate,
    ReadWrite,
    ReadOnly,
}

impl NonTSPseudoClass {
    pub fn state_flag(&self) -> ElementState {
        use element_state::*;
        use self::NonTSPseudoClass::*;
        match *self {
            Active => IN_ACTIVE_STATE,
            Focus => IN_FOCUS_STATE,
            Hover => IN_HOVER_STATE,
            Enabled => IN_ENABLED_STATE,
            Disabled => IN_DISABLED_STATE,
            Checked => IN_CHECKED_STATE,
            Indeterminate => IN_INDETERMINATE_STATE,
            ReadOnly | ReadWrite => IN_READ_WRITE_STATE,

            AnyLink |
            Link |
            Visited => ElementState::empty(),
        }
    }
}

impl SelectorImpl for GeckoSelectorImpl {
    type AttrValue = Atom;
    type Identifier = Atom;
    type ClassName = Atom;
    type LocalName = Atom;
    type Namespace = Namespace;
    type BorrowedNamespace = WeakNamespace;
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

    fn parse_non_ts_pseudo_class(_context: &ParserContext<Self>,
                                 name: &str) -> Result<NonTSPseudoClass, ()> {
        use self::NonTSPseudoClass::*;
        let pseudo_class = match_ignore_ascii_case! { name,
            "any-link" => AnyLink,
            "link" => Link,
            "visited" => Visited,
            "active" => Active,
            "focus" => Focus,
            "hover" => Hover,
            "enabled" => Enabled,
            "disabled" => Disabled,
            "checked" => Checked,
            "indeterminate" => Indeterminate,
            "read-write" => ReadWrite,
            "read-only" => ReadOnly,
            _ => return Err(())
        };

        Ok(pseudo_class)
    }

    fn parse_pseudo_element(context: &ParserContext<Self>,
                            name: &str) -> Result<PseudoElement, ()> {
        match PseudoElement::from_slice(name, context.in_user_agent_stylesheet) {
            Some(pseudo) => Ok(pseudo),
            None => Err(()),
        }
    }
}

impl GeckoSelectorImpl {
    #[inline]
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
    pub fn each_pseudo_element<F>(mut fun: F)
        where F: FnMut(PseudoElement)
    {
        macro_rules! pseudo_element {
            ($pseudo_str_with_colon:expr, $atom:expr, $is_anon_box:expr) => {{
                fun(PseudoElement($atom, $is_anon_box));
            }}
        }

        include!("generated/gecko_pseudo_element_helper.rs")
    }

    #[inline]
    pub fn pseudo_is_before_or_after(pseudo: &PseudoElement) -> bool {
        *pseudo.as_atom() == atom!(":before") ||
        *pseudo.as_atom() == atom!(":after")
    }

    #[inline]
    pub fn pseudo_class_state_flag(pc: &NonTSPseudoClass) -> ElementState {
        pc.state_flag()
    }

    #[inline]
    pub fn get_user_or_user_agent_stylesheets() -> &'static [Stylesheet] {
        &[]
    }

    #[inline]
    pub fn get_quirks_mode_stylesheet() -> Option<&'static Stylesheet> {
        None
    }
}
