/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The restyle damage is a hint that tells layout which kind of operations may
//! be needed in presence of incremental style changes.

use computed_values::display::T as Display;
use matching::{StyleChange, StyleDifference};
use properties::ComputedValues;
use std::fmt;

bitflags! {
    /// Individual layout actions that may be necessary after restyling.
    pub struct ServoRestyleDamage: u8 {
        /// Repaint the node itself.
        ///
        /// Currently unused; need to decide how this propagates.
        const REPAINT = 0x01;

        /// The stacking-context-relative position of this node or its
        /// descendants has changed.
        ///
        /// Propagates both up and down the flow tree.
        const REPOSITION = 0x02;

        /// Recompute the overflow regions (bounding box of object and all descendants).
        ///
        /// Propagates down the flow tree because the computation is bottom-up.
        const STORE_OVERFLOW = 0x04;

        /// Recompute intrinsic inline_sizes (minimum and preferred).
        ///
        /// Propagates down the flow tree because the computation is.
        /// bottom-up.
        const BUBBLE_ISIZES = 0x08;

        /// Recompute actual inline-sizes and block-sizes, only taking
        /// out-of-flow children into account.
        ///
        /// Propagates up the flow tree because the computation is top-down.
        const REFLOW_OUT_OF_FLOW = 0x10;

        /// Recompute actual inline_sizes and block_sizes.
        ///
        /// Propagates up the flow tree because the computation is top-down.
        const REFLOW = 0x20;

        /// Re-resolve generated content.
        ///
        /// Propagates up the flow tree because the computation is inorder.
        const RESOLVE_GENERATED_CONTENT = 0x40;

        /// The entire flow needs to be reconstructed.
        const RECONSTRUCT_FLOW = 0x80;
    }
}

malloc_size_of_is_0!(ServoRestyleDamage);

impl ServoRestyleDamage {
    /// Compute the `StyleDifference` (including the appropriate restyle damage)
    /// for a given style change between `old` and `new`.
    pub fn compute_style_difference(
        old: &ComputedValues,
        new: &ComputedValues,
    ) -> StyleDifference {
        let damage = compute_damage(old, new);
        let change = if damage.is_empty() {
            StyleChange::Unchanged
        } else {
            // FIXME(emilio): Differentiate between reset and inherited
            // properties here, and set `reset_only` appropriately so the
            // optimization to skip the cascade in those cases applies.
            StyleChange::Changed { reset_only: false }
        };
        StyleDifference { damage, change }
    }

    /// Returns a bitmask that represents a flow that needs to be rebuilt and
    /// reflowed.
    ///
    /// FIXME(bholley): Do we ever actually need this? Shouldn't
    /// RECONSTRUCT_FLOW imply everything else?
    pub fn rebuild_and_reflow() -> ServoRestyleDamage {
        ServoRestyleDamage::REPAINT | ServoRestyleDamage::REPOSITION |
        ServoRestyleDamage::STORE_OVERFLOW | ServoRestyleDamage::BUBBLE_ISIZES |
        ServoRestyleDamage::REFLOW_OUT_OF_FLOW | ServoRestyleDamage::REFLOW |
        ServoRestyleDamage::RECONSTRUCT_FLOW
    }

    /// Returns a bitmask indicating that the frame needs to be reconstructed.
    pub fn reconstruct() -> ServoRestyleDamage {
        ServoRestyleDamage::RECONSTRUCT_FLOW
    }

    /// Supposing a flow has the given `position` property and this damage,
    /// returns the damage that we should add to the *parent* of this flow.
    pub fn damage_for_parent(self, child_is_absolutely_positioned: bool) -> ServoRestyleDamage {
        if child_is_absolutely_positioned {
            self & (ServoRestyleDamage::REPAINT | ServoRestyleDamage::REPOSITION |
                    ServoRestyleDamage::STORE_OVERFLOW | ServoRestyleDamage::REFLOW_OUT_OF_FLOW |
                    ServoRestyleDamage::RESOLVE_GENERATED_CONTENT)
        } else {
            self & (ServoRestyleDamage::REPAINT | ServoRestyleDamage::REPOSITION |
                    ServoRestyleDamage::STORE_OVERFLOW | ServoRestyleDamage::REFLOW |
                    ServoRestyleDamage::REFLOW_OUT_OF_FLOW | ServoRestyleDamage::RESOLVE_GENERATED_CONTENT)
        }
    }

    /// Supposing the *parent* of a flow with the given `position` property has
    /// this damage, returns the damage that we should add to this flow.
    pub fn damage_for_child(self,
                            parent_is_absolutely_positioned: bool,
                            child_is_absolutely_positioned: bool)
                            -> ServoRestyleDamage {
        match (parent_is_absolutely_positioned, child_is_absolutely_positioned) {
            (false, true) => {
                // Absolute children are out-of-flow and therefore insulated from changes.
                //
                // FIXME(pcwalton): Au contraire, if the containing block dimensions change!
                self & (ServoRestyleDamage::REPAINT | ServoRestyleDamage::REPOSITION)
            }
            (true, false) => {
                // Changing the position of an absolutely-positioned block requires us to reflow
                // its kids.
                if self.contains(ServoRestyleDamage::REFLOW_OUT_OF_FLOW) {
                    self | ServoRestyleDamage::REFLOW
                } else {
                    self
                }
            }
            _ => {
                // TODO(pcwalton): Take floatedness into account.
                self & (ServoRestyleDamage::REPAINT | ServoRestyleDamage::REPOSITION | ServoRestyleDamage::REFLOW)
            }
        }
    }
}

impl Default for ServoRestyleDamage {
    fn default() -> Self {
        Self::empty()
    }
}

impl fmt::Display for ServoRestyleDamage {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut first_elem = true;

        let to_iter =
            [ (ServoRestyleDamage::REPAINT, "Repaint")
            , (ServoRestyleDamage::REPOSITION, "Reposition")
            , (ServoRestyleDamage::STORE_OVERFLOW, "StoreOverflow")
            , (ServoRestyleDamage::BUBBLE_ISIZES, "BubbleISizes")
            , (ServoRestyleDamage::REFLOW_OUT_OF_FLOW, "ReflowOutOfFlow")
            , (ServoRestyleDamage::REFLOW, "Reflow")
            , (ServoRestyleDamage::RESOLVE_GENERATED_CONTENT, "ResolveGeneratedContent")
            , (ServoRestyleDamage::RECONSTRUCT_FLOW, "ReconstructFlow")
            ];

        for &(damage, damage_str) in &to_iter {
            if self.contains(damage) {
                if !first_elem { write!(f, " | ")?; }
                write!(f, "{}", damage_str)?;
                first_elem = false;
            }
        }

        if first_elem {
            write!(f, "NoDamage")?;
        }

        Ok(())
    }
}

// NB: We need the braces inside the RHS due to Rust #8012.  This particular
// version of this macro might be safe anyway, but we want to avoid silent
// breakage on modifications.
macro_rules! add_if_not_equal(
    ($old:ident, $new:ident, $damage:ident,
     [ $($effect:path),* ], [ $($style_struct_getter:ident.$name:ident),* ]) => ({
        if $( ($old.$style_struct_getter().$name != $new.$style_struct_getter().$name) )||* {
            $damage.insert($($effect)|*);
            true
        } else {
            false
        }
    })
);

fn compute_damage(old: &ComputedValues, new: &ComputedValues) -> ServoRestyleDamage {
    let mut damage = ServoRestyleDamage::empty();

    // This should check every CSS property, as enumerated in the fields of
    // http://doc.servo.org/style/properties/struct.ComputedValues.html

    // FIXME: Test somehow that every property is included.

    add_if_not_equal!(old, new, damage,
                      [ServoRestyleDamage::REPAINT, ServoRestyleDamage::REPOSITION,
                      ServoRestyleDamage::STORE_OVERFLOW, ServoRestyleDamage::BUBBLE_ISIZES,
                      ServoRestyleDamage::REFLOW_OUT_OF_FLOW, ServoRestyleDamage::REFLOW,
                      ServoRestyleDamage::RECONSTRUCT_FLOW], [
        get_box.clear, get_box.float, get_box.display, get_box.position, get_counters.content,
        get_counters.counter_reset, get_counters.counter_increment,
        get_list.quotes, get_list.list_style_type,

        // If these text or font properties change, we need to reconstruct the flow so that
        // text shaping is re-run.
        get_inheritedtext.letter_spacing, get_inheritedtext.text_rendering,
        get_inheritedtext.text_transform, get_inheritedtext.word_spacing,
        get_inheritedtext.overflow_wrap, get_inheritedtext.text_justify,
        get_inheritedtext.white_space, get_inheritedtext.word_break, get_text.text_overflow,
        get_font.font_family, get_font.font_style, get_font.font_variant_caps, get_font.font_weight,
        get_font.font_size, get_font.font_stretch,
        get_inheritedbox.direction, get_inheritedbox.writing_mode,
        get_text.text_decoration_line, get_text.unicode_bidi,
        get_inheritedtable.empty_cells, get_inheritedtable.caption_side,
        get_column.column_width, get_column.column_count
    ]) || (new.get_box().display == Display::Inline &&
           add_if_not_equal!(old, new, damage,
                             [ServoRestyleDamage::REPAINT, ServoRestyleDamage::REPOSITION,
                             ServoRestyleDamage::STORE_OVERFLOW, ServoRestyleDamage::BUBBLE_ISIZES,
                             ServoRestyleDamage::REFLOW_OUT_OF_FLOW, ServoRestyleDamage::REFLOW,
                             ServoRestyleDamage::RECONSTRUCT_FLOW], [
        // For inline boxes only, border/padding styles are used in flow construction (to decide
        // whether to create fragments for empty flows).
        get_border.border_top_width, get_border.border_right_width,
        get_border.border_bottom_width, get_border.border_left_width,
        get_padding.padding_top, get_padding.padding_right,
        get_padding.padding_bottom, get_padding.padding_left
    ])) || add_if_not_equal!(old, new, damage,
                            [ServoRestyleDamage::REPAINT, ServoRestyleDamage::REPOSITION,
                            ServoRestyleDamage::STORE_OVERFLOW, ServoRestyleDamage::BUBBLE_ISIZES,
                            ServoRestyleDamage::REFLOW_OUT_OF_FLOW, ServoRestyleDamage::REFLOW],
        [get_border.border_top_width, get_border.border_right_width,
        get_border.border_bottom_width, get_border.border_left_width,
        get_margin.margin_top, get_margin.margin_right,
        get_margin.margin_bottom, get_margin.margin_left,
        get_padding.padding_top, get_padding.padding_right,
        get_padding.padding_bottom, get_padding.padding_left,
        get_position.width, get_position.height,
        get_inheritedtext.line_height,
        get_inheritedtext.text_align, get_inheritedtext.text_indent,
        get_table.table_layout,
        get_inheritedtable.border_collapse,
        get_inheritedtable.border_spacing,
        get_column.column_gap,
        get_position.flex_direction,
        get_position.flex_wrap,
        get_position.justify_content,
        get_position.align_items,
        get_position.align_content,
        get_position.order,
        get_position.flex_basis,
        get_position.flex_grow,
        get_position.flex_shrink,
        get_position.align_self
    ]) || add_if_not_equal!(old, new, damage,
                            [ServoRestyleDamage::REPAINT, ServoRestyleDamage::REPOSITION,
                            ServoRestyleDamage::STORE_OVERFLOW, ServoRestyleDamage::REFLOW_OUT_OF_FLOW],
        [get_position.top, get_position.left,
        get_position.right, get_position.bottom,
        get_effects.opacity,
        get_box.transform, get_box.transform_style, get_box.transform_origin,
        get_box.perspective, get_box.perspective_origin
    ]) || add_if_not_equal!(old, new, damage,
                            [ServoRestyleDamage::REPAINT], [
        get_color.color, get_background.background_color,
        get_background.background_image, get_background.background_position_x,
        get_background.background_position_y, get_background.background_repeat,
        get_background.background_attachment, get_background.background_clip,
        get_background.background_origin, get_background.background_size,
        get_border.border_top_color, get_border.border_right_color,
        get_border.border_bottom_color, get_border.border_left_color,
        get_border.border_top_style, get_border.border_right_style,
        get_border.border_bottom_style, get_border.border_left_style,
        get_border.border_top_left_radius, get_border.border_top_right_radius,
        get_border.border_bottom_left_radius, get_border.border_bottom_right_radius,
        get_position.z_index, get_box._servo_overflow_clip_box,
        get_inheritedtext._servo_text_decorations_in_effect,
        get_pointing.cursor, get_pointing.pointer_events,
        get_effects.box_shadow, get_effects.clip, get_inheritedtext.text_shadow, get_effects.filter,
        get_effects.mix_blend_mode, get_inheritedbox.image_rendering,

        // Note: May require REFLOW et al. if `visibility: collapse` is implemented.
        get_inheritedbox.visibility
    ]);


    // Paint worklets may depend on custom properties,
    // so if they have changed we should repaint.
    if old.custom_properties() != new.custom_properties() {
        damage.insert(ServoRestyleDamage::REPAINT);
    }

    // If the layer requirements of this flow have changed due to the value
    // of the transform, then reflow is required to rebuild the layers.
    if old.transform_requires_layer() != new.transform_requires_layer() {
        damage.insert(ServoRestyleDamage::rebuild_and_reflow());
    }

    damage
}
