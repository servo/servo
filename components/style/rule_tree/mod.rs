/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]
#![deny(missing_docs)]

//! The rule tree.

use arc_ptr_eq;
#[cfg(feature = "servo")]
use heapsize::HeapSizeOf;
use owning_handle::OwningHandle;
use parking_lot::{RwLock, RwLockReadGuard};
use properties::{Importance, PropertyDeclarationBlock};
use std::io::{self, Write};
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use stylesheets::StyleRule;
use thread_state;

/// The rule tree, the structure servo uses to preserve the results of selector
/// matching.
///
/// This is organized as a tree of rules. When a node matches a set of rules,
/// they're inserted in order in the tree, starting with the less specific one.
///
/// When a rule is inserted in the tree, other elements may share the path up to
/// a given rule. If that's the case, we don't duplicate child nodes, but share
/// them.
///
/// When the rule node refcount drops to zero, it doesn't get freed. It gets
/// instead put into a free list, and it is potentially GC'd after a while in a
/// single-threaded fashion.
///
/// That way, a rule node that represents a likely-to-match-again rule (like a
/// :hover rule) can be reused if we haven't GC'd it yet.
#[derive(Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct RuleTree {
    root: StrongRuleNode,
}

/// A style source for the rule node. It can either be a CSS style rule or a
/// declaration block.
///
/// Note that, even though the declaration block from inside the style rule
/// could be enough to implement the rule tree, keeping the whole rule provides
/// more debuggability, and also the ability of show those selectors to
/// devtools.
#[derive(Debug, Clone)]
pub enum StyleSource {
    /// A style rule stable pointer.
    Style(Arc<RwLock<StyleRule>>),
    /// A declaration block stable pointer.
    Declarations(Arc<RwLock<PropertyDeclarationBlock>>),
}

type StyleSourceGuardHandle<'a> =
    OwningHandle<
        RwLockReadGuard<'a, StyleRule>,
        RwLockReadGuard<'a, PropertyDeclarationBlock>>;

/// A guard for a given style source.
pub enum StyleSourceGuard<'a> {
    /// A guard for a style rule.
    Style(StyleSourceGuardHandle<'a>),
    /// A guard for a declaration block.
    Declarations(RwLockReadGuard<'a, PropertyDeclarationBlock>),
}

impl<'a> ::std::ops::Deref for StyleSourceGuard<'a> {
    type Target = PropertyDeclarationBlock;

    fn deref(&self) -> &Self::Target {
        match *self {
            StyleSourceGuard::Declarations(ref block) => &*block,
            StyleSourceGuard::Style(ref handle) => &*handle,
        }
    }
}

impl StyleSource {
    #[inline]
    fn ptr_equals(&self, other: &Self) -> bool {
        use self::StyleSource::*;
        match (self, other) {
            (&Style(ref one), &Style(ref other)) => arc_ptr_eq(one, other),
            (&Declarations(ref one), &Declarations(ref other)) => arc_ptr_eq(one, other),
            _ => false,
        }
    }

    fn dump<W: Write>(&self, writer: &mut W) {
        use self::StyleSource::*;

        if let Style(ref rule) = *self {
            let _ = write!(writer, "{:?}", rule.read().selectors);
        }

        let _ = write!(writer, "  -> {:?}", self.read().declarations);
    }

    /// Read the style source guard, and obtain thus read access to the
    /// underlying property declaration block.
    #[inline]
    pub fn read<'a>(&'a self) -> StyleSourceGuard<'a> {
        use self::StyleSource::*;
        match *self {
            Style(ref rule) => {
                let owning_ref = OwningHandle::new(rule.read(), |r| unsafe { &*r }.block.read());
                StyleSourceGuard::Style(owning_ref)
            }
            Declarations(ref block) => StyleSourceGuard::Declarations(block.read()),
        }
    }
}

/// This value exists here so a node that pushes itself to the list can know
/// that is in the free list by looking at is next pointer, and comparing it
/// with null.
///
/// The root node doesn't have a null pointer in the free list, but this value.
const FREE_LIST_SENTINEL: *mut RuleNode = 0x01 as *mut RuleNode;

impl RuleTree {
    /// Construct a new rule tree.
    pub fn new() -> Self {
        RuleTree {
            root: StrongRuleNode::new(Box::new(RuleNode::root())),
        }
    }

    /// Get the root rule node.
    pub fn root(&self) -> StrongRuleNode {
        self.root.clone()
    }

    fn dump<W: Write>(&self, writer: &mut W) {
        let _ = writeln!(writer, " + RuleTree");
        self.root.get().dump(writer, 0);
    }

    /// Dump the rule tree to stdout.
    pub fn dump_stdout(&self) {
        let mut stdout = io::stdout();
        self.dump(&mut stdout);
    }

    /// Insert the given rules, that must be in proper order by specifity, and
    /// return the corresponding rule node representing the last inserted one.
    pub fn insert_ordered_rules<'a, I>(&self, iter: I) -> StrongRuleNode
        where I: Iterator<Item=(StyleSource, Importance)>,
    {
        let mut current = self.root.clone();
        for (source, importance) in iter {
            current = current.ensure_child(self.root.downgrade(), source, importance);
        }
        current
    }

    /// This can only be called when no other threads is accessing this tree.
    pub unsafe fn gc(&self) {
        self.root.gc();
    }

    /// This can only be called when no other threads is accessing this tree.
    pub unsafe fn maybe_gc(&self) {
        self.root.maybe_gc();
    }
}

/// The number of RuleNodes added to the free list before we will consider
/// doing a GC when calling maybe_gc().  (The value is copied from Gecko,
/// where it likely did not result from a rigorous performance analysis.)
const RULE_TREE_GC_INTERVAL: usize = 300;

struct RuleNode {
    /// The root node. Only the root has no root pointer, for obvious reasons.
    root: Option<WeakRuleNode>,

    /// The parent rule node. Only the root has no parent.
    parent: Option<StrongRuleNode>,

    /// The actual style source, either coming from a selector in a StyleRule,
    /// or a raw property declaration block (like the style attribute).
    source: Option<StyleSource>,

    /// The importance of the declarations relevant in the style rule,
    /// meaningless in the root node.
    importance: Importance,

    refcount: AtomicUsize,
    first_child: AtomicPtr<RuleNode>,
    next_sibling: AtomicPtr<RuleNode>,
    prev_sibling: AtomicPtr<RuleNode>,

    /// The next item in the rule tree free list, that starts on the root node.
    next_free: AtomicPtr<RuleNode>,

    /// Number of RuleNodes we have added to the free list since the last GC.
    /// (We don't update this if we rescue a RuleNode from the free list.  It's
    /// just used as a heuristic to decide when to run GC.)
    ///
    /// Only used on the root RuleNode.  (We could probably re-use one of the
    /// sibling pointers to save space.)
    free_count: AtomicUsize,
}

unsafe impl Sync for RuleTree {}
unsafe impl Send for RuleTree {}

impl RuleNode {
    fn new(root: WeakRuleNode,
           parent: StrongRuleNode,
           source: StyleSource,
           importance: Importance) -> Self {
        debug_assert!(root.upgrade().parent().is_none());
        RuleNode {
            root: Some(root),
            parent: Some(parent),
            source: Some(source),
            importance: importance,
            refcount: AtomicUsize::new(1),
            first_child: AtomicPtr::new(ptr::null_mut()),
            next_sibling: AtomicPtr::new(ptr::null_mut()),
            prev_sibling: AtomicPtr::new(ptr::null_mut()),
            next_free: AtomicPtr::new(ptr::null_mut()),
            free_count: AtomicUsize::new(0),
        }
    }

    fn root() -> Self {
        RuleNode {
            root: None,
            parent: None,
            source: None,
            importance: Importance::Normal,
            refcount: AtomicUsize::new(1),
            first_child: AtomicPtr::new(ptr::null_mut()),
            next_sibling: AtomicPtr::new(ptr::null_mut()),
            prev_sibling: AtomicPtr::new(ptr::null_mut()),
            next_free: AtomicPtr::new(FREE_LIST_SENTINEL),
            free_count: AtomicUsize::new(0),
        }
    }

    fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    /// Remove this rule node from the child list.
    ///
    /// This method doesn't use proper synchronization, and it's expected to be
    /// called in a single-threaded fashion, thus the unsafety.
    ///
    /// This is expected to be called before freeing the node from the free
    /// list.
    unsafe fn remove_from_child_list(&self) {
        debug!("Remove from child list: {:?}, parent: {:?}",
               self as *const RuleNode, self.parent.as_ref().map(|p| p.ptr()));
        // NB: The other siblings we use in this function can also be dead, so
        // we can't use `get` here, since it asserts.
        let prev_sibling = self.prev_sibling.swap(ptr::null_mut(), Ordering::Relaxed);
        let next_sibling = self.next_sibling.swap(ptr::null_mut(), Ordering::Relaxed);

        // Store the `next` pointer as appropriate, either in the previous
        // sibling, or in the parent otherwise.
        if prev_sibling == ptr::null_mut() {
            let parent = self.parent.as_ref().unwrap();
            parent.get().first_child.store(next_sibling, Ordering::Relaxed);
        } else {
            let previous = &*prev_sibling;
            previous.next_sibling.store(next_sibling, Ordering::Relaxed);
        }

        // Store the previous sibling pointer in the next sibling if present,
        // otherwise we're done.
        if next_sibling != ptr::null_mut() {
            let next = &*next_sibling;
            next.prev_sibling.store(prev_sibling, Ordering::Relaxed);
        }
    }

    fn dump<W: Write>(&self, writer: &mut W, indent: usize) {
        const INDENT_INCREMENT: usize = 4;

        for _ in 0..indent {
            let _ = write!(writer, " ");
        }

        let _ = writeln!(writer, " - {:?} (ref: {:?}, parent: {:?})",
                         self as *const _, self.refcount.load(Ordering::SeqCst),
                         self.parent.as_ref().map(|p| p.ptr()));

        for _ in 0..indent {
            let _ = write!(writer, " ");
        }

        match self.source {
            Some(ref source) => {
                source.dump(writer);
            }
            None => {
                if indent != 0 {
                    error!("How has this happened?");
                }
                let _ = write!(writer, "(root)");
            }
        }

        let _ = write!(writer, "\n");
        for child in self.iter_children() {
            child.get().dump(writer, indent + INDENT_INCREMENT);
        }
    }

    fn iter_children(&self) -> RuleChildrenListIter {
        // FIXME(emilio): Fiddle with memory orderings.
        let first_child = self.first_child.load(Ordering::SeqCst);
        RuleChildrenListIter {
            current: if first_child.is_null() {
                None
            } else {
                Some(WeakRuleNode { ptr: first_child })
            }
        }
    }
}

#[derive(Clone)]
struct WeakRuleNode {
    ptr: *mut RuleNode,
}

/// A strong reference to a rule node.
#[derive(Debug)]
pub struct StrongRuleNode {
    ptr: *mut RuleNode,
}

#[cfg(feature = "servo")]
impl HeapSizeOf for StrongRuleNode {
    fn heap_size_of_children(&self) -> usize { 0 }
}


impl StrongRuleNode {
    fn new(n: Box<RuleNode>) -> Self {
        debug_assert!(n.parent.is_none() == n.source.is_none());

        let ptr = Box::into_raw(n);

        debug!("Creating rule node: {:p}", ptr);

        StrongRuleNode {
            ptr: ptr,
        }
    }

    fn downgrade(&self) -> WeakRuleNode {
        WeakRuleNode {
            ptr: self.ptr,
        }
    }

    fn next_sibling(&self) -> Option<WeakRuleNode> {
        // FIXME(emilio): Investigate what ordering can we achieve without
        // messing things up.
        let ptr = self.get().next_sibling.load(Ordering::SeqCst);
        if ptr.is_null() {
            None
        } else {
            Some(WeakRuleNode {
                ptr: ptr
            })
        }
    }

    fn parent(&self) -> Option<&StrongRuleNode> {
        self.get().parent.as_ref()
    }

    fn ensure_child(&self,
                    root: WeakRuleNode,
                    source: StyleSource,
                    importance: Importance) -> StrongRuleNode {
        let mut last = None;
        for child in self.get().iter_children() {
            if child .get().importance == importance &&
                child.get().source.as_ref().unwrap().ptr_equals(&source) {
                return child;
            }
            last = Some(child);
        }

        let mut node = Box::new(RuleNode::new(root,
                                             self.clone(),
                                             source.clone(),
                                             importance));
        let new_ptr: *mut RuleNode = &mut *node;

        loop {
            let strong;

            {
                let next_sibling_ptr = match last {
                    Some(ref l) => &l.get().next_sibling,
                    None => &self.get().first_child,
                };

                let existing =
                    next_sibling_ptr.compare_and_swap(ptr::null_mut(),
                                                      new_ptr,
                                                      Ordering::SeqCst);

                if existing == ptr::null_mut() {
                    // Now we know we're in the correct position in the child list,
                    // we can set the back pointer, knowing that this will only be
                    // accessed again in a single-threaded manner when we're
                    // sweeping possibly dead nodes.
                    if let Some(ref l) = last {
                        node.prev_sibling.store(l.ptr(), Ordering::Relaxed);
                    }

                    return StrongRuleNode::new(node);
                }

                // Existing is not null: some thread insert a child node since we accessed `last`.
                strong = WeakRuleNode { ptr: existing }.upgrade();

                if strong.get().source.as_ref().unwrap().ptr_equals(&source) {
                    // That node happens to be for the same style source, use that.
                    return strong;
                }
            }

            // Try again inserting after the new last child.
            last = Some(strong);
        }
    }

    fn ptr(&self) -> *mut RuleNode {
        self.ptr
    }

    fn get(&self) -> &RuleNode {
        if cfg!(debug_assertions) {
            let node = unsafe { &*self.ptr };
            assert!(node.refcount.load(Ordering::SeqCst) > 0);
        }
        unsafe { &*self.ptr }
    }

    /// Get the style source corresponding to this rule node. May return `None`
    /// if it's the root node, which means that the node hasn't matched any
    /// rules.
    pub fn style_source(&self) -> Option<&StyleSource> {
        self.get().source.as_ref()
    }

    /// Get the importance that this rule node represents.
    pub fn importance(&self) -> Importance {
        self.get().importance
    }

    /// Get an iterator for this rule node and its ancestors.
    pub fn self_and_ancestors(&self) -> SelfAndAncestors {
        SelfAndAncestors {
            current: Some(self)
        }
    }

    unsafe fn pop_from_free_list(&self) -> Option<WeakRuleNode> {
        // NB: This can run from the root node destructor, so we can't use
        // `get()`, since it asserts the refcount is bigger than zero.
        let me = &*self.ptr;

        debug_assert!(me.is_root());

        // FIXME(#14213): Apparently the layout data can be gone from script.
        //
        // That's... suspicious, but it's fine if it happens for the rule tree
        // case, so just don't crash in the case we're doing the final GC in
        // script.
        if !cfg!(feature = "testing") {
            debug_assert!(!thread_state::get().is_worker() &&
                          (thread_state::get().is_layout() ||
                           thread_state::get().is_script()));
        }

        let current = me.next_free.load(Ordering::SeqCst);
        if current == FREE_LIST_SENTINEL {
            return None;
        }

        debug_assert!(!current.is_null(),
                      "Multiple threads are operating on the free list at the \
                       same time?");
        debug_assert!(current != self.ptr,
                      "How did the root end up in the free list?");

        let next = (*current).next_free.swap(ptr::null_mut(), Ordering::SeqCst);

        debug_assert!(!next.is_null(),
                      "How did a null pointer end up in the free list?");

        me.next_free.store(next, Ordering::SeqCst);

        debug!("Popping from free list: cur: {:?}, next: {:?}", current, next);

        Some(WeakRuleNode { ptr: current })
    }

    unsafe fn assert_free_list_has_no_duplicates_or_null(&self) {
        assert!(cfg!(debug_assertions), "This is an expensive check!");
        use std::collections::HashSet;

        let me = &*self.ptr;
        assert!(me.is_root());

        let mut current = self.ptr;
        let mut seen = HashSet::new();
        while current != FREE_LIST_SENTINEL {
            let next = (*current).next_free.load(Ordering::SeqCst);
            assert!(!next.is_null());
            assert!(!seen.contains(&next));
            seen.insert(next);

            current = next;
        }
    }

    unsafe fn gc(&self) {
        if cfg!(debug_assertions) {
            self.assert_free_list_has_no_duplicates_or_null();
        }

        // NB: This can run from the root node destructor, so we can't use
        // `get()`, since it asserts the refcount is bigger than zero.
        let me = &*self.ptr;

        debug_assert!(me.is_root(), "Can't call GC on a non-root node!");

        while let Some(weak) = self.pop_from_free_list() {
            let needs_drop = {
                let node = &*weak.ptr();
                if node.refcount.load(Ordering::SeqCst) == 0 {
                    node.remove_from_child_list();
                    true
                } else {
                    false
                }
            };

            debug!("GC'ing {:?}: {}", weak.ptr(), needs_drop);
            if needs_drop {
                let _ = Box::from_raw(weak.ptr());
            }
        }

        me.free_count.store(0, Ordering::SeqCst);

        debug_assert!(me.next_free.load(Ordering::SeqCst) == FREE_LIST_SENTINEL);
    }

    unsafe fn maybe_gc(&self) {
        debug_assert!(self.get().is_root(), "Can't call GC on a non-root node!");
        if self.get().free_count.load(Ordering::SeqCst) > RULE_TREE_GC_INTERVAL {
            self.gc();
        }
    }
}

/// An iterator over a rule node and its ancestors.
#[derive(Clone)]
pub struct SelfAndAncestors<'a> {
    current: Option<&'a StrongRuleNode>,
}

impl<'a> Iterator for SelfAndAncestors<'a> {
    type Item = &'a StrongRuleNode;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.map(|node| {
            self.current = node.parent();
            node
        })
    }
}

impl Clone for StrongRuleNode {
    fn clone(&self) -> Self {
        debug!("{:?}: {:?}+", self.ptr(), self.get().refcount.load(Ordering::SeqCst));
        debug_assert!(self.get().refcount.load(Ordering::SeqCst) > 0);
        self.get().refcount.fetch_add(1, Ordering::SeqCst);
        StrongRuleNode {
            ptr: self.ptr,
        }
    }
}

impl Drop for StrongRuleNode {
    fn drop(&mut self) {
        let node = unsafe { &*self.ptr };

        debug!("{:?}: {:?}-", self.ptr(), node.refcount.load(Ordering::SeqCst));
        debug!("Dropping node: {:?}, root: {:?}, parent: {:?}",
               self.ptr,
               node.root.as_ref().map(|r| r.ptr()),
               node.parent.as_ref().map(|p| p.ptr()));
        let should_drop = {
            debug_assert!(node.refcount.load(Ordering::SeqCst) > 0);
            node.refcount.fetch_sub(1, Ordering::SeqCst) == 1
        };

        if !should_drop {
            return
        }

        debug_assert_eq!(node.first_child.load(Ordering::SeqCst),
                         ptr::null_mut());
        if node.parent.is_none() {
            debug!("Dropping root node!");
            // NOTE: Calling this is fine, because the rule tree root
            // destructor needs to happen from the layout thread, where the
            // stylist, and hence, the rule tree, is held.
            unsafe { self.gc() };
            let _ = unsafe { Box::from_raw(self.ptr()) };
            return;
        }

        let root = unsafe { &*node.root.as_ref().unwrap().ptr() };
        let free_list = &root.next_free;

        // We're sure we're already in the free list, don't spinloop.
        if node.next_free.load(Ordering::SeqCst) != ptr::null_mut() {
            return;
        }

        // Ensure we "lock" the free list head swapping it with a null pointer.
        let mut old_head = free_list.load(Ordering::SeqCst);
        loop {
            match free_list.compare_exchange_weak(old_head,
                                                  ptr::null_mut(),
                                                  Ordering::SeqCst,
                                                  Ordering::Relaxed) {
                Ok(..) => {
                    if old_head != ptr::null_mut() {
                        break;
                    }
                },
                Err(new) => old_head = new,
            }
        }

        // If other thread has raced with use while using the same rule node,
        // just store the old head again, we're done.
        if node.next_free.load(Ordering::SeqCst) != ptr::null_mut() {
            free_list.store(old_head, Ordering::SeqCst);
            return;
        }

        // Else store the old head as the next pointer, and store ourselves as
        // the new head of the free list.
        node.next_free.store(old_head, Ordering::SeqCst);
        free_list.store(self.ptr(), Ordering::SeqCst);
    }
}

impl<'a> From<&'a StrongRuleNode> for WeakRuleNode {
    fn from(node: &'a StrongRuleNode) -> Self {
        WeakRuleNode {
            ptr: node.ptr(),
        }
    }
}

impl WeakRuleNode {
    fn upgrade(&self) -> StrongRuleNode {
        debug!("Upgrading weak node: {:p}", self.ptr());

        let node = unsafe { &*self.ptr };
        node.refcount.fetch_add(1, Ordering::SeqCst);
        StrongRuleNode {
            ptr: self.ptr,
        }
    }

    fn ptr(&self) -> *mut RuleNode {
        self.ptr
    }
}

struct RuleChildrenListIter {
    current: Option<WeakRuleNode>,
}

impl Iterator for RuleChildrenListIter {
    type Item = StrongRuleNode;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.take().map(|current| {
            let current = current.upgrade();
            self.current = current.next_sibling();
            current
        })
    }
}
