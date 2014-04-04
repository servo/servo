/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data needed by the layout task.

use css::matching::{ApplicableDeclarationsCache, StyleSharingCandidateCache};

use geom::size::Size2D;
use gfx::display_list::OpaqueNode;
use gfx::font_context::{FontContext, FontContextInfo};
use green::task::GreenTask;
use script::layout_interface::LayoutChan;
use servo_msg::constellation_msg::ConstellationChan;
use servo_net::local_image_cache::LocalImageCache;
use servo_util::geometry::Au;
use servo_util::opts::Opts;
use std::cast;
use std::ptr;
use std::rt::Runtime;
use std::rt::local::Local;
use std::rt::task::Task;
use style::{ComputedValues, Stylist};
use sync::{Arc, MutexArc};
use url::Url;

#[thread_local]
static mut FONT_CONTEXT: *mut FontContext = 0 as *mut FontContext;

#[thread_local]
static mut APPLICABLE_DECLARATIONS_CACHE: *mut ApplicableDeclarationsCache =
    0 as *mut ApplicableDeclarationsCache;

#[thread_local]
static mut STYLE_SHARING_CANDIDATE_CACHE: *mut StyleSharingCandidateCache =
    0 as *mut StyleSharingCandidateCache;

/// Data shared by all layout workers.
#[deriving(Clone)]
pub struct LayoutContext {
    /// The local image cache.
    image_cache: MutexArc<LocalImageCache>,

    /// The current screen size.
    screen_size: Size2D<Au>,

    /// A channel up to the constellation.
    constellation_chan: ConstellationChan,

    /// A channel up to the layout task.
    layout_chan: LayoutChan,

    /// Information needed to construct a font context.
    font_context_info: FontContextInfo,

    /// The CSS selector stylist.
    ///
    /// FIXME(pcwalton): Make this no longer an unsafe pointer once we have fast `RWArc`s.
    stylist: *Stylist,

    /// The initial set of CSS properties.
    initial_css_values: Arc<ComputedValues>,

    /// The root node at which we're starting the layout.
    reflow_root: OpaqueNode,

    /// The URL.
    url: Url,

    /// The command line options.
    opts: Opts,
}

impl LayoutContext {
    pub fn font_context<'a>(&'a mut self) -> &'a mut FontContext {
        // Sanity check.
        {
            let mut task = Local::borrow(None::<Task>);
            match task.get().maybe_take_runtime::<GreenTask>() {
                Some(green) => {
                    task.get().put_runtime(green as ~Runtime);
                    fail!("can't call this on a green task!")
                }
                None => {}
            }
        }

        unsafe {
            if FONT_CONTEXT == ptr::mut_null() {
                let context = ~FontContext::new(self.font_context_info.clone());
                FONT_CONTEXT = cast::transmute(context)
            }
            cast::transmute(FONT_CONTEXT)
        }
    }

    pub fn applicable_declarations_cache<'a>(&'a self) -> &'a mut ApplicableDeclarationsCache {
        // Sanity check.
        {
            let mut task = Local::borrow(None::<Task>);
            match task.get().maybe_take_runtime::<GreenTask>() {
                Some(green) => {
                    task.get().put_runtime(green as ~Runtime);
                    fail!("can't call this on a green task!")
                }
                None => {}
            }
        }

        unsafe {
            if APPLICABLE_DECLARATIONS_CACHE == ptr::mut_null() {
                let cache = ~ApplicableDeclarationsCache::new();
                APPLICABLE_DECLARATIONS_CACHE = cast::transmute(cache)
            }
            cast::transmute(APPLICABLE_DECLARATIONS_CACHE)
        }
    }

    pub fn style_sharing_candidate_cache<'a>(&'a self) -> &'a mut StyleSharingCandidateCache {
        // Sanity check.
        {
            let mut task = Local::borrow(None::<Task>);
            match task.get().maybe_take_runtime::<GreenTask>() {
                Some(green) => {
                    task.get().put_runtime(green as ~Runtime);
                    fail!("can't call this on a green task!")
                }
                None => {}
            }
        }

        unsafe {
            if STYLE_SHARING_CANDIDATE_CACHE == ptr::mut_null() {
                let cache = ~StyleSharingCandidateCache::new();
                STYLE_SHARING_CANDIDATE_CACHE = cast::transmute(cache)
            }
            cast::transmute(STYLE_SHARING_CANDIDATE_CACHE)
        }
    }
}

