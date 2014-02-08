/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// High-level interface to CSS selector matching.

use css::node_style::StyledNode;
use layout::extra::LayoutAuxMethods;
use layout::util::LayoutDataAccess;
use layout::wrapper::LayoutNode;

use extra::arc::Arc;
use script::layout_interface::LayoutChan;
use servo_util::cache::{Cache, LRUCache, SimpleHashCache};
use servo_util::namespace::Null;
use servo_util::smallvec::{SmallVec, SmallVec0, SmallVec16};
use std::cast;
use std::to_bytes;
use style::{After, Before, ComputedValues, MatchedProperty, Stylist, TNode, cascade};

pub struct ApplicableDeclarations {
    normal: SmallVec16<MatchedProperty>,
    before: SmallVec0<MatchedProperty>,
    after: SmallVec0<MatchedProperty>,
}

impl ApplicableDeclarations {
    pub fn new() -> ApplicableDeclarations {
        ApplicableDeclarations {
            normal: SmallVec16::new(),
            before: SmallVec0::new(),
            after: SmallVec0::new(),
        }
    }

    pub fn clear(&mut self) {
        self.normal = SmallVec16::new();
        self.before = SmallVec0::new();
        self.after = SmallVec0::new();
    }
}

#[deriving(Clone)]
struct ApplicableDeclarationsCacheEntry {
    declarations: SmallVec16<MatchedProperty>,
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

impl IterBytes for ApplicableDeclarationsCacheEntry {
    fn iter_bytes(&self, lsb0: bool, f: to_bytes::Cb) -> bool {
        ApplicableDeclarationsCacheQuery::new(self.declarations.as_slice()).iter_bytes(lsb0, f)
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

impl<'a> Equiv<ApplicableDeclarationsCacheEntry> for ApplicableDeclarationsCacheQuery<'a> {
    fn equiv(&self, other: &ApplicableDeclarationsCacheEntry) -> bool {
        if self.declarations.len() != other.declarations.len() {
            return false
        }
        for (this, other) in self.declarations.iter().zip(other.declarations.iter()) {
            unsafe {
                // Workaround for lack of `ptr_eq` on Arcs...
                let this: uint = cast::transmute_copy(this);
                let other: uint = cast::transmute_copy(other);
                if this != other {
                    return false
                }
            }
        }
        return true
    }
}

impl<'a> IterBytes for ApplicableDeclarationsCacheQuery<'a> {
    fn iter_bytes(&self, lsb0: bool, f: to_bytes::Cb) -> bool {
        let mut result = true;
        for declaration in self.declarations.iter() {
            let ptr: uint = unsafe {
                cast::transmute_copy(declaration)
            };
            result = ptr.iter_bytes(lsb0, |x| f(x));
        }
        result
    }
}

static APPLICABLE_DECLARATIONS_CACHE_SIZE: uint = 32;

pub struct ApplicableDeclarationsCache {
    cache: SimpleHashCache<ApplicableDeclarationsCacheEntry,Arc<ComputedValues>>,
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

pub trait MatchMethods {
    /// Performs aux initialization, selector matching, and cascading sequentially.
    fn match_and_cascade_subtree(&self,
                                 stylist: &Stylist,
                                 layout_chan: &LayoutChan,
                                 applicable_declarations: &mut ApplicableDeclarations,
                                 initial_values: &ComputedValues,
                                 applicable_declarations_cache: &mut ApplicableDeclarationsCache,
                                 parent: Option<LayoutNode>);

    fn match_node(&self, stylist: &Stylist, applicable_declarations: &mut ApplicableDeclarations);

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
                                    ApplicableDeclarationsCache);
}

impl<'ln> PrivateMatchMethods for LayoutNode<'ln> {
    fn cascade_node_pseudo_element(&self,
                                   parent_style: Option<&Arc<ComputedValues>>,
                                   applicable_declarations: &[MatchedProperty],
                                   style: &mut Option<Arc<ComputedValues>>,
                                   initial_values: &ComputedValues,
                                   applicable_declarations_cache: &mut
                                    ApplicableDeclarationsCache) {
        let this_style;
        let cacheable;
        match parent_style {
            Some(ref parent_style) => {
                let cached_computed_values;
                let cache_entry = applicable_declarations_cache.find(applicable_declarations);
                match cache_entry {
                    None => cached_computed_values = None,
                    Some(ref style) => cached_computed_values = Some(style.get()),
                }
                let (the_style, is_cacheable) = cascade(applicable_declarations,
                                                        Some(parent_style.get()),
                                                        initial_values,
                                                        cached_computed_values);
                cacheable = is_cacheable;
                this_style = Arc::new(the_style);
            }
            None => {
                let (the_style, is_cacheable) = cascade(applicable_declarations,
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
}

impl<'ln> MatchMethods for LayoutNode<'ln> {
    fn match_node(&self,
                  stylist: &Stylist,
                  applicable_declarations: &mut ApplicableDeclarations) {
        let style_attribute = self.with_element(|element| {
            match *element.style_attribute() {
                None => None,
                Some(ref style_attribute) => Some(style_attribute)
            }
        });

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
    }

    fn match_and_cascade_subtree(&self,
                                 stylist: &Stylist,
                                 layout_chan: &LayoutChan,
                                 applicable_declarations: &mut ApplicableDeclarations,
                                 initial_values: &ComputedValues,
                                 applicable_declarations_cache: &mut ApplicableDeclarationsCache,
                                 parent: Option<LayoutNode>) {
        self.initialize_layout_data((*layout_chan).clone());

        if self.is_element() {
            self.match_node(stylist, applicable_declarations);
        }

        unsafe {
            self.cascade_node(parent,
                              initial_values,
                              applicable_declarations,
                              applicable_declarations_cache)
        }

        applicable_declarations.clear();

        for kid in self.children() {
            kid.match_and_cascade_subtree(stylist,
                                          layout_chan,
                                          applicable_declarations,
                                          initial_values,
                                          applicable_declarations_cache,
                                          Some(*self))
        }
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
                        match parent_layout_data.data.style {
                            None => fail!("parent hasn't been styled yet?!"),
                            Some(ref style) => Some(style),
                        }
                    }
                }
            }
        };

        let mut layout_data_ref = self.mutate_layout_data();
        match *layout_data_ref.get() {
            None => fail!("no layout data"),
            Some(ref mut layout_data) => {
                self.cascade_node_pseudo_element(parent_style,
                                                 applicable_declarations.normal.as_slice(),
                                                 &mut layout_data.data.style,
                                                 initial_values,
                                                 applicable_declarations_cache);
                if applicable_declarations.before.len() > 0 {
                    self.cascade_node_pseudo_element(parent_style,
                                                     applicable_declarations.before.as_slice(),
                                                     &mut layout_data.data.before_style,
                                                     initial_values,
                                                     applicable_declarations_cache);
                }
                if applicable_declarations.after.len() > 0 {
                    self.cascade_node_pseudo_element(parent_style,
                                                     applicable_declarations.after.as_slice(),
                                                     &mut layout_data.data.after_style,
                                                     initial_values,
                                                     applicable_declarations_cache);
                }
            }
        }
    }
}

