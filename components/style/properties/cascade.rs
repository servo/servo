/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The main cascading algorithm of the style system.

use crate::context::QuirksMode;
use crate::custom_properties::CustomPropertiesBuilder;
use crate::dom::TElement;
use crate::font_metrics::FontMetricsProvider;
use crate::logical_geometry::WritingMode;
use crate::media_queries::Device;
use crate::properties::{ComputedValues, StyleBuilder};
use crate::properties::{LonghandId, LonghandIdSet, CSSWideKeyword};
use crate::properties::{PropertyDeclaration, PropertyDeclarationId, DeclarationImportanceIterator};
use crate::properties::CASCADE_PROPERTY;
use crate::rule_cache::{RuleCache, RuleCacheConditions};
use crate::rule_tree::{CascadeLevel, StrongRuleNode};
use crate::selector_parser::PseudoElement;
use crate::stylesheets::{Origin, PerOrigin};
use servo_arc::Arc;
use crate::shared_lock::StylesheetGuards;
use smallbitvec::SmallBitVec;
use smallvec::SmallVec;
use std::borrow::Cow;
use std::cell::RefCell;
use crate::style_adjuster::StyleAdjuster;
use crate::values::computed;

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
    font_metrics_provider: &FontMetricsProvider,
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
        font_metrics_provider,
        CascadeMode::Unvisited { visited_rules },
        quirks_mode,
        rule_cache,
        rule_cache_conditions,
        element,
    )
}

fn cascade_rules<E>(
    device: &Device,
    pseudo: Option<&PseudoElement>,
    rule_node: &StrongRuleNode,
    guards: &StylesheetGuards,
    parent_style: Option<&ComputedValues>,
    parent_style_ignoring_first_line: Option<&ComputedValues>,
    layout_parent_style: Option<&ComputedValues>,
    font_metrics_provider: &FontMetricsProvider,
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
    let empty = SmallBitVec::new();
    let restriction = pseudo.and_then(|p| p.property_restriction());
    let iter_declarations = || {
        rule_node.self_and_ancestors().flat_map(|node| {
            let cascade_level = node.cascade_level();
            let node_importance = node.importance();
            let declarations = match node.style_source() {
                Some(source) => source
                    .read(cascade_level.guard(guards))
                    .declaration_importance_iter(),
                None => DeclarationImportanceIterator::new(&[], &empty),
            };

            declarations
                // Yield declarations later in source order (with more precedence) first.
                .rev()
                .filter_map(move |(declaration, declaration_importance)| {
                    if let Some(restriction) = restriction {
                        // declaration.id() is either a longhand or a custom
                        // property.  Custom properties are always allowed, but
                        // longhands are only allowed if they have our
                        // restriction flag set.
                        if let PropertyDeclarationId::Longhand(id) = declaration.id() {
                            if !id.flags().contains(restriction) {
                                return None;
                            }
                        }
                    }

                    if declaration_importance == node_importance {
                        Some((declaration, cascade_level))
                    } else {
                        None
                    }
                })
        })
    };

    apply_declarations(
        device,
        pseudo,
        rule_node,
        guards,
        iter_declarations,
        parent_style,
        parent_style_ignoring_first_line,
        layout_parent_style,
        font_metrics_provider,
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
pub fn apply_declarations<'a, E, F, I>(
    device: &Device,
    pseudo: Option<&PseudoElement>,
    rules: &StrongRuleNode,
    guards: &StylesheetGuards,
    iter_declarations: F,
    parent_style: Option<&ComputedValues>,
    parent_style_ignoring_first_line: Option<&ComputedValues>,
    layout_parent_style: Option<&ComputedValues>,
    font_metrics_provider: &FontMetricsProvider,
    cascade_mode: CascadeMode,
    quirks_mode: QuirksMode,
    rule_cache: Option<&RuleCache>,
    rule_cache_conditions: &mut RuleCacheConditions,
    element: Option<E>,
) -> Arc<ComputedValues>
where
    E: TElement,
    F: Fn() -> I,
    I: Iterator<Item = (&'a PropertyDeclaration, CascadeLevel)>,
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

    let mut declarations = SmallVec::<[(&_, CascadeLevel); 32]>::new();
    let custom_properties = {
        let mut builder = CustomPropertiesBuilder::new(
            inherited_style.custom_properties(),
            device.environment(),
        );

        for (declaration, cascade_level) in iter_declarations() {
            declarations.push((declaration, cascade_level));
            if let PropertyDeclaration::Custom(ref declaration) = *declaration {
                builder.cascade(declaration, cascade_level.origin());
            }
        }

        builder.build()
    };

    let mut context = computed::Context {
        is_root_element: pseudo.is_none() && element.map_or(false, |e| e.is_root()),
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
        ),
        cached_system_font: None,
        in_media_query: false,
        for_smil_animation: false,
        for_non_inherited_property: None,
        font_metrics_provider,
        quirks_mode,
        rule_cache_conditions: RefCell::new(rule_cache_conditions),
    };

    let using_cached_reset_properties = {
        let mut cascade = Cascade::new(&mut context, cascade_mode);

        cascade
            .apply_properties::<EarlyProperties, _>(ApplyResetProperties::Yes, declarations.iter().cloned());

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

        cascade.apply_properties::<LateProperties, _>(apply_reset, declarations.iter().cloned());

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

fn should_ignore_declaration_when_ignoring_document_colors(
    device: &Device,
    longhand_id: LonghandId,
    cascade_level: CascadeLevel,
    pseudo: Option<&PseudoElement>,
    declaration: &mut Cow<PropertyDeclaration>,
) -> bool {
    if !longhand_id.ignored_when_document_colors_disabled() {
        return false;
    }

    let is_ua_or_user_rule =
        matches!(cascade_level.origin(), Origin::User | Origin::UserAgent);
    if is_ua_or_user_rule {
        return false;
    }

    let is_style_attribute = matches!(
        cascade_level,
        CascadeLevel::StyleAttributeNormal | CascadeLevel::StyleAttributeImportant
    );

    // Don't override colors on pseudo-element's style attributes. The
    // background-color on ::-moz-color-swatch is an example. Those are set
    // as an author style (via the style attribute), but it's pretty
    // important for it to show up for obvious reasons :)
    if pseudo.is_some() && is_style_attribute {
        return false;
    }

    // Treat background-color a bit differently.  If the specified color is
    // anything other than a fully transparent color, convert it into the
    // Device's default background color.
    {
        let background_color = match **declaration {
            PropertyDeclaration::BackgroundColor(ref color) => color,
            _ => return true,
        };

        if background_color.is_transparent() {
            return false;
        }
    }

    let color = device.default_background_color();
    *declaration.to_mut() = PropertyDeclaration::BackgroundColor(color.into());

    false
}

struct Cascade<'a, 'b: 'a> {
    context: &'a mut computed::Context<'b>,
    cascade_mode: CascadeMode<'a>,
    seen: LonghandIdSet,
    reverted: PerOrigin<LonghandIdSet>,
}

impl<'a, 'b: 'a> Cascade<'a, 'b> {
    fn new(context: &'a mut computed::Context<'b>, cascade_mode: CascadeMode<'a>) -> Self {
        Self {
            context,
            cascade_mode,
            seen: LonghandIdSet::default(),
            reverted: Default::default(),
        }
    }

    fn substitute_variables_if_needed<'decl>(
        &mut self,
        declaration: &'decl PropertyDeclaration,
    ) -> Cow<'decl, PropertyDeclaration> {
        let declaration = match *declaration {
            PropertyDeclaration::WithVariables(ref declaration) => declaration,
            ref d => return Cow::Borrowed(d),
        };

        if !declaration.id.inherited() {
            self.context
                .rule_cache_conditions
                .borrow_mut()
                .set_uncacheable();
        }

        Cow::Owned(declaration.value.substitute_variables(
            declaration.id,
            self.context.builder.custom_properties.as_ref(),
            self.context.quirks_mode,
            self.context.device().environment(),
        ))
    }

    #[inline(always)]
    fn apply_declaration<Phase: CascadePhase>(
        &mut self,
        longhand_id: LonghandId,
        declaration: &PropertyDeclaration,
    ) {
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
    ) where
        Phase: CascadePhase,
        I: Iterator<Item = (&'decls PropertyDeclaration, CascadeLevel)>,
    {
        let apply_reset = apply_reset == ApplyResetProperties::Yes;

        debug_assert!(
            !Phase::is_early() || apply_reset,
            "Should always apply reset properties in the early phase, since we \
             need to know font-size / writing-mode to decide whether to use the \
             cached reset properties"
        );

        let ignore_colors = !self.context.builder.device.use_document_colors();

        for (declaration, cascade_level) in declarations {
            let declaration_id = declaration.id();
            let origin = cascade_level.origin();
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

            if self.reverted.borrow_for_origin(&origin).contains(physical_longhand_id) {
                continue;
            }

            // Only a few properties are allowed to depend on the visited state
            // of links.  When cascading visited styles, we can save time by
            // only processing these properties.
            if matches!(self.cascade_mode, CascadeMode::Visited { .. }) &&
                !physical_longhand_id.is_visited_dependent()
            {
                continue;
            }

            let mut declaration = self.substitute_variables_if_needed(declaration);

            // When document colors are disabled, skip properties that are
            // marked as ignored in that mode, unless they come from a UA or
            // user style sheet.
            if ignore_colors {
                let should_ignore = should_ignore_declaration_when_ignoring_document_colors(
                    self.context.builder.device,
                    longhand_id,
                    cascade_level,
                    self.context.builder.pseudo,
                    &mut declaration,
                );
                if should_ignore {
                    continue;
                }
            }

            let css_wide_keyword = declaration.get_css_wide_keyword();
            if let Some(CSSWideKeyword::Revert) = css_wide_keyword {
                // We intentionally don't want to insert it into `self.seen`,
                // `reverted` takes care of rejecting other declarations as
                // needed.
                for origin in origin.following_including() {
                    self.reverted
                        .borrow_mut_for_origin(&origin)
                        .insert(physical_longhand_id);
                }
                continue;
            }

            self.seen.insert(physical_longhand_id);

            let unset = css_wide_keyword.map_or(false, |css_wide_keyword| {
                match css_wide_keyword {
                    CSSWideKeyword::Unset => true,
                    CSSWideKeyword::Inherit => inherited,
                    CSSWideKeyword::Initial => !inherited,
                    CSSWideKeyword::Revert => unreachable!(),
                }
            });

            if unset {
                continue;
            }

            // FIXME(emilio): We should avoid generating code for logical
            // longhands and just use the physical ones, then rename
            // physical_longhand_id to just longhand_id.
            self.apply_declaration::<Phase>(longhand_id, &*declaration);
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
            self.context.font_metrics_provider,
            CascadeMode::Visited { writing_mode },
            self.context.quirks_mode,
            // The rule cache doesn't care about caching :visited
            // styles, we cache the unvisited style instead. We still do
            // need to set the caching dependencies properly if present
            // though, so the cache conditions need to match.
            /* rule_cache = */ None,
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

        #[cfg(feature = "servo")]
        {
            // TODO(emilio): Use get_font_if_mutated instead.
            if self.seen.contains(LonghandId::FontStyle) ||
                self.seen.contains(LonghandId::FontWeight) ||
                self.seen.contains(LonghandId::FontStretch) ||
                self.seen.contains(LonghandId::FontFamily)
            {
                builder.mutate_font().compute_font_hash();
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

        let cached_style = match cache.find(guards, &self.context.builder) {
            Some(style) => style,
            None => return false,
        };

        self.context.builder.copy_reset_from(cached_style);
        true
    }

    /// The default font type (which is stored in FontFamilyList's
    /// `mDefaultFontType`) depends on the current lang group and generic font
    /// family, so we may need to recompute it if or the family changed.
    ///
    /// Also, we prioritize non-document fonts here if we need to (see the pref
    /// `browser.display.use_document_fonts`).
    #[inline]
    #[cfg(feature = "gecko")]
    fn recompute_default_font_family_type_if_needed(&mut self) {
        use crate::gecko_bindings::{bindings, structs};
        use crate::values::computed::font::GenericFontFamily;

        if !self.seen.contains(LonghandId::XLang) &&
           !self.seen.contains(LonghandId::FontFamily) {
            return;
        }

        let use_document_fonts = unsafe { structs::StaticPrefs_sVarCache_browser_display_use_document_fonts != 0 };
        let builder = &mut self.context.builder;
        let (default_font_type, prioritize_user_fonts) = {
            let font = builder.get_font().gecko();

            // System fonts are all right, and should have the default font type
            // set to none already, so bail out early.
            if font.mFont.systemFont {
                debug_assert_eq!(font.mFont.fontlist.mDefaultFontType, GenericFontFamily::None);
                return;
            }

            let default_font_type = unsafe {
                bindings::Gecko_nsStyleFont_ComputeDefaultFontType(
                    builder.device.document(),
                    font.mGenericID,
                    font.mLanguage.mRawPtr,
                )
            };

            // We prioritize user fonts over document fonts if the pref is set,
            // and we don't have a generic family already (or we're using
            // cursive or fantasy, since they're ignored, see bug 789788), and
            // we have a generic family to actually replace it with.
            let prioritize_user_fonts =
                !use_document_fonts &&
                matches!(
                    font.mGenericID,
                    GenericFontFamily::None |
                    GenericFontFamily::Fantasy |
                    GenericFontFamily::Cursive
                ) &&
                default_font_type != GenericFontFamily::None;

            if !prioritize_user_fonts && default_font_type == font.mFont.fontlist.mDefaultFontType {
                // Nothing to do.
                return;
            }
            (default_font_type, prioritize_user_fonts)
        };

        let font = builder.mutate_font().gecko_mut();
        font.mFont.fontlist.mDefaultFontType = default_font_type;
        if prioritize_user_fonts {
            unsafe {
                bindings::Gecko_nsStyleFont_PrioritizeUserFonts(font, default_font_type)
            }
        }
    }

    /// Some keyword sizes depend on the font family and language.
    #[cfg(feature = "gecko")]
    fn recompute_keyword_font_size_if_needed(&mut self) {
        use crate::values::computed::ToComputedValue;
        use crate::values::specified;

        if !self.seen.contains(LonghandId::XLang) &&
           !self.seen.contains(LonghandId::FontFamily) {
            return;
        }

        let new_size = {
            let font = self.context.builder.get_font();
            let new_size = match font.clone_font_size().keyword_info {
                Some(info) => {
                    self.context.for_non_inherited_property = None;
                    specified::FontSize::Keyword(info).to_computed_value(self.context)
                }
                None => return,
            };

            if font.gecko().mScriptUnconstrainedSize == new_size.size().0 {
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

        if !self.seen.contains(LonghandId::XLang) &&
           !self.seen.contains(LonghandId::FontFamily) &&
           !self.seen.contains(LonghandId::MozMinFontSizeRatio) &&
           !self.seen.contains(LonghandId::FontSize) {
            return;
        }

        let builder = &mut self.context.builder;
        let min_font_size = {
            let font = builder.get_font().gecko();
            let min_font_size = unsafe {
                bindings::Gecko_nsStyleFont_ComputeMinSize(
                    font,
                    builder.device.document(),
                )
            };

            if font.mFont.size >= min_font_size {
                return;
            }

            min_font_size
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

        let parent_zoom = builder.get_parent_font().gecko().mAllowZoom;
        let zoom = builder.get_font().gecko().mAllowZoom;
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
        use app_units::Au;
        use std::cmp;

        if !self.seen.contains(LonghandId::MozScriptLevel) &&
           !self.seen.contains(LonghandId::MozScriptMinSize) &&
           !self.seen.contains(LonghandId::MozScriptSizeMultiplier) {
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

            let delta =
                font.mScriptLevel.saturating_sub(parent_font.mScriptLevel);

            if delta == 0 {
                return;
            }

            let mut min = Au(parent_font.mScriptMinSize);
            if font.mAllowZoom {
                min = builder.device.zoom_text(min);
            }

            let scale = (parent_font.mScriptSizeMultiplier as f32).powi(delta as i32);
            let parent_size = Au(parent_font.mSize);
            let parent_unconstrained_size = Au(parent_font.mScriptUnconstrainedSize);
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
                    (cmp::max(min, new_size), new_unconstrained_size)
                }
            } else {
                // If the new unconstrained size is larger than the min size,
                // this means we have escaped the grasp of scriptminsize and can
                // revert to using the unconstrained size.
                // However, if the new size is even larger (perhaps due to usage
                // of em units), use that instead.
                (
                    cmp::min(new_size, cmp::max(new_unconstrained_size, min)),
                    new_unconstrained_size
                )
            }
        };
        let font = builder.mutate_font().gecko_mut();
        font.mFont.size = new_size.0;
        font.mSize = new_size.0;
        font.mScriptUnconstrainedSize = new_unconstrained_size.0;
    }

    /// Various properties affect how font-size and font-family are computed.
    ///
    /// These need to be handled here, since relative lengths and ex / ch units
    /// for late properties depend on these.
    fn fixup_font_stuff(&mut self) {
        #[cfg(feature = "gecko")]
        {
            self.unzoom_fonts_if_needed();
            self.recompute_default_font_family_type_if_needed();
            self.recompute_keyword_font_size_if_needed();
            self.handle_mathml_scriptlevel_if_needed();
            self.constrain_font_size_if_needed()
        }
    }
}
