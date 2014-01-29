/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data needed by the layout task.

use extra::arc::MutexArc;
use green::task::GreenTask;
use layout::flow::FlowLeafSet;
use layout::util::OpaqueNode;
use layout::wrapper::DomLeafSet;
use std::cast;
use std::ptr;
use std::rt::Runtime;
use std::rt::local::Local;
use std::rt::task::Task;

use geom::size::Size2D;
use gfx::font_context::{FontContext, FontContextInfo};
use script::layout_interface::LayoutChan;
use servo_msg::constellation_msg::ConstellationChan;
use servo_net::local_image_cache::LocalImageCache;
use servo_util::geometry::Au;
use style::Stylist;

#[thread_local]
static mut FONT_CONTEXT: *mut FontContext = 0 as *mut FontContext;

/// Data shared by all layout workers.
#[deriving(Clone)]
pub struct LayoutContext {
    /// The local image cache.
    image_cache: MutexArc<LocalImageCache>,

    /// The current screen size.
    screen_size: Size2D<Au>,

    /// A channel up to the constellation.
    constellation_chan: ConstellationChan,

    /// The set of leaf DOM nodes.
    dom_leaf_set: MutexArc<DomLeafSet>,

    /// A channel up to the layout task.
    layout_chan: LayoutChan,

    /// The set of leaf flows.
    flow_leaf_set: MutexArc<FlowLeafSet>,

    /// Information needed to construct a font context.
    font_context_info: FontContextInfo,

    /// The CSS selector stylist.
    ///
    /// FIXME(pcwalton): Make this no longer an unsafe pointer once we have fast `RWArc`s.
    stylist: *Stylist,

    /// The root node at which we're starting the layout.
    reflow_root: OpaqueNode,
}

impl LayoutContext {
    pub fn font_context<'a>(&'a mut self) -> &'a mut FontContext {
        // Sanity check.
        let mut task = Local::borrow(None::<Task>);
        match task.get().maybe_take_runtime::<GreenTask>() {
            Some(green) => {
                task.get().put_runtime(green as ~Runtime);
                fail!("can't call this on a green task!")
            }
            None => {}
        }

        unsafe {
            if FONT_CONTEXT == ptr::mut_null() {
                let context = ~FontContext::new(self.font_context_info.clone());
                FONT_CONTEXT = cast::transmute(context)
            }
            cast::transmute(FONT_CONTEXT)
        }
    }
}

