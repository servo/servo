/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use element_state::ElementState;
use selector_matching::{USER_OR_USER_AGENT_STYLESHEETS, QUIRKS_MODE_STYLESHEET};
use selectors::Element;
use selectors::parser::{ParserContext, SelectorImpl};
use stylesheets::Stylesheet;

pub trait ElementExt: Element {
    fn is_link(&self) -> bool;
}

pub trait SelectorImplExt : SelectorImpl + Sized {
    fn each_eagerly_cascaded_pseudo_element<F>(mut fun: F)
        where F: FnMut(<Self as SelectorImpl>::PseudoElement);

    fn pseudo_class_state_flag(pc: &Self::NonTSPseudoClass) -> ElementState;

    fn get_user_or_user_agent_stylesheets() -> &'static [Stylesheet<Self>];

    fn get_quirks_mode_stylesheet() -> Option<&'static Stylesheet<Self>>;
}

#[derive(Clone, Debug, PartialEq, Eq, HeapSizeOf, Hash)]
pub enum PseudoElement {
    Before,
    After,
    Selection,
    DetailsSummary,
    DetailsContent,
}

#[derive(Clone, Debug, PartialEq, Eq, HeapSizeOf, Hash)]
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

            AnyLink |
            Link |
            Visited |
            ServoNonZeroBorder => ElementState::empty(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, HeapSizeOf)]
pub struct ServoSelectorImpl;

impl SelectorImpl for ServoSelectorImpl {
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
            "-servo-details-summary" => if context.in_user_agent_stylesheet {
                DetailsSummary
            } else {
                return Err(())
            },
            "-servo-details-content" => if context.in_user_agent_stylesheet {
                DetailsContent
            } else {
                return Err(())
            },
            _ => return Err(())
        };

        Ok(pseudo_element)
    }
}

impl<E: Element<Impl=ServoSelectorImpl>> ElementExt for E {
    fn is_link(&self) -> bool {
        self.match_non_ts_pseudo_class(NonTSPseudoClass::AnyLink)
    }
}

impl SelectorImplExt for ServoSelectorImpl {
    #[inline]
    fn each_eagerly_cascaded_pseudo_element<F>(mut fun: F)
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
    fn get_user_or_user_agent_stylesheets() -> &'static [Stylesheet<Self>] {
        &*USER_OR_USER_AGENT_STYLESHEETS
    }

    #[inline]
    fn get_quirks_mode_stylesheet() -> Option<&'static Stylesheet<Self>> {
        Some(&*QUIRKS_MODE_STYLESHEET)
    }
}
