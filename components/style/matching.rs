/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! High-level interface to CSS selector matching.

#![allow(unsafe_code)]

use animation::{self, Animation};
use arc_ptr_eq;
use cache::{LRUCache, SimpleHashCache};
use context::{StyleContext, SharedStyleContext};
use data::PrivateStyleData;
use dom::{TElement, TNode, TRestyleDamage, UnsafeNode};
use properties::{ComputedValues, PropertyDeclaration, cascade};
use selector_impl::{ElementExt, SelectorImplExt, TheSelectorImpl, PseudoElement};
use selector_matching::{DeclarationBlock, Stylist};
use selectors::Element;
use selectors::bloom::BloomFilter;
use selectors::matching::{CommonStyleAffectingAttributeMode, CommonStyleAffectingAttributes};
use selectors::matching::{StyleRelations, AFFECTED_BY_PSEUDO_ELEMENTS};
use selectors::matching::{common_style_affecting_attributes, rare_style_affecting_attributes};
use sink::ForgetfulSink;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::fmt;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::slice::Iter;
use std::sync::Arc;
use string_cache::{Atom, Namespace};
use util::opts;

fn create_common_style_affecting_attributes_from_element<E: TElement>(element: &E)
                                                         -> CommonStyleAffectingAttributes {
    let mut flags = CommonStyleAffectingAttributes::empty();
    for attribute_info in &common_style_affecting_attributes() {
        match attribute_info.mode {
            CommonStyleAffectingAttributeMode::IsPresent(flag) => {
                if element.has_attr(&ns!(), &attribute_info.atom) {
                    flags.insert(flag)
                }
            }
            CommonStyleAffectingAttributeMode::IsEqual(ref target_value, flag) => {
                if element.attr_equals(&ns!(), &attribute_info.atom, target_value) {
                    flags.insert(flag)
                }
            }
        }
    }
    flags
}

pub struct ApplicableDeclarations {
    pub normal: SmallVec<[DeclarationBlock; 16]>,
    pub per_pseudo: HashMap<PseudoElement,
                            Vec<DeclarationBlock>,
                            BuildHasherDefault<::fnv::FnvHasher>>,

    /// Whether the `normal` declarations are shareable with other nodes.
    pub normal_shareable: bool,
}

impl ApplicableDeclarations {
    pub fn new() -> Self {
        let mut applicable_declarations = ApplicableDeclarations {
            normal: SmallVec::new(),
            per_pseudo: HashMap::with_hasher(Default::default()),
            normal_shareable: false,
        };

        TheSelectorImpl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            applicable_declarations.per_pseudo.insert(pseudo, vec![]);
        });

        applicable_declarations
    }
}

#[derive(Clone)]
pub struct ApplicableDeclarationsCacheEntry {
    pub declarations: Vec<DeclarationBlock>,
}

impl ApplicableDeclarationsCacheEntry {
    fn new(declarations: Vec<DeclarationBlock>) -> ApplicableDeclarationsCacheEntry {
        ApplicableDeclarationsCacheEntry {
            declarations: declarations,
        }
    }
}

impl PartialEq for ApplicableDeclarationsCacheEntry {
    fn eq(&self, other: &ApplicableDeclarationsCacheEntry) -> bool {
        let this_as_query = ApplicableDeclarationsCacheQuery::new(&*self.declarations);
        this_as_query.eq(other)
    }
}
impl Eq for ApplicableDeclarationsCacheEntry {}

impl Hash for ApplicableDeclarationsCacheEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let tmp = ApplicableDeclarationsCacheQuery::new(&*self.declarations);
        tmp.hash(state);
    }
}

struct ApplicableDeclarationsCacheQuery<'a> {
    declarations: &'a [DeclarationBlock],
}

impl<'a> ApplicableDeclarationsCacheQuery<'a> {
    fn new(declarations: &'a [DeclarationBlock]) -> ApplicableDeclarationsCacheQuery<'a> {
        ApplicableDeclarationsCacheQuery {
            declarations: declarations,
        }
    }
}

impl<'a> PartialEq for ApplicableDeclarationsCacheQuery<'a> {
    fn eq(&self, other: &ApplicableDeclarationsCacheQuery<'a>) -> bool {
        if self.declarations.len() != other.declarations.len() {
            return false
        }
        for (this, other) in self.declarations.iter().zip(other.declarations) {
            if !arc_ptr_eq(&this.declarations, &other.declarations) {
                return false
            }
        }
        true
    }
}
impl<'a> Eq for ApplicableDeclarationsCacheQuery<'a> {}

impl<'a> PartialEq<ApplicableDeclarationsCacheEntry> for ApplicableDeclarationsCacheQuery<'a> {
    fn eq(&self, other: &ApplicableDeclarationsCacheEntry) -> bool {
        let other_as_query = ApplicableDeclarationsCacheQuery::new(&other.declarations);
        self.eq(&other_as_query)
    }
}

impl<'a> Hash for ApplicableDeclarationsCacheQuery<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for declaration in self.declarations {
            // Each declaration contians an Arc, which is a stable
            // pointer; we use that for hashing and equality.
            let ptr = &*declaration.declarations as *const Vec<PropertyDeclaration>;
            ptr.hash(state);
        }
    }
}

static APPLICABLE_DECLARATIONS_CACHE_SIZE: usize = 32;

pub struct ApplicableDeclarationsCache {
    cache: SimpleHashCache<ApplicableDeclarationsCacheEntry, Arc<ComputedValues>>,
}

impl ApplicableDeclarationsCache {
    pub fn new() -> Self {
        ApplicableDeclarationsCache {
            cache: SimpleHashCache::new(APPLICABLE_DECLARATIONS_CACHE_SIZE),
        }
    }

    pub fn find(&self, declarations: &[DeclarationBlock]) -> Option<Arc<ComputedValues>> {
        match self.cache.find(&ApplicableDeclarationsCacheQuery::new(declarations)) {
            None => None,
            Some(ref values) => Some((*values).clone()),
        }
    }

    pub fn insert(&mut self, declarations: Vec<DeclarationBlock>, style: Arc<ComputedValues>) {
        self.cache.insert(ApplicableDeclarationsCacheEntry::new(declarations), style)
    }

    pub fn evict_all(&mut self) {
        self.cache.evict_all();
    }
}

/// An LRU cache of the last few nodes seen, so that we can aggressively try to
/// reuse their styles.
///
/// Note that this cache is flushed every time we steal work from the queue, so
/// storing nodes here temporarily is safe.
///
/// NB: We store UnsafeNode's, but this is not unsafe. It's a shame being
/// generic over elements is unfeasible (you can make compile style without much
/// difficulty, but good luck with layout and all the types with assoc.
/// lifetimes).
pub struct StyleSharingCandidateCache {
    cache: LRUCache<UnsafeNode, ()>,
}

#[derive(Clone, Debug)]
pub enum CacheMiss {
    Parent,
    LocalName,
    Namespace,
    Link,
    State,
    IdAttr,
    StyleAttr,
    Class,
    CommonStyleAffectingAttributes,
    PresHints,
    SiblingRules,
    NonCommonAttrRules,
}

fn element_matches_candidate<E: TElement>(element: &E,
                                          candidate: &E,
                                          shared_context: &SharedStyleContext)
                                          -> Result<Arc<ComputedValues>, CacheMiss> {
    macro_rules! miss {
        ($miss: ident) => {
            return Err(CacheMiss::$miss);
        }
    }

    if element.parent_element() != candidate.parent_element() {
        miss!(Parent)
    }

    if *element.get_local_name() != *candidate.get_local_name() {
        miss!(LocalName)
    }

    if *element.get_namespace() != *candidate.get_namespace() {
        miss!(Namespace)
    }

    if element.is_link() != candidate.is_link() {
        miss!(Link)
    }

    if element.get_state() != candidate.get_state() {
        miss!(State)
    }

    if element.get_id().is_some() {
        miss!(IdAttr)
    }

    if element.style_attribute().is_some() {
        miss!(StyleAttr)
    }

    if !have_same_class(element, candidate) {
        miss!(Class)
    }

    if !have_same_common_style_affecting_attributes(element, candidate) {
        miss!(CommonStyleAffectingAttributes)
    }

    if !have_same_presentational_hints(element, candidate) {
        miss!(PresHints)
    }

    if !match_same_sibling_affecting_rules(element, candidate, shared_context) {
        miss!(SiblingRules)
    }

    if !match_same_not_common_style_affecting_attributes_rules(element, candidate, shared_context) {
        miss!(NonCommonAttrRules)
    }

    let candidate_node = candidate.as_node();
    let candidate_style = candidate_node.borrow_data().unwrap().style.as_ref().unwrap().clone();

    Ok(candidate_style)
}

fn have_same_common_style_affecting_attributes<E: TElement>(element: &E,
                                                            candidate: &E) -> bool {
    // XXX probably could do something smarter. Also, the cache should
    // precompute this for the parent. Just experimenting now though.
    create_common_style_affecting_attributes_from_element(element) ==
        create_common_style_affecting_attributes_from_element(candidate)
}

fn have_same_presentational_hints<E: TElement>(element: &E, candidate: &E) -> bool {
    let mut first = vec![];
    element.synthesize_presentational_hints_for_legacy_attributes(&mut first);
    if cfg!(debug_assertions) {
        let mut second = vec![];
        candidate.synthesize_presentational_hints_for_legacy_attributes(&mut second);
        debug_assert!(second.is_empty(),
                      "Should never have inserted an element with preshints in the cache!");
    }

    first.is_empty()
}

fn have_same_class<E: TElement>(element: &E, candidate: &E) -> bool {
    // XXX Efficiency here, I'm only validating ideas.
    let mut first = vec![];
    let mut second = vec![];

    element.each_class(|c| first.push(c.clone()));
    candidate.each_class(|c| second.push(c.clone()));

    first == second
}

fn match_same_not_common_style_affecting_attributes_rules<E: TElement>(element: &E,
                                                                       candidate: &E,
                                                                       ctx: &SharedStyleContext) -> bool {
    // XXX Same here, could store in the cache an index with the matched rules,
    // for example.
    ctx.stylist.match_same_not_common_style_affecting_attributes_rules(element, candidate)
}

fn match_same_sibling_affecting_rules<E: TElement>(element: &E,
                                                   candidate: &E,
                                                   ctx: &SharedStyleContext) -> bool {
    ctx.stylist.match_same_sibling_affecting_rules(element, candidate)
}

static STYLE_SHARING_CANDIDATE_CACHE_SIZE: usize = 8;

impl StyleSharingCandidateCache {
    pub fn new() -> Self {
        StyleSharingCandidateCache {
            cache: LRUCache::new(STYLE_SHARING_CANDIDATE_CACHE_SIZE),
        }
    }

    pub fn iter(&self) -> Iter<(UnsafeNode, ())> {
        self.cache.iter()
    }

    pub fn insert_if_possible<E: TElement>(&mut self,
                                           element: &E,
                                           relations: StyleRelations) {
        use selectors::matching::*; // For flags
        use traversal::relations_are_shareable;

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

        // XXX check transitions/animations and reject!
        debug!("Inserting into cache: {:?} with parent {:?}",
               element.as_node().to_unsafe(), parent.as_node().to_unsafe());

        self.cache.insert(element.as_node().to_unsafe(), ())
    }

    pub fn touch(&mut self, index: usize) {
        self.cache.touch(index);
    }

    pub fn clear(&mut self) {
        self.cache.evict_all()
    }
}

/// The results of attempting to share a style.
pub enum StyleSharingResult<ConcreteRestyleDamage: TRestyleDamage> {
    /// We didn't find anybody to share the style with.
    CannotShare,
    /// The node's style can be shared. The integer specifies the index in the LRU cache that was
    /// hit and the damage that was done.
    StyleWasShared(usize, ConcreteRestyleDamage),
}

trait PrivateMatchMethods: TNode
    where <Self::ConcreteElement as Element>::Impl: SelectorImplExt {
    /// Actually cascades style for a node or a pseudo-element of a node.
    ///
    /// Note that animations only apply to nodes or ::before or ::after
    /// pseudo-elements.
    fn cascade_node_pseudo_element<'a, Ctx>(&self,
                                            context: &Ctx,
                                            parent_style: Option<&Arc<ComputedValues>>,
                                            applicable_declarations: &[DeclarationBlock],
                                            mut style: Option<&mut Arc<ComputedValues>>,
                                            applicable_declarations_cache:
                                             &mut ApplicableDeclarationsCache,
                                            shareable: bool,
                                            animate_properties: bool)
                                            -> (Self::ConcreteRestyleDamage, Arc<ComputedValues>)
    where Ctx: StyleContext<'a> {
        let mut cacheable = true;
        let shared_context = context.shared_context();
        if animate_properties {
            cacheable = !self.update_animations_for_cascade(shared_context,
                                                            &mut style) && cacheable;
        }

        let this_style;
        match parent_style {
            Some(ref parent_style) => {
                let cache_entry = applicable_declarations_cache.find(applicable_declarations);
                let cached_computed_values = match cache_entry {
                    Some(ref style) => Some(&**style),
                    None => None,
                };

                let (the_style, is_cacheable) = cascade(shared_context.viewport_size,
                                                        applicable_declarations,
                                                        shareable,
                                                        Some(&***parent_style),
                                                        cached_computed_values,
                                                        shared_context.error_reporter.clone());
                cacheable = cacheable && is_cacheable;
                this_style = the_style
            }
            None => {
                let (the_style, is_cacheable) = cascade(shared_context.viewport_size,
                                                        applicable_declarations,
                                                        shareable,
                                                        None,
                                                        None,
                                                        shared_context.error_reporter.clone());
                cacheable = cacheable && is_cacheable;
                this_style = the_style
            }
        };

        let mut this_style = Arc::new(this_style);

        if animate_properties {
            let new_animations_sender = &context.local_context().new_animations_sender;
            let this_opaque = self.opaque();
            // Trigger any present animations if necessary.
            let mut animations_started = animation::maybe_start_animations(
                &shared_context,
                new_animations_sender,
                this_opaque,
                &this_style);

            // Trigger transitions if necessary. This will reset `this_style` back
            // to its old value if it did trigger a transition.
            if let Some(ref style) = style {
                animations_started |=
                    animation::start_transitions_if_applicable(
                        new_animations_sender,
                        this_opaque,
                        &**style,
                        &mut this_style,
                        &shared_context.timer);
            }

            cacheable = cacheable && !animations_started
        }

        // Calculate style difference.
        let damage = Self::ConcreteRestyleDamage::compute(style.map(|s| &*s), &*this_style);

        // Cache the resolved style if it was cacheable.
        if cacheable {
            applicable_declarations_cache.insert(applicable_declarations.to_vec(),
                                                 this_style.clone());
        }

        // Return the final style and the damage done to our caller.
        (damage, this_style)
    }

    fn update_animations_for_cascade(&self,
                                     context: &SharedStyleContext,
                                     style: &mut Option<&mut Arc<ComputedValues>>)
                                     -> bool {
        let style = match *style {
            None => return false,
            Some(ref mut style) => style,
        };

        // Finish any expired transitions.
        let this_opaque = self.opaque();
        let had_animations_to_expire =
            animation::complete_expired_transitions(this_opaque, style, context);

        // Merge any running transitions into the current style, and cancel them.
        let had_running_animations = context.running_animations
                                            .read()
                                            .unwrap()
                                            .get(&this_opaque)
                                            .is_some();
        if had_running_animations {
            let mut all_running_animations = context.running_animations.write().unwrap();
            for mut running_animation in all_running_animations.get_mut(&this_opaque).unwrap() {
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
                    animation::update_style_for_animation::<Self::ConcreteRestyleDamage>(
                        context, running_animation, style, None);
                    running_animation.mark_as_expired();
                }
            }
        }

        had_animations_to_expire || had_running_animations
    }
}

impl<N: TNode> PrivateMatchMethods for N
    where <N::ConcreteElement as Element>::Impl: SelectorImplExt {}

trait PrivateElementMatchMethods: TElement {
    fn share_style_with_candidate_if_possible(&self,
                                              parent_node: Self::ConcreteNode,
                                              shared_context: &SharedStyleContext,
                                              candidate: &Self)
                                              -> Option<Arc<ComputedValues>> {
        debug_assert!(parent_node.is_element());

        match element_matches_candidate(self, candidate, shared_context) {
            Ok(cv) => Some(cv),
            Err(error) => {
                debug!("Cache miss: {:?}", error);
                None
            }
        }
    }
}

impl<E: TElement> PrivateElementMatchMethods for E {}

pub trait ElementMatchMethods : TElement {
    fn match_element(&self,
                     stylist: &Stylist,
                     parent_bf: Option<&BloomFilter>,
                     applicable_declarations: &mut ApplicableDeclarations)
                     -> StyleRelations {
        use traversal::relations_are_shareable;
        let style_attribute = self.style_attribute().as_ref();

        let mut relations =
            stylist.push_applicable_declarations(self,
                                                 parent_bf,
                                                 style_attribute,
                                                 None,
                                                 &mut applicable_declarations.normal);

        applicable_declarations.normal_shareable = relations_are_shareable(&relations);

        TheSelectorImpl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            stylist.push_applicable_declarations(self,
                                                 parent_bf,
                                                 None,
                                                 Some(&pseudo.clone()),
                                                 applicable_declarations.per_pseudo.entry(pseudo).or_insert(vec![]));
        });

        let has_pseudos =
            applicable_declarations.per_pseudo.values().any(|v| !v.is_empty());

        if has_pseudos {
            relations |= AFFECTED_BY_PSEUDO_ELEMENTS;
        }

        relations
    }

    /// Attempts to share a style with another node. This method is unsafe because it depends on
    /// the `style_sharing_candidate_cache` having only live nodes in it, and we have no way to
    /// guarantee that at the type system level yet.
    unsafe fn share_style_if_possible(&self,
                                      style_sharing_candidate_cache:
                                        &mut StyleSharingCandidateCache,
                                      shared_context: &SharedStyleContext,
                                      parent: Option<Self::ConcreteNode>)
                                      -> StyleSharingResult<<Self::ConcreteNode as TNode>::ConcreteRestyleDamage> {
        if opts::get().disable_share_style_cache {
            return StyleSharingResult::CannotShare
        }

        if self.style_attribute().is_some() {
            return StyleSharingResult::CannotShare
        }

        if self.has_attr(&ns!(), &atom!("id")) {
            return StyleSharingResult::CannotShare
        }

        let parent = match parent {
            Some(parent) if parent.is_element() => parent,
            _ => return StyleSharingResult::CannotShare,
        };

        let iter = style_sharing_candidate_cache.iter().map(|&(unsafe_node, ())| {
            Self::ConcreteNode::from_unsafe(&unsafe_node).as_element().unwrap()
        });

        for (i, candidate) in iter.enumerate() {
            if let Some(shared_style) = self.share_style_with_candidate_if_possible(parent,
                                                                                    shared_context,
                                                                                    &candidate) {
                // Yay, cache hit. Share the style.
                let node = self.as_node();
                let style = &mut node.mutate_data().unwrap().style;
                let damage = <<Self as TElement>::ConcreteNode as TNode>
                                 ::ConcreteRestyleDamage::compute((*style).as_ref(), &*shared_style);
                *style = Some(shared_style);
                return StyleSharingResult::StyleWasShared(i, damage)
            }
        }

        StyleSharingResult::CannotShare
    }
}

impl<E: TElement> ElementMatchMethods for E
    where E::Impl: SelectorImplExt {}

pub trait MatchMethods : TNode {
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
        // Only elements are interesting.
        if let Some(element) = self.as_element() {
            bf.insert(&*element.get_local_name());
            bf.insert(&*element.get_namespace());
            element.get_id().map(|id| bf.insert(&id));

            // TODO: case-sensitivity depends on the document type and quirks mode
            element.each_class(|class| bf.insert(class));
        }
    }

    /// After all the children are done css selector matching, this must be
    /// called to reset the bloom filter after an `insert`.
    fn remove_from_bloom_filter(&self, bf: &mut BloomFilter) {
        // Only elements are interesting.
        if let Some(element) = self.as_element() {
            bf.remove(&*element.get_local_name());
            bf.remove(&*element.get_namespace());
            element.get_id().map(|id| bf.remove(&id));

            // TODO: case-sensitivity depends on the document type and quirks mode
            element.each_class(|class| bf.remove(class));
        }
    }

    unsafe fn cascade_node<'a, Ctx>(&self,
                                    context: &Ctx,
                                    parent: Option<Self>,
                                    applicable_declarations: &ApplicableDeclarations)
    where Ctx: StyleContext<'a> {
        // Get our parent's style. This must be unsafe so that we don't touch the parent's
        // borrow flags.
        //
        // FIXME(pcwalton): Isolate this unsafety into the `wrapper` module to allow
        // enforced safe, race-free access to the parent style.
        let parent_style = match parent {
            Some(parent_node) => {
                let parent_style = (*parent_node.borrow_data_unchecked().unwrap()).style.as_ref().unwrap();
                Some(parent_style)
            }
            None => None,
        };

        let mut applicable_declarations_cache =
            context.local_context().applicable_declarations_cache.borrow_mut();

        let damage;
        if self.is_text_node() {
            let mut data_ref = self.mutate_data().unwrap();
            let mut data = &mut *data_ref;
            let cloned_parent_style = ComputedValues::style_for_child_text_node(parent_style.unwrap());
            damage = Self::ConcreteRestyleDamage::compute(data.style.as_ref(),
                                                          &*cloned_parent_style);
            data.style = Some(cloned_parent_style);
        } else {
            damage = {
                let mut data_ref = self.mutate_data().unwrap();
                let mut data = &mut *data_ref;
                let (mut damage, final_style) = self.cascade_node_pseudo_element(
                    context,
                    parent_style,
                    &applicable_declarations.normal,
                    data.style.as_mut(),
                    &mut applicable_declarations_cache,
                    applicable_declarations.normal_shareable,
                    true);

                data.style = Some(final_style);

                <Self::ConcreteElement as Element>::Impl::each_eagerly_cascaded_pseudo_element(|pseudo| {
                    let applicable_declarations_for_this_pseudo =
                        applicable_declarations.per_pseudo.get(&pseudo).unwrap();


                    if !applicable_declarations_for_this_pseudo.is_empty() {
                        // NB: Transitions and animations should only work for
                        // pseudo-elements ::before and ::after
                        let should_animate_properties =
                            <Self::ConcreteElement as Element>::Impl::pseudo_is_before_or_after(&pseudo);
                        let (new_damage, style) = self.cascade_node_pseudo_element(
                            context,
                            Some(data.style.as_ref().unwrap()),
                            &*applicable_declarations_for_this_pseudo,
                            data.per_pseudo.get_mut(&pseudo),
                            &mut applicable_declarations_cache,
                            false,
                            should_animate_properties);
                        data.per_pseudo.insert(pseudo, style);

                        damage = damage | new_damage;
                    }
                });

                damage
            };

            // This method needs to borrow the data as mutable, so make sure data_ref goes out of
            // scope first.
            self.set_can_be_fragmented(parent.map_or(false, |p| {
                p.can_be_fragmented() ||
                parent_style.as_ref().unwrap().is_multicol()
            }));
        }

        // This method needs to borrow the data as mutable, so make sure data_ref goes out of
        // scope first.
        self.set_restyle_damage(damage);
    }
}

impl<N: TNode> MatchMethods for N {}
