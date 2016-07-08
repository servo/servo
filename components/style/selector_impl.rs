/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The pseudo-classes and pseudo-elements supported by the style system.

use element_state::ElementState;
use properties::{self, ServoComputedValues};
use selector_matching::{USER_OR_USER_AGENT_STYLESHEETS, QUIRKS_MODE_STYLESHEET};
use selectors::Element;
use selectors::parser::{ParserContext, SelectorImpl};
use std::fmt::Debug;
use stylesheets::Stylesheet;

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

pub trait ElementExt: Element {
    fn is_link(&self) -> bool;
}

// NB: The `Clone` trait is here for convenience due to:
// https://github.com/rust-lang/rust/issues/26925
pub trait SelectorImplExt : SelectorImpl + Clone + Debug + Sized + 'static {
    type ComputedValues: properties::ComputedValues;

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

    fn get_user_or_user_agent_stylesheets() -> &'static [Stylesheet<Self>];

    fn get_quirks_mode_stylesheet() -> Option<&'static Stylesheet<Self>>;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum PseudoElement {
    Before,
    After,
    Selection,
    DetailsSummary,
    DetailsContent,
}

impl PseudoElement {
    #[inline]
    pub fn is_before_or_after(&self) -> bool {
        match *self {
            PseudoElement::Before |
            PseudoElement::After => true,
            _ => false,
        }
    }

    #[inline]
    pub fn cascade_type(&self) -> PseudoElementCascadeType {
        match *self {
            PseudoElement::Before |
            PseudoElement::After |
            PseudoElement::Selection => PseudoElementCascadeType::Eager,
            PseudoElement::DetailsSummary => PseudoElementCascadeType::Lazy,
            PseudoElement::DetailsContent => PseudoElementCascadeType::Precomputed,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
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
    ServoNonZeroBorder,
    ReadWrite,
    ReadOnly,
    PlaceholderShown,
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
            PlaceholderShown => IN_PLACEHOLDER_SHOWN_STATE,

            AnyLink |
            Link |
            Visited |
            ServoNonZeroBorder => ElementState::empty(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct ServoSelectorImpl;

impl SelectorImpl for ServoSelectorImpl {
    type AttrString = String;
    type PseudoElement = PseudoElement;
    type NonTSPseudoClass = NonTSPseudoClass;

    fn parse_non_ts_pseudo_class(context: &ParserContext,
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
            "placeholder-shown" => PlaceholderShown,
            "-servo-nonzero-border" => {
                if !context.in_user_agent_stylesheet {
                    return Err(());
                }
                ServoNonZeroBorder
            },
            _ => return Err(())
        };

        Ok(pseudo_class)
    }

    fn parse_pseudo_element(context: &ParserContext,
                            name: &str) -> Result<PseudoElement, ()> {
        use self::PseudoElement::*;
        let pseudo_element = match_ignore_ascii_case! { name,
            "before" => Before,
            "after" => After,
            "selection" => Selection,
            "-servo-details-summary" => {
                if !context.in_user_agent_stylesheet {
                    return Err(())
                }
                DetailsSummary
            },
            "-servo-details-content" => {
                if !context.in_user_agent_stylesheet {
                    return Err(())
                }
                DetailsContent
            },
            _ => return Err(())
        };

        Ok(pseudo_element)
    }
}

impl SelectorImplExt for ServoSelectorImpl {
    type ComputedValues = ServoComputedValues;

    #[inline]
    fn pseudo_element_cascade_type(pseudo: &PseudoElement) -> PseudoElementCascadeType {
        pseudo.cascade_type()
    }

    #[inline]
    fn each_pseudo_element<F>(mut fun: F)
        where F: FnMut(PseudoElement) {
        fun(PseudoElement::Before);
        fun(PseudoElement::After);
        fun(PseudoElement::DetailsContent);
        fun(PseudoElement::DetailsSummary);
        fun(PseudoElement::Selection);
    }

    #[inline]
    fn pseudo_class_state_flag(pc: &NonTSPseudoClass) -> ElementState {
        pc.state_flag()
    }

    #[inline]
    fn pseudo_is_before_or_after(pseudo: &PseudoElement) -> bool {
        pseudo.is_before_or_after()
    }

    #[inline]
    fn get_user_or_user_agent_stylesheets() -> &'static [Stylesheet<Self>] {
        &*USER_OR_USER_AGENT_STYLESHEETS
    }

    #[inline]
    fn get_quirks_mode_stylesheet() -> Option<&'static Stylesheet<Self>> {
        Some(&*QUIRKS_MODE_STYLESHEET)
    }
}

impl<E: Element<Impl=ServoSelectorImpl, AttrString=String>> ElementExt for E {
    fn is_link(&self) -> bool {
        self.match_non_ts_pseudo_class(NonTSPseudoClass::AnyLink)
    }
}
