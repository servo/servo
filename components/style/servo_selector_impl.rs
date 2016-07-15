/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use attr::{AttrIdentifier, AttrValue};
use element_state::ElementState;
use error_reporting::StdoutErrorReporter;
use parser::ParserContextExtraData;
use restyle_hints::ElementSnapshot;
use selector_impl::{SelectorImplExt, ElementExt, PseudoElementCascadeType, TheSelectorImpl};
use selectors::parser::{AttrSelector, ParserContext, SelectorImpl};
use selectors::{Element, MatchAttrGeneric};
use std::process;
use string_cache::{Atom, Namespace};
use stylesheets::{Stylesheet, Origin};
use url::Url;
use util::opts;
use util::resource_files::read_resource_file;

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
    fn get_user_or_user_agent_stylesheets() -> &'static [Stylesheet] {
        &*USER_OR_USER_AGENT_STYLESHEETS
    }

    #[inline]
    fn get_quirks_mode_stylesheet() -> Option<&'static Stylesheet> {
        Some(&*QUIRKS_MODE_STYLESHEET)
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

    fn get_attr(&self, namespace: &Namespace, name: &Atom) -> Option<&AttrValue> {
        self.attrs.as_ref().unwrap().iter()
            .find(|&&(ref ident, _)| ident.local_name == *name &&
                                     ident.namespace == *namespace)
            .map(|&(_, ref v)| v)
    }

    fn get_attr_ignore_ns(&self, name: &Atom) -> Option<&AttrValue> {
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
        self.get_attr(&ns!(), &atom!("id")).map(|v| v.as_atom().clone())
    }

    fn has_class(&self, name: &Atom) -> bool {
        self.get_attr(&ns!(), &atom!("class"))
            .map_or(false, |v| v.as_tokens().iter().any(|atom| atom == name))
    }

    fn each_class<F>(&self, mut callback: F)
        where F: FnMut(&Atom)
    {
        if let Some(v) = self.get_attr(&ns!(), &atom!("class")) {
            for class in v.as_tokens() {
                callback(class);
            }
        }
    }
}

impl MatchAttrGeneric for ServoElementSnapshot {
    fn match_attr<F>(&self, attr: &AttrSelector, test: F) -> bool
        where F: Fn(&str) -> bool
    {
        use selectors::parser::NamespaceConstraint;
        let html = self.is_html_element_in_html_document;
        let local_name = if html { &attr.lower_name } else { &attr.name };
        match attr.namespace {
            NamespaceConstraint::Specific(ref ns) => self.get_attr(ns, local_name),
            NamespaceConstraint::Any => self.get_attr_ignore_ns(local_name),
        }.map_or(false, |v| test(v))
    }
}

impl<E: Element<Impl=TheSelectorImpl, AttrString=String>> ElementExt for E {
    type Snapshot = ServoElementSnapshot;

    fn is_link(&self) -> bool {
        self.match_non_ts_pseudo_class(NonTSPseudoClass::AnyLink)
    }
}

lazy_static! {
    pub static ref USER_OR_USER_AGENT_STYLESHEETS: Vec<Stylesheet> = {
        let mut stylesheets = vec!();
        // FIXME: presentational-hints.css should be at author origin with zero specificity.
        //        (Does it make a difference?)
        for &filename in &["user-agent.css", "servo.css", "presentational-hints.css"] {
            match read_resource_file(filename) {
                Ok(res) => {
                    let ua_stylesheet = Stylesheet::from_bytes(
                        &res,
                        Url::parse(&format!("chrome://resources/{:?}", filename)).unwrap(),
                        None,
                        None,
                        Origin::UserAgent,
                        Box::new(StdoutErrorReporter),
                        ParserContextExtraData::default());
                    stylesheets.push(ua_stylesheet);
                }
                Err(..) => {
                    error!("Failed to load UA stylesheet {}!", filename);
                    process::exit(1);
                }
            }
        }
        for &(ref contents, ref url) in &opts::get().user_stylesheets {
            stylesheets.push(Stylesheet::from_bytes(
                &contents, url.clone(), None, None, Origin::User, Box::new(StdoutErrorReporter),
                ParserContextExtraData::default()));
        }
        stylesheets
    };
}

lazy_static! {
    pub static ref QUIRKS_MODE_STYLESHEET: Stylesheet = {
        match read_resource_file("quirks-mode.css") {
            Ok(res) => {
                Stylesheet::from_bytes(
                    &res,
                    Url::parse("chrome://resources/quirks-mode.css").unwrap(),
                    None,
                    None,
                    Origin::UserAgent,
                    Box::new(StdoutErrorReporter),
                    ParserContextExtraData::default())
            },
            Err(..) => {
                error!("Stylist failed to load 'quirks-mode.css'!");
                process::exit(1);
            }
        }
    };
}
