/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data needed by the layout thread.

// for thread_local
#![allow(unsafe_code)]

use app_units::Au;
use canvas_traits::CanvasMsg;
use euclid::Rect;
use fnv::FnvHasher;
use gfx::font_cache_thread::FontCacheThread;
use gfx::font_context::FontContext;
use gfx_traits::LayerId;
use heapsize::HeapSizeOf;
use ipc_channel::ipc::{self, IpcSender};
use net_traits::image::base::Image;
use net_traits::image_cache_thread::{ImageCacheChan, ImageCacheThread, ImageResponse, ImageState};
use net_traits::image_cache_thread::{ImageOrMetadataAvailable, UsePlaceholder};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use style::context::{LocalStyleContext, StyleContext};
use style::matching::{ApplicableDeclarationsCache, StyleSharingCandidateCache};
use style::selector_impl::ServoSelectorImpl;
use style::servo::SharedStyleContext;
use url::Url;
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

// Keep this implementation in sync with the one in ports/geckolib/traversal.rs.
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
            let font_cache_thread = shared_layout_context.font_cache_thread.lock().unwrap().clone();
            let context = Rc::new(LocalLayoutContext {
                style_context: LocalStyleContext {
                    applicable_declarations_cache: RefCell::new(ApplicableDeclarationsCache::new()),
                    style_sharing_candidate_cache: RefCell::new(StyleSharingCandidateCache::new()),
                },
                font_context: RefCell::new(FontContext::new(font_cache_thread)),
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

    /// The shared image cache thread.
    pub image_cache_thread: ImageCacheThread,

    /// A channel for the image cache to send responses to.
    pub image_cache_sender: Mutex<ImageCacheChan>,

    /// Interface to the font cache thread.
    pub font_cache_thread: Mutex<FontCacheThread>,

    /// The URL.
    pub url: Url,

    /// A channel to send canvas renderers to paint thread, in order to correctly paint the layers
    pub canvas_layers_sender: Mutex<Sender<(LayerId, IpcSender<CanvasMsg>)>>,

    /// The visible rects for each layer, as reported to us by the compositor.
    pub visible_rects: Arc<HashMap<LayerId, Rect<Au>, BuildHasherDefault<FnvHasher>>>,
}

pub struct LayoutContext<'a> {
    pub shared: &'a SharedLayoutContext,
    cached_local_layout_context: Rc<LocalLayoutContext>,
}

impl<'a> StyleContext<'a, ServoSelectorImpl> for LayoutContext<'a> {
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

    pub fn get_or_request_image(&self, url: Url, use_placeholder: UsePlaceholder)
                                -> Option<Arc<Image>> {
        // See if the image is already available
        let result = self.shared.image_cache_thread.find_image(url.clone(),
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
                        self.shared.image_cache_thread.request_image(url,
                                                                   ImageCacheChan(sync_tx),
                                                                   None);
                        match sync_rx.recv().unwrap().image_response {
                            ImageResponse::Loaded(image) |
                            ImageResponse::PlaceholderLoaded(image) => Some(image),
                            ImageResponse::None | ImageResponse::MetadataLoaded(_) => None,
                        }
                    }
                    // Not yet requested, async mode - request image from the cache
                    (ImageState::NotRequested, false) => {
                        let sender = self.shared.image_cache_sender.lock().unwrap().clone();
                        self.shared.image_cache_thread.request_image(url, sender, None);
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

    pub fn get_or_request_image_or_meta(&self, url: Url, use_placeholder: UsePlaceholder)
                                -> Option<ImageOrMetadataAvailable> {
        // If we are emitting an output file, load the image synchronously.
        if opts::get().output_file.is_some() || opts::get().exit_after_load {
            return self.get_or_request_image(url, use_placeholder)
                       .map(|img| ImageOrMetadataAvailable::ImageAvailable(img));
        }
        // See if the image is already available
        let result = self.shared.image_cache_thread.find_image_or_metadata(url.clone(),
                                                                           use_placeholder);
        match result {
            Ok(image_or_metadata) => Some(image_or_metadata),
            // Image failed to load, so just return nothing
            Err(ImageState::LoadError) => None,
            // Not yet requested, async mode - request image or metadata from the cache
            Err(ImageState::NotRequested) => {
                let sender = self.shared.image_cache_sender.lock().unwrap().clone();
                self.shared.image_cache_thread.request_image_and_metadata(url, sender, None);
                None
            }
            // Image has been requested, is still pending. Return no image for this paint loop.
            // When the image loads it will trigger a reflow and/or repaint.
            Err(ImageState::Pending) => None,
        }
    }

}
