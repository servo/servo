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
    pub fn compute_style_difference(old: &ComputedValues, new: &ComputedValues) -> StyleDifference {
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
                ServoRestyleDamage::STORE_OVERFLOW |
                ServoRestyleDamage::REFLOW_OUT_OF_FLOW |
                ServoRestyleDamage::RESOLVE_GENERATED_CONTENT)
        } else {
            self & (ServoRestyleDamage::REPAINT | ServoRestyleDamage::REPOSITION |
                ServoRestyleDamage::STORE_OVERFLOW |
                ServoRestyleDamage::REFLOW |
                ServoRestyleDamage::REFLOW_OUT_OF_FLOW |
                ServoRestyleDamage::RESOLVE_GENERATED_CONTENT)
        }
    }

    /// Supposing the *parent* of a flow with the given `position` property has
    /// this damage, returns the damage that we should add to this flow.
    pub fn damage_for_child(
        self,
        parent_is_absolutely_positioned: bool,
        child_is_absolutely_positioned: bool,
    ) -> ServoRestyleDamage {
        match (
            parent_is_absolutely_positioned,
            child_is_absolutely_positioned,
        ) {
            (false, true) => {
                // Absolute children are out-of-flow and therefore insulated from changes.
                //
                // FIXME(pcwalton): Au contraire, if the containing block dimensions change!
                self & (ServoRestyleDamage::REPAINT | ServoRestyleDamage::REPOSITION)
            },
            (true, false) => {
                // Changing the position of an absolutely-positioned block requires us to reflow
                // its kids.
                if self.contains(ServoRestyleDamage::REFLOW_OUT_OF_FLOW) {
                    self | ServoRestyleDamage::REFLOW
                } else {
                    self
                }
            },
            _ => {
                // TODO(pcwalton): Take floatedness into account.
                self & (ServoRestyleDamage::REPAINT | ServoRestyleDamage::REPOSITION |
                    ServoRestyleDamage::REFLOW)
            },
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

        let to_iter = [
            (ServoRestyleDamage::REPAINT, "Repaint"),
            (ServoRestyleDamage::REPOSITION, "Reposition"),
            (ServoRestyleDamage::STORE_OVERFLOW, "StoreOverflow"),
            (ServoRestyleDamage::BUBBLE_ISIZES, "BubbleISizes"),
            (ServoRestyleDamage::REFLOW_OUT_OF_FLOW, "ReflowOutOfFlow"),
            (ServoRestyleDamage::REFLOW, "Reflow"),
            (
                ServoRestyleDamage::RESOLVE_GENERATED_CONTENT,
                "ResolveGeneratedContent",
            ),
            (ServoRestyleDamage::RECONSTRUCT_FLOW, "ReconstructFlow"),
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

fn compute_damage(old: &ComputedValues, new: &ComputedValues) -> ServoRestyleDamage {
    let mut damage = ServoRestyleDamage::empty();

    // This should check every CSS property, as enumerated in the fields of
    // http://doc.servo.org/style/properties/struct.ComputedValues.html

    // This uses short-circuiting boolean OR for its side effects and ignores the result.
    let _ = restyle_damage_rebuild_and_reflow!(
        old,
        new,
        damage,
        [
            ServoRestyleDamage::REPAINT,
            ServoRestyleDamage::REPOSITION,
            ServoRestyleDamage::STORE_OVERFLOW,
            ServoRestyleDamage::BUBBLE_ISIZES,
            ServoRestyleDamage::REFLOW_OUT_OF_FLOW,
            ServoRestyleDamage::REFLOW,
            ServoRestyleDamage::RECONSTRUCT_FLOW
        ]
    ) ||
        (new.get_box().display == Display::Inline &&
            restyle_damage_rebuild_and_reflow_inline!(
                old,
                new,
                damage,
                [
                    ServoRestyleDamage::REPAINT,
                    ServoRestyleDamage::REPOSITION,
                    ServoRestyleDamage::STORE_OVERFLOW,
                    ServoRestyleDamage::BUBBLE_ISIZES,
                    ServoRestyleDamage::REFLOW_OUT_OF_FLOW,
                    ServoRestyleDamage::REFLOW,
                    ServoRestyleDamage::RECONSTRUCT_FLOW
                ]
            )) ||
        restyle_damage_reflow!(
            old,
            new,
            damage,
            [
                ServoRestyleDamage::REPAINT,
                ServoRestyleDamage::REPOSITION,
                ServoRestyleDamage::STORE_OVERFLOW,
                ServoRestyleDamage::BUBBLE_ISIZES,
                ServoRestyleDamage::REFLOW_OUT_OF_FLOW,
                ServoRestyleDamage::REFLOW
            ]
        ) ||
        restyle_damage_reflow_out_of_flow!(
            old,
            new,
            damage,
            [
                ServoRestyleDamage::REPAINT,
                ServoRestyleDamage::REPOSITION,
                ServoRestyleDamage::STORE_OVERFLOW,
                ServoRestyleDamage::REFLOW_OUT_OF_FLOW
            ]
        ) || restyle_damage_repaint!(old, new, damage, [ServoRestyleDamage::REPAINT]);

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
