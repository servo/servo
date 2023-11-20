/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Style resolution for a given element or pseudo-element.

use crate::applicable_declarations::ApplicableDeclarationList;
use crate::computed_value_flags::ComputedValueFlags;
use crate::context::{CascadeInputs, ElementCascadeInputs, StyleContext};
use crate::data::{EagerPseudoStyles, ElementStyles};
use crate::dom::TElement;
use crate::matching::MatchMethods;
use crate::properties::longhands::display::computed_value::T as Display;
use crate::properties::ComputedValues;
use crate::rule_tree::StrongRuleNode;
use crate::selector_parser::{PseudoElement, SelectorImpl};
use crate::stylist::RuleInclusion;
use log::Level::Trace;
use selectors::matching::{MatchingContext, NeedsSelectorFlags};
use selectors::matching::{MatchingMode, VisitedHandlingMode};
use servo_arc::Arc;

/// Whether pseudo-elements should be resolved or not.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PseudoElementResolution {
    /// Only resolve pseudo-styles if possibly applicable.
    IfApplicable,
    /// Force pseudo-element resolution.
    Force,
}

/// A struct that takes care of resolving the style of a given element.
pub struct StyleResolverForElement<'a, 'ctx, 'le, E>
where
    'ctx: 'a,
    'le: 'ctx,
    E: TElement + MatchMethods + 'le,
{
    element: E,
    context: &'a mut StyleContext<'ctx, E>,
    rule_inclusion: RuleInclusion,
    pseudo_resolution: PseudoElementResolution,
    _marker: ::std::marker::PhantomData<&'le E>,
}

struct MatchingResults {
    rule_node: StrongRuleNode,
    flags: ComputedValueFlags,
}

/// A style returned from the resolver machinery.
pub struct ResolvedStyle(pub Arc<ComputedValues>);

/// The primary style of an element or an element-backed pseudo-element.
pub struct PrimaryStyle {
    /// The style itself.
    pub style: ResolvedStyle,
    /// Whether the style was reused from another element via the rule node (see
    /// `StyleSharingCache::lookup_by_rules`).
    pub reused_via_rule_node: bool,
}

/// A set of style returned from the resolver machinery.
pub struct ResolvedElementStyles {
    /// Primary style.
    pub primary: PrimaryStyle,
    /// Pseudo styles.
    pub pseudos: EagerPseudoStyles,
}

impl ResolvedElementStyles {
    /// Convenience accessor for the primary style.
    pub fn primary_style(&self) -> &Arc<ComputedValues> {
        &self.primary.style.0
    }

    /// Convenience mutable accessor for the style.
    pub fn primary_style_mut(&mut self) -> &mut Arc<ComputedValues> {
        &mut self.primary.style.0
    }
}

impl PrimaryStyle {
    /// Convenience accessor for the style.
    pub fn style(&self) -> &ComputedValues {
        &*self.style.0
    }
}

impl From<ResolvedElementStyles> for ElementStyles {
    fn from(r: ResolvedElementStyles) -> ElementStyles {
        ElementStyles {
            primary: Some(r.primary.style.0),
            pseudos: r.pseudos,
        }
    }
}

fn with_default_parent_styles<E, F, R>(element: E, f: F) -> R
where
    E: TElement,
    F: FnOnce(Option<&ComputedValues>, Option<&ComputedValues>) -> R,
{
    let parent_el = element.inheritance_parent();
    let parent_data = parent_el.as_ref().and_then(|e| e.borrow_data());
    let parent_style = parent_data.as_ref().map(|d| d.styles.primary());

    let mut layout_parent_el = parent_el.clone();
    let layout_parent_data;
    let mut layout_parent_style = parent_style;
    if parent_style.map_or(false, |s| s.is_display_contents()) {
        layout_parent_el = Some(layout_parent_el.unwrap().layout_parent());
        layout_parent_data = layout_parent_el.as_ref().unwrap().borrow_data().unwrap();
        layout_parent_style = Some(layout_parent_data.styles.primary());
    }

    f(
        parent_style.map(|x| &**x),
        layout_parent_style.map(|s| &**s),
    )
}

fn layout_parent_style_for_pseudo<'a>(
    primary_style: &'a PrimaryStyle,
    layout_parent_style: Option<&'a ComputedValues>,
) -> Option<&'a ComputedValues> {
    if primary_style.style().is_display_contents() {
        layout_parent_style
    } else {
        Some(primary_style.style())
    }
}

fn eager_pseudo_is_definitely_not_generated(
    pseudo: &PseudoElement,
    style: &ComputedValues,
) -> bool {
    if !pseudo.is_before_or_after() {
        return false;
    }

    if !style
        .flags
        .intersects(ComputedValueFlags::DISPLAY_DEPENDS_ON_INHERITED_STYLE) &&
        style.get_box().clone_display() == Display::None
    {
        return true;
    }

    if !style
        .flags
        .intersects(ComputedValueFlags::CONTENT_DEPENDS_ON_INHERITED_STYLE) &&
        style.ineffective_content_property()
    {
        return true;
    }

    false
}

impl<'a, 'ctx, 'le, E> StyleResolverForElement<'a, 'ctx, 'le, E>
where
    'ctx: 'a,
    'le: 'ctx,
    E: TElement + MatchMethods + 'le,
{
    /// Trivially construct a new StyleResolverForElement.
    pub fn new(
        element: E,
        context: &'a mut StyleContext<'ctx, E>,
        rule_inclusion: RuleInclusion,
        pseudo_resolution: PseudoElementResolution,
    ) -> Self {
        Self {
            element,
            context,
            rule_inclusion,
            pseudo_resolution,
            _marker: ::std::marker::PhantomData,
        }
    }

    /// Resolve just the style of a given element.
    pub fn resolve_primary_style(
        &mut self,
        parent_style: Option<&ComputedValues>,
        layout_parent_style: Option<&ComputedValues>,
    ) -> PrimaryStyle {
        let primary_results = self.match_primary(VisitedHandlingMode::AllLinksUnvisited);

        let inside_link = parent_style.map_or(false, |s| s.visited_style().is_some());

        let visited_rules = if self.context.shared.visited_styles_enabled &&
            (inside_link || self.element.is_link())
        {
            let visited_matching_results =
                self.match_primary(VisitedHandlingMode::RelevantLinkVisited);
            Some(visited_matching_results.rule_node)
        } else {
            None
        };

        self.cascade_primary_style(
            CascadeInputs {
                rules: Some(primary_results.rule_node),
                visited_rules,
                flags: primary_results.flags,
            },
            parent_style,
            layout_parent_style,
        )
    }

    fn cascade_primary_style(
        &mut self,
        inputs: CascadeInputs,
        parent_style: Option<&ComputedValues>,
        layout_parent_style: Option<&ComputedValues>,
    ) -> PrimaryStyle {
        // Before doing the cascade, check the sharing cache and see if we can
        // reuse the style via rule node identity.
        let may_reuse = self.element.matches_user_and_content_rules() &&
            parent_style.is_some() &&
            inputs.rules.is_some();

        if may_reuse {
            let cached = self.context.thread_local.sharing_cache.lookup_by_rules(
                self.context.shared,
                parent_style.unwrap(),
                inputs.rules.as_ref().unwrap(),
                inputs.visited_rules.as_ref(),
                self.element,
            );
            if let Some(mut primary_style) = cached {
                self.context.thread_local.statistics.styles_reused += 1;
                primary_style.reused_via_rule_node |= true;
                return primary_style;
            }
        }

        // No style to reuse. Cascade the style, starting with visited style
        // if necessary.
        PrimaryStyle {
            style: self.cascade_style_and_visited(
                inputs,
                parent_style,
                layout_parent_style,
                /* pseudo = */ None,
            ),
            reused_via_rule_node: false,
        }
    }

    /// Resolve the style of a given element, and all its eager pseudo-elements.
    pub fn resolve_style(
        &mut self,
        parent_style: Option<&ComputedValues>,
        layout_parent_style: Option<&ComputedValues>,
    ) -> ResolvedElementStyles {
        let primary_style = self.resolve_primary_style(parent_style, layout_parent_style);

        let mut pseudo_styles = EagerPseudoStyles::default();

        if !self.element.is_pseudo_element() {
            let layout_parent_style_for_pseudo =
                layout_parent_style_for_pseudo(&primary_style, layout_parent_style);
            SelectorImpl::each_eagerly_cascaded_pseudo_element(|pseudo| {
                let pseudo_style = self.resolve_pseudo_style(
                    &pseudo,
                    &primary_style,
                    layout_parent_style_for_pseudo,
                );

                if let Some(style) = pseudo_style {
                    if !matches!(self.pseudo_resolution, PseudoElementResolution::Force) &&
                        eager_pseudo_is_definitely_not_generated(&pseudo, &style.0)
                    {
                        return;
                    }
                    pseudo_styles.set(&pseudo, style.0);
                }
            })
        }

        ResolvedElementStyles {
            primary: primary_style,
            pseudos: pseudo_styles,
        }
    }

    /// Resolve an element's styles with the default inheritance parent/layout
    /// parents.
    pub fn resolve_style_with_default_parents(&mut self) -> ResolvedElementStyles {
        with_default_parent_styles(self.element, |parent_style, layout_parent_style| {
            self.resolve_style(parent_style, layout_parent_style)
        })
    }

    /// Cascade a set of rules, using the default parent for inheritance.
    pub fn cascade_style_and_visited_with_default_parents(
        &mut self,
        inputs: CascadeInputs,
    ) -> ResolvedStyle {
        with_default_parent_styles(self.element, |parent_style, layout_parent_style| {
            self.cascade_style_and_visited(
                inputs,
                parent_style,
                layout_parent_style,
                /* pseudo = */ None,
            )
        })
    }

    /// Cascade a set of rules for pseudo element, using the default parent for inheritance.
    pub fn cascade_style_and_visited_for_pseudo_with_default_parents(
        &mut self,
        inputs: CascadeInputs,
        pseudo: &PseudoElement,
        primary_style: &PrimaryStyle,
    ) -> ResolvedStyle {
        with_default_parent_styles(self.element, |_, layout_parent_style| {
            let layout_parent_style_for_pseudo =
                layout_parent_style_for_pseudo(primary_style, layout_parent_style);

            self.cascade_style_and_visited(
                inputs,
                Some(primary_style.style()),
                layout_parent_style_for_pseudo,
                Some(pseudo),
            )
        })
    }

    fn cascade_style_and_visited(
        &mut self,
        inputs: CascadeInputs,
        parent_style: Option<&ComputedValues>,
        layout_parent_style: Option<&ComputedValues>,
        pseudo: Option<&PseudoElement>,
    ) -> ResolvedStyle {
        debug_assert!(pseudo.map_or(true, |p| p.is_eager()));

        let implemented_pseudo = self.element.implemented_pseudo_element();
        let pseudo = pseudo.or(implemented_pseudo.as_ref());

        let mut conditions = Default::default();
        let values = self.context.shared.stylist.cascade_style_and_visited(
            Some(self.element),
            pseudo,
            inputs,
            &self.context.shared.guards,
            pseudo.and(parent_style),
            parent_style,
            parent_style,
            layout_parent_style,
            Some(&self.context.thread_local.rule_cache),
            &mut conditions,
        );

        self.context.thread_local.rule_cache.insert_if_possible(
            &self.context.shared.guards,
            &values,
            pseudo,
            &conditions,
        );

        ResolvedStyle(values)
    }

    /// Cascade the element and pseudo-element styles with the default parents.
    pub fn cascade_styles_with_default_parents(
        &mut self,
        inputs: ElementCascadeInputs,
    ) -> ResolvedElementStyles {
        with_default_parent_styles(self.element, move |parent_style, layout_parent_style| {
            let primary_style =
                self.cascade_primary_style(inputs.primary, parent_style, layout_parent_style);

            let mut pseudo_styles = EagerPseudoStyles::default();
            if let Some(mut pseudo_array) = inputs.pseudos.into_array() {
                let layout_parent_style_for_pseudo = if primary_style.style().is_display_contents()
                {
                    layout_parent_style
                } else {
                    Some(primary_style.style())
                };

                for (i, inputs) in pseudo_array.iter_mut().enumerate() {
                    if let Some(inputs) = inputs.take() {
                        let pseudo = PseudoElement::from_eager_index(i);

                        let style = self.cascade_style_and_visited(
                            inputs,
                            Some(primary_style.style()),
                            layout_parent_style_for_pseudo,
                            Some(&pseudo),
                        );

                        if !matches!(self.pseudo_resolution, PseudoElementResolution::Force) &&
                            eager_pseudo_is_definitely_not_generated(&pseudo, &style.0)
                        {
                            continue;
                        }

                        pseudo_styles.set(&pseudo, style.0);
                    }
                }
            }

            ResolvedElementStyles {
                primary: primary_style,
                pseudos: pseudo_styles,
            }
        })
    }

    fn resolve_pseudo_style(
        &mut self,
        pseudo: &PseudoElement,
        originating_element_style: &PrimaryStyle,
        layout_parent_style: Option<&ComputedValues>,
    ) -> Option<ResolvedStyle> {
        let MatchingResults { rule_node, mut flags } = self.match_pseudo(
            &originating_element_style.style.0,
            pseudo,
            VisitedHandlingMode::AllLinksUnvisited,
        )?;

        let mut visited_rules = None;
        if originating_element_style.style().visited_style().is_some() {
            visited_rules = self.match_pseudo(
                &originating_element_style.style.0,
                pseudo,
                VisitedHandlingMode::RelevantLinkVisited,
            ).map(|results| {
                flags |= results.flags;
                results.rule_node
            });
        }

        Some(self.cascade_style_and_visited(
            CascadeInputs {
                rules: Some(rule_node),
                visited_rules,
                flags,
            },
            Some(originating_element_style.style()),
            layout_parent_style,
            Some(pseudo),
        ))
    }

    fn match_primary(&mut self, visited_handling: VisitedHandlingMode) -> MatchingResults {
        debug!(
            "Match primary for {:?}, visited: {:?}",
            self.element, visited_handling
        );
        let mut applicable_declarations = ApplicableDeclarationList::new();

        let bloom_filter = self.context.thread_local.bloom_filter.filter();
        let nth_index_cache = &mut self.context.thread_local.nth_index_cache;
        let mut matching_context = MatchingContext::new_for_visited(
            MatchingMode::Normal,
            Some(bloom_filter),
            nth_index_cache,
            visited_handling,
            self.context.shared.quirks_mode(),
            NeedsSelectorFlags::Yes,
        );

        let stylist = &self.context.shared.stylist;
        let implemented_pseudo = self.element.implemented_pseudo_element();
        // Compute the primary rule node.
        stylist.push_applicable_declarations(
            self.element,
            implemented_pseudo.as_ref(),
            self.element.style_attribute(),
            self.element.smil_override(),
            self.element.animation_declarations(self.context.shared),
            self.rule_inclusion,
            &mut applicable_declarations,
            &mut matching_context,
        );

        // FIXME(emilio): This is a hack for animations, and should go away.
        self.element.unset_dirty_style_attribute();

        let rule_node = stylist
            .rule_tree()
            .compute_rule_node(&mut applicable_declarations, &self.context.shared.guards);

        if log_enabled!(Trace) {
            trace!("Matched rules for {:?}:", self.element);
            for rn in rule_node.self_and_ancestors() {
                let source = rn.style_source();
                if source.is_some() {
                    trace!(" > {:?}", source);
                }
            }
        }

        MatchingResults {
            rule_node,
            flags: matching_context.extra_data.cascade_input_flags,
        }
    }

    fn match_pseudo(
        &mut self,
        originating_element_style: &ComputedValues,
        pseudo_element: &PseudoElement,
        visited_handling: VisitedHandlingMode,
    ) -> Option<MatchingResults> {
        debug!(
            "Match pseudo {:?} for {:?}, visited: {:?}",
            self.element, pseudo_element, visited_handling
        );
        debug_assert!(pseudo_element.is_eager());
        debug_assert!(
            !self.element.is_pseudo_element(),
            "Element pseudos can't have any other eager pseudo."
        );

        let mut applicable_declarations = ApplicableDeclarationList::new();

        let stylist = &self.context.shared.stylist;

        if !self
            .element
            .may_generate_pseudo(pseudo_element, originating_element_style)
        {
            return None;
        }

        let bloom_filter = self.context.thread_local.bloom_filter.filter();
        let nth_index_cache = &mut self.context.thread_local.nth_index_cache;

        let mut matching_context = MatchingContext::<'_, E::Impl>::new_for_visited(
            MatchingMode::ForStatelessPseudoElement,
            Some(bloom_filter),
            nth_index_cache,
            visited_handling,
            self.context.shared.quirks_mode(),
            NeedsSelectorFlags::Yes,
        );
        matching_context.extra_data.originating_element_style =
            Some(originating_element_style);

        // NB: We handle animation rules for ::before and ::after when
        // traversing them.
        stylist.push_applicable_declarations(
            self.element,
            Some(pseudo_element),
            None,
            None,
            /* animation_declarations = */ Default::default(),
            self.rule_inclusion,
            &mut applicable_declarations,
            &mut matching_context,
        );

        if applicable_declarations.is_empty() {
            return None;
        }

        let rule_node = stylist
            .rule_tree()
            .compute_rule_node(&mut applicable_declarations, &self.context.shared.guards);

        Some(MatchingResults {
            rule_node,
            flags: matching_context.extra_data.cascade_input_flags,
        })
    }
}
