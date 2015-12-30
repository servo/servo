/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data needed by the layout task.

// for thread_local
#![allow(unsafe_code)]

use app_units::Au;
use canvas_traits::CanvasMsg;
use euclid::Rect;
use fnv::FnvHasher;
use gfx::font_cache_task::FontCacheTask;
use gfx::font_context::FontContext;
use gfx_traits::LayerId;
use ipc_channel::ipc::{self, IpcSender};
use net_traits::image::base::Image;
use net_traits::image_cache_task::{ImageCacheChan, ImageCacheTask, ImageResponse, ImageState};
use net_traits::image_cache_task::{UsePlaceholder};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::collections::hash_state::DefaultState;
use std::rc::Rc;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use style::context::{LocalStyleContext, SharedStyleContext, StyleContext};
use style::matching::{ApplicableDeclarationsCache, StyleSharingCandidateCache};
use url::Url;
use util::mem::HeapSizeOf;
use util::opts;

struct LocalLayoutContext {
    style_context: LocalStyleContext,
    font_context: RefCell<FontContext>,
}

impl HeapSizeOf for LocalLayoutContext {
    // FIXME(njn): measure other fields eventually.
    fn heap_size_of_children(&self) -> usize {
        self.font_context.heap_size_of_children()
    }
}

thread_local!(static LOCAL_CONTEXT_KEY: RefCell<Option<Rc<LocalLayoutContext>>> = RefCell::new(None));

pub fn heap_size_of_local_context() -> usize {
    LOCAL_CONTEXT_KEY.with(|r| {
        r.borrow().clone().map_or(0, |context| context.heap_size_of_children())
    })
}

fn create_or_get_local_context(shared_layout_context: &SharedLayoutContext)
                               -> Rc<LocalLayoutContext> {
    LOCAL_CONTEXT_KEY.with(|r| {
        let mut r = r.borrow_mut();
        if let Some(context) = r.clone() {
            if shared_layout_context.style_context.screen_size_changed {
                context.style_context.applicable_declarations_cache.borrow_mut().evict_all();
            }
            context
        } else {
            let font_cache_task = shared_layout_context.font_cache_task.lock().unwrap().clone();
            let context = Rc::new(LocalLayoutContext {
                style_context: LocalStyleContext {
                    applicable_declarations_cache: RefCell::new(ApplicableDeclarationsCache::new()),
                    style_sharing_candidate_cache: RefCell::new(StyleSharingCandidateCache::new()),
                },
                font_context: RefCell::new(FontContext::new(font_cache_task)),
            });
            *r = Some(context.clone());
            context
        }
    })
}

/// Layout information shared among all workers. This must be thread-safe.
pub struct SharedLayoutContext {
    /// Bits shared by the layout and style system.
    pub style_context: SharedStyleContext,

    /// The shared image cache task.
    pub image_cache_task: ImageCacheTask,

    /// A channel for the image cache to send responses to.
    pub image_cache_sender: Mutex<ImageCacheChan>,

    /// Interface to the font cache task.
    pub font_cache_task: Mutex<FontCacheTask>,

    /// The URL.
    pub url: Url,

    /// A channel to send canvas renderers to paint task, in order to correctly paint the layers
    pub canvas_layers_sender: Mutex<Sender<(LayerId, IpcSender<CanvasMsg>)>>,

    /// The visible rects for each layer, as reported to us by the compositor.
    pub visible_rects: Arc<HashMap<LayerId, Rect<Au>, DefaultState<FnvHasher>>>,
}

pub struct LayoutContext<'a> {
    pub shared: &'a SharedLayoutContext,
    cached_local_layout_context: Rc<LocalLayoutContext>,
}

impl<'a> StyleContext<'a> for LayoutContext<'a> {
    fn shared_context(&self) -> &'a SharedStyleContext {
        &self.shared.style_context
    }

    fn local_context(&self) -> &LocalStyleContext {
        &self.cached_local_layout_context.style_context
    }
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
    pub fn font_context(&self) -> RefMut<FontContext> {
        self.cached_local_layout_context.font_context.borrow_mut()
    }

    #[inline(always)]
    pub fn applicable_declarations_cache(&self) -> RefMut<ApplicableDeclarationsCache> {
        self.local_context().applicable_declarations_cache.borrow_mut()
    }

    #[inline(always)]
    pub fn style_sharing_candidate_cache(&self) -> RefMut<StyleSharingCandidateCache> {
        self.local_context().style_sharing_candidate_cache.borrow_mut()
    }

    pub fn get_or_request_image(&self, url: Url, use_placeholder: UsePlaceholder)
                                -> Option<Arc<Image>> {
        // See if the image is already available
        let result = self.shared.image_cache_task.find_image(url.clone(),
                                                             use_placeholder);

        match result {
            Ok(image) => Some(image),
            Err(state) => {
                // If we are emitting an output file, then we need to block on
                // image load or we risk emitting an output file missing the image.
                let is_sync = opts::get().output_file.is_some() ||
                              opts::get().exit_after_load;

                match (state, is_sync) {
                    // Image failed to load, so just return nothing
                    (ImageState::LoadError, _) => None,
                    // Not loaded, test mode - load the image synchronously
                    (_, true) => {
                        let (sync_tx, sync_rx) = ipc::channel().unwrap();
                        self.shared.image_cache_task.request_image(url,
                                                                   ImageCacheChan(sync_tx),
                                                                   None);
                        match sync_rx.recv().unwrap().image_response {
                            ImageResponse::Loaded(image) |
                            ImageResponse::PlaceholderLoaded(image) => Some(image),
                            ImageResponse::None => None,
                        }
                    }
                    // Not yet requested, async mode - request image from the cache
                    (ImageState::NotRequested, false) => {
                        let sender = self.shared.image_cache_sender.lock().unwrap().clone();
                        self.shared.image_cache_task.request_image(url, sender, None);
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
