/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use crate::properties::Importance;
use crate::shared_lock::StylesheetGuards;
use crate::thread_state;
use malloc_size_of::{MallocShallowSizeOf, MallocSizeOf, MallocSizeOfOps};
use parking_lot::RwLock;
use smallvec::SmallVec;
use std::fmt;
use std::hash;
use std::io::Write;
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

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
        // GC the rule tree.
        unsafe {
            self.gc();
        }

        // After the GC, the free list should be empty.
        debug_assert_eq!(
            self.root.p.next_free.load(Ordering::Relaxed),
            FREE_LIST_SENTINEL
        );

        // Remove the sentinel. This indicates that GCs will no longer occur.
        // Any further drops of StrongRuleNodes must occur on the main thread,
        // and will trigger synchronous dropping of the Rule nodes.
        self.root
            .p
            .next_free
            .store(ptr::null_mut(), Ordering::Relaxed);
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

/// This value exists here so a node that pushes itself to the list can know
/// that is in the free list by looking at is next pointer, and comparing it
/// with null.
///
/// The root node doesn't have a null pointer in the free list, but this value.
const FREE_LIST_SENTINEL: *mut RuleNode = 0x01 as *mut RuleNode;

/// A second sentinel value for the free list, indicating that it's locked (i.e.
/// another thread is currently adding an entry). We spin if we find this value.
const FREE_LIST_LOCKED: *mut RuleNode = 0x02 as *mut RuleNode;

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
    pub unsafe fn gc(&self) {
        self.root.gc();
    }

    /// This can only be called when no other threads is accessing this tree.
    pub unsafe fn maybe_gc(&self) {
        #[cfg(debug_assertions)]
        self.maybe_dump_stats();

        self.root.maybe_gc();
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
            next_free: AtomicPtr::new(FREE_LIST_SENTINEL),
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

    fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    fn free_count(&self) -> &AtomicUsize {
        debug_assert!(self.is_root());
        &self.free_count
    }

    /// Remove this rule node from the child list.
    ///
    /// This is expected to be called before freeing the node from the free
    /// list, on the main thread.
    unsafe fn remove_from_child_list(&self) {
        debug!(
            "Remove from child list: {:?}, parent: {:?}",
            self as *const RuleNode,
            self.parent.as_ref().map(|p| &*p.p as *const RuleNode)
        );

        if let Some(parent) = self.parent.as_ref() {
            let weak = parent
                .p
                .children
                .write()
                .remove(&self.key(), |node| node.p.key());
            assert_eq!(&*weak.unwrap().p as *const _, self as *const _);
        }
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
            return child.upgrade();
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

    unsafe fn pop_from_free_list(&self) -> Option<WeakRuleNode> {
        // NB: This can run from the root node destructor, so we can't use
        // `get()`, since it asserts the refcount is bigger than zero.
        let me = &self.p;

        debug_assert!(me.is_root());

        // FIXME(#14213): Apparently the layout data can be gone from script.
        //
        // That's... suspicious, but it's fine if it happens for the rule tree
        // case, so just don't crash in the case we're doing the final GC in
        // script.

        debug_assert!(
            !thread_state::get().is_worker() &&
                (thread_state::get().is_layout() || thread_state::get().is_script())
        );

        let current = me.next_free.load(Ordering::Relaxed);
        if current == FREE_LIST_SENTINEL {
            return None;
        }

        debug_assert!(
            !current.is_null(),
            "Multiple threads are operating on the free list at the \
             same time?"
        );
        debug_assert!(
            current != &*self.p as *const RuleNode as *mut RuleNode,
            "How did the root end up in the free list?"
        );

        let next = (*current)
            .next_free
            .swap(ptr::null_mut(), Ordering::Relaxed);

        debug_assert!(
            !next.is_null(),
            "How did a null pointer end up in the free list?"
        );

        me.next_free.store(next, Ordering::Relaxed);

        debug!(
            "Popping from free list: cur: {:?}, next: {:?}",
            current, next
        );

        Some(WeakRuleNode {
            p: UnsafeBox::from_raw(current),
        })
    }

    unsafe fn assert_free_list_has_no_duplicates_or_null(&self) {
        assert!(cfg!(debug_assertions), "This is an expensive check!");
        use crate::hash::FxHashSet;

        assert!(self.p.is_root());

        let mut current = &*self.p as *const RuleNode as *mut RuleNode;
        let mut seen = FxHashSet::default();
        while current != FREE_LIST_SENTINEL {
            let next = (*current).next_free.load(Ordering::Relaxed);
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
        let me = &self.p;

        debug_assert!(me.is_root(), "Can't call GC on a non-root node!");

        while let Some(mut weak) = self.pop_from_free_list() {
            if weak.p.refcount.load(Ordering::Relaxed) != 0 {
                // Nothing to do, the node is still alive.
                continue;
            }

            debug!("GC'ing {:?}", &*weak.p as *const RuleNode);
            weak.p.remove_from_child_list();
            log_drop(&*weak.p);
            UnsafeBox::drop(&mut weak.p);
        }

        me.free_count().store(0, Ordering::Relaxed);

        debug_assert_eq!(me.next_free.load(Ordering::Relaxed), FREE_LIST_SENTINEL);
    }

    unsafe fn maybe_gc(&self) {
        debug_assert!(self.p.is_root(), "Can't call GC on a non-root node!");
        if self.p.free_count.load(Ordering::Relaxed) > RULE_TREE_GC_INTERVAL {
            self.gc();
        }
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

        if node.parent.is_none() {
            debug!("Dropping root node!");
            // The free list should be null by this point
            debug_assert!(self.p.next_free.load(Ordering::Relaxed).is_null());
            log_drop(&*self.p);
            unsafe { UnsafeBox::drop(&mut self.p) };
            return;
        }

        let root = &node.root.as_ref().unwrap().p;
        let free_list = &root.next_free;
        let mut old_head = free_list.load(Ordering::Relaxed);

        // If the free list is null, that means that the rule tree has been
        // formally torn down, and the last standard GC has already occurred.
        // We require that any callers using the rule tree at this point are
        // on the main thread only, which lets us trigger a synchronous GC
        // here to avoid leaking anything. We use the GC machinery, rather
        // than just dropping directly, so that we benefit from the iterative
        // destruction and don't trigger unbounded recursion during drop. See
        // [1] and the associated crashtest.
        //
        // [1] https://bugzilla.mozilla.org/show_bug.cgi?id=439184
        if old_head.is_null() {
            debug_assert!(
                !thread_state::get().is_worker() &&
                    (thread_state::get().is_layout() || thread_state::get().is_script())
            );
            // Add the node as the sole entry in the free list.
            debug_assert!(node.next_free.load(Ordering::Relaxed).is_null());
            node.next_free.store(FREE_LIST_SENTINEL, Ordering::Relaxed);
            free_list.store(node as *const _ as *mut _, Ordering::Relaxed);

            // Invoke the GC.
            //
            // Note that we need hold a strong reference to the root so that it
            // doesn't go away during the GC (which would happen if we're freeing
            // the last external reference into the rule tree). This is nicely
            // enforced by having the gc() method live on StrongRuleNode rather than
            // RuleNode.
            let strong_root: StrongRuleNode = node.root.as_ref().unwrap().upgrade();
            unsafe {
                strong_root.gc();
            }

            // Leave the free list null, like we found it, such that additional
            // drops for straggling rule nodes will take this same codepath.
            debug_assert_eq!(root.next_free.load(Ordering::Relaxed), FREE_LIST_SENTINEL);
            root.next_free.store(ptr::null_mut(), Ordering::Relaxed);

            // Return. If strong_root is the last strong reference to the root,
            // this re-enter StrongRuleNode::drop, and take the root-dropping
            // path earlier in this function.
            return;
        }

        // We're sure we're already in the free list, don't spinloop if we're.
        // Note that this is just a fast path, so it doesn't need to have an
        // strong memory ordering.
        if node.next_free.load(Ordering::Relaxed) != ptr::null_mut() {
            return;
        }

        // Ensure we "lock" the free list head swapping it with FREE_LIST_LOCKED.
        //
        // Note that we use Acquire/Release semantics for the free list
        // synchronization, in order to guarantee that the next_free
        // reads/writes we do below are properly visible from multiple threads
        // racing.
        loop {
            match free_list.compare_exchange_weak(
                old_head,
                FREE_LIST_LOCKED,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(..) => {
                    if old_head != FREE_LIST_LOCKED {
                        break;
                    }
                },
                Err(new) => old_head = new,
            }
        }

        // If other thread has raced with use while using the same rule node,
        // just store the old head again, we're done.
        //
        // Note that we can use relaxed operations for loading since we're
        // effectively locking the free list with Acquire/Release semantics, and
        // the memory ordering is already guaranteed by that locking/unlocking.
        if node.next_free.load(Ordering::Relaxed) != ptr::null_mut() {
            free_list.store(old_head, Ordering::Release);
            return;
        }

        // Else store the old head as the next pointer, and store ourselves as
        // the new head of the free list.
        //
        // This can be relaxed since this pointer won't be read until GC.
        node.next_free.store(old_head, Ordering::Relaxed);

        // Increment the free count. This doesn't need to be an RMU atomic
        // operation, because the free list is "locked".
        let old_free_count = root.free_count().load(Ordering::Relaxed);
        root.free_count()
            .store(old_free_count + 1, Ordering::Relaxed);

        // This can be release because of the locking of the free list, that
        // ensures that all the other nodes racing with this one are using
        // `Acquire`.
        free_list.store(
            &*self.p as *const RuleNode as *mut RuleNode,
            Ordering::Release,
        );
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
