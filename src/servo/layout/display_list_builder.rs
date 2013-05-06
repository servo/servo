/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

///
/// Constructs display lists from render boxes.
///

use core::cell::Cell;

use layout::context::LayoutContext;
use layout::flow::FlowContext;

use geom::point::Point2D;
use geom::rect::Rect;
use gfx::display_list::DisplayList;
use gfx::geometry::Au;
use gfx;

/** A builder object that manages display list builder should mainly
 hold information about the initial request and desired result---for
 example, whether the DisplayList to be used for painting or hit
 testing. This can affect which boxes are created.

 Right now, the builder isn't used for much, but it  establishes the
 pattern we'll need once we support DL-based hit testing &c.  */
pub struct DisplayListBuilder<'self> {
    ctx:  &'self LayoutContext,
}

pub trait FlowDisplayListBuilderMethods {
    fn build_display_list(&self, a: &DisplayListBuilder, b: &Rect<Au>, c: &Cell<DisplayList>);
    fn build_display_list_for_child(&self,
                                    a: &DisplayListBuilder,
                                    b: FlowContext,
                                    c: &Rect<Au>,
                                    d: &Point2D<Au>,
                                    e: &Cell<DisplayList>);
}

impl FlowDisplayListBuilderMethods for FlowContext {
    fn build_display_list(&self,
                          builder: &DisplayListBuilder,
                          dirty: &Rect<Au>,
                          list: &Cell<DisplayList>) {
        let zero = gfx::geometry::zero_point();
        self.build_display_list_recurse(builder, dirty, &zero, list);
    }

    fn build_display_list_for_child(&self,
                                    builder: &DisplayListBuilder,
                                    child_flow: FlowContext,
                                    dirty: &Rect<Au>,
                                    offset: &Point2D<Au>,
                                    list: &Cell<DisplayList>) {
        // adjust the dirty rect to child flow context coordinates
        do child_flow.with_common_info |child_flow_info| {
            let abs_flow_bounds = child_flow_info.position.translate(offset);
            let adj_offset = offset.add(&child_flow_info.position.origin);

            debug!("build_display_list_for_child: rel=%?, abs=%?",
                   child_flow_info.position,
                   abs_flow_bounds);
            debug!("build_display_list_for_child: dirty=%?, offset=%?", dirty, offset);

            if dirty.intersects(&abs_flow_bounds) {
                debug!("build_display_list_for_child: intersected. recursing into child flow...");
                child_flow.build_display_list_recurse(builder, dirty, &adj_offset, list);
            } else {
                debug!("build_display_list_for_child: Did not intersect...");
            }
        }
    }
}

