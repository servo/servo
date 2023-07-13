/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*!
A GPU based renderer for the web.

It serves as an experimental render backend for [Servo](https://servo.org/),
but it can also be used as such in a standalone application.

# External dependencies
WebRender currently depends on [FreeType](https://www.freetype.org/)

# Api Structure
The main entry point to WebRender is the [`crate::Renderer`].

By calling [`Renderer::new(...)`](crate::Renderer::new) you get a [`Renderer`], as well as
a [`RenderApiSender`](api::RenderApiSender). Your [`Renderer`] is responsible to render the
previously processed frames onto the screen.

By calling [`yourRenderApiSender.create_api()`](api::RenderApiSender::create_api), you'll
get a [`RenderApi`](api::RenderApi) instance, which is responsible for managing resources
and documents. A worker thread is used internally to untie the workload from the application
thread and therefore be able to make better use of multicore systems.

## Frame

What is referred to as a `frame`, is the current geometry on the screen.
A new Frame is created by calling [`set_display_list()`](api::Transaction::set_display_list)
on the [`RenderApi`](api::RenderApi). When the geometry is processed, the application will be
informed via a [`RenderNotifier`](api::RenderNotifier), a callback which you pass to
[`Renderer::new`].
More information about [stacking contexts][stacking_contexts].

[`set_display_list()`](api::Transaction::set_display_list) also needs to be supplied with
[`BuiltDisplayList`](api::BuiltDisplayList)s. These are obtained by finalizing a
[`DisplayListBuilder`](api::DisplayListBuilder). These are used to draw your geometry. But it
doesn't only contain trivial geometry, it can also store another
[`StackingContext`](api::StackingContext), as they're nestable.

[stacking_contexts]: https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Positioning/Understanding_z_index/The_stacking_context
*/

#![cfg_attr(feature = "cargo-clippy", allow(clippy::unreadable_literal, clippy::new_without_default, clippy::too_many_arguments))]


// Cribbed from the |matches| crate, for simplicity.
macro_rules! matches {
    ($expression:expr, $($pattern:tt)+) => {
        match $expression {
            $($pattern)+ => true,
            _ => false
        }
    }
}

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate cstr;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate malloc_size_of_derive;
#[cfg(any(feature = "serde"))]
#[macro_use]
extern crate serde;
#[macro_use]
extern crate tracy_rs;
#[macro_use]
extern crate derive_more;
extern crate malloc_size_of;
extern crate svg_fmt;

#[macro_use]
mod profiler;

mod batch;
mod border;
mod box_shadow;
#[cfg(any(feature = "capture", feature = "replay"))]
mod capture;
mod clip;
mod space;
mod spatial_tree;
mod composite;
mod compositor;
mod debug_colors;
mod debug_font_data;
mod debug_item;
mod device;
mod ellipse;
mod filterdata;
mod frame_builder;
mod freelist;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod gamma_lut;
mod glyph_cache;
mod glyph_rasterizer;
mod gpu_cache;
mod gpu_types;
mod hit_test;
mod internal_types;
mod lru_cache;
mod picture;
mod prepare;
mod prim_store;
mod print_tree;
mod render_backend;
mod render_target;
mod render_task_graph;
mod render_task_cache;
mod render_task;
mod renderer;
mod resource_cache;
mod scene;
mod scene_builder_thread;
mod scene_building;
mod screen_capture;
mod segment;
mod spatial_node;
mod texture_pack;
mod texture_cache;
mod tile_cache;
mod util;
mod visibility;
mod api_resources;
mod image_tiling;
mod image_source;
mod rectangle_occlusion;
pub mod host_utils;

///
pub mod intern;
///
pub mod render_api;

mod shader_source {
    include!(concat!(env!("OUT_DIR"), "/shaders.rs"));
}

mod platform {
    #[cfg(target_os = "macos")]
    pub use crate::platform::macos::font;
    #[cfg(any(target_os = "android", all(unix, not(target_os = "macos"))))]
    pub use crate::platform::unix::font;
    #[cfg(target_os = "windows")]
    pub use crate::platform::windows::font;

    #[cfg(target_os = "macos")]
    pub mod macos {
        pub mod font;
    }
    #[cfg(any(target_os = "android", all(unix, not(target_os = "macos"))))]
    pub mod unix {
        pub mod font;
    }
    #[cfg(target_os = "windows")]
    pub mod windows {
        pub mod font;
    }
}

#[cfg(target_os = "macos")]
extern crate core_foundation;
#[cfg(target_os = "macos")]
extern crate core_graphics;
#[cfg(target_os = "macos")]
extern crate core_text;

#[cfg(all(unix, not(target_os = "macos")))]
extern crate freetype;
#[cfg(all(unix, not(target_os = "macos")))]
extern crate libc;

#[cfg(target_os = "windows")]
extern crate dwrote;

extern crate bincode;
extern crate byteorder;
pub extern crate euclid;
extern crate fxhash;
extern crate gleam;
extern crate num_traits;
extern crate plane_split;
extern crate rayon;
#[cfg(feature = "ron")]
extern crate ron;
#[macro_use]
extern crate smallvec;
extern crate time;
#[cfg(all(feature = "capture", feature = "png"))]
extern crate png;
#[cfg(test)]
extern crate rand;

pub extern crate api;
extern crate webrender_build;

#[doc(hidden)]
pub use crate::composite::{CompositorConfig, Compositor, CompositorCapabilities, CompositorSurfaceTransform};
pub use crate::composite::{NativeSurfaceId, NativeTileId, NativeSurfaceInfo, PartialPresentCompositor};
pub use crate::composite::{MappableCompositor, MappedTileInfo, SWGLCompositeSurfaceInfo};
pub use crate::device::{UploadMethod, VertexUsageHint, get_gl_target, get_unoptimized_shader_source};
pub use crate::device::{ProgramBinary, ProgramCache, ProgramCacheObserver, FormatDesc};
pub use crate::device::Device;
pub use crate::frame_builder::ChasePrimitive;
pub use crate::prim_store::PrimitiveDebugId;
pub use crate::profiler::{ProfilerHooks, set_profiler_hooks};
pub use crate::renderer::{
    AsyncPropertySampler, CpuProfile, DebugFlags, GpuProfile, GraphicsApi,
    GraphicsApiInfo, PipelineInfo, Renderer, RendererError, RendererOptions, RenderResults,
    RendererStats, SceneBuilderHooks, Shaders, SharedShaders, ShaderPrecacheFlags,
    MAX_VERTEX_TEXTURE_WIDTH, ONE_TIME_USAGE_HINT,
};
pub use crate::hit_test::SharedHitTester;
pub use crate::internal_types::FastHashMap;
pub use crate::screen_capture::{AsyncScreenshotHandle, RecordedFrameHandle};
pub use crate::texture_cache::TextureCacheConfig;
pub use api as webrender_api;
pub use webrender_build::shader::ProgramSourceDigest;
pub use crate::picture::{TileDescriptor, TileId, InvalidationReason};
pub use crate::picture::{PrimitiveCompareResult, PrimitiveCompareResultDetail, CompareHelperResult};
pub use crate::picture::{TileNode, TileNodeKind, TileSerializer, TileCacheInstanceSerializer, TileOffset, TileCacheLoggerUpdateLists};
pub use crate::intern::ItemUid;
pub use crate::render_api::*;
pub use crate::tile_cache::{PictureCacheDebugInfo, DirtyTileDebugInfo, TileDebugInfo, SliceDebugInfo};

#[cfg(feature = "sw_compositor")]
pub use crate::compositor::sw_compositor;
