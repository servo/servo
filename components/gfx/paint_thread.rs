/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The thread that handles all painting.

use app_units::Au;
use azure::AzFloat;
use azure::azure_hl::{BackendType, Color, DrawTarget, SurfaceFormat};
use display_list::{DisplayItem, DisplayList, DisplayListTraversal};
use display_list::{LayerInfo, StackingContext, StackingContextType};
use euclid::Matrix4D;
use euclid::point::Point2D;
use euclid::rect::Rect;
use euclid::size::Size2D;
use font_cache_thread::FontCacheThread;
use font_context::FontContext;
use gfx_traits::{ChromeToPaintMsg, Epoch, LayerId, LayerKind, LayerProperties};
use gfx_traits::{PaintListener, PaintRequest, StackingContextId};
use layers::layers::{BufferRequest, LayerBuffer, LayerBufferSet};
use layers::platform::surface::{NativeDisplay, NativeSurface};
use msg::constellation_msg::PipelineId;
use paint_context::PaintContext;
use profile_traits::mem;
use profile_traits::time;
use rand::{self, Rng};
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::mem as std_mem;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use url::Url;
use util::geometry::ExpandToPixelBoundaries;
use util::opts;
use util::thread;
use util::thread_state;

#[derive(Clone, HeapSizeOf)]
struct PaintLayer {
    /// The LayerProperties, which describe the layer in a way that the Compositor
    /// can consume.
    pub layer_properties: LayerProperties,

    /// The StackingContextId of the StackingContext that is the immediate
    /// parent of this layer. This is used to ensure applying the proper transform
    /// when painting.
    pub starting_stacking_context_id: StackingContextId,

    /// The indices (in the DisplayList) to the first and last display item
    /// that are the contents of this layer.
    pub display_list_indices: Option<(usize, usize)>,

    /// When painting, whether to draw the start by entering the surrounding StackingContext
    /// or simply to draw the single item this PaintLayer contains.
    pub single_item: bool,

    /// The layer's bounds start at the overflow origin, but display items are
    /// positioned relative to the stacking context bounds, so we need to
    /// offset by the overflow rect (which will be in the coordinate system of
    /// the stacking context bounds).
    pub display_list_origin: Point2D<f32>
}

impl PaintLayer {
    fn new_from_stacking_context(layer_info: &LayerInfo,
                                 stacking_context: &StackingContext,
                                 parent_origin: &Point2D<Au>,
                                 transform: &Matrix4D<f32>,
                                 perspective: &Matrix4D<f32>,
                                 parent_id: Option<LayerId>)
                                 -> PaintLayer {
        let bounds = Rect::new(stacking_context.bounds.origin + stacking_context.overflow.origin,
                               stacking_context.overflow.size);
        let layer_boundaries = Rect::new(
            Point2D::new((parent_origin.x + bounds.min_x()).to_nearest_px() as f32,
                         (parent_origin.y + bounds.min_y()).to_nearest_px() as f32),
            Size2D::new(bounds.size.width.to_nearest_px() as f32,
                        bounds.size.height.to_nearest_px() as f32));

        let transform = transform.mul(&stacking_context.transform);
        let perspective = perspective.mul(&stacking_context.perspective);
        let establishes_3d_context = stacking_context.establishes_3d_context;
        let scrolls_overflow_area = stacking_context.scrolls_overflow_area;

        PaintLayer {
            layer_properties: LayerProperties {
                id: layer_info.layer_id,
                parent_id: parent_id,
                rect: layer_boundaries,
                background_color: layer_info.background_color,
                scroll_policy: layer_info.scroll_policy,
                transform: transform,
                perspective: perspective,
                establishes_3d_context: establishes_3d_context,
                scrolls_overflow_area: scrolls_overflow_area,
                subpage_pipeline_id: layer_info.subpage_pipeline_id,
            },
            starting_stacking_context_id: stacking_context.id,
            display_list_indices: None,
            single_item: false,
            display_list_origin: Point2D::new(stacking_context.overflow.origin.x.to_f32_px(),
                                              stacking_context.overflow.origin.y.to_f32_px()),
        }
    }

    fn new_for_display_item(layer_info: &LayerInfo,
                            item_bounds: &Rect<Au>,
                            parent_origin: &Point2D<Au>,
                            transform: &Matrix4D<f32>,
                            perspective: &Matrix4D<f32>,
                            parent_id: Option<LayerId>,
                            stacking_context_id: StackingContextId,
                            item_index: usize)
                            -> PaintLayer {
        let bounds = item_bounds.expand_to_px_boundaries();
        let layer_boundaries = Rect::new(
            Point2D::new((parent_origin.x + bounds.min_x()).to_nearest_px() as f32,
                         (parent_origin.y + bounds.min_y()).to_nearest_px() as f32),
            Size2D::new(bounds.size.width.to_nearest_px() as f32,
                        bounds.size.height.to_nearest_px() as f32));

        PaintLayer {
            layer_properties: LayerProperties {
                id: layer_info.layer_id,
                parent_id: parent_id,
                rect: layer_boundaries,
                background_color: layer_info.background_color,
                scroll_policy: layer_info.scroll_policy,
                transform: *transform,
                perspective: *perspective,
                establishes_3d_context: false,
                scrolls_overflow_area: false,
                subpage_pipeline_id: layer_info.subpage_pipeline_id,
            },
            starting_stacking_context_id: stacking_context_id,
            display_list_indices: Some((item_index, item_index)),
            single_item: true,
            display_list_origin: Point2D::new(bounds.origin.x.to_f32_px(),
                                              bounds.origin.y.to_f32_px()),
        }
    }

    fn add_item(&mut self, index: usize) {
        let indices = match self.display_list_indices {
            Some((first, _)) => (first, index),
            None => (index, index),
        };
        self.display_list_indices = Some(indices);
    }

    fn make_companion_layer(&mut self) {
        self.layer_properties.id = self.layer_properties.id.companion_layer_id();
        self.display_list_indices = None;
    }
}

struct LayerCreator {
    layers: Vec<PaintLayer>,
    layer_details_stack: Vec<PaintLayer>,
    current_layer: Option<PaintLayer>,
    current_item_index: usize,
}

impl LayerCreator {
    fn create_layers_with_display_list(display_list: &DisplayList) -> Vec<PaintLayer> {
        let mut layer_creator = LayerCreator {
            layers: Vec::new(),
            layer_details_stack: Vec::new(),
            current_layer: None,
            current_item_index: 0,
        };
        let mut traversal = DisplayListTraversal {
            display_list: display_list,
            current_item_index: 0,
            last_item_index: display_list.list.len(),
        };
        layer_creator.create_layers_for_stacking_context(&display_list.root_stacking_context,
                                                         &mut traversal,
                                                         &Point2D::zero(),
                                                         &Matrix4D::identity(),
                                                         &Matrix4D::identity());
        layer_creator.layers
    }

    fn finalize_current_layer(&mut self) {
        if let Some(current_layer) = self.current_layer.take() {
            self.layers.push(current_layer);
        }
    }

    fn current_parent_layer_id(&self) -> Option<LayerId> {
        self.layer_details_stack.last().as_ref().map(|layer|
            layer.layer_properties.id
        )
    }

    fn current_parent_stacking_context_id(&self) -> StackingContextId {
        self.layer_details_stack.last().unwrap().starting_stacking_context_id
    }

    fn create_layers_for_stacking_context<'a>(&mut self,
                                              stacking_context: &StackingContext,
                                              traversal: &mut DisplayListTraversal<'a>,
                                              parent_origin: &Point2D<Au>,
                                              transform: &Matrix4D<f32>,
                                              perspective: &Matrix4D<f32>) {
        if let Some(ref layer_info) = stacking_context.layer_info {
            self.finalize_current_layer();
            let new_layer = PaintLayer::new_from_stacking_context(
                    layer_info,
                    stacking_context,
                    parent_origin,
                    transform,
                    perspective,
                    self.current_parent_layer_id());
            self.layer_details_stack.push(new_layer.clone());
            self.current_layer = Some(new_layer);

            // When there is a new layer, the transforms and origin are handled by
            // the compositor, so the new transform and perspective matrices are
            // just the identity.
            //
            // The origin for child layers which might be somewhere other than the
            // layer origin, since layer boundaries are expanded to include overflow.
            self.process_stacking_context_items(stacking_context,
                                                traversal,
                                                &-stacking_context.overflow.origin,
                                                &Matrix4D::identity(),
                                                &Matrix4D::identity());
            self.finalize_current_layer();
            self.layer_details_stack.pop();
            return;
        }

        if stacking_context.context_type != StackingContextType::Real {
            self.process_stacking_context_items(stacking_context,
                                                traversal,
                                                parent_origin,
                                                transform,
                                                perspective);
            return;
        }

        self.process_stacking_context_items(stacking_context,
                                            traversal,
                                            &(stacking_context.bounds.origin + *parent_origin),
                                            &transform.mul(&stacking_context.transform),
                                            &perspective.mul(&stacking_context.perspective));
    }

    fn process_stacking_context_items<'a>(&mut self,
                                          stacking_context: &StackingContext,
                                          traversal: &mut DisplayListTraversal<'a>,
                                          parent_origin: &Point2D<Au>,
                                          transform: &Matrix4D<f32>,
                                          perspective: &Matrix4D<f32>) {
        for kid in stacking_context.children.iter() {
            while let Some(item) = traversal.advance(stacking_context) {
                self.create_layers_for_item(item,
                                            parent_origin,
                                            transform,
                                            perspective);
            }
            self.create_layers_for_stacking_context(kid,
                                                    traversal,
                                                    parent_origin,
                                                    transform,
                                                    perspective);
        }

        while let Some(item) = traversal.advance(stacking_context) {
            self.create_layers_for_item(item,
                                        parent_origin,
                                        transform,
                                        perspective);
        }
    }


    fn create_layers_for_item<'a>(&mut self,
                                  item: &DisplayItem,
                                  parent_origin: &Point2D<Au>,
                                  transform: &Matrix4D<f32>,
                                  perspective: &Matrix4D<f32>) {
        if let &DisplayItem::LayeredItemClass(ref layered_item) = item {
            // We need to finalize the last layer here before incrementing the item
            // index, otherwise this item will be placed into the parent layer.
            self.finalize_current_layer();
            let layer = PaintLayer::new_for_display_item(
                &layered_item.layer_info,
                &layered_item.item.bounds(),
                parent_origin,
                transform,
                perspective,
                self.current_parent_layer_id(),
                self.current_parent_stacking_context_id(),
                self.current_item_index);
            self.layers.push(layer);
            self.current_item_index += 1;
            return;
        }

        // If we don't have a current layer, we are an item that belonged to a
        // previous layer that was finalized by a child layer. We need to
        // resurrect a copy of the original ancestor layer to ensure that this
        // item is ordered on top of the child layers when painted.
        if self.current_layer.is_none() {
            let mut new_layer = self.layer_details_stack.pop().unwrap();
            new_layer.make_companion_layer();

            if new_layer.layer_properties.parent_id == None {
                new_layer.layer_properties.parent_id =
                    Some(new_layer.layer_properties.id.original());
            }

            self.layer_details_stack.push(new_layer.clone());
            self.current_layer = Some(new_layer);
        }

        if let Some(ref mut current_layer) = self.current_layer {
            current_layer.add_item(self.current_item_index);
        }
        self.current_item_index += 1;
    }
}

pub enum Msg {
    FromLayout(LayoutToPaintMsg),
    FromChrome(ChromeToPaintMsg),
}

#[derive(Deserialize, Serialize)]
pub enum LayoutToPaintMsg {
    PaintInit(Epoch, Arc<DisplayList>),
    Exit,
}

pub struct PaintThread<C> {
    id: PipelineId,
    _url: Url,
    layout_to_paint_port: Receiver<LayoutToPaintMsg>,
    chrome_to_paint_port: Receiver<ChromeToPaintMsg>,
    compositor: C,

    /// A channel to the time profiler.
    time_profiler_chan: time::ProfilerChan,

    /// The root paint layer sent to us by the layout thread.
    root_display_list: Option<Arc<DisplayList>>,

    /// A map that associates LayerIds with their corresponding layers.
    layer_map: HashMap<LayerId, Arc<PaintLayer>>,

    /// Permission to send paint messages to the compositor
    paint_permission: bool,

    /// The current epoch counter is passed by the layout thread
    current_epoch: Option<Epoch>,

    /// Communication handles to each of the worker threads.
    worker_threads: Vec<WorkerThreadProxy>,
}

// If we implement this as a function, we get borrowck errors from borrowing
// the whole PaintThread struct.
macro_rules! native_display(
    ($thread:expr) => (
        $thread.native_display.as_ref().expect("Need a graphics context to do painting")
    )
);

impl<C> PaintThread<C> where C: PaintListener + Send + 'static {
    pub fn create(id: PipelineId,
                  url: Url,
                  chrome_to_paint_chan: Sender<ChromeToPaintMsg>,
                  layout_to_paint_port: Receiver<LayoutToPaintMsg>,
                  chrome_to_paint_port: Receiver<ChromeToPaintMsg>,
                  mut compositor: C,
                  font_cache_thread: FontCacheThread,
                  time_profiler_chan: time::ProfilerChan,
                  mem_profiler_chan: mem::ProfilerChan) {
        thread::spawn_named(format!("PaintThread {:?}", id),
                            move || {
            thread_state::initialize(thread_state::PAINT);
            PipelineId::install(id);

            let native_display = compositor.native_display();
            let worker_threads = WorkerThreadProxy::spawn(native_display,
                                                          font_cache_thread,
                                                          time_profiler_chan.clone());

            let mut paint_thread = PaintThread {
                id: id,
                _url: url,
                layout_to_paint_port: layout_to_paint_port,
                chrome_to_paint_port: chrome_to_paint_port,
                compositor: compositor,
                time_profiler_chan: time_profiler_chan,
                root_display_list: None,
                layer_map: HashMap::new(),
                paint_permission: false,
                current_epoch: None,
                worker_threads: worker_threads,
            };

            let reporter_name = format!("paint-reporter-{}", id);
            mem_profiler_chan.run_with_memory_reporting(|| {
                paint_thread.start();
            }, reporter_name, chrome_to_paint_chan, ChromeToPaintMsg::CollectReports);

            // Tell all the worker threads to shut down.
            for worker_thread in &mut paint_thread.worker_threads {
                worker_thread.exit()
            }
        });
    }

    #[allow(unsafe_code)]
    fn start(&mut self) {
        debug!("PaintThread: beginning painting loop");

        loop {
            let message = {
                let layout_to_paint = &self.layout_to_paint_port;
                let chrome_to_paint = &self.chrome_to_paint_port;
                select! {
                    msg = layout_to_paint.recv() =>
                        Msg::FromLayout(msg.unwrap()),
                    msg = chrome_to_paint.recv() =>
                        Msg::FromChrome(msg.unwrap())
                }
            };

            match message {
                Msg::FromLayout(LayoutToPaintMsg::PaintInit(epoch, display_list)) => {
                    self.current_epoch = Some(epoch);
                    self.root_display_list = Some(display_list);

                    if self.paint_permission {
                        self.initialize_layers();
                    }
                }
                Msg::FromChrome(ChromeToPaintMsg::Paint(requests, frame_tree_id)) => {
                    if self.paint_permission && self.root_display_list.is_some() {
                        let mut replies = Vec::new();
                        for PaintRequest { buffer_requests, scale, layer_id, epoch, layer_kind }
                              in requests {
                            if self.current_epoch == Some(epoch) {
                                self.paint(&mut replies, buffer_requests, scale, layer_id, layer_kind);
                            } else {
                                debug!("PaintThread: Ignoring requests with epoch mismatch: {:?} != {:?}",
                                       self.current_epoch,
                                       epoch);
                                self.compositor.ignore_buffer_requests(buffer_requests);
                            }
                        }

                        debug!("PaintThread: returning surfaces");
                        self.compositor.assign_painted_buffers(self.id,
                                                               self.current_epoch.unwrap(),
                                                               replies,
                                                               frame_tree_id);
                    }
                }
                Msg::FromChrome(ChromeToPaintMsg::PaintPermissionGranted) => {
                    self.paint_permission = true;

                    if self.root_display_list.is_some() {
                        self.initialize_layers();
                    }
                }
                Msg::FromChrome(ChromeToPaintMsg::PaintPermissionRevoked) => {
                    self.paint_permission = false;
                }
                Msg::FromChrome(ChromeToPaintMsg::CollectReports(ref channel)) => {
                    // FIXME(njn): should eventually measure the paint thread.
                    channel.send(Vec::new())
                }
                Msg::FromLayout(LayoutToPaintMsg::Exit) => {
                    // Ask the compositor to remove any layers it is holding for this paint thread.
                    // FIXME(mrobinson): This can probably move back to the constellation now.
                    debug!("PaintThread: Exiting.");
                    self.compositor.notify_paint_thread_exiting(self.id);

                    break;
                }
                Msg::FromChrome(ChromeToPaintMsg::Exit) => {
                    // Ask the compositor to remove any layers it is holding for this paint thread.
                    // FIXME(mrobinson): This can probably move back to the constellation now.
                    debug!("PaintThread: Exiting.");
                    self.compositor.notify_paint_thread_exiting(self.id);

                    break;
                }
            }
        }
    }

    /// Paints one layer and places the painted tiles in `replies`.
    fn paint(&mut self,
              replies: &mut Vec<(LayerId, Box<LayerBufferSet>)>,
              mut tiles: Vec<BufferRequest>,
              scale: f32,
              layer_id: LayerId,
              layer_kind: LayerKind) {
        time::profile(time::ProfilerCategory::Painting, None, self.time_profiler_chan.clone(), || {
            let display_list = match self.root_display_list {
                Some(ref display_list) => display_list.clone(),
                None => return,
            };

            // Bail out if there is no appropriate layer.
            let layer = match self.layer_map.get(&layer_id) {
                Some(layer) => layer.clone(),
                None => return,
            };

            // Divide up the layer into tiles and distribute them to workers via a simple round-
            // robin strategy.
            let tiles = std_mem::replace(&mut tiles, Vec::new());
            let tile_count = tiles.len();
            for (i, tile) in tiles.into_iter().enumerate() {
                let thread_id = i % self.worker_threads.len();
                self.worker_threads[thread_id].paint_tile(thread_id,
                                                          tile,
                                                          display_list.clone(),
                                                          layer.clone(),
                                                          scale,
                                                          layer_kind);
            }
            let new_buffers = (0..tile_count).map(|i| {
                let thread_id = i % self.worker_threads.len();
                self.worker_threads[thread_id].painted_tile_buffer()
            }).collect();

            let layer_buffer_set = box LayerBufferSet {
                buffers: new_buffers,
            };
            replies.push((layer_id, layer_buffer_set));
        })
    }

    fn initialize_layers(&mut self) {
        let root_display_list = match self.root_display_list {
            None => return,
            Some(ref root_display_list) => root_display_list,
        };
        let layers = LayerCreator::create_layers_with_display_list(&root_display_list);
        let properties = layers.iter().map(|layer| layer.layer_properties.clone()).collect();
        self.compositor.initialize_layers_for_pipeline(self.id,
                                                       properties,
                                                       self.current_epoch.unwrap());
        self.layer_map.clear();
        for layer in layers.into_iter() {
            self.layer_map.insert(layer.layer_properties.id, Arc::new(layer));
        }
    }
}

struct WorkerThreadProxy {
    sender: Sender<MsgToWorkerThread>,
    receiver: Receiver<MsgFromWorkerThread>,
}

impl WorkerThreadProxy {
    fn spawn(native_display: Option<NativeDisplay>,
             font_cache_thread: FontCacheThread,
             time_profiler_chan: time::ProfilerChan)
             -> Vec<WorkerThreadProxy> {
        // Don't make any paint threads if we're using WebRender. They're just a waste of
        // resources.
        if opts::get().use_webrender {
            return vec![]
        }

        let thread_count = opts::get().paint_threads;
        (0..thread_count).map(|_| {
            let (from_worker_sender, from_worker_receiver) = channel();
            let (to_worker_sender, to_worker_receiver) = channel();
            let font_cache_thread = font_cache_thread.clone();
            let time_profiler_chan = time_profiler_chan.clone();
            thread::spawn_named("PaintWorker".to_owned(), move || {
                let mut worker_thread = WorkerThread::new(from_worker_sender,
                                                          to_worker_receiver,
                                                          native_display,
                                                          font_cache_thread,
                                                          time_profiler_chan);
                worker_thread.main();
            });
            WorkerThreadProxy {
                receiver: from_worker_receiver,
                sender: to_worker_sender,
            }
        }).collect()
    }

    fn paint_tile(&mut self,
                  thread_id: usize,
                  tile: BufferRequest,
                  display_list: Arc<DisplayList>,
                  paint_layer: Arc<PaintLayer>,
                  scale: f32,
                  layer_kind: LayerKind) {
        let msg = MsgToWorkerThread::PaintTile(thread_id,
                                               tile,
                                               display_list,
                                               paint_layer,
                                               scale,
                                               layer_kind);
        self.sender.send(msg).unwrap()
    }

    fn painted_tile_buffer(&mut self) -> Box<LayerBuffer> {
        match self.receiver.recv().unwrap() {
            MsgFromWorkerThread::PaintedTile(layer_buffer) => layer_buffer,
        }
    }

    fn exit(&mut self) {
        self.sender.send(MsgToWorkerThread::Exit).unwrap()
    }
}

struct WorkerThread {
    sender: Sender<MsgFromWorkerThread>,
    receiver: Receiver<MsgToWorkerThread>,
    native_display: Option<NativeDisplay>,
    font_context: Box<FontContext>,
    time_profiler_sender: time::ProfilerChan,
}

impl WorkerThread {
    fn new(sender: Sender<MsgFromWorkerThread>,
           receiver: Receiver<MsgToWorkerThread>,
           native_display: Option<NativeDisplay>,
           font_cache_thread: FontCacheThread,
           time_profiler_sender: time::ProfilerChan)
           -> WorkerThread {
        WorkerThread {
            sender: sender,
            receiver: receiver,
            native_display: native_display,
            font_context: box FontContext::new(font_cache_thread.clone()),
            time_profiler_sender: time_profiler_sender,
        }
    }

    fn main(&mut self) {
        loop {
            match self.receiver.recv().unwrap() {
                MsgToWorkerThread::Exit => break,
                MsgToWorkerThread::PaintTile(thread_id,
                                             tile,
                                             display_list,
                                             paint_layer,
                                             scale,
                                             layer_kind) => {
                    let buffer = self.optimize_and_paint_tile(thread_id,
                                                              tile,
                                                              display_list,
                                                              paint_layer,
                                                              scale,
                                                              layer_kind);
                    self.sender.send(MsgFromWorkerThread::PaintedTile(buffer)).unwrap()
                }
            }
        }
    }

    fn optimize_and_paint_tile(&mut self,
                               thread_id: usize,
                               mut tile: BufferRequest,
                               display_list: Arc<DisplayList>,
                               paint_layer: Arc<PaintLayer>,
                               scale: f32,
                               layer_kind: LayerKind)
                               -> Box<LayerBuffer> {
        let size = Size2D::new(tile.screen_rect.size.width as i32,
                               tile.screen_rect.size.height as i32);
        let mut buffer = self.create_layer_buffer(&mut tile, scale);
        let draw_target = DrawTarget::new(BackendType::Skia, size, SurfaceFormat::B8G8R8A8);

        {
            // Build the paint context.
            let mut paint_context = PaintContext {
                draw_target: draw_target.clone(),
                font_context: &mut self.font_context,
                page_rect: Rect::from_untyped(&tile.page_rect),
                screen_rect: Rect::from_untyped(&tile.screen_rect),
                clip_rect: None,
                transient_clip: None,
                layer_kind: layer_kind,
            };

            // Apply the translation to paint the tile we want.
            let matrix = Matrix4D::identity();
            let matrix = matrix.scale(scale as AzFloat, scale as AzFloat, 1.0);
            let tile_bounds = tile.page_rect.translate(&paint_layer.display_list_origin);
            let matrix = matrix.translate(-tile_bounds.origin.x as AzFloat,
                                          -tile_bounds.origin.y as AzFloat,
                                          0.0);

            // Clear the buffer.
            paint_context.clear();

            // Draw the display list.
            time::profile(time::ProfilerCategory::PaintingPerTile,
                          None,
                          self.time_profiler_sender.clone(), || {
                              if let Some((start, end)) = paint_layer.display_list_indices {
                                  if paint_layer.single_item {
                                      display_list.draw_item_at_index_into_context(
                                        &mut paint_context, &matrix, start);
                                  } else {
                                      display_list.draw_into_context(
                                          &mut paint_context,
                                          &matrix,
                                          paint_layer.starting_stacking_context_id,
                                          start,
                                          end);
                                  }
                              }
                              paint_context.draw_target.flush();
                         });

            if opts::get().show_debug_parallel_paint {
                // Overlay a transparent solid color to identify the thread that
                // painted this tile.
                let color = THREAD_TINT_COLORS[thread_id % THREAD_TINT_COLORS.len()];
                paint_context.draw_solid_color(&Rect::new(Point2D::new(Au(0), Au(0)),
                                                          Size2D::new(Au::from_px(size.width),
                                                                      Au::from_px(size.height))),
                                               color);
            }
            if opts::get().paint_flashing {
                // Overlay a random transparent color.
                let color = *rand::thread_rng().choose(&THREAD_TINT_COLORS[..]).unwrap();
                paint_context.draw_solid_color(&Rect::new(Point2D::new(Au(0), Au(0)),
                                                          Size2D::new(Au::from_px(size.width),
                                                                      Au::from_px(size.height))),
                                               color);
            }
        }

        // Extract the texture from the draw target and place it into its slot in the buffer.
        // Upload it first.
        draw_target.snapshot().get_data_surface().with_data(|data| {
            buffer.native_surface.upload(native_display!(self), data);
            debug!("painting worker thread uploading to native surface {}",
                   buffer.native_surface.get_id());
        });

        draw_target.finish();
        buffer
    }

    fn create_layer_buffer(&mut self,
                           tile: &mut BufferRequest,
                           scale: f32)
                           -> Box<LayerBuffer> {
        // Create an empty native surface. We mark it as not leaking
        // in case it dies in transit to the compositor thread.
        let width = tile.screen_rect.size.width;
        let height = tile.screen_rect.size.height;
        let mut native_surface = tile.native_surface.take().unwrap_or_else(|| {
            NativeSurface::new(native_display!(self), Size2D::new(width as i32, height as i32))
        });
        native_surface.mark_wont_leak();

        box LayerBuffer {
            native_surface: native_surface,
            rect: tile.page_rect,
            screen_pos: tile.screen_rect,
            resolution: scale,
            painted_with_cpu: true,
            content_age: tile.content_age,
        }
    }
}

enum MsgToWorkerThread {
    Exit,
    PaintTile(usize, BufferRequest, Arc<DisplayList>, Arc<PaintLayer>, f32, LayerKind),
}

enum MsgFromWorkerThread {
    PaintedTile(Box<LayerBuffer>),
}

pub static THREAD_TINT_COLORS: [Color; 8] = [
    Color { r: 6.0 / 255.0, g: 153.0 / 255.0, b: 198.0 / 255.0, a: 0.7 },
    Color { r: 255.0 / 255.0, g: 212.0 / 255.0, b: 83.0 / 255.0, a: 0.7 },
    Color { r: 116.0 / 255.0, g: 29.0 / 255.0, b: 109.0 / 255.0, a: 0.7 },
    Color { r: 204.0 / 255.0, g: 158.0 / 255.0, b: 199.0 / 255.0, a: 0.7 },
    Color { r: 242.0 / 255.0, g: 46.0 / 255.0, b: 121.0 / 255.0, a: 0.7 },
    Color { r: 116.0 / 255.0, g: 203.0 / 255.0, b: 196.0 / 255.0, a: 0.7 },
    Color { r: 255.0 / 255.0, g: 249.0 / 255.0, b: 201.0 / 255.0, a: 0.7 },
    Color { r: 137.0 / 255.0, g: 196.0 / 255.0, b: 78.0 / 255.0, a: 0.7 },
];
