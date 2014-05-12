/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// High-level interface to CSS selector matching.

use css::node_style::StyledNode;
use layout::construct::FlowConstructor;
use layout::context::LayoutContext;
use layout::extra::LayoutAuxMethods;
use layout::util::{LayoutDataAccess, LayoutDataWrapper};
use layout::wrapper::{LayoutElement, LayoutNode, PostorderNodeMutTraversal, ThreadSafeLayoutNode};

use gfx::font_context::FontContext;
use servo_util::cache::{Cache, LRUCache, SimpleHashCache};
use servo_util::namespace::Null;
use servo_util::smallvec::{SmallVec, SmallVec0, SmallVec16};
use servo_util::str::DOMString;
use std::cast;
use std::hash::{Hash, sip};
use std::slice::Items;
use style::{After, Before, ComputedValues, MatchedProperty, Stylist, TElement, TNode, cascade};
use sync::Arc;

pub struct ApplicableDeclarations {
    pub normal: SmallVec16<MatchedProperty>,
    pub before: SmallVec0<MatchedProperty>,
    pub after: SmallVec0<MatchedProperty>,

    /// Whether the `normal` declarations are shareable with other nodes.
    pub normal_shareable: bool,
}

impl ApplicableDeclarations {
    pub fn new() -> ApplicableDeclarations {
        ApplicableDeclarations {
            normal: SmallVec16::new(),
            before: SmallVec0::new(),
            after: SmallVec0::new(),
            normal_shareable: false,
        }
    }

    pub fn clear(&mut self) {
        self.normal = SmallVec16::new();
        self.before = SmallVec0::new();
        self.after = SmallVec0::new();
        self.normal_shareable = false;
    }
}

#[deriving(Clone)]
pub struct ApplicableDeclarationsCacheEntry {
    pub declarations: SmallVec16<MatchedProperty>,
}

impl ApplicableDeclarationsCacheEntry {
    fn new(slice: &[MatchedProperty]) -> ApplicableDeclarationsCacheEntry {
        let mut entry_declarations = SmallVec16::new();
        for declarations in slice.iter() {
            entry_declarations.push(declarations.clone());
        }
        ApplicableDeclarationsCacheEntry {
            declarations: entry_declarations,
        }
    }
}

impl Eq for ApplicableDeclarationsCacheEntry {
    fn eq(&self, other: &ApplicableDeclarationsCacheEntry) -> bool {
        let this_as_query = ApplicableDeclarationsCacheQuery::new(self.declarations.as_slice());
        this_as_query.equiv(other)
    }
}

impl Hash for ApplicableDeclarationsCacheEntry {
    fn hash(&self, state: &mut sip::SipState) {
        let tmp = ApplicableDeclarationsCacheQuery::new(self.declarations.as_slice());
        tmp.hash(state);
    }
}

struct ApplicableDeclarationsCacheQuery<'a> {
    declarations: &'a [MatchedProperty],
}

impl<'a> ApplicableDeclarationsCacheQuery<'a> {
    fn new(declarations: &'a [MatchedProperty]) -> ApplicableDeclarationsCacheQuery<'a> {
        ApplicableDeclarationsCacheQuery {
            declarations: declarations,
        }
    }
}

// Workaround for lack of `ptr_eq` on Arcs...
#[inline]
fn arc_ptr_eq<T>(a: &Arc<T>, b: &Arc<T>) -> bool {
    unsafe {
        let a: uint = cast::transmute_copy(a);
        let b: uint = cast::transmute_copy(b);
        a == b
    }
}

impl<'a> Equiv<ApplicableDeclarationsCacheEntry> for ApplicableDeclarationsCacheQuery<'a> {
    fn equiv(&self, other: &ApplicableDeclarationsCacheEntry) -> bool {
        if self.declarations.len() != other.declarations.len() {
            return false
        }
        for (this, other) in self.declarations.iter().zip(other.declarations.iter()) {
            if !arc_ptr_eq(&this.declarations, &other.declarations) {
                return false
            }
        }
        return true
    }
}


impl<'a> Hash for ApplicableDeclarationsCacheQuery<'a> {
    fn hash(&self, state: &mut sip::SipState) {
        for declaration in self.declarations.iter() {
            let ptr: uint = unsafe {
                cast::transmute_copy(declaration)
            };
            ptr.hash(state);
        }
    }
}

static APPLICABLE_DECLARATIONS_CACHE_SIZE: uint = 32;

pub struct ApplicableDeclarationsCache {
    pub cache: SimpleHashCache<ApplicableDeclarationsCacheEntry,Arc<ComputedValues>>,
}

impl ApplicableDeclarationsCache {
    pub fn new() -> ApplicableDeclarationsCache {
        ApplicableDeclarationsCache {
            cache: SimpleHashCache::new(APPLICABLE_DECLARATIONS_CACHE_SIZE),
        }
    }

    fn find(&self, declarations: &[MatchedProperty]) -> Option<Arc<ComputedValues>> {
        match self.cache.find_equiv(&ApplicableDeclarationsCacheQuery::new(declarations)) {
            None => None,
            Some(ref values) => Some((*values).clone()),
        }
    }

    fn insert(&mut self, declarations: &[MatchedProperty], style: Arc<ComputedValues>) {
        drop(self.cache.insert(ApplicableDeclarationsCacheEntry::new(declarations), style))
    }
}

/// An LRU cache of the last few nodes seen, so that we can aggressively try to reuse their styles.
pub struct StyleSharingCandidateCache {
    cache: LRUCache<StyleSharingCandidate,()>,
}

#[deriving(Clone)]
pub struct StyleSharingCandidate {
    pub style: Arc<ComputedValues>,
    pub parent_style: Arc<ComputedValues>,

    // TODO(pcwalton): Intern.
    pub local_name: DOMString,

    pub class: Option<DOMString>,
}

impl Eq for StyleSharingCandidate {
    fn eq(&self, other: &StyleSharingCandidate) -> bool {
        arc_ptr_eq(&self.style, &other.style) &&
            arc_ptr_eq(&self.parent_style, &other.parent_style) &&
            self.local_name == other.local_name &&
            self.class == other.class
    }
}

impl StyleSharingCandidate {
    /// Attempts to create a style sharing candidate from this node. Returns
    /// the style sharing candidate or `None` if this node is ineligible for
    /// style sharing.
    fn new(node: &LayoutNode) -> Option<StyleSharingCandidate> {
        let parent_node = match node.parent_node() {
            None => return None,
            Some(parent_node) => parent_node,
        };
        if !parent_node.is_element() {
            return None
        }

        let style = unsafe {
            match *node.borrow_layout_data_unchecked() {
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
            match *parent_node.borrow_layout_data_unchecked() {
                None => return None,
                Some(ref parent_layout_data_ref) => {
                    match parent_layout_data_ref.shared_data.style {
                        None => return None,
                        Some(ref data) => (*data).clone(),
                    }
                }
            }
        };

        let mut style = Some(style);
        let mut parent_style = Some(parent_style);
        let element = node.as_element();
        if element.style_attribute().is_some() {
            return None
        }

        Some(StyleSharingCandidate {
            style: style.take_unwrap(),
            parent_style: parent_style.take_unwrap(),
            local_name: element.get_local_name().to_str(),
            class: element.get_attr(&Null, "class")
                          .map(|string| string.to_str()),
        })
    }

    fn can_share_style_with(&self, element: &LayoutElement) -> bool {
        if element.get_local_name() != self.local_name {
            return false
        }
        match (&self.class, element.get_attr(&Null, "class")) {
            (&None, Some(_)) | (&Some(_), None) => return false,
            (&Some(ref this_class), Some(element_class)) if element_class != *this_class => {
                return false
            }
            (&Some(_), Some(_)) | (&None, None) => {}
        }
        true
    }
}

static STYLE_SHARING_CANDIDATE_CACHE_SIZE: uint = 40;

impl StyleSharingCandidateCache {
    pub fn new() -> StyleSharingCandidateCache {
        StyleSharingCandidateCache {
            cache: LRUCache::new(STYLE_SHARING_CANDIDATE_CACHE_SIZE),
        }
    }

    pub fn iter<'a>(&'a self) -> Items<'a,(StyleSharingCandidate,())> {
        self.cache.iter()
    }

    pub fn insert_if_possible(&mut self, node: &LayoutNode) {
        match StyleSharingCandidate::new(node) {
            None => {}
            Some(candidate) => self.cache.insert(candidate, ())
        }
    }

    pub fn touch(&mut self, index: uint) {
        self.cache.touch(index)
    }
}

/// The results of attempting to share a style.
pub enum StyleSharingResult<'ln> {
    /// We didn't find anybody to share the style with. The boolean indicates whether the style
    /// is shareable at all.
    CannotShare(bool),
    /// The node's style can be shared. The integer specifies the index in the LRU cache that was
    /// hit.
    StyleWasShared(uint),
}

pub trait MatchMethods {
    /// Performs aux initialization, selector matching, cascading, and flow construction
    /// sequentially.
    fn recalc_style_for_subtree(&self,
                                stylist: &Stylist,
                                layout_context: &mut LayoutContext,
                                mut font_context: ~FontContext,
                                applicable_declarations: &mut ApplicableDeclarations,
                                applicable_declarations_cache: &mut ApplicableDeclarationsCache,
                                style_sharing_candidate_cache: &mut StyleSharingCandidateCache,
                                parent: Option<LayoutNode>)
                                -> ~FontContext;

    fn match_node(&self,
                  stylist: &Stylist,
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
                           parent: Option<LayoutNode>,
                           initial_values: &ComputedValues,
                           applicable_declarations: &ApplicableDeclarations,
                           applicable_declarations_cache: &mut ApplicableDeclarationsCache);
}

trait PrivateMatchMethods {
    fn cascade_node_pseudo_element(&self,
                                   parent_style: Option<&Arc<ComputedValues>>,
                                   applicable_declarations: &[MatchedProperty],
                                   style: &mut Option<Arc<ComputedValues>>,
                                   initial_values: &ComputedValues,
                                   applicable_declarations_cache: &mut
                                   ApplicableDeclarationsCache,
                                   shareable: bool);

    fn share_style_with_candidate_if_possible(&self,
                                              parent_node: Option<LayoutNode>,
                                              candidate: &StyleSharingCandidate)
                                              -> Option<Arc<ComputedValues>>;
}

impl<'ln> PrivateMatchMethods for LayoutNode<'ln> {
    fn cascade_node_pseudo_element(&self,
                                   parent_style: Option<&Arc<ComputedValues>>,
                                   applicable_declarations: &[MatchedProperty],
                                   style: &mut Option<Arc<ComputedValues>>,
                                   initial_values: &ComputedValues,
                                   applicable_declarations_cache: &mut
                                   ApplicableDeclarationsCache,
                                   shareable: bool) {
        let this_style;
        let cacheable;
        match parent_style {
            Some(ref parent_style) => {
                let cached_computed_values;
                let cache_entry = applicable_declarations_cache.find(applicable_declarations);
                match cache_entry {
                    None => cached_computed_values = None,
                    Some(ref style) => cached_computed_values = Some(&**style),
                }
                let (the_style, is_cacheable) = cascade(applicable_declarations,
                                                        shareable,
                                                        Some(&***parent_style),
                                                        initial_values,
                                                        cached_computed_values);
                cacheable = is_cacheable;
                this_style = Arc::new(the_style);
            }
            None => {
                let (the_style, is_cacheable) = cascade(applicable_declarations,
                                                        shareable,
                                                        None,
                                                        initial_values,
                                                        None);
                cacheable = is_cacheable;
                this_style = Arc::new(the_style);
            }
        };

        // Cache the resolved style if it was cacheable.
        if cacheable {
            applicable_declarations_cache.insert(applicable_declarations, this_style.clone());
        }

        *style = Some(this_style);
    }


    fn share_style_with_candidate_if_possible(&self,
                                              parent_node: Option<LayoutNode>,
                                              candidate: &StyleSharingCandidate)
                                              -> Option<Arc<ComputedValues>> {
        assert!(self.is_element());

        let parent_node = match parent_node {
            Some(ref parent_node) if parent_node.is_element() => parent_node,
            Some(_) | None => return None,
        };

        let parent_layout_data: &Option<LayoutDataWrapper> = unsafe {
            cast::transmute(parent_node.borrow_layout_data_unchecked())
        };
        match parent_layout_data {
            &Some(ref parent_layout_data_ref) => {
                // Check parent style.
                let parent_style = parent_layout_data_ref.shared_data.style.as_ref().unwrap();
                if !arc_ptr_eq(parent_style, &candidate.parent_style) {
                    return None
                }

                // Check tag names, classes, etc.
                if !candidate.can_share_style_with(&self.as_element()) {
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
                  applicable_declarations: &mut ApplicableDeclarations,
                  shareable: &mut bool) {
        let style_attribute = self.as_element().style_attribute().as_ref();

        applicable_declarations.normal_shareable =
            stylist.push_applicable_declarations(self,
                                                 style_attribute,
                                                 None,
                                                 &mut applicable_declarations.normal);
        stylist.push_applicable_declarations(self,
                                             None,
                                             Some(Before),
                                             &mut applicable_declarations.before);
        stylist.push_applicable_declarations(self,
                                             None,
                                             Some(After),
                                             &mut applicable_declarations.after);

        *shareable = applicable_declarations.normal_shareable
    }

    unsafe fn share_style_if_possible(&self,
                                      style_sharing_candidate_cache:
                                        &mut StyleSharingCandidateCache,
                                      parent: Option<LayoutNode>)
                                      -> StyleSharingResult {
        if !self.is_element() {
            return CannotShare(false)
        }
        let ok = {
            let element = self.as_element();
            element.style_attribute().is_none() && element.get_attr(&Null, "id").is_none()
        };
        if !ok {
            return CannotShare(false)
        }

        for (i, &(ref candidate, ())) in style_sharing_candidate_cache.iter().enumerate() {
            match self.share_style_with_candidate_if_possible(parent.clone(), candidate) {
                Some(shared_style) => {
                    // Yay, cache hit. Share the style.
                    let mut layout_data_ref = self.mutate_layout_data();
                    layout_data_ref.get_mut_ref().shared_data.style = Some(shared_style);
                    return StyleWasShared(i)
                }
                None => {}
            }
        }

        CannotShare(true)
    }

    fn recalc_style_for_subtree(&self,
                                stylist: &Stylist,
                                layout_context: &mut LayoutContext,
                                mut font_context: ~FontContext,
                                applicable_declarations: &mut ApplicableDeclarations,
                                applicable_declarations_cache: &mut ApplicableDeclarationsCache,
                                style_sharing_candidate_cache: &mut StyleSharingCandidateCache,
                                parent: Option<LayoutNode>)
                                -> ~FontContext {
        self.initialize_layout_data(layout_context.layout_chan.clone());

        // First, check to see whether we can share a style with someone.
        let sharing_result = unsafe {
            self.share_style_if_possible(style_sharing_candidate_cache, parent.clone())
        };

        // Otherwise, match and cascade selectors.
        match sharing_result {
            CannotShare(mut shareable) => {
                if self.is_element() {
                    self.match_node(stylist, applicable_declarations, &mut shareable)
                }

                unsafe {
                    let initial_values = &*layout_context.initial_css_values;
                    self.cascade_node(parent,
                                      initial_values,
                                      applicable_declarations,
                                      applicable_declarations_cache)
                }

                applicable_declarations.clear();

                // Add ourselves to the LRU cache.
                if shareable {
                    style_sharing_candidate_cache.insert_if_possible(self)
                }
            }
            StyleWasShared(index) => style_sharing_candidate_cache.touch(index),
        }

        for kid in self.children() {
            font_context = kid.recalc_style_for_subtree(stylist,
                                                        layout_context,
                                                        font_context,
                                                        applicable_declarations,
                                                        applicable_declarations_cache,
                                                        style_sharing_candidate_cache,
                                                        Some(self.clone()))
        }

        // Construct flows.
        let layout_node = ThreadSafeLayoutNode::new(self);
        let mut flow_constructor = FlowConstructor::new(layout_context, Some(font_context));
        flow_constructor.process(&layout_node);
        flow_constructor.unwrap_font_context().unwrap()
    }

    unsafe fn cascade_node(&self,
                           parent: Option<LayoutNode>,
                           initial_values: &ComputedValues,
                           applicable_declarations: &ApplicableDeclarations,
                           applicable_declarations_cache: &mut ApplicableDeclarationsCache) {
        // Get our parent's style. This must be unsafe so that we don't touch the parent's
        // borrow flags.
        //
        // FIXME(pcwalton): Isolate this unsafety into the `wrapper` module to allow
        // enforced safe, race-free access to the parent style.
        let parent_style = match parent {
            None => None,
            Some(parent_node) => {
                let parent_layout_data = parent_node.borrow_layout_data_unchecked();
                match *parent_layout_data {
                    None => fail!("no parent data?!"),
                    Some(ref parent_layout_data) => {
                        match parent_layout_data.shared_data.style {
                            None => fail!("parent hasn't been styled yet?!"),
                            Some(ref style) => Some(style),
                        }
                    }
                }
            }
        };

        let mut layout_data_ref = self.mutate_layout_data();
        match &mut *layout_data_ref {
            &None => fail!("no layout data"),
            &Some(ref mut layout_data) => {
                self.cascade_node_pseudo_element(parent_style,
                                                 applicable_declarations.normal.as_slice(),
                                                 &mut layout_data.shared_data.style,
                                                 initial_values,
                                                 applicable_declarations_cache,
                                                 applicable_declarations.normal_shareable);
                if applicable_declarations.before.len() > 0 {
                    self.cascade_node_pseudo_element(parent_style,
                                                     applicable_declarations.before.as_slice(),
                                                     &mut layout_data.data.before_style,
                                                     initial_values,
                                                     applicable_declarations_cache,
                                                     false);
                }
                if applicable_declarations.after.len() > 0 {
                    self.cascade_node_pseudo_element(parent_style,
                                                     applicable_declarations.after.as_slice(),
                                                     &mut layout_data.data.after_style,
                                                     initial_values,
                                                     applicable_declarations_cache,
                                                     false);
                }
            }
        }
    }
}

