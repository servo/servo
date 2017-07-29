/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! High-level interface to CSS selector matching.

#![allow(unsafe_code)]
#![deny(missing_docs)]

use context::{ElementCascadeInputs, SelectorFlagsMap, SharedStyleContext, StyleContext};
use data::{ElementData, ElementStyles, RestyleData};
use dom::TElement;
use invalidation::element::restyle_hints::{RESTYLE_CSS_ANIMATIONS, RESTYLE_CSS_TRANSITIONS};
use invalidation::element::restyle_hints::{RESTYLE_SMIL, RESTYLE_STYLE_ATTRIBUTE};
use invalidation::element::restyle_hints::RestyleHint;
use properties::ComputedValues;
use properties::longhands::display::computed_value as display;
use rule_tree::{CascadeLevel, StrongRuleNode};
use selector_parser::{PseudoElement, RestyleDamage};
use selectors::matching::ElementSelectorFlags;
use servo_arc::{Arc, ArcBorrow};
use traversal_flags;

/// Represents the result of comparing an element's old and new style.
pub struct StyleDifference {
    /// The resulting damage.
    pub damage: RestyleDamage,

    /// Whether any styles changed.
    pub change: StyleChange,
}

impl StyleDifference {
    /// Creates a new `StyleDifference`.
    pub fn new(damage: RestyleDamage, change: StyleChange) -> Self {
        StyleDifference {
            change: change,
            damage: damage,
        }
    }
}

/// Represents whether or not the style of an element has changed.
#[derive(Copy, Clone)]
pub enum StyleChange {
    /// The style hasn't changed.
    Unchanged,
    /// The style has changed.
    Changed,
}

/// Whether or not newly computed values for an element need to be cascade
/// to children.
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub enum ChildCascadeRequirement {
    /// Old and new computed values were the same, or we otherwise know that
    /// we won't bother recomputing style for children, so we can skip cascading
    /// the new values into child elements.
    CanSkipCascade = 0,
    /// Old and new computed values were different, so we must cascade the
    /// new values to children.
    ///
    /// FIXME(heycam) Although this is "must" cascade, in the future we should
    /// track whether child elements rely specifically on inheriting particular
    /// property values.  When we do that, we can treat `MustCascadeChildren` as
    /// "must cascade unless we know that changes to these properties can be
    /// ignored".
    MustCascadeChildren = 1,
    /// The same as `MustCascadeChildren`, but for the entire subtree.  This is
    /// used to handle root font-size updates needing to recascade the whole
    /// document.
    MustCascadeDescendants = 2,
}

bitflags! {
    /// Flags that represent the result of replace_rules.
    pub flags RulesChanged: u8 {
        /// Normal rules are changed.
        const NORMAL_RULES_CHANGED = 0x01,
        /// Important rules are changed.
        const IMPORTANT_RULES_CHANGED = 0x02,
    }
}

impl RulesChanged {
    /// Return true if there are any normal rules changed.
    #[inline]
    pub fn normal_rules_changed(&self) -> bool {
        self.contains(NORMAL_RULES_CHANGED)
    }

    /// Return true if there are any important rules changed.
    #[inline]
    pub fn important_rules_changed(&self) -> bool {
        self.contains(IMPORTANT_RULES_CHANGED)
    }
}

/// Determines which styles are being cascaded currently.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum CascadeVisitedMode {
    /// Cascade the regular, unvisited styles.
    Unvisited,
    /// Cascade the styles used when an element's relevant link is visited.  A
    /// "relevant link" is the element being matched if it is a link or the
    /// nearest ancestor link.
    Visited,
}

impl CascadeVisitedMode {
    /// Returns whether the cascade should filter to only visited dependent
    /// properties based on the cascade mode.
    pub fn visited_dependent_only(&self) -> bool {
        *self == CascadeVisitedMode::Visited
    }
}

trait PrivateMatchMethods: TElement {
    /// If there is no transition rule in the ComputedValues, it returns None.
    #[cfg(feature = "gecko")]
    fn get_after_change_style(
        &self,
        context: &mut StyleContext<Self>,
        primary_style: &Arc<ComputedValues>
    ) -> Option<Arc<ComputedValues>> {
        use context::CascadeInputs;
        use style_resolver::StyleResolverForElement;
        use stylist::RuleInclusion;

        let rule_node = primary_style.rules();
        let without_transition_rules =
            context.shared.stylist.rule_tree().remove_transition_rule_if_applicable(rule_node);
        if without_transition_rules == *rule_node {
            // We don't have transition rule in this case, so return None to let
            // the caller use the original ComputedValues.
            return None;
        }

        // FIXME(bug 868975): We probably need to transition visited style as
        // well.
        let inputs =
            CascadeInputs {
                rules: Some(without_transition_rules),
                visited_rules: primary_style.get_visited_style().and_then(|s| s.rules.clone()),
            };

        let style =
            StyleResolverForElement::new(*self, context, RuleInclusion::All)
                .cascade_style_and_visited_with_default_parents(inputs);

        Some(style)
    }

    #[cfg(feature = "gecko")]
    fn needs_animations_update(&self,
                               context: &mut StyleContext<Self>,
                               old_values: Option<&Arc<ComputedValues>>,
                               new_values: &ComputedValues)
                               -> bool {
        let new_box_style = new_values.get_box();
        let has_new_animation_style = new_box_style.animation_name_count() >= 1 &&
                                      new_box_style.animation_name_at(0).0.is_some();
        let has_animations = self.has_css_animations();

        old_values.map_or(has_new_animation_style, |old| {
            let old_box_style = old.get_box();
            let old_display_style = old_box_style.clone_display();
            let new_display_style = new_box_style.clone_display();

            // If the traverse is triggered by CSS rule changes, we need to
            // try to update all CSS animations on the element if the element
            // has CSS animation style regardless of whether the animation is
            // running or not.
            // TODO: We should check which @keyframes changed/added/deleted
            // and update only animations corresponding to those @keyframes.
            (context.shared.traversal_flags.contains(traversal_flags::ForCSSRuleChanges) &&
             has_new_animation_style) ||
            !old_box_style.animations_equals(&new_box_style) ||
             (old_display_style == display::T::none &&
              new_display_style != display::T::none &&
              has_new_animation_style) ||
             (old_display_style != display::T::none &&
              new_display_style == display::T::none &&
              has_animations)
        })
    }

    #[cfg(feature = "gecko")]
    fn process_animations(&self,
                          context: &mut StyleContext<Self>,
                          old_values: &mut Option<Arc<ComputedValues>>,
                          new_values: &mut Arc<ComputedValues>,
                          important_rules_changed: bool) {
        use context::{CASCADE_RESULTS, CSS_ANIMATIONS, CSS_TRANSITIONS, EFFECT_PROPERTIES};
        use context::UpdateAnimationsTasks;

        // Bug 868975: These steps should examine and update the visited styles
        // in addition to the unvisited styles.

        let mut tasks = UpdateAnimationsTasks::empty();
        if self.needs_animations_update(context, old_values.as_ref(), new_values) {
            tasks.insert(CSS_ANIMATIONS);
        }

        let before_change_style = if self.might_need_transitions_update(old_values.as_ref().map(|s| &**s),
                                                                        new_values) {
            let after_change_style = if self.has_css_transitions() {
                self.get_after_change_style(context, new_values)
            } else {
                None
            };

            // In order to avoid creating a SequentialTask for transitions which
            // may not be updated, we check it per property to make sure Gecko
            // side will really update transition.
            let needs_transitions_update = {
                // We borrow new_values here, so need to add a scope to make
                // sure we release it before assigning a new value to it.
                let after_change_style_ref =
                    after_change_style.as_ref().unwrap_or(&new_values);

                self.needs_transitions_update(old_values.as_ref().unwrap(),
                                              after_change_style_ref)
            };

            if needs_transitions_update {
                if let Some(values_without_transitions) = after_change_style {
                    *new_values = values_without_transitions;
                }
                tasks.insert(CSS_TRANSITIONS);

                // We need to clone old_values into SequentialTask, so we can use it later.
                old_values.clone()
            } else {
                None
            }
        } else {
            None
        };

        if self.has_animations() {
            tasks.insert(EFFECT_PROPERTIES);
            if important_rules_changed {
                tasks.insert(CASCADE_RESULTS);
            }
        }

        if !tasks.is_empty() {
            let task = ::context::SequentialTask::update_animations(*self,
                                                                    before_change_style,
                                                                    tasks);
            context.thread_local.tasks.push(task);
        }
    }

    #[cfg(feature = "servo")]
    fn process_animations(&self,
                          context: &mut StyleContext<Self>,
                          old_values: &mut Option<Arc<ComputedValues>>,
                          new_values: &mut Arc<ComputedValues>,
                          _important_rules_changed: bool) {
        use animation;
        use dom::TNode;

        let possibly_expired_animations =
            &mut context.thread_local.current_element_info.as_mut().unwrap()
                        .possibly_expired_animations;
        let shared_context = context.shared;
        if let Some(ref mut old) = *old_values {
            self.update_animations_for_cascade(shared_context, old,
                                               possibly_expired_animations,
                                               &context.thread_local.font_metrics_provider);
        }

        let new_animations_sender = &context.thread_local.new_animations_sender;
        let this_opaque = self.as_node().opaque();
        // Trigger any present animations if necessary.
        animation::maybe_start_animations(&shared_context,
                                          new_animations_sender,
                                          this_opaque, &new_values);

        // Trigger transitions if necessary. This will reset `new_values` back
        // to its old value if it did trigger a transition.
        if let Some(ref values) = *old_values {
            animation::start_transitions_if_applicable(
                new_animations_sender,
                this_opaque,
                &**values,
                new_values,
                &shared_context.timer,
                &possibly_expired_animations);
        }
    }


    /// Computes and applies non-redundant damage.
    #[cfg(feature = "gecko")]
    fn accumulate_damage_for(&self,
                             shared_context: &SharedStyleContext,
                             restyle: &mut RestyleData,
                             old_values: &ComputedValues,
                             new_values: &Arc<ComputedValues>,
                             pseudo: Option<&PseudoElement>)
                             -> ChildCascadeRequirement {
        // Don't accumulate damage if we're in a forgetful traversal.
        if shared_context.traversal_flags.contains(traversal_flags::Forgetful) {
            return ChildCascadeRequirement::MustCascadeChildren;
        }

        // If an ancestor is already getting reconstructed by Gecko's top-down
        // frame constructor, no need to apply damage.  Similarly if we already
        // have an explicitly stored ReconstructFrame hint.
        //
        // See https://bugzilla.mozilla.org/show_bug.cgi?id=1301258#c12
        // for followup work to make the optimization here more optimal by considering
        // each bit individually.
        let skip_applying_damage =
            restyle.reconstructed_self_or_ancestor();

        let difference =
            self.compute_style_difference(&old_values, &new_values, pseudo);

        if !skip_applying_damage {
            restyle.damage |= difference.damage;
        }

        match difference.change {
            StyleChange::Unchanged => {
                // We need to cascade the children in order to ensure the
                // correct propagation of computed value flags.
                if old_values.flags != new_values.flags {
                    return ChildCascadeRequirement::MustCascadeChildren;
                }
                ChildCascadeRequirement::CanSkipCascade
            },
            StyleChange::Changed => ChildCascadeRequirement::MustCascadeChildren,
        }
    }

    /// Computes and applies restyle damage unless we've already maxed it out.
    #[cfg(feature = "servo")]
    fn accumulate_damage_for(&self,
                             _shared_context: &SharedStyleContext,
                             restyle: &mut RestyleData,
                             old_values: &ComputedValues,
                             new_values: &Arc<ComputedValues>,
                             pseudo: Option<&PseudoElement>)
                             -> ChildCascadeRequirement {
        let difference = self.compute_style_difference(&old_values, &new_values, pseudo);
        restyle.damage |= difference.damage;
        match difference.change {
            StyleChange::Changed => ChildCascadeRequirement::MustCascadeChildren,
            StyleChange::Unchanged => ChildCascadeRequirement::CanSkipCascade,
        }
    }

    #[cfg(feature = "servo")]
    fn update_animations_for_cascade(&self,
                                     context: &SharedStyleContext,
                                     style: &mut Arc<ComputedValues>,
                                     possibly_expired_animations: &mut Vec<::animation::PropertyAnimation>,
                                     font_metrics: &::font_metrics::FontMetricsProvider) {
        use animation::{self, Animation};
        use dom::TNode;

        // Finish any expired transitions.
        let this_opaque = self.as_node().opaque();
        animation::complete_expired_transitions(this_opaque, style, context);

        // Merge any running transitions into the current style, and cancel them.
        let had_running_animations = context.running_animations
                                            .read()
                                            .get(&this_opaque)
                                            .is_some();
        if had_running_animations {
            let mut all_running_animations = context.running_animations.write();
            for running_animation in all_running_animations.get_mut(&this_opaque).unwrap() {
                // This shouldn't happen frequently, but under some
                // circumstances mainly huge load or debug builds, the
                // constellation might be delayed in sending the
                // `TickAllAnimations` message to layout.
                //
                // Thus, we can't assume all the animations have been already
                // updated by layout, because other restyle due to script might
                // be triggered by layout before the animation tick.
                //
                // See #12171 and the associated PR for an example where this
                // happened while debugging other release panic.
                if !running_animation.is_expired() {
                    animation::update_style_for_animation(context,
                                                          running_animation,
                                                          style,
                                                          font_metrics);
                    if let Animation::Transition(_, _, ref frame, _) = *running_animation {
                        possibly_expired_animations.push(frame.property_animation.clone())
                    }
                }
            }
        }
    }
}

impl<E: TElement> PrivateMatchMethods for E {}

/// The public API that elements expose for selector matching.
pub trait MatchMethods : TElement {
    /// Returns the closest parent element that doesn't have a display: contents
    /// style (and thus generates a box).
    ///
    /// This is needed to correctly handle blockification of flex and grid
    /// items.
    ///
    /// Returns itself if the element has no parent. In practice this doesn't
    /// happen because the root element is blockified per spec, but it could
    /// happen if we decide to not blockify for roots of disconnected subtrees,
    /// which is a kind of dubious beahavior.
    fn layout_parent(&self) -> Self {
        let mut current = self.clone();
        loop {
            current = match current.traversal_parent() {
                Some(el) => el,
                None => return current,
            };

            let is_display_contents =
                current.borrow_data().unwrap().styles.primary().is_display_contents();

            if !is_display_contents {
                return current;
            }
        }
    }

    /// Updates the styles with the new ones, diffs them, and stores the restyle
    /// damage.
    fn finish_restyle(
        &self,
        context: &mut StyleContext<Self>,
        mut data: &mut ElementData,
        mut new_styles: ElementStyles,
        important_rules_changed: bool,
    ) -> ChildCascadeRequirement {
        use dom::TNode;
        use std::cmp;
        use std::mem;

        debug_assert!(new_styles.primary.is_some(), "How did that happen?");

        if !context.shared.traversal_flags.for_animation_only() {
            self.process_animations(
                context,
                &mut data.styles.primary,
                &mut new_styles.primary.as_mut().unwrap(),
                important_rules_changed,
            );
        }

        // First of all, update the styles.
        let old_styles = mem::replace(&mut data.styles, new_styles);

        // Propagate the "can be fragmented" bit. It would be nice to
        // encapsulate this better.
        //
        // Note that this is technically not needed for pseudos since we already
        // do that when we resolve the non-pseudo style, but it doesn't hurt
        // anyway.
        if cfg!(feature = "servo") {
            let layout_parent =
                self.inheritance_parent().map(|e| e.layout_parent());
            let layout_parent_data =
                layout_parent.as_ref().and_then(|e| e.borrow_data());
            let layout_parent_style =
                layout_parent_data.as_ref().map(|d| d.styles.primary());

            if let Some(ref p) = layout_parent_style {
                let can_be_fragmented =
                    p.is_multicol() ||
                    layout_parent.as_ref().unwrap().as_node().can_be_fragmented();
                unsafe { self.as_node().set_can_be_fragmented(can_be_fragmented); }
            }
        }

        let new_primary_style = data.styles.primary.as_ref().unwrap();

        let mut cascade_requirement = ChildCascadeRequirement::CanSkipCascade;
        if self.is_root() && !self.is_native_anonymous() {
            let device = context.shared.stylist.device();
            let new_font_size = new_primary_style.get_font().clone_font_size();

            if old_styles.primary.as_ref().map_or(true, |s| s.get_font().clone_font_size() != new_font_size) {
                debug_assert!(self.owner_doc_matches_for_testing(device));
                device.set_root_font_size(new_font_size);
                // If the root font-size changed since last time, and something
                // in the document did use rem units, ensure we recascade the
                // entire tree.
                if device.used_root_font_size() {
                    cascade_requirement = ChildCascadeRequirement::MustCascadeDescendants;
                }
            }
        }

        // Don't accumulate damage if we're in a forgetful traversal.
        if context.shared.traversal_flags.contains(traversal_flags::Forgetful) {
            return ChildCascadeRequirement::MustCascadeChildren;
        }

        // Also, don't do anything if there was no style.
        let old_primary_style = match old_styles.primary {
            Some(s) => s,
            None => return ChildCascadeRequirement::MustCascadeChildren,
        };

        cascade_requirement = cmp::max(
            cascade_requirement,
            self.accumulate_damage_for(
                context.shared,
                &mut data.restyle,
                &old_primary_style,
                new_primary_style,
                None,
            )
        );

        if data.styles.pseudos.is_empty() && old_styles.pseudos.is_empty() {
            // This is the common case; no need to examine pseudos here.
            return cascade_requirement;
        }

        let pseudo_styles =
            old_styles.pseudos.as_array().iter().zip(
            data.styles.pseudos.as_array().iter());

        for (i, (old, new)) in pseudo_styles.enumerate() {
            match (old, new) {
                (&Some(ref old), &Some(ref new)) => {
                    self.accumulate_damage_for(
                        context.shared,
                        &mut data.restyle,
                        old,
                        new,
                        Some(&PseudoElement::from_eager_index(i)),
                    );
                }
                (&None, &None) => {},
                _ => {
                    // It's possible that we're switching from not having
                    // ::before/::after at all to having styles for them but not
                    // actually having a useful pseudo-element.  Check for that
                    // case.
                    let pseudo = PseudoElement::from_eager_index(i);
                    let new_pseudo_should_exist =
                        new.as_ref().map_or(false,
                                            |s| pseudo.should_exist(s));
                    let old_pseudo_should_exist =
                        old.as_ref().map_or(false,
                                            |s| pseudo.should_exist(s));
                    if new_pseudo_should_exist != old_pseudo_should_exist {
                        data.restyle.damage |= RestyleDamage::reconstruct();
                        return cascade_requirement;
                    }
                }
            }
        }

        cascade_requirement
    }


    /// Applies selector flags to an element, deferring mutations of the parent
    /// until after the traversal.
    ///
    /// TODO(emilio): This is somewhat inefficient, because it doesn't take
    /// advantage of us knowing that the traversal is sequential.
    fn apply_selector_flags(&self,
                            map: &mut SelectorFlagsMap<Self>,
                            element: &Self,
                            flags: ElementSelectorFlags) {
        // Handle flags that apply to the element.
        let self_flags = flags.for_self();
        if !self_flags.is_empty() {
            if element == self {
                // If this is the element we're styling, we have exclusive
                // access to the element, and thus it's fine inserting them,
                // even from the worker.
                unsafe { element.set_selector_flags(self_flags); }
            } else {
                // Otherwise, this element is an ancestor of the current element
                // we're styling, and thus multiple children could write to it
                // if we did from here.
                //
                // Instead, we can read them, and post them if necessary as a
                // sequential task in order for them to be processed later.
                if !element.has_selector_flags(self_flags) {
                    map.insert_flags(*element, self_flags);
                }
            }
        }

        // Handle flags that apply to the parent.
        let parent_flags = flags.for_parent();
        if !parent_flags.is_empty() {
            if let Some(p) = element.parent_element() {
                if !p.has_selector_flags(parent_flags) {
                    map.insert_flags(p, parent_flags);
                }
            }
        }
    }

    /// Computes and applies restyle damage.
    fn accumulate_damage(&self,
                         shared_context: &SharedStyleContext,
                         restyle: &mut RestyleData,
                         old_values: Option<&ComputedValues>,
                         new_values: &Arc<ComputedValues>,
                         pseudo: Option<&PseudoElement>)
                         -> ChildCascadeRequirement {
        let old_values = match old_values {
            Some(v) => v,
            None => return ChildCascadeRequirement::MustCascadeChildren,
        };

        // ::before and ::after are element-backed in Gecko, so they do the
        // damage calculation for themselves, when there's an actual pseudo.
        let is_existing_before_or_after =
            cfg!(feature = "gecko") &&
            pseudo.map_or(false, |p| p.is_before_or_after()) &&
            self.existing_style_for_restyle_damage(old_values, pseudo)
                .is_some();

        if is_existing_before_or_after {
            return ChildCascadeRequirement::CanSkipCascade;
        }

        self.accumulate_damage_for(shared_context,
                                   restyle,
                                   old_values,
                                   new_values,
                                   pseudo)
    }

    /// Updates the rule nodes without re-running selector matching, using just
    /// the rule tree.
    ///
    /// Returns true if an !important rule was replaced.
    fn replace_rules(
        &self,
        replacements: RestyleHint,
        context: &mut StyleContext<Self>,
        cascade_inputs: &mut ElementCascadeInputs,
    ) -> bool {
        let mut result = false;
        result |= self.replace_rules_internal(
            replacements,
            context,
            CascadeVisitedMode::Unvisited,
            cascade_inputs,
        );
        result |= self.replace_rules_internal(
            replacements,
            context,
            CascadeVisitedMode::Visited,
            cascade_inputs
        );
        result
    }

    /// Updates the rule nodes without re-running selector matching, using just
    /// the rule tree, for a specific visited mode.
    ///
    /// Returns true if an !important rule was replaced.
    fn replace_rules_internal(
        &self,
        replacements: RestyleHint,
        context: &mut StyleContext<Self>,
        cascade_visited: CascadeVisitedMode,
        cascade_inputs: &mut ElementCascadeInputs,
    ) -> bool {
        use properties::PropertyDeclarationBlock;
        use shared_lock::Locked;

        debug_assert!(replacements.intersects(RestyleHint::replacements()) &&
                      (replacements & !RestyleHint::replacements()).is_empty());

        let stylist = &context.shared.stylist;
        let guards = &context.shared.guards;

        let primary_rules =
            match cascade_visited {
                CascadeVisitedMode::Unvisited => cascade_inputs.primary.rules.as_mut(),
                CascadeVisitedMode::Visited => cascade_inputs.primary.visited_rules.as_mut(),
            };

        let primary_rules = match primary_rules {
            Some(r) => r,
            None => return false,
        };

        let replace_rule_node = |level: CascadeLevel,
                                 pdb: Option<ArcBorrow<Locked<PropertyDeclarationBlock>>>,
                                 path: &mut StrongRuleNode| -> bool {
            let new_node = stylist.rule_tree()
                                  .update_rule_at_level(level, pdb, path, guards);
            match new_node {
                Some(n) => {
                    *path = n;
                    level.is_important()
                },
                None => false,
            }
        };

        if !context.shared.traversal_flags.for_animation_only() {
            let mut result = false;
            if replacements.contains(RESTYLE_STYLE_ATTRIBUTE) {
                let style_attribute = self.style_attribute();
                result |= replace_rule_node(CascadeLevel::StyleAttributeNormal,
                                            style_attribute,
                                            primary_rules);
                result |= replace_rule_node(CascadeLevel::StyleAttributeImportant,
                                            style_attribute,
                                            primary_rules);
                // FIXME(emilio): Still a hack!
                self.unset_dirty_style_attribute();
            }
            return result;
        }

        // Animation restyle hints are processed prior to other restyle
        // hints in the animation-only traversal.
        //
        // Non-animation restyle hints will be processed in a subsequent
        // normal traversal.
        if replacements.intersects(RestyleHint::for_animations()) {
            debug_assert!(context.shared.traversal_flags.for_animation_only());

            if replacements.contains(RESTYLE_SMIL) {
                replace_rule_node(CascadeLevel::SMILOverride,
                                  self.get_smil_override(),
                                  primary_rules);
            }

            let replace_rule_node_for_animation = |level: CascadeLevel,
                                                   primary_rules: &mut StrongRuleNode| {
                let animation_rule = self.get_animation_rule_by_cascade(level);
                replace_rule_node(level,
                                  animation_rule.as_ref().map(|a| a.borrow_arc()),
                                  primary_rules);
            };

            // Apply Transition rules and Animation rules if the corresponding restyle hint
            // is contained.
            if replacements.contains(RESTYLE_CSS_TRANSITIONS) {
                replace_rule_node_for_animation(CascadeLevel::Transitions,
                                                primary_rules);
            }

            if replacements.contains(RESTYLE_CSS_ANIMATIONS) {
                replace_rule_node_for_animation(CascadeLevel::Animations,
                                                primary_rules);
            }
        }

        false
    }

    /// Given the old and new style of this element, and whether it's a
    /// pseudo-element, compute the restyle damage used to determine which
    /// kind of layout or painting operations we'll need.
    fn compute_style_difference(
        &self,
        old_values: &ComputedValues,
        new_values: &Arc<ComputedValues>,
        pseudo: Option<&PseudoElement>
    ) -> StyleDifference {
        debug_assert!(pseudo.map_or(true, |p| p.is_eager()));
        if let Some(source) = self.existing_style_for_restyle_damage(old_values, pseudo) {
            return RestyleDamage::compute_style_difference(source, old_values, new_values)
        }

        let new_display = new_values.get_box().clone_display();
        let old_display = old_values.get_box().clone_display();

        let new_style_is_display_none = new_display == display::T::none;
        let old_style_is_display_none = old_display == display::T::none;

        // If there's no style source, that likely means that Gecko couldn't
        // find a style context.
        //
        // This happens with display:none elements, and not-yet-existing
        // pseudo-elements.
        if new_style_is_display_none && old_style_is_display_none {
            // The style remains display:none.  The only case we need to care
            // about is if -moz-binding changed, and to generate a reconstruct
            // so that we can start the binding load.  Otherwise, there is no
            // need for damage.
            return RestyleDamage::compute_undisplayed_style_difference(old_values, new_values);
        }

        if pseudo.map_or(false, |p| p.is_before_or_after()) {
            // FIXME(bz) This duplicates some of the logic in
            // PseudoElement::should_exist, but it's not clear how best to share
            // that logic without redoing the "get the display" work.
            let old_style_generates_no_pseudo =
                old_style_is_display_none ||
                old_values.ineffective_content_property();

            let new_style_generates_no_pseudo =
                new_style_is_display_none ||
                new_values.ineffective_content_property();

            if old_style_generates_no_pseudo != new_style_generates_no_pseudo {
                return StyleDifference::new(RestyleDamage::reconstruct(), StyleChange::Changed)
            }

            // The pseudo-element will remain undisplayed, so just avoid
            // triggering any change.
            //
            // NOTE(emilio): We will only arrive here for pseudo-elements that
            // aren't generated (see the is_existing_before_or_after check in
            // accumulate_damage).
            //
            // However, it may be the case that the style of this element would
            // make us think we need a pseudo, but we don't, like for pseudos in
            // replaced elements, that's why we need the old != new instead of
            // just check whether the new style would generate a pseudo.
            return StyleDifference::new(RestyleDamage::empty(), StyleChange::Unchanged)
        }

        if pseudo.map_or(false, |p| p.is_first_letter() || p.is_first_line()) {
            // No one cares about this pseudo, and we've checked above that
            // we're not switching from a "cares" to a "doesn't care" state
            // or vice versa.
            return StyleDifference::new(RestyleDamage::empty(),
                                        StyleChange::Unchanged)
        }

        // If we are changing display property we need to accumulate
        // reconstruction damage for the change.
        // FIXME: Bug 1378972: This is a workaround for bug 1374175, we should
        // generate more accurate restyle damage in fallback cases.
        let needs_reconstruction = new_display != old_display;
        let damage = if needs_reconstruction {
            RestyleDamage::reconstruct()
        } else {
            RestyleDamage::empty()
        };
        // We don't really know if there was a change in any style (since we
        // didn't actually call compute_style_difference) but we return
        // StyleChange::Changed conservatively.
        StyleDifference::new(damage, StyleChange::Changed)
    }
}

impl<E: TElement> MatchMethods for E {}
