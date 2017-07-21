/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Style resolution for a given element or pseudo-element.

use applicable_declarations::ApplicableDeclarationList;
use cascade_info::CascadeInfo;
use context::{CascadeInputs, ElementCascadeInputs, StyleContext};
use data::{ElementStyles, EagerPseudoStyles};
use dom::TElement;
use log::LogLevel::Trace;
use matching::{CascadeVisitedMode, MatchMethods};
use properties::{AnimationRules, CascadeFlags, ComputedValues};
use properties::{IS_ROOT_ELEMENT, PROHIBIT_DISPLAY_CONTENTS, SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP};
use properties::{VISITED_DEPENDENT_ONLY, cascade};
use rule_tree::StrongRuleNode;
use selector_parser::{PseudoElement, SelectorImpl};
use selectors::matching::{ElementSelectorFlags, MatchingContext, MatchingMode, VisitedHandlingMode};
use servo_arc::Arc;
use stylist::RuleInclusion;

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
    _marker: ::std::marker::PhantomData<&'le E>,
}

struct MatchingResults {
    rule_node: StrongRuleNode,
    relevant_link_found: bool,
}

/// The primary style of an element or an element-backed pseudo-element.
pub struct PrimaryStyle {
    /// The style per se.
    pub style: Arc<ComputedValues>,
}

fn with_default_parent_styles<E, F, R>(element: E, f: F) -> R
where
    E: TElement,
    F: FnOnce(Option<&ComputedValues>, Option<&ComputedValues>) -> R,
{
    let parent_el = element.inheritance_parent();
    let parent_data = parent_el.as_ref().and_then(|e| e.borrow_data());
    let parent_style = parent_data.as_ref().map(|d| {
        // Sometimes Gecko eagerly styles things without processing
        // pending restyles first. In general we'd like to avoid this,
        // but there can be good reasons (for example, needing to
        // construct a frame for some small piece of newly-added
        // content in order to do something specific with that frame,
        // but not wanting to flush all of layout).
        debug_assert!(cfg!(feature = "gecko") ||
                      parent_el.unwrap().has_current_styles(d));
        d.styles.primary()
    });

    let mut layout_parent_el = parent_el.clone();
    let layout_parent_data;
    let mut layout_parent_style = parent_style;
    if parent_style.map_or(false, |s| s.is_display_contents()) {
        layout_parent_el = Some(layout_parent_el.unwrap().layout_parent());
        layout_parent_data = layout_parent_el.as_ref().unwrap().borrow_data().unwrap();
        layout_parent_style = Some(layout_parent_data.styles.primary());
    }

    f(parent_style.map(|x| &**x), layout_parent_style.map(|s| &**s))
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
    ) -> Self {
        Self {
            element,
            context,
            rule_inclusion,
            _marker: ::std::marker::PhantomData,
        }
    }

    /// Resolve just the style of a given element.
    pub fn resolve_primary_style(
        &mut self,
        parent_style: Option<&ComputedValues>,
        layout_parent_style: Option<&ComputedValues>,
    ) -> PrimaryStyle {
        let primary_results =
            self.match_primary(VisitedHandlingMode::AllLinksUnvisited);

        let relevant_link_found = primary_results.relevant_link_found;

        let visited_rules = if relevant_link_found {
            let visited_matching_results =
                self.match_primary(VisitedHandlingMode::RelevantLinkVisited);
            Some(visited_matching_results.rule_node)
        } else {
            None
        };

        let mut visited_style = None;
        let should_compute_visited_style =
            relevant_link_found ||
            parent_style.and_then(|s| s.get_visited_style()).is_some();

        let pseudo = self.element.implemented_pseudo_element();
        if should_compute_visited_style {
            visited_style = Some(self.cascade_style(
                visited_rules.as_ref().or(Some(&primary_results.rule_node)),
                /* style_if_visited = */ None,
                parent_style,
                layout_parent_style,
                CascadeVisitedMode::Visited,
                /* pseudo = */ pseudo.as_ref(),
            ));
        }
        let style = self.cascade_style(
            Some(&primary_results.rule_node),
            visited_style,
            parent_style,
            layout_parent_style,
            CascadeVisitedMode::Unvisited,
            /* pseudo = */ pseudo.as_ref(),
        );

        PrimaryStyle { style, }
    }


    /// Resolve the style of a given element, and all its eager pseudo-elements.
    pub fn resolve_style(
        &mut self,
        parent_style: Option<&ComputedValues>,
        layout_parent_style: Option<&ComputedValues>,
    ) -> ElementStyles {
        use properties::longhands::display::computed_value::T as display;

        let primary_style =
            self.resolve_primary_style(parent_style, layout_parent_style);

        let mut pseudo_styles = EagerPseudoStyles::default();
        if primary_style.style.get_box().clone_display() == display::none {
            return ElementStyles {
                // FIXME(emilio): Remove the Option<>.
                primary: Some(primary_style.style),
                pseudos: pseudo_styles,
            }
        }

        if self.element.implemented_pseudo_element().is_none() {
            let layout_parent_style_for_pseudo =
                if primary_style.style.is_display_contents() {
                    layout_parent_style
                } else {
                    Some(&*primary_style.style)
                };
            SelectorImpl::each_eagerly_cascaded_pseudo_element(|pseudo| {
                let pseudo_style = self.resolve_pseudo_style(
                    &pseudo,
                    &primary_style,
                    layout_parent_style_for_pseudo
                );
                if let Some(style) = pseudo_style {
                    pseudo_styles.set(&pseudo, style);
                }
            })
        }

        ElementStyles {
            // FIXME(emilio): Remove the Option<>.
            primary: Some(primary_style.style),
            pseudos: pseudo_styles,
        }
    }

    /// Resolve an element's styles with the default inheritance parent/layout
    /// parents.
    pub fn resolve_style_with_default_parents(&mut self) -> ElementStyles {
        with_default_parent_styles(self.element, |parent_style, layout_parent_style| {
            self.resolve_style(parent_style, layout_parent_style)
        })
    }

    /// Cascade a set of rules, using the default parent for inheritance.
    pub fn cascade_style_and_visited_with_default_parents(
        &mut self,
        inputs: CascadeInputs,
    ) -> Arc<ComputedValues> {
        with_default_parent_styles(self.element, |parent_style, layout_parent_style| {
            self.cascade_style_and_visited(
                inputs,
                parent_style,
                layout_parent_style,
                /* pseudo = */ None
            )
        })
    }

    fn cascade_style_and_visited(
        &mut self,
        inputs: CascadeInputs,
        parent_style: Option<&ComputedValues>,
        layout_parent_style: Option<&ComputedValues>,
        pseudo: Option<&PseudoElement>,
    ) -> Arc<ComputedValues> {
        let mut style_if_visited = None;
        if parent_style.map_or(false, |s| s.get_visited_style().is_some()) ||
            inputs.visited_rules.is_some() {
            style_if_visited = Some(self.cascade_style(
                inputs.visited_rules.as_ref().or(inputs.rules.as_ref()),
                /* style_if_visited = */ None,
                parent_style,
                layout_parent_style,
                CascadeVisitedMode::Visited,
                pseudo,
            ));
        }
        self.cascade_style(
            inputs.rules.as_ref(),
            style_if_visited,
            parent_style,
            layout_parent_style,
            CascadeVisitedMode::Unvisited,
            pseudo,
        )
    }

    /// Cascade the element and pseudo-element styles with the default parents.
    pub fn cascade_styles_with_default_parents(
        &mut self,
        inputs: ElementCascadeInputs,
    ) -> ElementStyles {
        use properties::longhands::display::computed_value::T as display;
        with_default_parent_styles(self.element, move |parent_style, layout_parent_style| {
            let primary_style = PrimaryStyle {
                style: self.cascade_style_and_visited(
                   inputs.primary,
                   parent_style,
                   layout_parent_style,
                   /* pseudo = */ None,
                ),
            };

            let mut pseudo_styles = EagerPseudoStyles::default();
            let pseudo_array = inputs.pseudos.into_array();
            if pseudo_array.is_none() ||
                primary_style.style.get_box().clone_display() == display::none {
                return ElementStyles {
                    primary: Some(primary_style.style),
                    pseudos: pseudo_styles,
                }
            }

            {
                let layout_parent_style_for_pseudo =
                    if primary_style.style.is_display_contents() {
                        layout_parent_style
                    } else {
                        Some(&*primary_style.style)
                    };

                for (i, mut inputs) in pseudo_array.unwrap().iter_mut().enumerate() {
                    if let Some(inputs) = inputs.take() {
                        let pseudo = PseudoElement::from_eager_index(i);
                        pseudo_styles.set(
                            &pseudo,
                            self.cascade_style_and_visited(
                                inputs,
                                Some(&*primary_style.style),
                                layout_parent_style_for_pseudo,
                                Some(&pseudo),
                            )
                        )
                    }
                }
            }

            ElementStyles {
                primary: Some(primary_style.style),
                pseudos: pseudo_styles,
            }
        })
    }

    fn resolve_pseudo_style(
        &mut self,
        pseudo: &PseudoElement,
        originating_element_style: &PrimaryStyle,
        layout_parent_style: Option<&ComputedValues>,
    ) -> Option<Arc<ComputedValues>> {
        let rules = self.match_pseudo(
            &originating_element_style.style,
            pseudo,
            VisitedHandlingMode::AllLinksUnvisited
        );
        let rules = match rules {
            Some(rules) => rules,
            None => return None,
        };

        let mut visited_rules = None;
        if originating_element_style.style.get_visited_style().is_some() {
            visited_rules = self.match_pseudo(
                &originating_element_style.style,
                pseudo,
                VisitedHandlingMode::RelevantLinkVisited,
            );
        }

        Some(self.cascade_style_and_visited(
            CascadeInputs {
                rules: Some(rules),
                visited_rules
            },
            Some(&originating_element_style.style),
            layout_parent_style,
            Some(pseudo),
        ))
    }

    fn match_primary(
        &mut self,
        visited_handling: VisitedHandlingMode,
    ) -> MatchingResults {
        debug!("Match primary for {:?}, visited: {:?}",
               self.element, visited_handling);
        let mut applicable_declarations = ApplicableDeclarationList::new();

        let map = &mut self.context.thread_local.selector_flags;
        let bloom_filter = self.context.thread_local.bloom_filter.filter();
        let mut matching_context =
            MatchingContext::new_for_visited(
                MatchingMode::Normal,
                Some(bloom_filter),
                visited_handling,
                self.context.shared.quirks_mode
            );

        let stylist = &self.context.shared.stylist;
        let implemented_pseudo = self.element.implemented_pseudo_element();
        {
            let resolving_element = self.element;
            let mut set_selector_flags = |element: &E, flags: ElementSelectorFlags| {
                resolving_element.apply_selector_flags(map, element, flags);
            };

            // Compute the primary rule node.
            stylist.push_applicable_declarations(
                &self.element,
                implemented_pseudo.as_ref(),
                self.element.style_attribute(),
                self.element.get_smil_override(),
                self.element.get_animation_rules(),
                self.rule_inclusion,
                &mut applicable_declarations,
                &mut matching_context,
                &mut set_selector_flags,
            );
        }

        // FIXME(emilio): This is a hack for animations, and should go away.
        self.element.unset_dirty_style_attribute();

        let relevant_link_found = matching_context.relevant_link_found;
        let rule_node = stylist.rule_tree().compute_rule_node(
            &mut applicable_declarations,
            &self.context.shared.guards
        );

        if log_enabled!(Trace) {
            trace!("Matched rules:");
            for rn in rule_node.self_and_ancestors() {
                let source = rn.style_source();
                if source.is_some() {
                    trace!(" > {:?}", source);
                }
            }
        }

        MatchingResults { rule_node, relevant_link_found }
    }

    fn match_pseudo(
        &mut self,
        originating_element_style: &ComputedValues,
        pseudo_element: &PseudoElement,
        visited_handling: VisitedHandlingMode,
    ) -> Option<StrongRuleNode> {
        debug!("Match pseudo {:?} for {:?}, visited: {:?}",
               self.element, pseudo_element, visited_handling);
        debug_assert!(pseudo_element.is_eager() || pseudo_element.is_lazy());
        debug_assert!(self.element.implemented_pseudo_element().is_none(),
                      "Element pseudos can't have any other pseudo.");

        let mut applicable_declarations = ApplicableDeclarationList::new();

        let stylist = &self.context.shared.stylist;

        if !self.element.may_generate_pseudo(pseudo_element, originating_element_style) {
            return None;
        }

        let bloom_filter = self.context.thread_local.bloom_filter.filter();

        let mut matching_context =
            MatchingContext::new_for_visited(
                MatchingMode::ForStatelessPseudoElement,
                Some(bloom_filter),
                visited_handling,
                self.context.shared.quirks_mode
            );

        let map = &mut self.context.thread_local.selector_flags;
        let resolving_element = self.element;
        let mut set_selector_flags = |element: &E, flags: ElementSelectorFlags| {
            resolving_element.apply_selector_flags(map, element, flags);
        };

        // NB: We handle animation rules for ::before and ::after when
        // traversing them.
        stylist.push_applicable_declarations(
            &self.element,
            Some(pseudo_element),
            None,
            None,
            AnimationRules(None, None),
            self.rule_inclusion,
            &mut applicable_declarations,
            &mut matching_context,
            &mut set_selector_flags
        );

        if applicable_declarations.is_empty() {
            return None;
        }

        let rule_node = stylist.rule_tree().compute_rule_node(
            &mut applicable_declarations,
            &self.context.shared.guards
        );

        Some(rule_node)
    }

    fn cascade_style(
        &mut self,
        rules: Option<&StrongRuleNode>,
        style_if_visited: Option<Arc<ComputedValues>>,
        mut parent_style: Option<&ComputedValues>,
        layout_parent_style: Option<&ComputedValues>,
        cascade_visited: CascadeVisitedMode,
        pseudo: Option<&PseudoElement>,
    ) -> Arc<ComputedValues> {
        let mut cascade_info = CascadeInfo::new();
        let mut cascade_flags = CascadeFlags::empty();

        if self.element.skip_root_and_item_based_display_fixup() {
            cascade_flags.insert(SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP);
        }
        if cascade_visited.visited_dependent_only() {
            // If this element is a link, we want its visited style to inherit
            // from the regular style of its parent, because only the
            // visitedness of the relevant link should influence style.
            if pseudo.is_some() || !self.element.is_link() {
                parent_style = parent_style.map(|s| {
                    s.get_visited_style().unwrap_or(s)
                });
            }
            cascade_flags.insert(VISITED_DEPENDENT_ONLY);
        }
        if self.element.is_native_anonymous() || pseudo.is_some() {
            cascade_flags.insert(PROHIBIT_DISPLAY_CONTENTS);
        } else if self.element.is_root() {
            cascade_flags.insert(IS_ROOT_ELEMENT);
        }

        let implemented_pseudo = self.element.implemented_pseudo_element();
        let values =
            cascade(
                self.context.shared.stylist.device(),
                pseudo.or(implemented_pseudo.as_ref()),
                rules.unwrap_or(self.context.shared.stylist.rule_tree().root()),
                &self.context.shared.guards,
                parent_style,
                layout_parent_style,
                style_if_visited,
                Some(&mut cascade_info),
                &self.context.thread_local.font_metrics_provider,
                cascade_flags,
                self.context.shared.quirks_mode
            );

        cascade_info.finish(&self.element.as_node());

        values
    }
}
