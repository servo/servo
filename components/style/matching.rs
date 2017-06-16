/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! High-level interface to CSS selector matching.

#![allow(unsafe_code)]
#![deny(missing_docs)]

use applicable_declarations::ApplicableDeclarationList;
use cascade_info::CascadeInfo;
use context::{SelectorFlagsMap, SharedStyleContext, StyleContext};
use data::{ComputedStyle, ElementData, RestyleData};
use dom::{TElement, TNode};
use font_metrics::FontMetricsProvider;
use invalidation::element::restyle_hints::{RESTYLE_CSS_ANIMATIONS, RESTYLE_CSS_TRANSITIONS};
use invalidation::element::restyle_hints::{RESTYLE_SMIL, RESTYLE_STYLE_ATTRIBUTE};
use invalidation::element::restyle_hints::RestyleHint;
use log::LogLevel::Trace;
use properties::{ALLOW_SET_ROOT_FONT_SIZE, PROHIBIT_DISPLAY_CONTENTS, SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP};
use properties::{AnimationRules, CascadeFlags, ComputedValues};
use properties::{VISITED_DEPENDENT_ONLY, cascade};
use properties::longhands::display::computed_value as display;
use rule_tree::{CascadeLevel, StrongRuleNode};
use selector_parser::{PseudoElement, RestyleDamage, SelectorImpl};
use selectors::matching::{ElementSelectorFlags, MatchingContext, MatchingMode, StyleRelations};
use selectors::matching::{VisitedHandlingMode, AFFECTED_BY_PSEUDO_ELEMENTS};
use sharing::StyleSharingBehavior;
use stylearc::Arc;
use stylist::RuleInclusion;

/// Whether we are cascading for an eager pseudo-element or something else.
///
/// Controls where we inherit styles from, and whether display:contents is
/// prohibited.
#[derive(PartialEq, Copy, Clone)]
enum CascadeTarget {
    /// Inherit from the parent element, as normal CSS dictates, _or_ from the
    /// closest non-Native Anonymous element in case this is Native Anonymous
    /// Content. display:contents is allowed.
    Normal,
    /// Inherit from the primary style, this is used while computing eager
    /// pseudos, like ::before and ::after when we're traversing the parent.
    /// Also prohibits display:contents from having an effect.
    ///
    /// TODO(emilio) display:contents really should apply to ::before/::after.
    /// https://github.com/w3c/csswg-drafts/issues/1345
    EagerPseudo,
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
    /// property values.  When we do that, we can treat `MustCascadeChildren` as
    /// "must cascade unless we know that changes to these properties can be
    /// ignored".
    MustCascadeChildren,
    /// The same as `MustCascadeChildren`, but for the entire subtree.  This is
    /// used to handle root font-size updates needing to recascade the whole
    /// document.
    MustCascadeDescendants,
}

impl From<StyleChange> for ChildCascadeRequirement {
    fn from(change: StyleChange) -> ChildCascadeRequirement {
        match change {
            StyleChange::Unchanged => ChildCascadeRequirement::CanSkipCascade,
            StyleChange::Changed => ChildCascadeRequirement::MustCascadeChildren,
        }
    }
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

/// Various helper methods to ease navigating the style storage locations
/// depending on the current cascade mode.
impl CascadeVisitedMode {
    /// Returns whether there is a rule node based on the cascade mode.
    fn has_rules(&self, style: &ComputedStyle) -> bool {
        match *self {
            CascadeVisitedMode::Unvisited => true,
            CascadeVisitedMode::Visited => style.has_visited_rules(),
        }
    }

    /// Returns the rule node based on the cascade mode.
    fn rules<'a>(&self, style: &'a ComputedStyle) -> &'a StrongRuleNode {
        match *self {
            CascadeVisitedMode::Unvisited => &style.rules,
            CascadeVisitedMode::Visited => style.visited_rules(),
        }
    }

    /// Returns a mutable rules node based on the cascade mode, if any.
    fn get_rules_mut<'a>(&self, style: &'a mut ComputedStyle) -> Option<&'a mut StrongRuleNode> {
        match *self {
            CascadeVisitedMode::Unvisited => Some(&mut style.rules),
            CascadeVisitedMode::Visited => style.get_visited_rules_mut(),
        }
    }

    /// Returns the computed values based on the cascade mode.  In visited mode,
    /// visited values are only returned if they already exist.  If they don't,
    /// we fallback to the regular, unvisited styles.
    fn values<'a>(&self, style: &'a ComputedStyle) -> &'a Arc<ComputedValues> {
        let mut values = style.values();

        if *self == CascadeVisitedMode::Visited && values.get_visited_style().is_some() {
            values = values.visited_style();
        }

        values
    }

    /// Set the computed values based on the cascade mode.
    fn set_values(&self, style: &mut ComputedStyle, values: Arc<ComputedValues>) {
        match *self {
            CascadeVisitedMode::Unvisited => style.values = Some(values),
            CascadeVisitedMode::Visited => style.set_visited_values(values),
        }
    }

    /// Take the computed values based on the cascade mode.
    fn take_values(&self, style: &mut ComputedStyle) -> Option<Arc<ComputedValues>> {
        match *self {
            CascadeVisitedMode::Unvisited => style.values.take(),
            CascadeVisitedMode::Visited => style.take_visited_values(),
        }
    }

    /// Returns whether there might be visited values that should be inserted
    /// within the regular computed values based on the cascade mode.
    fn visited_values_for_insertion(&self) -> bool {
        *self == CascadeVisitedMode::Unvisited
    }

    /// Returns whether animations should be processed based on the cascade
    /// mode.  At the moment, it appears we don't need to support animating
    /// visited styles.
    fn should_process_animations(&self) -> bool {
        *self == CascadeVisitedMode::Unvisited
    }

    /// Returns whether we should accumulate restyle damage based on the cascade
    /// mode.  At the moment, it appears we don't need to do so for visited
    /// styles.  TODO: Verify this is correct as part of
    /// https://bugzilla.mozilla.org/show_bug.cgi?id=1364484.
    fn should_accumulate_damage(&self) -> bool {
        *self == CascadeVisitedMode::Unvisited
    }

    /// Returns whether the cascade should filter to only visited dependent
    /// properties based on the cascade mode.
    fn visited_dependent_only(&self) -> bool {
        *self == CascadeVisitedMode::Visited
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
            current = match current.traversal_parent() {
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
                          cascade_target: CascadeTarget,
                          cascade_visited: CascadeVisitedMode,
                          visited_values_to_insert: Option<Arc<ComputedValues>>)
                          -> Arc<ComputedValues> {
        let mut cascade_info = CascadeInfo::new();
        let mut cascade_flags = CascadeFlags::empty();
        if self.skip_root_and_item_based_display_fixup() {
            cascade_flags.insert(SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP)
        }
        if cascade_visited.visited_dependent_only() {
            cascade_flags.insert(VISITED_DEPENDENT_ONLY);
        }
        if self.is_native_anonymous() || cascade_target == CascadeTarget::EagerPseudo {
            cascade_flags.insert(PROHIBIT_DISPLAY_CONTENTS);
        } else {
            cascade_flags.insert(ALLOW_SET_ROOT_FONT_SIZE);
        }

        // Grab the inherited values.
        let parent_el;
        let parent_data;
        let style_to_inherit_from = match cascade_target {
            CascadeTarget::Normal => {
                parent_el = self.inheritance_parent();
                parent_data = parent_el.as_ref().and_then(|e| e.borrow_data());
                let parent_style = parent_data.as_ref().map(|d| {
                    // Sometimes Gecko eagerly styles things without processing
                    // pending restyles first. In general we'd like to avoid this,
                    // but there can be good reasons (for example, needing to
                    // construct a frame for some small piece of newly-added
                    // content in order to do something specific with that frame,
                    // but not wanting to flush all of layout).
                    debug_assert!(cfg!(feature = "gecko") ||
                                  parent_el.unwrap().has_current_styles(d));
                    &d.styles().primary
                });
                parent_style.map(|s| cascade_visited.values(s))
            }
            CascadeTarget::EagerPseudo => {
                parent_el = Some(self.clone());
                Some(cascade_visited.values(primary_style))
            }
        };

        let mut layout_parent_el = parent_el.clone();
        let layout_parent_data;
        let mut layout_parent_style = style_to_inherit_from;
        if style_to_inherit_from.map_or(false, |s| s.is_display_contents()) {
            layout_parent_el = Some(layout_parent_el.unwrap().layout_parent());
            layout_parent_data = layout_parent_el.as_ref().unwrap().borrow_data().unwrap();
            layout_parent_style = Some(cascade_visited.values(&layout_parent_data.styles().primary));
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
                             visited_values_to_insert,
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
                        eager_pseudo_style: Option<&ComputedStyle>,
                        cascade_visited: CascadeVisitedMode)
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
            //
            // For the same reason as described in match_primary, if we are
            // computing default styles, we aren't guaranteed the parent
            // will have eagerly computed our styles, so we just handled it
            // below like a lazy pseudo.
            let only_default_rules = context.shared.traversal_flags.for_default_styles();
            if pseudo.is_eager() && !only_default_rules {
                debug_assert!(pseudo.is_before_or_after());
                let parent = self.parent_element().unwrap();
                if !parent.may_have_animations() ||
                   primary_style.rules.get_animation_rules().is_empty() {
                    let parent_data = parent.borrow_data().unwrap();
                    let pseudo_style =
                        parent_data.styles().pseudos.get(&pseudo).unwrap();
                    let values = cascade_visited.values(pseudo_style);
                    return values.clone()
                }
            }
        }

        // Find possible visited computed styles to insert within the regular
        // computed values we are about to create.
        let visited_values_to_insert = if cascade_visited.visited_values_for_insertion() {
            match eager_pseudo_style {
                Some(ref s) => s.clone_visited_values(),
                None => primary_style.clone_visited_values(),
            }
        } else {
            None
        };

        // Grab the rule node.
        let style = eager_pseudo_style.unwrap_or(primary_style);
        let rule_node = cascade_visited.rules(style);
        let cascade_target = if eager_pseudo_style.is_some() {
            CascadeTarget::EagerPseudo
        } else {
            CascadeTarget::Normal
        };

        self.cascade_with_rules(context.shared,
                                &context.thread_local.font_metrics_provider,
                                rule_node,
                                primary_style,
                                cascade_target,
                                cascade_visited,
                                visited_values_to_insert)
    }

    /// Computes values and damage for the primary style of an element, setting
    /// them on the ElementData.
    fn cascade_primary(&self,
                       context: &mut StyleContext<Self>,
                       data: &mut ElementData,
                       important_rules_changed: bool,
                       cascade_visited: CascadeVisitedMode)
                       -> ChildCascadeRequirement {
        debug!("Cascade primary for {:?}, visited: {:?}", self, cascade_visited);

        // Collect some values.
        let (mut styles, restyle) = data.styles_and_restyle_mut();
        let mut primary_style = &mut styles.primary;
        // If there was no relevant link, we won't have any visited rules, so
        // there may not be anything do for the visited case.  This early return
        // is especially important for the `cascade_primary_and_pseudos` path
        // since we rely on the state of some previous matching run.
        if !cascade_visited.has_rules(primary_style) {
            return ChildCascadeRequirement::CanSkipCascade
        }
        let mut old_values = cascade_visited.take_values(primary_style);

        // Compute the new values.
        let mut new_values = self.cascade_internal(context,
                                                   primary_style,
                                                   None,
                                                   cascade_visited);

        // NB: Animations for pseudo-elements in Gecko are handled while
        // traversing the pseudo-elements themselves.
        if !context.shared.traversal_flags.for_animation_only() &&
           cascade_visited.should_process_animations() {
            self.process_animations(context,
                                    &mut old_values,
                                    &mut new_values,
                                    primary_style,
                                    important_rules_changed);
        }

        let mut child_cascade_requirement =
            ChildCascadeRequirement::CanSkipCascade;
        if cascade_visited.should_accumulate_damage() {
            child_cascade_requirement =
                self.accumulate_damage(&context.shared,
                                       restyle,
                                       old_values.as_ref().map(|v| v.as_ref()),
                                       &new_values,
                                       None);

            // Handle root font-size changes.
            if self.is_root() && !self.is_native_anonymous() {
                // The new root font-size has already been updated on the Device
                // in properties::apply_declarations.
                let device = context.shared.stylist.device();
                let new_font_size = new_values.get_font().clone_font_size();

                // If the root font-size changed since last time, and something
                // in the document did use rem units, ensure we recascade the
                // entire tree.
                if old_values.map_or(false, |v| v.get_font().clone_font_size() != new_font_size) &&
                   device.used_root_font_size() {
                    child_cascade_requirement = ChildCascadeRequirement::MustCascadeDescendants;
                }
            }
        }

        // Set the new computed values.
        cascade_visited.set_values(primary_style, new_values);

        // Return whether the damage indicates we must cascade new inherited
        // values into children.
        child_cascade_requirement
    }

    /// Computes values and damage for the eager pseudo-element styles of an
    /// element, setting them on the ElementData.
    fn cascade_eager_pseudo(&self,
                            context: &mut StyleContext<Self>,
                            data: &mut ElementData,
                            pseudo: &PseudoElement,
                            cascade_visited: CascadeVisitedMode) {
        debug_assert!(pseudo.is_eager());
        let (mut styles, restyle) = data.styles_and_restyle_mut();
        let mut pseudo_style = styles.pseudos.get_mut(pseudo).unwrap();
        // If there was no relevant link, we won't have any visited rules, so
        // there may not be anything do for the visited case.  This early return
        // is especially important for the `cascade_primary_and_pseudos` path
        // since we rely on the state of some previous matching run.
        if !cascade_visited.has_rules(pseudo_style) {
            return
        }
        let old_values = cascade_visited.take_values(pseudo_style);

        let new_values = self.cascade_internal(context,
                                               &styles.primary,
                                               Some(pseudo_style),
                                               cascade_visited);

        if cascade_visited.should_accumulate_damage() {
            self.accumulate_damage(&context.shared,
                                   restyle,
                                   old_values.as_ref().map(|v| &**v),
                                   &new_values,
                                   Some(pseudo));
        }

        cascade_visited.set_values(pseudo_style, new_values);
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

        // This currently ignores visited styles, which seems acceptable,
        // as existing browsers don't appear to transition visited styles.
        Some(self.cascade_with_rules(context.shared,
                                     &context.thread_local.font_metrics_provider,
                                     &without_transition_rules,
                                     primary_style,
                                     CascadeTarget::Normal,
                                     CascadeVisitedMode::Unvisited,
                                     None))
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

impl<E: TElement> PrivateMatchMethods for E {}

/// Collects the outputs of the primary matching process, including the rule
/// node and other associated data.
#[derive(Debug)]
pub struct MatchingResults {
    /// Whether the rules changed.
    rules_changed: bool,
    /// Whether there are any changes of important rules overriding animations.
    important_rules_overriding_animation_changed: bool,
    /// Records certains relations between elements noticed during matching (and
    /// also extended after matching).
    relations: StyleRelations,
    /// Whether we encountered a "relevant link" while matching _any_ selector
    /// for this element. (This differs from `RelevantLinkStatus` which tracks
    /// the status for the _current_ selector only.)
    relevant_link_found: bool,
}

impl MatchingResults {
    /// Create `MatchingResults` with only the basic required outputs.
    fn new(rules_changed: bool, important_rules: bool) -> Self {
        Self {
            rules_changed: rules_changed,
            important_rules_overriding_animation_changed: important_rules,
            relations: StyleRelations::default(),
            relevant_link_found: false,
        }
    }

    /// Create `MatchingResults` from the output fields of `MatchingContext`.
    fn new_from_context(rules_changed: bool,
                        important_rules: bool,
                        context: MatchingContext)
                        -> Self {
        Self {
            rules_changed: rules_changed,
            important_rules_overriding_animation_changed: important_rules,
            relations: context.relations,
            relevant_link_found: context.relevant_link_found,
        }
    }
}

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
        debug!("Match and cascade for {:?}", self);

        // Perform selector matching for the primary style.
        let mut primary_results =
            self.match_primary(context, data, VisitedHandlingMode::AllLinksUnvisited);
        let important_rules_changed =
            primary_results.important_rules_overriding_animation_changed;

        // If there's a relevant link involved, match and cascade primary styles
        // as if the link is visited as well.  This is done before the regular
        // cascade because the visited ComputedValues are placed within the
        // regular ComputedValues, which is immutable after the cascade.
        let relevant_link_found = primary_results.relevant_link_found;
        if relevant_link_found {
            self.match_primary(context, data, VisitedHandlingMode::RelevantLinkVisited);
            self.cascade_primary(context, data, important_rules_changed,
                                 CascadeVisitedMode::Visited);
        }

        // Cascade properties and compute primary values.
        let child_cascade_requirement =
            self.cascade_primary(context, data, important_rules_changed,
                                 CascadeVisitedMode::Unvisited);

        // Match and cascade eager pseudo-elements.
        if !data.styles().is_display_none() {
            self.match_pseudos(context, data, VisitedHandlingMode::AllLinksUnvisited);

            // If there's a relevant link involved, match and cascade eager
            // pseudo-element styles as if the link is visited as well.
            // This runs after matching for regular styles because matching adds
            // each pseudo as needed to the PseudoMap, and this runs before
            // cascade for regular styles because the visited ComputedValues
            // are placed within the regular ComputedValues, which is immutable
            // after the cascade.
            if relevant_link_found {
                self.match_pseudos(context, data, VisitedHandlingMode::RelevantLinkVisited);
                self.cascade_pseudos(context, data, CascadeVisitedMode::Visited);
            }

            self.cascade_pseudos(context, data, CascadeVisitedMode::Unvisited);
        }

        // If we have any pseudo elements, indicate so in the primary StyleRelations.
        if !data.styles().pseudos.is_empty() {
            primary_results.relations |= AFFECTED_BY_PSEUDO_ELEMENTS;
        }

        // If the style is shareable, add it to the LRU cache.
        if sharing == StyleSharingBehavior::Allow {
            // If we previously tried to match this element against the cache,
            // the revalidation match results will already be cached. Otherwise
            // we'll have None, and compute them later on-demand.
            //
            // If we do have the results, grab them here to satisfy the borrow
            // checker.
            let validation_data =
                context.thread_local
                    .current_element_info
                    .as_mut().unwrap()
                    .validation_data
                    .take();

            let dom_depth = context.thread_local.bloom_filter.matching_depth();
            context.thread_local
                   .style_sharing_candidate_cache
                   .insert_if_possible(self,
                                       data.styles().primary.values(),
                                       primary_results.relations,
                                       validation_data,
                                       dom_depth);
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
        // If there's a relevant link involved, cascade styles as if the link is
        // visited as well. This is done before the regular cascade because the
        // visited ComputedValues are placed within the regular ComputedValues,
        // which is immutable after the cascade.  If there aren't any visited
        // rules, these calls will return without cascading.
        self.cascade_primary(context, &mut data, important_rules_changed,
                             CascadeVisitedMode::Visited);
        let child_cascade_requirement =
            self.cascade_primary(context, &mut data, important_rules_changed,
                                 CascadeVisitedMode::Unvisited);
        self.cascade_pseudos(context, &mut data, CascadeVisitedMode::Visited);
        self.cascade_pseudos(context, &mut data, CascadeVisitedMode::Unvisited);
        child_cascade_requirement
    }

    /// Runs selector matching to (re)compute the primary rule node for this
    /// element.
    ///
    /// Returns `MatchingResults` with the new rules and other associated data
    /// from the matching process.
    fn match_primary(&self,
                     context: &mut StyleContext<Self>,
                     data: &mut ElementData,
                     visited_handling: VisitedHandlingMode)
                     -> MatchingResults
    {
        debug!("Match primary for {:?}, visited: {:?}", self, visited_handling);

        let only_default_rules = context.shared.traversal_flags.for_default_styles();
        let implemented_pseudo = self.implemented_pseudo_element();
        if let Some(ref pseudo) = implemented_pseudo {
            // We don't expect to match against a non-canonical pseudo-element.
            debug_assert_eq!(*pseudo, pseudo.canonical());
            if pseudo.is_eager() && !only_default_rules {
                // If it's an eager element-backed pseudo, we can generally just
                // grab the matched rules from the parent, and then update
                // animations.
                //
                // However, if we're computing default styles, then we might
                // have traversed to this pseudo-implementing element without
                // any pseudo styles stored on the parent.  For example, if
                // document-level style sheets cause the element to exist, due
                // to ::before rules, then those rules won't be found when
                // computing default styles on the parent, so we won't have
                // bothered to store pseudo styles there.  In this case, we just
                // treat it like a lazily computed pseudo.
                let parent = self.parent_element().unwrap();
                let parent_data = parent.borrow_data().unwrap();
                let pseudo_style =
                    parent_data.styles().pseudos.get(&pseudo).unwrap();
                let mut rules = pseudo_style.rules.clone();
                if parent.may_have_animations() {
                    let animation_rules = data.get_animation_rules();

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
                }
                let important_rules_changed =
                    self.has_animations() &&
                    data.has_styles() &&
                    data.important_rules_are_different(&rules,
                                                   &context.shared.guards);

                let rules_changed = match visited_handling {
                    VisitedHandlingMode::AllLinksVisitedAndUnvisited => {
                        unreachable!("We should never try to selector match with \
                                     AllLinksVisitedAndUnvisited");
                    },
                    VisitedHandlingMode::AllLinksUnvisited => {
                        data.set_primary_rules(rules)
                    },
                    VisitedHandlingMode::RelevantLinkVisited => {
                        data.styles_mut().primary.set_visited_rules(rules)
                    },
                };

                return MatchingResults::new(rules_changed, important_rules_changed)
            }
        }

        let mut applicable_declarations = ApplicableDeclarationList::new();

        let stylist = &context.shared.stylist;
        let style_attribute = self.style_attribute();

        let map = &mut context.thread_local.selector_flags;
        let mut set_selector_flags = |element: &Self, flags: ElementSelectorFlags| {
            self.apply_selector_flags(map, element, flags);
        };

        let rule_inclusion = if only_default_rules {
            RuleInclusion::DefaultOnly
        } else {
            RuleInclusion::All
        };

        let bloom_filter = context.thread_local.bloom_filter.filter();
        let mut matching_context =
            MatchingContext::new_for_visited(MatchingMode::Normal,
                                             Some(bloom_filter),
                                             visited_handling,
                                             context.shared.quirks_mode);

        {
            let smil_override = data.get_smil_override();
            let animation_rules = if self.may_have_animations() {
                data.get_animation_rules()
            } else {
                AnimationRules(None, None)
            };

            // Compute the primary rule node.
            stylist.push_applicable_declarations(self,
                                                 implemented_pseudo.as_ref(),
                                                 style_attribute,
                                                 smil_override,
                                                 animation_rules,
                                                 rule_inclusion,
                                                 &mut applicable_declarations,
                                                 &mut matching_context,
                                                 &mut set_selector_flags);
        }
        self.unset_dirty_style_attribute();

        let primary_rule_node = stylist.rule_tree().compute_rule_node(
            &mut applicable_declarations,
            &context.shared.guards
        );

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

        let rules_changed = match visited_handling {
            VisitedHandlingMode::AllLinksVisitedAndUnvisited => {
                unreachable!("We should never try to selector match with \
                             AllLinksVisitedAndUnvisited");
            },
            VisitedHandlingMode::AllLinksUnvisited => {
                data.set_primary_rules(primary_rule_node)
            },
            VisitedHandlingMode::RelevantLinkVisited => {
                data.styles_mut().primary.set_visited_rules(primary_rule_node)
            },
        };

        MatchingResults::new_from_context(rules_changed,
                                          important_rules_changed,
                                          matching_context)
    }

    /// Runs selector matching to (re)compute eager pseudo-element rule nodes
    /// for this element.
    fn match_pseudos(&self,
                     context: &mut StyleContext<Self>,
                     data: &mut ElementData,
                     visited_handling: VisitedHandlingMode)
    {
        debug!("Match pseudos for {:?}, visited: {:?}", self, visited_handling);

        if self.implemented_pseudo_element().is_some() {
            // Element pseudos can't have any other pseudo.
            return;
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

        let rule_inclusion = if context.shared.traversal_flags.for_default_styles() {
            RuleInclusion::DefaultOnly
        } else {
            RuleInclusion::All
        };

        let bloom_filter = context.thread_local.bloom_filter.filter();

        let mut matching_context =
            MatchingContext::new_for_visited(MatchingMode::ForStatelessPseudoElement,
                                             Some(bloom_filter),
                                             visited_handling,
                                             context.shared.quirks_mode);

        // Compute rule nodes for eagerly-cascaded pseudo-elements.
        let mut matches_different_pseudos = false;
        SelectorImpl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            // For pseudo-elements, we only try to match visited rules if there
            // are also unvisited rules.  (This matches Gecko's behavior.)
            if visited_handling == VisitedHandlingMode::RelevantLinkVisited &&
               !data.styles().pseudos.has(&pseudo) {
                return
            }

            if !self.may_generate_pseudo(&pseudo, data.styles().primary.values()) {
                return;
            }

            debug_assert!(applicable_declarations.is_empty());
            // NB: We handle animation rules for ::before and ::after when
            // traversing them.
            stylist.push_applicable_declarations(self,
                                                 Some(&pseudo),
                                                 None,
                                                 None,
                                                 AnimationRules(None, None),
                                                 rule_inclusion,
                                                 &mut applicable_declarations,
                                                 &mut matching_context,
                                                 &mut set_selector_flags);

            let pseudos = &mut data.styles_mut().pseudos;
            if !applicable_declarations.is_empty() {
                let rules = stylist.rule_tree().compute_rule_node(
                    &mut applicable_declarations,
                    &guards
                );
                matches_different_pseudos |= pseudos.add_rules(
                    &pseudo,
                    visited_handling,
                    rules
                );
            } else {
                matches_different_pseudos |= pseudos.remove_rules(
                    &pseudo,
                    visited_handling
                );
            }
        });

        if matches_different_pseudos && data.restyle.is_restyle() {
            // Any changes to the matched pseudo-elements trigger
            // reconstruction.
            data.restyle.damage |= RestyleDamage::reconstruct();
        }
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
        context: &StyleContext<Self>,
        data: &mut ElementData
    ) -> bool {
        let mut result = false;
        result |= self.replace_rules_internal(replacements, context, data,
                                              CascadeVisitedMode::Unvisited);
        if !context.shared.traversal_flags.for_animation_only() {
            result |= self.replace_rules_internal(replacements, context, data,
                                                  CascadeVisitedMode::Visited);
        }
        result
    }

    /// Updates the rule nodes without re-running selector matching, using just
    /// the rule tree, for a specific visited mode.
    ///
    /// Returns true if an !important rule was replaced.
    fn replace_rules_internal(
        &self,
        replacements: RestyleHint,
        context: &StyleContext<Self>,
        data: &mut ElementData,
        cascade_visited: CascadeVisitedMode
    ) -> bool {
        use properties::PropertyDeclarationBlock;
        use shared_lock::Locked;

        debug_assert!(replacements.intersects(RestyleHint::replacements()) &&
                      (replacements & !RestyleHint::replacements()).is_empty());

        let element_styles = &mut data.styles_mut();
        let primary_rules = match cascade_visited.get_rules_mut(&mut element_styles.primary) {
            Some(r) => r,
            None => return false,
        };

        let replace_rule_node = |level: CascadeLevel,
                                 pdb: Option<&Arc<Locked<PropertyDeclarationBlock>>>,
                                 path: &mut StrongRuleNode| -> bool {
            let new_node = context.shared.stylist.rule_tree()
                .update_rule_at_level(level, pdb, path, &context.shared.guards);
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
        }

        false
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
                       mut data: &mut ElementData,
                       cascade_visited: CascadeVisitedMode)
    {
        debug!("Cascade pseudos for {:?}, visited: {:?}", self,
               cascade_visited);
        // Note that we've already set up the map of matching pseudo-elements
        // in match_pseudos (and handled the damage implications of changing
        // which pseudos match), so now we can just iterate what we have. This
        // does mean collecting owned pseudos, so that the borrow checker will
        // let us pass the mutable |data| to the cascade function.
        let matched_pseudos = data.styles().pseudos.keys();
        for pseudo in matched_pseudos {
            self.cascade_eager_pseudo(context, data, &pseudo, cascade_visited);
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

        // This currently ignores visited styles, which seems acceptable,
        // as existing browsers don't appear to animate visited styles.
        self.cascade_with_rules(shared_context,
                                font_metrics_provider,
                                &without_animation_rules,
                                primary_style,
                                CascadeTarget::Normal,
                                CascadeVisitedMode::Unvisited,
                                None)
    }

}

impl<E: TElement> MatchMethods for E {}
