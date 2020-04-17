/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use crate::properties::Importance;
use crate::shared_lock::StylesheetGuards;
use malloc_size_of::{MallocShallowSizeOf, MallocSizeOf, MallocSizeOfOps};
use parking_lot::RwLock;
use smallvec::SmallVec;
use std::fmt;
use std::hash;
use std::io::Write;
use std::mem;
use std::ptr;
use std::sync::atomic::{self, AtomicPtr, AtomicUsize, Ordering};

use super::map::Map;
use super::unsafe_box::UnsafeBox;
use super::{CascadeLevel, StyleSource};

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
///
/// See the discussion at https://github.com/servo/servo/pull/15562 and the IRC
/// logs at http://logs.glob.uno/?c=mozilla%23servo&s=3+Apr+2017&e=3+Apr+2017
/// logs from http://logs.glob.uno/?c=mozilla%23servo&s=3+Apr+2017&e=3+Apr+2017#c644094
/// to se a discussion about the different memory orderings used here.
#[derive(Debug)]
pub struct RuleTree {
    root: StrongRuleNode,
}

impl Drop for RuleTree {
    fn drop(&mut self) {
        unsafe { self.swap_free_list_and_gc(ptr::null_mut()) }
    }
}

impl MallocSizeOf for RuleTree {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut n = 0;
        let mut stack = SmallVec::<[_; 32]>::new();
        stack.push(self.root.clone());

        while let Some(node) = stack.pop() {
            n += unsafe { ops.malloc_size_of(&*node.p) };
            let children = node.p.children.read();
            children.shallow_size_of(ops);
            for c in &*children {
                stack.push(c.upgrade());
            }
        }

        n
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ChildKey(CascadeLevel, ptr::NonNull<()>);
unsafe impl Send for ChildKey {}
unsafe impl Sync for ChildKey {}

impl RuleTree {
    /// Construct a new rule tree.
    pub fn new() -> Self {
        RuleTree {
            root: StrongRuleNode::new(Box::new(RuleNode::root())),
        }
    }

    /// Get the root rule node.
    pub fn root(&self) -> &StrongRuleNode {
        &self.root
    }

    /// This can only be called when no other threads is accessing this tree.
    pub fn gc(&self) {
        unsafe { self.swap_free_list_and_gc(RuleNode::DANGLING_PTR) }
    }

    /// This can only be called when no other threads is accessing this tree.
    pub fn maybe_gc(&self) {
        #[cfg(debug_assertions)]
        self.maybe_dump_stats();

        if self.root.p.free_count.load(Ordering::Relaxed) > RULE_TREE_GC_INTERVAL {
            self.gc();
        }
    }

    #[cfg(debug_assertions)]
    fn maybe_dump_stats(&self) {
        use itertools::Itertools;
        use std::cell::Cell;
        use std::time::{Duration, Instant};

        if !log_enabled!(log::Level::Trace) {
            return;
        }

        const RULE_TREE_STATS_INTERVAL: Duration = Duration::from_secs(2);

        thread_local! {
            pub static LAST_STATS: Cell<Instant> = Cell::new(Instant::now());
        };

        let should_dump = LAST_STATS.with(|s| {
            let now = Instant::now();
            if now.duration_since(s.get()) < RULE_TREE_STATS_INTERVAL {
                return false;
            }
            s.set(now);
            true
        });

        if !should_dump {
            return;
        }

        let mut children_count = crate::hash::FxHashMap::default();

        let mut stack = SmallVec::<[_; 32]>::new();
        stack.push(self.root.clone());
        while let Some(node) = stack.pop() {
            let children = node.p.children.read();
            *children_count.entry(children.len()).or_insert(0) += 1;
            for c in &*children {
                stack.push(c.upgrade());
            }
        }

        trace!("Rule tree stats:");
        let counts = children_count.keys().sorted();
        for count in counts {
            trace!(" {} - {}", count, children_count[count]);
        }
    }

    /// Swaps the tree's free list's head with a given pointer and collects
    /// the free list, taking care of not swapping any pointer with its lowest
    /// bit set, given that would break the lock currently held by another
    /// thread.
    unsafe fn swap_free_list_and_gc(&self, ptr: *mut RuleNode) {
        let root = &self.root.p;
        let mut head = root.next_free.load(Ordering::Relaxed);
        loop {
            // This is only ever called when the tree is dropped or when
            // a GC is manually requested, so the free list's head should
            // never be null already.
            debug_assert!(!head.is_null());
            if head == ptr {
                // In the case of swapping the pointer with
                // `NodeInner::DANGLING_PTR`, we can return immediately
                // because the free list is already empty so there is nothing
                // to GC.
                return;
            }
            // Unmask the lock bit from the current head, this is the most
            // probable value `compare_exchange_weak` will read when the other
            // thread currently locking the free list unlocks it.
            head = (head as usize & !1) as *mut RuleNode;
            // This could fail if the free list head is
            // `NodeInner::DANGLING_PTR` with the lowest bit set, which
            // makes no sense.
            debug_assert!(head != ptr);
            match root.next_free.compare_exchange_weak(
                head,
                ptr,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(old_head) => {
                    head = old_head;
                    break;
                }
                Err(current_head) => head = current_head,
            }
        }
        loop {
            if head == RuleNode::DANGLING_PTR {
                // We reached the end of the free list.
                return;
            }
            let node = UnsafeBox::from_raw(head);
            let next = node.next_free.swap(ptr::null_mut(), Ordering::Relaxed);
            // This fails if we found a node on the free list with a next
            // free pointer that got its lowest bit set, that makes no sense.
            debug_assert!(head as usize & 1 == 0);
            // It wouldn't make sense for a node on the free list to have
            // a null next free pointer.
            debug_assert!(!head.is_null());
            drop(StrongRuleNode::from_unsafe_box(node));
            // Iterates on the next item in the free list.
            head = next;
        }
    }
}

/// The number of RuleNodes added to the free list before we will consider
/// doing a GC when calling maybe_gc().  (The value is copied from Gecko,
/// where it likely did not result from a rigorous performance analysis.)
const RULE_TREE_GC_INTERVAL: usize = 300;

/// A node in the rule tree.
struct RuleNode {
    /// The root node. Only the root has no root pointer, for obvious reasons.
    root: Option<WeakRuleNode>,

    /// The parent rule node. Only the root has no parent.
    parent: Option<StrongRuleNode>,

    /// The actual style source, either coming from a selector in a StyleRule,
    /// or a raw property declaration block (like the style attribute).
    ///
    /// None for the root node.
    source: Option<StyleSource>,

    /// The cascade level this rule is positioned at.
    level: CascadeLevel,

    refcount: AtomicUsize,

    /// Only used for the root, stores the number of free rule nodes that are
    /// around.
    free_count: AtomicUsize,

    /// The children of a given rule node. Children remove themselves from here
    /// when they go away.
    children: RwLock<Map<ChildKey, WeakRuleNode>>,

    /// The next item in the rule tree free list, that starts on the root node.
    ///
    /// When this is set to null, that means that the rule tree has been torn
    /// down, and GCs will no longer occur. When this happens, StrongRuleNodes
    /// may only be dropped on the main thread, and teardown happens
    /// synchronously.
    next_free: AtomicPtr<RuleNode>,
}

// On Gecko builds, hook into the leak checking machinery.
#[cfg(feature = "gecko_refcount_logging")]
mod gecko_leak_checking {
    use super::RuleNode;
    use std::mem::size_of;
    use std::os::raw::{c_char, c_void};

    extern "C" {
        fn NS_LogCtor(aPtr: *mut c_void, aTypeName: *const c_char, aSize: u32);
        fn NS_LogDtor(aPtr: *mut c_void, aTypeName: *const c_char, aSize: u32);
    }

    static NAME: &'static [u8] = b"RuleNode\0";

    /// Logs the creation of a heap-allocated object to Gecko's leak-checking machinery.
    pub(super) fn log_ctor(ptr: *const RuleNode) {
        let s = NAME as *const [u8] as *const u8 as *const c_char;
        unsafe {
            NS_LogCtor(ptr as *mut c_void, s, size_of::<RuleNode>() as u32);
        }
    }

    /// Logs the destruction of a heap-allocated object to Gecko's leak-checking machinery.
    pub(super) fn log_dtor(ptr: *const RuleNode) {
        let s = NAME as *const [u8] as *const u8 as *const c_char;
        unsafe {
            NS_LogDtor(ptr as *mut c_void, s, size_of::<RuleNode>() as u32);
        }
    }
}

#[inline(always)]
fn log_new(_ptr: *const RuleNode) {
    #[cfg(feature = "gecko_refcount_logging")]
    gecko_leak_checking::log_ctor(_ptr);
}

#[inline(always)]
fn log_drop(_ptr: *const RuleNode) {
    #[cfg(feature = "gecko_refcount_logging")]
    gecko_leak_checking::log_dtor(_ptr);
}

impl RuleNode {
    const DANGLING_PTR: *mut Self = ptr::NonNull::dangling().as_ptr();

    fn new(
        root: WeakRuleNode,
        parent: StrongRuleNode,
        source: StyleSource,
        level: CascadeLevel,
    ) -> Self {
        debug_assert!(root.upgrade().parent().is_none());
        RuleNode {
            root: Some(root),
            parent: Some(parent),
            source: Some(source),
            level: level,
            refcount: AtomicUsize::new(1),
            children: Default::default(),
            free_count: AtomicUsize::new(0),
            next_free: AtomicPtr::new(ptr::null_mut()),
        }
    }

    fn root() -> Self {
        RuleNode {
            root: None,
            parent: None,
            source: None,
            level: CascadeLevel::UANormal,
            refcount: AtomicUsize::new(1),
            free_count: AtomicUsize::new(0),
            children: Default::default(),
            next_free: AtomicPtr::new(RuleNode::DANGLING_PTR),
        }
    }

    fn key(&self) -> ChildKey {
        ChildKey(
            self.level,
            self.source
                .as_ref()
                .expect("Called key() on the root node")
                .key(),
        )
    }

    unsafe fn drop_without_free_list(this: &mut UnsafeBox<Self>) {
        let mut this = UnsafeBox::clone(this);
        loop {
            this.next_free.store(Self::DANGLING_PTR, Ordering::Relaxed);
            if let Some(parent) = this.parent.as_ref() {
                let mut children = parent.p.children.write();
                // Another thread may have resurrected this node while
                // we were trying to drop it, leave it alone. The
                // operation can be relaxed because we are currently
                // write-locking its parent children list and
                // resurrection is only done from `Node::ensure_child`
                // which read-locks that same children list.
                if this.refcount.load(Ordering::Relaxed) != 0 {
                    return;
                }
                debug!(
                    "Remove from child list: {:?}, parent: {:?}",
                    &*this as *const RuleNode,
                    this.parent.as_ref().map(|p| &*p.p as *const RuleNode)
                );
                let weak = children.remove(&this.key(), |node| node.p.key()).unwrap();
                assert_eq!(&*weak.p as *const RuleNode, &*this as *const RuleNode);
            }
            atomic::fence(Ordering::Acquire);
            debug_assert_eq!(this.refcount.load(Ordering::Relaxed), 0);
            // Remove the parent reference from the child to avoid
            // recursively dropping it.
            let parent = UnsafeBox::deref_mut(&mut this).parent.take();
            log_drop(&*this);
            UnsafeBox::drop(&mut this);
            if let Some(parent) = parent {
                this = UnsafeBox::clone(&parent.p);
                mem::forget(parent);
                if this.refcount.fetch_sub(1, Ordering::Release) == 1 {
                    // The node had a parent and its refcount reached
                    // zero, we reiterate the loop to drop it too.
                    continue;
                }
            }
            // The node didn't have a parent or the parent has other
            // live reference elsewhere, we don't have anything to do
            // anymore.
            return;
        }
    }

    /// Pushes this node on the tree's free list. Returns false if the free list
    /// is gone.
    unsafe fn push_on_free_list(this: &UnsafeBox<Self>) -> bool {
        let root = &this.root.as_ref().unwrap().p;
        let mut old_head = root.next_free.load(Ordering::Relaxed);
        let this_ptr = &**this as *const RuleNode as *mut RuleNode;
        let this_lock = (this_ptr as usize | 1) as *mut RuleNode;
        loop {
            if old_head.is_null() {
                // Tree was dropped and free list has been destroyed.
                return false;
            }
            // Unmask the lock bit from the current head, this is the most
            // probable value `compare_exchange_weak` will read when the other
            // thread currently locking the free list unlocks it.
            old_head = (old_head as usize & !1) as *mut RuleNode;
            if old_head == this_ptr {
                // The free list is currently locked or has finished being
                // locked to put this very node at the head of the free list,
                // which means we don't need to do it ourselves.
                return true;
            }
            match root.next_free.compare_exchange_weak(
                old_head,
                this_lock,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(current_head) => old_head = current_head,
            }
        }
        if !this.next_free.load(Ordering::Relaxed).is_null() {
            // Another thread managed to resurrect this node and put it on
            // the free list while we were still busy trying to lock the free
            // list. That means we are done and we just need to unlock the free
            // list with whatever head we read last.
            root.next_free.store(old_head, Ordering::Release);
        } else {
            // We increment the refcount of this node to account for its presence
            // in the tree's free list.
            this.refcount.fetch_add(1, Ordering::Relaxed);

            // The free count is only ever written to when the free list is
            // locked so we don't need an atomic increment here.
            let old_free_count = root.free_count.load(Ordering::Relaxed);
            root.free_count.store(old_free_count + 1, Ordering::Relaxed);

            // Finally, we store the old free list head into this node's next free
            // slot and we unlock the guard with the new head.
            this.next_free.store(old_head, Ordering::Relaxed);
            root.next_free.store(this_ptr, Ordering::Release);
        }
        true
    }
}

pub(crate) struct WeakRuleNode {
    p: UnsafeBox<RuleNode>,
}

/// A strong reference to a rule node.
pub struct StrongRuleNode {
    p: UnsafeBox<RuleNode>,
}

#[cfg(feature = "servo")]
malloc_size_of_is_0!(StrongRuleNode);

impl StrongRuleNode {
    fn new(n: Box<RuleNode>) -> Self {
        debug_assert_eq!(n.parent.is_none(), !n.source.is_some());

        log_new(&*n);

        debug!("Creating rule node: {:p}", &*n);

        Self {
            p: UnsafeBox::from_box(n),
        }
    }

    unsafe fn from_unsafe_box(p: UnsafeBox<RuleNode>) -> Self {
        Self { p }
    }

    unsafe fn downgrade(&self) -> WeakRuleNode {
        WeakRuleNode {
            p: UnsafeBox::clone(&self.p),
        }
    }

    /// Get the parent rule node of this rule node.
    pub fn parent(&self) -> Option<&StrongRuleNode> {
        self.p.parent.as_ref()
    }

    pub(super) fn ensure_child(
        &self,
        root: &StrongRuleNode,
        source: StyleSource,
        level: CascadeLevel,
    ) -> StrongRuleNode {
        use parking_lot::RwLockUpgradableReadGuard;

        debug_assert!(
            self.p.level <= level,
            "Should be ordered (instead {:?} > {:?}), from {:?} and {:?}",
            self.p.level,
            level,
            self.p.source,
            source,
        );

        let key = ChildKey(level, source.key());
        let children = self.p.children.upgradable_read();
        if let Some(child) = children.get(&key, |node| node.p.key()) {
            if child.p.refcount.fetch_add(1, Ordering::Relaxed) == 0 {
                // We just observed a node with a refcount of 0, that means
                // another thread is trying to drop the last reference of this
                // node. We spinlock on child.p.next_free being set to a value
                // different than ptr::null_mut(), which happens when the node
                // is added to the free list or just before we try to delete it
                // from its parent.
                while child.p.next_free.load(Ordering::Relaxed).is_null() {}
            }
            return unsafe { StrongRuleNode::from_unsafe_box(UnsafeBox::clone(&child.p)) };
        }
        let mut children = RwLockUpgradableReadGuard::upgrade(children);
        let weak = children.get_or_insert_with(
            key,
            |node| node.p.key(),
            move || {
                let root = unsafe { root.downgrade() };
                let strong =
                    StrongRuleNode::new(Box::new(RuleNode::new(root, self.clone(), source, level)));
                let weak = unsafe { strong.downgrade() };
                mem::forget(strong);
                weak
            },
        );
        unsafe { StrongRuleNode::from_unsafe_box(UnsafeBox::clone(&weak.p)) }
    }

    /// Get the style source corresponding to this rule node. May return `None`
    /// if it's the root node, which means that the node hasn't matched any
    /// rules.
    pub fn style_source(&self) -> Option<&StyleSource> {
        self.p.source.as_ref()
    }

    /// The cascade level for this node
    pub fn cascade_level(&self) -> CascadeLevel {
        self.p.level
    }

    /// Get the importance that this rule node represents.
    pub fn importance(&self) -> Importance {
        self.p.level.importance()
    }

    /// Returns whether this node has any child, only intended for testing
    /// purposes, and called on a single-threaded fashion only.
    pub unsafe fn has_children_for_testing(&self) -> bool {
        !self.p.children.read().is_empty()
    }

    pub(super) fn dump<W: Write>(&self, guards: &StylesheetGuards, writer: &mut W, indent: usize) {
        const INDENT_INCREMENT: usize = 4;

        for _ in 0..indent {
            let _ = write!(writer, " ");
        }

        let _ = writeln!(
            writer,
            " - {:p} (ref: {:?}, parent: {:?})",
            &*self.p,
            self.p.refcount.load(Ordering::Relaxed),
            self.parent().map(|p| &*p.p as *const RuleNode)
        );

        for _ in 0..indent {
            let _ = write!(writer, " ");
        }

        if let Some(source) = self.style_source() {
            source.dump(self.cascade_level().guard(guards), writer);
        } else {
            if indent != 0 {
                warn!("How has this happened?");
            }
            let _ = write!(writer, "(root)");
        }

        let _ = write!(writer, "\n");
        for child in &*self.p.children.read() {
            child
                .upgrade()
                .dump(guards, writer, indent + INDENT_INCREMENT);
        }
    }
}

impl Clone for StrongRuleNode {
    fn clone(&self) -> Self {
        debug!(
            "{:p}: {:?}+",
            &*self.p,
            self.p.refcount.load(Ordering::Relaxed)
        );
        debug_assert!(self.p.refcount.load(Ordering::Relaxed) > 0);
        self.p.refcount.fetch_add(1, Ordering::Relaxed);
        unsafe { StrongRuleNode::from_unsafe_box(UnsafeBox::clone(&self.p)) }
    }
}

impl Drop for StrongRuleNode {
    #[cfg_attr(feature = "servo", allow(unused_mut))]
    fn drop(&mut self) {
        let node = &*self.p;
        debug!("{:p}: {:?}-", node, node.refcount.load(Ordering::Relaxed));
        debug!(
            "Dropping node: {:p}, root: {:?}, parent: {:?}",
            node,
            node.root.as_ref().map(|r| &*r.p as *const RuleNode),
            node.parent.as_ref().map(|p| &*p.p as *const RuleNode)
        );

        let should_drop = {
            debug_assert!(node.refcount.load(Ordering::Relaxed) > 0);
            node.refcount.fetch_sub(1, Ordering::Relaxed) == 1
        };

        if !should_drop {
            return;
        }

        unsafe {
            if node.root.is_none() || !RuleNode::push_on_free_list(&self.p) {
                RuleNode::drop_without_free_list(&mut self.p)
            }
        }
    }
}

impl WeakRuleNode {
    fn upgrade(&self) -> StrongRuleNode {
        debug!("Upgrading weak node: {:p}", &*self.p);
        self.p.refcount.fetch_add(1, Ordering::Relaxed);
        unsafe { StrongRuleNode::from_unsafe_box(UnsafeBox::clone(&self.p)) }
    }
}

impl fmt::Debug for StrongRuleNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (&*self.p as *const RuleNode).fmt(f)
    }
}

impl Eq for StrongRuleNode {}
impl PartialEq for StrongRuleNode {
    fn eq(&self, other: &Self) -> bool {
        &*self.p as *const RuleNode == &*other.p
    }
}

impl hash::Hash for StrongRuleNode {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        (&*self.p as *const RuleNode).hash(state)
    }
}
