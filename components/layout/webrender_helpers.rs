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
use gfx::display_list::{DisplayItem, DisplayList, DisplayListTraversal, ScrollRootType};
use gfx::display_list::StackingContextType;
use msg::constellation_msg::PipelineId;
use style::computed_values::{image_rendering, mix_blend_mode, transform_style};
use style::values::computed::{BorderStyle, Filter};
use style::values::generics::effects::Filter as GenericFilter;
use webrender_api::{self, ComplexClipRegion, DisplayListBuilder, ExtendMode};
use webrender_api::{LayoutTransform, ClipId};

pub trait WebRenderDisplayListConverter {
    fn convert_to_webrender(&self, pipeline_id: PipelineId) -> DisplayListBuilder;
}

trait WebRenderDisplayItemConverter {
    fn convert_to_webrender(&self,
                            builder: &mut DisplayListBuilder,
                            current_scroll_root_id: &mut ClipId);
}

trait ToBorderStyle {
    fn to_border_style(&self) -> webrender_api::BorderStyle;
}

impl ToBorderStyle for BorderStyle {
    fn to_border_style(&self) -> webrender_api::BorderStyle {
        match *self {
            BorderStyle::none => webrender_api::BorderStyle::None,
            BorderStyle::solid => webrender_api::BorderStyle::Solid,
            BorderStyle::double => webrender_api::BorderStyle::Double,
            BorderStyle::dotted => webrender_api::BorderStyle::Dotted,
            BorderStyle::dashed => webrender_api::BorderStyle::Dashed,
            BorderStyle::hidden => webrender_api::BorderStyle::Hidden,
            BorderStyle::groove => webrender_api::BorderStyle::Groove,
            BorderStyle::ridge => webrender_api::BorderStyle::Ridge,
            BorderStyle::inset => webrender_api::BorderStyle::Inset,
            BorderStyle::outset => webrender_api::BorderStyle::Outset,
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
            BoxShadowClipMode::None => webrender_api::BoxShadowClipMode::None,
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

impl ToMixBlendMode for mix_blend_mode::T {
    fn to_mix_blend_mode(&self) -> webrender_api::MixBlendMode {
        match *self {
            mix_blend_mode::T::normal => webrender_api::MixBlendMode::Normal,
            mix_blend_mode::T::multiply => webrender_api::MixBlendMode::Multiply,
            mix_blend_mode::T::screen => webrender_api::MixBlendMode::Screen,
            mix_blend_mode::T::overlay => webrender_api::MixBlendMode::Overlay,
            mix_blend_mode::T::darken => webrender_api::MixBlendMode::Darken,
            mix_blend_mode::T::lighten => webrender_api::MixBlendMode::Lighten,
            mix_blend_mode::T::color_dodge => webrender_api::MixBlendMode::ColorDodge,
            mix_blend_mode::T::color_burn => webrender_api::MixBlendMode::ColorBurn,
            mix_blend_mode::T::hard_light => webrender_api::MixBlendMode::HardLight,
            mix_blend_mode::T::soft_light => webrender_api::MixBlendMode::SoftLight,
            mix_blend_mode::T::difference => webrender_api::MixBlendMode::Difference,
            mix_blend_mode::T::exclusion => webrender_api::MixBlendMode::Exclusion,
            mix_blend_mode::T::hue => webrender_api::MixBlendMode::Hue,
            mix_blend_mode::T::saturation => webrender_api::MixBlendMode::Saturation,
            mix_blend_mode::T::color => webrender_api::MixBlendMode::Color,
            mix_blend_mode::T::luminosity => webrender_api::MixBlendMode::Luminosity,
        }
    }
}

trait ToImageRendering {
    fn to_image_rendering(&self) -> webrender_api::ImageRendering;
}

impl ToImageRendering for image_rendering::T {
    fn to_image_rendering(&self) -> webrender_api::ImageRendering {
        match *self {
            image_rendering::T::crisp_edges => webrender_api::ImageRendering::CrispEdges,
            image_rendering::T::auto => webrender_api::ImageRendering::Auto,
            image_rendering::T::pixelated => webrender_api::ImageRendering::Pixelated,
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
                GenericFilter::Blur(radius) => result.push(webrender_api::FilterOp::Blur(radius)),
                GenericFilter::Brightness(amount) => result.push(webrender_api::FilterOp::Brightness(amount)),
                GenericFilter::Contrast(amount) => result.push(webrender_api::FilterOp::Contrast(amount)),
                GenericFilter::Grayscale(amount) => result.push(webrender_api::FilterOp::Grayscale(amount)),
                GenericFilter::HueRotate(angle) => result.push(webrender_api::FilterOp::HueRotate(angle.radians())),
                GenericFilter::Invert(amount) => result.push(webrender_api::FilterOp::Invert(amount)),
                GenericFilter::Opacity(amount) => result.push(webrender_api::FilterOp::Opacity(amount.into())),
                GenericFilter::Saturate(amount) => result.push(webrender_api::FilterOp::Saturate(amount)),
                GenericFilter::Sepia(amount) => result.push(webrender_api::FilterOp::Sepia(amount)),
                GenericFilter::DropShadow(ref shadow) => match *shadow {},
            }
        }
        result
    }
}

pub trait ToTransformStyle {
    fn to_transform_style(&self) -> webrender_api::TransformStyle;
}

impl ToTransformStyle for transform_style::T {
    fn to_transform_style(&self) -> webrender_api::TransformStyle {
        match *self {
            transform_style::T::auto | transform_style::T::flat => webrender_api::TransformStyle::Flat,
            transform_style::T::preserve_3d => webrender_api::TransformStyle::Preserve3D,
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
                    builder.push_rect(item.base.bounds.to_rectf(),
                                      Some(item.base.local_clip),
                                      color);
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
                    builder.push_text(item.base.bounds.to_rectf(),
                                      Some(item.base.local_clip),
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
                        builder.push_image(item.base.bounds.to_rectf(),
                                           Some(item.base.local_clip),
                                           item.stretch_size.to_sizef(),
                                           item.tile_spacing.to_sizef(),
                                           item.image_rendering.to_image_rendering(),
                                           id);
                    }
                }
            }
            DisplayItem::WebGL(ref item) => {
                builder.push_webgl_canvas(item.base.bounds.to_rectf(),
                                          Some(item.base.local_clip),
                                          item.context_id);
            }
            DisplayItem::Border(ref item) => {
                let rect = item.base.bounds.to_rectf();
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

                builder.push_border(rect, Some(item.base.local_clip), widths, details);
            }
            DisplayItem::Gradient(ref item) => {
                let rect = item.base.bounds.to_rectf();
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
                builder.push_gradient(rect,
                                      Some(item.base.local_clip),
                                      gradient,
                                      rect.size,
                                      webrender_api::LayoutSize::zero());
            }
            DisplayItem::RadialGradient(ref item) => {
                let rect = item.base.bounds.to_rectf();
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
                builder.push_radial_gradient(rect,
                                             Some(item.base.local_clip),
                                             gradient,
                                             rect.size,
                                             webrender_api::LayoutSize::zero());
            }
            DisplayItem::Line(..) => {
                println!("TODO DisplayItem::Line");
            }
            DisplayItem::BoxShadow(ref item) => {
                let rect = item.base.bounds.to_rectf();
                let box_bounds = item.box_bounds.to_rectf();
                builder.push_box_shadow(rect,
                                        Some(item.base.local_clip),
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
                builder.push_iframe(rect, pipeline_id);
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
                let webrender_id = match item.scroll_root.root_type {
                   ScrollRootType::Clip => {
                        builder.define_clip(Some(our_id),
                                            item.scroll_root.clip.main.to_rectf(),
                                            item.scroll_root.clip.get_complex_clips(),
                                            None)
                    }
                    ScrollRootType::ScrollFrame => {
                        builder.define_scroll_frame(Some(our_id),
                                                    item.scroll_root.content_rect.to_rectf(),
                                                    item.scroll_root.clip.main.to_rectf(),
                                                    item.scroll_root.clip.get_complex_clips(),
                                                    None)
                    }
                };
                debug_assert!(our_id == webrender_id);

                builder.pop_clip_id();
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
             )
        }).collect()
    }
}
