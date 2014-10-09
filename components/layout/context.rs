/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data needed by the layout task.

use css::matching::{ApplicableDeclarationsCache, StyleSharingCandidateCache};

use geom::{Rect, Size2D};
use gfx::display_list::OpaqueNode;
use gfx::font_context::FontContext;
use gfx::font_cache_task::FontCacheTask;
use script::layout_interface::LayoutChan;
use script_traits::UntrustedNodeAddress;
use servo_msg::constellation_msg::ConstellationChan;
use servo_net::local_image_cache::LocalImageCache;
use servo_util::geometry::Au;
use servo_util::opts::Opts;
use sync::{Arc, Mutex};
use std::mem;
use style::Stylist;
use url::Url;

struct LocalLayoutContext {
    font_context: FontContext,
    applicable_declarations_cache: ApplicableDeclarationsCache,
    style_sharing_candidate_cache: StyleSharingCandidateCache,
}

local_data_key!(local_context_key: *mut LocalLayoutContext)

fn create_or_get_local_context(shared_layout_context: &SharedLayoutContext) -> *mut LocalLayoutContext {
    let maybe_context = local_context_key.get();

    let context = match maybe_context {
        None => {
            let context = box LocalLayoutContext {
                font_context: FontContext::new(shared_layout_context.font_cache_task.clone()),
                applicable_declarations_cache: ApplicableDeclarationsCache::new(),
                style_sharing_candidate_cache: StyleSharingCandidateCache::new(),
            };
            local_context_key.replace(Some(unsafe { mem::transmute(context) }));
            local_context_key.get().unwrap()
        },
        Some(context) => context
    };

    *context
}

pub struct SharedLayoutContext {
    /// The local image cache.
    pub image_cache: Arc<Mutex<LocalImageCache<UntrustedNodeAddress>>>,

    /// The current screen size.
    pub screen_size: Size2D<Au>,

    /// A channel up to the constellation.
    pub constellation_chan: ConstellationChan,

    /// A channel up to the layout task.
    pub layout_chan: LayoutChan,

    /// Interface to the font cache task.
    pub font_cache_task: FontCacheTask,

    /// The CSS selector stylist.
    ///
    /// FIXME(#2604): Make this no longer an unsafe pointer once we have fast `RWArc`s.
    pub stylist: *const Stylist,

    /// The root node at which we're starting the layout.
    pub reflow_root: OpaqueNode,

    /// The URL.
    pub url: Url,

    /// The command line options.
    pub opts: Opts,

    /// The dirty rectangle, used during display list building.
    pub dirty: Rect<Au>,

    /// Starts at zero, and increased by one every time a layout completes.
    /// This can be used to easily check for invalid stale data.
    pub generation: uint,
}

pub struct LayoutContext<'a> {
    pub shared: &'a SharedLayoutContext,
    cached_local_layout_context: *mut LocalLayoutContext,
}

impl<'a> LayoutContext<'a> {
    pub fn new(shared_layout_context: &'a SharedLayoutContext) -> LayoutContext<'a> {

        let local_context = create_or_get_local_context(shared_layout_context);

        LayoutContext {
            shared: shared_layout_context,
            cached_local_layout_context: local_context,
        }
    }

    #[inline(always)]
    pub fn font_context<'a>(&'a self) -> &'a mut FontContext {
        unsafe {
            let cached_context = &*self.cached_local_layout_context;
            mem::transmute(&cached_context.font_context)
        }
    }

    #[inline(always)]
    pub fn applicable_declarations_cache<'a>(&'a self) -> &'a mut ApplicableDeclarationsCache {
        unsafe {
            let cached_context = &*self.cached_local_layout_context;
            mem::transmute(&cached_context.applicable_declarations_cache)
        }
    }

    #[inline(always)]
    pub fn style_sharing_candidate_cache<'a>(&'a self) -> &'a mut StyleSharingCandidateCache {
        unsafe {
            let cached_context = &*self.cached_local_layout_context;
            mem::transmute(&cached_context.style_sharing_candidate_cache)
        }
    }
}
