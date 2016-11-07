/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use {Atom, Prefix, Namespace, LocalName};
use attr::{AttrIdentifier, AttrValue};
use cssparser::ToCss;
use element_state::ElementState;
use restyle_hints::ElementSnapshot;
use selector_impl::{ElementExt, PseudoElementCascadeType, TheSelectorImpl};
use selector_impl::{attr_equals_selector_is_shareable, attr_exists_selector_is_shareable};
use selectors::{Element, MatchAttrGeneric};
use selectors::parser::{AttrSelector, ParserContext, SelectorImpl};
use std::fmt;

/// NB: If you add to this list, be sure to update `each_pseudo_element` too.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum PseudoElement {
    Before,
    After,
    Selection,
    DetailsSummary,
    DetailsContent,
    ServoInputText,
    ServoTableWrapper,
    ServoAnonymousTableWrapper,
    ServoAnonymousTable,
    ServoAnonymousTableRow,
    ServoAnonymousTableCell,
    ServoAnonymousBlock,
}

impl ToCss for PseudoElement {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        use self::PseudoElement::*;
        dest.write_str(match *self {
            Before => "::before",
            After => "::after",
            Selection => "::selection",
            DetailsSummary => "::-servo-details-summary",
            DetailsContent => "::-servo-details-content",
            ServoInputText => "::-servo-input-text",
            ServoTableWrapper => "::-servo-table-wrapper",
            ServoAnonymousTableWrapper => "::-servo-anonymous-table-wrapper",
            ServoAnonymousTable => "::-servo-anonymous-table",
            ServoAnonymousTableRow => "::-servo-anonymous-table-row",
            ServoAnonymousTableCell => "::-servo-anonymous-table-cell",
            ServoAnonymousBlock => "::-servo-anonymous-block",
        })
    }
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
            PseudoElement::DetailsContent |
            PseudoElement::ServoInputText |
            PseudoElement::ServoTableWrapper |
            PseudoElement::ServoAnonymousTableWrapper |
            PseudoElement::ServoAnonymousTable |
            PseudoElement::ServoAnonymousTableRow |
            PseudoElement::ServoAnonymousTableCell |
            PseudoElement::ServoAnonymousBlock => PseudoElementCascadeType::Precomputed,
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
    Target,
}

impl ToCss for NonTSPseudoClass {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        use self::NonTSPseudoClass::*;
        dest.write_str(match *self {
            AnyLink => ":any-link",
            Link => ":link",
            Visited => ":visited",
            Active => ":active",
            Focus => ":focus",
            Hover => ":hover",
            Enabled => ":enabled",
            Disabled => ":disabled",
            Checked => ":checked",
            Indeterminate => ":indeterminate",
            ReadWrite => ":read-write",
            ReadOnly => ":read-only",
            PlaceholderShown => ":placeholder-shown",
            Target => ":target",
            ServoNonZeroBorder => ":-servo-nonzero-border",
        })
    }
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
            Target => IN_TARGET_STATE,

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
    type PseudoElement = PseudoElement;
    type NonTSPseudoClass = NonTSPseudoClass;

    type AttrValue = String;
    type Identifier = Atom;
    type ClassName = Atom;
    type LocalName = LocalName;
    type NamespacePrefix = Prefix;
    type NamespaceUrl = Namespace;
    type BorrowedLocalName = LocalName;
    type BorrowedNamespaceUrl = Namespace;

    fn attr_exists_selector_is_shareable(attr_selector: &AttrSelector<Self>) -> bool {
        attr_exists_selector_is_shareable(attr_selector)
    }

    fn attr_equals_selector_is_shareable(attr_selector: &AttrSelector<Self>,
                                         value: &Self::AttrValue) -> bool {
        attr_equals_selector_is_shareable(attr_selector, value)
    }

    fn parse_non_ts_pseudo_class(context: &ParserContext<TheSelectorImpl>,
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
            "target" => Target,
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

    fn parse_pseudo_element(context: &ParserContext<TheSelectorImpl>,
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
            "-servo-input-text" => {
                if !context.in_user_agent_stylesheet {
                    return Err(())
                }
                ServoInputText
            },
            "-servo-table-wrapper" => {
                if !context.in_user_agent_stylesheet {
                    return Err(())
                }
                ServoTableWrapper
            },
            "-servo-anonymous-table-wrapper" => {
                if !context.in_user_agent_stylesheet {
                    return Err(())
                }
                ServoAnonymousTableWrapper
            },
            "-servo-anonymous-table" => {
                if !context.in_user_agent_stylesheet {
                    return Err(())
                }
                ServoAnonymousTable
            },
            "-servo-anonymous-table-row" => {
                if !context.in_user_agent_stylesheet {
                    return Err(())
                }
                ServoAnonymousTableRow
            },
            "-servo-anonymous-table-cell" => {
                if !context.in_user_agent_stylesheet {
                    return Err(())
                }
                ServoAnonymousTableCell
            },
            "-servo-anonymous-block" => {
                if !context.in_user_agent_stylesheet {
                    return Err(())
                }
                ServoAnonymousBlock
            },
            _ => return Err(())
        };

        Ok(pseudo_element)
    }
}

impl ServoSelectorImpl {
    #[inline]
    pub fn pseudo_element_cascade_type(pseudo: &PseudoElement) -> PseudoElementCascadeType {
        pseudo.cascade_type()
    }

    #[inline]
    pub fn each_pseudo_element<F>(mut fun: F)
        where F: FnMut(PseudoElement) {
        fun(PseudoElement::Before);
        fun(PseudoElement::After);
        fun(PseudoElement::DetailsContent);
        fun(PseudoElement::DetailsSummary);
        fun(PseudoElement::Selection);
        fun(PseudoElement::ServoInputText);
        fun(PseudoElement::ServoTableWrapper);
        fun(PseudoElement::ServoAnonymousTableWrapper);
        fun(PseudoElement::ServoAnonymousTable);
        fun(PseudoElement::ServoAnonymousTableRow);
        fun(PseudoElement::ServoAnonymousTableCell);
        fun(PseudoElement::ServoAnonymousBlock);
    }

    #[inline]
    pub fn pseudo_class_state_flag(pc: &NonTSPseudoClass) -> ElementState {
        pc.state_flag()
    }

    #[inline]
    pub fn pseudo_is_before_or_after(pseudo: &PseudoElement) -> bool {
        pseudo.is_before_or_after()
    }
}

/// Servo's version of an element snapshot.
#[derive(Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct ServoElementSnapshot {
    pub state: Option<ElementState>,
    pub attrs: Option<Vec<(AttrIdentifier, AttrValue)>>,
    pub is_html_element_in_html_document: bool,
}

impl ServoElementSnapshot {
    pub fn new(is_html_element_in_html_document: bool) -> Self {
        ServoElementSnapshot {
            state: None,
            attrs: None,
            is_html_element_in_html_document: is_html_element_in_html_document,
        }
    }

    fn get_attr(&self, namespace: &Namespace, name: &LocalName) -> Option<&AttrValue> {
        self.attrs.as_ref().unwrap().iter()
            .find(|&&(ref ident, _)| ident.local_name == *name &&
                                     ident.namespace == *namespace)
            .map(|&(_, ref v)| v)
    }

    fn get_attr_ignore_ns(&self, name: &LocalName) -> Option<&AttrValue> {
        self.attrs.as_ref().unwrap().iter()
                  .find(|&&(ref ident, _)| ident.local_name == *name)
                  .map(|&(_, ref v)| v)
    }
}

impl ElementSnapshot for ServoElementSnapshot {
    fn state(&self) -> Option<ElementState> {
        self.state.clone()
    }

    fn has_attrs(&self) -> bool {
        self.attrs.is_some()
    }

    fn id_attr(&self) -> Option<Atom> {
        self.get_attr(&ns!(), &local_name!("id")).map(|v| v.as_atom().clone())
    }

    fn has_class(&self, name: &Atom) -> bool {
        self.get_attr(&ns!(), &local_name!("class"))
            .map_or(false, |v| v.as_tokens().iter().any(|atom| atom == name))
    }

    fn each_class<F>(&self, mut callback: F)
        where F: FnMut(&Atom)
    {
        if let Some(v) = self.get_attr(&ns!(), &local_name!("class")) {
            for class in v.as_tokens() {
                callback(class);
            }
        }
    }
}

impl MatchAttrGeneric for ServoElementSnapshot {
    type Impl = ServoSelectorImpl;

    fn match_attr<F>(&self, attr: &AttrSelector<ServoSelectorImpl>, test: F) -> bool
        where F: Fn(&str) -> bool
    {
        use selectors::parser::NamespaceConstraint;
        let html = self.is_html_element_in_html_document;
        let local_name = if html { &attr.lower_name } else { &attr.name };
        match attr.namespace {
            NamespaceConstraint::Specific(ref ns) => self.get_attr(&ns.url, local_name),
            NamespaceConstraint::Any => self.get_attr_ignore_ns(local_name),
        }.map_or(false, |v| test(v))
    }
}

impl<E: Element<Impl=TheSelectorImpl>> ElementExt for E {
    fn is_link(&self) -> bool {
        self.match_non_ts_pseudo_class(NonTSPseudoClass::AnyLink)
    }
}
