/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A struct to encapsulate all the style fixups and flags propagations
//! a computed style needs in order for it to adhere to the CSS spec.

use app_units::Au;
use properties::{self, CascadeFlags, ComputedValues, StyleBuilder};
use properties::longhands::display::computed_value::T as Display;
use properties::longhands::float::computed_value::T as Float;
use properties::longhands::overflow_x::computed_value::T as Overflow;
use properties::longhands::position::computed_value::T as Position;

/// A struct that implements all the adjustment methods.
///
/// NOTE(emilio): If new adjustments are introduced that depend on reset
/// properties of the parent, you may need tweaking the
/// `ChildCascadeRequirement` code in `matching.rs`.
pub struct StyleAdjuster<'a, 'b: 'a> {
    style: &'a mut StyleBuilder<'b>,
}

impl<'a, 'b: 'a> StyleAdjuster<'a, 'b> {
    /// Trivially constructs a new StyleAdjuster.
    #[inline]
    pub fn new(style: &'a mut StyleBuilder<'b>) -> Self {
        StyleAdjuster { style }
    }

    /// <https://fullscreen.spec.whatwg.org/#new-stacking-layer>
    ///
    ///    Any position value other than 'absolute' and 'fixed' are
    ///    computed to 'absolute' if the element is in a top layer.
    ///
    fn adjust_for_top_layer(&mut self) {
        if !self.style.out_of_flow_positioned() && self.style.in_top_layer() {
            self.style.mutate_box().set_position(Position::Absolute);
        }
    }

    /// CSS 2.1 section 9.7:
    ///
    ///    If 'position' has the value 'absolute' or 'fixed', [...] the computed
    ///    value of 'float' is 'none'.
    ///
    fn adjust_for_position(&mut self) {
        if self.style.out_of_flow_positioned() && self.style.floated() {
            self.style.mutate_box().set_float(Float::None);
        }
    }

    /// Apply the blockification rules based on the table in CSS 2.2 section 9.7.
    /// <https://drafts.csswg.org/css2/visuren.html#dis-pos-flo>
    fn blockify_if_necessary(
        &mut self,
        layout_parent_style: &ComputedValues,
        flags: CascadeFlags,
    ) {
        let mut blockify = false;
        macro_rules! blockify_if {
            ($if_what:expr) => {
                if !blockify {
                    blockify = $if_what;
                }
            }
        }

        if !flags.contains(CascadeFlags::SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP) {
            blockify_if!(flags.contains(CascadeFlags::IS_ROOT_ELEMENT));
            blockify_if!(layout_parent_style.get_box().clone_display().is_item_container());
        }

        let is_item_or_root = blockify;

        blockify_if!(self.style.floated());
        blockify_if!(self.style.out_of_flow_positioned());

        if !blockify {
            return;
        }

        let display = self.style.get_box().clone_display();
        let blockified_display =
            display.equivalent_block_display(flags.contains(CascadeFlags::IS_ROOT_ELEMENT));
        if display != blockified_display {
            self.style.mutate_box().set_adjusted_display(
                blockified_display,
                is_item_or_root,
            );
        }
    }

    /// Compute a few common flags for both text and element's style.
    pub fn set_bits(&mut self) {
        use properties::computed_value_flags::ComputedValueFlags;

        if self.style.inherited_flags().contains(ComputedValueFlags::IS_IN_DISPLAY_NONE_SUBTREE) ||
            self.style.get_box().clone_display() == Display::None {
            self.style.flags.insert(ComputedValueFlags::IS_IN_DISPLAY_NONE_SUBTREE);
        }

        if self.style.inherited_flags().contains(ComputedValueFlags::IS_IN_PSEUDO_ELEMENT_SUBTREE) ||
            self.style.is_pseudo_element() {
            self.style.flags.insert(ComputedValueFlags::IS_IN_PSEUDO_ELEMENT_SUBTREE);
        }
    }

    /// Adjust the style for text style.
    ///
    /// The adjustments here are a subset of the adjustments generally, because
    /// text only inherits properties.
    ///
    /// Note that this, for Gecko, comes through Servo_ComputedValues_Inherit.
    #[cfg(feature = "gecko")]
    pub fn adjust_for_text(&mut self) {
        self.adjust_for_text_combine_upright();
        self.adjust_for_text_in_ruby();
        self.set_bits();
    }

    /// Change writing mode of the text frame for text-combine-upright.
    ///
    /// It is safe to look at our own style because we are looking at inherited
    /// properties, and text is just plain inheritance.
    ///
    /// TODO(emilio): we should (Gecko too) revise these adjustments in presence
    /// of display: contents.
    ///
    /// FIXME(emilio): How does this play with logical properties? Doesn't
    /// mutating writing-mode change the potential physical sides chosen?
    #[cfg(feature = "gecko")]
    fn adjust_for_text_combine_upright(&mut self) {
        use computed_values::text_combine_upright::T as TextCombineUpright;
        use computed_values::writing_mode::T as WritingMode;
        use properties::computed_value_flags::ComputedValueFlags;

        let writing_mode =
            self.style.get_inheritedbox().clone_writing_mode();
        let text_combine_upright =
            self.style.get_inheritedtext().clone_text_combine_upright();

        if writing_mode != WritingMode::HorizontalTb &&
           text_combine_upright == TextCombineUpright::All {
            self.style.flags.insert(ComputedValueFlags::IS_TEXT_COMBINED);
            self.style.mutate_inheritedbox().set_writing_mode(WritingMode::HorizontalTb);
        }
    }

    /// Applies the line break suppression flag to text if it is in any ruby
    /// box. This is necessary because its parent may not itself have the flag
    /// set (e.g. ruby or ruby containers), thus we may not inherit the flag
    /// from them.
    #[cfg(feature = "gecko")]
    fn adjust_for_text_in_ruby(&mut self) {
        use properties::computed_value_flags::ComputedValueFlags;
        let parent_display = self.style.get_parent_box().clone_display();
        if parent_display.is_ruby_type() {
            self.style.flags.insert(ComputedValueFlags::SHOULD_SUPPRESS_LINEBREAK);
        }
    }

    /// <https://drafts.csswg.org/css-writing-modes-3/#block-flow:>
    ///
    ///    If a box has a different writing-mode value than its containing
    ///    block:
    ///
    ///        - If the box has a specified display of inline, its display
    ///          computes to inline-block. [CSS21]
    ///
    /// This matches the adjustment that Gecko does, not exactly following
    /// the spec. See also:
    ///
    /// <https://lists.w3.org/Archives/Public/www-style/2017Mar/0045.html>
    /// <https://github.com/servo/servo/issues/15754>
    fn adjust_for_writing_mode(
        &mut self,
        layout_parent_style: &ComputedValues,
    ) {
        let our_writing_mode =
            self.style.get_inheritedbox().clone_writing_mode();
        let parent_writing_mode =
            layout_parent_style.get_inheritedbox().clone_writing_mode();

        if our_writing_mode != parent_writing_mode &&
           self.style.get_box().clone_display() == Display::Inline {
            self.style.mutate_box().set_display(Display::InlineBlock);
        }
    }

    #[cfg(feature = "gecko")]
    fn adjust_for_contain(&mut self) {
        use properties::longhands::contain::SpecifiedValue;

        // An element with contain: paint needs to be a formatting context, and
        // also implies overflow: clip.
        //
        // TODO(emilio): This mimics Gecko, but spec links are missing!
        let contain = self.style.get_box().clone_contain();
        if !contain.contains(SpecifiedValue::PAINT) {
            return;
        }

        if self.style.get_box().clone_display() == Display::Inline {
            self.style.mutate_box().set_adjusted_display(Display::InlineBlock,
                                                         false);
        }


        // When 'contain: paint', update overflow from 'visible' to 'clip'.
        if self.style.get_box().clone_contain().contains(SpecifiedValue::PAINT) {
            if self.style.get_box().clone_overflow_x() == Overflow::Visible {
                let box_style = self.style.mutate_box();
                box_style.set_overflow_x(Overflow::MozHiddenUnscrollable);
                box_style.set_overflow_y(Overflow::MozHiddenUnscrollable);
            }
        }
    }

    /// When mathvariant is not "none", font-weight and font-style are
    /// both forced to "normal".
    #[cfg(feature = "gecko")]
    fn adjust_for_mathvariant(&mut self) {
        use properties::longhands::_moz_math_variant::computed_value::T as MozMathVariant;
        use properties::longhands::font_style::computed_value::T as FontStyle;
        use properties::longhands::font_weight::computed_value::T as FontWeight;
        if self.style.get_font().clone__moz_math_variant() != MozMathVariant::None {
            let font_style = self.style.mutate_font();
            // Sadly we don't have a nice name for the computed value
            // of "font-weight: normal".
            font_style.set_font_weight(FontWeight::normal());
            font_style.set_font_style(FontStyle::Normal);
        }
    }

    /// This implements an out-of-date spec. The new spec moves the handling of
    /// this to layout, which Gecko implements but Servo doesn't.
    ///
    /// See https://github.com/servo/servo/issues/15229
    #[cfg(feature = "servo")]
    fn adjust_for_alignment(&mut self, layout_parent_style: &ComputedValues) {
        use computed_values::align_items::T as AlignItems;
        use computed_values::align_self::T as AlignSelf;

        if self.style.get_position().clone_align_self() == AlignSelf::Auto &&
           !self.style.out_of_flow_positioned() {
            let self_align =
                match layout_parent_style.get_position().clone_align_items() {
                    AlignItems::Stretch => AlignSelf::Stretch,
                    AlignItems::Baseline => AlignSelf::Baseline,
                    AlignItems::FlexStart => AlignSelf::FlexStart,
                    AlignItems::FlexEnd => AlignSelf::FlexEnd,
                    AlignItems::Center => AlignSelf::Center,
                };
            self.style.mutate_position().set_align_self(self_align);
        }
    }

    /// The initial value of border-*-width may be changed at computed value
    /// time.
    ///
    /// This is moved to properties.rs for convenience.
    fn adjust_for_border_width(&mut self) {
        properties::adjust_border_width(self.style);
    }

    /// The initial value of outline-width may be changed at computed value time.
    fn adjust_for_outline(&mut self) {
        if self.style.get_outline().clone_outline_style().none_or_hidden() &&
           self.style.get_outline().outline_has_nonzero_width() {
            self.style.mutate_outline().set_outline_width(Au(0).into());
        }
    }

    /// CSS3 overflow-x and overflow-y require some fixup as well in some
    /// cases.
    ///
    /// overflow: clip and overflow: visible are meaningful only when used in
    /// both dimensions.
    fn adjust_for_overflow(&mut self) {
        let original_overflow_x = self.style.get_box().clone_overflow_x();
        let original_overflow_y = self.style.get_box().clone_overflow_y();

        let mut overflow_x = original_overflow_x;
        let mut overflow_y = original_overflow_y;

        if overflow_x == overflow_y {
            return;
        }

        // If 'visible' is specified but doesn't match the other dimension,
        // it turns into 'auto'.
        if overflow_x == Overflow::Visible {
            overflow_x = Overflow::Auto;
        }

        if overflow_y == Overflow::Visible {
            overflow_y = Overflow::Auto;
        }

        #[cfg(feature = "gecko")]
        {
            // overflow: clip is deprecated, so convert to hidden if it's
            // specified in only one dimension.
            if overflow_x == Overflow::MozHiddenUnscrollable {
                overflow_x = Overflow::Hidden;
            }
            if overflow_y == Overflow::MozHiddenUnscrollable {
                overflow_y = Overflow::Hidden;
            }
        }

        if overflow_x != original_overflow_x ||
           overflow_y != original_overflow_y {
            let box_style = self.style.mutate_box();
            box_style.set_overflow_x(overflow_x);
            box_style.set_overflow_y(overflow_y);
        }
    }

    /// Native anonymous content converts display:contents into display:inline.
    #[cfg(feature = "gecko")]
    fn adjust_for_prohibited_display_contents(&mut self) {
        // TODO: We should probably convert display:contents into display:none
        // in some cases too: https://drafts.csswg.org/css-display/#unbox
        //
        // FIXME(emilio): ::before and ::after should support display: contents,
        // see bug 1418138.
        if self.style.pseudo.is_none() ||
           self.style.get_box().clone_display() != Display::Contents {
            return;
        }

        self.style.mutate_box().set_display(Display::Inline);
    }

    /// If a <fieldset> has grid/flex display type, we need to inherit
    /// this type into its ::-moz-fieldset-content anonymous box.
    ///
    /// NOTE(emilio): We don't need to handle the display change for this case
    /// in matching.rs because anonymous box restyling works separately to the
    /// normal cascading process.
    #[cfg(feature = "gecko")]
    fn adjust_for_fieldset_content(
        &mut self,
        layout_parent_style: &ComputedValues,
    ) {
        match self.style.pseudo {
            Some(ref p) if p.is_fieldset_content() => {},
            _ => return,
        }

        debug_assert_eq!(self.style.get_box().clone_display(), Display::Block);
        // TODO We actually want style from parent rather than layout
        // parent, so that this fixup doesn't happen incorrectly when
        // when <fieldset> has "display: contents".
        let parent_display = layout_parent_style.get_box().clone_display();
        let new_display = match parent_display {
            Display::Flex |
            Display::InlineFlex => Some(Display::Flex),
            Display::Grid |
            Display::InlineGrid => Some(Display::Grid),
            _ => None,
        };
        if let Some(new_display) = new_display {
            self.style.mutate_box().set_display(new_display);
        }
    }

    /// -moz-center, -moz-left and -moz-right are used for HTML's alignment.
    ///
    /// This is covering the <div align="right"><table>...</table></div> case.
    ///
    /// In this case, we don't want to inherit the text alignment into the
    /// table.
    #[cfg(feature = "gecko")]
    fn adjust_for_table_text_align(&mut self) {
        use properties::longhands::text_align::computed_value::T as TextAlign;
        if self.style.get_box().clone_display() != Display::Table {
            return;
        }

        match self.style.get_inheritedtext().clone_text_align() {
            TextAlign::MozLeft |
            TextAlign::MozCenter |
            TextAlign::MozRight => {},
            _ => return,
        }

        self.style.mutate_inheritedtext().set_text_align(TextAlign::Start)
    }

    /// Set the HAS_TEXT_DECORATION_LINES flag based on parent style.
    fn adjust_for_text_decoration_lines(
        &mut self,
        layout_parent_style: &ComputedValues,
    ) {
        use properties::computed_value_flags::ComputedValueFlags;
        if layout_parent_style.flags.contains(ComputedValueFlags::HAS_TEXT_DECORATION_LINES) ||
           !self.style.get_text().clone_text_decoration_line().is_empty() {
            self.style.flags.insert(ComputedValueFlags::HAS_TEXT_DECORATION_LINES);
        }
    }

    #[cfg(feature = "gecko")]
    fn should_suppress_linebreak(
        &self,
        layout_parent_style: &ComputedValues,
    ) -> bool {
        use properties::computed_value_flags::ComputedValueFlags;
        // Line break suppression should only be propagated to in-flow children.
        if self.style.floated() || self.style.out_of_flow_positioned() {
            return false;
        }
        let parent_display = layout_parent_style.get_box().clone_display();
        if layout_parent_style.flags.contains(ComputedValueFlags::SHOULD_SUPPRESS_LINEBREAK) {
            // Line break suppression is propagated to any children of
            // line participants.
            if parent_display.is_line_participant() {
                return true;
            }
        }
        match self.style.get_box().clone_display() {
            // Ruby base and text are always non-breakable.
            Display::RubyBase |
            Display::RubyText => true,
            // Ruby base container and text container are breakable.
            // Note that, when certain HTML tags, e.g. form controls, have ruby
            // level container display type, they could also escape from the
            // line break suppression flag while they shouldn't. However, it is
            // generally fine since they themselves are non-breakable.
            Display::RubyBaseContainer |
            Display::RubyTextContainer => false,
            // Anything else is non-breakable if and only if its layout parent
            // has a ruby display type, because any of the ruby boxes can be
            // anonymous.
            _ => parent_display.is_ruby_type(),
        }
    }

    /// Do ruby-related style adjustments, which include:
    /// * propagate the line break suppression flag,
    /// * inlinify block descendants,
    /// * suppress border and padding for ruby level containers,
    /// * correct unicode-bidi.
    #[cfg(feature = "gecko")]
    fn adjust_for_ruby(
        &mut self,
        layout_parent_style: &ComputedValues,
        flags: CascadeFlags,
    ) {
        use properties::CascadeFlags;
        use properties::computed_value_flags::ComputedValueFlags;
        use properties::longhands::unicode_bidi::computed_value::T as UnicodeBidi;

        let self_display = self.style.get_box().clone_display();
        // Check whether line break should be suppressed for this element.
        if self.should_suppress_linebreak(layout_parent_style) {
            self.style.flags.insert(ComputedValueFlags::SHOULD_SUPPRESS_LINEBREAK);
            // Inlinify the display type if allowed.
            if !flags.contains(CascadeFlags::SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP) {
                let inline_display = self_display.inlinify();
                if self_display != inline_display {
                    self.style.mutate_box().set_display(inline_display);
                }
            }
        }
        // Suppress border and padding for ruby level containers.
        // This is actually not part of the spec. It is currently unspecified
        // how border and padding should be handled for ruby level container,
        // and suppressing them here make it easier for layout to handle.
        if self_display.is_ruby_level_container() {
            self.style.reset_border_struct();
            self.style.reset_padding_struct();
        }

        // Force bidi isolation on all internal ruby boxes and ruby container
        // per spec https://drafts.csswg.org/css-ruby-1/#bidi
        if self_display.is_ruby_type() {
            let new_value = match self.style.get_text().clone_unicode_bidi() {
                UnicodeBidi::Normal |
                UnicodeBidi::Embed => Some(UnicodeBidi::Isolate),
                UnicodeBidi::BidiOverride => Some(UnicodeBidi::IsolateOverride),
                _ => None,
            };
            if let Some(new_value) = new_value {
                self.style.mutate_text().set_unicode_bidi(new_value);
            }
        }
    }

    /// Computes the RELEVANT_LINK_VISITED flag based on the parent style and on
    /// whether we're a relevant link.
    ///
    /// NOTE(emilio): We don't do this for text styles, which is... dubious, but
    /// Gecko doesn't seem to do it either. It's extremely easy to do if needed
    /// though.
    ///
    /// FIXME(emilio): This isn't technically a style adjustment thingie, could
    /// it move somewhere else?
    fn adjust_for_visited(&mut self, flags: CascadeFlags) {
        use properties::CascadeFlags;
        use properties::computed_value_flags::ComputedValueFlags;

        if !self.style.has_visited_style() {
            return;
        }

        let relevant_link_visited = if flags.contains(CascadeFlags::IS_LINK) {
            flags.contains(CascadeFlags::IS_VISITED_LINK)
        } else {
            self.style.inherited_flags().contains(ComputedValueFlags::IS_RELEVANT_LINK_VISITED)
        };

        if relevant_link_visited {
            self.style.flags.insert(ComputedValueFlags::IS_RELEVANT_LINK_VISITED);
        }
    }

    /// Resolves "justify-items: auto" based on the inherited style if needed to
    /// comply with:
    ///
    /// <https://drafts.csswg.org/css-align/#valdef-justify-items-legacy>
    ///
    /// (Note that "auto" is being renamed to "legacy")
    #[cfg(feature = "gecko")]
    fn adjust_for_justify_items(&mut self) {
        use values::specified::align;
        let justify_items = self.style.get_position().clone_justify_items();
        if justify_items.specified.0 != align::AlignFlags::AUTO {
            return;
        }

        let parent_justify_items =
            self.style.get_parent_position().clone_justify_items();

        if !parent_justify_items.computed.0.contains(align::AlignFlags::LEGACY) {
            return;
        }

        if parent_justify_items.computed == justify_items.computed {
            return;
        }

        self.style
            .mutate_position()
            .set_computed_justify_items(parent_justify_items.computed);
    }

    /// Adjusts the style to account for various fixups that don't fit naturally
    /// into the cascade.
    ///
    /// When comparing to Gecko, this is similar to the work done by
    /// `nsStyleContext::ApplyStyleFixups`, plus some parts of
    /// `nsStyleSet::GetContext`.
    pub fn adjust(
        &mut self,
        layout_parent_style: &ComputedValues,
        flags: CascadeFlags,
    ) {
        // Don't adjust visited styles, visited-dependent properties aren't
        // affected by these adjustments and it'd be just wasted work anyway.
        //
        // It also doesn't make much sense to adjust them, since we don't
        // cascade most properties anyway, and they wouldn't be looked up.
        if flags.contains(CascadeFlags::VISITED_DEPENDENT_ONLY) {
            return;
        }

        self.adjust_for_visited(flags);
        #[cfg(feature = "gecko")]
        {
            self.adjust_for_prohibited_display_contents();
            self.adjust_for_fieldset_content(layout_parent_style);
        }
        self.adjust_for_top_layer();
        self.blockify_if_necessary(layout_parent_style, flags);
        self.adjust_for_position();
        self.adjust_for_overflow();
        #[cfg(feature = "gecko")]
        {
            self.adjust_for_table_text_align();
            self.adjust_for_contain();
            self.adjust_for_mathvariant();
            self.adjust_for_justify_items();
        }
        #[cfg(feature = "servo")]
        {
            self.adjust_for_alignment(layout_parent_style);
        }
        self.adjust_for_border_width();
        self.adjust_for_outline();
        self.adjust_for_writing_mode(layout_parent_style);
        self.adjust_for_text_decoration_lines(layout_parent_style);
        #[cfg(feature = "gecko")]
        {
            self.adjust_for_ruby(layout_parent_style, flags);
        }
        self.set_bits();
    }
}
