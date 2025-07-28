/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use base::id::{BrowsingContextId, PipelineId};
use data_url::DataUrl;
use embedder_traits::ViewportDetails;
use euclid::{Scale, Size2D};
use layout_api::IFrameSize;
use malloc_size_of_derive::MallocSizeOf;
use net_traits::image_cache::{Image, ImageOrMetadataAvailable, UsePlaceholder};
use script::layout_dom::ServoLayoutNode;
use servo_arc::Arc as ServoArc;
use style::Zero;
use style::computed_values::object_fit::T as ObjectFit;
use style::dom::TNode;
use style::logical_geometry::{Direction, WritingMode};
use style::properties::ComputedValues;
use style::servo::url::ComputedUrl;
use style::values::CSSFloat;
use style::values::computed::image::Image as ComputedImage;
use url::Url;
use webrender_api::ImageKey;

use crate::cell::ArcRefCell;
use crate::context::{LayoutContext, LayoutImageCacheResult};
use crate::dom::NodeExt;
use crate::fragment_tree::{
    BaseFragmentInfo, CollapsedBlockMargins, Fragment, IFrameFragment, ImageFragment,
};
use crate::geom::{LazySize, LogicalVec2, PhysicalPoint, PhysicalRect, PhysicalSize};
use crate::layout_box_base::{CacheableLayoutResult, LayoutBoxBase};
use crate::sizing::{ComputeInlineContentSizes, InlineContentSizesResult};
use crate::style_ext::{AspectRatio, Clamp, ComputedValuesExt, LayoutStyle};
use crate::{ConstraintSpace, ContainingBlock, SizeConstraint};

#[derive(Debug, MallocSizeOf)]
pub(crate) struct ReplacedContents {
    pub kind: ReplacedContentKind,
    natural_size: NaturalSizes,
    base_fragment_info: BaseFragmentInfo,
}

/// The natural dimensions of a replaced element, including a height, width, and
/// aspect ratio.
///
/// * Raster images always have an natural width and height, with 1 image pixel = 1px.
///   The natural ratio should be based on dividing those.
///   See <https://github.com/w3c/csswg-drafts/issues/4572> for the case where either is zero.
///   PNG specifically disallows this but I (SimonSapin) am not sure about other formats.
///
/// * Form controls have both natural width and height **but no natural ratio**.
///   See <https://github.com/w3c/csswg-drafts/issues/1044> and
///   <https://drafts.csswg.org/css-images/#natural-dimensions> “In general, […]”
///
/// * For SVG, see <https://svgwg.org/svg2-draft/coords.html#SizingSVGInCSS>
///   and again <https://github.com/w3c/csswg-drafts/issues/4572>.
///
/// * IFrames do not have natural width and height or natural ratio according
///   to <https://drafts.csswg.org/css-images/#intrinsic-dimensions>.
#[derive(Debug, MallocSizeOf)]
pub(crate) struct NaturalSizes {
    pub width: Option<Au>,
    pub height: Option<Au>,
    pub ratio: Option<CSSFloat>,
}

impl NaturalSizes {
    pub(crate) fn from_width_and_height(width: f32, height: f32) -> Self {
        // https://drafts.csswg.org/css-images/#natural-aspect-ratio:
        // "If an object has a degenerate natural aspect ratio (at least one part being
        // zero or infinity), it is treated as having no natural aspect ratio.""
        let ratio = if width.is_normal() && height.is_normal() {
            Some(width / height)
        } else {
            None
        };

        Self {
            width: Some(Au::from_f32_px(width)),
            height: Some(Au::from_f32_px(height)),
            ratio,
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            width: None,
            height: None,
            ratio: None,
        }
    }
}

#[derive(Debug, MallocSizeOf)]
pub(crate) struct CanvasInfo {
    pub source: Option<ImageKey>,
}

#[derive(Debug, MallocSizeOf)]
pub(crate) struct IFrameInfo {
    pub pipeline_id: PipelineId,
    pub browsing_context_id: BrowsingContextId,
}

#[derive(Debug, MallocSizeOf)]
pub(crate) struct VideoInfo {
    pub image_key: webrender_api::ImageKey,
}

#[derive(Debug, MallocSizeOf)]
pub(crate) enum ReplacedContentKind {
    Image(Option<Image>),
    IFrame(IFrameInfo),
    Canvas(CanvasInfo),
    Video(Option<VideoInfo>),
}

impl ReplacedContents {
    pub fn for_element(element: ServoLayoutNode<'_>, context: &LayoutContext) -> Option<Self> {
        if let Some(ref data_attribute_string) = element.as_typeless_object_with_data_attribute() {
            if let Some(url) = try_to_parse_image_data_url(data_attribute_string) {
                return Self::from_image_url(
                    element,
                    context,
                    &ComputedUrl::Valid(ServoArc::new(url)),
                );
            }
        }

        let (kind, natural_size_in_dots) = {
            if let Some((image, natural_size_in_dots)) = element.as_image() {
                (
                    ReplacedContentKind::Image(image),
                    Some(natural_size_in_dots),
                )
            } else if let Some((canvas_info, natural_size_in_dots)) = element.as_canvas() {
                (
                    ReplacedContentKind::Canvas(canvas_info),
                    Some(natural_size_in_dots),
                )
            } else if let Some((pipeline_id, browsing_context_id)) = element.as_iframe() {
                (
                    ReplacedContentKind::IFrame(IFrameInfo {
                        pipeline_id,
                        browsing_context_id,
                    }),
                    None,
                )
            } else if let Some((image_key, natural_size_in_dots)) = element.as_video() {
                (
                    ReplacedContentKind::Video(image_key.map(|key| VideoInfo { image_key: key })),
                    natural_size_in_dots,
                )
            } else {
                return None;
            }
        };

        if let ReplacedContentKind::Image(Some(Image::Raster(ref image))) = kind {
            context
                .image_resolver
                .handle_animated_image(element.opaque(), image.clone());
        }

        let natural_size = if let Some(naturalc_size_in_dots) = natural_size_in_dots {
            // FIXME: should 'image-resolution' (when implemented) be used *instead* of
            // `script::dom::htmlimageelement::ImageRequest::current_pixel_density`?
            // https://drafts.csswg.org/css-images-4/#the-image-resolution
            let dppx = 1.0;
            let width = (naturalc_size_in_dots.width as CSSFloat) / dppx;
            let height = (naturalc_size_in_dots.height as CSSFloat) / dppx;
            NaturalSizes::from_width_and_height(width, height)
        } else {
            NaturalSizes::empty()
        };

        let base_fragment_info = BaseFragmentInfo::new_for_node(element.opaque());
        Some(Self {
            kind,
            natural_size,
            base_fragment_info,
        })
    }

    pub fn from_image_url(
        element: ServoLayoutNode<'_>,
        context: &LayoutContext,
        image_url: &ComputedUrl,
    ) -> Option<Self> {
        if let ComputedUrl::Valid(image_url) = image_url {
            let (image, width, height) = match context.image_resolver.get_or_request_image_or_meta(
                element.opaque(),
                image_url.clone().into(),
                UsePlaceholder::No,
            ) {
                LayoutImageCacheResult::DataAvailable(img_or_meta) => match img_or_meta {
                    ImageOrMetadataAvailable::ImageAvailable { image, .. } => {
                        let metadata = image.metadata();
                        (
                            Some(image.clone()),
                            metadata.width as f32,
                            metadata.height as f32,
                        )
                    },
                    ImageOrMetadataAvailable::MetadataAvailable(metadata, _id) => {
                        (None, metadata.width as f32, metadata.height as f32)
                    },
                },
                LayoutImageCacheResult::Pending | LayoutImageCacheResult::LoadError => return None,
            };

            return Some(Self {
                kind: ReplacedContentKind::Image(image),
                natural_size: NaturalSizes::from_width_and_height(width, height),
                base_fragment_info: BaseFragmentInfo::new_for_node(element.opaque()),
            });
        }
        None
    }

    pub fn from_image(
        element: ServoLayoutNode<'_>,
        context: &LayoutContext,
        image: &ComputedImage,
    ) -> Option<Self> {
        match image {
            ComputedImage::Url(image_url) => Self::from_image_url(element, context, image_url),
            _ => None, // TODO
        }
    }

    fn inline_size_over_block_size_intrinsic_ratio(
        &self,
        style: &ComputedValues,
    ) -> Option<CSSFloat> {
        self.natural_size.ratio.map(|width_over_height| {
            if style.writing_mode.is_vertical() {
                1. / width_over_height
            } else {
                width_over_height
            }
        })
    }

    #[inline]
    fn content_size(
        &self,
        axis: Direction,
        preferred_aspect_ratio: Option<AspectRatio>,
        get_size_in_opposite_axis: &dyn Fn() -> SizeConstraint,
        get_fallback_size: &dyn Fn() -> Au,
    ) -> Au {
        let Some(ratio) = preferred_aspect_ratio else {
            return get_fallback_size();
        };
        let transfer = |size| ratio.compute_dependent_size(axis, size);
        match get_size_in_opposite_axis() {
            SizeConstraint::Definite(size) => transfer(size),
            SizeConstraint::MinMax(min_size, max_size) => get_fallback_size()
                .clamp_between_extremums(transfer(min_size), max_size.map(transfer)),
        }
    }

    pub fn make_fragments(
        &self,
        layout_context: &LayoutContext,
        style: &ServoArc<ComputedValues>,
        size: PhysicalSize<Au>,
    ) -> Vec<Fragment> {
        let natural_size = PhysicalSize::new(
            self.natural_size.width.unwrap_or(size.width),
            self.natural_size.height.unwrap_or(size.height),
        );

        let object_fit_size = self.natural_size.ratio.map_or(size, |width_over_height| {
            let preserve_aspect_ratio_with_comparison =
                |size: PhysicalSize<Au>, comparison: fn(&Au, &Au) -> bool| {
                    let candidate_width = size.height.scale_by(width_over_height);
                    if comparison(&candidate_width, &size.width) {
                        return PhysicalSize::new(candidate_width, size.height);
                    }

                    let candidate_height = size.width.scale_by(1. / width_over_height);
                    debug_assert!(comparison(&candidate_height, &size.height));
                    PhysicalSize::new(size.width, candidate_height)
                };

            match style.clone_object_fit() {
                ObjectFit::Fill => size,
                ObjectFit::Contain => preserve_aspect_ratio_with_comparison(size, PartialOrd::le),
                ObjectFit::Cover => preserve_aspect_ratio_with_comparison(size, PartialOrd::ge),
                ObjectFit::None => natural_size,
                ObjectFit::ScaleDown => {
                    preserve_aspect_ratio_with_comparison(size.min(natural_size), PartialOrd::le)
                },
            }
        });

        let object_position = style.clone_object_position();
        let horizontal_position = object_position
            .horizontal
            .to_used_value(size.width - object_fit_size.width);
        let vertical_position = object_position
            .vertical
            .to_used_value(size.height - object_fit_size.height);

        let rect = PhysicalRect::new(
            PhysicalPoint::new(horizontal_position, vertical_position),
            object_fit_size,
        );
        let clip = PhysicalRect::new(PhysicalPoint::origin(), size);

        match &self.kind {
            ReplacedContentKind::Image(image) => image
                .as_ref()
                .and_then(|image| match image {
                    Image::Raster(raster_image) => raster_image.id,
                    Image::Vector(vector_image) => {
                        let scale = layout_context.style_context.device_pixel_ratio();
                        let width = object_fit_size.width.scale_by(scale.0).to_px();
                        let height = object_fit_size.height.scale_by(scale.0).to_px();
                        let size = Size2D::new(width, height);
                        let tag = self.base_fragment_info.tag?;
                        layout_context
                            .image_resolver
                            .rasterize_vector_image(vector_image.id, size, tag.node)
                            .and_then(|i| i.id)
                    },
                })
                .map(|image_key| {
                    Fragment::Image(ArcRefCell::new(ImageFragment {
                        base: self.base_fragment_info.into(),
                        style: style.clone(),
                        rect,
                        clip,
                        image_key: Some(image_key),
                    }))
                })
                .into_iter()
                .collect(),
            ReplacedContentKind::Video(video) => {
                vec![Fragment::Image(ArcRefCell::new(ImageFragment {
                    base: self.base_fragment_info.into(),
                    style: style.clone(),
                    rect,
                    clip,
                    image_key: video.as_ref().map(|video| video.image_key),
                }))]
            },
            ReplacedContentKind::IFrame(iframe) => {
                let size = Size2D::new(rect.size.width.to_f32_px(), rect.size.height.to_f32_px());
                let hidpi_scale_factor = layout_context.style_context.device_pixel_ratio();

                layout_context.iframe_sizes.lock().insert(
                    iframe.browsing_context_id,
                    IFrameSize {
                        browsing_context_id: iframe.browsing_context_id,
                        pipeline_id: iframe.pipeline_id,
                        viewport_details: ViewportDetails {
                            size,
                            hidpi_scale_factor: Scale::new(hidpi_scale_factor.0),
                        },
                    },
                );
                vec![Fragment::IFrame(ArcRefCell::new(IFrameFragment {
                    base: self.base_fragment_info.into(),
                    style: style.clone(),
                    pipeline_id: iframe.pipeline_id,
                    rect,
                }))]
            },
            ReplacedContentKind::Canvas(canvas_info) => {
                if self.natural_size.width == Some(Au::zero()) ||
                    self.natural_size.height == Some(Au::zero())
                {
                    return vec![];
                }

                let Some(image_key) = canvas_info.source else {
                    return vec![];
                };

                vec![Fragment::Image(ArcRefCell::new(ImageFragment {
                    base: self.base_fragment_info.into(),
                    style: style.clone(),
                    rect,
                    clip,
                    image_key: Some(image_key),
                }))]
            },
        }
    }

    pub(crate) fn preferred_aspect_ratio(
        &self,
        style: &ComputedValues,
        padding_border_sums: &LogicalVec2<Au>,
    ) -> Option<AspectRatio> {
        style
            .preferred_aspect_ratio(
                self.inline_size_over_block_size_intrinsic_ratio(style),
                padding_border_sums,
            )
            .or_else(|| {
                matches!(self.kind, ReplacedContentKind::Video(_)).then(Self::default_aspect_ratio)
            })
    }

    /// The aspect ratio of the default object sizes.
    /// <https://drafts.csswg.org/css-images-3/#default-object-size>
    pub(crate) fn default_aspect_ratio() -> AspectRatio {
        AspectRatio::from_content_ratio(2.0)
    }

    /// The inline size that would result from combining the natural size
    /// and the default object size, but disregarding the specified size.
    /// <https://drafts.csswg.org/css-images-3/#natural-dimensions>
    /// <https://drafts.csswg.org/css-images-3/#default-object-size>
    /// <https://drafts.csswg.org/css-images-3/#specified-size>
    pub(crate) fn fallback_inline_size(&self, writing_mode: WritingMode) -> Au {
        if writing_mode.is_horizontal() {
            self.natural_size.width.unwrap_or_else(|| Au::from_px(300))
        } else {
            self.natural_size.height.unwrap_or_else(|| Au::from_px(150))
        }
    }

    /// The block size that would result from combining the natural size
    /// and the default object size, but disregarding the specified size.
    /// <https://drafts.csswg.org/css-images-3/#natural-dimensions>
    /// <https://drafts.csswg.org/css-images-3/#default-object-size>
    /// <https://drafts.csswg.org/css-images-3/#specified-size>
    pub(crate) fn fallback_block_size(&self, writing_mode: WritingMode) -> Au {
        if writing_mode.is_horizontal() {
            self.natural_size.height.unwrap_or_else(|| Au::from_px(150))
        } else {
            self.natural_size.width.unwrap_or_else(|| Au::from_px(300))
        }
    }

    #[inline]
    pub(crate) fn layout_style<'a>(&self, base: &'a LayoutBoxBase) -> LayoutStyle<'a> {
        LayoutStyle::Default(&base.style)
    }

    pub(crate) fn layout(
        &self,
        layout_context: &LayoutContext,
        containing_block_for_children: &ContainingBlock,
        preferred_aspect_ratio: Option<AspectRatio>,
        base: &LayoutBoxBase,
        depends_on_block_constraints: bool,
        lazy_block_size: &LazySize,
    ) -> CacheableLayoutResult {
        let writing_mode = base.style.writing_mode;
        let inline_size = containing_block_for_children.size.inline;
        let content_block_size = self.content_size(
            Direction::Block,
            preferred_aspect_ratio,
            &|| SizeConstraint::Definite(inline_size),
            &|| self.fallback_block_size(writing_mode),
        );
        let size = LogicalVec2 {
            inline: inline_size,
            block: lazy_block_size.resolve(|| content_block_size),
        }
        .to_physical_size(writing_mode);
        CacheableLayoutResult {
            baselines: Default::default(),
            collapsible_margins_in_children: CollapsedBlockMargins::zero(),
            content_block_size,
            content_inline_size_for_table: None,
            depends_on_block_constraints,
            fragments: self.make_fragments(layout_context, &base.style, size),
            specific_layout_info: None,
        }
    }
}

impl ComputeInlineContentSizes for ReplacedContents {
    fn compute_inline_content_sizes(
        &self,
        _: &LayoutContext,
        constraint_space: &ConstraintSpace,
    ) -> InlineContentSizesResult {
        let inline_content_size = self.content_size(
            Direction::Inline,
            constraint_space.preferred_aspect_ratio,
            &|| constraint_space.block_size,
            &|| self.fallback_inline_size(constraint_space.writing_mode),
        );
        InlineContentSizesResult {
            sizes: inline_content_size.into(),
            depends_on_block_constraints: constraint_space.preferred_aspect_ratio.is_some(),
        }
    }
}

fn try_to_parse_image_data_url(string: &str) -> Option<Url> {
    if !string.starts_with("data:") {
        return None;
    }
    let data_url = DataUrl::process(string).ok()?;
    let mime_type = data_url.mime_type();
    if mime_type.type_ != "image" {
        return None;
    }

    // TODO: Find a better way to test for supported image formats. Currently this type of check is
    // repeated several places in Servo, but should be centralized somehow.
    if !matches!(
        mime_type.subtype.as_str(),
        "png" | "jpeg" | "gif" | "webp" | "bmp" | "ico"
    ) {
        return None;
    }

    Url::parse(string).ok()
}
