/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data needed by the layout task.

use css::matching::{ApplicableDeclarationsCache, StyleSharingCandidateCache};

use geom::rect::Rect;
use geom::size::Size2D;
use gfx::display_list::OpaqueNode;
use gfx::font_context::{FontContext, FontContextInfo};
#[cfg(not(target_os="android"))]
use green::task::GreenTask;
use script::layout_interface::LayoutChan;
use servo_msg::constellation_msg::ConstellationChan;
use servo_net::image::holder::LocalImageCacheHandle;
use servo_util::geometry::Au;
use servo_util::opts::Opts;
use std::cast;
#[cfg(not(target_os="android"))]
use std::ptr;
#[cfg(not(target_os="android"))]
use std::rt::local::Local;
#[cfg(not(target_os="android"))]
use std::rt::task::Task;
use style::{ComputedValues, Stylist};
use sync::Arc;
use url::Url;

#[cfg(target_os="android")]
use std::local_data;

#[cfg(not(target_os="android"))]
#[thread_local]
static mut FONT_CONTEXT: *mut FontContext = 0 as *mut FontContext;

#[cfg(target_os="android")]
local_data_key!(font_context: *mut FontContext)

#[cfg(not(target_os="android"))]
#[thread_local]
static mut APPLICABLE_DECLARATIONS_CACHE: *mut ApplicableDeclarationsCache =
    0 as *mut ApplicableDeclarationsCache;

#[cfg(target_os="android")]
local_data_key!(applicable_declarations_cache: *mut ApplicableDeclarationsCache)

#[cfg(not(target_os="android"))]
#[thread_local]
static mut STYLE_SHARING_CANDIDATE_CACHE: *mut StyleSharingCandidateCache =
    0 as *mut StyleSharingCandidateCache;

#[cfg(target_os="android")]
local_data_key!(style_sharing_candidate_cache: *mut StyleSharingCandidateCache)

/// Data shared by all layout workers.
#[deriving(Clone)]
pub struct LayoutContext {
    /// The local image cache.
    pub image_cache: LocalImageCacheHandle,

    /// The current screen size.
    pub screen_size: Size2D<Au>,

    /// A channel up to the constellation.
    pub constellation_chan: ConstellationChan,

    /// A channel up to the layout task.
    pub layout_chan: LayoutChan,

    /// Information needed to construct a font context.
    pub font_context_info: FontContextInfo,

    /// The CSS selector stylist.
    ///
    /// FIXME(pcwalton): Make this no longer an unsafe pointer once we have fast `RWArc`s.
    pub stylist: *Stylist,

    /// The initial set of CSS properties.
    pub initial_css_values: Arc<ComputedValues>,

    /// The root node at which we're starting the layout.
    pub reflow_root: OpaqueNode,

    /// The URL.
    pub url: Url,

    /// The command line options.
    pub opts: Opts,

    /// The dirty rectangle, used during display list building.
    pub dirty: Rect<Au>,
}

#[cfg(not(target_os="android"))]
impl LayoutContext {
    pub fn font_context<'a>(&'a mut self) -> &'a mut FontContext {
        // Sanity check.
        {
            let mut task = Local::borrow(None::<Task>);
            match task.get().maybe_take_runtime::<GreenTask>() {
                Some(green) => {
                    task.get().put_runtime(green);
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
                    task.get().put_runtime(green);
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
                    task.get().put_runtime(green);
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


// On Android, we don't have the __tls_* functions emitted by rustc, so we
// need to use the slower local_data functions.
// Making matters worse, the local_data functions are very particular about
// enforcing the lifetimes associated with objects that they hold onto,
// which causes us some trouble we work around as below.
#[cfg(target_os="android")]
impl LayoutContext {
    pub fn font_context<'a>(&'a mut self) -> &'a mut FontContext {
        unsafe {
            let opt = local_data::pop(font_context);
            let mut context;
            match opt {
                Some(c) => context = cast::transmute(c),
                None => {
                    context = cast::transmute(~FontContext::new(self.font_context_info.clone()))
                }
            }
            local_data::set(font_context, context);
            cast::transmute(context)
        }
    }

    pub fn applicable_declarations_cache<'a>(&'a self) -> &'a mut ApplicableDeclarationsCache {
        unsafe {
            let opt = local_data::pop(applicable_declarations_cache);
            let mut cache;
            match opt {
                Some(c) => cache = cast::transmute(c),
                None => {
                    cache = cast::transmute(~ApplicableDeclarationsCache::new());
                }
            }
            local_data::set(applicable_declarations_cache, cache);
            cast::transmute(cache)
        }
    }

    pub fn style_sharing_candidate_cache<'a>(&'a self) -> &'a mut StyleSharingCandidateCache {
        unsafe {
            let opt = local_data::pop(style_sharing_candidate_cache);
            let mut cache;
            match opt {
                Some(c) => cache = cast::transmute(c),
                None => {
                    cache = cast::transmute(~StyleSharingCandidateCache::new());
                }
            }
            local_data::set(style_sharing_candidate_cache, cache);
            cast::transmute(cache)
        }
    }
}

