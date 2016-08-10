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
/// NOTE: we can further optimize PartialEq comparing only the atoms.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PseudoElement(Atom, bool);

// TODO: automatize this list from
// http://searchfox.org/mozilla-central/source/layout/style/nsCSSPseudoElements.h and
impl PseudoElement {
    #[inline]
    fn before() -> Self {
        PseudoElement(atom!(":before"), false)
    }

    #[inline]
    fn after() -> Self {
        PseudoElement(atom!(":before"), false)
    }

    #[inline]
    fn as_atom(&self) -> &Atom {
        &self.0
    }

    #[inline]
    fn is_anon_box(&self) -> bool {
        self.1
    }

    #[inline]
    fn from_atom_unchecked(atom: Atom, is_anon_box: bool) -> Self {
        if cfg!(debug_assertions) {
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
        use std::slice::Iter;
        let mut slice = atom.as_slice();

        let len = slice.len();
        let mut iter = slice.iter();

        if iter.next().map(|c| *c) != Some(':' as u16) {
            return None
        }

        // This type is just needed because a plain iter.map() doesn't derive
        // Clone because the closure is not Clone.
        #[derive(Clone)]
        struct Utf16toASCIIIter<'a>(Iter<'a, u16>);

        impl<'a> Iterator for Utf16toASCIIIter<'a> {
            type Item = u8;

            fn next(&mut self) -> Option<u8> {
                self.0.next().map(|c| {
                    if *c <= 255 {
                        *c as u8
                    } else {
                        // Something that can't exist in any pseudo-element.
                        0u8
                    }
                })
            }
        }

        Self::from_bytes_iter(Utf16toASCIIIter(iter), in_ua, len)
    }

    #[inline]
    fn from_slice(s: &str, in_ua: bool) -> Option<Self> {
        let bytes = s.as_bytes();
        let len = bytes.len();
        Self::from_bytes_iter(bytes.iter().cloned(), in_ua, len)
    }

    fn from_bytes_iter<I: Iterator<Item=u8> + Clone>(iter: I,
                                                     in_ua_stylesheet: bool,
                                                     byte_len: usize) -> Option<Self> {
        use std::ascii::AsciiExt;
        macro_rules! parse {
            // NOTE: In an ideal world we should be able to either:
            //
            //  1. use only $pseudo_str, and generate the atom with
            //     atom!(concat!(":", $pseudo_str))
            //  2. use only the pseudo string with the colon, use
            //     atom!($pseudo), and use $pseudo[1..] for comparing.
            //
            // This is not an ideal world, and none of those options work.
            ($pseudo_str:expr, $atom:expr) => {
                parse!($pseudo_str, $atom, false)
            };
            ($pseudo_str:expr, $atom:expr, $is_anon_box:expr) => {{
                debug_assert!($pseudo_str[0..].chars().next().unwrap() != ':',
                              "Parser doesn't provide semicolons");
                debug_assert!($atom.chars().next().unwrap().unwrap()  == ':',
                              "Gecko expects semicolons");
                let bytes = $pseudo_str.as_bytes();
                if byte_len == bytes.len() &&
                    bytes.iter().zip(iter.clone()).all(|(a, b)| b.eq_ignore_ascii_case(a)) {
                    return Some(PseudoElement($atom, $is_anon_box))
                }
            }}
        }

        parse!("before", atom!(":before"));
        parse!("after", atom!(":after"));

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
        if *pseudo == PseudoElement::before() ||
           *pseudo == PseudoElement::after() {
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
        // XXX: This is really lame, and should be autogenerated.
        fun(PseudoElement(atom!(":before"), false));
        fun(PseudoElement(atom!(":after"), true));
    }

    #[inline]
    pub fn pseudo_is_before_or_after(pseudo: &PseudoElement) -> bool {
        *pseudo == PseudoElement::before() || *pseudo == PseudoElement::after()
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
