/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use crate::blob;
use crossbeam::sync::chase_lev;
#[cfg(windows)]
use dwrote;
#[cfg(all(unix, not(target_os = "android")))]
use font_loader::system_fonts;
use winit::EventsLoopProxy;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use time;
use webrender;
use webrender::api::*;
use webrender::render_api::*;
use webrender::api::units::*;
use webrender::{DebugFlags, RenderResults, ShaderPrecacheFlags};
use crate::{WindowWrapper, NotifierEvent};

// TODO(gw): This descriptor matches what we currently support for fonts
//           but is quite a mess. We should at least document and
//           use better types for things like the style and stretch.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum FontDescriptor {
    Path { path: PathBuf, font_index: u32 },
    Family { name: String },
    Properties {
        family: String,
        weight: u32,
        style: u32,
        stretch: u32,
    },
}

struct NotifierData {
    events_loop_proxy: Option<EventsLoopProxy>,
    frames_notified: u32,
    timing_receiver: chase_lev::Stealer<time::SteadyTime>,
    verbose: bool,
}

impl NotifierData {
    fn new(
        events_loop_proxy: Option<EventsLoopProxy>,
        timing_receiver: chase_lev::Stealer<time::SteadyTime>,
        verbose: bool,
    ) -> Self {
        NotifierData {
            events_loop_proxy,
            frames_notified: 0,
            timing_receiver,
            verbose,
        }
    }
}

struct Notifier(Arc<Mutex<NotifierData>>);

impl Notifier {
    fn update(&self, check_document: bool) {
        let mut data = self.0.lock();
        let data = data.as_mut().unwrap();
        if check_document {
            match data.timing_receiver.steal() {
                chase_lev::Steal::Data(last_timing) => {
                    data.frames_notified += 1;
                    if data.verbose && data.frames_notified == 600 {
                        let elapsed = time::SteadyTime::now() - last_timing;
                        println!(
                            "frame latency (consider queue depth here): {:3.6} ms",
                            elapsed.num_microseconds().unwrap() as f64 / 1000.
                        );
                        data.frames_notified = 0;
                    }
                }
                _ => {
                    println!("Notified of frame, but no frame was ready?");
                }
            }
        }

        if let Some(ref _elp) = data.events_loop_proxy {
            #[cfg(not(target_os = "android"))]
            let _ = _elp.wakeup();
        }
    }
}

impl RenderNotifier for Notifier {
    fn clone(&self) -> Box<dyn RenderNotifier> {
        Box::new(Notifier(self.0.clone()))
    }

    fn wake_up(&self, _composite_needed: bool) {
        self.update(false);
    }

    fn new_frame_ready(&self, _: DocumentId,
                       scrolled: bool,
                       _composite_needed: bool,
                       _render_time: Option<u64>) {
        self.update(!scrolled);
    }
}

pub trait WrenchThing {
    fn next_frame(&mut self);
    fn prev_frame(&mut self);
    fn do_frame(&mut self, _: &mut Wrench) -> u32;
}

impl WrenchThing for CapturedDocument {
    fn next_frame(&mut self) {}
    fn prev_frame(&mut self) {}
    fn do_frame(&mut self, wrench: &mut Wrench) -> u32 {
        match self.root_pipeline_id.take() {
            Some(root_pipeline_id) => {
                // skip the first frame - to not overwrite the loaded one
                let mut txn = Transaction::new();
                txn.set_root_pipeline(root_pipeline_id);
                wrench.api.send_transaction(self.document_id, txn);
            }
            None => {
                wrench.refresh();
            }
        }
        0
    }
}

pub struct CapturedSequence {
    root: PathBuf,
    frame: usize,
    frame_set: Vec<(u32, u32)>,
}

impl CapturedSequence {
    pub fn new(root: PathBuf, scene_start: u32, frame_start: u32) -> Self {
        // Build set of a scene and frame IDs.
        let mut scene = scene_start;
        let mut frame = frame_start;
        let mut frame_set = Vec::new();
        while Self::scene_root(&root, scene).as_path().is_dir() {
            while Self::frame_root(&root, scene, frame).as_path().is_dir() {
                frame_set.push((scene, frame));
                frame += 1;
            }
            scene += 1;
            frame = 1;
        }

        assert!(!frame_set.is_empty());

        Self {
            root,
            frame: 0,
            frame_set,
        }
    }

    fn scene_root(root: &PathBuf, scene: u32) -> PathBuf {
        let path = format!("scenes/{:05}", scene);
        root.join(path)
    }

    fn frame_root(root: &PathBuf, scene: u32, frame: u32) -> PathBuf {
        let path = format!("scenes/{:05}/frames/{:05}", scene, frame);
        root.join(path)
    }
}

impl WrenchThing for CapturedSequence {
    fn next_frame(&mut self) {
        if self.frame + 1 < self.frame_set.len() {
            self.frame += 1;
        }
    }

    fn prev_frame(&mut self) {
        if self.frame > 0 {
            self.frame -= 1;
        }
    }

    fn do_frame(&mut self, wrench: &mut Wrench) -> u32 {
        let mut documents = wrench.api.load_capture(self.root.clone(), Some(self.frame_set[self.frame]));
        println!("loaded {:?} from {:?}",
            documents.iter().map(|cd| cd.document_id).collect::<Vec<_>>(),
            self.frame_set[self.frame]);
        let captured = documents.swap_remove(0);
        wrench.document_id = captured.document_id;
        self.frame as u32
    }
}

pub struct Wrench {
    window_size: DeviceIntSize,
    pub device_pixel_ratio: f32,
    page_zoom_factor: ZoomFactor,

    pub renderer: webrender::Renderer,
    pub api: RenderApi,
    pub document_id: DocumentId,
    pub root_pipeline_id: PipelineId,

    window_title_to_set: Option<String>,

    graphics_api: webrender::GraphicsApiInfo,

    pub rebuild_display_lists: bool,
    pub verbose: bool,

    pub frame_start_sender: chase_lev::Worker<time::SteadyTime>,

    pub callbacks: Arc<Mutex<blob::BlobCallbacks>>,
}

impl Wrench {
    pub fn new(
        window: &mut WindowWrapper,
        proxy: Option<EventsLoopProxy>,
        shader_override_path: Option<PathBuf>,
        use_optimized_shaders: bool,
        dp_ratio: f32,
        size: DeviceIntSize,
        do_rebuild: bool,
        no_subpixel_aa: bool,
        verbose: bool,
        no_scissor: bool,
        no_batch: bool,
        precache_shaders: bool,
        disable_dual_source_blending: bool,
        zoom_factor: f32,
        chase_primitive: webrender::ChasePrimitive,
        dump_shader_source: Option<String>,
        notifier: Option<Box<dyn RenderNotifier>>,
    ) -> Self {
        println!("Shader override path: {:?}", shader_override_path);

        let mut debug_flags = DebugFlags::ECHO_DRIVER_MESSAGES;
        debug_flags.set(DebugFlags::DISABLE_BATCHING, no_batch);
        let callbacks = Arc::new(Mutex::new(blob::BlobCallbacks::new()));

        let precache_flags = if precache_shaders {
            ShaderPrecacheFlags::FULL_COMPILE
        } else {
            ShaderPrecacheFlags::empty()
        };

        let opts = webrender::RendererOptions {
            device_pixel_ratio: dp_ratio,
            resource_override_path: shader_override_path,
            use_optimized_shaders,
            enable_subpixel_aa: !no_subpixel_aa,
            debug_flags,
            enable_clear_scissor: !no_scissor,
            max_recorded_profiles: 16,
            precache_flags,
            blob_image_handler: Some(Box::new(blob::CheckerboardRenderer::new(callbacks.clone()))),
            chase_primitive,
            testing: true,
            max_internal_texture_size: Some(8196), // Needed for rawtest::test_resize_image.
            allow_dual_source_blending: !disable_dual_source_blending,
            allow_advanced_blend_equation: window.is_software(),
            dump_shader_source,
            // SWGL doesn't support the GL_ALWAYS depth comparison function used by
            // `clear_caches_with_quads`, but scissored clears work well.
            clear_caches_with_quads: !window.is_software(),
            ..Default::default()
        };

        // put an Awakened event into the queue to kick off the first frame
        if let Some(ref _elp) = proxy {
            #[cfg(not(target_os = "android"))]
            let _ = _elp.wakeup();
        }

        let (timing_sender, timing_receiver) = chase_lev::deque();
        let notifier = notifier.unwrap_or_else(|| {
            let data = Arc::new(Mutex::new(NotifierData::new(proxy, timing_receiver, verbose)));
            Box::new(Notifier(data))
        });

        let (renderer, sender) = webrender::Renderer::new(
            window.clone_gl(),
            notifier,
            opts,
            None,
        ).unwrap();

        let api = sender.create_api();
        let document_id = api.add_document(size);

        let graphics_api = renderer.get_graphics_api_info();
        let zoom_factor = ZoomFactor::new(zoom_factor);

        let mut wrench = Wrench {
            window_size: size,

            renderer,
            api,
            document_id,
            window_title_to_set: None,

            rebuild_display_lists: do_rebuild,
            verbose,
            device_pixel_ratio: dp_ratio,
            page_zoom_factor: ZoomFactor::new(0.0),

            root_pipeline_id: PipelineId(0, 0),

            graphics_api,
            frame_start_sender: timing_sender,

            callbacks,
        };

        wrench.set_page_zoom(zoom_factor);
        wrench.set_title("start");
        let mut txn = Transaction::new();
        txn.set_root_pipeline(wrench.root_pipeline_id);
        wrench.api.send_transaction(wrench.document_id, txn);

        wrench
    }

    pub fn set_quality_settings(&mut self, settings: QualitySettings) {
        let mut txn = Transaction::new();
        txn.set_quality_settings(settings);
        self.api.send_transaction(self.document_id, txn);
    }

    pub fn get_page_zoom(&self) -> ZoomFactor {
        self.page_zoom_factor
    }

    pub fn set_page_zoom(&mut self, zoom_factor: ZoomFactor) {
        if self.page_zoom_factor.get() != zoom_factor.get() {
            self.page_zoom_factor = zoom_factor;
            let mut txn = Transaction::new();
            txn.set_page_zoom(self.page_zoom_factor);
            self.api.send_transaction(self.document_id, txn);
            self.set_title("");
        }
    }

    pub fn layout_simple_ascii(
        &mut self,
        font_key: FontKey,
        instance_key: FontInstanceKey,
        text: &str,
        size: f32,
        origin: LayoutPoint,
        flags: FontInstanceFlags,
    ) -> (Vec<u32>, Vec<LayoutPoint>, LayoutRect) {
        // Map the string codepoints to glyph indices in this font.
        // Just drop any glyph that isn't present in this font.
        let indices: Vec<u32> = self.api
            .get_glyph_indices(font_key, text)
            .iter()
            .filter_map(|idx| *idx)
            .collect();

        // Retrieve the metrics for each glyph.
        let metrics = self.api.get_glyph_dimensions(instance_key, indices.clone());

        let mut bounding_rect = LayoutRect::zero();
        let mut positions = Vec::new();

        let mut cursor = origin;
        let direction = if flags.contains(FontInstanceFlags::TRANSPOSE) {
            LayoutVector2D::new(
                0.0,
                if flags.contains(FontInstanceFlags::FLIP_Y) { -1.0 } else { 1.0 },
            )
        } else {
            LayoutVector2D::new(
                if flags.contains(FontInstanceFlags::FLIP_X) { -1.0 } else { 1.0 },
                0.0,
            )
        };
        for metric in metrics {
            positions.push(cursor);

            match metric {
                Some(metric) => {
                    let glyph_rect = LayoutRect::new(
                        LayoutPoint::new(cursor.x + metric.left as f32, cursor.y - metric.top as f32),
                        LayoutSize::new(metric.width as f32, metric.height as f32)
                    );
                    bounding_rect = bounding_rect.union(&glyph_rect);
                    cursor += direction * metric.advance;
                }
                None => {
                    // Extract the advances from the metrics. The get_glyph_dimensions API
                    // has a limitation that it can't currently get dimensions for non-renderable
                    // glyphs (e.g. spaces), so just use a rough estimate in that case.
                    let space_advance = size / 3.0;
                    cursor += direction * space_advance;
                }
            }
        }

        // The platform font implementations don't always handle
        // the exact dimensions used when subpixel AA is enabled
        // on glyphs. As a workaround, inflate the bounds by
        // 2 pixels on either side, to give a slightly less
        // tight fitting bounding rect.
        let bounding_rect = bounding_rect.inflate(2.0, 2.0);

        (indices, positions, bounding_rect)
    }

    pub fn set_title(&mut self, extra: &str) {
        self.window_title_to_set = Some(format!(
            "Wrench: {} ({}x zoom={}) - {} - {}",
            extra,
            self.device_pixel_ratio,
            self.page_zoom_factor.get(),
            self.graphics_api.renderer,
            self.graphics_api.version
        ));
    }

    pub fn take_title(&mut self) -> Option<String> {
        self.window_title_to_set.take()
    }

    pub fn should_rebuild_display_lists(&self) -> bool {
        self.rebuild_display_lists
    }

    pub fn window_size_f32(&self) -> LayoutSize {
        LayoutSize::new(
            self.window_size.width as f32,
            self.window_size.height as f32,
        )
    }

    #[cfg(target_os = "windows")]
    pub fn font_key_from_native_handle(&mut self, descriptor: &NativeFontHandle) -> FontKey {
        let key = self.api.generate_font_key();
        let mut txn = Transaction::new();
        txn.add_native_font(key, descriptor.clone());
        self.api.send_transaction(self.document_id, txn);
        key
    }

    #[cfg(target_os = "windows")]
    pub fn font_key_from_name(&mut self, font_name: &str) -> FontKey {
        self.font_key_from_properties(
            font_name,
            dwrote::FontWeight::Regular.to_u32(),
            dwrote::FontStyle::Normal.to_u32(),
            dwrote::FontStretch::Normal.to_u32(),
        )
    }

    #[cfg(target_os = "windows")]
    pub fn font_key_from_properties(
        &mut self,
        family: &str,
        weight: u32,
        style: u32,
        stretch: u32,
    ) -> FontKey {
        let weight = dwrote::FontWeight::from_u32(weight);
        let style = dwrote::FontStyle::from_u32(style);
        let stretch = dwrote::FontStretch::from_u32(stretch);
        let desc = dwrote::FontDescriptor {
            family_name: family.to_owned(),
            weight,
            style,
            stretch,
        };
        let system_fc = dwrote::FontCollection::system();
        if let Some(font) = system_fc.get_font_from_descriptor(&desc) {
            let face = font.create_font_face();
            let files = face.get_files();
            if files.len() == 1 {
                if let Some(path) = files[0].get_font_file_path() {
                    return self.font_key_from_native_handle(&NativeFontHandle {
                        path,
                        index: face.get_index(),
                    });
                }
            }
        }
        panic!("failed loading font from properties {:?}", desc)
    }

    #[cfg(all(unix, not(target_os = "android")))]
    pub fn font_key_from_properties(
        &mut self,
        family: &str,
        _weight: u32,
        _style: u32,
        _stretch: u32,
    ) -> FontKey {
        let property = system_fonts::FontPropertyBuilder::new()
            .family(family)
            .build();
        let (font, index) = system_fonts::get(&property).unwrap();
        self.font_key_from_bytes(font, index as u32)
    }

    #[cfg(target_os = "android")]
    pub fn font_key_from_properties(
        &mut self,
        _family: &str,
        _weight: u32,
        _style: u32,
        _stretch: u32,
    ) -> FontKey {
        unimplemented!()
    }

    #[cfg(all(unix, not(target_os = "android")))]
    pub fn font_key_from_name(&mut self, font_name: &str) -> FontKey {
        let property = system_fonts::FontPropertyBuilder::new()
            .family(font_name)
            .build();
        let (font, index) = system_fonts::get(&property).unwrap();
        self.font_key_from_bytes(font, index as u32)
    }

    #[cfg(target_os = "android")]
    pub fn font_key_from_name(&mut self, _font_name: &str) -> FontKey {
        unimplemented!()
    }

    pub fn font_key_from_bytes(&mut self, bytes: Vec<u8>, index: u32) -> FontKey {
        let key = self.api.generate_font_key();
        let mut txn = Transaction::new();
        txn.add_raw_font(key, bytes, index);
        self.api.send_transaction(self.document_id, txn);
        key
    }

    pub fn add_font_instance(&mut self,
        font_key: FontKey,
        size: f32,
        flags: FontInstanceFlags,
        render_mode: Option<FontRenderMode>,
        bg_color: Option<ColorU>,
        synthetic_italics: SyntheticItalics,
    ) -> FontInstanceKey {
        let key = self.api.generate_font_instance_key();
        let mut txn = Transaction::new();
        let mut options: FontInstanceOptions = Default::default();
        options.flags |= flags;
        if let Some(render_mode) = render_mode {
            options.render_mode = render_mode;
        }
        if let Some(bg_color) = bg_color {
            options.bg_color = bg_color;
        }
        options.synthetic_italics = synthetic_italics;
        txn.add_font_instance(key, font_key, size, Some(options), None, Vec::new());
        self.api.send_transaction(self.document_id, txn);
        key
    }

    #[allow(dead_code)]
    pub fn delete_font_instance(&mut self, key: FontInstanceKey) {
        let mut txn = Transaction::new();
        txn.delete_font_instance(key);
        self.api.send_transaction(self.document_id, txn);
    }

    pub fn update(&mut self, dim: DeviceIntSize) {
        if dim != self.window_size {
            self.window_size = dim;
        }
    }

    pub fn begin_frame(&mut self) {
        self.frame_start_sender.push(time::SteadyTime::now());
    }

    pub fn send_lists(
        &mut self,
        frame_number: u32,
        display_lists: Vec<(PipelineId, BuiltDisplayList)>,
        scroll_offsets: &HashMap<ExternalScrollId, LayoutPoint>,
    ) {
        let root_background_color = Some(ColorF::new(1.0, 1.0, 1.0, 1.0));

        let mut txn = Transaction::new();
        for display_list in display_lists {
            txn.set_display_list(
                Epoch(frame_number),
                root_background_color,
                self.window_size_f32(),
                display_list,
                false,
            );
        }

        for (id, offset) in scroll_offsets {
            txn.scroll_node_with_id(*offset, *id, ScrollClamping::NoClamping);
        }

        txn.generate_frame(0);
        self.api.send_transaction(self.document_id, txn);
    }

    pub fn get_frame_profiles(
        &mut self,
    ) -> (Vec<webrender::CpuProfile>, Vec<webrender::GpuProfile>) {
        self.renderer.get_frame_profiles()
    }

    pub fn render(&mut self) -> RenderResults {
        self.renderer.update();
        let _ = self.renderer.flush_pipeline_info();
        self.renderer
            .render(self.window_size, 0)
            .expect("errors encountered during render!")
    }

    pub fn refresh(&mut self) {
        self.begin_frame();
        let mut txn = Transaction::new();
        txn.generate_frame(0);
        self.api.send_transaction(self.document_id, txn);
    }

    pub fn show_onscreen_help(&mut self) {
        let help_lines = [
            "Esc - Quit",
            "H - Toggle help",
            "R - Toggle recreating display items each frame",
            "P - Toggle profiler",
            "O - Toggle showing intermediate targets",
            "I - Toggle showing texture caches",
            "B - Toggle showing alpha primitive rects",
            "V - Toggle showing overdraw",
            "G - Toggle showing gpu cache updates",
            "S - Toggle compact profiler",
            "Q - Toggle GPU queries for time and samples",
            "M - Trigger memory pressure event",
            "T - Save CPU profile to a file",
            "C - Save a capture to captures/wrench/",
            "X - Do a hit test at the current cursor position",
            "Y - Clear all caches",
        ];

        let color_and_offset = [(ColorF::BLACK, 2.0), (ColorF::WHITE, 0.0)];
        self.renderer.device.begin_frame(); // next line might compile shaders:
        let dr = self.renderer.debug_renderer().unwrap();

        for ref co in &color_and_offset {
            let x = self.device_pixel_ratio * (15.0 + co.1);
            let mut y = self.device_pixel_ratio * (15.0 + co.1 + dr.line_height());
            for ref line in &help_lines {
                dr.add_text(x, y, line, co.0.into(), None);
                y += self.device_pixel_ratio * dr.line_height();
            }
        }
        self.renderer.device.end_frame();
    }

    pub fn shut_down(self, rx: Receiver<NotifierEvent>) {
        self.api.shut_down(true);

        loop {
            match rx.recv() {
                Ok(NotifierEvent::ShutDown) => { break; }
                Ok(_) => {}
                Err(e) => { panic!("Did not shut down properly: {:?}.", e); }
            }
        }

        self.renderer.deinit();
    }
}
