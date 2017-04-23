/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use flow::{self, AFFECTS_COUNTERS, Flow, HAS_COUNTER_AFFECTING_CHILDREN, IS_ABSOLUTELY_POSITIONED};
use style::computed_values::float;
use style::selector_parser::RestyleDamage;
use style::servo::restyle_damage::{REFLOW, RECONSTRUCT_FLOW};

/// Used in a flow traversal to indicate whether this re-layout should be incremental or not.
#[derive(Clone, Copy, PartialEq)]
pub enum RelayoutMode {
    Incremental,
    Force
}

bitflags! {
    pub flags SpecialRestyleDamage: u8 {
        #[doc = "If this flag is set, we need to reflow the entire document. This is more or less a \
                 temporary hack to deal with cases that we don't handle incrementally yet."]
        const REFLOW_ENTIRE_DOCUMENT = 0x01,
    }
}

pub trait LayoutDamageComputation {
    fn compute_layout_damage(self) -> SpecialRestyleDamage;
    fn reflow_entire_document(self);
}

impl<'a> LayoutDamageComputation for &'a mut Flow {
    fn compute_layout_damage(self) -> SpecialRestyleDamage {
        let mut special_damage = SpecialRestyleDamage::empty();
        let is_absolutely_positioned = flow::base(self).flags.contains(IS_ABSOLUTELY_POSITIONED);

        // In addition to damage, we use this phase to compute whether nodes affect CSS counters.
        let mut has_counter_affecting_children = false;

        {
            let self_base = flow::mut_base(self);
            // Take a snapshot of the parent damage before updating it with damage from children.
            let parent_damage = self_base.restyle_damage;

            for kid in self_base.children.iter_mut() {
                let child_is_absolutely_positioned =
                    flow::base(kid).flags.contains(IS_ABSOLUTELY_POSITIONED);
                flow::mut_base(kid).restyle_damage.insert(
                    parent_damage.damage_for_child(is_absolutely_positioned,
                                                   child_is_absolutely_positioned));
                {
                    let kid: &mut Flow = kid;
                    special_damage.insert(kid.compute_layout_damage());
                }
                self_base.restyle_damage
                         .insert(flow::base(kid).restyle_damage.damage_for_parent(
                                 child_is_absolutely_positioned));

                has_counter_affecting_children = has_counter_affecting_children ||
                    flow::base(kid).flags.intersects(AFFECTS_COUNTERS |
                                                     HAS_COUNTER_AFFECTING_CHILDREN);
            }
        }

        let self_base = flow::mut_base(self);
        if self_base.flags.float_kind() != float::T::none &&
                self_base.restyle_damage.intersects(REFLOW) {
            special_damage.insert(REFLOW_ENTIRE_DOCUMENT);
        }

        if has_counter_affecting_children {
            self_base.flags.insert(HAS_COUNTER_AFFECTING_CHILDREN)
        } else {
            self_base.flags.remove(HAS_COUNTER_AFFECTING_CHILDREN)
        }

        special_damage
    }

    fn reflow_entire_document(self) {
        let self_base = flow::mut_base(self);
        self_base.restyle_damage.insert(RestyleDamage::rebuild_and_reflow());
        self_base.restyle_damage.remove(RECONSTRUCT_FLOW);
        for kid in self_base.children.iter_mut() {
            kid.reflow_entire_document();
        }
    }
}
