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
    Image as CachedImage, ImageCache, ImageCacheResult, ImageOrMetadataAvailable, PendingImageId,
    UsePlaceholder,
};
use parking_lot::{Mutex, RwLock};
use pixels::Image as PixelImage;
use script_layout_interface::{
    IFrameSizes, ImageAnimationState, PendingImage, PendingImageRasterization, PendingImageState,
};
use servo_url::{ImmutableOrigin, ServoUrl};
use style::context::SharedStyleContext;
use style::dom::OpaqueNode;
use style::values::computed::image::{Gradient, Image};
use webrender_api::units::DeviceIntSize;

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

    /// A list of fully loaded vector images that need to be rasterized. This will be
    /// shared with the script thread.
    pub pending_rasterization_images: Mutex<Vec<PendingImageRasterization>>,

    /// A collection of `<iframe>` sizes to send back to script.
    pub iframe_sizes: Mutex<IFrameSizes>,

    pub resolved_image_cache:
        Arc<RwLock<FnvHashMap<(ServoUrl, UsePlaceholder), Option<CachedImage>>>>,

    pub node_image_animation_map: Arc<RwLock<FxHashMap<OpaqueNode, ImageAnimationState>>>,

    /// The DOM node that is highlighted by the devtools inspector, if any
    pub highlighted_dom_node: Option<OpaqueNode>,
}

pub struct LayoutImage {
    pub image: CachedImage,
    // image-set images can override the natural resolution
    // and hence the final size for raster images
    pub size: DeviceIntSize,
}

pub enum ResolvedImage<'a> {
    Gradient(&'a Gradient),
    Image(LayoutImage),
}

impl Drop for LayoutContext<'_> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(self.pending_images.lock().is_empty());
            assert!(self.pending_rasterization_images.lock().is_empty());
        }
    }
}

#[derive(Debug)]
pub enum ResolveImageError {
    LoadError,
    ImagePending,
    ImageRequested,
    OnlyMetadata,
    InvalidUrl,
    MissingNode,
    ImageMissingFromImageSet,
    FailedToResolveImageFromImageSet,
    NotImplementedYet(&'static str),
    None,
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
    ) -> Result<ImageOrMetadataAvailable, ResolveImageError> {
        // Check for available image or start tracking.
        let cache_result = self.image_cache.get_cached_image_status(
            url.clone(),
            self.origin.clone(),
            None,
            use_placeholder,
        );

        match cache_result {
            ImageCacheResult::Available(img_or_meta) => Ok(img_or_meta),
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
                Result::Err(ResolveImageError::ImagePending)
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
                Result::Err(ResolveImageError::ImageRequested)
            },
            // Image failed to load, so just return nothing
            ImageCacheResult::LoadError => Result::Err(ResolveImageError::LoadError),
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
    ) -> Result<CachedImage, ResolveImageError> {
        if let Some(cached_image) = self
            .resolved_image_cache
            .read()
            .get(&(url.clone(), use_placeholder))
        {
            return cached_image
                .as_ref()
                .map_or(Err(ResolveImageError::LoadError), |image| Ok(image.clone()));
        }

        let image_or_meta =
            self.get_or_request_image_or_meta(node, url.clone(), use_placeholder)?;
        match image_or_meta {
            ImageOrMetadataAvailable::ImageAvailable { image, .. } => {
                if let Some(image) = image.as_raster_image() {
                    self.handle_animated_image(node, image.clone());
                }

                let mut webrender_image_cache = self.resolved_image_cache.write();
                webrender_image_cache.insert((url, use_placeholder), Some(image.clone()));
                Ok(image)
            },
            ImageOrMetadataAvailable::MetadataAvailable(..) => {
                Result::Err(ResolveImageError::OnlyMetadata)
            },
        }
    }

    pub fn rasterize_vector_image(
        &self,
        image_id: PendingImageId,
        size: DeviceIntSize,
        node: OpaqueNode,
    ) -> Option<PixelImage> {
        let pipeline_id = self.id;
        let result = self
            .image_cache
            .rasterize_vector_image(pipeline_id, image_id, size);
        if result.is_none() {
            self.pending_rasterization_images
                .lock()
                .push(PendingImageRasterization {
                    id: image_id,
                    node: node.into(),
                    size,
                });
        }
        result
    }

    pub fn resolve_image<'a>(
        &self,
        node: Option<OpaqueNode>,
        image: &'a Image,
    ) -> Result<ResolvedImage<'a>, ResolveImageError> {
        match image {
            // TODO: Add support for PaintWorklet and CrossFade rendering.
            Image::None => Result::Err(ResolveImageError::None),
            Image::CrossFade(_) => Result::Err(ResolveImageError::NotImplementedYet("CrossFade")),
            Image::PaintWorklet(_) => {
                Result::Err(ResolveImageError::NotImplementedYet("PaintWorklet"))
            },
            Image::Gradient(gradient) => Ok(ResolvedImage::Gradient(gradient)),
            Image::Url(image_url) => {
                // FIXME: images won’t always have in intrinsic width or
                // height when support for SVG is added, or a WebRender
                // `ImageKey`, for that matter.
                //
                // FIXME: It feels like this should take into account the pseudo
                // element and not just the node.
                let image_url = image_url.url().ok_or(ResolveImageError::InvalidUrl)?;
                let node = node.ok_or(ResolveImageError::MissingNode)?;
                let image = self.get_webrender_image_for_url(
                    node,
                    image_url.clone().into(),
                    UsePlaceholder::No,
                )?;
                let size = Size2D::new(
                    image.metadata().width as i32,
                    image.metadata().height as i32,
                );
                Ok(ResolvedImage::Image(LayoutImage { image, size }))
            },
            Image::ImageSet(image_set) => {
                image_set
                    .items
                    .get(image_set.selected_index)
                    .ok_or(ResolveImageError::ImageMissingFromImageSet)
                    .and_then(|image| {
                        self.resolve_image(node, &image.image)
                            .map(|info| match info {
                                ResolvedImage::Image(layout_image) => {
                                    // From <https://drafts.csswg.org/css-images-4/#image-set-notation>:
                                    // > A <resolution> (optional). This is used to help the UA decide
                                    // > which <image-set-option> to choose. If the image reference is
                                    // > for a raster image, it also specifies the image’s natural
                                    // > resolution, overriding any other source of data that might
                                    // > supply a natural resolution.
                                    let image_metadata = layout_image.image.metadata();
                                    let size = if layout_image.image.as_raster_image().is_some() {
                                        let scale_factor = image.resolution.dppx();
                                        Size2D::new(
                                            image_metadata.width as f32 / scale_factor,
                                            image_metadata.height as f32 / scale_factor,
                                        )
                                        .to_i32()
                                    } else {
                                        Size2D::new(image_metadata.width, image_metadata.height)
                                            .to_i32()
                                    };
                                    ResolvedImage::Image(LayoutImage {
                                        size,
                                        ..layout_image
                                    })
                                },
                                _ => info,
                            })
                    })
            },
        }
    }
}
