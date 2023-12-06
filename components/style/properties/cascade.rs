/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The main cascading algorithm of the style system.

use crate::applicable_declarations::CascadePriority;
use crate::color::AbsoluteColor;
use crate::computed_value_flags::ComputedValueFlags;
use crate::context::QuirksMode;
use crate::custom_properties::CustomPropertiesBuilder;
use crate::dom::TElement;
use crate::logical_geometry::WritingMode;
use crate::media_queries::Device;
use crate::properties::declaration_block::{DeclarationImportanceIterator, Importance};
use crate::properties::generated::{
    CSSWideKeyword, ComputedValues, LonghandId, LonghandIdSet, PropertyDeclaration,
    PropertyDeclarationId, PropertyFlags, ShorthandsWithPropertyReferencesCache, StyleBuilder,
    CASCADE_PROPERTY,
};
use crate::rule_cache::{RuleCache, RuleCacheConditions};
use crate::rule_tree::{CascadeLevel, StrongRuleNode};
use crate::selector_parser::PseudoElement;
use crate::shared_lock::StylesheetGuards;
use crate::style_adjuster::StyleAdjuster;
use crate::stylesheets::container_rule::ContainerSizeQuery;
use crate::stylesheets::{layer_rule::LayerOrder, Origin};
#[cfg(feature = "gecko")]
use crate::values::specified::length::FontBaseSize;
use crate::values::{computed, specified};
use fxhash::FxHashMap;
use servo_arc::Arc;
use smallvec::SmallVec;
use std::borrow::Cow;
#[cfg(feature = "gecko")]
use std::mem;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CanHaveLogicalProperties {
    No,
    Yes,
}

/// Performs the CSS cascade, computing new styles for an element from its parent style.
///
/// The arguments are:
///
///   * `device`: Used to get the initial viewport and other external state.
///
///   * `rule_node`: The rule node in the tree that represent the CSS rules that
///   matched.
///
///   * `parent_style`: The parent style, if applicable; if `None`, this is the root node.
///
/// Returns the computed values.
///   * `flags`: Various flags.
///
pub fn cascade<E>(
    device: &Device,
    pseudo: Option<&PseudoElement>,
    rule_node: &StrongRuleNode,
    guards: &StylesheetGuards,
    originating_element_style: Option<&ComputedValues>,
    parent_style: Option<&ComputedValues>,
    parent_style_ignoring_first_line: Option<&ComputedValues>,
    layout_parent_style: Option<&ComputedValues>,
    visited_rules: Option<&StrongRuleNode>,
    cascade_input_flags: ComputedValueFlags,
    quirks_mode: QuirksMode,
    rule_cache: Option<&RuleCache>,
    rule_cache_conditions: &mut RuleCacheConditions,
    element: Option<E>,
) -> Arc<ComputedValues>
where
    E: TElement,
{
    cascade_rules(
        device,
        pseudo,
        rule_node,
        guards,
        originating_element_style,
        parent_style,
        parent_style_ignoring_first_line,
        layout_parent_style,
        CascadeMode::Unvisited { visited_rules },
        cascade_input_flags,
        quirks_mode,
        rule_cache,
        rule_cache_conditions,
        element,
    )
}

struct DeclarationIterator<'a> {
    // Global to the iteration.
    guards: &'a StylesheetGuards<'a>,
    restriction: Option<PropertyFlags>,
    // The rule we're iterating over.
    current_rule_node: Option<&'a StrongRuleNode>,
    // Per rule state.
    declarations: DeclarationImportanceIterator<'a>,
    origin: Origin,
    importance: Importance,
    priority: CascadePriority,
}

impl<'a> DeclarationIterator<'a> {
    #[inline]
    fn new(
        rule_node: &'a StrongRuleNode,
        guards: &'a StylesheetGuards,
        pseudo: Option<&PseudoElement>,
    ) -> Self {
        let restriction = pseudo.and_then(|p| p.property_restriction());
        let mut iter = Self {
            guards,
            current_rule_node: Some(rule_node),
            origin: Origin::UserAgent,
            importance: Importance::Normal,
            priority: CascadePriority::new(CascadeLevel::UANormal, LayerOrder::root()),
            declarations: DeclarationImportanceIterator::default(),
            restriction,
        };
        iter.update_for_node(rule_node);
        iter
    }

    fn update_for_node(&mut self, node: &'a StrongRuleNode) {
        self.priority = node.cascade_priority();
        let level = self.priority.cascade_level();
        self.origin = level.origin();
        self.importance = level.importance();
        let guard = match self.origin {
            Origin::Author => self.guards.author,
            Origin::User | Origin::UserAgent => self.guards.ua_or_user,
        };
        self.declarations = match node.style_source() {
            Some(source) => source.read(guard).declaration_importance_iter(),
            None => DeclarationImportanceIterator::default(),
        };
    }
}

impl<'a> Iterator for DeclarationIterator<'a> {
    type Item = (&'a PropertyDeclaration, CascadePriority);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((decl, importance)) = self.declarations.next_back() {
                if self.importance != importance {
                    continue;
                }

                if let Some(restriction) = self.restriction {
                    // decl.id() is either a longhand or a custom
                    // property.  Custom properties are always allowed, but
                    // longhands are only allowed if they have our
                    // restriction flag set.
                    if let PropertyDeclarationId::Longhand(id) = decl.id() {
                        if !id.flags().contains(restriction) && self.origin != Origin::UserAgent {
                            continue;
                        }
                    }
                }

                return Some((decl, self.priority));
            }

            let next_node = self.current_rule_node.take()?.parent()?;
            self.current_rule_node = Some(next_node);
            self.update_for_node(next_node);
        }
    }
}

fn cascade_rules<E>(
    device: &Device,
    pseudo: Option<&PseudoElement>,
    rule_node: &StrongRuleNode,
    guards: &StylesheetGuards,
    originating_element_style: Option<&ComputedValues>,
    parent_style: Option<&ComputedValues>,
    parent_style_ignoring_first_line: Option<&ComputedValues>,
    layout_parent_style: Option<&ComputedValues>,
    cascade_mode: CascadeMode,
    cascade_input_flags: ComputedValueFlags,
    quirks_mode: QuirksMode,
    rule_cache: Option<&RuleCache>,
    rule_cache_conditions: &mut RuleCacheConditions,
    element: Option<E>,
) -> Arc<ComputedValues>
where
    E: TElement,
{
    debug_assert_eq!(
        parent_style.is_some(),
        parent_style_ignoring_first_line.is_some()
    );
    apply_declarations(
        device,
        pseudo,
        rule_node,
        guards,
        DeclarationIterator::new(rule_node, guards, pseudo),
        originating_element_style,
        parent_style,
        parent_style_ignoring_first_line,
        layout_parent_style,
        cascade_mode,
        cascade_input_flags,
        quirks_mode,
        rule_cache,
        rule_cache_conditions,
        element,
    )
}

/// Whether we're cascading for visited or unvisited styles.
#[derive(Clone, Copy)]
pub enum CascadeMode<'a> {
    /// We're cascading for unvisited styles.
    Unvisited {
        /// The visited rules that should match the visited style.
        visited_rules: Option<&'a StrongRuleNode>,
    },
    /// We're cascading for visited styles.
    Visited {
        /// The writing mode of our unvisited style, needed to correctly resolve
        /// logical properties..
        writing_mode: WritingMode,
    },
}

/// NOTE: This function expects the declaration with more priority to appear
/// first.
pub fn apply_declarations<'a, E, I>(
    device: &Device,
    pseudo: Option<&PseudoElement>,
    rules: &StrongRuleNode,
    guards: &StylesheetGuards,
    iter: I,
    originating_element_style: Option<&ComputedValues>,
    parent_style: Option<&ComputedValues>,
    parent_style_ignoring_first_line: Option<&ComputedValues>,
    layout_parent_style: Option<&ComputedValues>,
    cascade_mode: CascadeMode,
    cascade_input_flags: ComputedValueFlags,
    quirks_mode: QuirksMode,
    rule_cache: Option<&RuleCache>,
    rule_cache_conditions: &mut RuleCacheConditions,
    element: Option<E>,
) -> Arc<ComputedValues>
where
    E: TElement,
    I: Iterator<Item = (&'a PropertyDeclaration, CascadePriority)>,
{
    debug_assert_eq!(
        originating_element_style.is_some(),
        element.is_some() && pseudo.is_some()
    );
    debug_assert!(layout_parent_style.is_none() || parent_style.is_some());
    debug_assert_eq!(
        parent_style.is_some(),
        parent_style_ignoring_first_line.is_some()
    );
    #[cfg(feature = "gecko")]
    debug_assert!(
        parent_style.is_none() ||
            ::std::ptr::eq(
                parent_style.unwrap(),
                parent_style_ignoring_first_line.unwrap()
            ) ||
            parent_style.unwrap().is_first_line_style()
    );

    let inherited_style = parent_style.unwrap_or(device.default_computed_values());

    let mut declarations = SmallVec::<[(&_, CascadePriority); 32]>::new();
    let mut referenced_properties = LonghandIdSet::default();
    let custom_properties = {
        let mut builder = CustomPropertiesBuilder::new(inherited_style.custom_properties(), device);

        for (declaration, priority) in iter {
            declarations.push((declaration, priority));
            if let PropertyDeclaration::Custom(ref declaration) = *declaration {
                builder.cascade(declaration, priority);
            } else {
                referenced_properties.insert(declaration.id().as_longhand().unwrap());
            }
        }

        builder.build()
    };

    let is_root_element = pseudo.is_none() && element.map_or(false, |e| e.is_root());
    let container_size_query =
        ContainerSizeQuery::for_option_element(element, originating_element_style);

    let mut context = computed::Context::new(
        // We'd really like to own the rules here to avoid refcount traffic, but
        // animation's usage of `apply_declarations` make this tricky. See bug
        // 1375525.
        StyleBuilder::new(
            device,
            parent_style,
            parent_style_ignoring_first_line,
            pseudo,
            Some(rules.clone()),
            custom_properties,
            is_root_element,
        ),
        quirks_mode,
        rule_cache_conditions,
        container_size_query,
    );

    context.style().add_flags(cascade_input_flags);

    let using_cached_reset_properties;
    let mut cascade = Cascade::new(&mut context, cascade_mode, &referenced_properties);
    let mut shorthand_cache = ShorthandsWithPropertyReferencesCache::default();

    let properties_to_apply = match cascade.cascade_mode {
        CascadeMode::Visited { writing_mode } => {
            cascade.context.builder.writing_mode = writing_mode;
            // We never insert visited styles into the cache so we don't need to
            // try looking it up. It also wouldn't be super-profitable, only a
            // handful reset properties are non-inherited.
            using_cached_reset_properties = false;
            LonghandIdSet::visited_dependent()
        },
        CascadeMode::Unvisited { visited_rules } => {
            if cascade.apply_properties(
                CanHaveLogicalProperties::No,
                LonghandIdSet::writing_mode_group(),
                declarations.iter().cloned(),
                &mut shorthand_cache,
            ) {
                cascade.compute_writing_mode();
            }

            if cascade.apply_properties(
                CanHaveLogicalProperties::No,
                LonghandIdSet::fonts_and_color_group(),
                declarations.iter().cloned(),
                &mut shorthand_cache,
            ) {
                cascade.fixup_font_stuff();
            }

            if let Some(visited_rules) = visited_rules {
                cascade.compute_visited_style_if_needed(
                    element,
                    originating_element_style,
                    parent_style,
                    parent_style_ignoring_first_line,
                    layout_parent_style,
                    visited_rules,
                    guards,
                );
            }

            using_cached_reset_properties =
                cascade.try_to_use_cached_reset_properties(rule_cache, guards);

            if using_cached_reset_properties {
                LonghandIdSet::late_group_only_inherited()
            } else {
                LonghandIdSet::late_group()
            }
        },
    };

    cascade.apply_properties(
        CanHaveLogicalProperties::Yes,
        properties_to_apply,
        declarations.iter().cloned(),
        &mut shorthand_cache,
    );

    cascade.finished_applying_properties();

    context.builder.clear_modified_reset();

    if matches!(cascade_mode, CascadeMode::Unvisited { .. }) {
        StyleAdjuster::new(&mut context.builder)
            .adjust(layout_parent_style.unwrap_or(inherited_style), element);
    }

    if context.builder.modified_reset() || using_cached_reset_properties {
        // If we adjusted any reset structs, we can't cache this ComputedValues.
        //
        // Also, if we re-used existing reset structs, don't bother caching it
        // back again. (Aside from being wasted effort, it will be wrong, since
        // context.rule_cache_conditions won't be set appropriately if we didn't
        // compute those reset properties.)
        context.rule_cache_conditions.borrow_mut().set_uncacheable();
    }

    context.builder.build()
}

/// For ignored colors mode, we sometimes want to do something equivalent to
/// "revert-or-initial", where we `revert` for a given origin, but then apply a
/// given initial value if nothing in other origins did override it.
///
/// This is a bit of a clunky way of achieving this.
type DeclarationsToApplyUnlessOverriden = SmallVec<[PropertyDeclaration; 2]>;

fn tweak_when_ignoring_colors(
    context: &computed::Context,
    longhand_id: LonghandId,
    origin: Origin,
    declaration: &mut Cow<PropertyDeclaration>,
    declarations_to_apply_unless_overriden: &mut DeclarationsToApplyUnlessOverriden,
) {
    use crate::values::computed::ToComputedValue;
    use crate::values::specified::Color;

    if !longhand_id.ignored_when_document_colors_disabled() {
        return;
    }

    let is_ua_or_user_rule = matches!(origin, Origin::User | Origin::UserAgent);
    if is_ua_or_user_rule {
        return;
    }

    // Always honor colors if forced-color-adjust is set to none.
    #[cfg(feature = "gecko")] {
        let forced = context
            .builder
            .get_inherited_text()
            .clone_forced_color_adjust();
        if forced == computed::ForcedColorAdjust::None {
            return;
        }
    }

    // Don't override background-color on ::-moz-color-swatch. It is set as an
    // author style (via the style attribute), but it's pretty important for it
    // to show up for obvious reasons :)
    if context
        .builder
        .pseudo
        .map_or(false, |p| p.is_color_swatch()) &&
        longhand_id == LonghandId::BackgroundColor
    {
        return;
    }

    fn alpha_channel(color: &Color, context: &computed::Context) -> f32 {
        // We assume here currentColor is opaque.
        color
            .to_computed_value(context)
            .resolve_to_absolute(&AbsoluteColor::black())
            .alpha
    }

    // A few special-cases ahead.
    match **declaration {
        PropertyDeclaration::BackgroundColor(ref color) => {
            // We honor system colors and transparent colors unconditionally.
            //
            // NOTE(emilio): We honor transparent unconditionally, like we do
            // for color, even though it causes issues like bug 1625036. The
            // reasoning is that the conditions that trigger that (having
            // mismatched widget and default backgrounds) are both uncommon, and
            // broken in other applications as well, and not honoring
            // transparent makes stuff uglier or break unconditionally
            // (bug 1666059, bug 1755713).
            if color.honored_in_forced_colors_mode(/* allow_transparent = */ true) {
                return;
            }
            // For background-color, we revert or initial-with-preserved-alpha
            // otherwise, this is needed to preserve semi-transparent
            // backgrounds.
            let alpha = alpha_channel(color, context);
            if alpha == 0.0 {
                return;
            }
            let mut color = context.builder.device.default_background_color();
            color.alpha = alpha;
            declarations_to_apply_unless_overriden
                .push(PropertyDeclaration::BackgroundColor(color.into()))
        },
        PropertyDeclaration::Color(ref color) => {
            // We honor color: transparent and system colors.
            if color
                .0
                .honored_in_forced_colors_mode(/* allow_transparent = */ true)
            {
                return;
            }
            // If the inherited color would be transparent, but we would
            // override this with a non-transparent color, then override it with
            // the default color. Otherwise just let it inherit through.
            if context
                .builder
                .get_parent_inherited_text()
                .clone_color()
                .alpha ==
                0.0
            {
                let color = context.builder.device.default_color();
                declarations_to_apply_unless_overriden.push(PropertyDeclaration::Color(
                    specified::ColorPropertyValue(color.into()),
                ))
            }
        },
        // We honor url background-images if backplating.
        #[cfg(feature = "gecko")]
        PropertyDeclaration::BackgroundImage(ref bkg) => {
            use crate::values::generics::image::Image;
            if static_prefs::pref!("browser.display.permit_backplate") {
                if bkg
                    .0
                    .iter()
                    .all(|image| matches!(*image, Image::Url(..) | Image::None))
                {
                    return;
                }
            }
        },
        _ => {
            // We honor system colors more generally for all colors.
            //
            // We used to honor transparent but that causes accessibility
            // regressions like bug 1740924.
            //
            // NOTE(emilio): This doesn't handle caret-color and accent-color
            // because those use a slightly different syntax (<color> | auto for
            // example).
            //
            // That's probably fine though, as using a system color for
            // caret-color doesn't make sense (using currentColor is fine), and
            // we ignore accent-color in high-contrast-mode anyways.
            if let Some(color) = declaration.color_value() {
                if color.honored_in_forced_colors_mode(/* allow_transparent = */ false) {
                    return;
                }
            }
        },
    }

    *declaration.to_mut() =
        PropertyDeclaration::css_wide_keyword(longhand_id, CSSWideKeyword::Revert);
}

struct Cascade<'a, 'b: 'a> {
    context: &'a mut computed::Context<'b>,
    cascade_mode: CascadeMode<'a>,
    /// All the properties that have a declaration in the cascade.
    referenced: &'a LonghandIdSet,
    seen: LonghandIdSet,
    author_specified: LonghandIdSet,
    reverted_set: LonghandIdSet,
    reverted: FxHashMap<LonghandId, (CascadePriority, bool)>,
}

impl<'a, 'b: 'a> Cascade<'a, 'b> {
    fn new(
        context: &'a mut computed::Context<'b>,
        cascade_mode: CascadeMode<'a>,
        referenced: &'a LonghandIdSet,
    ) -> Self {
        Self {
            context,
            cascade_mode,
            referenced,
            seen: LonghandIdSet::default(),
            author_specified: LonghandIdSet::default(),
            reverted_set: Default::default(),
            reverted: Default::default(),
        }
    }

    fn substitute_variables_if_needed<'decl, 'cache>(
        &mut self,
        declaration: &'decl PropertyDeclaration,
        cache: &'cache mut ShorthandsWithPropertyReferencesCache,
    ) -> Cow<'decl, PropertyDeclaration>
    where
        'cache: 'decl,
    {
        let declaration = match *declaration {
            PropertyDeclaration::WithVariables(ref declaration) => declaration,
            ref d => return Cow::Borrowed(d),
        };

        if !declaration.id.inherited() {
            self.context
                .rule_cache_conditions
                .borrow_mut()
                .set_uncacheable();

            // NOTE(emilio): We only really need to add the `display` /
            // `content` flag if the CSS variable has not been specified on our
            // declarations, but we don't have that information at this point,
            // and it doesn't seem like an important enough optimization to
            // warrant it.
            match declaration.id {
                LonghandId::Display => {
                    self.context
                        .builder
                        .add_flags(ComputedValueFlags::DISPLAY_DEPENDS_ON_INHERITED_STYLE);
                },
                LonghandId::Content => {
                    self.context
                        .builder
                        .add_flags(ComputedValueFlags::CONTENT_DEPENDS_ON_INHERITED_STYLE);
                },
                _ => {},
            }
        }

        declaration.value.substitute_variables(
            declaration.id,
            self.context.builder.writing_mode,
            self.context.builder.custom_properties(),
            self.context.quirks_mode,
            self.context.device(),
            cache,
        )
    }

    #[inline(always)]
    fn apply_declaration(&mut self, longhand_id: LonghandId, declaration: &PropertyDeclaration) {
        // We could (and used to) use a pattern match here, but that bloats this
        // function to over 100K of compiled code!
        //
        // To improve i-cache behavior, we outline the individual functions and
        // use virtual dispatch instead.
        let discriminant = longhand_id as usize;
        (CASCADE_PROPERTY[discriminant])(declaration, &mut self.context);
    }

    fn apply_properties<'decls, I>(
        &mut self,
        can_have_logical_properties: CanHaveLogicalProperties,
        properties_to_apply: &'a LonghandIdSet,
        declarations: I,
        mut shorthand_cache: &mut ShorthandsWithPropertyReferencesCache,
    ) -> bool
    where
        I: Iterator<Item = (&'decls PropertyDeclaration, CascadePriority)>,
    {
        if !self.referenced.contains_any(properties_to_apply) {
            return false;
        }

        let can_have_logical_properties =
            can_have_logical_properties == CanHaveLogicalProperties::Yes;

        let ignore_colors = !self.context.builder.device.use_document_colors();
        let mut declarations_to_apply_unless_overriden = DeclarationsToApplyUnlessOverriden::new();

        for (declaration, priority) in declarations {
            let origin = priority.cascade_level().origin();

            let declaration_id = declaration.id();
            let longhand_id = match declaration_id {
                PropertyDeclarationId::Longhand(id) => id,
                PropertyDeclarationId::Custom(..) => continue,
            };

            if !properties_to_apply.contains(longhand_id) {
                continue;
            }

            debug_assert!(can_have_logical_properties || !longhand_id.is_logical());
            let physical_longhand_id = if can_have_logical_properties {
                longhand_id.to_physical(self.context.builder.writing_mode)
            } else {
                longhand_id
            };

            if self.seen.contains(physical_longhand_id) {
                continue;
            }

            if self.reverted_set.contains(physical_longhand_id) {
                if let Some(&(reverted_priority, is_origin_revert)) =
                    self.reverted.get(&physical_longhand_id)
                {
                    if !reverted_priority.allows_when_reverted(&priority, is_origin_revert) {
                        continue;
                    }
                }
            }

            let mut declaration =
                self.substitute_variables_if_needed(declaration, &mut shorthand_cache);

            // When document colors are disabled, do special handling of
            // properties that are marked as ignored in that mode.
            if ignore_colors {
                tweak_when_ignoring_colors(
                    &self.context,
                    longhand_id,
                    origin,
                    &mut declaration,
                    &mut declarations_to_apply_unless_overriden,
                );
                debug_assert_eq!(
                    declaration.id(),
                    PropertyDeclarationId::Longhand(longhand_id),
                    "Shouldn't change the declaration id!",
                );
            }

            let is_unset = match declaration.get_css_wide_keyword() {
                Some(keyword) => match keyword {
                    CSSWideKeyword::RevertLayer | CSSWideKeyword::Revert => {
                        let origin_revert = keyword == CSSWideKeyword::Revert;
                        // We intentionally don't want to insert it into
                        // `self.seen`, `reverted` takes care of rejecting other
                        // declarations as needed.
                        self.reverted_set.insert(physical_longhand_id);
                        self.reverted
                            .insert(physical_longhand_id, (priority, origin_revert));
                        continue;
                    },
                    CSSWideKeyword::Unset => true,
                    CSSWideKeyword::Inherit => longhand_id.inherited(),
                    CSSWideKeyword::Initial => !longhand_id.inherited(),
                },
                None => false,
            };

            self.seen.insert(physical_longhand_id);
            if origin == Origin::Author {
                self.author_specified.insert(physical_longhand_id);
            }

            if is_unset {
                continue;
            }

            // FIXME(emilio): We should avoid generating code for logical
            // longhands and just use the physical ones, then rename
            // physical_longhand_id to just longhand_id.
            self.apply_declaration(longhand_id, &*declaration);
        }

        if ignore_colors {
            for declaration in declarations_to_apply_unless_overriden.iter() {
                let longhand_id = match declaration.id() {
                    PropertyDeclarationId::Longhand(id) => id,
                    PropertyDeclarationId::Custom(..) => unreachable!(),
                };
                debug_assert!(!longhand_id.is_logical());
                if self.seen.contains(longhand_id) {
                    continue;
                }
                self.apply_declaration(longhand_id, declaration);
            }
        }

        true
    }

    fn compute_writing_mode(&mut self) {
        debug_assert!(matches!(self.cascade_mode, CascadeMode::Unvisited { .. }));
        self.context.builder.writing_mode =
            WritingMode::new(self.context.builder.get_inherited_box())
    }

    fn compute_visited_style_if_needed<E>(
        &mut self,
        element: Option<E>,
        originating_element_style: Option<&ComputedValues>,
        parent_style: Option<&ComputedValues>,
        parent_style_ignoring_first_line: Option<&ComputedValues>,
        layout_parent_style: Option<&ComputedValues>,
        visited_rules: &StrongRuleNode,
        guards: &StylesheetGuards,
    ) where
        E: TElement,
    {
        debug_assert!(matches!(self.cascade_mode, CascadeMode::Unvisited { .. }));
        let is_link = self.context.builder.pseudo.is_none() && element.unwrap().is_link();

        macro_rules! visited_parent {
            ($parent:expr) => {
                if is_link {
                    $parent
                } else {
                    $parent.map(|p| p.visited_style().unwrap_or(p))
                }
            };
        }

        let writing_mode = self.context.builder.writing_mode;

        // We could call apply_declarations directly, but that'd cause
        // another instantiation of this function which is not great.
        let style = cascade_rules(
            self.context.builder.device,
            self.context.builder.pseudo,
            visited_rules,
            guards,
            visited_parent!(originating_element_style),
            visited_parent!(parent_style),
            visited_parent!(parent_style_ignoring_first_line),
            visited_parent!(layout_parent_style),
            CascadeMode::Visited { writing_mode },
            // Cascade input flags don't matter for the visited style, they are
            // in the main (unvisited) style.
            Default::default(),
            self.context.quirks_mode,
            // The rule cache doesn't care about caching :visited
            // styles, we cache the unvisited style instead. We still do
            // need to set the caching dependencies properly if present
            // though, so the cache conditions need to match.
            None, // rule_cache
            &mut *self.context.rule_cache_conditions.borrow_mut(),
            element,
        );
        self.context.builder.visited_style = Some(style);
    }

    fn finished_applying_properties(&mut self) {
        let builder = &mut self.context.builder;

        #[cfg(feature = "gecko")]
        {
            if let Some(bg) = builder.get_background_if_mutated() {
                bg.fill_arrays();
            }

            if let Some(svg) = builder.get_svg_if_mutated() {
                svg.fill_arrays();
            }
        }

        if self
            .author_specified
            .contains_any(LonghandIdSet::border_background_properties())
        {
            builder.add_flags(ComputedValueFlags::HAS_AUTHOR_SPECIFIED_BORDER_BACKGROUND);
        }

        if self.author_specified.contains(LonghandId::FontFamily) {
            builder.add_flags(ComputedValueFlags::HAS_AUTHOR_SPECIFIED_FONT_FAMILY);
        }

        if self.author_specified.contains(LonghandId::LetterSpacing) {
            builder.add_flags(ComputedValueFlags::HAS_AUTHOR_SPECIFIED_LETTER_SPACING);
        }

        if self.author_specified.contains(LonghandId::WordSpacing) {
            builder.add_flags(ComputedValueFlags::HAS_AUTHOR_SPECIFIED_WORD_SPACING);
        }

        #[cfg(feature = "gecko")]
        if self
            .author_specified
            .contains(LonghandId::FontSynthesisWeight)
        {
            builder.add_flags(ComputedValueFlags::HAS_AUTHOR_SPECIFIED_FONT_SYNTHESIS_WEIGHT);
        }

        #[cfg(feature = "gecko")]
        if self
            .author_specified
            .contains(LonghandId::FontSynthesisStyle)
        {
            builder.add_flags(ComputedValueFlags::HAS_AUTHOR_SPECIFIED_FONT_SYNTHESIS_STYLE);
        }

        #[cfg(feature = "servo")]
        {
            if let Some(font) = builder.get_font_if_mutated() {
                font.compute_font_hash();
            }
        }
    }

    fn try_to_use_cached_reset_properties(
        &mut self,
        cache: Option<&'b RuleCache>,
        guards: &StylesheetGuards,
    ) -> bool {
        let cache = match cache {
            Some(cache) => cache,
            None => return false,
        };

        let builder = &mut self.context.builder;

        let cached_style = match cache.find(guards, &builder) {
            Some(style) => style,
            None => return false,
        };

        builder.copy_reset_from(cached_style);

        // We're using the same reset style as another element, and we'll skip
        // applying the relevant properties. So we need to do the relevant
        // bookkeeping here to keep these bits correct.
        //
        // Note that the border/background properties are non-inherited, so we
        // don't need to do anything else other than just copying the bits over.
        //
        // When using this optimization, we also need to copy whether the old
        // style specified viewport units / used font-relative lengths, this one
        // would as well.  It matches the same rules, so it is the right thing
        // to do anyways, even if it's only used on inherited properties.
        let bits_to_copy = ComputedValueFlags::HAS_AUTHOR_SPECIFIED_BORDER_BACKGROUND |
            ComputedValueFlags::DEPENDS_ON_SELF_FONT_METRICS |
            ComputedValueFlags::DEPENDS_ON_INHERITED_FONT_METRICS |
            ComputedValueFlags::USES_VIEWPORT_UNITS;
        builder.add_flags(cached_style.flags & bits_to_copy);

        true
    }

    /// The initial font depends on the current lang group so we may need to
    /// recompute it if the language changed.
    #[inline]
    #[cfg(feature = "gecko")]
    fn recompute_initial_font_family_if_needed(&mut self) {
        use crate::gecko_bindings::bindings;
        use crate::values::computed::font::FontFamily;

        if !self.seen.contains(LonghandId::XLang) {
            return;
        }

        let builder = &mut self.context.builder;
        let default_font_type = {
            let font = builder.get_font();

            if !font.mFont.family.is_initial {
                return;
            }

            let default_font_type = unsafe {
                bindings::Gecko_nsStyleFont_ComputeFallbackFontTypeForLanguage(
                    builder.device.document(),
                    font.mLanguage.mRawPtr,
                )
            };

            let initial_generic = font.mFont.family.families.single_generic();
            debug_assert!(
                initial_generic.is_some(),
                "Initial font should be just one generic font"
            );
            if initial_generic == Some(default_font_type) {
                return;
            }

            default_font_type
        };

        // NOTE: Leaves is_initial untouched.
        builder.mutate_font().mFont.family.families =
            FontFamily::generic(default_font_type).families.clone();
    }

    /// Prioritize user fonts if needed by pref.
    #[inline]
    #[cfg(feature = "gecko")]
    fn prioritize_user_fonts_if_needed(&mut self) {
        use crate::gecko_bindings::bindings;

        if !self.seen.contains(LonghandId::FontFamily) {
            return;
        }

        if static_prefs::pref!("browser.display.use_document_fonts") != 0 {
            return;
        }

        let builder = &mut self.context.builder;
        let default_font_type = {
            let font = builder.get_font();

            if font.mFont.family.is_system_font {
                return;
            }

            if !font.mFont.family.families.needs_user_font_prioritization() {
                return;
            }

            unsafe {
                bindings::Gecko_nsStyleFont_ComputeFallbackFontTypeForLanguage(
                    builder.device.document(),
                    font.mLanguage.mRawPtr,
                )
            }
        };

        let font = builder.mutate_font();
        font.mFont
            .family
            .families
            .prioritize_first_generic_or_prepend(default_font_type);
    }

    /// Some keyword sizes depend on the font family and language.
    #[cfg(feature = "gecko")]
    fn recompute_keyword_font_size_if_needed(&mut self) {
        use crate::values::computed::ToComputedValue;

        if !self.seen.contains(LonghandId::XLang) && !self.seen.contains(LonghandId::FontFamily) {
            return;
        }

        let new_size = {
            let font = self.context.builder.get_font();
            let info = font.clone_font_size().keyword_info;
            let new_size = match info.kw {
                specified::FontSizeKeyword::None => return,
                _ => {
                    self.context.for_non_inherited_property = false;
                    specified::FontSize::Keyword(info).to_computed_value(self.context)
                },
            };

            if font.mScriptUnconstrainedSize == new_size.computed_size {
                return;
            }

            new_size
        };

        self.context.builder.mutate_font().set_font_size(new_size);
    }

    /// Some properties, plus setting font-size itself, may make us go out of
    /// our minimum font-size range.
    #[cfg(feature = "gecko")]
    fn constrain_font_size_if_needed(&mut self) {
        use crate::gecko_bindings::bindings;
        use crate::values::generics::NonNegative;

        if !self.seen.contains(LonghandId::XLang) &&
            !self.seen.contains(LonghandId::FontFamily) &&
            !self.seen.contains(LonghandId::MozMinFontSizeRatio) &&
            !self.seen.contains(LonghandId::FontSize)
        {
            return;
        }

        let builder = &mut self.context.builder;
        let min_font_size = {
            let font = builder.get_font();
            let min_font_size = unsafe {
                bindings::Gecko_nsStyleFont_ComputeMinSize(&**font, builder.device.document())
            };

            if font.mFont.size.0 >= min_font_size {
                return;
            }

            NonNegative(min_font_size)
        };

        builder.mutate_font().mFont.size = min_font_size;
    }

    /// <svg:text> is not affected by text zoom, and it uses a preshint to disable it. We fix up
    /// the struct when this happens by unzooming its contained font values, which will have been
    /// zoomed in the parent.
    ///
    /// FIXME(emilio): Why doing this _before_ handling font-size? That sounds wrong.
    #[cfg(feature = "gecko")]
    fn unzoom_fonts_if_needed(&mut self) {
        if !self.seen.contains(LonghandId::XTextScale) {
            return;
        }

        let builder = &mut self.context.builder;

        let parent_text_scale = builder.get_parent_font().clone__x_text_scale();
        let text_scale = builder.get_font().clone__x_text_scale();
        if parent_text_scale == text_scale {
            return;
        }
        debug_assert_ne!(
            parent_text_scale.text_zoom_enabled(),
            text_scale.text_zoom_enabled(),
            "There's only one value that disables it"
        );
        debug_assert!(
            !text_scale.text_zoom_enabled(),
            "We only ever disable text zoom (in svg:text), never enable it"
        );
        let device = builder.device;
        builder.mutate_font().unzoom_fonts(device);
    }

    /// MathML script* attributes do some very weird shit with font-size.
    ///
    /// Handle them specially here, separate from other font-size stuff.
    ///
    /// How this should interact with lang="" and font-family-dependent sizes is
    /// not clear to me. For now just pretend those don't exist here.
    #[cfg(feature = "gecko")]
    fn handle_mathml_scriptlevel_if_needed(&mut self) {
        use crate::values::generics::NonNegative;

        if !self.seen.contains(LonghandId::MathDepth) &&
            !self.seen.contains(LonghandId::MozScriptMinSize) &&
            !self.seen.contains(LonghandId::MozScriptSizeMultiplier)
        {
            return;
        }

        // If the user specifies a font-size, just let it be.
        if self.seen.contains(LonghandId::FontSize) {
            return;
        }

        const SCALE_FACTOR_WHEN_INCREMENTING_MATH_DEPTH_BY_ONE: f32 = 0.71;

        // Helper function that calculates the scale factor applied to font-size
        // when math-depth goes from parent_math_depth to computed_math_depth.
        // This function is essentially a modification of the MathML3's formula
        // 0.71^(parent_math_depth - computed_math_depth) so that a scale factor
        // of parent_script_percent_scale_down is applied when math-depth goes
        // from 0 to 1 and parent_script_script_percent_scale_down is applied
        // when math-depth goes from 0 to 2. This is also a straightforward
        // implementation of the specification's algorithm:
        // https://w3c.github.io/mathml-core/#the-math-script-level-property
        fn scale_factor_for_math_depth_change(
            parent_math_depth: i32,
            computed_math_depth: i32,
            parent_script_percent_scale_down: Option<f32>,
            parent_script_script_percent_scale_down: Option<f32>,
        ) -> f32 {
            let mut a = parent_math_depth;
            let mut b = computed_math_depth;
            let c = SCALE_FACTOR_WHEN_INCREMENTING_MATH_DEPTH_BY_ONE;
            let scale_between_0_and_1 = parent_script_percent_scale_down.unwrap_or_else(|| c);
            let scale_between_0_and_2 =
                parent_script_script_percent_scale_down.unwrap_or_else(|| c * c);
            let mut s = 1.0;
            let mut invert_scale_factor = false;
            if a == b {
                return s;
            }
            if b < a {
                mem::swap(&mut a, &mut b);
                invert_scale_factor = true;
            }
            let mut e = b - a;
            if a <= 0 && b >= 2 {
                s *= scale_between_0_and_2;
                e -= 2;
            } else if a == 1 {
                s *= scale_between_0_and_2 / scale_between_0_and_1;
                e -= 1;
            } else if b == 1 {
                s *= scale_between_0_and_1;
                e -= 1;
            }
            s *= (c as f32).powi(e);
            if invert_scale_factor {
                1.0 / s.max(f32::MIN_POSITIVE)
            } else {
                s
            }
        }

        let (new_size, new_unconstrained_size) = {
            let builder = &self.context.builder;
            let font = builder.get_font();
            let parent_font = builder.get_parent_font();

            let delta = font.mMathDepth.saturating_sub(parent_font.mMathDepth);

            if delta == 0 {
                return;
            }

            let mut min = parent_font.mScriptMinSize;
            if font.mXTextScale.text_zoom_enabled() {
                min = builder.device.zoom_text(min);
            }

            // If the scriptsizemultiplier has been set to something other than
            // the default scale, use MathML3's implementation for backward
            // compatibility. Otherwise, follow MathML Core's algorithm.
            let scale = if parent_font.mScriptSizeMultiplier !=
                SCALE_FACTOR_WHEN_INCREMENTING_MATH_DEPTH_BY_ONE
            {
                (parent_font.mScriptSizeMultiplier as f32).powi(delta as i32)
            } else {
                // Script scale factors are independent of orientation.
                let font_metrics = self.context.query_font_metrics(
                    FontBaseSize::InheritedStyle,
                    FontMetricsOrientation::Horizontal,
                    /* retrieve_math_scales = */ true,
                );
                scale_factor_for_math_depth_change(
                    parent_font.mMathDepth as i32,
                    font.mMathDepth as i32,
                    font_metrics.script_percent_scale_down,
                    font_metrics.script_script_percent_scale_down,
                )
            };

            let parent_size = parent_font.mSize.0;
            let parent_unconstrained_size = parent_font.mScriptUnconstrainedSize.0;
            let new_size = parent_size.scale_by(scale);
            let new_unconstrained_size = parent_unconstrained_size.scale_by(scale);

            if scale <= 1. {
                // The parent size can be smaller than scriptminsize, e.g. if it
                // was specified explicitly. Don't scale in this case, but we
                // don't want to set it to scriptminsize either since that will
                // make it larger.
                if parent_size <= min {
                    (parent_size, new_unconstrained_size)
                } else {
                    (min.max(new_size), new_unconstrained_size)
                }
            } else {
                // If the new unconstrained size is larger than the min size,
                // this means we have escaped the grasp of scriptminsize and can
                // revert to using the unconstrained size.
                // However, if the new size is even larger (perhaps due to usage
                // of em units), use that instead.
                (
                    new_size.min(new_unconstrained_size.max(min)),
                    new_unconstrained_size,
                )
            }
        };
        let font = self.context.builder.mutate_font();
        font.mFont.size = NonNegative(new_size);
        font.mSize = NonNegative(new_size);
        font.mScriptUnconstrainedSize = NonNegative(new_unconstrained_size);
    }

    /// Various properties affect how font-size and font-family are computed.
    ///
    /// These need to be handled here, since relative lengths and ex / ch units
    /// for late properties depend on these.
    fn fixup_font_stuff(&mut self) {
        #[cfg(feature = "gecko")]
        {
            self.unzoom_fonts_if_needed();
            self.recompute_initial_font_family_if_needed();
            self.prioritize_user_fonts_if_needed();
            self.recompute_keyword_font_size_if_needed();
            self.handle_mathml_scriptlevel_if_needed();
            self.constrain_font_size_if_needed()
        }
    }
}
