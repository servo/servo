/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use arc_ptr_eq;
#[cfg(feature = "servo")]
use heapsize::HeapSizeOf;
use properties::{Importance, PropertyDeclarationBlock};
use selector_matching::Rule;
use std::io::{self, Write};
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use stylesheets::StyleRule;
use thread_state;

#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct RuleTree {
    root: StrongRuleNode,
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
        where I: Iterator<Item=(&'a Arc<StyleRule>, Importance)>
    {
        let root = self.root.clone();
        let mut current = self.root.clone();
        for (rule, importance) in iter {
            current = current.ensure_child(&root, rule, importance);
        }
        current
    }

    unsafe fn pop_from_free_list(&self) -> Option<WeakRuleNode> {
        debug_assert!(thread_state::get().is_layout() &&
                      !thread_state::get().is_worker());

        let current = self.root.get().next_free.load(Ordering::SeqCst);
        if current == FREE_LIST_SENTINEL {
            return None;
        }

        let current = WeakRuleNode { ptr: current };

        let node = &*current.ptr();
        let next = node.next_free.swap(ptr::null_mut(), Ordering::SeqCst);
        self.root.get().next_free.store(next, Ordering::SeqCst);

        debug!("Popping from free list: cur: {:?}, next: {:?}", current.ptr(), next);

        Some(current)
    }

    pub unsafe fn gc(&self) {
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

        debug_assert!(self.root.get().next_free.load(Ordering::SeqCst) == FREE_LIST_SENTINEL);
    }
}

struct RuleNode {
    /// The root node. Only the root has no root pointer, for obvious reasons.
    root: Option<StrongRuleNode>,

    /// The parent rule node. Only the root has no parent.
    parent: Option<StrongRuleNode>,

    /// The actual style rule that matched. Only the root has no declarations.
    rule: Option<Arc<StyleRule>>,

    /// The importance of the declarations relevant in the style rule,
    /// meaningless in the root node.
    importance: Importance,

    children: RuleChildrenList,
    refcount: AtomicUsize,
    next: AtomicPtr<RuleNode>,
    prev: AtomicPtr<RuleNode>,

    /// The next item in the rule tree free list, that starts on the root node.
    next_free: AtomicPtr<RuleNode>,
}

unsafe impl Sync for RuleTree {}
unsafe impl Send for RuleTree {}

impl RuleNode {
    fn new(root: StrongRuleNode,
           parent: StrongRuleNode,
           style_rule: Arc<StyleRule>,
           importance: Importance) -> Self {
        debug_assert!(root.get().parent.is_none());
        RuleNode {
            root: Some(root),
            parent: Some(parent),
            rule: Some(style_rule),
            importance: importance,
            children: RuleChildrenList::new(),
            refcount: AtomicUsize::new(1),
            next: AtomicPtr::new(ptr::null_mut()),
            prev: AtomicPtr::new(ptr::null_mut()),
            next_free: AtomicPtr::new(ptr::null_mut()),
        }
    }

    fn root() -> Self {
        RuleNode {
            root: None,
            parent: None,
            rule: None,
            importance: Importance::Normal,
            children: RuleChildrenList::new(),
            refcount: AtomicUsize::new(1),
            next: AtomicPtr::new(ptr::null_mut()),
            prev: AtomicPtr::new(ptr::null_mut()),
            next_free: AtomicPtr::new(FREE_LIST_SENTINEL),
        }
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
        let previous = self.prev.swap(ptr::null_mut(), Ordering::SeqCst);
        let next = self.next.swap(ptr::null_mut(), Ordering::SeqCst);

        if previous != ptr::null_mut() {
            let really_previous = WeakRuleNode { ptr: previous };
            really_previous.upgrade()
                .get().next.store(next, Ordering::SeqCst);
        } else {
            self.parent.as_ref().unwrap()
                .get().children.head.store(ptr::null_mut(), Ordering::SeqCst);
        }

        if next != ptr::null_mut() {
            let really_next = WeakRuleNode { ptr: next };
            really_next.upgrade().get().prev.store(previous, Ordering::SeqCst);
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

        match self.rule {
            Some(ref rule) => {
                // let _ = write!(writer, "{:?}", rule);
                let _ = write!(writer, "{:?}", rule.selectors);
            }
            None => {
                if indent != 0 {
                    error!("How has this happened?");
                }
                let _ = write!(writer, "(root)");
            }
        }
        write!(writer, "\n");
        for child in self.children.iter() {
            child.get().dump(writer, indent + INDENT_INCREMENT);
        }
    }
}

#[derive(Clone)]
struct WeakRuleNode {
    ptr: *mut RuleNode,
}

pub struct StrongRuleNode {
    ptr: *mut RuleNode,
}

#[cfg(feature = "servo")]
impl HeapSizeOf for StrongRuleNode {
    fn heap_size_of_children(&self) -> usize { 0 }
}


impl StrongRuleNode {
    fn new(n: Box<RuleNode>) -> Self {
        debug_assert!(n.parent.is_none() == n.rule.is_none());

        let ptr = Box::into_raw(n);

        debug!("Creating rule node: 0x{:x}", ptr as usize);

        StrongRuleNode {
            ptr: ptr,
        }
    }

    fn downgrade(&self) -> WeakRuleNode {
        WeakRuleNode {
            ptr: self.ptr,
        }
    }

    fn next(&self) -> Option<WeakRuleNode> {
        // FIXME(emilio): Investigate what ordering can we achieve without
        // messing things up.
        let ptr = self.get().next.load(Ordering::SeqCst);
        if ptr.is_null() {
            None
        } else {
            Some(WeakRuleNode {
                ptr: ptr
            })
        }
    }

    fn parent(&self) -> Option<StrongRuleNode> {
        self.get().parent.clone()
    }

    fn ensure_child(&self,
                    root: &StrongRuleNode,
                    rule: &Arc<StyleRule>,
                    importance: Importance) -> StrongRuleNode {
        let mut last = None;
        let mut iter = self.get().children.iter();
        while let Some(current) = iter.next() {
            if current.get().importance == importance &&
               arc_ptr_eq(current.get().rule.as_ref().unwrap(), rule) {
                return current;
            }
            last = Some(current);
        }

        let node = Box::new(RuleNode::new(root.clone(),
                                          self.clone(),
                                          rule.clone(),
                                          importance));
        let new_ptr = &*node as *const _ as *mut RuleNode;

        loop {
            let strong;

            {
                let next_ptr = match last {
                    Some(ref l) => &l.get().next,
                    None => &self.get().children.head,
                };

                let existing =
                    next_ptr.compare_and_swap(ptr::null_mut(),
                                              new_ptr,
                                              Ordering::SeqCst);

                if existing == ptr::null_mut() {
                    // Now we know we're in the correct position in the child list,
                    // we can set the back pointer, knowing that this will only be
                    // accessed again in a single-threaded manner when we're
                    // sweeping possibly dead nodes.
                    if let Some(ref l) = last {
                        node.prev.store(l.ptr(), Ordering::Relaxed);
                    }

                    return StrongRuleNode::new(node);
                }

                strong = WeakRuleNode { ptr: existing }.upgrade();

                if arc_ptr_eq(strong.get().rule.as_ref().unwrap(), rule) {
                    // Some thread that was racing with as inserted the same rule
                    // node than us, so give up and just use that.
                    return strong;
                }
            }

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

        if should_drop {
            debug_assert_eq!(node.children.head.load(Ordering::SeqCst),
                             ptr::null_mut());
            if node.parent.is_none() {
                debug!("Dropping root node!");
                debug_assert!(node.next_free.load(Ordering::SeqCst) == FREE_LIST_SENTINEL);
                let _ = unsafe { Box::from_raw(self.ptr()) };
                return;
            }

            // The node is already in the free list, so do nothing.
            if node.next_free.load(Ordering::SeqCst) != ptr::null_mut() {
                return;
            }

            let root = node.root.as_ref().unwrap();
            let free_list = &root.get().next_free;

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
}

impl<'a> From<&'a StrongRuleNode> for WeakRuleNode {
    fn from(node: &'a StrongRuleNode) -> Self {
        WeakRuleNode {
            ptr: node.ptr(),
        }
    }
}

impl WeakRuleNode {
    fn has_strong_reference(&self) -> bool {
        let node = unsafe { &*self.ptr };
        node.refcount.load(Ordering::SeqCst) != 0
    }

    fn upgrade(&self) -> StrongRuleNode {
        debug!("Upgrading weak node: 0x{:?}", self.ptr());

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

struct RuleChildrenList {
    head: AtomicPtr<RuleNode>,
}

impl RuleChildrenList {
    fn new() -> Self {
        RuleChildrenList {
            head: AtomicPtr::new(ptr::null_mut())
        }
    }

    fn contains(&self, weak: &WeakRuleNode) -> bool {
        self.find(weak).is_some()
    }

    fn find(&self, weak_node: &WeakRuleNode) -> Option<StrongRuleNode> {
        for node in self.iter() {
            if node.ptr() == weak_node.ptr() {
                return Some(node.clone())
            }
        }
        None
    }

    fn iter(&self) -> RuleChildrenListIter {
        // FIXME(emilio): Fiddle with memory orderings.
        let head = self.head.load(Ordering::SeqCst);
        RuleChildrenListIter {
            current: if head.is_null() {
                None
            } else {
                Some(WeakRuleNode {
                    ptr: head,
                })
            },
        }
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
            self.current = current.next();
            current
        })
    }
}
