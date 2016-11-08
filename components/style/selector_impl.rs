/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The pseudo-classes and pseudo-elements supported by the style system.

use matching::{common_style_affecting_attributes, CommonStyleAffectingAttributeMode};
use selectors::Element;
use selectors::parser::{AttrSelector, SelectorImpl};

pub type AttrValue = <TheSelectorImpl as SelectorImpl>::AttrValue;

#[cfg(feature = "servo")]
pub use servo::selector_impl::*;

#[cfg(feature = "gecko")]
pub use gecko::selector_impl::*;

#[cfg(feature = "servo")]
pub use servo::selector_impl::ServoSelectorImpl as TheSelectorImpl;

#[cfg(feature = "gecko")]
pub use gecko::selector_impl::GeckoSelectorImpl as TheSelectorImpl;

#[cfg(feature = "servo")]
pub use servo::selector_impl::ServoElementSnapshot as Snapshot;

#[cfg(feature = "gecko")]
pub use gecko::snapshot::GeckoElementSnapshot as Snapshot;

#[cfg(feature = "servo")]
pub use servo::restyle_damage::ServoRestyleDamage as RestyleDamage;

#[cfg(feature = "gecko")]
pub use gecko::restyle_damage::GeckoRestyleDamage as RestyleDamage;

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

pub trait ElementExt: Element<Impl=TheSelectorImpl> {
    fn is_link(&self) -> bool;
}

impl TheSelectorImpl {
    #[inline]
    pub fn each_eagerly_cascaded_pseudo_element<F>(mut fun: F)
        where F: FnMut(PseudoElement)
    {
        Self::each_pseudo_element(|pseudo| {
            if Self::pseudo_element_cascade_type(&pseudo).is_eager() {
                fun(pseudo)
            }
        })
    }

    #[inline]
    pub fn each_precomputed_pseudo_element<F>(mut fun: F)
        where F: FnMut(PseudoElement)
    {
        Self::each_pseudo_element(|pseudo| {
            if Self::pseudo_element_cascade_type(&pseudo).is_precomputed() {
                fun(pseudo)
            }
        })
    }
}

pub fn attr_exists_selector_is_shareable(attr_selector: &AttrSelector<TheSelectorImpl>) -> bool {
    // NB(pcwalton): If you update this, remember to update the corresponding list in
    // `can_share_style_with()` as well.
    common_style_affecting_attributes().iter().any(|common_attr_info| {
        common_attr_info.attr_name == attr_selector.name && match common_attr_info.mode {
            CommonStyleAffectingAttributeMode::IsPresent(_) => true,
            CommonStyleAffectingAttributeMode::IsEqual(..) => false,
        }
    })
}

pub fn attr_equals_selector_is_shareable(attr_selector: &AttrSelector<TheSelectorImpl>,
                                         value: &AttrValue) -> bool {
    // FIXME(pcwalton): Remove once we start actually supporting RTL text. This is in
    // here because the UA style otherwise disables all style sharing completely.
    // FIXME(SimonSapin): should this be the attribute *name* rather than value?
    atom!("dir") == *value ||
    common_style_affecting_attributes().iter().any(|common_attr_info| {
        common_attr_info.attr_name == attr_selector.name && match common_attr_info.mode {
            CommonStyleAffectingAttributeMode::IsEqual(ref target_value, _) => {
                *target_value == *value
            }
            CommonStyleAffectingAttributeMode::IsPresent(_) => false,
        }
    })
}
