/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::{TElement, TNode};
use properties::{ComputedValues, PropertyDeclaration};
use selector_matching::DeclarationBlock;
use selectors::matching::{CommonStyleAffectingAttributeMode, CommonStyleAffectingAttributes};
use selectors::matching::{common_style_affecting_attributes, rare_style_affecting_attributes};
use smallvec::SmallVec;
use std::hash::{Hash, Hasher};
use std::slice::Iter;
use std::sync::Arc;
use string_cache::{Atom, Namespace};
use util::arc_ptr_eq;
use util::cache::{LRUCache, SimpleHashCache};
use util::vec::ForgetfulSink;

/// Pieces of layout/css/matching.rs, which will eventually be merged
/// into this file.

fn create_common_style_affecting_attributes_from_element<'le, E: TElement<'le>>(element: &E)
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
    fn new<'le, E: TElement<'le>>(element: &E) -> Option<StyleSharingCandidate> {
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
                   create_common_style_affecting_attributes_from_element::<'le, E>(&element)
        })
    }

    pub fn can_share_style_with<'a, E: TElement<'a>>(&self, element: &E) -> bool {
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

    pub fn insert_if_possible<'le, E: TElement<'le>>(&mut self, element: &E) {
        match StyleSharingCandidate::new(element) {
            None => {}
            Some(candidate) => self.cache.insert(candidate, ())
        }
    }

    pub fn touch(&mut self, index: usize) {
        self.cache.touch(index)
    }
}


