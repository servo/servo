/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use flow::{self, AFFECTS_COUNTERS, Flow, HAS_COUNTER_AFFECTING_CHILDREN, IS_ABSOLUTELY_POSITIONED};

use std::fmt;
use std::sync::Arc;
use style::computed_values::float;
use style::properties::ComputedValues;

bitflags! {
    #[doc = "Individual layout actions that may be necessary after restyling."]
    flags RestyleDamage: u8 {
        #[doc = "Repaint the node itself."]
        #[doc = "Currently unused; need to decide how this propagates."]
        const REPAINT = 0x01,

        #[doc = "Recompute intrinsic inline_sizes (minimum and preferred)."]
        #[doc = "Propagates down the flow tree because the computation is"]
        #[doc = "bottom-up."]
        const BUBBLE_ISIZES = 0x02,

        #[doc = "Recompute actual inline-sizes and block-sizes, only taking out-of-flow children \
                 into account. \
                 Propagates up the flow tree because the computation is top-down."]
        const REFLOW_OUT_OF_FLOW = 0x04,

        #[doc = "Recompute actual inline_sizes and block_sizes."]
        #[doc = "Propagates up the flow tree because the computation is"]
        #[doc = "top-down."]
        const REFLOW = 0x08,

        #[doc = "Re-resolve generated content. \
                 Propagates up the flow tree because the computation is inorder."]
        const RESOLVE_GENERATED_CONTENT = 0x10,

        #[doc = "The entire flow needs to be reconstructed."]
        const RECONSTRUCT_FLOW = 0x20
    }
}

bitflags! {
    flags SpecialRestyleDamage: u8 {
        #[doc="If this flag is set, we need to reflow the entire document. This is more or less a \
               temporary hack to deal with cases that we don't handle incrementally yet."]
        const REFLOW_ENTIRE_DOCUMENT = 0x01,
    }
}

impl RestyleDamage {
    /// Supposing a flow has the given `position` property and this damage, returns the damage that
    /// we should add to the *parent* of this flow.
    pub fn damage_for_parent(self, child_is_absolutely_positioned: bool) -> RestyleDamage {
        if child_is_absolutely_positioned {
            self & (REPAINT | REFLOW_OUT_OF_FLOW | RESOLVE_GENERATED_CONTENT)
        } else {
            self & (REPAINT | REFLOW | REFLOW_OUT_OF_FLOW | RESOLVE_GENERATED_CONTENT)
        }
    }

    /// Supposing the *parent* of a flow with the given `position` property has this damage,
    /// returns the damage that we should add to this flow.
    pub fn damage_for_child(self,
                            parent_is_absolutely_positioned: bool,
                            child_is_absolutely_positioned: bool)
                            -> RestyleDamage {
        match (parent_is_absolutely_positioned, child_is_absolutely_positioned) {
            (false, true) => {
                // Absolute children are out-of-flow and therefore insulated from changes.
                //
                // FIXME(pcwalton): Au contraire, if the containing block dimensions change!
                self & REPAINT
            }
            (true, false) => {
                // Changing the position of an absolutely-positioned block requires us to reflow
                // its kids.
                if self.contains(REFLOW_OUT_OF_FLOW) {
                    self | REFLOW
                } else {
                    self
                }
            }
            _ => {
                // TODO(pcwalton): Take floatedness into account.
                self & (REPAINT | REFLOW)
            }
        }
    }
}

impl fmt::Display for RestyleDamage {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut first_elem = true;

        let to_iter =
            [ (REPAINT, "Repaint")
            , (BUBBLE_ISIZES, "BubbleISizes")
            , (REFLOW_OUT_OF_FLOW, "ReflowOutOfFlow")
            , (REFLOW, "Reflow")
            , (RESOLVE_GENERATED_CONTENT, "ResolveGeneratedContent")
            , (RECONSTRUCT_FLOW, "ReconstructFlow")
            ];

        for &(damage, damage_str) in &to_iter {
            if self.contains(damage) {
                if !first_elem { try!(write!(f, " | ")); }
                try!(write!(f, "{}", damage_str));
                first_elem = false;
            }
        }

        if first_elem {
            try!(write!(f, "NoDamage"));
        }

        Ok(())
    }
}

// NB: We need the braces inside the RHS due to Rust #8012.  This particular
// version of this macro might be safe anyway, but we want to avoid silent
// breakage on modifications.
macro_rules! add_if_not_equal(
    ($old:ident, $new:ident, $damage:ident,
     [ $($effect:ident),* ], [ $($style_struct_getter:ident.$name:ident),* ]) => ({
        if $( ($old.$style_struct_getter().$name != $new.$style_struct_getter().$name) )||* {
            $damage.insert($($effect)|*);
        }
    })
);

/// Returns a bitmask that represents a flow that needs to be rebuilt and reflowed.
///
/// Use this instead of `RestyleDamage::all()` because `RestyleDamage::all()` will result in
/// unnecessary sequential resolution of generated content.
pub fn rebuild_and_reflow() -> RestyleDamage {
    REPAINT | BUBBLE_ISIZES | REFLOW_OUT_OF_FLOW | REFLOW | RECONSTRUCT_FLOW
}

pub fn compute_damage(old: &Option<Arc<ComputedValues>>, new: &ComputedValues) -> RestyleDamage {
    let old: &ComputedValues =
        match old.as_ref() {
            None => return rebuild_and_reflow(),
            Some(cv) => &**cv,
        };

    let mut damage = RestyleDamage::empty();

    // This checks every CSS property, as enumerated in
    // impl<'self> CssComputedStyle<'self>
    // in src/support/netsurfcss/rust-netsurfcss/netsurfcss.rc.

    // FIXME: We can short-circuit more of this.

    add_if_not_equal!(old, new, damage,
                      [ REPAINT ], [
        get_color.color, get_background.background_color,
        get_border.border_top_color, get_border.border_right_color,
        get_border.border_bottom_color, get_border.border_left_color
    ]);

    add_if_not_equal!(old, new, damage,
                      [ REPAINT, REFLOW_OUT_OF_FLOW ], [
        get_positionoffsets.top, get_positionoffsets.left,
        get_positionoffsets.right, get_positionoffsets.bottom
    ]);

    add_if_not_equal!(old, new, damage,
                      [ REPAINT, BUBBLE_ISIZES, REFLOW_OUT_OF_FLOW, REFLOW ], [
        get_border.border_top_width, get_border.border_right_width,
        get_border.border_bottom_width, get_border.border_left_width,
        get_margin.margin_top, get_margin.margin_right,
        get_margin.margin_bottom, get_margin.margin_left,
        get_padding.padding_top, get_padding.padding_right,
        get_padding.padding_bottom, get_padding.padding_left,
        get_box.width, get_box.height,
        get_font.font_family, get_font.font_size, get_font.font_style, get_font.font_weight,
        get_inheritedtext.text_align, get_text.text_decoration, get_inheritedbox.line_height
    ]);

    add_if_not_equal!(old, new, damage,
                      [ REPAINT, BUBBLE_ISIZES, REFLOW_OUT_OF_FLOW, REFLOW, RECONSTRUCT_FLOW ],
                      [ get_box.float, get_box.display, get_box.position ]);

    // FIXME: test somehow that we checked every CSS property
    damage
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
            for kid in self_base.children.iter_mut() {
                let child_is_absolutely_positioned =
                    flow::base(kid).flags.contains(IS_ABSOLUTELY_POSITIONED);
                flow::mut_base(kid).restyle_damage
                                   .insert(self_base.restyle_damage.damage_for_child(
                                            is_absolutely_positioned,
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
        self_base.restyle_damage.insert(rebuild_and_reflow());
        self_base.restyle_damage.remove(RECONSTRUCT_FLOW);
        for kid in self_base.children.iter_mut() {
            kid.reflow_entire_document();
        }
    }
}
