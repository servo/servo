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
                                Ok(CanvasMsg::FromScript(message, canvas_id)) => match message {
                                    FromScriptMsg::SendPixels(chan) => {
                                        chan.send(canvas_paint_thread
                                            .canvas(canvas_id)
                                            .read_pixels(None, None)
                                            .as_ipc()
                                        ).unwrap();
                                    },
                                },
                                Err(e) => {
                                    warn!("Error on CanvasPaintThread receive ({})", e);
                                },
                            }
                        }
                        recv(create_receiver) -> msg => {
                            match msg {
                                Ok(ConstellationCanvasMsg::Create { sender: creator, size }) => {
                                    let canvas_data = canvas_paint_thread.create_canvas(size);
                                    creator.send(canvas_data).unwrap();
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

    pub fn create_canvas(&mut self, size: Size2D<u64>) -> (CanvasId, ImageKey) {
        let canvas_id = self.next_canvas_id;
        self.next_canvas_id.0 += 1;

        let canvas_data =
            CanvasData::new(size, self.compositor_api.clone(), self.font_context.clone());
        let image_key = canvas_data.image_key();
        self.canvases.insert(canvas_id, Canvas::Raqote(canvas_data));

        (canvas_id, image_key)
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
            Canvas2dMsg::FillPath(style, path, shadow_options, composition_options, transform) => {
                self.canvas(canvas_id).fill_path(
                    &path,
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
            Canvas2dMsg::ClipPath(path, transform) => {
                self.canvas(canvas_id).clip_path(&path, transform);
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
                image_size,
                dest_rect,
                source_rect,
                smoothing,
                shadow_options,
                composition_options,
                transform,
            ) => {
                let snapshot = self
                    .canvas(canvas_id)
                    .read_pixels(Some(source_rect.to_u32()), Some(image_size));
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
            Canvas2dMsg::GetImageData(dest_rect, canvas_size, sender) => {
                let snapshot = self
                    .canvas(canvas_id)
                    .read_pixels(Some(dest_rect), Some(canvas_size));
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

enum Canvas {
    Raqote(CanvasData<raqote::DrawTarget>),
}

impl Canvas {
    fn pop_clip(&mut self) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.pop_clip(),
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
        }
    }

    fn fill_path(
        &mut self,
        path: &Path,
        style: FillOrStrokeStyle,
        shadow_options: ShadowOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    ) {
        match self {
            Canvas::Raqote(canvas_data) => {
                canvas_data.fill_path(path, style, shadow_options, composition_options, transform)
            },
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
        }
    }

    fn clear_rect(&mut self, rect: &Rect<f32>, transform: Transform2D<f32>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.clear_rect(rect, transform),
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
        }
    }

    fn read_pixels(
        &mut self,
        read_rect: Option<Rect<u32>>,
        canvas_size: Option<Size2D<u32>>,
    ) -> Snapshot {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.read_pixels(read_rect, canvas_size),
        }
    }

    fn measure_text(&mut self, text: String, text_options: TextOptions) -> TextMetrics {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.measure_text(text, text_options),
        }
    }

    fn clip_path(&mut self, path: &Path, transform: Transform2D<f32>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.clip_path(path, transform),
        }
    }

    fn put_image_data(&mut self, snapshot: Snapshot, rect: Rect<u32>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.put_image_data(snapshot, rect),
        }
    }

    fn update_image_rendering(&mut self) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.update_image_rendering(),
        }
    }

    fn recreate(&mut self, size: Option<Size2D<u64>>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.recreate(size),
        }
    }
}
