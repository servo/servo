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

use super::map::{Entry, Map};
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
/// instead put into a free list, and it is potentially GC'd after a while.
///
/// That way, a rule node that represents a likely-to-match-again rule (like a
/// :hover rule) can be reused if we haven't GC'd it yet.
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
                stack.push(unsafe { c.upgrade() });
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

        if self.root.p.approximate_free_count.load(Ordering::Relaxed) > RULE_TREE_GC_INTERVAL {
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
                stack.push(unsafe { c.upgrade() });
            }
        }

        trace!("Rule tree stats:");
        let counts = children_count.keys().sorted();
        for count in counts {
            trace!(" {} - {}", count, children_count[count]);
        }
    }

    /// Steals the free list and drops its contents.
    unsafe fn swap_free_list_and_gc(&self, ptr: *mut RuleNode) {
        let root = &self.root.p;

        debug_assert!(!root.next_free.load(Ordering::Relaxed).is_null());

        // Reset the approximate free count to zero, as we are going to steal
        // the free list.
        root.approximate_free_count.store(0, Ordering::Relaxed);

        // Steal the free list head. Memory loads on nodes while iterating it
        // must observe any prior changes that occured so this requires
        // acquire ordering, but there are no writes that need to be kept
        // before this swap so there is no need for release.
        let mut head = root.next_free.swap(ptr, Ordering::Acquire);

        while head != RuleNode::DANGLING_PTR {
            debug_assert!(!head.is_null());

            let mut node = UnsafeBox::from_raw(head);

            // The root node cannot go on the free list.
            debug_assert!(node.root.is_some());

            // The refcount of nodes on the free list never goes below 1.
            debug_assert!(node.refcount.load(Ordering::Relaxed) > 0);

            // No one else is currently writing to that field. Get the address
            // of the next node in the free list and replace it with null,
            // other threads will now consider that this node is not on the
            // free list.
            head = node.next_free.swap(ptr::null_mut(), Ordering::Relaxed);

            // This release write synchronises with the acquire fence in
            // `WeakRuleNode::upgrade`, making sure that if `upgrade` observes
            // decrements the refcount to 0, it will also observe the
            // `node.next_free` swap to null above.
            if node.refcount.fetch_sub(1, Ordering::Release) == 1 {
                // And given it observed the null swap above, it will need
                // `pretend_to_be_on_free_list` to finish its job, writing
                // `RuleNode::DANGLING_PTR` in `node.next_free`.
                RuleNode::pretend_to_be_on_free_list(&node);

                // Drop this node now that we just observed its refcount going
                // down to zero.
                RuleNode::drop_without_free_list(&mut node);
            }
        }
    }
}

/// The number of RuleNodes added to the free list before we will consider
/// doing a GC when calling maybe_gc().  (The value is copied from Gecko,
/// where it likely did not result from a rigorous performance analysis.)
const RULE_TREE_GC_INTERVAL: usize = 300;

/// Used for some size assertions.
pub const RULE_NODE_SIZE: usize = std::mem::size_of::<RuleNode>();

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

    /// The refcount of this node.
    ///
    /// Starts at one. Incremented in `StrongRuleNode::clone` and
    /// `WeakRuleNode::upgrade`. Decremented in `StrongRuleNode::drop`
    /// and `RuleTree::swap_free_list_and_gc`.
    ///
    /// If a non-root node's refcount reaches zero, it is incremented back to at
    /// least one in `RuleNode::pretend_to_be_on_free_list` until the caller who
    /// observed it dropping to zero had a chance to try to remove it from its
    /// parent's children list.
    ///
    /// The refcount should never be decremented to zero if the value in
    /// `next_free` is not null.
    refcount: AtomicUsize,

    /// Only used for the root, stores the number of free rule nodes that are
    /// around.
    approximate_free_count: AtomicUsize,

    /// The children of a given rule node. Children remove themselves from here
    /// when they go away.
    children: RwLock<Map<ChildKey, WeakRuleNode>>,

    /// This field has two different meanings depending on whether this is the
    /// root node or not.
    ///
    /// If it is the root, it represents the head of the free list. It may be
    /// null, which means the free list is gone because the tree was dropped,
    /// and it may be `RuleNode::DANGLING_PTR`, which means the free list is
    /// empty.
    ///
    /// If it is not the root node, this field is either null if the node is
    /// not on the free list, `RuleNode::DANGLING_PTR` if it is the last item
    /// on the free list or the node is pretending to be on the free list, or
    /// any valid non-null pointer representing the next item on the free list
    /// after this one.
    ///
    /// See `RuleNode::push_on_free_list`, `swap_free_list_and_gc`, and
    /// `WeakRuleNode::upgrade`.
    ///
    /// Two threads should never attempt to put the same node on the free list
    /// both at the same time.
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

    unsafe fn new(
        root: WeakRuleNode,
        parent: StrongRuleNode,
        source: StyleSource,
        level: CascadeLevel,
    ) -> Self {
        debug_assert!(root.p.parent.is_none());
        RuleNode {
            root: Some(root),
            parent: Some(parent),
            source: Some(source),
            level: level,
            refcount: AtomicUsize::new(1),
            children: Default::default(),
            approximate_free_count: AtomicUsize::new(0),
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
            approximate_free_count: AtomicUsize::new(0),
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

    /// Drops a node without ever putting it on the free list.
    ///
    /// Note that the node may not be dropped if we observe that its refcount
    /// isn't zero anymore when we write-lock its parent's children map to
    /// remove it.
    ///
    /// This loops over parents of dropped nodes if their own refcount reaches
    /// zero to avoid recursion when dropping deep hierarchies of nodes.
    ///
    /// For non-root nodes, this should always be preceded by a call of
    /// `RuleNode::pretend_to_be_on_free_list`.
    unsafe fn drop_without_free_list(this: &mut UnsafeBox<Self>) {
        // We clone the box and shadow the original one to be able to loop
        // over its ancestors if they also need to be dropped.
        let mut this = UnsafeBox::clone(this);
        loop {
            // If the node has a parent, we need to remove it from its parent's
            // children list.
            if let Some(parent) = this.parent.as_ref() {
                debug_assert!(!this.next_free.load(Ordering::Relaxed).is_null());

                // We lock the parent's children list, which means no other
                // thread will have any more opportunity to resurrect the node
                // anymore.
                let mut children = parent.p.children.write();

                this.next_free.store(ptr::null_mut(), Ordering::Relaxed);

                // We decrement the counter to remove the "pretend to be
                // on the free list" reference.
                let old_refcount = this.refcount.fetch_sub(1, Ordering::Release);
                debug_assert!(old_refcount != 0);
                if old_refcount != 1 {
                    // Other threads resurrected this node and those references
                    // are still alive, we have nothing to do anymore.
                    return;
                }

                // We finally remove the node from its parent's children list,
                // there are now no other references to it and it cannot
                // be resurrected anymore even after we unlock the list.
                debug!(
                    "Remove from child list: {:?}, parent: {:?}",
                    this.as_mut_ptr(),
                    this.parent.as_ref().map(|p| p.p.as_mut_ptr())
                );
                let weak = children.remove(&this.key(), |node| node.p.key()).unwrap();
                assert_eq!(weak.p.as_mut_ptr(), this.as_mut_ptr());
            } else {
                debug_assert_eq!(this.next_free.load(Ordering::Relaxed), ptr::null_mut());
                debug_assert_eq!(this.refcount.load(Ordering::Relaxed), 0);
            }

            // We are going to drop this node for good this time, as per the
            // usual refcounting protocol we need an acquire fence here before
            // we run the destructor.
            //
            // See https://github.com/rust-lang/rust/pull/41714#issuecomment-298996916
            // for why it doesn't matter whether this is a load or a fence.
            atomic::fence(Ordering::Acquire);

            // Remove the parent reference from the child to avoid
            // recursively dropping it and putting it on the free list.
            let parent = UnsafeBox::deref_mut(&mut this).parent.take();

            // We now drop the actual box and its contents, no one should
            // access the current value in `this` anymore.
            log_drop(&*this);
            UnsafeBox::drop(&mut this);

            if let Some(parent) = parent {
                // We will attempt to drop the node's parent without the free
                // list, so we clone the inner unsafe box and forget the
                // original parent to avoid running its `StrongRuleNode`
                // destructor which would attempt to use the free list if it
                // still exists.
                this = UnsafeBox::clone(&parent.p);
                mem::forget(parent);
                if this.refcount.fetch_sub(1, Ordering::Release) == 1 {
                    debug_assert_eq!(this.next_free.load(Ordering::Relaxed), ptr::null_mut());
                    if this.root.is_some() {
                        RuleNode::pretend_to_be_on_free_list(&this);
                    }
                    // Parent also reached refcount zero, we loop to drop it.
                    continue;
                }
            }

            return;
        }
    }

    /// Pushes this node on the tree's free list. Returns false if the free list
    /// is gone. Should only be called after we decremented a node's refcount
    /// to zero and pretended to be on the free list.
    unsafe fn push_on_free_list(this: &UnsafeBox<Self>) -> bool {
        let root = &this.root.as_ref().unwrap().p;

        debug_assert!(this.refcount.load(Ordering::Relaxed) > 0);
        debug_assert_eq!(this.next_free.load(Ordering::Relaxed), Self::DANGLING_PTR);

        // Increment the approximate free count by one.
        root.approximate_free_count.fetch_add(1, Ordering::Relaxed);

        // If the compare-exchange operation fails in the loop, we will retry
        // with the new head value, so this can be a relaxed load.
        let mut head = root.next_free.load(Ordering::Relaxed);

        while !head.is_null() {
            // Two threads can never attempt to push the same node on the free
            // list both at the same time, so whoever else pushed a node on the
            // free list cannot have done so with this node.
            debug_assert_ne!(head, this.as_mut_ptr());

            // Store the current head of the free list in this node.
            this.next_free.store(head, Ordering::Relaxed);

            // Any thread acquiring the free list must observe the previous
            // next_free changes that occured, hence the release ordering
            // on success.
            match root.next_free.compare_exchange_weak(
                head,
                this.as_mut_ptr(),
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    // This node is now on the free list, caller should not use
                    // the node anymore.
                    return true;
                },
                Err(new_head) => head = new_head,
            }
        }

        // Tree was dropped and free list has been destroyed. We did not push
        // this node on the free list but we still pretend to be on the free
        // list to be ready to call `drop_without_free_list`.
        false
    }

    /// Makes the node pretend to be on the free list. This will increment the
    /// refcount by 1 and store `Self::DANGLING_PTR` in `next_free`. This
    /// method should only be called after caller decremented the refcount to
    /// zero, with the null pointer stored in `next_free`.
    unsafe fn pretend_to_be_on_free_list(this: &UnsafeBox<Self>) {
        debug_assert_eq!(this.next_free.load(Ordering::Relaxed), ptr::null_mut());
        this.refcount.fetch_add(1, Ordering::Relaxed);
        this.next_free.store(Self::DANGLING_PTR, Ordering::Release);
    }

    fn as_mut_ptr(&self) -> *mut RuleNode {
        self as *const RuleNode as *mut RuleNode
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
            // Sound to call because we read-locked the parent's children.
            return unsafe { child.upgrade() };
        }
        let mut children = RwLockUpgradableReadGuard::upgrade(children);
        match children.entry(key, |node| node.p.key()) {
            Entry::Occupied(child) => {
                // Sound to call because we write-locked the parent's children.
                unsafe { child.upgrade() }
            },
            Entry::Vacant(entry) => unsafe {
                let node = StrongRuleNode::new(Box::new(RuleNode::new(
                    root.downgrade(),
                    self.clone(),
                    source,
                    level,
                )));
                // Sound to call because we still own a strong reference to
                // this node, through the `node` variable itself that we are
                // going to return to the caller.
                entry.insert(node.downgrade());
                node
            },
        }
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
    /// purposes.
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
            unsafe {
                child
                    .upgrade()
                    .dump(guards, writer, indent + INDENT_INCREMENT);
            }
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
            node.refcount.fetch_sub(1, Ordering::Release) == 1
        };

        if !should_drop {
            // The refcount didn't even drop zero yet, there is nothing for us
            // to do anymore.
            return;
        }

        unsafe {
            if node.root.is_some() {
                // This is a non-root node and we just observed the refcount
                // dropping to zero, we need to pretend to be on the free list
                // to unstuck any thread who tried to resurrect this node first
                // through `WeakRuleNode::upgrade`.
                RuleNode::pretend_to_be_on_free_list(&self.p);

                // Attempt to push the node on the free list. This may fail
                // if the free list is gone.
                if RuleNode::push_on_free_list(&self.p) {
                    return;
                }
            }

            // Either this was the last reference of the root node, or the
            // tree rule is gone and there is no free list anymore. Drop the
            // node.
            RuleNode::drop_without_free_list(&mut self.p);
        }
    }
}

impl WeakRuleNode {
    /// Upgrades this weak node reference, returning a strong one.
    ///
    /// Must be called with items stored in a node's children list. The children
    /// list must at least be read-locked when this is called.
    unsafe fn upgrade(&self) -> StrongRuleNode {
        debug!("Upgrading weak node: {:p}", &*self.p);

        if self.p.refcount.fetch_add(1, Ordering::Relaxed) == 0 {
            // We observed a refcount of 0, we need to wait for this node to
            // be put on the free list. Resetting the `next_free` pointer to
            // null is only done in `RuleNode::drop_without_free_list`, just
            // before a release refcount decrement, so this acquire fence here
            // makes sure that we observed the write to null before we loop
            // until there is a non-null value.
            atomic::fence(Ordering::Acquire);
            while self.p.next_free.load(Ordering::Relaxed).is_null() {}
        }
        StrongRuleNode::from_unsafe_box(UnsafeBox::clone(&self.p))
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
