/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A list of CSS rules.

use crate::shared_lock::{DeepCloneParams, DeepCloneWithLock, Locked};
use crate::shared_lock::{SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard};
use crate::str::CssStringWriter;
use crate::stylesheets::loader::StylesheetLoader;
use crate::stylesheets::rule_parser::{InsertRuleContext, State};
use crate::stylesheets::stylesheet::StylesheetContents;
use crate::stylesheets::{CssRule, RulesMutateError, AllowImportRules};
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocShallowSizeOf, MallocSizeOfOps};
use servo_arc::{Arc, RawOffsetArc};
use std::fmt::{self, Write};

/// A list of CSS rules.
#[derive(Debug, ToShmem)]
pub struct CssRules(pub Vec<CssRule>);

impl CssRules {
    /// Whether this CSS rules is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl DeepCloneWithLock for CssRules {
    fn deep_clone_with_lock(
        &self,
        lock: &SharedRwLock,
        guard: &SharedRwLockReadGuard,
        params: &DeepCloneParams,
    ) -> Self {
        CssRules(
            self.0
                .iter()
                .map(|x| x.deep_clone_with_lock(lock, guard, params))
                .collect(),
        )
    }
}

impl CssRules {
    /// Measure heap usage.
    #[cfg(feature = "gecko")]
    pub fn size_of(&self, guard: &SharedRwLockReadGuard, ops: &mut MallocSizeOfOps) -> usize {
        let mut n = self.0.shallow_size_of(ops);
        for rule in self.0.iter() {
            n += rule.size_of(guard, ops);
        }
        n
    }

    /// Trivially construct a new set of CSS rules.
    pub fn new(rules: Vec<CssRule>, shared_lock: &SharedRwLock) -> Arc<Locked<CssRules>> {
        Arc::new(shared_lock.wrap(CssRules(rules)))
    }

    /// Returns whether all the rules in this list are namespace or import
    /// rules.
    fn only_ns_or_import(&self) -> bool {
        self.0.iter().all(|r| match *r {
            CssRule::Namespace(..) | CssRule::Import(..) => true,
            _ => false,
        })
    }

    /// <https://drafts.csswg.org/cssom/#remove-a-css-rule>
    pub fn remove_rule(&mut self, index: usize) -> Result<(), RulesMutateError> {
        // Step 1, 2
        if index >= self.0.len() {
            return Err(RulesMutateError::IndexSize);
        }

        {
            // Step 3
            let ref rule = self.0[index];

            // Step 4
            if let CssRule::Namespace(..) = *rule {
                if !self.only_ns_or_import() {
                    return Err(RulesMutateError::InvalidState);
                }
            }
        }

        // Step 5, 6
        self.0.remove(index);
        Ok(())
    }

    /// Serializes this CSSRules to CSS text as a block of rules.
    ///
    /// This should be speced into CSSOM spec at some point. See
    /// <https://github.com/w3c/csswg-drafts/issues/1985>
    pub fn to_css_block(
        &self,
        guard: &SharedRwLockReadGuard,
        dest: &mut CssStringWriter,
    ) -> fmt::Result {
        dest.write_str(" {")?;
        for rule in self.0.iter() {
            dest.write_str("\n  ")?;
            rule.to_css(guard, dest)?;
        }
        dest.write_str("\n}")
    }
}

/// A trait to implement helpers for `Arc<Locked<CssRules>>`.
pub trait CssRulesHelpers {
    /// <https://drafts.csswg.org/cssom/#insert-a-css-rule>
    ///
    /// Written in this funky way because parsing an @import rule may cause us
    /// to clone a stylesheet from the same document due to caching in the CSS
    /// loader.
    ///
    /// TODO(emilio): We could also pass the write guard down into the loader
    /// instead, but that seems overkill.
    fn insert_rule(
        &self,
        lock: &SharedRwLock,
        rule: &str,
        parent_stylesheet_contents: &StylesheetContents,
        index: usize,
        nested: bool,
        loader: Option<&dyn StylesheetLoader>,
        allow_import_rules: AllowImportRules,
    ) -> Result<CssRule, RulesMutateError>;
}

impl CssRulesHelpers for RawOffsetArc<Locked<CssRules>> {
    fn insert_rule(
        &self,
        lock: &SharedRwLock,
        rule: &str,
        parent_stylesheet_contents: &StylesheetContents,
        index: usize,
        nested: bool,
        loader: Option<&dyn StylesheetLoader>,
        allow_import_rules: AllowImportRules,
    ) -> Result<CssRule, RulesMutateError> {
        let new_rule = {
            let read_guard = lock.read();
            let rules = self.read_with(&read_guard);

            // Step 1, 2
            if index > rules.0.len() {
                return Err(RulesMutateError::IndexSize);
            }

            // Computes the parser state at the given index
            let state = if nested {
                State::Body
            } else if index == 0 {
                State::Start
            } else {
                rules
                    .0
                    .get(index - 1)
                    .map(CssRule::rule_state)
                    .unwrap_or(State::Body)
            };

            let insert_rule_context = InsertRuleContext {
                rule_list: &rules.0,
                index,
            };

            // Steps 3, 4, 5, 6
            CssRule::parse(
                &rule,
                insert_rule_context,
                parent_stylesheet_contents,
                lock,
                state,
                loader,
                allow_import_rules,
            )?
        };

        {
            let mut write_guard = lock.write();
            let rules = self.write_with(&mut write_guard);
            rules.0.insert(index, new_rule.clone());
        }

        Ok(new_rule)
    }
}
