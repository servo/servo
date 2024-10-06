/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use canvas::canvas_data::CanvasData;
use canvas::canvas_paint_thread::AntialiasMode;
use canvas_traits::canvas::FillOrStrokeStyle;
use euclid::default::{Point2D, Rect, Transform2D};
use euclid::point2;
use fonts::{FontContext, SystemFontServiceProxy};
use ipc_channel::ipc::{self, IpcSharedMemory};
use net_traits::ResourceThreads;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon::{ThreadPool, ThreadPoolBuilder};
use serde::{Deserialize, Serialize};
use style::color::AbsoluteColor;
use webrender_api::units::{BlobDirtyRect, BlobToDeviceTranslation, DeviceIntRect};
use webrender_api::{
    AsyncBlobImageRasterizer, BlobImageData, BlobImageHandler, BlobImageKey, BlobImageParams,
    BlobImageRequest, BlobImageResult, DirtyRect, ImageFormat, RasterizedBlobImage, TileSize,
};
use webrender_traits::CrossProcessCompositorApi;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum BlobImageEntryData {
    Fill,
    BeginPath,
    ClosePath,
    SetOpaqueWhite,
    SetTransform(Transform2D<f32>),
    MoveTo(Point2D<f32>),
    LineTo(Point2D<f32>),
    FillRect(Rect<f32>),
}

#[derive(Clone, Deserialize, Serialize)]
pub struct BlobImageEntry {
    pub bounds: DeviceIntRect,
    pub data: BlobImageEntryData,
}

#[derive(Clone, Debug)]
pub struct BlobImageCommand {
    data: Arc<BlobImageData>,
    visible_rect: DeviceIntRect,
    #[allow(unused)]
    tile_size: TileSize,
}

pub struct ServoBlobImageHandler {
    workers: Arc<ThreadPool>,
    font_context: Arc<FontContext>,
    blob_commands: Arc<Mutex<HashMap<BlobImageKey, BlobImageCommand>>>,
    enable_multithreading: bool,
}

pub struct ServoBlobRasterizer {
    workers: Arc<ThreadPool>,
    font_context: Arc<FontContext>,
    blob_commands: Arc<Mutex<HashMap<BlobImageKey, BlobImageCommand>>>,
    enable_multithreading: bool,
}

impl ServoBlobImageHandler {
    pub fn new(
        system_font_service: Arc<SystemFontServiceProxy>,
        resource_threads: ResourceThreads,
    ) -> ServoBlobImageHandler {
        let mock_compositor_api = CrossProcessCompositorApi::dummy();
        let thread_count = std::thread::available_parallelism()
            .unwrap_or(NonZeroUsize::new(4).unwrap())
            .get();
        let workers = ThreadPoolBuilder::new()
            .thread_name(|i| format!("ServoBlobImageRasterizer#{i}"))
            .num_threads(thread_count)
            .build()
            .unwrap();
        Self {
            workers: Arc::new(workers),
            enable_multithreading: true,
            blob_commands: Arc::new(Mutex::new(HashMap::new())),
            font_context: Arc::new(FontContext::new(
                system_font_service,
                mock_compositor_api,
                resource_threads,
            )),
        }
    }
}

impl BlobImageEntry {
    fn from_serialized(data: &[u8]) -> Vec<Self> {
        bincode::deserialize(data).unwrap()
    }
}

impl BlobImageHandler for ServoBlobImageHandler {
    fn create_similar(&self) -> Box<dyn BlobImageHandler> {
        Box::new(ServoBlobImageHandler {
            workers: self.workers.clone(),
            enable_multithreading: self.enable_multithreading,
            blob_commands: self.blob_commands.clone(),
            font_context: self.font_context.clone(),
        })
    }

    fn create_blob_rasterizer(&mut self) -> Box<dyn AsyncBlobImageRasterizer> {
        Box::new(ServoBlobRasterizer {
            workers: self.workers.clone(),
            enable_multithreading: self.enable_multithreading,
            blob_commands: self.blob_commands.clone(),
            font_context: self.font_context.clone(),
        })
    }

    fn add(
        &mut self,
        key: BlobImageKey,
        data: Arc<BlobImageData>,
        visible_rect: &DeviceIntRect,
        tile_size: TileSize,
    ) {
        self.blob_commands.lock().unwrap().insert(
            key,
            BlobImageCommand {
                data,
                visible_rect: *visible_rect,
                tile_size,
            },
        );
    }

    fn update(
        &mut self,
        key: BlobImageKey,
        data: Arc<BlobImageData>,
        visible_rect: &DeviceIntRect,
        dirty_rect: &BlobDirtyRect,
    ) {
        let new_blob_entry = BlobImageEntry::from_serialized(data.as_ref());
        let dirty_rect = match *dirty_rect {
            DirtyRect::Partial(rect) => rect.cast_unit(),
            DirtyRect::All => DeviceIntRect {
                min: point2(i32::MIN, i32::MIN),
                max: point2(i32::MAX, i32::MAX),
            },
        };
        if let Some(command) = self.blob_commands.lock().unwrap().get_mut(&key) {
            let mut dirty_entry = Vec::new();
            for entry in new_blob_entry {
                let preserved_rect = command.visible_rect.intersection_unchecked(&visible_rect);
                let preserved_bounds = preserved_rect.intersection_unchecked(&entry.bounds);
                if dirty_rect.contains_box(&preserved_bounds) {
                    dirty_entry.push(entry);
                } else {
                    let old = BlobImageEntry::from_serialized(command.data.as_ref());
                    let old = old
                        .iter()
                        .find(|&x| x.bounds == entry.bounds)
                        .unwrap()
                        .clone();
                    dirty_entry.push(old);
                }
            }
            command.data = bincode::serialize(&dirty_entry).unwrap().into();
            command.visible_rect = *visible_rect;
        }
    }

    fn delete(&mut self, key: BlobImageKey) {
        self.blob_commands.lock().unwrap().remove(&key);
    }

    fn enable_multithreading(&mut self, enable: bool) {
        self.enable_multithreading = enable;
    }

    // Servo doesn't have cache for non-system fonts.
    fn delete_font(&mut self, _key: webrender_api::FontKey) {}
    fn delete_font_instance(&mut self, _key: webrender_api::FontInstanceKey) {}
    fn prepare_resources(
        &mut self,
        _services: &dyn webrender_api::BlobImageResources,
        _requests: &[webrender_api::BlobImageParams],
    ) {
    }
    fn clear_namespace(&mut self, _namespace: webrender_api::IdNamespace) {}
}

impl ServoBlobRasterizer {
    #[tracing::instrument(level = "trace", skip(self, canvas))]
    fn process_blob(&self, entry: BlobImageEntryData, canvas: &mut CanvasData) {
        match entry {
            BlobImageEntryData::Fill => canvas.fill(),
            BlobImageEntryData::BeginPath => canvas.begin_path(),
            BlobImageEntryData::ClosePath => canvas.close_path(),
            BlobImageEntryData::MoveTo(point) => canvas.move_to(&point),
            BlobImageEntryData::LineTo(point) => canvas.line_to(&point),
            BlobImageEntryData::FillRect(rect) => canvas.fill_rect(&rect),
            BlobImageEntryData::SetOpaqueWhite => {
                canvas.set_fill_style(FillOrStrokeStyle::Color(AbsoluteColor::WHITE))
            },
            BlobImageEntryData::SetTransform(transform) => canvas.set_transform(&transform),
        }
    }

    fn write_canvas(&self, canvas: CanvasData) {
        if let Ok(path) = std::env::var("SERVO_BLOB_OUTPUT_DIR") {
            use std::path::Path;
            use std::sync::atomic::{AtomicU32, Ordering};
            static RASTERIZED_BLOB_COUNT: AtomicU32 = AtomicU32::new(0);
            let filename = format!(
                "rasterized-blob-{}.png",
                RASTERIZED_BLOB_COUNT.load(Ordering::SeqCst)
            );
            let _ = canvas.save_png(&Path::new(&path).join(filename));
            RASTERIZED_BLOB_COUNT.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn rasterize_blob(&self, params: &BlobImageParams) -> (BlobImageRequest, BlobImageResult) {
        if params.descriptor.format != ImageFormat::BGRA8 {
            panic!("Swizzling non-BGRA8 format is not supported");
        }
        let mut canvas_data = CanvasData::new(
            params.descriptor.rect.size().cast::<u64>().cast_unit(),
            AntialiasMode::Default,
            self.font_context.clone(),
        );
        let command = &self.blob_commands.lock().unwrap()[&params.request.key];
        BlobImageEntry::from_serialized(command.data.as_ref())
            .into_iter()
            .for_each(|entry| self.process_blob(entry.data, &mut canvas_data));
        let (tx, rx) = ipc::channel::<IpcSharedMemory>().unwrap();
        canvas_data.send_pixels(tx);
        self.write_canvas(canvas_data);
        let dirty_rect = params.dirty_rect.to_subrect_of(&params.descriptor.rect);
        let tx: BlobToDeviceTranslation = (-params.descriptor.rect.min.to_vector()).into();
        let rasterized_rect = tx.transform_box(&dirty_rect);
        (
            params.request,
            Ok(RasterizedBlobImage {
                rasterized_rect,
                data: Arc::new(rx.recv().unwrap().to_vec()),
            }),
        )
    }
}

impl AsyncBlobImageRasterizer for ServoBlobRasterizer {
    fn rasterize(
        &mut self,
        requests: &[BlobImageParams],
        _low_priority: bool,
    ) -> Vec<(BlobImageRequest, BlobImageResult)> {
        if self.enable_multithreading {
            self.workers.install(|| {
                requests
                    .into_par_iter()
                    .map(|r| self.rasterize_blob(r))
                    .collect()
            })
        } else {
            requests
                .into_iter()
                .map(|r| self.rasterize_blob(r))
                .collect()
        }
    }
}
