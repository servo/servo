/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! High-level interface to CSS selector matching.

#![allow(unsafe_code)]
#![deny(missing_docs)]

use {Atom, LocalName};
use animation::{self, Animation, PropertyAnimation};
use atomic_refcell::AtomicRefMut;
use cache::LRUCache;
use cascade_info::CascadeInfo;
use context::{SharedStyleContext, StyleContext};
use data::{ComputedStyle, ElementData, ElementStyles, PseudoStyles};
use dom::{SendElement, TElement, TNode};
use properties::{CascadeFlags, ComputedValues, SHAREABLE, SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP, cascade};
use properties::longhands::display::computed_value as display;
use rule_tree::StrongRuleNode;
use selector_parser::{PseudoElement, RestyleDamage, SelectorImpl};
use selectors::MatchAttr;
use selectors::bloom::BloomFilter;
use selectors::matching::{AFFECTED_BY_PSEUDO_ELEMENTS, MatchingReason, StyleRelations};
use servo_config::opts;
use sink::ForgetfulSink;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::slice::IterMut;
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

fn create_common_style_affecting_attributes_from_element<E: TElement>(element: &E)
                                                         -> CommonStyleAffectingAttributes {
    let mut flags = CommonStyleAffectingAttributes::empty();
    for attribute_info in &common_style_affecting_attributes() {
        match attribute_info.mode {
            CommonStyleAffectingAttributeMode::IsPresent(flag) => {
                if element.has_attr(&ns!(), &attribute_info.attr_name) {
                    flags.insert(flag)
                }
            }
            CommonStyleAffectingAttributeMode::IsEqual(ref target_value, flag) => {
                if element.attr_equals(&ns!(), &attribute_info.attr_name, target_value) {
                    flags.insert(flag)
                }
            }
        }
    }
    flags
}

/// The rule nodes for each of the pseudo-elements of an element.
///
/// TODO(emilio): Probably shouldn't be a `HashMap` by default, but a smaller
/// array.
type PseudoRuleNodes = HashMap<PseudoElement, StrongRuleNode,
                               BuildHasherDefault<::fnv::FnvHasher>>;

/// The results of selector matching on an element.
pub struct MatchResults {
    /// The rule node reference that represents the rules matched by the
    /// element.
    pub primary: StrongRuleNode,
    /// A set of style relations (different hints about what rules matched or
    /// could have matched).
    pub relations: StyleRelations,
    /// The results of selector-matching the pseudo-elements.
    pub per_pseudo: PseudoRuleNodes,
}

impl MatchResults {
    /// Returns true if the primary rule node is shareable with other nodes.
    pub fn primary_is_shareable(&self) -> bool {
        relations_are_shareable(&self.relations)
    }
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
    /// The cached common style affecting attribute info.
    common_style_affecting_attributes: Option<CommonStyleAffectingAttributes>,
    /// The cached class names.
    class_attributes: Option<Vec<Atom>>,
}

impl<E: TElement> PartialEq<StyleSharingCandidate<E>> for StyleSharingCandidate<E> {
    fn eq(&self, other: &Self) -> bool {
        self.element == other.element &&
            self.common_style_affecting_attributes == other.common_style_affecting_attributes
    }
}

/// An LRU cache of the last few nodes seen, so that we can aggressively try to
/// reuse their styles.
///
/// Note that this cache is flushed every time we steal work from the queue, so
/// storing nodes here temporarily is safe.
pub struct StyleSharingCandidateCache<E: TElement> {
    cache: LRUCache<StyleSharingCandidate<E>, ()>,
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
    /// The element and the candidate common style affecting attributes didn't
    /// match.
    CommonStyleAffectingAttributes,
    /// The presentation hints didn't match.
    PresHints,
    /// The element and the candidate didn't match the same set of
    /// sibling-affecting rules.
    SiblingRules,
    /// The element and the candidate didn't match the same set of non-common
    /// style affecting attribute selectors.
    NonCommonAttrRules,
}

fn element_matches_candidate<E: TElement>(element: &E,
                                          candidate: &mut StyleSharingCandidate<E>,
                                          candidate_element: &E,
                                          shared_context: &SharedStyleContext)
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

    if !have_same_common_style_affecting_attributes(element,
                                                    candidate,
                                                    candidate_element) {
        miss!(CommonStyleAffectingAttributes)
    }

    if !have_same_presentational_hints(element, candidate_element) {
        miss!(PresHints)
    }

    if !match_same_sibling_affecting_rules(element,
                                           candidate_element,
                                           shared_context) {
        miss!(SiblingRules)
    }

    if !match_same_not_common_style_affecting_attributes_rules(element,
                                                               candidate_element,
                                                               shared_context) {
        miss!(NonCommonAttrRules)
    }

    let data = candidate_element.borrow_data().unwrap();
    debug_assert!(data.has_current_styles());
    let current_styles = data.styles();

    Ok(current_styles.primary.clone())
}

fn have_same_common_style_affecting_attributes<E: TElement>(element: &E,
                                                            candidate: &mut StyleSharingCandidate<E>,
                                                            candidate_element: &E) -> bool {
    if candidate.common_style_affecting_attributes.is_none() {
        candidate.common_style_affecting_attributes =
            Some(create_common_style_affecting_attributes_from_element(candidate_element))
    }
    create_common_style_affecting_attributes_from_element(element) ==
        candidate.common_style_affecting_attributes.unwrap()
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

bitflags! {
    /// A set of common style-affecting attributes we check separately to
    /// optimize the style sharing cache.
    pub flags CommonStyleAffectingAttributes: u8 {
        /// The `hidden` attribute.
        const HIDDEN_ATTRIBUTE = 0x01,
        /// The `nowrap` attribute.
        const NO_WRAP_ATTRIBUTE = 0x02,
        /// The `align="left"` attribute.
        const ALIGN_LEFT_ATTRIBUTE = 0x04,
        /// The `align="center"` attribute.
        const ALIGN_CENTER_ATTRIBUTE = 0x08,
        /// The `align="right"` attribute.
        const ALIGN_RIGHT_ATTRIBUTE = 0x10,
    }
}

/// The information of how to match a given common-style affecting attribute.
pub struct CommonStyleAffectingAttributeInfo {
    /// The attribute name.
    pub attr_name: LocalName,
    /// The matching mode for the attribute.
    pub mode: CommonStyleAffectingAttributeMode,
}

/// How should we match a given common style-affecting attribute?
#[derive(Clone)]
pub enum CommonStyleAffectingAttributeMode {
    /// Just for presence?
    IsPresent(CommonStyleAffectingAttributes),
    /// For presence and equality with a given value.
    IsEqual(Atom, CommonStyleAffectingAttributes),
}

/// The common style affecting attribute array.
///
/// TODO: This should be a `const static` or similar, but couldn't be because
/// `Atom`s have destructors.
#[inline]
pub fn common_style_affecting_attributes() -> [CommonStyleAffectingAttributeInfo; 5] {
    [
        CommonStyleAffectingAttributeInfo {
            attr_name: local_name!("hidden"),
            mode: CommonStyleAffectingAttributeMode::IsPresent(HIDDEN_ATTRIBUTE),
        },
        CommonStyleAffectingAttributeInfo {
            attr_name: local_name!("nowrap"),
            mode: CommonStyleAffectingAttributeMode::IsPresent(NO_WRAP_ATTRIBUTE),
        },
        CommonStyleAffectingAttributeInfo {
            attr_name: local_name!("align"),
            mode: CommonStyleAffectingAttributeMode::IsEqual(atom!("left"), ALIGN_LEFT_ATTRIBUTE),
        },
        CommonStyleAffectingAttributeInfo {
            attr_name: local_name!("align"),
            mode: CommonStyleAffectingAttributeMode::IsEqual(atom!("center"), ALIGN_CENTER_ATTRIBUTE),
        },
        CommonStyleAffectingAttributeInfo {
            attr_name: local_name!("align"),
            mode: CommonStyleAffectingAttributeMode::IsEqual(atom!("right"), ALIGN_RIGHT_ATTRIBUTE),
        }
    ]
}

/// Attributes that, if present, disable style sharing. All legacy HTML
/// attributes must be in either this list or
/// `common_style_affecting_attributes`. See the comment in
/// `synthesize_presentational_hints_for_legacy_attributes`.
///
/// TODO(emilio): This is not accurate now, we don't disable style sharing for
/// this now since we check for attribute selectors in the stylesheet. Consider
/// removing this.
pub fn rare_style_affecting_attributes() -> [LocalName; 4] {
    [local_name!("bgcolor"), local_name!("border"), local_name!("colspan"), local_name!("rowspan")]
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

// TODO: These re-match the candidate every time, which is suboptimal.
#[inline]
fn match_same_not_common_style_affecting_attributes_rules<E: TElement>(element: &E,
                                                                       candidate: &E,
                                                                       ctx: &SharedStyleContext) -> bool {
    ctx.stylist.match_same_not_common_style_affecting_attributes_rules(element, candidate)
}

#[inline]
fn match_same_sibling_affecting_rules<E: TElement>(element: &E,
                                                   candidate: &E,
                                                   ctx: &SharedStyleContext) -> bool {
    ctx.stylist.match_same_sibling_affecting_rules(element, candidate)
}

static STYLE_SHARING_CANDIDATE_CACHE_SIZE: usize = 8;

impl<E: TElement> StyleSharingCandidateCache<E> {
    /// Create a new style sharing candidate cache.
    pub fn new() -> Self {
        StyleSharingCandidateCache {
            cache: LRUCache::new(STYLE_SHARING_CANDIDATE_CACHE_SIZE),
        }
    }

    fn iter_mut(&mut self) -> IterMut<(StyleSharingCandidate<E>, ())> {
        self.cache.iter_mut()
    }

    /// Tries to insert an element in the style sharing cache.
    ///
    /// Fails if we know it should never be in the cache.
    pub fn insert_if_possible(&mut self,
                              element: &E,
                              style: &Arc<ComputedValues>,
                              relations: StyleRelations) {
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
        if box_style.transition_property_count() > 0 {
            debug!("Failing to insert to the cache: transitions");
            return;
        }

        if box_style.animation_name_count() > 0 {
            debug!("Failing to insert to the cache: animations");
            return;
        }

        debug!("Inserting into cache: {:?} with parent {:?}",
               element, parent);

        self.cache.insert(StyleSharingCandidate {
            element: unsafe { SendElement::new(*element) },
            common_style_affecting_attributes: None,
            class_attributes: None,
        }, ());
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

/// Callers need to pass several boolean flags to cascade_node_pseudo_element.
/// We encapsulate them in this struct to avoid mixing them up.
///
/// FIXME(pcwalton): Unify with `CascadeFlags`, perhaps?
struct CascadeBooleans {
    shareable: bool,
    animate: bool,
}

trait PrivateMatchMethods: TElement {
    /// Actually cascades style for a node or a pseudo-element of a node.
    ///
    /// Note that animations only apply to nodes or ::before or ::after
    /// pseudo-elements.
    fn cascade_node_pseudo_element<'a>(&self,
                                       context: &StyleContext<Self>,
                                       parent_style: Option<&Arc<ComputedValues>>,
                                       old_style: Option<&Arc<ComputedValues>>,
                                       rule_node: &StrongRuleNode,
                                       possibly_expired_animations: &[PropertyAnimation],
                                       booleans: CascadeBooleans)
                                       -> Arc<ComputedValues> {
        let shared_context = context.shared;
        let mut cascade_info = CascadeInfo::new();
        let mut cascade_flags = CascadeFlags::empty();
        if booleans.shareable {
            cascade_flags.insert(SHAREABLE)
        }
        if self.skip_root_and_item_based_display_fixup() {
            cascade_flags.insert(SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP)
        }

        let this_style = match parent_style {
            Some(ref parent_style) => {
                cascade(shared_context.viewport_size,
                        rule_node,
                        Some(&***parent_style),
                        &shared_context.default_computed_values,
                        Some(&mut cascade_info),
                        shared_context.error_reporter.clone(),
                        cascade_flags)
            }
            None => {
                cascade(shared_context.viewport_size,
                        rule_node,
                        None,
                        &shared_context.default_computed_values,
                        Some(&mut cascade_info),
                        shared_context.error_reporter.clone(),
                        cascade_flags)
            }
        };
        cascade_info.finish(&self.as_node());

        let mut this_style = Arc::new(this_style);

        if booleans.animate {
            let new_animations_sender = &context.thread_local.new_animations_sender;
            let this_opaque = self.as_node().opaque();
            // Trigger any present animations if necessary.
            animation::maybe_start_animations(&shared_context,
                                              new_animations_sender,
                                              this_opaque, &this_style);

            // Trigger transitions if necessary. This will reset `this_style` back
            // to its old value if it did trigger a transition.
            if let Some(ref style) = old_style {
                animation::start_transitions_if_applicable(
                    new_animations_sender,
                    this_opaque,
                    self.as_node().to_unsafe(),
                    &**style,
                    &mut this_style,
                    &shared_context.timer,
                    &possibly_expired_animations);
            }
        }

        this_style
    }

    fn update_animations_for_cascade(&self,
                                     context: &SharedStyleContext,
                                     style: &mut Arc<ComputedValues>,
                                     possibly_expired_animations: &mut Vec<PropertyAnimation>) {
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
                                                          style);
                    if let Animation::Transition(_, _, _, ref frame, _) = *running_animation {
                        possibly_expired_animations.push(frame.property_animation.clone())
                    }
                }
            }
        }
    }

    fn share_style_with_candidate_if_possible(&self,
                                              shared_context: &SharedStyleContext,
                                              candidate: &mut StyleSharingCandidate<Self>)
                                              -> Result<ComputedStyle, CacheMiss> {
        let candidate_element = *candidate.element;
        element_matches_candidate(self, candidate, &candidate_element, shared_context)
    }
}

fn compute_rule_node<E: TElement>(context: &StyleContext<E>,
                                  applicable_declarations: &mut Vec<ApplicableDeclarationBlock>)
                                  -> StrongRuleNode
{
    let rules = applicable_declarations.drain(..).map(|d| (d.source, d.importance));
    let rule_node = context.shared.stylist.rule_tree.insert_ordered_rules(rules);
    rule_node
}

impl<E: TElement> PrivateMatchMethods for E {}

/// The public API that elements expose for selector matching.
pub trait MatchMethods : TElement {
    /// Runs selector matching of this element, and returns the result.
    fn match_element(&self, context: &StyleContext<Self>, parent_bf: Option<&BloomFilter>)
                     -> MatchResults
    {
        let mut applicable_declarations: Vec<ApplicableDeclarationBlock> = Vec::with_capacity(16);
        let stylist = &context.shared.stylist;
        let style_attribute = self.style_attribute();

        // Compute the primary rule node.
        let mut primary_relations =
            stylist.push_applicable_declarations(self,
                                                 parent_bf,
                                                 style_attribute,
                                                 None,
                                                 &mut applicable_declarations,
                                                 MatchingReason::ForStyling);
        let primary_rule_node = compute_rule_node(context, &mut applicable_declarations);

        // Compute the pseudo rule nodes.
        let mut per_pseudo: PseudoRuleNodes = HashMap::with_hasher(Default::default());
        SelectorImpl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            debug_assert!(applicable_declarations.is_empty());
            stylist.push_applicable_declarations(self, parent_bf, None,
                                                 Some(&pseudo.clone()),
                                                 &mut applicable_declarations,
                                                 MatchingReason::ForStyling);

            if !applicable_declarations.is_empty() {
                let rule_node = compute_rule_node(context, &mut applicable_declarations);
                per_pseudo.insert(pseudo, rule_node);
            }
        });

        // If we have any pseudo elements, indicate so in the primary StyleRelations.
        if !per_pseudo.is_empty() {
            primary_relations |= AFFECTED_BY_PSEUDO_ELEMENTS;
        }

        MatchResults {
            primary: primary_rule_node,
            relations: primary_relations,
            per_pseudo: per_pseudo,
        }
    }

    /// Attempts to share a style with another node. This method is unsafe
    /// because it depends on the `style_sharing_candidate_cache` having only
    /// live nodes in it, and we have no way to guarantee that at the type
    /// system level yet.
    unsafe fn share_style_if_possible(&self,
                                      style_sharing_candidate_cache:
                                        &mut StyleSharingCandidateCache<Self>,
                                      shared_context: &SharedStyleContext,
                                      data: &mut AtomicRefMut<ElementData>)
                                      -> StyleSharingResult {
        if opts::get().disable_share_style_cache {
            return StyleSharingResult::CannotShare
        }

        if self.style_attribute().is_some() {
            return StyleSharingResult::CannotShare
        }

        if self.has_attr(&ns!(), &local_name!("id")) {
            return StyleSharingResult::CannotShare
        }

        let mut should_clear_cache = false;
        for (i, &mut (ref mut candidate, ())) in style_sharing_candidate_cache.iter_mut().enumerate() {
            let sharing_result = self.share_style_with_candidate_if_possible(shared_context, candidate);
            match sharing_result {
                Ok(shared_style) => {
                    // Yay, cache hit. Share the style.

                    // TODO: add the display: none optimisation here too! Even
                    // better, factor it out/make it a bit more generic so Gecko
                    // can decide more easily if it knows that it's a child of
                    // replaced content, or similar stuff!
                    let damage = {
                        debug_assert!(!data.has_current_styles());
                        let previous_values = data.get_styles().map(|x| &x.primary.values);
                        match self.existing_style_for_restyle_damage(previous_values, None) {
                            Some(ref source) => RestyleDamage::compute(source, &shared_style.values),
                            None => RestyleDamage::rebuild_and_reflow(),
                        }
                    };

                    data.finish_styling(ElementStyles::new(shared_style), damage);
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
                        CacheMiss::CommonStyleAffectingAttributes |
                        CacheMiss::PresHints |
                        CacheMiss::SiblingRules |
                        CacheMiss::NonCommonAttrRules => break,
                        _ => {}
                    }
                }
            }
        }
        if should_clear_cache {
            style_sharing_candidate_cache.clear();
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
        bf.insert(&*self.get_local_name());
        bf.insert(&*self.get_namespace());
        self.get_id().map(|id| bf.insert(&id));

        // TODO: case-sensitivity depends on the document type and quirks mode
        self.each_class(|class| bf.insert(class));
    }

    /// After all the children are done css selector matching, this must be
    /// called to reset the bloom filter after an `insert`.
    fn remove_from_bloom_filter(&self, bf: &mut BloomFilter) {
        bf.remove(&*self.get_local_name());
        bf.remove(&*self.get_namespace());
        self.get_id().map(|id| bf.remove(&id));

        // TODO: case-sensitivity depends on the document type and quirks mode
        self.each_class(|class| bf.remove(class));
    }

    /// Given the old and new style of this element, and whether it's a
    /// pseudo-element, compute the restyle damage used to determine which
    /// kind of layout or painting operations we'll need.
    fn compute_restyle_damage(&self,
                              old_style: Option<&Arc<ComputedValues>>,
                              new_style: &Arc<ComputedValues>,
                              pseudo: Option<&PseudoElement>)
                              -> RestyleDamage
    {
        match self.existing_style_for_restyle_damage(old_style, pseudo) {
            Some(ref source) => RestyleDamage::compute(source, new_style),
            None => {
                // If there's no style source, two things can happen:
                //
                //  1. This is not an incremental restyle (old_style is none).
                //     In this case we can't do too much than sending
                //     rebuild_and_reflow.
                //
                //  2. This is an incremental restyle, but the old display value
                //     is none, so there's no effective way for Gecko to get the
                //     style source. In this case, we could return either
                //     RestyleDamage::empty(), in the case both displays are
                //     none, or rebuild_and_reflow, otherwise. The first case
                //     should be already handled when calling this function, so
                //     we can assert that the new display value is not none.
                //
                //     Also, this can be a text node (in which case we don't
                //     care of watching the new display value).
                //
                // Unfortunately we can't strongly assert part of this, since
                // we style some nodes that in Gecko never generate a frame,
                // like children of replaced content. Arguably, we shouldn't be
                // styling those here, but until we implement that we'll have to
                // stick without the assertions.
                debug_assert!(pseudo.is_none() ||
                              new_style.get_box().clone_display() != display::T::none);
                RestyleDamage::rebuild_and_reflow()
            }
        }
    }

    /// Given the results of selector matching, run the CSS cascade and style
    /// the node, potentially starting any new transitions or animations.
    fn cascade_node(&self,
                    context: &StyleContext<Self>,
                    mut data: &mut AtomicRefMut<ElementData>,
                    parent: Option<Self>,
                    primary_rule_node: StrongRuleNode,
                    pseudo_rule_nodes: PseudoRuleNodes,
                    primary_is_shareable: bool)
    {
        // Get our parent's style.
        let parent_data = parent.as_ref().map(|x| x.borrow_data().unwrap());
        let parent_style = parent_data.as_ref().map(|d| {
            // Sometimes Gecko eagerly styles things without processing pending
            // restyles first. In general we'd like to avoid this, but there can
            // be good reasons (for example, needing to construct a frame for
            // some small piece of newly-added content in order to do something
            // specific with that frame, but not wanting to flush all of
            // layout).
            debug_assert!(cfg!(feature = "gecko") || d.has_current_styles());
            &d.styles().primary.values
        });

        let mut new_styles;
        let mut possibly_expired_animations = vec![];

        let damage = {
            debug_assert!(!data.has_current_styles());
            let (old_primary, old_pseudos) = match data.get_styles_mut() {
                None => (None, None),
                Some(previous) => {
                    // Update animations before the cascade. This may modify the
                    // value of the old primary style.
                    self.update_animations_for_cascade(&context.shared,
                                                       &mut previous.primary.values,
                                                       &mut possibly_expired_animations);
                    (Some(&previous.primary.values), Some(&mut previous.pseudos))
                }
            };

            let new_style =
                self.cascade_node_pseudo_element(context,
                                                 parent_style,
                                                 old_primary,
                                                 &primary_rule_node,
                                                 &possibly_expired_animations,
                                                 CascadeBooleans {
                                                     shareable: primary_is_shareable,
                                                     animate: true,
                                                 });

            let primary = ComputedStyle::new(primary_rule_node, new_style);
            new_styles = ElementStyles::new(primary);

            let damage =
                self.compute_damage_and_cascade_pseudos(old_primary,
                                                        old_pseudos,
                                                        &new_styles.primary.values,
                                                        &mut new_styles.pseudos,
                                                        context,
                                                        pseudo_rule_nodes,
                                                        &mut possibly_expired_animations);

            unsafe {
                self.as_node().set_can_be_fragmented(parent.map_or(false, |p| {
                    p.as_node().can_be_fragmented() ||
                    parent_style.unwrap().is_multicol()
                }));
            }

            damage
        };

        data.finish_styling(new_styles, damage);
    }

    /// Given the old and new styling results, compute the final restyle damage.
    fn compute_damage_and_cascade_pseudos(
            &self,
            old_primary: Option<&Arc<ComputedValues>>,
            mut old_pseudos: Option<&mut PseudoStyles>,
            new_primary: &Arc<ComputedValues>,
            new_pseudos: &mut PseudoStyles,
            context: &StyleContext<Self>,
            mut pseudo_rule_nodes: PseudoRuleNodes,
            possibly_expired_animations: &mut Vec<PropertyAnimation>)
            -> RestyleDamage
    {
        // Here we optimise the case of the style changing but both the
        // previous and the new styles having display: none. In this
        // case, we can always optimize the traversal, regardless of the
        // restyle hint.
        let this_display = new_primary.get_box().clone_display();
        if this_display == display::T::none {
            let old_display = old_primary.map(|old| {
                old.get_box().clone_display()
            });

            // If display passed from none to something, then we need to reflow,
            // otherwise, we don't do anything.
            let damage = match old_display {
                Some(display) if display == this_display => {
                    RestyleDamage::empty()
                }
                _ => RestyleDamage::rebuild_and_reflow()
            };

            debug!("Short-circuiting traversal: {:?} {:?} {:?}",
                   this_display, old_display, damage);

            return damage
        }

        // Compute the damage and sum up the damage related to pseudo-elements.
        let mut damage =
            self.compute_restyle_damage(old_primary, new_primary, None);

        // If the new style is display:none, we don't need pseudo-elements styles.
        if new_primary.get_box().clone_display() == display::T::none {
            return damage;
        }

        let rebuild_and_reflow = RestyleDamage::rebuild_and_reflow();

        debug_assert!(new_pseudos.is_empty());
        <Self as MatchAttr>::Impl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            let maybe_rule_node = pseudo_rule_nodes.remove(&pseudo);

            // Grab the old pseudo style for analysis.
            let mut maybe_old_pseudo_style =
                old_pseudos.as_mut().and_then(|x| x.remove(&pseudo));

            if maybe_rule_node.is_some() {
                let new_rule_node = maybe_rule_node.unwrap();

                // We have declarations, so we need to cascade. Compute parameters.
                let animate = <Self as MatchAttr>::Impl::pseudo_is_before_or_after(&pseudo);
                if animate {
                    if let Some(ref mut old_pseudo_style) = maybe_old_pseudo_style {
                        // Update animations before the cascade. This may modify
                        // the value of old_pseudo_style.
                        self.update_animations_for_cascade(&context.shared,
                                                           &mut old_pseudo_style.values,
                                                           possibly_expired_animations);
                    }
                }

                let new_pseudo_values =
                    self.cascade_node_pseudo_element(context,
                                                     Some(new_primary),
                                                     maybe_old_pseudo_style.as_ref()
                                                                           .map(|s| &s.values),
                                                     &new_rule_node,
                                                     &possibly_expired_animations,
                                                     CascadeBooleans {
                                                         shareable: false,
                                                         animate: animate,
                                                     });

                // Compute restyle damage unless we've already maxed it out.
                if damage != rebuild_and_reflow {
                    damage = damage | match maybe_old_pseudo_style {
                        None => rebuild_and_reflow,
                        Some(ref old) => self.compute_restyle_damage(Some(&old.values),
                                                                     &new_pseudo_values,
                                                                     Some(&pseudo)),
                    };
                }

                // Insert the new entry into the map.
                let new_pseudo_style = ComputedStyle::new(new_rule_node, new_pseudo_values);
                let existing = new_pseudos.insert(pseudo, new_pseudo_style);
                debug_assert!(existing.is_none());
            } else {
                if maybe_old_pseudo_style.is_some() {
                    damage = rebuild_and_reflow;
                }
            }
        });

        damage
    }
}

impl<E: TElement> MatchMethods for E {}
