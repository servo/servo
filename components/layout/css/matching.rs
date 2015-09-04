/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! High-level interface to CSS selector matching.

#![allow(unsafe_code)]

use animation;
use context::SharedLayoutContext;
use data::LayoutDataWrapper;
use incremental::{self, RestyleDamage};
use smallvec::SmallVec;
use wrapper::{LayoutElement, LayoutNode};

use script::dom::characterdata::CharacterDataTypeId;
use script::dom::node::NodeTypeId;
use script::layout_interface::Animation;
use selectors::bloom::BloomFilter;
use selectors::matching::{CommonStyleAffectingAttributeMode, CommonStyleAffectingAttributes};
use selectors::matching::{common_style_affecting_attributes, rare_style_affecting_attributes};
use selectors::parser::PseudoElement;
use selectors::{Element};
use std::borrow::ToOwned;
use std::hash::{Hash, Hasher};
use std::mem;
use std::slice::Iter;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use string_cache::{Atom, Namespace};
use style::node::TElementAttributes;
use style::properties::{ComputedValues, cascade};
use style::selector_matching::{Stylist, DeclarationBlock};
use util::arc_ptr_eq;
use util::cache::{LRUCache, SimpleHashCache};
use util::opts;
use util::vec::ForgetfulSink;

pub struct ApplicableDeclarations {
    pub normal: SmallVec<[DeclarationBlock; 16]>,
    pub before: Vec<DeclarationBlock>,
    pub after: Vec<DeclarationBlock>,

    /// Whether the `normal` declarations are shareable with other nodes.
    pub normal_shareable: bool,
}

impl ApplicableDeclarations {
    pub fn new() -> ApplicableDeclarations {
        ApplicableDeclarations {
            normal: SmallVec::new(),
            before: Vec::new(),
            after: Vec::new(),
            normal_shareable: false,
        }
    }

    pub fn clear(&mut self) {
        self.normal = SmallVec::new();
        self.before = Vec::new();
        self.after = Vec::new();
        self.normal_shareable = false;
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
            let ptr: usize = unsafe {
                mem::transmute_copy(declaration)
            };
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

    fn find(&self, declarations: &[DeclarationBlock]) -> Option<Arc<ComputedValues>> {
        match self.cache.find(&ApplicableDeclarationsCacheQuery::new(declarations)) {
            None => None,
            Some(ref values) => Some((*values).clone()),
        }
    }

    fn insert(&mut self, declarations: Vec<DeclarationBlock>, style: Arc<ComputedValues>) {
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

fn create_common_style_affecting_attributes_from_element(element: &LayoutElement)
                                                         -> CommonStyleAffectingAttributes {
    let mut flags = CommonStyleAffectingAttributes::empty();
    for attribute_info in &common_style_affecting_attributes() {
        match attribute_info.mode {
            CommonStyleAffectingAttributeMode::IsPresent(flag) => {
                if element.get_attr(&ns!(""), &attribute_info.atom).is_some() {
                    flags.insert(flag)
                }
            }
            CommonStyleAffectingAttributeMode::IsEqual(target_value, flag) => {
                match element.get_attr(&ns!(""), &attribute_info.atom) {
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
    fn new(element: &LayoutElement) -> Option<StyleSharingCandidate> {
        let parent_element = match element.parent_element() {
            None => return None,
            Some(parent_element) => parent_element,
        };

        let style = unsafe {
            match *element.as_node().borrow_layout_data_unchecked() {
                None => return None,
                Some(ref layout_data_ref) => {
                    match layout_data_ref.shared_data.style {
                        None => return None,
                        Some(ref data) => (*data).clone(),
                    }
                }
            }
        };
        let parent_style = unsafe {
            match *parent_element.as_node().borrow_layout_data_unchecked() {
                None => return None,
                Some(ref parent_layout_data_ref) => {
                    match parent_layout_data_ref.shared_data.style {
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
            class: element.get_attr(&ns!(""), &atom!("class"))
                          .map(|string| string.to_owned()),
            link: element.is_link(),
            namespace: (*element.get_namespace()).clone(),
            common_style_affecting_attributes:
                   create_common_style_affecting_attributes_from_element(&element)
        })
    }

    fn can_share_style_with(&self, element: &LayoutElement) -> bool {
        if *element.get_local_name() != self.local_name {
            return false
        }

        // FIXME(pcwalton): Use `each_class` here instead of slow string comparison.
        match (&self.class, element.get_attr(&ns!(""), &atom!("class"))) {
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
                            element.get_attr(&ns!(""), &attribute_info.atom).is_some() {
                        return false
                    }
                }
                CommonStyleAffectingAttributeMode::IsEqual(target_value, flag) => {
                    match element.get_attr(&ns!(""), &attribute_info.atom) {
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
            if element.get_attr(&ns!(""), attribute_name).is_some() {
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

    pub fn insert_if_possible(&mut self, element: &LayoutElement) {
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
pub enum StyleSharingResult {
    /// We didn't find anybody to share the style with. The boolean indicates whether the style
    /// is shareable at all.
    CannotShare(bool),
    /// The node's style can be shared. The integer specifies the index in the LRU cache that was
    /// hit and the damage that was done.
    StyleWasShared(usize, RestyleDamage),
}

pub trait MatchMethods {
    /// Inserts and removes the matching `Descendant` selectors from a bloom
    /// filter. This is used to speed up CSS selector matching to remove
    /// unnecessary tree climbs for `Descendant` queries.
    ///
    /// A bloom filter of the local names, namespaces, IDs, and classes is kept.
    /// Therefore, each node must have its matching selectors inserted _after_
    /// its own selector matching and _before_ its children start.
    fn insert_into_bloom_filter(&self, bf: &mut BloomFilter);

    /// After all the children are done css selector matching, this must be
    /// called to reset the bloom filter after an `insert`.
    fn remove_from_bloom_filter(&self, bf: &mut BloomFilter);

    fn match_node(&self,
                  stylist: &Stylist,
                  parent_bf: Option<&BloomFilter>,
                  applicable_declarations: &mut ApplicableDeclarations,
                  shareable: &mut bool);

    /// Attempts to share a style with another node. This method is unsafe because it depends on
    /// the `style_sharing_candidate_cache` having only live nodes in it, and we have no way to
    /// guarantee that at the type system level yet.
    unsafe fn share_style_if_possible(&self,
                                      style_sharing_candidate_cache:
                                        &mut StyleSharingCandidateCache,
                                      parent: Option<LayoutNode>)
                                      -> StyleSharingResult;

    unsafe fn cascade_node(&self,
                           layout_context: &SharedLayoutContext,
                           parent: Option<LayoutNode>,
                           applicable_declarations: &ApplicableDeclarations,
                           applicable_declarations_cache: &mut ApplicableDeclarationsCache,
                           new_animations_sender: &Sender<Animation>);
}

trait PrivateMatchMethods {
    fn cascade_node_pseudo_element(&self,
                                   layout_context: &SharedLayoutContext,
                                   parent_style: Option<&Arc<ComputedValues>>,
                                   applicable_declarations: &[DeclarationBlock],
                                   style: &mut Option<Arc<ComputedValues>>,
                                   applicable_declarations_cache:
                                    &mut ApplicableDeclarationsCache,
                                   new_animations_sender: &Sender<Animation>,
                                   shareable: bool,
                                   animate_properties: bool)
                                   -> RestyleDamage;

    fn share_style_with_candidate_if_possible(&self,
                                              parent_node: Option<LayoutNode>,
                                              candidate: &StyleSharingCandidate)
                                              -> Option<Arc<ComputedValues>>;
}

impl<'ln> PrivateMatchMethods for LayoutNode<'ln> {
    fn cascade_node_pseudo_element(&self,
                                   layout_context: &SharedLayoutContext,
                                   parent_style: Option<&Arc<ComputedValues>>,
                                   applicable_declarations: &[DeclarationBlock],
                                   style: &mut Option<Arc<ComputedValues>>,
                                   applicable_declarations_cache:
                                    &mut ApplicableDeclarationsCache,
                                   new_animations_sender: &Sender<Animation>,
                                   shareable: bool,
                                   animate_properties: bool)
                                   -> RestyleDamage {
        // Finish any transitions.
        if animate_properties {
            if let Some(ref mut style) = *style {
                let this_opaque = self.opaque();
                if let Some(ref animations) = layout_context.running_animations.get(&this_opaque) {
                    for animation in *animations {
                        animation.property_animation.update(&mut *Arc::make_mut(style), 1.0);
                    }
                }
            }
        }

        let mut this_style;
        let cacheable;
        match parent_style {
            Some(ref parent_style) => {
                let cache_entry = applicable_declarations_cache.find(applicable_declarations);
                let cached_computed_values = match cache_entry {
                    None => None,
                    Some(ref style) => Some(&**style),
                };
                let (the_style, is_cacheable) = cascade(layout_context.screen_size,
                                                        applicable_declarations,
                                                        shareable,
                                                        Some(&***parent_style),
                                                        cached_computed_values);
                cacheable = is_cacheable;
                this_style = the_style
            }
            None => {
                let (the_style, is_cacheable) = cascade(layout_context.screen_size,
                                                        applicable_declarations,
                                                        shareable,
                                                        None,
                                                        None);
                cacheable = is_cacheable;
                this_style = the_style
            }
        };

        // Trigger transitions if necessary. This will reset `this_style` back to its old value if
        // it did trigger a transition.
        if animate_properties {
            if let Some(ref style) = *style {
                animation::start_transitions_if_applicable(new_animations_sender,
                                                           self.opaque(),
                                                           &**style,
                                                           &mut this_style);
            }
        }

        // Calculate style difference.
        let this_style = Arc::new(this_style);
        let damage = incremental::compute_damage(style, &*this_style);

        // Cache the resolved style if it was cacheable.
        if cacheable {
            applicable_declarations_cache.insert(applicable_declarations.to_vec(),
                                                 this_style.clone());
        }

        // Write in the final style and return the damage done to our caller.
        *style = Some(this_style);
        damage
    }

    fn share_style_with_candidate_if_possible(&self,
                                              parent_node: Option<LayoutNode>,
                                              candidate: &StyleSharingCandidate)
                                              -> Option<Arc<ComputedValues>> {
        let element = self.as_element().unwrap();

        let parent_node = match parent_node {
            Some(ref parent_node) if parent_node.as_element().is_some() => parent_node,
            Some(_) | None => return None,
        };

        let parent_layout_data: &Option<LayoutDataWrapper> = unsafe {
            &*parent_node.borrow_layout_data_unchecked()
        };
        match *parent_layout_data {
            Some(ref parent_layout_data_ref) => {
                // Check parent style.
                let parent_style = parent_layout_data_ref.shared_data.style.as_ref().unwrap();
                if !arc_ptr_eq(parent_style, &candidate.parent_style) {
                    return None
                }

                // Check tag names, classes, etc.
                if !candidate.can_share_style_with(&element) {
                    return None
                }

                return Some(candidate.style.clone())
            }
            _ => {}
        }

        None
    }
}

impl<'ln> MatchMethods for LayoutNode<'ln> {
    fn match_node(&self,
                  stylist: &Stylist,
                  parent_bf: Option<&BloomFilter>,
                  applicable_declarations: &mut ApplicableDeclarations,
                  shareable: &mut bool) {
        let element = self.as_element().unwrap();
        let style_attribute = element.style_attribute().as_ref();

        applicable_declarations.normal_shareable =
            stylist.push_applicable_declarations(&element,
                                                 parent_bf,
                                                 style_attribute,
                                                 None,
                                                 &mut applicable_declarations.normal);
        stylist.push_applicable_declarations(&element,
                                             parent_bf,
                                             None,
                                             Some(PseudoElement::Before),
                                             &mut applicable_declarations.before);
        stylist.push_applicable_declarations(&element,
                                             parent_bf,
                                             None,
                                             Some(PseudoElement::After),
                                             &mut applicable_declarations.after);

        *shareable = applicable_declarations.normal_shareable &&
            applicable_declarations.before.is_empty() &&
            applicable_declarations.after.is_empty()
    }

    unsafe fn share_style_if_possible(&self,
                                      style_sharing_candidate_cache:
                                        &mut StyleSharingCandidateCache,
                                      parent: Option<LayoutNode>)
                                      -> StyleSharingResult {
        if opts::get().disable_share_style_cache {
            return StyleSharingResult::CannotShare(false)
        }
        let ok = {
            if let Some(element) = self.as_element() {
                element.style_attribute().is_none() &&
                    element.get_attr(&ns!(""), &atom!("id")).is_none()
            } else {
                false
            }
        };
        if !ok {
            return StyleSharingResult::CannotShare(false)
        }

        for (i, &(ref candidate, ())) in style_sharing_candidate_cache.iter().enumerate() {
            match self.share_style_with_candidate_if_possible(parent.clone(), candidate) {
                Some(shared_style) => {
                    // Yay, cache hit. Share the style.
                    let mut layout_data_ref = self.mutate_layout_data();
                    let shared_data = &mut layout_data_ref.as_mut().unwrap().shared_data;
                    let style = &mut shared_data.style;
                    let damage = incremental::compute_damage(style, &*shared_style);
                    *style = Some(shared_style);
                    return StyleSharingResult::StyleWasShared(i, damage)
                }
                None => {}
            }
        }

        StyleSharingResult::CannotShare(true)
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
                           layout_context: &SharedLayoutContext,
                           parent: Option<LayoutNode>,
                           applicable_declarations: &ApplicableDeclarations,
                           applicable_declarations_cache: &mut ApplicableDeclarationsCache,
                           new_animations_sender: &Sender<Animation>) {
        // Get our parent's style. This must be unsafe so that we don't touch the parent's
        // borrow flags.
        //
        // FIXME(pcwalton): Isolate this unsafety into the `wrapper` module to allow
        // enforced safe, race-free access to the parent style.
        let parent_style = match parent {
            None => None,
            Some(parent_node) => {
                let parent_layout_data_ref = parent_node.borrow_layout_data_unchecked();
                let parent_layout_data = (&*parent_layout_data_ref).as_ref()
                                                                   .expect("no parent data!?");
                let parent_style = parent_layout_data.shared_data
                                                     .style
                                                     .as_ref()
                                                     .expect("parent hasn't been styled yet!");
                Some(parent_style)
            }
        };

        let mut layout_data_ref = self.mutate_layout_data();
        match *layout_data_ref {
            None => panic!("no layout data"),
            Some(ref mut layout_data) => {
                match self.type_id() {
                    NodeTypeId::CharacterData(CharacterDataTypeId::Text) => {
                        // Text nodes get a copy of the parent style. This ensures
                        // that during fragment construction any non-inherited
                        // CSS properties (such as vertical-align) are correctly
                        // set on the fragment(s).
                        let cloned_parent_style = parent_style.unwrap().clone();
                        layout_data.shared_data.style = Some(cloned_parent_style);
                    }
                    _ => {
                        let mut damage = self.cascade_node_pseudo_element(
                            layout_context,
                            parent_style,
                            &applicable_declarations.normal,
                            &mut layout_data.shared_data.style,
                            applicable_declarations_cache,
                            new_animations_sender,
                            applicable_declarations.normal_shareable,
                            true);
                        if applicable_declarations.before.len() > 0 {
                            damage = damage | self.cascade_node_pseudo_element(
                                layout_context,
                                Some(layout_data.shared_data.style.as_ref().unwrap()),
                                &*applicable_declarations.before,
                                &mut layout_data.data.before_style,
                                applicable_declarations_cache,
                                new_animations_sender,
                                false,
                                false);
                        }
                        if applicable_declarations.after.len() > 0 {
                            damage = damage | self.cascade_node_pseudo_element(
                                layout_context,
                                Some(layout_data.shared_data.style.as_ref().unwrap()),
                                &*applicable_declarations.after,
                                &mut layout_data.data.after_style,
                                applicable_declarations_cache,
                                new_animations_sender,
                                false,
                                false);
                        }
                        layout_data.data.restyle_damage = damage;
                    }
                }
            }
        }
    }
}
