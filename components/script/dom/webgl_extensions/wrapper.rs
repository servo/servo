/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr::NonNull;

use js::jsapi::JSObject;
use malloc_size_of::MallocSizeOf;

use super::{WebGLExtension, WebGLExtensionSpec, WebGLExtensions};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::MutNullableDom;
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::script_runtime::CanGc;

/// Trait used internally by WebGLExtensions to store and
/// handle the different WebGL extensions in a common list.
pub(crate) trait WebGLExtensionWrapper: JSTraceable + MallocSizeOf {
    fn instance_or_init(
        &self,
        ctx: &WebGLRenderingContext,
        ext: &WebGLExtensions,
    ) -> NonNull<JSObject>;
    fn spec(&self) -> WebGLExtensionSpec;
    fn is_supported(&self, _: &WebGLExtensions) -> bool;
    fn is_enabled(&self) -> bool;
    fn enable(&self, ext: &WebGLExtensions);
    fn name(&self) -> &'static str;
}

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct TypedWebGLExtensionWrapper<T: WebGLExtension> {
    extension: MutNullableDom<T::Extension>,
}

/// Typed WebGL Extension implementation.
/// Exposes the exact `MutNullableDom<DOMObject>` type defined by the extension.
impl<T: WebGLExtension> TypedWebGLExtensionWrapper<T> {
    pub(crate) fn new() -> TypedWebGLExtensionWrapper<T> {
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
            T::new(ctx, CanGc::note())
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
}
