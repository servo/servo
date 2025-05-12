/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Vello implementation of 2D canvas backend.
//!
//! Vello only encodes commands for GPU, then runs rendering when
//! image is explicitly requested. This requires to copy image
//! from texture to buffer, then download buffer to CPU
//! (where we also need to un pad it).
//!
//! All Vello images are in no alpha premultiplied RGBA8 pixel format.

use std::cell::RefCell;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::rc::Rc;

use canvas_traits::canvas::{
    BlendingStyle, CanvasGradientStop, CompositionOptions, CompositionOrBlending, CompositionStyle,
    FillOrStrokeStyle, LineCapStyle, LineJoinStyle, LineOptions, Path, ShadowOptions,
};
use compositing_traits::SerializableImageData;
use euclid::default::{Point2D, Rect, Size2D, Transform2D};
use fonts::{ByteIndex, FontIdentifier, FontTemplateRefMethods as _};
use ipc_channel::ipc::IpcSharedMemory;
use pixels::{Snapshot, SnapshotAlphaMode, SnapshotPixelFormat};
use range::Range;
use style::color::AbsoluteColor;
use vello::wgpu::{
    BackendOptions, Backends, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Device,
    Extent3d, Instance, InstanceDescriptor, InstanceFlags, MapMode, Queue, TexelCopyBufferInfo,
    TexelCopyBufferLayout, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureViewDescriptor,
};
use vello::{kurbo, peniko};
use webrender_api::{ImageDescriptor, ImageDescriptorFlags};

use crate::backend::GenericDrawTarget;
use crate::canvas_data::{Filter, TextRun};

thread_local! {
    /// The shared font cache used by all canvases that render on a thread. It would be nicer
    /// to have a global cache, but it looks like font-kit uses a per-thread FreeType, so
    /// in order to ensure that fonts are particular to a thread we have to make our own
    /// cache thread local as well.
    static SHARED_FONT_CACHE: RefCell<HashMap<FontIdentifier, peniko::Font>> = RefCell::default();
}

pub(crate) struct VelloDrawTarget {
    device: Device,
    queue: Queue,
    renderer: Rc<RefCell<vello::Renderer>>,
    scene: vello::Scene,
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

impl VelloDrawTarget {
    fn with_draw_options<F: FnOnce(&mut Self)>(&mut self, draw_options: &CompositionOptions, f: F) {
        self.scene.push_layer(
            draw_options.composition_operation.convert(),
            1.0,
            kurbo::Affine::IDENTITY,
            &kurbo::Rect::ZERO.with_size(self.size.cast()),
        );
        f(self);
        self.scene.pop_layer();
    }

    fn render_and_download<F, R>(&self, f: F) -> R
    where
        F: FnOnce(u32, Option<&[u8]>) -> R,
    {
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
        // TODO(perf): do a render pass that will multiply with alpha on GPU
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
        let result = {
            let buf_slice = buffer.slice(..);
            let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
            buf_slice.map_async(MapMode::Read, move |v| sender.send(v).unwrap());
            if let Err(error) =
                vello::util::block_on_wgpu(&self.device, receiver.receive()).unwrap()
            {
                log::warn!("VELLO WGPU MAP ASYNC ERROR {error}");
                return f(padded_byte_width, None);
            }
            let data = buf_slice.get_mapped_range();
            f(padded_byte_width, Some(&data))
        };
        buffer.unmap();
        result
    }
}

impl GenericDrawTarget for VelloDrawTarget {
    type SourceSurface = Vec<u8>; // TODO: this should be texture

    fn new(size: Size2D<u32>) -> Self {
        // TODO: we should read prefs instead of env

        // we forbid GL because it clashes with servo's GL usage
        let backends = Backends::from_env().unwrap_or_default() - Backends::GL;
        let flags = InstanceFlags::from_build_config().with_env();
        let backend_options = BackendOptions::from_env_or_default();
        let instance = Instance::new(&InstanceDescriptor {
            backends,
            flags,
            backend_options,
        });
        let mut context = vello::util::RenderContext {
            instance,
            devices: Vec::new(),
        };
        let device_id = pollster::block_on(context.device(None)).unwrap();
        let device_handle = &mut context.devices[device_id];
        let device = device_handle.device.clone();
        let queue = device_handle.queue.clone();
        let renderer = vello::Renderer::new(&device, options()).unwrap();
        let scene = vello::Scene::new();
        device.on_uncaptured_error(Box::new(|error| {
            log::error!("VELLO WGPU ERROR: {error}");
        }));
        Self {
            device,
            queue,
            renderer: Rc::new(RefCell::new(renderer)),
            scene,
            size,
        }
    }

    fn clear_rect(&mut self, rect: &Rect<f32>, transform: Transform2D<f32>) {
        let rect: kurbo::Rect = rect.cast().into();
        let transform = transform.cast().into();
        self.scene
            .push_layer(peniko::Compose::Clear, 0.0, transform, &rect);
        self.scene.fill(
            peniko::Fill::NonZero,
            transform,
            peniko::BrushRef::Solid(peniko::color::AlphaColor::TRANSPARENT),
            None,
            &rect,
        );
        self.scene.pop_layer();
    }

    fn copy_surface(&mut self, surface: Vec<u8>, source: Rect<i32>, destination: Point2D<i32>) {
        let destination: kurbo::Point = destination.cast::<f64>().into();
        let rect = kurbo::Rect::from_origin_size(destination, source.size.cast());

        // TODO: ignore clip from prev layers
        // this will require creating a stacks of applicable clips
        // that will be popped and reinserted after
        // or we could impl this in vello directly

        // then there is also this nasty vello bug where clipping does not work correctly:
        // https://xi.zulipchat.com/#narrow/channel/197075-vello/topic/Servo.202D.20canvas.20backend/near/525153593

        self.scene
            .push_layer(peniko::Compose::Copy, 1.0, kurbo::Affine::IDENTITY, &rect);

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
            Some(kurbo::Affine::translate(destination.to_vec2())),
            &rect,
        );

        self.scene.pop_layer();
    }

    fn create_similar_draw_target(&self, size: &Size2D<i32>) -> Self {
        Self {
            device: self.device.clone(),
            queue: self.queue.clone(),
            renderer: self.renderer.clone(),
            scene: vello::Scene::new(),
            size: size.cast(),
        }
    }

    fn draw_surface(
        &mut self,
        surface: Vec<u8>,
        dest: Rect<f64>,
        source: Rect<f64>,
        filter: Filter,
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    ) {
        let scale_up = dest.size.width > source.size.width || dest.size.height > source.size.height;
        let shape: kurbo::Rect = dest.into();
        self.with_draw_options(&composition_options, move |self_| {
            self_.scene.fill(
                peniko::Fill::NonZero,
                transform.cast().into(),
                &peniko::Image {
                    data: peniko::Blob::from(surface),
                    format: peniko::ImageFormat::Rgba8,
                    width: source.size.width as u32,
                    height: source.size.height as u32,
                    x_extend: peniko::Extend::Pad,
                    y_extend: peniko::Extend::Pad,
                    // we should only do bicubic when scaling up
                    quality: if scale_up {
                        filter.convert()
                    } else {
                        peniko::ImageQuality::Low
                    },
                    alpha: composition_options.alpha as f32,
                },
                Some(
                    kurbo::Affine::translate((dest.origin.x, dest.origin.y)).pre_scale_non_uniform(
                        dest.size.width / source.size.width,
                        dest.size.height / source.size.height,
                    ),
                ),
                &shape,
            )
        })
    }

    fn draw_surface_with_shadow(
        &self,
        _surface: Vec<u8>,
        _dest: &Point2D<f32>,
        _shadow_options: ShadowOptions,
        _composition_options: CompositionOptions,
    ) {
        log::warn!("no support for drawing shadows");
        /*
        We will need to do some changes to support drawing shadows with vello, as current abstraction is made for azure.
        In vello we do not need new draw target (we will use layers) and we need to pass whole rect.
        offsets will be applied to rect directly. shadow blur will be passed directly to let backend do transforms.
        */
        //self_.scene.draw_blurred_rounded_rect(self_.transform, rect, color, 0.0, sigma);
    }

    fn fill(
        &mut self,
        path: &Path,
        style: FillOrStrokeStyle,
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    ) {
        self.with_draw_options(&composition_options, |self_| {
            self_.scene.fill(
                peniko::Fill::NonZero,
                transform.cast().into(),
                &style
                    .convert()
                    .multiply_alpha(composition_options.alpha as f32),
                None,
                &path.0,
            );
        })
    }

    fn fill_text(
        &mut self,
        text_runs: Vec<TextRun>,
        start: Point2D<f32>,
        style: FillOrStrokeStyle,
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    ) {
        let pattern = style
            .convert()
            .multiply_alpha(composition_options.alpha as f32);
        let transform = transform.cast().into();
        self.with_draw_options(&composition_options, |self_| {
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

                    self_
                        .scene
                        .draw_glyphs(font)
                        .transform(transform)
                        .brush(&pattern)
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
        })
    }

    fn fill_rect(
        &mut self,
        rect: &Rect<f32>,
        style: FillOrStrokeStyle,
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    ) {
        let pattern = style
            .convert()
            .multiply_alpha(composition_options.alpha as f32);
        let transform = transform.cast().into();
        let rect: kurbo::Rect = rect.cast().into();
        self.with_draw_options(&composition_options, |self_| {
            self_
                .scene
                .fill(peniko::Fill::NonZero, transform, &pattern, None, &rect);
        })
    }

    fn get_size(&self) -> Size2D<i32> {
        self.size.cast()
    }

    fn pop_clip(&mut self) {
        self.scene.pop_layer();
    }

    fn push_clip(&mut self, path: &Path, transform: Transform2D<f32>) {
        self.scene
            .push_layer(peniko::Mix::Clip, 1.0, transform.cast().into(), &path.0);
    }

    fn push_clip_rect(&mut self, rect: &Rect<i32>) {
        let mut path = Path::new();
        let rect = rect.cast();
        path.rect(
            rect.origin.x,
            rect.origin.y,
            rect.size.width,
            rect.size.height,
        );
        self.push_clip(&path, Transform2D::identity());
    }

    fn stroke(
        &mut self,
        path: &Path,
        style: FillOrStrokeStyle,
        line_options: LineOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    ) {
        self.with_draw_options(&composition_options, |self_| {
            self_.scene.stroke(
                &line_options.convert(),
                transform.cast().into(),
                &style
                    .convert()
                    .multiply_alpha(composition_options.alpha as f32),
                None,
                &path.0,
            );
        })
    }

    fn stroke_rect(
        &mut self,
        rect: &Rect<f32>,
        style: FillOrStrokeStyle,
        line_options: LineOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    ) {
        let rect: kurbo::Rect = rect.cast().into();
        self.with_draw_options(&composition_options, |self_| {
            self_.scene.stroke(
                &line_options.convert(),
                transform.cast().into(),
                &style
                    .convert()
                    .multiply_alpha(composition_options.alpha as f32),
                None,
                &rect,
            );
        })
    }

    fn image_descriptor_and_serializable_data(
        &mut self,
    ) -> (ImageDescriptor, SerializableImageData) {
        let size = self.size;
        self.render_and_download(|stride, data| {
            let image_desc = ImageDescriptor {
                format: webrender_api::ImageFormat::RGBA8,
                size: size.cast().cast_unit(),
                stride: data.map(|_| stride as i32),
                offset: 0,
                flags: ImageDescriptorFlags::empty(),
            };
            let data = SerializableImageData::Raw(if let Some(data) = data {
                let mut data = IpcSharedMemory::from_bytes(data);
                #[allow(unsafe_code)]
                unsafe {
                    pixels::generic_transform_inplace::<1, false, false>(data.deref_mut());
                };
                data
            } else {
                IpcSharedMemory::from_byte(0, size.area() as usize * 4)
            });
            (image_desc, data)
        })
    }

    fn snapshot(&mut self) -> pixels::Snapshot {
        let size = self.size;
        self.render_and_download(|padded_byte_width, data| {
            let data = data
                .map(|data| {
                    let mut result_unpadded = Vec::<u8>::with_capacity(size.area() as usize * 4);
                    for row in 0..self.size.height {
                        let start = (row * padded_byte_width).try_into().unwrap();
                        result_unpadded
                            .extend(&data[start..start + (self.size.width * 4) as usize]);
                    }
                    result_unpadded
                })
                .unwrap_or_else(|| vec![0; size.area() as usize * 4]);
            Snapshot::from_vec(
                size,
                SnapshotPixelFormat::RGBA,
                SnapshotAlphaMode::Transparent {
                    premultiplied: false,
                },
                data,
            )
        })
    }

    fn surface(&mut self) -> Vec<u8> {
        self.snapshot().to_vec(None, None).0
    }

    fn create_source_surface_from_data(&self, data: Snapshot) -> Option<Vec<u8>> {
        let (data, _, _) = data.to_vec(
            Some(SnapshotAlphaMode::Transparent {
                premultiplied: false,
            }),
            Some(SnapshotPixelFormat::RGBA),
        );
        Some(data)
    }
}

/// A version of the `Into<T>` trait from the standard library that can be used
/// to convert between two types that are not defined in the canvas crate.
pub(crate) trait Convert<T> {
    fn convert(self) -> T;
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
            CompositionStyle::SourceIn => peniko::Compose::SrcIn,
            CompositionStyle::SourceOut => peniko::Compose::SrcOut,
            CompositionStyle::SourceOver => peniko::Compose::SrcOver,
            CompositionStyle::SourceAtop => peniko::Compose::SrcAtop,
            CompositionStyle::DestinationIn => peniko::Compose::DestIn,
            CompositionStyle::DestinationOut => peniko::Compose::DestOut,
            CompositionStyle::DestinationOver => peniko::Compose::DestOver,
            CompositionStyle::DestinationAtop => peniko::Compose::DestAtop,
            CompositionStyle::Copy => peniko::Compose::Copy,
            CompositionStyle::Lighter => peniko::Compose::Plus,
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

impl Convert<kurbo::Stroke> for LineOptions {
    fn convert(self) -> kurbo::Stroke {
        let LineOptions {
            width,
            cap_style,
            join_style,
            miter_limit,
            dash,
            dash_offset,
        } = self;
        kurbo::Stroke {
            width,
            join: join_style.convert(),
            miter_limit,
            start_cap: cap_style.convert(),
            end_cap: cap_style.convert(),
            dash_pattern: dash.iter().map(|x| *x as f64).collect(),
            dash_offset,
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
            Surface(surface_style) => {
                let data = surface_style
                    .surface_data
                    .to_owned()
                    .to_vec(
                        Some(SnapshotAlphaMode::Transparent {
                            premultiplied: false,
                        }),
                        Some(SnapshotPixelFormat::RGBA),
                    )
                    .0;
                peniko::Brush::Image(peniko::Image {
                    data: peniko::Blob::from(data),
                    format: peniko::ImageFormat::Rgba8,
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
                    quality: peniko::ImageQuality::Low,
                    alpha: 1.0,
                })
            },
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

impl Convert<peniko::ImageQuality> for Filter {
    fn convert(self) -> peniko::ImageQuality {
        match self {
            Filter::Bilinear => peniko::ImageQuality::Medium,
            Filter::Nearest => peniko::ImageQuality::Low,
        }
    }
}
