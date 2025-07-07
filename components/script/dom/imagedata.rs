/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::vec::Vec;

use dom_struct::dom_struct;
use euclid::default::{Rect, Size2D};
use ipc_channel::ipc::IpcSharedMemory;
use js::gc::CustomAutoRooterGuard;
use js::jsapi::JSObject;
use js::rust::HandleObject;
use js::typedarray::{ClampedU8, Uint8ClampedArray};
use pixels::{Snapshot, SnapshotAlphaMode, SnapshotPixelFormat};

use super::bindings::buffer_source::{HeapBufferSource, create_heap_buffer_source_with_length};
use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::{
    ImageDataMethods, ImageDataPixelFormat, ImageDataSettings, PredefinedColorSpace,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub(crate) struct ImageData {
    reflector_: Reflector,
    width: u32,
    height: u32,
    /// <https://html.spec.whatwg.org/multipage/#dom-imagedata-data>
    #[ignore_malloc_size_of = "mozjs"]
    data: HeapBufferSource<ClampedU8>,
    pixel_format: ImageDataPixelFormat,
    color_space: PredefinedColorSpace,
}

impl ImageData {
    #[allow(unsafe_code)]
    pub(crate) fn new(
        global: &GlobalScope,
        width: u32,
        height: u32,
        mut data: Option<Vec<u8>>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ImageData>> {
        let len =
            pixels::compute_rgba8_byte_length_if_within_limit(width as usize, height as usize)
                .ok_or(Error::Range(
                    "The requested image size exceeds the supported range".to_owned(),
                ))?;

        let settings = ImageDataSettings {
            colorSpace: Some(PredefinedColorSpace::Srgb),
            pixelFormat: ImageDataPixelFormat::Rgba_unorm8,
        };

        if let Some(ref mut d) = data {
            d.resize(len as usize, 0);

            let cx = GlobalScope::get_cx();
            rooted!(in (*cx) let mut js_object = std::ptr::null_mut::<JSObject>());
            auto_root!(in(*cx) let data = create_buffer_source::<ClampedU8>(cx, &d[..], js_object.handle_mut(), can_gc)
                        .map_err(|_| Error::JSFailed)?);

            Self::Constructor_(global, None, can_gc, data, width, Some(height), &settings)
        } else {
            Self::Constructor(global, None, can_gc, width, height, &settings)
        }
    }

    #[allow(clippy::too_many_arguments)]
    /// <https://html.spec.whatwg.org/multipage/#initialize-an-imagedata-object>
    fn initialize(
        pixels_per_row: u32,
        rows: u32,
        settings: &ImageDataSettings,
        source: Option<CustomAutoRooterGuard<Uint8ClampedArray>>,
        default_color_space: Option<PredefinedColorSpace>,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ImageData>> {
        // 1. If source was given:
        let data = if let Some(source) = source {
            // 1. If settings["pixelFormat"] equals "rgba-unorm8" and source is not a Uint8ClampedArray,
            // then throw an "InvalidStateError" DOMException.
            // 2. If settings["pixelFormat"] is "rgba-float16" and source is not a Float16Array,
            // then throw an "InvalidStateError" DOMException.
            if !matches!(settings.pixelFormat, ImageDataPixelFormat::Rgba_unorm8) {
                // we currently support only rgba-unorm8
                return Err(Error::InvalidState);
            }
            // 3. Initialize the data attribute of imageData to source.
            HeapBufferSource::<ClampedU8>::from_view(source)
        } else {
            // 2. Otherwise (source was not given):
            match settings.pixelFormat {
                ImageDataPixelFormat::Rgba_unorm8 => {
                    // 1. If settings["pixelFormat"] is "rgba-unorm8",
                    // then initialize the data attribute of imageData to a new Uint8ClampedArray object.
                    // The Uint8ClampedArray object must use a new ArrayBuffer for its storage,
                    // and must have a zero byte offset and byte length equal to the length of its storage, in bytes.
                    // The storage ArrayBuffer must have a length of 4 × rows × pixelsPerRow bytes.
                    // 3. If the storage ArrayBuffer could not be allocated,
                    // then rethrow the RangeError thrown by JavaScript, and return.
                    create_heap_buffer_source_with_length(
                        GlobalScope::get_cx(),
                        4 * rows * pixels_per_row,
                        can_gc,
                    )?
                },
                // 3. Otherwise, if settings["pixelFormat"] is "rgba-float16",
                // then initialize the data attribute of imageData to a new Float16Array object.
                // The Float16Array object must use a new ArrayBuffer for its storage,
                // and must have a zero byte offset and byte length equal to the length of its storage, in bytes.
                // The storage ArrayBuffer must have a length of 8 × rows × pixelsPerRow bytes.
                // not implemented yet
            }
        };
        // 3. Initialize the width attribute of imageData to pixelsPerRow.
        let width = pixels_per_row;
        // 4. Initialize the height attribute of imageData to rows.
        let height = rows;
        // 5. Initialize the pixelFormat attribute of imageData to settings["pixelFormat"].
        let pixel_format = settings.pixelFormat;
        // 6. If settings["colorSpace"] exists,
        // then initialize the colorSpace attribute of imageData to settings["colorSpace"].
        let color_space = settings
            .colorSpace
            // 7. Otherwise, if defaultColorSpace was given,
            // then initialize the colorSpace attribute of imageData to defaultColorSpace.
            .or(default_color_space)
            // 8. Otherwise, initialize the colorSpace attribute of imageData to "srgb".
            .unwrap_or(PredefinedColorSpace::Srgb);

        Ok(reflect_dom_object_with_proto(
            Box::new(ImageData {
                reflector_: Reflector::new(),
                width,
                height,
                data,
                pixel_format,
                color_space,
            }),
            global,
            proto,
            can_gc,
        ))
    }

    pub(crate) fn is_detached(&self) -> bool {
        self.data.is_detached_buffer(GlobalScope::get_cx())
    }

    pub(crate) fn get_size(&self) -> Size2D<u32> {
        Size2D::new(self.Width(), self.Height())
    }

    /// Nothing must change the array on the JS side while the slice is live.
    #[allow(unsafe_code)]
    pub(crate) unsafe fn as_slice(&self) -> &[u8] {
        assert!(self.data.is_initialized());
        let internal_data = self
            .data
            .get_typed_array()
            .expect("Failed to get Data from ImageData.");
        // NOTE(nox): This is just as unsafe as `as_slice` itself even though we
        // are extending the lifetime of the slice, because the data in
        // this ImageData instance will never change. The method is thus unsafe
        // because the array may be manipulated from JS while the reference
        // is live.
        unsafe {
            let ptr: *const [u8] = internal_data.as_slice() as *const _;
            &*ptr
        }
    }

    /// Nothing must change the array on the JS side while the slice is live.
    #[allow(unsafe_code)]
    pub(crate) unsafe fn get_rect(&self, rect: Rect<u32>) -> Cow<[u8]> {
        pixels::rgba8_get_rect(unsafe { self.as_slice() }, self.get_size().to_u32(), rect)
    }

    #[allow(unsafe_code)]
    pub(crate) fn get_snapshot_rect(&self, rect: Rect<u32>) -> Snapshot {
        Snapshot::from_vec(
            rect.size,
            SnapshotPixelFormat::RGBA,
            SnapshotAlphaMode::Transparent {
                premultiplied: false,
            },
            unsafe { self.get_rect(rect).into_owned() },
        )
    }

    #[allow(unsafe_code)]
    pub(crate) fn to_shared_memory(&self) -> IpcSharedMemory {
        // This is safe because we copy the slice content
        IpcSharedMemory::from_bytes(unsafe { self.as_slice() })
    }

    #[allow(unsafe_code)]
    pub(crate) fn to_vec(&self) -> Vec<u8> {
        // This is safe because we copy the slice content
        unsafe { self.as_slice() }.to_vec()
    }
}

impl ImageDataMethods<crate::DomTypeHolder> for ImageData {
    /// <https://html.spec.whatwg.org/multipage/#dom-imagedata>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        sw: u32,
        sh: u32,
        settings: &ImageDataSettings,
    ) -> Fallible<DomRoot<Self>> {
        // 1. If one or both of sw and sh are zero, then throw an "IndexSizeError" DOMException.
        if sw == 0 || sh == 0 {
            return Err(Error::IndexSize);
        }

        // When a constructor is called for an ImageData that is too large, other browsers throw
        // IndexSizeError rather than RangeError here, so we do the same.
        pixels::compute_rgba8_byte_length_if_within_limit(sw as usize, sh as usize)
            .ok_or(Error::IndexSize)?;

        // 2. Initialize this given sw, sh, and settings.
        // 3. Initialize the image data of this to transparent black.
        Self::initialize(sw, sh, settings, None, None, global, proto, can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-imagedata-with-data>
    fn Constructor_(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        data: CustomAutoRooterGuard<Uint8ClampedArray>,
        sw: u32,
        sh: Option<u32>,
        settings: &ImageDataSettings,
    ) -> Fallible<DomRoot<Self>> {
        // 1. Let bytesPerPixel be 4 if settings["pixelFormat"] is "rgba-unorm8"; otherwise 8.
        let bytes_per_pixel = match settings.pixelFormat {
            ImageDataPixelFormat::Rgba_unorm8 => 4,
        };
        // 2. Let length be the buffer source byte length of data.
        let length = data.len();
        if length == 0 {
            return Err(Error::InvalidState);
        }
        // 3. If length is not a nonzero integral multiple of bytesPerPixel,
        // then throw an "InvalidStateError" DOMException.
        if length % bytes_per_pixel != 0 {
            return Err(Error::InvalidState);
        }
        // 4. Let length be length divided by bytesPerPixel.
        let length = length / bytes_per_pixel;
        // 5. If length is not an integral multiple of sw, then throw an "IndexSizeError" DOMException.
        if sw == 0 || length % sw as usize != 0 {
            return Err(Error::IndexSize);
        }
        // 6. Let height be length divided by sw.
        let height = length / sw as usize;
        // 7. If sh was given and its value is not equal to height, then throw an "IndexSizeError" DOMException.
        if sh.is_some_and(|x| height != x as usize) {
            return Err(Error::IndexSize);
        }
        // 8. Initialize this given sw, sh, settings, and source set to data.
        Self::initialize(
            sw,
            height as u32,
            settings,
            Some(data),
            None,
            global,
            proto,
            can_gc,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-imagedata-width>
    fn Width(&self) -> u32 {
        self.width
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-imagedata-height>
    fn Height(&self) -> u32 {
        self.height
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-imagedata-data>
    fn GetData(&self, _: JSContext) -> Fallible<Uint8ClampedArray> {
        self.data.get_typed_array().map_err(|_| Error::JSFailed)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-imagedata-pixelformat>
    fn PixelFormat(&self) -> ImageDataPixelFormat {
        self.pixel_format
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-imagedata-colorspace>
    fn ColorSpace(&self) -> PredefinedColorSpace {
        self.color_space
    }
}
