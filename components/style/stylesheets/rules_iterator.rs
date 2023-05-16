/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An iterator over a list of rules.

use crate::context::QuirksMode;
use crate::media_queries::Device;
use crate::shared_lock::SharedRwLockReadGuard;
use crate::stylesheets::{CssRule, DocumentRule, ImportRule, MediaRule, SupportsRule};
use smallvec::SmallVec;
use std::slice;

/// An iterator over a list of rules.
pub struct RulesIterator<'a, 'b, C>
where
    'b: 'a,
    C: NestedRuleIterationCondition + 'static,
{
    device: &'a Device,
    quirks_mode: QuirksMode,
    guard: &'a SharedRwLockReadGuard<'b>,
    stack: SmallVec<[slice::Iter<'a, CssRule>; 3]>,
    _phantom: ::std::marker::PhantomData<C>,
}

impl<'a, 'b, C> RulesIterator<'a, 'b, C>
where
    'b: 'a,
    C: NestedRuleIterationCondition + 'static,
{
    /// Creates a new `RulesIterator` to iterate over `rules`.
    pub fn new(
        device: &'a Device,
        quirks_mode: QuirksMode,
        guard: &'a SharedRwLockReadGuard<'b>,
        rules: slice::Iter<'a, CssRule>,
    ) -> Self {
        let mut stack = SmallVec::new();
        stack.push(rules);
        Self {
            device,
            quirks_mode,
            guard,
            stack,
            _phantom: ::std::marker::PhantomData,
        }
    }

    /// Skips all the remaining children of the last nested rule processed.
    pub fn skip_children(&mut self) {
        self.stack.pop();
    }
}

fn children_of_rule<'a, C>(
    rule: &'a CssRule,
    device: &'a Device,
    quirks_mode: QuirksMode,
    guard: &'a SharedRwLockReadGuard<'_>,
    effective: &mut bool,
) -> Option<slice::Iter<'a, CssRule>>
where
    C: NestedRuleIterationCondition + 'static,
{
    *effective = true;
    match *rule {
        CssRule::Namespace(_) |
        CssRule::Style(_) |
        CssRule::FontFace(_) |
        CssRule::CounterStyle(_) |
        CssRule::Viewport(_) |
        CssRule::Keyframes(_) |
        CssRule::Page(_) |
        CssRule::FontFeatureValues(_) => None,
        CssRule::Import(ref import_rule) => {
            let import_rule = import_rule.read_with(guard);
            if !C::process_import(guard, device, quirks_mode, import_rule) {
                *effective = false;
                return None;
            }
            Some(import_rule.stylesheet.rules(guard).iter())
        },
        CssRule::Document(ref doc_rule) => {
            let doc_rule = doc_rule.read_with(guard);
            if !C::process_document(guard, device, quirks_mode, doc_rule) {
                *effective = false;
                return None;
            }
            Some(doc_rule.rules.read_with(guard).0.iter())
        },
        CssRule::Media(ref lock) => {
            let media_rule = lock.read_with(guard);
            if !C::process_media(guard, device, quirks_mode, media_rule) {
                *effective = false;
                return None;
            }
            Some(media_rule.rules.read_with(guard).0.iter())
        },
        CssRule::Supports(ref lock) => {
            let supports_rule = lock.read_with(guard);
            if !C::process_supports(guard, device, quirks_mode, supports_rule) {
                *effective = false;
                return None;
            }
            Some(supports_rule.rules.read_with(guard).0.iter())
        },
    }
}

impl<'a, 'b, C> Iterator for RulesIterator<'a, 'b, C>
where
    'b: 'a,
    C: NestedRuleIterationCondition + 'static,
{
    type Item = &'a CssRule;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.stack.is_empty() {
            let rule = {
                let nested_iter = self.stack.last_mut().unwrap();
                match nested_iter.next() {
                    Some(r) => r,
                    None => {
                        self.stack.pop();
                        continue;
                    },
                }
            };

            let mut effective = true;
            let children = children_of_rule::<C>(
                rule,
                self.device,
                self.quirks_mode,
                self.guard,
                &mut effective,
            );
            if !effective {
                continue;
            }

            if let Some(children) = children {
                // NOTE: It's important that `children` gets pushed even if
                // empty, so that `skip_children()` works as expected.
                self.stack.push(children);
            }

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
        rule: &ImportRule,
    ) -> bool;

    /// Whether we should process the nested rules in a given `@media` rule.
    fn process_media(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &MediaRule,
    ) -> bool;

    /// Whether we should process the nested rules in a given `@-moz-document`
    /// rule.
    fn process_document(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &DocumentRule,
    ) -> bool;

    /// Whether we should process the nested rules in a given `@supports` rule.
    fn process_supports(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &SupportsRule,
    ) -> bool;
}

/// A struct that represents the condition that a rule applies to the document.
pub struct EffectiveRules;

impl EffectiveRules {
    /// Returns whether a given rule is effective.
    pub fn is_effective(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &CssRule,
    ) -> bool {
        match *rule {
            CssRule::Import(ref import_rule) => {
                let import_rule = import_rule.read_with(guard);
                Self::process_import(guard, device, quirks_mode, import_rule)
            },
            CssRule::Document(ref doc_rule) => {
                let doc_rule = doc_rule.read_with(guard);
                Self::process_document(guard, device, quirks_mode, doc_rule)
            },
            CssRule::Media(ref lock) => {
                let media_rule = lock.read_with(guard);
                Self::process_media(guard, device, quirks_mode, media_rule)
            },
            CssRule::Supports(ref lock) => {
                let supports_rule = lock.read_with(guard);
                Self::process_supports(guard, device, quirks_mode, supports_rule)
            },
            _ => true,
        }
    }
}

impl NestedRuleIterationCondition for EffectiveRules {
    fn process_import(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &ImportRule,
    ) -> bool {
        match rule.stylesheet.media(guard) {
            Some(m) => m.evaluate(device, quirks_mode),
            None => true,
        }
    }

    fn process_media(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &MediaRule,
    ) -> bool {
        rule.media_queries
            .read_with(guard)
            .evaluate(device, quirks_mode)
    }

    fn process_document(
        _: &SharedRwLockReadGuard,
        device: &Device,
        _: QuirksMode,
        rule: &DocumentRule,
    ) -> bool {
        rule.condition.evaluate(device)
    }

    fn process_supports(
        _: &SharedRwLockReadGuard,
        _: &Device,
        _: QuirksMode,
        rule: &SupportsRule,
    ) -> bool {
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
        _: &ImportRule,
    ) -> bool {
        true
    }

    fn process_media(_: &SharedRwLockReadGuard, _: &Device, _: QuirksMode, _: &MediaRule) -> bool {
        true
    }

    fn process_document(
        _: &SharedRwLockReadGuard,
        _: &Device,
        _: QuirksMode,
        _: &DocumentRule,
    ) -> bool {
        true
    }

    fn process_supports(
        _: &SharedRwLockReadGuard,
        _: &Device,
        _: QuirksMode,
        _: &SupportsRule,
    ) -> bool {
        true
    }
}

/// An iterator over all the effective rules of a stylesheet.
///
/// NOTE: This iterator recurses into `@import` rules.
pub type EffectiveRulesIterator<'a, 'b> = RulesIterator<'a, 'b, EffectiveRules>;

impl<'a, 'b> EffectiveRulesIterator<'a, 'b> {
    /// Returns an iterator over the effective children of a rule, even if
    /// `rule` itself is not effective.
    pub fn effective_children(
        device: &'a Device,
        quirks_mode: QuirksMode,
        guard: &'a SharedRwLockReadGuard<'b>,
        rule: &'a CssRule,
    ) -> Self {
        let children = children_of_rule::<AllRules>(rule, device, quirks_mode, guard, &mut false);
        EffectiveRulesIterator::new(device, quirks_mode, guard, children.unwrap_or([].iter()))
    }
}
