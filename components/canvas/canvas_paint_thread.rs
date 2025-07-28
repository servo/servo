/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(clippy::too_many_arguments)]

use std::borrow::ToOwned;
use std::collections::HashMap;
use std::sync::Arc;
use std::{f32, thread};

use canvas_traits::ConstellationCanvasMsg;
use canvas_traits::canvas::*;
use compositing_traits::CrossProcessCompositorApi;
use crossbeam_channel::{Sender, select, unbounded};
use euclid::default::{Rect, Size2D, Transform2D};
use fonts::{FontContext, SystemFontServiceProxy};
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use log::warn;
use net_traits::ResourceThreads;
use pixels::Snapshot;
use webrender_api::ImageKey;

use crate::canvas_data::*;

pub struct CanvasPaintThread {
    canvases: HashMap<CanvasId, Canvas>,
    next_canvas_id: CanvasId,
    compositor_api: CrossProcessCompositorApi,
    font_context: Arc<FontContext>,
}

impl CanvasPaintThread {
    fn new(
        compositor_api: CrossProcessCompositorApi,
        system_font_service: Arc<SystemFontServiceProxy>,
        resource_threads: ResourceThreads,
    ) -> CanvasPaintThread {
        CanvasPaintThread {
            canvases: HashMap::new(),
            next_canvas_id: CanvasId(0),
            compositor_api: compositor_api.clone(),
            font_context: Arc::new(FontContext::new(
                system_font_service,
                compositor_api,
                resource_threads,
            )),
        }
    }

    /// Creates a new `CanvasPaintThread` and returns an `IpcSender` to
    /// communicate with it.
    pub fn start(
        compositor_api: CrossProcessCompositorApi,
        system_font_service: Arc<SystemFontServiceProxy>,
        resource_threads: ResourceThreads,
    ) -> (Sender<ConstellationCanvasMsg>, IpcSender<CanvasMsg>) {
        let (ipc_sender, ipc_receiver) = ipc::channel::<CanvasMsg>().unwrap();
        let msg_receiver = ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(ipc_receiver);
        let (create_sender, create_receiver) = unbounded();
        thread::Builder::new()
            .name("Canvas".to_owned())
            .spawn(move || {
                let mut canvas_paint_thread = CanvasPaintThread::new(
                    compositor_api, system_font_service, resource_threads);
                loop {
                    select! {
                        recv(msg_receiver) -> msg => {
                            match msg {
                                Ok(CanvasMsg::Canvas2d(message, canvas_id)) => {
                                    canvas_paint_thread.process_canvas_2d_message(message, canvas_id);
                                },
                                Ok(CanvasMsg::Close(canvas_id)) => {
                                    canvas_paint_thread.canvases.remove(&canvas_id);
                                },
                                Ok(CanvasMsg::Recreate(size, canvas_id)) => {
                                    canvas_paint_thread.canvas(canvas_id).recreate(size);
                                },
                                Err(e) => {
                                    warn!("Error on CanvasPaintThread receive ({})", e);
                                },
                            }
                        }
                        recv(create_receiver) -> msg => {
                            match msg {
                                Ok(ConstellationCanvasMsg::Create { sender: creator, size }) => {
                                    creator.send(canvas_paint_thread.create_canvas(size)).unwrap();
                                },
                                Ok(ConstellationCanvasMsg::Exit(exit_sender)) => {
                                    let _ = exit_sender.send(());
                                    break;
                                },
                                Err(e) => {
                                    warn!("Error on CanvasPaintThread receive ({})", e);
                                    break;
                                },
                            }
                        }
                    }
                }
            })
            .expect("Thread spawning failed");

        (create_sender, ipc_sender)
    }

    pub fn create_canvas(&mut self, size: Size2D<u64>) -> Option<(CanvasId, ImageKey)> {
        let canvas_id = self.next_canvas_id;
        self.next_canvas_id.0 += 1;

        let canvas = Canvas::new(size, self.compositor_api.clone(), self.font_context.clone())?;
        let image_key = canvas.image_key();
        self.canvases.insert(canvas_id, canvas);

        Some((canvas_id, image_key))
    }

    fn process_canvas_2d_message(&mut self, message: Canvas2dMsg, canvas_id: CanvasId) {
        match message {
            Canvas2dMsg::FillText(
                text,
                x,
                y,
                max_width,
                style,
                is_rtl,
                text_options,
                shadow_options,
                composition_options,
                transform,
            ) => {
                self.canvas(canvas_id).fill_text(
                    text,
                    x,
                    y,
                    max_width,
                    is_rtl,
                    style,
                    text_options,
                    shadow_options,
                    composition_options,
                    transform,
                );
            },
            Canvas2dMsg::FillRect(rect, style, shadow_options, composition_options, transform) => {
                self.canvas(canvas_id).fill_rect(
                    &rect,
                    style,
                    shadow_options,
                    composition_options,
                    transform,
                );
            },
            Canvas2dMsg::StrokeRect(
                rect,
                style,
                line_options,
                shadow_options,
                composition_options,
                transform,
            ) => {
                self.canvas(canvas_id).stroke_rect(
                    &rect,
                    style,
                    line_options,
                    shadow_options,
                    composition_options,
                    transform,
                );
            },
            Canvas2dMsg::ClearRect(ref rect, transform) => {
                self.canvas(canvas_id).clear_rect(rect, transform)
            },
            Canvas2dMsg::FillPath(
                style,
                path,
                fill_rule,
                shadow_options,
                composition_options,
                transform,
            ) => {
                self.canvas(canvas_id).fill_path(
                    &path,
                    fill_rule,
                    style,
                    shadow_options,
                    composition_options,
                    transform,
                );
            },
            Canvas2dMsg::StrokePath(
                path,
                style,
                line_options,
                shadow_options,
                composition_options,
                transform,
            ) => {
                self.canvas(canvas_id).stroke_path(
                    &path,
                    style,
                    line_options,
                    shadow_options,
                    composition_options,
                    transform,
                );
            },
            Canvas2dMsg::ClipPath(path, fill_rule, transform) => {
                self.canvas(canvas_id)
                    .clip_path(&path, fill_rule, transform);
            },
            Canvas2dMsg::DrawImage(
                snapshot,
                dest_rect,
                source_rect,
                smoothing_enabled,
                shadow_options,
                composition_options,
                transform,
            ) => self.canvas(canvas_id).draw_image(
                snapshot.to_owned(),
                dest_rect,
                source_rect,
                smoothing_enabled,
                shadow_options,
                composition_options,
                transform,
            ),
            Canvas2dMsg::DrawEmptyImage(
                image_size,
                dest_rect,
                source_rect,
                shadow_options,
                composition_options,
                transform,
            ) => self.canvas(canvas_id).draw_image(
                Snapshot::cleared(image_size),
                dest_rect,
                source_rect,
                false,
                shadow_options,
                composition_options,
                transform,
            ),
            Canvas2dMsg::DrawImageInOther(
                other_canvas_id,
                dest_rect,
                source_rect,
                smoothing,
                shadow_options,
                composition_options,
                transform,
            ) => {
                let snapshot = self
                    .canvas(canvas_id)
                    .read_pixels(Some(source_rect.to_u32()));
                self.canvas(other_canvas_id).draw_image(
                    snapshot,
                    dest_rect,
                    source_rect,
                    smoothing,
                    shadow_options,
                    composition_options,
                    transform,
                );
            },
            Canvas2dMsg::MeasureText(text, sender, text_options) => {
                let metrics = self.canvas(canvas_id).measure_text(text, text_options);
                sender.send(metrics).unwrap();
            },
            Canvas2dMsg::GetImageData(dest_rect, sender) => {
                let snapshot = self.canvas(canvas_id).read_pixels(dest_rect);
                sender.send(snapshot.as_ipc()).unwrap();
            },
            Canvas2dMsg::PutImageData(rect, snapshot) => {
                self.canvas(canvas_id)
                    .put_image_data(snapshot.to_owned(), rect);
            },
            Canvas2dMsg::UpdateImage(sender) => {
                self.canvas(canvas_id).update_image_rendering();
                sender.send(()).unwrap();
            },
            Canvas2dMsg::PopClip => self.canvas(canvas_id).pop_clip(),
        }
    }

    fn canvas(&mut self, canvas_id: CanvasId) -> &mut Canvas {
        self.canvases.get_mut(&canvas_id).expect("Bogus canvas id")
    }
}

#[allow(clippy::large_enum_variant)]
enum Canvas {
    Raqote(CanvasData<raqote::DrawTarget>),
    #[cfg(feature = "vello")]
    Vello(CanvasData<crate::vello_backend::VelloDrawTarget>),
    #[cfg(feature = "vello_cpu")]
    VelloCPU(CanvasData<crate::vello_cpu_backend::VelloCPUDrawTarget>),
}

impl Canvas {
    fn new(
        size: Size2D<u64>,
        compositor_api: CrossProcessCompositorApi,
        font_context: Arc<FontContext>,
    ) -> Option<Self> {
        match servo_config::pref!(dom_canvas_backend)
            .to_lowercase()
            .as_str()
        {
            #[cfg(feature = "vello")]
            "vello" => Some(Self::Vello(CanvasData::new(
                size,
                compositor_api,
                font_context,
            ))),
            #[cfg(feature = "vello_cpu")]
            "vello_cpu" => Some(Self::VelloCPU(CanvasData::new(
                size,
                compositor_api,
                font_context,
            ))),
            "" | "auto" | "raqote" => Some(Self::Raqote(CanvasData::new(
                size,
                compositor_api,
                font_context,
            ))),
            s => {
                warn!("Unknown 2D canvas backend: `{s}`");
                None
            },
        }
    }

    fn image_key(&self) -> ImageKey {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.image_key(),
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.image_key(),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.image_key(),
        }
    }

    fn pop_clip(&mut self) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.pop_clip(),
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.pop_clip(),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.pop_clip(),
        }
    }

    fn fill_text(
        &mut self,
        text: String,
        x: f64,
        y: f64,
        max_width: Option<f64>,
        is_rtl: bool,
        style: FillOrStrokeStyle,
        text_options: TextOptions,
        shadow_options: ShadowOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    ) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.fill_text(
                text,
                x,
                y,
                max_width,
                is_rtl,
                style,
                text_options,
                shadow_options,
                composition_options,
                transform,
            ),
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.fill_text(
                text,
                x,
                y,
                max_width,
                is_rtl,
                style,
                text_options,
                shadow_options,
                composition_options,
                transform,
            ),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.fill_text(
                text,
                x,
                y,
                max_width,
                is_rtl,
                style,
                text_options,
                shadow_options,
                composition_options,
                transform,
            ),
        }
    }

    fn fill_rect(
        &mut self,
        rect: &Rect<f32>,
        style: FillOrStrokeStyle,
        shadow_options: ShadowOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    ) {
        match self {
            Canvas::Raqote(canvas_data) => {
                canvas_data.fill_rect(rect, style, shadow_options, composition_options, transform)
            },
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => {
                canvas_data.fill_rect(rect, style, shadow_options, composition_options, transform)
            },
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => {
                canvas_data.fill_rect(rect, style, shadow_options, composition_options, transform)
            },
        }
    }

    fn stroke_rect(
        &mut self,
        rect: &Rect<f32>,
        style: FillOrStrokeStyle,
        line_options: LineOptions,
        shadow_options: ShadowOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    ) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.stroke_rect(
                rect,
                style,
                line_options,
                shadow_options,
                composition_options,
                transform,
            ),
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.stroke_rect(
                rect,
                style,
                line_options,
                shadow_options,
                composition_options,
                transform,
            ),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.stroke_rect(
                rect,
                style,
                line_options,
                shadow_options,
                composition_options,
                transform,
            ),
        }
    }

    fn fill_path(
        &mut self,
        path: &Path,
        fill_rule: FillRule,
        style: FillOrStrokeStyle,
        shadow_options: ShadowOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    ) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.fill_path(
                path,
                fill_rule,
                style,
                shadow_options,
                composition_options,
                transform,
            ),
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.fill_path(
                path,
                fill_rule,
                style,
                shadow_options,
                composition_options,
                transform,
            ),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.fill_path(
                path,
                fill_rule,
                style,
                shadow_options,
                composition_options,
                transform,
            ),
        }
    }

    fn stroke_path(
        &mut self,
        path: &Path,
        style: FillOrStrokeStyle,
        line_options: LineOptions,
        shadow_options: ShadowOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    ) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.stroke_path(
                path,
                style,
                line_options,
                shadow_options,
                composition_options,
                transform,
            ),
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.stroke_path(
                path,
                style,
                line_options,
                shadow_options,
                composition_options,
                transform,
            ),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.stroke_path(
                path,
                style,
                line_options,
                shadow_options,
                composition_options,
                transform,
            ),
        }
    }

    fn clear_rect(&mut self, rect: &Rect<f32>, transform: Transform2D<f32>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.clear_rect(rect, transform),
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.clear_rect(rect, transform),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.clear_rect(rect, transform),
        }
    }

    fn draw_image(
        &mut self,
        snapshot: Snapshot,
        dest_rect: Rect<f64>,
        source_rect: Rect<f64>,
        smoothing_enabled: bool,
        shadow_options: ShadowOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    ) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.draw_image(
                snapshot,
                dest_rect,
                source_rect,
                smoothing_enabled,
                shadow_options,
                composition_options,
                transform,
            ),
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.draw_image(
                snapshot,
                dest_rect,
                source_rect,
                smoothing_enabled,
                shadow_options,
                composition_options,
                transform,
            ),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.draw_image(
                snapshot,
                dest_rect,
                source_rect,
                smoothing_enabled,
                shadow_options,
                composition_options,
                transform,
            ),
        }
    }

    fn read_pixels(&mut self, read_rect: Option<Rect<u32>>) -> Snapshot {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.read_pixels(read_rect),
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.read_pixels(read_rect),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.read_pixels(read_rect),
        }
    }

    fn measure_text(&mut self, text: String, text_options: TextOptions) -> TextMetrics {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.measure_text(text, text_options),
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.measure_text(text, text_options),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.measure_text(text, text_options),
        }
    }

    fn clip_path(&mut self, path: &Path, fill_rule: FillRule, transform: Transform2D<f32>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.clip_path(path, fill_rule, transform),
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.clip_path(path, fill_rule, transform),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.clip_path(path, fill_rule, transform),
        }
    }

    fn put_image_data(&mut self, snapshot: Snapshot, rect: Rect<u32>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.put_image_data(snapshot, rect),
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.put_image_data(snapshot, rect),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.put_image_data(snapshot, rect),
        }
    }

    fn update_image_rendering(&mut self) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.update_image_rendering(),
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.update_image_rendering(),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.update_image_rendering(),
        }
    }

    fn recreate(&mut self, size: Option<Size2D<u64>>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.recreate(size),
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.recreate(size),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.recreate(size),
        }
    }
}
