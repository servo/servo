/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::sync::Arc;
use style::ComputedValues;

bitflags! {
    #[doc = "Individual layout actions that may be necessary after restyling."]
    flags RestyleDamage: int {
        #[doc = "Repaint the node itself."]
        #[doc = "Currently unused; need to decide how this propagates."]
        static Repaint = 0x01,

        #[doc = "Recompute intrinsic inline_sizes (minimum and preferred)."]
        #[doc = "Propagates down the flow tree because the computation is"]
        #[doc = "bottom-up."]
        static BubbleISizes = 0x02,

        #[doc = "Recompute actual inline_sizes and block_sizes."]
        #[doc = "Propagates up the flow tree because the computation is"]
        #[doc = "top-down."]
        static Reflow = 0x04
    }
}

impl RestyleDamage {
    /// Elements of self which should also get set on any ancestor flow.
    pub fn propagate_up(self) -> RestyleDamage {
        self & Reflow
    }

    /// Elements of self which should also get set on any child flows.
    pub fn propagate_down(self) -> RestyleDamage {
        self & BubbleISizes
    }
}

impl fmt::Show for RestyleDamage {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::FormatError> {
        let mut first_elem = true;

        let to_iter =
            [ (Repaint,      "Repaint")
            , (BubbleISizes, "BubbleISizes")
            , (Reflow,       "Reflow")
            ];

        for &(damage, damage_str) in to_iter.iter() {
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
)

pub fn compute_damage(old: &Option<Arc<ComputedValues>>, new: &ComputedValues) -> RestyleDamage {
    let old: &ComputedValues =
        match old.as_ref() {
            None => return Repaint | BubbleISizes | Reflow,
            Some(cv) => &**cv,
        };

    let mut damage = RestyleDamage::empty();

    // This checks every CSS property, as enumerated in
    // impl<'self> CssComputedStyle<'self>
    // in src/support/netsurfcss/rust-netsurfcss/netsurfcss.rc.

    // FIXME: We can short-circuit more of this.

    add_if_not_equal!(old, new, damage, [ Repaint ],
        [ get_color.color, get_background.background_color,
          get_border.border_top_color, get_border.border_right_color,
          get_border.border_bottom_color, get_border.border_left_color ]);

    add_if_not_equal!(old, new, damage, [ Repaint, BubbleISizes, Reflow ],
        [ get_border.border_top_width, get_border.border_right_width,
          get_border.border_bottom_width, get_border.border_left_width,
          get_margin.margin_top, get_margin.margin_right,
          get_margin.margin_bottom, get_margin.margin_left,
          get_padding.padding_top, get_padding.padding_right,
          get_padding.padding_bottom, get_padding.padding_left,
          get_box.position, get_box.width, get_box.height, get_box.float, get_box.display,
          get_font.font_family, get_font.font_size, get_font.font_style, get_font.font_weight,
          get_inheritedtext.text_align, get_text.text_decoration, get_inheritedbox.line_height ]);

    // FIXME: test somehow that we checked every CSS property

    damage
}
