/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::ptr;
use std::vec::Vec;

use dom_struct::dom_struct;
use euclid::default::{Rect, Size2D};
use ipc_channel::ipc::IpcSharedMemory;
use js::jsapi::{Heap, JSObject};
use js::rust::HandleObject;
use js::typedarray::{ClampedU8, CreateWith, Uint8ClampedArray};

use super::bindings::buffer_source::{
    new_initialized_heap_buffer_source, BufferSource, HeapBufferSource, HeapTypedArrayInit,
};
use crate::dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::ImageDataMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext;

#[dom_struct]
pub struct ImageData {
    reflector_: Reflector,
    width: u32,
    height: u32,
    #[ignore_malloc_size_of = "mozjs"]
    data: HeapBufferSource<ClampedU8>,
}

impl ImageData {
    #[allow(unsafe_code)]
    pub fn new(
        global: &GlobalScope,
        width: u32,
        height: u32,
        mut data: Option<Vec<u8>>,
    ) -> Fallible<DomRoot<ImageData>> {
        let len = width * height * 4;
        unsafe {
            let cx = GlobalScope::get_cx();
            rooted!(in (*cx) let mut js_object = ptr::null_mut::<JSObject>());
            if let Some(ref mut d) = data {
                d.resize(len as usize, 0);
                let data = CreateWith::Slice(&d[..]);
                Uint8ClampedArray::create(*cx, data, js_object.handle_mut()).unwrap();
                Self::new_with_jsobject(global, None, width, Some(height), js_object.get())
            } else {
                Self::new_without_jsobject(global, None, width, height)
            }
        }
    }

    #[allow(unsafe_code)]
    fn new_with_jsobject(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        width: u32,
        opt_height: Option<u32>,
        jsobject: *mut JSObject,
    ) -> Fallible<DomRoot<ImageData>> {
        let heap_typed_array = match new_initialized_heap_buffer_source::<ClampedU8>(
            HeapTypedArrayInit::Buffer(BufferSource::Uint8ClampedArray(Heap::boxed(jsobject))),
        ) {
            Ok(heap_typed_array) => heap_typed_array,
            Err(_) => return Err(Error::JSFailed),
        };

        let typed_array = match heap_typed_array.get_buffer() {
            Ok(array) => array,
            Err(_) => {
                return Err(Error::Type(
                    "Argument to Image data is not an Uint8ClampedArray".to_owned(),
                ))
            },
        };

        let byte_len = unsafe { typed_array.as_slice().len() } as u32;
        if byte_len == 0 || byte_len % 4 != 0 {
            return Err(Error::InvalidState);
        }

        let len = byte_len / 4;
        if width == 0 || len % width != 0 {
            return Err(Error::IndexSize);
        }

        let height = len / width;
        if opt_height.map_or(false, |x| height != x) {
            return Err(Error::IndexSize);
        }

        let imagedata = Box::new(ImageData {
            reflector_: Reflector::new(),
            width,
            height,
            data: heap_typed_array,
        });

        Ok(reflect_dom_object_with_proto(imagedata, global, proto))
    }

    fn new_without_jsobject(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        width: u32,
        height: u32,
    ) -> Fallible<DomRoot<ImageData>> {
        if width == 0 || height == 0 {
            return Err(Error::IndexSize);
        }
        let cx = GlobalScope::get_cx();
        let len = width * height * 4;

        let heap_typed_array =
            match new_initialized_heap_buffer_source::<ClampedU8>(HeapTypedArrayInit::Info {
                len,
                cx,
            }) {
                Ok(heap_typed_array) => heap_typed_array,
                Err(_) => return Err(Error::JSFailed),
            };
        let imagedata = Box::new(ImageData {
            reflector_: Reflector::new(),
            width,
            height,
            data: heap_typed_array,
        });

        Ok(reflect_dom_object_with_proto(imagedata, global, proto))
    }
    /// <https://html.spec.whatwg.org/multipage/#pixel-manipulation:dom-imagedata-3>
    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        width: u32,
        height: u32,
    ) -> Fallible<DomRoot<Self>> {
        Self::new_without_jsobject(global, proto, width, height)
    }

    /// <https://html.spec.whatwg.org/multipage/#pixel-manipulation:dom-imagedata-4>
    #[allow(unused_variables, non_snake_case)]
    pub fn Constructor_(
        cx: JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        jsobject: *mut JSObject,
        width: u32,
        opt_height: Option<u32>,
    ) -> Fallible<DomRoot<Self>> {
        Self::new_with_jsobject(global, proto, width, opt_height, jsobject)
    }

    /// Nothing must change the array on the JS side while the slice is live.
    #[allow(unsafe_code)]
    pub unsafe fn as_slice(&self) -> &[u8] {
        assert!(self.data.is_initialized());
        let internal_data = self
            .data
            .get_buffer()
            .expect("Failed to get Data from ImageData.");
        // NOTE(nox): This is just as unsafe as `as_slice` itself even though we
        // are extending the lifetime of the slice, because the data in
        // this ImageData instance will never change. The method is thus unsafe
        // because the array may be manipulated from JS while the reference
        // is live.
        let ptr: *const [u8] = internal_data.as_slice() as *const _;
        &*ptr
    }

    #[allow(unsafe_code)]
    pub fn to_shared_memory(&self) -> IpcSharedMemory {
        IpcSharedMemory::from_bytes(unsafe { self.as_slice() })
    }

    #[allow(unsafe_code)]
    pub unsafe fn get_rect(&self, rect: Rect<u64>) -> Cow<[u8]> {
        pixels::rgba8_get_rect(self.as_slice(), self.get_size().to_u64(), rect)
    }

    pub fn get_size(&self) -> Size2D<u32> {
        Size2D::new(self.Width(), self.Height())
    }
}

impl ImageDataMethods for ImageData {
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
        self.data.get_buffer().map_err(|_| Error::JSFailed)
    }
}
