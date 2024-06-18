/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use base::id::PipelineId;
use fnv::FnvHashMap;
use fonts::{FontCacheThread, FontContext};
use net_traits::image_cache::{
    ImageCache, ImageCacheResult, ImageOrMetadataAvailable, UsePlaceholder,
};
use parking_lot::{Mutex, RwLock};
use script_layout_interface::{PendingImage, PendingImageState};
use servo_url::{ImmutableOrigin, ServoUrl};
use style::context::SharedStyleContext;
use style::dom::OpaqueNode;

use crate::display_list::WebRenderImageInfo;

pub struct LayoutContext<'a> {
    pub id: PipelineId,
    pub use_rayon: bool,
    pub origin: ImmutableOrigin,

    /// Bits shared by the layout and style system.
    pub style_context: SharedStyleContext<'a>,

    /// A FontContext to be used during layout.
    pub font_context: Arc<FontContext<FontCacheThread>>,

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
            assert!(self.pending_images.lock().is_empty());
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
                    node: node.into(),
                    id,
                    origin: self.origin.clone(),
                };
                self.pending_images.lock().push(image);
                None
            },
            // Not yet requested - request image or metadata from the cache
            ImageCacheResult::ReadyForRequest(id) => {
                let image = PendingImage {
                    state: PendingImageState::Unrequested(url),
                    node: node.into(),
                    id,
                    origin: self.origin.clone(),
                };
                self.pending_images.lock().push(image);
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
            return Some(*existing_webrender_image);
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
