/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// TODO(gw): This contains helper traits and implementations for converting Servo display lists
//           into WebRender display lists. In the future, this step should be completely removed.
//           This might be achieved by sharing types between WR and Servo display lists, or
//           completely converting layout to directly generate WebRender display lists, for example.

use app_units::Au;
use euclid::{Point2D, Vector2D, Rect, SideOffsets2D, Size2D};
use gfx::display_list::{BorderDetails, BorderRadii, BoxShadowClipMode, ClipScrollNode};
use gfx::display_list::{ClipScrollNodeIndex, ClipScrollNodeType, ClippingRegion, DisplayItem};
use gfx::display_list::{DisplayList, StackingContextType};
use msg::constellation_msg::PipelineId;
use style::computed_values::image_rendering::T as ImageRendering;
use style::computed_values::mix_blend_mode::T as MixBlendMode;
use style::computed_values::transform_style::T as TransformStyle;
use style::values::computed::{BorderStyle, Filter};
use style::values::generics::effects::Filter as GenericFilter;
use webrender_api::{self, ClipAndScrollInfo, ClipId, ClipMode, ComplexClipRegion};
use webrender_api::{DisplayListBuilder, ExtendMode, LayoutTransform};

pub trait WebRenderDisplayListConverter {
    fn convert_to_webrender(&self, pipeline_id: PipelineId) -> DisplayListBuilder;
}

trait WebRenderDisplayItemConverter {
    fn prim_info(&self) -> webrender_api::LayoutPrimitiveInfo;
    fn convert_to_webrender(
        &self,
        builder: &mut DisplayListBuilder,
        clip_scroll_nodes: &[ClipScrollNode],
        clip_ids: &mut Vec<Option<ClipId>>,
        current_clip_and_scroll_info: &mut ClipAndScrollInfo
    );
}

trait ToBorderStyle {
    fn to_border_style(&self) -> webrender_api::BorderStyle;
}

impl ToBorderStyle for BorderStyle {
    fn to_border_style(&self) -> webrender_api::BorderStyle {
        match *self {
            BorderStyle::None => webrender_api::BorderStyle::None,
            BorderStyle::Solid => webrender_api::BorderStyle::Solid,
            BorderStyle::Double => webrender_api::BorderStyle::Double,
            BorderStyle::Dotted => webrender_api::BorderStyle::Dotted,
            BorderStyle::Dashed => webrender_api::BorderStyle::Dashed,
            BorderStyle::Hidden => webrender_api::BorderStyle::Hidden,
            BorderStyle::Groove => webrender_api::BorderStyle::Groove,
            BorderStyle::Ridge => webrender_api::BorderStyle::Ridge,
            BorderStyle::Inset => webrender_api::BorderStyle::Inset,
            BorderStyle::Outset => webrender_api::BorderStyle::Outset,
        }
    }
}

trait ToBorderWidths {
    fn to_border_widths(&self) -> webrender_api::BorderWidths;
}

impl ToBorderWidths for SideOffsets2D<Au> {
    fn to_border_widths(&self) -> webrender_api::BorderWidths {
        webrender_api::BorderWidths {
            left: self.left.to_f32_px(),
            top: self.top.to_f32_px(),
            right: self.right.to_f32_px(),
            bottom: self.bottom.to_f32_px(),
        }
    }
}

trait ToBoxShadowClipMode {
    fn to_clip_mode(&self) -> webrender_api::BoxShadowClipMode;
}

impl ToBoxShadowClipMode for BoxShadowClipMode {
    fn to_clip_mode(&self) -> webrender_api::BoxShadowClipMode {
        match *self {
            BoxShadowClipMode::Inset => webrender_api::BoxShadowClipMode::Inset,
            BoxShadowClipMode::Outset => webrender_api::BoxShadowClipMode::Outset,
        }
    }
}

trait ToSizeF {
    fn to_sizef(&self) -> webrender_api::LayoutSize;
}

trait ToPointF {
    fn to_pointf(&self) -> webrender_api::LayoutPoint;
}

trait ToVectorF {
    fn to_vectorf(&self) -> webrender_api::LayoutVector2D;
}

impl ToPointF for Point2D<Au> {
    fn to_pointf(&self) -> webrender_api::LayoutPoint {
        webrender_api::LayoutPoint::new(self.x.to_f32_px(), self.y.to_f32_px())
    }
}

impl ToVectorF for Vector2D<Au> {
    fn to_vectorf(&self) -> webrender_api::LayoutVector2D {
        webrender_api::LayoutVector2D::new(self.x.to_f32_px(), self.y.to_f32_px())
    }
}

impl ToSizeF for Size2D<Au> {
    fn to_sizef(&self) -> webrender_api::LayoutSize {
        webrender_api::LayoutSize::new(self.width.to_f32_px(), self.height.to_f32_px())
    }
}

pub trait ToRectF {
    fn to_rectf(&self) -> webrender_api::LayoutRect;
}

impl ToRectF for Rect<Au> {
    fn to_rectf(&self) -> webrender_api::LayoutRect {
        let x = self.origin.x.to_f32_px();
        let y = self.origin.y.to_f32_px();
        let w = self.size.width.to_f32_px();
        let h = self.size.height.to_f32_px();
        let point = webrender_api::LayoutPoint::new(x, y);
        let size = webrender_api::LayoutSize::new(w, h);
        webrender_api::LayoutRect::new(point, size)
    }
}

pub trait ToBorderRadius {
    fn to_border_radius(&self) -> webrender_api::BorderRadius;
}

impl ToBorderRadius for BorderRadii<Au> {
    fn to_border_radius(&self) -> webrender_api::BorderRadius {
        webrender_api::BorderRadius {
            top_left: self.top_left.to_sizef(),
            top_right: self.top_right.to_sizef(),
            bottom_left: self.bottom_left.to_sizef(),
            bottom_right: self.bottom_right.to_sizef(),
        }
    }
}

pub trait ToMixBlendMode {
    fn to_mix_blend_mode(&self) -> webrender_api::MixBlendMode;
}

impl ToMixBlendMode for MixBlendMode {
    fn to_mix_blend_mode(&self) -> webrender_api::MixBlendMode {
        match *self {
            MixBlendMode::Normal => webrender_api::MixBlendMode::Normal,
            MixBlendMode::Multiply => webrender_api::MixBlendMode::Multiply,
            MixBlendMode::Screen => webrender_api::MixBlendMode::Screen,
            MixBlendMode::Overlay => webrender_api::MixBlendMode::Overlay,
            MixBlendMode::Darken => webrender_api::MixBlendMode::Darken,
            MixBlendMode::Lighten => webrender_api::MixBlendMode::Lighten,
            MixBlendMode::ColorDodge => webrender_api::MixBlendMode::ColorDodge,
            MixBlendMode::ColorBurn => webrender_api::MixBlendMode::ColorBurn,
            MixBlendMode::HardLight => webrender_api::MixBlendMode::HardLight,
            MixBlendMode::SoftLight => webrender_api::MixBlendMode::SoftLight,
            MixBlendMode::Difference => webrender_api::MixBlendMode::Difference,
            MixBlendMode::Exclusion => webrender_api::MixBlendMode::Exclusion,
            MixBlendMode::Hue => webrender_api::MixBlendMode::Hue,
            MixBlendMode::Saturation => webrender_api::MixBlendMode::Saturation,
            MixBlendMode::Color => webrender_api::MixBlendMode::Color,
            MixBlendMode::Luminosity => webrender_api::MixBlendMode::Luminosity,
        }
    }
}

trait ToImageRendering {
    fn to_image_rendering(&self) -> webrender_api::ImageRendering;
}

impl ToImageRendering for ImageRendering {
    fn to_image_rendering(&self) -> webrender_api::ImageRendering {
        match *self {
            ImageRendering::CrispEdges => webrender_api::ImageRendering::CrispEdges,
            ImageRendering::Auto => webrender_api::ImageRendering::Auto,
            ImageRendering::Pixelated => webrender_api::ImageRendering::Pixelated,
        }
    }
}

trait ToFilterOps {
    fn to_filter_ops(&self) -> Vec<webrender_api::FilterOp>;
}

impl ToFilterOps for Vec<Filter> {
    fn to_filter_ops(&self) -> Vec<webrender_api::FilterOp> {
        let mut result = Vec::with_capacity(self.len());
        for filter in self.iter() {
            match *filter {
                GenericFilter::Blur(radius) => result.push(webrender_api::FilterOp::Blur(radius.px())),
                GenericFilter::Brightness(amount) => result.push(webrender_api::FilterOp::Brightness(amount.0)),
                GenericFilter::Contrast(amount) => result.push(webrender_api::FilterOp::Contrast(amount.0)),
                GenericFilter::Grayscale(amount) => result.push(webrender_api::FilterOp::Grayscale(amount.0)),
                GenericFilter::HueRotate(angle) => result.push(webrender_api::FilterOp::HueRotate(angle.radians())),
                GenericFilter::Invert(amount) => result.push(webrender_api::FilterOp::Invert(amount.0)),
                GenericFilter::Opacity(amount) => {
                    result.push(webrender_api::FilterOp::Opacity(amount.0.into(), amount.0));
                }
                GenericFilter::Saturate(amount) => result.push(webrender_api::FilterOp::Saturate(amount.0)),
                GenericFilter::Sepia(amount) => result.push(webrender_api::FilterOp::Sepia(amount.0)),
                GenericFilter::DropShadow(ref shadow) => match *shadow {},
            }
        }
        result
    }
}

pub trait ToTransformStyle {
    fn to_transform_style(&self) -> webrender_api::TransformStyle;
}

impl ToTransformStyle for TransformStyle {
    fn to_transform_style(&self) -> webrender_api::TransformStyle {
        match *self {
            TransformStyle::Auto | TransformStyle::Flat => webrender_api::TransformStyle::Flat,
            TransformStyle::Preserve3d => webrender_api::TransformStyle::Preserve3D,
        }
    }
}

impl WebRenderDisplayListConverter for DisplayList {
    fn convert_to_webrender(&self, pipeline_id: PipelineId) -> DisplayListBuilder {
        let mut builder = DisplayListBuilder::with_capacity(pipeline_id.to_webrender(),
                                                            self.bounds().size.to_sizef(),
                                                            1024 * 1024); // 1 MB of space

        let mut current_clip_and_scroll_info = pipeline_id.root_clip_and_scroll_info();
        builder.push_clip_and_scroll_info(current_clip_and_scroll_info);

        let mut clip_ids = Vec::with_capacity(self.clip_scroll_nodes.len());
        clip_ids.resize(self.clip_scroll_nodes.len(), None);
        clip_ids[0] = Some(ClipId::root_scroll_node(pipeline_id.to_webrender()));

        for item in &self.list {
            item.convert_to_webrender(
                &mut builder,
                &self.clip_scroll_nodes,
                &mut clip_ids,
                &mut current_clip_and_scroll_info
            );
        }
        builder
    }
}

impl WebRenderDisplayItemConverter for DisplayItem {
    fn prim_info(&self) -> webrender_api::LayoutPrimitiveInfo {
        let tag = match self.base().metadata.pointing {
            Some(cursor) => Some((self.base().metadata.node.0 as u64, cursor as u16)),
            None => None,
        };
        webrender_api::LayoutPrimitiveInfo {
            rect: self.base().bounds.to_rectf(),
            local_clip: self.base().local_clip,
            // TODO(gw): Make use of the WR backface visibility functionality.
            is_backface_visible: true,
            tag,
        }
    }

    fn convert_to_webrender(
        &self,
        builder: &mut DisplayListBuilder,
        clip_scroll_nodes: &[ClipScrollNode],
        clip_ids: &mut Vec<Option<ClipId>>,
        current_clip_and_scroll_info: &mut ClipAndScrollInfo
    ) {
        let get_id = |clip_ids: &[Option<ClipId>], index: ClipScrollNodeIndex| -> ClipId {
            match clip_ids[index.0] {
                Some(id) => id,
                None => unreachable!("Tried to use WebRender ClipId before it was defined."),
            }
        };

        let clip_and_scroll_indices = self.base().clipping_and_scrolling;
        let scrolling_id = get_id(clip_ids, clip_and_scroll_indices.scrolling);
        let clip_and_scroll_info = match clip_and_scroll_indices.clipping {
            None => ClipAndScrollInfo::simple(scrolling_id),
            Some(index) => ClipAndScrollInfo::new(scrolling_id, get_id(clip_ids, index)),
        };

        if clip_and_scroll_info != *current_clip_and_scroll_info {
            builder.pop_clip_id();
            builder.push_clip_and_scroll_info(clip_and_scroll_info);
            *current_clip_and_scroll_info = clip_and_scroll_info;
        }

        match *self {
            DisplayItem::SolidColor(ref item) => {
                builder.push_rect(&self.prim_info(), item.color);
            }
            DisplayItem::Text(ref item) => {
                let mut origin = item.baseline_origin.clone();
                let mut glyphs = vec!();

                for slice in item.text_run.natural_word_slices_in_visual_order(&item.range) {
                    for glyph in slice.glyphs.iter_glyphs_for_byte_range(&slice.range) {
                        let glyph_advance = if glyph.char_is_space() {
                            glyph.advance() + item.text_run.extra_word_spacing
                        } else {
                            glyph.advance()
                        };
                        if !slice.glyphs.is_whitespace() {
                            let glyph_offset = glyph.offset().unwrap_or(Point2D::zero());
                            let x = (origin.x + glyph_offset.x).to_f32_px();
                            let y = (origin.y + glyph_offset.y).to_f32_px();
                            let point = webrender_api::LayoutPoint::new(x, y);
                            let glyph = webrender_api::GlyphInstance {
                                index: glyph.id(),
                                point: point,
                            };
                            glyphs.push(glyph);
                        }
                        origin.x = origin.x + glyph_advance;
                    };
                }

                if glyphs.len() > 0 {
                    builder.push_text(&self.prim_info(),
                                      &glyphs,
                                      item.text_run.font_key,
                                      item.text_color,
                                      None);
                }
            }
            DisplayItem::Image(ref item) => {
                if let Some(id) = item.webrender_image.key {
                    if item.stretch_size.width > Au(0) &&
                       item.stretch_size.height > Au(0) {
                        builder.push_image(&self.prim_info(),
                                           item.stretch_size.to_sizef(),
                                           item.tile_spacing.to_sizef(),
                                           item.image_rendering.to_image_rendering(),
                                           id);
                    }
                }
            }
            DisplayItem::Border(ref item) => {
                let widths = item.border_widths.to_border_widths();

                let details = match item.details {
                    BorderDetails::Normal(ref border) => {
                        let left = webrender_api::BorderSide {
                            color: border.color.left,
                            style: border.style.left.to_border_style(),
                        };
                        let top = webrender_api::BorderSide {
                            color: border.color.top,
                            style: border.style.top.to_border_style(),
                        };
                        let right = webrender_api::BorderSide {
                            color: border.color.right,
                            style: border.style.right.to_border_style(),
                        };
                        let bottom = webrender_api::BorderSide {
                            color: border.color.bottom,
                            style: border.style.bottom.to_border_style(),
                        };
                        let radius = border.radius.to_border_radius();
                        webrender_api::BorderDetails::Normal(webrender_api::NormalBorder {
                            left: left,
                            top: top,
                            right: right,
                            bottom: bottom,
                            radius: radius,
                        })
                    }
                    BorderDetails::Image(ref image) => {
                        match image.image.key {
                            None => return,
                            Some(key) => {
                                webrender_api::BorderDetails::Image(webrender_api::ImageBorder {
                                    image_key: key,
                                    patch: webrender_api::NinePatchDescriptor {
                                        width: image.image.width,
                                        height: image.image.height,
                                        slice: image.slice,
                                    },
                                    fill: image.fill,
                                    outset: image.outset,
                                    repeat_horizontal: image.repeat_horizontal,
                                    repeat_vertical: image.repeat_vertical,
                                })
                            }
                        }
                    }
                    BorderDetails::Gradient(ref gradient) => {
                        let extend_mode = if gradient.gradient.repeating {
                            ExtendMode::Repeat
                        } else {
                            ExtendMode::Clamp
                        };
                        webrender_api::BorderDetails::Gradient(webrender_api::GradientBorder {
                            gradient: builder.create_gradient(
                                          gradient.gradient.start_point.to_pointf(),
                                          gradient.gradient.end_point.to_pointf(),
                                          gradient.gradient.stops.clone(),
                                          extend_mode),
                            outset: gradient.outset,
                        })
                    }
                    BorderDetails::RadialGradient(ref gradient) => {
                        let extend_mode = if gradient.gradient.repeating {
                            ExtendMode::Repeat
                        } else {
                            ExtendMode::Clamp
                        };
                       webrender_api::BorderDetails::RadialGradient(webrender_api::RadialGradientBorder {
                           gradient: builder.create_radial_gradient(
                               gradient.gradient.center.to_pointf(),
                               gradient.gradient.radius.to_sizef(),
                               gradient.gradient.stops.clone(),
                               extend_mode),
                           outset: gradient.outset,
                       })
                    }
                };

                builder.push_border(&self.prim_info(), widths, details);
            }
            DisplayItem::Gradient(ref item) => {
                let start_point = item.gradient.start_point.to_pointf();
                let end_point = item.gradient.end_point.to_pointf();
                let extend_mode = if item.gradient.repeating {
                    ExtendMode::Repeat
                } else {
                    ExtendMode::Clamp
                };
                let gradient = builder.create_gradient(start_point,
                                                       end_point,
                                                       item.gradient.stops.clone(),
                                                       extend_mode);
                builder.push_gradient(&self.prim_info(),
                                      gradient,
                                      item.tile.to_sizef(),
                                      item.tile_spacing.to_sizef());
            }
            DisplayItem::RadialGradient(ref item) => {
                let center = item.gradient.center.to_pointf();
                let radius = item.gradient.radius.to_sizef();
                let extend_mode = if item.gradient.repeating {
                    ExtendMode::Repeat
                } else {
                    ExtendMode::Clamp
                };
                let gradient = builder.create_radial_gradient(center,
                                                              radius,
                                                              item.gradient.stops.clone(),
                                                              extend_mode);
                builder.push_radial_gradient(&self.prim_info(),
                                             gradient,
                                             item.tile.to_sizef(),
                                             item.tile_spacing.to_sizef());
            }
            DisplayItem::Line(ref item) => {
                builder.push_line(&self.prim_info(),
                                  // TODO(gw): Use a better estimate for wavy line thickness.
                                  (0.33 * item.base.bounds.size.height.to_f32_px()).ceil(),
                                  webrender_api::LineOrientation::Horizontal,
                                  &item.color,
                                  item.style);
            }
            DisplayItem::BoxShadow(ref item) => {
                let box_bounds = item.box_bounds.to_rectf();
                builder.push_box_shadow(&self.prim_info(),
                                        box_bounds,
                                        item.offset.to_vectorf(),
                                        item.color,
                                        item.blur_radius.to_f32_px(),
                                        item.spread_radius.to_f32_px(),
                                        item.border_radius.to_border_radius(),
                                        item.clip_mode.to_clip_mode());
            }
            DisplayItem::PushTextShadow(ref item) => {
                builder.push_shadow(&self.prim_info(),
                                    webrender_api::Shadow {
                                        blur_radius: item.blur_radius.to_f32_px(),
                                        offset: item.offset.to_vectorf(),
                                        color: item.color,
                                    });
            }
            DisplayItem::PopAllTextShadows(_) => {
                builder.pop_all_shadows();
            }
            DisplayItem::Iframe(ref item) => {
                builder.push_iframe(&self.prim_info(), item.iframe.to_webrender());
            }
            DisplayItem::PushStackingContext(ref item) => {
                let stacking_context = &item.stacking_context;
                debug_assert!(stacking_context.context_type == StackingContextType::Real);

                let transform = stacking_context.transform.map(|transform| {
                    LayoutTransform::from_untyped(&transform).into()
                });
                let perspective = stacking_context.perspective.map(|perspective| {
                    LayoutTransform::from_untyped(&perspective)
                });

                builder.push_stacking_context(
                    &webrender_api::LayoutPrimitiveInfo::new(stacking_context.bounds.to_rectf()),
                    stacking_context.scroll_policy,
                    transform,
                    stacking_context.transform_style,
                    perspective,
                    stacking_context.mix_blend_mode,
                    stacking_context.filters.to_filter_ops()
                );
            }
            DisplayItem::PopStackingContext(_) => builder.pop_stacking_context(),
            DisplayItem::DefineClipScrollNode(ref item) => {
                let node = &clip_scroll_nodes[item.node_index.0];
                let parent_id = get_id(clip_ids, node.parent_index);
                let item_rect = node.clip.main.to_rectf();

                let webrender_id = match node.node_type {
                   ClipScrollNodeType::Clip => {
                        builder.define_clip_with_parent(
                            node.id,
                            parent_id,
                            item_rect,
                            node.clip.get_complex_clips(),
                            None
                        )
                    }
                    ClipScrollNodeType::ScrollFrame(scroll_sensitivity) => {
                        builder.define_scroll_frame_with_parent(
                            node.id,
                            parent_id,
                            node.content_rect.to_rectf(),
                            node.clip.main.to_rectf(),
                            node.clip.get_complex_clips(),
                            None,
                            scroll_sensitivity
                        )
                    }
                    ClipScrollNodeType::StickyFrame(ref sticky_data) => {
                        // TODO: Add define_sticky_frame_with_parent to WebRender.
                        builder.push_clip_id(parent_id);
                        let id = builder.define_sticky_frame(
                            node.id,
                            item_rect,
                            sticky_data.margins,
                            sticky_data.vertical_offset_bounds,
                            sticky_data.horizontal_offset_bounds,
                            webrender_api::LayoutVector2D::zero(),
                        );
                        builder.pop_clip_id();
                        id
                    }
                };

                debug_assert!(node.id.is_none() || node.id == Some(webrender_id));
                clip_ids[item.node_index.0] = Some(webrender_id);
            }
        }
    }
}

trait ToWebRenderClip {
    fn get_complex_clips(&self) -> Vec<ComplexClipRegion>;
}

impl ToWebRenderClip for ClippingRegion {
    fn get_complex_clips(&self) -> Vec<ComplexClipRegion> {
        self.complex.iter().map(|complex_clipping_region| {
            ComplexClipRegion::new(
                complex_clipping_region.rect.to_rectf(),
                complex_clipping_region.radii.to_border_radius(),
                ClipMode::Clip,
             )
        }).collect()
    }
}
