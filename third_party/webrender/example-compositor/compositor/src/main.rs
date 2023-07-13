/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*

    An example of how to implement the Compositor trait that
    allows picture caching surfaces to be composited by the operating
    system.

    The current example supports DirectComposite on Windows only.

 */

use euclid::Angle;
use gleam::gl;
use std::ffi::CString;
use std::sync::mpsc;
use webrender::{CompositorSurfaceTransform, Transaction, api::*, euclid::point2};
use webrender::api::units::*;
#[cfg(target_os = "windows")]
use compositor_windows as compositor;
#[cfg(target_os = "linux")]
use compositor_wayland as compositor;
use std::{env, f32, process};

// A very hacky integration with DirectComposite. It proxies calls from the compositor
// interface to a simple C99 library which does the DirectComposition / D3D11 / ANGLE
// interfacing. This is a very unsafe impl due to the way the window pointer is passed
// around!
struct DirectCompositeInterface {
    window: *mut compositor::Window,
}

impl DirectCompositeInterface {
    fn new(window: *mut compositor::Window) -> Self {
        DirectCompositeInterface {
            window,
        }
    }
}

impl webrender::Compositor for DirectCompositeInterface {
    fn create_surface(
        &mut self,
        id: webrender::NativeSurfaceId,
        _virtual_offset: DeviceIntPoint,
        tile_size: DeviceIntSize,
        is_opaque: bool,
    ) {
        compositor::create_surface(
            self.window,
            id.0,
            tile_size.width,
            tile_size.height,
            is_opaque,
        );
    }

    fn destroy_surface(
        &mut self,
        id: webrender::NativeSurfaceId,
    ) {
        compositor::destroy_surface(self.window, id.0);
    }

    fn create_tile(
        &mut self,
        id: webrender::NativeTileId,
    ) {
        compositor::create_tile(
            self.window,
            id.surface_id.0,
            id.x,
            id.y,
        );
    }

    fn destroy_tile(
        &mut self,
        id: webrender::NativeTileId,
    ) {
        compositor::destroy_tile(
            self.window,
            id.surface_id.0,
            id.x,
            id.y,
        );
    }

    fn bind(
        &mut self,
        id: webrender::NativeTileId,
        dirty_rect: DeviceIntRect,
        _valid_rect: DeviceIntRect,
    ) -> webrender::NativeSurfaceInfo {
        let (fbo_id, x, y) = compositor::bind_surface(
            self.window,
            id.surface_id.0,
            id.x,
            id.y,
            dirty_rect.origin.x,
            dirty_rect.origin.y,
            dirty_rect.size.width,
            dirty_rect.size.height,
        );

        webrender::NativeSurfaceInfo {
            origin: DeviceIntPoint::new(x, y),
            fbo_id,
        }
    }

    fn unbind(&mut self) {
        compositor::unbind_surface(self.window);
    }

    fn begin_frame(&mut self) {
        compositor::begin_transaction(self.window);
    }

    fn add_surface(
        &mut self,
        id: webrender::NativeSurfaceId,
        transform: CompositorSurfaceTransform,
        clip_rect: DeviceIntRect,
        _image_rendering: ImageRendering,
    ) {
        compositor::add_surface(
            self.window,
            id.0,
            transform.transform_point2d(point2(0., 0.)).unwrap().x as i32,
            transform.transform_point2d(point2(0., 0.)).unwrap().y as i32,
            clip_rect.origin.x,
            clip_rect.origin.y,
            clip_rect.size.width,
            clip_rect.size.height,
        );
    }

    fn end_frame(&mut self) {
        compositor::end_transaction(self.window);
    }
    fn create_external_surface(&mut self, _: webrender::NativeSurfaceId, _: bool) { todo!() }

    fn attach_external_image(
        &mut self,
        _id: webrender::NativeSurfaceId,
        _external_image: ExternalImageId
    ) {
        todo!()
    }

    fn enable_native_compositor(&mut self, _enable: bool) {
        todo!()
    }

    fn deinit(&mut self) {
        compositor::deinit(self.window);
    }

    fn get_capabilities(&self) -> webrender::CompositorCapabilities {
        webrender::CompositorCapabilities {
            virtual_surface_size: 1024 * 1024,
            ..Default::default()
        }
    }

    fn invalidate_tile(
        &mut self,
        _id: webrender::NativeTileId,
        _valid_rect: DeviceIntRect,
    ) {}

    fn start_compositing(
        &mut self,
        _dirty_rects: &[DeviceIntRect],
        _opaque_rects: &[DeviceIntRect],
    ) {}
}

// Simplisitic implementation of the WR notifier interface to know when a frame
// has been prepared and can be rendered.
struct Notifier {
    tx: mpsc::Sender<()>,
}

impl Notifier {
    fn new(tx: mpsc::Sender<()>) -> Self {
        Notifier {
            tx,
        }
    }
}

impl RenderNotifier for Notifier {
    fn clone(&self) -> Box<dyn RenderNotifier> {
        Box::new(Notifier {
            tx: self.tx.clone()
        })
    }

    fn wake_up(&self, _composite_needed: bool) {
    }

    fn new_frame_ready(&self,
                       _: DocumentId,
                       _scrolled: bool,
                       _composite_needed: bool,
                       _render_time: Option<u64>) {
        self.tx.send(()).ok();
    }
}

fn push_rotated_rect(
    builder: &mut DisplayListBuilder,
    rect: LayoutRect,
    color: ColorF,
    spatial_id: SpatialId,
    root_pipeline_id: PipelineId,
    angle: f32,
    time: f32,
) {
    let color = color.scale_rgb(time);
    let rotation = LayoutTransform::rotation(
        0.0,
        0.0,
        1.0,
        Angle::radians(2.0 * std::f32::consts::PI * angle),
    );
    let transform_origin = LayoutVector3D::new(
        rect.origin.x + rect.size.width * 0.5,
        rect.origin.y + rect.size.height * 0.5,
        0.0,
    );
    let transform = rotation
        .pre_translate(-transform_origin)
        .then_translate(transform_origin);
    let spatial_id = builder.push_reference_frame(
        LayoutPoint::zero(),
        spatial_id,
        TransformStyle::Flat,
        PropertyBinding::Value(transform),
        ReferenceFrameKind::Transform {
            is_2d_scale_translation: false,
            should_snap: false,
        },
    );
    builder.push_rect(
        &CommonItemProperties::new(
            rect,
            SpaceAndClipInfo {
                spatial_id,
                clip_id: ClipId::root(root_pipeline_id),
            },
        ),
        rect,
        color,
    );
}

fn build_display_list(
    builder: &mut DisplayListBuilder,
    scroll_id: ExternalScrollId,
    root_pipeline_id: PipelineId,
    layout_size: LayoutSize,
    time: f32,
    invalidations: Invalidations,
) {
    let size_factor = match invalidations {
        Invalidations::Small => 0.1,
        Invalidations::Large | Invalidations::Scrolling => 1.0,
    };

    let fixed_space_info = SpaceAndClipInfo {
        spatial_id: SpatialId::root_scroll_node(root_pipeline_id),
        clip_id: ClipId::root(root_pipeline_id),
    };

    let scroll_space_info = builder.define_scroll_frame(
        &fixed_space_info,
        scroll_id,
        LayoutRect::new(LayoutPoint::zero(), layout_size),
        LayoutRect::new(LayoutPoint::zero(), layout_size),
        ScrollSensitivity::Script,
        LayoutVector2D::zero(),
    );

    builder.push_rect(
        &CommonItemProperties::new(
            LayoutRect::new(LayoutPoint::zero(), layout_size).inflate(-10.0, -10.0),
            fixed_space_info,
        ),
        LayoutRect::new(LayoutPoint::zero(), layout_size).inflate(-10.0, -10.0),
        ColorF::new(0.8, 0.8, 0.8, 1.0),
    );

    push_rotated_rect(
        builder,
        LayoutRect::new(
            LayoutPoint::new(100.0, 100.0),
            LayoutSize::new(size_factor * 400.0, size_factor * 400.0),
        ),
        ColorF::new(1.0, 0.0, 0.0, 1.0),
        scroll_space_info.spatial_id,
        root_pipeline_id,
        time,
        time,
    );

    push_rotated_rect(
        builder,
        LayoutRect::new(
            LayoutPoint::new(800.0, 100.0),
            LayoutSize::new(size_factor * 100.0, size_factor * 600.0),
        ),
        ColorF::new(0.0, 1.0, 0.0, 1.0),
        fixed_space_info.spatial_id,
        root_pipeline_id,
        0.2,
        time,
    );

    push_rotated_rect(
        builder,
        LayoutRect::new(
            LayoutPoint::new(700.0, 200.0),
            LayoutSize::new(size_factor * 300.0, size_factor * 300.0),
        ),
        ColorF::new(0.0, 0.0, 1.0, 1.0),
        scroll_space_info.spatial_id,
        root_pipeline_id,
        0.1,
        time,
    );

    push_rotated_rect(
        builder,
        LayoutRect::new(
            LayoutPoint::new(100.0, 600.0),
            LayoutSize::new(size_factor * 400.0, size_factor * 400.0),
        ),
        ColorF::new(1.0, 1.0, 0.0, 1.0),
        scroll_space_info.spatial_id,
        root_pipeline_id,
        time,
        time,
    );

    push_rotated_rect(
        builder,
        LayoutRect::new(
            LayoutPoint::new(700.0, 600.0),
            LayoutSize::new(size_factor * 400.0, size_factor * 400.0),
        ),
        ColorF::new(0.0, 1.0, 1.0, 1.0),
        scroll_space_info.spatial_id,
        root_pipeline_id,
        time,
        time,
    );
}

#[derive(Debug, Copy, Clone)]
enum Invalidations {
    Large,
    Small,
    Scrolling,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
enum Sync {
    None = 0,
    Swap = 1,
    Commit = 2,
    Flush = 3,
    Query = 4,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 6 {
        println!("USAGE: compositor [native|none] [small|large|scroll] [none|swap|commit|flush|query] width height");
        process::exit(0);
    }

    let enable_compositor = match args[1].parse::<String>().unwrap().as_str() {
        "native" => true,
        "none" => false,
        _ => panic!("invalid compositor [native, none]"),
    };

    let inv_mode = match args[2].parse::<String>().unwrap().as_str() {
        "small" => Invalidations::Small,
        "large" => Invalidations::Large,
        "scroll" => Invalidations::Scrolling,
        _ => panic!("invalid invalidations [small, large, scroll]"),
    };

    let sync_mode = match args[3].parse::<String>().unwrap().as_str() {
        "none" => Sync::None,
        "swap" => Sync::Swap,
        "commit" => Sync::Commit,
        "flush" => Sync::Flush,
        "query" => Sync::Query,
        _ => panic!("invalid sync mode [none, swap, commit, flush, query]"),
    };

    let width = args[4].parse().unwrap();
    let height = args[5].parse().unwrap();
    let device_size = DeviceIntSize::new(width, height);

    // Load GL, construct WR and the native compositor interface.
    let window = compositor::create_window(
        device_size.width,
        device_size.height,
        enable_compositor,
        sync_mode as i32,
    );
    let debug_flags = DebugFlags::empty();
    let compositor_config = if enable_compositor {
        webrender::CompositorConfig::Native {
            max_update_rects: 1,
            compositor: Box::new(DirectCompositeInterface::new(window)),
        }
    } else {
        webrender::CompositorConfig::Draw {
            max_partial_present_rects: 0,
            draw_previous_partial_present_regions: false,
            partial_present: None,
        }
    };
    let opts = webrender::RendererOptions {
        clear_color: Some(ColorF::new(1.0, 1.0, 1.0, 1.0)),
        debug_flags,
        compositor_config,
        surface_origin_is_top_left: false,
        ..webrender::RendererOptions::default()
    };
    let (tx, rx) = mpsc::channel();
    let notifier = Box::new(Notifier::new(tx));
    let gl = unsafe {
        gl::GlesFns::load_with(
            |symbol| {
                let symbol = CString::new(symbol).unwrap();
                let ptr = compositor::get_proc_address(symbol.as_ptr());
                ptr
            }
        )
    };
    let (mut renderer, sender) = webrender::Renderer::new(
        gl.clone(),
        notifier,
        opts,
        None,
    ).unwrap();
    let mut api = sender.create_api();
    let document_id = api.add_document(device_size);
    let device_pixel_ratio = 1.0;
    let mut current_epoch = Epoch(0);
    let root_pipeline_id = PipelineId(0, 0);
    let layout_size = device_size.to_f32() / euclid::Scale::new(device_pixel_ratio);
    let mut time = 0.0;
    let scroll_id = ExternalScrollId(3, root_pipeline_id);

    // Kick off first transaction which will mean we get a notify below to build the DL and render.
    let mut txn = Transaction::new();
    txn.set_root_pipeline(root_pipeline_id);

    if let Invalidations::Scrolling = inv_mode {
        let mut root_builder = DisplayListBuilder::new(root_pipeline_id);

        build_display_list(
            &mut root_builder,
            scroll_id,
            root_pipeline_id,
            layout_size,
            1.0,
            inv_mode,
        );

        txn.set_display_list(
            current_epoch,
            None,
            layout_size,
            root_builder.finalize(),
            true,
        );
    }

    txn.generate_frame(0);
    api.send_transaction(document_id, txn);

    // Tick the compositor (in this sample, we don't block on UI events)
    while compositor::tick(window) {
        // If there is a new frame ready to draw
        if let Ok(..) = rx.try_recv() {
            // Update and render. This will invoke the native compositor interface implemented above
            // as required.
            renderer.update();
            renderer.render(device_size, 0).unwrap();
            let _ = renderer.flush_pipeline_info();

            // Construct a simple display list that can be drawn and composited by DC.
            let mut txn = Transaction::new();

            match inv_mode {
                Invalidations::Small | Invalidations::Large => {
                    let mut root_builder = DisplayListBuilder::new(root_pipeline_id);

                    build_display_list(
                        &mut root_builder,
                        scroll_id,
                        root_pipeline_id,
                        layout_size,
                        time,
                        inv_mode,
                    );

                    txn.set_display_list(
                        current_epoch,
                        None,
                        layout_size,
                        root_builder.finalize(),
                        true,
                    );
                }
                Invalidations::Scrolling => {
                    let d = 0.5 - 0.5 * (2.0 * f32::consts::PI * 5.0 * time).cos();
                    txn.scroll_node_with_id(
                        LayoutPoint::new(0.0, (d * 100.0).round()),
                        scroll_id,
                        ScrollClamping::NoClamping,
                    );
                }
            }

            txn.generate_frame(0);
            api.send_transaction(document_id, txn);
            current_epoch.0 += 1;
            time += 0.001;
            if time > 1.0 {
                time = 0.0;
            }

            // This does nothing when native compositor is enabled
            compositor::swap_buffers(window);
        }
    }

    renderer.deinit();
    compositor::destroy_window(window);
}
