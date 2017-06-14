/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data needed by the layout thread.

use fnv::FnvHashMap;
use fnv::FnvHasher;
use gfx::display_list::{WebRenderImageInfo, OpaqueNode};
use gfx::font_cache_thread::FontCacheThread;
use gfx::font_context::FontContext;
use heapsize::HeapSizeOf;
use msg::constellation_msg::PipelineId;
use net_traits::image_cache::{CanRequestImages, ImageCache, ImageState};
use net_traits::image_cache::{ImageOrMetadataAvailable, UsePlaceholder};
use opaque_node::OpaqueNodeMethods;
use parking_lot::RwLock;
use script_layout_interface::{PendingImage, PendingImageState};
use script_traits::Painter;
use script_traits::UntrustedNodeAddress;
use servo_atoms::Atom;
use servo_url::ServoUrl;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::sync::{Arc, Mutex};
use std::thread;
use style::context::SharedStyleContext;
use style::properties::PropertyId;

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
pub struct LayoutContext<'a> {
    /// The pipeline id of this LayoutContext.
    pub id: PipelineId,

    /// Bits shared by the layout and style system.
    pub style_context: SharedStyleContext<'a>,

    /// Reference to the script thread image cache.
    pub image_cache: Arc<ImageCache>,

    /// Interface to the font cache thread.
    pub font_cache_thread: Mutex<FontCacheThread>,

    /// A cache of WebRender image info.
    pub webrender_image_cache: Arc<RwLock<HashMap<(ServoUrl, UsePlaceholder),
                                                  WebRenderImageInfo,
                                                  BuildHasherDefault<FnvHasher>>>>,

    /// Paint worklets
    pub registered_painters: Arc<RwLock<FnvHashMap<Atom, RegisteredPainter>>>,

    /// A list of in-progress image loads to be shared with the script thread.
    /// A None value means that this layout was not initiated by the script thread.
    pub pending_images: Option<Mutex<Vec<PendingImage>>>,

    /// A list of nodes that have just initiated a CSS transition.
    /// A None value means that this layout was not initiated by the script thread.
    pub newly_transitioning_nodes: Option<Mutex<Vec<UntrustedNodeAddress>>>,
}

impl<'a> Drop for LayoutContext<'a> {
    fn drop(&mut self) {
        if !thread::panicking() {
            if let Some(ref pending_images) = self.pending_images {
                assert!(pending_images.lock().unwrap().is_empty());
            }
        }
    }
}

impl<'a> LayoutContext<'a> {
    #[inline(always)]
    pub fn shared_context(&self) -> &SharedStyleContext {
        &self.style_context
    }

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
        let result = self.image_cache.find_image_or_metadata(url.clone(),
                                                             use_placeholder,
                                                             can_request);
        match result {
            Ok(image_or_metadata) => Some(image_or_metadata),
            // Image failed to load, so just return nothing
            Err(ImageState::LoadError) => None,
            // Not yet requested - request image or metadata from the cache
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
            Some(ImageOrMetadataAvailable::ImageAvailable(image, _)) => {
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

/// A registered paint worklet.
pub struct RegisteredPainter {
    pub name: Atom,
    pub properties: FnvHashMap<Atom, PropertyId>,
    pub painter: Arc<Painter>,
}
