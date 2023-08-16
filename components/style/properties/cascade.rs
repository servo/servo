/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The main cascading algorithm of the style system.

use crate::applicable_declarations::CascadePriority;
use crate::context::QuirksMode;
use crate::custom_properties::CustomPropertiesBuilder;
use crate::dom::TElement;
use crate::logical_geometry::WritingMode;
use crate::media_queries::Device;
use crate::properties::{
    CSSWideKeyword, ComputedValueFlags, ComputedValues, DeclarationImportanceIterator, Importance,
    LonghandId, LonghandIdSet, PropertyDeclaration, PropertyDeclarationId, PropertyFlags,
    ShorthandsWithPropertyReferencesCache, StyleBuilder, CASCADE_PROPERTY,
};
use crate::rule_cache::{RuleCache, RuleCacheConditions};
use crate::rule_tree::{StrongRuleNode, CascadeLevel};
use crate::selector_parser::PseudoElement;
use crate::shared_lock::StylesheetGuards;
use crate::style_adjuster::StyleAdjuster;
use crate::stylesheets::{Origin, layer_rule::LayerOrder};
use crate::values::{computed, specified};
use fxhash::FxHashMap;
use servo_arc::Arc;
use smallvec::SmallVec;
use std::borrow::Cow;
use std::cell::RefCell;

/// We split the cascade in two phases: 'early' properties, and 'late'
/// properties.
///
/// Early properties are the ones that don't have dependencies _and_ other
/// properties depend on, for example, writing-mode related properties, color
/// (for currentColor), or font-size (for em, etc).
///
/// Late properties are all the others.
trait CascadePhase {
    fn is_early() -> bool;
}

struct EarlyProperties;
impl CascadePhase for EarlyProperties {
    fn is_early() -> bool {
        true
    }
}

struct LateProperties;
impl CascadePhase for LateProperties {
    fn is_early() -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ApplyResetProperties {
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
    parent_style: Option<&ComputedValues>,
    parent_style_ignoring_first_line: Option<&ComputedValues>,
    layout_parent_style: Option<&ComputedValues>,
    visited_rules: Option<&StrongRuleNode>,
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
        parent_style,
        parent_style_ignoring_first_line,
        layout_parent_style,
        CascadeMode::Unvisited { visited_rules },
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
    parent_style: Option<&ComputedValues>,
    parent_style_ignoring_first_line: Option<&ComputedValues>,
    layout_parent_style: Option<&ComputedValues>,
    cascade_mode: CascadeMode,
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
        parent_style,
        parent_style_ignoring_first_line,
        layout_parent_style,
        cascade_mode,
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
    parent_style: Option<&ComputedValues>,
    parent_style_ignoring_first_line: Option<&ComputedValues>,
    layout_parent_style: Option<&ComputedValues>,
    cascade_mode: CascadeMode,
    quirks_mode: QuirksMode,
    rule_cache: Option<&RuleCache>,
    rule_cache_conditions: &mut RuleCacheConditions,
    element: Option<E>,
) -> Arc<ComputedValues>
where
    E: TElement,
    I: Iterator<Item = (&'a PropertyDeclaration, CascadePriority)>,
{
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
    let custom_properties = {
        let mut builder = CustomPropertiesBuilder::new(inherited_style.custom_properties(), device);

        for (declaration, priority) in iter {
            declarations.push((declaration, priority));
            if let PropertyDeclaration::Custom(ref declaration) = *declaration {
                builder.cascade(declaration, priority);
            }
        }

        builder.build()
    };

    let is_root_element = pseudo.is_none() && element.map_or(false, |e| e.is_root());

    let mut context = computed::Context {
        // We'd really like to own the rules here to avoid refcount traffic, but
        // animation's usage of `apply_declarations` make this tricky. See bug
        // 1375525.
        builder: StyleBuilder::new(
            device,
            parent_style,
            parent_style_ignoring_first_line,
            pseudo,
            Some(rules.clone()),
            custom_properties,
            is_root_element,
        ),
        cached_system_font: None,
        in_media_query: false,
        for_smil_animation: false,
        for_non_inherited_property: None,
        quirks_mode,
        rule_cache_conditions: RefCell::new(rule_cache_conditions),
    };

    let using_cached_reset_properties = {
        let mut cascade = Cascade::new(&mut context, cascade_mode);
        let mut shorthand_cache = ShorthandsWithPropertyReferencesCache::default();

        cascade.apply_properties::<EarlyProperties, _>(
            ApplyResetProperties::Yes,
            declarations.iter().cloned(),
            &mut shorthand_cache,
        );

        cascade.compute_visited_style_if_needed(
            element,
            parent_style,
            parent_style_ignoring_first_line,
            layout_parent_style,
            guards,
        );

        let using_cached_reset_properties =
            cascade.try_to_use_cached_reset_properties(rule_cache, guards);

        let apply_reset = if using_cached_reset_properties {
            ApplyResetProperties::No
        } else {
            ApplyResetProperties::Yes
        };

        cascade.apply_properties::<LateProperties, _>(
            apply_reset,
            declarations.iter().cloned(),
            &mut shorthand_cache,
        );

        using_cached_reset_properties
    };

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
    use crate::values::specified::Color;
    use crate::values::computed::ToComputedValue;
    use cssparser::RGBA;

    if !longhand_id.ignored_when_document_colors_disabled() {
        return;
    }

    let is_ua_or_user_rule = matches!(origin, Origin::User | Origin::UserAgent);
    if is_ua_or_user_rule {
        return;
    }

    // Don't override background-color on ::-moz-color-swatch. It is set as an
    // author style (via the style attribute), but it's pretty important for it
    // to show up for obvious reasons :)
    if context.builder.pseudo.map_or(false, |p| p.is_color_swatch()) &&
        longhand_id == LonghandId::BackgroundColor
    {
        return;
    }

    fn alpha_channel(color: &Color, context: &computed::Context) -> u8 {
        // We assume here currentColor is opaque.
        let color = color.to_computed_value(context).to_rgba(RGBA::new(0, 0, 0, 255));
        color.alpha
    }

    // A few special-cases ahead.
    match **declaration {
        PropertyDeclaration::BackgroundColor(ref color) => {
            // We honor system colors.
            if color.is_system() {
                return;
            }
            // For background-color, we revert or initial-with-preserved-alpha
            // otherwise, this is needed to preserve semi-transparent
            // backgrounds.
            //
            // NOTE(emilio): We revert even for alpha == 0. Not doing so would
            // be a bit special casey, even though it causes issues like
            // bug 1625036. The reasoning is that the conditions that trigger
            // that (having mismatched widget and default backgrounds) are both
            // uncommon, and broken in other applications as well, and not
            // honoring transparent makes stuff uglier or break unconditionally
            // (bug 1666059, bug 1755713).
            let alpha = alpha_channel(color, context);
            let mut color = context.builder.device.default_background_color();
            color.alpha = alpha;
            declarations_to_apply_unless_overriden
                .push(PropertyDeclaration::BackgroundColor(color.into()))
        },
        PropertyDeclaration::Color(ref color) => {
            // We honor color: transparent and system colors.
            if color.0.is_system() {
                return;
            }
            if alpha_channel(&color.0, context) == 0 {
                return;
            }
            // If the inherited color would be transparent, but we would
            // override this with a non-transparent color, then override it with
            // the default color. Otherwise just let it inherit through.
            if context.builder.get_parent_inherited_text().clone_color().alpha == 0 {
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
                if bkg.0.iter().all(|image| matches!(*image, Image::Url(..))) {
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
                if color.is_system() {
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
    seen: LonghandIdSet,
    author_specified: LonghandIdSet,
    reverted_set: LonghandIdSet,
    reverted: FxHashMap<LonghandId, (CascadePriority, bool)>,
}

impl<'a, 'b: 'a> Cascade<'a, 'b> {
    fn new(context: &'a mut computed::Context<'b>, cascade_mode: CascadeMode<'a>) -> Self {
        Self {
            context,
            cascade_mode,
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
            self.context.builder.custom_properties.as_ref(),
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

    fn apply_properties<'decls, Phase, I>(
        &mut self,
        apply_reset: ApplyResetProperties,
        declarations: I,
        mut shorthand_cache: &mut ShorthandsWithPropertyReferencesCache,
    ) where
        Phase: CascadePhase,
        I: Iterator<Item = (&'decls PropertyDeclaration, CascadePriority)>,
    {
        let apply_reset = apply_reset == ApplyResetProperties::Yes;

        debug_assert!(
            !Phase::is_early() || apply_reset,
            "Should always apply reset properties in the early phase, since we \
             need to know font-size / writing-mode to decide whether to use the \
             cached reset properties"
        );

        let ignore_colors = !self.context.builder.device.use_document_colors();
        let mut declarations_to_apply_unless_overriden = DeclarationsToApplyUnlessOverriden::new();

        for (declaration, priority) in declarations {
            let origin = priority.cascade_level().origin();

            let declaration_id = declaration.id();
            let longhand_id = match declaration_id {
                PropertyDeclarationId::Longhand(id) => id,
                PropertyDeclarationId::Custom(..) => continue,
            };

            let inherited = longhand_id.inherited();
            if !apply_reset && !inherited {
                continue;
            }

            if Phase::is_early() != longhand_id.is_early_property() {
                continue;
            }

            debug_assert!(!Phase::is_early() || !longhand_id.is_logical());
            let physical_longhand_id = if Phase::is_early() {
                longhand_id
            } else {
                longhand_id.to_physical(self.context.builder.writing_mode)
            };

            if self.seen.contains(physical_longhand_id) {
                continue;
            }

            if self.reverted_set.contains(physical_longhand_id) {
                if let Some(&(reverted_priority, is_origin_revert)) = self.reverted.get(&physical_longhand_id) {
                    if !reverted_priority.allows_when_reverted(&priority, is_origin_revert) {
                        continue;
                    }
                }
            }

            // Only a few properties are allowed to depend on the visited state
            // of links.  When cascading visited styles, we can save time by
            // only processing these properties.
            if matches!(self.cascade_mode, CascadeMode::Visited { .. }) &&
                !physical_longhand_id.is_visited_dependent()
            {
                continue;
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
                    CSSWideKeyword::RevertLayer |
                    CSSWideKeyword::Revert => {
                        let origin_revert = keyword == CSSWideKeyword::Revert;
                        // We intentionally don't want to insert it into
                        // `self.seen`, `reverted` takes care of rejecting other
                        // declarations as needed.
                        self.reverted_set.insert(physical_longhand_id);
                        self.reverted.insert(physical_longhand_id, (priority, origin_revert));
                        continue;
                    },
                    CSSWideKeyword::Unset => true,
                    CSSWideKeyword::Inherit => inherited,
                    CSSWideKeyword::Initial => !inherited,
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

        if Phase::is_early() {
            self.fixup_font_stuff();
            self.compute_writing_mode();
        } else {
            self.finished_applying_properties();
        }
    }

    fn compute_writing_mode(&mut self) {
        let writing_mode = match self.cascade_mode {
            CascadeMode::Unvisited { .. } => {
                WritingMode::new(self.context.builder.get_inherited_box())
            },
            CascadeMode::Visited { writing_mode } => writing_mode,
        };
        self.context.builder.writing_mode = writing_mode;
    }

    fn compute_visited_style_if_needed<E>(
        &mut self,
        element: Option<E>,
        parent_style: Option<&ComputedValues>,
        parent_style_ignoring_first_line: Option<&ComputedValues>,
        layout_parent_style: Option<&ComputedValues>,
        guards: &StylesheetGuards,
    ) where
        E: TElement,
    {
        let visited_rules = match self.cascade_mode {
            CascadeMode::Unvisited { visited_rules } => visited_rules,
            CascadeMode::Visited { .. } => return,
        };

        let visited_rules = match visited_rules {
            Some(rules) => rules,
            None => return,
        };

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
            visited_parent!(parent_style),
            visited_parent!(parent_style_ignoring_first_line),
            visited_parent!(layout_parent_style),
            CascadeMode::Visited { writing_mode },
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

        if self
            .author_specified
            .contains(LonghandId::FontFamily)
        {
            builder.add_flags(ComputedValueFlags::HAS_AUTHOR_SPECIFIED_FONT_FAMILY);
        }

        if self
            .author_specified
            .contains(LonghandId::LetterSpacing)
        {
            builder.add_flags(ComputedValueFlags::HAS_AUTHOR_SPECIFIED_LETTER_SPACING);
        }

        if self
            .author_specified
            .contains(LonghandId::WordSpacing)
        {
            builder.add_flags(ComputedValueFlags::HAS_AUTHOR_SPECIFIED_WORD_SPACING);
        }

        #[cfg(feature = "gecko")]
        if self
            .author_specified
            .contains(LonghandId::FontSynthesis)
        {
            builder.add_flags(ComputedValueFlags::HAS_AUTHOR_SPECIFIED_FONT_SYNTHESIS);
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
            let font = builder.get_font().gecko();

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
            debug_assert!(initial_generic.is_some(), "Initial font should be just one generic font");
            if initial_generic == Some(default_font_type) {
                return;
            }

            default_font_type
        };

        let font = builder.mutate_font().gecko_mut();
        // NOTE: Leaves is_initial untouched.
        font.mFont.family.families = FontFamily::generic(default_font_type).families.clone();
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
            let font = builder.get_font().gecko();

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

        let font = builder.mutate_font().gecko_mut();
        font.mFont.family.families.prioritize_first_generic_or_prepend(default_font_type);
    }

    /// Some keyword sizes depend on the font family and language.
    #[cfg(feature = "gecko")]
    fn recompute_keyword_font_size_if_needed(&mut self) {
        use crate::values::computed::ToComputedValue;
        use crate::values::specified;

        if !self.seen.contains(LonghandId::XLang) && !self.seen.contains(LonghandId::FontFamily) {
            return;
        }

        let new_size = {
            let font = self.context.builder.get_font();
            let info = font.clone_font_size().keyword_info;
            let new_size = match info.kw {
                specified::FontSizeKeyword::None => return,
                _ => {
                    self.context.for_non_inherited_property = None;
                    specified::FontSize::Keyword(info).to_computed_value(self.context)
                },
            };

            if font.gecko().mScriptUnconstrainedSize == new_size.size {
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
            let font = builder.get_font().gecko();
            let min_font_size = unsafe {
                bindings::Gecko_nsStyleFont_ComputeMinSize(font, builder.device.document())
            };

            if font.mFont.size.0 >= min_font_size {
                return;
            }

            NonNegative(min_font_size)
        };

        builder.mutate_font().gecko_mut().mFont.size = min_font_size;
    }

    /// <svg:text> is not affected by text zoom, and it uses a preshint
    /// to disable it. We fix up the struct when this happens by
    /// unzooming its contained font values, which will have been zoomed
    /// in the parent.
    ///
    /// FIXME(emilio): Also, why doing this _before_ handling font-size? That
    /// sounds wrong.
    #[cfg(feature = "gecko")]
    fn unzoom_fonts_if_needed(&mut self) {
        if !self.seen.contains(LonghandId::XTextZoom) {
            return;
        }

        let builder = &mut self.context.builder;

        let parent_zoom = builder.get_parent_font().gecko().mAllowZoomAndMinSize;
        let zoom = builder.get_font().gecko().mAllowZoomAndMinSize;
        if zoom == parent_zoom {
            return;
        }
        debug_assert!(
            !zoom,
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

        let builder = &mut self.context.builder;
        let (new_size, new_unconstrained_size) = {
            let font = builder.get_font().gecko();
            let parent_font = builder.get_parent_font().gecko();

            let delta = font.mMathDepth.saturating_sub(parent_font.mMathDepth);

            if delta == 0 {
                return;
            }

            let mut min = parent_font.mScriptMinSize;
            if font.mAllowZoomAndMinSize {
                min = builder.device.zoom_text(min);
            }

            let scale = (parent_font.mScriptSizeMultiplier as f32).powi(delta as i32);
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
        let font = builder.mutate_font().gecko_mut();
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
