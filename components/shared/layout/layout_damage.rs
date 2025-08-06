/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The restyle damage is a hint that tells layout which kind of operations may
//! be needed in presence of incremental style changes.

use std::fmt;

use bitflags::{Flags, bitflags};
use malloc_size_of::malloc_size_of_is_0;
use style::dom::{StyleChange, StyleDifference, TRestyleDamage};
use style::properties::{
    ComputedValues, restyle_damage_rebuild_box, restyle_damage_rebuild_stacking_context,
    restyle_damage_recalculate_overflow, restyle_damage_repaint,
};
use style::values::computed::Image;
use style::values::specified::align::AlignFlags;
use style::values::specified::box_::{DisplayInside, DisplayOutside};

bitflags! {
    /// Major phases of layout that need to be run due to the damage to a node during restyling. In
    /// addition to the 4 bytes used for that, the rest of the `u16` is exposed as an extension point
    /// for users of the crate to add their own custom types of damage that correspond to the
    /// layout system they are implementing.
    #[derive(Clone, Copy, Eq, PartialEq)]
    pub struct ServoRestyleDamage: u16 {
        /// Repaint the node itself.
        ///
        /// Propagates both up and down the flow tree.
        const REPAINT = 0b0001;

        /// Rebuilds the stacking contexts.
        ///
        /// Propagates both up and down the flow tree.
        const REBUILD_STACKING_CONTEXT = 0b0011;

        /// Recalculates the scrollable overflow.
        ///
        /// Propagates both up and down the flow tree.
        const RECALCULATE_OVERFLOW = 0b0111;

        /// Any other type of damage, which requires running layout again.
        ///
        /// Propagates both up and down the flow tree.
        const RELAYOUT = 0b1111;

        /// Recollect the box children for this element, because some of the them will be
        /// rebuilt.
        const RECOLLECT_BOX_TREE_CHILDREN = 0b0001_1111;

        /// Rebuild the entire box for this element, which means that every part of layout
        /// needs to happen again.
        const REBUILD_BOX = 0b0011_1111;
    }
}

malloc_size_of_is_0!(ServoRestyleDamage);

impl Default for ServoRestyleDamage {
    fn default() -> Self {
        Self::empty()
    }
}

impl std::fmt::Debug for ServoRestyleDamage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.contains(Self::REBUILD_BOX) {
            f.write_str("REBUILD_BOX")
        } else if self.contains(Self::RECOLLECT_BOX_TREE_CHILDREN) {
            f.write_str("RECOLLECT_BOX_TREE_CHILDREN")
        } else if self.contains(Self::RELAYOUT) {
            f.write_str("RELAYOUT")
        } else if self.contains(Self::RECALCULATE_OVERFLOW) {
            f.write_str("RECALCULATE_OVERFLOW")
        } else if self.contains(Self::REBUILD_STACKING_CONTEXT) {
            f.write_str("REBUILD_STACKING_CONTEXT")
        } else if self.contains(Self::REPAINT) {
            f.write_str("REPAINT")
        } else {
            f.write_str("EMPTY")
        }
    }
}

impl fmt::Display for ServoRestyleDamage {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut first_elem = true;

        let to_iter = [
            (ServoRestyleDamage::REPAINT, "Repaint"),
            (
                ServoRestyleDamage::REBUILD_STACKING_CONTEXT,
                "Rebuild stacking context",
            ),
            (
                ServoRestyleDamage::RECALCULATE_OVERFLOW,
                "Recalculate overflow",
            ),
            (ServoRestyleDamage::RELAYOUT, "Relayout"),
        ];

        for &(damage, damage_str) in &to_iter {
            if self.contains(damage) {
                if !first_elem {
                    write!(f, " | ")?;
                }
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

impl TRestyleDamage for ServoRestyleDamage {
    /// Clear/reset all damage flags
    fn clear(&mut self) {
        Flags::clear(self)
    }
    /// Whether the damage is empty ("no styles changed")
    fn is_empty(&self) -> bool {
        Flags::is_empty(self)
    }
    /// Mark the element as needing it's (eager) pseudo-elements rebuilt
    fn set_rebuild_pseudos(&mut self) {
        self.insert(Self::REBUILD_BOX);
    }
}

pub fn augmented_restyle_damage_rebuild_box(old: &ComputedValues, new: &ComputedValues) -> bool {
    let old_box = old.get_box();
    let new_box = new.get_box();
    restyle_damage_rebuild_box(old, new) ||
        old_box.original_display != new_box.original_display ||
        old_box.has_transform_or_perspective() != new_box.has_transform_or_perspective() ||
        old.get_effects().filter.0.is_empty() != new.get_effects().filter.0.is_empty()
}

pub fn augmented_restyle_damage_rebuild_stacking_context(
    old: &ComputedValues,
    new: &ComputedValues,
) -> bool {
    restyle_damage_rebuild_stacking_context(old, new) ||
        old.guarantees_stacking_context() != new.guarantees_stacking_context()
}

impl ServoRestyleDamage {
    /// Compute the `StyleDifference` (including the appropriate restyle damage)
    /// for a given style change between `old` and `new`.
    pub fn compute_style_difference(
        old: &ComputedValues,
        new: &ComputedValues,
    ) -> StyleDifference<Self> {
        let damage = Self::compute_damage(old, new);
        StyleDifference {
            damage,
            change: if damage.is_empty() {
                StyleChange::Unchanged
            } else {
                // FIXME(emilio): Differentiate between reset and inherited
                // properties here, and set `reset_only` appropriately so the
                // optimization to skip the cascade in those cases applies.
                StyleChange::Changed { reset_only: false }
            },
        }
    }

    fn compute_damage(old: &ComputedValues, new: &ComputedValues) -> Self {
        let mut damage = ServoRestyleDamage::empty();

        // Damage flags higher up the if-else chain imply damage flags lower down the if-else chain,
        // so we can skip the diffing process for later flags if an earlier flag is true
        if augmented_restyle_damage_rebuild_box(old, new) {
            damage.insert(Self::compute_layout_damage(old, new));
        } else if restyle_damage_recalculate_overflow(old, new) {
            damage.insert(ServoRestyleDamage::RECALCULATE_OVERFLOW)
        } else if augmented_restyle_damage_rebuild_stacking_context(old, new) {
            damage.insert(ServoRestyleDamage::REBUILD_STACKING_CONTEXT);
        } else if restyle_damage_repaint(old, new) || !old.custom_properties_equal(new) {
            // Paint worklets may depend on custom properties, so if they have changed we should repaint.
            damage.insert(ServoRestyleDamage::REPAINT);
        }

        damage
    }

    fn compute_layout_damage(old: &ComputedValues, new: &ComputedValues) -> ServoRestyleDamage {
        let box_tree_needs_rebuild = || {
            let old_box = old.get_box();
            let new_box = new.get_box();

            if old_box.display != new_box.display ||
                old_box.float != new_box.float ||
                old_box.position != new_box.position
            {
                return true;
            }

            if old.get_font() != new.get_font() {
                return true;
            }

            // NOTE: This should be kept in sync with the checks in `impl
            // StyleExt::establishes_block_formatting_context` for `ComputedValues` in
            // `components/layout/style_ext.rs`.
            if new_box.display.outside() == DisplayOutside::Block &&
                new_box.display.inside() == DisplayInside::Flow
            {
                let alignment_establishes_new_block_formatting_context =
                    |style: &ComputedValues| {
                        style.get_position().align_content.0.primary() != AlignFlags::NORMAL
                    };

                let old_column = old.get_column();
                let new_column = new.get_column();
                if old_box.overflow_x.is_scrollable() != new_box.overflow_x.is_scrollable() ||
                    old_column.is_multicol() != new_column.is_multicol() ||
                    old_column.column_span != new_column.column_span ||
                    alignment_establishes_new_block_formatting_context(old) !=
                        alignment_establishes_new_block_formatting_context(new)
                {
                    return true;
                }
            }

            if old_box.display.is_list_item() {
                let old_list = old.get_list();
                let new_list = new.get_list();
                if old_list.list_style_position != new_list.list_style_position ||
                    old_list.list_style_image != new_list.list_style_image ||
                    (new_list.list_style_image == Image::None &&
                        old_list.list_style_type != new_list.list_style_type)
                {
                    return true;
                }
            }

            if new.is_pseudo_style() && old.get_counters().content != new.get_counters().content {
                return true;
            }

            false
        };

        let text_shaping_needs_recollect = || {
            if old.clone_direction() != new.clone_direction() ||
                old.clone_unicode_bidi() != new.clone_unicode_bidi()
            {
                return true;
            }

            let old_text = old.get_inherited_text().clone();
            let new_text = new.get_inherited_text().clone();
            if old_text.white_space_collapse != new_text.white_space_collapse ||
                old_text.text_transform != new_text.text_transform ||
                old_text.word_break != new_text.word_break ||
                old_text.overflow_wrap != new_text.overflow_wrap ||
                old_text.letter_spacing != new_text.letter_spacing ||
                old_text.word_spacing != new_text.word_spacing ||
                old_text.text_rendering != new_text.text_rendering
            {
                return true;
            }

            false
        };

        if box_tree_needs_rebuild() {
            ServoRestyleDamage::REBUILD_BOX
        } else if text_shaping_needs_recollect() {
            ServoRestyleDamage::RECOLLECT_BOX_TREE_CHILDREN
        } else {
            // This element needs to be laid out again, but does not have any damage to
            // its box. In the future, we will distinguish between types of damage to the
            // fragment as well.
            ServoRestyleDamage::RELAYOUT
        }
    }

    pub fn has_box_damage(&self) -> bool {
        self.intersects(Self::REBUILD_BOX)
    }
}
