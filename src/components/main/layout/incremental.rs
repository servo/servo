/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use style::ComputedValues;

/// Individual layout actions that may be necessary after restyling.
///
/// If you add to this enum, also add the value to RestyleDamage::all below.
/// (FIXME: do this automatically)
pub enum RestyleEffect {
    /// Repaint the node itself.
    /// Currently unused; need to decide how this propagates.
    Repaint = 0x01,

    /// Recompute intrinsic widths (minimum and preferred).
    /// Propagates down the flow tree because the computation is
    /// bottom-up.
    BubbleWidths = 0x02,

    /// Recompute actual widths and heights.
    /// Propagates up the flow tree because the computation is
    /// top-down.
    Reflow = 0x04,
}

/// A set of RestyleEffects.
// FIXME: Switch to librustc/util/enum_set.rs if that gets moved into
// libextra (Rust #8054)
pub struct RestyleDamage {
    priv bits: int
}

// Provide literal syntax of the form restyle_damage!(Repaint, Reflow)
macro_rules! restyle_damage(
    ( $($damage:ident),* ) => (
        RestyleDamage::none() $( .add($damage) )*
    )
)

impl RestyleDamage {
    pub fn none() -> RestyleDamage {
        RestyleDamage { bits: 0 }
    }

    pub fn all() -> RestyleDamage {
        restyle_damage!(Repaint, BubbleWidths, Reflow)
    }

    /// Create a RestyleDamage from the underlying bit field.
    /// We would rather not allow this, but some types in script
    /// need to store RestyleDamage without depending on this crate.
    pub fn from_int(n: int) -> RestyleDamage {
        RestyleDamage { bits: n }
    }

    pub fn to_int(self) -> int {
        self.bits
    }

    pub fn is_empty(self) -> bool {
        self.bits == 0
    }

    pub fn is_nonempty(self) -> bool {
        self.bits != 0
    }

    pub fn add(self, effect: RestyleEffect) -> RestyleDamage {
        RestyleDamage { bits: self.bits | (effect as int) }
    }

    pub fn has(self, effect: RestyleEffect) -> bool {
        (self.bits & (effect as int)) != 0
    }

    pub fn lacks(self, effect: RestyleEffect) -> bool {
        (self.bits & (effect as int)) == 0
    }

    pub fn union(self, other: RestyleDamage) -> RestyleDamage {
        RestyleDamage { bits: self.bits | other.bits }
    }

    pub fn union_in_place(&mut self, other: RestyleDamage) {
        self.bits = self.bits | other.bits;
    }

    pub fn intersect(self, other: RestyleDamage) -> RestyleDamage {
        RestyleDamage { bits: self.bits & other.bits }
    }

    /// Elements of self which should also get set on any ancestor flow.
    pub fn propagate_up(self) -> RestyleDamage {
        self.intersect(restyle_damage!(Reflow))
    }

    /// Elements of self which should also get set on any child flows.
    pub fn propagate_down(self) -> RestyleDamage {
        self.intersect(restyle_damage!(BubbleWidths))
    }
}

// NB: We need the braces inside the RHS due to Rust #8012.  This particular
// version of this macro might be safe anyway, but we want to avoid silent
// breakage on modifications.
macro_rules! add_if_not_equal(
    ($old:ident, $new:ident, $damage:ident,
     [ $($effect:ident),* ], [ $($style_struct:ident.$name:ident),* ]) => ({
        if $( ($old.$style_struct.$name != $new.$style_struct.$name) )||* {
            $damage.union_in_place( restyle_damage!( $($effect),* ) );
        }
    })
)

pub fn compute_damage(old: &ComputedValues, new: &ComputedValues) -> RestyleDamage {
    let mut damage = RestyleDamage::none();

    // This checks every CSS property, as enumerated in
    // impl<'self> CssComputedStyle<'self>
    // in src/support/netsurfcss/rust-netsurfcss/netsurfcss.rc.

    // FIXME: We can short-circuit more of this.

    add_if_not_equal!(old, new, damage, [ Repaint ],
        [ Color.color, Background.background_color,
          Border.border_top_color, Border.border_right_color,
          Border.border_bottom_color, Border.border_left_color ]);

    add_if_not_equal!(old, new, damage, [ Repaint, BubbleWidths, Reflow ],
        [ Border.border_top_width, Border.border_right_width,
          Border.border_bottom_width, Border.border_left_width,
          Margin.margin_top, Margin.margin_right, Margin.margin_bottom, Margin.margin_left,
          Padding.padding_top, Padding.padding_right, Padding.padding_bottom, Padding.padding_left,
          Box.position, Box.width, Box.height, Box.float, Box.display,
          Font.font_family, Font.font_size, Font.font_style, Font.font_weight,
          Text.text_align, Text.text_decoration, Box.line_height ]);

    // FIXME: test somehow that we checked every CSS property

    damage
}


#[cfg(test)]
mod restyle_damage_tests {
    use super::*;

    #[test]
    fn none_is_empty() {
        let d = RestyleDamage::none();
        assert!(!d.has(Repaint));
        assert!(!d.has(BubbleWidths));
        assert!(d.lacks(Repaint));
        assert!(d.lacks(BubbleWidths));
    }

    #[test]
    fn all_is_full() {
        let d = RestyleDamage::all();
        assert!(d.has(Repaint));
        assert!(d.has(BubbleWidths));
        assert!(!d.lacks(Repaint));
        assert!(!d.lacks(BubbleWidths));
    }

    #[test]
    fn can_add() {
        assert!(RestyleDamage::none().add(BubbleWidths).has(BubbleWidths));
    }

    #[test]
    fn can_union() {
        let d = restyle_damage!(Repaint).union(restyle_damage!(BubbleWidths));
        assert!(d.has(Repaint));
        assert!(d.has(BubbleWidths));
    }

    #[test]
    fn can_union_in_place() {
        let mut d = restyle_damage!(Repaint);
        d.union_in_place(restyle_damage!(BubbleWidths));
        assert!(d.has(Repaint));
        assert!(d.has(BubbleWidths));
    }

    #[test]
    fn can_intersect() {
        let x = restyle_damage!(Repaint, BubbleWidths);
        let y = restyle_damage!(Repaint, Reflow);
        let d = x.intersect(y);
        assert!(d.has(Repaint));
        assert!(d.lacks(BubbleWidths));
        assert!(d.lacks(Reflow));
    }
}
