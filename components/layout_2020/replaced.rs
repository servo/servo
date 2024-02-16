/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::sync::{Arc, Mutex};

use app_units::Au;
use canvas_traits::canvas::{CanvasId, CanvasMsg, FromLayoutMsg};
use ipc_channel::ipc::{self, IpcSender};
use msg::constellation_msg::{BrowsingContextId, PipelineId};
use net_traits::image::base::Image;
use net_traits::image_cache::{ImageOrMetadataAvailable, UsePlaceholder};
use serde::Serialize;
use servo_arc::Arc as ServoArc;
use style::properties::ComputedValues;
use style::servo::url::ComputedUrl;
use style::values::computed::image::Image as ComputedImage;
use style::values::computed::{Length, LengthOrAuto};
use style::values::CSSFloat;
use style::Zero;
use webrender_api::ImageKey;

use crate::context::LayoutContext;
use crate::dom::NodeExt;
use crate::fragment_tree::{BaseFragmentInfo, Fragment, IFrameFragment, ImageFragment};
use crate::geom::{LogicalRect, LogicalVec2, PhysicalSize};
use crate::sizing::ContentSizes;
use crate::style_ext::{Clamp, ComputedValuesExt, PaddingBorderMargin};
use crate::ContainingBlock;

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
            width: Some(Length::new(width).into()),
            height: Some(Length::new(height).into()),
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
pub(crate) enum ReplacedContentKind {
    Image(Option<Arc<Image>>),
    IFrame(IFrameInfo),
    Canvas(CanvasInfo),
}

impl ReplacedContent {
    pub fn for_element<'dom>(element: impl NodeExt<'dom>) -> Option<Self> {
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
                            size: size.into(),
                        },
                        image_key,
                    })
                })
                .into_iter()
                .collect(),
            ReplacedContentKind::IFrame(iframe) => {
                vec![Fragment::IFrame(IFrameFragment {
                    base: self.base_fragment_info.into(),
                    style: style.clone(),
                    pipeline_id: iframe.pipeline_id,
                    browsing_context_id: iframe.browsing_context_id,
                    rect: LogicalRect {
                        start_corner: LogicalVec2::zero(),
                        size: size.into(),
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
                        size: size.into(),
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
        box_size: Option<LogicalVec2<LengthOrAuto>>,
        pbm: &PaddingBorderMargin,
    ) -> LogicalVec2<Au> {
        let mode = style.writing_mode;
        let intrinsic_size = self.flow_relative_intrinsic_size(style);
        let intrinsic_ratio = self.inline_size_over_block_size_intrinsic_ratio(style);

        let box_size = box_size.unwrap_or(style.content_box_size(containing_block, pbm));
        let max_box_size = style.content_max_box_size(containing_block, pbm);
        let min_box_size = style
            .content_min_box_size(containing_block, pbm)
            .auto_is(Length::zero);

        let default_object_size = || {
            // FIXME:
            // https://drafts.csswg.org/css-images/#default-object-size
            // “If 300px is too wide to fit the device, UAs should use the width of
            //  the largest rectangle that has a 2:1 ratio and fits the device instead.”
            // “height of the largest rectangle that has a 2:1 ratio, has a height not greater
            //  than 150px, and has a width not greater than the device width.”
            LogicalVec2::from_physical_size(
                &PhysicalSize::new(Length::new(300.), Length::new(150.)),
                mode,
            )
        };
        let clamp = |inline_size: Au, block_size: Au| LogicalVec2 {
            inline: inline_size.clamp_between_extremums(
                min_box_size.inline.into(),
                max_box_size.inline.map(|t| t.into()),
            ),
            block: block_size.clamp_between_extremums(
                min_box_size.block.into(),
                max_box_size.block.map(|t| t.into()),
            ),
        };
        // https://drafts.csswg.org/css2/visudet.html#min-max-widths
        // https://drafts.csswg.org/css2/visudet.html#min-max-heights
        match (box_size.inline, box_size.block) {
            (LengthOrAuto::LengthPercentage(inline), LengthOrAuto::LengthPercentage(block)) => {
                clamp(inline.into(), block.into())
            },
            (LengthOrAuto::LengthPercentage(inline), LengthOrAuto::Auto) => {
                let block = if let Some(i_over_b) = intrinsic_ratio {
                    inline / i_over_b
                } else if let Some(block) = intrinsic_size.block {
                    block.into()
                } else {
                    default_object_size().block
                };
                clamp(inline.into(), block.into())
            },
            (LengthOrAuto::Auto, LengthOrAuto::LengthPercentage(block)) => {
                let inline = if let Some(i_over_b) = intrinsic_ratio {
                    block * i_over_b
                } else if let Some(inline) = intrinsic_size.inline {
                    inline.into()
                } else {
                    default_object_size().inline
                };
                clamp(inline.into(), block.into())
            },
            (LengthOrAuto::Auto, LengthOrAuto::Auto) => {
                let inline_size =
                    match (intrinsic_size.inline, intrinsic_size.block, intrinsic_ratio) {
                        (Some(inline), _, _) => inline,
                        (None, Some(block), Some(i_over_b)) => {
                            // “used height” in CSS 2 is always gonna be the intrinsic one,
                            // since it is available.
                            block.scale_by(i_over_b)
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
                        _ => default_object_size().inline.into(),
                    };
                let block_size = if let Some(block) = intrinsic_size.block {
                    block
                } else if let Some(i_over_b) = intrinsic_ratio {
                    // “used width” in CSS 2 is what we just computed above
                    inline_size.scale_by(1.0 / i_over_b)
                } else {
                    default_object_size().block.into()
                };

                let i_over_b = if let Some(i_over_b) = intrinsic_ratio {
                    i_over_b
                } else {
                    return clamp(inline_size, block_size);
                };

                // https://drafts.csswg.org/css2/visudet.html#min-max-widths
                // “However, for replaced elements with an intrinsic ratio and both
                //  'width' and 'height' specified as 'auto', the algorithm is as follows”
                enum Violation {
                    None,
                    Below(Length),
                    Above(Length),
                }
                let violation = |size: Au, min_size, mut max_size: Option<Length>| {
                    if let Some(max) = max_size.as_mut() {
                        max.max_assign(min_size);
                    }
                    if size < min_size.into() {
                        return Violation::Below(min_size);
                    }
                    match max_size {
                        Some(max_size) if size > max_size.into() => Violation::Above(max_size),
                        _ => Violation::None,
                    }
                };
                match (
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
                        inline: max_inline_size.into(),
                        block: (max_inline_size / i_over_b).max(min_box_size.block).into(),
                    },
                    // Row 3.
                    (Violation::Below(min_inline_size), Violation::None) => LogicalVec2 {
                        inline: min_inline_size.into(),
                        block: (min_inline_size / i_over_b)
                            .clamp_below_max(max_box_size.block)
                            .into(),
                    },
                    // Row 4.
                    (Violation::None, Violation::Above(max_block_size)) => LogicalVec2 {
                        inline: (max_block_size * i_over_b).max(min_box_size.inline).into(),
                        block: max_block_size.into(),
                    },
                    // Row 5.
                    (Violation::None, Violation::Below(min_block_size)) => LogicalVec2 {
                        inline: (min_block_size * i_over_b)
                            .clamp_below_max(max_box_size.inline)
                            .into(),
                        block: min_block_size.into(),
                    },
                    // Rows 6-7.
                    (Violation::Above(max_inline_size), Violation::Above(max_block_size)) => {
                        if max_inline_size.px() / inline_size.to_f32_px() <=
                            max_block_size.px() / block_size.to_f32_px()
                        {
                            // Row 6.
                            LogicalVec2 {
                                inline: max_inline_size.into(),
                                block: (max_inline_size / i_over_b).max(min_box_size.block).into(),
                            }
                        } else {
                            // Row 7.
                            LogicalVec2 {
                                inline: (max_block_size * i_over_b).max(min_box_size.inline).into(),
                                block: max_block_size.into(),
                            }
                        }
                    },
                    // Rows 8-9.
                    (Violation::Below(min_inline_size), Violation::Below(min_block_size)) => {
                        if min_inline_size.px() / inline_size.to_f32_px() <=
                            min_block_size.px() / block_size.to_f32_px()
                        {
                            // Row 8.
                            LogicalVec2 {
                                inline: (min_block_size * i_over_b)
                                    .clamp_below_max(max_box_size.inline)
                                    .into(),
                                block: min_block_size.into(),
                            }
                        } else {
                            // Row 9.
                            LogicalVec2 {
                                inline: min_inline_size.into(),
                                block: (min_inline_size / i_over_b)
                                    .clamp_below_max(max_box_size.block)
                                    .into(),
                            }
                        }
                    },
                    // Row 10.
                    (Violation::Below(min_inline_size), Violation::Above(max_block_size)) => {
                        LogicalVec2 {
                            inline: min_inline_size.into(),
                            block: max_block_size.into(),
                        }
                    },
                    // Row 11.
                    (Violation::Above(max_inline_size), Violation::Below(min_block_size)) => {
                        LogicalVec2 {
                            inline: max_inline_size.into(),
                            block: min_block_size.into(),
                        }
                    },
                }
            },
        }
    }
}
