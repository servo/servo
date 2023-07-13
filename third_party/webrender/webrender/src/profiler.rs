/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Overlay profiler
//!
//! ## Profiler UI string syntax
//!
//! Comma-separated list of of tokens with trailing and leading spaces trimmed.
//! Each tokens can be:
//! - A counter name with an optional prefix. The name corresponds to the displayed name (see the
//!   counters vector below.
//!   - By default (no prefix) the counter is shown as average + max over half a second.
//!   - With a '#' prefix the counter is shown as a graph.
//!   - With a '*' prefix the counter is shown as a change indicator.
//!   - Some special counters such as GPU time queries have specific visualizations ignoring prefixes.
//! - A preset name to append the preset to the UI (see PROFILER_PRESETS).
//! - An empty token to insert a bit of vertical space.
//! - A '|' token to start a new column.
//! - A '_' token to start a new row.

use api::{ColorF, ColorU};
use crate::renderer::DebugRenderer;
use crate::device::query::GpuTimer;
use euclid::{Point2D, Rect, Size2D, vec2, default};
use crate::internal_types::FastHashMap;
use crate::renderer::{FullFrameStats, MAX_VERTEX_TEXTURE_WIDTH, wr_has_been_initialized};
use api::units::DeviceIntSize;
use std::collections::vec_deque::VecDeque;
use std::fmt::{Write, Debug};
use std::f32;
use std::ffi::CStr;
use std::ops::Range;
use std::time::Duration;
use time::precise_time_ns;

macro_rules! set_text {
    ($dst:expr, $($arg:tt)*) => {
        $dst.clear();
        write!($dst, $($arg)*).unwrap();
    };
}

const GRAPH_WIDTH: f32 = 1024.0;
const GRAPH_HEIGHT: f32 = 320.0;
const GRAPH_PADDING: f32 = 8.0;
const GRAPH_FRAME_HEIGHT: f32 = 16.0;
const PROFILE_SPACING: f32 = 15.0;
const PROFILE_PADDING: f32 = 10.0;
const BACKGROUND_COLOR: ColorU = ColorU { r: 20, g: 20, b: 20, a: 220 };

const ONE_SECOND_NS: u64 = 1_000_000_000;

/// Profiler UI string presets. Defined in the profiler UI string syntax, can contain other presets.
static PROFILER_PRESETS: &'static[(&'static str, &'static str)] = &[
    // Default view, doesn't show everything, but still shows quite a bit.
    (&"Default", &"FPS,|,Slow indicators,_,Time graphs,|,Frame times, ,Transaction times, ,Frame stats, ,Memory, ,Interners,_,GPU time queries,_,Paint phase graph"),
    // Smaller, less intrusive overview
    (&"Compact", &"FPS, ,Frame times, ,Frame stats"),
    // Even less intrusive, only slow transactions and frame indicators.
    (&"Slow indicators", &"*Slow transaction,*Slow frame"),

    // Counters:

    // Timing information for per layout transaction stages.
    (&"Transaction times", &"DisplayList,Scene building,Content send,API send"),
    // Timing information for per-frame stages.
    (&"Frame times", &"Frame CPU total,Frame building,Visibility,Prepare,Batching,Glyph resolve,Texture cache update,Renderer,GPU"),
    // Stats about the content of the frame.
    (&"Frame stats", &"Primitives,Visible primitives,Draw calls,Vertices,Color passes,Alpha passes,Rendered picture tiles,Rasterized glyphs"),
    // Texture cache allocation stats.
    (&"Texture cache stats", &"Texture cache RGBA8 linear textures, Texture cache RGBA8 linear pixels, Texture cache RGBA8 linear pressure,
        , ,Texture cache RGBA8 glyphs textures, Texture cache RGBA8 glyphs pixels, Texture cache RGBA8 glyphs pressure,
        , ,Texture cache A8 glyphs textures, Texture cache A8 glyphs pixels, Texture cache A8 glyphs pressure,
        , ,Texture cache A8 textures, Texture cache A8 pixels, Texture cache A8 pressure,
        , ,Texture cache A16 textures, Texture cache A16 pixels, Texture cache A16 pressure,
        , ,Texture cache RGBA8 nearest textures, Texture cache RGBA8 nearest pixels, Texture cache RGBA8 nearest pressure,
        , ,Texture cache shared mem, Texture cache standalone mem, Texture cache standalone pressure,
        , ,Texture cache eviction count, Texture cache youngest evicted"
    ),
    // Graphs to investigate driver overhead of texture cache updates.
    (&"Texture upload perf", &"#Texture cache update,#Texture cache upload, ,#Staging CPU allocation,#Staging GPU allocation,#Staging CPU copy,#Staging GPU copy,#Upload time, ,#Upload copy batches,#Rasterized glyphs, ,#Cache texture creation,#Cache texture deletion"),

    // Graphs:

    // Graph overview of time spent in WebRender's main stages.
    (&"Time graphs", &"#DisplayList,#Scene building,#Blob rasterization, ,#Frame CPU total,#Frame building,#Renderer,#Texture cache update, ,#GPU,"),
    // Useful when investigating render backend bottlenecks.
    (&"Backend graphs", &"#Frame building, #Visibility, #Prepare, #Batching, #Glyph resolve"),
    // Useful when investigating renderer bottlenecks.
    (&"Renderer graphs", &"#Rendered picture tiles,#Draw calls,#Rasterized glyphs,#Texture uploads,#Texture uploads mem, ,#Texture cache update,#Renderer,"),

    // Misc:

    (&"Memory", &"Image templates,Image templates mem,Font templates,Font templates mem,DisplayList mem,Picture tiles mem"),
    (&"Interners", "Interned primitives,Interned clips,Interned pictures,Interned text runs,Interned normal borders,Interned image borders,Interned images,Interned YUV images,Interned line decorations,Interned linear gradients,Interned radial gradients,Interned conic gradients,Interned filter data,Interned backdrops"),
    // Gpu sampler queries (need the pref gfx.webrender.debug.gpu-sampler-queries).
    (&"GPU samplers", &"Alpha targets samplers,Transparent pass samplers,Opaque pass samplers,Total samplers"),
];

fn find_preset(name: &str) -> Option<&'static str> {
    for preset in PROFILER_PRESETS {
        if preset.0 == name {
            return Some(preset.1);
        }
    }

    None
}

// The indices here must match the PROFILE_COUNTERS array (checked at runtime).
pub const FRAME_BUILDING_TIME: usize = 0;
pub const FRAME_VISIBILITY_TIME: usize = 1;
pub const FRAME_PREPARE_TIME: usize = 2;
pub const FRAME_BATCHING_TIME: usize = 3;

pub const RENDERER_TIME: usize = 4;
pub const TOTAL_FRAME_CPU_TIME: usize = 5;
pub const GPU_TIME: usize = 6;

pub const CONTENT_SEND_TIME: usize = 7;
pub const API_SEND_TIME: usize = 8;

pub const DISPLAY_LIST_BUILD_TIME: usize = 9;
pub const DISPLAY_LIST_MEM: usize = 10;

pub const SCENE_BUILD_TIME: usize = 11;

pub const RASTERIZED_BLOBS: usize = 12;
pub const RASTERIZED_BLOB_TILES: usize = 13;
pub const RASTERIZED_BLOBS_PX: usize = 14;
pub const BLOB_RASTERIZATION_TIME: usize = 15;

pub const RASTERIZED_GLYPHS: usize = 16;
pub const GLYPH_RESOLVE_TIME: usize = 17;

pub const DRAW_CALLS: usize = 18;
pub const VERTICES: usize = 19;
pub const PRIMITIVES: usize = 20;
pub const VISIBLE_PRIMITIVES: usize = 21;

pub const USED_TARGETS: usize = 22;
pub const CREATED_TARGETS: usize = 23;
pub const PICTURE_CACHE_SLICES: usize = 24;

pub const COLOR_PASSES: usize = 25;
pub const ALPHA_PASSES: usize = 26;
pub const PICTURE_TILES: usize = 27;
pub const PICTURE_TILES_MEM: usize = 28;
pub const RENDERED_PICTURE_TILES: usize = 29;
pub const TEXTURE_UPLOADS: usize = 30;
pub const TEXTURE_UPLOADS_MEM: usize = 31;

pub const FONT_TEMPLATES: usize = 32;
pub const FONT_TEMPLATES_MEM: usize = 33;
pub const IMAGE_TEMPLATES: usize = 34;
pub const IMAGE_TEMPLATES_MEM: usize = 35;

pub const GPU_CACHE_ROWS_TOTAL: usize = 36;
pub const GPU_CACHE_ROWS_UPDATED: usize = 37;
pub const GPU_CACHE_BLOCKS_TOTAL: usize = 38;
pub const GPU_CACHE_BLOCKS_UPDATED: usize = 39;
pub const GPU_CACHE_BLOCKS_SAVED: usize = 40;

pub const TEXTURE_CACHE_A8_PIXELS: usize = 41;
pub const TEXTURE_CACHE_A8_TEXTURES: usize = 42;
pub const TEXTURE_CACHE_A16_PIXELS: usize = 43;
pub const TEXTURE_CACHE_A16_TEXTURES: usize = 44;
pub const TEXTURE_CACHE_RGBA8_LINEAR_PIXELS: usize = 45;
pub const TEXTURE_CACHE_RGBA8_LINEAR_TEXTURES: usize = 46;
pub const TEXTURE_CACHE_RGBA8_NEAREST_PIXELS: usize = 47;
pub const TEXTURE_CACHE_RGBA8_NEAREST_TEXTURES: usize = 48;
pub const TEXTURE_CACHE_SHARED_MEM: usize = 49;
pub const TEXTURE_CACHE_STANDALONE_MEM: usize = 50;

pub const SLOW_FRAME: usize = 51;
pub const SLOW_TXN: usize = 52;

pub const GPU_CACHE_UPLOAD_TIME: usize = 53;
pub const TEXTURE_CACHE_UPDATE_TIME: usize = 54;

pub const FRAME_TIME: usize = 55;

pub const ALPHA_TARGETS_SAMPLERS: usize = 56;
pub const TRANSPARENT_PASS_SAMPLERS: usize = 57;
pub const OPAQUE_PASS_SAMPLERS: usize = 58;
pub const TOTAL_SAMPLERS: usize = 59;

pub const INTERNED_PRIMITIVES: usize = 60;
pub const INTERNED_CLIPS: usize = 61;
pub const INTERNED_TEXT_RUNS: usize = 62;
pub const INTERNED_NORMAL_BORDERS: usize = 63;
pub const INTERNED_IMAGE_BORDERS: usize = 64;
pub const INTERNED_IMAGES: usize = 65;
pub const INTERNED_YUV_IMAGES: usize = 66;
pub const INTERNED_LINE_DECORATIONS: usize = 67;
pub const INTERNED_LINEAR_GRADIENTS: usize = 68;
pub const INTERNED_RADIAL_GRADIENTS: usize = 69;
pub const INTERNED_CONIC_GRADIENTS: usize = 70;
pub const INTERNED_PICTURES: usize = 71;
pub const INTERNED_FILTER_DATA: usize = 72;
pub const INTERNED_BACKDROPS: usize = 73;
pub const INTERNED_POLYGONS: usize = 74;

pub const TEXTURE_CACHE_RGBA8_GLYPHS_PIXELS: usize = 75;
pub const TEXTURE_CACHE_RGBA8_GLYPHS_TEXTURES: usize = 76;
pub const TEXTURE_CACHE_A8_GLYPHS_PIXELS: usize = 77;
pub const TEXTURE_CACHE_A8_GLYPHS_TEXTURES: usize = 78;

pub const CPU_TEXTURE_ALLOCATION_TIME: usize = 79;
pub const STAGING_TEXTURE_ALLOCATION_TIME: usize = 80;
pub const UPLOAD_CPU_COPY_TIME: usize = 81;
pub const UPLOAD_GPU_COPY_TIME: usize = 82;
pub const UPLOAD_TIME: usize = 83;
pub const UPLOAD_NUM_COPY_BATCHES: usize = 84;
pub const TOTAL_UPLOAD_TIME: usize = 85;
pub const CREATE_CACHE_TEXTURE_TIME: usize = 86;
pub const DELETE_CACHE_TEXTURE_TIME: usize = 87;

pub const TEXTURE_CACHE_COLOR8_LINEAR_PRESSURE: usize = 88;
pub const TEXTURE_CACHE_COLOR8_NEAREST_PRESSURE: usize = 89;
pub const TEXTURE_CACHE_COLOR8_GLYPHS_PRESSURE: usize = 90;
pub const TEXTURE_CACHE_ALPHA8_PRESSURE: usize = 91;
pub const TEXTURE_CACHE_ALPHA8_GLYPHS_PRESSURE: usize = 92;
pub const TEXTURE_CACHE_ALPHA16_PRESSURE: usize = 93;
pub const TEXTURE_CACHE_STANDALONE_PRESSURE: usize = 94;
pub const TEXTURE_CACHE_EVICTION_COUNT: usize = 95;
pub const TEXTURE_CACHE_YOUNGEST_EVICTION: usize = 96;

pub const NUM_PROFILER_EVENTS: usize = 97;

pub struct Profiler {
    counters: Vec<Counter>,
    gpu_frames: ProfilerFrameCollection,
    frame_stats: ProfilerFrameCollection,

    start: u64,
    avg_over_period: u64,
    num_graph_samples: usize,

    // For FPS computation. Updated in update().
    frame_timestamps_within_last_second: Vec<u64>,

    ui: Vec<Item>,
}

impl Profiler {
    pub fn new() -> Self {

        fn float(name: &'static str, unit: &'static str, index: usize, expected: Expected<f64>) -> CounterDescriptor {
            CounterDescriptor { name, unit, show_as: ShowAs::Float, index, expected }
        }

        fn int(name: &'static str, unit: &'static str, index: usize, expected: Expected<i64>) -> CounterDescriptor {
            CounterDescriptor { name, unit, show_as: ShowAs::Int, index, expected: expected.into_float() }
        }

        // Not in the list below:
        // - "GPU time queries" shows the details of the GPU time queries if selected as a graph.
        // - "GPU cache bars" shows some info about the GPU cache.

        // TODO: This should be a global variable but to keep things readable we need to be able to
        // use match in const fn which isn't supported by the current rustc version in gecko's build
        // system.
        let profile_counters = &[
            float("Frame building", "ms", FRAME_BUILDING_TIME, expected(0.0..6.0).avg(0.0..3.0)),

            float("Visibility", "ms", FRAME_VISIBILITY_TIME, expected(0.0..3.0).avg(0.0..2.0)),
            float("Prepare", "ms", FRAME_PREPARE_TIME, expected(0.0..3.0).avg(0.0..2.0)),
            float("Batching", "ms", FRAME_BATCHING_TIME, expected(0.0..3.0).avg(0.0..2.0)),

            float("Renderer", "ms", RENDERER_TIME, expected(0.0..8.0).avg(0.0..5.0)),
            float("Frame CPU total", "ms", TOTAL_FRAME_CPU_TIME, expected(0.0..15.0).avg(0.0..6.0)),
            float("GPU", "ms", GPU_TIME, expected(0.0..15.0).avg(0.0..8.0)),

            float("Content send", "ms", CONTENT_SEND_TIME, expected(0.0..1.0).avg(0.0..1.0)),
            float("API send", "ms", API_SEND_TIME, expected(0.0..1.0).avg(0.0..0.4)),
            float("DisplayList", "ms", DISPLAY_LIST_BUILD_TIME, expected(0.0..5.0).avg(0.0..3.0)),
            float("DisplayList mem", "MB", DISPLAY_LIST_MEM, expected(0.0..20.0)),
            float("Scene building", "ms", SCENE_BUILD_TIME, expected(0.0..4.0).avg(0.0..3.0)),

            int("Rasterized blobs", "", RASTERIZED_BLOBS, expected(0..15)),
            int("Rasterized blob tiles", "", RASTERIZED_BLOB_TILES, expected(0..15)),
            int("Rasterized blob pixels", "px", RASTERIZED_BLOBS_PX, expected(0..300_000)),
            float("Blob rasterization", "ms", BLOB_RASTERIZATION_TIME, expected(0.0..8.0)),

            int("Rasterized glyphs", "", RASTERIZED_GLYPHS, expected(0..15)),
            float("Glyph resolve", "ms", GLYPH_RESOLVE_TIME, expected(0.0..4.0)),

            int("Draw calls", "", DRAW_CALLS, expected(1..120).avg(1..90)),
            int("Vertices", "", VERTICES, expected(10..5000)),
            int("Primitives", "", PRIMITIVES, expected(10..5000)),
            int("Visible primitives", "", VISIBLE_PRIMITIVES, expected(1..5000)),

            int("Used targets", "", USED_TARGETS, expected(1..4)),
            int("Created targets", "", CREATED_TARGETS, expected(0..3)),
            int("Picture cache slices", "", PICTURE_CACHE_SLICES, expected(0..5)),

            int("Color passes", "", COLOR_PASSES, expected(1..4)),
            int("Alpha passes", "", ALPHA_PASSES, expected(0..3)),
            int("Picture tiles", "", PICTURE_TILES, expected(0..15)),
            float("Picture tiles mem", "MB", PICTURE_TILES_MEM, expected(0.0..150.0)),
            int("Rendered picture tiles", "", RENDERED_PICTURE_TILES, expected(0..5)),
            int("Texture uploads", "", TEXTURE_UPLOADS, expected(0..10)),
            float("Texture uploads mem", "MB", TEXTURE_UPLOADS_MEM, expected(0.0..10.0)),

            int("Font templates", "", FONT_TEMPLATES, expected(0..40)),
            float("Font templates mem", "MB", FONT_TEMPLATES_MEM, expected(0.0..20.0)),
            int("Image templates", "", IMAGE_TEMPLATES, expected(0..100)),
            float("Image templates mem", "MB", IMAGE_TEMPLATES_MEM, expected(0.0..50.0)),

            int("GPU cache rows total", "", GPU_CACHE_ROWS_TOTAL, expected(1..50)),
            int("GPU cache rows updated", "", GPU_CACHE_ROWS_UPDATED, expected(0..25)),
            int("GPU blocks total", "", GPU_CACHE_BLOCKS_TOTAL, expected(1..65_000)),
            int("GPU blocks updated", "", GPU_CACHE_BLOCKS_UPDATED, expected(0..1000)),
            int("GPU blocks saved", "", GPU_CACHE_BLOCKS_SAVED, expected(0..50_000)),

            int("Texture cache A8 pixels", "px", TEXTURE_CACHE_A8_PIXELS, expected(0..1_000_000)),
            int("Texture cache A8 textures", "", TEXTURE_CACHE_A8_TEXTURES, expected(0..2)),
            int("Texture cache A16 pixels", "px", TEXTURE_CACHE_A16_PIXELS, expected(0..260_000)),
            int("Texture cache A16 textures", "", TEXTURE_CACHE_A16_TEXTURES, expected(0..2)),
            int("Texture cache RGBA8 linear pixels", "px", TEXTURE_CACHE_RGBA8_LINEAR_PIXELS, expected(0..8_000_000)),
            int("Texture cache RGBA8 linear textures", "", TEXTURE_CACHE_RGBA8_LINEAR_TEXTURES, expected(0..3)),
            int("Texture cache RGBA8 nearest pixels", "px", TEXTURE_CACHE_RGBA8_NEAREST_PIXELS, expected(0..260_000)),
            int("Texture cache RGBA8 nearest textures", "", TEXTURE_CACHE_RGBA8_NEAREST_TEXTURES, expected(0..2)),
            float("Texture cache shared mem", "MB", TEXTURE_CACHE_SHARED_MEM, expected(0.0..100.0)),
            float("Texture cache standalone mem", "MB", TEXTURE_CACHE_STANDALONE_MEM, expected(0.0..100.0)),


            float("Slow frame", "", SLOW_FRAME, expected(0.0..0.0)),
            float("Slow transaction", "", SLOW_TXN, expected(0.0..0.0)),

            float("GPU cache upload", "ms", GPU_CACHE_UPLOAD_TIME, expected(0.0..2.0)),
            float("Texture cache update", "ms", TEXTURE_CACHE_UPDATE_TIME, expected(0.0..3.0)),

            float("Frame", "ms", FRAME_TIME, Expected::none()),

            float("Alpha targets samplers", "%", ALPHA_TARGETS_SAMPLERS, Expected::none()),
            float("Transparent pass samplers", "%", TRANSPARENT_PASS_SAMPLERS, Expected::none()),
            float("Opaque pass samplers", "%", OPAQUE_PASS_SAMPLERS, Expected::none()),
            float("Total samplers", "%", TOTAL_SAMPLERS, Expected::none()),

            int("Interned primitives", "", INTERNED_PRIMITIVES, Expected::none()),
            int("Interned clips", "", INTERNED_CLIPS, Expected::none()),
            int("Interned text runs", "", INTERNED_TEXT_RUNS, Expected::none()),
            int("Interned normal borders", "", INTERNED_NORMAL_BORDERS, Expected::none()),
            int("Interned image borders", "", INTERNED_IMAGE_BORDERS, Expected::none()),
            int("Interned images", "", INTERNED_IMAGES, Expected::none()),
            int("Interned YUV images", "", INTERNED_YUV_IMAGES, Expected::none()),
            int("Interned line decorations", "", INTERNED_LINE_DECORATIONS, Expected::none()),
            int("Interned linear gradients", "", INTERNED_LINEAR_GRADIENTS, Expected::none()),
            int("Interned radial gradients", "", INTERNED_RADIAL_GRADIENTS, Expected::none()),
            int("Interned conic gradients", "", INTERNED_CONIC_GRADIENTS, Expected::none()),
            int("Interned pictures", "", INTERNED_PICTURES, Expected::none()),
            int("Interned filter data", "", INTERNED_FILTER_DATA, Expected::none()),
            int("Interned backdrops", "", INTERNED_BACKDROPS, Expected::none()),
            int("Interned polygons", "", INTERNED_POLYGONS, Expected::none()),

            int("Texture cache RGBA8 glyphs pixels", "px", TEXTURE_CACHE_RGBA8_GLYPHS_PIXELS, expected(0..4_000_000)),
            int("Texture cache RGBA8 glyphs textures", "", TEXTURE_CACHE_RGBA8_GLYPHS_TEXTURES, expected(0..2)),
            int("Texture cache A8 glyphs pixels", "px", TEXTURE_CACHE_A8_GLYPHS_PIXELS, expected(0..4_000_000)),
            int("Texture cache A8 glyphs textures", "", TEXTURE_CACHE_A8_GLYPHS_TEXTURES, expected(0..2)),

            float("Staging CPU allocation", "ms", CPU_TEXTURE_ALLOCATION_TIME, Expected::none()),
            float("Staging GPU allocation", "ms", STAGING_TEXTURE_ALLOCATION_TIME, Expected::none()),
            float("Staging CPU copy", "ms", UPLOAD_CPU_COPY_TIME, Expected::none()),
            float("Staging GPU copy", "ms", UPLOAD_GPU_COPY_TIME, Expected::none()),
            float("Upload time", "ms", UPLOAD_TIME, Expected::none()),
            int("Upload copy batches", "", UPLOAD_NUM_COPY_BATCHES, Expected::none()),
            float("Texture cache upload", "ms", TOTAL_UPLOAD_TIME, expected(0.0..5.0)),
            float("Cache texture creation", "ms", CREATE_CACHE_TEXTURE_TIME, expected(0.0..2.0)),
            float("Cache texture deletion", "ms", DELETE_CACHE_TEXTURE_TIME, expected(0.0..1.0)),

            float("Texture cache RGBA8 linear pressure", "", TEXTURE_CACHE_COLOR8_LINEAR_PRESSURE, expected(0.0..1.0)),
            float("Texture cache RGBA8 nearest pressure", "", TEXTURE_CACHE_COLOR8_NEAREST_PRESSURE, expected(0.0..1.0)),
            float("Texture cache RGBA8 glyphs pressure", "", TEXTURE_CACHE_COLOR8_GLYPHS_PRESSURE, expected(0.0..1.0)),
            float("Texture cache A8 pressure", "", TEXTURE_CACHE_ALPHA8_PRESSURE, expected(0.0..1.0)),
            float("Texture cache A8 glyphs pressure", "", TEXTURE_CACHE_ALPHA8_GLYPHS_PRESSURE, expected(0.0..1.0)),
            float("Texture cache A16 pressure", "", TEXTURE_CACHE_ALPHA16_PRESSURE, expected(0.0..1.0)),
            float("Texture cache standalone pressure", "", TEXTURE_CACHE_STANDALONE_PRESSURE, expected(0.0..1.0)),
            int("Texture cache eviction count", "items", TEXTURE_CACHE_EVICTION_COUNT, Expected::none()),
            int("Texture cache youngest evicted", "frames", TEXTURE_CACHE_YOUNGEST_EVICTION, Expected::none()),
        ];

        let mut counters = Vec::with_capacity(profile_counters.len());

        for (idx, descriptor) in profile_counters.iter().enumerate() {
            debug_assert_eq!(descriptor.index, idx);
            counters.push(Counter::new(descriptor));
        }

        Profiler {
            gpu_frames: ProfilerFrameCollection::new(),
            frame_stats: ProfilerFrameCollection::new(),

            counters,
            start: precise_time_ns(),
            avg_over_period: ONE_SECOND_NS / 2,

            num_graph_samples: 500, // Would it be useful to control this via a pref?
            frame_timestamps_within_last_second: Vec::new(),
            ui: Vec::new(),
        }
    }

    /// Sum a few counters and if the total amount is larger than a threshold, update
    /// a specific counter.
    ///
    /// This is useful to monitor slow frame and slow transactions.
    fn update_slow_event(&mut self, dst_counter: usize, counters: &[usize], threshold: f64) {
        let mut total = 0.0;
        for &counter in counters {
            if self.counters[counter].value.is_finite() {
                total += self.counters[counter].value;
            }
        }

        if total > threshold {
            self.counters[dst_counter].set(total);
        }
    }

    // Call at the end of every frame, after setting the counter values and before drawing the counters.
    pub fn update(&mut self) {
        let now = precise_time_ns();
        let update_avg = (now - self.start) > self.avg_over_period;
        if update_avg {
            self.start = now;
        }
        let one_second_ago = now - ONE_SECOND_NS;
        self.frame_timestamps_within_last_second.retain(|t| *t > one_second_ago);
        self.frame_timestamps_within_last_second.push(now);

        self.update_slow_event(
            SLOW_FRAME,
            &[TOTAL_FRAME_CPU_TIME],
            15.0,
        );
        self.update_slow_event(
            SLOW_TXN,
            &[DISPLAY_LIST_BUILD_TIME, CONTENT_SEND_TIME, SCENE_BUILD_TIME],
            80.0
        );

        for counter in &mut self.counters {
            counter.update(update_avg);
        }
    }

    pub fn update_frame_stats(&mut self, stats: FullFrameStats) {
        if stats.gecko_display_list_time != 0.0 {
          self.frame_stats.push(stats.into());
        }
    }

    pub fn set_gpu_time_queries(&mut self, gpu_queries: Vec<GpuTimer>) {
        let mut gpu_time_ns = 0;
        for sample in &gpu_queries {
            gpu_time_ns += sample.time_ns;
        }

        self.gpu_frames.push(ProfilerFrame {
          total_time: gpu_time_ns,
          samples: gpu_queries
        });

        self.counters[GPU_TIME].set_f64(ns_to_ms(gpu_time_ns));
    }

    // Find the index of a counter by its name.
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.counters.iter().position(|counter| counter.name == name)
    }

    // Define the profiler UI, see comment about the syntax at the top of this file.
    pub fn set_ui(&mut self, names: &str) {
        let mut selection = Vec::new();

        self.append_to_ui(&mut selection, names);

        if selection == self.ui {
            return;
        }

        for counter in &mut self.counters {
            counter.disable_graph();
        }

        for item in &selection {
            if let Item::Graph(idx) = item {
                self.counters[*idx].enable_graph(self.num_graph_samples);
            }
        }

        self.ui = selection;
    }

    fn append_to_ui(&mut self, selection: &mut Vec<Item>, names: &str) {
        // Group successive counters together.
        fn flush_counters(counters: &mut Vec<usize>, selection: &mut Vec<Item>) {
            if !counters.is_empty() {
                selection.push(Item::Counters(std::mem::take(counters)))
            }
        }

        let mut counters = Vec::new();

        for name in names.split(",") {
            let name = name.trim();
            let is_graph = name.starts_with("#");
            let is_indicator = name.starts_with("*");
            let name = if is_graph || is_indicator {
                &name[1..]
            } else {
                name
            };
            // See comment about the ui string syntax at the top of this file.
            match name {
                "" => {
                    flush_counters(&mut counters, selection);
                    selection.push(Item::Space);
                }
                "|" => {
                    flush_counters(&mut counters, selection);
                    selection.push(Item::Column);
                }
                "_" => {
                    flush_counters(&mut counters, selection);
                    selection.push(Item::Row);
                }
                "FPS" => {
                    flush_counters(&mut counters, selection);
                    selection.push(Item::Fps);
                }
                "GPU time queries" => {
                    flush_counters(&mut counters, selection);
                    selection.push(Item::GpuTimeQueries);
                }
                "GPU cache bars" => {
                    flush_counters(&mut counters, selection);
                    selection.push(Item::GpuCacheBars);
                }
                "Paint phase graph" => {
                    flush_counters(&mut counters, selection);
                    selection.push(Item::PaintPhaseGraph);
                }
                _ => {
                    if let Some(idx) = self.index_of(name) {
                        if is_graph {
                            flush_counters(&mut counters, selection);
                            selection.push(Item::Graph(idx));
                        } else if is_indicator {
                            flush_counters(&mut counters, selection);
                            selection.push(Item::ChangeIndicator(idx));
                        } else {
                            counters.push(idx);
                        }
                    } else if let Some(preset_str) = find_preset(name) {
                        flush_counters(&mut counters, selection);
                        self.append_to_ui(selection, preset_str);
                    } else {
                        selection.push(Item::Text(format!("Unknonw counter: {}", name)));
                    }
                }
            }
        }

        flush_counters(&mut counters, selection);
    }

    pub fn set_counters(&mut self, counters: &mut TransactionProfile) {
        for (id, evt) in counters.events.iter_mut().enumerate() {
            if let Event::Value(val) = *evt {
                self.counters[id].set(val);
            }
            *evt = Event::None;
        }
    }

    pub fn get(&self, id: usize) -> Option<f64> {
        self.counters[id].get()
    }

    fn draw_counters(
        counters: &[Counter],
        selected: &[usize],
        mut x: f32, mut y: f32,
        text_buffer: &mut String,
        debug_renderer: &mut DebugRenderer,
    ) -> default::Rect<f32> {
        let line_height = debug_renderer.line_height();

        x += PROFILE_PADDING;
        y += PROFILE_PADDING;
        let origin = default::Point2D::new(x, y);
        y += line_height * 0.5;

        let mut total_rect = Rect::zero();

        let mut color_index = 0;
        let colors = [
            // Regular values,
            ColorU::new(255, 255, 255, 255),
            ColorU::new(255, 255, 0, 255),
            // Unexpected values,
            ColorU::new(255, 80, 0, 255),
            ColorU::new(255, 0, 0, 255),
        ];

        for idx in selected {
            // If The index is invalid, add some vertical space.
            let counter = &counters[*idx];

            let rect = debug_renderer.add_text(
                x, y,
                counter.name,
                colors[color_index],
                None,
            );
            color_index = (color_index + 1) % 2;

            total_rect = total_rect.union(&rect);
            y += line_height;
        }

        color_index = 0;
        x = total_rect.max_x() + 60.0;
        y = origin.y + line_height * 0.5;

        for idx in selected {
            let counter = &counters[*idx];
            let expected_offset = if counter.has_unexpected_avg_max() { 2 } else { 0 };

            counter.write_value(text_buffer);

            let rect = debug_renderer.add_text(
                x,
                y,
                &text_buffer,
                colors[color_index + expected_offset],
                None,
            );
            color_index = (color_index + 1) % 2;

            total_rect = total_rect.union(&rect);
            y += line_height;
        }

        total_rect = total_rect
            .union(&Rect { origin, size: Size2D::new(1.0, 1.0) })
            .inflate(PROFILE_PADDING, PROFILE_PADDING);

        debug_renderer.add_quad(
            total_rect.min_x(),
            total_rect.min_y(),
            total_rect.max_x(),
            total_rect.max_y(),
            BACKGROUND_COLOR,
            BACKGROUND_COLOR,
        );

        total_rect
    }

    fn draw_graph(
        counter: &Counter,
        x: f32,
        y: f32,
        text_buffer: &mut String,
        debug_renderer: &mut DebugRenderer,
    ) -> default::Rect<f32> {
        let graph = counter.graph.as_ref().unwrap();

        let max_samples = graph.values.capacity() as f32;

        let size = Size2D::new(max_samples, 100.0);
        let line_height = debug_renderer.line_height();
        let graph_rect = Rect::new(Point2D::new(x + PROFILE_PADDING, y + PROFILE_PADDING), size);
        let mut rect = graph_rect.inflate(PROFILE_PADDING, PROFILE_PADDING);

        let stats = graph.stats();

        let text_color = ColorU::new(255, 255, 0, 255);
        let text_origin = rect.origin + vec2(rect.size.width, 25.0);
        set_text!(text_buffer, "{} ({})", counter.name, counter.unit);
        debug_renderer.add_text(
            text_origin.x,
            text_origin.y,
            if counter.unit == "" { counter.name } else { text_buffer },
            ColorU::new(0, 255, 0, 255),
            None,
        );

        set_text!(text_buffer, "Samples: {}", stats.samples);

        debug_renderer.add_text(
            text_origin.x,
            text_origin.y + line_height,
            text_buffer,
            text_color,
            None,
        );

        if stats.samples > 0 {
            set_text!(text_buffer, "Min: {:.2} {}", stats.min, counter.unit);
            debug_renderer.add_text(
                text_origin.x,
                text_origin.y + line_height * 2.0,
                text_buffer,
                text_color,
                None,
            );

            set_text!(text_buffer, "Avg: {:.2} {}", stats.avg, counter.unit);
            debug_renderer.add_text(
                text_origin.x,
                text_origin.y + line_height * 3.0,
                text_buffer,
                text_color,
                None,
            );

            set_text!(text_buffer, "Max: {:.2} {}", stats.max, counter.unit);
            debug_renderer.add_text(
                text_origin.x,
                text_origin.y + line_height * 4.0,
                text_buffer,
                text_color,
                None,
            );
        }

        rect.size.width += 220.0;
        debug_renderer.add_quad(
            rect.min_x(),
            rect.min_y(),
            rect.max_x(),
            rect.max_y(),
            BACKGROUND_COLOR,
            BACKGROUND_COLOR,
        );

        let bx1 = graph_rect.max_x();
        let by1 = graph_rect.max_y();

        let w = graph_rect.size.width / max_samples;
        let h = graph_rect.size.height;

        let color_t0 = ColorU::new(0, 255, 0, 255);
        let color_b0 = ColorU::new(0, 180, 0, 255);

        let color_t2 = ColorU::new(255, 0, 0, 255);
        let color_b2 = ColorU::new(180, 0, 0, 255);

        for (index, sample) in graph.values.iter().enumerate() {
            if !sample.is_finite() {
                // NAN means no sample this frame.
                continue;
            }
            let sample = *sample as f32;
            let x1 = bx1 - index as f32 * w;
            let x0 = x1 - w;

            let y0 = by1 - (sample / stats.max as f32) as f32 * h;
            let y1 = by1;

            let (color_top, color_bottom) = if counter.is_unexpected_value(sample as f64) {
                (color_t2, color_b2)
            } else {
                (color_t0, color_b0)
            };

            debug_renderer.add_quad(x0, y0, x1, y1, color_top, color_bottom);
        }

        rect
    }


    fn draw_change_indicator(
        counter: &Counter,
        x: f32, y: f32,
        debug_renderer: &mut DebugRenderer
    ) -> default::Rect<f32> {
        let height = 10.0;
        let width = 20.0;

        // Draw the indicator red instead of blue if is is not within expected ranges.
        let color = if counter.has_unexpected_value() || counter.has_unexpected_avg_max() {
            ColorU::new(255, 20, 20, 255)
        } else {
            ColorU::new(0, 100, 250, 255)
        };

        let tx = counter.change_indicator as f32 * width;
        debug_renderer.add_quad(
            x,
            y,
            x + 15.0 * width,
            y + height,
            ColorU::new(0, 0, 0, 150),
            ColorU::new(0, 0, 0, 150),
        );

        debug_renderer.add_quad(
            x + tx,
            y,
            x + tx + width,
            y + height,
            color,
            ColorU::new(25, 25, 25, 255),
        );

        Rect {
            origin: Point2D::new(x, y),
            size: Size2D::new(15.0 * width + 20.0, height),
        }
    }

    fn draw_bar(
        label: &str,
        label_color: ColorU,
        counters: &[(ColorU, usize)],
        x: f32, y: f32,
        debug_renderer: &mut DebugRenderer,
    ) -> default::Rect<f32> {
        let x = x + 8.0;
        let y = y + 24.0;
        let text_rect = debug_renderer.add_text(
            x, y,
            label,
            label_color,
            None,
        );

        let x_base = text_rect.max_x() + 10.0;
        let width = 300.0;
        let total_value = counters.last().unwrap().1;
        let scale = width / total_value as f32;
        let mut x_current = x_base;

        for &(color, counter) in counters {
            let x_stop = x_base + counter as f32 * scale;
            debug_renderer.add_quad(
                x_current,
                text_rect.origin.y,
                x_stop,
                text_rect.max_y(),
                color,
                color,
            );
            x_current = x_stop;

        }

        let mut total_rect = text_rect;
        total_rect.size.width += width + 10.0;

        total_rect
    }

    fn draw_gpu_cache_bars(&self, x: f32, mut y: f32, text_buffer: &mut String, debug_renderer: &mut DebugRenderer) -> default::Rect<f32> {
        let color_updated = ColorU::new(0xFF, 0, 0, 0xFF);
        let color_free = ColorU::new(0, 0, 0xFF, 0xFF);
        let color_saved = ColorU::new(0, 0xFF, 0, 0xFF);

        let updated_blocks = self.get(GPU_CACHE_BLOCKS_UPDATED).unwrap_or(0.0) as usize;
        let saved_blocks = self.get(GPU_CACHE_BLOCKS_SAVED).unwrap_or(0.0) as usize;
        let allocated_blocks = self.get(GPU_CACHE_BLOCKS_TOTAL).unwrap_or(0.0) as usize;
        let allocated_rows = self.get(GPU_CACHE_ROWS_TOTAL).unwrap_or(0.0) as usize;
        let updated_rows = self.get(GPU_CACHE_ROWS_UPDATED).unwrap_or(0.0) as usize;
        let requested_blocks = updated_blocks + saved_blocks;
        let total_blocks = allocated_rows * MAX_VERTEX_TEXTURE_WIDTH;

        set_text!(text_buffer, "GPU cache rows ({}):", allocated_rows);

        let rect0 = Profiler::draw_bar(
            text_buffer,
            ColorU::new(0xFF, 0xFF, 0xFF, 0xFF),
            &[
                (color_updated, updated_rows),
                (color_free, allocated_rows),
            ],
            x, y,
            debug_renderer,
        );

        y = rect0.max_y();

        let rect1 = Profiler::draw_bar(
            "GPU cache blocks",
            ColorU::new(0xFF, 0xFF, 0, 0xFF),
            &[
                (color_updated, updated_blocks),
                (color_saved, requested_blocks),
                (color_free, allocated_blocks),
                (ColorU::new(0, 0, 0, 0xFF), total_blocks),
            ],
            x, y,
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

        total_rect
    }

    // Draws a frame graph for a given frame collection.
    fn draw_frame_graph(
        frame_collection: &ProfilerFrameCollection,
        x: f32, y: f32,
        debug_renderer: &mut DebugRenderer,
    ) -> default::Rect<f32> {
        let mut has_data = false;
        for frame in &frame_collection.frames {
            if !frame.samples.is_empty() {
                has_data = true;
                break;
            }
        }

        if !has_data {
            return Rect::zero();
        }

        let graph_rect = Rect::new(
            Point2D::new(x + GRAPH_PADDING, y + GRAPH_PADDING),
            Size2D::new(GRAPH_WIDTH, GRAPH_HEIGHT),
        );
        let bounding_rect = graph_rect.inflate(GRAPH_PADDING, GRAPH_PADDING);

        debug_renderer.add_quad(
            bounding_rect.origin.x,
            bounding_rect.origin.y,
            bounding_rect.origin.x + bounding_rect.size.width,
            bounding_rect.origin.y + bounding_rect.size.height,
            BACKGROUND_COLOR,
            BACKGROUND_COLOR,
        );

        let w = graph_rect.size.width;
        let mut y0 = graph_rect.origin.y;

        let mut max_time = frame_collection.frames
            .iter()
            .max_by_key(|f| f.total_time)
            .unwrap()
            .total_time as f32;

        // If the max time is lower than 16ms, fix the scale
        // at 16ms so that the graph is easier to interpret.
        let baseline_ns = 16_000_000.0; // 16ms
        max_time = max_time.max(baseline_ns);

        let mut tags_present = FastHashMap::default();

        for frame in &frame_collection.frames {
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

        // If the max time is higher than 16ms, show a vertical line at the
        // 16ms mark.
        if max_time > baseline_ns {
            let x = graph_rect.origin.x + w * baseline_ns as f32 / max_time;
            let height = frame_collection.frames.len() as f32 * GRAPH_FRAME_HEIGHT;

            debug_renderer.add_quad(
                x,
                graph_rect.origin.y,
                x + 4.0,
                graph_rect.origin.y + height,
                ColorU::new(120, 00, 00, 150),
                ColorU::new(120, 00, 00, 100),
            );
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
                BACKGROUND_COLOR,
                BACKGROUND_COLOR,
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

    pub fn draw_profile(
        &mut self,
        _frame_index: u64,
        debug_renderer: &mut DebugRenderer,
        device_size: DeviceIntSize,
    ) {
        let x_start = 20.0;
        let mut y_start = 150.0;
        let default_column_width = 400.0;

        // set_text!(..) into this string instead of using format!(..) to avoid
        // unnecessary allocations.
        let mut text_buffer = String::with_capacity(32);

        let mut column_width = default_column_width;
        let mut max_y = y_start;

        let mut x = x_start;
        let mut y = y_start;

        for elt in &self.ui {
            let rect = match elt {
                Item::Counters(indices) => {
                    Profiler::draw_counters(&self.counters, &indices, x, y, &mut text_buffer, debug_renderer)
                }
                Item::Graph(idx) => {
                    Profiler::draw_graph(&self.counters[*idx], x, y, &mut text_buffer, debug_renderer)
                }
                Item::ChangeIndicator(idx) => {
                    Profiler::draw_change_indicator(&self.counters[*idx], x, y, debug_renderer)
                }
                Item::GpuTimeQueries => {
                    Profiler::draw_frame_graph(&self.gpu_frames, x, y, debug_renderer)
                }
                Item::GpuCacheBars => {
                    self.draw_gpu_cache_bars(x, y, &mut text_buffer, debug_renderer)
                }
                Item::PaintPhaseGraph => {
                    Profiler::draw_frame_graph(&self.frame_stats, x, y, debug_renderer)
                }
                Item::Text(text) => {
                    let p = 10.0;
                    let mut rect = debug_renderer.add_text(
                        x + p,
                        y + p,
                        &text,
                        ColorU::new(255, 255, 255, 255),
                        None,
                    );
                    rect = rect.inflate(p, p);

                    debug_renderer.add_quad(
                        rect.origin.x,
                        rect.origin.y,
                        rect.max_x(),
                        rect.max_y(),
                        BACKGROUND_COLOR,
                        BACKGROUND_COLOR,
                    );

                    rect
                }
                Item::Fps => {
                    let fps = self.frame_timestamps_within_last_second.len();
                    set_text!(&mut text_buffer, "{} fps", fps);
                    let mut rect = debug_renderer.add_text(
                        x + PROFILE_PADDING,
                        y + PROFILE_PADDING + 5.0,
                        &text_buffer,
                        ColorU::new(255, 255, 255, 255),
                        None,
                    );
                    rect = rect.inflate(PROFILE_PADDING, PROFILE_PADDING);

                    debug_renderer.add_quad(
                        rect.min_x(),
                        rect.min_y(),
                        rect.max_x(),
                        rect.max_y(),
                        BACKGROUND_COLOR,
                        BACKGROUND_COLOR,
                    );

                    rect
                }
                Item::Space => {
                    Rect { origin: Point2D::new(x, y), size: Size2D::new(0.0, PROFILE_SPACING) }
                }
                Item::Column => {
                    max_y = max_y.max(y);
                    x += column_width + PROFILE_SPACING;
                    y = y_start;
                    column_width = default_column_width;

                    continue;
                }
                Item::Row => {
                    max_y = max_y.max(y);
                    y_start = max_y + PROFILE_SPACING;
                    y = y_start;
                    x = x_start;
                    column_width = default_column_width;

                    continue;
                }
            };

            column_width = column_width.max(rect.size.width);
            y = rect.max_y();

            if y > device_size.height as f32 - 100.0 {
                max_y = max_y.max(y);
                x += column_width + PROFILE_SPACING;
                y = y_start;
                column_width = default_column_width;
            }
        }
    }

    #[cfg(feature = "capture")]
    pub fn dump_stats(&self, sink: &mut dyn std::io::Write) -> std::io::Result<()> {
        for counter in &self.counters {
            if counter.value.is_finite() {
                writeln!(sink, "{} {:?}{}", counter.name, counter.value, counter.unit)?;
            }
        }

        Ok(())
    }
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

/// Ranges of expected value for a profile counter.
#[derive(Clone, Debug)]
pub struct Expected<T> {
    pub range: Option<Range<T>>,
    pub avg: Option<Range<T>>,
}

impl<T> Expected<T> {
     const fn none() -> Self {
        Expected {
            range: None,
            avg: None,
        }
    }
}

const fn expected<T>(range: Range<T>) -> Expected<T> {
    Expected {
        range: Some(range),
        avg: None,
    }
}

impl Expected<f64> {
    const fn avg(mut self, avg: Range<f64>) -> Self {
        self.avg = Some(avg);
        self
    }
}

impl Expected<i64> {
    const fn avg(mut self, avg: Range<i64>) -> Self {
        self.avg = Some(avg);
        self
    }

    fn into_float(self) -> Expected<f64> {
        Expected {
            range: match self.range {
                Some(r) => Some(r.start as f64 .. r.end as f64),
                None => None,
            },
            avg: match self.avg {
                Some(r) => Some(r.start as f64 .. r.end as f64),
                None => None,
            },
        }
    }
}

pub struct CounterDescriptor {
    pub name: &'static str,
    pub unit: &'static str,
    pub index: usize,
    pub show_as: ShowAs,
    pub expected: Expected<f64>,
}

#[derive(Debug)]
pub struct Counter {
    pub name: &'static str,
    pub unit: &'static str,
    pub show_as: ShowAs,
    pub expected: Expected<f64>,

    ///
    value: f64,
    /// Number of samples in the current time slice.
    num_samples: u64,
    /// Sum of the values recorded during the current time slice.
    sum: f64,
    /// The max value in in-progress time slice.
    next_max: f64,
    /// The max value of the previous time slice (displayed).
    max: f64,
    /// The average value of the previous time slice (displayed).
    avg: f64,
    /// Incremented when the counter changes.
    change_indicator: u8,

    /// Only used to check that the constants match the real index.
    index: usize,

    graph: Option<Graph>,
}

impl Counter {
    pub fn new(descriptor: &CounterDescriptor) -> Self {
        Counter {
            name: descriptor.name,
            unit: descriptor.unit,
            show_as: descriptor.show_as,
            expected: descriptor.expected.clone(),
            index: descriptor.index,
            value: std::f64::NAN,
            num_samples: 0,
            sum: 0.0,
            next_max: 0.0,
            max: 0.0,
            avg: 0.0,
            change_indicator: 0,
            graph: None,
        }
    }
    pub fn set_f64(&mut self, val: f64) {
        self.value = val;
    }

    pub fn set<T>(&mut self, val: T) where T: Into<f64> {
        self.set_f64(val.into());
    }

    pub fn get(&self) -> Option<f64> {
        if self.value.is_finite() {
            Some(self.value)
        } else {
            None
        }
    }

    pub fn write_value(&self, output: &mut String) {
        match self.show_as {
            ShowAs::Float => {
                set_text!(output, "{:.2} {} (max: {:.2})", self.avg, self.unit, self.max);
            }
            ShowAs::Int => {
                set_text!(output, "{:.0} {} (max: {:.0})", self.avg.round(), self.unit, self.max.round());
            }
        }
    }

    pub fn enable_graph(&mut self, max_samples: usize) {
        if self.graph.is_some() {
            return;
        }

        self.graph = Some(Graph::new(max_samples));
    }

    pub fn disable_graph(&mut self) {
        self.graph = None;
    }

    pub fn is_unexpected_value(&self, value: f64) -> bool {
        if let Some(range) = &self.expected.range {
            return value.is_finite() && value >= range.end;
        }

        false
    }

    pub fn has_unexpected_value(&self) -> bool {
        self.is_unexpected_value(self.value)
    }

    pub fn has_unexpected_avg_max(&self) -> bool {
        if let Some(range) = &self.expected.range {
            if self.max.is_finite() && self.max >= range.end {
                return true;
            }
        }

        if let Some(range) = &self.expected.avg {
            if self.avg < range.start || self.avg >= range.end {
                return true;
            }
        }

        false
    }

    fn update(&mut self, update_avg: bool) {
        let updated = self.value.is_finite();
        if updated {
            self.next_max = self.next_max.max(self.value);
            self.sum += self.value;
            self.num_samples += 1;
            self.change_indicator = (self.change_indicator + 1) % 15;
        }

        if let Some(graph) = &mut self.graph {
            graph.set(self.value);
        }

        self.value = std::f64::NAN;

        if update_avg && self.num_samples > 0 {
            self.avg = self.sum / self.num_samples as f64;
            self.max = self.next_max;
            self.sum = 0.0;
            self.num_samples = 0;
            self.next_max = std::f64::MIN;
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Event {
    Start(f64),
    Value(f64),
    None,
}

// std::convert::From/TryFrom can't deal with integer to f64 so we roll our own...
pub trait EventValue {
    fn into_f64(self) -> f64;
}

impl EventValue for f64 { fn into_f64(self) -> f64 { self } }
impl EventValue for f32 { fn into_f64(self) -> f64 { self as f64 } }
impl EventValue for u32 { fn into_f64(self) -> f64 { self as f64 } }
impl EventValue for i32 { fn into_f64(self) -> f64 { self as f64 } }
impl EventValue for u64 { fn into_f64(self) -> f64 { self as f64 } }
impl EventValue for usize { fn into_f64(self) -> f64 { self as f64 } }

/// A container for profiling information that moves along the rendering pipeline
/// and is handed off to the profiler at the end.
pub struct TransactionProfile {
    pub events: Vec<Event>,
}

impl TransactionProfile {
    pub fn new() -> Self {
        TransactionProfile {
            events: vec![Event::None; NUM_PROFILER_EVENTS],
        }
    }

    pub fn start_time(&mut self, id: usize) {
        let ms = ns_to_ms(precise_time_ns());
        self.events[id] = Event::Start(ms);
    }

    pub fn end_time(&mut self, id: usize) -> f64 {
        self.end_time_if_started(id).unwrap()
    }

    /// Similar to end_time, but doesn't panic if not matched with start_time.
    pub fn end_time_if_started(&mut self, id: usize) -> Option<f64> {
        if let Event::Start(start) = self.events[id] {
            let time = ns_to_ms(precise_time_ns()) - start;
            self.events[id] = Event::Value(time);

            Some(time)
        } else {
            None
        }
    }

    pub fn set<T>(&mut self, id: usize, value: T) where T: EventValue {
        self.set_f64(id, value.into_f64());
    }


    pub fn set_f64(&mut self, id: usize, value: f64) {
        self.events[id] = Event::Value(value);
    }

    pub fn get(&self, id: usize) -> Option<f64> {
        if let Event::Value(val) = self.events[id] {
            Some(val)
        } else {
            None
        }
    }

    pub fn get_or(&self, id: usize, or: f64) -> f64 {
        self.get(id).unwrap_or(or)
    }

    pub fn add<T>(&mut self, id: usize, n: T) where T: EventValue {
        let n = n.into_f64();

        let evt = &mut self.events[id];

        let val = match *evt {
            Event::Value(v) => v + n,
            Event::None => n,
            Event::Start(..) => { panic!(); }
        };

        *evt = Event::Value(val);
    }

    pub fn inc(&mut self, id: usize) {
        self.add(id, 1.0);
    }

    pub fn take(&mut self) -> Self {
        TransactionProfile {
            events: std::mem::take(&mut self.events),
        }
    }

    pub fn take_and_reset(&mut self) -> Self {
        let events = std::mem::take(&mut self.events);

        *self = TransactionProfile::new();

        TransactionProfile { events }
    }

    pub fn merge(&mut self, other: &mut Self) {
        for i in 0..self.events.len() {
            match (self.events[i], other.events[i]) {
                (Event::Value(v1), Event::Value(v2)) => {
                    self.events[i] = Event::Value(v1.max(v2));
                }
                (Event::Value(_), _) => {}
                (_, Event::Value(v2)) => {
                    self.events[i] = Event::Value(v2);
                }
                (Event::None, evt) => {
                    self.events[i] = evt;
                }
                (Event::Start(..), Event::Start(s)) => {
                    self.events[i] = Event::Start(s);
                }
                _=> {}
            }
            other.events[i] = Event::None;
        }
    }

    pub fn clear(&mut self) {
        for evt in &mut self.events {
            *evt = Event::None;
        }
    }
}

#[derive(Debug)]
pub struct GraphStats {
    pub min: f64,
    pub avg: f64,
    pub max: f64,
    pub sum: f64,
    pub samples: usize,
}

#[derive(Debug)]
pub struct Graph {
    values: VecDeque<f64>,
}

impl Graph {
    fn new(max_samples: usize) -> Self {
        let mut values = VecDeque::new();
        values.reserve(max_samples);

        Graph { values }
    }

    fn set(&mut self, val: f64) {
        if self.values.len() == self.values.capacity() {
            self.values.pop_back();
        }
        self.values.push_front(val);
    }

    pub fn stats(&self) -> GraphStats {
        let mut stats = GraphStats {
            min: f64::MAX,
            avg: 0.0,
            max: -f64::MAX,
            sum: 0.0,
            samples: 0,
        };

        let mut samples = 0;
        for value in &self.values {
            if value.is_finite() {
                stats.min = stats.min.min(*value);
                stats.max = stats.max.max(*value);
                stats.sum += *value;
                samples += 1;
            }
        }

        if samples > 0 {
            stats.avg = stats.sum / samples as f64;
            stats.samples = samples;
        }

        stats
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShowAs {
    Float,
    Int,
}

struct ProfilerFrame {
    total_time: u64,
    samples: Vec<GpuTimer>,
}

struct ProfilerFrameCollection {
    frames: VecDeque<ProfilerFrame>,
}

impl ProfilerFrameCollection {
    fn new() -> Self {
        ProfilerFrameCollection {
            frames: VecDeque::new(),
        }
    }

    fn push(&mut self, frame: ProfilerFrame) {
        if self.frames.len() == 20 {
            self.frames.pop_back();
        }
        self.frames.push_front(frame);
    }
}

impl From<FullFrameStats> for ProfilerFrame {
  fn from(stats: FullFrameStats) -> ProfilerFrame {
    let new_sample = |time, label, color| -> GpuTimer {
      let tag = GpuProfileTag {
        label,
        color
      };

      let time_ns = ms_to_ns(time);

      GpuTimer {
        tag, time_ns
      }
    };

    let samples = vec![
      new_sample(stats.gecko_display_list_time, "Gecko DL", ColorF { r: 0.0, g: 1.0, b: 0.0, a: 1.0 }),
      new_sample(stats.wr_display_list_time, "WR DL", ColorF { r: 0.0, g: 1.0, b: 1.0, a: 1.0 }),
      new_sample(stats.scene_build_time, "Scene Build", ColorF { r: 1.0, g: 0.0, b: 1.0, a: 1.0 }),
      new_sample(stats.frame_build_time, "Frame Build", ColorF { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }),
    ];

    ProfilerFrame {
      total_time: ms_to_ns(stats.total()),
      samples
    }
  }
}

pub fn ns_to_ms(ns: u64) -> f64 {
    ns as f64 / 1_000_000.0
}

pub fn ms_to_ns(ms: f64) -> u64 {
  (ms * 1_000_000.0) as u64
}

pub fn bytes_to_mb(bytes: usize) -> f64 {
    bytes as f64 / 1_000_000.0
}

#[derive(Debug, PartialEq)]
enum Item {
    Counters(Vec<usize>),
    Graph(usize),
    ChangeIndicator(usize),
    Fps,
    GpuTimeQueries,
    GpuCacheBars,
    PaintPhaseGraph,
    Text(String),
    Space,
    Column,
    Row,
}

