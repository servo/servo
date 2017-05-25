/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A struct to encapsulate all the style fixups a computed style needs in order
//! for it to adhere to the CSS spec.

use app_units::Au;
use properties::{self, ComputedValues, StyleBuilder};
use properties::longhands::display::computed_value::T as display;
use properties::longhands::float::computed_value::T as float;
use properties::longhands::overflow_x::computed_value::T as overflow;
use properties::longhands::position::computed_value::T as position;


/// An unsized struct that implements all the adjustment methods.
pub struct StyleAdjuster<'a, 'b: 'a> {
    style: &'a mut StyleBuilder<'b>,
    is_root_element: bool,
}

impl<'a, 'b: 'a> StyleAdjuster<'a, 'b> {
    /// Trivially constructs a new StyleAdjuster.
    pub fn new(style: &'a mut StyleBuilder<'b>, is_root_element: bool) -> Self {
        StyleAdjuster {
            style: style,
            is_root_element: is_root_element,
        }
    }

    /// https://fullscreen.spec.whatwg.org/#new-stacking-layer
    ///
    ///    Any position value other than 'absolute' and 'fixed' are
    ///    computed to 'absolute' if the element is in a top layer.
    ///
    fn adjust_for_top_layer(&mut self) {
        if !self.style.out_of_flow_positioned() && self.style.in_top_layer() {
            self.style.mutate_box().set_position(position::absolute);
        }
    }

    /// CSS 2.1 section 9.7:
    ///
    ///    If 'position' has the value 'absolute' or 'fixed', [...] the computed
    ///    value of 'float' is 'none'.
    ///
    fn adjust_for_position(&mut self) {
        if self.style.out_of_flow_positioned() && self.style.floated() {
            self.style.mutate_box().set_float(float::none);
        }
    }

    /// Apply the blockification rules based on the table in CSS 2.2 section 9.7.
    /// https://drafts.csswg.org/css2/visuren.html#dis-pos-flo
    fn blockify_if_necessary(&mut self,
                             layout_parent_style: &ComputedValues,
                             skip_root_and_element_display_fixup: bool) {
        let mut blockify = false;
        macro_rules! blockify_if {
            ($if_what:expr) => {
                if !blockify {
                    blockify = $if_what;
                }
            }
        }

        if !skip_root_and_element_display_fixup {
            blockify_if!(self.is_root_element);
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
            display.equivalent_block_display(self.is_root_element);
        if display != blockified_display {
            self.style.mutate_box().set_adjusted_display(blockified_display,
                                                         is_item_or_root);
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
    }

    /// Change writing mode of the text frame for text-combine-upright.
    ///
    /// It is safe to look at our own style because we are looking at inherited
    /// properties, and text is just plain inheritance.
    ///
    /// TODO(emilio): we should (Gecko too) revise these adjustments in presence
    /// of display: contents.
    #[cfg(feature = "gecko")]
    fn adjust_for_text_combine_upright(&mut self) {
        use computed_values::text_combine_upright::T as text_combine_upright;
        use computed_values::writing_mode::T as writing_mode;

        let writing_mode =
            self.style.get_inheritedbox().clone_writing_mode();
        let text_combine_upright =
            self.style.get_inheritedtext().clone_text_combine_upright();

        if writing_mode != writing_mode::horizontal_tb &&
           text_combine_upright == text_combine_upright::all {
            self.style.mutate_inheritedbox().set_writing_mode(writing_mode::horizontal_tb);
        }
    }

    /// https://drafts.csswg.org/css-writing-modes-3/#block-flow:
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
    /// https://lists.w3.org/Archives/Public/www-style/2017Mar/0045.html
    /// https://github.com/servo/servo/issues/15754
    fn adjust_for_writing_mode(&mut self,
                               layout_parent_style: &ComputedValues) {
        let our_writing_mode = self.style.get_inheritedbox().clone_writing_mode();
        let parent_writing_mode = layout_parent_style.get_inheritedbox().clone_writing_mode();

        if our_writing_mode != parent_writing_mode &&
           self.style.get_box().clone_display() == display::inline {
            self.style.mutate_box().set_display(display::inline_block);
        }
    }

    #[cfg(feature = "gecko")]
    fn adjust_for_contain(&mut self) {
        use properties::longhands::contain;

        // An element with contain: paint needs to be a formatting context, and
        // also implies overflow: clip.
        //
        // TODO(emilio): This mimics Gecko, but spec links are missing!
        let contain = self.style.get_box().clone_contain();
        if !contain.contains(contain::PAINT) {
            return;
        }

        if self.style.get_box().clone_display() == display::inline {
            self.style.mutate_box().set_adjusted_display(display::inline_block,
                                                         false);
        }


        // When 'contain: paint', update overflow from 'visible' to 'clip'.
        if self.style.get_box().clone_contain().contains(contain::PAINT) {
            if self.style.get_box().clone_overflow_x() == overflow::visible {
                let mut box_style = self.style.mutate_box();
                box_style.set_overflow_x(overflow::_moz_hidden_unscrollable);
                box_style.set_overflow_y(overflow::_moz_hidden_unscrollable);
            }
        }
    }

    /// When mathvariant is not "none", font-weight and font-style are
    /// both forced to "normal".
    #[cfg(feature = "gecko")]
    fn adjust_for_mathvariant(&mut self) {
        use properties::longhands::_moz_math_variant::computed_value::T as moz_math_variant;
        use properties::longhands::font_style::computed_value::T as font_style;
        use properties::longhands::font_weight::computed_value::T as font_weight;
        if self.style.get_font().clone__moz_math_variant() != moz_math_variant::none {
            let mut font_style = self.style.mutate_font();
            // Sadly we don't have a nice name for the computed value
            // of "font-weight: normal".
            font_style.set_font_weight(font_weight::Weight400);
            font_style.set_font_style(font_style::normal);
        }
    }

    /// This implements an out-of-date spec. The new spec moves the handling of
    /// this to layout, which Gecko implements but Servo doesn't.
    ///
    /// See https://github.com/servo/servo/issues/15229
    #[cfg(feature = "servo")]
    fn adjust_for_alignment(&mut self, layout_parent_style: &ComputedValues) {
        use computed_values::align_items::T as align_items;
        use computed_values::align_self::T as align_self;

        if self.style.get_position().clone_align_self() == align_self::auto &&
           !self.style.out_of_flow_positioned() {
            let self_align =
                match layout_parent_style.get_position().clone_align_items() {
                    align_items::stretch => align_self::stretch,
                    align_items::baseline => align_self::baseline,
                    align_items::flex_start => align_self::flex_start,
                    align_items::flex_end => align_self::flex_end,
                    align_items::center => align_self::center,
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
            self.style.mutate_outline().set_outline_width(Au(0));
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
        if overflow_x == overflow::visible {
            overflow_x = overflow::auto;
        }

        if overflow_y == overflow::visible {
            overflow_y = overflow::auto;
        }

        #[cfg(feature = "gecko")]
        {
            // overflow: clip is deprecated, so convert to hidden if it's
            // specified in only one dimension.
            if overflow_x == overflow::_moz_hidden_unscrollable {
                overflow_x = overflow::hidden;
            }
            if overflow_y == overflow::_moz_hidden_unscrollable {
                overflow_y = overflow::hidden;
            }
        }

        if overflow_x != original_overflow_x ||
           overflow_y != original_overflow_y {
            let mut box_style = self.style.mutate_box();
            box_style.set_overflow_x(overflow_x);
            box_style.set_overflow_y(overflow_y);
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
        use properties::longhands::text_align::computed_value::T as text_align;
       if self.style.get_box().clone_display() != display::table {
           return;
       }

       match self.style.get_inheritedtext().clone_text_align() {
           text_align::_moz_left |
           text_align::_moz_center |
           text_align::_moz_right => {}
           _ => return,
       }

       self.style.mutate_inheritedtext().set_text_align(text_align::start);
    }

    /// Adjusts the style to account for various fixups that don't fit naturally
    /// into the cascade.
    ///
    /// When comparing to Gecko, this is similar to the work done by
    /// `nsStyleContext::ApplyStyleFixups`.
    pub fn adjust(&mut self,
                  layout_parent_style: &ComputedValues,
                  skip_root_and_element_display_fixup: bool) {
        self.adjust_for_top_layer();
        self.blockify_if_necessary(layout_parent_style,
                                   skip_root_and_element_display_fixup);
        self.adjust_for_position();
        self.adjust_for_overflow();
        #[cfg(feature = "gecko")]
        {
            self.adjust_for_table_text_align();
            self.adjust_for_contain();
            self.adjust_for_mathvariant();
        }
        #[cfg(feature = "servo")]
        {
            self.adjust_for_alignment(layout_parent_style);
        }
        self.adjust_for_border_width();
        self.adjust_for_outline();
        self.adjust_for_writing_mode(layout_parent_style);
    }
}
