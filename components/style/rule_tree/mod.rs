/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

//! The rule tree.

use crate::applicable_declarations::ApplicableDeclarationList;
#[cfg(feature = "gecko")]
use crate::gecko::selector_parser::PseudoElement;
use crate::hash::{self, FxHashMap};
use crate::properties::{Importance, LonghandIdSet, PropertyDeclarationBlock};
use crate::shared_lock::{Locked, SharedRwLockReadGuard, StylesheetGuards};
use crate::stylesheets::{Origin, StyleRule};
use crate::thread_state;
use malloc_size_of::{MallocShallowSizeOf, MallocSizeOf, MallocSizeOfOps};
use parking_lot::RwLock;
use servo_arc::{Arc, ArcBorrow, ArcUnion, ArcUnionBorrow};
use smallvec::SmallVec;
use std::io::{self, Write};
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

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
            self.root.get().next_free.load(Ordering::Relaxed),
            FREE_LIST_SENTINEL
        );

        // Remove the sentinel. This indicates that GCs will no longer occur.
        // Any further drops of StrongRuleNodes must occur on the main thread,
        // and will trigger synchronous dropping of the Rule nodes.
        self.root
            .get()
            .next_free
            .store(ptr::null_mut(), Ordering::Relaxed);
    }
}

impl MallocSizeOf for RuleTree {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut n = 0;
        let mut stack = SmallVec::<[_; 32]>::new();
        stack.push(self.root.downgrade());

        while let Some(node) = stack.pop() {
            n += unsafe { ops.malloc_size_of(node.ptr()) };
            let children = unsafe { (*node.ptr()).children.read() };
            children.shallow_size_of(ops);
            children.each(|c| stack.push(c.clone()));
        }

        n
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ChildKey(CascadeLevel, ptr::NonNull<()>);

unsafe impl Send for ChildKey {}
unsafe impl Sync for ChildKey {}

/// A style source for the rule node. It can either be a CSS style rule or a
/// declaration block.
///
/// Note that, even though the declaration block from inside the style rule
/// could be enough to implement the rule tree, keeping the whole rule provides
/// more debuggability, and also the ability of show those selectors to
/// devtools.
#[derive(Clone, Debug)]
pub struct StyleSource(ArcUnion<Locked<StyleRule>, Locked<PropertyDeclarationBlock>>);

impl PartialEq for StyleSource {
    fn eq(&self, other: &Self) -> bool {
        ArcUnion::ptr_eq(&self.0, &other.0)
    }
}

impl StyleSource {
    /// Creates a StyleSource from a StyleRule.
    pub fn from_rule(rule: Arc<Locked<StyleRule>>) -> Self {
        StyleSource(ArcUnion::from_first(rule))
    }

    #[inline]
    fn key(&self) -> ptr::NonNull<()> {
        self.0.ptr()
    }

    /// Creates a StyleSource from a PropertyDeclarationBlock.
    pub fn from_declarations(decls: Arc<Locked<PropertyDeclarationBlock>>) -> Self {
        StyleSource(ArcUnion::from_second(decls))
    }

    fn dump<W: Write>(&self, guard: &SharedRwLockReadGuard, writer: &mut W) {
        if let Some(ref rule) = self.0.as_first() {
            let rule = rule.read_with(guard);
            let _ = write!(writer, "{:?}", rule.selectors);
        }

        let _ = write!(writer, "  -> {:?}", self.read(guard).declarations());
    }

    /// Read the style source guard, and obtain thus read access to the
    /// underlying property declaration block.
    #[inline]
    pub fn read<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> &'a PropertyDeclarationBlock {
        let block: &Locked<PropertyDeclarationBlock> = match self.0.borrow() {
            ArcUnionBorrow::First(ref rule) => &rule.get().read_with(guard).block,
            ArcUnionBorrow::Second(ref block) => block.get(),
        };
        block.read_with(guard)
    }

    /// Returns the style rule if applicable, otherwise None.
    pub fn as_rule(&self) -> Option<ArcBorrow<Locked<StyleRule>>> {
        self.0.as_first()
    }

    /// Returns the declaration block if applicable, otherwise None.
    pub fn as_declarations(&self) -> Option<ArcBorrow<Locked<PropertyDeclarationBlock>>> {
        self.0.as_second()
    }
}

/// This value exists here so a node that pushes itself to the list can know
/// that is in the free list by looking at is next pointer, and comparing it
/// with null.
///
/// The root node doesn't have a null pointer in the free list, but this value.
const FREE_LIST_SENTINEL: *mut RuleNode = 0x01 as *mut RuleNode;

/// A second sentinel value for the free list, indicating that it's locked (i.e.
/// another thread is currently adding an entry). We spin if we find this value.
const FREE_LIST_LOCKED: *mut RuleNode = 0x02 as *mut RuleNode;

/// A counter to track how many inner shadow roots rules deep we are.
///
/// This is used to handle:
///
/// https://drafts.csswg.org/css-scoping/#shadow-cascading
///
/// In particular, it'd be `0` for the innermost shadow host, `1` for the next,
/// and so on.
pub type ShadowCascadeOrder = u8;

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

    fn dump<W: Write>(&self, guards: &StylesheetGuards, writer: &mut W) {
        let _ = writeln!(writer, " + RuleTree");
        self.root.get().dump(guards, writer, 0);
    }

    /// Dump the rule tree to stdout.
    pub fn dump_stdout(&self, guards: &StylesheetGuards) {
        let mut stdout = io::stdout();
        self.dump(guards, &mut stdout);
    }

    /// Inserts the given rules, that must be in proper order by specifity, and
    /// returns the corresponding rule node representing the last inserted one.
    ///
    /// !important rules are detected and inserted into the appropriate position
    /// in the rule tree. This allows selector matching to ignore importance,
    /// while still maintaining the appropriate cascade order in the rule tree.
    pub fn insert_ordered_rules_with_important<'a, I>(
        &self,
        iter: I,
        guards: &StylesheetGuards,
    ) -> StrongRuleNode
    where
        I: Iterator<Item = (StyleSource, CascadeLevel, ShadowCascadeOrder)>,
    {
        use self::CascadeLevel::*;
        let mut current = self.root.clone();
        let mut last_level = current.get().level;

        let mut found_important = false;
        let mut important_style_attr = None;

        let mut important_same_tree = SmallVec::<[StyleSource; 4]>::new();
        let mut important_inner_shadow = SmallVec::<[SmallVec<[StyleSource; 4]>; 4]>::new();
        important_inner_shadow.push(SmallVec::new());

        let mut important_user = SmallVec::<[StyleSource; 4]>::new();
        let mut important_ua = SmallVec::<[StyleSource; 4]>::new();
        let mut transition = None;

        let mut last_cascade_order = 0;
        for (source, level, shadow_cascade_order) in iter {
            debug_assert!(level >= last_level, "Not really ordered");
            debug_assert!(!level.is_important(), "Important levels handled internally");
            let any_important = {
                let pdb = source.read(level.guard(guards));
                pdb.any_important()
            };

            if any_important {
                found_important = true;
                match level {
                    InnerShadowNormal => {
                        debug_assert!(
                            shadow_cascade_order >= last_cascade_order,
                            "Not really ordered"
                        );
                        if shadow_cascade_order > last_cascade_order &&
                            !important_inner_shadow.last().unwrap().is_empty()
                        {
                            last_cascade_order = shadow_cascade_order;
                            important_inner_shadow.push(SmallVec::new());
                        }
                        important_inner_shadow
                            .last_mut()
                            .unwrap()
                            .push(source.clone())
                    },
                    SameTreeAuthorNormal => important_same_tree.push(source.clone()),
                    UANormal => important_ua.push(source.clone()),
                    UserNormal => important_user.push(source.clone()),
                    StyleAttributeNormal => {
                        debug_assert!(important_style_attr.is_none());
                        important_style_attr = Some(source.clone());
                    },
                    _ => {},
                };
            }

            // We don't optimize out empty rules, even though we could.
            //
            // Inspector relies on every rule being inserted in the normal level
            // at least once, in order to return the rules with the correct
            // specificity order.
            //
            // TODO(emilio): If we want to apply these optimizations without
            // breaking inspector's expectations, we'd need to run
            // selector-matching again at the inspector's request. That may or
            // may not be a better trade-off.
            if matches!(level, Transitions) && found_important {
                // There can be at most one transition, and it will come at
                // the end of the iterator. Stash it and apply it after
                // !important rules.
                debug_assert!(transition.is_none());
                transition = Some(source);
            } else {
                current = current.ensure_child(self.root.downgrade(), source, level);
            }
            last_level = level;
        }

        // Early-return in the common case of no !important declarations.
        if !found_important {
            return current;
        }

        //
        // Insert important declarations, in order of increasing importance,
        // followed by any transition rule.
        //

        for source in important_same_tree.drain() {
            current = current.ensure_child(self.root.downgrade(), source, SameTreeAuthorImportant);
        }

        if let Some(source) = important_style_attr {
            current = current.ensure_child(self.root.downgrade(), source, StyleAttributeImportant);
        }

        for mut list in important_inner_shadow.drain().rev() {
            for source in list.drain() {
                current = current.ensure_child(self.root.downgrade(), source, InnerShadowImportant);
            }
        }

        for source in important_user.drain() {
            current = current.ensure_child(self.root.downgrade(), source, UserImportant);
        }

        for source in important_ua.drain() {
            current = current.ensure_child(self.root.downgrade(), source, UAImportant);
        }

        if let Some(source) = transition {
            current = current.ensure_child(self.root.downgrade(), source, Transitions);
        }

        current
    }

    /// Given a list of applicable declarations, insert the rules and return the
    /// corresponding rule node.
    pub fn compute_rule_node(
        &self,
        applicable_declarations: &mut ApplicableDeclarationList,
        guards: &StylesheetGuards,
    ) -> StrongRuleNode {
        self.insert_ordered_rules_with_important(
            applicable_declarations.drain().map(|d| d.for_rule_tree()),
            guards,
        )
    }

    /// Insert the given rules, that must be in proper order by specifity, and
    /// return the corresponding rule node representing the last inserted one.
    pub fn insert_ordered_rules<'a, I>(&self, iter: I) -> StrongRuleNode
    where
        I: Iterator<Item = (StyleSource, CascadeLevel)>,
    {
        self.insert_ordered_rules_from(self.root.clone(), iter)
    }

    fn insert_ordered_rules_from<'a, I>(&self, from: StrongRuleNode, iter: I) -> StrongRuleNode
    where
        I: Iterator<Item = (StyleSource, CascadeLevel)>,
    {
        let mut current = from;
        let mut last_level = current.get().level;
        for (source, level) in iter {
            debug_assert!(last_level <= level, "Not really ordered");
            current = current.ensure_child(self.root.downgrade(), source, level);
            last_level = level;
        }
        current
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

        let mut children_count = FxHashMap::default();

        let mut stack = SmallVec::<[_; 32]>::new();
        stack.push(self.root.clone());
        while let Some(node) = stack.pop() {
            let children = node.get().children.read();
            *children_count.entry(children.len()).or_insert(0) += 1;
            children.each(|c| stack.push(c.upgrade()));
        }

        trace!("Rule tree stats:");
        let counts = children_count.keys().sorted();
        for count in counts {
            trace!(" {} - {}", count, children_count[count]);
        }
    }

    /// Replaces a rule in a given level (if present) for another rule.
    ///
    /// Returns the resulting node that represents the new path, or None if
    /// the old path is still valid.
    pub fn update_rule_at_level(
        &self,
        level: CascadeLevel,
        pdb: Option<ArcBorrow<Locked<PropertyDeclarationBlock>>>,
        path: &StrongRuleNode,
        guards: &StylesheetGuards,
        important_rules_changed: &mut bool,
    ) -> Option<StrongRuleNode> {
        debug_assert!(level.is_unique_per_element());
        // TODO(emilio): Being smarter with lifetimes we could avoid a bit of
        // the refcount churn.
        let mut current = path.clone();
        *important_rules_changed = false;

        // First walk up until the first less-or-equally specific rule.
        let mut children = SmallVec::<[_; 10]>::new();
        while current.get().level > level {
            children.push((
                current.get().source.as_ref().unwrap().clone(),
                current.get().level,
            ));
            current = current.parent().unwrap().clone();
        }

        // Then remove the one at the level we want to replace, if any.
        //
        // NOTE: Here we assume that only one rule can be at the level we're
        // replacing.
        //
        // This is certainly true for HTML style attribute rules, animations and
        // transitions, but could not be so for SMIL animations, which we'd need
        // to special-case (isn't hard, it's just about removing the `if` and
        // special cases, and replacing them for a `while` loop, avoiding the
        // optimizations).
        if current.get().level == level {
            *important_rules_changed |= level.is_important();

            if let Some(pdb) = pdb {
                // If the only rule at the level we're replacing is exactly the
                // same as `pdb`, we're done, and `path` is still valid.
                //
                // TODO(emilio): Another potential optimization is the one where
                // we can just replace the rule at that level for `pdb`, and
                // then we don't need to re-create the children, and `path` is
                // also equally valid. This is less likely, and would require an
                // in-place mutation of the source, which is, at best, fiddly,
                // so let's skip it for now.
                let current_decls = current
                    .get()
                    .source
                    .as_ref()
                    .unwrap()
                    .as_declarations()
                    .expect("Replacing non-declarations style?");
                let is_here_already = ArcBorrow::ptr_eq(&pdb, &current_decls);
                if is_here_already {
                    debug!("Picking the fast path in rule replacement");
                    return None;
                }
            }
            current = current.parent().unwrap().clone();
        }
        debug_assert!(
            current.get().level != level,
            "Multiple rules should've been replaced?"
        );

        // Insert the rule if it's relevant at this level in the cascade.
        //
        // These optimizations are likely to be important, because the levels
        // where replacements apply (style and animations) tend to trigger
        // pretty bad styling cases already.
        if let Some(pdb) = pdb {
            if level.is_important() {
                if pdb.read_with(level.guard(guards)).any_important() {
                    current = current.ensure_child(
                        self.root.downgrade(),
                        StyleSource::from_declarations(pdb.clone_arc()),
                        level,
                    );
                    *important_rules_changed = true;
                }
            } else {
                if pdb.read_with(level.guard(guards)).any_normal() {
                    current = current.ensure_child(
                        self.root.downgrade(),
                        StyleSource::from_declarations(pdb.clone_arc()),
                        level,
                    );
                }
            }
        }

        // Now the rule is in the relevant place, push the children as
        // necessary.
        let rule = self.insert_ordered_rules_from(current, children.drain().rev());
        Some(rule)
    }

    /// Returns new rule nodes without Transitions level rule.
    pub fn remove_transition_rule_if_applicable(&self, path: &StrongRuleNode) -> StrongRuleNode {
        // Return a clone if there is no transition level.
        if path.cascade_level() != CascadeLevel::Transitions {
            return path.clone();
        }

        path.parent().unwrap().clone()
    }

    /// Returns new rule node without rules from declarative animations.
    pub fn remove_animation_rules(&self, path: &StrongRuleNode) -> StrongRuleNode {
        // Return a clone if there are no animation rules.
        if !path.has_animation_or_transition_rules() {
            return path.clone();
        }

        let iter = path
            .self_and_ancestors()
            .take_while(|node| node.cascade_level() >= CascadeLevel::SMILOverride);
        let mut last = path;
        let mut children = SmallVec::<[_; 10]>::new();
        for node in iter {
            if !node.cascade_level().is_animation() {
                children.push((
                    node.get().source.as_ref().unwrap().clone(),
                    node.cascade_level(),
                ));
            }
            last = node;
        }

        let rule =
            self.insert_ordered_rules_from(last.parent().unwrap().clone(), children.drain().rev());
        rule
    }

    /// Returns new rule node by adding animation rules at transition level.
    /// The additional rules must be appropriate for the transition
    /// level of the cascade, which is the highest level of the cascade.
    /// (This is the case for one current caller, the cover rule used
    /// for CSS transitions.)
    pub fn add_animation_rules_at_transition_level(
        &self,
        path: &StrongRuleNode,
        pdb: Arc<Locked<PropertyDeclarationBlock>>,
        guards: &StylesheetGuards,
    ) -> StrongRuleNode {
        let mut dummy = false;
        self.update_rule_at_level(
            CascadeLevel::Transitions,
            Some(pdb.borrow_arc()),
            path,
            guards,
            &mut dummy,
        )
        .expect("Should return a valid rule node")
    }
}

/// The number of RuleNodes added to the free list before we will consider
/// doing a GC when calling maybe_gc().  (The value is copied from Gecko,
/// where it likely did not result from a rigorous performance analysis.)
const RULE_TREE_GC_INTERVAL: usize = 300;

/// The cascade level these rules are relevant at, as per[1][2][3].
///
/// Presentational hints for SVG and HTML are in the "author-level
/// zero-specificity" level, that is, right after user rules, and before author
/// rules.
///
/// The order of variants declared here is significant, and must be in
/// _ascending_ order of precedence.
///
/// See also [4] for the Shadow DOM bits. We rely on the invariant that rules
/// from outside the tree the element is in can't affect the element.
///
/// The opposite is not true (i.e., :host and ::slotted) from an "inner" shadow
/// tree may affect an element connected to the document or an "outer" shadow
/// tree.
///
/// We need to differentiate between rules from the same tree and "inner" shadow
/// trees in order to be able to find the right position for the style attribute
/// easily. Otherwise we wouldn't be able to avoid selector-matching when a
/// style attribute is added or removed.
///
/// [1]: https://drafts.csswg.org/css-cascade/#cascade-origin
/// [2]: https://drafts.csswg.org/css-cascade/#preshint
/// [3]: https://html.spec.whatwg.org/multipage/#presentational-hints
/// [4]: https://drafts.csswg.org/css-scoping/#shadow-cascading
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
pub enum CascadeLevel {
    /// Normal User-Agent rules.
    UANormal = 0,
    /// User normal rules.
    UserNormal,
    /// Presentational hints.
    PresHints,
    /// Shadow DOM styles from "inner" shadow trees.
    ///
    /// See above for why this is needed instead of merging InnerShadowNormal,
    /// SameTreeAuthorNormal and StyleAttributeNormal inside something like
    /// AuthorNormal.
    InnerShadowNormal,
    /// Author normal rules from the same tree the element is in.
    SameTreeAuthorNormal,
    /// Style attribute normal rules.
    StyleAttributeNormal,
    /// SVG SMIL animations.
    SMILOverride,
    /// CSS animations and script-generated animations.
    Animations,
    /// Author-supplied important rules from the same tree the element came
    /// from.
    SameTreeAuthorImportant,
    /// Style attribute important rules.
    StyleAttributeImportant,
    /// Shadow DOM important rules.
    InnerShadowImportant,
    /// User important rules.
    UserImportant,
    /// User-agent important rules.
    UAImportant,
    /// Transitions
    ///
    /// NB: If this changes from being last, change from_byte below.
    Transitions,
}

impl CascadeLevel {
    /// Converts a raw byte to a CascadeLevel.
    pub unsafe fn from_byte(byte: u8) -> Self {
        debug_assert!(byte <= CascadeLevel::Transitions as u8);
        mem::transmute(byte)
    }

    /// Select a lock guard for this level
    pub fn guard<'a>(&self, guards: &'a StylesheetGuards<'a>) -> &'a SharedRwLockReadGuard<'a> {
        match *self {
            CascadeLevel::UANormal |
            CascadeLevel::UserNormal |
            CascadeLevel::UserImportant |
            CascadeLevel::UAImportant => guards.ua_or_user,
            _ => guards.author,
        }
    }

    /// Returns whether this cascade level is unique per element, in which case
    /// we can replace the path in the cascade without fear.
    pub fn is_unique_per_element(&self) -> bool {
        match *self {
            CascadeLevel::Transitions |
            CascadeLevel::Animations |
            CascadeLevel::SMILOverride |
            CascadeLevel::StyleAttributeNormal |
            CascadeLevel::StyleAttributeImportant => true,
            _ => false,
        }
    }

    /// Returns whether this cascade level represents important rules of some
    /// sort.
    #[inline]
    pub fn is_important(&self) -> bool {
        match *self {
            CascadeLevel::SameTreeAuthorImportant |
            CascadeLevel::InnerShadowImportant |
            CascadeLevel::StyleAttributeImportant |
            CascadeLevel::UserImportant |
            CascadeLevel::UAImportant => true,
            _ => false,
        }
    }

    /// Returns the importance relevant for this rule. Pretty similar to
    /// `is_important`.
    #[inline]
    pub fn importance(&self) -> Importance {
        if self.is_important() {
            Importance::Important
        } else {
            Importance::Normal
        }
    }

    /// Returns the cascade origin of the rule.
    #[inline]
    pub fn origin(&self) -> Origin {
        match *self {
            CascadeLevel::UAImportant | CascadeLevel::UANormal => Origin::UserAgent,
            CascadeLevel::UserImportant | CascadeLevel::UserNormal => Origin::User,
            CascadeLevel::PresHints |
            CascadeLevel::InnerShadowNormal |
            CascadeLevel::SameTreeAuthorNormal |
            CascadeLevel::StyleAttributeNormal |
            CascadeLevel::SMILOverride |
            CascadeLevel::Animations |
            CascadeLevel::SameTreeAuthorImportant |
            CascadeLevel::StyleAttributeImportant |
            CascadeLevel::InnerShadowImportant |
            CascadeLevel::Transitions => Origin::Author,
        }
    }

    /// Returns whether this cascade level represents an animation rules.
    #[inline]
    pub fn is_animation(&self) -> bool {
        match *self {
            CascadeLevel::SMILOverride | CascadeLevel::Animations | CascadeLevel::Transitions => {
                true
            },
            _ => false,
        }
    }
}

/// The children of a single rule node.
///
/// We optimize the case of no kids and a single child, since they're by far the
/// most common case and it'd cause a bunch of bloat for no reason.
///
/// The children remove themselves when they go away, which means that it's ok
/// for us to store weak pointers to them.
enum RuleNodeChildren {
    /// There are no kids.
    Empty,
    /// There's just one kid. This is an extremely common case, so we don't
    /// bother allocating a map for it.
    One(WeakRuleNode),
    /// At least at one point in time there was more than one kid (that is to
    /// say, we don't bother re-allocating if children are removed dynamically).
    Map(Box<FxHashMap<ChildKey, WeakRuleNode>>),
}

impl MallocShallowSizeOf for RuleNodeChildren {
    fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        match *self {
            RuleNodeChildren::One(..) | RuleNodeChildren::Empty => 0,
            RuleNodeChildren::Map(ref m) => {
                // Want to account for both the box and the hashmap.
                m.shallow_size_of(ops) + (**m).shallow_size_of(ops)
            },
        }
    }
}

impl Default for RuleNodeChildren {
    fn default() -> Self {
        RuleNodeChildren::Empty
    }
}

impl RuleNodeChildren {
    /// Executes a given function for each of the children.
    fn each(&self, mut f: impl FnMut(&WeakRuleNode)) {
        match *self {
            RuleNodeChildren::Empty => {},
            RuleNodeChildren::One(ref child) => f(child),
            RuleNodeChildren::Map(ref map) => {
                for (_key, kid) in map.iter() {
                    f(kid)
                }
            },
        }
    }

    fn len(&self) -> usize {
        match *self {
            RuleNodeChildren::Empty => 0,
            RuleNodeChildren::One(..) => 1,
            RuleNodeChildren::Map(ref map) => map.len(),
        }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn get(&self, key: &ChildKey) -> Option<&WeakRuleNode> {
        match *self {
            RuleNodeChildren::Empty => return None,
            RuleNodeChildren::One(ref kid) => {
                // We're read-locked, so no need to do refcount stuff, since the
                // child is only removed from the main thread, _and_ it'd need
                // to write-lock us anyway.
                if unsafe { (*kid.ptr()).key() } == *key {
                    Some(kid)
                } else {
                    None
                }
            },
            RuleNodeChildren::Map(ref map) => map.get(&key),
        }
    }

    fn get_or_insert_with(
        &mut self,
        key: ChildKey,
        get_new_child: impl FnOnce() -> StrongRuleNode,
    ) -> StrongRuleNode {
        let existing_child_key = match *self {
            RuleNodeChildren::Empty => {
                let new = get_new_child();
                debug_assert_eq!(new.get().key(), key);
                *self = RuleNodeChildren::One(new.downgrade());
                return new;
            },
            RuleNodeChildren::One(ref weak) => unsafe {
                // We're locked necessarily, so it's fine to look at our
                // weak-child without refcount-traffic.
                let existing_child_key = (*weak.ptr()).key();
                if existing_child_key == key {
                    return weak.upgrade();
                }
                existing_child_key
            },
            RuleNodeChildren::Map(ref mut map) => {
                return match map.entry(key) {
                    hash::map::Entry::Occupied(ref occupied) => occupied.get().upgrade(),
                    hash::map::Entry::Vacant(vacant) => {
                        let new = get_new_child();

                        debug_assert_eq!(new.get().key(), key);
                        vacant.insert(new.downgrade());

                        new
                    },
                };
            },
        };

        let existing_child = match mem::replace(self, RuleNodeChildren::Empty) {
            RuleNodeChildren::One(o) => o,
            _ => unreachable!(),
        };
        // Two rule-nodes are still a not-totally-uncommon thing, so
        // avoid over-allocating entries.
        //
        // TODO(emilio): Maybe just inline two kids too?
        let mut children = Box::new(FxHashMap::with_capacity_and_hasher(2, Default::default()));
        children.insert(existing_child_key, existing_child);

        let new = get_new_child();
        debug_assert_eq!(new.get().key(), key);
        children.insert(key, new.downgrade());

        *self = RuleNodeChildren::Map(children);

        new
    }

    fn remove(&mut self, key: &ChildKey) -> Option<WeakRuleNode> {
        match *self {
            RuleNodeChildren::Empty => return None,
            RuleNodeChildren::One(ref one) => {
                if unsafe { (*one.ptr()).key() } != *key {
                    return None;
                }
            },
            RuleNodeChildren::Map(ref mut multiple) => {
                return multiple.remove(key);
            },
        }

        match mem::replace(self, RuleNodeChildren::Empty) {
            RuleNodeChildren::One(o) => Some(o),
            _ => unreachable!(),
        }
    }
}

/// A node in the rule tree.
pub struct RuleNode {
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
    children: RwLock<RuleNodeChildren>,

    /// The next item in the rule tree free list, that starts on the root node.
    ///
    /// When this is set to null, that means that the rule tree has been torn
    /// down, and GCs will no longer occur. When this happens, StrongRuleNodes
    /// may only be dropped on the main thread, and teardown happens
    /// synchronously.
    next_free: AtomicPtr<RuleNode>,
}

unsafe impl Sync for RuleTree {}
unsafe impl Send for RuleTree {}

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
    pub fn log_ctor(ptr: *const RuleNode) {
        let s = NAME as *const [u8] as *const u8 as *const c_char;
        unsafe {
            NS_LogCtor(ptr as *mut c_void, s, size_of::<RuleNode>() as u32);
        }
    }

    /// Logs the destruction of a heap-allocated object to Gecko's leak-checking machinery.
    pub fn log_dtor(ptr: *const RuleNode) {
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
            self.parent.as_ref().map(|p| p.ptr())
        );

        if let Some(parent) = self.parent.as_ref() {
            let weak = parent.get().children.write().remove(&self.key());
            assert_eq!(weak.unwrap().ptr() as *const _, self as *const _);
        }
    }

    fn dump<W: Write>(&self, guards: &StylesheetGuards, writer: &mut W, indent: usize) {
        const INDENT_INCREMENT: usize = 4;

        for _ in 0..indent {
            let _ = write!(writer, " ");
        }

        let _ = writeln!(
            writer,
            " - {:?} (ref: {:?}, parent: {:?})",
            self as *const _,
            self.refcount.load(Ordering::Relaxed),
            self.parent.as_ref().map(|p| p.ptr())
        );

        for _ in 0..indent {
            let _ = write!(writer, " ");
        }

        if self.source.is_some() {
            self.source
                .as_ref()
                .unwrap()
                .dump(self.level.guard(guards), writer);
        } else {
            if indent != 0 {
                warn!("How has this happened?");
            }
            let _ = write!(writer, "(root)");
        }

        let _ = write!(writer, "\n");
        self.children.read().each(|child| {
            child
                .upgrade()
                .get()
                .dump(guards, writer, indent + INDENT_INCREMENT);
        });
    }
}

#[derive(Clone)]
struct WeakRuleNode {
    p: ptr::NonNull<RuleNode>,
}

/// A strong reference to a rule node.
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct StrongRuleNode {
    p: ptr::NonNull<RuleNode>,
}

unsafe impl Send for StrongRuleNode {}
unsafe impl Sync for StrongRuleNode {}

#[cfg(feature = "servo")]
malloc_size_of_is_0!(StrongRuleNode);

impl StrongRuleNode {
    fn new(n: Box<RuleNode>) -> Self {
        debug_assert_eq!(n.parent.is_none(), !n.source.is_some());

        // TODO(emilio): Use into_raw_non_null when it's stable.
        let ptr = unsafe { ptr::NonNull::new_unchecked(Box::into_raw(n)) };
        log_new(ptr.as_ptr());

        debug!("Creating rule node: {:p}", ptr);

        StrongRuleNode::from_ptr(ptr)
    }

    fn from_ptr(p: ptr::NonNull<RuleNode>) -> Self {
        StrongRuleNode { p }
    }

    fn downgrade(&self) -> WeakRuleNode {
        WeakRuleNode::from_ptr(self.p)
    }

    /// Get the parent rule node of this rule node.
    pub fn parent(&self) -> Option<&StrongRuleNode> {
        self.get().parent.as_ref()
    }

    fn ensure_child(
        &self,
        root: WeakRuleNode,
        source: StyleSource,
        level: CascadeLevel,
    ) -> StrongRuleNode {
        use parking_lot::RwLockUpgradableReadGuard;

        let key = ChildKey(level, source.key());

        let read_guard = self.get().children.upgradable_read();
        if let Some(child) = read_guard.get(&key) {
            return child.upgrade();
        }

        RwLockUpgradableReadGuard::upgrade(read_guard).get_or_insert_with(key, move || {
            StrongRuleNode::new(Box::new(RuleNode::new(
                root,
                self.clone(),
                source.clone(),
                level,
            )))
        })
    }

    /// Raw pointer to the RuleNode
    #[inline]
    pub fn ptr(&self) -> *mut RuleNode {
        self.p.as_ptr()
    }

    fn get(&self) -> &RuleNode {
        if cfg!(debug_assertions) {
            let node = unsafe { &*self.p.as_ptr() };
            assert!(node.refcount.load(Ordering::Relaxed) > 0);
        }
        unsafe { &*self.p.as_ptr() }
    }

    /// Get the style source corresponding to this rule node. May return `None`
    /// if it's the root node, which means that the node hasn't matched any
    /// rules.
    pub fn style_source(&self) -> Option<&StyleSource> {
        self.get().source.as_ref()
    }

    /// The cascade level for this node
    pub fn cascade_level(&self) -> CascadeLevel {
        self.get().level
    }

    /// Get the importance that this rule node represents.
    pub fn importance(&self) -> Importance {
        self.get().level.importance()
    }

    /// Get an iterator for this rule node and its ancestors.
    pub fn self_and_ancestors(&self) -> SelfAndAncestors {
        SelfAndAncestors {
            current: Some(self),
        }
    }

    /// Returns whether this node has any child, only intended for testing
    /// purposes, and called on a single-threaded fashion only.
    pub unsafe fn has_children_for_testing(&self) -> bool {
        !self.get().children.read().is_empty()
    }

    unsafe fn pop_from_free_list(&self) -> Option<WeakRuleNode> {
        // NB: This can run from the root node destructor, so we can't use
        // `get()`, since it asserts the refcount is bigger than zero.
        let me = &*self.p.as_ptr();

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
            current != self.p.as_ptr(),
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

        Some(WeakRuleNode::from_ptr(ptr::NonNull::new_unchecked(current)))
    }

    unsafe fn assert_free_list_has_no_duplicates_or_null(&self) {
        assert!(cfg!(debug_assertions), "This is an expensive check!");
        use crate::hash::FxHashSet;

        let me = &*self.p.as_ptr();
        assert!(me.is_root());

        let mut current = self.p.as_ptr();
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
        let me = &*self.p.as_ptr();

        debug_assert!(me.is_root(), "Can't call GC on a non-root node!");

        while let Some(weak) = self.pop_from_free_list() {
            let node = &*weak.p.as_ptr();
            if node.refcount.load(Ordering::Relaxed) != 0 {
                // Nothing to do, the node is still alive.
                continue;
            }

            debug!("GC'ing {:?}", weak.p.as_ptr());
            node.remove_from_child_list();
            log_drop(weak.p.as_ptr());
            let _ = Box::from_raw(weak.p.as_ptr());
        }

        me.free_count().store(0, Ordering::Relaxed);

        debug_assert_eq!(me.next_free.load(Ordering::Relaxed), FREE_LIST_SENTINEL);
    }

    unsafe fn maybe_gc(&self) {
        debug_assert!(self.get().is_root(), "Can't call GC on a non-root node!");
        if self.get().free_count().load(Ordering::Relaxed) > RULE_TREE_GC_INTERVAL {
            self.gc();
        }
    }

    /// Returns true if any properties specified by `rule_type_mask` was set by
    /// an author rule.
    #[cfg(feature = "gecko")]
    pub fn has_author_specified_rules<E>(
        &self,
        mut element: E,
        mut pseudo: Option<PseudoElement>,
        guards: &StylesheetGuards,
        rule_type_mask: u32,
        author_colors_allowed: bool,
    ) -> bool
    where
        E: crate::dom::TElement,
    {
        use crate::gecko_bindings::structs::NS_AUTHOR_SPECIFIED_BACKGROUND;
        use crate::gecko_bindings::structs::NS_AUTHOR_SPECIFIED_BORDER;
        use crate::gecko_bindings::structs::NS_AUTHOR_SPECIFIED_PADDING;
        use crate::properties::{CSSWideKeyword, LonghandId};
        use crate::properties::{PropertyDeclaration, PropertyDeclarationId};
        use crate::values::specified::Color;
        use std::borrow::Cow;

        // Reset properties:
        const BACKGROUND_PROPS: &'static [LonghandId] =
            &[LonghandId::BackgroundColor, LonghandId::BackgroundImage];

        const BORDER_PROPS: &'static [LonghandId] = &[
            LonghandId::BorderTopColor,
            LonghandId::BorderTopStyle,
            LonghandId::BorderTopWidth,
            LonghandId::BorderRightColor,
            LonghandId::BorderRightStyle,
            LonghandId::BorderRightWidth,
            LonghandId::BorderBottomColor,
            LonghandId::BorderBottomStyle,
            LonghandId::BorderBottomWidth,
            LonghandId::BorderLeftColor,
            LonghandId::BorderLeftStyle,
            LonghandId::BorderLeftWidth,
            LonghandId::BorderTopLeftRadius,
            LonghandId::BorderTopRightRadius,
            LonghandId::BorderBottomRightRadius,
            LonghandId::BorderBottomLeftRadius,
            LonghandId::BorderInlineStartColor,
            LonghandId::BorderInlineStartStyle,
            LonghandId::BorderInlineStartWidth,
            LonghandId::BorderInlineEndColor,
            LonghandId::BorderInlineEndStyle,
            LonghandId::BorderInlineEndWidth,
            LonghandId::BorderBlockStartColor,
            LonghandId::BorderBlockStartStyle,
            LonghandId::BorderBlockStartWidth,
            LonghandId::BorderBlockEndColor,
            LonghandId::BorderBlockEndStyle,
            LonghandId::BorderBlockEndWidth,
        ];

        const PADDING_PROPS: &'static [LonghandId] = &[
            LonghandId::PaddingTop,
            LonghandId::PaddingRight,
            LonghandId::PaddingBottom,
            LonghandId::PaddingLeft,
            LonghandId::PaddingInlineStart,
            LonghandId::PaddingInlineEnd,
            LonghandId::PaddingBlockStart,
            LonghandId::PaddingBlockEnd,
        ];

        // Set of properties that we are currently interested in.
        let mut properties = LonghandIdSet::new();

        if rule_type_mask & NS_AUTHOR_SPECIFIED_BACKGROUND != 0 {
            for id in BACKGROUND_PROPS {
                properties.insert(*id);
            }
        }
        if rule_type_mask & NS_AUTHOR_SPECIFIED_BORDER != 0 {
            for id in BORDER_PROPS {
                properties.insert(*id);
            }
        }
        if rule_type_mask & NS_AUTHOR_SPECIFIED_PADDING != 0 {
            for id in PADDING_PROPS {
                properties.insert(*id);
            }
        }

        // If author colors are not allowed, don't look at those properties
        // (except for background-color which is special and we handle below).
        if !author_colors_allowed {
            properties.remove_all(LonghandIdSet::ignored_when_colors_disabled());
            if rule_type_mask & NS_AUTHOR_SPECIFIED_BACKGROUND != 0 {
                properties.insert(LonghandId::BackgroundColor);
            }
        }

        let mut element_rule_node = Cow::Borrowed(self);

        loop {
            // We need to be careful not to count styles covered up by
            // user-important or UA-important declarations.  But we do want to
            // catch explicit inherit styling in those and check our parent
            // element to see whether we have user styling for those properties.
            // Note that we don't care here about inheritance due to lack of a
            // specified value, since all the properties we care about are reset
            // properties.

            let mut inherited_properties = LonghandIdSet::new();
            let mut have_explicit_ua_inherit = false;

            for node in element_rule_node.self_and_ancestors() {
                let source = node.style_source();
                let declarations = if source.is_some() {
                    source
                        .as_ref()
                        .unwrap()
                        .read(node.cascade_level().guard(guards))
                        .declaration_importance_iter()
                } else {
                    continue;
                };

                // Iterate over declarations of the longhands we care about.
                let node_importance = node.importance();
                let longhands = declarations.rev().filter_map(|(declaration, importance)| {
                    if importance != node_importance {
                        return None;
                    }
                    match declaration.id() {
                        PropertyDeclarationId::Longhand(id) => Some((id, declaration)),
                        _ => None,
                    }
                });

                match node.cascade_level() {
                    // Non-author rules:
                    CascadeLevel::UANormal |
                    CascadeLevel::UAImportant |
                    CascadeLevel::UserNormal |
                    CascadeLevel::UserImportant => {
                        for (id, declaration) in longhands {
                            if properties.contains(id) {
                                // This property was set by a non-author rule.
                                // Stop looking for it in this element's rule
                                // nodes.
                                properties.remove(id);

                                // However, if it is inherited, then it might be
                                // inherited from an author rule from an
                                // ancestor element's rule nodes.
                                if declaration.get_css_wide_keyword() ==
                                    Some(CSSWideKeyword::Inherit)
                                {
                                    have_explicit_ua_inherit = true;
                                    inherited_properties.insert(id);
                                }
                            }
                        }
                    },
                    // Author rules:
                    CascadeLevel::PresHints |
                    CascadeLevel::SameTreeAuthorNormal |
                    CascadeLevel::InnerShadowNormal |
                    CascadeLevel::StyleAttributeNormal |
                    CascadeLevel::SMILOverride |
                    CascadeLevel::Animations |
                    CascadeLevel::SameTreeAuthorImportant |
                    CascadeLevel::InnerShadowImportant |
                    CascadeLevel::StyleAttributeImportant |
                    CascadeLevel::Transitions => {
                        for (id, declaration) in longhands {
                            if properties.contains(id) {
                                if !author_colors_allowed {
                                    if let PropertyDeclaration::BackgroundColor(ref color) =
                                        *declaration
                                    {
                                        return *color == Color::transparent();
                                    }
                                }
                                return true;
                            }
                        }
                    },
                }
            }

            if !have_explicit_ua_inherit {
                break;
            }

            // Continue to the parent element and search for the inherited properties.
            if let Some(pseudo) = pseudo.take() {
                if pseudo.inherits_from_default_values() {
                    break;
                }
            } else {
                element = match element.inheritance_parent() {
                    Some(parent) => parent,
                    None => break,
                };

                let parent_data = element.mutate_data().unwrap();
                let parent_rule_node = parent_data.styles.primary().rules().clone();
                element_rule_node = Cow::Owned(parent_rule_node);
            }

            properties = inherited_properties;
        }

        false
    }

    /// Returns true if there is either animation or transition level rule.
    pub fn has_animation_or_transition_rules(&self) -> bool {
        self.self_and_ancestors()
            .take_while(|node| node.cascade_level() >= CascadeLevel::SMILOverride)
            .any(|node| node.cascade_level().is_animation())
    }

    /// Get a set of properties whose CascadeLevel are higher than Animations
    /// but not equal to Transitions.
    ///
    /// If there are any custom properties, we set the boolean value of the
    /// returned tuple to true.
    pub fn get_properties_overriding_animations(
        &self,
        guards: &StylesheetGuards,
    ) -> (LonghandIdSet, bool) {
        use crate::properties::PropertyDeclarationId;

        // We want to iterate over cascade levels that override the animations
        // level, i.e.  !important levels and the transitions level.
        //
        // However, we actually want to skip the transitions level because
        // although it is higher in the cascade than animations, when both
        // transitions and animations are present for a given element and
        // property, transitions are suppressed so that they don't actually
        // override animations.
        let iter = self
            .self_and_ancestors()
            .skip_while(|node| node.cascade_level() == CascadeLevel::Transitions)
            .take_while(|node| node.cascade_level() > CascadeLevel::Animations);
        let mut result = (LonghandIdSet::new(), false);
        for node in iter {
            let style = node.style_source().unwrap();
            for (decl, important) in style
                .read(node.cascade_level().guard(guards))
                .declaration_importance_iter()
            {
                // Although we are only iterating over cascade levels that
                // override animations, in a given property declaration block we
                // can have a mixture of !important and non-!important
                // declarations but only the !important declarations actually
                // override animations.
                if important.important() {
                    match decl.id() {
                        PropertyDeclarationId::Longhand(id) => result.0.insert(id),
                        PropertyDeclarationId::Custom(_) => result.1 = true,
                    }
                }
            }
        }
        result
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
        debug!(
            "{:?}: {:?}+",
            self.ptr(),
            self.get().refcount.load(Ordering::Relaxed)
        );
        debug_assert!(self.get().refcount.load(Ordering::Relaxed) > 0);
        self.get().refcount.fetch_add(1, Ordering::Relaxed);
        StrongRuleNode::from_ptr(self.p)
    }
}

impl Drop for StrongRuleNode {
    fn drop(&mut self) {
        let node = unsafe { &*self.ptr() };

        debug!(
            "{:?}: {:?}-",
            self.ptr(),
            node.refcount.load(Ordering::Relaxed)
        );
        debug!(
            "Dropping node: {:?}, root: {:?}, parent: {:?}",
            self.ptr(),
            node.root.as_ref().map(|r| r.ptr()),
            node.parent.as_ref().map(|p| p.ptr())
        );
        let should_drop = {
            debug_assert!(node.refcount.load(Ordering::Relaxed) > 0);
            node.refcount.fetch_sub(1, Ordering::Relaxed) == 1
        };

        if !should_drop {
            return;
        }

        debug_assert!(node.children.read().is_empty());
        if node.parent.is_none() {
            debug!("Dropping root node!");
            // The free list should be null by this point
            debug_assert!(node.next_free.load(Ordering::Relaxed).is_null());
            log_drop(self.ptr());
            let _ = unsafe { Box::from_raw(self.ptr()) };
            return;
        }

        let root = unsafe { &*node.root.as_ref().unwrap().ptr() };
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
        free_list.store(self.ptr(), Ordering::Release);
    }
}

impl<'a> From<&'a StrongRuleNode> for WeakRuleNode {
    fn from(node: &'a StrongRuleNode) -> Self {
        WeakRuleNode::from_ptr(node.p)
    }
}

impl WeakRuleNode {
    #[inline]
    fn ptr(&self) -> *mut RuleNode {
        self.p.as_ptr()
    }

    fn upgrade(&self) -> StrongRuleNode {
        debug!("Upgrading weak node: {:p}", self.ptr());

        let node = unsafe { &*self.ptr() };
        node.refcount.fetch_add(1, Ordering::Relaxed);
        StrongRuleNode::from_ptr(self.p)
    }

    fn from_ptr(p: ptr::NonNull<RuleNode>) -> Self {
        WeakRuleNode { p }
    }
}
