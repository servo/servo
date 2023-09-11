/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Screen capture infrastructure for the Gecko Profiler and Composition Recorder.

use std::collections::HashMap;

use api::{ImageFormat, ImageBufferKind};
use api::units::*;
use gleam::gl::GlType;

use crate::device::{Device, PBO, DrawTarget, ReadTarget, Texture, TextureFilter};
use crate::internal_types::RenderTargetInfo;
use crate::renderer::Renderer;
use crate::util::round_up_to_multiple;

/// A handle to a screenshot that is being asynchronously captured and scaled.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AsyncScreenshotHandle(usize);

/// A handle to a recorded frame that was captured.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordedFrameHandle(usize);

/// An asynchronously captured screenshot bound to a PBO which has not yet been mapped for copying.
struct AsyncScreenshot {
    /// The PBO that will contain the screenshot data.
    pbo: PBO,
    /// The size of the screenshot.
    screenshot_size: DeviceIntSize,
    /// The stride of the data in the PBO.
    buffer_stride: usize,
    /// Thge image format of the screenshot.
    image_format: ImageFormat,
}

/// How the `AsyncScreenshotGrabber` captures frames.
#[derive(Debug, Eq, PartialEq)]
enum AsyncScreenshotGrabberMode {
    /// Capture screenshots for the Gecko profiler.
    ///
    /// This mode will asynchronously scale the screenshots captured.
    ProfilerScreenshots,

    /// Capture screenshots for the CompositionRecorder.
    ///
    /// This mode does not scale the captured screenshots.
    CompositionRecorder,
}

/// Renderer infrastructure for capturing screenshots and scaling them asynchronously.
pub(in crate) struct AsyncScreenshotGrabber {
    /// The textures used to scale screenshots.
    scaling_textures: Vec<Texture>,
    /// PBOs available to be used for screenshot readback.
    available_pbos: Vec<PBO>,
    /// PBOs containing screenshots that are awaiting readback.
    awaiting_readback: HashMap<AsyncScreenshotHandle, AsyncScreenshot>,
    /// The handle for the net PBO that will be inserted into `in_use_pbos`.
    next_pbo_handle: usize,
    /// The mode the grabber operates in.
    mode: AsyncScreenshotGrabberMode,
}

impl Default for AsyncScreenshotGrabber {
    fn default() -> Self {
        AsyncScreenshotGrabber {
            scaling_textures: Vec::new(),
            available_pbos: Vec::new(),
            awaiting_readback: HashMap::new(),
            next_pbo_handle: 1,
            mode: AsyncScreenshotGrabberMode::ProfilerScreenshots,
        }
    }
}

impl AsyncScreenshotGrabber {
    /// Create a new AsyncScreenshotGrabber for the composition recorder.
    pub fn new_composition_recorder() -> Self {
        let mut recorder = Self::default();
        recorder.mode = AsyncScreenshotGrabberMode::CompositionRecorder;

        recorder
    }

    /// Deinitialize the allocated textures and PBOs.
    pub fn deinit(self, device: &mut Device) {
        for texture in self.scaling_textures {
            device.delete_texture(texture);
        }

        for pbo in self.available_pbos {
            device.delete_pbo(pbo);
        }

        for (_, async_screenshot) in self.awaiting_readback {
            device.delete_pbo(async_screenshot.pbo);
        }
    }

    /// Take a screenshot and scale it asynchronously.
    ///
    /// The returned handle can be used to access the mapped screenshot data via
    /// `map_and_recycle_screenshot`.
    /// The returned size is the size of the screenshot.
    pub fn get_screenshot(
        &mut self,
        device: &mut Device,
        window_rect: DeviceIntRect,
        buffer_size: DeviceIntSize,
        image_format: ImageFormat,
    ) -> (AsyncScreenshotHandle, DeviceIntSize) {
        let screenshot_size = match self.mode {
            AsyncScreenshotGrabberMode::ProfilerScreenshots => {
                assert_ne!(window_rect.size.width, 0);
                assert_ne!(window_rect.size.height, 0);

                let scale = (buffer_size.width as f32 / window_rect.size.width as f32)
                    .min(buffer_size.height as f32 / window_rect.size.height as f32);

                (window_rect.size.to_f32() * scale).round().to_i32()
            }

            AsyncScreenshotGrabberMode::CompositionRecorder => {
                assert_eq!(buffer_size, window_rect.size);
                buffer_size
            }
        };

        assert!(screenshot_size.width <= buffer_size.width);
        assert!(screenshot_size.height <= buffer_size.height);

        // To ensure that we hit the fast path when reading from a
        // framebuffer we must ensure that the width of the area we read
        // is a multiple of the device's optimal pixel-transfer stride.
        // The read_size should therefore be the screenshot_size with the width
        // increased to a suitable value. We will also pass this value to
        // scale_screenshot() as the min_texture_size, to ensure the texture is
        // large enough to read from. In CompositionRecorder mode we read
        // directly from the default framebuffer so are unable choose this size.
        let read_size = match self.mode {
            AsyncScreenshotGrabberMode::ProfilerScreenshots => {
                let stride = (screenshot_size.width * image_format.bytes_per_pixel()) as usize;
                let rounded = round_up_to_multiple(stride, device.required_pbo_stride().num_bytes(image_format));
                let optimal_width = rounded as i32 / image_format.bytes_per_pixel();

                DeviceIntSize::new(
                    optimal_width,
                    screenshot_size.height,
                )
            }
            AsyncScreenshotGrabberMode::CompositionRecorder => buffer_size,
        };
        let required_size = read_size.area() as usize * image_format.bytes_per_pixel() as usize;

        // Find an available PBO with the required size, creating a new one if necessary.
        let pbo = {
            let mut reusable_pbo = None;
            while let Some(pbo) = self.available_pbos.pop() {
                if pbo.get_reserved_size() != required_size {
                    device.delete_pbo(pbo);
                } else {
                    reusable_pbo = Some(pbo);
                    break;
                }
            };

            reusable_pbo.unwrap_or_else(|| device.create_pbo_with_size(required_size))
        };
        assert_eq!(pbo.get_reserved_size(), required_size);

        let read_target = match self.mode {
            AsyncScreenshotGrabberMode::ProfilerScreenshots => {
                self.scale_screenshot(
                    device,
                    ReadTarget::Default,
                    window_rect,
                    buffer_size,
                    read_size,
                    screenshot_size,
                    image_format,
                    0,
                );

                ReadTarget::from_texture(&self.scaling_textures[0])
            }

            AsyncScreenshotGrabberMode::CompositionRecorder => ReadTarget::Default,
        };

        device.read_pixels_into_pbo(
            read_target,
            DeviceIntRect::new(DeviceIntPoint::new(0, 0), read_size),
            image_format,
            &pbo,
        );

        let handle = AsyncScreenshotHandle(self.next_pbo_handle);
        self.next_pbo_handle += 1;

        self.awaiting_readback.insert(
            handle,
            AsyncScreenshot {
                pbo,
                screenshot_size,
                buffer_stride: (read_size.width * image_format.bytes_per_pixel()) as usize,
                image_format,
            },
        );

        (handle, screenshot_size)
    }

    /// Take the screenshot in the given `ReadTarget` and scale it to `dest_size` recursively.
    ///
    /// Each scaling operation scales only by a factor of two to preserve quality.
    ///
    /// Textures are scaled such that `scaling_textures[n]` is half the size of
    /// `scaling_textures[n+1]`.
    ///
    /// After the scaling completes, the final screenshot will be in
    /// `scaling_textures[0]`.
    ///
    /// The size of `scaling_textures[0]` will be increased to `min_texture_size`
    /// so that an optimally-sized area can be read from it.
    fn scale_screenshot(
        &mut self,
        device: &mut Device,
        read_target: ReadTarget,
        read_target_rect: DeviceIntRect,
        buffer_size: DeviceIntSize,
        min_texture_size: DeviceIntSize,
        dest_size: DeviceIntSize,
        image_format: ImageFormat,
        level: usize,
    ) {
        assert_eq!(self.mode, AsyncScreenshotGrabberMode::ProfilerScreenshots);

        let texture_size = {
            let size = buffer_size * (1 << level);
            DeviceIntSize::new(
                size.width.max(min_texture_size.width),
                size.height.max(min_texture_size.height),
            )
        };

        // If we haven't created a texture for this level, or the existing
        // texture is the wrong size, then create a new one.
        if level == self.scaling_textures.len() || self.scaling_textures[level].get_dimensions() != texture_size {
            let texture = device.create_texture(
                ImageBufferKind::Texture2D,
                image_format,
                texture_size.width,
                texture_size.height,
                TextureFilter::Linear,
                Some(RenderTargetInfo { has_depth: false }),
            );
            if level == self.scaling_textures.len() {
                self.scaling_textures.push(texture);
            } else {
                let old_texture = std::mem::replace(&mut self.scaling_textures[level], texture);
                device.delete_texture(old_texture);
            }
        }
        assert_eq!(self.scaling_textures[level].get_dimensions(), texture_size);

        let (read_target, read_target_rect) = if read_target_rect.size.width > 2 * dest_size.width {
            self.scale_screenshot(
                device,
                read_target,
                read_target_rect,
                buffer_size,
                min_texture_size,
                dest_size * 2,
                image_format,
                level + 1,
            );

            (
                ReadTarget::from_texture(&self.scaling_textures[level + 1]),
                DeviceIntRect::new(DeviceIntPoint::new(0, 0), dest_size * 2),
            )
        } else {
            (read_target, read_target_rect)
        };

        let draw_target = DrawTarget::from_texture(&self.scaling_textures[level], false);

        let draw_target_rect = draw_target
            .to_framebuffer_rect(DeviceIntRect::new(DeviceIntPoint::new(0, 0), dest_size));

        let read_target_rect = device_rect_as_framebuffer_rect(&read_target_rect);

        if level == 0 && !device.surface_origin_is_top_left() {
            device.blit_render_target_invert_y(
                read_target,
                read_target_rect,
                draw_target,
                draw_target_rect,
            );
        } else {
            device.blit_render_target(
                read_target,
                read_target_rect,
                draw_target,
                draw_target_rect,
                TextureFilter::Linear,
            );
        }
    }

    /// Map the contents of the screenshot given by the handle and copy it into
    /// the given buffer.
    pub fn map_and_recycle_screenshot(
        &mut self,
        device: &mut Device,
        handle: AsyncScreenshotHandle,
        dst_buffer: &mut [u8],
        dst_stride: usize,
    ) -> bool {
        let AsyncScreenshot {
            pbo,
            screenshot_size,
            buffer_stride,
            image_format,
        } = match self.awaiting_readback.remove(&handle) {
            Some(screenshot) => screenshot,
            None => return false,
        };

        let gl_type = device.gl().get_type();

        let success = if let Some(bound_pbo) = device.map_pbo_for_readback(&pbo) {
            let src_buffer = &bound_pbo.data;
            let src_stride = buffer_stride;
            let src_width =
                screenshot_size.width as usize * image_format.bytes_per_pixel() as usize;

            for (src_slice, dst_slice) in self
                .iter_src_buffer_chunked(gl_type, src_buffer, src_stride)
                .zip(dst_buffer.chunks_mut(dst_stride))
                .take(screenshot_size.height as usize)
            {
                dst_slice[.. src_width].copy_from_slice(&src_slice[.. src_width]);
            }

            true
        } else {
            false
        };

        match self.mode {
            AsyncScreenshotGrabberMode::ProfilerScreenshots => self.available_pbos.push(pbo),
            AsyncScreenshotGrabberMode::CompositionRecorder => device.delete_pbo(pbo),
        }

        success
    }

    fn iter_src_buffer_chunked<'a>(
        &self,
        gl_type: GlType,
        src_buffer: &'a [u8],
        src_stride: usize,
    ) -> Box<dyn Iterator<Item = &'a [u8]> + 'a> {
        use AsyncScreenshotGrabberMode::*;

        let is_angle = cfg!(windows) && gl_type == GlType::Gles;

        if self.mode == CompositionRecorder && !is_angle {
            // This is a non-ANGLE configuration. in this case, the recorded frames were captured
            // upside down, so we have to flip them right side up.
            Box::new(src_buffer.chunks(src_stride).rev())
        } else {
            // This is either an ANGLE configuration in the `CompositionRecorder` mode or a
            // non-ANGLE configuration in the `ProfilerScreenshots` mode. In either case, the
            // captured frames are right-side up.
            Box::new(src_buffer.chunks(src_stride))
        }
    }
}

// Screen-capture specific Renderer impls.
impl Renderer {
    /// Record a frame for the Composition Recorder.
    ///
    /// The returned handle can be passed to `map_recorded_frame` to copy it into
    /// a buffer.
    /// The returned size is the size of the frame.
    pub fn record_frame(
        &mut self,
        image_format: ImageFormat,
    ) -> Option<(RecordedFrameHandle, DeviceIntSize)> {
        let device_size = self.device_size()?;
        self.device.begin_frame();

        let (handle, _) = self
            .async_frame_recorder
            .get_or_insert_with(AsyncScreenshotGrabber::new_composition_recorder)
            .get_screenshot(
                &mut self.device,
                DeviceIntRect::new(DeviceIntPoint::new(0, 0), device_size),
                device_size,
                image_format,
            );

        self.device.end_frame();

        Some((RecordedFrameHandle(handle.0), device_size))
    }

    /// Map a frame captured for the composition recorder into the given buffer.
    pub fn map_recorded_frame(
        &mut self,
        handle: RecordedFrameHandle,
        dst_buffer: &mut [u8],
        dst_stride: usize,
    ) -> bool {
        if let Some(async_frame_recorder) = self.async_frame_recorder.as_mut() {
            async_frame_recorder.map_and_recycle_screenshot(
                &mut self.device,
                AsyncScreenshotHandle(handle.0),
                dst_buffer,
                dst_stride,
            )
        } else {
            false
        }
    }

    /// Free the data structures used by the composition recorder.
    pub fn release_composition_recorder_structures(&mut self) {
        if let Some(async_frame_recorder) = self.async_frame_recorder.take() {
            self.device.begin_frame();
            async_frame_recorder.deinit(&mut self.device);
            self.device.end_frame();
        }
    }

    /// Take a screenshot and scale it asynchronously.
    ///
    /// The returned handle can be used to access the mapped screenshot data via
    /// `map_and_recycle_screenshot`.
    ///
    /// The returned size is the size of the screenshot.
    pub fn get_screenshot_async(
        &mut self,
        window_rect: DeviceIntRect,
        buffer_size: DeviceIntSize,
        image_format: ImageFormat,
    ) -> (AsyncScreenshotHandle, DeviceIntSize) {
        self.device.begin_frame();

        let handle = self
            .async_screenshots
            .get_or_insert_with(AsyncScreenshotGrabber::default)
            .get_screenshot(&mut self.device, window_rect, buffer_size, image_format);

        self.device.end_frame();

        handle
    }

    /// Map the contents of the screenshot given by the handle and copy it into
    /// the given buffer.
    pub fn map_and_recycle_screenshot(
        &mut self,
        handle: AsyncScreenshotHandle,
        dst_buffer: &mut [u8],
        dst_stride: usize,
    ) -> bool {
        if let Some(async_screenshots) = self.async_screenshots.as_mut() {
            async_screenshots.map_and_recycle_screenshot(
                &mut self.device,
                handle,
                dst_buffer,
                dst_stride,
            )
        } else {
            false
        }
    }

    /// Release the screenshot grabbing structures that the profiler was using.
    pub fn release_profiler_structures(&mut self) {
        if let Some(async_screenshots) = self.async_screenshots.take() {
            self.device.begin_frame();
            async_screenshots.deinit(&mut self.device);
            self.device.end_frame();
        }
    }
}
