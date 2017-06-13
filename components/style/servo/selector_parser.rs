/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

//! Servo's selector parser.

use {Atom, Prefix, Namespace, LocalName, CaseSensitivityExt};
use attr::{AttrIdentifier, AttrValue};
use cssparser::{Parser as CssParser, ToCss, serialize_identifier};
use dom::{OpaqueNode, TElement, TNode};
use element_state::ElementState;
use fnv::FnvHashMap;
use invalidation::element::element_wrapper::ElementSnapshot;
use selector_parser::{AttrValue as SelectorAttrValue, ElementExt, PseudoElementCascadeType, SelectorParser};
use selectors::Element;
use selectors::attr::{AttrSelectorOperation, NamespaceConstraint, CaseSensitivity};
use selectors::parser::{SelectorMethods, SelectorParseError};
use selectors::visitor::SelectorVisitor;
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::fmt;
use std::fmt::Debug;
use std::mem;
use std::ops::{Deref, DerefMut};
use style_traits::{ParseError, StyleParseError};

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

/// The type used for storing pseudo-class string arguments.
pub type PseudoClassStringArg = Box<str>;

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
    Lang(PseudoClassStringArg),
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
        matches!(*self, NonTSPseudoClass::Lang(..))
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

    #[inline]
    fn is_active_or_hover(pseudo_class: &Self::NonTSPseudoClass) -> bool {
        matches!(*pseudo_class, NonTSPseudoClass::Active |
                                NonTSPseudoClass::Hover)
    }
}

impl<'a, 'i> ::selectors::Parser<'i> for SelectorParser<'a> {
    type Impl = SelectorImpl;
    type Error = StyleParseError<'i>;

    fn parse_non_ts_pseudo_class(&self, name: Cow<'i, str>)
                                 -> Result<NonTSPseudoClass, ParseError<'i>> {
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
                    return Err(SelectorParseError::UnexpectedIdent(
                        "-servo-nonzero-border".into()).into());
                }
                ServoNonZeroBorder
            },
            _ => return Err(SelectorParseError::UnexpectedIdent(name.clone()).into()),
        };

        Ok(pseudo_class)
    }

    fn parse_non_ts_functional_pseudo_class<'t>(&self,
                                                name: Cow<'i, str>,
                                                parser: &mut CssParser<'i, 't>)
                                                -> Result<NonTSPseudoClass, ParseError<'i>> {
        use self::NonTSPseudoClass::*;
        let pseudo_class = match_ignore_ascii_case!{ &name,
            "lang" => {
                Lang(parser.expect_ident_or_string()?.into_owned().into_boxed_str())
            }
            "-servo-case-sensitive-type-attr" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(SelectorParseError::UnexpectedIdent(name.clone()).into());
                }
                ServoCaseSensitiveTypeAttr(Atom::from(parser.expect_ident()?))
            }
            _ => return Err(SelectorParseError::UnexpectedIdent(name.clone()).into())
        };

        Ok(pseudo_class)
    }

    fn parse_pseudo_element(&self, name: Cow<'i, str>) -> Result<PseudoElement, ParseError<'i>> {
        use self::PseudoElement::*;
        let pseudo_element = match_ignore_ascii_case! { &name,
            "before" => Before,
            "after" => After,
            "selection" => Selection,
            "-servo-details-summary" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(SelectorParseError::UnexpectedIdent(name.clone()).into())
                }
                DetailsSummary
            },
            "-servo-details-content" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(SelectorParseError::UnexpectedIdent(name.clone()).into())
                }
                DetailsContent
            },
            "-servo-text" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(SelectorParseError::UnexpectedIdent(name.clone()).into())
                }
                ServoText
            },
            "-servo-input-text" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(SelectorParseError::UnexpectedIdent(name.clone()).into())
                }
                ServoInputText
            },
            "-servo-table-wrapper" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(SelectorParseError::UnexpectedIdent(name.clone()).into())
                }
                ServoTableWrapper
            },
            "-servo-anonymous-table-wrapper" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(SelectorParseError::UnexpectedIdent(name.clone()).into())
                }
                ServoAnonymousTableWrapper
            },
            "-servo-anonymous-table" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(SelectorParseError::UnexpectedIdent(name.clone()).into())
                }
                ServoAnonymousTable
            },
            "-servo-anonymous-table-row" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(SelectorParseError::UnexpectedIdent(name.clone()).into())
                }
                ServoAnonymousTableRow
            },
            "-servo-anonymous-table-cell" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(SelectorParseError::UnexpectedIdent(name.clone()).into())
                }
                ServoAnonymousTableCell
            },
            "-servo-anonymous-block" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(SelectorParseError::UnexpectedIdent(name.clone()).into())
                }
                ServoAnonymousBlock
            },
            "-servo-inline-block-wrapper" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(SelectorParseError::UnexpectedIdent(name.clone()).into())
                }
                ServoInlineBlockWrapper
            },
            "-servo-input-absolute" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(SelectorParseError::UnexpectedIdent(name.clone()).into())
                }
                ServoInlineAbsolute
            },
            _ => return Err(SelectorParseError::UnexpectedIdent(name.clone()).into())

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
    /// Whether the class attribute changed or not.
    pub class_changed: bool,
    /// Whether the id attribute changed or not.
    pub id_changed: bool,
    /// Whether other attributes other than id or class changed or not.
    pub other_attributes_changed: bool,
}

impl ServoElementSnapshot {
    /// Create an empty element snapshot.
    pub fn new(is_html_element_in_html_document: bool) -> Self {
        ServoElementSnapshot {
            state: None,
            attrs: None,
            is_html_element_in_html_document: is_html_element_in_html_document,
            class_changed: false,
            id_changed: false,
            other_attributes_changed: false,
        }
    }

    /// Returns whether the id attribute changed or not.
    pub fn id_changed(&self) -> bool {
        self.id_changed
    }

    /// Returns whether the class attribute changed or not.
    pub fn class_changed(&self) -> bool {
        self.class_changed
    }

    /// Returns whether other attributes other than id or class changed or not.
    pub fn other_attr_changed(&self) -> bool {
        self.other_attributes_changed
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

    fn has_class(&self, name: &Atom, case_sensitivity: CaseSensitivity) -> bool {
        self.get_attr(&ns!(), &local_name!("class"))
            .map_or(false, |v| v.as_tokens().iter().any(|atom| case_sensitivity.eq_atom(atom, name)))
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

    fn lang_attr(&self) -> Option<SelectorAttrValue> {
        self.get_attr(&ns!(xml), &local_name!("lang"))
            .or_else(|| self.get_attr(&ns!(), &local_name!("lang")))
            .map(|v| String::from(v as &str))
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

/// Returns whether the language is matched, as defined by
/// [RFC 4647](https://tools.ietf.org/html/rfc4647#section-3.3.2).
pub fn extended_filtering(tag: &str, range: &str) -> bool {
    range.split(',').any(|lang_range| {
        // step 1
        let mut range_subtags = lang_range.split('\x2d');
        let mut tag_subtags = tag.split('\x2d');

        // step 2
        // Note: [Level-4 spec](https://drafts.csswg.org/selectors/#lang-pseudo) check for wild card
        if let (Some(range_subtag), Some(tag_subtag)) = (range_subtags.next(), tag_subtags.next()) {
            if !(range_subtag.eq_ignore_ascii_case(tag_subtag) || range_subtag.eq_ignore_ascii_case("*")) {
                return false;
            }
        }

        let mut current_tag_subtag = tag_subtags.next();

        // step 3
        for range_subtag in range_subtags {
            // step 3a
            if range_subtag == "*" {
                continue;
            }
            match current_tag_subtag.clone() {
                Some(tag_subtag) => {
                    // step 3c
                    if range_subtag.eq_ignore_ascii_case(tag_subtag) {
                        current_tag_subtag = tag_subtags.next();
                        continue;
                    }
                    // step 3d
                    if tag_subtag.len() == 1 {
                        return false;
                    }
                    // else step 3e - continue with loop
                    current_tag_subtag = tag_subtags.next();
                    if current_tag_subtag.is_none() {
                        return false;
                    }
                },
                // step 3b
                None => {
                    return false;
                }
            }
        }
        // step 4
        true
    })
}
