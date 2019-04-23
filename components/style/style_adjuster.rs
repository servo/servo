/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A struct to encapsulate all the style fixups and flags propagations
//! a computed style needs in order for it to adhere to the CSS spec.

use crate::dom::TElement;
use crate::properties::computed_value_flags::ComputedValueFlags;
use crate::properties::longhands::display::computed_value::T as Display;
use crate::properties::longhands::float::computed_value::T as Float;
use crate::properties::longhands::overflow_x::computed_value::T as Overflow;
use crate::properties::longhands::position::computed_value::T as Position;
use crate::properties::{self, ComputedValues, StyleBuilder};
use app_units::Au;

/// A struct that implements all the adjustment methods.
///
/// NOTE(emilio): If new adjustments are introduced that depend on reset
/// properties of the parent, you may need tweaking the
/// `ChildCascadeRequirement` code in `matching.rs`.
///
/// NOTE(emilio): Also, if new adjustments are introduced that break the
/// following invariant:
///
///   Given same tag name, namespace, rules and parent style, two elements would
///   end up with exactly the same style.
///
/// Then you need to adjust the lookup_by_rules conditions in the sharing cache.
pub struct StyleAdjuster<'a, 'b: 'a> {
    style: &'a mut StyleBuilder<'b>,
}

#[cfg(feature = "gecko")]
fn is_topmost_svg_svg_element<E>(e: E) -> bool
where
    E: TElement,
{
    debug_assert!(e.is_svg_element());
    if e.local_name() != &*atom!("svg") {
        return false;
    }

    let parent = match e.traversal_parent() {
        Some(n) => n,
        None => return true,
    };

    if !parent.is_svg_element() {
        return true;
    }

    parent.local_name() == &*atom!("foreignObject")
}

// https://drafts.csswg.org/css-display/#unbox
#[cfg(feature = "gecko")]
fn is_effective_display_none_for_display_contents<E>(element: E) -> bool
where
    E: TElement,
{
    use crate::Atom;

    // FIXME(emilio): This should be an actual static.
    lazy_static! {
        static ref SPECIAL_HTML_ELEMENTS: [Atom; 16] = [
            atom!("br"),
            atom!("wbr"),
            atom!("meter"),
            atom!("progress"),
            atom!("canvas"),
            atom!("embed"),
            atom!("object"),
            atom!("audio"),
            atom!("iframe"),
            atom!("img"),
            atom!("video"),
            atom!("frame"),
            atom!("frameset"),
            atom!("input"),
            atom!("textarea"),
            atom!("select"),
        ];
    }

    // https://drafts.csswg.org/css-display/#unbox-svg
    //
    // There's a note about "Unknown elements", but there's not a good way to
    // know what that means, or to get that information from here, and no other
    // UA implements this either.
    lazy_static! {
        static ref SPECIAL_SVG_ELEMENTS: [Atom; 6] = [
            atom!("svg"),
            atom!("a"),
            atom!("g"),
            atom!("use"),
            atom!("tspan"),
            atom!("textPath"),
        ];
    }

    // https://drafts.csswg.org/css-display/#unbox-html
    if element.is_html_element() {
        let local_name = element.local_name();
        return SPECIAL_HTML_ELEMENTS
            .iter()
            .any(|name| &**name == local_name);
    }

    // https://drafts.csswg.org/css-display/#unbox-svg
    if element.is_svg_element() {
        if is_topmost_svg_svg_element(element) {
            return true;
        }
        let local_name = element.local_name();
        return !SPECIAL_SVG_ELEMENTS
            .iter()
            .any(|name| &**name == local_name);
    }

    // https://drafts.csswg.org/css-display/#unbox-mathml
    //
    // We always treat XUL as display: none. We don't use display:
    // contents in XUL anyway, so should be fine to be consistent with
    // MathML unless there's a use case for it.
    if element.is_mathml_element() || element.is_xul_element() {
        return true;
    }

    false
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

    /// Whether we should skip any item-based display property blockification on
    /// this element.
    fn skip_item_display_fixup<E>(&self, element: Option<E>) -> bool
    where
        E: TElement,
    {
        if let Some(pseudo) = self.style.pseudo {
            return pseudo.skip_item_display_fixup();
        }

        element.map_or(false, |e| e.skip_item_display_fixup())
    }

    /// Apply the blockification rules based on the table in CSS 2.2 section 9.7.
    /// <https://drafts.csswg.org/css2/visuren.html#dis-pos-flo>
    /// A ::marker pseudo-element with 'list-style-position:outside' needs to
    /// have its 'display' blockified.
    fn blockify_if_necessary<E>(&mut self, layout_parent_style: &ComputedValues, element: Option<E>)
    where
        E: TElement,
    {
        use crate::computed_values::list_style_position::T as ListStylePosition;

        let mut blockify = false;
        macro_rules! blockify_if {
            ($if_what:expr) => {
                if !blockify {
                    blockify = $if_what;
                }
            };
        }

        let is_root = self.style.pseudo.is_none() && element.map_or(false, |e| e.is_root());
        blockify_if!(is_root);
        if !self.skip_item_display_fixup(element) {
            blockify_if!(layout_parent_style
                .get_box()
                .clone_display()
                .is_item_container());
        }

        let is_item_or_root = blockify;

        blockify_if!(self.style.floated());
        blockify_if!(self.style.out_of_flow_positioned());
        blockify_if!(
            self.style.pseudo.map_or(false, |p| p.is_marker()) &&
                self.style.get_parent_list().clone_list_style_position() ==
                    ListStylePosition::Outside
        );

        if !blockify {
            return;
        }

        let display = self.style.get_box().clone_display();
        let blockified_display = display.equivalent_block_display(is_root);
        if display != blockified_display {
            self.style
                .mutate_box()
                .set_adjusted_display(blockified_display, is_item_or_root);
        }
    }

    /// Compute a few common flags for both text and element's style.
    pub fn set_bits(&mut self) {
        let display = self.style.get_box().clone_display();

        if !display.is_contents() &&
            !self
                .style
                .get_text()
                .clone_text_decoration_line()
                .is_empty()
        {
            self.style
                .add_flags(ComputedValueFlags::HAS_TEXT_DECORATION_LINES);
        }

        if self.style.is_pseudo_element() {
            self.style
                .add_flags(ComputedValueFlags::IS_IN_PSEUDO_ELEMENT_SUBTREE);
        }

        #[cfg(feature = "servo")]
        {
            if self.style.get_parent_column().is_multicol() {
                self.style.add_flags(ComputedValueFlags::CAN_BE_FRAGMENTED);
            }
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
        use crate::computed_values::text_combine_upright::T as TextCombineUpright;
        use crate::computed_values::writing_mode::T as WritingMode;

        let writing_mode = self.style.get_inherited_box().clone_writing_mode();
        let text_combine_upright = self.style.get_inherited_text().clone_text_combine_upright();

        if writing_mode != WritingMode::HorizontalTb &&
            text_combine_upright == TextCombineUpright::All
        {
            self.style.add_flags(ComputedValueFlags::IS_TEXT_COMBINED);
            self.style
                .mutate_inherited_box()
                .set_writing_mode(WritingMode::HorizontalTb);
        }
    }

    /// Unconditionally propagates the line break suppression flag to text, and
    /// additionally it applies it if it is in any ruby box.
    ///
    /// This is necessary because its parent may not itself have the flag set
    /// (e.g. ruby or ruby containers), thus we may not inherit the flag from
    /// them.
    #[cfg(feature = "gecko")]
    fn adjust_for_text_in_ruby(&mut self) {
        let parent_display = self.style.get_parent_box().clone_display();
        if parent_display.is_ruby_type() ||
            self.style
                .get_parent_flags()
                .contains(ComputedValueFlags::SHOULD_SUPPRESS_LINEBREAK)
        {
            self.style
                .add_flags(ComputedValueFlags::SHOULD_SUPPRESS_LINEBREAK);
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
    fn adjust_for_writing_mode(&mut self, layout_parent_style: &ComputedValues) {
        let our_writing_mode = self.style.get_inherited_box().clone_writing_mode();
        let parent_writing_mode = layout_parent_style.get_inherited_box().clone_writing_mode();

        if our_writing_mode != parent_writing_mode &&
            self.style.get_box().clone_display() == Display::Inline
        {
            // TODO(emilio): Figure out if we can just set the adjusted display
            // on Gecko too and unify this code path.
            if cfg!(feature = "servo") {
                self.style
                    .mutate_box()
                    .set_adjusted_display(Display::InlineBlock, false);
            } else {
                self.style.mutate_box().set_display(Display::InlineBlock);
            }
        }
    }

    /// When mathvariant is not "none", font-weight and font-style are
    /// both forced to "normal".
    #[cfg(feature = "gecko")]
    fn adjust_for_mathvariant(&mut self) {
        use crate::properties::longhands::_moz_math_variant::computed_value::T as MozMathVariant;
        use crate::properties::longhands::font_weight::computed_value::T as FontWeight;
        use crate::values::generics::font::FontStyle;
        if self.style.get_font().clone__moz_math_variant() != MozMathVariant::None {
            let font_style = self.style.mutate_font();
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
        use crate::computed_values::align_items::T as AlignItems;
        use crate::computed_values::align_self::T as AlignSelf;

        if self.style.get_position().clone_align_self() == AlignSelf::Auto &&
            !self.style.out_of_flow_positioned()
        {
            let self_align = match layout_parent_style.get_position().clone_align_items() {
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
        if self
            .style
            .get_outline()
            .clone_outline_style()
            .none_or_hidden() &&
            self.style.get_outline().outline_has_nonzero_width()
        {
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

        if overflow_x != original_overflow_x || overflow_y != original_overflow_y {
            let box_style = self.style.mutate_box();
            box_style.set_overflow_x(overflow_x);
            box_style.set_overflow_y(overflow_y);
        }
    }

    /// Handles the relevant sections in:
    ///
    /// https://drafts.csswg.org/css-display/#unbox-html
    ///
    /// And forbidding display: contents in pseudo-elements, at least for now.
    #[cfg(feature = "gecko")]
    fn adjust_for_prohibited_display_contents<E>(&mut self, element: Option<E>)
    where
        E: TElement,
    {
        if self.style.get_box().clone_display() != Display::Contents {
            return;
        }

        // FIXME(emilio): ::before and ::after should support display: contents,
        // see bug 1418138.
        if self.style.pseudo.is_some() {
            self.style.mutate_box().set_display(Display::Inline);
            return;
        }

        let element = match element {
            Some(e) => e,
            None => return,
        };

        if is_effective_display_none_for_display_contents(element) {
            self.style.mutate_box().set_display(Display::None);
        }
    }

    /// If a <fieldset> has grid/flex display type, we need to inherit
    /// this type into its ::-moz-fieldset-content anonymous box.
    ///
    /// NOTE(emilio): We don't need to handle the display change for this case
    /// in matching.rs because anonymous box restyling works separately to the
    /// normal cascading process.
    #[cfg(feature = "gecko")]
    fn adjust_for_fieldset_content(&mut self, layout_parent_style: &ComputedValues) {
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
            Display::Flex | Display::InlineFlex => Some(Display::Flex),
            Display::Grid | Display::InlineGrid => Some(Display::Grid),
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
        use crate::properties::longhands::text_align::computed_value::T as TextAlign;
        if self.style.get_box().clone_display() != Display::Table {
            return;
        }

        match self.style.get_inherited_text().clone_text_align() {
            TextAlign::MozLeft | TextAlign::MozCenter | TextAlign::MozRight => {},
            _ => return,
        }

        self.style
            .mutate_inherited_text()
            .set_text_align(TextAlign::Start)
    }

    /// Computes the used text decoration for Servo.
    ///
    /// FIXME(emilio): This is a layout tree concept, should move away from
    /// style, since otherwise we're going to have the same subtle bugs WebKit
    /// and Blink have with this very same thing.
    #[cfg(feature = "servo")]
    fn adjust_for_text_decorations_in_effect(&mut self) {
        use crate::values::computed::text::TextDecorationsInEffect;

        let decorations_in_effect = TextDecorationsInEffect::from_style(&self.style);
        if self.style.get_inherited_text().text_decorations_in_effect != decorations_in_effect {
            self.style
                .mutate_inherited_text()
                .text_decorations_in_effect = decorations_in_effect;
        }
    }

    #[cfg(feature = "gecko")]
    fn should_suppress_linebreak(&self, layout_parent_style: &ComputedValues) -> bool {
        // Line break suppression should only be propagated to in-flow children.
        if self.style.floated() || self.style.out_of_flow_positioned() {
            return false;
        }
        let parent_display = layout_parent_style.get_box().clone_display();
        if layout_parent_style
            .flags
            .contains(ComputedValueFlags::SHOULD_SUPPRESS_LINEBREAK)
        {
            // Line break suppression is propagated to any children of
            // line participants.
            if parent_display.is_line_participant() {
                return true;
            }
        }
        match self.style.get_box().clone_display() {
            // Ruby base and text are always non-breakable.
            Display::RubyBase | Display::RubyText => true,
            // Ruby base container and text container are breakable.
            // Note that, when certain HTML tags, e.g. form controls, have ruby
            // level container display type, they could also escape from the
            // line break suppression flag while they shouldn't. However, it is
            // generally fine since they themselves are non-breakable.
            Display::RubyBaseContainer | Display::RubyTextContainer => false,
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
    fn adjust_for_ruby<E>(&mut self, layout_parent_style: &ComputedValues, element: Option<E>)
    where
        E: TElement,
    {
        use crate::properties::longhands::unicode_bidi::computed_value::T as UnicodeBidi;

        let self_display = self.style.get_box().clone_display();
        // Check whether line break should be suppressed for this element.
        if self.should_suppress_linebreak(layout_parent_style) {
            self.style
                .add_flags(ComputedValueFlags::SHOULD_SUPPRESS_LINEBREAK);
            // Inlinify the display type if allowed.
            if !self.skip_item_display_fixup(element) {
                let inline_display = self_display.inlinify();
                if self_display != inline_display {
                    self.style
                        .mutate_box()
                        .set_adjusted_display(inline_display, false);
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
                UnicodeBidi::Normal | UnicodeBidi::Embed => Some(UnicodeBidi::Isolate),
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
    fn adjust_for_visited<E>(&mut self, element: Option<E>)
    where
        E: TElement,
    {
        if !self.style.has_visited_style() {
            return;
        }

        let is_link_element = self.style.pseudo.is_none() && element.map_or(false, |e| e.is_link());

        if !is_link_element {
            return;
        }

        if element.unwrap().is_visited_link() {
            self.style
                .add_flags(ComputedValueFlags::IS_RELEVANT_LINK_VISITED);
        } else {
            // Need to remove to handle unvisited link inside visited.
            self.style
                .remove_flags(ComputedValueFlags::IS_RELEVANT_LINK_VISITED);
        }
    }

    /// Resolves "justify-items: legacy" based on the inherited style if needed
    /// to comply with:
    ///
    /// <https://drafts.csswg.org/css-align/#valdef-justify-items-legacy>
    #[cfg(feature = "gecko")]
    fn adjust_for_justify_items(&mut self) {
        use crate::values::specified::align;
        let justify_items = self.style.get_position().clone_justify_items();
        if justify_items.specified.0 != align::AlignFlags::LEGACY {
            return;
        }

        let parent_justify_items = self.style.get_parent_position().clone_justify_items();

        if !parent_justify_items
            .computed
            .0
            .contains(align::AlignFlags::LEGACY)
        {
            return;
        }

        if parent_justify_items.computed == justify_items.computed {
            return;
        }

        self.style
            .mutate_position()
            .set_computed_justify_items(parent_justify_items.computed);
    }

    /// If '-webkit-appearance' is 'menulist' on a <select> element then
    /// the computed value of 'line-height' is 'normal'.
    ///
    /// https://github.com/w3c/csswg-drafts/issues/3257
    #[cfg(feature = "gecko")]
    fn adjust_for_appearance<E>(&mut self, element: Option<E>)
    where
        E: TElement,
    {
        use crate::properties::longhands::_moz_appearance::computed_value::T as Appearance;
        use crate::properties::longhands::line_height::computed_value::T as LineHeight;

        if self.style.get_box().clone__moz_appearance() == Appearance::Menulist {
            if self.style.get_inherited_text().clone_line_height() == LineHeight::normal() {
                return;
            }
            if self.style.pseudo.is_some() {
                return;
            }
            let is_html_select_element = element.map_or(false, |e| {
                e.is_html_element() && e.local_name() == &*local_name!("select")
            });
            if !is_html_select_element {
                return;
            }
            self.style
                .mutate_inherited_text()
                .set_line_height(LineHeight::normal());
        }
    }

    /// Adjusts the style to account for various fixups that don't fit naturally
    /// into the cascade.
    ///
    /// When comparing to Gecko, this is similar to the work done by
    /// `ComputedStyle::ApplyStyleFixups`, plus some parts of
    /// `nsStyleSet::GetContext`.
    pub fn adjust<E>(&mut self, layout_parent_style: &ComputedValues, element: Option<E>)
    where
        E: TElement,
    {
        if cfg!(debug_assertions) {
            if element
                .and_then(|e| e.implemented_pseudo_element())
                .is_some()
            {
                // It'd be nice to assert `self.style.pseudo == Some(&pseudo)`,
                // but we do resolve ::-moz-list pseudos on ::before / ::after
                // content, sigh.
                debug_assert!(self.style.pseudo.is_some(), "Someone really messed up");
            }
        }
        // FIXME(emilio): The apply_declarations callsite in Servo's
        // animation, and the font stuff for Gecko
        // (Stylist::compute_for_declarations) should pass an element to
        // cascade(), then we can make this assertion hold everywhere.
        // debug_assert!(
        //     element.is_some() || self.style.pseudo.is_some(),
        //     "Should always have an element around for non-pseudo styles"
        // );

        self.adjust_for_visited(element);
        #[cfg(feature = "gecko")]
        {
            self.adjust_for_prohibited_display_contents(element);
            self.adjust_for_fieldset_content(layout_parent_style);
        }
        self.adjust_for_top_layer();
        self.blockify_if_necessary(layout_parent_style, element);
        self.adjust_for_position();
        self.adjust_for_overflow();
        #[cfg(feature = "gecko")]
        {
            self.adjust_for_table_text_align();
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
        #[cfg(feature = "gecko")]
        {
            self.adjust_for_ruby(layout_parent_style, element);
        }
        #[cfg(feature = "servo")]
        {
            self.adjust_for_text_decorations_in_effect();
        }
        #[cfg(feature = "gecko")]
        {
            self.adjust_for_appearance(element);
        }
        self.set_bits();
    }
}
