/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::{point2, size2, rect};
use std::sync::Arc;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::mpsc::Receiver;
use webrender::api::*;
use webrender::api::units::*;
use crate::{WindowWrapper, NotifierEvent};
use crate::blob;
use crate::reftest::{ReftestImage, ReftestImageComparison};
use crate::wrench::Wrench;

pub struct RawtestHarness<'a> {
    wrench: &'a mut Wrench,
    rx: &'a Receiver<NotifierEvent>,
    window: &'a mut WindowWrapper,
}


impl<'a> RawtestHarness<'a> {
    pub fn new(wrench: &'a mut Wrench,
               window: &'a mut WindowWrapper,
               rx: &'a Receiver<NotifierEvent>) -> Self {
        RawtestHarness {
            wrench,
            rx,
            window,
        }
    }

    pub fn run(mut self) {
        self.test_hit_testing();
        self.test_resize_image();
        self.test_retained_blob_images_test();
        self.test_blob_update_test();
        self.test_blob_update_epoch_test();
        self.test_tile_decomposition();
        self.test_very_large_blob();
        self.test_blob_visible_area();
        self.test_blob_set_visible_area();
        self.test_offscreen_blob();
        self.test_save_restore();
        self.test_blur_cache();
        self.test_capture();
        self.test_zero_height_window();
        self.test_clear_cache();
    }

    fn render_and_get_pixels(&mut self, window_rect: FramebufferIntRect) -> Vec<u8> {
        self.rx.recv().unwrap();
        self.wrench.render();
        self.wrench.renderer.read_pixels_rgba8(window_rect)
    }

    fn compare_pixels(&self, data1: Vec<u8>, data2: Vec<u8>, size: FramebufferIntSize) {
        let size = DeviceIntSize::new(size.width, size.height);
        let image1 = ReftestImage {
            data: data1,
            size,
        };
        let image2 = ReftestImage {
            data: data2,
            size,
        };

        match image1.compare(&image2) {
            ReftestImageComparison::Equal => {}
            ReftestImageComparison::NotEqual { max_difference, count_different, .. } => {
                let t = "rawtest";
                println!(
                    "{} | {} | {}: {}, {}: {}",
                    "REFTEST TEST-UNEXPECTED-FAIL",
                    t,
                    "image comparison, max difference",
                    max_difference,
                    "number of differing pixels",
                    count_different
                );
                println!("REFTEST   IMAGE 1: {}", image1.create_data_uri());
                println!("REFTEST   IMAGE 2: {}", image2.create_data_uri());
                println!("REFTEST TEST-END | {}", t);
                panic!();
            }
        }
    }

    fn submit_dl(
        &mut self,
        epoch: &mut Epoch,
        layout_size: LayoutSize,
        builder: DisplayListBuilder,
        mut txn: Transaction,
    ) {
        let root_background_color = Some(ColorF::new(1.0, 1.0, 1.0, 1.0));
        txn.use_scene_builder_thread();

        txn.set_display_list(
            *epoch,
            root_background_color,
            layout_size,
            builder.finalize(),
            false,
        );
        epoch.0 += 1;

        txn.generate_frame();
        self.wrench.api.send_transaction(self.wrench.document_id, txn);
    }

    fn make_common_properties(&self, clip_rect: LayoutRect) -> CommonItemProperties {
        let space_and_clip = SpaceAndClipInfo::root_scroll(self.wrench.root_pipeline_id);
        CommonItemProperties {
            clip_rect,
            clip_id: space_and_clip.clip_id,
            spatial_id: space_and_clip.spatial_id,
            flags: PrimitiveFlags::default(),
            hit_info: None,
        }
    }

    fn make_common_properties_with_clip_and_spatial(
        &self,
        clip_rect: LayoutRect,
        clip_id: ClipId,
        spatial_id: SpatialId
    ) -> CommonItemProperties {
        CommonItemProperties {
            clip_rect,
            clip_id,
            spatial_id,
            flags: PrimitiveFlags::default(),
            hit_info: None,
        }
    }

    fn test_resize_image(&mut self) {
        println!("\tresize image...");
        // This test changes the size of an image to make it go switch back and forth
        // between tiled and non-tiled.
        // The resource cache should be able to handle this without crashing.

        let layout_size = LayoutSize::new(800., 800.);

        let mut txn = Transaction::new();
        let img = self.wrench.api.generate_image_key();

        // Start with a non-tiled image.
        txn.add_image(
            img,
            ImageDescriptor::new(64, 64, ImageFormat::BGRA8, ImageDescriptorFlags::IS_OPAQUE),
            ImageData::new(vec![255; 64 * 64 * 4]),
            None,
        );

        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);
        let info = self.make_common_properties(rect(0.0, 0.0, 64.0, 64.0));

        builder.push_image(
            &info,
            info.clip_rect,
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            img,
            ColorF::WHITE,
        );

        let mut epoch = Epoch(0);

        self.submit_dl(&mut epoch, layout_size, builder, txn);
        self.rx.recv().unwrap();
        self.wrench.render();

        let mut txn = Transaction::new();
        // Resize the image to something bigger than the max texture size (8196) to force tiling.
        txn.update_image(
            img,
            ImageDescriptor::new(8200, 32, ImageFormat::BGRA8, ImageDescriptorFlags::IS_OPAQUE),
            ImageData::new(vec![255; 8200 * 32 * 4]),
            &DirtyRect::All,
        );

        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);
        let info = self.make_common_properties(rect(0.0, 0.0, 1024.0, 1024.0));

        builder.push_image(
            &info,
            info.clip_rect,
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            img,
            ColorF::WHITE,
        );

        self.submit_dl(&mut epoch, layout_size, builder, txn);
        self.rx.recv().unwrap();
        self.wrench.render();

        let mut txn = Transaction::new();
        // Resize back to something doesn't require tiling.
        txn.update_image(
            img,
            ImageDescriptor::new(64, 64, ImageFormat::BGRA8, ImageDescriptorFlags::IS_OPAQUE),
            ImageData::new(vec![64; 64 * 64 * 4]),
            &DirtyRect::All,
        );

        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);
        let info = self.make_common_properties(rect(0.0, 0.0, 1024.0, 1024.0));

        builder.push_image(
            &info,
            info.clip_rect,
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            img,
            ColorF::WHITE,
        );

        self.submit_dl(&mut epoch, layout_size, builder, txn);
        self.rx.recv().unwrap();
        self.wrench.render();

        txn = Transaction::new();
        txn.delete_image(img);
        self.wrench.api.send_transaction(self.wrench.document_id, txn);
    }

    fn test_tile_decomposition(&mut self) {
        println!("\ttile decomposition...");
        // This exposes a crash in tile decomposition
        let layout_size = LayoutSize::new(800., 800.);
        let mut txn = Transaction::new();

        let blob_img = self.wrench.api.generate_blob_image_key();
        txn.add_blob_image(
            blob_img,
            ImageDescriptor::new(151, 56, ImageFormat::BGRA8, ImageDescriptorFlags::IS_OPAQUE),
            blob::serialize_blob(ColorU::new(50, 50, 150, 255)),
            rect(0, 0, 151, 56),
            Some(128),
        );

        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);

        let info = self.make_common_properties(rect(448.899994, 74.0, 151.000031, 56.));

        // setup some malicious image size parameters
        builder.push_repeating_image(
            &info,
            info.clip_rect,
            size2(151., 56.0),
            size2(151.0, 56.0),
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            blob_img.as_image(),
            ColorF::WHITE,
        );

        let mut epoch = Epoch(0);

        self.submit_dl(&mut epoch, layout_size, builder, txn);

        self.rx.recv().unwrap();
        self.wrench.render();

        // Leaving a tiled blob image in the resource cache
        // confuses the `test_capture`. TODO: remove this
        txn = Transaction::new();
        txn.delete_blob_image(blob_img);
        self.wrench.api.send_transaction(self.wrench.document_id, txn);
    }

    fn test_very_large_blob(&mut self) {
        println!("\tvery large blob...");

        assert_eq!(self.wrench.device_pixel_ratio, 1.);

        let window_size = self.window.get_inner_size();

        let test_size = FramebufferIntSize::new(800, 800);

        let window_rect = FramebufferIntRect::new(
            FramebufferIntPoint::new(0, window_size.height - test_size.height),
            test_size,
        );

        // This exposes a crash in tile decomposition
        let layout_size = LayoutSize::new(800., 800.);
        let mut txn = Transaction::new();

        let blob_img = self.wrench.api.generate_blob_image_key();
        txn.add_blob_image(
            blob_img,
            ImageDescriptor::new(15000, 15000, ImageFormat::BGRA8, ImageDescriptorFlags::empty()),
            blob::serialize_blob(ColorU::new(50, 50, 150, 255)),
            rect(0, 0, 15000, 15000),
            Some(100),
        );

        let called = Arc::new(AtomicIsize::new(0));
        let called_inner = Arc::clone(&called);

        self.wrench.callbacks.lock().unwrap().request = Box::new(move |_| {
            called_inner.fetch_add(1, Ordering::SeqCst);
        });

        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);

        let root_space_and_clip = SpaceAndClipInfo::root_scroll(self.wrench.root_pipeline_id);
        let clip_id = builder.define_clip_rect(
            &root_space_and_clip,
            rect(40., 41., 200., 201.),
        );

        let info = CommonItemProperties {
            clip_rect: rect(0.0, 0.0, 800.0, 800.0),
            clip_id,
            spatial_id: root_space_and_clip.spatial_id,
            flags: PrimitiveFlags::default(),
            hit_info: None,
        };

        // setup some malicious image size parameters
        builder.push_repeating_image(
            &info,
            size2(15000.0, 15000.0).into(),
            size2(15000.0, 15000.0),
            size2(0.0, 0.0),
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            blob_img.as_image(),
            ColorF::WHITE,
        );

        let mut epoch = Epoch(0);

        self.submit_dl(&mut epoch, layout_size, builder, txn);

        let pixels = self.render_and_get_pixels(window_rect);

        // make sure we didn't request too many blobs
        assert!(called.load(Ordering::SeqCst) < 20);

        //use crate::png;
        //png::save_flipped("out.png", pixels.clone(), size2(window_rect.size.width, window_rect.size.height));

        // make sure things are in the right spot
        let w = window_rect.size.width as usize;
        let h = window_rect.size.height as usize;
        let p1 = (40 + (h - 100) * w) * 4;
        assert_eq!(pixels[p1 + 0], 50);
        assert_eq!(pixels[p1 + 1], 50);
        assert_eq!(pixels[p1 + 2], 150);
        assert_eq!(pixels[p1 + 3], 255);

        // Leaving a tiled blob image in the resource cache
        // confuses the `test_capture`. TODO: remove this
        txn = Transaction::new();
        txn.delete_blob_image(blob_img);
        self.wrench.api.send_transaction(self.wrench.document_id, txn);

        *self.wrench.callbacks.lock().unwrap() = blob::BlobCallbacks::new();
    }

    fn test_blob_visible_area(&mut self) {
        println!("\tblob visible area...");

        assert_eq!(self.wrench.device_pixel_ratio, 1.0);

        let window_size = self.window.get_inner_size();
        let test_size = FramebufferIntSize::new(800, 800);
        let window_rect = FramebufferIntRect::new(
            FramebufferIntPoint::new(0, window_size.height - test_size.height),
            test_size,
        );
        let layout_size = LayoutSize::new(800.0, 800.0);
        let mut txn = Transaction::new();

        let blob_img = self.wrench.api.generate_blob_image_key();
        txn.add_blob_image(
            blob_img,
            ImageDescriptor::new(500, 500, ImageFormat::BGRA8, ImageDescriptorFlags::empty()),
            blob::serialize_blob(ColorU::new(50, 50, 150, 255)),
            DeviceIntRect {
                origin: point2(50, 20),
                size: size2(400, 400),
            },
            Some(100),
        );

        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);

        let image_size = size2(400.0, 400.0);

        let root_space_and_clip = SpaceAndClipInfo::root_scroll(self.wrench.root_pipeline_id);
        let clip_id = builder.define_clip_rect(
            &root_space_and_clip,
            rect(-1000.0, -1000.0, 2000.0, 2000.0),
        );

        let info = CommonItemProperties {
            clip_rect: rect(10.0, 10.0, 400.0, 400.0),
            clip_id,
            spatial_id: root_space_and_clip.spatial_id,
            flags: PrimitiveFlags::default(),
            hit_info: None,
        };

        builder.push_repeating_image(
            &info,
            info.clip_rect,
            image_size,
            image_size,
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            blob_img.as_image(),
            ColorF::WHITE,
        );
        let mut epoch = Epoch(0);

        self.submit_dl(&mut epoch, layout_size, builder, txn);

        let pixels = self.render_and_get_pixels(window_rect);

        //use super::png;
        //png::save_flipped("out.png", pixels.clone(), size2(window_rect.size.width, window_rect.size.height));


        // make sure things are in the right spot
        let w = window_rect.size.width as usize;
        let h = window_rect.size.height as usize;
        let p1 = (65 + (h - 15) * w) * 4;
        assert_eq!(pixels[p1 + 0], 255);
        assert_eq!(pixels[p1 + 1], 255);
        assert_eq!(pixels[p1 + 2], 255);
        assert_eq!(pixels[p1 + 3], 255);

        let p2 = (25 + (h - 15) * w) * 4;
        assert_eq!(pixels[p2 + 0], 221);
        assert_eq!(pixels[p2 + 1], 221);
        assert_eq!(pixels[p2 + 2], 221);
        assert_eq!(pixels[p2 + 3], 255);

        let p3 = (15 + (h - 15) * w) * 4;
        assert_eq!(pixels[p3 + 0], 50);
        assert_eq!(pixels[p3 + 1], 50);
        assert_eq!(pixels[p3 + 2], 150);
        assert_eq!(pixels[p3 + 3], 255);

        // Leaving a tiled blob image in the resource cache
        // confuses the `test_capture`. TODO: remove this
        txn = Transaction::new();
        txn.delete_blob_image(blob_img);
        self.wrench.api.send_transaction(self.wrench.document_id, txn);

        *self.wrench.callbacks.lock().unwrap() = blob::BlobCallbacks::new();
    }

    fn test_blob_set_visible_area(&mut self) {
        // In this test we first render a blob with a certain visible area,
        // then change the visible area without updating the blob image.

        println!("\tblob visible area update...");

        assert_eq!(self.wrench.device_pixel_ratio, 1.0);

        let window_size = self.window.get_inner_size();
        let test_size = FramebufferIntSize::new(800, 800);
        let window_rect = FramebufferIntRect::new(
            FramebufferIntPoint::new(0, window_size.height - test_size.height),
            test_size,
        );
        let layout_size = LayoutSize::new(800.0, 800.0);
        let mut txn = Transaction::new();

        let blob_img = self.wrench.api.generate_blob_image_key();
        txn.add_blob_image(
            blob_img,
            ImageDescriptor::new(500, 500, ImageFormat::BGRA8, ImageDescriptorFlags::empty()),
            blob::serialize_blob(ColorU::new(50, 50, 150, 255)),
            DeviceIntRect {
                origin: point2(0, 0),
                size: size2(500, 500),
            },
            Some(128),
        );

        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);

        let root_space_and_clip = SpaceAndClipInfo::root_scroll(self.wrench.root_pipeline_id);
        let clip_id = builder.define_clip_rect(
            &root_space_and_clip,
            rect(-1000.0, -1000.0, 2000.0, 2000.0),
        );

        let info = CommonItemProperties {
            clip_rect: rect(0.0, 0.0, 1000.0, 1000.0),
            clip_id,
            spatial_id: root_space_and_clip.spatial_id,
            flags: PrimitiveFlags::default(),
            hit_info: None,
        };

        builder.push_repeating_image(
            &info,
            rect(0.0, 0.0, 500.0, 500.0),
            size2(500.0, 500.0),
            size2(500.0, 500.0),
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            blob_img.as_image(),
            ColorF::WHITE,
        );
        let mut epoch = Epoch(0);

        // Render the first display list. We don't care about the result but we
        // want to make sure the next display list updates an already rendered
        // state.
        self.submit_dl(&mut epoch, layout_size, builder, txn);
        let _ = self.render_and_get_pixels(window_rect);

        // Now render a similar scene with an updated blob visible area.
        // In this test we care about the fact that the visible area was updated
        // without using update_blob_image.

        let mut txn = Transaction::new();

        txn.set_blob_image_visible_area(blob_img, DeviceIntRect {
            origin: point2(50, 50),
            size: size2(400, 400),
        });

        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);

        let root_space_and_clip = SpaceAndClipInfo::root_scroll(self.wrench.root_pipeline_id);
        let clip_id = builder.define_clip_rect(
            &root_space_and_clip,
            rect(-1000.0, -1000.0, 2000.0, 2000.0),
        );

        let info = CommonItemProperties {
            clip_rect: rect(0.0, 0.0, 1000.0, 1000.0),
            clip_id,
            spatial_id: root_space_and_clip.spatial_id,
            flags: PrimitiveFlags::default(),
            hit_info: None,
        };

        builder.push_repeating_image(
            &info,
            rect(50.0, 50.0, 400.0, 400.0),
            size2(400.0, 400.0),
            size2(400.0, 400.0),
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            blob_img.as_image(),
            ColorF::WHITE,
        );

        self.submit_dl(&mut epoch, layout_size, builder, txn);
        let resized_pixels = self.render_and_get_pixels(window_rect);

        // Now render the same scene with a new blob image created with the same
        // visible area as the previous scene, without going through an update.

        let mut txn = Transaction::new();

        let blob_img2 = self.wrench.api.generate_blob_image_key();
        txn.add_blob_image(
            blob_img2,
            ImageDescriptor::new(500, 500, ImageFormat::BGRA8, ImageDescriptorFlags::empty()),
            blob::serialize_blob(ColorU::new(50, 50, 150, 255)),
            DeviceIntRect {
                origin: point2(50, 50),
                size: size2(400, 400),
            },
            Some(128),
        );

        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);

        let root_space_and_clip = SpaceAndClipInfo::root_scroll(self.wrench.root_pipeline_id);
        let clip_id = builder.define_clip_rect(
            &root_space_and_clip,
            rect(-1000.0, -1000.0, 2000.0, 2000.0),
        );

        let info = CommonItemProperties {
            clip_rect: rect(0.0, 0.0, 1000.0, 1000.0),
            clip_id,
            spatial_id: root_space_and_clip.spatial_id,
            flags: PrimitiveFlags::default(),
            hit_info: None,
        };

        builder.push_repeating_image(
            &info,
            rect(50.0, 50.0, 400.0, 400.0),
            size2(400.0, 400.0),
            size2(400.0, 400.0),
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            blob_img2.as_image(),
            ColorF::WHITE,
        );
        let mut epoch = Epoch(0);

        self.submit_dl(&mut epoch, layout_size, builder, txn);

        let reference_pixels = self.render_and_get_pixels(window_rect);

        assert_eq!(resized_pixels, reference_pixels);

        txn = Transaction::new();
        txn.delete_blob_image(blob_img);
        txn.delete_blob_image(blob_img2);
        self.wrench.api.send_transaction(self.wrench.document_id, txn);
    }

    fn test_offscreen_blob(&mut self) {
        println!("\toffscreen blob update...");

        assert_eq!(self.wrench.device_pixel_ratio, 1.);

        let window_size = self.window.get_inner_size();

        let test_size = FramebufferIntSize::new(800, 800);
        let window_rect = FramebufferIntRect::new(
            point2(0, window_size.height - test_size.height),
            test_size,
        );

        // This exposes a crash in tile decomposition
        let mut txn = Transaction::new();
        let layout_size = LayoutSize::new(800., 800.);

        let blob_img = self.wrench.api.generate_blob_image_key();
        txn.add_blob_image(
            blob_img,
            ImageDescriptor::new(1510, 1510, ImageFormat::BGRA8, ImageDescriptorFlags::empty()),
            blob::serialize_blob(ColorU::new(50, 50, 150, 255)),
            rect(0, 0, 1510, 1510),
            None,
        );

        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);

        let info = self.make_common_properties(rect(0., 0.0, 1510., 1510.));

        let image_size = size2(1510., 1510.);

        // setup some malicious image size parameters
        builder.push_repeating_image(
            &info,
            info.clip_rect,
            image_size,
            image_size,
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            blob_img.as_image(),
            ColorF::WHITE,
        );

        let mut epoch = Epoch(0);

        self.submit_dl(&mut epoch, layout_size, builder, txn);

        let original_pixels = self.render_and_get_pixels(window_rect);

        let mut epoch = Epoch(1);

        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);

        let info = self.make_common_properties(rect(-10000., 0.0, 1510., 1510.));

        let image_size = size2(1510., 1510.);

        // setup some malicious image size parameters
        builder.push_repeating_image(
            &info,
            info.clip_rect,
            image_size,
            image_size,
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            blob_img.as_image(),
            ColorF::WHITE,
        );

        self.submit_dl(&mut epoch, layout_size, builder, Transaction::new());

        let _offscreen_pixels = self.render_and_get_pixels(window_rect);

        let mut txn = Transaction::new();

        txn.update_blob_image(
            blob_img,
            ImageDescriptor::new(1510, 1510, ImageFormat::BGRA8, ImageDescriptorFlags::empty()),
            blob::serialize_blob(ColorU::new(50, 50, 150, 255)),
            rect(0, 0, 1510, 1510),
            &rect(10, 10, 100, 100).into(),
        );

        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);

        let info = self.make_common_properties(rect(0., 0.0, 1510., 1510.));

        let image_size = size2(1510., 1510.);

        // setup some malicious image size parameters
        builder.push_repeating_image(
            &info,
            info.clip_rect,
            image_size,
            image_size,
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            blob_img.as_image(),
            ColorF::WHITE,
        );

        let mut epoch = Epoch(2);

        self.submit_dl(&mut epoch, layout_size, builder, txn);

        let pixels = self.render_and_get_pixels(window_rect);

        self.compare_pixels(original_pixels, pixels, window_rect.size);

        // Leaving a tiled blob image in the resource cache
        // confuses the `test_capture`. TODO: remove this
        txn = Transaction::new();
        txn.delete_blob_image(blob_img);
        self.wrench.api.send_transaction(self.wrench.document_id, txn);
    }

    fn test_retained_blob_images_test(&mut self) {
        println!("\tretained blob images test...");
        let blob_img;
        let window_size = self.window.get_inner_size();

        let test_size = FramebufferIntSize::new(400, 400);
        let window_rect = FramebufferIntRect::new(
            FramebufferIntPoint::new(0, window_size.height - test_size.height),
            test_size,
        );
        let layout_size = LayoutSize::new(400., 400.);

        let mut txn = Transaction::new();
        {
            let api = &self.wrench.api;

            blob_img = api.generate_blob_image_key();
            txn.add_blob_image(
                blob_img,
                ImageDescriptor::new(500, 500, ImageFormat::BGRA8, ImageDescriptorFlags::empty()),
                blob::serialize_blob(ColorU::new(50, 50, 150, 255)),
                rect(0, 0, 500, 500),
                None,
            );
        }

        let called = Arc::new(AtomicIsize::new(0));
        let called_inner = Arc::clone(&called);

        self.wrench.callbacks.lock().unwrap().request = Box::new(move |_| {
            assert_eq!(0, called_inner.fetch_add(1, Ordering::SeqCst));
        });

        // draw the blob the first time
        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);
        let info = self.make_common_properties(rect(0.0, 60.0, 200.0, 200.0));

        builder.push_image(
            &info,
            info.clip_rect,
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            blob_img.as_image(),
            ColorF::WHITE,
        );

        let mut epoch = Epoch(0);

        self.submit_dl(&mut epoch, layout_size, builder, txn);

        let pixels_first = self.render_and_get_pixels(window_rect);

        assert_eq!(1, called.load(Ordering::SeqCst));

        // draw the blob image a second time at a different location

        // make a new display list that refers to the first image
        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);
        let info = self.make_common_properties(rect(1.0, 60.0, 200.0, 200.0));
        builder.push_image(
            &info,
            info.clip_rect,
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            blob_img.as_image(),
            ColorF::WHITE,
        );

        let mut txn = Transaction::new();
        txn.resource_updates.clear();

        self.submit_dl(&mut epoch, layout_size, builder, txn);

        let pixels_second = self.render_and_get_pixels(window_rect);

        // make sure we only requested once
        assert_eq!(1, called.load(Ordering::SeqCst));

        // use png;
        // png::save_flipped("out1.png", &pixels_first, window_rect.size);
        // png::save_flipped("out2.png", &pixels_second, window_rect.size);
        assert!(pixels_first != pixels_second);

        // cleanup
        *self.wrench.callbacks.lock().unwrap() = blob::BlobCallbacks::new();
    }

    fn test_blob_update_epoch_test(&mut self) {
        println!("\tblob update epoch test...");
        let (blob_img, blob_img2);
        let window_size = self.window.get_inner_size();

        let test_size = FramebufferIntSize::new(400, 400);
        let window_rect = FramebufferIntRect::new(
            point2(0, window_size.height - test_size.height),
            test_size,
        );
        let layout_size = LayoutSize::new(400., 400.);

        let mut txn = Transaction::new();
        let (blob_img, blob_img2) = {
            let api = &self.wrench.api;

            blob_img = api.generate_blob_image_key();
            txn.add_blob_image(
                blob_img,
                ImageDescriptor::new(500, 500, ImageFormat::BGRA8, ImageDescriptorFlags::empty()),
                blob::serialize_blob(ColorU::new(50, 50, 150, 255)),
                rect(0, 0, 500, 500),
                None,
            );
            blob_img2 = api.generate_blob_image_key();
            txn.add_blob_image(
                blob_img2,
                ImageDescriptor::new(500, 500, ImageFormat::BGRA8, ImageDescriptorFlags::empty()),
                blob::serialize_blob(ColorU::new(80, 50, 150, 255)),
                rect(0, 0, 500, 500),
                None,
            );
            (blob_img, blob_img2)
        };

        // setup some counters to count how many times each image is requested
        let img1_requested = Arc::new(AtomicIsize::new(0));
        let img1_requested_inner = Arc::clone(&img1_requested);
        let img2_requested = Arc::new(AtomicIsize::new(0));
        let img2_requested_inner = Arc::clone(&img2_requested);

        // track the number of times that the second image has been requested
        self.wrench.callbacks.lock().unwrap().request = Box::new(move |requests| {
            for item in requests {
                if item.request.key == blob_img {
                    img1_requested_inner.fetch_add(1, Ordering::SeqCst);
                }
                if item.request.key == blob_img2 {
                    img2_requested_inner.fetch_add(1, Ordering::SeqCst);
                }
            }
        });

        // create two blob images and draw them
        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);
        let info = self.make_common_properties(rect(0.0, 60.0, 200.0, 200.0));
        let info2 = self.make_common_properties(rect(200.0, 60.0, 200.0, 200.0));
        let push_images = |builder: &mut DisplayListBuilder| {
            builder.push_image(
                &info,
                info.clip_rect,
                ImageRendering::Auto,
                AlphaType::PremultipliedAlpha,
                blob_img.as_image(),
                ColorF::WHITE,
            );
            builder.push_image(
                &info2,
                info2.clip_rect,
                ImageRendering::Auto,
                AlphaType::PremultipliedAlpha,
                blob_img2.as_image(),
                ColorF::WHITE,
            );
        };

        push_images(&mut builder);

        let mut epoch = Epoch(0);

        self.submit_dl(&mut epoch, layout_size, builder, txn);
        let _pixels_first = self.render_and_get_pixels(window_rect);

        // update and redraw both images
        let mut txn = Transaction::new();
        txn.update_blob_image(
            blob_img,
            ImageDescriptor::new(500, 500, ImageFormat::BGRA8, ImageDescriptorFlags::empty()),
            blob::serialize_blob(ColorU::new(50, 50, 150, 255)),
            rect(0, 0, 500, 500),
            &rect(100, 100, 100, 100).into(),
        );
        txn.update_blob_image(
            blob_img2,
            ImageDescriptor::new(500, 500, ImageFormat::BGRA8, ImageDescriptorFlags::empty()),
            blob::serialize_blob(ColorU::new(59, 50, 150, 255)),
            rect(0, 0, 500, 500),
            &rect(100, 100, 100, 100).into(),
        );

        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);
        push_images(&mut builder);
        self.submit_dl(&mut epoch, layout_size, builder, txn);
        let _pixels_second = self.render_and_get_pixels(window_rect);

        // only update the first image
        let mut txn = Transaction::new();
        txn.update_blob_image(
            blob_img,
            ImageDescriptor::new(500, 500, ImageFormat::BGRA8, ImageDescriptorFlags::empty()),
            blob::serialize_blob(ColorU::new(50, 150, 150, 255)),
            rect(0, 0, 500, 500),
            &rect(200, 200, 100, 100).into(),
        );

        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);
        push_images(&mut builder);
        self.submit_dl(&mut epoch, layout_size, builder, txn);
        let _pixels_third = self.render_and_get_pixels(window_rect);

        // the first image should be requested 3 times
        assert_eq!(img1_requested.load(Ordering::SeqCst), 3);
        // the second image should've been requested twice
        assert_eq!(img2_requested.load(Ordering::SeqCst), 2);

        // cleanup
        *self.wrench.callbacks.lock().unwrap() = blob::BlobCallbacks::new();
    }

    fn test_blob_update_test(&mut self) {
        println!("\tblob update test...");
        let window_size = self.window.get_inner_size();

        let test_size = FramebufferIntSize::new(400, 400);
        let window_rect = FramebufferIntRect::new(
            point2(0, window_size.height - test_size.height),
            test_size,
        );
        let layout_size = LayoutSize::new(400., 400.);
        let mut txn = Transaction::new();

        let blob_img = {
            let img = self.wrench.api.generate_blob_image_key();
            txn.add_blob_image(
                img,
                ImageDescriptor::new(500, 500, ImageFormat::BGRA8, ImageDescriptorFlags::empty()),
                blob::serialize_blob(ColorU::new(50, 50, 150, 255)),
                rect(0, 0, 500, 500),
                None,
            );
            img
        };

        // draw the blobs the first time
        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);
        let info = self.make_common_properties(rect(0.0, 60.0, 200.0, 200.0));

        builder.push_image(
            &info,
            info.clip_rect,
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            blob_img.as_image(),
            ColorF::WHITE,
        );

        let mut epoch = Epoch(0);

        self.submit_dl(&mut epoch, layout_size, builder, txn);
        let pixels_first = self.render_and_get_pixels(window_rect);

        // draw the blob image a second time after updating it with the same color
        let mut txn = Transaction::new();
        txn.update_blob_image(
            blob_img,
            ImageDescriptor::new(500, 500, ImageFormat::BGRA8, ImageDescriptorFlags::empty()),
            blob::serialize_blob(ColorU::new(50, 50, 150, 255)),
            rect(0, 0, 500, 500),
            &rect(100, 100, 100, 100).into(),
        );

        // make a new display list that refers to the first image
        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);
        let info = self.make_common_properties(rect(0.0, 60.0, 200.0, 200.0));
        builder.push_image(
            &info,
            info.clip_rect,
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            blob_img.as_image(),
            ColorF::WHITE,
        );

        self.submit_dl(&mut epoch, layout_size, builder, txn);
        let pixels_second = self.render_and_get_pixels(window_rect);

        // draw the blob image a third time after updating it with a different color
        let mut txn = Transaction::new();
        txn.update_blob_image(
            blob_img,
            ImageDescriptor::new(500, 500, ImageFormat::BGRA8, ImageDescriptorFlags::empty()),
            blob::serialize_blob(ColorU::new(50, 150, 150, 255)),
            rect(0, 0, 500, 500),
            &rect(200, 200, 100, 100).into(),
        );

        // make a new display list that refers to the first image
        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);
        let info = self.make_common_properties(rect(0.0, 60.0, 200.0, 200.0));
        builder.push_image(
            &info,
            info.clip_rect,
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            blob_img.as_image(),
            ColorF::WHITE,
        );

        self.submit_dl(&mut epoch, layout_size, builder, txn);
        let pixels_third = self.render_and_get_pixels(window_rect);

        assert!(pixels_first != pixels_third);
        self.compare_pixels(pixels_first, pixels_second, window_rect.size);
    }

    // Ensures that content doing a save-restore produces the same results as not
    fn test_save_restore(&mut self) {
        println!("\tsave/restore...");
        let window_size = self.window.get_inner_size();

        let test_size = FramebufferIntSize::new(400, 400);
        let window_rect = FramebufferIntRect::new(
            point2(0, window_size.height - test_size.height),
            test_size,
        );
        let layout_size = LayoutSize::new(400., 400.);

        let mut do_test = |should_try_and_fail| {
            let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);

            let spatial_id = SpatialId::root_scroll_node(self.wrench.root_pipeline_id);
            let clip_id = builder.define_clip_rect(
                &SpaceAndClipInfo::root_scroll(self.wrench.root_pipeline_id),
                rect(110., 120., 200., 200.),
            );
            builder.push_rect(
                &self.make_common_properties_with_clip_and_spatial(
                    rect(100., 100., 100., 100.),
                    clip_id,
                    spatial_id),
                rect(100., 100., 100., 100.),
                ColorF::new(0.0, 0.0, 1.0, 1.0),
            );

            if should_try_and_fail {
                builder.save();
                let clip_id = builder.define_clip_rect(
                    &SpaceAndClipInfo { spatial_id, clip_id },
                    rect(80., 80., 90., 90.),
                );
                let space_and_clip = SpaceAndClipInfo {
                    spatial_id,
                    clip_id
                };
                builder.push_rect(
                    &self.make_common_properties_with_clip_and_spatial(
                        rect(110., 110., 50., 50.),
                        clip_id,
                        spatial_id),
                    rect(110., 110., 50., 50.),
                    ColorF::new(0.0, 1.0, 0.0, 1.0),
                );
                builder.push_shadow(
                    &space_and_clip,
                    Shadow {
                        offset: LayoutVector2D::new(1.0, 1.0),
                        blur_radius: 1.0,
                        color: ColorF::new(0.0, 0.0, 0.0, 1.0),
                    },
                    true,
                );
                let info = CommonItemProperties {
                    clip_rect: rect(110., 110., 50., 2.),
                    clip_id,
                    spatial_id,
                    flags: PrimitiveFlags::default(),
                    hit_info: None,
                };
                builder.push_line(
                    &info,
                    &info.clip_rect,
                    0.0, LineOrientation::Horizontal,
                    &ColorF::new(0.0, 0.0, 0.0, 1.0),
                    LineStyle::Solid,
                );
                builder.restore();
            }

            {
                builder.save();
                let clip_id = builder.define_clip_rect(
                    &SpaceAndClipInfo { spatial_id, clip_id },
                    rect(80., 80., 100., 100.),
                );
                builder.push_rect(
                    &self.make_common_properties_with_clip_and_spatial(
                        rect(150., 150., 100., 100.),
                        clip_id,
                        spatial_id),
                    rect(150., 150., 100., 100.),
                    ColorF::new(0.0, 0.0, 1.0, 1.0),
                );
                builder.clear_save();
            }

            let txn = Transaction::new();

            self.submit_dl(&mut Epoch(0), layout_size, builder, txn);

            self.render_and_get_pixels(window_rect)
        };

        let first = do_test(false);
        let second = do_test(true);

        self.compare_pixels(first, second, window_rect.size);
    }

    // regression test for #2769
    // "async scene building: cache collisions from reused picture ids"
    fn test_blur_cache(&mut self) {
        println!("\tblur cache...");
        let window_size = self.window.get_inner_size();

        let test_size = FramebufferIntSize::new(400, 400);
        let window_rect = FramebufferIntRect::new(
            point2(0, window_size.height - test_size.height),
            test_size,
        );
        let layout_size = LayoutSize::new(400., 400.);

        let mut do_test = |shadow_is_red| {
            let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);
            let shadow_color = if shadow_is_red {
                ColorF::new(1.0, 0.0, 0.0, 1.0)
            } else {
                ColorF::new(0.0, 1.0, 0.0, 1.0)
            };

            builder.push_shadow(
                &SpaceAndClipInfo::root_scroll(self.wrench.root_pipeline_id),
                Shadow {
                    offset: LayoutVector2D::new(1.0, 1.0),
                    blur_radius: 1.0,
                    color: shadow_color,
                },
                true,
            );
            let info = self.make_common_properties(rect(110., 110., 50., 2.));
            builder.push_line(
                &info,
                &info.clip_rect,
                0.0, LineOrientation::Horizontal,
                &ColorF::new(0.0, 0.0, 0.0, 1.0),
                LineStyle::Solid,
            );
            builder.pop_all_shadows();

            let txn = Transaction::new();
            self.submit_dl(&mut Epoch(0), layout_size, builder, txn);

            self.render_and_get_pixels(window_rect)
        };

        let first = do_test(false);
        let second = do_test(true);

        assert_ne!(first, second);
    }

    fn test_capture(&mut self) {
        println!("\tcapture...");
        let path = "../captures/test";
        let layout_size = LayoutSize::new(400., 400.);
        let dim = self.window.get_inner_size();
        let window_rect = FramebufferIntRect::new(
            point2(0, dim.height - layout_size.height as i32),
            size2(layout_size.width as i32, layout_size.height as i32),
        );

        // 1. render some scene

        let mut txn = Transaction::new();
        let image = self.wrench.api.generate_image_key();
        txn.add_image(
            image,
            ImageDescriptor::new(1, 1, ImageFormat::BGRA8, ImageDescriptorFlags::IS_OPAQUE),
            ImageData::new(vec![0xFF, 0, 0, 0xFF]),
            None,
        );

        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);

        let info = self.make_common_properties(rect(300.0, 70.0, 150.0, 50.0));
        builder.push_image(
            &info,
            info.clip_rect,
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            image,
            ColorF::WHITE,
        );

        let mut txn = Transaction::new();

        txn.set_display_list(
            Epoch(0),
            Some(ColorF::new(1.0, 1.0, 1.0, 1.0)),
            layout_size,
            builder.finalize(),
            false,
        );
        txn.generate_frame();

        self.wrench.api.send_transaction(self.wrench.document_id, txn);

        let pixels0 = self.render_and_get_pixels(window_rect);

        // 2. capture it
        self.wrench.api.save_capture(path.into(), CaptureBits::all());

        // 3. set a different scene

        builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);

        let mut txn = Transaction::new();
        txn.set_display_list(
            Epoch(1),
            Some(ColorF::new(1.0, 0.0, 0.0, 1.0)),
            layout_size,
            builder.finalize(),
            false,
        );
        self.wrench.api.send_transaction(self.wrench.document_id, txn);

        // 4. load the first one

        let mut documents = self.wrench.api.load_capture(path.into(), None);
        let captured = documents.swap_remove(0);

        // 5. render the built frame and compare
        let pixels1 = self.render_and_get_pixels(window_rect);
        self.compare_pixels(pixels0.clone(), pixels1, window_rect.size);

        // 6. rebuild the scene and compare again
        let mut txn = Transaction::new();
        txn.set_root_pipeline(captured.root_pipeline_id.unwrap());
        txn.generate_frame();
        self.wrench.api.send_transaction(captured.document_id, txn);
        let pixels2 = self.render_and_get_pixels(window_rect);
        self.compare_pixels(pixels0, pixels2, window_rect.size);
    }

    fn test_zero_height_window(&mut self) {
        println!("\tzero height test...");

        let layout_size = LayoutSize::new(120.0, 0.0);
        let window_size = DeviceIntSize::new(layout_size.width as i32, layout_size.height as i32);
        let doc_id = self.wrench.api.add_document(window_size, 1);

        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);
        let info = self.make_common_properties(LayoutRect::new(LayoutPoint::zero(),
                                                            LayoutSize::new(100.0, 100.0)));
        builder.push_rect(
            &info,
            info.clip_rect,
            ColorF::new(0.0, 1.0, 0.0, 1.0),
        );

        let mut txn = Transaction::new();
        txn.set_root_pipeline(self.wrench.root_pipeline_id);
        txn.set_display_list(
            Epoch(1),
            Some(ColorF::new(1.0, 0.0, 0.0, 1.0)),
            layout_size,
            builder.finalize(),
            false,
        );
        txn.generate_frame();
        self.wrench.api.send_transaction(doc_id, txn);

        // Ensure we get a notification from rendering the above, even though
        // there are zero visible pixels
        assert!(self.rx.recv().unwrap() == NotifierEvent::WakeUp);
    }


    fn test_hit_testing(&mut self) {
        println!("\thit testing test...");

        let layout_size = LayoutSize::new(400., 400.);
        let mut builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);

        // Add a rectangle that covers the entire scene.
        let mut info = self.make_common_properties(LayoutRect::new(LayoutPoint::zero(), layout_size));
        info.hit_info = Some((0, 1));
        builder.push_rect(&info, info.clip_rect, ColorF::new(1.0, 1.0, 1.0, 1.0));

        // Add a simple 100x100 rectangle at 100,0.
        let mut info = self.make_common_properties(LayoutRect::new(
            LayoutPoint::new(100., 0.),
            LayoutSize::new(100., 100.)
        ));
        info.hit_info = Some((0, 2));
        builder.push_rect(&info, info.clip_rect, ColorF::new(1.0, 1.0, 1.0, 1.0));

        let space_and_clip = SpaceAndClipInfo::root_scroll(self.wrench.root_pipeline_id);

        let make_rounded_complex_clip = |rect: &LayoutRect, radius: f32| -> ComplexClipRegion {
            ComplexClipRegion::new(
                *rect,
                BorderRadius::uniform_size(LayoutSize::new(radius, radius)),
                ClipMode::Clip
            )
        };

        // Add a rectangle that is clipped by a rounded rect clip item.
        let rect = LayoutRect::new(LayoutPoint::new(100., 100.), LayoutSize::new(100., 100.));
        let temp_clip_id = builder.define_clip_rounded_rect(
            &space_and_clip,
            make_rounded_complex_clip(&rect, 20.),
        );
        builder.push_rect(
            &CommonItemProperties {
                hit_info: Some((0, 4)),
                clip_rect: rect,
                clip_id: temp_clip_id,
                spatial_id: space_and_clip.spatial_id,
                flags: PrimitiveFlags::default(),
            },
            rect,
            ColorF::new(1.0, 1.0, 1.0, 1.0),
        );

        // Add a rectangle that is clipped by a ClipChain containing a rounded rect.
        let rect = LayoutRect::new(LayoutPoint::new(200., 100.), LayoutSize::new(100., 100.));
        let clip_id = builder.define_clip_rounded_rect(
            &space_and_clip,
            make_rounded_complex_clip(&rect, 20.),
        );
        let clip_chain_id = builder.define_clip_chain(None, vec![clip_id]);
        builder.push_rect(
            &CommonItemProperties {
                hit_info: Some((0, 5)),
                clip_rect: rect,
                clip_id: ClipId::ClipChain(clip_chain_id),
                spatial_id: space_and_clip.spatial_id,
                flags: PrimitiveFlags::default(),
            },
            rect,
            ColorF::new(1.0, 1.0, 1.0, 1.0),
        );

        let mut epoch = Epoch(0);
        let txn = Transaction::new();
        self.submit_dl(&mut epoch, layout_size, builder, txn);

        // We render to ensure that the hit tester is up to date with the current scene.
        self.rx.recv().unwrap();
        self.wrench.render();

        let hit_test = |point: WorldPoint| -> HitTestResult {
            self.wrench.api.hit_test(
                self.wrench.document_id,
                None,
                point,
                HitTestFlags::FIND_ALL,
            )
        };

        let assert_hit_test = |point: WorldPoint, tags: Vec<ItemTag>| {
            let result = hit_test(point);
            assert_eq!(result.items.len(), tags.len());

            for (hit_test_item, item_b) in result.items.iter().zip(tags.iter()) {
                assert_eq!(hit_test_item.tag, *item_b);
            }
        };

        // We should not have any hits outside the boundaries of the scene.
        assert_hit_test(WorldPoint::new(-10., -10.), Vec::new());
        assert_hit_test(WorldPoint::new(-10., 10.), Vec::new());
        assert_hit_test(WorldPoint::new(450., 450.), Vec::new());
        assert_hit_test(WorldPoint::new(100., 450.), Vec::new());

        // The top left corner of the scene should only contain the background.
        assert_hit_test(WorldPoint::new(50., 50.), vec![(0, 1)]);

        // The middle of the normal rectangle should be hit.
        assert_hit_test(WorldPoint::new(150., 50.), vec![(0, 2), (0, 1)]);

        let test_rounded_rectangle = |point: WorldPoint, size: WorldSize, tag: ItemTag| {
            // The cut out corners of the rounded rectangle should not be hit.
            let top_left = point + WorldVector2D::new(5., 5.);
            let bottom_right = point + size.to_vector() - WorldVector2D::new(5., 5.);

            assert_hit_test(
                WorldPoint::new(point.x + (size.width / 2.), point.y + (size.height / 2.)),
                vec![tag, (0, 1)]
            );

            assert_hit_test(top_left, vec![(0, 1)]);
            assert_hit_test(WorldPoint::new(bottom_right.x, top_left.y), vec![(0, 1)]);
            assert_hit_test(WorldPoint::new(top_left.x, bottom_right.y), vec![(0, 1)]);
            assert_hit_test(bottom_right, vec![(0, 1)]);
        };

        test_rounded_rectangle(WorldPoint::new(100., 100.), WorldSize::new(100., 100.), (0, 4));
        test_rounded_rectangle(WorldPoint::new(200., 100.), WorldSize::new(100., 100.), (0, 5));
    }

    fn test_clear_cache(&mut self) {
        println!("\tclear cache test...");

        self.wrench.api.send_message(ApiMsg::DebugCommand(DebugCommand::ClearCaches(ClearCache::all())));

        let layout_size = LayoutSize::new(400., 400.);
        let builder = DisplayListBuilder::new(self.wrench.root_pipeline_id, layout_size);

        let txn = Transaction::new();
        let mut epoch = Epoch(0);
        self.submit_dl(&mut epoch, layout_size, builder, txn);

        self.rx.recv().unwrap();
        self.wrench.render();
    }
}
