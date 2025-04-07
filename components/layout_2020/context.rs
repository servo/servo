/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use base::id::PipelineId;
use euclid::Size2D;
use fnv::FnvHashMap;
use fonts::FontContext;
use fxhash::FxHashMap;
use net_traits::image_cache::{
    ImageCache, ImageCacheResult, ImageOrMetadataAvailable, UsePlaceholder,
};
use parking_lot::{Mutex, RwLock};
use pixels::Image as PixelImage;
use script_layout_interface::{IFrameSizes, ImageAnimationState, PendingImage, PendingImageState};
use servo_url::{ImmutableOrigin, ServoUrl};
use style::context::SharedStyleContext;
use style::dom::OpaqueNode;
use style::values::computed::image::{Gradient, Image};

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

    pub node_image_animation_map: Arc<RwLock<FxHashMap<OpaqueNode, ImageAnimationState>>>,
}

pub enum ResolvedImage<'a> {
    Gradient(&'a Gradient),
    Image(WebRenderImageInfo),
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

    pub fn handle_animated_image(&self, node: OpaqueNode, image: Arc<PixelImage>) {
        let mut store = self.node_image_animation_map.write();

        // 1. first check whether node previously being track for animated image.
        if let Some(image_state) = store.get(&node) {
            // a. if the node is not containing the same image as before.
            if image_state.image_key() != image.id {
                if image.should_animate() {
                    // i. Register/Replace tracking item in image_animation_manager.
                    store.insert(node, ImageAnimationState::new(image));
                } else {
                    // ii. Cancel Action if the node's image is no longer animated.
                    store.remove(&node);
                }
            }
        } else if image.should_animate() {
            store.insert(node, ImageAnimationState::new(image));
        }
    }

    fn get_webrender_image_for_url(
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
                self.handle_animated_image(node, image.clone());
                let image_info = WebRenderImageInfo {
                    size: Size2D::new(image.width, image.height),
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
            None | Some(ImageOrMetadataAvailable::MetadataAvailable(..)) => None,
        }
    }

    pub fn resolve_image<'a>(
        &self,
        node: Option<OpaqueNode>,
        image: &'a Image,
    ) -> Option<ResolvedImage<'a>> {
        match image {
            // TODO: Add support for PaintWorklet and CrossFade rendering.
            Image::None | Image::CrossFade(_) | Image::PaintWorklet(_) => None,
            Image::Gradient(gradient) => Some(ResolvedImage::Gradient(gradient)),
            Image::Url(image_url) => {
                // FIXME: images won’t always have in intrinsic width or
                // height when support for SVG is added, or a WebRender
                // `ImageKey`, for that matter.
                //
                // FIXME: It feels like this should take into account the pseudo
                // element and not just the node.
                let image_url = image_url.url()?;
                let webrender_info = self.get_webrender_image_for_url(
                    node?,
                    image_url.clone().into(),
                    UsePlaceholder::No,
                )?;
                Some(ResolvedImage::Image(webrender_info))
            },
            Image::ImageSet(image_set) => {
                image_set
                    .items
                    .get(image_set.selected_index)
                    .and_then(|image| {
                        self.resolve_image(node, &image.image)
                            .map(|info| match info {
                                ResolvedImage::Image(mut image_info) => {
                                    // From <https://drafts.csswg.org/css-images-4/#image-set-notation>:
                                    // > A <resolution> (optional). This is used to help the UA decide
                                    // > which <image-set-option> to choose. If the image reference is
                                    // > for a raster image, it also specifies the image’s natural
                                    // > resolution, overriding any other source of data that might
                                    // > supply a natural resolution.
                                    image_info.size = (image_info.size.to_f32() /
                                        image.resolution.dppx())
                                    .to_u32();
                                    ResolvedImage::Image(image_info)
                                },
                                _ => info,
                            })
                    })
            },
        }
    }
}
