/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Collects a series of applicable rules for a given element.

use crate::applicable_declarations::{ApplicableDeclarationBlock, ApplicableDeclarationList};
use crate::dom::{TElement, TNode, TShadowRoot};
use crate::properties::{AnimationRules, PropertyDeclarationBlock};
use crate::rule_tree::{CascadeLevel, ShadowCascadeOrder};
use crate::selector_map::SelectorMap;
use crate::selector_parser::PseudoElement;
use crate::shared_lock::Locked;
use crate::stylesheets::Origin;
use crate::stylist::{AuthorStylesEnabled, Rule, RuleInclusion, Stylist};
use crate::Atom;
use selectors::matching::{ElementSelectorFlags, MatchingContext, MatchingMode};
use servo_arc::ArcBorrow;
use smallvec::SmallVec;

/// This is a bit of a hack so <svg:use> matches the rules of the enclosing
/// tree.
///
/// This function returns the containing shadow host ignoring <svg:use> shadow
/// trees, since those match the enclosing tree's rules.
///
/// Only a handful of places need to really care about this. This is not a
/// problem for invalidation and that kind of stuff because they still don't
/// match rules based on elements outside of the shadow tree, and because the
/// <svg:use> subtrees are immutable and recreated each time the source tree
/// changes.
///
/// We historically allow cross-document <svg:use> to have these rules applied,
/// but I think that's not great. Gecko is the only engine supporting that.
///
/// See https://github.com/w3c/svgwg/issues/504 for the relevant spec
/// discussion.
#[inline]
pub fn containing_shadow_ignoring_svg_use<E: TElement>(
    element: E,
) -> Option<<E::ConcreteNode as TNode>::ConcreteShadowRoot> {
    let mut shadow = element.containing_shadow()?;
    loop {
        let host = shadow.host();
        let host_is_svg_use_element =
            host.is_svg_element() && host.local_name() == &*local_name!("use");
        if !host_is_svg_use_element {
            return Some(shadow);
        }
        debug_assert!(
            shadow.style_data().is_none(),
            "We allow no stylesheets in <svg:use> subtrees"
        );
        shadow = host.containing_shadow()?;
    }
}

#[inline]
fn sort_rules_from(rules: &mut ApplicableDeclarationList, start: usize) {
    rules[start..].sort_unstable_by_key(|block| (block.specificity, block.source_order()));
}

/// An object that we use with all the intermediate state needed for the
/// cascade.
///
/// This is done basically to be able to organize the cascade in smaller
/// functions, and be able to reason about it easily.
pub struct RuleCollector<'a, 'b: 'a, E, F: 'a>
where
    E: TElement,
{
    element: E,
    rule_hash_target: E,
    stylist: &'a Stylist,
    pseudo_element: Option<&'a PseudoElement>,
    style_attribute: Option<ArcBorrow<'a, Locked<PropertyDeclarationBlock>>>,
    smil_override: Option<ArcBorrow<'a, Locked<PropertyDeclarationBlock>>>,
    animation_rules: AnimationRules,
    rule_inclusion: RuleInclusion,
    rules: &'a mut ApplicableDeclarationList,
    context: &'a mut MatchingContext<'b, E::Impl>,
    flags_setter: &'a mut F,
    matches_user_and_author_rules: bool,
    matches_document_author_rules: bool,
}

impl<'a, 'b: 'a, E, F: 'a> RuleCollector<'a, 'b, E, F>
where
    E: TElement,
    F: FnMut(&E, ElementSelectorFlags),
{
    /// Trivially construct a new collector.
    pub fn new(
        stylist: &'a Stylist,
        element: E,
        pseudo_element: Option<&'a PseudoElement>,
        style_attribute: Option<ArcBorrow<'a, Locked<PropertyDeclarationBlock>>>,
        smil_override: Option<ArcBorrow<'a, Locked<PropertyDeclarationBlock>>>,
        animation_rules: AnimationRules,
        rule_inclusion: RuleInclusion,
        rules: &'a mut ApplicableDeclarationList,
        context: &'a mut MatchingContext<'b, E::Impl>,
        flags_setter: &'a mut F,
    ) -> Self {
        // When we're matching with matching_mode =
        // `ForStatelessPseudoeElement`, the "target" for the rule hash is the
        // element itself, since it's what's generating the pseudo-element.
        let rule_hash_target = match context.matching_mode() {
            MatchingMode::ForStatelessPseudoElement => element,
            MatchingMode::Normal => element.rule_hash_target(),
        };

        let matches_user_and_author_rules = rule_hash_target.matches_user_and_author_rules();

        // Gecko definitely has pseudo-elements with style attributes, like
        // ::-moz-color-swatch.
        debug_assert!(
            cfg!(feature = "gecko") || style_attribute.is_none() || pseudo_element.is_none(),
            "Style attributes do not apply to pseudo-elements"
        );
        debug_assert!(pseudo_element.map_or(true, |p| !p.is_precomputed()));

        Self {
            element,
            rule_hash_target,
            stylist,
            pseudo_element,
            style_attribute,
            smil_override,
            animation_rules,
            rule_inclusion,
            context,
            flags_setter,
            rules,
            matches_user_and_author_rules,
            matches_document_author_rules: matches_user_and_author_rules,
        }
    }

    fn collect_stylist_rules(&mut self, origin: Origin) {
        let cascade_level = match origin {
            Origin::UserAgent => CascadeLevel::UANormal,
            Origin::User => CascadeLevel::UserNormal,
            Origin::Author => CascadeLevel::same_tree_author_normal(),
        };

        let cascade_data = self.stylist.cascade_data().borrow_for_origin(origin);
        let map = match cascade_data.normal_rules(self.pseudo_element) {
            Some(m) => m,
            None => return,
        };

        self.collect_rules_internal(None, map, cascade_level);
    }

    fn collect_user_agent_rules(&mut self) {
        self.collect_stylist_rules(Origin::UserAgent);
    }

    fn collect_user_rules(&mut self) {
        if !self.matches_user_and_author_rules {
            return;
        }

        self.collect_stylist_rules(Origin::User);
    }

    /// Presentational hints.
    ///
    /// These go before author rules, but after user rules, see:
    /// https://drafts.csswg.org/css-cascade/#preshint
    fn collect_presentational_hints(&mut self) {
        if self.pseudo_element.is_some() {
            return;
        }

        let length_before_preshints = self.rules.len();
        self.element
            .synthesize_presentational_hints_for_legacy_attributes(
                self.context.visited_handling(),
                self.rules,
            );
        if cfg!(debug_assertions) {
            if self.rules.len() != length_before_preshints {
                for declaration in &self.rules[length_before_preshints..] {
                    assert_eq!(declaration.level(), CascadeLevel::PresHints);
                }
            }
        }
    }

    fn collect_rules_in_shadow_tree(
        &mut self,
        shadow_host: E,
        map: &SelectorMap<Rule>,
        cascade_level: CascadeLevel,
    ) {
        debug_assert!(shadow_host.shadow_root().is_some());
        self.collect_rules_internal(Some(shadow_host), map, cascade_level);
    }

    #[inline]
    fn collect_rules_internal(
        &mut self,
        shadow_host: Option<E>,
        map: &SelectorMap<Rule>,
        cascade_level: CascadeLevel,
    ) {
        let element = self.element;
        let rule_hash_target = self.rule_hash_target;
        let rules = &mut self.rules;
        let flags_setter = &mut self.flags_setter;
        let start = rules.len();
        self.context.with_shadow_host(shadow_host, |context| {
            map.get_all_matching_rules(
                element,
                rule_hash_target,
                rules,
                context,
                flags_setter,
                cascade_level,
            );
        });
        sort_rules_from(rules, start);
    }

    /// Collects the rules for the ::slotted pseudo-element and the :host
    /// pseudo-class.
    fn collect_host_and_slotted_rules(&mut self) {
        let mut slots = SmallVec::<[_; 3]>::new();
        let mut current = self.rule_hash_target.assigned_slot();
        let mut shadow_cascade_order = ShadowCascadeOrder::for_outermost_shadow_tree();

        while let Some(slot) = current {
            debug_assert!(
                self.matches_user_and_author_rules,
                "We should not slot NAC anywhere"
            );
            slots.push(slot);
            current = slot.assigned_slot();
            shadow_cascade_order.dec();
        }

        self.collect_host_rules(shadow_cascade_order);

        // Match slotted rules in reverse order, so that the outer slotted rules
        // come before the inner rules (and thus have less priority).
        for slot in slots.iter().rev() {
            shadow_cascade_order.inc();

            let shadow = slot.containing_shadow().unwrap();
            let data = match shadow.style_data() {
                Some(d) => d,
                None => continue,
            };
            let slotted_rules = match data.slotted_rules(self.pseudo_element) {
                Some(r) => r,
                None => continue,
            };

            self.collect_rules_in_shadow_tree(
                shadow.host(),
                slotted_rules,
                CascadeLevel::AuthorNormal {
                    shadow_cascade_order,
                },
            );
        }
    }

    fn collect_normal_rules_from_containing_shadow_tree(&mut self) {
        if !self.matches_user_and_author_rules {
            return;
        }

        let containing_shadow = containing_shadow_ignoring_svg_use(self.rule_hash_target);
        let containing_shadow = match containing_shadow {
            Some(s) => s,
            None => return,
        };

        self.matches_document_author_rules = false;

        let cascade_data = containing_shadow.style_data();
        let host = containing_shadow.host();
        if let Some(map) = cascade_data.and_then(|data| data.normal_rules(self.pseudo_element)) {
            self.collect_rules_in_shadow_tree(host, map, CascadeLevel::same_tree_author_normal());
        }
    }

    /// Collects the rules for the :host pseudo-class.
    fn collect_host_rules(&mut self, shadow_cascade_order: ShadowCascadeOrder) {
        let shadow = match self.rule_hash_target.shadow_root() {
            Some(s) => s,
            None => return,
        };

        debug_assert!(
            self.matches_user_and_author_rules,
            "NAC should not be a shadow host"
        );

        let style_data = match shadow.style_data() {
            Some(d) => d,
            None => return,
        };

        let host_rules = match style_data.host_rules(self.pseudo_element) {
            Some(rules) => rules,
            None => return,
        };

        let rule_hash_target = self.rule_hash_target;
        self.collect_rules_in_shadow_tree(
            rule_hash_target,
            host_rules,
            CascadeLevel::AuthorNormal {
                shadow_cascade_order,
            },
        );
    }

    fn collect_document_author_rules(&mut self) {
        if !self.matches_document_author_rules {
            return;
        }

        self.collect_stylist_rules(Origin::Author);
    }

    fn collect_part_rules(&mut self) {
        if !self.rule_hash_target.has_part_attr() {
            return;
        }

        let mut inner_shadow = match self.rule_hash_target.containing_shadow() {
            Some(s) => s,
            None => return,
        };

        let mut shadow_cascade_order = ShadowCascadeOrder::for_innermost_containing_tree();

        let mut parts = SmallVec::<[Atom; 3]>::new();
        self.rule_hash_target.each_part(|p| parts.push(p.clone()));

        loop {
            if parts.is_empty() {
                return;
            }

            let inner_shadow_host = inner_shadow.host();
            let outer_shadow = inner_shadow_host.containing_shadow();
            let part_rules = match outer_shadow {
                Some(shadow) => shadow
                    .style_data()
                    .and_then(|data| data.part_rules(self.pseudo_element)),
                None => self
                    .stylist
                    .cascade_data()
                    .borrow_for_origin(Origin::Author)
                    .part_rules(self.pseudo_element),
            };

            if let Some(part_rules) = part_rules {
                let containing_host = outer_shadow.map(|s| s.host());
                let element = self.element;
                let rules = &mut self.rules;
                let flags_setter = &mut self.flags_setter;
                let cascade_level = CascadeLevel::AuthorNormal {
                    shadow_cascade_order,
                };
                let start = rules.len();
                self.context.with_shadow_host(containing_host, |context| {
                    for p in &parts {
                        if let Some(part_rules) = part_rules.get(p) {
                            SelectorMap::get_matching_rules(
                                element,
                                &part_rules,
                                rules,
                                context,
                                flags_setter,
                                cascade_level,
                            );
                        }
                    }
                });
                sort_rules_from(rules, start);
                shadow_cascade_order.inc();
            }

            inner_shadow = match outer_shadow {
                Some(s) => s,
                None => break, // Nowhere to export to.
            };

            parts.retain(|part| {
                let exported_part = match inner_shadow_host.exported_part(part) {
                    Some(part) => part,
                    None => return false,
                };
                std::mem::replace(part, exported_part);
                true
            });
        }
    }

    fn collect_style_attribute(&mut self) {
        if let Some(sa) = self.style_attribute {
            self.rules
                .push(ApplicableDeclarationBlock::from_declarations(
                    sa.clone_arc(),
                    CascadeLevel::same_tree_author_normal(),
                ));
        }
    }

    fn collect_animation_rules(&mut self) {
        if let Some(so) = self.smil_override {
            self.rules
                .push(ApplicableDeclarationBlock::from_declarations(
                    so.clone_arc(),
                    CascadeLevel::SMILOverride,
                ));
        }

        // The animations sheet (CSS animations, script-generated
        // animations, and CSS transitions that are no longer tied to CSS
        // markup).
        if let Some(anim) = self.animation_rules.0.take() {
            self.rules
                .push(ApplicableDeclarationBlock::from_declarations(
                    anim,
                    CascadeLevel::Animations,
                ));
        }

        // The transitions sheet (CSS transitions that are tied to CSS
        // markup).
        if let Some(anim) = self.animation_rules.1.take() {
            self.rules
                .push(ApplicableDeclarationBlock::from_declarations(
                    anim,
                    CascadeLevel::Transitions,
                ));
        }
    }

    /// Collects all the rules, leaving the result in `self.rules`.
    ///
    /// Note that `!important` rules are handled during rule tree insertion.
    pub fn collect_all(mut self) {
        self.collect_user_agent_rules();
        self.collect_user_rules();
        if self.rule_inclusion == RuleInclusion::DefaultOnly {
            return;
        }
        self.collect_presentational_hints();
        // FIXME(emilio): Should the author styles enabled stuff avoid the
        // presentational hints from getting pushed? See bug 1505770.
        if self.stylist.author_styles_enabled() == AuthorStylesEnabled::No {
            return;
        }
        self.collect_host_and_slotted_rules();
        self.collect_normal_rules_from_containing_shadow_tree();
        self.collect_document_author_rules();
        self.collect_style_attribute();
        self.collect_part_rules();
        self.collect_animation_rules();
    }
}
