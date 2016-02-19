/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// TODO(gw): This contains helper traits and implementations for converting Servo display lists
//           into WebRender display lists. In the future, this step should be completely removed.
//           This might be achieved by sharing types between WR and Servo display lists, or
//           completely converting layout to directly generate WebRender display lists, for example.

use app_units::Au;
use azure::azure_hl::Color;
use euclid::num::Zero;
use euclid::{Point2D, Rect, Size2D};
use gfx::display_list::{BorderRadii, BoxShadowClipMode, ClippingRegion};
use gfx::display_list::{DisplayItem, DisplayList, DisplayListEntry, DisplayListSection};
use gfx::display_list::{DisplayListTraversal, GradientStop, StackingContext, StackingContextType};
use gfx_traits::ScrollPolicy;
use msg::constellation_msg::ConvertPipelineIdToWebRender;
use style::computed_values::filter::{self, Filter};
use style::computed_values::{image_rendering, mix_blend_mode};
use style::values::computed::BorderStyle;
use webrender_traits;

trait WebRenderStackingContextConverter {
    fn convert_to_webrender<'a>(&self,
                                traversal: &mut DisplayListTraversal<'a>,
                                api: &webrender_traits::RenderApi,
                                pipeline_id: webrender_traits::PipelineId,
                                epoch: webrender_traits::Epoch,
                                scroll_layer_id: Option<webrender_traits::ScrollLayerId>)
                                -> webrender_traits::StackingContextId;

    fn convert_children_to_webrender<'a>(&self,
                                         traversal: &mut DisplayListTraversal<'a>,
                                         api: &webrender_traits::RenderApi,
                                         pipeline_id: webrender_traits::PipelineId,
                                         epoch: webrender_traits::Epoch,
                                         scroll_layer_id: Option<webrender_traits::ScrollLayerId>,
                                         builder: &mut webrender_traits::DisplayListBuilder,
                                         force_positioned_stacking_level: bool);

    fn web_render_stacking_level(&self) -> webrender_traits::StackingLevel;
}

pub trait WebRenderDisplayListConverter {
    fn convert_to_webrender(&self,
                            api: &webrender_traits::RenderApi,
                            pipeline_id: webrender_traits::PipelineId,
                            epoch: webrender_traits::Epoch,
                            scroll_layer_id: Option<webrender_traits::ScrollLayerId>)
                            -> webrender_traits::StackingContextId;
}

trait WebRenderDisplayItemConverter {
    fn convert_to_webrender(&self,
                            level: webrender_traits::StackingLevel,
                            builder: &mut webrender_traits::DisplayListBuilder);
}

trait WebRenderDisplayListEntryConverter {
    fn web_render_stacking_level(&self) -> webrender_traits::StackingLevel;
}

impl WebRenderDisplayListEntryConverter for DisplayListEntry {
    fn web_render_stacking_level(&self) -> webrender_traits::StackingLevel {
        match self.section {
            DisplayListSection::BackgroundAndBorders =>
                webrender_traits::StackingLevel::BackgroundAndBorders,
            DisplayListSection::BlockBackgroundsAndBorders =>
                webrender_traits::StackingLevel::BlockBackgroundAndBorders,
            DisplayListSection::Content => webrender_traits::StackingLevel::Content,
            DisplayListSection::Outlines => webrender_traits::StackingLevel::Outlines,
        }
    }
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
    fn to_sizef(&self) -> Size2D<f32>;
}

trait ToPointF {
    fn to_pointf(&self) -> Point2D<f32>;
}

impl ToPointF for Point2D<Au> {
    fn to_pointf(&self) -> Point2D<f32> {
        Point2D::new(self.x.to_f32_px(), self.y.to_f32_px())
    }
}

impl ToSizeF for Size2D<Au> {
    fn to_sizef(&self) -> Size2D<f32> {
        Size2D::new(self.width.to_f32_px(), self.height.to_f32_px())
    }
}

trait ToRectF {
    fn to_rectf(&self) -> Rect<f32>;
}

impl ToRectF for Rect<Au> {
    fn to_rectf(&self) -> Rect<f32> {
        let x = self.origin.x.to_f32_px();
        let y = self.origin.y.to_f32_px();
        let w = self.size.width.to_f32_px();
        let h = self.size.height.to_f32_px();
        Rect::new(Point2D::new(x, y), Size2D::new(w, h))
    }
}

trait ToColorF {
    fn to_colorf(&self) -> webrender_traits::ColorF;
}

impl ToColorF for Color {
    fn to_colorf(&self) -> webrender_traits::ColorF {
        webrender_traits::ColorF::new(self.r, self.g, self.b, self.a)
    }
}

trait ToGradientStop {
    fn to_gradient_stop(&self) -> webrender_traits::GradientStop;
}

impl ToGradientStop for GradientStop {
    fn to_gradient_stop(&self) -> webrender_traits::GradientStop {
        webrender_traits::GradientStop {
            offset: self.offset,
            color: self.color.to_colorf(),
        }
    }
}

trait ToClipRegion {
    fn to_clip_region(&self) -> webrender_traits::ClipRegion;
}

impl ToClipRegion for ClippingRegion {
    fn to_clip_region(&self) -> webrender_traits::ClipRegion {
        webrender_traits::ClipRegion::new(self.main.to_rectf(),
                                   self.complex.iter().map(|complex_clipping_region| {
                                       webrender_traits::ComplexClipRegion::new(
                                           complex_clipping_region.rect.to_rectf(),
                                           complex_clipping_region.radii.to_border_radius(),
                                        )
                                   }).collect())
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

trait ToBlendMode {
    fn to_blend_mode(&self) -> webrender_traits::MixBlendMode;
}

impl ToBlendMode for mix_blend_mode::T {
    fn to_blend_mode(&self) -> webrender_traits::MixBlendMode {
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
            image_rendering::T::CrispEdges => webrender_traits::ImageRendering::CrispEdges,
            image_rendering::T::Auto => webrender_traits::ImageRendering::Auto,
            image_rendering::T::Pixelated => webrender_traits::ImageRendering::Pixelated,
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
                Filter::HueRotate(angle) => result.push(webrender_traits::FilterOp::HueRotate(angle.0)),
                Filter::Invert(amount) => result.push(webrender_traits::FilterOp::Invert(amount)),
                Filter::Opacity(amount) => result.push(webrender_traits::FilterOp::Opacity(amount)),
                Filter::Saturate(amount) => result.push(webrender_traits::FilterOp::Saturate(amount)),
                Filter::Sepia(amount) => result.push(webrender_traits::FilterOp::Sepia(amount)),
            }
        }
        result
    }
}

impl WebRenderStackingContextConverter for StackingContext {
    fn convert_children_to_webrender<'a>(&self,
                                         traversal: &mut DisplayListTraversal<'a>,
                                         api: &webrender_traits::RenderApi,
                                         pipeline_id: webrender_traits::PipelineId,
                                         epoch: webrender_traits::Epoch,
                                         scroll_layer_id: Option<webrender_traits::ScrollLayerId>,
                                         builder: &mut webrender_traits::DisplayListBuilder,
                                         force_positioned_stacking_level: bool) {
        for child in self.children.iter() {
            while let Some(item) = traversal.advance(self) {
                let stacking_level = if force_positioned_stacking_level {
                    webrender_traits::StackingLevel::PositionedContent
                } else {
                    item.web_render_stacking_level()
                };
                item.item.convert_to_webrender(stacking_level, builder);

            }
            if child.context_type == StackingContextType::Real {
                let stacking_context_id = child.convert_to_webrender(traversal,
                                                                     api,
                                                                     pipeline_id,
                                                                     epoch,
                                                                     None);
                builder.push_stacking_context(child.web_render_stacking_level(),
                                              stacking_context_id);
            } else {
                child.convert_children_to_webrender(traversal,
                                                    api,
                                                    pipeline_id,
                                                    epoch,
                                                    scroll_layer_id,
                                                    builder,
                                                    true);
            }
        }

        while let Some(item) = traversal.advance(self) {
            item.item.convert_to_webrender(webrender_traits::StackingLevel::PositionedContent,
                                           builder);
        }
    }

    fn convert_to_webrender<'a>(&self,
                                traversal: &mut DisplayListTraversal<'a>,
                                api: &webrender_traits::RenderApi,
                                pipeline_id: webrender_traits::PipelineId,
                                epoch: webrender_traits::Epoch,
                                scroll_layer_id: Option<webrender_traits::ScrollLayerId>)
                                -> webrender_traits::StackingContextId {
        let scroll_policy = self.layer_info
                                .map_or(webrender_traits::ScrollPolicy::Scrollable, |info| {
            match info.scroll_policy {
                ScrollPolicy::Scrollable => webrender_traits::ScrollPolicy::Scrollable,
                ScrollPolicy::FixedPosition => webrender_traits::ScrollPolicy::Fixed,
            }
        });

        let mut sc = webrender_traits::StackingContext::new(scroll_layer_id,
                                                            scroll_policy,
                                                            self.bounds.to_rectf(),
                                                            self.overflow.to_rectf(),
                                                            self.z_index,
                                                            &self.transform,
                                                            &self.perspective,
                                                            self.establishes_3d_context,
                                                            self.blend_mode.to_blend_mode(),
                                                            self.filters.to_filter_ops());
        let mut builder = webrender_traits::DisplayListBuilder::new();
        self.convert_children_to_webrender(traversal,
                                           api,
                                           pipeline_id,
                                           epoch,
                                           scroll_layer_id,
                                           &mut builder,
                                           false);
        api.add_display_list(builder, &mut sc, pipeline_id, epoch);
        api.add_stacking_context(sc, pipeline_id, epoch)
    }

    fn web_render_stacking_level(&self) -> webrender_traits::StackingLevel {
        match self.context_type {
            StackingContextType::Real | StackingContextType::PseudoPositioned =>
                webrender_traits::StackingLevel::PositionedContent,
            StackingContextType::PseudoFloat => webrender_traits::StackingLevel::Floats,
        }
    }
}

impl WebRenderDisplayListConverter for DisplayList {
    fn convert_to_webrender(&self,
                            api: &webrender_traits::RenderApi,
                            pipeline_id: webrender_traits::PipelineId,
                            epoch: webrender_traits::Epoch,
                            scroll_layer_id: Option<webrender_traits::ScrollLayerId>)
                            -> webrender_traits::StackingContextId {
        let mut traversal = DisplayListTraversal {
            display_list: self,
            current_item_index: 0,
            last_item_index: self.list.len() - 1,
        };

        self.root_stacking_context.convert_to_webrender(&mut traversal,
                                                        api,
                                                        pipeline_id,
                                                        epoch,
                                                        scroll_layer_id)
    }
}

impl WebRenderDisplayItemConverter for DisplayItem {
    fn convert_to_webrender(&self,
                            level: webrender_traits::StackingLevel,
                            builder: &mut webrender_traits::DisplayListBuilder) {
        match *self {
            DisplayItem::SolidColorClass(ref item) => {
                let color = item.color.to_colorf();
                if color.a > 0.0 {
                    builder.push_rect(level,
                                      item.base.bounds.to_rectf(),
                                      item.base.clip.to_clip_region(),
                                      color);
                }
            }
            DisplayItem::TextClass(ref item) => {
                let mut origin = item.baseline_origin.clone();
                let mut glyphs = vec!();

                for slice in item.text_run.natural_word_slices_in_visual_order(&item.range) {
                    for glyph in slice.glyphs.iter_glyphs_for_char_range(&slice.range) {
                        let glyph_advance = glyph.advance();
                        let glyph_offset = glyph.offset().unwrap_or(Point2D::zero());
                        let glyph = webrender_traits::GlyphInstance {
                            index: glyph.id(),
                            x: (origin.x + glyph_offset.x).to_f32_px(),
                            y: (origin.y + glyph_offset.y).to_f32_px(),
                        };
                        origin = Point2D::new(origin.x + glyph_advance, origin.y);
                        glyphs.push(glyph);
                    };
                }

                if glyphs.len() > 0 {
                    builder.push_text(level,
                                      item.base.bounds.to_rectf(),
                                      item.base.clip.to_clip_region(),
                                      glyphs,
                                      item.text_run.font_key.expect("Font not added to webrender!"),
                                      item.text_color.to_colorf(),
                                      item.text_run.actual_pt_size,
                                      item.blur_radius);
                }
            }
            DisplayItem::ImageClass(ref item) => {
                if let Some(id) = item.image.id {
                    if item.stretch_size.width > Au(0) &&
                       item.stretch_size.height > Au(0) {
                        builder.push_image(level,
                                           item.base.bounds.to_rectf(),
                                           item.base.clip.to_clip_region(),
                                           item.stretch_size.to_sizef(),
                                           item.image_rendering.to_image_rendering(),
                                           id);
                    }
                }
            }
            DisplayItem::WebGLClass(ref item) => {
                builder.push_webgl_canvas(level,
                                          item.base.bounds.to_rectf(),
                                          item.base.clip.to_clip_region(),
                                          item.context_id);
            }
            DisplayItem::BorderClass(ref item) => {
                let rect = item.base.bounds.to_rectf();
                let left = webrender_traits::BorderSide {
                    width: item.border_widths.left.to_f32_px(),
                    color: item.color.left.to_colorf(),
                    style: item.style.left.to_border_style(),
                };
                let top = webrender_traits::BorderSide {
                    width: item.border_widths.top.to_f32_px(),
                    color: item.color.top.to_colorf(),
                    style: item.style.top.to_border_style(),
                };
                let right = webrender_traits::BorderSide {
                    width: item.border_widths.right.to_f32_px(),
                    color: item.color.right.to_colorf(),
                    style: item.style.right.to_border_style(),
                };
                let bottom = webrender_traits::BorderSide {
                    width: item.border_widths.bottom.to_f32_px(),
                    color: item.color.bottom.to_colorf(),
                    style: item.style.bottom.to_border_style(),
                };
                let radius = item.radius.to_border_radius();
                builder.push_border(level,
                                    rect,
                                    item.base.clip.to_clip_region(),
                                    left,
                                    top,
                                    right,
                                    bottom,
                                    radius);
            }
            DisplayItem::GradientClass(ref item) => {
                let rect = item.base.bounds.to_rectf();
                let start_point = item.start_point.to_pointf();
                let end_point = item.end_point.to_pointf();
                let mut stops = Vec::new();
                for stop in &item.stops {
                    stops.push(stop.to_gradient_stop());
                }
                builder.push_gradient(level,
                                      rect,
                                      item.base.clip.to_clip_region(),
                                      start_point,
                                      end_point,
                                      stops);
            }
            DisplayItem::LineClass(..) => {
                println!("TODO DisplayItem::LineClass");
            }
            DisplayItem::LayeredItemClass(..) |
            DisplayItem::NoopClass(..) => {
                panic!("Unexpected in webrender!");
            }
            DisplayItem::BoxShadowClass(ref item) => {
                let rect = item.base.bounds.to_rectf();
                let box_bounds = item.box_bounds.to_rectf();
                builder.push_box_shadow(level,
                                        rect,
                                        item.base.clip.to_clip_region(),
                                        box_bounds,
                                        item.offset.to_pointf(),
                                        item.color.to_colorf(),
                                        item.blur_radius.to_f32_px(),
                                        item.spread_radius.to_f32_px(),
                                        item.border_radius.to_f32_px(),
                                        item.clip_mode.to_clip_mode());
            }
            DisplayItem::IframeClass(ref item) => {
                let rect = item.base.bounds.to_rectf();
                let pipeline_id = item.iframe.to_webrender();
                builder.push_iframe(level,
                                    rect,
                                    item.base.clip.to_clip_region(),
                                    pipeline_id);
            }
        }
    }
}
