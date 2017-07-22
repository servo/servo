/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Per-node data used in style calculation.

use context::SharedStyleContext;
use dom::TElement;
use invalidation::element::restyle_hints::RestyleHint;
use properties::ComputedValues;
use properties::longhands::display::computed_value as display;
use rule_tree::StrongRuleNode;
use selector_parser::{EAGER_PSEUDO_COUNT, PseudoElement, RestyleDamage};
use servo_arc::Arc;
use shared_lock::StylesheetGuards;
use std::fmt;
use std::ops::{Deref, DerefMut};

bitflags! {
    flags RestyleFlags: u8 {
        /// Whether the styles changed for this restyle.
        const WAS_RESTYLED = 1 << 0,
        /// Whether we reframed/reconstructed any ancestor or self.
        const ANCESTOR_WAS_RECONSTRUCTED = 1 << 1,
    }
}

/// Transient data used by the restyle algorithm. This structure is instantiated
/// either before or during restyle traversal, and is cleared at the end of node
/// processing.
#[derive(Debug)]
pub struct RestyleData {
    /// The restyle hint, which indicates whether selectors need to be rematched
    /// for this element, its children, and its descendants.
    pub hint: RestyleHint,

    /// A few flags to have in mind.
    flags: RestyleFlags,

    /// The restyle damage, indicating what kind of layout changes are required
    /// afte restyling.
    pub damage: RestyleDamage,
}

impl Default for RestyleData {
    fn default() -> Self {
        Self::new()
    }
}

impl RestyleData {
    fn new() -> Self {
        Self {
            hint: RestyleHint::empty(),
            flags: RestyleFlags::empty(),
            damage: RestyleDamage::empty(),
        }
    }

    /// Clear all the restyle state associated with this element.
    fn clear(&mut self) {
        *self = Self::new();
    }

    /// Clear restyle flags and damage.
    fn clear_flags_and_damage(&mut self) {
        self.damage = RestyleDamage::empty();
        self.flags = RestyleFlags::empty();
    }

    /// Returns whether this element or any ancestor is going to be
    /// reconstructed.
    pub fn reconstructed_self_or_ancestor(&self) -> bool {
        self.reconstructed_ancestor() || self.reconstructed_self()
    }

    /// Returns whether this element is going to be reconstructed.
    pub fn reconstructed_self(&self) -> bool {
        self.damage.contains(RestyleDamage::reconstruct())
    }

    /// Returns whether any ancestor of this element is going to be
    /// reconstructed.
    fn reconstructed_ancestor(&self) -> bool {
        self.flags.contains(ANCESTOR_WAS_RECONSTRUCTED)
    }

    /// Sets the flag that tells us whether we've reconstructed an ancestor.
    pub fn set_reconstructed_ancestor(&mut self) {
        // If it weren't for animation-only traversals, we could assert
        // `!self.reconstructed_ancestor()` here.
        self.flags.insert(ANCESTOR_WAS_RECONSTRUCTED);
    }

    /// Mark this element as restyled, which is useful to know whether we need
    /// to do a post-traversal.
    pub fn set_restyled(&mut self) {
        self.flags.insert(WAS_RESTYLED);
    }

    /// Mark this element as restyled, which is useful to know whether we need
    /// to do a post-traversal.
    pub fn is_restyle(&self) -> bool {
        self.flags.contains(WAS_RESTYLED)
    }

    /// Returns whether this element has been part of a restyle.
    pub fn contains_restyle_data(&self) -> bool {
        self.is_restyle() || !self.hint.is_empty() || !self.damage.is_empty()
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

impl EagerPseudoStyles {
    /// Returns whether there are any pseudo styles.
    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    /// Grabs a reference to the list of styles, if they exist.
    pub fn as_array(&self) -> Option<&EagerPseudoArrayInner> {
        match self.0 {
            None => None,
            Some(ref x) => Some(&x.0),
        }
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
        self.primary().get_box().clone_display() == display::T::none
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

    /// Restyle state.
    pub restyle: RestyleData,
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
        shared_context: &SharedStyleContext)
    {
        // In animation-only restyle we shouldn't touch snapshot at all.
        if shared_context.traversal_flags.for_animation_only() {
            return;
        }

        use invalidation::element::invalidator::TreeStyleInvalidator;

        debug!("invalidate_style_if_needed: {:?}, flags: {:?}, has_snapshot: {}, \
                handled_snapshot: {}, pseudo: {:?}",
                element,
                shared_context.traversal_flags,
                element.has_snapshot(),
                element.handled_snapshot(),
                element.implemented_pseudo_element());

        if element.has_snapshot() && !element.handled_snapshot() {
            let invalidator = TreeStyleInvalidator::new(
                element,
                Some(self),
                shared_context,
            );
            invalidator.invalidate();
            unsafe { element.set_handled_snapshot() }
            debug_assert!(element.handled_snapshot());
        }
    }

    /// Returns true if this element has styles.
    pub fn has_styles(&self) -> bool {
        self.styles.primary.is_some()
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

        let hint = self.restyle.hint;
        if hint.match_self() {
            return RestyleKind::MatchAndCascade;
        }

        if hint.has_replacements() {
            debug_assert!(!hint.has_animation_hint(),
                          "Animation only restyle hint should have already processed");
            return RestyleKind::CascadeWithReplacements(hint & RestyleHint::replacements());
        }

        debug_assert!(hint.has_recascade_self(),
                      "We definitely need to do something!");
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
        let hint = self.restyle.hint;
        if hint.has_animation_hint() {
            return RestyleKind::CascadeWithReplacements(hint & RestyleHint::for_animations());
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
    pub fn clear_restyle_state(&mut self) {
        self.restyle.clear();
    }

    /// Drops restyle flags and damage from the element.
    pub fn clear_restyle_flags_and_damage(&mut self) {
        self.restyle.clear_flags_and_damage();
    }
}
