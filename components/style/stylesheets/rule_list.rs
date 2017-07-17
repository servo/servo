/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A list of CSS rules.

use shared_lock::{DeepCloneParams, DeepCloneWithLock, Locked, SharedRwLock, SharedRwLockReadGuard};
use stylearc::{Arc, RawOffsetArc};
use stylesheets::{CssRule, RulesMutateError};
use stylesheets::loader::StylesheetLoader;
use stylesheets::memory::{MallocSizeOfFn, MallocSizeOfWithGuard};
use stylesheets::rule_parser::State;
use stylesheets::stylesheet::StylesheetContents;

/// A list of CSS rules.
#[derive(Debug)]
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
        CssRules(self.0.iter().map(|x| {
            x.deep_clone_with_lock(lock, guard, params)
        }).collect())
    }
}

impl MallocSizeOfWithGuard for CssRules {
    fn malloc_size_of_children(
        &self,
        guard: &SharedRwLockReadGuard,
        malloc_size_of: MallocSizeOfFn
    ) -> usize {
        self.0.malloc_size_of_children(guard, malloc_size_of)
    }
}

impl CssRules {
    /// Trivially construct a new set of CSS rules.
    pub fn new(rules: Vec<CssRule>, shared_lock: &SharedRwLock) -> Arc<Locked<CssRules>> {
        Arc::new(shared_lock.wrap(CssRules(rules)))
    }

    /// Returns whether all the rules in this list are namespace or import
    /// rules.
    fn only_ns_or_import(&self) -> bool {
        self.0.iter().all(|r| {
            match *r {
                CssRule::Namespace(..) |
                CssRule::Import(..) => true,
                _ => false
            }
        })
    }

    /// https://drafts.csswg.org/cssom/#remove-a-css-rule
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
}

/// A trait to implement helpers for `Arc<Locked<CssRules>>`.
pub trait CssRulesHelpers {
    /// https://drafts.csswg.org/cssom/#insert-a-css-rule
    ///
    /// Written in this funky way because parsing an @import rule may cause us
    /// to clone a stylesheet from the same document due to caching in the CSS
    /// loader.
    ///
    /// TODO(emilio): We could also pass the write guard down into the loader
    /// instead, but that seems overkill.
    fn insert_rule(&self,
                   lock: &SharedRwLock,
                   rule: &str,
                   parent_stylesheet_contents: &StylesheetContents,
                   index: usize,
                   nested: bool,
                   loader: Option<&StylesheetLoader>)
                   -> Result<CssRule, RulesMutateError>;
}

impl CssRulesHelpers for RawOffsetArc<Locked<CssRules>> {
    fn insert_rule(&self,
                   lock: &SharedRwLock,
                   rule: &str,
                   parent_stylesheet_contents: &StylesheetContents,
                   index: usize,
                   nested: bool,
                   loader: Option<&StylesheetLoader>)
                   -> Result<CssRule, RulesMutateError> {
        let state = {
            let read_guard = lock.read();
            let rules = self.read_with(&read_guard);

            // Step 1, 2
            if index > rules.0.len() {
                return Err(RulesMutateError::IndexSize);
            }

            // Computes the parser state at the given index
            if nested {
                None
            } else if index == 0 {
                Some(State::Start)
            } else {
                rules.0.get(index - 1).map(CssRule::rule_state)
            }
        };

        // Step 3, 4
        // XXXManishearth should we also store the namespace map?
        let (new_rule, new_state) =
            CssRule::parse(
                &rule,
                parent_stylesheet_contents,
                lock,
                state,
                loader
            )?;

        {
            let mut write_guard = lock.write();
            let mut rules = self.write_with(&mut write_guard);
            // Step 5
            // Computes the maximum allowed parser state at a given index.
            let rev_state = rules.0.get(index).map_or(State::Body, CssRule::rule_state);
            if new_state > rev_state {
                // We inserted a rule too early, e.g. inserting
                // a regular style rule before @namespace rules
                return Err(RulesMutateError::HierarchyRequest);
            }

            // Step 6
            if let CssRule::Namespace(..) = new_rule {
                if !rules.only_ns_or_import() {
                    return Err(RulesMutateError::InvalidState);
                }
            }

            rules.0.insert(index, new_rule.clone());
        }

        Ok(new_rule)
    }
}
