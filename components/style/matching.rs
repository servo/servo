/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! High-level interface to CSS selector matching.

#![allow(unsafe_code)]

use animation;
use arc_ptr_eq;
use cache::{LRUCache, SimpleHashCache};
use cascade_info::CascadeInfo;
use context::{StyleContext, SharedStyleContext};
use data::PrivateStyleData;
use dom::{TElement, TNode, TRestyleDamage, UnsafeNode};
use properties::longhands::display::computed_value as display;
use properties::{ComputedValues, cascade};
use selector_impl::{TheSelectorImpl, PseudoElement};
use selector_matching::{ApplicableDeclarationBlock, Stylist};
use selectors::bloom::BloomFilter;
use selectors::matching::{MatchingReason, StyleRelations, AFFECTED_BY_PSEUDO_ELEMENTS};
use selectors::{Element, MatchAttr};
use sink::ForgetfulSink;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::ops::Deref;
use std::slice::IterMut;
use std::sync::Arc;
use string_cache::Atom;
use traversal::RestyleResult;
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
    pub normal: SmallVec<[ApplicableDeclarationBlock; 16]>,
    pub per_pseudo: HashMap<PseudoElement,
                            Vec<ApplicableDeclarationBlock>,
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
    pub declarations: Vec<ApplicableDeclarationBlock>,
}

impl ApplicableDeclarationsCacheEntry {
    fn new(declarations: Vec<ApplicableDeclarationBlock>) -> ApplicableDeclarationsCacheEntry {
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
    declarations: &'a [ApplicableDeclarationBlock],
}

impl<'a> ApplicableDeclarationsCacheQuery<'a> {
    fn new(declarations: &'a [ApplicableDeclarationBlock]) -> ApplicableDeclarationsCacheQuery<'a> {
        ApplicableDeclarationsCacheQuery {
            declarations: declarations,
        }
    }
}

impl<'a> PartialEq for ApplicableDeclarationsCacheQuery<'a> {
    fn eq(&self, other: &ApplicableDeclarationsCacheQuery<'a>) -> bool {
        self.declarations.len() == other.declarations.len() &&
        self.declarations.iter().zip(other.declarations).all(|(this, other)| {
            arc_ptr_eq(&this.mixed_declarations, &other.mixed_declarations) &&
            this.importance == other.importance
        })
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
            let ptr: *const _ = Arc::deref(&declaration.mixed_declarations);
            ptr.hash(state);
            declaration.importance.hash(state);
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

    pub fn find(&self, declarations: &[ApplicableDeclarationBlock]) -> Option<Arc<ComputedValues>> {
        match self.cache.find(&ApplicableDeclarationsCacheQuery::new(declarations)) {
            None => None,
            Some(ref values) => Some((*values).clone()),
        }
    }

    pub fn insert(&mut self, declarations: Vec<ApplicableDeclarationBlock>, style: Arc<ComputedValues>) {
        self.cache.insert(ApplicableDeclarationsCacheEntry::new(declarations), style)
    }

    pub fn evict_all(&mut self) {
        self.cache.evict_all();
    }
}

/// Information regarding a candidate.
///
/// TODO: We can stick a lot more info here.
#[derive(Debug)]
struct StyleSharingCandidate {
    /// The node, guaranteed to be an element.
    node: UnsafeNode,
    /// The cached computed style, here for convenience.
    style: Arc<ComputedValues>,
    /// The cached common style affecting attribute info.
    common_style_affecting_attributes: Option<CommonStyleAffectingAttributes>,
    /// the cached class names.
    class_attributes: Option<Vec<Atom>>,
}

impl PartialEq<StyleSharingCandidate> for StyleSharingCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node &&
            arc_ptr_eq(&self.style, &other.style) &&
            self.common_style_affecting_attributes == other.common_style_affecting_attributes
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
    cache: LRUCache<StyleSharingCandidate, ()>,
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
                                          candidate: &mut StyleSharingCandidate,
                                          candidate_element: &E,
                                          shared_context: &SharedStyleContext)
                                          -> Result<Arc<ComputedValues>, CacheMiss> {
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

    Ok(candidate.style.clone())
}

fn have_same_common_style_affecting_attributes<E: TElement>(element: &E,
                                                            candidate: &mut StyleSharingCandidate,
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
    pub flags CommonStyleAffectingAttributes: u8 {
        const HIDDEN_ATTRIBUTE = 0x01,
        const NO_WRAP_ATTRIBUTE = 0x02,
        const ALIGN_LEFT_ATTRIBUTE = 0x04,
        const ALIGN_CENTER_ATTRIBUTE = 0x08,
        const ALIGN_RIGHT_ATTRIBUTE = 0x10,
    }
}

pub struct CommonStyleAffectingAttributeInfo {
    pub atom: Atom,
    pub mode: CommonStyleAffectingAttributeMode,
}

#[derive(Clone)]
pub enum CommonStyleAffectingAttributeMode {
    IsPresent(CommonStyleAffectingAttributes),
    IsEqual(Atom, CommonStyleAffectingAttributes),
}

// NB: This must match the order in `selectors::matching::CommonStyleAffectingAttributes`.
#[inline]
pub fn common_style_affecting_attributes() -> [CommonStyleAffectingAttributeInfo; 5] {
    [
        CommonStyleAffectingAttributeInfo {
            atom: atom!("hidden"),
            mode: CommonStyleAffectingAttributeMode::IsPresent(HIDDEN_ATTRIBUTE),
        },
        CommonStyleAffectingAttributeInfo {
            atom: atom!("nowrap"),
            mode: CommonStyleAffectingAttributeMode::IsPresent(NO_WRAP_ATTRIBUTE),
        },
        CommonStyleAffectingAttributeInfo {
            atom: atom!("align"),
            mode: CommonStyleAffectingAttributeMode::IsEqual(atom!("left"), ALIGN_LEFT_ATTRIBUTE),
        },
        CommonStyleAffectingAttributeInfo {
            atom: atom!("align"),
            mode: CommonStyleAffectingAttributeMode::IsEqual(atom!("center"), ALIGN_CENTER_ATTRIBUTE),
        },
        CommonStyleAffectingAttributeInfo {
            atom: atom!("align"),
            mode: CommonStyleAffectingAttributeMode::IsEqual(atom!("right"), ALIGN_RIGHT_ATTRIBUTE),
        }
    ]
}

/// Attributes that, if present, disable style sharing. All legacy HTML attributes must be in
/// either this list or `common_style_affecting_attributes`. See the comment in
/// `synthesize_presentational_hints_for_legacy_attributes`.
pub fn rare_style_affecting_attributes() -> [Atom; 3] {
    [ atom!("bgcolor"), atom!("border"), atom!("colspan") ]
}

fn have_same_class<E: TElement>(element: &E,
                                candidate: &mut StyleSharingCandidate,
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

impl StyleSharingCandidateCache {
    pub fn new() -> Self {
        StyleSharingCandidateCache {
            cache: LRUCache::new(STYLE_SHARING_CANDIDATE_CACHE_SIZE),
        }
    }

    fn iter_mut(&mut self) -> IterMut<(StyleSharingCandidate, ())> {
        self.cache.iter_mut()
    }

    pub fn insert_if_possible<E: TElement>(&mut self,
                                           element: &E,
                                           relations: StyleRelations) {
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

        let node = element.as_node();
        let data = node.borrow_data().unwrap();
        let style = data.style.as_ref().unwrap();

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
               element.as_node().to_unsafe(), parent.as_node().to_unsafe());

        self.cache.insert(StyleSharingCandidate {
            node: node.to_unsafe(),
            style: style.clone(),
            common_style_affecting_attributes: None,
            class_attributes: None,
        }, ());
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
    /// The node's style can be shared. The integer specifies the index in the
    /// LRU cache that was hit and the damage that was done, and the restyle
    /// result the original result of the candidate's styling, that is, whether
    /// it should stop the traversal or not.
    StyleWasShared(usize, ConcreteRestyleDamage, RestyleResult),
}

trait PrivateMatchMethods: TNode {
    /// Actually cascades style for a node or a pseudo-element of a node.
    ///
    /// Note that animations only apply to nodes or ::before or ::after
    /// pseudo-elements.
    fn cascade_node_pseudo_element<'a, Ctx>(&self,
                                            context: &Ctx,
                                            parent_style: Option<&Arc<ComputedValues>>,
                                            applicable_declarations: &[ApplicableDeclarationBlock],
                                            mut old_style: Option<&mut Arc<ComputedValues>>,
                                            applicable_declarations_cache:
                                             &mut ApplicableDeclarationsCache,
                                            shareable: bool,
                                            animate_properties: bool)
                                            -> Arc<ComputedValues>
        where Ctx: StyleContext<'a>
    {
        let mut cacheable = true;
        let shared_context = context.shared_context();
        if animate_properties {
            cacheable = !self.update_animations_for_cascade(shared_context,
                                                            &mut old_style) && cacheable;
        }

        let mut cascade_info = CascadeInfo::new();
        let (this_style, is_cacheable) = match parent_style {
            Some(ref parent_style) => {
                let cache_entry = applicable_declarations_cache.find(applicable_declarations);
                let cached_computed_values = match cache_entry {
                    Some(ref style) => Some(&**style),
                    None => None,
                };

                cascade(shared_context.viewport_size,
                        applicable_declarations,
                        shareable,
                        Some(&***parent_style),
                        cached_computed_values,
                        Some(&mut cascade_info),
                        shared_context.error_reporter.clone())
            }
            None => {
                cascade(shared_context.viewport_size,
                        applicable_declarations,
                        shareable,
                        None,
                        None,
                        Some(&mut cascade_info),
                        shared_context.error_reporter.clone())
            }
        };
        cascade_info.finish(self);

        cacheable = cacheable && is_cacheable;

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
            if let Some(ref style) = old_style {
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

        // Cache the resolved style if it was cacheable.
        if cacheable {
            applicable_declarations_cache.insert(applicable_declarations.to_vec(),
                                                 this_style.clone());
        }

        this_style
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
                    animation::update_style_for_animation(context,
                                                          running_animation,
                                                          style);
                    running_animation.mark_as_expired();
                }
            }
        }

        had_animations_to_expire || had_running_animations
    }
}

impl<N: TNode> PrivateMatchMethods for N {}

trait PrivateElementMatchMethods: TElement {
    fn share_style_with_candidate_if_possible(&self,
                                              parent_node: Self::ConcreteNode,
                                              shared_context: &SharedStyleContext,
                                              candidate: &mut StyleSharingCandidate)
                                              -> Result<Arc<ComputedValues>, CacheMiss> {
        debug_assert!(parent_node.is_element());

        let candidate_element = unsafe {
            Self::ConcreteNode::from_unsafe(&candidate.node).as_element().unwrap()
        };

        element_matches_candidate(self, candidate, &candidate_element,
                                  shared_context)
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
        let style_attribute = self.style_attribute();

        let mut relations =
            stylist.push_applicable_declarations(self,
                                                 parent_bf,
                                                 style_attribute,
                                                 None,
                                                 &mut applicable_declarations.normal,
                                                 MatchingReason::ForStyling);

        applicable_declarations.normal_shareable = relations_are_shareable(&relations);

        TheSelectorImpl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            stylist.push_applicable_declarations(self,
                                                 parent_bf,
                                                 None,
                                                 Some(&pseudo.clone()),
                                                 applicable_declarations.per_pseudo.entry(pseudo).or_insert(vec![]),
                                                 MatchingReason::ForStyling);
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

        for (i, &mut (ref mut candidate, ())) in style_sharing_candidate_cache.iter_mut().enumerate() {
            let sharing_result = self.share_style_with_candidate_if_possible(parent,
                                                                             shared_context,
                                                                             candidate);
            match sharing_result {
                Ok(shared_style) => {
                    // Yay, cache hit. Share the style.
                    let node = self.as_node();
                    let style = &mut node.mutate_data().unwrap().style;

                    // TODO: add the display: none optimisation here too! Even
                    // better, factor it out/make it a bit more generic so Gecko
                    // can decide more easily if it knows that it's a child of
                    // replaced content, or similar stuff!
                    let damage =
                        match node.existing_style_for_restyle_damage((*style).as_ref(), None) {
                            Some(ref source) => {
                                <<Self as TElement>::ConcreteNode as TNode>
                                ::ConcreteRestyleDamage::compute(source, &shared_style)
                            }
                            None => {
                                <<Self as TElement>::ConcreteNode as TNode>
                                ::ConcreteRestyleDamage::rebuild_and_reflow()
                            }
                        };

                    let restyle_result = if shared_style.get_box().clone_display() == display::T::none {
                        RestyleResult::Stop
                    } else {
                        RestyleResult::Continue
                    };

                    *style = Some(shared_style);

                    return StyleSharingResult::StyleWasShared(i, damage, restyle_result)
                }
                Err(miss) => {
                    debug!("Cache miss: {:?}", miss);

                    // Cache miss, let's see what kind of failure to decide
                    // whether we keep trying or not.
                    match miss {
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

        StyleSharingResult::CannotShare
    }
}

impl<E: TElement> ElementMatchMethods for E {}

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

    fn compute_restyle_damage(&self,
                              old_style: Option<&Arc<ComputedValues>>,
                              new_style: &Arc<ComputedValues>,
                              pseudo: Option<&PseudoElement>)
                              -> Self::ConcreteRestyleDamage
    {
        match self.existing_style_for_restyle_damage(old_style, pseudo) {
            Some(ref source) => {
                Self::ConcreteRestyleDamage::compute(source,
                                                     new_style)
            }
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
                Self::ConcreteRestyleDamage::rebuild_and_reflow()
            }
        }
    }

    unsafe fn cascade_node<'a, Ctx>(&self,
                                    context: &Ctx,
                                    parent: Option<Self>,
                                    applicable_declarations: &ApplicableDeclarations)
                                    -> RestyleResult
        where Ctx: StyleContext<'a>
    {
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


        // In the case we're styling a text node, we don't need to compute the
        // restyle damage, since it's a subset of the restyle damage of the
        // parent.
        //
        // In Gecko, we're done, we don't need anything else from text nodes.
        //
        // In Servo, this is also true, since text nodes generate UnscannedText
        // fragments, which aren't repairable by incremental layout.
        if self.is_text_node() {
            let mut data_ref = self.mutate_data().unwrap();
            let mut data = &mut *data_ref;
            let cloned_parent_style = ComputedValues::style_for_child_text_node(parent_style.unwrap());

            data.style = Some(cloned_parent_style);

            return RestyleResult::Continue;
        }

        let mut applicable_declarations_cache =
            context.local_context().applicable_declarations_cache.borrow_mut();

        let (damage, restyle_result) = {
            let mut data_ref = self.mutate_data().unwrap();
            let mut data = &mut *data_ref;
            let final_style =
                self.cascade_node_pseudo_element(context, parent_style,
                                                 &applicable_declarations.normal,
                                                 data.style.as_mut(),
                                                 &mut applicable_declarations_cache,
                                                 applicable_declarations.normal_shareable,
                                                 /* should_animate = */ true);

            let (damage, restyle_result) =
                self.compute_damage_and_cascade_pseudos(final_style,
                                                        data,
                                                        context,
                                                        applicable_declarations,
                                                        &mut applicable_declarations_cache);

            self.set_can_be_fragmented(parent.map_or(false, |p| {
                p.can_be_fragmented() ||
                parent_style.as_ref().unwrap().is_multicol()
            }));

            (damage, restyle_result)
        };


        // This method needs to borrow the data as mutable, so make sure
        // data_ref goes out of scope first.
        self.set_restyle_damage(damage);

        restyle_result
    }

    fn compute_damage_and_cascade_pseudos<'a, Ctx>(&self,
                                                   final_style: Arc<ComputedValues>,
                                                   data: &mut PrivateStyleData,
                                                   context: &Ctx,
                                                   applicable_declarations: &ApplicableDeclarations,
                                                   mut applicable_declarations_cache: &mut ApplicableDeclarationsCache)
                                                   -> (Self::ConcreteRestyleDamage, RestyleResult)
        where Ctx: StyleContext<'a>
    {
        // Here we optimise the case of the style changing but both the
        // previous and the new styles having display: none. In this
        // case, we can always optimize the traversal, regardless of the
        // restyle hint.
        let this_display = final_style.get_box().clone_display();
        if this_display == display::T::none {
            let old_display = data.style.as_ref().map(|old_style| {
                old_style.get_box().clone_display()
            });

            // If display passed from none to something, then we need to reflow,
            // otherwise, we don't do anything.
            let damage = match old_display {
                Some(display) if display == this_display => {
                    Self::ConcreteRestyleDamage::empty()
                }
                _ => Self::ConcreteRestyleDamage::rebuild_and_reflow()
            };

            debug!("Short-circuiting traversal: {:?} {:?} {:?}",
                   this_display, old_display, damage);

            data.style = Some(final_style);
            return (damage, RestyleResult::Stop);
        }

        // Otherwise, we just compute the damage normally, and sum up the damage
        // related to pseudo-elements.
        let mut damage =
            self.compute_restyle_damage(data.style.as_ref(), &final_style, None);

        data.style = Some(final_style);

        let data_per_pseudo = &mut data.per_pseudo;
        let new_style = data.style.as_ref();

        debug_assert!(new_style.is_some());

        let rebuild_and_reflow =
            Self::ConcreteRestyleDamage::rebuild_and_reflow();

        <Self::ConcreteElement as MatchAttr>::Impl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            use std::collections::hash_map::Entry;

            let applicable_declarations_for_this_pseudo =
                applicable_declarations.per_pseudo.get(&pseudo).unwrap();

            let has_declarations =
                !applicable_declarations_for_this_pseudo.is_empty();

            // If there are declarations matching, we're going to need to
            // recompute the style anyway, so do it now to simplify the logic
            // below.
            let pseudo_style_if_declarations = if has_declarations {
                // NB: Transitions and animations should only work for
                // pseudo-elements ::before and ::after
                let should_animate_properties =
                    <Self::ConcreteElement as MatchAttr>::Impl::pseudo_is_before_or_after(&pseudo);

                Some(self.cascade_node_pseudo_element(context,
                                                      new_style,
                                                      &*applicable_declarations_for_this_pseudo,
                                                      data_per_pseudo.get_mut(&pseudo),
                                                      &mut applicable_declarations_cache,
                                                      /* shareable = */ false,
                                                      should_animate_properties))
            } else {
                None
            };

            // Let's see what we had before.
            match data_per_pseudo.entry(pseudo.clone()) {
                Entry::Vacant(vacant_entry) => {
                    // If we had a vacant entry, and no rules that match, we're
                    // fine so far.
                    if !has_declarations {
                        return;
                    }

                    // Otherwise, we need to insert the new computed styles, and
                    // generate a rebuild_and_reflow damage.
                    damage = damage | Self::ConcreteRestyleDamage::rebuild_and_reflow();
                    vacant_entry.insert(pseudo_style_if_declarations.unwrap());
                }
                Entry::Occupied(mut occupied_entry) => {
                    // If there was an existing style, and no declarations, we
                    // need to remove us from the map, and ensure we're
                    // reconstructing.
                    if !has_declarations {
                        damage = damage | Self::ConcreteRestyleDamage::rebuild_and_reflow();
                        occupied_entry.remove();
                        return;
                    }

                    // If there's a new style, we need to diff it and add the
                    // damage, except if the damage was already
                    // rebuild_and_reflow, in which case we can avoid it.
                    if damage != rebuild_and_reflow {
                        damage = damage |
                                 self.compute_restyle_damage(Some(occupied_entry.get()),
                                                             pseudo_style_if_declarations.as_ref().unwrap(),
                                                             Some(&pseudo));
                    }

                    // And now, of course, use the new style.
                    occupied_entry.insert(pseudo_style_if_declarations.unwrap());
                }
            }
        });

        (damage, RestyleResult::Continue)
    }
}

impl<N: TNode> MatchMethods for N {}
