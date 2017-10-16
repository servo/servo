/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::nonnull::NonNullJSObjectPtr;
use dom::bindings::reflector::DomObject;
use dom::bindings::root::{DomRoot, MutNullableDom};
use dom::bindings::trace::JSTraceable;
use dom::webglrenderingcontext::WebGLRenderingContext;
use heapsize::HeapSizeOf;
use std::any::Any;
use super::{WebGLExtension, WebGLExtensions};

/// Trait used internally by WebGLExtensions to store and
/// handle the different WebGL extensions in a common list.
pub trait WebGLExtensionWrapper: JSTraceable + HeapSizeOf {
    fn instance_or_init(&self,
                        ctx: &WebGLRenderingContext,
                        ext: &WebGLExtensions)
                        -> NonNullJSObjectPtr;
    fn is_supported(&self, &WebGLExtensions) -> bool;
    fn is_enabled(&self) -> bool;
    fn enable(&self, ext: &WebGLExtensions);
    fn name(&self) -> &'static str;
    fn as_any(&self) -> &Any;
}

#[must_root]
#[derive(HeapSizeOf, JSTraceable)]
pub struct TypedWebGLExtensionWrapper<T: WebGLExtension> {
    extension: MutNullableDom<T::Extension>
}

/// Typed WebGL Extension implementation.
/// Exposes the exact MutNullableDom<DOMObject> type defined by the extension.
impl<T: WebGLExtension> TypedWebGLExtensionWrapper<T> {
    pub fn new() -> TypedWebGLExtensionWrapper<T> {
        TypedWebGLExtensionWrapper {
            extension: MutNullableDom::new(None)
        }
    }
}

impl<T> WebGLExtensionWrapper for TypedWebGLExtensionWrapper<T>
                              where T: WebGLExtension + JSTraceable + HeapSizeOf + 'static {
    #[allow(unsafe_code)]
    fn instance_or_init(&self,
                        ctx: &WebGLRenderingContext,
                        ext: &WebGLExtensions)
                        -> NonNullJSObjectPtr {
        let mut enabled = true;
        let extension = self.extension.or_init(|| {
            enabled = false;
            T::new(ctx)
        });
        if !enabled {
            self.enable(ext);
        }
        unsafe {
            NonNullJSObjectPtr::new_unchecked(extension.reflector().get_jsobject().get())
        }
    }

    fn is_supported(&self, ext: &WebGLExtensions) -> bool {
        self.is_enabled() || T::is_supported(ext)
    }

    fn is_enabled(&self) -> bool {
        self.extension.get().is_some()
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
    pub fn dom_object(&self) -> Option<DomRoot<T::Extension>> {
        self.extension.get()
    }
}
