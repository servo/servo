/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::Arc;

use base::id::PainterId;
use embedder_traits::UntrustedNodeAddress;
use euclid::Size2D;
use fonts::FontContext;
use layout_api::wrapper_traits::ThreadSafeLayoutNode;
use layout_api::{
    AnimatingImages, IFrameSizes, LayoutImageDestination, PendingImage, PendingImageState,
    PendingRasterizationImage,
};
use net_traits::image_cache::{
    Image as CachedImage, ImageCache, ImageCacheResult, ImageOrMetadataAvailable, PendingImageId,
};
use parking_lot::{Mutex, RwLock};
use pixels::RasterImage;
use script::layout_dom::ServoThreadSafeLayoutNode;
use servo_url::{ImmutableOrigin, ServoUrl};
use style::context::SharedStyleContext;
use style::dom::OpaqueNode;
use style::values::computed::image::{Gradient, Image};
use webrender_api::units::{DeviceIntSize, DeviceSize};

pub(crate) type CachedImageOrError = Result<CachedImage, ResolveImageError>;

pub(crate) struct LayoutContext<'a> {
    pub use_rayon: bool,

    /// Bits shared by the layout and style system.
    pub style_context: SharedStyleContext<'a>,

    /// A FontContext to be used during layout.
    pub font_context: Arc<FontContext>,

    /// A collection of `<iframe>` sizes to send back to script.
    pub iframe_sizes: Mutex<IFrameSizes>,

    /// An [`ImageResolver`] used for resolving images during box and fragment
    /// tree construction. Later passed to display list construction.
    pub image_resolver: Arc<ImageResolver>,

    /// The [`PainterId`] that identifies which `RenderingContext` that this layout targets.
    pub painter_id: PainterId,
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

#[derive(Clone, Copy, Debug)]
pub enum ResolveImageError {
    LoadError,
    ImagePending,
    OnlyMetadata,
    InvalidUrl,
    MissingNode,
    ImageMissingFromImageSet,
    NotImplementedYet,
    None,
}

pub(crate) enum LayoutImageCacheResult {
    Pending,
    DataAvailable(ImageOrMetadataAvailable),
    LoadError,
}

pub(crate) struct ImageResolver {
    /// The origin of the `Document` that this [`ImageResolver`] resolves images for.
    pub origin: ImmutableOrigin,

    /// Reference to the script thread image cache.
    pub image_cache: Arc<dyn ImageCache>,

    /// A list of in-progress image loads to be shared with the script thread.
    pub pending_images: Mutex<Vec<PendingImage>>,

    /// A list of fully loaded vector images that need to be rasterized to a specific
    /// size determined by layout. This will be shared with the script thread.
    pub pending_rasterization_images: Mutex<Vec<PendingRasterizationImage>>,

    /// A list of `SVGSVGElement`s encountered during layout that are not
    /// serialized yet. This is needed to support inline SVGs as they are treated
    /// as replaced elements and the layout is responsible for triggering the
    /// network load for the corresponding serialized data: urls (similar to
    /// background images).
    pub pending_svg_elements_for_serialization: Mutex<Vec<UntrustedNodeAddress>>,

    /// A shared reference to script's map of DOM nodes with animated images. This is used
    /// to manage image animations in script and inform the script about newly animating
    /// nodes.
    pub animating_images: Arc<RwLock<AnimatingImages>>,

    // A cache that maps image resources used in CSS (e.g as the `url()` value
    // for `background-image` or `content` property) to the final resolved image data.
    pub resolved_images_cache: Arc<RwLock<HashMap<ServoUrl, CachedImageOrError>>>,

    /// The current animation timeline value used to properly initialize animating images.
    pub animation_timeline_value: f64,
}

impl Drop for ImageResolver {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(self.pending_images.lock().is_empty());
            assert!(self.pending_rasterization_images.lock().is_empty());
            assert!(
                self.pending_svg_elements_for_serialization
                    .lock()
                    .is_empty()
            );
        }
    }
}

impl ImageResolver {
    pub(crate) fn get_or_request_image_or_meta(
        &self,
        node: OpaqueNode,
        url: ServoUrl,
        destination: LayoutImageDestination,
        svg_fallback_font_size: Option<f32>,
    ) -> LayoutImageCacheResult {
        // Check for available image or start tracking.
        let cache_result = self.image_cache.get_cached_image_status(
            url.clone(),
            self.origin.clone(),
            None,
            svg_fallback_font_size,
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
                    destination,
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
                    destination,
                };
                self.pending_images.lock().push(image);
                LayoutImageCacheResult::Pending
            },
            // Image failed to load, so just return the same error.
            ImageCacheResult::FailedToLoadOrDecode => LayoutImageCacheResult::LoadError,
        }
    }

    pub(crate) fn handle_animated_image(&self, node: OpaqueNode, image: Arc<RasterImage>) {
        let mut animating_images = self.animating_images.write();
        if !image.should_animate() {
            animating_images.remove(node);
        } else {
            animating_images.maybe_insert_or_update(node, image, self.animation_timeline_value);
        }
    }

    pub(crate) fn get_cached_image_for_url(
        &self,
        node: OpaqueNode,
        url: ServoUrl,
        destination: LayoutImageDestination,
        svg_fallback_font_size: Option<f32>,
    ) -> Result<CachedImage, ResolveImageError> {
        if let Some(cached_image) = self.resolved_images_cache.read().get(&url) {
            return cached_image.clone();
        }

        let result = self.get_or_request_image_or_meta(
            node,
            url.clone(),
            destination,
            svg_fallback_font_size,
        );
        match result {
            LayoutImageCacheResult::DataAvailable(img_or_meta) => match img_or_meta {
                ImageOrMetadataAvailable::ImageAvailable { image, .. } => {
                    if let Some(image) = image.as_raster_image() {
                        self.handle_animated_image(node, image.clone());
                    }

                    let mut resolved_images_cache = self.resolved_images_cache.write();
                    resolved_images_cache.insert(url, Ok(image.clone()));
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
                    .insert(url, error.clone());
                error
            },
        }
    }

    pub(crate) fn rasterize_vector_image(
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

    pub(crate) fn queue_svg_element_for_serialization(
        &self,
        element: ServoThreadSafeLayoutNode<'_>,
    ) {
        self.pending_svg_elements_for_serialization
            .lock()
            .push(element.opaque().into())
    }

    pub(crate) fn resolve_image<'a>(
        &self,
        node: Option<OpaqueNode>,
        image: &'a Image,
        svg_fallback_font_size: Option<f32>,
    ) -> Result<ResolvedImage<'a>, ResolveImageError> {
        match image {
            // TODO: Add support for PaintWorklet and CrossFade rendering.
            Image::None => Result::Err(ResolveImageError::None),
            Image::CrossFade(_) => Result::Err(ResolveImageError::NotImplementedYet),
            Image::PaintWorklet(_) => Result::Err(ResolveImageError::NotImplementedYet),
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
                    LayoutImageDestination::DisplayListBuilding,
                    svg_fallback_font_size,
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
                        self.resolve_image(node, &image.image, svg_fallback_font_size)
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
