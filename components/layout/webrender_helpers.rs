/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use gfx::display_list::{ClippingRegion, DisplayItem, StackingContextType};
use style::computed_values::{image_rendering, mix_blend_mode};
use style::computed_values::filter::{self, Filter};
use style::values::computed::BorderStyle;
use webrender_traits::{self, DisplayListBuilder};

pub trait WebRenderDisplayItemConverter {
    fn convert_to_webrender(&self, builder: &mut DisplayListBuilder);
}

pub trait ToBorderStyle {
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

trait ToClipRegion {
    fn to_clip_region(&self, builder: &mut DisplayListBuilder) -> webrender_traits::ClipRegion;
}

impl ToClipRegion for ClippingRegion {
    fn to_clip_region(&self, builder: &mut DisplayListBuilder) -> webrender_traits::ClipRegion {
        builder.new_clip_region(&self.main,
                                self.complex.iter().map(|complex_clipping_region| {
                                    webrender_traits::ComplexClipRegion::new(
                                        complex_clipping_region.rect,
                                        complex_clipping_region.radii,
                                    )
                                }).collect(),
                                None)
    }
}

pub trait ToBlendMode {
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

pub trait ToImageRendering {
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

pub trait ToFilterOps {
    fn to_filter_ops(&self) -> Vec<webrender_traits::FilterOp>;
}

impl ToFilterOps for filter::T {
    fn to_filter_ops(&self) -> Vec<webrender_traits::FilterOp> {
        self.filters.iter().map(|filter| {
            match *filter {
                Filter::Blur(radius) => webrender_traits::FilterOp::Blur(radius),
                Filter::Brightness(amount) => webrender_traits::FilterOp::Brightness(amount),
                Filter::Contrast(amount) => webrender_traits::FilterOp::Contrast(amount),
                Filter::Grayscale(amount) => webrender_traits::FilterOp::Grayscale(amount),
                Filter::HueRotate(angle) => webrender_traits::FilterOp::HueRotate(angle.0),
                Filter::Invert(amount) => webrender_traits::FilterOp::Invert(amount),
                Filter::Opacity(amount) => webrender_traits::FilterOp::Opacity(amount),
                Filter::Saturate(amount) => webrender_traits::FilterOp::Saturate(amount),
                Filter::Sepia(amount) => webrender_traits::FilterOp::Sepia(amount),
            }
        }).collect()
    }
}

impl WebRenderDisplayItemConverter for DisplayItem {
    fn convert_to_webrender(&self, builder: &mut DisplayListBuilder) {
        match *self {
            DisplayItem::SolidColor(ref item) => {
                let clip = item.base.clip.to_clip_region(builder);
                builder.push_rect(item.base.id, &item.base.bounds, clip, item.color);
            }
            DisplayItem::Text(ref item) => {
                let clip = item.base.clip.to_clip_region(builder);
                builder.push_text(item.base.id,
                                  &item.base.bounds,
                                  clip,
                                  &item.glyphs,
                                  item.font_key,
                                  item.text_color,
                                  item.size,
                                  item.blur_radius);
            }
            DisplayItem::Image(ref item) => {
                if let Some(id) = item.webrender_image.key {
                    if item.stretch_size.width > 0.0 && item.stretch_size.height > 0.0 {
                        let clip = item.base.clip.to_clip_region(builder);
                        builder.push_image(item.base.id,
                                           &item.base.bounds,
                                           clip,
                                           &item.stretch_size,
                                           &item.tile_spacing,
                                           item.image_rendering,
                                           id);
                    }
                }
            }
            DisplayItem::WebGL(ref item) => {
                let clip = item.base.clip.to_clip_region(builder);
                builder.push_webgl_canvas(item.base.id, &item.base.bounds, clip, item.context_id);
            }
            DisplayItem::Border(ref item) => {
                let clip = item.base.clip.to_clip_region(builder);
                builder.push_border(item.base.id,
                                    &item.base.bounds,
                                    clip,
                                    &item.left,
                                    &item.top,
                                    &item.right,
                                    &item.bottom,
                                    item.radius);
            }
            DisplayItem::Gradient(ref item) => {
                let clip = item.base.clip.to_clip_region(builder);
                builder.push_gradient(item.base.id,
                                      &item.base.bounds,
                                      clip,
                                      &item.start_point,
                                      &item.end_point,
                                      item.stops.clone());
            }
            DisplayItem::BoxShadow(ref item) => {
                let clip = item.base.clip.to_clip_region(builder);
                builder.push_box_shadow(item.base.id,
                                        &item.base.bounds,
                                        clip,
                                        &item.box_bounds,
                                        &item.offset,
                                        item.color,
                                        item.blur_radius,
                                        item.spread_radius,
                                        item.border_radius,
                                        item.clip_mode);
            }
            DisplayItem::Iframe(ref item) => {
                let clip = item.base.clip.to_clip_region(builder);
                builder.push_iframe(item.base.id,
                                    &item.base.bounds,
                                    clip,
                                    item.iframe.to_webrender());
            }
            DisplayItem::PushStackingContext(ref item) => {
                debug_assert!(item.stacking_context.context_type == StackingContextType::Real);
                builder.push_stacking_context(item.stacking_context.id,
                                              item.stacking_context.scroll_policy,
                                              &item.stacking_context.bounds,
                                              &item.stacking_context.overflow,
                                              item.stacking_context.z_index,
                                              &item.stacking_context.transform,
                                              &item.stacking_context.perspective,
                                              item.stacking_context.blend_mode,
                                              item.stacking_context.filters.clone());
            }
            DisplayItem::PopStackingContext(_) => {
                builder.pop_stacking_context();
            }
            DisplayItem::PushScrollRoot(ref item) => {
                builder.push_scroll_layer(&item.scroll_root.clip,
                                          &item.scroll_root.size,
                                          item.scroll_root.id);
            }
            DisplayItem::PopScrollRoot(_) => {
                builder.pop_scroll_layer();
            }
        }
    }
}

