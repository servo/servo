/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::ImageDataBinding;
use crate::dom::bindings::codegen::Bindings::ImageDataBinding::ImageDataMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use euclid::{Rect, Size2D};
use ipc_channel::ipc::IpcSharedMemory;
use js::jsapi::{Heap, JSContext, JSObject};
use js::rust::Runtime;
use js::typedarray::{CreateWith, Uint8ClampedArray};
use std::borrow::Cow;
use std::default::Default;
use std::ptr;
use std::ptr::NonNull;
use std::vec::Vec;

#[dom_struct]
pub struct ImageData {
    reflector_: Reflector,
    width: u32,
    height: u32,
    #[ignore_malloc_size_of = "mozjs"]
    data: Heap<*mut JSObject>,
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
            let cx = global.get_cx();
            rooted!(in (cx) let mut js_object = ptr::null_mut::<JSObject>());
            let data = match data {
                Some(ref mut d) => {
                    d.resize(len as usize, 0);
                    CreateWith::Slice(&d[..])
                },
                None => CreateWith::Length(len),
            };
            Uint8ClampedArray::create(cx, data, js_object.handle_mut()).unwrap();
            Self::new_with_jsobject(global, width, Some(height), Some(js_object.get()))
        }
    }

    #[allow(unsafe_code)]
    unsafe fn new_with_jsobject(
        global: &GlobalScope,
        width: u32,
        mut opt_height: Option<u32>,
        opt_jsobject: Option<*mut JSObject>,
    ) -> Fallible<DomRoot<ImageData>> {
        assert!(opt_jsobject.is_some() || opt_height.is_some());

        if width == 0 {
            return Err(Error::IndexSize);
        }

        // checking jsobject type and verifying (height * width * 4 == jsobject.byte_len())
        if let Some(jsobject) = opt_jsobject {
            let cx = global.get_cx();
            typedarray!(in(cx) let array_res: Uint8ClampedArray = jsobject);
            let array = array_res.map_err(|_| {
                Error::Type("Argument to Image data is not an Uint8ClampedArray".to_owned())
            })?;

            let byte_len = array.as_slice().len() as u32;
            if byte_len % 4 != 0 {
                return Err(Error::InvalidState);
            }

            let len = byte_len / 4;
            if width == 0 || len % width != 0 {
                return Err(Error::IndexSize);
            }

            let height = len / width;
            if opt_height.map_or(false, |x| height != x) {
                return Err(Error::IndexSize);
            } else {
                opt_height = Some(height);
            }
        }

        let height = opt_height.unwrap();
        if height == 0 {
            return Err(Error::IndexSize);
        }

        let imagedata = Box::new(ImageData {
            reflector_: Reflector::new(),
            width: width,
            height: height,
            data: Heap::default(),
        });

        if let Some(jsobject) = opt_jsobject {
            (*imagedata).data.set(jsobject);
        } else {
            let len = width * height * 4;
            let cx = global.get_cx();
            rooted!(in (cx) let mut array = ptr::null_mut::<JSObject>());
            Uint8ClampedArray::create(cx, CreateWith::Length(len), array.handle_mut()).unwrap();
            (*imagedata).data.set(array.get());
        }

        Ok(reflect_dom_object(
            imagedata,
            global,
            ImageDataBinding::Wrap,
        ))
    }

    // https://html.spec.whatwg.org/multipage/#pixel-manipulation:dom-imagedata-3
    #[allow(unsafe_code)]
    pub fn Constructor(global: &GlobalScope, width: u32, height: u32) -> Fallible<DomRoot<Self>> {
        unsafe { Self::new_with_jsobject(global, width, Some(height), None) }
    }

    // https://html.spec.whatwg.org/multipage/#pixel-manipulation:dom-imagedata-4
    #[allow(unsafe_code)]
    #[allow(unused_variables)]
    pub unsafe fn Constructor_(
        cx: *mut JSContext,
        global: &GlobalScope,
        jsobject: *mut JSObject,
        width: u32,
        opt_height: Option<u32>,
    ) -> Fallible<DomRoot<Self>> {
        Self::new_with_jsobject(global, width, opt_height, Some(jsobject))
    }

    /// Nothing must change the array on the JS side while the slice is live.
    #[allow(unsafe_code)]
    pub unsafe fn as_slice(&self) -> &[u8] {
        assert!(!self.data.get().is_null());
        let cx = Runtime::get();
        assert!(!cx.is_null());
        typedarray!(in(cx) let array: Uint8ClampedArray = self.data.get());
        let array = array.as_ref().unwrap();
        // NOTE(nox): This is just as unsafe as `as_slice` itself even though we
        // are extending the lifetime of the slice, because the data in
        // this ImageData instance will never change. The method is thus unsafe
        // because the array may be manipulated from JS while the reference
        // is live.
        let ptr = array.as_slice() as *const _;
        &*ptr
    }

    #[allow(unsafe_code)]
    pub fn to_shared_memory(&self) -> IpcSharedMemory {
        IpcSharedMemory::from_bytes(unsafe { self.as_slice() })
    }

    #[allow(unsafe_code)]
    pub unsafe fn get_rect(&self, rect: Rect<u32>) -> Cow<[u8]> {
        pixels::rgba8_get_rect(self.as_slice(), self.get_size(), rect)
    }

    pub fn get_size(&self) -> Size2D<u32> {
        Size2D::new(self.Width(), self.Height())
    }
}

impl ImageDataMethods for ImageData {
    // https://html.spec.whatwg.org/multipage/#dom-imagedata-width
    fn Width(&self) -> u32 {
        self.width
    }

    // https://html.spec.whatwg.org/multipage/#dom-imagedata-height
    fn Height(&self) -> u32 {
        self.height
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-imagedata-data
    unsafe fn Data(&self, _: *mut JSContext) -> NonNull<JSObject> {
        NonNull::new(self.data.get()).expect("got a null pointer")
    }
}
