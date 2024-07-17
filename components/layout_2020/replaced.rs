/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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
use style::logical_geometry::Direction;
use style::properties::ComputedValues;
use style::servo::url::ComputedUrl;
use style::values::computed::image::Image as ComputedImage;
use style::values::generics::length::GenericLengthPercentageOrAuto;
use style::values::CSSFloat;
use style::Zero;
use url::Url;
use webrender_api::ImageKey;

use crate::context::LayoutContext;
use crate::dom::NodeExt;
use crate::fragment_tree::{BaseFragmentInfo, Fragment, IFrameFragment, ImageFragment};
use crate::geom::{LogicalRect, LogicalVec2, PhysicalSize};
use crate::sizing::ContentSizes;
use crate::style_ext::{Clamp, ComputedValuesExt, PaddingBorderMargin};
use crate::{AuOrAuto, ContainingBlock};

#[derive(Debug, Serialize)]
pub(crate) struct ReplacedContent {
    pub kind: ReplacedContentKind,
    intrinsic: IntrinsicSizes,
    base_fragment_info: BaseFragmentInfo,
}

/// * Raster images always have an intrinsic width and height, with 1 image pixel = 1px.
///   The intrinsic ratio should be based on dividing those.
///   See <https://github.com/w3c/csswg-drafts/issues/4572> for the case where either is zero.
///   PNG specifically disallows this but I (SimonSapin) am not sure about other formats.
///
/// * Form controls have both intrinsic width and height **but no intrinsic ratio**.
///   See <https://github.com/w3c/csswg-drafts/issues/1044> and
///   <https://drafts.csswg.org/css-images/#intrinsic-dimensions> “In general, […]”
///
/// * For SVG, see <https://svgwg.org/svg2-draft/coords.html#SizingSVGInCSS>
///   and again <https://github.com/w3c/csswg-drafts/issues/4572>.
///
/// * IFrames do not have intrinsic width and height or intrinsic ratio according
///   to <https://drafts.csswg.org/css-images/#intrinsic-dimensions>.
#[derive(Debug, Serialize)]
pub(crate) struct IntrinsicSizes {
    pub width: Option<Au>,
    pub height: Option<Au>,
    pub ratio: Option<CSSFloat>,
}

impl IntrinsicSizes {
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
    Image(Option<Arc<Mutex<IpcSender<CanvasMsg>>>>),
    WebGPU(ImageKey),
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
    Video(VideoInfo),
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

        let (kind, intrinsic_size_in_dots) = {
            if let Some((image, intrinsic_size_in_dots)) = element.as_image() {
                (
                    ReplacedContentKind::Image(image),
                    Some(intrinsic_size_in_dots),
                )
            } else if let Some((canvas_info, intrinsic_size_in_dots)) = element.as_canvas() {
                (
                    ReplacedContentKind::Canvas(canvas_info),
                    Some(intrinsic_size_in_dots),
                )
            } else if let Some((pipeline_id, browsing_context_id)) = element.as_iframe() {
                (
                    ReplacedContentKind::IFrame(IFrameInfo {
                        pipeline_id,
                        browsing_context_id,
                    }),
                    None,
                )
            } else if let Some((image_key, intrinsic_size_in_dots)) = element.as_video() {
                (
                    ReplacedContentKind::Video(VideoInfo { image_key }),
                    Some(intrinsic_size_in_dots),
                )
            } else {
                return None;
            }
        };

        let intrinsic =
            intrinsic_size_in_dots.map_or_else(IntrinsicSizes::empty, |intrinsic_size_in_dots| {
                // FIXME: should 'image-resolution' (when implemented) be used *instead* of
                // `script::dom::htmlimageelement::ImageRequest::current_pixel_density`?
                // https://drafts.csswg.org/css-images-4/#the-image-resolution
                let dppx = 1.0;
                let width = (intrinsic_size_in_dots.width as CSSFloat) / dppx;
                let height = (intrinsic_size_in_dots.height as CSSFloat) / dppx;
                IntrinsicSizes::from_width_and_height(width, height)
            });

        let base_fragment_info = BaseFragmentInfo::new_for_node(element.opaque());
        Some(Self {
            kind,
            intrinsic,
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
                intrinsic: IntrinsicSizes::from_width_and_height(width, height),
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

    fn flow_relative_intrinsic_size(&self, style: &ComputedValues) -> LogicalVec2<Option<Au>> {
        let intrinsic_size = PhysicalSize::new(self.intrinsic.width, self.intrinsic.height);
        LogicalVec2::from_physical_size(&intrinsic_size, style.writing_mode)
    }

    pub fn inline_size_over_block_size_intrinsic_ratio(
        &self,
        style: &ComputedValues,
    ) -> Option<CSSFloat> {
        self.intrinsic.ratio.map(|width_over_height| {
            if style.writing_mode.is_vertical() {
                1. / width_over_height
            } else {
                width_over_height
            }
        })
    }

    pub fn inline_content_sizes(&self, style: &ComputedValues) -> ContentSizes {
        // FIXME: min/max-content of replaced elements is not defined in
        // https://dbaron.org/css/intrinsic/
        // This seems sensible?
        let inline = self
            .flow_relative_intrinsic_size(style)
            .inline
            .unwrap_or(Au::zero());
        ContentSizes {
            min_content: inline,
            max_content: inline,
        }
    }

    pub fn make_fragments(
        &self,
        style: &ServoArc<ComputedValues>,
        size: LogicalVec2<Au>,
    ) -> Vec<Fragment> {
        match &self.kind {
            ReplacedContentKind::Image(image) => image
                .as_ref()
                .and_then(|image| image.id)
                .map(|image_key| {
                    Fragment::Image(ImageFragment {
                        base: self.base_fragment_info.into(),
                        style: style.clone(),
                        rect: LogicalRect {
                            start_corner: LogicalVec2::zero(),
                            size,
                        },
                        image_key,
                    })
                })
                .into_iter()
                .collect(),
            ReplacedContentKind::Video(video) => vec![Fragment::Image(ImageFragment {
                base: self.base_fragment_info.into(),
                style: style.clone(),
                rect: LogicalRect {
                    start_corner: LogicalVec2::zero(),
                    size,
                },
                image_key: video.image_key,
            })],
            ReplacedContentKind::IFrame(iframe) => {
                vec![Fragment::IFrame(IFrameFragment {
                    base: self.base_fragment_info.into(),
                    style: style.clone(),
                    pipeline_id: iframe.pipeline_id,
                    browsing_context_id: iframe.browsing_context_id,
                    rect: LogicalRect {
                        start_corner: LogicalVec2::zero(),
                        size,
                    },
                })]
            },
            ReplacedContentKind::Canvas(canvas_info) => {
                if self.intrinsic.width == Some(Au::zero()) ||
                    self.intrinsic.height == Some(Au::zero())
                {
                    return vec![];
                }

                let image_key = match canvas_info.source {
                    CanvasSource::WebGL(image_key) => image_key,
                    CanvasSource::WebGPU(image_key) => image_key,
                    CanvasSource::Image(ref ipc_renderer) => match *ipc_renderer {
                        Some(ref ipc_renderer) => {
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
                        None => return vec![],
                    },
                };
                vec![Fragment::Image(ImageFragment {
                    base: self.base_fragment_info.into(),
                    style: style.clone(),
                    rect: LogicalRect {
                        start_corner: LogicalVec2::zero(),
                        size,
                    },
                    image_key,
                })]
            },
        }
    }

    /// <https://drafts.csswg.org/css2/visudet.html#inline-replaced-width>
    /// <https://drafts.csswg.org/css2/visudet.html#inline-replaced-height>
    ///
    /// Also used in other cases, for example
    /// <https://drafts.csswg.org/css2/visudet.html#block-replaced-width>
    pub fn used_size_as_if_inline_element(
        &self,
        containing_block: &ContainingBlock,
        style: &ComputedValues,
        box_size: Option<LogicalVec2<AuOrAuto>>,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<Au> {
        let mode = style.writing_mode;
        let intrinsic_size = self.flow_relative_intrinsic_size(style);
        let intrinsic_ratio = style.preferred_aspect_ratio(
            self.inline_size_over_block_size_intrinsic_ratio(style),
            containing_block,
        );

        let box_size = box_size.unwrap_or(
            style
                .content_box_size(containing_block, pbm)
                // We need to clamp to zero here to obtain the proper aspect
                // ratio when box-sizing is border-box and the inner box size
                // would otherwise be negative.
                .map(|v| v.map(|v| Au::from(v).max(Au::zero()))),
        );
        let max_box_size = style
            .content_max_box_size(containing_block, pbm)
            .map(|v| v.map(Au::from));
        let min_box_size = style
            .content_min_box_size(containing_block, pbm)
            .map(|v| v.map(Au::from))
            .auto_is(Au::zero);

        let default_object_size = || {
            // FIXME:
            // https://drafts.csswg.org/css-images/#default-object-size
            // “If 300px is too wide to fit the device, UAs should use the width of
            //  the largest rectangle that has a 2:1 ratio and fits the device instead.”
            // “height of the largest rectangle that has a 2:1 ratio, has a height not greater
            //  than 150px, and has a width not greater than the device width.”
            LogicalVec2::from_physical_size(
                &PhysicalSize::new(Au::from_px(300), Au::from_px(150)),
                mode,
            )
        };

        let get_tentative_size = |LogicalVec2 { inline, block }| -> LogicalVec2<Au> {
            match (inline, block) {
                (AuOrAuto::LengthPercentage(inline), AuOrAuto::LengthPercentage(block)) => {
                    LogicalVec2 { inline, block }
                },
                (AuOrAuto::LengthPercentage(inline), AuOrAuto::Auto) => {
                    let block = if let Some(ratio) = intrinsic_ratio {
                        ratio.compute_dependent_size(Direction::Block, inline)
                    } else if let Some(block) = intrinsic_size.block {
                        block
                    } else {
                        default_object_size().block
                    };
                    LogicalVec2 { inline, block }
                },
                (AuOrAuto::Auto, AuOrAuto::LengthPercentage(block)) => {
                    let inline = if let Some(ratio) = intrinsic_ratio {
                        ratio.compute_dependent_size(Direction::Inline, block)
                    } else if let Some(inline) = intrinsic_size.inline {
                        inline
                    } else {
                        default_object_size().inline
                    };
                    LogicalVec2 { inline, block }
                },
                (AuOrAuto::Auto, AuOrAuto::Auto) => {
                    let inline_size =
                        match (intrinsic_size.inline, intrinsic_size.block, intrinsic_ratio) {
                            (Some(inline), _, _) => inline,
                            (None, Some(block), Some(ratio)) => {
                                // “used height” in CSS 2 is always gonna be the intrinsic one,
                                // since it is available.
                                ratio.compute_dependent_size(Direction::Inline, block)
                            },
                            // FIXME
                            //
                            // “If 'height' and 'width' both have computed values of 'auto'
                            // and the element has an intrinsic ratio but no intrinsic height or width,
                            // […]”
                            //
                            // In this `match` expression this would be an additional arm here:
                            //
                            // ```
                            // (Vec2 { inline: None, block: None }, Some(_)) => {…}
                            // ```
                            //
                            // “[…] then the used value of 'width' is undefined in CSS 2.
                            // However, it is suggested that, if the containing block's width
                            // does not itself depend on the replaced element's width,
                            // then the used value of 'width' is calculated from the constraint
                            // equation used for block-level, non-replaced elements in normal flow.”
                            _ => default_object_size().inline,
                        };
                    let block_size = if let Some(block) = intrinsic_size.block {
                        block
                    } else if let Some(ratio) = intrinsic_ratio {
                        // “used width” in CSS 2 is what we just computed above
                        ratio.compute_dependent_size(Direction::Block, inline_size)
                    } else {
                        default_object_size().block
                    };
                    LogicalVec2 {
                        inline: inline_size,
                        block: block_size,
                    }
                },
            }
        };

        // https://drafts.csswg.org/css2/visudet.html#min-max-widths
        // “However, for replaced elements with an intrinsic ratio and both
        //  'width' and 'height' specified as 'auto', the algorithm is as follows”
        if let (AuOrAuto::Auto, AuOrAuto::Auto, Some(ratio)) =
            (box_size.inline, box_size.block, intrinsic_ratio)
        {
            let LogicalVec2 {
                inline: inline_size,
                block: block_size,
            } = get_tentative_size(box_size);
            enum Violation {
                None,
                Below(Au),
                Above(Au),
            }
            let violation = |size: Au, min_size: Au, mut max_size: Option<Au>| {
                if let Some(max) = max_size.as_mut() {
                    max.max_assign(min_size);
                }
                if size < min_size {
                    return Violation::Below(min_size);
                }
                match max_size {
                    Some(max_size) if size > max_size => Violation::Above(max_size),
                    _ => Violation::None,
                }
            };
            return match (
                violation(inline_size, min_box_size.inline, max_box_size.inline),
                violation(block_size, min_box_size.block, max_box_size.block),
            ) {
                // Row 1.
                (Violation::None, Violation::None) => LogicalVec2 {
                    inline: inline_size,
                    block: block_size,
                },
                // Row 2.
                (Violation::Above(max_inline_size), Violation::None) => LogicalVec2 {
                    inline: max_inline_size,
                    block: ratio
                        .compute_dependent_size(Direction::Block, max_inline_size)
                        .max(min_box_size.block),
                },
                // Row 3.
                (Violation::Below(min_inline_size), Violation::None) => LogicalVec2 {
                    inline: min_inline_size,
                    block: ratio
                        .compute_dependent_size(Direction::Block, min_inline_size)
                        .clamp_below_max(max_box_size.block),
                },
                // Row 4.
                (Violation::None, Violation::Above(max_block_size)) => LogicalVec2 {
                    inline: ratio
                        .compute_dependent_size(Direction::Inline, max_block_size)
                        .max(min_box_size.inline),
                    block: max_block_size,
                },
                // Row 5.
                (Violation::None, Violation::Below(min_block_size)) => LogicalVec2 {
                    inline: ratio
                        .compute_dependent_size(Direction::Inline, min_block_size)
                        .clamp_below_max(max_box_size.inline),
                    block: min_block_size,
                },
                // Rows 6-7.
                (Violation::Above(max_inline_size), Violation::Above(max_block_size)) => {
                    if max_inline_size.0 * block_size.0 <= max_block_size.0 * inline_size.0 {
                        // Row 6.
                        LogicalVec2 {
                            inline: max_inline_size,
                            block: ratio
                                .compute_dependent_size(Direction::Block, max_inline_size)
                                .max(min_box_size.block),
                        }
                    } else {
                        // Row 7.
                        LogicalVec2 {
                            inline: ratio
                                .compute_dependent_size(Direction::Inline, max_block_size)
                                .max(min_box_size.inline),
                            block: max_block_size,
                        }
                    }
                },
                // Rows 8-9.
                (Violation::Below(min_inline_size), Violation::Below(min_block_size)) => {
                    if min_inline_size.0 * block_size.0 <= min_block_size.0 * inline_size.0 {
                        // Row 8.
                        LogicalVec2 {
                            inline: ratio
                                .compute_dependent_size(Direction::Inline, min_block_size)
                                .clamp_below_max(max_box_size.inline),
                            block: min_block_size,
                        }
                    } else {
                        // Row 9.
                        LogicalVec2 {
                            inline: min_inline_size,
                            block: ratio
                                .compute_dependent_size(Direction::Block, min_inline_size)
                                .clamp_below_max(max_box_size.block),
                        }
                    }
                },
                // Row 10.
                (Violation::Below(min_inline_size), Violation::Above(max_block_size)) => {
                    LogicalVec2 {
                        inline: min_inline_size,
                        block: max_block_size,
                    }
                },
                // Row 11.
                (Violation::Above(max_inline_size), Violation::Below(min_block_size)) => {
                    LogicalVec2 {
                        inline: max_inline_size,
                        block: min_block_size,
                    }
                },
            };
        }

        // https://drafts.csswg.org/css2/#min-max-widths "The following algorithm describes how the two properties
        // influence the used value of the width property:
        //
        // 1. The tentative used width is calculated (without min-width and max-width) following the rules under
        //    "Calculating widths and margins" above.
        // 2. If the tentative used width is greater than max-width, the rules above are applied again, but this time
        //    using the computed value of max-width as the computed value for width.
        // 3. If the resulting width is smaller than min-width, the rules above are applied again, but this time using
        //    the value of min-width as the computed value for width."
        let mut tentative_size = get_tentative_size(box_size);

        // Create an inline/block size vector from the given clamped inline and block sizes if they are provided,
        // falling back to the regular box size if they are not
        let size_from_maybe_clamped =
            |(clamped_inline, clamped_block): (Option<Au>, Option<Au>)| {
                let clamped_inline = clamped_inline
                    .map(GenericLengthPercentageOrAuto::LengthPercentage)
                    .unwrap_or(box_size.inline);
                let clamped_block = clamped_block
                    .map(GenericLengthPercentageOrAuto::LengthPercentage)
                    .unwrap_or(box_size.block);
                LogicalVec2 {
                    inline: clamped_inline,
                    block: clamped_block,
                }
            };

        let clamped_max = (
            max_box_size
                .inline
                .filter(|max_inline_size| tentative_size.inline > *max_inline_size),
            max_box_size
                .block
                .filter(|max_block_size| tentative_size.block > *max_block_size),
        );

        if clamped_max.0.is_some() || clamped_max.1.is_some() {
            tentative_size = get_tentative_size(size_from_maybe_clamped(clamped_max));
        }

        let clamped_min = (
            Some(min_box_size.inline)
                .filter(|min_inline_size| tentative_size.inline < *min_inline_size),
            Some(min_box_size.block)
                .filter(|min_block_size| tentative_size.block < *min_block_size),
        );

        if clamped_min.0.is_some() || clamped_min.1.is_some() {
            tentative_size = get_tentative_size(size_from_maybe_clamped(clamped_min));
        }

        tentative_size
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
