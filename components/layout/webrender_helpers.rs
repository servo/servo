/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// TODO(gw): This contains helper traits and implementations for converting Servo display lists
//           into WebRender display lists. In the future, this step should be completely removed.
//           This might be achieved by sharing types between WR and Servo display lists, or
//           completely converting layout to directly generate WebRender display lists, for example.

use app_units::Au;
use euclid::{Point2D, Vector2D, Rect, SideOffsets2D, Size2D};
use gfx::display_list::{BorderDetails, BorderRadii, BoxShadowClipMode, ClippingRegion};
use gfx::display_list::{DisplayItem, DisplayList, DisplayListTraversal, StackingContextType};
use msg::constellation_msg::PipelineId;
use style::computed_values::{image_rendering, mix_blend_mode, transform_style};
use style::computed_values::filter::{self, Filter};
use style::values::computed::BorderStyle;
use webrender_traits::{self, DisplayListBuilder, ExtendMode};
use webrender_traits::{LayoutTransform, ClipId, ClipRegionToken};

pub trait WebRenderDisplayListConverter {
    fn convert_to_webrender(&self, pipeline_id: PipelineId) -> DisplayListBuilder;
}

trait WebRenderDisplayItemConverter {
    fn convert_to_webrender(&self,
                            builder: &mut DisplayListBuilder,
                            current_scroll_root_id: &mut ClipId);
}

trait ToBorderStyle {
    fn to_border_style(&self) -> webrender_traits::BorderStyle;
}

impl ToBorderStyle for BorderStyle {
    fn to_border_style(&self) -> webrender_traits::BorderStyle {
        match *self {
            BorderStyle::none => webrender_traits::BorderStyle::None,
            BorderStyle::solid => webrender_traits::BorderStyle::Solid,
            BorderStyle::double => webrender_traits::BorderStyle::Double,
            BorderStyle::dotted => webrender_traits::BorderStyle::Dotted,
            BorderStyle::dashed => webrender_traits::BorderStyle::Dashed,
            BorderStyle::hidden => webrender_traits::BorderStyle::Hidden,
            BorderStyle::groove => webrender_traits::BorderStyle::Groove,
            BorderStyle::ridge => webrender_traits::BorderStyle::Ridge,
            BorderStyle::inset => webrender_traits::BorderStyle::Inset,
            BorderStyle::outset => webrender_traits::BorderStyle::Outset,
        }
    }
}

trait ToBorderWidths {
    fn to_border_widths(&self) -> webrender_traits::BorderWidths;
}

impl ToBorderWidths for SideOffsets2D<Au> {
    fn to_border_widths(&self) -> webrender_traits::BorderWidths {
        webrender_traits::BorderWidths {
            left: self.left.to_f32_px(),
            top: self.top.to_f32_px(),
            right: self.right.to_f32_px(),
            bottom: self.bottom.to_f32_px(),
        }
    }
}

trait ToBoxShadowClipMode {
    fn to_clip_mode(&self) -> webrender_traits::BoxShadowClipMode;
}

impl ToBoxShadowClipMode for BoxShadowClipMode {
    fn to_clip_mode(&self) -> webrender_traits::BoxShadowClipMode {
        match *self {
            BoxShadowClipMode::None => webrender_traits::BoxShadowClipMode::None,
            BoxShadowClipMode::Inset => webrender_traits::BoxShadowClipMode::Inset,
            BoxShadowClipMode::Outset => webrender_traits::BoxShadowClipMode::Outset,
        }
    }
}

trait ToSizeF {
    fn to_sizef(&self) -> webrender_traits::LayoutSize;
}

trait ToPointF {
    fn to_pointf(&self) -> webrender_traits::LayoutPoint;
}

trait ToVectorF {
    fn to_vectorf(&self) -> webrender_traits::LayoutVector2D;
}

impl ToPointF for Point2D<Au> {
    fn to_pointf(&self) -> webrender_traits::LayoutPoint {
        webrender_traits::LayoutPoint::new(self.x.to_f32_px(), self.y.to_f32_px())
    }
}

impl ToVectorF for Vector2D<Au> {
    fn to_vectorf(&self) -> webrender_traits::LayoutVector2D {
        webrender_traits::LayoutVector2D::new(self.x.to_f32_px(), self.y.to_f32_px())
    }
}

impl ToSizeF for Size2D<Au> {
    fn to_sizef(&self) -> webrender_traits::LayoutSize {
        webrender_traits::LayoutSize::new(self.width.to_f32_px(), self.height.to_f32_px())
    }
}

trait ToRectF {
    fn to_rectf(&self) -> webrender_traits::LayoutRect;
}

impl ToRectF for Rect<Au> {
    fn to_rectf(&self) -> webrender_traits::LayoutRect {
        let x = self.origin.x.to_f32_px();
        let y = self.origin.y.to_f32_px();
        let w = self.size.width.to_f32_px();
        let h = self.size.height.to_f32_px();
        let point = webrender_traits::LayoutPoint::new(x, y);
        let size = webrender_traits::LayoutSize::new(w, h);
        webrender_traits::LayoutRect::new(point, size)
    }
}

trait ToClipRegion {
    fn push_clip_region(&self, builder: &mut DisplayListBuilder) -> ClipRegionToken;
}

impl ToClipRegion for ClippingRegion {
    fn push_clip_region(&self, builder: &mut DisplayListBuilder) -> ClipRegionToken {
        builder.push_clip_region(&self.main.to_rectf(),
                                self.complex.iter().map(|complex_clipping_region| {
                                    webrender_traits::ComplexClipRegion::new(
                                        complex_clipping_region.rect.to_rectf(),
                                        complex_clipping_region.radii.to_border_radius(),
                                     )
                                }),
                                None)
    }
}

trait ToBorderRadius {
    fn to_border_radius(&self) -> webrender_traits::BorderRadius;
}

impl ToBorderRadius for BorderRadii<Au> {
    fn to_border_radius(&self) -> webrender_traits::BorderRadius {
        webrender_traits::BorderRadius {
            top_left: self.top_left.to_sizef(),
            top_right: self.top_right.to_sizef(),
            bottom_left: self.bottom_left.to_sizef(),
            bottom_right: self.bottom_right.to_sizef(),
        }
    }
}

pub trait ToMixBlendMode {
    fn to_mix_blend_mode(&self) -> webrender_traits::MixBlendMode;
}

impl ToMixBlendMode for mix_blend_mode::T {
    fn to_mix_blend_mode(&self) -> webrender_traits::MixBlendMode {
        match *self {
            mix_blend_mode::T::normal => webrender_traits::MixBlendMode::Normal,
            mix_blend_mode::T::multiply => webrender_traits::MixBlendMode::Multiply,
            mix_blend_mode::T::screen => webrender_traits::MixBlendMode::Screen,
            mix_blend_mode::T::overlay => webrender_traits::MixBlendMode::Overlay,
            mix_blend_mode::T::darken => webrender_traits::MixBlendMode::Darken,
            mix_blend_mode::T::lighten => webrender_traits::MixBlendMode::Lighten,
            mix_blend_mode::T::color_dodge => webrender_traits::MixBlendMode::ColorDodge,
            mix_blend_mode::T::color_burn => webrender_traits::MixBlendMode::ColorBurn,
            mix_blend_mode::T::hard_light => webrender_traits::MixBlendMode::HardLight,
            mix_blend_mode::T::soft_light => webrender_traits::MixBlendMode::SoftLight,
            mix_blend_mode::T::difference => webrender_traits::MixBlendMode::Difference,
            mix_blend_mode::T::exclusion => webrender_traits::MixBlendMode::Exclusion,
            mix_blend_mode::T::hue => webrender_traits::MixBlendMode::Hue,
            mix_blend_mode::T::saturation => webrender_traits::MixBlendMode::Saturation,
            mix_blend_mode::T::color => webrender_traits::MixBlendMode::Color,
            mix_blend_mode::T::luminosity => webrender_traits::MixBlendMode::Luminosity,
        }
    }
}

trait ToImageRendering {
    fn to_image_rendering(&self) -> webrender_traits::ImageRendering;
}

impl ToImageRendering for image_rendering::T {
    fn to_image_rendering(&self) -> webrender_traits::ImageRendering {
        match *self {
            image_rendering::T::crisp_edges => webrender_traits::ImageRendering::CrispEdges,
            image_rendering::T::auto => webrender_traits::ImageRendering::Auto,
            image_rendering::T::pixelated => webrender_traits::ImageRendering::Pixelated,
        }
    }
}

trait ToFilterOps {
    fn to_filter_ops(&self) -> Vec<webrender_traits::FilterOp>;
}

impl ToFilterOps for filter::T {
    fn to_filter_ops(&self) -> Vec<webrender_traits::FilterOp> {
        let mut result = Vec::with_capacity(self.filters.len());
        for filter in self.filters.iter() {
            match *filter {
                Filter::Blur(radius) => result.push(webrender_traits::FilterOp::Blur(radius)),
                Filter::Brightness(amount) => result.push(webrender_traits::FilterOp::Brightness(amount)),
                Filter::Contrast(amount) => result.push(webrender_traits::FilterOp::Contrast(amount)),
                Filter::Grayscale(amount) => result.push(webrender_traits::FilterOp::Grayscale(amount)),
                Filter::HueRotate(angle) => result.push(webrender_traits::FilterOp::HueRotate(angle.radians())),
                Filter::Invert(amount) => result.push(webrender_traits::FilterOp::Invert(amount)),
                Filter::Opacity(amount) => result.push(webrender_traits::FilterOp::Opacity(amount.into())),
                Filter::Saturate(amount) => result.push(webrender_traits::FilterOp::Saturate(amount)),
                Filter::Sepia(amount) => result.push(webrender_traits::FilterOp::Sepia(amount)),
            }
        }
        result
    }
}

pub trait ToTransformStyle {
    fn to_transform_style(&self) -> webrender_traits::TransformStyle;
}

impl ToTransformStyle for transform_style::T {
    fn to_transform_style(&self) -> webrender_traits::TransformStyle {
        match *self {
            transform_style::T::auto | transform_style::T::flat => webrender_traits::TransformStyle::Flat,
            transform_style::T::preserve_3d => webrender_traits::TransformStyle::Preserve3D,
        }
    }
}

impl WebRenderDisplayListConverter for DisplayList {
    fn convert_to_webrender(&self, pipeline_id: PipelineId) -> DisplayListBuilder {
        let traversal = DisplayListTraversal::new(self);
        let webrender_pipeline_id = pipeline_id.to_webrender();
        let mut builder = DisplayListBuilder::new(webrender_pipeline_id,
                                                  self.bounds().size.to_sizef());

        let mut current_scroll_root_id = ClipId::root_scroll_node(webrender_pipeline_id);
        builder.push_clip_id(current_scroll_root_id);

        for item in traversal {
            item.convert_to_webrender(&mut builder, &mut current_scroll_root_id);
        }
        builder
    }
}

impl WebRenderDisplayItemConverter for DisplayItem {
    fn convert_to_webrender(&self,
                            builder: &mut DisplayListBuilder,
                            current_scroll_root_id: &mut ClipId) {
        let scroll_root_id = self.base().scroll_root_id;
        if scroll_root_id != *current_scroll_root_id {
            builder.pop_clip_id();
            builder.push_clip_id(scroll_root_id);
            *current_scroll_root_id = scroll_root_id;
        }

        match *self {
            DisplayItem::SolidColor(ref item) => {
                let color = item.color;
                if color.a > 0.0 {
                    let clip = item.base.clip.push_clip_region(builder);
                    builder.push_rect(item.base.bounds.to_rectf(), clip, color);
                }
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
                            let point = webrender_traits::LayoutPoint::new(x, y);
                            let glyph = webrender_traits::GlyphInstance {
                                index: glyph.id(),
                                point: point,
                            };
                            glyphs.push(glyph);
                        }
                        origin.x = origin.x + glyph_advance;
                    };
                }

                if glyphs.len() > 0 {
                    let clip = item.base.clip.push_clip_region(builder);
                    builder.push_text(item.base.bounds.to_rectf(),
                                      clip,
                                      &glyphs,
                                      item.text_run.font_key,
                                      item.text_color,
                                      item.text_run.actual_pt_size,
                                      item.blur_radius.to_f32_px(),
                                      None);
                }
            }
            DisplayItem::Image(ref item) => {
                if let Some(id) = item.webrender_image.key {
                    if item.stretch_size.width > Au(0) &&
                       item.stretch_size.height > Au(0) {
                        let clip = item.base.clip.push_clip_region(builder);
                        builder.push_image(item.base.bounds.to_rectf(),
                                           clip,
                                           item.stretch_size.to_sizef(),
                                           item.tile_spacing.to_sizef(),
                                           item.image_rendering.to_image_rendering(),
                                           id);
                    }
                }
            }
            DisplayItem::WebGL(ref item) => {
                let clip = item.base.clip.push_clip_region(builder);
                builder.push_webgl_canvas(item.base.bounds.to_rectf(), clip, item.context_id);
            }
            DisplayItem::Border(ref item) => {
                let rect = item.base.bounds.to_rectf();
                let widths = item.border_widths.to_border_widths();
                let clip = item.base.clip.push_clip_region(builder);

                let details = match item.details {
                    BorderDetails::Normal(ref border) => {
                        let left = webrender_traits::BorderSide {
                            color: border.color.left,
                            style: border.style.left.to_border_style(),
                        };
                        let top = webrender_traits::BorderSide {
                            color: border.color.top,
                            style: border.style.top.to_border_style(),
                        };
                        let right = webrender_traits::BorderSide {
                            color: border.color.right,
                            style: border.style.right.to_border_style(),
                        };
                        let bottom = webrender_traits::BorderSide {
                            color: border.color.bottom,
                            style: border.style.bottom.to_border_style(),
                        };
                        let radius = border.radius.to_border_radius();
                        webrender_traits::BorderDetails::Normal(webrender_traits::NormalBorder {
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
                                webrender_traits::BorderDetails::Image(webrender_traits::ImageBorder {
                                    image_key: key,
                                    patch: webrender_traits::NinePatchDescriptor {
                                        width: image.image.width,
                                        height: image.image.height,
                                        slice: image.slice,
                                    },
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
                        webrender_traits::BorderDetails::Gradient(webrender_traits::GradientBorder {
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
                       webrender_traits::BorderDetails::RadialGradient(webrender_traits::RadialGradientBorder {
                           gradient: builder.create_radial_gradient(
                               gradient.gradient.center.to_pointf(),
                               gradient.gradient.radius.to_sizef(),
                               gradient.gradient.stops.clone(),
                               extend_mode),
                           outset: gradient.outset,
                       })
                    }
                };

                builder.push_border(rect, clip, widths, details);
            }
            DisplayItem::Gradient(ref item) => {
                let rect = item.base.bounds.to_rectf();
                let start_point = item.gradient.start_point.to_pointf();
                let end_point = item.gradient.end_point.to_pointf();
                let clip = item.base.clip.push_clip_region(builder);
                let extend_mode = if item.gradient.repeating {
                    ExtendMode::Repeat
                } else {
                    ExtendMode::Clamp
                };
                let gradient = builder.create_gradient(start_point,
                                                       end_point,
                                                       item.gradient.stops.clone(),
                                                       extend_mode);
                builder.push_gradient(rect,
                                      clip,
                                      gradient,
                                      rect.size,
                                      webrender_traits::LayoutSize::zero());
            }
            DisplayItem::RadialGradient(ref item) => {
                let rect = item.base.bounds.to_rectf();
                let center = item.gradient.center.to_pointf();
                let radius = item.gradient.radius.to_sizef();
                let clip = item.base.clip.push_clip_region(builder);
                let extend_mode = if item.gradient.repeating {
                    ExtendMode::Repeat
                } else {
                    ExtendMode::Clamp
                };
                let gradient = builder.create_radial_gradient(center,
                                                              radius,
                                                              item.gradient.stops.clone(),
                                                              extend_mode);
                builder.push_radial_gradient(rect,
                                             clip,
                                             gradient,
                                             rect.size,
                                             webrender_traits::LayoutSize::zero());
            }
            DisplayItem::Line(..) => {
                println!("TODO DisplayItem::Line");
            }
            DisplayItem::BoxShadow(ref item) => {
                let rect = item.base.bounds.to_rectf();
                let box_bounds = item.box_bounds.to_rectf();
                let clip = item.base.clip.push_clip_region(builder);
                builder.push_box_shadow(rect,
                                        clip,
                                        box_bounds,
                                        item.offset.to_vectorf(),
                                        item.color,
                                        item.blur_radius.to_f32_px(),
                                        item.spread_radius.to_f32_px(),
                                        item.border_radius.to_f32_px(),
                                        item.clip_mode.to_clip_mode());
            }
            DisplayItem::Iframe(ref item) => {
                let rect = item.base.bounds.to_rectf();
                let pipeline_id = item.iframe.to_webrender();
                let clip = item.base.clip.push_clip_region(builder);
                builder.push_iframe(rect, clip, pipeline_id);
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

                builder.push_stacking_context(stacking_context.scroll_policy,
                                              stacking_context.bounds.to_rectf(),
                                              transform,
                                              stacking_context.transform_style,
                                              perspective,
                                              stacking_context.mix_blend_mode,
                                              stacking_context.filters.to_filter_ops());
            }
            DisplayItem::PopStackingContext(_) => builder.pop_stacking_context(),
            DisplayItem::DefineClip(ref item) => {
                builder.push_clip_id(item.scroll_root.parent_id);

                let our_id = item.scroll_root.id;
                let clip = item.scroll_root.clip.push_clip_region(builder);
                let content_rect = item.scroll_root.content_rect.to_rectf();
                let webrender_id = builder.define_clip(content_rect, clip, Some(our_id));
                debug_assert!(our_id == webrender_id);

                builder.pop_clip_id();
            }
        }
    }
}
