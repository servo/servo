/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::NotifierEvent;
use crate::WindowWrapper;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use crate::wrench::{Wrench, WrenchThing};
use crate::yaml_frame_reader::YamlFrameReader;
use webrender::{PictureCacheDebugInfo, TileDebugInfo};
use webrender::api::units::*;

pub struct TestHarness<'a> {
    wrench: &'a mut Wrench,
    window: &'a mut WindowWrapper,
    rx: Receiver<NotifierEvent>,
}

struct RenderResult {
    pc_debug: PictureCacheDebugInfo,
    composite_needed: bool,
}

// Convenience method to build a picture rect
fn pr(x: f32, y: f32, w: f32, h: f32) -> PictureRect {
    PictureRect::new(
        PicturePoint::new(x, y),
        PictureSize::new(w, h),
    )
}

impl<'a> TestHarness<'a> {
    pub fn new(
        wrench: &'a mut Wrench,
        window: &'a mut WindowWrapper,
        rx: Receiver<NotifierEvent>
    ) -> Self {
        TestHarness {
            wrench,
            window,
            rx,
        }
    }

    /// Main entry point for invalidation tests
    pub fn run(
        mut self,
    ) {
        // List all invalidation tests here
        self.test_basic();
        self.test_composite_nop();
    }

    /// Simple validation / proof of concept of invalidation testing
    fn test_basic(
        &mut self,
    ) {
        // Render basic.yaml, ensure that the valid/dirty rects are as expected
        let results = self.render_yaml("basic");
        let tile_info = results.pc_debug.slice(0).tile(0, 0).as_dirty();
        assert_eq!(
            tile_info.local_valid_rect,
            pr(100.0, 100.0, 500.0, 100.0),
        );
        assert_eq!(
            tile_info.local_dirty_rect,
            pr(100.0, 100.0, 500.0, 100.0),
        );

        // Render it again and ensure the tile was considered valid (no rasterization was done)
        let results = self.render_yaml("basic");
        assert_eq!(*results.pc_debug.slice(0).tile(0, 0), TileDebugInfo::Valid);
    }

    /// Ensure WR detects composites are needed for position changes within a single tile.
    fn test_composite_nop(
        &mut self,
    ) {
        // Render composite_nop_1.yaml, ensure that the valid/dirty rects are as expected
        let results = self.render_yaml("composite_nop_1");
        let tile_info = results.pc_debug.slice(0).tile(0, 0).as_dirty();
        assert_eq!(
            tile_info.local_valid_rect,
            pr(100.0, 100.0, 100.0, 100.0),
        );
        assert_eq!(
            tile_info.local_dirty_rect,
            pr(100.0, 100.0, 100.0, 100.0),
        );

        // Render composite_nop_2.yaml, ensure that the valid/dirty rects are as expected
        let results = self.render_yaml("composite_nop_2");
        let tile_info = results.pc_debug.slice(0).tile(0, 0).as_dirty();
        assert_eq!(
            tile_info.local_valid_rect,
            pr(100.0, 120.0, 100.0, 100.0),
        );
        assert_eq!(
            tile_info.local_dirty_rect,
            pr(100.0, 120.0, 100.0, 100.0),
        );

        // Main part of this test - ensure WR detects a composite is required in this case
        assert!(results.composite_needed);
    }

    /// Render a YAML file, and return the picture cache debug info
    fn render_yaml(
        &mut self,
        filename: &str,
    ) -> RenderResult {
        let path = format!("invalidation/{}.yaml", filename);
        let mut reader = YamlFrameReader::new(&PathBuf::from(path));

        reader.do_frame(self.wrench);
        let composite_needed = match self.rx.recv().unwrap() {
            NotifierEvent::WakeUp { composite_needed } => composite_needed,
            NotifierEvent::ShutDown => unreachable!(),
        };
        let results = self.wrench.render();
        self.window.swap_buffers();

        RenderResult {
            pc_debug: results.picture_cache_debug,
            composite_needed,
        }
    }
}
