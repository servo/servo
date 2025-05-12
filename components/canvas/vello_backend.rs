/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unused_variables)]
#![allow(unsafe_code)]

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::rc::Rc;

use canvas_traits::canvas::*;
use euclid::default::{Point2D, Rect, Size2D, Transform2D, Vector2D};
use fonts::{ByteIndex, FontIdentifier, FontTemplateRefMethods as _};
use range::Range;
use style::color::AbsoluteColor;
use vello::kurbo::{self, Shape as _};
use vello::peniko;
use vello::wgpu::{
    BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Device, Extent3d, MapMode, Queue,
    TexelCopyBufferInfo, TexelCopyBufferLayout, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureViewDescriptor,
};

use crate::backend::{
    Backend, DrawOptionsHelpers, GenericDrawTarget, GenericPathBuilder, PathHelpers,
    PatternHelpers, StrokeOptionsHelpers,
};
use crate::canvas_data::{CanvasPaintState, Filter, TextRun};

thread_local! {
    /// The shared font cache used by all canvases that render on a thread. It would be nicer
    /// to have a global cache, but it looks like font-kit uses a per-thread FreeType, so
    /// in order to ensure that fonts are particular to a thread we have to make our own
    /// cache thread local as well.
    /// TODO: this is not true for vello
    static SHARED_FONT_CACHE: RefCell<HashMap<FontIdentifier, peniko::Font>> = RefCell::default();
}

#[derive(Clone, Default)]
pub(crate) struct VelloBackend;

impl Backend for VelloBackend {
    type Pattern<'a> = peniko::Brush;
    type StrokeOptions = kurbo::Stroke;
    type Color = peniko::Color;
    type DrawOptions = DrawOptions;
    type CompositionOp = peniko::BlendMode;
    type DrawTarget = DrawTarget;
    type PathBuilder = kurbo::BezPath;
    type SourceSurface = Vec<u8>; // TODO: this should be texture
    type Path = kurbo::BezPath;
    type GradientStop = peniko::ColorStop;
    type GradientStops = peniko::ColorStops;

    fn get_composition_op(&self, opts: &Self::DrawOptions) -> Self::CompositionOp {
        opts.blend_mode
    }

    fn need_to_draw_shadow(&self, color: &Self::Color) -> bool {
        color.components[3] != 0.
    }

    fn set_shadow_color(&mut self, color: AbsoluteColor, state: &mut CanvasPaintState<'_, Self>) {
        state.shadow_color = color.convert();
    }

    fn set_fill_style(
        &mut self,
        style: FillOrStrokeStyle,
        state: &mut CanvasPaintState<'_, Self>,
        _drawtarget: &Self::DrawTarget,
    ) {
        state.fill_style = style.convert();
    }

    fn set_stroke_style(
        &mut self,
        style: FillOrStrokeStyle,
        state: &mut CanvasPaintState<'_, Self>,
        _drawtarget: &Self::DrawTarget,
    ) {
        state.stroke_style = style.convert();
    }

    fn set_global_composition(
        &mut self,
        op: CompositionOrBlending,
        state: &mut CanvasPaintState<'_, Self>,
    ) {
        state.draw_options.blend_mode = op.convert();
    }

    fn create_drawtarget(&self, size: Size2D<u64>) -> Self::DrawTarget {
        DrawTarget::new(size.cast())
    }

    fn new_paint_state<'a>(&self) -> CanvasPaintState<'a, Self> {
        let pattern = peniko::Brush::Solid(peniko::color::AlphaColor::BLACK);
        CanvasPaintState {
            draw_options: DrawOptions::default(),
            fill_style: pattern.clone(),
            stroke_style: pattern,
            stroke_opts: Default::default(),
            transform: Transform2D::identity(),
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_blur: 0.0,
            shadow_color: peniko::Color::TRANSPARENT,
            font_style: None,
            text_align: TextAlign::default(),
            text_baseline: TextBaseline::default(),
            _backend: std::marker::PhantomData,
        }
    }
}

pub(crate) struct DrawTarget {
    device: Device,
    queue: Queue,
    renderer: Rc<RefCell<vello::Renderer>>,
    scene: vello::Scene,
    transform: kurbo::Affine,
    size: Size2D<u32>,
}

fn options() -> vello::RendererOptions {
    vello::RendererOptions {
        use_cpu: false,
        num_init_threads: NonZeroUsize::new(1),
        antialiasing_support: vello::AaSupport::area_only(),
        pipeline_cache: None,
    }
}

impl DrawTarget {
    fn new(size: Size2D<u32>) -> Self {
        let mut context = vello::util::RenderContext::new();
        let device_id = pollster::block_on(context.device(None)).unwrap();
        let device_handle = &mut context.devices[device_id];
        let device = device_handle.device.clone();
        let queue = device_handle.queue.clone();
        let renderer = vello::Renderer::new(&device, options()).unwrap();
        let scene = vello::Scene::new();
        Self {
            device,
            queue,
            renderer: Rc::new(RefCell::new(renderer)),
            scene,
            transform: kurbo::Affine::IDENTITY,
            size,
        }
    }
}

impl GenericDrawTarget<VelloBackend> for DrawTarget {
    fn clear_rect(&mut self, rect: &Rect<f32>) {
        // TODO: verify
        self.scene
            .push_layer(peniko::Compose::Clear, 0.0, self.transform, &rect.convert());
        self.scene.fill(
            peniko::Fill::NonZero,
            self.transform,
            peniko::BrushRef::Solid(peniko::color::AlphaColor::TRANSPARENT),
            None,
            &rect.convert(),
        );
        self.scene.pop_layer();
    }

    fn copy_surface(&mut self, surface: Vec<u8>, source: Rect<i32>, destination: Point2D<i32>) {
        self.scene.fill(
            peniko::Fill::NonZero,
            kurbo::Affine::IDENTITY,
            &peniko::Image {
                data: peniko::Blob::from(surface),
                format: peniko::ImageFormat::Rgba8,
                width: source.size.width as u32,
                height: source.size.height as u32,
                x_extend: peniko::Extend::Pad,
                y_extend: peniko::Extend::Pad,
                quality: peniko::ImageQuality::Low,
                alpha: 1.0,
            },
            None,
            &kurbo::Rect::from_origin_size(
                destination.cast::<f64>().convert(),
                source.size.cast::<f64>().convert(),
            ),
        );
    }

    fn create_path_builder(&self) -> kurbo::BezPath {
        kurbo::BezPath::new()
    }

    fn create_similar_draw_target(&self, size: &Size2D<i32>) -> Self {
        Self {
            device: self.device.clone(),
            queue: self.queue.clone(),
            renderer: self.renderer.clone(),
            scene: vello::Scene::new(),
            transform: kurbo::Affine::IDENTITY,
            size: size.cast(),
        }
    }

    fn create_source_surface_from_data(&self, data: &[u8]) -> Option<Vec<u8>> {
        // data is bgra
        let mut data = data.to_vec();
        pixels::generic_transform_inplace::<0, true, false>(&mut data);
        Some(data)
    }

    fn draw_surface(
        &mut self,
        surface: Vec<u8>,
        dest: Rect<f64>,
        source: Rect<f64>,
        filter: Filter,
        draw_options: &DrawOptions,
    ) {
        self.scene.fill(
            peniko::Fill::NonZero,
            self.transform,
            &peniko::Image {
                data: peniko::Blob::from(surface),
                format: peniko::ImageFormat::Rgba8,
                width: source.size.width as u32,
                height: source.size.height as u32,
                x_extend: peniko::Extend::Pad,
                y_extend: peniko::Extend::Pad,
                quality: peniko::ImageQuality::Low, // TODO: image smoothing
                alpha: 1.0,
            },
            Some(
                kurbo::Affine::translate((-dest.origin.x, -dest.origin.y)).then_scale_non_uniform(
                    source.size.width / dest.size.width,
                    source.size.height / dest.size.height,
                ),
            ),
            &dest.convert(),
        );
    }

    fn draw_surface_with_shadow(
        &self,
        surface: Vec<u8>,
        dest: &Point2D<f32>,
        color: &peniko::Color,
        offset: &Vector2D<f32>,
        sigma: f32,
        operator: peniko::BlendMode,
    ) {
        log::warn!("no support for drawing shadows");
    }

    fn fill(&mut self, path: &kurbo::BezPath, pattern: peniko::Brush, _draw_options: &DrawOptions) {
        self.scene
            .fill(peniko::Fill::NonZero, self.transform, &pattern, None, path);
    }

    fn fill_text(
        &mut self,
        text_runs: Vec<TextRun>,
        start: Point2D<f32>,
        pattern: &peniko::Brush,
        draw_options: &DrawOptions,
    ) {
        let mut advance = 0.;
        for run in text_runs.iter() {
            let glyphs = &run.glyphs;

            let template = &run.font.template;

            SHARED_FONT_CACHE.with(|font_cache| {
                let identifier = template.identifier();
                if !font_cache.borrow().contains_key(&identifier) {
                    font_cache.borrow_mut().insert(
                        identifier.clone(),
                        peniko::Font::new(
                            peniko::Blob::from(run.font.data().as_ref().to_vec()),
                            identifier.index(),
                        ),
                    );
                }

                let font_cache = font_cache.borrow();
                let Some(font) = font_cache.get(&identifier) else {
                    return;
                };

                self.scene
                    .draw_glyphs(font)
                    .transform(self.transform)
                    .brush(pattern)
                    .brush_alpha(draw_options.alpha)
                    .font_size(run.font.descriptor.pt_size.to_f32_px())
                    .draw(
                        peniko::Fill::NonZero,
                        glyphs
                            .iter_glyphs_for_byte_range(&Range::new(ByteIndex(0), glyphs.len()))
                            .map(|glyph| {
                                let glyph_offset = glyph.offset().unwrap_or(Point2D::zero());
                                let x = advance + start.x + glyph_offset.x.to_f32_px();
                                let y = start.y + glyph_offset.y.to_f32_px();
                                advance += glyph.advance().to_f32_px();
                                vello::Glyph {
                                    id: glyph.id(),
                                    x,
                                    y,
                                }
                            }),
                    );
            });
        }
    }

    fn fill_rect(
        &mut self,
        rect: &Rect<f32>,
        pattern: peniko::Brush,
        _draw_options: Option<&DrawOptions>,
    ) {
        self.scene.fill(
            peniko::Fill::NonZero,
            self.transform,
            &pattern,
            None,
            &rect.convert(),
        );
    }

    fn get_size(&self) -> Size2D<i32> {
        self.size.cast()
    }

    fn get_transform(&self) -> Transform2D<f32> {
        self.transform.convert()
    }

    fn pop_clip(&mut self) {
        self.scene.pop_layer();
    }

    fn push_clip(&mut self, path: &kurbo::BezPath) {
        self.scene
            .push_layer(peniko::Mix::Clip, 1.0, self.transform, path); // TODO: verify
    }

    fn set_transform(&mut self, matrix: &Transform2D<f32>) {
        self.transform = matrix.convert();
    }

    fn surface(&self) -> Vec<u8> {
        self.bytes().into_owned()
    }

    fn stroke(
        &mut self,
        path: &kurbo::BezPath,
        pattern: peniko::Brush,
        stroke_options: &kurbo::Stroke,
        _draw_options: &DrawOptions,
    ) {
        self.scene
            .stroke(stroke_options, self.transform, &pattern, None, path);
    }

    fn stroke_line(
        &mut self,
        start: Point2D<f32>,
        end: Point2D<f32>,
        pattern: peniko::Brush,
        stroke_options: &kurbo::Stroke,
        _draw_options: &DrawOptions,
    ) {
        self.scene.stroke(
            stroke_options,
            self.transform,
            &pattern,
            None,
            &kurbo::Line::new(start.convert(), end.convert()),
        );
    }

    fn stroke_rect(
        &mut self,
        rect: &Rect<f32>,
        pattern: peniko::Brush,
        stroke_options: &kurbo::Stroke,
        _draw_options: &DrawOptions,
    ) {
        self.scene.stroke(
            stroke_options,
            self.transform,
            &pattern,
            None,
            &rect.convert(),
        );
    }

    fn bytes(&self) -> Cow<[u8]> {
        let size = Extent3d {
            width: self.size.width,
            height: self.size.height,
            depth_or_array_layers: 1,
        };
        let target = self.device.create_texture(&TextureDescriptor {
            label: Some("Target texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&TextureViewDescriptor::default());
        self.renderer
            .borrow_mut()
            .render_to_texture(
                &self.device,
                &self.queue,
                &self.scene,
                &view,
                &vello::RenderParams {
                    base_color: peniko::color::AlphaColor::TRANSPARENT,
                    width: self.size.width,
                    height: self.size.height,
                    antialiasing_method: vello::AaConfig::Area,
                },
            )
            .unwrap();
        let padded_byte_width = (self.size.width * 4).next_multiple_of(256);
        let buffer_size = padded_byte_width as u64 * self.size.height as u64;
        let buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("val"),
            size: buffer_size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Copy out buffer"),
            });
        encoder.copy_texture_to_buffer(
            target.as_image_copy(),
            TexelCopyBufferInfo {
                buffer: &buffer,
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_byte_width),
                    rows_per_image: None,
                },
            },
            size,
        );
        self.queue.submit([encoder.finish()]);
        // TODO(perf): return buffer view
        let result = {
            let buf_slice = buffer.slice(..);
            let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
            buf_slice.map_async(MapMode::Read, move |v| sender.send(v).unwrap());
            vello::util::block_on_wgpu(&self.device, receiver.receive())
                .unwrap()
                .unwrap();
            let data = buf_slice.get_mapped_range();
            let mut result_unpadded = Vec::<u8>::with_capacity(
                (self.size.width * self.size.height * 4).try_into().unwrap(),
            );
            for row in 0..self.size.height {
                let start = (row * padded_byte_width).try_into().unwrap();
                result_unpadded.extend(&data[start..start + (self.size.width * 4) as usize]);
            }
            // TODO(perf): support both or make vello do bgra
            // swap RB
            pixels::generic_transform_inplace::<0, true, false>(&mut result_unpadded);
            result_unpadded
        };
        buffer.unmap();
        Cow::Owned(result)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DrawOptions {
    blend_mode: peniko::BlendMode,
    alpha: f32,
}

impl Default for DrawOptions {
    fn default() -> Self {
        Self {
            blend_mode: Default::default(),
            alpha: 1.,
        }
    }
}

impl DrawOptionsHelpers for DrawOptions {
    fn set_alpha(&mut self, val: f32) {
        self.alpha = val;
    }
}

impl GenericPathBuilder<VelloBackend> for kurbo::BezPath {
    fn arc(
        &mut self,
        origin: Point2D<f32>,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        anticlockwise: bool,
    ) {
        let mut arc = kurbo::Arc::new(
            origin.convert(),
            kurbo::Vec2::new(radius as f64, radius as f64),
            start_angle as f64,
            end_angle as f64 - start_angle as f64,
            0.,
        );
        if anticlockwise {
            arc = arc.reversed();
        }
        self.extend(arc.path_elements(0.1));
    }

    fn bezier_curve_to(&mut self, p1: &Point2D<f32>, p2: &Point2D<f32>, p3: &Point2D<f32>) {
        self.curve_to(p1.convert(), p2.convert(), p3.convert());
    }

    fn close(&mut self) {
        self.close_path();
    }

    fn get_current_point(&mut self) -> Option<Point2D<f32>> {
        self.elements()
            .last()
            .and_then(|last| last.end_point().map(Convert::convert))
    }

    fn line_to(&mut self, point: Point2D<f32>) {
        self.line_to(point.convert());
    }

    fn move_to(&mut self, point: Point2D<f32>) {
        self.move_to(point.convert());
    }

    fn quadratic_curve_to(&mut self, p1: &Point2D<f32>, p2: &Point2D<f32>) {
        self.quad_to(p1.convert(), p2.convert());
    }

    fn svg_arc(
        &mut self,
        radius_x: f32,
        radius_y: f32,
        rotation_angle: f32,
        large_arc: bool,
        sweep: bool,
        end_point: Point2D<f32>,
    ) {
        let sarc = kurbo::SvgArc {
            from: self.get_current_point().unwrap().convert(),
            to: end_point.convert(),
            radii: kurbo::Vec2::new(radius_x as f64, radius_y as f64),
            x_rotation: rotation_angle as f64,
            large_arc,
            sweep,
        };
        let arc = kurbo::Arc::from_svg_arc(&sarc).unwrap();
        self.extend(arc.path_elements(0.1));
    }

    fn finish(&mut self) -> kurbo::BezPath {
        self.clone()
    }
}

impl PathHelpers<VelloBackend> for kurbo::BezPath {
    fn transformed_copy_to_builder(&self, transform: &Transform2D<f32>) -> kurbo::BezPath {
        let mut copy = self.clone();
        copy.apply_affine(transform.convert());
        copy
    }

    fn contains_point(&self, x: f64, y: f64, path_transform: &Transform2D<f32>) -> bool {
        let mut t = self.clone();
        t.apply_affine(path_transform.convert());
        t.contains(kurbo::Point::new(x, y))
    }

    fn copy_to_builder(&self) -> kurbo::BezPath {
        self.clone()
    }
}

impl StrokeOptionsHelpers for kurbo::Stroke {
    fn set_line_width(&mut self, val: f32) {
        self.width = val as f64;
    }

    fn set_miter_limit(&mut self, val: f32) {
        self.miter_limit = val as f64;
    }

    fn set_line_join(&mut self, val: LineJoinStyle) {
        self.join = val.convert()
    }

    fn set_line_cap(&mut self, val: LineCapStyle) {
        self.start_cap = val.convert();
        self.end_cap = val.convert();
    }

    fn set_line_dash(&mut self, items: Vec<f32>) {
        self.dash_pattern = items.iter().map(|x| *x as f64).collect();
    }

    fn set_line_dash_offset(&mut self, offset: f32) {
        self.dash_offset = offset as f64;
    }
}

impl PatternHelpers for peniko::Brush {
    fn is_zero_size_gradient(&self) -> bool {
        match self {
            Self::Gradient(gradient) => gradient.stops.is_empty(), // TODO(vello): more
            Self::Image(_) | Self::Solid(_) => false,
        }
    }

    fn draw_rect(&self, rect: &Rect<f32>) -> Rect<f32> {
        match self {
            Self::Gradient(_gradient) => *rect, // TODO(vello): actual impl
            Self::Solid(_) | Self::Image(_) => *rect,
        }
    }
}

/// A version of the `Into<T>` trait from the standard library that can be used
/// to convert between two types that are not defined in the canvas crate.
pub(crate) trait Convert<T> {
    fn convert(self) -> T;
}

impl Convert<kurbo::Point> for Point2D<f32> {
    fn convert(self) -> kurbo::Point {
        kurbo::Point {
            x: self.x as f64,
            y: self.y as f64,
        }
    }
}

impl Convert<kurbo::Point> for Point2D<f64> {
    fn convert(self) -> kurbo::Point {
        kurbo::Point {
            x: self.x,
            y: self.y,
        }
    }
}

impl Convert<Point2D<f32>> for kurbo::Point {
    fn convert(self) -> Point2D<f32> {
        Point2D::new(self.x, self.y).cast()
    }
}

impl Convert<Transform2D<f32>> for kurbo::Affine {
    fn convert(self) -> Transform2D<f32> {
        // TODO(vello): check?
        Transform2D::from_array(self.as_coeffs()).cast()
    }
}

impl Convert<kurbo::Affine> for Transform2D<f32> {
    fn convert(self) -> kurbo::Affine {
        // TODO(vello): check?
        kurbo::Affine::new(self.cast().to_array())
    }
}

impl Convert<kurbo::Join> for LineJoinStyle {
    fn convert(self) -> kurbo::Join {
        match self {
            LineJoinStyle::Round => kurbo::Join::Round,
            LineJoinStyle::Bevel => kurbo::Join::Bevel,
            LineJoinStyle::Miter => kurbo::Join::Miter,
        }
    }
}

impl Convert<kurbo::Cap> for LineCapStyle {
    fn convert(self) -> kurbo::Cap {
        match self {
            LineCapStyle::Butt => kurbo::Cap::Butt,
            LineCapStyle::Round => kurbo::Cap::Round,
            LineCapStyle::Square => kurbo::Cap::Square,
        }
    }
}

impl Convert<peniko::Color> for AbsoluteColor {
    fn convert(self) -> peniko::Color {
        let srgb = self.into_srgb_legacy();
        peniko::Color::new([
            srgb.components.0,
            srgb.components.1,
            srgb.components.2,
            srgb.alpha,
        ])
    }
}

impl Convert<peniko::BlendMode> for CompositionOrBlending {
    fn convert(self) -> peniko::BlendMode {
        match self {
            CompositionOrBlending::Composition(composition_style) => {
                composition_style.convert().into()
            },
            CompositionOrBlending::Blending(blending_style) => blending_style.convert().into(),
        }
    }
}

impl Convert<peniko::Compose> for CompositionStyle {
    fn convert(self) -> peniko::Compose {
        match self {
            CompositionStyle::SrcIn => peniko::Compose::SrcIn,
            CompositionStyle::SrcOut => peniko::Compose::SrcOut,
            CompositionStyle::SrcOver => peniko::Compose::SrcOver,
            CompositionStyle::SrcAtop => peniko::Compose::SrcAtop,
            CompositionStyle::DestIn => peniko::Compose::DestIn,
            CompositionStyle::DestOut => peniko::Compose::DestOut,
            CompositionStyle::DestOver => peniko::Compose::DestOver,
            CompositionStyle::DestAtop => peniko::Compose::DestAtop,
            CompositionStyle::Copy => peniko::Compose::Copy,
            CompositionStyle::Lighter => peniko::Compose::PlusLighter, // TODO(vello): verify
            CompositionStyle::Xor => peniko::Compose::Xor,
            CompositionStyle::Clear => peniko::Compose::Clear,
        }
    }
}

impl Convert<peniko::Mix> for BlendingStyle {
    fn convert(self) -> peniko::Mix {
        match self {
            BlendingStyle::Multiply => peniko::Mix::Multiply,
            BlendingStyle::Screen => peniko::Mix::Screen,
            BlendingStyle::Overlay => peniko::Mix::Overlay,
            BlendingStyle::Darken => peniko::Mix::Darken,
            BlendingStyle::Lighten => peniko::Mix::Lighten,
            BlendingStyle::ColorDodge => peniko::Mix::ColorDodge,
            BlendingStyle::ColorBurn => peniko::Mix::ColorBurn,
            BlendingStyle::HardLight => peniko::Mix::HardLight,
            BlendingStyle::SoftLight => peniko::Mix::SoftLight,
            BlendingStyle::Difference => peniko::Mix::Difference,
            BlendingStyle::Exclusion => peniko::Mix::Exclusion,
            BlendingStyle::Hue => peniko::Mix::Hue,
            BlendingStyle::Saturation => peniko::Mix::Saturation,
            BlendingStyle::Color => peniko::Mix::Color,
            BlendingStyle::Luminosity => peniko::Mix::Luminosity,
        }
    }
}

impl Convert<peniko::Brush> for FillOrStrokeStyle {
    fn convert(self) -> peniko::Brush {
        use canvas_traits::canvas::FillOrStrokeStyle::*;
        match self {
            Color(absolute_color) => peniko::Brush::Solid(absolute_color.convert()),
            LinearGradient(style) => {
                let start = kurbo::Point::new(style.x0, style.y0);
                let end = kurbo::Point::new(style.x1, style.y1);
                let mut gradient = peniko::Gradient::new_linear(start, end);
                gradient.stops = style.stops.convert();
                peniko::Brush::Gradient(gradient)
            },
            RadialGradient(style) => {
                let center1 = kurbo::Point::new(style.x0, style.y0);
                let center2 = kurbo::Point::new(style.x1, style.y1);
                let mut gradient = peniko::Gradient::new_two_point_radial(
                    center1,
                    style.r0 as f32,
                    center2,
                    style.r1 as f32,
                );
                gradient.stops = style.stops.convert();
                peniko::Brush::Gradient(gradient)
            },
            Surface(surface_style) => peniko::Brush::Image(peniko::Image {
                data: peniko::Blob::from(surface_style.surface_data.into_vec()),
                format: peniko::ImageFormat::Rgba8, // TODO: is this really RGBA?
                // be do not do any coversions for snapshot in script
                width: surface_style.surface_size.width,
                height: surface_style.surface_size.height,
                x_extend: if surface_style.repeat_x {
                    peniko::Extend::Repeat
                } else {
                    peniko::Extend::Pad
                },
                y_extend: if surface_style.repeat_y {
                    peniko::Extend::Repeat
                } else {
                    peniko::Extend::Pad
                },
                quality: peniko::ImageQuality::Low, // TODO: global option
                alpha: 1.0,
            }),
        }
    }
}

impl Convert<peniko::color::DynamicColor> for AbsoluteColor {
    fn convert(self) -> peniko::color::DynamicColor {
        peniko::color::DynamicColor::from_alpha_color(self.convert())
    }
}

impl Convert<peniko::ColorStop> for CanvasGradientStop {
    fn convert(self) -> peniko::ColorStop {
        peniko::ColorStop {
            offset: self.offset as f32,
            color: self.color.convert(),
        }
    }
}

impl Convert<peniko::ColorStops> for Vec<CanvasGradientStop> {
    fn convert(self) -> peniko::ColorStops {
        let mut stops = peniko::ColorStops(self.into_iter().map(|item| item.convert()).collect());
        // https://www.w3.org/html/test/results/2dcontext/annotated-spec/canvas.html#testrefs.2d.gradient.interpolate.overlap
        stops
            .0
            .sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
        stops
    }
}

impl Convert<kurbo::Size> for Size2D<f32> {
    fn convert(self) -> kurbo::Size {
        kurbo::Size::new(self.width as f64, self.height as f64)
    }
}

impl Convert<kurbo::Size> for Size2D<f64> {
    fn convert(self) -> kurbo::Size {
        kurbo::Size::new(self.width, self.height)
    }
}

impl Convert<kurbo::Rect> for Rect<f32> {
    fn convert(self) -> kurbo::Rect {
        kurbo::Rect::from_center_size(self.center().convert(), self.size.convert())
    }
}

impl Convert<kurbo::Rect> for Rect<f64> {
    fn convert(self) -> kurbo::Rect {
        kurbo::Rect::from_center_size(self.center().convert(), self.size.convert())
    }
}
