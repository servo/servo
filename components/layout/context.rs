/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use base::id::PipelineId;
use euclid::Size2D;
use fnv::FnvHashMap;
use fonts::FontContext;
use fxhash::FxHashMap;
use layout_api::{
    IFrameSizes, ImageAnimationState, PendingImage, PendingImageState, PendingRasterizationImage,
};
use net_traits::image_cache::{
    Image as CachedImage, ImageCache, ImageCacheResult, ImageOrMetadataAvailable, PendingImageId,
    UsePlaceholder,
};
use parking_lot::{Mutex, RwLock};
use pixels::RasterImage;
use servo_url::{ImmutableOrigin, ServoUrl};
use style::context::SharedStyleContext;
use style::dom::OpaqueNode;
use style::values::computed::image::{Gradient, Image};
use webrender_api::units::{DeviceIntSize, DeviceSize};

pub(crate) type CachedImageOrError = Result<CachedImage, ResolveImageError>;

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

    /// A list of fully loaded vector images that need to be rasterized to a specific
    /// size determined by layout. This will be shared with the script thread.
    pub pending_rasterization_images: Mutex<Vec<PendingRasterizationImage>>,

    /// A collection of `<iframe>` sizes to send back to script.
    pub iframe_sizes: Mutex<IFrameSizes>,

    // A cache that maps image resources used in CSS (e.g as the `url()` value
    // for `background-image` or `content` property) to the final resolved image data.
    pub resolved_images_cache:
        Arc<RwLock<FnvHashMap<(ServoUrl, UsePlaceholder), CachedImageOrError>>>,

    pub node_image_animation_map: Arc<RwLock<FxHashMap<OpaqueNode, ImageAnimationState>>>,

    /// The DOM node that is highlighted by the devtools inspector, if any
    pub highlighted_dom_node: Option<OpaqueNode>,
}

pub enum ResolvedImage<'a> {
    Gradient(&'a Gradient),
    // The size is tracked explicitly as image-set images can specify their
    // natural resolution which affects the final size for raster images.
    Image {
        image: CachedImage,
        size: DeviceSize,
    },
}

impl Drop for LayoutContext<'_> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(self.pending_images.lock().is_empty());
            assert!(self.pending_rasterization_images.lock().is_empty());
        }
    }
}

#[derive(Clone, Copy, Debug)]
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

pub(crate) enum LayoutImageCacheResult {
    Pending,
    DataAvailable(ImageOrMetadataAvailable),
    LoadError,
}

impl LayoutContext<'_> {
    #[inline(always)]
    pub fn shared_context(&self) -> &SharedStyleContext {
        &self.style_context
    }

    pub(crate) fn get_or_request_image_or_meta(
        &self,
        node: OpaqueNode,
        url: ServoUrl,
        use_placeholder: UsePlaceholder,
    ) -> LayoutImageCacheResult {
        // Check for available image or start tracking.
        let cache_result = self.image_cache.get_cached_image_status(
            url.clone(),
            self.origin.clone(),
            None,
            use_placeholder,
        );

        match cache_result {
            ImageCacheResult::Available(img_or_meta) => {
                LayoutImageCacheResult::DataAvailable(img_or_meta)
            },
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
                LayoutImageCacheResult::Pending
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
                LayoutImageCacheResult::Pending
            },
            // Image failed to load, so just return the same error.
            ImageCacheResult::LoadError => LayoutImageCacheResult::LoadError,
        }
    }

    pub fn handle_animated_image(&self, node: OpaqueNode, image: Arc<RasterImage>) {
        let mut store = self.node_image_animation_map.write();

        // 1. first check whether node previously being track for animated image.
        if let Some(image_state) = store.get(&node) {
            // a. if the node is not containing the same image as before.
            if image_state.image_key() != image.id {
                if image.should_animate() {
                    // i. Register/Replace tracking item in image_animation_manager.
                    store.insert(
                        node,
                        ImageAnimationState::new(
                            image,
                            self.shared_context().current_time_for_animations,
                        ),
                    );
                } else {
                    // ii. Cancel Action if the node's image is no longer animated.
                    store.remove(&node);
                }
            }
        } else if image.should_animate() {
            store.insert(
                node,
                ImageAnimationState::new(image, self.shared_context().current_time_for_animations),
            );
        }
    }

    fn get_cached_image_for_url(
        &self,
        node: OpaqueNode,
        url: ServoUrl,
        use_placeholder: UsePlaceholder,
    ) -> Result<CachedImage, ResolveImageError> {
        if let Some(cached_image) = self
            .resolved_images_cache
            .read()
            .get(&(url.clone(), use_placeholder))
        {
            return cached_image.clone();
        }

        let result = self.get_or_request_image_or_meta(node, url.clone(), use_placeholder);
        match result {
            LayoutImageCacheResult::DataAvailable(img_or_meta) => match img_or_meta {
                ImageOrMetadataAvailable::ImageAvailable { image, .. } => {
                    if let Some(image) = image.as_raster_image() {
                        self.handle_animated_image(node, image.clone());
                    }

                    let mut resolved_images_cache = self.resolved_images_cache.write();
                    resolved_images_cache.insert((url, use_placeholder), Ok(image.clone()));
                    Ok(image)
                },
                ImageOrMetadataAvailable::MetadataAvailable(..) => {
                    Result::Err(ResolveImageError::OnlyMetadata)
                },
            },
            LayoutImageCacheResult::Pending => Result::Err(ResolveImageError::ImagePending),
            LayoutImageCacheResult::LoadError => {
                let error = Err(ResolveImageError::LoadError);
                self.resolved_images_cache
                    .write()
                    .insert((url, use_placeholder), error.clone());
                error
            },
        }
    }

    pub fn rasterize_vector_image(
        &self,
        image_id: PendingImageId,
        size: DeviceIntSize,
        node: OpaqueNode,
    ) -> Option<RasterImage> {
        let result = self.image_cache.rasterize_vector_image(image_id, size);
        if result.is_none() {
            self.pending_rasterization_images
                .lock()
                .push(PendingRasterizationImage {
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
                let image = self.get_cached_image_for_url(
                    node,
                    image_url.clone().into(),
                    UsePlaceholder::No,
                )?;
                let metadata = image.metadata();
                let size = Size2D::new(metadata.width, metadata.height).to_f32();
                Ok(ResolvedImage::Image { image, size })
            },
            Image::ImageSet(image_set) => {
                image_set
                    .items
                    .get(image_set.selected_index)
                    .ok_or(ResolveImageError::ImageMissingFromImageSet)
                    .and_then(|image| {
                        self.resolve_image(node, &image.image)
                            .map(|info| match info {
                                ResolvedImage::Image {
                                    image: cached_image,
                                    ..
                                } => {
                                    // From <https://drafts.csswg.org/css-images-4/#image-set-notation>:
                                    // > A <resolution> (optional). This is used to help the UA decide
                                    // > which <image-set-option> to choose. If the image reference is
                                    // > for a raster image, it also specifies the image’s natural
                                    // > resolution, overriding any other source of data that might
                                    // > supply a natural resolution.
                                    let image_metadata = cached_image.metadata();
                                    let size = if cached_image.as_raster_image().is_some() {
                                        let scale_factor = image.resolution.dppx();
                                        Size2D::new(
                                            image_metadata.width as f32 / scale_factor,
                                            image_metadata.height as f32 / scale_factor,
                                        )
                                    } else {
                                        Size2D::new(image_metadata.width, image_metadata.height)
                                            .to_f32()
                                    };

                                    ResolvedImage::Image {
                                        image: cached_image,
                                        size,
                                    }
                                },
                                _ => info,
                            })
                    })
            },
            Image::LightDark(..) => unreachable!("light-dark() should be disabled"),
        }
    }
}
