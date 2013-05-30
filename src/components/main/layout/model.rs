/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Borders, padding, and margins.

use layout::display_list_builder::ToGfxColor;
use layout::box::RenderBox;

use core::cell::Cell;
use core::num::Zero;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use geom::side_offsets::SideOffsets2D;
use gfx::display_list::{BaseDisplayItem, BorderDisplayItem, BorderDisplayItemClass, DisplayList};
use gfx::geometry::Au;
use newcss::complete::CompleteStyle;
use newcss::units::{Em, Pt, Px};
use newcss::values::{CSSBorderWidth, CSSBorderWidthLength, CSSBorderWidthMedium};
use newcss::values::{CSSBorderWidthThick, CSSBorderWidthThin};

/// Encapsulates the borders, padding, and margins, which we collectively call the "box model".
pub struct BoxModel {
    border: SideOffsets2D<Au>,
    padding: SideOffsets2D<Au>,
    margin: SideOffsets2D<Au>,
}

impl Zero for BoxModel {
    fn zero() -> BoxModel {
        BoxModel {
            border: Zero::zero(),
            padding: Zero::zero(),
            margin: Zero::zero(),
        }
    }

    fn is_zero(&self) -> bool {
        self.padding.is_zero() && self.border.is_zero() && self.margin.is_zero()
    }
}

impl BoxModel {
    /// Populates the box model parameters from the given computed style.
    pub fn populate(&mut self, style: CompleteStyle) {
        // Populate the borders.
        self.border.top = self.compute_border_width(style.border_top_width());
        self.border.right = self.compute_border_width(style.border_right_width());
        self.border.bottom = self.compute_border_width(style.border_bottom_width());
        self.border.left = self.compute_border_width(style.border_left_width());

        // TODO(pcwalton): Padding, margins.
    }

    /// Helper function to compute the border width in app units from the CSS border width.
    fn compute_border_width(&self, width: CSSBorderWidth) -> Au {
        match width {
            CSSBorderWidthLength(Px(v)) |
            CSSBorderWidthLength(Em(v)) |
            CSSBorderWidthLength(Pt(v)) => {
                // FIXME(pcwalton): Handle `em` and `pt` correctly.
                Au::from_frac_px(v)
            }
            CSSBorderWidthThin => Au::from_px(1),
            CSSBorderWidthMedium => Au::from_px(5),
            CSSBorderWidthThick => Au::from_px(10),
        }
    }
}

//
// Painting
//

impl RenderBox {
    /// Adds the display items necessary to paint the borders of this render box to a display list
    /// if necessary.
    pub fn paint_borders_if_applicable(&self, list: &Cell<DisplayList>, abs_bounds: &Rect<Au>) {
        // Fast path.
        let border = do self.with_imm_base |base| {
            base.model.border
        };
        if border.is_zero() {
            return
        }

        // Are all the widths equal?
        //
        // FIXME(pcwalton): Obviously this is wrong.
        if [ border.top, border.right, border.bottom ].all(|a| *a == border.left) {
            let border_width = border.top;
            let bounds = Rect {
                origin: Point2D {
                    x: abs_bounds.origin.x - border_width / Au(2),
                    y: abs_bounds.origin.y - border_width / Au(2),
                },
                size: Size2D {
                    width: abs_bounds.size.width + border_width,
                    height: abs_bounds.size.height + border_width
                }
            };

            let top_color = self.style().border_top_color();
            let color = top_color.to_gfx_color(); // FIXME

            // Append the border to the display list.
            do list.with_mut_ref |list| {
                let border_display_item = ~BorderDisplayItem {
                    base: BaseDisplayItem {
                        bounds: bounds,
                    },
                    width: border_width,
                    color: color,
                };

                list.append_item(BorderDisplayItemClass(border_display_item))
            }
        } else {
            warn!("ignoring unimplemented border widths");
        }
    }

}

