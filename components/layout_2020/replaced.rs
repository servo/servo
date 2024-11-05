/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::LazyCell;
use std::fmt;
use std::sync::{Arc, Mutex};

use app_units::Au;
use base::id::{BrowsingContextId, PipelineId};
use canvas_traits::canvas::{CanvasId, CanvasMsg, FromLayoutMsg};
use data_url::DataUrl;
use ipc_channel::ipc::{self, IpcSender};
use net_traits::image_cache::{ImageOrMetadataAvailable, UsePlaceholder};
use pixels::Image;
use serde::Serialize;
use servo_arc::Arc as ServoArc;
use style::computed_values::object_fit::T as ObjectFit;
use style::logical_geometry::{Direction, WritingMode};
use style::properties::ComputedValues;
use style::servo::url::ComputedUrl;
use style::values::computed::image::Image as ComputedImage;
use style::values::CSSFloat;
use style::Zero;
use url::Url;
use webrender_api::ImageKey;

use crate::context::LayoutContext;
use crate::dom::NodeExt;
use crate::fragment_tree::{BaseFragmentInfo, Fragment, IFrameFragment, ImageFragment};
use crate::geom::{LogicalVec2, PhysicalPoint, PhysicalRect, PhysicalSize, Size};
use crate::sizing::InlineContentSizesResult;
use crate::style_ext::{AspectRatio, Clamp, ComputedValuesExt, ContentBoxSizesAndPBM};
use crate::{AuOrAuto, ContainingBlock, IndefiniteContainingBlock};

#[derive(Debug, Serialize)]
pub(crate) struct ReplacedContent {
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
#[derive(Debug, Serialize)]
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

#[derive(Serialize)]
pub(crate) enum CanvasSource {
    WebGL(ImageKey),
    Image(Arc<Mutex<IpcSender<CanvasMsg>>>),
    WebGPU(ImageKey),
    /// transparent black
    Empty,
}

impl fmt::Debug for CanvasSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                CanvasSource::WebGL(_) => "WebGL",
                CanvasSource::Image(_) => "Image",
                CanvasSource::WebGPU(_) => "WebGPU",
                CanvasSource::Empty => "Empty",
            }
        )
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct CanvasInfo {
    pub source: CanvasSource,
    pub canvas_id: CanvasId,
}

#[derive(Debug, Serialize)]
pub(crate) struct IFrameInfo {
    pub pipeline_id: PipelineId,
    pub browsing_context_id: BrowsingContextId,
}

#[derive(Debug, Serialize)]
pub(crate) struct VideoInfo {
    pub image_key: webrender_api::ImageKey,
}

#[derive(Debug, Serialize)]
pub(crate) enum ReplacedContentKind {
    Image(Option<Arc<Image>>),
    IFrame(IFrameInfo),
    Canvas(CanvasInfo),
    Video(Option<VideoInfo>),
}

impl ReplacedContent {
    pub fn for_element<'dom>(element: impl NodeExt<'dom>, context: &LayoutContext) -> Option<Self> {
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

    pub fn from_image_url<'dom>(
        element: impl NodeExt<'dom>,
        context: &LayoutContext,
        image_url: &ComputedUrl,
    ) -> Option<Self> {
        if let ComputedUrl::Valid(image_url) = image_url {
            let (image, width, height) = match context.get_or_request_image_or_meta(
                element.opaque(),
                image_url.clone().into(),
                UsePlaceholder::No,
            ) {
                Some(ImageOrMetadataAvailable::ImageAvailable { image, .. }) => {
                    (Some(image.clone()), image.width as f32, image.height as f32)
                },
                Some(ImageOrMetadataAvailable::MetadataAvailable(metadata)) => {
                    (None, metadata.width as f32, metadata.height as f32)
                },
                None => return None,
            };

            return Some(Self {
                kind: ReplacedContentKind::Image(image),
                natural_size: NaturalSizes::from_width_and_height(width, height),
                base_fragment_info: BaseFragmentInfo::new_for_node(element.opaque()),
            });
        }
        None
    }

    pub fn from_image<'dom>(
        element: impl NodeExt<'dom>,
        context: &LayoutContext,
        image: &ComputedImage,
    ) -> Option<Self> {
        match image {
            ComputedImage::Url(image_url) => Self::from_image_url(element, context, image_url),
            _ => None, // TODO
        }
    }

    fn flow_relative_natural_size(&self, writing_mode: WritingMode) -> LogicalVec2<Option<Au>> {
        let natural_size = PhysicalSize::new(self.natural_size.width, self.natural_size.height);
        LogicalVec2::from_physical_size(&natural_size, writing_mode)
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

    pub fn inline_content_sizes(
        &self,
        _: &LayoutContext,
        containing_block_for_children: &IndefiniteContainingBlock,
        preferred_aspect_ratio: Option<AspectRatio>,
    ) -> InlineContentSizesResult {
        // FIXME: min/max-content of replaced elements is not defined in
        // https://dbaron.org/css/intrinsic/
        // This seems sensible?
        let block_size = containing_block_for_children.size.block;
        match (block_size, preferred_aspect_ratio) {
            (AuOrAuto::LengthPercentage(block_size), Some(ratio)) => InlineContentSizesResult {
                sizes: ratio
                    .compute_dependent_size(Direction::Inline, block_size)
                    .into(),
                depends_on_block_constraints: true,
            },
            _ => {
                let writing_mode = containing_block_for_children.writing_mode;
                InlineContentSizesResult {
                    sizes: self
                        .flow_relative_natural_size(writing_mode)
                        .inline
                        .unwrap_or_else(|| {
                            Self::flow_relative_default_object_size(writing_mode).inline
                        })
                        .into(),
                    depends_on_block_constraints: false,
                }
            },
        }
    }

    pub fn make_fragments(
        &self,
        style: &ServoArc<ComputedValues>,
        containing_block: &ContainingBlock,
        size: PhysicalSize<Au>,
    ) -> Vec<Fragment> {
        let aspect_ratio = self.preferred_aspect_ratio(&containing_block.into(), style);
        let natural_size = PhysicalSize::new(
            self.natural_size.width.unwrap_or(size.width),
            self.natural_size.height.unwrap_or(size.height),
        );

        let object_fit_size = aspect_ratio.map_or(size, |aspect_ratio| {
            let preserve_aspect_ratio_with_comparison =
                |size: PhysicalSize<Au>, comparison: fn(&Au, &Au) -> bool| {
                    let (width_axis, height_axis) = if style.writing_mode.is_horizontal() {
                        (Direction::Inline, Direction::Block)
                    } else {
                        (Direction::Block, Direction::Inline)
                    };

                    let candidate_width =
                        aspect_ratio.compute_dependent_size(width_axis, size.height);
                    if comparison(&candidate_width, &size.width) {
                        return PhysicalSize::new(candidate_width, size.height);
                    }

                    let candidate_height =
                        aspect_ratio.compute_dependent_size(height_axis, size.width);
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
                .and_then(|image| image.id)
                .map(|image_key| {
                    Fragment::Image(ImageFragment {
                        base: self.base_fragment_info.into(),
                        style: style.clone(),
                        rect,
                        clip,
                        image_key: Some(image_key),
                    })
                })
                .into_iter()
                .collect(),
            ReplacedContentKind::Video(video) => vec![Fragment::Image(ImageFragment {
                base: self.base_fragment_info.into(),
                style: style.clone(),
                rect,
                clip,
                image_key: video.as_ref().map(|video| video.image_key),
            })],
            ReplacedContentKind::IFrame(iframe) => {
                vec![Fragment::IFrame(IFrameFragment {
                    base: self.base_fragment_info.into(),
                    style: style.clone(),
                    pipeline_id: iframe.pipeline_id,
                    browsing_context_id: iframe.browsing_context_id,
                    rect,
                })]
            },
            ReplacedContentKind::Canvas(canvas_info) => {
                if self.natural_size.width == Some(Au::zero()) ||
                    self.natural_size.height == Some(Au::zero())
                {
                    return vec![];
                }

                let image_key = match canvas_info.source {
                    CanvasSource::WebGL(image_key) => image_key,
                    CanvasSource::WebGPU(image_key) => image_key,
                    CanvasSource::Image(ref ipc_renderer) => {
                        let ipc_renderer = ipc_renderer.lock().unwrap();
                        let (sender, receiver) = ipc::channel().unwrap();
                        ipc_renderer
                            .send(CanvasMsg::FromLayout(
                                FromLayoutMsg::SendData(sender),
                                canvas_info.canvas_id,
                            ))
                            .unwrap();
                        receiver.recv().unwrap().image_key
                    },
                    CanvasSource::Empty => return vec![],
                };
                vec![Fragment::Image(ImageFragment {
                    base: self.base_fragment_info.into(),
                    style: style.clone(),
                    rect,
                    clip,
                    image_key: Some(image_key),
                })]
            },
        }
    }

    pub(crate) fn preferred_aspect_ratio(
        &self,
        containing_block: &IndefiniteContainingBlock,
        style: &ComputedValues,
    ) -> Option<AspectRatio> {
        style
            .preferred_aspect_ratio(
                self.inline_size_over_block_size_intrinsic_ratio(style),
                containing_block,
            )
            .or_else(|| {
                matches!(self.kind, ReplacedContentKind::Video(_)).then(|| {
                    let size = Self::default_object_size();
                    AspectRatio::from_content_ratio(
                        size.width.to_f32_px() / size.height.to_f32_px(),
                    )
                })
            })
    }

    /// <https://drafts.csswg.org/css2/visudet.html#inline-replaced-width>
    /// <https://drafts.csswg.org/css2/visudet.html#inline-replaced-height>
    ///
    /// Also used in other cases, for example
    /// <https://drafts.csswg.org/css2/visudet.html#block-replaced-width>
    pub(crate) fn used_size_as_if_inline_element(
        &self,
        containing_block: &ContainingBlock,
        style: &ComputedValues,
        content_box_sizes_and_pbm: &ContentBoxSizesAndPBM,
    ) -> LogicalVec2<Au> {
        let pbm = &content_box_sizes_and_pbm.pbm;
        self.used_size_as_if_inline_element_from_content_box_sizes(
            containing_block,
            style,
            content_box_sizes_and_pbm.content_box_size,
            content_box_sizes_and_pbm.content_min_box_size,
            content_box_sizes_and_pbm.content_max_box_size,
            pbm.padding_border_sums + pbm.margin.auto_is(Au::zero).sum(),
        )
    }

    pub(crate) fn default_object_size() -> PhysicalSize<Au> {
        // FIXME:
        // https://drafts.csswg.org/css-images/#default-object-size
        // “If 300px is too wide to fit the device, UAs should use the width of
        //  the largest rectangle that has a 2:1 ratio and fits the device instead.”
        // “height of the largest rectangle that has a 2:1 ratio, has a height not greater
        //  than 150px, and has a width not greater than the device width.”
        PhysicalSize::new(Au::from_px(300), Au::from_px(150))
    }

    pub(crate) fn flow_relative_default_object_size(writing_mode: WritingMode) -> LogicalVec2<Au> {
        LogicalVec2::from_physical_size(&Self::default_object_size(), writing_mode)
    }

    /// <https://drafts.csswg.org/css2/visudet.html#inline-replaced-width>
    /// <https://drafts.csswg.org/css2/visudet.html#inline-replaced-height>
    ///
    /// Also used in other cases, for example
    /// <https://drafts.csswg.org/css2/visudet.html#block-replaced-width>
    ///
    /// The logic differs from CSS2 in order to properly handle `aspect-ratio` and keyword sizes.
    /// Each axis can have preferred, min and max sizing constraints, plus constraints transferred
    /// from the other axis if there is an aspect ratio, plus a natural and default size.
    /// In case of conflict, the order of precedence (from highest to lowest) is:
    /// 1. Non-transferred min constraint
    /// 2. Non-transferred max constraint
    /// 3. Non-transferred preferred constraint
    /// 4. Transferred min constraint
    /// 5. Transferred max constraint
    /// 6. Transferred preferred constraint
    /// 7. Natural size
    /// 8. Default object size
    ///
    /// <https://drafts.csswg.org/css-sizing-4/#aspect-ratio-size-transfers>
    /// <https://github.com/w3c/csswg-drafts/issues/6071#issuecomment-2243986313>
    pub(crate) fn used_size_as_if_inline_element_from_content_box_sizes(
        &self,
        containing_block: &ContainingBlock,
        style: &ComputedValues,
        box_size: LogicalVec2<Size<Au>>,
        min_box_size: LogicalVec2<Size<Au>>,
        max_box_size: LogicalVec2<Size<Au>>,
        pbm_sums: LogicalVec2<Au>,
    ) -> LogicalVec2<Au> {
        // <https://drafts.csswg.org/css-sizing-4/#preferred-aspect-ratio>
        let ratio = self.preferred_aspect_ratio(&containing_block.into(), style);

        // <https://drafts.csswg.org/css-images-3/#natural-dimensions>
        // <https://drafts.csswg.org/css-images-3/#default-object-size>
        let writing_mode = style.writing_mode;
        let natural_size = LazyCell::new(|| self.flow_relative_natural_size(writing_mode));
        let default_object_size =
            LazyCell::new(|| Self::flow_relative_default_object_size(writing_mode));
        let get_inline_fallback_size = || {
            natural_size
                .inline
                .unwrap_or_else(|| default_object_size.inline)
        };
        let get_block_fallback_size = || {
            natural_size
                .block
                .unwrap_or_else(|| default_object_size.block)
        };

        // <https://drafts.csswg.org/css-sizing-4/#stretch-fit-sizing>
        let inline_stretch_size = Au::zero().max(containing_block.inline_size - pbm_sums.inline);
        let block_stretch_size = containing_block
            .block_size
            .non_auto()
            .map(|block_size| Au::zero().max(block_size - pbm_sums.block));

        // <https://drafts.csswg.org/css-sizing-3/#intrinsic-sizes>
        // FIXME: Use ReplacedContent::inline_content_sizes() once it's fixed to correctly handle
        // min and max constraints.
        let inline_content_size = LazyCell::new(|| {
            let Some(ratio) = ratio else {
                return get_inline_fallback_size();
            };
            let block_stretch_size = block_stretch_size.unwrap_or_else(get_block_fallback_size);
            let transfer = |size| ratio.compute_dependent_size(Direction::Inline, size);
            let min = transfer(
                min_box_size
                    .block
                    .maybe_resolve_extrinsic(Some(block_stretch_size))
                    .unwrap_or_default(),
            );
            let max = max_box_size
                .block
                .maybe_resolve_extrinsic(Some(block_stretch_size))
                .map(transfer);
            box_size
                .block
                .maybe_resolve_extrinsic(Some(block_stretch_size))
                .map_or_else(get_inline_fallback_size, transfer)
                .clamp_between_extremums(min, max)
        });
        let block_content_size = LazyCell::new(|| {
            let Some(ratio) = ratio else {
                return get_block_fallback_size();
            };
            let mut get_inline_content_size = || (*inline_content_size).into();
            let transfer = |size| ratio.compute_dependent_size(Direction::Block, size);
            let min = transfer(
                min_box_size
                    .inline
                    .resolve_non_initial(inline_stretch_size, &mut get_inline_content_size)
                    .unwrap_or_default(),
            );
            let max = max_box_size
                .inline
                .resolve_non_initial(inline_stretch_size, &mut get_inline_content_size)
                .map(transfer);
            box_size
                .inline
                .maybe_resolve_extrinsic(Some(inline_stretch_size))
                .map_or_else(get_block_fallback_size, transfer)
                .clamp_between_extremums(min, max)
        });
        let mut get_inline_content_size = || (*inline_content_size).into();
        let mut get_block_content_size = || (*block_content_size).into();
        let block_stretch_size = block_stretch_size.unwrap_or_else(|| *block_content_size);

        // <https://drafts.csswg.org/css-sizing-3/#sizing-properties>
        let preferred_inline = box_size.inline.resolve(
            Size::FitContent,
            inline_stretch_size,
            &mut get_inline_content_size,
        );
        let preferred_block = box_size.block.resolve(
            Size::FitContent,
            block_stretch_size,
            &mut get_block_content_size,
        );
        let min_inline = min_box_size
            .inline
            .resolve_non_initial(inline_stretch_size, &mut get_inline_content_size)
            .unwrap_or_default();
        let min_block = min_box_size
            .block
            .resolve_non_initial(block_stretch_size, &mut get_block_content_size)
            .unwrap_or_default();
        let max_inline = max_box_size
            .inline
            .resolve_non_initial(inline_stretch_size, &mut get_inline_content_size);
        let max_block = max_box_size
            .block
            .resolve_non_initial(block_stretch_size, &mut get_block_content_size);
        LogicalVec2 {
            inline: preferred_inline.clamp_between_extremums(min_inline, max_inline),
            block: preferred_block.clamp_between_extremums(min_block, max_block),
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
