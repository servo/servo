/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Per-node data used in style calculation.

use context::{SharedStyleContext, StackLimitChecker};
use dom::TElement;
use invalidation::element::invalidator::InvalidationResult;
use invalidation::element::restyle_hints::RestyleHint;
#[cfg(feature = "gecko")]
use malloc_size_of::MallocSizeOfOps;
use properties::ComputedValues;
use properties::longhands::display::computed_value::T as Display;
use rule_tree::StrongRuleNode;
use selector_parser::{EAGER_PSEUDO_COUNT, PseudoElement, RestyleDamage};
use selectors::NthIndexCache;
use servo_arc::Arc;
use shared_lock::StylesheetGuards;
use smallvec::SmallVec;
use std::fmt;
use std::mem;
use std::ops::{Deref, DerefMut};
use style_resolver::{PrimaryStyle, ResolvedElementStyles, ResolvedStyle};

bitflags! {
    /// Various flags stored on ElementData.
    #[derive(Default)]
    pub struct ElementDataFlags: u8 {
        /// Whether the styles changed for this restyle.
        const WAS_RESTYLED = 1 << 0;
        /// Whether the last traversal of this element did not do
        /// any style computation. This is not true during the initial
        /// styling pass, nor is it true when we restyle (in which case
        /// WAS_RESTYLED is set).
        ///
        /// This bit always corresponds to the last time the element was
        /// traversed, so each traversal simply updates it with the appropriate
        /// value.
        const TRAVERSED_WITHOUT_STYLING = 1 << 1;

        /// Whether the primary style of this element data was reused from
        /// another element via a rule node comparison. This allows us to
        /// differentiate between elements that shared styles because they met
        /// all the criteria of the style sharing cache, compared to elements
        /// that reused style structs via rule node identity.
        ///
        /// The former gives us stronger transitive guarantees that allows us to
        /// apply the style sharing cache to cousins.
        const PRIMARY_STYLE_REUSED_VIA_RULE_NODE = 1 << 2;
    }
}

/// A lazily-allocated list of styles for eagerly-cascaded pseudo-elements.
///
/// We use an Arc so that sharing these styles via the style sharing cache does
/// not require duplicate allocations. We leverage the copy-on-write semantics of
/// Arc::make_mut(), which is free (i.e. does not require atomic RMU operations)
/// in servo_arc.
#[derive(Clone, Debug, Default)]
pub struct EagerPseudoStyles(Option<Arc<EagerPseudoArray>>);

#[derive(Default)]
struct EagerPseudoArray(EagerPseudoArrayInner);
type EagerPseudoArrayInner = [Option<Arc<ComputedValues>>; EAGER_PSEUDO_COUNT];

impl Deref for EagerPseudoArray {
    type Target = EagerPseudoArrayInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EagerPseudoArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Manually implement `Clone` here because the derived impl of `Clone` for
// array types assumes the value inside is `Copy`.
impl Clone for EagerPseudoArray {
    fn clone(&self) -> Self {
        let mut clone = Self::default();
        for i in 0..EAGER_PSEUDO_COUNT {
            clone[i] = self.0[i].clone();
        }
        clone
    }
}

// Override Debug to print which pseudos we have, and substitute the rule node
// for the much-more-verbose ComputedValues stringification.
impl fmt::Debug for EagerPseudoArray {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EagerPseudoArray {{ ")?;
        for i in 0..EAGER_PSEUDO_COUNT {
            if let Some(ref values) = self[i] {
                write!(f, "{:?}: {:?}, ", PseudoElement::from_eager_index(i), &values.rules)?;
            }
        }
        write!(f, "}}")
    }
}

// Can't use [None; EAGER_PSEUDO_COUNT] here because it complains
// about Copy not being implemented for our Arc type.
#[cfg(feature = "gecko")]
const EMPTY_PSEUDO_ARRAY: &'static EagerPseudoArrayInner = &[None, None, None, None];
#[cfg(feature = "servo")]
const EMPTY_PSEUDO_ARRAY: &'static EagerPseudoArrayInner = &[None, None, None];

impl EagerPseudoStyles {
    /// Returns whether there are any pseudo styles.
    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    /// Grabs a reference to the list of styles, if they exist.
    pub fn as_optional_array(&self) -> Option<&EagerPseudoArrayInner> {
        match self.0 {
            None => None,
            Some(ref x) => Some(&x.0),
        }
    }

    /// Grabs a reference to the list of styles or a list of None if
    /// there are no styles to be had.
    pub fn as_array(&self) -> &EagerPseudoArrayInner {
        self.as_optional_array().unwrap_or(EMPTY_PSEUDO_ARRAY)
    }

    /// Returns a reference to the style for a given eager pseudo, if it exists.
    pub fn get(&self, pseudo: &PseudoElement) -> Option<&Arc<ComputedValues>> {
        debug_assert!(pseudo.is_eager());
        self.0.as_ref().and_then(|p| p[pseudo.eager_index()].as_ref())
    }

    /// Sets the style for the eager pseudo.
    pub fn set(&mut self, pseudo: &PseudoElement, value: Arc<ComputedValues>) {
        if self.0.is_none() {
            self.0 = Some(Arc::new(Default::default()));
        }
        let arr = Arc::make_mut(self.0.as_mut().unwrap());
        arr[pseudo.eager_index()] = Some(value);
    }
}

/// The styles associated with a node, including the styles for any
/// pseudo-elements.
#[derive(Clone, Default)]
pub struct ElementStyles {
    /// The element's style.
    pub primary: Option<Arc<ComputedValues>>,
    /// A list of the styles for the element's eagerly-cascaded pseudo-elements.
    pub pseudos: EagerPseudoStyles,
}

impl ElementStyles {
    /// Returns the primary style.
    pub fn get_primary(&self) -> Option<&Arc<ComputedValues>> {
        self.primary.as_ref()
    }

    /// Returns the primary style.  Panic if no style available.
    pub fn primary(&self) -> &Arc<ComputedValues> {
        self.primary.as_ref().unwrap()
    }

    /// Whether this element `display` value is `none`.
    pub fn is_display_none(&self) -> bool {
        self.primary().get_box().clone_display() == Display::None
    }

    #[cfg(feature = "gecko")]
    fn size_of_excluding_cvs(&self, _ops: &mut MallocSizeOfOps) -> usize {
        // As the method name suggests, we don't measures the ComputedValues
        // here, because they are measured on the C++ side.

        // XXX: measure the EagerPseudoArray itself, but not the ComputedValues
        // within it.

        0
    }
}

// We manually implement Debug for ElementStyles so that we can avoid the
// verbose stringification of every property in the ComputedValues. We
// substitute the rule node instead.
impl fmt::Debug for ElementStyles {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ElementStyles {{ primary: {:?}, pseudos: {:?} }}",
               self.primary.as_ref().map(|x| &x.rules), self.pseudos)
    }
}

/// Style system data associated with an Element.
///
/// In Gecko, this hangs directly off the Element. Servo, this is embedded
/// inside of layout data, which itself hangs directly off the Element. In
/// both cases, it is wrapped inside an AtomicRefCell to ensure thread safety.
#[derive(Debug, Default)]
pub struct ElementData {
    /// The styles for the element and its pseudo-elements.
    pub styles: ElementStyles,

    /// The restyle damage, indicating what kind of layout changes are required
    /// afte restyling.
    pub damage: RestyleDamage,

    /// The restyle hint, which indicates whether selectors need to be rematched
    /// for this element, its children, and its descendants.
    pub hint: RestyleHint,

    /// Flags.
    pub flags: ElementDataFlags,
}

/// The kind of restyle that a single element should do.
#[derive(Debug)]
pub enum RestyleKind {
    /// We need to run selector matching plus re-cascade, that is, a full
    /// restyle.
    MatchAndCascade,
    /// We need to recascade with some replacement rule, such as the style
    /// attribute, or animation rules.
    CascadeWithReplacements(RestyleHint),
    /// We only need to recascade, for example, because only inherited
    /// properties in the parent changed.
    CascadeOnly,
}

impl ElementData {
    /// Invalidates style for this element, its descendants, and later siblings,
    /// based on the snapshot of the element that we took when attributes or
    /// state changed.
    pub fn invalidate_style_if_needed<'a, E: TElement>(
        &mut self,
        element: E,
        shared_context: &SharedStyleContext,
        stack_limit_checker: Option<&StackLimitChecker>,
        nth_index_cache: &mut NthIndexCache,
    ) -> InvalidationResult {
        // In animation-only restyle we shouldn't touch snapshot at all.
        if shared_context.traversal_flags.for_animation_only() {
            return InvalidationResult::empty();
        }

        use invalidation::element::invalidator::TreeStyleInvalidator;
        use invalidation::element::state_and_attributes::StateAndAttrInvalidationProcessor;

        debug!("invalidate_style_if_needed: {:?}, flags: {:?}, has_snapshot: {}, \
                handled_snapshot: {}, pseudo: {:?}",
                element,
                shared_context.traversal_flags,
                element.has_snapshot(),
                element.handled_snapshot(),
                element.implemented_pseudo_element());

        if !element.has_snapshot() || element.handled_snapshot() {
            return InvalidationResult::empty();
        }

        let mut xbl_stylists = SmallVec::<[_; 3]>::new();
        // FIXME(emilio): This is wrong, needs to account for ::slotted rules
        // that may apply to elements down the tree.
        let cut_off_inheritance =
            element.each_applicable_non_document_style_rule_data(|data, quirks_mode| {
                xbl_stylists.push((data, quirks_mode))
            });

        let mut processor = StateAndAttrInvalidationProcessor::new(
            shared_context,
            &xbl_stylists,
            cut_off_inheritance,
            element,
            self,
            nth_index_cache,
        );

        let invalidator = TreeStyleInvalidator::new(
            element,
            stack_limit_checker,
            &mut processor,
        );

        let result = invalidator.invalidate();

        unsafe { element.set_handled_snapshot() }
        debug_assert!(element.handled_snapshot());

        result
    }

    /// Returns true if this element has styles.
    #[inline]
    pub fn has_styles(&self) -> bool {
        self.styles.primary.is_some()
    }

    /// Returns this element's styles as resolved styles to use for sharing.
    pub fn share_styles(&self) -> ResolvedElementStyles {
        ResolvedElementStyles {
            primary: self.share_primary_style(),
            pseudos: self.styles.pseudos.clone(),
        }
    }

    /// Returns this element's primary style as a resolved style to use for sharing.
    pub fn share_primary_style(&self) -> PrimaryStyle {
        let reused_via_rule_node =
            self.flags.contains(ElementDataFlags::PRIMARY_STYLE_REUSED_VIA_RULE_NODE);

        PrimaryStyle {
            style: ResolvedStyle(self.styles.primary().clone()),
            reused_via_rule_node,
        }
    }

    /// Sets a new set of styles, returning the old ones.
    pub fn set_styles(&mut self, new_styles: ResolvedElementStyles) -> ElementStyles {
        if new_styles.primary.reused_via_rule_node {
            self.flags.insert(ElementDataFlags::PRIMARY_STYLE_REUSED_VIA_RULE_NODE);
        } else {
            self.flags.remove(ElementDataFlags::PRIMARY_STYLE_REUSED_VIA_RULE_NODE);
        }
        mem::replace(&mut self.styles, new_styles.into())
    }

    /// Returns the kind of restyling that we're going to need to do on this
    /// element, based of the stored restyle hint.
    pub fn restyle_kind(
        &self,
        shared_context: &SharedStyleContext
    ) -> RestyleKind {
        if shared_context.traversal_flags.for_animation_only() {
            return self.restyle_kind_for_animation(shared_context);
        }

        if !self.has_styles() {
            return RestyleKind::MatchAndCascade;
        }

        if self.hint.match_self() {
            return RestyleKind::MatchAndCascade;
        }

        if self.hint.has_replacements() {
            debug_assert!(!self.hint.has_animation_hint(),
                          "Animation only restyle hint should have already processed");
            return RestyleKind::CascadeWithReplacements(self.hint & RestyleHint::replacements());
        }

        debug_assert!(self.hint.has_recascade_self(),
                      "We definitely need to do something: {:?}!", self.hint);
        return RestyleKind::CascadeOnly;
    }

    /// Returns the kind of restyling for animation-only restyle.
    fn restyle_kind_for_animation(
        &self,
        shared_context: &SharedStyleContext,
    ) -> RestyleKind {
        debug_assert!(shared_context.traversal_flags.for_animation_only());
        debug_assert!(self.has_styles(),
                      "Unstyled element shouldn't be traversed during \
                       animation-only traversal");

        // return either CascadeWithReplacements or CascadeOnly in case of
        // animation-only restyle. I.e. animation-only restyle never does
        // selector matching.
        if self.hint.has_animation_hint() {
            return RestyleKind::CascadeWithReplacements(self.hint & RestyleHint::for_animations());
        }

        return RestyleKind::CascadeOnly;
    }

    /// Return true if important rules are different.
    /// We use this to make sure the cascade of off-main thread animations is correct.
    /// Note: Ignore custom properties for now because we only support opacity and transform
    ///       properties for animations running on compositor. Actually, we only care about opacity
    ///       and transform for now, but it's fine to compare all properties and let the user
    ///       the check which properties do they want.
    ///       If it costs too much, get_properties_overriding_animations() should return a set
    ///       containing only opacity and transform properties.
    pub fn important_rules_are_different(
        &self,
        rules: &StrongRuleNode,
        guards: &StylesheetGuards
    ) -> bool {
        debug_assert!(self.has_styles());
        let (important_rules, _custom) =
            self.styles.primary().rules().get_properties_overriding_animations(&guards);
        let (other_important_rules, _custom) = rules.get_properties_overriding_animations(&guards);
        important_rules != other_important_rules
    }

    /// Drops any restyle state from the element.
    ///
    /// FIXME(bholley): The only caller of this should probably just assert that
    /// the hint is empty and call clear_flags_and_damage().
    #[inline]
    pub fn clear_restyle_state(&mut self) {
        self.hint = RestyleHint::empty();
        self.clear_restyle_flags_and_damage();
    }

    /// Drops restyle flags and damage from the element.
    #[inline]
    pub fn clear_restyle_flags_and_damage(&mut self) {
        self.damage = RestyleDamage::empty();
        self.flags.remove(ElementDataFlags::WAS_RESTYLED);
    }

    /// Returns whether this element is going to be reconstructed.
    pub fn reconstructed_self(&self) -> bool {
        self.damage.contains(RestyleDamage::reconstruct())
    }

    /// Mark this element as restyled, which is useful to know whether we need
    /// to do a post-traversal.
    pub fn set_restyled(&mut self) {
        self.flags.insert(ElementDataFlags::WAS_RESTYLED);
        self.flags.remove(ElementDataFlags::TRAVERSED_WITHOUT_STYLING);
    }

    /// Returns true if this element was restyled.
    #[inline]
    pub fn is_restyle(&self) -> bool {
        self.flags.contains(ElementDataFlags::WAS_RESTYLED)
    }

    /// Mark that we traversed this element without computing any style for it.
    pub fn set_traversed_without_styling(&mut self) {
        self.flags.insert(ElementDataFlags::TRAVERSED_WITHOUT_STYLING);
    }

    /// Returns whether the element was traversed without computing any style for
    /// it.
    pub fn traversed_without_styling(&self) -> bool {
        self.flags.contains(ElementDataFlags::TRAVERSED_WITHOUT_STYLING)
    }

    /// Returns whether this element has been part of a restyle.
    #[inline]
    pub fn contains_restyle_data(&self) -> bool {
        self.is_restyle() || !self.hint.is_empty() || !self.damage.is_empty()
    }

    /// Returns whether it is safe to perform cousin sharing based on the ComputedValues
    /// identity of the primary style in this ElementData. There are a few subtle things
    /// to check.
    ///
    /// First, if a parent element was already styled and we traversed past it without
    /// restyling it, that may be because our clever invalidation logic was able to prove
    /// that the styles of that element would remain unchanged despite changes to the id
    /// or class attributes. However, style sharing relies on the strong guarantee that all
    /// the classes and ids up the respective parent chains are identical. As such, if we
    /// skipped styling for one (or both) of the parents on this traversal, we can't share
    /// styles across cousins. Note that this is a somewhat conservative check. We could
    /// tighten it by having the invalidation logic explicitly flag elements for which it
    /// ellided styling.
    ///
    /// Second, we want to only consider elements whose ComputedValues match due to a hit
    /// in the style sharing cache, rather than due to the rule-node-based reuse that
    /// happens later in the styling pipeline. The former gives us the stronger guarantees
    /// we need for style sharing, the latter does not.
    pub fn safe_for_cousin_sharing(&self) -> bool {
        !self.flags.intersects(ElementDataFlags::TRAVERSED_WITHOUT_STYLING |
                               ElementDataFlags::PRIMARY_STYLE_REUSED_VIA_RULE_NODE)
    }

    /// Measures memory usage.
    #[cfg(feature = "gecko")]
    pub fn size_of_excluding_cvs(&self, ops: &mut MallocSizeOfOps) -> usize {
        let n = self.styles.size_of_excluding_cvs(ops);

        // We may measure more fields in the future if DMD says it's worth it.

        n
    }
}
