/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

//! The rule tree.

use crate::applicable_declarations::ApplicableDeclarationList;
use crate::properties::{LonghandIdSet, PropertyDeclarationBlock};
use crate::shared_lock::{Locked, StylesheetGuards};
use servo_arc::{Arc, ArcBorrow};
use smallvec::SmallVec;
use std::io::{self, Write};

mod core;
mod level;
mod map;
mod source;
mod unsafe_box;

pub use self::core::{RuleTree, StrongRuleNode, RULE_NODE_SIZE};
pub use self::level::{CascadeLevel, ShadowCascadeOrder};
pub use self::source::StyleSource;

impl RuleTree {
    fn dump<W: Write>(&self, guards: &StylesheetGuards, writer: &mut W) {
        let _ = writeln!(writer, " + RuleTree");
        self.root().dump(guards, writer, 0);
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
        I: Iterator<Item = (StyleSource, CascadeLevel)>,
    {
        use self::CascadeLevel::*;
        let mut current = self.root().clone();

        let mut found_important = false;

        let mut important_author = SmallVec::<[(StyleSource, ShadowCascadeOrder); 4]>::new();

        let mut important_user = SmallVec::<[StyleSource; 4]>::new();
        let mut important_ua = SmallVec::<[StyleSource; 4]>::new();
        let mut transition = None;

        for (source, level) in iter {
            debug_assert!(!level.is_important(), "Important levels handled internally");
            let any_important = {
                let pdb = source.read(level.guard(guards));
                pdb.any_important()
            };

            if any_important {
                found_important = true;
                match level {
                    AuthorNormal {
                        shadow_cascade_order,
                    } => {
                        important_author.push((source.clone(), shadow_cascade_order));
                    },
                    UANormal => important_ua.push(source.clone()),
                    UserNormal => important_user.push(source.clone()),
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
                current = current.ensure_child(self.root(), source, level);
            }
        }

        // Early-return in the common case of no !important declarations.
        if !found_important {
            return current;
        }

        // Insert important declarations, in order of increasing importance,
        // followed by any transition rule.
        //
        // Inner shadow wins over same-tree, which wins over outer-shadow.
        //
        // We negate the shadow cascade order to preserve the right PartialOrd
        // behavior.
        if !important_author.is_empty() &&
            important_author.first().unwrap().1 != important_author.last().unwrap().1
        {
            // We only need to sort if the important rules come from
            // different trees, but we need this sort to be stable.
            //
            // FIXME(emilio): This could maybe be smarter, probably by chunking
            // the important rules while inserting, and iterating the outer
            // chunks in reverse order.
            //
            // That is, if we have rules with levels like: -1 -1 -1 0 0 0 1 1 1,
            // we're really only sorting the chunks, while keeping elements
            // inside the same chunk already sorted. Seems like we could try to
            // keep a SmallVec-of-SmallVecs with the chunks and just iterate the
            // outer in reverse.
            important_author.sort_by_key(|&(_, order)| -order);
        }

        for (source, shadow_cascade_order) in important_author.drain(..) {
            current = current.ensure_child(
                self.root(),
                source,
                AuthorImportant {
                    shadow_cascade_order: -shadow_cascade_order,
                },
            );
        }

        for source in important_user.drain(..) {
            current = current.ensure_child(self.root(), source, UserImportant);
        }

        for source in important_ua.drain(..) {
            current = current.ensure_child(self.root(), source, UAImportant);
        }

        if let Some(source) = transition {
            current = current.ensure_child(self.root(), source, Transitions);
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
            applicable_declarations.drain(..).map(|d| d.for_rule_tree()),
            guards,
        )
    }

    /// Insert the given rules, that must be in proper order by specifity, and
    /// return the corresponding rule node representing the last inserted one.
    pub fn insert_ordered_rules<'a, I>(&self, iter: I) -> StrongRuleNode
    where
        I: Iterator<Item = (StyleSource, CascadeLevel)>,
    {
        self.insert_ordered_rules_from(self.root().clone(), iter)
    }

    fn insert_ordered_rules_from<'a, I>(&self, from: StrongRuleNode, iter: I) -> StrongRuleNode
    where
        I: Iterator<Item = (StyleSource, CascadeLevel)>,
    {
        let mut current = from;
        for (source, level) in iter {
            current = current.ensure_child(self.root(), source, level);
        }
        current
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
        // TODO(emilio): Being smarter with lifetimes we could avoid a bit of
        // the refcount churn.
        let mut current = path.clone();
        *important_rules_changed = false;

        // First walk up until the first less-or-equally specific rule.
        let mut children = SmallVec::<[_; 10]>::new();
        while current.cascade_level() > level {
            children.push((
                current.style_source().unwrap().clone(),
                current.cascade_level(),
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
        if current.cascade_level() == level {
            *important_rules_changed |= level.is_important();

            let current_decls = current.style_source().unwrap().as_declarations();

            // If the only rule at the level we're replacing is exactly the
            // same as `pdb`, we're done, and `path` is still valid.
            if let (Some(ref pdb), Some(ref current_decls)) = (pdb, current_decls) {
                // If the only rule at the level we're replacing is exactly the
                // same as `pdb`, we're done, and `path` is still valid.
                //
                // TODO(emilio): Another potential optimization is the one where
                // we can just replace the rule at that level for `pdb`, and
                // then we don't need to re-create the children, and `path` is
                // also equally valid. This is less likely, and would require an
                // in-place mutation of the source, which is, at best, fiddly,
                // so let's skip it for now.
                let is_here_already = ArcBorrow::ptr_eq(pdb, current_decls);
                if is_here_already {
                    debug!("Picking the fast path in rule replacement");
                    return None;
                }
            }

            if current_decls.is_some() {
                current = current.parent().unwrap().clone();
            }
        }

        // Insert the rule if it's relevant at this level in the cascade.
        //
        // These optimizations are likely to be important, because the levels
        // where replacements apply (style and animations) tend to trigger
        // pretty bad styling cases already.
        if let Some(pdb) = pdb {
            if level.is_important() {
                if pdb.read_with(level.guard(guards)).any_important() {
                    current = current.ensure_child(
                        self.root(),
                        StyleSource::from_declarations(pdb.clone_arc()),
                        level,
                    );
                    *important_rules_changed = true;
                }
            } else {
                if pdb.read_with(level.guard(guards)).any_normal() {
                    current = current.ensure_child(
                        self.root(),
                        StyleSource::from_declarations(pdb.clone_arc()),
                        level,
                    );
                }
            }
        }

        // Now the rule is in the relevant place, push the children as
        // necessary.
        let rule = self.insert_ordered_rules_from(current, children.drain(..).rev());
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
                children.push((node.style_source().unwrap().clone(), node.cascade_level()));
            }
            last = node;
        }

        let rule = self
            .insert_ordered_rules_from(last.parent().unwrap().clone(), children.drain(..).rev());
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

impl StrongRuleNode {
    /// Get an iterator for this rule node and its ancestors.
    pub fn self_and_ancestors(&self) -> SelfAndAncestors {
        SelfAndAncestors {
            current: Some(self),
        }
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
