/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! High-level interface to CSS selector matching.

#![allow(unsafe_code)]
#![deny(missing_docs)]

use crate::computed_value_flags::ComputedValueFlags;
use crate::context::{CascadeInputs, ElementCascadeInputs, QuirksMode};
use crate::context::{SharedStyleContext, StyleContext};
use crate::data::{ElementData, ElementStyles};
use crate::dom::TElement;
#[cfg(feature = "servo")]
use crate::dom::TNode;
use crate::invalidation::element::restyle_hints::RestyleHint;
use crate::properties::longhands::display::computed_value::T as Display;
use crate::properties::ComputedValues;
use crate::properties::PropertyDeclarationBlock;
use crate::rule_tree::{CascadeLevel, StrongRuleNode};
use crate::selector_parser::{PseudoElement, RestyleDamage};
use crate::shared_lock::Locked;
use crate::style_resolver::ResolvedElementStyles;
use crate::style_resolver::{PseudoElementResolution, StyleResolverForElement};
use crate::stylesheets::layer_rule::LayerOrder;
use crate::stylist::RuleInclusion;
use crate::traversal_flags::TraversalFlags;
use servo_arc::{Arc, ArcBorrow};

/// Represents the result of comparing an element's old and new style.
#[derive(Debug)]
pub struct StyleDifference {
    /// The resulting damage.
    pub damage: RestyleDamage,

    /// Whether any styles changed.
    pub change: StyleChange,
}

/// Represents whether or not the style of an element has changed.
#[derive(Clone, Copy, Debug)]
pub enum StyleChange {
    /// The style hasn't changed.
    Unchanged,
    /// The style has changed.
    Changed {
        /// Whether only reset structs changed.
        reset_only: bool,
    },
}

/// Whether or not newly computed values for an element need to be cascaded to
/// children (or children might need to be re-matched, e.g., for container
/// queries).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ChildRestyleRequirement {
    /// Old and new computed values were the same, or we otherwise know that
    /// we won't bother recomputing style for children, so we can skip cascading
    /// the new values into child elements.
    CanSkipCascade = 0,
    /// The same as `MustCascadeChildren`, but we only need to actually
    /// recascade if the child inherits any explicit reset style.
    MustCascadeChildrenIfInheritResetStyle = 1,
    /// Old and new computed values were different, so we must cascade the
    /// new values to children.
    MustCascadeChildren = 2,
    /// The same as `MustCascadeChildren`, but for the entire subtree.  This is
    /// used to handle root font-size updates needing to recascade the whole
    /// document.
    MustCascadeDescendants = 3,
    /// We need to re-match the whole subttree. This is used to handle container
    /// query relative unit changes for example. Container query size changes
    /// also trigger re-match, but after layout.
    MustMatchDescendants = 4,
}

/// Determines which styles are being cascaded currently.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CascadeVisitedMode {
    /// Cascade the regular, unvisited styles.
    Unvisited,
    /// Cascade the styles used when an element's relevant link is visited.  A
    /// "relevant link" is the element being matched if it is a link or the
    /// nearest ancestor link.
    Visited,
}

trait PrivateMatchMethods: TElement {
    fn replace_single_rule_node(
        context: &SharedStyleContext,
        level: CascadeLevel,
        layer_order: LayerOrder,
        pdb: Option<ArcBorrow<Locked<PropertyDeclarationBlock>>>,
        path: &mut StrongRuleNode,
    ) -> bool {
        let stylist = &context.stylist;
        let guards = &context.guards;

        let mut important_rules_changed = false;
        let new_node = stylist.rule_tree().update_rule_at_level(
            level,
            layer_order,
            pdb,
            path,
            guards,
            &mut important_rules_changed,
        );
        if let Some(n) = new_node {
            *path = n;
        }
        important_rules_changed
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
        debug_assert!(
            replacements.intersects(RestyleHint::replacements()) &&
                (replacements & !RestyleHint::replacements()).is_empty()
        );

        let primary_rules = match cascade_visited {
            CascadeVisitedMode::Unvisited => cascade_inputs.primary.rules.as_mut(),
            CascadeVisitedMode::Visited => cascade_inputs.primary.visited_rules.as_mut(),
        };

        let primary_rules = match primary_rules {
            Some(r) => r,
            None => return false,
        };

        if !context.shared.traversal_flags.for_animation_only() {
            let mut result = false;
            if replacements.contains(RestyleHint::RESTYLE_STYLE_ATTRIBUTE) {
                let style_attribute = self.style_attribute();
                result |= Self::replace_single_rule_node(
                    context.shared,
                    CascadeLevel::same_tree_author_normal(),
                    LayerOrder::root(),
                    style_attribute,
                    primary_rules,
                );
                result |= Self::replace_single_rule_node(
                    context.shared,
                    CascadeLevel::same_tree_author_important(),
                    LayerOrder::root(),
                    style_attribute,
                    primary_rules,
                );
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

            if replacements.contains(RestyleHint::RESTYLE_SMIL) {
                Self::replace_single_rule_node(
                    context.shared,
                    CascadeLevel::SMILOverride,
                    LayerOrder::root(),
                    self.smil_override(),
                    primary_rules,
                );
            }

            if replacements.contains(RestyleHint::RESTYLE_CSS_TRANSITIONS) {
                Self::replace_single_rule_node(
                    context.shared,
                    CascadeLevel::Transitions,
                    LayerOrder::root(),
                    self.transition_rule(&context.shared)
                        .as_ref()
                        .map(|a| a.borrow_arc()),
                    primary_rules,
                );
            }

            if replacements.contains(RestyleHint::RESTYLE_CSS_ANIMATIONS) {
                Self::replace_single_rule_node(
                    context.shared,
                    CascadeLevel::Animations,
                    LayerOrder::root(),
                    self.animation_rule(&context.shared)
                        .as_ref()
                        .map(|a| a.borrow_arc()),
                    primary_rules,
                );
            }
        }

        false
    }

    /// If there is no transition rule in the ComputedValues, it returns None.
    fn after_change_style(
        &self,
        context: &mut StyleContext<Self>,
        primary_style: &Arc<ComputedValues>,
    ) -> Option<Arc<ComputedValues>> {
        let rule_node = primary_style.rules();
        let without_transition_rules = context
            .shared
            .stylist
            .rule_tree()
            .remove_transition_rule_if_applicable(rule_node);
        if without_transition_rules == *rule_node {
            // We don't have transition rule in this case, so return None to let
            // the caller use the original ComputedValues.
            return None;
        }

        // FIXME(bug 868975): We probably need to transition visited style as
        // well.
        let inputs = CascadeInputs {
            rules: Some(without_transition_rules),
            visited_rules: primary_style.visited_rules().cloned(),
            flags: primary_style.flags.for_cascade_inputs(),
        };

        // Actually `PseudoElementResolution` doesn't really matter.
        let style = StyleResolverForElement::new(
            *self,
            context,
            RuleInclusion::All,
            PseudoElementResolution::IfApplicable,
        )
        .cascade_style_and_visited_with_default_parents(inputs);

        Some(style.0)
    }

    fn needs_animations_update(
        &self,
        context: &mut StyleContext<Self>,
        old_style: Option<&ComputedValues>,
        new_style: &ComputedValues,
        pseudo_element: Option<PseudoElement>,
    ) -> bool {
        let new_ui_style = new_style.get_ui();
        let new_style_specifies_animations = new_ui_style.specifies_animations();

        let has_animations = self.has_css_animations(&context.shared, pseudo_element);
        if !new_style_specifies_animations && !has_animations {
            return false;
        }

        let old_style = match old_style {
            Some(old) => old,
            // If we have no old style but have animations, we may be a
            // pseudo-element which was re-created without style changes.
            //
            // This can happen when we reframe the pseudo-element without
            // restyling it (due to content insertion on a flex container or
            // such, for example). See bug 1564366.
            //
            // FIXME(emilio): The really right fix for this is keeping the
            // pseudo-element itself around on reframes, but that's a bit
            // harder. If we do that we can probably remove quite a lot of the
            // EffectSet complexity though, since right now it's stored on the
            // parent element for pseudo-elements given we need to keep it
            // around...
            None => {
                return new_style_specifies_animations || new_style.is_pseudo_style();
            },
        };

        let old_ui_style = old_style.get_ui();

        let keyframes_could_have_changed = context
            .shared
            .traversal_flags
            .contains(TraversalFlags::ForCSSRuleChanges);

        // If the traversal is triggered due to changes in CSS rules changes, we
        // need to try to update all CSS animations on the element if the
        // element has or will have CSS animation style regardless of whether
        // the animation is running or not.
        //
        // TODO: We should check which @keyframes were added/changed/deleted and
        // update only animations corresponding to those @keyframes.
        if keyframes_could_have_changed {
            return true;
        }

        // If the animations changed, well...
        if !old_ui_style.animations_equals(new_ui_style) {
            return true;
        }

        let old_display = old_style.clone_display();
        let new_display = new_style.clone_display();

        // If we were display: none, we may need to trigger animations.
        if old_display == Display::None && new_display != Display::None {
            return new_style_specifies_animations;
        }

        // If we are becoming display: none, we may need to stop animations.
        if old_display != Display::None && new_display == Display::None {
            return has_animations;
        }

        // We might need to update animations if writing-mode or direction
        // changed, and any of the animations contained logical properties.
        //
        // We may want to be more granular, but it's probably not worth it.
        if new_style.writing_mode != old_style.writing_mode {
            return has_animations;
        }

        false
    }

    fn might_need_transitions_update(
        &self,
        context: &StyleContext<Self>,
        old_style: Option<&ComputedValues>,
        new_style: &ComputedValues,
        pseudo_element: Option<PseudoElement>,
    ) -> bool {
        let old_style = match old_style {
            Some(v) => v,
            None => return false,
        };

        if !self.has_css_transitions(context.shared, pseudo_element) &&
            !new_style.get_ui().specifies_transitions()
        {
            return false;
        }

        if old_style.clone_display().is_none() {
            return false;
        }

        return true;
    }

    /// Create a SequentialTask for resolving descendants in a SMIL display
    /// property animation if the display property changed from none.
    #[cfg(feature = "gecko")]
    fn handle_display_change_for_smil_if_needed(
        &self,
        context: &mut StyleContext<Self>,
        old_values: Option<&ComputedValues>,
        new_values: &ComputedValues,
        restyle_hints: RestyleHint,
    ) {
        use crate::context::PostAnimationTasks;

        if !restyle_hints.intersects(RestyleHint::RESTYLE_SMIL) {
            return;
        }

        if new_values.is_display_property_changed_from_none(old_values) {
            // When display value is changed from none to other, we need to
            // traverse descendant elements in a subsequent normal
            // traversal (we can't traverse them in this animation-only restyle
            // since we have no way to know whether the decendants
            // need to be traversed at the beginning of the animation-only
            // restyle).
            let task = crate::context::SequentialTask::process_post_animation(
                *self,
                PostAnimationTasks::DISPLAY_CHANGED_FROM_NONE_FOR_SMIL,
            );
            context.thread_local.tasks.push(task);
        }
    }

    #[cfg(feature = "gecko")]
    fn process_animations(
        &self,
        context: &mut StyleContext<Self>,
        old_styles: &mut ElementStyles,
        new_styles: &mut ResolvedElementStyles,
        restyle_hint: RestyleHint,
        important_rules_changed: bool,
    ) {
        use crate::context::UpdateAnimationsTasks;

        let new_values = new_styles.primary_style_mut();
        let old_values = &old_styles.primary;
        if context.shared.traversal_flags.for_animation_only() {
            self.handle_display_change_for_smil_if_needed(
                context,
                old_values.as_deref(),
                new_values,
                restyle_hint,
            );
            return;
        }

        // Bug 868975: These steps should examine and update the visited styles
        // in addition to the unvisited styles.

        let mut tasks = UpdateAnimationsTasks::empty();

        if old_values.as_deref().map_or_else(
            || new_values.get_ui().specifies_scroll_timelines(),
            |old| !old.get_ui().scroll_timelines_equals(new_values.get_ui()),
        ) {
            tasks.insert(UpdateAnimationsTasks::SCROLL_TIMELINES);
        }

        if old_values.as_deref().map_or_else(
            || new_values.get_ui().specifies_view_timelines(),
            |old| !old.get_ui().view_timelines_equals(new_values.get_ui()),
        ) {
            tasks.insert(UpdateAnimationsTasks::VIEW_TIMELINES);
        }

        if self.needs_animations_update(
            context,
            old_values.as_deref(),
            new_values,
            /* pseudo_element = */ None,
        ) {
            tasks.insert(UpdateAnimationsTasks::CSS_ANIMATIONS);
        }

        let before_change_style = if self.might_need_transitions_update(
            context,
            old_values.as_deref(),
            new_values,
            /* pseudo_element = */ None,
        ) {
            let after_change_style =
                if self.has_css_transitions(context.shared, /* pseudo_element = */ None) {
                    self.after_change_style(context, new_values)
                } else {
                    None
                };

            // In order to avoid creating a SequentialTask for transitions which
            // may not be updated, we check it per property to make sure Gecko
            // side will really update transition.
            let needs_transitions_update = {
                // We borrow new_values here, so need to add a scope to make
                // sure we release it before assigning a new value to it.
                let after_change_style_ref = after_change_style.as_ref().unwrap_or(&new_values);

                self.needs_transitions_update(old_values.as_ref().unwrap(), after_change_style_ref)
            };

            if needs_transitions_update {
                if let Some(values_without_transitions) = after_change_style {
                    *new_values = values_without_transitions;
                }
                tasks.insert(UpdateAnimationsTasks::CSS_TRANSITIONS);

                // We need to clone old_values into SequentialTask, so we can
                // use it later.
                old_values.clone()
            } else {
                None
            }
        } else {
            None
        };

        if self.has_animations(&context.shared) {
            tasks.insert(UpdateAnimationsTasks::EFFECT_PROPERTIES);
            if important_rules_changed {
                tasks.insert(UpdateAnimationsTasks::CASCADE_RESULTS);
            }
            if new_values.is_display_property_changed_from_none(old_values.as_deref()) {
                tasks.insert(UpdateAnimationsTasks::DISPLAY_CHANGED_FROM_NONE);
            }
        }

        if !tasks.is_empty() {
            let task = crate::context::SequentialTask::update_animations(
                *self,
                before_change_style,
                tasks,
            );
            context.thread_local.tasks.push(task);
        }
    }

    #[cfg(feature = "servo")]
    fn process_animations(
        &self,
        context: &mut StyleContext<Self>,
        old_styles: &mut ElementStyles,
        new_resolved_styles: &mut ResolvedElementStyles,
        _restyle_hint: RestyleHint,
        _important_rules_changed: bool,
    ) {
        use crate::animation::AnimationSetKey;
        use crate::dom::TDocument;

        let style_changed = self.process_animations_for_style(
            context,
            &mut old_styles.primary,
            new_resolved_styles.primary_style_mut(),
            /* pseudo_element = */ None,
        );

        // If we have modified animation or transitions, we recascade style for this node.
        if style_changed {
            let primary_style = new_resolved_styles.primary_style();
            let mut rule_node = primary_style.rules().clone();
            let declarations = context.shared.animations.get_all_declarations(
                &AnimationSetKey::new_for_non_pseudo(self.as_node().opaque()),
                context.shared.current_time_for_animations,
                self.as_node().owner_doc().shared_lock(),
            );
            Self::replace_single_rule_node(
                &context.shared,
                CascadeLevel::Transitions,
                LayerOrder::root(),
                declarations.transitions.as_ref().map(|a| a.borrow_arc()),
                &mut rule_node,
            );
            Self::replace_single_rule_node(
                &context.shared,
                CascadeLevel::Animations,
                LayerOrder::root(),
                declarations.animations.as_ref().map(|a| a.borrow_arc()),
                &mut rule_node,
            );

            if rule_node != *primary_style.rules() {
                let inputs = CascadeInputs {
                    rules: Some(rule_node),
                    visited_rules: primary_style.visited_rules().cloned(),
                    flags: primary_style.flags.for_cascade_inputs(),
                };

                new_resolved_styles.primary.style = StyleResolverForElement::new(
                    *self,
                    context,
                    RuleInclusion::All,
                    PseudoElementResolution::IfApplicable,
                )
                .cascade_style_and_visited_with_default_parents(inputs);
            }
        }

        self.process_animations_for_pseudo(
            context,
            old_styles,
            new_resolved_styles,
            PseudoElement::Before,
        );
        self.process_animations_for_pseudo(
            context,
            old_styles,
            new_resolved_styles,
            PseudoElement::After,
        );
    }

    #[cfg(feature = "servo")]
    fn process_animations_for_pseudo(
        &self,
        context: &mut StyleContext<Self>,
        old_styles: &mut ElementStyles,
        new_resolved_styles: &mut ResolvedElementStyles,
        pseudo_element: PseudoElement,
    ) {
        use crate::animation::AnimationSetKey;
        use crate::dom::TDocument;

        let key = AnimationSetKey::new_for_pseudo(self.as_node().opaque(), pseudo_element.clone());
        let mut style = match new_resolved_styles.pseudos.get(&pseudo_element) {
            Some(style) => Arc::clone(style),
            None => {
                context
                    .shared
                    .animations
                    .cancel_all_animations_for_key(&key);
                return;
            },
        };

        let mut old_style = old_styles.pseudos.get(&pseudo_element).cloned();
        self.process_animations_for_style(
            context,
            &mut old_style,
            &mut style,
            Some(pseudo_element.clone()),
        );

        let declarations = context.shared.animations.get_all_declarations(
            &key,
            context.shared.current_time_for_animations,
            self.as_node().owner_doc().shared_lock(),
        );
        if declarations.is_empty() {
            return;
        }

        let mut rule_node = style.rules().clone();
        Self::replace_single_rule_node(
            &context.shared,
            CascadeLevel::Transitions,
            LayerOrder::root(),
            declarations.transitions.as_ref().map(|a| a.borrow_arc()),
            &mut rule_node,
        );
        Self::replace_single_rule_node(
            &context.shared,
            CascadeLevel::Animations,
            LayerOrder::root(),
            declarations.animations.as_ref().map(|a| a.borrow_arc()),
            &mut rule_node,
        );
        if rule_node == *style.rules() {
            return;
        }

        let inputs = CascadeInputs {
            rules: Some(rule_node),
            visited_rules: style.visited_rules().cloned(),
            flags: style.flags.for_cascade_inputs(),
        };

        let new_style = StyleResolverForElement::new(
            *self,
            context,
            RuleInclusion::All,
            PseudoElementResolution::IfApplicable,
        )
        .cascade_style_and_visited_for_pseudo_with_default_parents(
            inputs,
            &pseudo_element,
            &new_resolved_styles.primary,
        );

        new_resolved_styles
            .pseudos
            .set(&pseudo_element, new_style.0);
    }

    #[cfg(feature = "servo")]
    fn process_animations_for_style(
        &self,
        context: &mut StyleContext<Self>,
        old_values: &mut Option<Arc<ComputedValues>>,
        new_values: &mut Arc<ComputedValues>,
        pseudo_element: Option<PseudoElement>,
    ) -> bool {
        use crate::animation::{AnimationSetKey, AnimationState};

        // We need to call this before accessing the `ElementAnimationSet` from the
        // map because this call will do a RwLock::read().
        let needs_animations_update = self.needs_animations_update(
            context,
            old_values.as_deref(),
            new_values,
            pseudo_element,
        );

        let might_need_transitions_update = self.might_need_transitions_update(
            context,
            old_values.as_deref(),
            new_values,
            pseudo_element,
        );

        let mut after_change_style = None;
        if might_need_transitions_update {
            after_change_style = self.after_change_style(context, new_values);
        }

        let key = AnimationSetKey::new(self.as_node().opaque(), pseudo_element);
        let shared_context = context.shared;
        let mut animation_set = shared_context
            .animations
            .sets
            .write()
            .remove(&key)
            .unwrap_or_default();

        // Starting animations is expensive, because we have to recalculate the style
        // for all the keyframes. We only want to do this if we think that there's a
        // chance that the animations really changed.
        if needs_animations_update {
            let mut resolver = StyleResolverForElement::new(
                *self,
                context,
                RuleInclusion::All,
                PseudoElementResolution::IfApplicable,
            );

            animation_set.update_animations_for_new_style::<Self>(
                *self,
                &shared_context,
                &new_values,
                &mut resolver,
            );
        }

        animation_set.update_transitions_for_new_style(
            might_need_transitions_update,
            &shared_context,
            old_values.as_ref(),
            after_change_style.as_ref().unwrap_or(new_values),
        );

        // We clear away any finished transitions, but retain animations, because they
        // might still be used for proper calculation of `animation-fill-mode`. This
        // should change the computed values in the style, so we don't need to mark
        // this set as dirty.
        animation_set
            .transitions
            .retain(|transition| transition.state != AnimationState::Finished);

        // If the ElementAnimationSet is empty, and don't store it in order to
        // save memory and to avoid extra processing later.
        let changed_animations = animation_set.dirty;
        if !animation_set.is_empty() {
            animation_set.dirty = false;
            shared_context
                .animations
                .sets
                .write()
                .insert(key, animation_set);
        }

        changed_animations
    }

    /// Computes and applies non-redundant damage.
    fn accumulate_damage_for(
        &self,
        shared_context: &SharedStyleContext,
        damage: &mut RestyleDamage,
        old_values: &ComputedValues,
        new_values: &ComputedValues,
        pseudo: Option<&PseudoElement>,
    ) -> ChildRestyleRequirement {
        debug!("accumulate_damage_for: {:?}", self);
        debug_assert!(!shared_context
            .traversal_flags
            .contains(TraversalFlags::FinalAnimationTraversal));

        let difference = self.compute_style_difference(old_values, new_values, pseudo);

        *damage |= difference.damage;

        debug!(" > style difference: {:?}", difference);

        // We need to cascade the children in order to ensure the correct
        // propagation of inherited computed value flags.
        if old_values.flags.maybe_inherited() != new_values.flags.maybe_inherited() {
            debug!(
                " > flags changed: {:?} != {:?}",
                old_values.flags, new_values.flags
            );
            return ChildRestyleRequirement::MustCascadeChildren;
        }

        match difference.change {
            StyleChange::Unchanged => return ChildRestyleRequirement::CanSkipCascade,
            StyleChange::Changed { reset_only } => {
                // If inherited properties changed, the best we can do is
                // cascade the children.
                if !reset_only {
                    return ChildRestyleRequirement::MustCascadeChildren;
                }
            },
        }

        let old_display = old_values.clone_display();
        let new_display = new_values.clone_display();

        if old_display != new_display {
            // If we used to be a display: none element, and no longer are, our
            // children need to be restyled because they're unstyled.
            if old_display == Display::None {
                return ChildRestyleRequirement::MustCascadeChildren;
            }
            // Blockification of children may depend on our display value,
            // so we need to actually do the recascade. We could potentially
            // do better, but it doesn't seem worth it.
            if old_display.is_item_container() != new_display.is_item_container() {
                return ChildRestyleRequirement::MustCascadeChildren;
            }
            // We may also need to blockify and un-blockify descendants if our
            // display goes from / to display: contents, since the "layout
            // parent style" changes.
            if old_display.is_contents() || new_display.is_contents() {
                return ChildRestyleRequirement::MustCascadeChildren;
            }
            // Line break suppression may also be affected if the display
            // type changes from ruby to non-ruby.
            #[cfg(feature = "gecko")]
            {
                if old_display.is_ruby_type() != new_display.is_ruby_type() {
                    return ChildRestyleRequirement::MustCascadeChildren;
                }
            }
        }

        // Children with justify-items: auto may depend on our
        // justify-items property value.
        //
        // Similarly, we could potentially do better, but this really
        // seems not common enough to care about.
        #[cfg(feature = "gecko")]
        {
            use crate::values::specified::align::AlignFlags;

            let old_justify_items = old_values.get_position().clone_justify_items();
            let new_justify_items = new_values.get_position().clone_justify_items();

            let was_legacy_justify_items =
                old_justify_items.computed.0.contains(AlignFlags::LEGACY);

            let is_legacy_justify_items = new_justify_items.computed.0.contains(AlignFlags::LEGACY);

            if is_legacy_justify_items != was_legacy_justify_items {
                return ChildRestyleRequirement::MustCascadeChildren;
            }

            if was_legacy_justify_items && old_justify_items.computed != new_justify_items.computed
            {
                return ChildRestyleRequirement::MustCascadeChildren;
            }
        }

        #[cfg(feature = "servo")]
        {
            // We may need to set or propagate the CAN_BE_FRAGMENTED bit
            // on our children.
            if old_values.is_multicol() != new_values.is_multicol() {
                return ChildRestyleRequirement::MustCascadeChildren;
            }
        }

        // We could prove that, if our children don't inherit reset
        // properties, we can stop the cascade.
        ChildRestyleRequirement::MustCascadeChildrenIfInheritResetStyle
    }
}

impl<E: TElement> PrivateMatchMethods for E {}

/// The public API that elements expose for selector matching.
pub trait MatchMethods: TElement {
    /// Returns the closest parent element that doesn't have a display: contents
    /// style (and thus generates a box).
    ///
    /// This is needed to correctly handle blockification of flex and grid
    /// items.
    ///
    /// Returns itself if the element has no parent. In practice this doesn't
    /// happen because the root element is blockified per spec, but it could
    /// happen if we decide to not blockify for roots of disconnected subtrees,
    /// which is a kind of dubious behavior.
    fn layout_parent(&self) -> Self {
        let mut current = self.clone();
        loop {
            current = match current.traversal_parent() {
                Some(el) => el,
                None => return current,
            };

            let is_display_contents = current
                .borrow_data()
                .unwrap()
                .styles
                .primary()
                .is_display_contents();

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
        data: &mut ElementData,
        mut new_styles: ResolvedElementStyles,
        important_rules_changed: bool,
    ) -> ChildRestyleRequirement {
        use std::cmp;

        self.process_animations(
            context,
            &mut data.styles,
            &mut new_styles,
            data.hint,
            important_rules_changed,
        );

        // First of all, update the styles.
        let old_styles = data.set_styles(new_styles);

        let new_primary_style = data.styles.primary.as_ref().unwrap();

        let mut restyle_requirement = ChildRestyleRequirement::CanSkipCascade;
        let is_root = new_primary_style
            .flags
            .contains(ComputedValueFlags::IS_ROOT_ELEMENT_STYLE);
        let is_container = !new_primary_style
            .get_box()
            .clone_container_type()
            .is_normal();
        if is_root || is_container {
            let new_font_size = new_primary_style.get_font().clone_font_size();
            let old_font_size = old_styles
                .primary
                .as_ref()
                .map(|s| s.get_font().clone_font_size());

            if old_font_size != Some(new_font_size) {
                if is_root {
                    let device = context.shared.stylist.device();
                    debug_assert!(self.owner_doc_matches_for_testing(device));
                    device.set_root_font_size(new_font_size.computed_size().into());
                    if device.used_root_font_size() {
                        // If the root font-size changed since last time, and something
                        // in the document did use rem units, ensure we recascade the
                        // entire tree.
                        restyle_requirement = ChildRestyleRequirement::MustCascadeDescendants;
                    }
                }

                if is_container && old_font_size.is_some() {
                    // TODO(emilio): Maybe only do this if we were matched
                    // against relative font sizes?
                    // Also, maybe we should do this as well for font-family /
                    // etc changes (for ex/ch/ic units to work correctly)? We
                    // should probably do the optimization mentioned above if
                    // so.
                    restyle_requirement = ChildRestyleRequirement::MustMatchDescendants;
                }
            }
        }

        if context.shared.stylist.quirks_mode() == QuirksMode::Quirks {
            if self.is_html_document_body_element() {
                // NOTE(emilio): We _could_ handle dynamic changes to it if it
                // changes and before we reach our children the cascade stops,
                // but we don't track right now whether we use the document body
                // color, and nobody else handles that properly anyway.
                let device = context.shared.stylist.device();

                // Needed for the "inherit from body" quirk.
                let text_color = new_primary_style.get_inherited_text().clone_color();
                device.set_body_text_color(text_color);
            }
        }

        // Don't accumulate damage if we're in the final animation traversal.
        if context
            .shared
            .traversal_flags
            .contains(TraversalFlags::FinalAnimationTraversal)
        {
            return ChildRestyleRequirement::MustCascadeChildren;
        }

        // Also, don't do anything if there was no style.
        let old_primary_style = match old_styles.primary {
            Some(s) => s,
            None => return ChildRestyleRequirement::MustCascadeChildren,
        };

        let old_container_type = old_primary_style.clone_container_type();
        let new_container_type = new_primary_style.clone_container_type();
        if old_container_type != new_container_type && !new_container_type.is_size_container_type()
        {
            // Stopped being a size container. Re-evaluate container queries and units on all our descendants.
            // Changes into and between different size containment is handled in `UpdateContainerQueryStyles`.
            restyle_requirement = ChildRestyleRequirement::MustMatchDescendants;
        } else if old_container_type.is_size_container_type() &&
            !old_primary_style.is_display_contents() &&
            new_primary_style.is_display_contents()
        {
            // Also re-evaluate when a container gets 'display: contents', since size queries will now evaluate to unknown.
            // Other displays like 'inline' will keep generating a box, so they are handled in `UpdateContainerQueryStyles`.
            restyle_requirement = ChildRestyleRequirement::MustMatchDescendants;
        }

        restyle_requirement = cmp::max(
            restyle_requirement,
            self.accumulate_damage_for(
                context.shared,
                &mut data.damage,
                &old_primary_style,
                new_primary_style,
                None,
            ),
        );

        if data.styles.pseudos.is_empty() && old_styles.pseudos.is_empty() {
            // This is the common case; no need to examine pseudos here.
            return restyle_requirement;
        }

        let pseudo_styles = old_styles
            .pseudos
            .as_array()
            .iter()
            .zip(data.styles.pseudos.as_array().iter());

        for (i, (old, new)) in pseudo_styles.enumerate() {
            match (old, new) {
                (&Some(ref old), &Some(ref new)) => {
                    self.accumulate_damage_for(
                        context.shared,
                        &mut data.damage,
                        old,
                        new,
                        Some(&PseudoElement::from_eager_index(i)),
                    );
                },
                (&None, &None) => {},
                _ => {
                    // It's possible that we're switching from not having
                    // ::before/::after at all to having styles for them but not
                    // actually having a useful pseudo-element.  Check for that
                    // case.
                    let pseudo = PseudoElement::from_eager_index(i);
                    let new_pseudo_should_exist =
                        new.as_ref().map_or(false, |s| pseudo.should_exist(s));
                    let old_pseudo_should_exist =
                        old.as_ref().map_or(false, |s| pseudo.should_exist(s));
                    if new_pseudo_should_exist != old_pseudo_should_exist {
                        data.damage |= RestyleDamage::reconstruct();
                        return restyle_requirement;
                    }
                },
            }
        }

        restyle_requirement
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
            cascade_inputs,
        );
        result
    }

    /// Given the old and new style of this element, and whether it's a
    /// pseudo-element, compute the restyle damage used to determine which
    /// kind of layout or painting operations we'll need.
    fn compute_style_difference(
        &self,
        old_values: &ComputedValues,
        new_values: &ComputedValues,
        pseudo: Option<&PseudoElement>,
    ) -> StyleDifference {
        debug_assert!(pseudo.map_or(true, |p| p.is_eager()));
        RestyleDamage::compute_style_difference(old_values, new_values)
    }
}

impl<E: TElement> MatchMethods for E {}
