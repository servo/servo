/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::{f32, thread};

use base::generic_channel::GenericSender;
use base::{Epoch, generic_channel};
use canvas_traits::ConstellationCanvasMsg;
use canvas_traits::canvas::*;
use compositing_traits::CrossProcessPaintApi;
use crossbeam_channel::{Sender, select, unbounded};
use euclid::default::{Rect, Size2D, Transform2D};
use log::warn;
use pixels::Snapshot;
use rustc_hash::FxHashMap;
use webrender_api::ImageKey;

use crate::canvas_data::*;

pub struct CanvasPaintThread {
    canvases: FxHashMap<CanvasId, Canvas>,
    next_canvas_id: CanvasId,
    paint_api: CrossProcessPaintApi,
}

impl CanvasPaintThread {
    fn new(paint_api: CrossProcessPaintApi) -> CanvasPaintThread {
        CanvasPaintThread {
            canvases: FxHashMap::default(),
            next_canvas_id: CanvasId(0),
            paint_api: paint_api.clone(),
        }
    }

    /// Creates a new `CanvasPaintThread` and returns an `IpcSender` to
    /// communicate with it.
    pub fn start(
        paint_api: CrossProcessPaintApi,
    ) -> (Sender<ConstellationCanvasMsg>, GenericSender<CanvasMsg>) {
        let (ipc_sender, ipc_receiver) = generic_channel::channel::<CanvasMsg>().unwrap();
        let msg_receiver = ipc_receiver.route_preserving_errors();
        let (create_sender, create_receiver) = unbounded();
        thread::Builder::new()
            .name("Canvas".to_owned())
            .spawn(move || {
                let mut canvas_paint_thread = CanvasPaintThread::new(
                    paint_api);
                loop {
                    select! {
                        recv(msg_receiver) -> msg => {
                            match msg {
                                Ok(Ok(CanvasMsg::Canvas2d(message, canvas_id))) => {
                                    canvas_paint_thread.process_canvas_2d_message(message, canvas_id);
                                },
                                Ok(Ok(CanvasMsg::Close(canvas_id))) => {
                                    canvas_paint_thread.canvases.remove(&canvas_id);
                                },
                                Ok(Ok(CanvasMsg::Recreate(size, canvas_id))) => {
                                    canvas_paint_thread.canvas(canvas_id).recreate(size);
                                },
                                Ok(Err(e)) => {
                                    warn!("CanvasPaintThread message deserialization error: {e:?}");
                                }
                                Err(_disconnected) => {
                                    warn!("CanvasMsg receiver disconnected");
                                    break;
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

    #[servo_tracing::instrument(skip_all)]
    pub fn create_canvas(&mut self, size: Size2D<u64>) -> Option<CanvasId> {
        let canvas_id = self.next_canvas_id;
        self.next_canvas_id.0 += 1;

        let canvas = Canvas::new(size, self.paint_api.clone())?;
        self.canvases.insert(canvas_id, canvas);

        Some(canvas_id)
    }

    #[servo_tracing::instrument(
        skip_all,
        fields(message = message.to_string())
    )]
    fn process_canvas_2d_message(&mut self, message: Canvas2dMsg, canvas_id: CanvasId) {
        match message {
            Canvas2dMsg::SetImageKey(image_key) => {
                self.canvas(canvas_id).set_image_key(image_key);
            },
            Canvas2dMsg::FillText(
                text_bounds,
                text_runs,
                fill_or_stroke_style,
                shadow_options,
                composition_options,
                transform,
            ) => {
                self.canvas(canvas_id).fill_text(
                    text_bounds,
                    text_runs,
                    fill_or_stroke_style,
                    shadow_options,
                    composition_options,
                    transform,
                );
            },
            Canvas2dMsg::StrokeText(
                text_bounds,
                text_runs,
                fill_or_stroke_style,
                line_options,
                shadow_options,
                composition_options,
                transform,
            ) => {
                self.canvas(canvas_id).stroke_text(
                    text_bounds,
                    text_runs,
                    fill_or_stroke_style,
                    line_options,
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
            Canvas2dMsg::GetImageData(dest_rect, sender) => {
                let snapshot = self.canvas(canvas_id).read_pixels(dest_rect);
                sender.send(snapshot.to_shared()).unwrap();
            },
            Canvas2dMsg::PutImageData(rect, snapshot) => {
                self.canvas(canvas_id)
                    .put_image_data(snapshot.to_owned(), rect);
            },
            Canvas2dMsg::UpdateImage(canvas_epoch) => {
                self.canvas(canvas_id).update_image_rendering(canvas_epoch);
            },
            Canvas2dMsg::PopClips(clips) => self.canvas(canvas_id).pop_clips(clips),
        }
    }

    fn canvas(&mut self, canvas_id: CanvasId) -> &mut Canvas {
        self.canvases.get_mut(&canvas_id).expect("Bogus canvas id")
    }
}

enum Canvas {
    #[cfg(feature = "vello")]
    Vello(CanvasData<crate::vello_backend::VelloDrawTarget>),
    #[cfg(feature = "vello_cpu")]
    VelloCPU(CanvasData<crate::vello_cpu_backend::VelloCPUDrawTarget>),
}

impl Canvas {
    fn new(size: Size2D<u64>, paint_api: CrossProcessPaintApi) -> Option<Self> {
        match servo_config::pref!(dom_canvas_backend)
            .to_lowercase()
            .as_str()
        {
            #[cfg(feature = "vello_cpu")]
            "" | "auto" | "vello_cpu" => Some(Self::VelloCPU(CanvasData::new(size, paint_api))),
            #[cfg(feature = "vello")]
            "" | "auto" | "vello" => Some(Self::Vello(CanvasData::new(size, paint_api))),
            s => {
                warn!("Unknown 2D canvas backend: `{s}`");
                None
            },
        }
    }

    fn set_image_key(&mut self, image_key: ImageKey) {
        match self {
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.set_image_key(image_key),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.set_image_key(image_key),
        }
    }

    fn pop_clips(&mut self, clips: usize) {
        match self {
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.pop_clips(clips),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.pop_clips(clips),
        }
    }

    fn stroke_text(
        &mut self,
        text_bounds: Rect<f64>,
        text_runs: Vec<TextRun>,
        fill_or_stroke_style: FillOrStrokeStyle,
        line_options: LineOptions,
        shadow_options: ShadowOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f64>,
    ) {
        match self {
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.stroke_text(
                text_bounds,
                text_runs,
                fill_or_stroke_style,
                line_options,
                shadow_options,
                composition_options,
                transform,
            ),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.stroke_text(
                text_bounds,
                text_runs,
                fill_or_stroke_style,
                line_options,
                shadow_options,
                composition_options,
                transform,
            ),
        }
    }

    fn fill_text(
        &mut self,
        text_bounds: Rect<f64>,
        text_runs: Vec<TextRun>,
        fill_or_stroke_style: FillOrStrokeStyle,
        shadow_options: ShadowOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f64>,
    ) {
        match self {
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.fill_text(
                text_bounds,
                text_runs,
                fill_or_stroke_style,
                shadow_options,
                composition_options,
                transform,
            ),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.fill_text(
                text_bounds,
                text_runs,
                fill_or_stroke_style,
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
        transform: Transform2D<f64>,
    ) {
        match self {
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
        transform: Transform2D<f64>,
    ) {
        match self {
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
        transform: Transform2D<f64>,
    ) {
        match self {
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
        transform: Transform2D<f64>,
    ) {
        match self {
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

    fn clear_rect(&mut self, rect: &Rect<f32>, transform: Transform2D<f64>) {
        match self {
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.clear_rect(rect, transform),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.clear_rect(rect, transform),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_image(
        &mut self,
        snapshot: Snapshot,
        dest_rect: Rect<f64>,
        source_rect: Rect<f64>,
        smoothing_enabled: bool,
        shadow_options: ShadowOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f64>,
    ) {
        match self {
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
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.read_pixels(read_rect),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.read_pixels(read_rect),
        }
    }

    fn clip_path(&mut self, path: &Path, fill_rule: FillRule, transform: Transform2D<f64>) {
        match self {
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.clip_path(path, fill_rule, transform),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.clip_path(path, fill_rule, transform),
        }
    }

    fn put_image_data(&mut self, snapshot: Snapshot, rect: Rect<u32>) {
        match self {
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.put_image_data(snapshot, rect),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.put_image_data(snapshot, rect),
        }
    }

    fn update_image_rendering(&mut self, canvas_epoch: Option<Epoch>) {
        match self {
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.update_image_rendering(canvas_epoch),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.update_image_rendering(canvas_epoch),
        }
    }

    fn recreate(&mut self, size: Option<Size2D<u64>>) {
        match self {
            #[cfg(feature = "vello")]
            Canvas::Vello(canvas_data) => canvas_data.recreate(size),
            #[cfg(feature = "vello_cpu")]
            Canvas::VelloCPU(canvas_data) => canvas_data.recreate(size),
        }
    }
}
