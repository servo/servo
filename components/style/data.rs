/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Per-node data used in style calculation.

use context::SharedStyleContext;
use dom::TElement;
use properties::{AnimationRules, ComputedValues, PropertyDeclarationBlock};
use properties::longhands::display::computed_value as display;
use restyle_hints::{HintComputationContext, RestyleReplacements, RestyleHint};
use rule_tree::StrongRuleNode;
use selector_parser::{EAGER_PSEUDO_COUNT, PseudoElement, RestyleDamage};
use selectors::matching::VisitedHandlingMode;
use shared_lock::{Locked, StylesheetGuards};
use std::fmt;
use stylearc::Arc;
use traversal::TraversalFlags;

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
    pub fn keys(&self) -> Vec<PseudoElement> {
        let mut v = Vec::new();
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
            VisitedHandlingMode::AllLinksUnvisited => {
                self.remove_unvisited_rules(&pseudo)
            },
            VisitedHandlingMode::RelevantLinkVisited => {
                self.remove_visited_rules(&pseudo)
            },
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

/// Restyle hint for storing on ElementData.
///
/// We wrap it in a newtype to force the encapsulation of the complexity of
/// handling the correct invalidations in this file.
#[derive(Clone, Debug)]
pub struct StoredRestyleHint(RestyleHint);

impl StoredRestyleHint {
    /// Propagates this restyle hint to a child element.
    pub fn propagate(&mut self, traversal_flags: &TraversalFlags) -> Self {
        use std::mem;

        // In the middle of an animation only restyle, we don't need to
        // propagate any restyle hints, and we need to remove ourselves.
        if traversal_flags.for_animation_only() {
            self.0.remove_animation_hints();
            return Self::empty();
        }

        debug_assert!(!self.0.has_animation_hint(),
                      "There should not be any animation restyle hints \
                       during normal traversal");

        // Else we should clear ourselves, and return the propagated hint.
        let new_hint = mem::replace(&mut self.0, RestyleHint::empty())
                       .propagate_for_non_animation_restyle();
        StoredRestyleHint(new_hint)
    }

    /// Creates an empty `StoredRestyleHint`.
    pub fn empty() -> Self {
        StoredRestyleHint(RestyleHint::empty())
    }

    /// Creates a restyle hint that forces the whole subtree to be restyled,
    /// including the element.
    pub fn subtree() -> Self {
        StoredRestyleHint(RestyleHint::subtree())
    }

    /// Creates a restyle hint that forces the element and all its later
    /// siblings to have their whole subtrees restyled, including the elements
    /// themselves.
    pub fn subtree_and_later_siblings() -> Self {
        StoredRestyleHint(RestyleHint::subtree_and_later_siblings())
    }

    /// Creates a restyle hint that indicates the element must be recascaded.
    pub fn recascade_self() -> Self {
        StoredRestyleHint(RestyleHint::recascade_self())
    }

    /// Returns true if the hint indicates that our style may be invalidated.
    pub fn has_self_invalidations(&self) -> bool {
        self.0.affects_self()
    }

    /// Returns true if the hint indicates that our sibling's style may be
    /// invalidated.
    pub fn has_sibling_invalidations(&self) -> bool {
        self.0.affects_later_siblings()
    }

    /// Whether the restyle hint is empty (nothing requires to be restyled).
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Insert another restyle hint, effectively resulting in the union of both.
    pub fn insert(&mut self, other: Self) {
        self.0.insert(other.0)
    }

    /// Contains whether the whole subtree is invalid.
    pub fn contains_subtree(&self) -> bool {
        self.0.contains(&RestyleHint::subtree())
    }

    /// Insert another restyle hint, effectively resulting in the union of both.
    pub fn insert_from(&mut self, other: &Self) {
        self.0.insert_from(&other.0)
    }

    /// Returns true if the hint has animation-only restyle.
    pub fn has_animation_hint(&self) -> bool {
        self.0.has_animation_hint()
    }

    /// Returns true if the hint indicates the current element must be
    /// recascaded.
    pub fn has_recascade_self(&self) -> bool {
        self.0.has_recascade_self()
    }
}

impl Default for StoredRestyleHint {
    fn default() -> Self {
        StoredRestyleHint::empty()
    }
}

impl From<RestyleHint> for StoredRestyleHint {
    fn from(hint: RestyleHint) -> Self {
        StoredRestyleHint(hint)
    }
}

/// Transient data used by the restyle algorithm. This structure is instantiated
/// either before or during restyle traversal, and is cleared at the end of node
/// processing.
#[derive(Debug, Default)]
pub struct RestyleData {
    /// The restyle hint, which indicates whether selectors need to be rematched
    /// for this element, its children, and its descendants.
    pub hint: StoredRestyleHint,

    /// The restyle damage, indicating what kind of layout changes are required
    /// afte restyling.
    pub damage: RestyleDamage,

    /// The restyle damage that has already been handled by our ancestors, and does
    /// not need to be applied again at this element. Only non-empty during the
    /// traversal, once ancestor damage has been calculated.
    ///
    /// Note that this optimization mostly makes sense in terms of Gecko's top-down
    /// frame constructor and change list processing model. We don't bother with it
    /// for Servo for now.
    #[cfg(feature = "gecko")]
    pub damage_handled: RestyleDamage,
}

impl RestyleData {
    /// Returns true if this RestyleData might invalidate the current style.
    pub fn has_invalidations(&self) -> bool {
        self.hint.has_self_invalidations()
    }

    /// Returns true if this RestyleData might invalidate sibling styles.
    pub fn has_sibling_invalidations(&self) -> bool {
        self.hint.has_sibling_invalidations()
    }

    /// Returns damage handled.
    #[cfg(feature = "gecko")]
    pub fn damage_handled(&self) -> RestyleDamage {
        self.damage_handled
    }

    /// Returns damage handled (always empty for servo).
    #[cfg(feature = "servo")]
    pub fn damage_handled(&self) -> RestyleDamage {
        RestyleDamage::empty()
    }

    /// Sets damage handled.
    #[cfg(feature = "gecko")]
    pub fn set_damage_handled(&mut self, d: RestyleDamage) {
        self.damage_handled = d;
    }

    /// Sets damage handled. No-op for Servo.
    #[cfg(feature = "servo")]
    pub fn set_damage_handled(&mut self, _: RestyleDamage) {}
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

    /// Restyle tracking. We separate this into a separate allocation so that
    /// we can drop it when no restyles are pending on the elemnt.
    restyle: Option<Box<RestyleData>>,
}

/// The kind of restyle that a single element should do.
pub enum RestyleKind {
    /// We need to run selector matching plus re-cascade, that is, a full
    /// restyle.
    MatchAndCascade,
    /// We need to recascade with some replacement rule, such as the style
    /// attribute, or animation rules.
    CascadeWithReplacements(RestyleReplacements),
    /// We only need to recascade, for example, because only inherited
    /// properties in the parent changed.
    CascadeOnly,
}

impl ElementData {
    /// Computes the final restyle hint for this element, potentially allocating
    /// a `RestyleData` if we need to.
    ///
    /// This expands the snapshot (if any) into a restyle hint, and handles
    /// explicit sibling restyle hints from the stored restyle hint.
    ///
    /// Returns true if later siblings must be restyled.
    pub fn compute_final_hint<'a, E: TElement>(
        &mut self,
        element: E,
        shared_context: &SharedStyleContext,
        hint_context: HintComputationContext<'a, E>)
        -> bool
    {
        debug!("compute_final_hint: {:?}, {:?}",
               element,
               shared_context.traversal_flags);

        let mut hint = match self.get_restyle() {
            Some(r) => r.hint.0.clone(),
            None => RestyleHint::empty(),
        };

        debug!("compute_final_hint: {:?}, has_snapshot: {}, handled_snapshot: {}, \
                pseudo: {:?}",
                element,
                element.has_snapshot(),
                element.handled_snapshot(),
                element.implemented_pseudo_element());

        if element.has_snapshot() && !element.handled_snapshot() {
            let snapshot_hint =
                shared_context.stylist.compute_restyle_hint(&element,
                                                            shared_context,
                                                            hint_context);
            hint.insert(snapshot_hint);
            unsafe { element.set_handled_snapshot() }
            debug_assert!(element.handled_snapshot());
        }

        let empty_hint = hint.is_empty();

        // If the hint includes a directive for later siblings, strip it out and
        // notify the caller to modify the base hint for future siblings.
        let later_siblings = hint.remove_later_siblings_hint();

        // Insert the hint, overriding the previous hint. This effectively takes
        // care of removing the later siblings restyle hint.
        if !empty_hint {
            self.ensure_restyle().hint = hint.into();
        }

        later_siblings
    }


    /// Trivially construct an ElementData.
    pub fn new(existing: Option<ElementStyles>) -> Self {
        ElementData {
            styles: existing,
            restyle: None,
        }
    }

    /// Returns true if this element has a computed style.
    pub fn has_styles(&self) -> bool {
        self.styles.is_some()
    }

    /// Returns whether we have any outstanding style invalidation.
    pub fn has_invalidations(&self) -> bool {
        self.restyle.as_ref().map_or(false, |r| r.has_invalidations())
    }

    /// Returns the kind of restyling that we're going to need to do on this
    /// element, based of the stored restyle hint.
    pub fn restyle_kind(&self) -> RestyleKind {
        debug_assert!(!self.has_styles() || self.has_invalidations(),
                      "Should've stopped earlier");
        if !self.has_styles() {
            return RestyleKind::MatchAndCascade;
        }

        debug_assert!(self.restyle.is_some());
        let restyle_data = self.restyle.as_ref().unwrap();

        let hint = &restyle_data.hint.0;
        if hint.match_self() {
            return RestyleKind::MatchAndCascade;
        }

        if hint.has_replacements() {
            return RestyleKind::CascadeWithReplacements(hint.replacements);
        }

        debug_assert!(hint.has_recascade_self(), "We definitely need to do something!");
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

    /// Borrows both styles and restyle mutably at the same time.
    pub fn styles_and_restyle_mut(&mut self) -> (&mut ElementStyles,
                                                 Option<&mut RestyleData>) {
        (self.styles.as_mut().unwrap(),
         self.restyle.as_mut().map(|r| &mut **r))
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

    /// Returns true if the Element has a RestyleData.
    pub fn has_restyle(&self) -> bool {
        self.restyle.is_some()
    }

    /// Drops any RestyleData.
    pub fn clear_restyle(&mut self) {
        self.restyle = None;
    }

    /// Creates a RestyleData if one doesn't exist.
    ///
    /// Asserts that the Element has been styled.
    pub fn ensure_restyle(&mut self) -> &mut RestyleData {
        debug_assert!(self.styles.is_some(), "restyling unstyled element");
        if self.restyle.is_none() {
            self.restyle = Some(Box::new(RestyleData::default()));
        }
        self.restyle.as_mut().unwrap()
    }

    /// Gets a reference to the restyle data, if any.
    pub fn get_restyle(&self) -> Option<&RestyleData> {
        self.restyle.as_ref().map(|r| &**r)
    }

    /// Gets a reference to the restyle data. Panic if the element does not
    /// have restyle data.
    pub fn restyle(&self) -> &RestyleData {
        self.get_restyle().expect("Calling restyle without RestyleData")
    }

    /// Gets a mutable reference to the restyle data, if any.
    pub fn get_restyle_mut(&mut self) -> Option<&mut RestyleData> {
        self.restyle.as_mut().map(|r| &mut **r)
    }

    /// Gets a mutable reference to the restyle data. Panic if the element does
    /// not have restyle data.
    pub fn restyle_mut(&mut self) -> &mut RestyleData {
        self.get_restyle_mut().expect("Calling restyle_mut without RestyleData")
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
