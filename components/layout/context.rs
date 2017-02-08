/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data needed by the layout thread.

use fnv::FnvHasher;
use gfx::display_list::WebRenderImageInfo;
use gfx::font_cache_thread::FontCacheThread;
use gfx::font_context::FontContext;
use heapsize::HeapSizeOf;
use ipc_channel::ipc;
use net_traits::image::base::Image;
use net_traits::image_cache_thread::{ImageCacheChan, ImageCacheThread, ImageResponse, ImageState};
use net_traits::image_cache_thread::{ImageOrMetadataAvailable, UsePlaceholder};
use parking_lot::RwLock;
use servo_config::opts;
use servo_url::ServoUrl;
use std::borrow::{Borrow, BorrowMut};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::sync::{Arc, Mutex};
use style::context::{SharedStyleContext, ThreadLocalStyleContext};
use style::dom::TElement;

/// TLS data scoped to the traversal.
pub struct ScopedThreadLocalLayoutContext<E: TElement> {
    pub style_context: ThreadLocalStyleContext<E>,
}

impl<E: TElement> ScopedThreadLocalLayoutContext<E> {
    pub fn new(context: &LayoutContext) -> Self {
        ScopedThreadLocalLayoutContext {
            style_context: ThreadLocalStyleContext::new(&context.style_context),
        }
    }
}

impl<E: TElement> Borrow<ThreadLocalStyleContext<E>> for ScopedThreadLocalLayoutContext<E> {
    fn borrow(&self) -> &ThreadLocalStyleContext<E> {
        &self.style_context
    }
}

impl<E: TElement> BorrowMut<ThreadLocalStyleContext<E>> for ScopedThreadLocalLayoutContext<E> {
    fn borrow_mut(&mut self) -> &mut ThreadLocalStyleContext<E> {
        &mut self.style_context
    }
}

thread_local!(static FONT_CONTEXT_KEY: RefCell<Option<FontContext>> = RefCell::new(None));

pub fn with_thread_local_font_context<F, R>(layout_context: &LayoutContext, f: F) -> R
    where F: FnOnce(&mut FontContext) -> R
{
    FONT_CONTEXT_KEY.with(|k| {
        let mut font_context = k.borrow_mut();
        if font_context.is_none() {
            let font_cache_thread = layout_context.font_cache_thread.lock().unwrap().clone();
            *font_context = Some(FontContext::new(font_cache_thread));
        }
        f(&mut RefMut::map(font_context, |x| x.as_mut().unwrap()))
    })
}

pub fn heap_size_of_persistent_local_context() -> usize {
    FONT_CONTEXT_KEY.with(|r| {
        if let Some(ref context) = *r.borrow() {
            context.heap_size_of_children()
        } else {
            0
        }
    })
}

/// Layout information shared among all workers. This must be thread-safe.
pub struct LayoutContext {
    /// Bits shared by the layout and style system.
    pub style_context: SharedStyleContext,

    /// The shared image cache thread.
    pub image_cache_thread: Mutex<ImageCacheThread>,

    /// A channel for the image cache to send responses to.
    pub image_cache_sender: Mutex<ImageCacheChan>,

    /// Interface to the font cache thread.
    pub font_cache_thread: Mutex<FontCacheThread>,

    /// A cache of WebRender image info.
    pub webrender_image_cache: Arc<RwLock<HashMap<(ServoUrl, UsePlaceholder),
                                                  WebRenderImageInfo,
                                                  BuildHasherDefault<FnvHasher>>>>,
}

impl LayoutContext {
    #[inline(always)]
    pub fn shared_context(&self) -> &SharedStyleContext {
        &self.style_context
    }

    fn get_or_request_image_synchronously(&self, url: ServoUrl, use_placeholder: UsePlaceholder)
                                          -> Option<Arc<Image>> {
        debug_assert!(opts::get().output_file.is_some() || opts::get().exit_after_load);

        // See if the image is already available
        let result = self.image_cache_thread.lock().unwrap()
                                            .find_image(url.clone(), use_placeholder);

        match result {
            Ok(image) => return Some(image),
            Err(ImageState::LoadError) => {
                // Image failed to load, so just return nothing
                return None
            }
            Err(_) => {}
        }

        // If we are emitting an output file, then we need to block on
        // image load or we risk emitting an output file missing the image.
        let (sync_tx, sync_rx) = ipc::channel().unwrap();
        self.image_cache_thread.lock().unwrap().request_image(url, ImageCacheChan(sync_tx), None);
        loop {
            match sync_rx.recv() {
                Err(_) => return None,
                Ok(response) => {
                    match response.image_response {
                        ImageResponse::Loaded(image) | ImageResponse::PlaceholderLoaded(image) => {
                            return Some(image)
                        }
                        ImageResponse::None | ImageResponse::MetadataLoaded(_) => {}
                    }
                }
            }
        }
    }

    pub fn get_or_request_image_or_meta(&self, url: ServoUrl, use_placeholder: UsePlaceholder)
                                -> Option<ImageOrMetadataAvailable> {
        // If we are emitting an output file, load the image synchronously.
        if opts::get().output_file.is_some() || opts::get().exit_after_load {
            return self.get_or_request_image_synchronously(url, use_placeholder)
                       .map(|img| ImageOrMetadataAvailable::ImageAvailable(img));
        }
        // See if the image is already available
        let result = self.image_cache_thread.lock().unwrap()
                                            .find_image_or_metadata(url.clone(),
                                                                    use_placeholder);
        match result {
            Ok(image_or_metadata) => Some(image_or_metadata),
            // Image failed to load, so just return nothing
            Err(ImageState::LoadError) => None,
            // Not yet requested, async mode - request image or metadata from the cache
            Err(ImageState::NotRequested) => {
                let sender = self.image_cache_sender.lock().unwrap().clone();
                self.image_cache_thread.lock().unwrap()
                                       .request_image_and_metadata(url, sender, None);
                None
            }
            // Image has been requested, is still pending. Return no image for this paint loop.
            // When the image loads it will trigger a reflow and/or repaint.
            Err(ImageState::Pending) => None,
        }
    }

    pub fn get_webrender_image_for_url(&self,
                                       url: ServoUrl,
                                       use_placeholder: UsePlaceholder)
                                       -> Option<WebRenderImageInfo> {
        if let Some(existing_webrender_image) = self.webrender_image_cache
                                                    .read()
                                                    .get(&(url.clone(), use_placeholder)) {
            return Some((*existing_webrender_image).clone())
        }

        match self.get_or_request_image_or_meta(url.clone(), use_placeholder) {
            Some(ImageOrMetadataAvailable::ImageAvailable(image)) => {
                let image_info = WebRenderImageInfo::from_image(&*image);
                if image_info.key.is_none() {
                    Some(image_info)
                } else {
                    let mut webrender_image_cache = self.webrender_image_cache.write();
                    webrender_image_cache.insert((url, use_placeholder),
                                                 image_info);
                    Some(image_info)
                }
            }
            None | Some(ImageOrMetadataAvailable::MetadataAvailable(_)) => None,
        }
    }
}
