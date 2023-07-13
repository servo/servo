/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use gleam::{gl, gl::Gl};
use std::cell::{Cell, UnsafeCell};
use std::collections::{hash_map::HashMap, VecDeque};
use std::ops::{Deref, DerefMut, Range};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicI8, AtomicIsize, AtomicPtr, AtomicU32, AtomicU8, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread;
use crate::{
    api::units::*, api::ColorDepth, api::ExternalImageId, api::ImageRendering, api::YuvColorSpace, Compositor,
    CompositorCapabilities, CompositorSurfaceTransform, NativeSurfaceId, NativeSurfaceInfo, NativeTileId,
    host_utils::{thread_started, thread_stopped}, MappableCompositor, SWGLCompositeSurfaceInfo,
};

pub struct SwTile {
    x: i32,
    y: i32,
    fbo_id: u32,
    color_id: u32,
    valid_rect: DeviceIntRect,
    /// Composition of tiles must be ordered such that any tiles that may overlap
    /// an invalidated tile in an earlier surface only get drawn after that tile
    /// is actually updated. We store a count of the number of overlapping invalid
    /// here, that gets decremented when the invalid tiles are finally updated so
    /// that we know when it is finally safe to draw. Must use a Cell as we might
    /// be analyzing multiple tiles and surfaces
    overlaps: Cell<u32>,
    /// Whether the tile's contents has been invalidated
    invalid: Cell<bool>,
    /// Graph node for job dependencies of this tile
    graph_node: SwCompositeGraphNodeRef,
}

impl SwTile {
    fn new(x: i32, y: i32) -> Self {
        SwTile {
            x,
            y,
            fbo_id: 0,
            color_id: 0,
            valid_rect: DeviceIntRect::zero(),
            overlaps: Cell::new(0),
            invalid: Cell::new(false),
            graph_node: SwCompositeGraphNode::new(),
        }
    }

    /// The offset of the tile in the local space of the surface before any
    /// transform is applied.
    fn origin(&self, surface: &SwSurface) -> DeviceIntPoint {
        DeviceIntPoint::new(self.x * surface.tile_size.width, self.y * surface.tile_size.height)
    }

    /// The offset valid rect positioned within the local space of the surface
    /// before any transform is applied.
    fn local_bounds(&self, surface: &SwSurface) -> DeviceIntRect {
        self.valid_rect.translate(self.origin(surface).to_vector())
    }

    /// Bounds used for determining overlap dependencies. This may either be the
    /// full tile bounds or the actual valid rect, depending on whether the tile
    /// is invalidated this frame. These bounds are more conservative as such and
    /// may differ from the precise bounds used to actually composite the tile.
    fn overlap_rect(
        &self,
        surface: &SwSurface,
        transform: &CompositorSurfaceTransform,
        clip_rect: &DeviceIntRect,
    ) -> Option<DeviceIntRect> {
        let bounds = self.local_bounds(surface);
        let device_rect = transform.outer_transformed_rect(&bounds.to_f32())?.round_out().to_i32();
        device_rect.intersection(clip_rect)
    }

    /// Determine if the tile's bounds may overlap the dependency rect if it were
    /// to be composited at the given position.
    fn may_overlap(
        &self,
        surface: &SwSurface,
        transform: &CompositorSurfaceTransform,
        clip_rect: &DeviceIntRect,
        dep_rect: &DeviceIntRect,
    ) -> bool {
        self.overlap_rect(surface, transform, clip_rect)
            .map_or(false, |r| r.intersects(dep_rect))
    }

    /// Get valid source and destination rectangles for composition of the tile
    /// within a surface, bounded by the clipping rectangle. May return None if
    /// it falls outside of the clip rect.
    fn composite_rects(
        &self,
        surface: &SwSurface,
        transform: &CompositorSurfaceTransform,
        clip_rect: &DeviceIntRect,
    ) -> Option<(DeviceIntRect, DeviceIntRect, bool)> {
        // Offset the valid rect to the appropriate surface origin.
        let valid = self.local_bounds(surface);
        // The destination rect is the valid rect transformed and then clipped.
        let dest_rect = transform.outer_transformed_rect(&valid.to_f32())?.round_out().to_i32();
        if !dest_rect.intersects(clip_rect) {
            return None;
        }
        // To get a valid source rect, we need to inverse transform the clipped destination rect to find out the effect
        // of the clip rect in source-space. After this, we subtract off the source-space valid rect origin to get
        // a source rect that is now relative to the surface origin rather than absolute.
        let inv_transform = transform.inverse()?;
        let src_rect = inv_transform
            .outer_transformed_rect(&dest_rect.to_f32())?
            .round()
            .to_i32()
            .translate(-valid.origin.to_vector());
        Some((src_rect, dest_rect, transform.m22 < 0.0))
    }
}

pub struct SwSurface {
    tile_size: DeviceIntSize,
    is_opaque: bool,
    tiles: Vec<SwTile>,
    /// An attached external image for this surface.
    external_image: Option<ExternalImageId>,
    /// Descriptor for the external image if successfully locked for composite.
    composite_surface: Option<SWGLCompositeSurfaceInfo>,
}

impl SwSurface {
    fn new(tile_size: DeviceIntSize, is_opaque: bool) -> Self {
        SwSurface {
            tile_size,
            is_opaque,
            tiles: Vec::new(),
            external_image: None,
            composite_surface: None,
        }
    }

    /// Conserative approximation of local bounds of the surface by combining
    /// the local bounds of all enclosed tiles.
    fn local_bounds(&self) -> DeviceIntRect {
        let mut bounds = DeviceIntRect::zero();
        for tile in &self.tiles {
            bounds = bounds.union(&tile.local_bounds(self));
        }
        bounds
    }

    /// The transformed and clipped conservative device-space bounds of the
    /// surface.
    fn device_bounds(
        &self,
        transform: &CompositorSurfaceTransform,
        clip_rect: &DeviceIntRect,
    ) -> Option<DeviceIntRect> {
        let bounds = self.local_bounds();
        let device_rect = transform.outer_transformed_rect(&bounds.to_f32())?.round_out().to_i32();
        device_rect.intersection(clip_rect)
    }
}

fn image_rendering_to_gl_filter(filter: ImageRendering) -> gl::GLenum {
    match filter {
        ImageRendering::Pixelated => gl::NEAREST,
        ImageRendering::Auto | ImageRendering::CrispEdges => gl::LINEAR,
    }
}

/// A source for a composite job which can either be a single BGRA locked SWGL
/// resource or a collection of SWGL resources representing a YUV surface.
#[derive(Clone)]
enum SwCompositeSource {
    BGRA(swgl::LockedResource),
    YUV(
        swgl::LockedResource,
        swgl::LockedResource,
        swgl::LockedResource,
        YuvColorSpace,
        ColorDepth,
    ),
}

/// Mark ExternalImage's renderer field as safe to send to SwComposite thread.
unsafe impl Send for SwCompositeSource {}

/// A tile composition job to be processed by the SwComposite thread.
/// Stores relevant details about the tile and where to composite it.
#[derive(Clone)]
struct SwCompositeJob {
    /// Locked texture that will be unlocked immediately following the job
    locked_src: SwCompositeSource,
    /// Locked framebuffer that may be shared among many jobs
    locked_dst: swgl::LockedResource,
    src_rect: DeviceIntRect,
    dst_rect: DeviceIntRect,
    clipped_dst: DeviceIntRect,
    opaque: bool,
    flip_y: bool,
    filter: ImageRendering,
    /// The total number of bands for this job
    num_bands: u8,
}

impl SwCompositeJob {
    /// Process a composite job
    fn process(&self, band_index: i32) {
        // Bands are allocated in reverse order, but we want to process them in increasing order.
        let num_bands = self.num_bands as i32;
        let band_index = num_bands - 1 - band_index;
        // Calculate the Y extents for the job's band, starting at the current index and spanning to
        // the following index.
        let band_offset = (self.clipped_dst.size.height * band_index) / num_bands;
        let band_height = (self.clipped_dst.size.height * (band_index + 1)) / num_bands - band_offset;
        // Create a rect that is the intersection of the band with the clipped dest
        let band_clip = DeviceIntRect::new(
            DeviceIntPoint::new(self.clipped_dst.origin.x, self.clipped_dst.origin.y + band_offset),
            DeviceIntSize::new(self.clipped_dst.size.width, band_height),
        );
        match self.locked_src {
            SwCompositeSource::BGRA(ref resource) => {
                self.locked_dst.composite(
                    resource,
                    self.src_rect.origin.x,
                    self.src_rect.origin.y,
                    self.src_rect.size.width,
                    self.src_rect.size.height,
                    self.dst_rect.origin.x,
                    self.dst_rect.origin.y,
                    self.dst_rect.size.width,
                    self.dst_rect.size.height,
                    self.opaque,
                    self.flip_y,
                    image_rendering_to_gl_filter(self.filter),
                    band_clip.origin.x,
                    band_clip.origin.y,
                    band_clip.size.width,
                    band_clip.size.height,
                );
            }
            SwCompositeSource::YUV(ref y, ref u, ref v, color_space, color_depth) => {
                let swgl_color_space = match color_space {
                    YuvColorSpace::Rec601 => swgl::YUVColorSpace::Rec601,
                    YuvColorSpace::Rec709 => swgl::YUVColorSpace::Rec709,
                    YuvColorSpace::Rec2020 => swgl::YUVColorSpace::Rec2020,
                    YuvColorSpace::Identity => swgl::YUVColorSpace::Identity,
                };
                self.locked_dst.composite_yuv(
                    y,
                    u,
                    v,
                    swgl_color_space,
                    color_depth.bit_depth(),
                    self.src_rect.origin.x,
                    self.src_rect.origin.y,
                    self.src_rect.size.width,
                    self.src_rect.size.height,
                    self.dst_rect.origin.x,
                    self.dst_rect.origin.y,
                    self.dst_rect.size.width,
                    self.dst_rect.size.height,
                    self.flip_y,
                    band_clip.origin.x,
                    band_clip.origin.y,
                    band_clip.size.width,
                    band_clip.size.height,
                );
            }
        }
    }
}

/// A reference to a SwCompositeGraph node that can be passed from the render
/// thread to the SwComposite thread. Consistency of mutation is ensured in
/// SwCompositeGraphNode via use of Atomic operations that prevent more than
/// one thread from mutating SwCompositeGraphNode at once. This avoids using
/// messy and not-thread-safe RefCells or expensive Mutexes inside the graph
/// node and at least signals to the compiler that potentially unsafe coercions
/// are occurring.
#[derive(Clone)]
struct SwCompositeGraphNodeRef(Arc<UnsafeCell<SwCompositeGraphNode>>);

impl SwCompositeGraphNodeRef {
    fn new(graph_node: SwCompositeGraphNode) -> Self {
        SwCompositeGraphNodeRef(Arc::new(UnsafeCell::new(graph_node)))
    }

    fn get(&self) -> &SwCompositeGraphNode {
        unsafe { &*self.0.get() }
    }

    fn get_mut(&self) -> &mut SwCompositeGraphNode {
        unsafe { &mut *self.0.get() }
    }

    fn get_ptr_mut(&self) -> *mut SwCompositeGraphNode {
        self.0.get()
    }
}

unsafe impl Send for SwCompositeGraphNodeRef {}

impl Deref for SwCompositeGraphNodeRef {
    type Target = SwCompositeGraphNode;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl DerefMut for SwCompositeGraphNodeRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

/// Dependency graph of composite jobs to be completed. Keeps a list of child jobs that are dependent on the completion of this job.
/// Also keeps track of the number of parent jobs that this job is dependent upon before it can be processed. Once there are no more
/// in-flight parent jobs that it depends on, the graph node is finally added to the job queue for processing.
struct SwCompositeGraphNode {
    /// Job to be queued for this graph node once ready.
    job: Option<SwCompositeJob>,
    /// The number of remaining bands associated with this job. When this is
    /// non-zero and the node has no more parents left, then the node is being
    /// actively used by the composite thread to process jobs. Once it hits
    /// zero, the owning thread (which brought it to zero) can safely retire
    /// the node as no other thread is using it.
    remaining_bands: AtomicU8,
    /// The number of bands that are available for processing.
    available_bands: AtomicI8,
    /// Count of parents this graph node depends on. While this is non-zero the
    /// node must ensure that it is only being actively mutated by the render
    /// thread and otherwise never being accessed by the render thread.
    parents: AtomicU32,
    /// Graph nodes of child jobs that are dependent on this job
    children: Vec<SwCompositeGraphNodeRef>,
}

unsafe impl Sync for SwCompositeGraphNode {}

impl SwCompositeGraphNode {
    fn new() -> SwCompositeGraphNodeRef {
        SwCompositeGraphNodeRef::new(SwCompositeGraphNode {
            job: None,
            remaining_bands: AtomicU8::new(0),
            available_bands: AtomicI8::new(0),
            parents: AtomicU32::new(0),
            children: Vec::new(),
        })
    }

    /// Reset the node's state for a new frame
    fn reset(&mut self) {
        self.job = None;
        self.remaining_bands.store(0, Ordering::SeqCst);
        self.available_bands.store(0, Ordering::SeqCst);
        // Initialize parents to 1 as sentinel dependency for uninitialized job
        // to avoid queuing unitialized job as unblocked child dependency.
        self.parents.store(1, Ordering::SeqCst);
        self.children.clear();
    }

    /// Add a dependent child node to dependency list. Update its parent count.
    fn add_child(&mut self, child: SwCompositeGraphNodeRef) {
        child.parents.fetch_add(1, Ordering::SeqCst);
        self.children.push(child);
    }

    /// Install a job for this node. Return whether or not the job has any unprocessed parents
    /// that would block immediate composition.
    fn set_job(&mut self, job: SwCompositeJob, num_bands: u8) -> bool {
        self.job = Some(job);
        self.remaining_bands.store(num_bands, Ordering::SeqCst);
        self.available_bands.store(num_bands as _, Ordering::SeqCst);
        // Subtract off the sentinel parent dependency now that job is initialized and check
        // whether there are any remaining parent dependencies to see if this job is ready.
        self.parents.fetch_sub(1, Ordering::SeqCst) <= 1
    }

    /// Take an available band if possible. Also return whether there are no more bands left
    /// so the caller may properly clean up after.
    fn take_band(&self) -> (Option<i32>, bool) {
        let available = self.available_bands.fetch_sub(1, Ordering::SeqCst);
        if available > 0 {
            (Some(available as i32 - 1), available == 1)
        } else {
            (None, true)
        }
    }

    /// Try to take the job from this node for processing and then process it within the current band.
    fn process_job(&self, band_index: i32) {
        if let Some(ref job) = self.job {
            job.process(band_index);
        }
    }

    /// After processing a band, check all child dependencies and remove this parent from
    /// their dependency counts. If applicable, queue the new child bands for composition.
    fn unblock_children(&mut self, thread: &SwCompositeThread) {
        if self.remaining_bands.fetch_sub(1, Ordering::SeqCst) > 1 {
            return;
        }
        // Clear the job to release any locked resources.
        self.job = None;
        let mut lock = None;
        for child in self.children.drain(..) {
            // Remove the child's parent dependency on this node. If there are no more
            // parent dependencies left, send the child job bands for composition.
            if child.parents.fetch_sub(1, Ordering::SeqCst) <= 1 {
                if lock.is_none() {
                    lock = Some(thread.lock());
                }
                thread.send_job(lock.as_mut().unwrap(), child);
            }
        }
    }
}

/// The SwComposite thread processes a queue of composite jobs, also signaling
/// via a condition when all available jobs have been processed, as tracked by
/// the job count.
struct SwCompositeThread {
    /// Queue of available composite jobs
    jobs: Mutex<SwCompositeJobQueue>,
    /// Cache of the current job being processed. This maintains a pointer to
    /// the contents of the SwCompositeGraphNodeRef, which is safe due to the
    /// fact that SwCompositor maintains a strong reference to the contents
    /// in an SwTile to keep it alive while this is in use.
    current_job: AtomicPtr<SwCompositeGraphNode>,
    /// Count of unprocessed jobs still in the queue
    job_count: AtomicIsize,
    /// Condition signaled when either there are jobs available to process or
    /// there are no more jobs left to process. Otherwise stated, this signals
    /// when the job queue transitions from an empty to non-empty state or from
    /// a non-empty to empty state.
    jobs_available: Condvar,
    /// Whether all available jobs have been processed.
    jobs_completed: AtomicBool,
}

/// The SwCompositeThread struct is shared between the SwComposite thread
/// and the rendering thread so that both ends can access the job queue.
unsafe impl Sync for SwCompositeThread {}

/// A FIFO queue of composite jobs to be processed.
type SwCompositeJobQueue = VecDeque<SwCompositeGraphNodeRef>;

/// Locked access to the composite job queue.
type SwCompositeThreadLock<'a> = MutexGuard<'a, SwCompositeJobQueue>;

impl SwCompositeThread {
    /// Create the SwComposite thread. Requires a SWGL context in which
    /// to do the composition.
    fn new() -> Arc<SwCompositeThread> {
        let info = Arc::new(SwCompositeThread {
            jobs: Mutex::new(SwCompositeJobQueue::new()),
            current_job: AtomicPtr::new(ptr::null_mut()),
            job_count: AtomicIsize::new(0),
            jobs_available: Condvar::new(),
            jobs_completed: AtomicBool::new(false),
        });
        let result = info.clone();
        let thread_name = "SwComposite";
        thread::Builder::new()
            .name(thread_name.into())
            // The composite thread only calls into SWGL to composite, and we
            // have potentially many composite threads for different windows,
            // so using the default stack size is excessive. A reasonably small
            // stack size should be more than enough for SWGL and reduce memory
            // overhead.
            .stack_size(32 * 1024)
            .spawn(move || {
                thread_started(thread_name);
                // Process any available jobs. This will return a non-Ok
                // result when the job queue is dropped, causing the thread
                // to eventually exit.
                while let Some((job, band)) = info.take_job(true) {
                    info.process_job(job, band);
                }
                thread_stopped();
            })
            .expect("Failed creating SwComposite thread");
        result
    }

    fn deinit(&self) {
        // Force the job count to be negative to signal the thread needs to exit.
        self.job_count.store(isize::MIN / 2, Ordering::SeqCst);
        // Wake up the thread in case it is blocked waiting for new jobs
        self.jobs_available.notify_all();
    }

    /// Process a job contained in a dependency graph node received from the job queue.
    /// Any child dependencies will be unblocked as appropriate after processing. The
    /// job count will be updated to reflect this.
    fn process_job(&self, graph_node: &mut SwCompositeGraphNode, band: i32) {
        // Do the actual processing of the job contained in this node.
        graph_node.process_job(band);
        // Unblock any child dependencies now that this job has been processed.
        graph_node.unblock_children(self);
        // Decrement the job count.
        self.job_count.fetch_sub(1, Ordering::SeqCst);
    }

    /// Queue a tile for composition by adding to the queue and increasing the job count.
    fn queue_composite(
        &self,
        locked_src: SwCompositeSource,
        locked_dst: swgl::LockedResource,
        src_rect: DeviceIntRect,
        dst_rect: DeviceIntRect,
        clip_rect: DeviceIntRect,
        opaque: bool,
        flip_y: bool,
        filter: ImageRendering,
        mut graph_node: SwCompositeGraphNodeRef,
        job_queue: &mut SwCompositeJobQueue,
    ) {
        // For jobs that would span a sufficiently large destination rectangle, split
        // it into multiple horizontal bands so that multiple threads can process them.
        let clipped_dst = match dst_rect.intersection(&clip_rect) {
            Some(clipped_dst) => clipped_dst,
            None => return,
        };

        let num_bands = if clipped_dst.size.width >= 64 && clipped_dst.size.height >= 64 {
            (clipped_dst.size.height / 64).min(4) as u8
        } else {
            1
        };
        let job = SwCompositeJob {
            locked_src,
            locked_dst,
            src_rect,
            dst_rect,
            clipped_dst,
            opaque,
            flip_y,
            filter,
            num_bands,
        };
        self.job_count.fetch_add(num_bands as isize, Ordering::SeqCst);
        if graph_node.set_job(job, num_bands) {
            self.send_job(job_queue, graph_node);
        }
    }

    fn prepare_for_composites(&self) {
        // Initialize the job count to 1 to prevent spurious signaling of job completion
        // in the middle of queuing compositing jobs until we're actually waiting for
        // composition.
        self.job_count.store(1, Ordering::SeqCst);
    }

    /// Lock the thread for access to the job queue.
    fn lock(&self) -> SwCompositeThreadLock {
        self.jobs.lock().unwrap()
    }

    /// Send a job to the composite thread by adding it to the job queue.
    /// Signal that this job has been added in case the queue was empty and the
    /// SwComposite thread is waiting for jobs.
    fn send_job(&self, queue: &mut SwCompositeJobQueue, job: SwCompositeGraphNodeRef) {
        if queue.is_empty() {
            self.jobs_completed.store(false, Ordering::SeqCst);
            self.jobs_available.notify_all();
        }
        queue.push_back(job);
    }

    /// Try to get a band of work from the currently cached job when available.
    /// If there is a job, but it has no available bands left, null out the job
    /// so that other threads do not bother checking the job.
    fn try_take_job(&self) -> Option<(&mut SwCompositeGraphNode, i32)> {
        let current_job_ptr = self.current_job.load(Ordering::SeqCst);
        if let Some(current_job) = unsafe { current_job_ptr.as_mut() } {
            let (band, done) = current_job.take_band();
            if done {
                let _ = self.current_job.compare_exchange(
                    current_job_ptr,
                    ptr::null_mut(),
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                );
            }
            if let Some(band) = band {
                return Some((current_job, band));
            }
        }
        return None;
    }

    /// Take a job from the queue. Optionally block waiting for jobs to become
    /// available if this is called from the SwComposite thread.
    fn take_job(&self, wait: bool) -> Option<(&mut SwCompositeGraphNode, i32)> {
        // First try checking the cached job outside the scope of the mutex.
        // For jobs that have multiple bands, this allows us to avoid having
        // to lock the mutex multiple times to check the job for each band.
        if let Some((job, band)) = self.try_take_job() {
            return Some((job, band));
        }
        // Lock the job queue while checking for available jobs. The lock
        // won't be held while the job is processed later outside of this
        // function so that other threads can pull from the queue meanwhile.
        let mut jobs = self.lock();
        loop {
            // While inside the mutex, check the cached job again to see if it
            // has been updated.
            if let Some((job, band)) = self.try_take_job() {
                return Some((job, band));
            }
            // If no cached job was available, try to take a job from the queue
            // and install it as the current job.
            if let Some(job) = jobs.pop_front() {
                self.current_job.store(job.get_ptr_mut(), Ordering::SeqCst);
                continue;
            }
            // Otherwise, the job queue is currently empty. Depending on the
            // value of the job count we may either wait for jobs to become
            // available or exit.
            if wait {
                self.jobs_completed.store(true, Ordering::SeqCst);
            }
            match self.job_count.load(Ordering::SeqCst) {
                // If we completed all available jobs, signal completion. If
                // waiting inside the SwCompositeThread, then block waiting for
                // more jobs to become available in a new frame. Otherwise,
                // return immediately.
                0 => {
                    self.jobs_available.notify_all();
                    if !wait {
                        return None;
                    }
                }
                // A negative job count signals to exit immediately.
                job_count if job_count < 0 => return None,
                _ => {}
            }
            // The SwCompositeThread needs to wait for jobs to become
            // available to avoid busy waiting on the queue.
            jobs = self.jobs_available.wait(jobs).unwrap();
        }
    }

    /// Wait for all queued composition jobs to be processed.
    /// Instead of blocking on the SwComposite thread to complete all jobs,
    /// this may steal some jobs and attempt to process them while waiting.
    /// This may optionally process jobs synchronously. When normally doing
    /// asynchronous processing, the graph dependencies are relied upon to
    /// properly order the jobs, which makes it safe for the render thread
    /// to steal jobs from the composite thread without violating those
    /// dependencies. Synchronous processing just disables this job stealing
    /// so that the composite thread always handles the jobs in the order
    /// they were queued without having to rely upon possibly unavailable
    /// graph dependencies.
    fn wait_for_composites(&self, sync: bool) {
        // Subtract off the bias to signal we're now waiting on composition and
        // need to know if jobs are completed.
        self.job_count.fetch_sub(1, Ordering::SeqCst);
        // If processing asynchronously, try to steal jobs from the composite
        // thread if it is busy.
        if !sync {
            while let Some((job, band)) = self.take_job(false) {
                self.process_job(job, band);
            }
            // Once there are no more jobs, just fall through to waiting
            // synchronously for the composite thread to finish processing.
        }
        // If processing synchronously, just wait for the composite thread
        // to complete processing any in-flight jobs, then bail.
        let mut jobs = self.lock();
        // If the job count is non-zero here, then there are in-flight jobs.
        while !self.jobs_completed.load(Ordering::SeqCst) {
            jobs = self.jobs_available.wait(jobs).unwrap();
        }
    }

    /// Check if there is a non-zero job count (including sentinel job) that
    /// would indicate we are starting to already process jobs in the composite
    /// thread.
    fn is_busy_compositing(&self) -> bool {
        self.job_count.load(Ordering::SeqCst) > 0
    }
}

/// Parameters describing how to composite a surface within a frame
type FrameSurface = (
    NativeSurfaceId,
    CompositorSurfaceTransform,
    DeviceIntRect,
    ImageRendering,
);

/// Adapter for RenderCompositors to work with SWGL that shuttles between
/// WebRender and the RenderCompositr via the Compositor API.
pub struct SwCompositor {
    gl: swgl::Context,
    compositor: Box<dyn MappableCompositor>,
    use_native_compositor: bool,
    surfaces: HashMap<NativeSurfaceId, SwSurface>,
    frame_surfaces: Vec<FrameSurface>,
    /// Any surface added after we're already compositing (i.e. debug overlay)
    /// needs to be processed after those frame surfaces. For simplicity we
    /// store them in a separate queue that gets processed later.
    late_surfaces: Vec<FrameSurface>,
    cur_tile: NativeTileId,
    /// The maximum tile size required for any of the allocated surfaces.
    max_tile_size: DeviceIntSize,
    /// Reuse the same depth texture amongst all tiles in all surfaces.
    /// This depth texture must be big enough to accommodate the largest used
    /// tile size for any surface. The maximum requested tile size is tracked
    /// to ensure that this depth texture is at least that big.
    depth_id: u32,
    /// Instance of the SwComposite thread, only created if we are not relying
    /// on a native RenderCompositor.
    composite_thread: Option<Arc<SwCompositeThread>>,
    /// SWGL locked resource for sharing framebuffer with SwComposite thread
    locked_framebuffer: Option<swgl::LockedResource>,
}

impl SwCompositor {
    pub fn new(
        gl: swgl::Context,
        compositor: Box<dyn MappableCompositor>,
        use_native_compositor: bool,
    ) -> Self {
        let depth_id = gl.gen_textures(1)[0];
        // Only create the SwComposite thread if we're not using a native render
        // compositor. Thus, we are compositing into the main software framebuffer,
        // which benefits from compositing asynchronously while updating tiles.
        let composite_thread = if !use_native_compositor {
            Some(SwCompositeThread::new())
        } else {
            None
        };
        SwCompositor {
            gl,
            compositor,
            use_native_compositor,
            surfaces: HashMap::new(),
            frame_surfaces: Vec::new(),
            late_surfaces: Vec::new(),
            cur_tile: NativeTileId {
                surface_id: NativeSurfaceId(0),
                x: 0,
                y: 0,
            },
            max_tile_size: DeviceIntSize::zero(),
            depth_id,
            composite_thread,
            locked_framebuffer: None,
        }
    }

    fn deinit_tile(&self, tile: &SwTile) {
        self.gl.delete_framebuffers(&[tile.fbo_id]);
        self.gl.delete_textures(&[tile.color_id]);
    }

    fn deinit_surface(&self, surface: &SwSurface) {
        for tile in &surface.tiles {
            self.deinit_tile(tile);
        }
    }

    /// Attempt to occlude any queued surfaces with an opaque occluder rect. If
    /// an existing surface is occluded, we attempt to restrict its clip rect
    /// so long as it can remain a single clip rect. Existing frame surfaces
    /// that are opaque will be fused if possible with the supplied occluder
    /// rect to further try and restrict any underlying surfaces.
    fn occlude_surfaces(&mut self) {
        // Check if inner rect is fully included in outer rect
        fn includes(outer: &Range<i32>, inner: &Range<i32>) -> bool {
            outer.start <= inner.start && outer.end >= inner.end
        }

        // Check if outer range overlaps either the start or end of a range. If
        // there is overlap, return the portion of the inner range remaining
        // after the overlap has been removed.
        fn overlaps(outer: &Range<i32>, inner: &Range<i32>) -> Option<Range<i32>> {
            if outer.start <= inner.start && outer.end >= inner.start {
                Some(outer.end..inner.end.max(outer.end))
            } else if outer.start <= inner.end && outer.end >= inner.end {
                Some(inner.start..outer.start.max(inner.start))
            } else {
                None
            }
        }

        fn set_x_range(rect: &mut DeviceIntRect, range: &Range<i32>) {
            rect.origin.x = range.start;
            rect.size.width = range.end - range.start;
        }

        fn set_y_range(rect: &mut DeviceIntRect, range: &Range<i32>) {
            rect.origin.y = range.start;
            rect.size.height = range.end - range.start;
        }

        fn union(base: Range<i32>, extra: Range<i32>) -> Range<i32> {
            base.start.min(extra.start)..base.end.max(extra.end)
        }

        // Before we can try to occlude any surfaces, we need to fix their clip rects to tightly
        // bound the valid region. The clip rect might otherwise enclose an invalid area that
        // can't fully occlude anything even if the surface is opaque.
        for &mut (ref id, ref transform, ref mut clip_rect, _) in &mut self.frame_surfaces {
            if let Some(surface) = self.surfaces.get(id) {
                // Restrict the clip rect to fall within the valid region of the surface.
                *clip_rect = surface.device_bounds(transform, clip_rect).unwrap_or_default();
            }
        }

        // For each frame surface, treat it as an occluder if it is non-empty and opaque. Look
        // through the preceding surfaces to see if any can be occluded.
        for occlude_index in 0..self.frame_surfaces.len() {
            let (ref occlude_id, _, ref occlude_rect, _) = self.frame_surfaces[occlude_index];
            match self.surfaces.get(occlude_id) {
                Some(occluder) if occluder.is_opaque && !occlude_rect.is_empty() => {}
                _ => continue,
            }

            // Traverse the queued surfaces for this frame in the reverse order of
            // how they are composited, or rather, in order of visibility. For each
            // surface, check if the occluder can restrict the clip rect such that
            // the clip rect can remain a single rect. If the clip rect overlaps
            // the occluder on one axis interval while remaining fully included in
            // the occluder's other axis interval, then we can chop down the edge
            // of the clip rect on the overlapped axis. Further, if the surface is
            // opaque and its clip rect exactly matches the occluder rect on one
            // axis interval while overlapping on the other, fuse it with the
            // occluder rect before considering any underlying surfaces.
            let (mut occlude_x, mut occlude_y) = (occlude_rect.x_range(), occlude_rect.y_range());
            for &mut (ref id, _, ref mut clip_rect, _) in self.frame_surfaces[..occlude_index].iter_mut().rev() {
                if let Some(surface) = self.surfaces.get(id) {
                    let (clip_x, clip_y) = (clip_rect.x_range(), clip_rect.y_range());
                    if includes(&occlude_x, &clip_x) {
                        if let Some(visible) = overlaps(&occlude_y, &clip_y) {
                            set_y_range(clip_rect, &visible);
                            if surface.is_opaque && occlude_x == clip_x {
                                occlude_y = union(occlude_y, visible);
                            }
                        }
                    } else if includes(&occlude_y, &clip_y) {
                        if let Some(visible) = overlaps(&occlude_x, &clip_x) {
                            set_x_range(clip_rect, &visible);
                            if surface.is_opaque && occlude_y == clip_y {
                                occlude_x = union(occlude_x, visible);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Reset tile dependency state for a new frame.
    fn reset_overlaps(&mut self) {
        for surface in self.surfaces.values_mut() {
            for tile in &mut surface.tiles {
                tile.overlaps.set(0);
                tile.invalid.set(false);
                tile.graph_node.reset();
            }
        }
    }

    /// Computes an overlap count for a tile that falls within the given composite
    /// destination rectangle. This requires checking all surfaces currently queued for
    /// composition so far in this frame and seeing if they have any invalidated tiles
    /// whose destination rectangles would also overlap the supplied tile. If so, then the
    /// increment the overlap count to account for all such dependencies on invalid tiles.
    /// Tiles with the same overlap count will still be drawn with a stable ordering in
    /// the order the surfaces were queued, so it is safe to ignore other possible sources
    /// of composition ordering dependencies, as the later queued tile will still be drawn
    /// later than the blocking tiles within that stable order. We assume that the tile's
    /// surface hasn't yet been added to the current frame list of surfaces to composite
    /// so that we only process potential blockers from surfaces that would come earlier
    /// in composition.
    fn init_overlaps(
        &self,
        overlap_id: &NativeSurfaceId,
        overlap_surface: &SwSurface,
        overlap_tile: &SwTile,
        overlap_transform: &CompositorSurfaceTransform,
        overlap_clip_rect: &DeviceIntRect,
    ) {
        // Record an extra overlap for an invalid tile to track the tile's dependency
        // on its own future update.
        let mut overlaps = if overlap_tile.invalid.get() { 1 } else { 0 };

        let overlap_rect = match overlap_tile.overlap_rect(overlap_surface, overlap_transform, overlap_clip_rect) {
            Some(overlap_rect) => overlap_rect,
            None => {
                overlap_tile.overlaps.set(overlaps);
                return;
            }
        };

        for &(ref id, ref transform, ref clip_rect, _) in &self.frame_surfaces {
            // We only want to consider surfaces that were added before the current one we're
            // checking for overlaps. If we find that surface, then we're done.
            if id == overlap_id {
                break;
            }
            // If the surface's clip rect doesn't overlap the tile's rect,
            // then there is no need to check any tiles within the surface.
            if !overlap_rect.intersects(clip_rect) {
                continue;
            }
            if let Some(surface) = self.surfaces.get(id) {
                for tile in &surface.tiles {
                    // If there is a deferred tile that might overlap the destination rectangle,
                    // record the overlap.
                    if tile.may_overlap(surface, transform, clip_rect, &overlap_rect) {
                        if tile.overlaps.get() > 0 {
                            overlaps += 1;
                        }
                        // Regardless of whether this tile is deferred, if it has dependency
                        // overlaps, then record that it is potentially a dependency parent.
                        tile.graph_node.get_mut().add_child(overlap_tile.graph_node.clone());
                    }
                }
            }
        }
        if overlaps > 0 {
            // Has a dependency on some invalid tiles, so need to defer composition.
            overlap_tile.overlaps.set(overlaps);
        }
    }

    /// Helper function that queues a composite job to the current locked framebuffer
    fn queue_composite(
        &self,
        surface: &SwSurface,
        transform: &CompositorSurfaceTransform,
        clip_rect: &DeviceIntRect,
        filter: ImageRendering,
        tile: &SwTile,
        job_queue: &mut SwCompositeJobQueue,
    ) {
        if let Some(ref composite_thread) = self.composite_thread {
            if let Some((src_rect, dst_rect, flip_y)) = tile.composite_rects(surface, transform, clip_rect) {
                let source = if surface.external_image.is_some() {
                    // If the surface has an attached external image, lock any textures supplied in the descriptor.
                    match surface.composite_surface {
                        Some(ref info) => match info.yuv_planes {
                            0 => match self.gl.lock_texture(info.textures[0]) {
                                Some(texture) => SwCompositeSource::BGRA(texture),
                                None => return,
                            },
                            3 => match (
                                self.gl.lock_texture(info.textures[0]),
                                self.gl.lock_texture(info.textures[1]),
                                self.gl.lock_texture(info.textures[2]),
                            ) {
                                (Some(y_texture), Some(u_texture), Some(v_texture)) => SwCompositeSource::YUV(
                                    y_texture,
                                    u_texture,
                                    v_texture,
                                    info.color_space,
                                    info.color_depth,
                                ),
                                _ => return,
                            },
                            _ => panic!("unsupported number of YUV planes: {}", info.yuv_planes),
                        },
                        None => return,
                    }
                } else if let Some(texture) = self.gl.lock_texture(tile.color_id) {
                    // Lock the texture representing the picture cache tile.
                    SwCompositeSource::BGRA(texture)
                } else {
                    return;
                };
                if let Some(ref framebuffer) = self.locked_framebuffer {
                    composite_thread.queue_composite(
                        source,
                        framebuffer.clone(),
                        src_rect,
                        dst_rect,
                        *clip_rect,
                        surface.is_opaque,
                        flip_y,
                        filter,
                        tile.graph_node.clone(),
                        job_queue,
                    );
                }
            }
        }
    }

    /// Lock a surface with an attached external image for compositing.
    fn try_lock_composite_surface(&mut self, id: &NativeSurfaceId) {
        if let Some(surface) = self.surfaces.get_mut(id) {
            if let Some(external_image) = surface.external_image {
                // If the surface has an attached external image, attempt to lock the external image
                // for compositing. Yields a descriptor of textures and data necessary for their
                // interpretation on success.
                let mut info = SWGLCompositeSurfaceInfo {
                    yuv_planes: 0,
                    textures: [0; 3],
                    color_space: YuvColorSpace::Identity,
                    color_depth: ColorDepth::Color8,
                    size: DeviceIntSize::zero(),
                };
                assert!(!surface.tiles.is_empty());
                let mut tile = &mut surface.tiles[0];
                if self.compositor.lock_composite_surface(self.gl.into(), external_image, &mut info) {
                    tile.valid_rect = DeviceIntRect::from_size(info.size);
                    surface.composite_surface = Some(info);
                } else {
                    tile.valid_rect = DeviceIntRect::zero();
                    surface.composite_surface = None;
                }
            }
        }
    }

    /// Look for any attached external images that have been locked and then unlock them.
    fn unlock_composite_surfaces(&mut self) {
        for &(ref id, _, _, _) in self.frame_surfaces.iter().chain(self.late_surfaces.iter()) {
            if let Some(surface) = self.surfaces.get_mut(id) {
                if let Some(external_image) = surface.external_image {
                    if surface.composite_surface.is_some() {
                        self.compositor.unlock_composite_surface(self.gl.into(), external_image);
                        surface.composite_surface = None;
                    }
                }
            }
        }
    }

    /// Issue composites for any tiles that are no longer blocked following a tile update.
    /// We process all surfaces and tiles in the order they were queued.
    fn flush_composites(&self, tile_id: &NativeTileId, surface: &SwSurface, tile: &SwTile) {
        let composite_thread = match &self.composite_thread {
            Some(composite_thread) => composite_thread,
            None => return,
        };

        // Look for the tile in the frame list and composite it if it has no dependencies.
        let mut frame_surfaces = self
            .frame_surfaces
            .iter()
            .skip_while(|&(ref id, _, _, _)| *id != tile_id.surface_id);
        let (overlap_rect, mut lock) = match frame_surfaces.next() {
            Some(&(_, ref transform, ref clip_rect, filter)) => {
                // Remove invalid tile's update dependency.
                if tile.invalid.get() {
                    tile.overlaps.set(tile.overlaps.get() - 1);
                }
                // If the tile still has overlaps, keep deferring it till later.
                if tile.overlaps.get() > 0 {
                    return;
                }
                // Otherwise, the tile's dependencies are all resolved, so composite it.
                let mut lock = composite_thread.lock();
                self.queue_composite(surface, transform, clip_rect, filter, tile, &mut lock);
                // Finally, get the tile's overlap rect used for tracking dependencies
                match tile.overlap_rect(surface, transform, clip_rect) {
                    Some(overlap_rect) => (overlap_rect, lock),
                    None => return,
                }
            }
            None => return,
        };

        // Accumulate rects whose dependencies have been satisfied from this update.
        // Store the union of all these bounds to quickly reject unaffected tiles.
        let mut flushed_bounds = overlap_rect;
        let mut flushed_rects = vec![overlap_rect];

        // Check surfaces following the update in the frame list and see if they would overlap it.
        for &(ref id, ref transform, ref clip_rect, filter) in frame_surfaces {
            // If the clip rect doesn't overlap the conservative bounds, we can skip the whole surface.
            if !flushed_bounds.intersects(clip_rect) {
                continue;
            }
            if let Some(surface) = self.surfaces.get(&id) {
                // Search through the surface's tiles for any blocked on this update and queue jobs for them.
                for tile in &surface.tiles {
                    let mut overlaps = tile.overlaps.get();
                    // Only check tiles that have existing unresolved dependencies
                    if overlaps == 0 {
                        continue;
                    }
                    // Get this tile's overlap rect for tracking dependencies
                    let overlap_rect = match tile.overlap_rect(surface, transform, clip_rect) {
                        Some(overlap_rect) => overlap_rect,
                        None => continue,
                    };
                    // Do a quick check to see if the tile overlaps the conservative bounds.
                    if !overlap_rect.intersects(&flushed_bounds) {
                        continue;
                    }
                    // Decrement the overlap count if this tile is dependent on any flushed rects.
                    for flushed_rect in &flushed_rects {
                        if overlap_rect.intersects(flushed_rect) {
                            overlaps -= 1;
                        }
                    }
                    if overlaps != tile.overlaps.get() {
                        // If the overlap count changed, this tile had a dependency on some flush rects.
                        // If the count hit zero, it is ready to composite.
                        tile.overlaps.set(overlaps);
                        if overlaps == 0 {
                            self.queue_composite(surface, transform, clip_rect, filter, tile, &mut lock);
                            // Record that the tile got flushed to update any downwind dependencies.
                            flushed_bounds = flushed_bounds.union(&overlap_rect);
                            flushed_rects.push(overlap_rect);
                        }
                    }
                }
            }
        }
    }
}

impl Compositor for SwCompositor {
    fn create_surface(
        &mut self,
        id: NativeSurfaceId,
        virtual_offset: DeviceIntPoint,
        tile_size: DeviceIntSize,
        is_opaque: bool,
    ) {
        if self.use_native_compositor {
            self.compositor.create_surface(id, virtual_offset, tile_size, is_opaque);
        }
        self.max_tile_size = DeviceIntSize::new(
            self.max_tile_size.width.max(tile_size.width),
            self.max_tile_size.height.max(tile_size.height),
        );
        self.surfaces.insert(id, SwSurface::new(tile_size, is_opaque));
    }

    fn create_external_surface(&mut self, id: NativeSurfaceId, is_opaque: bool) {
        if self.use_native_compositor {
            self.compositor.create_external_surface(id, is_opaque);
        }
        self.surfaces
            .insert(id, SwSurface::new(DeviceIntSize::zero(), is_opaque));
    }

    fn destroy_surface(&mut self, id: NativeSurfaceId) {
        if let Some(surface) = self.surfaces.remove(&id) {
            self.deinit_surface(&surface);
        }
        if self.use_native_compositor {
            self.compositor.destroy_surface(id);
        }
    }

    fn deinit(&mut self) {
        if let Some(ref composite_thread) = self.composite_thread {
            composite_thread.deinit();
        }

        for surface in self.surfaces.values() {
            self.deinit_surface(surface);
        }

        self.gl.delete_textures(&[self.depth_id]);

        if self.use_native_compositor {
            self.compositor.deinit();
        }
    }

    fn create_tile(&mut self, id: NativeTileId) {
        if self.use_native_compositor {
            self.compositor.create_tile(id);
        }
        if let Some(surface) = self.surfaces.get_mut(&id.surface_id) {
            let mut tile = SwTile::new(id.x, id.y);
            tile.color_id = self.gl.gen_textures(1)[0];
            tile.fbo_id = self.gl.gen_framebuffers(1)[0];
            self.gl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, tile.fbo_id);
            self.gl.framebuffer_texture_2d(
                gl::DRAW_FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                tile.color_id,
                0,
            );
            self.gl.framebuffer_texture_2d(
                gl::DRAW_FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                self.depth_id,
                0,
            );
            self.gl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, 0);

            surface.tiles.push(tile);
        }
    }

    fn destroy_tile(&mut self, id: NativeTileId) {
        if let Some(surface) = self.surfaces.get_mut(&id.surface_id) {
            if let Some(idx) = surface.tiles.iter().position(|t| t.x == id.x && t.y == id.y) {
                let tile = surface.tiles.remove(idx);
                self.deinit_tile(&tile);
            }
        }
        if self.use_native_compositor {
            self.compositor.destroy_tile(id);
        }
    }

    fn attach_external_image(&mut self, id: NativeSurfaceId, external_image: ExternalImageId) {
        if self.use_native_compositor {
            self.compositor.attach_external_image(id, external_image);
        }
        if let Some(surface) = self.surfaces.get_mut(&id) {
            // Surfaces with attached external images have a single tile at the origin encompassing
            // the entire surface.
            assert!(surface.tile_size.is_empty());
            surface.external_image = Some(external_image);
            if surface.tiles.is_empty() {
                surface.tiles.push(SwTile::new(0, 0));
            }
        }
    }

    fn invalidate_tile(&mut self, id: NativeTileId, valid_rect: DeviceIntRect) {
        if self.use_native_compositor {
            self.compositor.invalidate_tile(id, valid_rect);
        }
        if let Some(surface) = self.surfaces.get_mut(&id.surface_id) {
            if let Some(tile) = surface.tiles.iter_mut().find(|t| t.x == id.x && t.y == id.y) {
                tile.invalid.set(true);
                tile.valid_rect = valid_rect;
            }
        }
    }

    fn bind(&mut self, id: NativeTileId, dirty_rect: DeviceIntRect, valid_rect: DeviceIntRect) -> NativeSurfaceInfo {
        let mut surface_info = NativeSurfaceInfo {
            origin: DeviceIntPoint::zero(),
            fbo_id: 0,
        };

        self.cur_tile = id;

        if let Some(surface) = self.surfaces.get_mut(&id.surface_id) {
            if let Some(tile) = surface.tiles.iter_mut().find(|t| t.x == id.x && t.y == id.y) {
                assert_eq!(tile.valid_rect, valid_rect);
                if valid_rect.is_empty() {
                    return surface_info;
                }

                let mut stride = 0;
                let mut buf = ptr::null_mut();
                if self.use_native_compositor {
                    if let Some(tile_info) = self.compositor.map_tile(id, dirty_rect, valid_rect) {
                        stride = tile_info.stride;
                        buf = tile_info.data;
                    }
                }
                self.gl.set_texture_buffer(
                    tile.color_id,
                    gl::RGBA8,
                    valid_rect.size.width,
                    valid_rect.size.height,
                    stride,
                    buf,
                    surface.tile_size.width,
                    surface.tile_size.height,
                );
                // Reallocate the shared depth buffer to fit the valid rect, but within
                // a buffer sized to actually fit at least the maximum possible tile size.
                // The maximum tile size is supplied to avoid reallocation by ensuring the
                // allocated buffer is actually big enough to accommodate the largest tile
                // size requested by any used surface, even though supplied valid rect may
                // actually be much smaller than this. This will only force a texture
                // reallocation inside SWGL if the maximum tile size has grown since the
                // last time it was supplied, instead simply reusing the buffer if the max
                // tile size is not bigger than what was previously allocated.
                self.gl.set_texture_buffer(
                    self.depth_id,
                    gl::DEPTH_COMPONENT,
                    valid_rect.size.width,
                    valid_rect.size.height,
                    0,
                    ptr::null_mut(),
                    self.max_tile_size.width,
                    self.max_tile_size.height,
                );
                surface_info.fbo_id = tile.fbo_id;
                surface_info.origin -= valid_rect.origin.to_vector();
            }
        }

        surface_info
    }

    fn unbind(&mut self) {
        let id = self.cur_tile;
        if let Some(surface) = self.surfaces.get(&id.surface_id) {
            if let Some(tile) = surface.tiles.iter().find(|t| t.x == id.x && t.y == id.y) {
                if tile.valid_rect.is_empty() {
                    // If we didn't actually render anything, then just queue any
                    // dependencies.
                    self.flush_composites(&id, surface, tile);
                    return;
                }

                // Force any delayed clears to be resolved.
                self.gl.resolve_framebuffer(tile.fbo_id);

                if self.use_native_compositor {
                    self.compositor.unmap_tile();
                } else {
                    // If we're not relying on a native compositor, then composite
                    // any tiles that are dependent on this tile being updated but
                    // are otherwise ready to composite.
                    self.flush_composites(&id, surface, tile);
                }
            }
        }
    }

    fn begin_frame(&mut self) {
        if self.use_native_compositor {
            self.compositor.begin_frame();
        }
    }

    fn add_surface(
        &mut self,
        id: NativeSurfaceId,
        transform: CompositorSurfaceTransform,
        clip_rect: DeviceIntRect,
        filter: ImageRendering,
    ) {
        if self.use_native_compositor {
            self.compositor.add_surface(id, transform, clip_rect, filter);
        }

        if self.composite_thread.is_some() {
            // If the surface has an attached external image, try to lock that now.
            self.try_lock_composite_surface(&id);

            // If we're already busy compositing, then add to the queue of late
            // surfaces instead of trying to sort into the main frame queue.
            // These late surfaces will not have any overlap tracking done for
            // them and must be processed synchronously at the end of the frame.
            if self.composite_thread.as_ref().unwrap().is_busy_compositing() {
                self.late_surfaces.push((id, transform, clip_rect, filter));
                return;
            }
        }

        self.frame_surfaces.push((id, transform, clip_rect, filter));
    }

    /// Now that all the dependency graph nodes have been built, start queuing
    /// composition jobs. Any surfaces that get added after this point in the
    /// frame will not have overlap dependencies assigned and so must instead
    /// be added to the late_surfaces queue to be processed at the end of the
    /// frame.
    fn start_compositing(&mut self, dirty_rects: &[DeviceIntRect], _opaque_rects: &[DeviceIntRect]) {
        // Opaque rects are currently only computed here, not by WR itself, so we
        // ignore the passed parameter and forward our own version onto the native
        // compositor.
        let mut opaque_rects: Vec<DeviceIntRect> = Vec::new();
        for &(ref id, ref transform, ref clip_rect, _filter) in &self.frame_surfaces {
            if let Some(surface) = self.surfaces.get(id) {
                if !surface.is_opaque {
                    continue;
                }

                for tile in &surface.tiles {
                    if let Some(rect) = tile.overlap_rect(surface, transform, clip_rect) {
                        opaque_rects.push(rect);
                    }
                }
            }
        }

        self.compositor.start_compositing(dirty_rects, &opaque_rects);

        if let Some(dirty_rect) = dirty_rects
            .iter()
            .fold(DeviceIntRect::zero(), |acc, dirty_rect| acc.union(dirty_rect))
            .to_non_empty()
        {
            // Factor dirty rect into surface clip rects
            for &mut (_, _, ref mut clip_rect, _) in &mut self.frame_surfaces {
                *clip_rect = clip_rect.intersection(&dirty_rect).unwrap_or_default();
            }
        }

        self.occlude_surfaces();

        // Discard surfaces that are entirely clipped out
        self.frame_surfaces
            .retain(|&(_, _, clip_rect, _)| !clip_rect.is_empty());

        if let Some(ref composite_thread) = self.composite_thread {
            // Compute overlap dependencies for surfaces.
            for &(ref id, ref transform, ref clip_rect, _filter) in &self.frame_surfaces {
                if let Some(surface) = self.surfaces.get(id) {
                    for tile in &surface.tiles {
                        self.init_overlaps(id, surface, tile, transform, clip_rect);
                    }
                }
            }

            self.locked_framebuffer = self.gl.lock_framebuffer(0);

            composite_thread.prepare_for_composites();

            // Issue any initial composite jobs for the SwComposite thread.
            let mut lock = composite_thread.lock();
            for &(ref id, ref transform, ref clip_rect, filter) in &self.frame_surfaces {
                if let Some(surface) = self.surfaces.get(id) {
                    for tile in &surface.tiles {
                        if tile.overlaps.get() == 0 {
                            // Not dependent on any tiles, so go ahead and composite now.
                            self.queue_composite(surface, transform, clip_rect, filter, tile, &mut lock);
                        }
                    }
                }
            }
        }
    }

    fn end_frame(&mut self) {
        if self.use_native_compositor {
            self.compositor.end_frame();
        } else if let Some(ref composite_thread) = self.composite_thread {
            // Need to wait for the SwComposite thread to finish any queued jobs.
            composite_thread.wait_for_composites(false);

            if !self.late_surfaces.is_empty() {
                // All of the main frame surface have been processed by now. But if there
                // are any late surfaces, we need to kick off a new synchronous composite
                // phase. These late surfaces don't have any overlap/dependency tracking,
                // so we just queue them directly and wait synchronously for the composite
                // thread to process them in order.
                composite_thread.prepare_for_composites();
                {
                    let mut lock = composite_thread.lock();
                    for &(ref id, ref transform, ref clip_rect, filter) in &self.late_surfaces {
                        if let Some(surface) = self.surfaces.get(id) {
                            for tile in &surface.tiles {
                                self.queue_composite(surface, transform, clip_rect, filter, tile, &mut lock);
                            }
                        }
                    }
                }
                composite_thread.wait_for_composites(true);
            }

            self.locked_framebuffer = None;

            self.unlock_composite_surfaces();
        }

        self.frame_surfaces.clear();
        self.late_surfaces.clear();

        self.reset_overlaps();
    }

    fn enable_native_compositor(&mut self, enable: bool) {
        // TODO: The SwComposite thread is not properly instantiated if this is
        // ever actually toggled.
        assert_eq!(self.use_native_compositor, enable);
        self.compositor.enable_native_compositor(enable);
        self.use_native_compositor = enable;
    }

    fn get_capabilities(&self) -> CompositorCapabilities {
        self.compositor.get_capabilities()
    }
}
