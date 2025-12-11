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
    CompositionOptions, CompositionOrBlending, CompositionStyle, FillOrStrokeStyle, FillRule,
    LineOptions, Path, ShadowOptions, TextRun,
};
use compositing_traits::SerializableImageData;
use euclid::default::{Point2D, Rect, Size2D, Transform2D};
use fonts::FontIdentifier;
use ipc_channel::ipc::GenericSharedMemory;
use kurbo::Shape as _;
use pixels::{Snapshot, SnapshotAlphaMode, SnapshotPixelFormat};
use vello::wgpu::{
    BackendOptions, Backends, Buffer, BufferDescriptor, BufferUsages, COPY_BYTES_PER_ROW_ALIGNMENT,
    CommandEncoderDescriptor, Device, Extent3d, Instance, InstanceDescriptor, InstanceFlags,
    MapMode, MemoryBudgetThresholds, Origin3d, Queue, TexelCopyBufferInfo, TexelCopyBufferLayout,
    TexelCopyTextureInfoBase, Texture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureView, TextureViewDescriptor,
};
use vello::{kurbo, peniko};
use webrender_api::{ImageDescriptor, ImageDescriptorFlags};

use crate::backend::{Convert as _, GenericDrawTarget};
use crate::canvas_data::Filter;

thread_local! {
    /// The shared font cache used by all canvases that render on a thread.
    static SHARED_FONT_CACHE: RefCell<HashMap<FontIdentifier, peniko::FontData>> = RefCell::default();
}

pub(crate) struct VelloDrawTarget {
    device: Device,
    queue: Queue,
    renderer: Rc<RefCell<vello::Renderer>>,
    scene: vello::Scene,
    size: Size2D<u32>,
    clips: Vec<Path>,
    state: State,
    render_texture: Texture,
    render_texture_view: TextureView,
    render_image: peniko::ImageBrush,
    padded_byte_width: u32,
    rendered_buffer: Buffer,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
enum State {
    /// Scene is drawing. It will be consumed when rendered.
    Drawing,
    /// Scene is already rendered
    /// Before next draw we need to put current rendering
    /// in the background by calling [`VelloDrawTarget::ensure_drawing`].
    RenderedToTexture,
    RenderedToBuffer,
}

impl VelloDrawTarget {
    fn new_with_renderer(
        device: Device,
        queue: Queue,
        renderer: Rc<RefCell<vello::Renderer>>,
        size: Size2D<u32>,
    ) -> Self {
        let render_texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: extend3d(size),
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::COPY_SRC | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });
        let render_texture_view = render_texture.create_view(&TextureViewDescriptor::default());
        let render_image = peniko::ImageBrush {
            image: peniko::ImageData {
                data: vec![].into(),
                format: peniko::ImageFormat::Rgba8,
                width: size.width,
                height: size.height,
                alpha_type: peniko::ImageAlphaType::Alpha,
            },
            sampler: peniko::ImageSampler::default(),
        };
        renderer.borrow_mut().override_image(
            &render_image.image,
            Some(TexelCopyTextureInfoBase {
                texture: render_texture.clone(),
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: vello::wgpu::TextureAspect::All,
            }),
        );
        let padded_byte_width = (size.width * 4).next_multiple_of(COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer_size = padded_byte_width as u64 * size.height as u64;
        let rendered_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("val"),
            size: buffer_size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            device,
            queue,
            renderer,
            scene: vello::Scene::new(),
            size,
            clips: Vec::new(),
            state: State::RenderedToBuffer,
            render_texture,
            render_texture_view,
            render_image,
            padded_byte_width,
            rendered_buffer,
        }
    }

    fn with_composition<F: FnOnce(&mut Self)>(
        &mut self,
        composition_operation: CompositionOrBlending,
        f: F,
    ) {
        // Fast-path for default and most common composition operation
        if composition_operation == CompositionOrBlending::Composition(CompositionStyle::SourceOver)
        {
            f(self);
            return;
        }
        self.scene.push_layer(
            composition_operation.convert(),
            1.0,
            kurbo::Affine::IDENTITY,
            &kurbo::Rect::ZERO.with_size(self.size.cast()),
        );
        f(self);
        self.scene.pop_layer();
    }

    fn ignore_clips(&mut self, f: impl FnOnce(&mut Self)) {
        // pop all clip layers
        for _ in &self.clips {
            self.scene.pop_layer();
        }
        f(self);
        // push all clip layers back
        for path in &self.clips {
            self.scene.push_clip_layer(kurbo::Affine::IDENTITY, &path.0);
        }
    }

    fn is_viewport_cleared(&mut self, rect: &Rect<f32>, transform: Transform2D<f64>) -> bool {
        let transformed_rect = transform.outer_transformed_rect(&rect.cast());
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

    fn ensure_drawing(&mut self) {
        match self.state {
            State::Drawing => {},
            State::RenderedToBuffer | State::RenderedToTexture => {
                self.ignore_clips(|self_| {
                    self_
                        .scene
                        .draw_image(&self_.render_image, kurbo::Affine::IDENTITY);
                });
                self.state = State::Drawing;
            },
        }
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
            memory_budget_thresholds: MemoryBudgetThresholds::default(),
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
        device.on_uncaptured_error(Box::new(|error| {
            log::error!("VELLO WGPU ERROR: {error}");
        }));
        Self::new_with_renderer(device, queue, Rc::new(RefCell::new(renderer)), size)
    }

    fn clear_rect(&mut self, rect: &Rect<f32>, transform: Transform2D<f64>) {
        // vello scene only ever grows,
        // so we use every opportunity to shrink it
        if self.is_viewport_cleared(rect, transform) {
            self.scene.reset();
            self.clips.clear(); // no clips are affecting rendering
            self.state = State::Drawing;
            return;
        }
        self.ensure_drawing();
        let rect: kurbo::Rect = rect.cast().into();
        let transform = transform.into();
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
        self.ensure_drawing();
        let destination: kurbo::Point = destination.cast::<f64>().into();
        let rect = kurbo::Rect::from_origin_size(destination, source.size.cast());

        self.ignore_clips(|self_| {
            self_
                .scene
                .push_layer(peniko::Compose::Copy, 1.0, kurbo::Affine::IDENTITY, &rect);

            self_.scene.fill(
                peniko::Fill::NonZero,
                kurbo::Affine::IDENTITY,
                &peniko::ImageBrush {
                    image: peniko::ImageData {
                        data: peniko::Blob::from(surface),
                        format: peniko::ImageFormat::Rgba8,
                        width: source.size.width as u32,
                        height: source.size.height as u32,
                        alpha_type: peniko::ImageAlphaType::Alpha,
                    },
                    sampler: peniko::ImageSampler {
                        x_extend: peniko::Extend::Pad,
                        y_extend: peniko::Extend::Pad,
                        quality: peniko::ImageQuality::Low,
                        alpha: 1.0,
                    },
                },
                Some(kurbo::Affine::translate(destination.to_vec2())),
                &rect,
            );

            self_.scene.pop_layer();
        });
    }

    fn create_similar_draw_target(&self, size: &Size2D<i32>) -> Self {
        Self::new_with_renderer(
            self.device.clone(),
            self.queue.clone(),
            self.renderer.clone(),
            size.cast(),
        )
    }

    fn draw_surface(
        &mut self,
        surface: Vec<u8>,
        dest: Rect<f64>,
        source: Rect<f64>,
        filter: Filter,
        composition_options: CompositionOptions,
        transform: Transform2D<f64>,
    ) {
        self.ensure_drawing();
        let scale_up = dest.size.width > source.size.width || dest.size.height > source.size.height;
        let shape: kurbo::Rect = dest.into();
        self.with_composition(composition_options.composition_operation, move |self_| {
            self_.scene.fill(
                peniko::Fill::NonZero,
                transform.cast().into(),
                &peniko::ImageBrush {
                    image: peniko::ImageData {
                        data: peniko::Blob::from(surface),
                        format: peniko::ImageFormat::Rgba8,
                        width: source.size.width as u32,
                        height: source.size.height as u32,
                        alpha_type: peniko::ImageAlphaType::Alpha,
                    },
                    sampler: peniko::ImageSampler {
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
        // self_.scene.draw_blurred_rounded_rect(self_.transform, rect, color, 0.0, sigma);
    }

    fn fill(
        &mut self,
        path: &Path,
        fill_rule: FillRule,
        style: FillOrStrokeStyle,
        composition_options: CompositionOptions,
        transform: Transform2D<f64>,
    ) {
        self.ensure_drawing();
        self.with_composition(composition_options.composition_operation, |self_| {
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
        style: FillOrStrokeStyle,
        composition_options: CompositionOptions,
        transform: Transform2D<f64>,
    ) {
        self.ensure_drawing();
        let pattern = convert_to_brush(style, composition_options);
        let transform = transform.cast().into();
        self.with_composition(composition_options.composition_operation, |self_| {
            for text_run in text_runs.iter() {
                SHARED_FONT_CACHE.with(|font_cache| {
                    let identifier = &text_run.font.identifier;
                    if !font_cache.borrow().contains_key(identifier) {
                        let Some(font_data_and_index) = text_run.font.font_data_and_index() else {
                            return;
                        };
                        let font = font_data_and_index.convert();
                        font_cache.borrow_mut().insert(identifier.clone(), font);
                    }

                    let font_cache = font_cache.borrow();
                    let Some(font) = font_cache.get(identifier) else {
                        return;
                    };

                    self_
                        .scene
                        .draw_glyphs(font)
                        .transform(transform)
                        .brush(&pattern)
                        .font_size(text_run.pt_size)
                        .draw(
                            peniko::Fill::NonZero,
                            text_run
                                .glyphs_and_positions
                                .iter()
                                .map(|glyph_and_position| vello::Glyph {
                                    id: glyph_and_position.id,
                                    x: glyph_and_position.point.x,
                                    y: glyph_and_position.point.y,
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
        transform: Transform2D<f64>,
    ) {
        self.ensure_drawing();
        let pattern = convert_to_brush(style, composition_options);
        let transform = transform.cast().into();
        let rect: kurbo::Rect = rect.cast().into();
        self.with_composition(composition_options.composition_operation, |self_| {
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

    fn push_clip(&mut self, path: &Path, _fill_rule: FillRule, transform: Transform2D<f64>) {
        self.scene.push_clip_layer(transform.cast().into(), &path.0);
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
        transform: Transform2D<f64>,
    ) {
        self.ensure_drawing();
        self.with_composition(composition_options.composition_operation, |self_| {
            self_.scene.stroke(
                &line_options.convert(),
                transform.cast().into(),
                &convert_to_brush(style, composition_options),
                None,
                &path.0,
            );
        })
    }

    fn stroke_text(
        &mut self,
        text_runs: Vec<TextRun>,
        style: FillOrStrokeStyle,
        line_options: LineOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f64>,
    ) {
        self.ensure_drawing();
        let pattern = convert_to_brush(style, composition_options);
        let transform = transform.cast().into();
        let line_options: kurbo::Stroke = line_options.convert();
        self.with_composition(composition_options.composition_operation, |self_| {
            for text_run in text_runs.iter() {
                SHARED_FONT_CACHE.with(|font_cache| {
                    let identifier = &text_run.font.identifier;
                    if !font_cache.borrow().contains_key(identifier) {
                        let Some(font_data_and_index) = text_run.font.font_data_and_index() else {
                            return;
                        };
                        let font = font_data_and_index.convert();
                        font_cache.borrow_mut().insert(identifier.clone(), font);
                    }

                    let font_cache = font_cache.borrow();
                    let Some(font) = font_cache.get(identifier) else {
                        return;
                    };

                    self_
                        .scene
                        .draw_glyphs(font)
                        .transform(transform)
                        .brush(&pattern)
                        .font_size(text_run.pt_size)
                        .draw(
                            &line_options,
                            text_run
                                .glyphs_and_positions
                                .iter()
                                .map(|glyph_and_position| vello::Glyph {
                                    id: glyph_and_position.id,
                                    x: glyph_and_position.point.x,
                                    y: glyph_and_position.point.y,
                                }),
                        );
                });
            }
        })
    }

    fn stroke_rect(
        &mut self,
        rect: &Rect<f32>,
        style: FillOrStrokeStyle,
        line_options: LineOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f64>,
    ) {
        self.ensure_drawing();
        let rect: kurbo::Rect = rect.cast().into();
        self.with_composition(composition_options.composition_operation, |self_| {
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
        let stride = self.padded_byte_width;
        self.map_read(|data| {
            let image_desc = ImageDescriptor {
                format: webrender_api::ImageFormat::RGBA8,
                size: size.cast().cast_unit(),
                stride: data.map(|_| stride as i32),
                offset: 0,
                flags: ImageDescriptorFlags::empty(),
            };
            let data = SerializableImageData::Raw(if let Some(data) = data {
                let mut data = GenericSharedMemory::from_bytes(data);
                #[expect(unsafe_code)]
                unsafe {
                    pixels::generic_transform_inplace::<1, false, false>(data.deref_mut());
                };
                data
            } else {
                GenericSharedMemory::from_byte(0, size.area() as usize * 4)
            });
            (image_desc, data)
        })
    }

    fn snapshot(&mut self) -> pixels::Snapshot {
        let size = self.size;
        let padded_byte_width = self.padded_byte_width;
        self.map_read(|data| {
            let data = data
                .map(|data| {
                    let mut result_unpadded = Vec::<u8>::with_capacity(size.area() as usize * 4);
                    for row in 0..size.height {
                        let start = (row * padded_byte_width).try_into().unwrap();
                        result_unpadded.extend(&data[start..start + (size.width * 4) as usize]);
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

impl Drop for VelloDrawTarget {
    fn drop(&mut self) {
        self.renderer
            .borrow_mut()
            .override_image(&self.render_image.image, None);
    }
}

fn convert_to_brush(
    style: FillOrStrokeStyle,
    composition_options: CompositionOptions,
) -> peniko::Brush {
    let brush: peniko::Brush = style.convert();
    brush.multiply_alpha(composition_options.alpha as f32)
}

impl VelloDrawTarget {
    fn render_to_texture(&mut self) {
        if matches!(
            self.state,
            State::RenderedToTexture | State::RenderedToBuffer
        ) {
            return;
        }

        self.renderer
            .borrow_mut()
            .render_to_texture(
                &self.device,
                &self.queue,
                &self.scene,
                &self.render_texture_view,
                &vello::RenderParams {
                    base_color: peniko::color::AlphaColor::TRANSPARENT,
                    width: self.size.width,
                    height: self.size.height,
                    antialiasing_method: vello::AaConfig::Area,
                },
            )
            .unwrap();
        self.state = State::RenderedToTexture;
        // prune scene
        self.scene.reset();
        // push all clip layers back
        for path in &self.clips {
            self.scene.push_clip_layer(kurbo::Affine::IDENTITY, &path.0);
        }
    }

    fn render_to_buffer(&mut self) {
        if matches!(self.state, State::RenderedToBuffer) {
            return;
        }
        self.render_to_texture();

        let size = extend3d(self.size);
        // TODO(perf): do a render pass that will multiply with alpha on GPU
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Copy out buffer"),
            });
        encoder.copy_texture_to_buffer(
            self.render_texture.as_image_copy(),
            TexelCopyBufferInfo {
                buffer: &self.rendered_buffer,
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.padded_byte_width),
                    rows_per_image: None,
                },
            },
            size,
        );
        self.queue.submit([encoder.finish()]);
        self.state = State::RenderedToBuffer;
    }

    fn map_read<R>(&mut self, f: impl FnOnce(Option<&[u8]>) -> R) -> R {
        self.render_to_buffer();
        let result = {
            let buf_slice = self.rendered_buffer.slice(..);
            let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
            buf_slice.map_async(MapMode::Read, move |v| sender.send(v).unwrap());
            if let Err(error) =
                vello::util::block_on_wgpu(&self.device, receiver.receive()).unwrap()
            {
                log::warn!("VELLO WGPU MAP ASYNC ERROR {error}");
                return f(None);
            }
            let data = buf_slice.get_mapped_range();
            f(Some(&data))
        };
        self.rendered_buffer.unmap();
        result
    }
}

fn extend3d(size: Size2D<u32>) -> Extent3d {
    Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: 1,
    }
}
