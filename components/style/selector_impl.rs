/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use selectors::parser::{ParserContext, SelectorImpl};
use element_state::ElementState;

#[derive(Clone, Debug, PartialEq, HeapSizeOf)]
pub enum PseudoElement {
    Before,
    After,
}

#[derive(Clone, Debug, PartialEq, HeapSizeOf)]
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
        use self::NonTSPseudoClass::*;
        use element_state::*;
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
            "-servo-non-zero-border" => {
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
        match_ignore_ascii_case! { name,
            "before" => Ok(Before),
            "after" => Ok(After),
            _ => Err(())
        }
    }
}
