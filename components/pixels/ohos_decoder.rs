/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::vec::IntoIter;
use std::{fmt, mem, ptr};

use image::error::{ImageFormatHint, UnsupportedError};
use image::{
    AnimationDecoder, Frame, Frames, ImageBuffer, ImageDecoder, ImageError, ImageFormat,
    ImageResult, Pixel, RgbaImage,
};
use ohos_image_kit_sys::native_image::common::ImageResult as OhosImageResult;
use ohos_image_kit_sys::native_image::image_source::{
    OH_DecodingOptions, OH_DecodingOptions_Create, OH_DecodingOptions_Release,
    OH_DecodingOptions_SetPixelFormat, OH_ImageSource_Info, OH_ImageSourceInfo_Create,
    OH_ImageSourceInfo_GetHeight, OH_ImageSourceInfo_GetWidth, OH_ImageSourceInfo_Release,
    OH_ImageSourceNative, OH_ImageSourceNative_CreateFromDataWithUserBuffer,
    OH_ImageSourceNative_CreatePixelmap, OH_ImageSourceNative_CreatePixelmapList,
    OH_ImageSourceNative_GetFrameCount, OH_ImageSourceNative_GetImageInfo,
    OH_ImageSourceNative_Release,
};
use ohos_image_kit_sys::native_image::pixelmap::{
    OH_PixelmapNative, OH_PixelmapNative_GetByteCount, OH_PixelmapNative_ReadPixels,
    OH_PixelmapNative_Release, PIXEL_FORMAT,
};

use crate::decoding::ServoImageDecoder;

pub(crate) struct OhosImageDecoder<'a> {
    format: ImageFormat,
    /// The data needs to be alive according to documentation on `OH_ImageSourceNative_CreateFromDataWithUserBuffer`.
    _data: &'a [u8],
    image_source: *mut OH_ImageSourceNative,
    decoding_option: *mut OH_DecodingOptions,
    image_info: *mut OH_ImageSource_Info,
}

impl<'a> std::fmt::Debug for OhosImageDecoder<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OhosImageDecoder").finish()
    }
}

impl<'a> OhosImageDecoder<'a> {
    fn new(format: ImageFormat, data: &'a [u8]) -> Result<Self, ()> {
        unsafe {
            /// Use OH_ImageSourceNative_GetSupportedFormats to ask for which mimetypes are ok.
            let mut image_source_native = ptr::null_mut();
            let res = OH_ImageSourceNative_CreateFromDataWithUserBuffer(
                data.as_ptr().cast_mut(),
                data.len(),
                &raw mut image_source_native,
            );
            if res != OhosImageResult::SUCCESS || image_source_native.is_null() {
                log::error!("Something wrong with CreateFromData");
                return Err(());
            }
            let mut decoding_options = ptr::null_mut();
            let res = OH_DecodingOptions_Create(&raw mut decoding_options);
            OH_DecodingOptions_SetPixelFormat(
                decoding_options,
                PIXEL_FORMAT::PIXEL_FORMAT_RGBA_8888.0 as i32,
            );
            if res != OhosImageResult::SUCCESS || decoding_options.is_null() {
                log::error!("Something wrong with doing decoding options");
                todo!("Cleanup imagesourcneative");
                return Err(());
            }

            let mut image_info = ptr::null_mut();
            let res = OH_ImageSourceInfo_Create(&raw mut image_info);
            if res != OhosImageResult::SUCCESS || image_info.is_null() {
                log::error!("Could not get image info");
                todo!("Cleanup imagesourcenative and decodingoptions");
                return Err(());
            }

            if OH_ImageSourceNative_GetImageInfo(image_source_native, 0, image_info) !=
                OhosImageResult::SUCCESS
            {
                log::error!("Could not populate image info");
                todo!("cleanup imagesourcenative, decodingoption and imagesourceinfo");
                return Err(());
            }

            Ok(OhosImageDecoder {
                format,
                _data: data,
                image_source: image_source_native,
                decoding_option: decoding_options,
                image_info,
            })
        }
    }
}

impl<'a> Drop for OhosImageDecoder<'a> {
    fn drop(&mut self) {
        unsafe {
            if OH_DecodingOptions_Release(self.decoding_option) != OhosImageResult::SUCCESS {
                log::error!("Cleaning up of decoding options failed");
            }
            if OH_ImageSourceNative_Release(self.image_source) != OhosImageResult::SUCCESS {
                log::error!("Cleaning up of ImageSourceNative failed");
            }
            if OH_ImageSourceInfo_Release(self.image_info) != OhosImageResult::SUCCESS {
                log::error!("Cleaning up of ImageSourceInfo failed");
            }
        }
    }
}

impl<'a> ImageDecoder for OhosImageDecoder<'a> {
    fn dimensions(&self) -> (u32, u32) {
        unsafe {
            let mut width = 20;
            if OH_ImageSourceInfo_GetWidth(self.image_info, &raw mut width) !=
                OhosImageResult::SUCCESS
            {
                log::error!("Could not get width");
            }
            let mut height = 20;
            if OH_ImageSourceInfo_GetHeight(self.image_info, &raw mut height) !=
                OhosImageResult::SUCCESS
            {
                log::error!("Could not get height");
            }
            (width, height)
        }
    }

    fn color_type(&self) -> image::ColorType {
        // Fixed by setup
        image::ColorType::Rgba8
    }

    fn read_image(self, buf: &mut [u8]) -> ImageResult<()>
    where
        Self: Sized,
    {
        unsafe {
            let mut pixmap = ptr::null_mut();
            let res = OH_ImageSourceNative_CreatePixelmap(
                self.image_source,
                self.decoding_option,
                &raw mut pixmap,
            );
            if res != OhosImageResult::SUCCESS || pixmap.is_null() {
                log::error!("Could not create pixmap");
                return Err(ImageError::Unsupported(
                    ImageFormatHint::Exact(self.format).into(),
                ));
            }

            if write_pixmap_to_buffer(pixmap, buf).is_err() {
                log::error!("Could not decode pixmap");
                return Err(ImageError::Unsupported(
                    ImageFormatHint::Exact(self.format).into(),
                ));
            }

            if OH_PixelmapNative_Release(pixmap) != OhosImageResult::SUCCESS {
                log::error!("Could not release pixmap");
            }
        }
        Ok(())
    }

    fn read_image_boxed(self: Box<Self>, buf: &mut [u8]) -> ImageResult<()> {
        self.read_image(buf)
    }
}

/// We will only write as much data as the pixmap is given.
/// SAFETY:
/// The caller is responsible for having the buffer be large enough.
unsafe fn write_pixmap_to_buffer(
    pixmap: *mut OH_PixelmapNative,
    buffer: &mut [u8],
) -> Result<(), ()> {
    unsafe {
        let mut buffer_size = 0;
        if OH_PixelmapNative_GetByteCount(pixmap, &raw mut buffer_size) != OhosImageResult::SUCCESS
        {
            log::error!("Could not get byte count");
            return Err(());
        }

        if OH_PixelmapNative_ReadPixels(
            pixmap,
            buffer.as_mut_ptr(),
            &raw mut buffer_size as *mut usize,
        ) != OhosImageResult::SUCCESS
        {
            log::error!("Could not read pixels from pixmap");
            return Err(());
        }
    }
    Ok(())
}

impl<'a> ServoImageDecoder<'a> for OhosImageDecoder<'a> {
    fn make_decoder(format: ImageFormat, buffer: &'a [u8]) -> ImageResult<Self> {
        OhosImageDecoder::new(format, buffer)
            .map_err(|_| ImageError::Unsupported(ImageFormatHint::Exact(format).into()))
    }

    fn is_animated(&self) -> bool {
        let mut frame_count = 0;
        unsafe {
            if OH_ImageSourceNative_GetFrameCount(self.image_source, &mut frame_count) !=
                OhosImageResult::SUCCESS
            {
                log::error!("Frame call failed. Just going to abort");
                return false;
            }
        }
        frame_count > 1
    }

    fn decoder(self) -> impl ImageDecoder {
        self
    }

    fn animated_decoder(self) -> impl AnimationDecoder<'a> {
        self
    }
}

struct OhosAnimationIterator {
    inner_iterator: IntoIter<*mut OH_PixelmapNative>,
    width: u32,
    height: u32,
}

impl Iterator for OhosAnimationIterator {
    type Item = ImageResult<Frame>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(next_frame) = self.inner_iterator.next() else {
            return None;
        };

        let mut buffer = vec![0_u8; 4 * (self.width * self.height) as usize];
        unsafe {
            if write_pixmap_to_buffer(next_frame, &mut buffer).is_err() {
                log::error!("ERROR IN DOING BUFFER :(");
            }
        };

        let Some(rgba_image) = RgbaImage::from_raw(self.width, self.height, buffer) else {
            log::error!("failed, aborting");
            return None;
        };
        Some(Ok(Frame::new(rgba_image)))
    }
}

impl<'a> AnimationDecoder<'a> for OhosImageDecoder<'a> {
    fn into_frames(self) -> image::Frames<'a> {
        unsafe {
            let (width, height) = self.dimensions();
            let mut frame_count = 0;
            if OH_ImageSourceNative_GetFrameCount(self.image_source, &mut frame_count) !=
                OhosImageResult::SUCCESS
            {
                log::error!("Could not get frame count");
            }
            let mut result_pixmap_vector = vec![ptr::null_mut(); frame_count as usize];
            let result_pixmap_ptr = result_pixmap_vector.as_mut_ptr();
            if OH_ImageSourceNative_CreatePixelmapList(
                self.image_source,
                self.decoding_option,
                result_pixmap_ptr,
                frame_count as usize,
            ) != OhosImageResult::SUCCESS
            {
                log::error!("Something wrong with pixmap vector thingy");
            }

            let frame_iterator = Box::new(OhosAnimationIterator {
                inner_iterator: result_pixmap_vector.into_iter(),
                width,
                height,
            });

            Frames::new(frame_iterator)
        }
    }
}
