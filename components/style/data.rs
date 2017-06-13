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
use selectors::matching::VisitedHandlingMode;
use shared_lock::{Locked, StylesheetGuards};
use std::fmt;
use stylearc::Arc;

/// The structure that represents the result of style computation. This is
/// effectively a tuple of rules and computed values, that is, the rule node,
/// and the result of computing that rule node's rules, the `ComputedValues`.
#[derive(Clone)]
pub struct ComputedStyle {
    /// The rule node representing the ordered list of rules matched for this
    /// node.
    pub rules: StrongRuleNode,

    /// The computed values for each property obtained by cascading the
    /// matched rules. This can only be none during a transient interval of
    /// the styling algorithm, and callers can safely unwrap it.
    pub values: Option<Arc<ComputedValues>>,

    /// The rule node representing the ordered list of rules matched for this
    /// node if visited, only computed if there's a relevant link for this
    /// element. A element's "relevant link" is the element being matched if it
    /// is a link or the nearest ancestor link.
    visited_rules: Option<StrongRuleNode>,

    /// The element's computed values if visited, only computed if there's a
    /// relevant link for this element. A element's "relevant link" is the
    /// element being matched if it is a link or the nearest ancestor link.
    ///
    /// We also store a reference to this inside the regular ComputedValues to
    /// avoid refactoring all APIs to become aware of multiple ComputedValues
    /// objects.
    visited_values: Option<Arc<ComputedValues>>,
}

impl ComputedStyle {
    /// Trivially construct a new `ComputedStyle`.
    pub fn new(rules: StrongRuleNode, values: Arc<ComputedValues>) -> Self {
        ComputedStyle {
            rules: rules,
            values: Some(values),
            visited_rules: None,
            visited_values: None,
        }
    }

    /// Constructs a partial ComputedStyle, whose ComputedVaues will be filled
    /// in later.
    pub fn new_partial(rules: StrongRuleNode) -> Self {
        ComputedStyle {
            rules: rules,
            values: None,
            visited_rules: None,
            visited_values: None,
        }
    }

    /// Returns a reference to the ComputedValues. The values can only be null during
    /// the styling algorithm, so this is safe to call elsewhere.
    pub fn values(&self) -> &Arc<ComputedValues> {
        self.values.as_ref().unwrap()
    }

    /// Whether there are any visited rules.
    pub fn has_visited_rules(&self) -> bool {
        self.visited_rules.is_some()
    }

    /// Gets a reference to the visited rule node, if any.
    pub fn get_visited_rules(&self) -> Option<&StrongRuleNode> {
        self.visited_rules.as_ref()
    }

    /// Gets a mutable reference to the visited rule node, if any.
    pub fn get_visited_rules_mut(&mut self) -> Option<&mut StrongRuleNode> {
        self.visited_rules.as_mut()
    }

    /// Gets a reference to the visited rule node. Panic if the element does not
    /// have visited rule node.
    pub fn visited_rules(&self) -> &StrongRuleNode {
        self.get_visited_rules().unwrap()
    }

    /// Sets the visited rule node, and returns whether it changed.
    pub fn set_visited_rules(&mut self, rules: StrongRuleNode) -> bool {
        if let Some(ref old_rules) = self.visited_rules {
            if *old_rules == rules {
                return false
            }
        }
        self.visited_rules = Some(rules);
        true
    }

    /// Takes the visited rule node.
    pub fn take_visited_rules(&mut self) -> Option<StrongRuleNode> {
        self.visited_rules.take()
    }

    /// Gets a reference to the visited computed values. Panic if the element
    /// does not have visited computed values.
    pub fn visited_values(&self) -> &Arc<ComputedValues> {
        self.visited_values.as_ref().unwrap()
    }

    /// Sets the visited computed values.
    pub fn set_visited_values(&mut self, values: Arc<ComputedValues>) {
        self.visited_values = Some(values);
    }

    /// Take the visited computed values.
    pub fn take_visited_values(&mut self) -> Option<Arc<ComputedValues>> {
        self.visited_values.take()
    }

    /// Clone the visited computed values Arc.  Used to store a reference to the
    /// visited values inside the regular values.
    pub fn clone_visited_values(&self) -> Option<Arc<ComputedValues>> {
        self.visited_values.clone()
    }
}

// We manually implement Debug for ComputedStyle so that we can avoid the
// verbose stringification of ComputedValues for normal logging.
impl fmt::Debug for ComputedStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ComputedStyle {{ rules: {:?}, values: {{..}} }}", self.rules)
    }
}

/// A list of styles for eagerly-cascaded pseudo-elements. Lazily-allocated.
#[derive(Clone, Debug)]
pub struct EagerPseudoStyles(Option<Box<[Option<ComputedStyle>]>>);

impl EagerPseudoStyles {
    /// Returns whether there are any pseudo styles.
    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    /// Returns a reference to the style for a given eager pseudo, if it exists.
    pub fn get(&self, pseudo: &PseudoElement) -> Option<&ComputedStyle> {
        debug_assert!(pseudo.is_eager());
        self.0.as_ref().and_then(|p| p[pseudo.eager_index()].as_ref())
    }

    /// Returns a mutable reference to the style for a given eager pseudo, if it exists.
    pub fn get_mut(&mut self, pseudo: &PseudoElement) -> Option<&mut ComputedStyle> {
        debug_assert!(pseudo.is_eager());
        self.0.as_mut().and_then(|p| p[pseudo.eager_index()].as_mut())
    }

    /// Returns true if the EagerPseudoStyles has a ComputedStyle for |pseudo|.
    pub fn has(&self, pseudo: &PseudoElement) -> bool {
        self.get(pseudo).is_some()
    }

    /// Inserts a pseudo-element. The pseudo-element must not already exist.
    pub fn insert(&mut self, pseudo: &PseudoElement, style: ComputedStyle) {
        debug_assert!(!self.has(pseudo));
        if self.0.is_none() {
            self.0 = Some(vec![None; EAGER_PSEUDO_COUNT].into_boxed_slice());
        }
        self.0.as_mut().unwrap()[pseudo.eager_index()] = Some(style);
    }

    /// Removes a pseudo-element style if it exists, and returns it.
    fn take(&mut self, pseudo: &PseudoElement) -> Option<ComputedStyle> {
        let result = match self.0.as_mut() {
            None => return None,
            Some(arr) => arr[pseudo.eager_index()].take(),
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

    /// Adds the unvisited rule node for a given pseudo-element, which may or
    /// may not exist.
    ///
    /// Returns true if the pseudo-element is new.
    fn add_unvisited_rules(&mut self,
                           pseudo: &PseudoElement,
                           rules: StrongRuleNode)
                           -> bool {
        if let Some(mut style) = self.get_mut(pseudo) {
            style.rules = rules;
            return false
        }
        self.insert(pseudo, ComputedStyle::new_partial(rules));
        true
    }

    /// Remove the unvisited rule node for a given pseudo-element, which may or
    /// may not exist. Since removing the rule node implies we don't need any
    /// other data for the pseudo, take the entire pseudo if found.
    ///
    /// Returns true if the pseudo-element was removed.
    fn remove_unvisited_rules(&mut self, pseudo: &PseudoElement) -> bool {
        self.take(pseudo).is_some()
    }

    /// Adds the visited rule node for a given pseudo-element.  It is assumed to
    /// already exist because unvisited styles should have been added first.
    ///
    /// Returns true if the pseudo-element is new.  (Always false, but returns a
    /// bool for parity with `add_unvisited_rules`.)
    fn add_visited_rules(&mut self,
                         pseudo: &PseudoElement,
                         rules: StrongRuleNode)
                         -> bool {
        debug_assert!(self.has(pseudo));
        let mut style = self.get_mut(pseudo).unwrap();
        style.set_visited_rules(rules);
        false
    }

    /// Remove the visited rule node for a given pseudo-element, which may or
    /// may not exist.
    ///
    /// Returns true if the psuedo-element was removed. (Always false, but
    /// returns a bool for parity with `remove_unvisited_rules`.)
    fn remove_visited_rules(&mut self, pseudo: &PseudoElement) -> bool {
        if let Some(mut style) = self.get_mut(pseudo) {
            style.take_visited_rules();
        }
        false
    }

    /// Adds a rule node for a given pseudo-element, which may or may not exist.
    /// The type of rule node depends on the visited mode.
    ///
    /// Returns true if the pseudo-element is new.
    pub fn add_rules(&mut self,
                     pseudo: &PseudoElement,
                     visited_handling: VisitedHandlingMode,
                     rules: StrongRuleNode)
                     -> bool {
        match visited_handling {
            VisitedHandlingMode::AllLinksVisitedAndUnvisited => {
                unreachable!("We should never try to selector match with \
                             AllLinksVisitedAndUnvisited");
            },
            VisitedHandlingMode::AllLinksUnvisited => {
                self.add_unvisited_rules(&pseudo, rules)
            },
            VisitedHandlingMode::RelevantLinkVisited => {
                self.add_visited_rules(&pseudo, rules)
            },
        }
    }

    /// Removes a rule node for a given pseudo-element, which may or may not
    /// exist. The type of rule node depends on the visited mode.
    ///
    /// Returns true if the psuedo-element was removed.
    pub fn remove_rules(&mut self,
                        pseudo: &PseudoElement,
                        visited_handling: VisitedHandlingMode)
                        -> bool {
        match visited_handling {
            VisitedHandlingMode::AllLinksVisitedAndUnvisited => {
                unreachable!("We should never try to selector match with \
                             AllLinksVisitedAndUnvisited");
            },
            VisitedHandlingMode::AllLinksUnvisited => {
                self.remove_unvisited_rules(&pseudo)
            },
            VisitedHandlingMode::RelevantLinkVisited => {
                self.remove_visited_rules(&pseudo)
            },
        }
    }

    /// Returns whether this EagerPseudoStyles has the same set of
    /// pseudos as the given one.
    pub fn has_same_pseudos_as(&self, other: &EagerPseudoStyles) -> bool {
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
    pub primary: ComputedStyle,
    /// A list of the styles for the element's eagerly-cascaded pseudo-elements.
    pub pseudos: EagerPseudoStyles,
}

impl ElementStyles {
    /// Trivially construct a new `ElementStyles`.
    pub fn new(primary: ComputedStyle) -> Self {
        ElementStyles {
            primary: primary,
            pseudos: EagerPseudoStyles(None),
        }
    }

    /// Whether this element `display` value is `none`.
    pub fn is_display_none(&self) -> bool {
        self.primary.values().get_box().clone_display() == display::T::none
    }
}

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

/// Style system data associated with an Element.
///
/// In Gecko, this hangs directly off the Element. Servo, this is embedded
/// inside of layout data, which itself hangs directly off the Element. In
/// both cases, it is wrapped inside an AtomicRefCell to ensure thread safety.
#[derive(Debug)]
pub struct ElementData {
    /// The computed styles for the element and its pseudo-elements.
    styles: Option<ElementStyles>,

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
        (self.styles.as_mut().unwrap(),
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


    /// Trivially construct an ElementData.
    pub fn new(existing: Option<ElementStyles>) -> Self {
        ElementData {
            styles: existing,
            restyle: RestyleData::new(),
        }
    }

    /// Returns true if this element has a computed style.
    pub fn has_styles(&self) -> bool {
        self.styles.is_some()
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

    /// Gets the element styles, if any.
    pub fn get_styles(&self) -> Option<&ElementStyles> {
        self.styles.as_ref()
    }

    /// Gets the element styles. Panic if the element has never been styled.
    pub fn styles(&self) -> &ElementStyles {
        self.styles.as_ref().expect("Calling styles() on unstyled ElementData")
    }

    /// Gets a mutable reference to the element styles, if any.
    pub fn get_styles_mut(&mut self) -> Option<&mut ElementStyles> {
        self.styles.as_mut()
    }

    /// Gets a mutable reference to the element styles. Panic if the element has
    /// never been styled.
    pub fn styles_mut(&mut self) -> &mut ElementStyles {
        self.styles.as_mut().expect("Calling styles_mut() on unstyled ElementData")
    }

    /// Sets the computed element styles.
    pub fn set_styles(&mut self, styles: ElementStyles) {
        self.styles = Some(styles);
    }

    /// Sets the computed element rules, and returns whether the rules changed.
    pub fn set_primary_rules(&mut self, rules: StrongRuleNode) -> bool {
        if !self.has_styles() {
            self.set_styles(ElementStyles::new(ComputedStyle::new_partial(rules)));
            return true;
        }

        if self.styles().primary.rules == rules {
            return false;
        }

        self.styles_mut().primary.rules = rules;
        true
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
            self.styles().primary.rules.get_properties_overriding_animations(&guards);
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

        match self.get_styles() {
            Some(s) => s.primary.rules.get_smil_animation_rule(),
            None => None,
        }
    }

    /// Returns AnimationRules that has processed during animation-only restyles.
    pub fn get_animation_rules(&self) -> AnimationRules {
        if cfg!(feature = "servo") {
            return AnimationRules(None, None)
        }

        match self.get_styles() {
            Some(s) => s.primary.rules.get_animation_rules(),
            None => AnimationRules(None, None),
        }
    }
}
