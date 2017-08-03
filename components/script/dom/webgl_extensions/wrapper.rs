/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::nonzero::NonZero;
use dom::bindings::js::{MutNullableJS, Root};
use dom::bindings::reflector::DomObject;
use dom::bindings::trace::JSTraceable;
use dom::webglrenderingcontext::WebGLRenderingContext;
use heapsize::HeapSizeOf;
use js::jsapi::JSObject;
use std::any::Any;
use super::{WebGLExtension, WebGLExtensions};

/// Trait used internally by WebGLExtensions to store and
/// handle the different WebGL extensions in a common list.
pub trait WebGLExtensionWrapper: JSTraceable + HeapSizeOf {
    fn instance_or_init(&self,
                        ctx: &WebGLRenderingContext,
                        ext: &WebGLExtensions)
                        -> NonZero<*mut JSObject>;
    fn is_supported(&self, &WebGLExtensions) -> bool;
    fn enable(&self, ext: &WebGLExtensions);
    fn name(&self) -> &'static str;
    fn as_any(&self) -> &Any;
}

#[must_root]
#[derive(JSTraceable, HeapSizeOf)]
pub struct TypedWebGLExtensionWrapper<T: WebGLExtension> {
    extension: MutNullableJS<T::Extension>
}

/// Typed WebGL Extension implementation.
/// Exposes the exact MutNullableJS<DOMObject> type defined by the extension.
impl<T: WebGLExtension> TypedWebGLExtensionWrapper<T> {
    pub fn new() -> TypedWebGLExtensionWrapper<T> {
        TypedWebGLExtensionWrapper {
            extension: MutNullableJS::new(None)
        }
    }
}

impl<T> WebGLExtensionWrapper for TypedWebGLExtensionWrapper<T>
                              where T: WebGLExtension + JSTraceable + HeapSizeOf + 'static {
    #[allow(unsafe_code)]
    fn instance_or_init(&self,
                        ctx: &WebGLRenderingContext,
                        ext: &WebGLExtensions)
                        -> NonZero<*mut JSObject> {
        let mut enabled = true;
        let extension = self.extension.or_init(|| {
            enabled = false;
            T::new(ctx)
        });
        if !enabled {
            self.enable(ext);
        }
        unsafe {
            NonZero::new_unchecked(extension.reflector().get_jsobject().get())
        }
    }

    fn is_supported(&self, ext: &WebGLExtensions) -> bool {
        self.extension.get().is_some() || T::is_supported(ext)
    }

    fn enable(&self, ext: &WebGLExtensions) {
        T::enable(ext);
    }

    fn name(&self) -> &'static str {
        T::name()
    }

    fn as_any<'a>(&'a self) -> &'a Any {
        self
    }
}

impl<T> TypedWebGLExtensionWrapper<T> where T: WebGLExtension + JSTraceable + HeapSizeOf + 'static {
    pub fn dom_object(&self) -> Option<Root<T::Extension>> {
        self.extension.get()
    }
}
