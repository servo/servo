/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data needed by the layout task.

#![allow(unsafe_code)]

use css::matching::{ApplicableDeclarationsCache, StyleSharingCandidateCache};

use geom::{Rect, Size2D};
use gfx::display_list::OpaqueNode;
use gfx::font_cache_task::FontCacheTask;
use gfx::font_context::FontContext;
use msg::constellation_msg::ConstellationChan;
use net_traits::image::base::Image;
use net_traits::image_cache_task::{ImageCacheChan, ImageCacheTask, ImageState};
use script::layout_interface::{Animation, LayoutChan, ReflowGoal};
use std::boxed;
use std::cell::Cell;
use std::ptr;
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender};
use style::selector_matching::Stylist;
use url::Url;
use util::geometry::Au;
use util::opts;

struct LocalLayoutContext {
    font_context: FontContext,
    applicable_declarations_cache: ApplicableDeclarationsCache,
    style_sharing_candidate_cache: StyleSharingCandidateCache,
}

thread_local!(static LOCAL_CONTEXT_KEY: Cell<*mut LocalLayoutContext> = Cell::new(ptr::null_mut()));

fn create_or_get_local_context(shared_layout_context: &SharedLayoutContext)
                               -> *mut LocalLayoutContext {
    LOCAL_CONTEXT_KEY.with(|ref r| {
        if r.get().is_null() {
            let context = box LocalLayoutContext {
                font_context: FontContext::new(shared_layout_context.font_cache_task.clone()),
                applicable_declarations_cache: ApplicableDeclarationsCache::new(),
                style_sharing_candidate_cache: StyleSharingCandidateCache::new(),
            };
            r.set(unsafe { boxed::into_raw(context) });
        } else if shared_layout_context.screen_size_changed {
            unsafe {
                (*r.get()).applicable_declarations_cache.evict_all();
            }
        }

        r.get()
    })
}

/// Layout information shared among all workers. This must be thread-safe.
pub struct SharedLayoutContext {
    /// The shared image cache task.
    pub image_cache_task: ImageCacheTask,

    /// A channel for the image cache to send responses to.
    pub image_cache_sender: ImageCacheChan,

    /// The current screen size.
    pub screen_size: Size2D<Au>,

    /// Screen sized changed?
    pub screen_size_changed: bool,

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
    pub reflow_root: Option<OpaqueNode>,

    /// The URL.
    pub url: Url,

    /// The dirty rectangle, used during display list building.
    pub dirty: Rect<Au>,

    /// Starts at zero, and increased by one every time a layout completes.
    /// This can be used to easily check for invalid stale data.
    pub generation: u32,

    /// A channel on which new animations that have been triggered by style recalculation can be
    /// sent.
    pub new_animations_sender: Sender<Animation>,

    /// Why is this reflow occurring
    pub goal: ReflowGoal,
}

pub struct SharedLayoutContextWrapper(pub *const SharedLayoutContext);

unsafe impl Send for SharedLayoutContextWrapper {}

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
    pub fn font_context<'b>(&'b self) -> &'b mut FontContext {
        unsafe {
            let cached_context = &mut *self.cached_local_layout_context;
            &mut cached_context.font_context
        }
    }

    #[inline(always)]
    pub fn applicable_declarations_cache<'b>(&'b self) -> &'b mut ApplicableDeclarationsCache {
        unsafe {
            let cached_context = &mut *self.cached_local_layout_context;
            &mut cached_context.applicable_declarations_cache
        }
    }

    #[inline(always)]
    pub fn style_sharing_candidate_cache<'b>(&'b self) -> &'b mut StyleSharingCandidateCache {
        unsafe {
            let cached_context = &mut *self.cached_local_layout_context;
            &mut cached_context.style_sharing_candidate_cache
        }
    }

    pub fn get_or_request_image(&self, url: Url) -> Option<Arc<Image>> {
        // See if the image is already available
        let result = self.shared.image_cache_task.get_image_if_available(url.clone());

        match result {
            Ok(image) => Some(image),
            Err(state) => {
                // If we are emitting an output file, then we need to block on
                // image load or we risk emitting an output file missing the image.
                let is_sync = opts::get().output_file.is_some();

                match (state, is_sync) {
                    // Image failed to load, so just return nothing
                    (ImageState::LoadError, _) => None,
                    // Not loaded, test mode - load the image synchronously
                    (_, true) => {
                        let (sync_tx, sync_rx) = channel();
                        self.shared.image_cache_task.request_image(url,
                                                                   ImageCacheChan(sync_tx),
                                                                   None);
                        sync_rx.recv().unwrap().image
                    }
                    // Not yet requested, async mode - request image from the cache
                    (ImageState::NotRequested, false) => {
                        self.shared.image_cache_task.request_image(url,
                                                                   self.shared.image_cache_sender.clone(),
                                                                   None);
                        None
                    }
                    // Image has been requested, is still pending. Return no image
                    // for this paint loop. When the image loads it will trigger
                    // a reflow and/or repaint.
                    (ImageState::Pending, false) => None,
                }
            }
        }
    }
}
