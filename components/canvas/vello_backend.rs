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
    CompositionOptions, FillOrStrokeStyle, FillRule, LineOptions, Path, ShadowOptions,
};
use compositing_traits::SerializableImageData;
use euclid::default::{Point2D, Rect, Size2D, Transform2D};
use fonts::{ByteIndex, FontIdentifier, FontTemplateRefMethods as _};
use ipc_channel::ipc::IpcSharedMemory;
use kurbo::Shape as _;
use pixels::{Snapshot, SnapshotAlphaMode, SnapshotPixelFormat};
use range::Range;
use vello::wgpu::{
    BackendOptions, Backends, Buffer, BufferDescriptor, BufferUsages, COPY_BYTES_PER_ROW_ALIGNMENT,
    CommandEncoderDescriptor, Device, Extent3d, Instance, InstanceDescriptor, InstanceFlags,
    MapMode, Queue, TexelCopyBufferInfo, TexelCopyBufferLayout, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
};
use vello::{kurbo, peniko};
use webrender_api::{ImageDescriptor, ImageDescriptorFlags};

use crate::backend::{Convert as _, GenericDrawTarget};
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
    clips: Vec<Path>,
    downloader: GPUTextureDownloader,
    render_texture: Texture,
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

    fn render(&self) {
        self.renderer
            .borrow_mut()
            .render_to_texture(
                &self.device,
                &self.queue,
                &self.scene,
                &self
                    .render_texture
                    .create_view(&TextureViewDescriptor::default()),
                &vello::RenderParams {
                    base_color: peniko::color::AlphaColor::TRANSPARENT,
                    width: self.size.width,
                    height: self.size.height,
                    antialiasing_method: vello::AaConfig::Area,
                },
            )
            .unwrap();
    }

    fn ignore_clips(&mut self, f: impl FnOnce(&mut Self)) {
        // pop all clip layers
        for _ in &self.clips {
            self.scene.pop_layer();
        }
        f(self);
        // push all clip layers back
        for path in &self.clips {
            self.scene
                .push_layer(peniko::Mix::Clip, 1.0, kurbo::Affine::IDENTITY, &path.0);
        }
    }

    fn is_viewport_cleared(&mut self, rect: &Rect<f32>, transform: Transform2D<f32>) -> bool {
        let transformed_rect = transform.outer_transformed_rect(rect);
        if transformed_rect.is_empty() {
            return false;
        }
        let viewport: Rect<f64> = Rect::from_size(self.get_size().cast());
        let Some(clip) = self.clips.iter().try_fold(viewport, |acc, e| {
            acc.intersection(&e.0.bounding_box().into())
        }) else {
            // clip makes no visible side effects
            return false;
        };
        transformed_rect.cast().contains_rect(&viewport) && // whole viewport is cleared
            clip.contains_rect(&viewport) // viewport is not clipped
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
        let renderer = vello::Renderer::new(
            &device,
            vello::RendererOptions {
                use_cpu: false,
                num_init_threads: NonZeroUsize::new(1),
                antialiasing_support: vello::AaSupport::area_only(),
                pipeline_cache: None,
            },
        )
        .unwrap();
        let scene = vello::Scene::new();
        device.on_uncaptured_error(Box::new(|error| {
            log::error!("VELLO WGPU ERROR: {error}");
        }));
        let texture = device.create_texture(&create_texture_descriptor(
            size,
            TextureUsages::COPY_SRC | TextureUsages::STORAGE_BINDING,
        ));
        let downloader = GPUTextureDownloader::new(&device, &queue, size);
        Self {
            device,
            queue,
            renderer: Rc::new(RefCell::new(renderer)),
            scene,
            size,
            clips: Vec::new(),
            render_texture: texture,
            downloader,
        }
    }

    fn clear_rect(&mut self, rect: &Rect<f32>, transform: Transform2D<f32>) {
        // vello scene only ever grows,
        // so we need to use every opportunity to shrink it
        if self.is_viewport_cleared(rect, transform) {
            self.scene.reset();
            self.clips.clear(); // no clips are affecting rendering
            return;
        }
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

        self.ignore_clips(|self_| {
            self_
                .scene
                .push_layer(peniko::Compose::Copy, 1.0, kurbo::Affine::IDENTITY, &rect);

            self_.scene.fill(
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

            self_.scene.pop_layer();
        });
    }

    fn create_similar_draw_target(&self, size: &Size2D<i32>) -> Self {
        Self {
            device: self.device.clone(),
            queue: self.queue.clone(),
            renderer: self.renderer.clone(),
            scene: vello::Scene::new(),
            size: size.cast(),
            clips: Vec::new(),
            downloader: if size.cast() == self.size {
                self.downloader.clone()
            } else {
                GPUTextureDownloader::new(&self.device, &self.queue, size.cast())
            },
            render_texture: self.device.create_texture(&create_texture_descriptor(
                size.cast(),
                TextureUsages::COPY_SRC | TextureUsages::STORAGE_BINDING,
            )),
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
        fill_rule: FillRule,
        style: FillOrStrokeStyle,
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    ) {
        self.with_draw_options(&composition_options, |self_| {
            self_.scene.fill(
                fill_rule.convert(),
                transform.cast().into(),
                &convert_to_brush(style, composition_options),
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
        let pattern = convert_to_brush(style, composition_options);
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
        let pattern = convert_to_brush(style, composition_options);
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
        if self.clips.pop().is_some() {
            self.scene.pop_layer();
        }
    }

    fn push_clip(&mut self, path: &Path, _fill_rule: FillRule, transform: Transform2D<f32>) {
        self.scene
            .push_layer(peniko::Mix::Clip, 1.0, transform.cast().into(), &path.0);
        let mut path = path.clone();
        path.transform(transform.cast());
        self.clips.push(path);
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
        self.push_clip(&path, FillRule::Nonzero, Transform2D::identity());
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
                &convert_to_brush(style, composition_options),
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
                &convert_to_brush(style, composition_options),
                None,
                &rect,
            );
        })
    }

    fn image_descriptor_and_serializable_data(
        &mut self,
    ) -> (ImageDescriptor, SerializableImageData) {
        let size = self.size;
        self.render();
        self.downloader
            .download_texture(&self.render_texture, |stride, data| {
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
        self.render();
        self.downloader
            .download_texture(&self.render_texture, |padded_byte_width, data| {
                let data = data
                    .map(|data| {
                        let mut result_unpadded =
                            Vec::<u8>::with_capacity(size.area() as usize * 4);
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

fn convert_to_brush(
    style: FillOrStrokeStyle,
    composition_options: CompositionOptions,
) -> peniko::Brush {
    let brush: peniko::Brush = style.convert();
    brush.multiply_alpha(composition_options.alpha as f32)
}

fn extend3d(size: Size2D<u32>) -> Extent3d {
    Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: 1,
    }
}

fn create_texture_descriptor(
    size: Size2D<u32>,
    usage: TextureUsages,
) -> TextureDescriptor<'static> {
    TextureDescriptor {
        label: None,
        size: extend3d(size),
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage,
        view_formats: &[],
    }
}

#[derive(Clone, Debug)]
struct GPUTextureDownloader {
    device: Device,
    queue: Queue,
    buffer: Buffer,
    padded_byte_width: u32,
    size: Size2D<u32>,
}

impl GPUTextureDownloader {
    fn new(device: &Device, queue: &Queue, size: Size2D<u32>) -> Self {
        let padded_byte_width = (size.width * 4).next_multiple_of(COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer_size = padded_byte_width as u64 * size.height as u64;
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("val"),
            size: buffer_size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            device: device.clone(),
            queue: queue.clone(),
            buffer,
            padded_byte_width,
            size,
        }
    }

    fn download_texture<R>(&self, texture: &Texture, f: impl FnOnce(u32, Option<&[u8]>) -> R) -> R {
        let size = extend3d(self.size);
        assert_eq!(texture.size(), size);
        // TODO(perf): do a render pass that will multiply with alpha on GPU
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Copy out buffer"),
            });
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            TexelCopyBufferInfo {
                buffer: &self.buffer,
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.padded_byte_width),
                    rows_per_image: None,
                },
            },
            size,
        );
        self.queue.submit([encoder.finish()]);
        let result = {
            let buf_slice = self.buffer.slice(..);
            let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
            buf_slice.map_async(MapMode::Read, move |v| sender.send(v).unwrap());
            if let Err(error) =
                vello::util::block_on_wgpu(&self.device, receiver.receive()).unwrap()
            {
                log::warn!("VELLO WGPU MAP ASYNC ERROR {error}");
                return f(self.padded_byte_width, None);
            }
            let data = buf_slice.get_mapped_range();
            f(self.padded_byte_width, Some(&data))
        };
        self.buffer.unmap();
        result
    }
}
