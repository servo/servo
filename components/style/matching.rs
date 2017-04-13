/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! High-level interface to CSS selector matching.

#![allow(unsafe_code)]
#![deny(missing_docs)]

use Atom;
use animation::{self, Animation, PropertyAnimation};
use atomic_refcell::AtomicRefMut;
use bit_vec::BitVec;
use cache::{LRUCache, LRUCacheMutIterator};
use cascade_info::CascadeInfo;
use context::{CurrentElementInfo, SequentialTask, SharedStyleContext, StyleContext};
use data::{ComputedStyle, ElementData, ElementStyles, RestyleData};
use dom::{AnimationRules, SendElement, TElement, TNode};
use font_metrics::FontMetricsProvider;
use properties::{CascadeFlags, ComputedValues, SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP, cascade};
use properties::longhands::display::computed_value as display;
use restyle_hints::{RESTYLE_STYLE_ATTRIBUTE, RESTYLE_CSS_ANIMATIONS, RestyleHint};
use rule_tree::{CascadeLevel, RuleTree, StrongRuleNode};
use selector_parser::{PseudoElement, RestyleDamage, SelectorImpl};
use selectors::bloom::BloomFilter;
use selectors::matching::{ElementSelectorFlags, StyleRelations};
use selectors::matching::AFFECTED_BY_PSEUDO_ELEMENTS;
use sink::ForgetfulSink;
use std::sync::Arc;
use stylist::ApplicableDeclarationBlock;

/// Determines the amount of relations where we're going to share style.
#[inline]
fn relations_are_shareable(relations: &StyleRelations) -> bool {
    use selectors::matching::*;
    !relations.intersects(AFFECTED_BY_ID_SELECTOR |
                          AFFECTED_BY_PSEUDO_ELEMENTS | AFFECTED_BY_STATE |
                          AFFECTED_BY_STYLE_ATTRIBUTE |
                          AFFECTED_BY_PRESENTATIONAL_HINTS)
}

/// Information regarding a style sharing candidate.
///
/// Note that this information is stored in TLS and cleared after the traversal,
/// and once here, the style information of the element is immutable, so it's
/// safe to access.
///
/// TODO: We can stick a lot more info here.
#[derive(Debug)]
struct StyleSharingCandidate<E: TElement> {
    /// The element. We use SendElement here so that the cache may live in
    /// ScopedTLS.
    element: SendElement<E>,
    /// The cached class names.
    class_attributes: Option<Vec<Atom>>,
    /// The cached result of matching this entry against the revalidation selectors.
    revalidation_match_results: Option<BitVec>,
}

impl<E: TElement> PartialEq<StyleSharingCandidate<E>> for StyleSharingCandidate<E> {
    fn eq(&self, other: &Self) -> bool {
        self.element == other.element
    }
}

/// An LRU cache of the last few nodes seen, so that we can aggressively try to
/// reuse their styles.
///
/// Note that this cache is flushed every time we steal work from the queue, so
/// storing nodes here temporarily is safe.
pub struct StyleSharingCandidateCache<E: TElement> {
    cache: LRUCache<StyleSharingCandidate<E>>,
}

/// A cache miss result.
#[derive(Clone, Debug)]
pub enum CacheMiss {
    /// The parents don't match.
    Parent,
    /// The local name of the element and the candidate don't match.
    LocalName,
    /// The namespace of the element and the candidate don't match.
    Namespace,
    /// One of the element or the candidate was a link, but the other one
    /// wasn't.
    Link,
    /// The element and the candidate match different kind of rules. This can
    /// only happen in Gecko.
    UserAndAuthorRules,
    /// The element and the candidate are in a different state.
    State,
    /// The element had an id attribute, which qualifies for a unique style.
    IdAttr,
    /// The element had a style attribute, which qualifies for a unique style.
    StyleAttr,
    /// The element and the candidate class names didn't match.
    Class,
    /// The presentation hints didn't match.
    PresHints,
    /// The element and the candidate didn't match the same set of revalidation
    /// selectors.
    Revalidation,
}

fn element_matches_candidate<E: TElement>(element: &E,
                                          candidate: &mut StyleSharingCandidate<E>,
                                          candidate_element: &E,
                                          shared: &SharedStyleContext,
                                          bloom: &BloomFilter,
                                          info: &mut CurrentElementInfo,
                                          tasks: &mut Vec<SequentialTask<E>>)
                                          -> Result<ComputedStyle, CacheMiss> {
    macro_rules! miss {
        ($miss: ident) => {
            return Err(CacheMiss::$miss);
        }
    }

    if element.parent_element() != candidate_element.parent_element() {
        miss!(Parent)
    }

    if *element.get_local_name() != *candidate_element.get_local_name() {
        miss!(LocalName)
    }

    if *element.get_namespace() != *candidate_element.get_namespace() {
        miss!(Namespace)
    }

    if element.is_link() != candidate_element.is_link() {
        miss!(Link)
    }

    if element.matches_user_and_author_rules() != candidate_element.matches_user_and_author_rules() {
        miss!(UserAndAuthorRules)
    }

    if element.get_state() != candidate_element.get_state() {
        miss!(State)
    }

    if element.get_id().is_some() {
        miss!(IdAttr)
    }

    if element.style_attribute().is_some() {
        miss!(StyleAttr)
    }

    if !have_same_class(element, candidate, candidate_element) {
        miss!(Class)
    }

    if !have_same_presentational_hints(element, candidate_element) {
        miss!(PresHints)
    }

    if !revalidate(element, candidate, candidate_element,
                   shared, bloom, info, tasks) {
        miss!(Revalidation)
    }

    let data = candidate_element.borrow_data().unwrap();
    debug_assert!(data.has_current_styles());
    let current_styles = data.styles();

    Ok(current_styles.primary.clone())
}

fn have_same_presentational_hints<E: TElement>(element: &E, candidate: &E) -> bool {
    let mut first = ForgetfulSink::new();
    element.synthesize_presentational_hints_for_legacy_attributes(&mut first);

    if cfg!(debug_assertions) {
        let mut second = vec![];
        candidate.synthesize_presentational_hints_for_legacy_attributes(&mut second);
        debug_assert!(second.is_empty(),
                      "Should never have inserted an element with preshints in the cache!");
    }

    first.is_empty()
}

fn have_same_class<E: TElement>(element: &E,
                                candidate: &mut StyleSharingCandidate<E>,
                                candidate_element: &E) -> bool {
    // XXX Efficiency here, I'm only validating ideas.
    let mut element_class_attributes = vec![];
    element.each_class(|c| element_class_attributes.push(c.clone()));

    if candidate.class_attributes.is_none() {
        let mut attrs = vec![];
        candidate_element.each_class(|c| attrs.push(c.clone()));
        candidate.class_attributes = Some(attrs)
    }

    element_class_attributes == *candidate.class_attributes.as_ref().unwrap()
}

#[inline]
fn revalidate<E: TElement>(element: &E,
                           candidate: &mut StyleSharingCandidate<E>,
                           candidate_element: &E,
                           shared: &SharedStyleContext,
                           bloom: &BloomFilter,
                           info: &mut CurrentElementInfo,
                           tasks: &mut Vec<SequentialTask<E>>)
                           -> bool {
    // NB: We could avoid matching ancestor selectors entirely (rather than
    // just depending on the bloom filter), at the expense of some complexity.
    // Gecko bug 1354965 tracks this.
    //
    // We could also be even more careful about only matching the minimal number
    // of revalidation selectors until we find a mismatch. Gecko bug 1355668
    // tracks this.
    //
    // These potential optimizations may not be worth the complexity.
    let stylist = &shared.stylist;

    if info.revalidation_match_results.is_none() {
        // It's important to set the selector flags. Otherwise, if we succeed in
        // sharing the style, we may not set the slow selector flags for the
        // right elements (which may not necessarily be |element|), causing missed
        // restyles after future DOM mutations.
        //
        // Gecko's test_bug534804.html exercises this. A minimal testcase is:
        // <style> #e:empty + span { ... } </style>
        // <span id="e">
        //   <span></span>
        // </span>
        // <span></span>
        //
        // The style sharing cache will get a hit for the second span. When the
        // child span is subsequently removed from the DOM, missing selector
        // flags would cause us to miss the restyle on the second span.
        let mut set_selector_flags = |el: &E, flags: ElementSelectorFlags| {
            element.apply_selector_flags(tasks, el, flags);
        };
        info.revalidation_match_results =
            Some(stylist.match_revalidation_selectors(element, bloom,
                                                      &mut set_selector_flags));
    }

    if candidate.revalidation_match_results.is_none() {
        candidate.revalidation_match_results =
            Some(stylist.match_revalidation_selectors(candidate_element, bloom,
                                                      &mut |_, _| {}));
    }

    let for_element = info.revalidation_match_results.as_ref().unwrap();
    let for_candidate = candidate.revalidation_match_results.as_ref().unwrap();
    debug_assert!(for_element.len() == for_candidate.len());
    for_element == for_candidate
}

static STYLE_SHARING_CANDIDATE_CACHE_SIZE: usize = 8;

impl<E: TElement> StyleSharingCandidateCache<E> {
    /// Create a new style sharing candidate cache.
    pub fn new() -> Self {
        StyleSharingCandidateCache {
            cache: LRUCache::new(STYLE_SHARING_CANDIDATE_CACHE_SIZE),
        }
    }

    /// Returns the number of entries in the cache.
    pub fn num_entries(&self) -> usize {
        self.cache.num_entries()
    }

    fn iter_mut(&mut self) -> LRUCacheMutIterator<StyleSharingCandidate<E>> {
        self.cache.iter_mut()
    }

    /// Tries to insert an element in the style sharing cache.
    ///
    /// Fails if we know it should never be in the cache.
    pub fn insert_if_possible(&mut self,
                              element: &E,
                              style: &Arc<ComputedValues>,
                              relations: StyleRelations,
                              revalidation_match_results: Option<BitVec>) {
        let parent = match element.parent_element() {
            Some(element) => element,
            None => {
                debug!("Failing to insert to the cache: no parent element");
                return;
            }
        };

        // These are things we don't check in the candidate match because they
        // are either uncommon or expensive.
        if !relations_are_shareable(&relations) {
            debug!("Failing to insert to the cache: {:?}", relations);
            return;
        }

        let box_style = style.get_box();
        if box_style.specifies_transitions() {
            debug!("Failing to insert to the cache: transitions");
            return;
        }

        if box_style.specifies_animations() {
            debug!("Failing to insert to the cache: animations");
            return;
        }

        debug!("Inserting into cache: {:?} with parent {:?}",
               element, parent);

        self.cache.insert(StyleSharingCandidate {
            element: unsafe { SendElement::new(*element) },
            class_attributes: None,
            revalidation_match_results: revalidation_match_results,
        });
    }

    /// Touch a given index in the style sharing candidate cache.
    pub fn touch(&mut self, index: usize) {
        self.cache.touch(index);
    }

    /// Clear the style sharing candidate cache.
    pub fn clear(&mut self) {
        self.cache.evict_all()
    }
}

/// The results of attempting to share a style.
pub enum StyleSharingResult {
    /// We didn't find anybody to share the style with.
    CannotShare,
    /// The node's style can be shared. The integer specifies the index in the
    /// LRU cache that was hit and the damage that was done.
    StyleWasShared(usize),
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
                          cascade_flags: CascadeFlags,
                          is_pseudo: bool)
                          -> Arc<ComputedValues> {
        let mut cascade_info = CascadeInfo::new();

        // Grab the inherited values.
        let parent_el;
        let parent_data;
        let inherited_values_ = if !is_pseudo {
            parent_el = self.parent_element();
            parent_data = parent_el.as_ref().and_then(|e| e.borrow_data());
            let parent_values = parent_data.as_ref().map(|d| {
                // Sometimes Gecko eagerly styles things without processing
                // pending restyles first. In general we'd like to avoid this,
                // but there can be good reasons (for example, needing to
                // construct a frame for some small piece of newly-added
                // content in order to do something specific with that frame,
                // but not wanting to flush all of layout).
                debug_assert!(cfg!(feature = "gecko") || d.has_current_styles());
                d.styles().primary.values()
            });

            parent_values
        } else {
            parent_el = Some(self.clone());
            Some(primary_style.values())
        };

        let mut layout_parent_el = parent_el.clone();
        let layout_parent_data;
        let mut layout_parent_style = inherited_values_;
        if inherited_values_.map_or(false, |s| s.is_display_contents()) {
            layout_parent_el = Some(layout_parent_el.unwrap().layout_parent());
            layout_parent_data = layout_parent_el.as_ref().unwrap().borrow_data().unwrap();
            layout_parent_style = Some(layout_parent_data.styles().primary.values())
        }

        let inherited_values = inherited_values_.map(|x| &**x);
        let layout_parent_style = layout_parent_style.map(|x| &**x);

        // Propagate the "can be fragmented" bit. It would be nice to
        // encapsulate this better.
        //
        // Note that this is not needed for pseudos since we already do that
        // when we resolve the non-pseudo style.
        if !is_pseudo {
            if let Some(ref p) = layout_parent_style {
                let can_be_fragmented =
                    p.is_multicol() ||
                    layout_parent_el.as_ref().unwrap().as_node().can_be_fragmented();
                unsafe { self.as_node().set_can_be_fragmented(can_be_fragmented); }
            }
        }

        // Invoke the cascade algorithm.
        let values =
            Arc::new(cascade(&shared_context.stylist.device,
                             rule_node,
                             &shared_context.guards,
                             inherited_values,
                             layout_parent_style,
                             Some(&mut cascade_info),
                             &*shared_context.error_reporter,
                             font_metrics_provider,
                             cascade_flags));

        cascade_info.finish(&self.as_node());
        values
    }

    fn cascade_internal(&self,
                        context: &StyleContext<Self>,
                        primary_style: &ComputedStyle,
                        pseudo_style: &Option<(&PseudoElement, &mut ComputedStyle)>)
                        -> Arc<ComputedValues> {
        let mut cascade_flags = CascadeFlags::empty();
        if self.skip_root_and_item_based_display_fixup() {
            cascade_flags.insert(SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP)
        }

        // Grab the rule node.
        let rule_node = &pseudo_style.as_ref().map_or(primary_style, |p| &*p.1).rules;
        self.cascade_with_rules(context.shared, &context.thread_local.font_metrics_provider,
                                rule_node, primary_style, cascade_flags, pseudo_style.is_some())
    }

    /// Computes values and damage for the primary or pseudo style of an element,
    /// setting them on the ElementData.
    fn cascade_primary_or_pseudo<'a>(&self,
                                     context: &mut StyleContext<Self>,
                                     data: &mut ElementData,
                                     pseudo: Option<&PseudoElement>,
                                     animate: bool) {
        // Collect some values.
        let (mut styles, restyle) = data.styles_and_restyle_mut();
        let mut primary_style = &mut styles.primary;
        let pseudos = &mut styles.pseudos;
        let mut pseudo_style = pseudo.map(|p| (p, pseudos.get_mut(p).unwrap()));
        let mut old_values =
            pseudo_style.as_mut().map_or_else(|| primary_style.values.take(), |p| p.1.values.take());

        // Compute the new values.
        let mut new_values = self.cascade_internal(context, primary_style,
                                                   &pseudo_style);

        // Handle animations.
        if animate && !context.shared.traversal_flags.for_animation_only() {
            self.process_animations(context,
                                    &mut old_values,
                                    &mut new_values,
                                    pseudo);
        }

        // Accumulate restyle damage.
        if let Some(old) = old_values {
            self.accumulate_damage(&context.shared, restyle.unwrap(), &old, &new_values, pseudo);
        }

        // Set the new computed values.
        if let Some((_, ref mut style)) = pseudo_style {
            style.values = Some(new_values);
        } else {
            primary_style.values = Some(new_values);
        }
    }

    #[cfg(feature = "gecko")]
    fn get_after_change_style(&self,
                              context: &mut StyleContext<Self>,
                              primary_style: &ComputedStyle,
                              pseudo_style: &Option<(&PseudoElement, &ComputedStyle)>)
                              -> Arc<ComputedValues> {
        let style = &pseudo_style.as_ref().map_or(primary_style, |p| &*p.1);
        let rule_node = &style.rules;
        let without_transition_rules =
            context.shared.stylist.rule_tree.remove_transition_rule_if_applicable(rule_node);
        if without_transition_rules == *rule_node {
            // Note that unwrapping here is fine, because the style is
            // only incomplete during the styling process.
            return style.values.as_ref().unwrap().clone();
        }

        let mut cascade_flags = CascadeFlags::empty();
        if self.skip_root_and_item_based_display_fixup() {
            cascade_flags.insert(SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP)
        }
        self.cascade_with_rules(context.shared,
                                &context.thread_local.font_metrics_provider,
                                &without_transition_rules,
                                primary_style,
                                cascade_flags,
                                pseudo_style.is_some())
    }

    #[cfg(feature = "gecko")]
    fn process_animations(&self,
                          context: &mut StyleContext<Self>,
                          old_values: &mut Option<Arc<ComputedValues>>,
                          new_values: &mut Arc<ComputedValues>,
                          pseudo: Option<&PseudoElement>) {
        use context::{CSS_ANIMATIONS, EFFECT_PROPERTIES};
        use context::UpdateAnimationsTasks;

        let ref new_box_style = new_values.get_box();
        let has_new_animation_style = new_box_style.animation_name_count() >= 1 &&
                                      new_box_style.animation_name_at(0).0.len() != 0;
        let has_animations = self.has_css_animations(pseudo);

        let mut tasks = UpdateAnimationsTasks::empty();
        let needs_update_animations =
            old_values.as_ref().map_or(has_new_animation_style, |ref old| {
                let ref old_box_style = old.get_box();
                let old_display_style = old_box_style.clone_display();
                let new_display_style = new_box_style.clone_display();
                // FIXME: Bug 1344581: We still need to compare keyframe rules.
                !old_box_style.animations_equals(&new_box_style) ||
                 (old_display_style == display::T::none &&
                  new_display_style != display::T::none &&
                  has_new_animation_style) ||
                 (old_display_style != display::T::none &&
                  new_display_style == display::T::none &&
                  has_animations)
            });
        if needs_update_animations {
            tasks.insert(CSS_ANIMATIONS);
        }
        if self.has_animations(pseudo) {
            tasks.insert(EFFECT_PROPERTIES);
        }
        if !tasks.is_empty() {
            let task = SequentialTask::update_animations(self.as_node().as_element().unwrap(),
                                                         pseudo.cloned(),
                                                         tasks);
            context.thread_local.tasks.push(task);
        }
    }

    #[cfg(feature = "servo")]
    fn process_animations(&self,
                          context: &mut StyleContext<Self>,
                          old_values: &mut Option<Arc<ComputedValues>>,
                          new_values: &mut Arc<ComputedValues>,
                          _pseudo: Option<&PseudoElement>) {
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
                self.as_node().to_unsafe(),
                &**values,
                new_values,
                &shared_context.timer,
                &possibly_expired_animations);
        }
    }

    /// Computes and applies non-redundant damage.
    #[cfg(feature = "gecko")]
    fn accumulate_damage(&self,
                         shared_context: &SharedStyleContext,
                         restyle: &mut RestyleData,
                         old_values: &Arc<ComputedValues>,
                         new_values: &Arc<ComputedValues>,
                         pseudo: Option<&PseudoElement>) {
        // Don't accumulate damage if we're in a restyle for reconstruction.
        if shared_context.traversal_flags.for_reconstruct() {
            return;
        }

        // If an ancestor is already getting reconstructed by Gecko's top-down
        // frame constructor, no need to apply damage.
        if restyle.damage_handled.contains(RestyleDamage::reconstruct()) {
            restyle.damage = RestyleDamage::empty();
            return;
        }

        // Add restyle damage, but only the bits that aren't redundant with respect
        // to damage applied on our ancestors.
        //
        // See https://bugzilla.mozilla.org/show_bug.cgi?id=1301258#c12
        // for followup work to make the optimization here more optimal by considering
        // each bit individually.
        if !restyle.damage.contains(RestyleDamage::reconstruct()) {
            let new_damage = self.compute_restyle_damage(&old_values, &new_values, pseudo);
            if !restyle.damage_handled.contains(new_damage) {
                restyle.damage |= new_damage;
            }
        }
    }

    /// Computes and applies restyle damage unless we've already maxed it out.
    #[cfg(feature = "servo")]
    fn accumulate_damage(&self,
                         _shared_context: &SharedStyleContext,
                         restyle: &mut RestyleData,
                         old_values: &Arc<ComputedValues>,
                         new_values: &Arc<ComputedValues>,
                         pseudo: Option<&PseudoElement>) {
        if restyle.damage != RestyleDamage::rebuild_and_reflow() {
            let d = self.compute_restyle_damage(&old_values, &new_values, pseudo);
            restyle.damage |= d;
        }
    }

    fn update_animations_for_cascade(&self,
                                     context: &SharedStyleContext,
                                     style: &mut Arc<ComputedValues>,
                                     possibly_expired_animations: &mut Vec<PropertyAnimation>,
                                     font_metrics: &FontMetricsProvider) {
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
                    if let Animation::Transition(_, _, _, ref frame, _) = *running_animation {
                        possibly_expired_animations.push(frame.property_animation.clone())
                    }
                }
            }
        }
    }

    fn share_style_with_candidate_if_possible(&self,
                                              candidate: &mut StyleSharingCandidate<Self>,
                                              shared: &SharedStyleContext,
                                              bloom: &BloomFilter,
                                              info: &mut CurrentElementInfo,
                                              tasks: &mut Vec<SequentialTask<Self>>)
                                              -> Result<ComputedStyle, CacheMiss> {
        let candidate_element = *candidate.element;
        element_matches_candidate(self, candidate, &candidate_element,
                                  shared, bloom, info, tasks)
    }
}

fn compute_rule_node<E: TElement>(rule_tree: &RuleTree,
                                  applicable_declarations: &mut Vec<ApplicableDeclarationBlock>)
                                  -> StrongRuleNode
{
    let rules = applicable_declarations.drain(..).map(|d| (d.source, d.level));
    let rule_node = rule_tree.insert_ordered_rules(rules);
    rule_node
}

impl<E: TElement> PrivateMatchMethods for E {}

/// Controls whether the style sharing cache is used.
#[derive(Clone, Copy, PartialEq)]
pub enum StyleSharingBehavior {
    /// Style sharing allowed.
    Allow,
    /// Style sharing disallowed.
    Disallow,
}

/// The public API that elements expose for selector matching.
pub trait MatchMethods : TElement {
    /// Performs selector matching and property cascading on an element and its eager pseudos.
    fn match_and_cascade(&self,
                         context: &mut StyleContext<Self>,
                         data: &mut ElementData,
                         sharing: StyleSharingBehavior)
    {
        // Perform selector matching for the primary style.
        let mut primary_relations = StyleRelations::empty();
        let _rule_node_changed = self.match_primary(context, data, &mut primary_relations);

        // Cascade properties and compute primary values.
        self.cascade_primary(context, data);

        // Match and cascade eager pseudo-elements.
        if !data.styles().is_display_none() {
            let _pseudo_rule_nodes_changed =
                self.match_pseudos(context, data);
            self.cascade_pseudos(context, data);
        }

        // If we have any pseudo elements, indicate so in the primary StyleRelations.
        if !data.styles().pseudos.is_empty() {
            primary_relations |= AFFECTED_BY_PSEUDO_ELEMENTS;
        }

        // If the style is shareable, add it to the LRU cache.
        if sharing == StyleSharingBehavior::Allow && relations_are_shareable(&primary_relations) {
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
                                       primary_relations,
                                       revalidation_match_results);
        }
    }

    /// Performs the cascade, without matching.
    fn cascade_primary_and_pseudos(&self,
                                   context: &mut StyleContext<Self>,
                                   mut data: &mut ElementData)
    {
        self.cascade_primary(context, &mut data);
        self.cascade_pseudos(context, &mut data);
    }

    /// Runs selector matching to (re)compute the primary rule node for this element.
    ///
    /// Returns whether the primary rule node changed.
    fn match_primary(&self,
                     context: &mut StyleContext<Self>,
                     data: &mut ElementData,
                     relations: &mut StyleRelations)
                     -> bool
    {
        let mut applicable_declarations =
            Vec::<ApplicableDeclarationBlock>::with_capacity(16);

        let stylist = &context.shared.stylist;
        let style_attribute = self.style_attribute();
        let animation_rules = self.get_animation_rules(None);
        let mut rule_nodes_changed = false;
        let bloom = context.thread_local.bloom_filter.filter();

        let tasks = &mut context.thread_local.tasks;
        let mut set_selector_flags = |element: &Self, flags: ElementSelectorFlags| {
            self.apply_selector_flags(tasks, element, flags);
        };

        // Compute the primary rule node.
        *relations = stylist.push_applicable_declarations(self,
                                                          Some(bloom),
                                                          style_attribute,
                                                          animation_rules,
                                                          None,
                                                          &context.shared.guards,
                                                          &mut applicable_declarations,
                                                          &mut set_selector_flags);

        let primary_rule_node =
            compute_rule_node::<Self>(&stylist.rule_tree, &mut applicable_declarations);
        if !data.has_styles() {
            data.set_styles(ElementStyles::new(ComputedStyle::new_partial(primary_rule_node)));
            rule_nodes_changed = true;
        } else if data.styles().primary.rules != primary_rule_node {
            data.styles_mut().primary.rules = primary_rule_node;
            rule_nodes_changed = true;
        }

        rule_nodes_changed
    }

    /// Runs selector matching to (re)compute eager pseudo-element rule nodes for this
    /// element.
    ///
    /// Returns whether any of the pseudo rule nodes changed (including, but not
    /// limited to, cases where we match different pseudos altogether).
    fn match_pseudos(&self,
                     context: &mut StyleContext<Self>,
                     data: &mut ElementData)
                     -> bool
    {
        let mut applicable_declarations =
            Vec::<ApplicableDeclarationBlock>::with_capacity(16);
        let mut rule_nodes_changed = false;

        let tasks = &mut context.thread_local.tasks;
        let mut set_selector_flags = |element: &Self, flags: ElementSelectorFlags| {
            self.apply_selector_flags(tasks, element, flags);
        };

        // Borrow the stuff we need here so the borrow checker doesn't get mad
        // at us later in the closure.
        let stylist = &context.shared.stylist;
        let guards = &context.shared.guards;
        let rule_tree = &stylist.rule_tree;
        let bloom_filter = context.thread_local.bloom_filter.filter();

        // Compute rule nodes for eagerly-cascaded pseudo-elements.
        let mut matches_different_pseudos = false;
        SelectorImpl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            let mut pseudos = &mut data.styles_mut().pseudos;
            debug_assert!(applicable_declarations.is_empty());
            let pseudo_animation_rules = if pseudo.is_before_or_after() {
                self.get_animation_rules(Some(&pseudo))
            } else {
                AnimationRules(None, None)
            };
            stylist.push_applicable_declarations(self,
                                                 Some(bloom_filter),
                                                 None, pseudo_animation_rules,
                                                 Some(&pseudo),
                                                 &guards,
                                                 &mut applicable_declarations,
                                                 &mut set_selector_flags);

            if !applicable_declarations.is_empty() {
                let new_rules =
                    compute_rule_node::<Self>(rule_tree, &mut applicable_declarations);
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
                            tasks: &mut Vec<SequentialTask<Self>>,
                            element: &Self,
                            flags: ElementSelectorFlags) {
        // Apply the selector flags.
        let self_flags = flags.for_self();
        if !self_flags.is_empty() {
            if element == self {
                // If this is the element we're styling, we have exclussive
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
                    let task =
                        SequentialTask::set_selector_flags(element.clone(),
                                                           self_flags);
                    tasks.push(task);
                }
            }
        }
        let parent_flags = flags.for_parent();
        if !parent_flags.is_empty() {
            if let Some(p) = element.parent_element() {
                // Avoid the overhead of the SequentialTask if the flags are
                // already set.
                if !p.has_selector_flags(parent_flags) {
                    let task = SequentialTask::set_selector_flags(p, parent_flags);
                    tasks.push(task);
                }
            }
        }
    }

    /// Updates the rule nodes without re-running selector matching, using just
    /// the rule tree. Returns true if the rule nodes changed.
    fn replace_rules(&self,
                     hint: RestyleHint,
                     context: &StyleContext<Self>,
                     data: &mut AtomicRefMut<ElementData>)
                     -> bool {
        use properties::PropertyDeclarationBlock;
        use shared_lock::Locked;

        let element_styles = &mut data.styles_mut();
        let primary_rules = &mut element_styles.primary.rules;
        let mut rule_node_changed = false;

        {
            let mut replace_rule_node = |level: CascadeLevel,
                                         pdb: Option<&Arc<Locked<PropertyDeclarationBlock>>>,
                                         path: &mut StrongRuleNode| {
                let new_node = context.shared.stylist.rule_tree
                    .update_rule_at_level(level, pdb, path, &context.shared.guards);
                if let Some(n) = new_node {
                    *path = n;
                    rule_node_changed = true;
                }
            };

            // RESTYLE_CSS_ANIMATIONS is processed prior to other restyle hints
            // in the name of animation-only traversal. Rest of restyle hints
            // will be processed in a subsequent normal traversal.
            if hint.contains(RESTYLE_CSS_ANIMATIONS) {
                debug_assert!(context.shared.traversal_flags.for_animation_only());

                let animation_rule = self.get_animation_rule(None);
                replace_rule_node(CascadeLevel::Animations,
                                  animation_rule.as_ref(),
                                  primary_rules);

                let pseudos = &mut element_styles.pseudos;
                for pseudo in pseudos.keys().iter().filter(|p| p.is_before_or_after()) {
                    let animation_rule = self.get_animation_rule(Some(&pseudo));
                    let pseudo_rules = &mut pseudos.get_mut(&pseudo).unwrap().rules;
                    replace_rule_node(CascadeLevel::Animations,
                                      animation_rule.as_ref(),
                                      pseudo_rules);
                }
            } else if hint.contains(RESTYLE_STYLE_ATTRIBUTE) {
                let style_attribute = self.style_attribute();
                replace_rule_node(CascadeLevel::StyleAttributeNormal,
                                  style_attribute,
                                  primary_rules);
                replace_rule_node(CascadeLevel::StyleAttributeImportant,
                                  style_attribute,
                                  primary_rules);
                // The per-pseudo rule nodes never change in this path.
            }
        }

        // The per-pseudo rule nodes never change in this path.
        rule_node_changed
    }

    /// Attempts to share a style with another node. This method is unsafe
    /// because it depends on the `style_sharing_candidate_cache` having only
    /// live nodes in it, and we have no way to guarantee that at the type
    /// system level yet.
    unsafe fn share_style_if_possible(&self,
                                      context: &mut StyleContext<Self>,
                                      data: &mut AtomicRefMut<ElementData>)
                                      -> StyleSharingResult {
        if context.shared.options.disable_style_sharing_cache {
            debug!("{:?} Cannot share style: style sharing cache disabled", self);
            return StyleSharingResult::CannotShare
        }

        if self.parent_element().is_none() {
            debug!("{:?} Cannot share style: element has style attribute", self);
            return StyleSharingResult::CannotShare
        }

        if self.style_attribute().is_some() {
            debug!("{:?} Cannot share style: element has style attribute", self);
            return StyleSharingResult::CannotShare
        }

        if self.has_attr(&ns!(), &local_name!("id")) {
            debug!("{:?} Cannot share style: element has id", self);
            return StyleSharingResult::CannotShare
        }

        let cache = &mut context.thread_local.style_sharing_candidate_cache;
        let current_element_info =
            &mut context.thread_local.current_element_info.as_mut().unwrap();
        let bloom = context.thread_local.bloom_filter.filter();
        let tasks = &mut context.thread_local.tasks;
        let mut should_clear_cache = false;
        for (i, candidate) in cache.iter_mut().enumerate() {
            let sharing_result =
                self.share_style_with_candidate_if_possible(candidate,
                                                            &context.shared,
                                                            bloom,
                                                            current_element_info,
                                                            tasks);
            match sharing_result {
                Ok(shared_style) => {
                    // Yay, cache hit. Share the style.

                    // Accumulate restyle damage.
                    debug_assert_eq!(data.has_styles(), data.has_restyle());
                    let old_values = data.get_styles_mut()
                                         .and_then(|s| s.primary.values.take());
                    if let Some(old) = old_values {
                        self.accumulate_damage(&context.shared,
                                               data.restyle_mut(), &old,
                                               shared_style.values(), None);
                    }

                    // We never put elements with pseudo style into the style
                    // sharing cache, so we can just mint an ElementStyles
                    // directly here.
                    //
                    // See https://bugzilla.mozilla.org/show_bug.cgi?id=1329361
                    let styles = ElementStyles::new(shared_style);
                    data.set_styles(styles);

                    return StyleSharingResult::StyleWasShared(i)
                }
                Err(miss) => {
                    debug!("Cache miss: {:?}", miss);

                    // Cache miss, let's see what kind of failure to decide
                    // whether we keep trying or not.
                    match miss {
                        // Cache miss because of parent, clear the candidate cache.
                        CacheMiss::Parent => {
                            should_clear_cache = true;
                            break;
                        },
                        // Too expensive failure, give up, we don't want another
                        // one of these.
                        CacheMiss::PresHints |
                        CacheMiss::Revalidation => break,
                        _ => {}
                    }
                }
            }
        }

        debug!("{:?} Cannot share style: {} cache entries", self, cache.num_entries());

        if should_clear_cache {
            cache.clear();
        }

        StyleSharingResult::CannotShare
    }

    // The below two functions are copy+paste because I can't figure out how to
    // write a function which takes a generic function. I don't think it can
    // be done.
    //
    // Ideally, I'd want something like:
    //
    //   > fn with_really_simple_selectors(&self, f: <H: Hash>|&H|);


    // In terms of `SimpleSelector`s, these two functions will insert and remove:
    //   - `SimpleSelector::LocalName`
    //   - `SimpleSelector::Namepace`
    //   - `SimpleSelector::ID`
    //   - `SimpleSelector::Class`

    /// Inserts and removes the matching `Descendant` selectors from a bloom
    /// filter. This is used to speed up CSS selector matching to remove
    /// unnecessary tree climbs for `Descendant` queries.
    ///
    /// A bloom filter of the local names, namespaces, IDs, and classes is kept.
    /// Therefore, each node must have its matching selectors inserted _after_
    /// its own selector matching and _before_ its children start.
    fn insert_into_bloom_filter(&self, bf: &mut BloomFilter) {
        bf.insert_hash(self.get_local_name().get_hash());
        bf.insert_hash(self.get_namespace().get_hash());
        if let Some(id) = self.get_id() {
            bf.insert_hash(id.get_hash());
        }
        // TODO: case-sensitivity depends on the document type and quirks mode
        self.each_class(|class| {
            bf.insert_hash(class.get_hash())
        });
    }

    /// After all the children are done css selector matching, this must be
    /// called to reset the bloom filter after an `insert`.
    fn remove_from_bloom_filter(&self, bf: &mut BloomFilter) {
        bf.remove_hash(self.get_local_name().get_hash());
        bf.remove_hash(self.get_namespace().get_hash());
        if let Some(id) = self.get_id() {
            bf.remove_hash(id.get_hash());
        }

        // TODO: case-sensitivity depends on the document type and quirks mode
        self.each_class(|class| {
            bf.remove_hash(class.get_hash())
        });
    }

    /// Given the old and new style of this element, and whether it's a
    /// pseudo-element, compute the restyle damage used to determine which
    /// kind of layout or painting operations we'll need.
    fn compute_restyle_damage(&self,
                              old_values: &Arc<ComputedValues>,
                              new_values: &Arc<ComputedValues>,
                              pseudo: Option<&PseudoElement>)
                              -> RestyleDamage
    {
        match self.existing_style_for_restyle_damage(old_values, pseudo) {
            Some(ref source) => RestyleDamage::compute(source, new_values),
            None => {
                // If there's no style source, that likely means that Gecko
                // couldn't find a style context. This happens with display:none
                // elements, and probably a number of other edge cases that
                // we don't handle well yet (like display:contents).
                if new_values.get_box().clone_display() == display::T::none &&
                    old_values.get_box().clone_display() == display::T::none {
                    // The style remains display:none. No need for damage.
                    RestyleDamage::empty()
                } else {
                    // Something else. Be conservative for now.
                    RestyleDamage::reconstruct()
                }
            }
        }
    }

    /// Performs the cascade for the element's primary style.
    fn cascade_primary(&self,
                       context: &mut StyleContext<Self>,
                       mut data: &mut ElementData)
    {
        self.cascade_primary_or_pseudo(context, &mut data, None, /* animate = */ true);
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
            // Only ::before and ::after are animatable.
            let animate = pseudo.is_before_or_after();
            self.cascade_primary_or_pseudo(context, data, Some(&pseudo), animate);
        }
    }

    /// Returns computed values without animation and transition rules.
    fn get_base_style(&self,
                      shared_context: &SharedStyleContext,
                      font_metrics_provider: &FontMetricsProvider,
                      primary_style: &ComputedStyle,
                      pseudo_style: &Option<(&PseudoElement, &ComputedStyle)>)
                      -> Arc<ComputedValues> {
        let style = &pseudo_style.as_ref().map_or(primary_style, |p| &*p.1);
        let rule_node = &style.rules;
        let without_animation_rules =
            shared_context.stylist.rule_tree.remove_animation_and_transition_rules(rule_node);
        if without_animation_rules == *rule_node {
            // Note that unwrapping here is fine, because the style is
            // only incomplete during the styling process.
            return style.values.as_ref().unwrap().clone();
        }

        let mut cascade_flags = CascadeFlags::empty();
        if self.skip_root_and_item_based_display_fixup() {
            cascade_flags.insert(SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP)
        }
        self.cascade_with_rules(shared_context,
                                font_metrics_provider,
                                &without_animation_rules,
                                primary_style,
                                cascade_flags,
                                pseudo_style.is_some())
    }

}

impl<E: TElement> MatchMethods for E {}
