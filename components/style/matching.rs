/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use animation::{self, Animation};
use context::SharedStyleContext;
use data::PrivateStyleData;
use dom::{TElement, TNode, TRestyleDamage};
use properties::{ComputedValues, PropertyDeclaration, cascade};
use selector_impl::SelectorImplExt;
use selector_matching::{DeclarationBlock, Stylist};
use selectors::Element;
use selectors::bloom::BloomFilter;
use selectors::matching::{CommonStyleAffectingAttributeMode, CommonStyleAffectingAttributes};
use selectors::matching::{common_style_affecting_attributes, rare_style_affecting_attributes};
use smallvec::SmallVec;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::slice::Iter;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use string_cache::{Atom, Namespace};
use util::arc_ptr_eq;
use util::cache::{LRUCache, SimpleHashCache};
use util::opts;
use util::vec::ForgetfulSink;

/// High-level interface to CSS selector matching.

fn create_common_style_affecting_attributes_from_element<E: TElement>(element: &E)
                                                         -> CommonStyleAffectingAttributes {
    let mut flags = CommonStyleAffectingAttributes::empty();
    for attribute_info in &common_style_affecting_attributes() {
        match attribute_info.mode {
            CommonStyleAffectingAttributeMode::IsPresent(flag) => {
                if element.get_attr(&ns!(), &attribute_info.atom).is_some() {
                    flags.insert(flag)
                }
            }
            CommonStyleAffectingAttributeMode::IsEqual(target_value, flag) => {
                match element.get_attr(&ns!(), &attribute_info.atom) {
                    Some(element_value) if element_value == target_value => {
                        flags.insert(flag)
                    }
                    _ => {}
                }
            }
        }
    }
    flags
}

pub struct ApplicableDeclarations<Impl: SelectorImplExt> {
    pub normal: SmallVec<[DeclarationBlock; 16]>,
    pub per_pseudo: HashMap<Impl::PseudoElement, Vec<DeclarationBlock>, BuildHasherDefault<::fnv::FnvHasher>>,

    /// Whether the `normal` declarations are shareable with other nodes.
    pub normal_shareable: bool,
}

impl<Impl: SelectorImplExt> ApplicableDeclarations<Impl> {
    pub fn new() -> ApplicableDeclarations<Impl> {
        let mut applicable_declarations = ApplicableDeclarations {
            normal: SmallVec::new(),
            per_pseudo: HashMap::with_hasher(Default::default()),
            normal_shareable: false,
        };

        Impl::each_eagerly_cascaded_pseudo_element(|pseudo| {
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
    pub fn new() -> ApplicableDeclarationsCache {
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

/// An LRU cache of the last few nodes seen, so that we can aggressively try to reuse their styles.
pub struct StyleSharingCandidateCache {
    cache: LRUCache<StyleSharingCandidate, ()>,
}

#[derive(Clone)]
pub struct StyleSharingCandidate {
    pub style: Arc<ComputedValues>,
    pub parent_style: Arc<ComputedValues>,
    pub local_name: Atom,
    // FIXME(pcwalton): Should be a list of atoms instead.
    pub class: Option<String>,
    pub namespace: Namespace,
    pub common_style_affecting_attributes: CommonStyleAffectingAttributes,
    pub link: bool,
}

impl PartialEq for StyleSharingCandidate {
    fn eq(&self, other: &StyleSharingCandidate) -> bool {
        arc_ptr_eq(&self.style, &other.style) &&
            arc_ptr_eq(&self.parent_style, &other.parent_style) &&
            self.local_name == other.local_name &&
            self.class == other.class &&
            self.link == other.link &&
            self.namespace == other.namespace &&
            self.common_style_affecting_attributes == other.common_style_affecting_attributes
    }
}

impl StyleSharingCandidate {
    /// Attempts to create a style sharing candidate from this node. Returns
    /// the style sharing candidate or `None` if this node is ineligible for
    /// style sharing.
    #[allow(unsafe_code)]
    fn new<E: TElement>(element: &E) -> Option<StyleSharingCandidate> {
        let parent_element = match element.parent_element() {
            None => return None,
            Some(parent_element) => parent_element,
        };

        let style = unsafe {
            match element.as_node().borrow_data_unchecked() {
                None => return None,
                Some(data_ref) => {
                    match (*data_ref).style {
                        None => return None,
                        Some(ref data) => (*data).clone(),
                    }
                }
            }
        };
        let parent_style = unsafe {
            match parent_element.as_node().borrow_data_unchecked() {
                None => return None,
                Some(parent_data_ref) => {
                    match (*parent_data_ref).style {
                        None => return None,
                        Some(ref data) => (*data).clone(),
                    }
                }
            }
        };

        if element.style_attribute().is_some() {
            return None
        }

        Some(StyleSharingCandidate {
            style: style,
            parent_style: parent_style,
            local_name: element.get_local_name().clone(),
            class: element.get_attr(&ns!(), &atom!("class"))
                          .map(|string| string.to_owned()),
            link: element.is_link(),
            namespace: (*element.get_namespace()).clone(),
            common_style_affecting_attributes:
                   create_common_style_affecting_attributes_from_element::<E>(&element)
        })
    }

    pub fn can_share_style_with<E: TElement>(&self, element: &E) -> bool {
        if *element.get_local_name() != self.local_name {
            return false
        }

        // FIXME(pcwalton): Use `each_class` here instead of slow string comparison.
        match (&self.class, element.get_attr(&ns!(), &atom!("class"))) {
            (&None, Some(_)) | (&Some(_), None) => return false,
            (&Some(ref this_class), Some(element_class)) if
                    element_class != &**this_class => {
                return false
            }
            (&Some(_), Some(_)) | (&None, None) => {}
        }

        if *element.get_namespace() != self.namespace {
            return false
        }

        let mut matching_rules = ForgetfulSink::new();
        element.synthesize_presentational_hints_for_legacy_attributes(&mut matching_rules);
        if !matching_rules.is_empty() {
            return false;
        }

        // FIXME(pcwalton): It's probably faster to iterate over all the element's attributes and
        // use the {common, rare}-style-affecting-attributes tables as lookup tables.

        for attribute_info in &common_style_affecting_attributes() {
            match attribute_info.mode {
                CommonStyleAffectingAttributeMode::IsPresent(flag) => {
                    if self.common_style_affecting_attributes.contains(flag) !=
                            element.get_attr(&ns!(), &attribute_info.atom).is_some() {
                        return false
                    }
                }
                CommonStyleAffectingAttributeMode::IsEqual(target_value, flag) => {
                    match element.get_attr(&ns!(), &attribute_info.atom) {
                        Some(ref element_value) if self.common_style_affecting_attributes
                                                       .contains(flag) &&
                                                       *element_value != target_value => {
                            return false
                        }
                        Some(_) if !self.common_style_affecting_attributes.contains(flag) => {
                            return false
                        }
                        None if self.common_style_affecting_attributes.contains(flag) => {
                            return false
                        }
                        _ => {}
                    }
                }
            }
        }

        for attribute_name in &rare_style_affecting_attributes() {
            if element.get_attr(&ns!(), attribute_name).is_some() {
                return false
            }
        }

        if element.is_link() != self.link {
            return false
        }

        // TODO(pcwalton): We don't support visited links yet, but when we do there will need to
        // be some logic here.

        true
    }
}

static STYLE_SHARING_CANDIDATE_CACHE_SIZE: usize = 40;

impl StyleSharingCandidateCache {
    pub fn new() -> StyleSharingCandidateCache {
        StyleSharingCandidateCache {
            cache: LRUCache::new(STYLE_SHARING_CANDIDATE_CACHE_SIZE),
        }
    }

    pub fn iter(&self) -> Iter<(StyleSharingCandidate, ())> {
        self.cache.iter()
    }

    pub fn insert_if_possible<E: TElement>(&mut self, element: &E) {
        match StyleSharingCandidate::new(element) {
            None => {}
            Some(candidate) => self.cache.insert(candidate, ())
        }
    }

    pub fn touch(&mut self, index: usize) {
        self.cache.touch(index)
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
    fn cascade_node_pseudo_element(&self,
                                   context: &SharedStyleContext<<Self::ConcreteElement as Element>::Impl>,
                                   parent_style: Option<&Arc<ComputedValues>>,
                                   applicable_declarations: &[DeclarationBlock],
                                   mut style: Option<&mut Arc<ComputedValues>>,
                                   applicable_declarations_cache:
                                    &mut ApplicableDeclarationsCache,
                                   new_animations_sender: &Mutex<Sender<Animation>>,
                                   shareable: bool,
                                   animate_properties: bool)
                                   -> (Self::ConcreteRestyleDamage, Arc<ComputedValues>) {
        let mut cacheable = true;
        if animate_properties {
            cacheable = !self.update_animations_for_cascade(context, &mut style) && cacheable;
        }

        let mut this_style;
        match parent_style {
            Some(ref parent_style) => {
                let cache_entry = applicable_declarations_cache.find(applicable_declarations);
                let cached_computed_values = match cache_entry {
                    None => None,
                    Some(ref style) => Some(&**style),
                };
                let (the_style, is_cacheable) = cascade(context.viewport_size,
                                                        applicable_declarations,
                                                        shareable,
                                                        Some(&***parent_style),
                                                        cached_computed_values,
                                                        context.error_reporter.clone());
                cacheable = cacheable && is_cacheable;
                this_style = the_style
            }
            None => {
                let (the_style, is_cacheable) = cascade(context.viewport_size,
                                                        applicable_declarations,
                                                        shareable,
                                                        None,
                                                        None,
                                                        context.error_reporter.clone());
                cacheable = cacheable && is_cacheable;
                this_style = the_style
            }
        };

        // Trigger transitions if necessary. This will reset `this_style` back to its old value if
        // it did trigger a transition.
        if animate_properties {
            if let Some(ref style) = style {
                let animations_started =
                    animation::start_transitions_if_applicable(new_animations_sender,
                                                               self.opaque(),
                                                               &**style,
                                                               &mut this_style);
                cacheable = cacheable && !animations_started
            }
        }

        // Calculate style difference.
        let this_style = Arc::new(this_style);
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
                                     context: &SharedStyleContext<<Self::ConcreteElement as Element>::Impl>,
                                     style: &mut Option<&mut Arc<ComputedValues>>)
                                     -> bool {
        let style = match *style {
            None => return false,
            Some(ref mut style) => style,
        };

        // Finish any expired transitions.
        let this_opaque = self.opaque();
        let had_animations_to_expire;
        {
            let all_expired_animations = context.expired_animations.read().unwrap();
            let animations_to_expire = all_expired_animations.get(&this_opaque);
            had_animations_to_expire = animations_to_expire.is_some();
            if let Some(ref animations) = animations_to_expire {
                for animation in *animations {
                    animation.property_animation.update(&mut *Arc::make_mut(style), 1.0);
                }
            }
        }

        if had_animations_to_expire {
            context.expired_animations.write().unwrap().remove(&this_opaque);
        }

        // Merge any running transitions into the current style, and cancel them.
        let had_running_animations = context.running_animations
                                            .read()
                                            .unwrap()
                                            .get(&this_opaque)
                                            .is_some();
        if had_running_animations {
            let mut all_running_animations = context.running_animations.write().unwrap();
            for running_animation in all_running_animations.get(&this_opaque).unwrap() {
                animation::update_style_for_animation::<Self::ConcreteRestyleDamage>(running_animation, style, None);
            }
            all_running_animations.remove(&this_opaque);
        }

        had_animations_to_expire || had_running_animations
    }
}

impl<N: TNode> PrivateMatchMethods for N
    where <N::ConcreteElement as Element>::Impl: SelectorImplExt {}

trait PrivateElementMatchMethods: TElement {
    fn share_style_with_candidate_if_possible(&self,
                                              parent_node: Option<Self::ConcreteNode>,
                                              candidate: &StyleSharingCandidate)
                                              -> Option<Arc<ComputedValues>> {
        let parent_node = match parent_node {
            Some(ref parent_node) if parent_node.as_element().is_some() => parent_node,
            Some(_) | None => return None,
        };

        let parent_data: Option<&PrivateStyleData<_>> = unsafe {
            parent_node.borrow_data_unchecked().map(|d| &*d)
        };

        if let Some(parent_data_ref) = parent_data {
            // Check parent style.
            let parent_style = (*parent_data_ref).style.as_ref().unwrap();
            if !arc_ptr_eq(parent_style, &candidate.parent_style) {
                return None
            }
            // Check tag names, classes, etc.
            if !candidate.can_share_style_with(self) {
                return None
            }
            return Some(candidate.style.clone())
        }
        None
    }
}

impl<E: TElement> PrivateElementMatchMethods for E {}

pub trait ElementMatchMethods : TElement
    where Self::Impl: SelectorImplExt {
    fn match_element(&self,
                     stylist: &Stylist<Self::Impl>,
                     parent_bf: Option<&BloomFilter>,
                     applicable_declarations: &mut ApplicableDeclarations<Self::Impl>)
                     -> bool {
        let style_attribute = self.style_attribute().as_ref();

        applicable_declarations.normal_shareable =
            stylist.push_applicable_declarations(self,
                                                 parent_bf,
                                                 style_attribute,
                                                 None,
                                                 &mut applicable_declarations.normal);
        Self::Impl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            stylist.push_applicable_declarations(self,
                                                 parent_bf,
                                                 None,
                                                 Some(pseudo.clone()),
                                                 applicable_declarations.per_pseudo.entry(pseudo).or_insert(vec![]));
        });

        applicable_declarations.normal_shareable &&
        applicable_declarations.per_pseudo.values().all(|v| v.is_empty())
    }

    /// Attempts to share a style with another node. This method is unsafe because it depends on
    /// the `style_sharing_candidate_cache` having only live nodes in it, and we have no way to
    /// guarantee that at the type system level yet.
    unsafe fn share_style_if_possible(&self,
                                      style_sharing_candidate_cache:
                                        &mut StyleSharingCandidateCache,
                                      parent: Option<Self::ConcreteNode>)
                                      -> StyleSharingResult<<Self::ConcreteNode as TNode>::ConcreteRestyleDamage> {
        if opts::get().disable_share_style_cache {
            return StyleSharingResult::CannotShare
        }

        if self.style_attribute().is_some() {
            return StyleSharingResult::CannotShare
        }
        if self.get_attr(&ns!(), &atom!("id")).is_some() {
            return StyleSharingResult::CannotShare
        }

        for (i, &(ref candidate, ())) in style_sharing_candidate_cache.iter().enumerate() {
            match self.share_style_with_candidate_if_possible(parent.clone(), candidate) {
                Some(shared_style) => {
                    // Yay, cache hit. Share the style.
                    let node = self.as_node();
                    let style = &mut node.mutate_data().unwrap().style;
                    let damage = <<Self as TElement>::ConcreteNode as TNode>
                                     ::ConcreteRestyleDamage::compute((*style).as_ref(), &*shared_style);
                    *style = Some(shared_style);
                    return StyleSharingResult::StyleWasShared(i, damage)
                }
                None => {}
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
            bf.insert(element.get_local_name());
            bf.insert(element.get_namespace());
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
            bf.remove(element.get_local_name());
            bf.remove(element.get_namespace());
            element.get_id().map(|id| bf.remove(&id));

            // TODO: case-sensitivity depends on the document type and quirks mode
            element.each_class(|class| bf.remove(class));
        }
    }

    unsafe fn cascade_node(&self,
                           context: &SharedStyleContext<<Self::ConcreteElement as Element>::Impl>,
                           parent: Option<Self>,
                           applicable_declarations: &ApplicableDeclarations<<Self::ConcreteElement as Element>::Impl>,
                           applicable_declarations_cache: &mut ApplicableDeclarationsCache,
                           new_animations_sender: &Mutex<Sender<Animation>>)
                           where <Self::ConcreteElement as Element>::Impl: SelectorImplExt {
        // Get our parent's style. This must be unsafe so that we don't touch the parent's
        // borrow flags.
        //
        // FIXME(pcwalton): Isolate this unsafety into the `wrapper` module to allow
        // enforced safe, race-free access to the parent style.
        let parent_style = match parent {
            None => None,
            Some(parent_node) => {
                let parent_style = (*parent_node.borrow_data_unchecked().unwrap()).style.as_ref().unwrap();
                Some(parent_style)
            }
        };

        let damage;
        if self.is_text_node() {
            // Text nodes get a copy of the parent style. This ensures
            // that during fragment construction any non-inherited
            // CSS properties (such as vertical-align) are correctly
            // set on the fragment(s).
            let mut data_ref = self.mutate_data().unwrap();
            let mut data = &mut *data_ref;
            let cloned_parent_style = parent_style.unwrap().clone();
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
                    applicable_declarations_cache,
                    new_animations_sender,
                    applicable_declarations.normal_shareable,
                    true);

                data.style = Some(final_style);

                <Self::ConcreteElement as Element>::Impl::each_eagerly_cascaded_pseudo_element(|pseudo| {
                    let applicable_declarations_for_this_pseudo =
                        applicable_declarations.per_pseudo.get(&pseudo).unwrap();


                    if !applicable_declarations_for_this_pseudo.is_empty() {
                        let (new_damage, style) = self.cascade_node_pseudo_element(
                            context,
                            Some(data.style.as_ref().unwrap()),
                            &*applicable_declarations_for_this_pseudo,
                            data.per_pseudo.get_mut(&pseudo),
                            applicable_declarations_cache,
                            new_animations_sender,
                            false,
                            false);
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
