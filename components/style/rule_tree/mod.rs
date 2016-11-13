/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

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

#[derive(Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct RuleTree {
    root: StrongRuleNode,
}

#[derive(Debug, Clone)]
pub enum StyleSource {
    Style(Arc<RwLock<StyleRule>>),
    Declarations(Arc<RwLock<PropertyDeclarationBlock>>),
}

type StyleSourceGuardHandle<'a> =
    OwningHandle<
        RwLockReadGuard<'a, StyleRule>,
        RwLockReadGuard<'a, PropertyDeclarationBlock>>;

pub enum StyleSourceGuard<'a> {
    Style(StyleSourceGuardHandle<'a>),
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
const FREE_LIST_SENTINEL: *mut RuleNode = 0x01 as *mut RuleNode;

impl RuleTree {
    pub fn new() -> Self {
        RuleTree {
            root: StrongRuleNode::new(Box::new(RuleNode::root())),
        }
    }

    pub fn root(&self) -> StrongRuleNode {
        self.root.clone()
    }

    fn dump<W: Write>(&self, writer: &mut W) {
        let _ = writeln!(writer, " + RuleTree");
        self.root.get().dump(writer, 0);
    }

    pub fn dump_stdout(&self) {
        let mut stdout = io::stdout();
        self.dump(&mut stdout);
    }

    pub fn insert_ordered_rules<'a, I>(&self, iter: I) -> StrongRuleNode
        where I: Iterator<Item=(StyleSource, Importance)>
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
}

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

    pub fn style_source(&self) -> Option<&StyleSource> {
        self.get().source.as_ref()
    }

    pub fn importance(&self) -> Importance {
        self.get().importance
    }

    pub fn self_and_ancestors(&self) -> SelfAndAncestors {
        SelfAndAncestors {
            current: Some(self)
        }
    }

    unsafe fn pop_from_free_list(&self) -> Option<WeakRuleNode> {
        debug_assert!(thread_state::get().is_layout() &&
                      !thread_state::get().is_worker());

        let me = &*self.ptr;
        debug_assert!(me.is_root());

        let current = self.get().next_free.load(Ordering::SeqCst);
        if current == FREE_LIST_SENTINEL {
            return None;
        }

        let current = WeakRuleNode { ptr: current };

        let node = &*current.ptr();
        let next = node.next_free.swap(ptr::null_mut(), Ordering::SeqCst);
        me.next_free.store(next, Ordering::SeqCst);

        debug!("Popping from free list: cur: {:?}, next: {:?}", current.ptr(), next);

        Some(current)
    }

    unsafe fn gc(&self) {
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

        debug_assert!(me.next_free.load(Ordering::SeqCst) == FREE_LIST_SENTINEL);
    }
}

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

        // The node is already in the free list, so do nothing.
        if node.next_free.load(Ordering::SeqCst) != ptr::null_mut() {
            return;
        }

        let free_list =
            &unsafe { &*node.root.as_ref().unwrap().ptr() }.next_free;
        loop {
            let next_free = free_list.load(Ordering::SeqCst);
            debug_assert!(!next_free.is_null());

            node.next_free.store(next_free, Ordering::SeqCst);

            let existing =
                free_list.compare_and_swap(next_free,
                                           self.ptr(),
                                           Ordering::SeqCst);
            if existing == next_free {
                // Successfully inserted, yay! Otherwise try again.
                break;
            }
        }
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
