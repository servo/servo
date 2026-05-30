/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::Arc;

use embedder_traits::UntrustedNodeAddress;
use euclid::Size2D;
use fonts::{FontContext, FontRef, ResolvedFontVariantAlternates};
use layout_api::{
    AnimatingImages, IFrameSizes, LayoutImageDestination, LayoutNode, PendingImage,
    PendingImageState, PendingRasterizationImage,
};
use net_traits::image_cache::{
    Image as CachedImage, ImageCache, ImageCacheResult, ImageOrMetadataAvailable, PendingImageId,
};
use net_traits::request::InternalRequest;
use parking_lot::{Mutex, RwLock};
use pixels::RasterImage;
use script::layout_dom::ServoLayoutNode;
use servo_base::id::PainterId;
use servo_url::{ImmutableOrigin, ServoUrl};
use style::Atom;
use style::context::SharedStyleContext;
use style::dom::OpaqueNode;
use style::values::computed::FontVariantAlternates;
use style::values::computed::image::{Gradient, Image};
use style::values::specified::font::VariantAlternates;
use webrender_api::units::{DeviceIntSize, DeviceSize};

use crate::font_feature_value::{FontFeatureValue, FontFeatureValueKind, FontFeatureValueMap};

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

    /// A lazily-computed map of feature names from `@font-feature-value` rules.
    pub font_feature_value_map: RwLock<Option<FontFeatureValueMap>>,
}

impl<'a> LayoutContext<'a> {
    pub(crate) fn resolve_font_variant_alternate_identifiers_for(
        &self,
        font: &FontRef,
        alternates: &FontVariantAlternates,
    ) -> ResolvedFontVariantAlternates {
        let mut resolved_alternates = ResolvedFontVariantAlternates::default();
        if alternates.is_empty() {
            return resolved_alternates;
        }
        let Some(family_name) = font.family_name() else {
            return resolved_alternates;
        };

        for alternate in alternates.iter() {
            match alternate {
                VariantAlternates::Stylistic(stylistic) => {
                    let Some(FontFeatureValue::Single(value)) = self
                        .lookup_font_feature_alternate_name(
                            family_name.clone(),
                            FontFeatureValueKind::Stylistic,
                            stylistic.0.clone(),
                        )
                    else {
                        continue;
                    };

                    resolved_alternates.stylistic = Some(value);
                },
                VariantAlternates::Styleset(styleset_list) => {
                    for styleset in styleset_list.iter() {
                        let Some(FontFeatureValue::Vector(value)) = self
                            .lookup_font_feature_alternate_name(
                                family_name.clone(),
                                FontFeatureValueKind::Styleset,
                                styleset.0.clone(),
                            )
                        else {
                            continue;
                        };

                        resolved_alternates.styleset.extend(value.0.iter());
                    }
                },
                VariantAlternates::CharacterVariant(character_variant_list) => {
                    for character_variant in character_variant_list.iter() {
                        let Some(FontFeatureValue::Pair(value)) = self
                            .lookup_font_feature_alternate_name(
                                family_name.clone(),
                                FontFeatureValueKind::CharacterVariant,
                                character_variant.0.clone(),
                            )
                        else {
                            continue;
                        };

                        resolved_alternates.character_variant.push(value);
                    }
                },
                VariantAlternates::Swash(swash) => {
                    let Some(FontFeatureValue::Single(value)) = self
                        .lookup_font_feature_alternate_name(
                            family_name.clone(),
                            FontFeatureValueKind::Swash,
                            swash.0.clone(),
                        )
                    else {
                        continue;
                    };

                    resolved_alternates.swash = Some(value);
                },
                VariantAlternates::Ornaments(ornaments) => {
                    let Some(FontFeatureValue::Single(value)) = self
                        .lookup_font_feature_alternate_name(
                            family_name.clone(),
                            FontFeatureValueKind::Ornaments,
                            ornaments.0.clone(),
                        )
                    else {
                        continue;
                    };

                    resolved_alternates.ornaments = Some(value);
                },
                VariantAlternates::Annotation(annotation) => {
                    let Some(FontFeatureValue::Single(value)) = self
                        .lookup_font_feature_alternate_name(
                            family_name.clone(),
                            FontFeatureValueKind::Annotation,
                            annotation.0.clone(),
                        )
                    else {
                        continue;
                    };

                    resolved_alternates.annotation = Some(value);
                },
                VariantAlternates::HistoricalForms => {
                    resolved_alternates.historical_forms = true;
                },
            }
        }

        resolved_alternates
    }

    fn lookup_font_feature_alternate_name(
        &self,
        family_name: Atom,
        kind: FontFeatureValueKind,
        name: Atom,
    ) -> Option<FontFeatureValue> {
        // First, check if the map was initialized previously by acquiring a read lock.
        let read_guard = self.font_feature_value_map.read();
        if let Some(map) = &*read_guard {
            // This is the cheap case, we just need to read from the map
            map.lookup(family_name, kind, name)
        } else {
            // Map was not initialized yet - need to acquire a write lock and initialize it.
            drop(read_guard);
            let mut write_guard = self.font_feature_value_map.write();
            if let Some(map) = &*write_guard {
                // We lost a race, some other thread initialized the map while we were waiting
                // on the lock.
                return map.lookup(family_name, kind, name);
            }

            log::debug!("Initializing @font-feature-values map");
            let mut map = FontFeatureValueMap::default();
            self.style_context
                .stylist
                .iter_extra_data_origins_rev()
                .flat_map(|(extra_data, _)| extra_data.font_feature_values.iter())
                .for_each(|(rule, _)| map.add_rule(rule));
            let map = &*write_guard.insert(map);
            // Finally, perform the actual lookup
            map.lookup(family_name, kind, name)
        }
    }

    /// This should be called whenever the stylesheets changed, after a restyle.
    pub(crate) fn invalidate_font_feature_value_map(&self) {
        log::debug!("Stylesheets changed, invalidating @font-feature-values map");
        self.font_feature_value_map.write().take();
    }
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
        is_internal_request: InternalRequest,
    ) -> LayoutImageCacheResult {
        // Check for available image or start tracking.
        let cache_result =
            self.image_cache
                .get_cached_image_status(url.clone(), self.origin.clone(), None);

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
                    is_internal_request,
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
                    is_internal_request,
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
        is_internal_request: InternalRequest,
    ) -> Result<CachedImage, ResolveImageError> {
        if let Some(cached_image) = self.resolved_images_cache.read().get(&url) {
            return cached_image.clone();
        }

        let result =
            self.get_or_request_image_or_meta(node, url.clone(), destination, is_internal_request);
        match result {
            LayoutImageCacheResult::DataAvailable(img_or_meta) => match img_or_meta {
                ImageOrMetadataAvailable::ImageAvailable { image, .. } => {
                    if let Some(image) = image.as_raster_image() {
                        self.handle_animated_image(node, image);
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
        svg_id: Option<String>,
    ) -> Option<RasterImage> {
        let result = self
            .image_cache
            .rasterize_vector_image(image_id, size, svg_id);
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

    pub(crate) fn queue_svg_element_for_serialization(&self, element: ServoLayoutNode<'_>) {
        self.pending_svg_elements_for_serialization
            .lock()
            .push(element.opaque().into())
    }

    pub(crate) fn resolve_image<'a>(
        &self,
        node: Option<OpaqueNode>,
        image: &'a Image,
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
                    InternalRequest::No,
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
