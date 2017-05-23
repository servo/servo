/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! High-level interface to CSS selector matching.

#![allow(unsafe_code)]
#![deny(missing_docs)]

use cascade_info::CascadeInfo;
use context::{SelectorFlagsMap, SharedStyleContext, StyleContext};
use data::{ComputedStyle, ElementData, RestyleData};
use dom::{AnimationRules, TElement, TNode};
use font_metrics::FontMetricsProvider;
use log::LogLevel::Trace;
use properties::{CascadeFlags, ComputedValues, SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP, cascade};
use properties::longhands::display::computed_value as display;
use restyle_hints::{RESTYLE_CSS_ANIMATIONS, RESTYLE_CSS_TRANSITIONS, RestyleReplacements};
use restyle_hints::{RESTYLE_STYLE_ATTRIBUTE, RESTYLE_SMIL};
use rule_tree::{CascadeLevel, RuleTree, StrongRuleNode};
use selector_parser::{PseudoElement, RestyleDamage, SelectorImpl};
use selectors::matching::{ElementSelectorFlags, MatchingContext, MatchingMode, StyleRelations};
use selectors::matching::AFFECTED_BY_PSEUDO_ELEMENTS;
use shared_lock::StylesheetGuards;
use sharing::{StyleSharingBehavior, StyleSharingResult};
use stylearc::Arc;
use stylist::ApplicableDeclarationList;

/// The way a style should be inherited.
enum InheritMode {
    /// Inherit from the parent element, as normal CSS dictates, _or_ from the
    /// closest non-Native Anonymous element in case this is Native Anonymous
    /// Content.
    Normal,
    /// Inherit from the primary style, this is used while computing eager
    /// pseudos, like ::before and ::after when we're traversing the parent.
    FromPrimaryStyle,
}

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
pub enum ChildCascadeRequirement {
    /// Old and new computed values were the same, or we otherwise know that
    /// we won't bother recomputing style for children, so we can skip cascading
    /// the new values into child elements.
    CanSkipCascade,
    /// Old and new computed values were different, so we must cascade the
    /// new values to children.
    ///
    /// FIXME(heycam) Although this is "must" cascade, in the future we should
    /// track whether child elements rely specifically on inheriting particular
    /// property values.  When we do that, we can treat `MustCascade` as "must
    /// cascade unless we know that changes to these properties can be
    /// ignored".
    MustCascade,
}

impl From<StyleChange> for ChildCascadeRequirement {
    fn from(change: StyleChange) -> ChildCascadeRequirement {
        match change {
            StyleChange::Unchanged => ChildCascadeRequirement::CanSkipCascade,
            StyleChange::Changed => ChildCascadeRequirement::MustCascade,
        }
    }
}

/// The result status for match primary rules.
#[derive(Debug)]
pub struct RulesMatchedResult {
    /// Indicate that the rule nodes are changed.
    rule_nodes_changed: bool,
    /// Indicate that there are any changes of important rules overriding animations.
    important_rules_overriding_animation_changed: bool,
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

trait PrivateMatchMethods: TElement {
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
            current = match current.parent_element() {
                Some(el) => el,
                None => return current,
            };

            let is_display_contents =
                current.borrow_data().unwrap().styles().primary.values().is_display_contents();

            if !is_display_contents {
                return current;
            }
        }
    }

    fn cascade_with_rules(&self,
                          shared_context: &SharedStyleContext,
                          font_metrics_provider: &FontMetricsProvider,
                          rule_node: &StrongRuleNode,
                          primary_style: &ComputedStyle,
                          inherit_mode: InheritMode)
                          -> Arc<ComputedValues> {
        let mut cascade_info = CascadeInfo::new();
        let mut cascade_flags = CascadeFlags::empty();
        if self.skip_root_and_item_based_display_fixup() {
            cascade_flags.insert(SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP)
        }

        // Grab the inherited values.
        let parent_el;
        let parent_data;
        let style_to_inherit_from = match inherit_mode {
            InheritMode::Normal => {
                parent_el = self.inheritance_parent();
                parent_data = parent_el.as_ref().and_then(|e| e.borrow_data());
                let parent_values = parent_data.as_ref().map(|d| {
                    // Sometimes Gecko eagerly styles things without processing
                    // pending restyles first. In general we'd like to avoid this,
                    // but there can be good reasons (for example, needing to
                    // construct a frame for some small piece of newly-added
                    // content in order to do something specific with that frame,
                    // but not wanting to flush all of layout).
                    debug_assert!(cfg!(feature = "gecko") ||
                                  parent_el.unwrap().has_current_styles(d));
                    d.styles().primary.values()
                });

                parent_values
            }
            InheritMode::FromPrimaryStyle => {
                parent_el = Some(self.clone());
                Some(primary_style.values())
            }
        };

        let mut layout_parent_el = parent_el.clone();
        let layout_parent_data;
        let mut layout_parent_style = style_to_inherit_from;
        if style_to_inherit_from.map_or(false, |s| s.is_display_contents()) {
            layout_parent_el = Some(layout_parent_el.unwrap().layout_parent());
            layout_parent_data = layout_parent_el.as_ref().unwrap().borrow_data().unwrap();
            layout_parent_style = Some(layout_parent_data.styles().primary.values())
        }

        let style_to_inherit_from = style_to_inherit_from.map(|x| &**x);
        let layout_parent_style = layout_parent_style.map(|x| &**x);

        // Propagate the "can be fragmented" bit. It would be nice to
        // encapsulate this better.
        //
        // Note that this is technically not needed for pseudos since we already
        // do that when we resolve the non-pseudo style, but it doesn't hurt
        // anyway.
        //
        // TODO(emilio): This is servo-only, move somewhere else?
        if let Some(ref p) = layout_parent_style {
            let can_be_fragmented =
                p.is_multicol() ||
                layout_parent_el.as_ref().unwrap().as_node().can_be_fragmented();
            unsafe { self.as_node().set_can_be_fragmented(can_be_fragmented); }
        }

        // Invoke the cascade algorithm.
        let values =
            Arc::new(cascade(shared_context.stylist.device(),
                             rule_node,
                             &shared_context.guards,
                             style_to_inherit_from,
                             layout_parent_style,
                             Some(&mut cascade_info),
                             &*shared_context.error_reporter,
                             font_metrics_provider,
                             cascade_flags,
                             shared_context.quirks_mode));

        cascade_info.finish(&self.as_node());
        values
    }

    fn cascade_internal(&self,
                        context: &StyleContext<Self>,
                        primary_style: &ComputedStyle,
                        eager_pseudo_style: Option<&ComputedStyle>)
                        -> Arc<ComputedValues> {
        if let Some(pseudo) = self.implemented_pseudo_element() {
            debug_assert!(eager_pseudo_style.is_none());

            // This is an element-backed pseudo, just grab the styles from the
            // parent if it's eager, and recascade otherwise.
            //
            // We also recascade if the eager pseudo-style has any animation
            // rules, because we don't cascade those during the eager traversal.
            //
            // We could make that a bit better if the complexity cost is not too
            // big, but given further restyles are posted directly to
            // pseudo-elements, it doesn't seem worth the effort at a glance.
            if pseudo.is_eager() && self.get_animation_rules().is_empty() {
                let parent = self.parent_element().unwrap();
                let parent_data = parent.borrow_data().unwrap();
                let pseudo_style =
                    parent_data.styles().pseudos.get(&pseudo).unwrap();
                return pseudo_style.values().clone()
            }
        }

        // Grab the rule node.
        let rule_node = &eager_pseudo_style.unwrap_or(primary_style).rules;
        let inherit_mode = if eager_pseudo_style.is_some() {
            InheritMode::FromPrimaryStyle
        } else {
            InheritMode::Normal
        };

        self.cascade_with_rules(context.shared,
                                &context.thread_local.font_metrics_provider,
                                rule_node,
                                primary_style,
                                inherit_mode)
    }

    /// Computes values and damage for the primary or pseudo style of an element,
    /// setting them on the ElementData.
    fn cascade_primary(&self,
                       context: &mut StyleContext<Self>,
                       data: &mut ElementData,
                       important_rules_changed: bool)
                       -> ChildCascadeRequirement {
        // Collect some values.
        let (mut styles, restyle) = data.styles_and_restyle_mut();
        let mut primary_style = &mut styles.primary;
        let mut old_values = primary_style.values.take();

        // Compute the new values.
        let mut new_values = self.cascade_internal(context, primary_style, None);

        // NB: Animations for pseudo-elements in Gecko are handled while
        // traversing the pseudo-elements themselves.
        if !context.shared.traversal_flags.for_animation_only() {
            self.process_animations(context,
                                    &mut old_values,
                                    &mut new_values,
                                    primary_style,
                                    important_rules_changed);
        }

        let child_cascade_requirement =
            self.accumulate_damage(&context.shared,
                                   restyle,
                                   old_values.as_ref().map(|v| v.as_ref()),
                                   &new_values,
                                   None);

        // Set the new computed values.
        primary_style.values = Some(new_values);

        // Return whether the damage indicates we must cascade new inherited
        // values into children.
        child_cascade_requirement
    }

    fn cascade_eager_pseudo(&self,
                            context: &mut StyleContext<Self>,
                            data: &mut ElementData,
                            pseudo: &PseudoElement) {
        debug_assert!(pseudo.is_eager());
        let (mut styles, restyle) = data.styles_and_restyle_mut();
        let mut pseudo_style = styles.pseudos.get_mut(pseudo).unwrap();
        let old_values = pseudo_style.values.take();

        let new_values =
            self.cascade_internal(context, &styles.primary, Some(pseudo_style));

        self.accumulate_damage(&context.shared,
                               restyle,
                               old_values.as_ref().map(|v| &**v),
                               &new_values,
                               Some(pseudo));

        pseudo_style.values = Some(new_values)
    }


    /// get_after_change_style removes the transition rules from the ComputedValues.
    /// If there is no transition rule in the ComputedValues, it returns None.
    #[cfg(feature = "gecko")]
    fn get_after_change_style(&self,
                              context: &mut StyleContext<Self>,
                              primary_style: &ComputedStyle)
                              -> Option<Arc<ComputedValues>> {
        let rule_node = &primary_style.rules;
        let without_transition_rules =
            context.shared.stylist.rule_tree().remove_transition_rule_if_applicable(rule_node);
        if without_transition_rules == *rule_node {
            // We don't have transition rule in this case, so return None to let the caller
            // use the original ComputedValues.
            return None;
        }

        Some(self.cascade_with_rules(context.shared,
                                     &context.thread_local.font_metrics_provider,
                                     &without_transition_rules,
                                     primary_style,
                                     InheritMode::Normal))
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

            // If the traverse is triggered by CSS rule changes,
            // we need to try to update all CSS animations.
            context.shared.traversal_flags.for_css_rule_changes() ||
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
                          primary_style: &ComputedStyle,
                          important_rules_changed: bool) {
        use context::{CASCADE_RESULTS, CSS_ANIMATIONS, CSS_TRANSITIONS, EFFECT_PROPERTIES};
        use context::UpdateAnimationsTasks;

        let mut tasks = UpdateAnimationsTasks::empty();
        if self.needs_animations_update(context, old_values.as_ref(), new_values) {
            tasks.insert(CSS_ANIMATIONS);
        }

        let before_change_style = if self.might_need_transitions_update(old_values.as_ref().map(|s| &**s),
                                                                        new_values) {
            let after_change_style = if self.has_css_transitions() {
                self.get_after_change_style(context, primary_style)
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
                          _primary_style: &ComputedStyle,
                          _important_rules_changed: bool) {
        use animation;

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
        // Don't accumulate damage if we're in a restyle for reconstruction.
        if shared_context.traversal_flags.for_reconstruct() {
            return ChildCascadeRequirement::MustCascade;
        }

        // If an ancestor is already getting reconstructed by Gecko's top-down
        // frame constructor, no need to apply damage.  Similarly if we already
        // have an explicitly stored ReconstructFrame hint.
        //
        // See https://bugzilla.mozilla.org/show_bug.cgi?id=1301258#c12
        // for followup work to make the optimization here more optimal by considering
        // each bit individually.
        let skip_applying_damage =
            restyle.damage_handled.contains(RestyleDamage::reconstruct()) ||
            restyle.damage.contains(RestyleDamage::reconstruct());

        let difference = self.compute_style_difference(&old_values,
                                                       &new_values,
                                                       pseudo);
        if !skip_applying_damage {
            restyle.damage |= difference.damage;
        }
        difference.change.into()
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
        difference.change.into()
    }

    #[cfg(feature = "servo")]
    fn update_animations_for_cascade(&self,
                                     context: &SharedStyleContext,
                                     style: &mut Arc<ComputedValues>,
                                     possibly_expired_animations: &mut Vec<::animation::PropertyAnimation>,
                                     font_metrics: &FontMetricsProvider) {
        use animation::{self, Animation};

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

fn compute_rule_node<E: TElement>(rule_tree: &RuleTree,
                                  applicable_declarations: &mut ApplicableDeclarationList,
                                  guards: &StylesheetGuards)
                                  -> StrongRuleNode
{
    let rules = applicable_declarations.drain().map(|d| (d.source, d.level));
    let rule_node = rule_tree.insert_ordered_rules_with_important(rules, guards);
    rule_node
}

impl<E: TElement> PrivateMatchMethods for E {}

/// The public API that elements expose for selector matching.
pub trait MatchMethods : TElement {
    /// Performs selector matching and property cascading on an element and its
    /// eager pseudos.
    fn match_and_cascade(&self,
                         context: &mut StyleContext<Self>,
                         data: &mut ElementData,
                         sharing: StyleSharingBehavior)
                         -> ChildCascadeRequirement
    {
        // Perform selector matching for the primary style.
        let mut relations = StyleRelations::empty();
        let result = self.match_primary(context, data, &mut relations);

        // Cascade properties and compute primary values.
        let child_cascade_requirement =
            self.cascade_primary(
                context,
                data,
                result.important_rules_overriding_animation_changed
            );

        // Match and cascade eager pseudo-elements.
        if !data.styles().is_display_none() {
            let _pseudo_rule_nodes_changed = self.match_pseudos(context, data);
            self.cascade_pseudos(context, data);
        }

        // If we have any pseudo elements, indicate so in the primary StyleRelations.
        if !data.styles().pseudos.is_empty() {
            relations |= AFFECTED_BY_PSEUDO_ELEMENTS;
        }

        // If the style is shareable, add it to the LRU cache.
        if sharing == StyleSharingBehavior::Allow {
            // If we previously tried to match this element against the cache,
            // the revalidation match results will already be cached. Otherwise
            // we'll have None, and compute them later on-demand.
            //
            // If we do have the results, grab them here to satisfy the borrow
            // checker.
            let revalidation_match_results = context.thread_local
                                                    .current_element_info
                                                    .as_mut().unwrap()
                                                    .revalidation_match_results
                                                    .take();
            context.thread_local
                   .style_sharing_candidate_cache
                   .insert_if_possible(self,
                                       data.styles().primary.values(),
                                       relations,
                                       revalidation_match_results);
        }

        child_cascade_requirement
    }

    /// Performs the cascade, without matching.
    fn cascade_primary_and_pseudos(&self,
                                   context: &mut StyleContext<Self>,
                                   mut data: &mut ElementData,
                                   important_rules_changed: bool)
                                   -> ChildCascadeRequirement
    {
        let child_cascade_requirement =
            self.cascade_primary(context, &mut data, important_rules_changed);
        self.cascade_pseudos(context, &mut data);
        child_cascade_requirement
    }

    /// Runs selector matching to (re)compute the primary rule node for this element.
    ///
    /// Returns RulesMatchedResult which indicates whether the primary rule node changed
    /// and whether the change includes important rules.
    fn match_primary(&self,
                     context: &mut StyleContext<Self>,
                     data: &mut ElementData,
                     relations: &mut StyleRelations)
                     -> RulesMatchedResult
    {
        let implemented_pseudo = self.implemented_pseudo_element();
        if let Some(ref pseudo) = implemented_pseudo {
            if pseudo.is_eager() {
                // If it's an eager element-backed pseudo, just grab the matched
                // rules from the parent, and update animations.
                let parent = self.parent_element().unwrap();
                let parent_data = parent.borrow_data().unwrap();
                let pseudo_style =
                    parent_data.styles().pseudos.get(&pseudo).unwrap();
                let mut rules = pseudo_style.rules.clone();
                let animation_rules = self.get_animation_rules();

                // Handle animations here.
                if let Some(animation_rule) = animation_rules.0 {
                    let animation_rule_node =
                        context.shared.stylist.rule_tree()
                            .update_rule_at_level(CascadeLevel::Animations,
                                                  Some(&animation_rule),
                                                  &mut rules,
                                                  &context.shared.guards);
                    if let Some(node) = animation_rule_node {
                        rules = node;
                    }
                }

                if let Some(animation_rule) = animation_rules.1 {
                    let animation_rule_node =
                        context.shared.stylist.rule_tree()
                            .update_rule_at_level(CascadeLevel::Transitions,
                                                  Some(&animation_rule),
                                                  &mut rules,
                                                  &context.shared.guards);
                    if let Some(node) = animation_rule_node {
                        rules = node;
                    }
                }

                let important_rules_changed =
                    self.has_animations() &&
                    data.has_styles() &&
                    data.important_rules_are_different(&rules,
                                                       &context.shared.guards);

                return RulesMatchedResult {
                    rule_nodes_changed: data.set_primary_rules(rules),
                    important_rules_overriding_animation_changed: important_rules_changed,
                };
            }
        }

        let mut applicable_declarations = ApplicableDeclarationList::new();

        let stylist = &context.shared.stylist;
        let style_attribute = self.style_attribute();
        let smil_override = self.get_smil_override();
        let animation_rules = self.get_animation_rules();
        let bloom = context.thread_local.bloom_filter.filter();


        let map = &mut context.thread_local.selector_flags;
        let mut set_selector_flags = |element: &Self, flags: ElementSelectorFlags| {
            self.apply_selector_flags(map, element, flags);
        };

        let mut matching_context =
            MatchingContext::new(MatchingMode::Normal, Some(bloom));

        // Compute the primary rule node.
        stylist.push_applicable_declarations(self,
                                             implemented_pseudo.as_ref(),
                                             style_attribute,
                                             smil_override,
                                             animation_rules,
                                             &mut applicable_declarations,
                                             &mut matching_context,
                                             &mut set_selector_flags);

        *relations = matching_context.relations;

        let primary_rule_node =
            compute_rule_node::<Self>(stylist.rule_tree(),
                                      &mut applicable_declarations,
                                      &context.shared.guards);

        if log_enabled!(Trace) {
            trace!("Matched rules:");
            for rn in primary_rule_node.self_and_ancestors() {
                if let Some(source) = rn.style_source() {
                    trace!(" > {:?}", source);
                }
            }
        }

        let important_rules_changed =
            self.has_animations() &&
            data.has_styles() &&
            data.important_rules_are_different(
                &primary_rule_node,
                &context.shared.guards
            );

        RulesMatchedResult {
            rule_nodes_changed: data.set_primary_rules(primary_rule_node),
            important_rules_overriding_animation_changed: important_rules_changed,
        }
    }

    /// Runs selector matching to (re)compute eager pseudo-element rule nodes
    /// for this element.
    ///
    /// Returns whether any of the pseudo rule nodes changed (including, but not
    /// limited to, cases where we match different pseudos altogether).
    fn match_pseudos(&self,
                     context: &mut StyleContext<Self>,
                     data: &mut ElementData)
                     -> bool
    {
        if self.implemented_pseudo_element().is_some() {
            // Element pseudos can't have any other pseudo.
            return false;
        }

        let mut applicable_declarations = ApplicableDeclarationList::new();

        let map = &mut context.thread_local.selector_flags;
        let mut set_selector_flags = |element: &Self, flags: ElementSelectorFlags| {
            self.apply_selector_flags(map, element, flags);
        };

        // Borrow the stuff we need here so the borrow checker doesn't get mad
        // at us later in the closure.
        let stylist = &context.shared.stylist;
        let guards = &context.shared.guards;
        let rule_tree = stylist.rule_tree();
        let bloom_filter = context.thread_local.bloom_filter.filter();

        let mut matching_context =
            MatchingContext::new(MatchingMode::ForStatelessPseudoElement,
                                 Some(bloom_filter));

        // Compute rule nodes for eagerly-cascaded pseudo-elements.
        let mut matches_different_pseudos = false;
        let mut rule_nodes_changed = false;
        SelectorImpl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            let mut pseudos = &mut data.styles_mut().pseudos;
            debug_assert!(applicable_declarations.is_empty());
            // NB: We handle animation rules for ::before and ::after when
            // traversing them.
            stylist.push_applicable_declarations(self,
                                                 Some(&pseudo),
                                                 None,
                                                 None,
                                                 AnimationRules(None, None),
                                                 &mut applicable_declarations,
                                                 &mut matching_context,
                                                 &mut set_selector_flags);

            if !applicable_declarations.is_empty() {
                let new_rules =
                    compute_rule_node::<Self>(rule_tree,
                                              &mut applicable_declarations,
                                              &guards);
                if pseudos.has(&pseudo) {
                    rule_nodes_changed = pseudos.set_rules(&pseudo, new_rules);
                } else {
                    pseudos.insert(&pseudo, ComputedStyle::new_partial(new_rules));
                    matches_different_pseudos = true;
                }
            } else if pseudos.take(&pseudo).is_some() {
                matches_different_pseudos = true;
            }
        });

        if matches_different_pseudos {
            rule_nodes_changed = true;
            if let Some(r) = data.get_restyle_mut() {
                // Any changes to the matched pseudo-elements trigger
                // reconstruction.
                r.damage |= RestyleDamage::reconstruct();
            }
        }

        rule_nodes_changed
    }

    /// Applies selector flags to an element, deferring mutations of the parent
    /// until after the traversal.
    ///
    /// TODO(emilio): This is somewhat inefficient, because of a variety of
    /// reasons:
    ///
    ///  * It doesn't coalesce flags.
    ///  * It doesn't look at flags already sent in a task for the main
    ///    thread to process.
    ///  * It doesn't take advantage of us knowing that the traversal is
    ///    sequential.
    ///
    /// I suspect (need to measure!) that we don't use to set flags on
    /// a lot of different elements, but we could end up posting the same
    /// flag over and over with this approach.
    ///
    /// If the number of elements is low, perhaps a small cache with the
    /// flags already sent would be appropriate.
    ///
    /// The sequential task business for this is kind of sad :(.
    ///
    /// Anyway, let's do the obvious thing for now.
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
                         restyle: Option<&mut RestyleData>,
                         old_values: Option<&ComputedValues>,
                         new_values: &Arc<ComputedValues>,
                         pseudo: Option<&PseudoElement>)
                         -> ChildCascadeRequirement {
        let restyle = match restyle {
            Some(r) => r,
            None => return ChildCascadeRequirement::MustCascade,
        };

        let old_values = match old_values {
            Some(v) => v,
            None => return ChildCascadeRequirement::MustCascade,
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
    /// the rule tree. Returns RulesChanged which indicates whether the rule nodes changed
    /// and whether the important rules changed.
    fn replace_rules(&self,
                     replacements: RestyleReplacements,
                     context: &StyleContext<Self>,
                     data: &mut ElementData)
                     -> RulesChanged {
        use properties::PropertyDeclarationBlock;
        use shared_lock::Locked;

        let element_styles = &mut data.styles_mut();
        let primary_rules = &mut element_styles.primary.rules;
        let mut result = RulesChanged::empty();

        {
            let mut replace_rule_node = |level: CascadeLevel,
                                         pdb: Option<&Arc<Locked<PropertyDeclarationBlock>>>,
                                         path: &mut StrongRuleNode| {
                let new_node = context.shared.stylist.rule_tree()
                    .update_rule_at_level(level, pdb, path, &context.shared.guards);
                if let Some(n) = new_node {
                    *path = n;
                    if level.is_important() {
                        result.insert(IMPORTANT_RULES_CHANGED);
                    } else {
                        result.insert(NORMAL_RULES_CHANGED);
                    }
                }
            };

            // Animation restyle hints are processed prior to other restyle
            // hints in the animation-only traversal.
            //
            // Non-animation restyle hints will be processed in a subsequent
            // normal traversal.
            if replacements.intersects(RestyleReplacements::for_animations()) {
                debug_assert!(context.shared.traversal_flags.for_animation_only());

                if replacements.contains(RESTYLE_SMIL) {
                    replace_rule_node(CascadeLevel::SMILOverride,
                                      self.get_smil_override(),
                                      primary_rules);
                }

                let mut replace_rule_node_for_animation = |level: CascadeLevel,
                                                           primary_rules: &mut StrongRuleNode| {
                    let animation_rule = self.get_animation_rule_by_cascade(level);
                    replace_rule_node(level,
                                      animation_rule.as_ref(),
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
            } else if replacements.contains(RESTYLE_STYLE_ATTRIBUTE) {
                let style_attribute = self.style_attribute();
                replace_rule_node(CascadeLevel::StyleAttributeNormal,
                                  style_attribute,
                                  primary_rules);
                replace_rule_node(CascadeLevel::StyleAttributeImportant,
                                  style_attribute,
                                  primary_rules);
            }
        }

        result
    }

    /// Attempts to share a style with another node. This method is unsafe
    /// because it depends on the `style_sharing_candidate_cache` having only
    /// live nodes in it, and we have no way to guarantee that at the type
    /// system level yet.
    unsafe fn share_style_if_possible(&self,
                                      context: &mut StyleContext<Self>,
                                      data: &mut ElementData)
                                      -> StyleSharingResult {
        let shared_context = &context.shared;
        let current_element_info =
            context.thread_local.current_element_info.as_mut().unwrap();
        let selector_flags_map = &mut context.thread_local.selector_flags;
        let bloom_filter = context.thread_local.bloom_filter.filter();

        context.thread_local
            .style_sharing_candidate_cache
            .share_style_if_possible(shared_context,
                                     current_element_info,
                                     selector_flags_map,
                                     bloom_filter,
                                     *self,
                                     data)
    }

    /// Given the old and new style of this element, and whether it's a
    /// pseudo-element, compute the restyle damage used to determine which
    /// kind of layout or painting operations we'll need.
    fn compute_style_difference(&self,
                                old_values: &ComputedValues,
                                new_values: &Arc<ComputedValues>,
                                pseudo: Option<&PseudoElement>)
                                -> StyleDifference
    {
        if let Some(source) = self.existing_style_for_restyle_damage(old_values, pseudo) {
            return RestyleDamage::compute_style_difference(source, new_values)
        }

        let new_style_is_display_none =
            new_values.get_box().clone_display() == display::T::none;
        let old_style_is_display_none =
            old_values.get_box().clone_display() == display::T::none;

        // If there's no style source, that likely means that Gecko couldn't
        // find a style context.
        //
        // This happens with display:none elements, and not-yet-existing
        // pseudo-elements.
        if new_style_is_display_none && old_style_is_display_none {
            // The style remains display:none. No need for damage.
            return StyleDifference::new(RestyleDamage::empty(), StyleChange::Unchanged)
        }

        if pseudo.map_or(false, |p| p.is_before_or_after()) {
            if (old_style_is_display_none ||
                old_values.ineffective_content_property()) &&
               (new_style_is_display_none ||
                new_values.ineffective_content_property()) {
                // The pseudo-element will remain undisplayed, so just avoid
                // triggering any change.
                return StyleDifference::new(RestyleDamage::empty(), StyleChange::Unchanged)
            }
            return StyleDifference::new(RestyleDamage::reconstruct(), StyleChange::Changed)
        }

        // Something else. Be conservative for now.
        warn!("Reframing due to lack of old style source: {:?}, pseudo: {:?}",
               self, pseudo);
        // Something else. Be conservative for now.
        StyleDifference::new(RestyleDamage::reconstruct(), StyleChange::Changed)
    }

    /// Performs the cascade for the element's eager pseudos.
    fn cascade_pseudos(&self,
                       context: &mut StyleContext<Self>,
                       mut data: &mut ElementData)
    {
        // Note that we've already set up the map of matching pseudo-elements
        // in match_pseudos (and handled the damage implications of changing
        // which pseudos match), so now we can just iterate what we have. This
        // does mean collecting owned pseudos, so that the borrow checker will
        // let us pass the mutable |data| to the cascade function.
        let matched_pseudos = data.styles().pseudos.keys();
        for pseudo in matched_pseudos {
            self.cascade_eager_pseudo(context, data, &pseudo);
        }
    }

    /// Returns computed values without animation and transition rules.
    fn get_base_style(&self,
                      shared_context: &SharedStyleContext,
                      font_metrics_provider: &FontMetricsProvider,
                      primary_style: &ComputedStyle,
                      pseudo_style: Option<&ComputedStyle>)
                      -> Arc<ComputedValues> {
        let relevant_style = pseudo_style.unwrap_or(primary_style);
        let rule_node = &relevant_style.rules;
        let without_animation_rules =
            shared_context.stylist.rule_tree().remove_animation_rules(rule_node);
        if without_animation_rules == *rule_node {
            // Note that unwrapping here is fine, because the style is
            // only incomplete during the styling process.
            return relevant_style.values.as_ref().unwrap().clone();
        }

        self.cascade_with_rules(shared_context,
                                font_metrics_provider,
                                &without_animation_rules,
                                primary_style,
                                InheritMode::Normal)
    }

}

impl<E: TElement> MatchMethods for E {}
