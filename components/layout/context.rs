/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data needed by the layout thread.

use fnv::FnvHasher;
use gfx::display_list::{WebRenderImageInfo, OpaqueNode};
use gfx::font_cache_thread::FontCacheThread;
use gfx::font_context::FontContext;
use heapsize::HeapSizeOf;
use net_traits::image_cache_thread::{ImageCacheThread, ImageState, CanRequestImages};
use net_traits::image_cache_thread::{ImageOrMetadataAvailable, UsePlaceholder};
use opaque_node::OpaqueNodeMethods;
use parking_lot::RwLock;
use servo_url::ServoUrl;
use script_layout_interface::{PendingImage, PendingImageState};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use style::context::{SharedStyleContext, ThreadLocalStyleContext};
use style::dom::TElement;

/// TLS data scoped to the traversal.
pub struct ScopedThreadLocalLayoutContext<E: TElement> {
    pub style_context: ThreadLocalStyleContext<E>,
}

impl<E: TElement> ScopedThreadLocalLayoutContext<E> {
    pub fn new(shared: &SharedLayoutContext) -> Self {
        ScopedThreadLocalLayoutContext {
            style_context: ThreadLocalStyleContext::new(&shared.style_context),
        }
    }
}

/// TLS data that persists across traversals.
pub struct PersistentThreadLocalLayoutContext {
    // FontContext uses Rc all over the place and so isn't Send, which means we
    // can't use ScopedTLS for it. There's also no reason to scope it to the
    // traversal, and performance is probably better if we don't.
    pub font_context: RefCell<FontContext>,
}

impl PersistentThreadLocalLayoutContext {
    pub fn new(shared: &SharedLayoutContext) -> Rc<Self> {
        let font_cache_thread = shared.font_cache_thread.lock().unwrap().clone();
        Rc::new(PersistentThreadLocalLayoutContext {
            font_context: RefCell::new(FontContext::new(font_cache_thread)),
        })
    }
}

impl HeapSizeOf for PersistentThreadLocalLayoutContext {
    fn heap_size_of_children(&self) -> usize {
        self.font_context.heap_size_of_children()
    }
}

thread_local!(static LOCAL_CONTEXT_KEY: RefCell<Option<Rc<PersistentThreadLocalLayoutContext>>> = RefCell::new(None));

fn create_or_get_persistent_context(shared: &SharedLayoutContext)
                                    -> Rc<PersistentThreadLocalLayoutContext> {
    LOCAL_CONTEXT_KEY.with(|r| {
        let mut r = r.borrow_mut();
        if let Some(context) = r.clone() {
            context
        } else {
            let context = PersistentThreadLocalLayoutContext::new(shared);
            *r = Some(context.clone());
            context
        }
    })
}

pub fn heap_size_of_persistent_local_context() -> usize {
    LOCAL_CONTEXT_KEY.with(|r| {
        r.borrow().clone().map_or(0, |context| context.heap_size_of_children())
    })
}

/// Layout information shared among all workers. This must be thread-safe.
pub struct SharedLayoutContext {
    /// Bits shared by the layout and style system.
    pub style_context: SharedStyleContext,

    /// The shared image cache thread.
    pub image_cache_thread: Mutex<ImageCacheThread>,

    /// Interface to the font cache thread.
    pub font_cache_thread: Mutex<FontCacheThread>,

    /// A cache of WebRender image info.
    pub webrender_image_cache: Arc<RwLock<HashMap<(ServoUrl, UsePlaceholder),
                                                  WebRenderImageInfo,
                                                  BuildHasherDefault<FnvHasher>>>>,

    ///
    pub pending_images: Option<Mutex<Vec<PendingImage>>>
}

impl Drop for SharedLayoutContext {
    fn drop(&mut self) {
        if let Some(ref pending_images) = self.pending_images {
            assert!(pending_images.lock().unwrap().is_empty());
        }
    }
}

pub struct LayoutContext<'a> {
    pub shared: &'a SharedLayoutContext,
    pub persistent: Rc<PersistentThreadLocalLayoutContext>,
}

impl<'a> LayoutContext<'a> {
    pub fn new(shared: &'a SharedLayoutContext) -> Self
    {
        LayoutContext {
            shared: shared,
            persistent: create_or_get_persistent_context(shared),
        }
    }
}

impl<'a> LayoutContext<'a> {
    // FIXME(bholley): The following two methods are identical and should be merged.
    // shared_context() is the appropriate name, but it involves renaming a lot of
    // calls.
    #[inline(always)]
    pub fn shared_context(&self) -> &SharedStyleContext {
        &self.shared.style_context
    }

    #[inline(always)]
    pub fn style_context(&self) -> &SharedStyleContext {
        &self.shared.style_context
    }

    #[inline(always)]
    pub fn font_context(&self) -> RefMut<FontContext> {
        self.persistent.font_context.borrow_mut()
    }
}

impl SharedLayoutContext {
    pub fn get_or_request_image_or_meta(&self,
                                        node: OpaqueNode,
                                        url: ServoUrl,
                                        use_placeholder: UsePlaceholder)
                                        -> Option<ImageOrMetadataAvailable> {
        //XXXjdm For cases where we do not request an image, we still need to
        //       ensure the node gets another script-initiated reflow or it
        //       won't be requested at all.
        let can_request = if self.pending_images.is_some() {
            CanRequestImages::Yes
        } else {
            CanRequestImages::No
        };

        // See if the image is already available
        let result = self.image_cache_thread.lock().unwrap()
                                            .find_image_or_metadata(url.clone(),
                                                                    use_placeholder,
                                                                    can_request);
        match result {
            Ok(image_or_metadata) => Some(image_or_metadata),
            // Image failed to load, so just return nothing
            Err(ImageState::LoadError) => None,
            // Not yet requested, async mode - request image or metadata from the cache
            Err(ImageState::NotRequested(id)) => {
                let image = PendingImage {
                    state: PendingImageState::Unrequested(url),
                    node: node.to_untrusted_node_address(),
                    id: id,
                };
                self.pending_images.as_ref().unwrap().lock().unwrap().push(image);
                None
            }
            // Image has been requested, is still pending. Return no image for this paint loop.
            // When the image loads it will trigger a reflow and/or repaint.
            Err(ImageState::Pending(id)) => {
                //XXXjdm if self.pending_images is not available, we should make sure that
                //       this node gets marked dirty again so it gets a script-initiated
                //       reflow that deals with this properly.
                if let Some(ref pending_images) = self.pending_images {
                    let image = PendingImage {
                        state: PendingImageState::PendingResponse,
                        node: node.to_untrusted_node_address(),
                        id: id,
                    };
                    pending_images.lock().unwrap().push(image);
                }
                None
            }
        }
    }

    pub fn get_webrender_image_for_url(&self,
                                       node: OpaqueNode,
                                       url: ServoUrl,
                                       use_placeholder: UsePlaceholder)
                                       -> Option<WebRenderImageInfo> {
        if let Some(existing_webrender_image) = self.webrender_image_cache
                                                    .read()
                                                    .get(&(url.clone(), use_placeholder)) {
            return Some((*existing_webrender_image).clone())
        }

        match self.get_or_request_image_or_meta(node, url.clone(), use_placeholder) {
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
