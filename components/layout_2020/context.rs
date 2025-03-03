/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use base::id::PipelineId;
use fnv::FnvHashMap;
use fonts::FontContext;
use net_traits::image_cache::{
    ImageCache, ImageCacheResult, ImageOrMetadataAvailable, UsePlaceholder,
};
use parking_lot::{Mutex, RwLock};
use pixels::{ImageContainer, ImageFrame};
use script_layout_interface::{
    IFrameSizes, ImageAnimateRegisterItem, LayoutImageAnimateHelper, PendingImage,
    PendingImageState,
};
use servo_url::{ImmutableOrigin, ServoUrl};
use style::context::SharedStyleContext;
use style::dom::OpaqueNode;
use webrender_api::ImageKey;

use crate::display_list::WebRenderImageInfo;

pub struct LayoutContext<'a> {
    pub id: PipelineId,
    pub use_rayon: bool,
    pub origin: ImmutableOrigin,

    /// Bits shared by the layout and style system.
    pub style_context: SharedStyleContext<'a>,

    /// A FontContext to be used during layout.
    pub font_context: Arc<FontContext>,

    /// Reference to the script thread image cache.
    pub image_cache: Arc<dyn ImageCache>,

    /// A list of in-progress image loads to be shared with the script thread.
    pub pending_images: Mutex<Vec<PendingImage>>,

    /// A collection of `<iframe>` sizes to send back to script.
    pub iframe_sizes: Mutex<IFrameSizes>,

    pub webrender_image_cache:
        Arc<RwLock<FnvHashMap<(ServoUrl, UsePlaceholder), WebRenderImageInfo>>>,

    pub image_animation_helper: Arc<LayoutImageAnimateHelper>,

    /// A list of image animation need to be registered.
    pub register_image_animations: Mutex<Vec<ImageAnimateRegisterItem>>,
}

impl Drop for LayoutContext<'_> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(self.pending_images.lock().is_empty());
        }
    }
}

impl LayoutContext<'_> {
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

    pub fn get_frame_from_image(
        &self,
        img: Option<Arc<ImageContainer>>,
        node: OpaqueNode,
    ) -> Option<ImageFrame> {
        img.map(|image| {
            self.image_animation_helper
                .get_image_frame_from_image(&image, &node)
        })
    }

    pub fn get_frame_key_from_image(
        &self,
        img: Option<Arc<ImageContainer>>,
        node: OpaqueNode,
    ) -> Option<ImageKey> {
        img.map(|image| {
            self.image_animation_helper
                .get_image_frame_key_from_image(&image, &node)
        })?
    }

    pub fn should_be_register(&self, node: OpaqueNode, img: Option<Arc<ImageContainer>>) -> bool {
        if let Some(img) = img {
            if img.is_animate {
                !self.image_animation_helper.entry_exist(&node)
            } else {
                false
            }
        } else {
            false
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
                //(Ray)TODO: better handle Option<Arc<Image>> mitigate copy.
                let image_frame = self
                    .get_frame_from_image(Some(image.clone()), node)
                    .unwrap(); // (Ray)TODO: better handle unwrap.
                if self.should_be_register(node, Some(image.clone())) {
                    self.register_image_animations
                        .lock()
                        .push(ImageAnimateRegisterItem {
                            node,
                            url: Some(url.clone()),
                            image: image.clone(),
                        });
                }
                let image_info = WebRenderImageInfo {
                    width: image_frame.width,
                    height: image_frame.height,
                    key: image_frame.id,
                };
                if image_info.key.is_none() {
                    Some(image_info)
                } else {
                    // only put into cache if the image is not animated.
                    if !image.is_animate {
                        let mut webrender_image_cache = self.webrender_image_cache.write();
                        webrender_image_cache.insert((url, use_placeholder), image_info);
                    }

                    Some(image_info)
                }
            },
            None | Some(ImageOrMetadataAvailable::MetadataAvailable(..)) => None,
        }
    }
}
