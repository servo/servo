/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{ColorF, ColorU};
use crate::debug_render::DebugRenderer;
use crate::device::query::{GpuSampler, GpuTimer, NamedTag};
use euclid::{Point2D, Rect, Size2D, vec2, default};
use crate::internal_types::FastHashMap;
use crate::renderer::{MAX_VERTEX_TEXTURE_WIDTH, wr_has_been_initialized};
use std::collections::vec_deque::VecDeque;
use std::{f32, mem};
use std::ffi::CStr;
use std::ops::Range;
use std::time::Duration;
use time::precise_time_ns;

pub mod expected {
    use std::ops::Range;
    pub const AVG_BACKEND_CPU_TIME: Range<f64> =    0.0..3.0;
    pub const MAX_BACKEND_CPU_TIME: Range<f64> =    0.0..6.0;
    pub const AVG_RENDERER_CPU_TIME: Range<f64> =   0.0..5.0;
    pub const MAX_RENDERER_CPU_TIME: Range<f64> =   0.0..10.0;
    pub const AVG_IPC_TIME: Range<f64> =            0.0..2.0;
    pub const MAX_IPC_TIME: Range<f64> =            0.0..4.0;
    pub const AVG_GPU_TIME: Range<f64> =            0.0..8.0;
    pub const MAX_GPU_TIME: Range<f64> =            0.0..15.0;
    pub const DRAW_CALLS: Range<u64> =              1..100;
    pub const VERTICES: Range<u64> =                10..25_000;
    pub const TOTAL_PRIMITIVES: Range<u64> =        1..5000;
    pub const VISIBLE_PRIMITIVES: Range<u64> =      1..5000;
    pub const USED_TARGETS: Range<u64> =            1..4;
    pub const COLOR_PASSES: Range<u64> =            1..4;
    pub const ALPHA_PASSES: Range<u64> =            0..3;
    pub const RENDERED_PICTURE_CACHE_TILES: Range<u64> = 0..5;
    pub const TOTAL_PICTURE_CACHE_TILES: Range<u64> = 0..15;
    pub const CREATED_TARGETS: Range<u64> =         0..3;
    pub const CHANGED_TARGETS: Range<u64> =         0..3;
    pub const TEXTURE_DATA_UPLOADED: Range<u64> =   0..10;
    pub const GPU_CACHE_ROWS_TOTAL: Range<u64> =    1..50;
    pub const GPU_CACHE_ROWS_UPDATED: Range<u64> =  0..25;
    pub const GPU_CACHE_BLOCKS_TOTAL: Range<u64> =  1..65_000;
    pub const GPU_CACHE_BLOCKS_UPDATED: Range<u64> = 0..1000;
    pub const GPU_CACHE_BLOCKS_SAVED: Range<u64> =  0..50_000;
    pub const DISPLAY_LIST_BUILD_TIME: Range<f64> = 0.0..3.0;
    pub const MAX_SCENE_BUILD_TIME: Range<f64> = 0.0..3.0;
    pub const DISPLAY_LIST_SEND_TIME: Range<f64> =  0.0..1.0;
    pub const DISPLAY_LIST_TOTAL_TIME: Range<f64> = 0.0..4.0;
    pub const NUM_FONT_TEMPLATES: Range<usize> =    0..50;
    pub const FONT_TEMPLATES_MB: Range<f32> =       0.0..40.0;
    pub const NUM_IMAGE_TEMPLATES: Range<usize> =   0..20;
    pub const IMAGE_TEMPLATES_MB: Range<f32> =      0.0..10.0;
    pub const DISPLAY_LIST_MB: Range<f32> =         0.0..0.2;
    pub const NUM_RASTERIZED_BLOBS: Range<usize> =  0..25; // in tiles
    pub const RASTERIZED_BLOBS_MB: Range<f32> =     0.0..4.0;
}

const GRAPH_WIDTH: f32 = 1024.0;
const GRAPH_HEIGHT: f32 = 320.0;
const GRAPH_PADDING: f32 = 8.0;
const GRAPH_FRAME_HEIGHT: f32 = 16.0;
const PROFILE_PADDING: f32 = 8.0;

const ONE_SECOND_NS: u64 = 1000000000;
const AVERAGE_OVER_NS: u64 = ONE_SECOND_NS / 2;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ProfileStyle {
    Full,
    Compact,
    Smart,
}

/// Defines the interface for hooking up an external profiler to WR.
pub trait ProfilerHooks : Send + Sync {
    /// Called at the beginning of a profile scope. The label must
    /// be a C string (null terminated).
    fn begin_marker(&self, label: &CStr);

    /// Called at the end of a profile scope. The label must
    /// be a C string (null terminated).
    fn end_marker(&self, label: &CStr);

    /// Called to mark an event happening. The label must
    /// be a C string (null terminated).
    fn event_marker(&self, label: &CStr);

    /// Called with a duration to indicate a text marker that just ended. Text
    /// markers allow different types of entries to be recorded on the same row
    /// in the timeline, by adding labels to the entry.
    ///
    /// This variant is also useful when the caller only wants to record events
    /// longer than a certain threshold, and thus they don't know in advance
    /// whether the event will qualify.
    fn add_text_marker(&self, label: &CStr, text: &str, duration: Duration);

    /// Returns true if the current thread is being profiled.
    fn thread_is_being_profiled(&self) -> bool;
}

/// The current global profiler callbacks, if set by embedder.
pub static mut PROFILER_HOOKS: Option<&'static dyn ProfilerHooks> = None;

/// Set the profiler callbacks, or None to disable the profiler.
/// This function must only ever be called before any WR instances
/// have been created, or the hooks will not be set.
pub fn set_profiler_hooks(hooks: Option<&'static dyn ProfilerHooks>) {
    if !wr_has_been_initialized() {
        unsafe {
            PROFILER_HOOKS = hooks;
        }
    }
}

/// A simple RAII style struct to manage a profile scope.
pub struct ProfileScope {
    name: &'static CStr,
}

/// Records a marker of the given duration that just ended.
pub fn add_text_marker(label: &CStr, text: &str, duration: Duration) {
    unsafe {
        if let Some(ref hooks) = PROFILER_HOOKS {
            hooks.add_text_marker(label, text, duration);
        }
    }
}

/// Records a marker of the given duration that just ended.
pub fn add_event_marker(label: &CStr) {
    unsafe {
        if let Some(ref hooks) = PROFILER_HOOKS {
            hooks.event_marker(label);
        }
    }
}

/// Returns true if the current thread is being profiled.
pub fn thread_is_being_profiled() -> bool {
    unsafe {
        PROFILER_HOOKS.map_or(false, |h| h.thread_is_being_profiled())
    }
}

impl ProfileScope {
    /// Begin a new profile scope
    pub fn new(name: &'static CStr) -> Self {
        unsafe {
            if let Some(ref hooks) = PROFILER_HOOKS {
                hooks.begin_marker(name);
            }
        }

        ProfileScope {
            name,
        }
    }
}

impl Drop for ProfileScope {
    fn drop(&mut self) {
        unsafe {
            if let Some(ref hooks) = PROFILER_HOOKS {
                hooks.end_marker(self.name);
            }
        }
    }
}

/// A helper macro to define profile scopes.
macro_rules! profile_marker {
    ($string:expr) => {
        let _scope = $crate::profiler::ProfileScope::new(cstr!($string));
    };
}

#[derive(Debug, Clone)]
pub struct GpuProfileTag {
    pub label: &'static str,
    pub color: ColorF,
}

impl NamedTag for GpuProfileTag {
    fn get_label(&self) -> &str {
        self.label
    }
}

trait ProfileCounter {
    fn description(&self) -> &'static str;
    fn value(&self) -> String;
    fn is_expected(&self) -> bool;
}

#[derive(Clone)]
pub struct IntProfileCounter {
    description: &'static str,
    value: usize,
    expect: Option<Range<u64>>,
}

impl IntProfileCounter {
    fn new(description: &'static str, expect: Option<Range<u64>>) -> Self {
        IntProfileCounter {
            description,
            value: 0,
            expect,
        }
    }

    #[inline(always)]
    pub fn inc(&mut self) {
        self.value += 1;
    }

    pub fn set(&mut self, value: usize) {
        self.value = value;
    }
}

impl ProfileCounter for IntProfileCounter {
    fn description(&self) -> &'static str {
        self.description
    }

    fn value(&self) -> String {
        format!("{}", self.value)
    }

    fn is_expected(&self) -> bool {
        self.expect.as_ref().map(|range| range.contains(&(self.value as u64))).unwrap_or(true)
    }
}

/// A profile counter recording average and maximum integer values over time slices
/// of half a second.
#[derive(Clone)]
pub struct AverageIntProfileCounter {
    description: &'static str,
    /// Start of the current time slice.
    start_ns: u64,
    /// Sum of the values recorded during the current time slice.
    sum: u64,
    /// Number of samples in the current time slice.
    num_samples: u64,
    /// The max value in in-progress time slice.
    next_max: u64,
    /// The max value of the previous time slice (displayed).
    max: u64,
    /// The average value of the previous time slice (displayed). 
    avg: u64,
    /// Intermediate accumulator for `add` and `inc`.
    accum: u64,
    /// Expected average range of values, if any.
    expect_avg: Option<Range<u64>>,
    /// Expected maximum range of values, if any.
    expect_max: Option<Range<u64>>,
}

impl AverageIntProfileCounter {
    pub fn new(
        description: &'static str,
        expect_avg: Option<Range<u64>>,
        expect_max: Option<Range<u64>>,
    ) -> Self {
        AverageIntProfileCounter {
            description,
            start_ns: precise_time_ns(),
            sum: 0,
            num_samples: 0,
            next_max: 0,
            max: 0,
            avg: 0,
            accum: 0,
            expect_avg,
            expect_max,
        }
    }

    pub fn reset(&mut self) {
        if self.accum > 0 {
            self.set_u64(self.accum);
            self.accum = 0;
        }
    }

    pub fn set(&mut self, val: usize) {
        self.set_u64(val as u64);
    }

    pub fn set_u64(&mut self, val: u64) {
        let now = precise_time_ns();
        if (now - self.start_ns) > AVERAGE_OVER_NS && self.num_samples > 0 {
            self.avg = self.sum / self.num_samples;
            self.max = self.next_max;
            self.start_ns = now;
            self.sum = 0;
            self.num_samples = 0;
            self.next_max = 0;
        }
        self.next_max = self.next_max.max(val);
        self.sum += val;
        self.num_samples += 1;
        self.accum = 0;
    }

    pub fn add(&mut self, val: usize) {
        self.accum += val as u64;
    }

    pub fn inc(&mut self) {
        self.accum += 1;
    }

    pub fn get_accum(&mut self) -> u64{
        self.accum
    }

    /// Returns either the most up to date value if the counter is updated
    /// with add add inc, or the average over the previous time slice.
    pub fn get(&self) -> usize {
        let result = if self.accum != 0 {
            self.accum
        } else {
            self.avg
        };

        result as usize
    }
}

impl ProfileCounter for AverageIntProfileCounter {
    fn description(&self) -> &'static str {
        self.description
    }

    fn value(&self) -> String {
        format!("{:.2} (max {:.2})", self.avg, self.max)
    }

    fn is_expected(&self) -> bool {
        self.expect_avg.as_ref().map(|range| range.contains(&self.avg)).unwrap_or(true)
            && self.expect_max.as_ref().map(|range| range.contains(&self.max)).unwrap_or(true)
    }
}

pub struct PercentageProfileCounter {
    description: &'static str,
    value: f32,
}

impl ProfileCounter for PercentageProfileCounter {
    fn description(&self) -> &'static str {
        self.description
    }

    fn value(&self) -> String {
        format!("{:.2}%", self.value * 100.0)
    }

    fn is_expected(&self) -> bool { true }
}

#[derive(Clone)]
pub struct ResourceProfileCounter {
    description: &'static str,
    value: usize,
    // in bytes.
    size: usize,
    expected_count: Option<Range<usize>>,
    // in MB
    expected_size: Option<Range<f32>>,
}

impl ResourceProfileCounter {
    fn new(
        description: &'static str,
        expected_count: Option<Range<usize>>,
        expected_size: Option<Range<f32>>
    ) -> Self {
        ResourceProfileCounter {
            description,
            value: 0,
            size: 0,
            expected_count,
            expected_size,
        }
    }

    #[allow(dead_code)]
    fn reset(&mut self) {
        self.value = 0;
        self.size = 0;
    }

    #[inline(always)]
    pub fn inc(&mut self, size: usize) {
        self.value += 1;
        self.size += size;
    }

    pub fn set(&mut self, count: usize, size: usize) {
        self.value = count;
        self.size = size;
    }

    pub fn size_mb(&self) -> f32 {
        self.size as f32 / (1024.0 * 1024.0)
    }
}

impl ProfileCounter for ResourceProfileCounter {
    fn description(&self) -> &'static str {
        self.description
    }

    fn value(&self) -> String {
        format!("{} ({:.2} MB)", self.value, self.size_mb())
    }

    fn is_expected(&self) -> bool {
        self.expected_count.as_ref().map(|range| range.contains(&self.value)).unwrap_or(true)
            && self.expected_size.as_ref().map(|range| range.contains(&self.size_mb())).unwrap_or(true)
    }
}

#[derive(Clone)]
pub struct TimeProfileCounter {
    description: &'static str,
    nanoseconds: u64,
    invert: bool,
    expect_ms: Option<Range<f64>>,
}

pub struct Timer<'a> {
    start: u64,
    result: &'a mut u64,
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        let end = precise_time_ns();
        *self.result += end - self.start;
    }
}

impl TimeProfileCounter {
    pub fn new(description: &'static str, invert: bool, expect_ms: Option<Range<f64>>) -> Self {
        TimeProfileCounter {
            description,
            nanoseconds: 0,
            invert,
            expect_ms,
        }
    }

    fn reset(&mut self) {
        self.nanoseconds = 0;
    }

    #[allow(dead_code)]
    pub fn set(&mut self, ns: u64) {
        self.nanoseconds = ns;
    }

    pub fn profile<T, F>(&mut self, callback: F) -> T
    where
        F: FnOnce() -> T,
    {
        let t0 = precise_time_ns();
        let val = callback();
        let t1 = precise_time_ns();
        let ns = t1 - t0;
        self.nanoseconds += ns;
        val
    }

    pub fn timer(&mut self) -> Timer {
        Timer {
            start: precise_time_ns(),
            result: &mut self.nanoseconds,
        }
    }

    pub fn inc(&mut self, ns: u64) {
        self.nanoseconds += ns;
    }

    pub fn get(&self) -> u64 {
        self.nanoseconds
    }

    pub fn get_ms(&self) -> f64 {
        self.nanoseconds as f64 / 1000000.0
    }
}

impl ProfileCounter for TimeProfileCounter {
    fn description(&self) -> &'static str {
        self.description
    }

    fn value(&self) -> String {
        if self.invert {
            format!("{:.2} fps", 1000000000.0 / self.nanoseconds as f64)
        } else {
            format!("{:.2} ms", self.get_ms())
        }
    }

    fn is_expected(&self) -> bool {
        self.expect_ms.as_ref()
            .map(|range| range.contains(&(self.nanoseconds as f64 / 1000000.0)))
            .unwrap_or(true)
    }
}

#[derive(Clone)]
pub struct AverageTimeProfileCounter {
    counter: AverageIntProfileCounter,
    invert: bool,
}

impl AverageTimeProfileCounter {
    pub fn new(
        description: &'static str,
        invert: bool,
        expect_avg: Option<Range<f64>>,
        expect_max: Option<Range<f64>>,
    ) -> Self {
        let expect_avg_ns = expect_avg.map(
            |range| (range.start * 1000000.0) as u64 .. (range.end * 1000000.0) as u64
        );
        let expect_max_ns = expect_max.map(
            |range| (range.start * 1000000.0) as u64 .. (range.end * 1000000.0) as u64
        );

        AverageTimeProfileCounter {
            counter: AverageIntProfileCounter::new(
                description,
                expect_avg_ns,
                expect_max_ns,
            ),
            invert,
        }
    }

    pub fn set(&mut self, ns: u64) {
        self.counter.set_u64(ns);
    }

    #[allow(dead_code)]
    pub fn profile<T, F>(&mut self, callback: F) -> T
    where
        F: FnOnce() -> T,
    {
        let t0 = precise_time_ns();
        let val = callback();
        let t1 = precise_time_ns();
        self.counter.set_u64(t1 - t0);
        val
    }

    pub fn avg_ms(&self) -> f64 { self.counter.avg as f64 / 1000000.0 }

    pub fn max_ms(&self) -> f64 { self.counter.max as f64 / 1000000.0 }
}

impl ProfileCounter for AverageTimeProfileCounter {
    fn description(&self) -> &'static str {
        self.counter.description
    }

    fn value(&self) -> String {
        if self.invert {
            format!("{:.2} fps", 1000000000.0 / self.counter.avg as f64)
        } else {
            format!("{:.2} ms (max {:.2} ms)", self.avg_ms(), self.max_ms())
        }
    }

    fn is_expected(&self) -> bool {
        self.counter.is_expected()
    }
}


#[derive(Clone)]
pub struct FrameProfileCounters {
    pub total_primitives: AverageIntProfileCounter,
    pub visible_primitives: AverageIntProfileCounter,
    pub targets_used: AverageIntProfileCounter,
    pub targets_changed: AverageIntProfileCounter,
    pub targets_created: AverageIntProfileCounter,
}

impl FrameProfileCounters {
    pub fn new() -> Self {
        FrameProfileCounters {
            total_primitives: AverageIntProfileCounter::new(
                "Total Primitives",
                None, Some(expected::TOTAL_PRIMITIVES),
            ),
            visible_primitives: AverageIntProfileCounter::new(
                "Visible Primitives",
                None, Some(expected::VISIBLE_PRIMITIVES),
            ),
            targets_used: AverageIntProfileCounter::new(
                "Used targets",
                None, Some(expected::USED_TARGETS),
            ),
            targets_changed: AverageIntProfileCounter::new(
                "Changed targets",
                None, Some(expected::CHANGED_TARGETS),
            ),
            targets_created: AverageIntProfileCounter::new(
                "Created targets",
                None, Some(expected::CREATED_TARGETS),
            ),
        }
    }

    pub fn reset_targets(&mut self) {
        self.targets_used.reset();
        self.targets_changed.reset();
        self.targets_created.reset();
    }
}

#[derive(Clone)]
pub struct TextureCacheProfileCounters {
    pub pages_alpha8_linear: ResourceProfileCounter,
    pub pages_alpha16_linear: ResourceProfileCounter,
    pub pages_color8_linear: ResourceProfileCounter,
    pub pages_color8_nearest: ResourceProfileCounter,
    pub pages_picture: ResourceProfileCounter,
    pub rasterized_blob_pixels: ResourceProfileCounter,
    pub standalone_bytes: IntProfileCounter,
    pub shared_bytes: IntProfileCounter,
}

impl TextureCacheProfileCounters {
    pub fn new() -> Self {
        TextureCacheProfileCounters {
            pages_alpha8_linear: ResourceProfileCounter::new("Texture A8 cached pages", None, None),
            pages_alpha16_linear: ResourceProfileCounter::new("Texture A16 cached pages", None, None),
            pages_color8_linear: ResourceProfileCounter::new("Texture RGBA8 cached pages (L)", None, None),
            pages_color8_nearest: ResourceProfileCounter::new("Texture RGBA8 cached pages (N)", None, None),
            pages_picture: ResourceProfileCounter::new("Picture cached pages", None, None),
            rasterized_blob_pixels: ResourceProfileCounter::new(
                "Rasterized Blob Pixels",
                Some(expected::NUM_RASTERIZED_BLOBS),
                Some(expected::RASTERIZED_BLOBS_MB),
            ),
            standalone_bytes: IntProfileCounter::new("Standalone", None),
            shared_bytes: IntProfileCounter::new("Shared", None),
        }
    }
}

#[derive(Clone)]
pub struct GpuCacheProfileCounters {
    pub allocated_rows: AverageIntProfileCounter,
    pub allocated_blocks: AverageIntProfileCounter,
    pub updated_rows: AverageIntProfileCounter,
    pub updated_blocks: AverageIntProfileCounter,
    pub saved_blocks: AverageIntProfileCounter,
}

impl GpuCacheProfileCounters {
    pub fn new() -> Self {
        GpuCacheProfileCounters {
            allocated_rows: AverageIntProfileCounter::new(
                "GPU cache rows: total",
                None, Some(expected::GPU_CACHE_ROWS_TOTAL),
            ),
            updated_rows: AverageIntProfileCounter::new(
                "GPU cache rows: updated",
                None, Some(expected::GPU_CACHE_ROWS_UPDATED),
            ),
            allocated_blocks: AverageIntProfileCounter::new(
                "GPU cache blocks: total",
                None, Some(expected::GPU_CACHE_BLOCKS_TOTAL),
            ),
            updated_blocks: AverageIntProfileCounter::new(
                "GPU cache blocks: updated",
                None, Some(expected::GPU_CACHE_BLOCKS_UPDATED),
            ),
            saved_blocks: AverageIntProfileCounter::new(
                "GPU cache blocks: saved",
                None, Some(expected::GPU_CACHE_BLOCKS_SAVED),
            ),
        }
    }
}

#[derive(Clone)]
pub struct BackendProfileCounters {
    pub total_time: TimeProfileCounter,
    pub resources: ResourceProfileCounters,
    pub txn: TransactionProfileCounters,
    pub intern: InternProfileCounters,
    pub scene_changed: bool,
}

#[derive(Clone)]
pub struct ResourceProfileCounters {
    pub font_templates: ResourceProfileCounter,
    pub image_templates: ResourceProfileCounter,
    pub texture_cache: TextureCacheProfileCounters,
    pub gpu_cache: GpuCacheProfileCounters,
    pub content_slices: IntProfileCounter,
}

#[derive(Clone)]
pub struct TransactionProfileCounters {
    pub display_list_build_time: TimeProfileCounter,
    pub scene_build_time: TimeProfileCounter,
    /// Time between when the display list is built and when it is sent by the API.
    pub content_send_time: TimeProfileCounter,
    /// Time between sending the SetDisplayList from the API and picking it up on
    /// the render scene builder thread.
    pub api_send_time: TimeProfileCounter,
    /// Sum of content_send_time and api_send_time.
    pub total_send_time: TimeProfileCounter,
    pub display_lists: ResourceProfileCounter,
}

macro_rules! declare_intern_profile_counters {
    ( $( $name:ident : $ty:ty, )+ ) => {
        #[derive(Clone)]
        pub struct InternProfileCounters {
            $(
                pub $name: ResourceProfileCounter,
            )+
        }

        impl InternProfileCounters {
            fn draw(
                &self,
                debug_renderer: &mut DebugRenderer,
                draw_state: &mut DrawState,
            ) {
                Profiler::draw_counters(
                    &[
                        $(
                            &self.$name,
                        )+
                    ],
                    None,
                    debug_renderer,
                    false,
                    draw_state,
                );
            }
        }
    }
}

enumerate_interners!(declare_intern_profile_counters);

impl TransactionProfileCounters {
    pub fn set(
        &mut self,
        dl_build_start: u64,
        dl_build_end: u64,
        send_start: u64,
        scene_build_start: u64,
        scene_build_end: u64,
        display_len: usize,
    ) {
        self.display_list_build_time.reset();
        self.content_send_time.reset();
        self.api_send_time.reset();
        self.total_send_time.reset();
        self.scene_build_time.reset();
        self.display_lists.reset();

        let dl_build_time = dl_build_end - dl_build_start;
        let scene_build_time = scene_build_end - scene_build_start;
        let content_send_time = send_start - dl_build_end;
        let api_send_time = scene_build_start - send_start;
        self.display_list_build_time.inc(dl_build_time);
        self.scene_build_time.inc(scene_build_time);
        self.content_send_time.inc(content_send_time);
        self.api_send_time.inc(api_send_time);
        self.total_send_time.inc(content_send_time + api_send_time);
        self.display_lists.inc(display_len);
    }
}

impl BackendProfileCounters {
    pub fn new() -> Self {
        BackendProfileCounters {
            total_time: TimeProfileCounter::new(
                "Backend CPU Time", false,
                Some(expected::MAX_BACKEND_CPU_TIME),
            ),
            resources: ResourceProfileCounters {
                font_templates: ResourceProfileCounter::new(
                    "Font Templates",
                    Some(expected::NUM_FONT_TEMPLATES),
                    Some(expected::FONT_TEMPLATES_MB),
                ),
                image_templates: ResourceProfileCounter::new(
                    "Image Templates",
                    Some(expected::NUM_IMAGE_TEMPLATES),
                    Some(expected::IMAGE_TEMPLATES_MB),
                ),
                content_slices: IntProfileCounter::new(
                    "Content Slices",
                    None,
                ),
                texture_cache: TextureCacheProfileCounters::new(),
                gpu_cache: GpuCacheProfileCounters::new(),
            },
            txn: TransactionProfileCounters {
                display_list_build_time: TimeProfileCounter::new(
                    "DisplayList Build Time", false,
                    Some(expected::DISPLAY_LIST_BUILD_TIME)
                ),
                scene_build_time: TimeProfileCounter::new(
                    "Scene build time", false,
                    Some(expected::MAX_SCENE_BUILD_TIME),
                ),
                content_send_time: TimeProfileCounter::new(
                    "Content Send Time", false,
                    Some(expected::DISPLAY_LIST_SEND_TIME),
                ),
                api_send_time: TimeProfileCounter::new(
                    "API Send Time", false,
                    Some(expected::DISPLAY_LIST_SEND_TIME),
                ),
                total_send_time: TimeProfileCounter::new(
                    "Total IPC Time", false,
                    Some(expected::DISPLAY_LIST_TOTAL_TIME),
                ),
                display_lists: ResourceProfileCounter::new(
                    "DisplayLists Sent",
                    None, Some(expected::DISPLAY_LIST_MB),
                ),
            },
            //TODO: generate this by a macro
            intern: InternProfileCounters {
                prim: ResourceProfileCounter::new("Interned primitives", None, None),
                conic_grad: ResourceProfileCounter::new("Interned conic gradients", None, None),
                image: ResourceProfileCounter::new("Interned images", None, None),
                image_border: ResourceProfileCounter::new("Interned image borders", None, None),
                line_decoration: ResourceProfileCounter::new("Interned line decorations", None, None),
                linear_grad: ResourceProfileCounter::new("Interned linear gradients", None, None),
                normal_border: ResourceProfileCounter::new("Interned normal borders", None, None),
                picture: ResourceProfileCounter::new("Interned pictures", None, None),
                radial_grad: ResourceProfileCounter::new("Interned radial gradients", None, None),
                text_run: ResourceProfileCounter::new("Interned text runs", None, None),
                yuv_image: ResourceProfileCounter::new("Interned YUV images", None, None),
                clip: ResourceProfileCounter::new("Interned clips", None, None),
                filter_data: ResourceProfileCounter::new("Interned filter data", None, None),
                backdrop: ResourceProfileCounter::new("Interned backdrops", None, None),
            },
            scene_changed: false,
        }
    }

    pub fn reset(&mut self) {
        self.total_time.reset();
        self.resources.texture_cache.rasterized_blob_pixels.reset();
        self.scene_changed = false;
    }
}

pub struct RendererProfileCounters {
    pub frame_counter: IntProfileCounter,
    pub frame_time: AverageTimeProfileCounter,
    pub draw_calls: AverageIntProfileCounter,
    pub vertices: AverageIntProfileCounter,
    pub vao_count_and_size: ResourceProfileCounter,
    pub color_passes: AverageIntProfileCounter,
    pub alpha_passes: AverageIntProfileCounter,
    pub texture_data_uploaded: AverageIntProfileCounter,
    pub rendered_picture_cache_tiles: AverageIntProfileCounter,
    pub total_picture_cache_tiles: AverageIntProfileCounter,
}

pub struct RendererProfileTimers {
    pub cpu_time: TimeProfileCounter,
    pub gpu_graph: TimeProfileCounter,
    pub gpu_samples: Vec<GpuTimer<GpuProfileTag>>,
}

impl RendererProfileCounters {
    pub fn new() -> Self {
        RendererProfileCounters {
            frame_counter: IntProfileCounter::new("Frame", None),
            frame_time: AverageTimeProfileCounter::new(
                "FPS", true, None, None,
            ),
            draw_calls: AverageIntProfileCounter::new(
                "Draw Calls",
                None, Some(expected::DRAW_CALLS),
            ),
            vertices: AverageIntProfileCounter::new(
                "Vertices",
                None, Some(expected::VERTICES),
            ),
            vao_count_and_size: ResourceProfileCounter::new("VAO", None, None),
            color_passes: AverageIntProfileCounter::new(
                "Color passes",
                None, Some(expected::COLOR_PASSES),
            ),
            alpha_passes: AverageIntProfileCounter::new(
                "Alpha passes",
                None, Some(expected::ALPHA_PASSES),
            ),
            texture_data_uploaded: AverageIntProfileCounter::new(
                "Texture data, kb",
                None, Some(expected::TEXTURE_DATA_UPLOADED),
            ),
            rendered_picture_cache_tiles: AverageIntProfileCounter::new(
                "Rendered tiles",
                None, Some(expected::RENDERED_PICTURE_CACHE_TILES),
            ),
            total_picture_cache_tiles: AverageIntProfileCounter::new(
                "Total tiles",
                None, Some(expected::TOTAL_PICTURE_CACHE_TILES),
            ),
        }
    }

    pub fn reset(&mut self) {
        self.draw_calls.reset();
        self.vertices.reset();
        self.color_passes.reset();
        self.alpha_passes.reset();
        self.texture_data_uploaded.reset();
        self.rendered_picture_cache_tiles.reset();
        self.total_picture_cache_tiles.reset();
    }
}

impl RendererProfileTimers {
    pub fn new() -> Self {
        RendererProfileTimers {
            cpu_time: TimeProfileCounter::new("Renderer CPU Time", false, None),
            gpu_samples: Vec::new(),
            gpu_graph: TimeProfileCounter::new("GPU Time", false, None),
        }
    }
}

struct GraphStats {
    min_value: f32,
    mean_value: f32,
    max_value: f32,
}

struct ProfileGraph {
    max_samples: usize,
    scale: f32,
    values: VecDeque<f32>,
    short_description: &'static str,
    unit_description: &'static str,
}

impl ProfileGraph {
    fn new(
        max_samples: usize,
        scale: f32,
        short_description: &'static str,
        unit_description: &'static str,
    ) -> Self {
        ProfileGraph {
            max_samples,
            scale,
            values: VecDeque::new(),
            short_description,
            unit_description,
        }
    }

    fn push(&mut self, ns: u64) {
        let val = ns as f64 * self.scale as f64;
        if self.values.len() == self.max_samples {
            self.values.pop_back();
        }
        self.values.push_front(val as f32);
    }

    fn stats(&self) -> GraphStats {
        let mut stats = GraphStats {
            min_value: f32::MAX,
            mean_value: 0.0,
            max_value: -f32::MAX,
        };

        for value in &self.values {
            stats.min_value = stats.min_value.min(*value);
            stats.mean_value += *value;
            stats.max_value = stats.max_value.max(*value);
        }

        if !self.values.is_empty() {
            stats.mean_value /= self.values.len() as f32;
        }

        stats
    }

    fn draw_graph(
        &self,
        x: f32,
        y: f32,
        description: &'static str,
        debug_renderer: &mut DebugRenderer,
    ) -> default::Rect<f32> {
        let size = Size2D::new(600.0, 100.0);
        let line_height = debug_renderer.line_height();
        let graph_rect = Rect::new(Point2D::new(x, y), size);
        let mut rect = graph_rect.inflate(10.0, 10.0);

        let stats = self.stats();

        let text_color = ColorU::new(255, 255, 0, 255);
        let text_origin = rect.origin + vec2(rect.size.width, 20.0);
        debug_renderer.add_text(
            text_origin.x,
            text_origin.y,
            description,
            ColorU::new(0, 255, 0, 255),
            None,
        );
        debug_renderer.add_text(
            text_origin.x,
            text_origin.y + line_height,
            &format!("Min: {:.2} {}", stats.min_value, self.unit_description),
            text_color,
            None,
        );
        debug_renderer.add_text(
            text_origin.x,
            text_origin.y + line_height * 2.0,
            &format!("Mean: {:.2} {}", stats.mean_value, self.unit_description),
            text_color,
            None,
        );
        debug_renderer.add_text(
            text_origin.x,
            text_origin.y + line_height * 3.0,
            &format!("Max: {:.2} {}", stats.max_value, self.unit_description),
            text_color,
            None,
        );

        rect.size.width += 140.0;
        debug_renderer.add_quad(
            rect.origin.x,
            rect.origin.y,
            rect.origin.x + rect.size.width + 10.0,
            rect.origin.y + rect.size.height,
            ColorU::new(25, 25, 25, 200),
            ColorU::new(51, 51, 51, 200),
        );

        let bx1 = graph_rect.max_x();
        let by1 = graph_rect.max_y();

        let w = graph_rect.size.width / self.max_samples as f32;
        let h = graph_rect.size.height;

        let color_t0 = ColorU::new(0, 255, 0, 255);
        let color_b0 = ColorU::new(0, 180, 0, 255);

        let color_t1 = ColorU::new(0, 255, 0, 255);
        let color_b1 = ColorU::new(0, 180, 0, 255);

        let color_t2 = ColorU::new(255, 0, 0, 255);
        let color_b2 = ColorU::new(180, 0, 0, 255);

        for (index, sample) in self.values.iter().enumerate() {
            let sample = *sample;
            let x1 = bx1 - index as f32 * w;
            let x0 = x1 - w;

            let y0 = by1 - (sample / stats.max_value) as f32 * h;
            let y1 = by1;

            let (color_top, color_bottom) = if sample < 1000.0 / 60.0 {
                (color_t0, color_b0)
            } else if sample < 1000.0 / 30.0 {
                (color_t1, color_b1)
            } else {
                (color_t2, color_b2)
            };

            debug_renderer.add_quad(x0, y0, x1, y1, color_top, color_bottom);
        }

        rect
    }
}

impl ProfileCounter for ProfileGraph {
    fn description(&self) -> &'static str {
        self.short_description
    }

    fn value(&self) -> String {
        format!("{:.2}ms", self.stats().mean_value)
    }

    fn is_expected(&self) -> bool { true }
}

struct GpuFrame {
    total_time: u64,
    samples: Vec<GpuTimer<GpuProfileTag>>,
}

struct GpuFrameCollection {
    frames: VecDeque<GpuFrame>,
}

impl GpuFrameCollection {
    fn new() -> Self {
        GpuFrameCollection {
            frames: VecDeque::new(),
        }
    }

    fn push(&mut self, total_time: u64, samples: Vec<GpuTimer<GpuProfileTag>>) {
        if self.frames.len() == 20 {
            self.frames.pop_back();
        }
        self.frames.push_front(GpuFrame {
            total_time,
            samples,
        });
    }
}

impl GpuFrameCollection {
    fn draw(&self, x: f32, y: f32, debug_renderer: &mut DebugRenderer) -> default::Rect<f32> {
        let graph_rect = Rect::new(
            Point2D::new(x, y),
            Size2D::new(GRAPH_WIDTH, GRAPH_HEIGHT),
        );
        let bounding_rect = graph_rect.inflate(GRAPH_PADDING, GRAPH_PADDING);

        debug_renderer.add_quad(
            bounding_rect.origin.x,
            bounding_rect.origin.y,
            bounding_rect.origin.x + bounding_rect.size.width,
            bounding_rect.origin.y + bounding_rect.size.height,
            ColorU::new(25, 25, 25, 200),
            ColorU::new(51, 51, 51, 200),
        );

        let w = graph_rect.size.width;
        let mut y0 = graph_rect.origin.y;

        let max_time = self.frames
            .iter()
            .max_by_key(|f| f.total_time)
            .unwrap()
            .total_time as f32;

        let mut tags_present = FastHashMap::default();

        for frame in &self.frames {
            let y1 = y0 + GRAPH_FRAME_HEIGHT;

            let mut current_ns = 0;
            for sample in &frame.samples {
                let x0 = graph_rect.origin.x + w * current_ns as f32 / max_time;
                current_ns += sample.time_ns;
                let x1 = graph_rect.origin.x + w * current_ns as f32 / max_time;
                let mut bottom_color = sample.tag.color;
                bottom_color.a *= 0.5;

                debug_renderer.add_quad(
                    x0,
                    y0,
                    x1,
                    y1,
                    sample.tag.color.into(),
                    bottom_color.into(),
                );

                tags_present.insert(sample.tag.label, sample.tag.color);
            }

            y0 = y1;
        }

        // Add a legend to see which color correspond to what primitive.
        const LEGEND_SIZE: f32 = 20.0;
        const PADDED_LEGEND_SIZE: f32 = 25.0;
        if !tags_present.is_empty() {
            debug_renderer.add_quad(
                bounding_rect.max_x() + GRAPH_PADDING,
                bounding_rect.origin.y,
                bounding_rect.max_x() + GRAPH_PADDING + 200.0,
                bounding_rect.origin.y + tags_present.len() as f32 * PADDED_LEGEND_SIZE + GRAPH_PADDING,
                ColorU::new(25, 25, 25, 200),
                ColorU::new(51, 51, 51, 200),
            );
        }

        for (i, (label, &color)) in tags_present.iter().enumerate() {
            let x0 = bounding_rect.origin.x + bounding_rect.size.width + GRAPH_PADDING * 2.0;
            let y0 = bounding_rect.origin.y + GRAPH_PADDING + i as f32 * PADDED_LEGEND_SIZE;

            debug_renderer.add_quad(
                x0, y0, x0 + LEGEND_SIZE, y0 + LEGEND_SIZE,
                color.into(),
                color.into(),
            );

            debug_renderer.add_text(
                x0 + PADDED_LEGEND_SIZE,
                y0 + LEGEND_SIZE * 0.75,
                label,
                ColorU::new(255, 255, 0, 255),
                None,
            );
        }

        bounding_rect
    }
}

struct DrawState {
    x_left: f32,
    y_left: f32,
    x_right: f32,
    y_right: f32,
}

pub struct Profiler {
    draw_state: DrawState,
    backend_graph: ProfileGraph,
    renderer_graph: ProfileGraph,
    gpu_graph: ProfileGraph,
    ipc_graph: ProfileGraph,
    display_list_build_graph: ProfileGraph,
    scene_build_graph: ProfileGraph,
    blob_raster_graph: ProfileGraph,
    backend_time: AverageTimeProfileCounter,
    renderer_time: AverageTimeProfileCounter,
    gpu_time: AverageTimeProfileCounter,
    ipc_time: AverageTimeProfileCounter,
    gpu_frames: GpuFrameCollection,
    cooldowns: Vec<i32>,
}

impl Profiler {
    pub fn new() -> Self {
        let to_ms_scale = 1.0 / 1000000.0;
        Profiler {
            draw_state: DrawState {
                x_left: 0.0,
                y_left: 0.0,
                x_right: 0.0,
                y_right: 0.0,
            },
            backend_graph: ProfileGraph::new(600, to_ms_scale, "Backend:", "ms"),
            renderer_graph: ProfileGraph::new(600, to_ms_scale, "Renderer:", "ms"),
            gpu_graph: ProfileGraph::new(600, to_ms_scale, "GPU:", "ms"),
            ipc_graph: ProfileGraph::new(600, to_ms_scale, "IPC:", "ms"),
            display_list_build_graph: ProfileGraph::new(600, to_ms_scale, "DisplayList build", "ms"),
            scene_build_graph: ProfileGraph::new(600, to_ms_scale, "Scene build:", "ms"),
            blob_raster_graph: ProfileGraph::new(600, 1.0, "Rasterized blob pixels:", "px"),
            gpu_frames: GpuFrameCollection::new(),
            backend_time: AverageTimeProfileCounter::new(
                "Backend:", false,
                Some(expected::AVG_BACKEND_CPU_TIME),
                Some(expected::MAX_BACKEND_CPU_TIME),
            ),
            renderer_time: AverageTimeProfileCounter::new(
                "Renderer:", false,
                Some(expected::AVG_RENDERER_CPU_TIME),
                Some(expected::MAX_RENDERER_CPU_TIME),
            ),
            ipc_time: AverageTimeProfileCounter::new(
                "IPC:", false,
                Some(expected::AVG_IPC_TIME),
                Some(expected::MAX_IPC_TIME),
            ),
            gpu_time: AverageTimeProfileCounter::new(
                "GPU:", false,
                Some(expected::AVG_GPU_TIME),
                Some(expected::MAX_GPU_TIME),
            ),
            cooldowns: Vec::new(),
        }
    }

    // If we have an array of "cooldown" counters, then only display profiles that
    // are out of the ordinary and keep displaying them until the cooldown is over.
    fn draw_counters<T: ProfileCounter + ?Sized>(
        counters: &[&T],
        mut cooldowns: Option<&mut [i32]>,
        debug_renderer: &mut DebugRenderer,
        left: bool,
        draw_state: &mut DrawState,
    ) {
        let mut label_rect = Rect::zero();
        let mut value_rect = Rect::zero();
        let (mut current_x, mut current_y) = if left {
            (draw_state.x_left, draw_state.y_left)
        } else {
            (draw_state.x_right, draw_state.y_right)
        };
        let mut color_index = 0;
        let line_height = debug_renderer.line_height();

        let colors = [
            // Regular values,
            ColorU::new(255, 255, 255, 255),
            ColorU::new(255, 255, 0, 255),
            // Unexpected values,
            ColorU::new(255, 80, 0, 255),
            ColorU::new(255, 0, 0, 255),
        ];

        for (idx, counter) in counters.iter().enumerate() {
            if let Some(cooldowns) = cooldowns.as_mut() {
                if !counter.is_expected() {
                    cooldowns[idx] = 40;
                }
                if cooldowns[idx] == 0 {
                    continue;
                }
            }
            let rect = debug_renderer.add_text(
                current_x,
                current_y,
                counter.description(),
                colors[color_index],
                None,
            );
            color_index = (color_index + 1) % 2;

            label_rect = label_rect.union(&rect);
            current_y += line_height;
        }

        color_index = 0;
        current_x = label_rect.origin.x + label_rect.size.width + 60.0;
        current_y = if left { draw_state.y_left } else { draw_state.y_right };

        for (idx, counter) in counters.iter().enumerate() {
            let expected_offset = if counter.is_expected() || cooldowns.is_some() { 0 } else { 2 };
            if let Some(cooldowns) = cooldowns.as_mut() {
                if cooldowns[idx] > 0 {
                    cooldowns[idx] -= 1;
                } else {
                    continue;
                }
            }
            let rect = debug_renderer.add_text(
                current_x,
                current_y,
                &counter.value(),
                colors[color_index + expected_offset],
                None,
            );
            color_index = (color_index + 1) % 2;

            value_rect = value_rect.union(&rect);
            current_y += line_height;
        }

        let total_rect = label_rect.union(&value_rect).inflate(10.0, 10.0);
        debug_renderer.add_quad(
            total_rect.origin.x,
            total_rect.origin.y,
            total_rect.origin.x + total_rect.size.width,
            total_rect.origin.y + total_rect.size.height,
            ColorF::new(0.1, 0.1, 0.1, 0.8).into(),
            ColorF::new(0.2, 0.2, 0.2, 0.8).into(),
        );
        let new_y = total_rect.origin.y + total_rect.size.height + 30.0;
        if left {
            draw_state.y_left = new_y;
        } else {
            draw_state.y_right = new_y;
        }
    }

    fn draw_bar(
        &mut self,
        label: &str,
        label_color: ColorU,
        counters: &[(ColorU, &AverageIntProfileCounter)],
        debug_renderer: &mut DebugRenderer,
    ) -> default::Rect<f32> {
        let mut rect = debug_renderer.add_text(
            self.draw_state.x_left,
            self.draw_state.y_left,
            label,
            label_color,
            None,
        );

        let x_base = rect.origin.x + rect.size.width + 10.0;
        let height = debug_renderer.line_height();
        let width = (self.draw_state.x_right - 30.0 - x_base).max(0.0);
        let total_value = counters.last().unwrap().1.get();
        let scale = width / total_value as f32;
        let mut x_current = x_base;

        for &(color, counter) in counters {
            let x_stop = x_base + counter.get() as f32 * scale;
            debug_renderer.add_quad(
                x_current,
                rect.origin.y,
                x_stop,
                rect.origin.y + height,
                color,
                color,
            );
            x_current = x_stop;
        }

        self.draw_state.y_left += height;

        rect.size.width += width + 10.0;
        rect
    }

    fn draw_gpu_cache_bars(
        &mut self,
        counters: &GpuCacheProfileCounters,
        debug_renderer: &mut DebugRenderer,
    ) {
        let color_updated = ColorU::new(0xFF, 0, 0, 0xFF);
        let color_free = ColorU::new(0, 0, 0xFF, 0xFF);
        let color_saved = ColorU::new(0, 0xFF, 0, 0xFF);

        let mut requested_blocks = AverageIntProfileCounter::new("", None, None);
        requested_blocks.set(counters.updated_blocks.get() + counters.saved_blocks.get());

        let mut total_blocks = AverageIntProfileCounter::new("", None, None);
        total_blocks.set(counters.allocated_rows.get() * MAX_VERTEX_TEXTURE_WIDTH);

        let rect0 = self.draw_bar(
            &format!("GPU cache rows ({}):", counters.allocated_rows.get()),
            ColorU::new(0xFF, 0xFF, 0xFF, 0xFF),
            &[
                (color_updated, &counters.updated_rows),
                (color_free, &counters.allocated_rows),
            ],
            debug_renderer,
        );

        let rect1 = self.draw_bar(
            "GPU cache blocks",
            ColorU::new(0xFF, 0xFF, 0, 0xFF),
            &[
                (color_updated, &counters.updated_blocks),
                (color_saved, &requested_blocks),
                (color_free, &counters.allocated_blocks),
                (ColorU::new(0, 0, 0, 0xFF), &total_blocks),
            ],
            debug_renderer,
        );

        let total_rect = rect0.union(&rect1).inflate(10.0, 10.0);
        debug_renderer.add_quad(
            total_rect.origin.x,
            total_rect.origin.y,
            total_rect.origin.x + total_rect.size.width,
            total_rect.origin.y + total_rect.size.height,
            ColorF::new(0.1, 0.1, 0.1, 0.8).into(),
            ColorF::new(0.2, 0.2, 0.2, 0.8).into(),
        );

        self.draw_state.y_left = total_rect.origin.y + total_rect.size.height + 30.0;
    }

    fn draw_frame_bars(
        &mut self,
        counters: &FrameProfileCounters,
        debug_renderer: &mut DebugRenderer,
    ) {
        let rect0 = self.draw_bar(
            &format!("primitives ({}):", counters.total_primitives.get()),
            ColorU::new(0xFF, 0xFF, 0xFF, 0xFF),
            &[
                (ColorU::new(0, 0, 0xFF, 0xFF), &counters.visible_primitives),
                (ColorU::new(0, 0, 0, 0xFF), &counters.total_primitives),
            ],
            debug_renderer,
        );

        let rect1 = self.draw_bar(
            &format!("GPU targets ({}):", &counters.targets_used.get()),
            ColorU::new(0xFF, 0xFF, 0, 0xFF),
            &[
                (ColorU::new(0, 0, 0xFF, 0xFF), &counters.targets_created),
                (ColorU::new(0xFF, 0, 0, 0xFF), &counters.targets_changed),
                (ColorU::new(0, 0xFF, 0, 0xFF), &counters.targets_used),
            ],
            debug_renderer,
        );

        let total_rect = rect0.union(&rect1).inflate(10.0, 10.0);
        debug_renderer.add_quad(
            total_rect.origin.x,
            total_rect.origin.y,
            total_rect.origin.x + total_rect.size.width,
            total_rect.origin.y + total_rect.size.height,
            ColorF::new(0.1, 0.1, 0.1, 0.8).into(),
            ColorF::new(0.2, 0.2, 0.2, 0.8).into(),
        );

        self.draw_state.y_left = total_rect.origin.y + total_rect.size.height + 30.0;
    }

    fn draw_compact_profile(
        &mut self,
        backend_profile: &BackendProfileCounters,
        renderer_profile: &RendererProfileCounters,
        debug_renderer: &mut DebugRenderer,
    ) {
        Profiler::draw_counters(
            &[
                &renderer_profile.frame_time as &dyn ProfileCounter,
                &renderer_profile.color_passes,
                &renderer_profile.alpha_passes,
                &renderer_profile.draw_calls,
                &renderer_profile.vertices,
                &renderer_profile.rendered_picture_cache_tiles,
                &renderer_profile.texture_data_uploaded,
                &backend_profile.resources.content_slices,
                &self.ipc_time,
                &self.backend_time,
                &self.renderer_time,
                &self.gpu_time,
            ],
            None,
            debug_renderer,
            true,
            &mut self.draw_state,
        );
    }

    fn draw_full_profile(
        &mut self,
        frame_profiles: &[FrameProfileCounters],
        backend_profile: &BackendProfileCounters,
        renderer_profile: &RendererProfileCounters,
        renderer_timers: &mut RendererProfileTimers,
        gpu_samplers: &[GpuSampler<GpuProfileTag>],
        screen_fraction: f32,
        debug_renderer: &mut DebugRenderer,
    ) {
        Profiler::draw_counters(
            &[
                &renderer_profile.frame_time as &dyn ProfileCounter,
                &renderer_profile.frame_counter,
                &renderer_profile.color_passes,
                &renderer_profile.alpha_passes,
                &renderer_profile.rendered_picture_cache_tiles,
                &renderer_profile.total_picture_cache_tiles,
                &renderer_profile.texture_data_uploaded,
                &backend_profile.resources.content_slices,
                &backend_profile.resources.texture_cache.shared_bytes,
                &backend_profile.resources.texture_cache.standalone_bytes,
            ],
            None,
            debug_renderer,
            true,
            &mut self.draw_state
        );

        self.draw_gpu_cache_bars(
            &backend_profile.resources.gpu_cache,
            debug_renderer,
        );

        Profiler::draw_counters(
            &[
                &backend_profile.resources.font_templates,
                &backend_profile.resources.image_templates,
            ],
            None,
            debug_renderer,
            true,
            &mut self.draw_state
        );

        backend_profile.intern.draw(debug_renderer, &mut self.draw_state);

        Profiler::draw_counters(
            &[
                &backend_profile.resources.texture_cache.pages_alpha8_linear,
                &backend_profile.resources.texture_cache.pages_color8_linear,
                &backend_profile.resources.texture_cache.pages_color8_nearest,
                &backend_profile.txn.display_lists,
            ],
            None,
            debug_renderer,
            true,
            &mut self.draw_state
        );

        Profiler::draw_counters(
            &[
                &backend_profile.txn.display_list_build_time,
                &backend_profile.txn.scene_build_time,
                &backend_profile.txn.content_send_time,
                &backend_profile.txn.api_send_time,
                &backend_profile.txn.total_send_time,
            ],
            None,
            debug_renderer,
            true,
            &mut self.draw_state
        );

        for frame_profile in frame_profiles {
            self.draw_frame_bars(frame_profile, debug_renderer);
        }

        Profiler::draw_counters(
            &[&renderer_profile.draw_calls, &renderer_profile.vertices],
            None,
            debug_renderer,
            true,
            &mut self.draw_state
        );

        Profiler::draw_counters(
            &[
                &backend_profile.total_time,
                &renderer_timers.cpu_time,
                &renderer_timers.gpu_graph,
            ],
            None,
            debug_renderer,
            false,
            &mut self.draw_state
        );

        if !gpu_samplers.is_empty() {
            let mut samplers = Vec::<PercentageProfileCounter>::new();
            // Gathering unique GPU samplers. This has O(N^2) complexity,
            // but we only have a few samplers per target.
            let mut total = 0.0;
            for sampler in gpu_samplers {
                let value = sampler.count as f32 * screen_fraction;
                total += value;
                match samplers.iter().position(|s| {
                    s.description as *const _ == sampler.tag.label as *const _
                }) {
                    Some(pos) => samplers[pos].value += value,
                    None => samplers.push(PercentageProfileCounter {
                        description: sampler.tag.label,
                        value,
                    }),
                }
            }
            samplers.push(PercentageProfileCounter {
                description: "Total",
                value: total,
            });
            let samplers: Vec<&dyn ProfileCounter> = samplers.iter().map(|sampler| {
                sampler as &dyn ProfileCounter
            }).collect();
            Profiler::draw_counters(
                &samplers,
                None,
                debug_renderer,
                false,
                &mut self.draw_state,
            );
        }

        let rect =
            self.backend_graph
                .draw_graph(self.draw_state.x_right, self.draw_state.y_right, "CPU (backend)", debug_renderer);
        self.draw_state.y_right += rect.size.height + PROFILE_PADDING;
        let rect = self.renderer_graph.draw_graph(
            self.draw_state.x_right,
            self.draw_state.y_right,
            "CPU (renderer)",
            debug_renderer,
        );
        self.draw_state.y_right += rect.size.height + PROFILE_PADDING;
        let rect =
            self.ipc_graph
                .draw_graph(self.draw_state.x_right, self.draw_state.y_right, "DisplayList IPC", debug_renderer);
        self.draw_state.y_right += rect.size.height + PROFILE_PADDING;

        let rect = self.display_list_build_graph
            .draw_graph(self.draw_state.x_right, self.draw_state.y_right, "DisplayList build", debug_renderer);
        self.draw_state.y_right += rect.size.height + PROFILE_PADDING;

        let rect = self.scene_build_graph
            .draw_graph(self.draw_state.x_right, self.draw_state.y_right, "Scene build", debug_renderer);
        self.draw_state.y_right += rect.size.height + PROFILE_PADDING;

        let rect = self.gpu_graph
            .draw_graph(self.draw_state.x_right, self.draw_state.y_right, "GPU", debug_renderer);
        self.draw_state.y_right += rect.size.height + PROFILE_PADDING;

        let rect = self.blob_raster_graph
            .draw_graph(self.draw_state.x_right, self.draw_state.y_right, "Blob pixels", debug_renderer);
        self.draw_state.y_right += rect.size.height + PROFILE_PADDING;

        let rect = self.gpu_frames
            .draw(self.draw_state.x_left, f32::max(self.draw_state.y_left, self.draw_state.y_right), debug_renderer);
        self.draw_state.y_right += rect.size.height + PROFILE_PADDING;
    }

    fn draw_smart_profile(
        &mut self,
        backend_profile: &BackendProfileCounters,
        renderer_profile: &RendererProfileCounters,
        debug_renderer: &mut DebugRenderer,
    ) {
        while self.cooldowns.len() < 18 {
            self.cooldowns.push(0);
        }

        // Always show the fps counter.
        Profiler::draw_counters(
            &[
                &renderer_profile.frame_time,
            ],
            None,
            debug_renderer,
            true,
            &mut self.draw_state,
        );

        let mut start = 0;
        let counters: &[&[&dyn ProfileCounter]] = &[
            &[
                &self.backend_time,
                &self.renderer_time,
                &self.gpu_time,
            ],
            &[
                &renderer_profile.color_passes,
                &renderer_profile.alpha_passes,
                &renderer_profile.draw_calls,
                &renderer_profile.vertices,
                &renderer_profile.rendered_picture_cache_tiles,
                &renderer_profile.total_picture_cache_tiles,
            ],
            &[
                &backend_profile.resources.gpu_cache.allocated_rows,
                &backend_profile.resources.gpu_cache.updated_rows,
                &backend_profile.resources.gpu_cache.allocated_blocks,
                &backend_profile.resources.gpu_cache.updated_blocks,
                &backend_profile.resources.gpu_cache.saved_blocks,
            ],
            &[
                &backend_profile.resources.image_templates,
                &backend_profile.resources.font_templates,
                &backend_profile.resources.texture_cache.rasterized_blob_pixels,
                &backend_profile.txn.display_lists,
            ],
        ];

        for group in counters {
            let end = start + group.len();
            Profiler::draw_counters(
                &group[..],
                Some(&mut self.cooldowns[start..end]),
                debug_renderer,
                true,
                &mut self.draw_state,
            );
            start = end;
        }
    }

    pub fn draw_profile(
        &mut self,
        frame_profiles: &[FrameProfileCounters],
        backend_profile: &BackendProfileCounters,
        renderer_profile: &RendererProfileCounters,
        renderer_timers: &mut RendererProfileTimers,
        gpu_samplers: &[GpuSampler<GpuProfileTag>],
        screen_fraction: f32,
        debug_renderer: &mut DebugRenderer,
        style: ProfileStyle,
    ) {
        self.draw_state.x_left = 20.0;
        self.draw_state.y_left = 50.0;
        self.draw_state.x_right = 450.0;
        self.draw_state.y_right = 40.0;

        let mut gpu_graph = 0;
        let gpu_graphrs = mem::replace(&mut renderer_timers.gpu_samples, Vec::new());
        for sample in &gpu_graphrs {
            gpu_graph += sample.time_ns;
        }
        renderer_timers.gpu_graph.set(gpu_graph);

        self.backend_graph
            .push(backend_profile.total_time.nanoseconds);
        self.backend_time.set(backend_profile.total_time.nanoseconds);
        self.renderer_graph
            .push(renderer_timers.cpu_time.nanoseconds);
        self.renderer_time.set(renderer_timers.cpu_time.nanoseconds);
        self.ipc_graph
            .push(backend_profile.txn.total_send_time.nanoseconds);
        self.display_list_build_graph
            .push(backend_profile.txn.display_list_build_time.nanoseconds);
        self.scene_build_graph
            .push(backend_profile.txn.scene_build_time.nanoseconds);
        self.blob_raster_graph
            .push(backend_profile.resources.texture_cache.rasterized_blob_pixels.size as u64);
        self.ipc_time.set(backend_profile.txn.total_send_time.nanoseconds);
        self.gpu_graph.push(gpu_graph);
        self.gpu_time.set(gpu_graph);
        self.gpu_frames.push(gpu_graph, gpu_graphrs);

        match style {
            ProfileStyle::Full => {
                self.draw_full_profile(
                    frame_profiles,
                    backend_profile,
                    renderer_profile,
                    renderer_timers,
                    gpu_samplers,
                    screen_fraction,
                    debug_renderer,
                );
            }
            ProfileStyle::Compact => {
                self.draw_compact_profile(
                    backend_profile,
                    renderer_profile,
                    debug_renderer,
                );
            }
            ProfileStyle::Smart => {
                self.draw_smart_profile(
                    backend_profile,
                    renderer_profile,
                    debug_renderer,
                );
            }
        }
    }
}

pub struct ChangeIndicator {
    counter: u32,
}

impl ChangeIndicator {
    pub fn new() -> Self {
        ChangeIndicator {
            counter: 0
        }
    }

    pub fn changed(&mut self) {
        self.counter = (self.counter + 1) % 15;
    }

    const WIDTH : f32 = 20.0;
    const HEIGHT: f32 = 10.0;

    pub fn width() -> f32 {
      ChangeIndicator::WIDTH * 16.0
    }

    pub fn draw(
        &self,
        x: f32, y: f32,
        color: ColorU,
        debug_renderer: &mut DebugRenderer
    ) {
        let margin = 0.0;
        let tx = self.counter as f32 * ChangeIndicator::WIDTH;
        debug_renderer.add_quad(
            x - margin,
            y - margin,
            x + 15.0 * ChangeIndicator::WIDTH + margin,
            y + ChangeIndicator::HEIGHT + margin,
            ColorU::new(0, 0, 0, 150),
            ColorU::new(0, 0, 0, 150),
        );

        debug_renderer.add_quad(
            x + tx,
            y,
            x + tx + ChangeIndicator::WIDTH,
            y + ChangeIndicator::HEIGHT,
            color,
            ColorU::new(25, 25, 25, 255),
        );
    }
}
