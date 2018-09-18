/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::reflector::DomObject;
use dom::bindings::root::MutNullableDom;
use dom::bindings::trace::JSTraceable;
use dom::webglrenderingcontext::WebGLRenderingContext;
use js::jsapi::JSObject;
use malloc_size_of::MallocSizeOf;
use std::any::Any;
use std::ptr::NonNull;
use super::{WebGLExtension, WebGLExtensions, WebGLExtensionSpec};

/// Trait used internally by WebGLExtensions to store and
/// handle the different WebGL extensions in a common list.
pub trait WebGLExtensionWrapper: JSTraceable + MallocSizeOf {
    fn instance_or_init(
        &self,
        ctx: &WebGLRenderingContext,
        ext: &WebGLExtensions,
    ) -> NonNull<JSObject>;
    fn spec(&self) -> WebGLExtensionSpec;
    fn is_supported(&self, &WebGLExtensions) -> bool;
    fn is_enabled(&self) -> bool;
    fn enable(&self, ext: &WebGLExtensions);
    fn name(&self) -> &'static str;
    fn as_any(&self) -> &Any;
}

#[must_root]
#[derive(JSTraceable, MallocSizeOf)]
pub struct TypedWebGLExtensionWrapper<T: WebGLExtension> {
    extension: MutNullableDom<T::Extension>,
}

/// Typed WebGL Extension implementation.
/// Exposes the exact MutNullableDom<DOMObject> type defined by the extension.
impl<T: WebGLExtension> TypedWebGLExtensionWrapper<T> {
    pub fn new() -> TypedWebGLExtensionWrapper<T> {
        TypedWebGLExtensionWrapper {
            extension: MutNullableDom::new(None),
        }
    }
}

impl<T> WebGLExtensionWrapper for TypedWebGLExtensionWrapper<T>
where
    T: WebGLExtension + JSTraceable + MallocSizeOf + 'static,
{
    #[allow(unsafe_code)]
    fn instance_or_init(
        &self,
        ctx: &WebGLRenderingContext,
        ext: &WebGLExtensions,
    ) -> NonNull<JSObject> {
        let mut enabled = true;
        let extension = self.extension.or_init(|| {
            enabled = false;
            T::new(ctx)
        });
        if !enabled {
            self.enable(ext);
        }
        unsafe { NonNull::new_unchecked(extension.reflector().get_jsobject().get()) }
    }

    fn spec(&self) -> WebGLExtensionSpec {
        T::spec()
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
