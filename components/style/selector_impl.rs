/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The pseudo-classes and pseudo-elements supported by the style system.

use element_state::ElementState;
use restyle_hints;
use selectors::Element;
use selectors::parser::SelectorImpl;
use std::fmt::Debug;
use stylesheets::Stylesheet;

pub type AttrString = <TheSelectorImpl as SelectorImpl>::AttrString;

#[cfg(feature = "servo")]
pub use servo_selector_impl::*;

#[cfg(feature = "servo")]
pub use servo_selector_impl::{ServoSelectorImpl as TheSelectorImpl, ServoElementSnapshot as ElementSnapshot};

#[cfg(feature = "gecko")]
pub use gecko_selector_impl::*;

#[cfg(feature = "gecko")]
pub use gecko_selector_impl::{GeckoSelectorImpl as TheSelectorImpl};

/// This function determines if a pseudo-element is eagerly cascaded or not.
///
/// Eagerly cascaded pseudo-elements are "normal" pseudo-elements (i.e.
/// `::before` and `::after`). They inherit styles normally as another
/// selector would do, and they're part of the cascade.
///
/// Lazy pseudo-elements are affected by selector matching, but they're only
/// computed when needed, and not before. They're useful for general
/// pseudo-elements that are not very common.
///
/// Note that in Servo lazy pseudo-elements are restricted to a subset of
/// selectors, so you can't use it for public pseudo-elements. This is not the
/// case with Gecko though.
///
/// Precomputed ones skip the cascade process entirely, mostly as an
/// optimisation since they are private pseudo-elements (like
/// `::-servo-details-content`).
///
/// This pseudo-elements are resolved on the fly using *only* global rules
/// (rules of the form `*|*`), and applying them to the parent style.
///
/// If you're implementing a public selector that the end-user might customize,
/// then you probably need to make it eager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PseudoElementCascadeType {
    Eager,
    Lazy,
    Precomputed,
}

impl PseudoElementCascadeType {
    #[inline]
    pub fn is_eager(&self) -> bool {
        *self == PseudoElementCascadeType::Eager
    }

    #[inline]
    pub fn is_lazy(&self) -> bool {
        *self == PseudoElementCascadeType::Lazy
    }

    #[inline]
    pub fn is_precomputed(&self) -> bool {
        *self == PseudoElementCascadeType::Precomputed
    }
}

pub trait ElementExt: Element<Impl=TheSelectorImpl, AttrString=<TheSelectorImpl as SelectorImpl>::AttrString> {
    type Snapshot: restyle_hints::ElementSnapshot<AttrString = Self::AttrString> + 'static;

    fn is_link(&self) -> bool;
}

// NB: The `Clone` trait is here for convenience due to:
// https://github.com/rust-lang/rust/issues/26925
pub trait SelectorImplExt : SelectorImpl + Clone + Debug + Sized + 'static {
    fn pseudo_element_cascade_type(pseudo: &Self::PseudoElement) -> PseudoElementCascadeType;

    fn each_pseudo_element<F>(mut fun: F)
        where F: FnMut(Self::PseudoElement);

    #[inline]
    fn each_eagerly_cascaded_pseudo_element<F>(mut fun: F)
        where F: FnMut(<Self as SelectorImpl>::PseudoElement) {
        Self::each_pseudo_element(|pseudo| {
            if Self::pseudo_element_cascade_type(&pseudo).is_eager() {
                fun(pseudo)
            }
        })
    }

    #[inline]
    fn each_precomputed_pseudo_element<F>(mut fun: F)
        where F: FnMut(<Self as SelectorImpl>::PseudoElement) {
        Self::each_pseudo_element(|pseudo| {
            if Self::pseudo_element_cascade_type(&pseudo).is_precomputed() {
                fun(pseudo)
            }
        })
    }

    fn pseudo_is_before_or_after(pseudo: &Self::PseudoElement) -> bool;

    fn pseudo_class_state_flag(pc: &Self::NonTSPseudoClass) -> ElementState;

    fn get_user_or_user_agent_stylesheets() -> &'static [Stylesheet];

    fn get_quirks_mode_stylesheet() -> Option<&'static Stylesheet>;
}
