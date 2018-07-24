/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

//! Servo's selector parser.

use {Atom, CaseSensitivityExt, LocalName, Namespace, Prefix};
use attr::{AttrIdentifier, AttrValue};
use cssparser::{serialize_identifier, CowRcStr, Parser as CssParser, SourceLocation, ToCss};
use dom::{OpaqueNode, TElement, TNode};
use element_state::{DocumentState, ElementState};
use fnv::FnvHashMap;
use invalidation::element::document_state::InvalidationMatchingData;
use invalidation::element::element_wrapper::ElementSnapshot;
use properties::{ComputedValues, PropertyFlags};
use properties::longhands::display::computed_value::T as Display;
use selector_parser::{AttrValue as SelectorAttrValue, PseudoElementCascadeType, SelectorParser};
use selectors::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use selectors::parser::{SelectorParseErrorKind, Visit};
use selectors::visitor::SelectorVisitor;
use std::fmt;
use std::mem;
use std::ops::{Deref, DerefMut};
use style_traits::{ParseError, StyleParseErrorKind};

/// A pseudo-element, both public and private.
///
/// NB: If you add to this list, be sure to update `each_simple_pseudo_element` too.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
#[allow(missing_docs)]
#[repr(usize)]
pub enum PseudoElement {
    // Eager pseudos. Keep these first so that eager_index() works.
    After = 0,
    Before,
    Selection,
    // If/when :first-letter is added, update is_first_letter accordingly.

    // If/when :first-line is added, update is_first_line accordingly.

    // If/when ::first-letter, ::first-line, or ::placeholder are added, adjust
    // our property_restriction implementation to do property filtering for
    // them.  Also, make sure the UA sheet has the !important rules some of the
    // APPLIES_TO_PLACEHOLDER properties expect!

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

/// The count of all pseudo-elements.
pub const PSEUDO_COUNT: usize = PseudoElement::ServoInlineAbsolute as usize + 1;

impl ::selectors::parser::PseudoElement for PseudoElement {
    type Impl = SelectorImpl;

    fn supports_pseudo_class(&self, _: &NonTSPseudoClass) -> bool {
        false
    }
}

impl ToCss for PseudoElement {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
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

    /// An index for this pseudo-element to be indexed in an enumerated array.
    #[inline]
    pub fn index(&self) -> usize {
        self.clone() as usize
    }

    /// An array of `None`, one per pseudo-element.
    pub fn pseudo_none_array<T>() -> [Option<T>; PSEUDO_COUNT] {
        Default::default()
    }

    /// Creates a pseudo-element from an eager index.
    #[inline]
    pub fn from_eager_index(i: usize) -> Self {
        assert!(i < EAGER_PSEUDO_COUNT);
        let result: PseudoElement = unsafe { mem::transmute(i) };
        debug_assert!(result.is_eager());
        result
    }

    /// Whether the current pseudo element is ::before or ::after.
    #[inline]
    pub fn is_before_or_after(&self) -> bool {
        self.is_before() || self.is_after()
    }

    /// Whether this pseudo-element is the ::before pseudo.
    #[inline]
    pub fn is_before(&self) -> bool {
        *self == PseudoElement::Before
    }

    /// Whether this pseudo-element is the ::after pseudo.
    #[inline]
    pub fn is_after(&self) -> bool {
        *self == PseudoElement::After
    }

    /// Whether the current pseudo element is :first-letter
    #[inline]
    pub fn is_first_letter(&self) -> bool {
        false
    }

    /// Whether the current pseudo element is :first-line
    #[inline]
    pub fn is_first_line(&self) -> bool {
        false
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

    /// Whether this pseudo-element is for an anonymous box.
    pub fn is_anon_box(&self) -> bool {
        self.is_precomputed()
    }

    /// Whether this pseudo-element skips flex/grid container display-based
    /// fixup.
    #[inline]
    pub fn skip_item_display_fixup(&self) -> bool {
        !self.is_before_or_after()
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
            PseudoElement::After | PseudoElement::Before | PseudoElement::Selection => {
                PseudoElementCascadeType::Eager
            },
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

    /// For most (but not all) anon-boxes, we inherit all values from the
    /// parent, this is the hook in the style system to allow this.
    ///
    /// FIXME(emilio): It's likely that this is broken in a variety of
    /// situations, and what it really wants is just inherit some reset
    /// properties...  Also, I guess it just could do all: inherit on the
    /// stylesheet, though chances are that'd be kinda slow if we don't cache
    /// them...
    pub fn inherits_all(&self) -> bool {
        match *self {
            PseudoElement::After |
            PseudoElement::Before |
            PseudoElement::Selection |
            PseudoElement::DetailsContent |
            PseudoElement::DetailsSummary |
            // Anonymous table flows shouldn't inherit their parents properties in order
            // to avoid doubling up styles such as transformations.
            PseudoElement::ServoAnonymousTableCell |
            PseudoElement::ServoAnonymousTableRow |
            PseudoElement::ServoText |
            PseudoElement::ServoInputText => false,

            // For tables, we do want style to inherit, because TableWrapper is
            // responsible for handling clipping and scrolling, while Table is
            // responsible for creating stacking contexts.
            //
            // StackingContextCollectionFlags makes sure this is processed
            // properly.
            PseudoElement::ServoAnonymousTable |
            PseudoElement::ServoAnonymousTableWrapper |
            PseudoElement::ServoTableWrapper |
            PseudoElement::ServoAnonymousBlock |
            PseudoElement::ServoInlineBlockWrapper |
            PseudoElement::ServoInlineAbsolute => true,
        }
    }

    /// Covert non-canonical pseudo-element to canonical one, and keep a
    /// canonical one as it is.
    pub fn canonical(&self) -> PseudoElement {
        self.clone()
    }

    /// Stub, only Gecko needs this
    pub fn pseudo_info(&self) {
        ()
    }

    /// Property flag that properties must have to apply to this pseudo-element.
    #[inline]
    pub fn property_restriction(&self) -> Option<PropertyFlags> {
        None
    }

    /// Whether this pseudo-element should actually exist if it has
    /// the given styles.
    pub fn should_exist(&self, style: &ComputedValues) -> bool {
        let display = style.get_box().clone_display();
        if display == Display::None {
            return false;
        }
        if self.is_before_or_after() && style.ineffective_content_property() {
            return false;
        }

        true
    }
}

/// The type used for storing pseudo-class string arguments.
pub type PseudoClassStringArg = Box<str>;

/// A non tree-structural pseudo-class.
/// See https://drafts.csswg.org/selectors-4/#structural-pseudos
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
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

impl ::selectors::parser::NonTSPseudoClass for NonTSPseudoClass {
    type Impl = SelectorImpl;

    #[inline]
    fn is_active_or_hover(&self) -> bool {
        matches!(*self, NonTSPseudoClass::Active | NonTSPseudoClass::Hover)
    }
}

impl ToCss for NonTSPseudoClass {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        use self::NonTSPseudoClass::*;
        match *self {
            Lang(ref lang) => {
                dest.write_str(":lang(")?;
                serialize_identifier(lang, dest)?;
                return dest.write_str(")");
            },
            ServoCaseSensitiveTypeAttr(ref value) => {
                dest.write_str(":-servo-case-sensitive-type-attr(")?;
                serialize_identifier(value, dest)?;
                return dest.write_str(")");
            },
            _ => {},
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
            Lang(_) | ServoCaseSensitiveTypeAttr(_) => unreachable!(),
        })
    }
}

impl Visit for NonTSPseudoClass {
    type Impl = SelectorImpl;

    fn visit<V>(&self, _: &mut V) -> bool
    where
        V: SelectorVisitor<Impl = Self::Impl>,
    {
        true
    }
}

impl NonTSPseudoClass {
    /// Gets a given state flag for this pseudo-class. This is used to do
    /// selector matching, and it's set from the DOM.
    pub fn state_flag(&self) -> ElementState {
        use element_state::ElementState;
        use self::NonTSPseudoClass::*;
        match *self {
            Active => ElementState::IN_ACTIVE_STATE,
            Focus => ElementState::IN_FOCUS_STATE,
            Fullscreen => ElementState::IN_FULLSCREEN_STATE,
            Hover => ElementState::IN_HOVER_STATE,
            Enabled => ElementState::IN_ENABLED_STATE,
            Disabled => ElementState::IN_DISABLED_STATE,
            Checked => ElementState::IN_CHECKED_STATE,
            Indeterminate => ElementState::IN_INDETERMINATE_STATE,
            ReadOnly | ReadWrite => ElementState::IN_READ_WRITE_STATE,
            PlaceholderShown => ElementState::IN_PLACEHOLDER_SHOWN_STATE,
            Target => ElementState::IN_TARGET_STATE,

            AnyLink |
            Lang(_) |
            Link |
            Visited |
            ServoNonZeroBorder |
            ServoCaseSensitiveTypeAttr(_) => ElementState::empty(),
        }
    }

    /// Get the document state flag associated with a pseudo-class, if any.
    pub fn document_state_flag(&self) -> DocumentState {
        DocumentState::empty()
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
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
pub struct SelectorImpl;

impl ::selectors::SelectorImpl for SelectorImpl {
    type PseudoElement = PseudoElement;
    type NonTSPseudoClass = NonTSPseudoClass;

    type ExtraMatchingData = InvalidationMatchingData;
    type AttrValue = String;
    type Identifier = Atom;
    type ClassName = Atom;
    type LocalName = LocalName;
    type NamespacePrefix = Prefix;
    type NamespaceUrl = Namespace;
    type BorrowedLocalName = LocalName;
    type BorrowedNamespaceUrl = Namespace;
}

impl<'a, 'i> ::selectors::Parser<'i> for SelectorParser<'a> {
    type Impl = SelectorImpl;
    type Error = StyleParseErrorKind<'i>;

    fn parse_non_ts_pseudo_class(
        &self,
        location: SourceLocation,
        name: CowRcStr<'i>,
    ) -> Result<NonTSPseudoClass, ParseError<'i>> {
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
                    return Err(location.new_custom_error(
                        SelectorParseErrorKind::UnexpectedIdent("-servo-nonzero-border".into())
                    ))
                }
                ServoNonZeroBorder
            },
            _ => return Err(location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone()))),
        };

        Ok(pseudo_class)
    }

    fn parse_non_ts_functional_pseudo_class<'t>(
        &self,
        name: CowRcStr<'i>,
        parser: &mut CssParser<'i, 't>,
    ) -> Result<NonTSPseudoClass, ParseError<'i>> {
        use self::NonTSPseudoClass::*;
        let pseudo_class = match_ignore_ascii_case!{ &name,
            "lang" => {
                Lang(parser.expect_ident_or_string()?.as_ref().into())
            }
            "-servo-case-sensitive-type-attr" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(parser.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone())));
                }
                ServoCaseSensitiveTypeAttr(Atom::from(parser.expect_ident()?.as_ref()))
            }
            _ => return Err(parser.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone())))
        };

        Ok(pseudo_class)
    }

    fn parse_pseudo_element(
        &self,
        location: SourceLocation,
        name: CowRcStr<'i>,
    ) -> Result<PseudoElement, ParseError<'i>> {
        use self::PseudoElement::*;
        let pseudo_element = match_ignore_ascii_case! { &name,
            "before" => Before,
            "after" => After,
            "selection" => Selection,
            "-servo-details-summary" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone())))
                }
                DetailsSummary
            },
            "-servo-details-content" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone())))
                }
                DetailsContent
            },
            "-servo-text" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone())))
                }
                ServoText
            },
            "-servo-input-text" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone())))
                }
                ServoInputText
            },
            "-servo-table-wrapper" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone())))
                }
                ServoTableWrapper
            },
            "-servo-anonymous-table-wrapper" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone())))
                }
                ServoAnonymousTableWrapper
            },
            "-servo-anonymous-table" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone())))
                }
                ServoAnonymousTable
            },
            "-servo-anonymous-table-row" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone())))
                }
                ServoAnonymousTableRow
            },
            "-servo-anonymous-table-cell" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone())))
                }
                ServoAnonymousTableCell
            },
            "-servo-anonymous-block" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone())))
                }
                ServoAnonymousBlock
            },
            "-servo-inline-block-wrapper" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone())))
                }
                ServoInlineBlockWrapper
            },
            "-servo-input-absolute" => {
                if !self.in_user_agent_stylesheet() {
                    return Err(location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone())))
                }
                ServoInlineAbsolute
            },
            _ => return Err(location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone())))

        };

        Ok(pseudo_element)
    }

    fn default_namespace(&self) -> Option<Namespace> {
        self.namespaces.default.as_ref().map(|ns| ns.clone())
    }

    fn namespace_for_prefix(&self, prefix: &Prefix) -> Option<Namespace> {
        self.namespaces.prefixes.get(prefix).cloned()
    }
}

impl SelectorImpl {
    /// A helper to traverse each eagerly cascaded pseudo-element, executing
    /// `fun` on it.
    #[inline]
    pub fn each_eagerly_cascaded_pseudo_element<F>(mut fun: F)
    where
        F: FnMut(PseudoElement),
    {
        for i in 0..EAGER_PSEUDO_COUNT {
            fun(PseudoElement::from_eager_index(i));
        }
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
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
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
        self.attrs
            .as_ref()
            .unwrap()
            .iter()
            .find(|&&(ref ident, _)| ident.local_name == *name && ident.namespace == *namespace)
            .map(|&(_, ref v)| v)
    }

    fn any_attr_ignore_ns<F>(&self, name: &LocalName, mut f: F) -> bool
    where
        F: FnMut(&AttrValue) -> bool,
    {
        self.attrs
            .as_ref()
            .unwrap()
            .iter()
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

    fn id_attr(&self) -> Option<&Atom> {
        self.get_attr(&ns!(), &local_name!("id"))
            .map(|v| v.as_atom())
    }

    fn has_class(&self, name: &Atom, case_sensitivity: CaseSensitivity) -> bool {
        self.get_attr(&ns!(), &local_name!("class"))
            .map_or(false, |v| {
                v.as_tokens()
                    .iter()
                    .any(|atom| case_sensitivity.eq_atom(atom, name))
            })
    }

    fn each_class<F>(&self, mut callback: F)
    where
        F: FnMut(&Atom),
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
    pub fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&Namespace>,
        local_name: &LocalName,
        operation: &AttrSelectorOperation<&String>,
    ) -> bool {
        match *ns {
            NamespaceConstraint::Specific(ref ns) => self.get_attr(ns, local_name)
                .map_or(false, |value| value.eval_selector(operation)),
            NamespaceConstraint::Any => {
                self.any_attr_ignore_ns(local_name, |value| value.eval_selector(operation))
            },
        }
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
            if !(range_subtag.eq_ignore_ascii_case(tag_subtag) ||
                range_subtag.eq_ignore_ascii_case("*"))
            {
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
                },
            }
        }
        // step 4
        true
    })
}
