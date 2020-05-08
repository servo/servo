/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::display_list::WebRenderImageInfo;
use crate::opaque_node::OpaqueNodeMethods;
use fnv::FnvHashMap;
use gfx::font_cache_thread::FontCacheThread;
use gfx::font_context::FontContext;
use msg::constellation_msg::PipelineId;
use net_traits::image_cache::{
    ImageCache, ImageCacheResult, ImageOrMetadataAvailable, UsePlaceholder,
};
use parking_lot::RwLock;
use script_layout_interface::{PendingImage, PendingImageState};
use servo_url::{ImmutableOrigin, ServoUrl};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use style::context::SharedStyleContext;
use style::dom::OpaqueNode;

pub struct LayoutContext<'a> {
    pub id: PipelineId,
    pub use_rayon: bool,
    pub origin: ImmutableOrigin,

    /// Bits shared by the layout and style system.
    pub style_context: SharedStyleContext<'a>,

    /// Interface to the font cache thread.
    pub font_cache_thread: Mutex<FontCacheThread>,

    /// Reference to the script thread image cache.
    pub image_cache: Arc<dyn ImageCache>,

    /// A list of in-progress image loads to be shared with the script thread.
    pub pending_images: Mutex<Vec<PendingImage>>,

    pub webrender_image_cache:
        Arc<RwLock<FnvHashMap<(ServoUrl, UsePlaceholder), WebRenderImageInfo>>>,
}

impl<'a> Drop for LayoutContext<'a> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(self.pending_images.lock().unwrap().is_empty());
        }
    }
}

impl<'a> LayoutContext<'a> {
    #[inline(always)]
    pub fn shared_context(&self) -> &SharedStyleContext {
        &self.style_context
    }

    pub fn get_or_request_image_or_meta(
        &self,
        node: OpaqueNode,
        url: ServoUrl,
        use_placeholder: UsePlaceholder,
    ) -> Option<ImageOrMetadataAvailable> {
        // Check for available image or start tracking.
        let cache_result = self.image_cache.get_cached_image_status(
            url.clone(),
            self.origin.clone(),
            None,
            use_placeholder,
        );

        match cache_result {
            ImageCacheResult::Available(img_or_meta) => Some(img_or_meta),
            // Image has been requested, is still pending. Return no image for this paint loop.
            // When the image loads it will trigger a reflow and/or repaint.
            ImageCacheResult::Pending(id) => {
                let image = PendingImage {
                    state: PendingImageState::PendingResponse,
                    node: node.to_untrusted_node_address(),
                    id,
                    origin: self.origin.clone(),
                };
                self.pending_images.lock().unwrap().push(image);
                None
            },
            // Not yet requested - request image or metadata from the cache
            ImageCacheResult::ReadyForRequest(id) => {
                let image = PendingImage {
                    state: PendingImageState::Unrequested(url),
                    node: node.to_untrusted_node_address(),
                    id,
                    origin: self.origin.clone(),
                };
                self.pending_images.lock().unwrap().push(image);
                None
            },
            // Image failed to load, so just return nothing
            ImageCacheResult::LoadError => None,
        }
    }

    pub fn get_webrender_image_for_url(
        &self,
        node: OpaqueNode,
        url: ServoUrl,
        use_placeholder: UsePlaceholder,
    ) -> Option<WebRenderImageInfo> {
        if let Some(existing_webrender_image) = self
            .webrender_image_cache
            .read()
            .get(&(url.clone(), use_placeholder))
        {
            return Some((*existing_webrender_image).clone());
        }

        match self.get_or_request_image_or_meta(node, url.clone(), use_placeholder) {
            Some(ImageOrMetadataAvailable::ImageAvailable { image, .. }) => {
                let image_info = WebRenderImageInfo {
                    width: image.width,
                    height: image.height,
                    key: image.id,
                };
                if image_info.key.is_none() {
                    Some(image_info)
                } else {
                    let mut webrender_image_cache = self.webrender_image_cache.write();
                    webrender_image_cache.insert((url, use_placeholder), image_info);
                    Some(image_info)
                }
            },
            None | Some(ImageOrMetadataAvailable::MetadataAvailable(_)) => None,
        }
    }
}

pub(crate) type LayoutFontContext = FontContext<FontCacheThread>;

thread_local!(static FONT_CONTEXT: RefCell<Option<LayoutFontContext>> = RefCell::new(None));

pub(crate) fn with_thread_local_font_context<F, R>(layout_context: &LayoutContext, f: F) -> R
where
    F: FnOnce(&mut LayoutFontContext) -> R,
{
    FONT_CONTEXT.with(|font_context| {
        f(font_context.borrow_mut().get_or_insert_with(|| {
            FontContext::new(layout_context.font_cache_thread.lock().unwrap().clone())
        }))
    })
}
