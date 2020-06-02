/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUTextureViewBinding::GPUTextureViewDescriptor;
use crate::dom::bindings::codegen::Bindings::GPUTextureViewBinding::GPUTextureViewMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gputexture::GPUTexture;
use dom_struct::dom_struct;
use std::cell::Cell;
use std::hash::{Hash, Hasher};
use webgpu::WebGPUTextureView;

#[derive(MallocSizeOf, JSTraceable)]
pub struct TextureSubresource {
    pub texture: DomRoot<GPUTexture>,
    pub mipmap_level: u32,
    pub array_layer: u32,
}

impl PartialEq for TextureSubresource {
    fn eq(&self, other: &Self) -> bool {
        self.texture.id().0 == other.texture.id().0 &&
            self.mipmap_level == other.mipmap_level &&
            self.array_layer == other.array_layer
    }
}

impl Eq for TextureSubresource {}

impl Hash for TextureSubresource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.texture.id().0.hash(state);
        self.mipmap_level.hash(state);
        self.array_layer.hash(state);
    }
}

#[dom_struct]
pub struct GPUTextureView {
    reflector_: Reflector,
    label: DomRefCell<Option<DOMString>>,
    texture_view: WebGPUTextureView,
    texture: Dom<GPUTexture>,
    valid: Cell<bool>,
    #[ignore_malloc_size_of = "defined in webgpu"]
    descriptor: GPUTextureViewDescriptor,
}

impl GPUTextureView {
    fn new_inherited(
        texture_view: WebGPUTextureView,
        texture: &GPUTexture,
        valid: bool,
        descriptor: GPUTextureViewDescriptor,
    ) -> GPUTextureView {
        Self {
            reflector_: Reflector::new(),
            texture: Dom::from_ref(texture),
            label: DomRefCell::new(None),
            texture_view,
            valid: Cell::new(valid),
            descriptor,
        }
    }

    pub fn new(
        global: &GlobalScope,
        texture_view: WebGPUTextureView,
        texture: &GPUTexture,
        valid: bool,
        descriptor: GPUTextureViewDescriptor,
    ) -> DomRoot<GPUTextureView> {
        reflect_dom_object(
            Box::new(GPUTextureView::new_inherited(
                texture_view,
                texture,
                valid,
                descriptor,
            )),
            global,
        )
    }
}

impl GPUTextureView {
    pub fn id(&self) -> WebGPUTextureView {
        self.texture_view
    }

    pub fn is_valid(&self) -> bool {
        self.valid.get()
    }

    pub fn descriptor(&self) -> &GPUTextureViewDescriptor {
        &self.descriptor
    }

    pub fn texture(&self) -> &GPUTexture {
        &*self.texture
    }
}

impl GPUTextureViewMethods for GPUTextureView {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<DOMString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<DOMString>) {
        *self.label.borrow_mut() = value;
    }
}
