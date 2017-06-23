/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Per-node data used in style calculation.

use arrayvec::ArrayVec;
use context::SharedStyleContext;
use dom::TElement;
use invalidation::element::restyle_hints::RestyleHint;
use properties::{AnimationRules, ComputedValues, PropertyDeclarationBlock};
use properties::longhands::display::computed_value as display;
use rule_tree::StrongRuleNode;
use selector_parser::{EAGER_PSEUDO_COUNT, PseudoElement, RestyleDamage};
use shared_lock::{Locked, StylesheetGuards};
use std::ops::{Deref, DerefMut};
use stylearc::Arc;

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

    /// Returns whether this element or any ancestor is going to be
    /// reconstructed.
    pub fn reconstructed_self_or_ancestor(&self) -> bool {
        self.reconstructed_ancestor() ||
        self.damage.contains(RestyleDamage::reconstruct())
    }

    /// Returns whether any ancestor of this element was restyled.
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
#[derive(Clone, Debug)]
pub struct EagerPseudoStyles(Option<Arc<EagerPseudoArray>>);

#[derive(Debug, Default)]
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

    /// Returns a mutable reference to the style for a given eager pseudo, if it exists.
    pub fn get_mut(&mut self, pseudo: &PseudoElement) -> Option<&mut Arc<ComputedValues>> {
        debug_assert!(pseudo.is_eager());
        match self.0 {
            None => return None,
            Some(ref mut arc) => Arc::make_mut(arc)[pseudo.eager_index()].as_mut(),
        }
    }

    /// Returns true if the EagerPseudoStyles has the style for |pseudo|.
    pub fn has(&self, pseudo: &PseudoElement) -> bool {
        self.get(pseudo).is_some()
    }

    /// Sets the style for the eager pseudo.
    pub fn set(&mut self, pseudo: &PseudoElement, value: Arc<ComputedValues>) {
        if self.0.is_none() {
            self.0 = Some(Arc::new(Default::default()));
        }
        let arr = Arc::make_mut(self.0.as_mut().unwrap());
        arr[pseudo.eager_index()] = Some(value);
    }

    /// Inserts a pseudo-element. The pseudo-element must not already exist.
    pub fn insert(&mut self, pseudo: &PseudoElement, value: Arc<ComputedValues>) {
        debug_assert!(!self.has(pseudo));
        self.set(pseudo, value);
    }

    /// Removes a pseudo-element style if it exists, and returns it.
    pub fn take(&mut self, pseudo: &PseudoElement) -> Option<Arc<ComputedValues>> {
        let result = match self.0 {
            None => return None,
            Some(ref mut arc) => Arc::make_mut(arc)[pseudo.eager_index()].take(),
        };
        let empty = self.0.as_ref().unwrap().iter().all(|x| x.is_none());
        if empty {
            self.0 = None;
        }
        result
    }

    /// Returns a list of the pseudo-elements.
    pub fn keys(&self) -> ArrayVec<[PseudoElement; EAGER_PSEUDO_COUNT]> {
        let mut v = ArrayVec::new();
        if let Some(ref arr) = self.0 {
            for i in 0..EAGER_PSEUDO_COUNT {
                if arr[i].is_some() {
                    v.push(PseudoElement::from_eager_index(i));
                }
            }
        }
        v
    }

    /// Returns whether this map has the same set of pseudos as the given one.
    pub fn has_same_pseudos_as(&self, other: &Self) -> bool {
        // We could probably just compare self.keys() to other.keys(), but that
        // seems like it'll involve a bunch more moving stuff around and
        // whatnot.
        match (&self.0, &other.0) {
            (&Some(ref our_arr), &Some(ref other_arr)) => {
                for i in 0..EAGER_PSEUDO_COUNT {
                    if our_arr[i].is_some() != other_arr[i].is_some() {
                        return false
                    }
                }
                true
            },
            (&None, &None) => true,
            _ => false,
        }
    }
}

/// The styles associated with a node, including the styles for any
/// pseudo-elements.
#[derive(Clone, Debug)]
pub struct ElementStyles {
    /// The element's style.
    pub primary: Option<Arc<ComputedValues>>,
    /// A list of the styles for the element's eagerly-cascaded pseudo-elements.
    pub pseudos: EagerPseudoStyles,
}

impl Default for ElementStyles {
    /// Construct an empty `ElementStyles`.
    fn default() -> Self {
        ElementStyles {
            primary: None,
            pseudos: EagerPseudoStyles(None),
        }
    }
}

impl ElementStyles {
    /// Returns the primary style.
    pub fn get_primary(&self) -> Option<&Arc<ComputedValues>> {
        self.primary.as_ref()
    }

    /// Returns the mutable primary style.
    pub fn get_primary_mut(&mut self) -> Option<&mut Arc<ComputedValues>> {
        self.primary.as_mut()
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
    /// Borrows both styles and restyle mutably at the same time.
    pub fn styles_and_restyle_mut(
        &mut self
    ) -> (&mut ElementStyles, &mut RestyleData) {
        (&mut self.styles,
         &mut self.restyle)
    }

    /// Invalidates style for this element, its descendants, and later siblings,
    /// based on the snapshot of the element that we took when attributes or
    /// state changed.
    pub fn invalidate_style_if_needed<'a, E: TElement>(
        &mut self,
        element: E,
        shared_context: &SharedStyleContext)
    {
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

    /// Returns whether we have any outstanding style invalidation.
    pub fn has_invalidations(&self) -> bool {
        self.restyle.hint.has_self_invalidations()
    }

    /// Returns the kind of restyling that we're going to need to do on this
    /// element, based of the stored restyle hint.
    pub fn restyle_kind(&self,
                        shared_context: &SharedStyleContext)
                        -> RestyleKind {
        debug_assert!(!self.has_styles() || self.has_invalidations(),
                      "Should've stopped earlier");
        if !self.has_styles() {
            debug_assert!(!shared_context.traversal_flags.for_animation_only(),
                          "Unstyled element shouldn't be traversed during \
                           animation-only traversal");
            return RestyleKind::MatchAndCascade;
        }

        let hint = self.restyle.hint;
        if shared_context.traversal_flags.for_animation_only() {
            // return either CascadeWithReplacements or CascadeOnly in case of
            // animation-only restyle.
            if hint.has_animation_hint() {
                return RestyleKind::CascadeWithReplacements(hint & RestyleHint::for_animations());
            }
            return RestyleKind::CascadeOnly;
        }

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

    /// Return true if important rules are different.
    /// We use this to make sure the cascade of off-main thread animations is correct.
    /// Note: Ignore custom properties for now because we only support opacity and transform
    ///       properties for animations running on compositor. Actually, we only care about opacity
    ///       and transform for now, but it's fine to compare all properties and let the user
    ///       the check which properties do they want.
    ///       If it costs too much, get_properties_overriding_animations() should return a set
    ///       containing only opacity and transform properties.
    pub fn important_rules_are_different(&self,
                                         rules: &StrongRuleNode,
                                         guards: &StylesheetGuards) -> bool {
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

    /// Returns SMIL overriden value if exists.
    pub fn get_smil_override(&self) -> Option<&Arc<Locked<PropertyDeclarationBlock>>> {
        if cfg!(feature = "servo") {
            // Servo has no knowledge of a SMIL rule, so just avoid looking for it.
            return None;
        }

        match self.styles.get_primary() {
            Some(v) => v.rules().get_smil_animation_rule(),
            None => None,
        }
    }

    /// Returns AnimationRules that has processed during animation-only restyles.
    pub fn get_animation_rules(&self) -> AnimationRules {
        if cfg!(feature = "servo") {
            return AnimationRules(None, None)
        }

        match self.styles.get_primary() {
            Some(v) => v.rules().get_animation_rules(),
            None => AnimationRules(None, None),
        }
    }
}
