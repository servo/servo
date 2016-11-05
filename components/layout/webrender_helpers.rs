/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// TODO(gw): This contains helper traits and implementations for converting Servo display lists
//           into WebRender display lists. In the future, this step should be completely removed.
//           This might be achieved by sharing types between WR and Servo display lists, or
//           completely converting layout to directly generate WebRender display lists, for example.

use app_units::Au;
use euclid::{Point2D, Rect, Size2D};
use gfx::display_list::{BorderRadii, BoxShadowClipMode, ClippingRegion};
use gfx::display_list::{DisplayItem, DisplayList, DisplayListTraversal};
use gfx::display_list::{StackingContext, StackingContextType};
use gfx_traits::{FragmentType, ScrollPolicy, StackingContextId, ScrollRootId};
use style::computed_values::{image_rendering, mix_blend_mode};
use style::computed_values::filter::{self, Filter};
use style::values::computed::BorderStyle;
use webrender_traits::{self, AuxiliaryListsBuilder, DisplayListId, PipelineId};

trait WebRenderStackingContextConverter {
    fn convert_to_webrender<'a>(&self,
                                traversal: &mut DisplayListTraversal<'a>,
                                api: &mut webrender_traits::RenderApi,
                                pipeline_id: webrender_traits::PipelineId,
                                epoch: webrender_traits::Epoch,
                                frame_builder: &mut WebRenderFrameBuilder)
                                -> webrender_traits::StackingContextId;

    fn convert_children_to_webrender<'a>(&self,
                                         traversal: &mut DisplayListTraversal<'a>,
                                         api: &mut webrender_traits::RenderApi,
                                         pipeline_id: webrender_traits::PipelineId,
                                         epoch: webrender_traits::Epoch,
                                         builder: &mut webrender_traits::DisplayListBuilder,
                                         frame_builder: &mut WebRenderFrameBuilder,
                                         force_positioned_stacking_level: bool);
}

pub trait WebRenderDisplayListConverter {
    fn convert_to_webrender(&self,
                            api: &mut webrender_traits::RenderApi,
                            pipeline_id: webrender_traits::PipelineId,
                            epoch: webrender_traits::Epoch,
                            frame_builder: &mut WebRenderFrameBuilder)
                            -> webrender_traits::StackingContextId;
}

trait WebRenderDisplayItemConverter {
    fn convert_to_webrender(&self,
                            builder: &mut webrender_traits::DisplayListBuilder,
                            frame_builder: &mut WebRenderFrameBuilder);
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

trait ToClipRegion {
    fn to_clip_region(&self, frame_builder: &mut WebRenderFrameBuilder)
                      -> webrender_traits::ClipRegion;
}

impl ToClipRegion for ClippingRegion {
    fn to_clip_region(&self, frame_builder: &mut WebRenderFrameBuilder)
                      -> webrender_traits::ClipRegion {
        webrender_traits::ClipRegion::new(&self.main.to_rectf(),
                                   self.complex.iter().map(|complex_clipping_region| {
                                       webrender_traits::ComplexClipRegion::new(
                                           complex_clipping_region.rect.to_rectf(),
                                           complex_clipping_region.radii.to_border_radius(),
                                        )
                                   }).collect(),
                                   None,
                                   &mut frame_builder.auxiliary_lists_builder)
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
                                         api: &mut webrender_traits::RenderApi,
                                         pipeline_id: webrender_traits::PipelineId,
                                         epoch: webrender_traits::Epoch,
                                         builder: &mut webrender_traits::DisplayListBuilder,
                                         frame_builder: &mut WebRenderFrameBuilder,
                                         _force_positioned_stacking_level: bool) {
        while let Some(item) = traversal.next() {
            match item {
                &DisplayItem::PushStackingContext(ref stacking_context_item) => {
                    let stacking_context = &stacking_context_item.stacking_context;
                    debug_assert!(stacking_context.context_type == StackingContextType::Real);

                    let stacking_context_id =
                        stacking_context.convert_to_webrender(traversal,
                                                              api,
                                                              pipeline_id,
                                                              epoch,
                                                              frame_builder);
                    builder.push_stacking_context(stacking_context_id);

                }
                &DisplayItem::PopStackingContext(_) => return,
                _ => item.convert_to_webrender(builder, frame_builder),
            }
        }
    }

    fn convert_to_webrender<'a>(&self,
                                traversal: &mut DisplayListTraversal<'a>,
                                api: &mut webrender_traits::RenderApi,
                                pipeline_id: webrender_traits::PipelineId,
                                epoch: webrender_traits::Epoch,
                                frame_builder: &mut WebRenderFrameBuilder)
                                -> webrender_traits::StackingContextId {
        let webrender_scroll_policy = match self.scroll_policy {
            ScrollPolicy::Scrollable => webrender_traits::ScrollPolicy::Scrollable,
            ScrollPolicy::FixedPosition => webrender_traits::ScrollPolicy::Fixed,
        };

        let scroll_layer_id = if let Some(scroll_root_id) = self.overflow_scroll_id {
            Some(frame_builder.next_scroll_layer_id(scroll_root_id))
        } else if self.id == StackingContextId::root() {
            Some(frame_builder.next_scroll_layer_id(ScrollRootId::root()))
        } else {
            None
        };

        let mut sc =
            webrender_traits::StackingContext::new(scroll_layer_id,
                                                   webrender_scroll_policy,
                                                   self.bounds.to_rectf(),
                                                   self.overflow.to_rectf(),
                                                   self.z_index,
                                                   &self.transform,
                                                   &self.perspective,
                                                   self.establishes_3d_context,
                                                   self.blend_mode.to_blend_mode(),
                                                   self.filters.to_filter_ops(),
                                                   &mut frame_builder.auxiliary_lists_builder);

        let mut builder = webrender_traits::DisplayListBuilder::new();
        self.convert_children_to_webrender(traversal,
                                           api,
                                           pipeline_id,
                                           epoch,
                                           &mut builder,
                                           frame_builder,
                                           false);

        frame_builder.add_display_list(api, builder.finalize(), &mut sc);
        frame_builder.add_stacking_context(api, pipeline_id, sc)
    }
}

impl WebRenderDisplayListConverter for DisplayList {
    fn convert_to_webrender(&self,
                            api: &mut webrender_traits::RenderApi,
                            pipeline_id: webrender_traits::PipelineId,
                            epoch: webrender_traits::Epoch,
                            frame_builder: &mut WebRenderFrameBuilder)
                            -> webrender_traits::StackingContextId {
        let mut traversal = DisplayListTraversal::new(self);
        let item = traversal.next();
        match item {
            Some(&DisplayItem::PushStackingContext(ref stacking_context_item)) => {
                let stacking_context = &stacking_context_item.stacking_context;
                stacking_context.convert_to_webrender(&mut traversal,
                                                      api,
                                                      pipeline_id,
                                                      epoch,
                                                      frame_builder)
            }
            _ => unreachable!("DisplayList did not start with StackingContext."),

        }
    }
}

impl WebRenderDisplayItemConverter for DisplayItem {
    fn convert_to_webrender(&self,
                            builder: &mut webrender_traits::DisplayListBuilder,
                            frame_builder: &mut WebRenderFrameBuilder) {
        match *self {
            DisplayItem::SolidColor(ref item) => {
                let color = item.color;
                if color.a > 0.0 {
                    builder.push_rect(item.base.bounds.to_rectf(),
                                      item.base.clip.to_clip_region(frame_builder),
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
                            let glyph = webrender_traits::GlyphInstance {
                                index: glyph.id(),
                                x: (origin.x + glyph_offset.x).to_f32_px(),
                                y: (origin.y + glyph_offset.y).to_f32_px(),
                            };
                            glyphs.push(glyph);
                        }
                        origin.x = origin.x + glyph_advance;
                    };
                }

                if glyphs.len() > 0 {
                    builder.push_text(item.base.bounds.to_rectf(),
                                      item.base.clip.to_clip_region(frame_builder),
                                      glyphs,
                                      item.text_run.font_key,
                                      item.text_color,
                                      item.text_run.actual_pt_size,
                                      item.blur_radius,
                                      &mut frame_builder.auxiliary_lists_builder);
                }
            }
            DisplayItem::Image(ref item) => {
                if let Some(id) = item.webrender_image.key {
                    if item.stretch_size.width > Au(0) &&
                       item.stretch_size.height > Au(0) {
                        builder.push_image(item.base.bounds.to_rectf(),
                                           item.base.clip.to_clip_region(frame_builder),
                                           item.stretch_size.to_sizef(),
                                           item.tile_spacing.to_sizef(),
                                           item.image_rendering.to_image_rendering(),
                                           id);
                    }
                }
            }
            DisplayItem::WebGL(ref item) => {
                builder.push_webgl_canvas(item.base.bounds.to_rectf(),
                                          item.base.clip.to_clip_region(frame_builder),
                                          item.context_id);
            }
            DisplayItem::Border(ref item) => {
                let rect = item.base.bounds.to_rectf();
                let left = webrender_traits::BorderSide {
                    width: item.border_widths.left.to_f32_px(),
                    color: item.color.left,
                    style: item.style.left.to_border_style(),
                };
                let top = webrender_traits::BorderSide {
                    width: item.border_widths.top.to_f32_px(),
                    color: item.color.top,
                    style: item.style.top.to_border_style(),
                };
                let right = webrender_traits::BorderSide {
                    width: item.border_widths.right.to_f32_px(),
                    color: item.color.right,
                    style: item.style.right.to_border_style(),
                };
                let bottom = webrender_traits::BorderSide {
                    width: item.border_widths.bottom.to_f32_px(),
                    color: item.color.bottom,
                    style: item.style.bottom.to_border_style(),
                };
                let radius = item.radius.to_border_radius();
                builder.push_border(rect,
                                    item.base.clip.to_clip_region(frame_builder),
                                    left,
                                    top,
                                    right,
                                    bottom,
                                    radius);
            }
            DisplayItem::Gradient(ref item) => {
                let rect = item.base.bounds.to_rectf();
                let start_point = item.start_point.to_pointf();
                let end_point = item.end_point.to_pointf();
                builder.push_gradient(rect,
                                      item.base.clip.to_clip_region(frame_builder),
                                      start_point,
                                      end_point,
                                      item.stops.clone(),
                                      &mut frame_builder.auxiliary_lists_builder);
            }
            DisplayItem::Line(..) => {
                println!("TODO DisplayItem::Line");
            }
            DisplayItem::BoxShadow(ref item) => {
                let rect = item.base.bounds.to_rectf();
                let box_bounds = item.box_bounds.to_rectf();
                builder.push_box_shadow(rect,
                                        item.base.clip.to_clip_region(frame_builder),
                                        box_bounds,
                                        item.offset.to_pointf(),
                                        item.color,
                                        item.blur_radius.to_f32_px(),
                                        item.spread_radius.to_f32_px(),
                                        item.border_radius.to_f32_px(),
                                        item.clip_mode.to_clip_mode());
            }
            DisplayItem::Iframe(ref item) => {
                let rect = item.base.bounds.to_rectf();
                let pipeline_id = item.iframe.to_webrender();
                builder.push_iframe(rect,
                                    item.base.clip.to_clip_region(frame_builder),
                                    pipeline_id);
            }
            DisplayItem::PushStackingContext(_) | DisplayItem::PopStackingContext(_) => {}
        }
    }
}

pub struct WebRenderFrameBuilder {
    pub stacking_contexts: Vec<(webrender_traits::StackingContextId,
                                webrender_traits::StackingContext)>,
    pub display_lists: Vec<(DisplayListId, webrender_traits::BuiltDisplayList)>,
    pub auxiliary_lists_builder: AuxiliaryListsBuilder,
    pub root_pipeline_id: PipelineId,
    pub next_scroll_layer_id: usize,
}

impl WebRenderFrameBuilder {
    pub fn new(root_pipeline_id: PipelineId) -> WebRenderFrameBuilder {
        WebRenderFrameBuilder {
            stacking_contexts: vec![],
            display_lists: vec![],
            auxiliary_lists_builder: AuxiliaryListsBuilder::new(),
            root_pipeline_id: root_pipeline_id,
            next_scroll_layer_id: 0,
        }
    }

    pub fn add_stacking_context(&mut self,
                                api: &mut webrender_traits::RenderApi,
                                pipeline_id: PipelineId,
                                stacking_context: webrender_traits::StackingContext)
                                -> webrender_traits::StackingContextId {
        assert!(pipeline_id == self.root_pipeline_id);
        let id = api.next_stacking_context_id();
        self.stacking_contexts.push((id, stacking_context));
        id
    }

    pub fn add_display_list(&mut self,
                            api: &mut webrender_traits::RenderApi,
                            display_list: webrender_traits::BuiltDisplayList,
                            stacking_context: &mut webrender_traits::StackingContext)
                            -> DisplayListId {
        let id = api.next_display_list_id();
        stacking_context.display_lists.push(id);
        self.display_lists.push((id, display_list));
        id
    }

    pub fn next_scroll_layer_id(&mut self,
                                scroll_root_id: ScrollRootId)
                                -> webrender_traits::ScrollLayerId {
        let scroll_layer_id = self.next_scroll_layer_id;
        self.next_scroll_layer_id += 1;
        webrender_traits::ScrollLayerId::new(self.root_pipeline_id,
                                             scroll_layer_id,
                                             scroll_root_id.convert_to_webrender())

    }
}

trait WebRenderScrollRootIdConverter {
    fn convert_to_webrender(&self) -> webrender_traits::ServoScrollRootId;
}

impl WebRenderScrollRootIdConverter for ScrollRootId {
    fn convert_to_webrender(&self) -> webrender_traits::ServoScrollRootId {
        webrender_traits::ServoScrollRootId(self.0)
    }
}

trait WebRenderFragmentTypeConverter {
    fn convert_to_webrender(&self) -> webrender_traits::FragmentType;
}

impl WebRenderFragmentTypeConverter for FragmentType {
    fn convert_to_webrender(&self) -> webrender_traits::FragmentType {
        match *self {
            FragmentType::FragmentBody => webrender_traits::FragmentType::FragmentBody,
            FragmentType::BeforePseudoContent => {
                webrender_traits::FragmentType::BeforePseudoContent
            }
            FragmentType::AfterPseudoContent => webrender_traits::FragmentType::AfterPseudoContent,
        }
    }
}
