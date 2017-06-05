/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

//! Servo's selector parser.

use {Atom, Prefix, Namespace, LocalName};
use attr::{AttrIdentifier, AttrValue};
use cssparser::{Parser as CssParser, ToCss, serialize_identifier};
use dom::{OpaqueNode, TElement, TNode};
use element_state::ElementState;
use fnv::FnvHashMap;
use restyle_hints::ElementSnapshot;
use selector_parser::{ElementExt, PseudoElementCascadeType, SelectorParser};
use selectors::Element;
use selectors::attr::{AttrSelectorOperation, NamespaceConstraint};
use selectors::parser::SelectorMethods;
use selectors::visitor::SelectorVisitor;
use std::borrow::Cow;
use std::fmt;
use std::fmt::Debug;
use std::mem;
use std::ops::{Deref, DerefMut};

/// A pseudo-element, both public and private.
///
/// NB: If you add to this list, be sure to update `each_pseudo_element` too.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
#[repr(usize)]
pub enum PseudoElement {
    // Eager pseudos. Keep these first so that eager_index() works.
    After = 0,
    Before,
    Selection,
    // Non-eager pseudos.
    DetailsSummary,
    DetailsContent,
    ServoText,
    ServoInputText,
    ServoTableWrapper,
    ServoAnonymousTableWrapper,
    ServoAnonymousTable,
    ServoAnonymousTableRow,
    ServoAnonymousTableCell,
    ServoAnonymousBlock,
    ServoInlineBlockWrapper,
    ServoInlineAbsolute,
}

impl ::selectors::parser::PseudoElement for PseudoElement {
    type Impl = SelectorImpl;

    fn supports_pseudo_class(&self, _: &NonTSPseudoClass) -> bool {
        false
    }
}

impl ToCss for PseudoElement {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        use self::PseudoElement::*;
        dest.write_str(match *self {
            After => "::after",
            Before => "::before",
            Selection => "::selection",
            DetailsSummary => "::-servo-details-summary",
            DetailsContent => "::-servo-details-content",
            ServoText => "::-servo-text",
            ServoInputText => "::-servo-input-text",
            ServoTableWrapper => "::-servo-table-wrapper",
            ServoAnonymousTableWrapper => "::-servo-anonymous-table-wrapper",
            ServoAnonymousTable => "::-servo-anonymous-table",
            ServoAnonymousTableRow => "::-servo-anonymous-table-row",
            ServoAnonymousTableCell => "::-servo-anonymous-table-cell",
            ServoAnonymousBlock => "::-servo-anonymous-block",
            ServoInlineBlockWrapper => "::-servo-inline-block-wrapper",
            ServoInlineAbsolute => "::-servo-inline-absolute",
        })
    }
}

/// The number of eager pseudo-elements. Keep this in sync with cascade_type.
pub const EAGER_PSEUDO_COUNT: usize = 3;

impl PseudoElement {
    /// Gets the canonical index of this eagerly-cascaded pseudo-element.
    #[inline]
    pub fn eager_index(&self) -> usize {
        debug_assert!(self.is_eager());
        self.clone() as usize
    }

    /// Creates a pseudo-element from an eager index.
    #[inline]
    pub fn from_eager_index(i: usize) -> Self {
        assert!(i < EAGER_PSEUDO_COUNT);
        let result: PseudoElement = unsafe { mem::transmute(i) };
        debug_assert!(result.is_eager());
        result
    }

    /// Whether the current pseudo element is :before or :after.
    #[inline]
    pub fn is_before_or_after(&self) -> bool {
        matches!(*self, PseudoElement::After | PseudoElement::Before)
    }

    /// Whether this pseudo-element is eagerly-cascaded.
    #[inline]
    pub fn is_eager(&self) -> bool {
        self.cascade_type() == PseudoElementCascadeType::Eager
    }

    /// Whether this pseudo-element is lazily-cascaded.
    #[inline]
    pub fn is_lazy(&self) -> bool {
        self.cascade_type() == PseudoElementCascadeType::Lazy
    }

    /// Whether this pseudo-element is precomputed.
    #[inline]
    pub fn is_precomputed(&self) -> bool {
        self.cascade_type() == PseudoElementCascadeType::Precomputed
    }

    /// Returns which kind of cascade type has this pseudo.
    ///
    /// For more info on cascade types, see docs/components/style.md
    ///
    /// Note: Keep this in sync with EAGER_PSEUDO_COUNT.
    #[inline]
    pub fn cascade_type(&self) -> PseudoElementCascadeType {
        match *self {
            PseudoElement::After |
            PseudoElement::Before |
            PseudoElement::Selection => PseudoElementCascadeType::Eager,
            PseudoElement::DetailsSummary => PseudoElementCascadeType::Lazy,
            PseudoElement::DetailsContent |
            PseudoElement::ServoText |
            PseudoElement::ServoInputText |
            PseudoElement::ServoTableWrapper |
            PseudoElement::ServoAnonymousTableWrapper |
            PseudoElement::ServoAnonymousTable |
            PseudoElement::ServoAnonymousTableRow |
            PseudoElement::ServoAnonymousTableCell |
            PseudoElement::ServoAnonymousBlock |
            PseudoElement::ServoInlineBlockWrapper |
            PseudoElement::ServoInlineAbsolute => PseudoElementCascadeType::Precomputed,
        }
    }

    /// Covert non-canonical pseudo-element to canonical one, and keep a
    /// canonical one as it is.
    pub fn canonical(&self) -> PseudoElement {
        self.clone()
    }
}

/// A non tree-structural pseudo-class.
/// See https://drafts.csswg.org/selectors-4/#structural-pseudos
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum NonTSPseudoClass {
    Active,
    AnyLink,
    Checked,
    Disabled,
    Enabled,
    Focus,
    Fullscreen,
    Hover,
    Indeterminate,
    Lang(Box<str>),
    Link,
    PlaceholderShown,
    ReadWrite,
    ReadOnly,
    ServoNonZeroBorder,
    ServoCaseSensitiveTypeAttr(Atom),
    Target,
    Visited,
}

impl ToCss for NonTSPseudoClass {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        use self::NonTSPseudoClass::*;
        match *self {
            Lang(ref lang) => {
                dest.write_str(":lang(")?;
                serialize_identifier(lang, dest)?;
                return dest.write_str(")")
            }
            ServoCaseSensitiveTypeAttr(ref value) => {
                dest.write_str(":-servo-case-sensitive-type-attr(")?;
                serialize_identifier(value, dest)?;
                return dest.write_str(")")
            }
            _ => {}
        }

        dest.write_str(match *self {
            Active => ":active",
            AnyLink => ":any-link",
            Checked => ":checked",
            Disabled => ":disabled",
            Enabled => ":enabled",
            Focus => ":focus",
            Fullscreen => ":fullscreen",
            Hover => ":hover",
            Indeterminate => ":indeterminate",
            Link => ":link",
            PlaceholderShown => ":placeholder-shown",
            ReadWrite => ":read-write",
            ReadOnly => ":read-only",
            ServoNonZeroBorder => ":-servo-nonzero-border",
            Target => ":target",
            Visited => ":visited",
            Lang(_) |
            ServoCaseSensitiveTypeAttr(_) => unreachable!(),
        })
    }
}

impl SelectorMethods for NonTSPseudoClass {
    type Impl = SelectorImpl;


    fn visit<V>(&self, _: &mut V) -> bool
        where V: SelectorVisitor<Impl = Self::Impl>
    {
        true
    }
}

impl NonTSPseudoClass {
    /// Gets a given state flag for this pseudo-class. This is used to do
    /// selector matching, and it's set from the DOM.
    pub fn state_flag(&self) -> ElementState {
        use element_state::*;
        use self::NonTSPseudoClass::*;
        match *self {
            Active => IN_ACTIVE_STATE,
            Focus => IN_FOCUS_STATE,
            Fullscreen => IN_FULLSCREEN_STATE,
            Hover => IN_HOVER_STATE,
            Enabled => IN_ENABLED_STATE,
            Disabled => IN_DISABLED_STATE,
            Checked => IN_CHECKED_STATE,
            Indeterminate => IN_INDETERMINATE_STATE,
            ReadOnly | ReadWrite => IN_READ_WRITE_STATE,
            PlaceholderShown => IN_PLACEHOLDER_SHOWN_STATE,
            Target => IN_TARGET_STATE,

            AnyLink |
            Lang(_) |
            Link |
            Visited |
            ServoNonZeroBorder |
            ServoCaseSensitiveTypeAttr(_) => ElementState::empty(),
        }
    }

    /// Returns true if the given pseudoclass should trigger style sharing cache revalidation.
    pub fn needs_cache_revalidation(&self) -> bool {
        self.state_flag().is_empty()
    }

    /// Returns true if the evaluation of the pseudo-class depends on the
    /// element's attributes.
    pub fn is_attr_based(&self) -> bool {
        false
    }
}

/// The abstract struct we implement the selector parser implementation on top
/// of.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct SelectorImpl;

impl ::selectors::SelectorImpl for SelectorImpl {
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
}

impl<'a> ::selectors::Parser for SelectorParser<'a> {
    type Impl = SelectorImpl;

    fn parse_non_ts_pseudo_class(&self, name: Cow<str>) -> Result<NonTSPseudoClass, ()> {
        use self::NonTSPseudoClass::*;
        let pseudo_class = match_ignore_ascii_case! { &name,
            "active" => Active,
            "any-link" => AnyLink,
            "checked" => Checked,
            "disabled" => Disabled,
            "enabled" => Enabled,
            "focus" => Focus,
            "fullscreen" => Fullscreen,
            "hover" => Hover,
            "indeterminate" => Indeterminate,
            "link" => Link,
            "placeholder-shown" => PlaceholderShown,
            "read-write" => ReadWrite,
            "read-only" => ReadOnly,
            "target" => Target,
            "visited" => Visited,
            "-servo-nonzero-border" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(());
                }
                ServoNonZeroBorder
            },
            _ => return Err(())
        };

        Ok(pseudo_class)
    }

    fn parse_non_ts_functional_pseudo_class(&self,
                                            name: Cow<str>,
                                            parser: &mut CssParser)
                                            -> Result<NonTSPseudoClass, ()> {
        use self::NonTSPseudoClass::*;
        let pseudo_class = match_ignore_ascii_case!{ &name,
            "lang" => {
                Lang(parser.expect_ident_or_string()?.into_owned().into_boxed_str())
            }
            "-servo-case-sensitive-type-attr" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(());
                }
                ServoCaseSensitiveTypeAttr(Atom::from(parser.expect_ident()?))
            }
            _ => return Err(())
        };

        Ok(pseudo_class)
    }

    fn parse_pseudo_element(&self, name: Cow<str>)
                            -> Result<PseudoElement, ()> {
        use self::PseudoElement::*;
        let pseudo_element = match_ignore_ascii_case! { &name,
            "before" => Before,
            "after" => After,
            "selection" => Selection,
            "-servo-details-summary" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(())
                }
                DetailsSummary
            },
            "-servo-details-content" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(())
                }
                DetailsContent
            },
            "-servo-text" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(())
                }
                ServoText
            },
            "-servo-input-text" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(())
                }
                ServoInputText
            },
            "-servo-table-wrapper" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(())
                }
                ServoTableWrapper
            },
            "-servo-anonymous-table-wrapper" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(())
                }
                ServoAnonymousTableWrapper
            },
            "-servo-anonymous-table" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(())
                }
                ServoAnonymousTable
            },
            "-servo-anonymous-table-row" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(())
                }
                ServoAnonymousTableRow
            },
            "-servo-anonymous-table-cell" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(())
                }
                ServoAnonymousTableCell
            },
            "-servo-anonymous-block" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(())
                }
                ServoAnonymousBlock
            },
            "-servo-inline-block-wrapper" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(())
                }
                ServoInlineBlockWrapper
            },
            "-servo-input-absolute" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(())
                }
                ServoInlineAbsolute
            },
            _ => return Err(())
        };

        Ok(pseudo_element)
    }

    fn default_namespace(&self) -> Option<Namespace> {
        self.namespaces.default.as_ref().map(|&(ref ns, _)| ns.clone())
    }

    fn namespace_for_prefix(&self, prefix: &Prefix) -> Option<Namespace> {
        self.namespaces.prefixes.get(prefix).map(|&(ref ns, _)| ns.clone())
    }
}

impl SelectorImpl {
    /// Returns the pseudo-element cascade type of the given `pseudo`.
    #[inline]
    pub fn pseudo_element_cascade_type(pseudo: &PseudoElement) -> PseudoElementCascadeType {
        pseudo.cascade_type()
    }

    /// A helper to traverse each eagerly cascaded pseudo-element, executing
    /// `fun` on it.
    #[inline]
    pub fn each_eagerly_cascaded_pseudo_element<F>(mut fun: F)
        where F: FnMut(PseudoElement),
    {
        for i in 0..EAGER_PSEUDO_COUNT {
            fun(PseudoElement::from_eager_index(i));
        }
    }

    /// Executes `fun` for each pseudo-element.
    #[inline]
    pub fn each_pseudo_element<F>(mut fun: F)
        where F: FnMut(PseudoElement),
    {
        fun(PseudoElement::Before);
        fun(PseudoElement::After);
        fun(PseudoElement::DetailsContent);
        fun(PseudoElement::DetailsSummary);
        fun(PseudoElement::Selection);
        fun(PseudoElement::ServoText);
        fun(PseudoElement::ServoInputText);
        fun(PseudoElement::ServoTableWrapper);
        fun(PseudoElement::ServoAnonymousTableWrapper);
        fun(PseudoElement::ServoAnonymousTable);
        fun(PseudoElement::ServoAnonymousTableRow);
        fun(PseudoElement::ServoAnonymousTableCell);
        fun(PseudoElement::ServoAnonymousBlock);
        fun(PseudoElement::ServoInlineBlockWrapper);
        fun(PseudoElement::ServoInlineAbsolute);
    }

    /// Returns the pseudo-class state flag for selector matching.
    #[inline]
    pub fn pseudo_class_state_flag(pc: &NonTSPseudoClass) -> ElementState {
        pc.state_flag()
    }
}

/// A map from elements to snapshots for the Servo style back-end.
#[derive(Debug)]
pub struct SnapshotMap(FnvHashMap<OpaqueNode, ServoElementSnapshot>);

impl SnapshotMap {
    /// Create a new empty `SnapshotMap`.
    pub fn new() -> Self {
        SnapshotMap(FnvHashMap::default())
    }

    /// Get a snapshot given an element.
    pub fn get<T: TElement>(&self, el: &T) -> Option<&ServoElementSnapshot> {
        self.0.get(&el.as_node().opaque())
    }
}

impl Deref for SnapshotMap {
    type Target = FnvHashMap<OpaqueNode, ServoElementSnapshot>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SnapshotMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Servo's version of an element snapshot.
#[derive(Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct ServoElementSnapshot {
    /// The stored state of the element.
    pub state: Option<ElementState>,
    /// The set of stored attributes and its values.
    pub attrs: Option<Vec<(AttrIdentifier, AttrValue)>>,
    /// Whether this element is an HTML element in an HTML document.
    pub is_html_element_in_html_document: bool,
}

impl ServoElementSnapshot {
    /// Create an empty element snapshot.
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

    fn any_attr_ignore_ns<F>(&self, name: &LocalName, mut f: F) -> bool
    where F: FnMut(&AttrValue) -> bool {
        self.attrs.as_ref().unwrap().iter()
                  .any(|&(ref ident, ref v)| ident.local_name == *name && f(v))
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
        where F: FnMut(&Atom),
    {
        if let Some(v) = self.get_attr(&ns!(), &local_name!("class")) {
            for class in v.as_tokens() {
                callback(class);
            }
        }
    }
}

impl ServoElementSnapshot {
    /// selectors::Element::attr_matches
    pub fn attr_matches(&self,
                        ns: &NamespaceConstraint<&Namespace>,
                        local_name: &LocalName,
                        operation: &AttrSelectorOperation<&String>)
                        -> bool {
        match *ns {
            NamespaceConstraint::Specific(ref ns) => {
                self.get_attr(ns, local_name)
                    .map_or(false, |value| value.eval_selector(operation))
            }
            NamespaceConstraint::Any => {
                self.any_attr_ignore_ns(local_name, |value| value.eval_selector(operation))
            }
        }
    }
}

impl<E: Element<Impl=SelectorImpl> + Debug> ElementExt for E {
    #[inline]
    fn matches_user_and_author_rules(&self) -> bool {
        true
    }
}
