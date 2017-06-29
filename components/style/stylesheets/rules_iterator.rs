/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! An iterator over a list of rules.

use context::QuirksMode;
use media_queries::Device;
use shared_lock::SharedRwLockReadGuard;
use smallvec::SmallVec;
use std::slice;
use stylesheets::{CssRule, CssRules, DocumentRule, ImportRule, MediaRule, SupportsRule};
use stylesheets::StylesheetInDocument;

/// An iterator over a list of rules.
pub struct RulesIterator<'a, 'b, C>
    where 'b: 'a,
          C: NestedRuleIterationCondition + 'static,
{
    device: &'a Device,
    quirks_mode: QuirksMode,
    guard: &'a SharedRwLockReadGuard<'b>,
    stack: SmallVec<[slice::Iter<'a, CssRule>; 3]>,
    _phantom: ::std::marker::PhantomData<C>,
}

impl<'a, 'b, C> RulesIterator<'a, 'b, C>
    where 'b: 'a,
          C: NestedRuleIterationCondition + 'static,
{
    /// Creates a new `RulesIterator` to iterate over `rules`.
    pub fn new(
        device: &'a Device,
        quirks_mode: QuirksMode,
        guard: &'a SharedRwLockReadGuard<'b>,
        rules: &'a CssRules)
        -> Self
    {
        let mut stack = SmallVec::new();
        stack.push(rules.0.iter());
        Self {
            device: device,
            quirks_mode: quirks_mode,
            guard: guard,
            stack: stack,
            _phantom: ::std::marker::PhantomData,
        }
    }

    /// Skips all the remaining children of the last nested rule processed.
    pub fn skip_children(&mut self) {
        self.stack.pop();
    }
}

impl<'a, 'b, C> Iterator for RulesIterator<'a, 'b, C>
    where 'b: 'a,
          C: NestedRuleIterationCondition + 'static,
{
    type Item = &'a CssRule;

    fn next(&mut self) -> Option<Self::Item> {
        let mut nested_iter_finished = false;
        while !self.stack.is_empty() {
            if nested_iter_finished {
                self.stack.pop();
                nested_iter_finished = false;
                continue;
            }

            let rule;
            let sub_iter = {
                let mut nested_iter = self.stack.last_mut().unwrap();
                rule = match nested_iter.next() {
                    Some(r) => r,
                    None => {
                        nested_iter_finished = true;
                        continue
                    }
                };

                match *rule {
                    CssRule::Namespace(_) |
                    CssRule::Style(_) |
                    CssRule::FontFace(_) |
                    CssRule::CounterStyle(_) |
                    CssRule::Viewport(_) |
                    CssRule::Keyframes(_) |
                    CssRule::Page(_) => {
                        return Some(rule)
                    },
                    CssRule::Import(ref import_rule) => {
                        let import_rule = import_rule.read_with(self.guard);
                        if !C::process_import(self.guard,
                                              self.device,
                                              self.quirks_mode,
                                              import_rule) {
                            continue;
                        }
                        import_rule
                            .stylesheet.contents(self.guard).rules
                            .read_with(self.guard).0.iter()
                    }
                    CssRule::Document(ref doc_rule) => {
                        let doc_rule = doc_rule.read_with(self.guard);
                        if !C::process_document(self.guard,
                                                self.device,
                                                self.quirks_mode,
                                                doc_rule) {
                            continue;
                        }
                        doc_rule.rules.read_with(self.guard).0.iter()
                    }
                    CssRule::Media(ref lock) => {
                        let media_rule = lock.read_with(self.guard);
                        if !C::process_media(self.guard,
                                             self.device,
                                             self.quirks_mode,
                                             media_rule) {
                            continue;
                        }
                        media_rule.rules.read_with(self.guard).0.iter()
                    }
                    CssRule::Supports(ref lock) => {
                        let supports_rule = lock.read_with(self.guard);
                        if !C::process_supports(self.guard,
                                                self.device,
                                                self.quirks_mode,
                                                supports_rule) {
                            continue;
                        }
                        supports_rule.rules.read_with(self.guard).0.iter()
                    }
                }
            };

            self.stack.push(sub_iter);
            return Some(rule);
        }

        None
    }
}

/// RulesIterator.
pub trait NestedRuleIterationCondition {
    /// Whether we should process the nested rules in a given `@import` rule.
    fn process_import(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &ImportRule)
        -> bool;

    /// Whether we should process the nested rules in a given `@media` rule.
    fn process_media(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &MediaRule)
        -> bool;

    /// Whether we should process the nested rules in a given `@-moz-document`
    /// rule.
    fn process_document(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &DocumentRule)
        -> bool;

    /// Whether we should process the nested rules in a given `@supports` rule.
    fn process_supports(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &SupportsRule)
        -> bool;
}

/// A struct that represents the condition that a rule applies to the document.
pub struct EffectiveRules;

impl NestedRuleIterationCondition for EffectiveRules {
    fn process_import(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        _quirks_mode: QuirksMode,
        rule: &ImportRule)
        -> bool
    {
        rule.stylesheet.is_effective_for_device(device, guard)
    }

    fn process_media(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &MediaRule)
        -> bool
    {
        rule.media_queries.read_with(guard).evaluate(device, quirks_mode)
    }

    fn process_document(
        _: &SharedRwLockReadGuard,
        device: &Device,
        _: QuirksMode,
        rule: &DocumentRule)
        -> bool
    {
        rule.condition.evaluate(device)
    }

    fn process_supports(
        _: &SharedRwLockReadGuard,
        _: &Device,
        _: QuirksMode,
        rule: &SupportsRule)
        -> bool
    {
        rule.enabled
    }
}

/// A filter that processes all the rules in a rule list.
pub struct AllRules;

impl NestedRuleIterationCondition for AllRules {
    fn process_import(
        _: &SharedRwLockReadGuard,
        _: &Device,
        _: QuirksMode,
        _: &ImportRule)
        -> bool
    {
        true
    }

    fn process_media(
        _: &SharedRwLockReadGuard,
        _: &Device,
        _: QuirksMode,
        _: &MediaRule)
        -> bool
    {
        true
    }

    fn process_document(
        _: &SharedRwLockReadGuard,
        _: &Device,
        _: QuirksMode,
        _: &DocumentRule)
        -> bool
    {
        true
    }

    fn process_supports(
        _: &SharedRwLockReadGuard,
        _: &Device,
        _: QuirksMode,
        _: &SupportsRule)
        -> bool
    {
        true
    }
}

/// An iterator over all the effective rules of a stylesheet.
///
/// NOTE: This iterator recurses into `@import` rules.
pub type EffectiveRulesIterator<'a, 'b> = RulesIterator<'a, 'b, EffectiveRules>;
