/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use euclid::default::Size2D;
use js::context::JSContext;
use pixels::Snapshot;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WebGPUBinding::GPUDeviceMethods as _;
use script_bindings::error::Fallible;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use webgpu_traits::{
    WebGPU, WebGPUDevice, WebGPUExternalTexture, WebGPUQueue, WebGPURequest, WebGPUTexture,
    WebGPUTextureView,
};
use wgpu_types::Features;

use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUExternalTextureDescriptor, GPUExternalTextureMethods,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal as _;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpudevice::GPUDevice;

/// Backing of GPUExternalTexture
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct PlanarTexture {
    #[ignore_malloc_size_of = "defined in webgpu"]
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
    device_id: WebGPUDevice,
    #[no_trace]
    queue_id: WebGPUQueue,
    #[no_trace]
    texture_id: WebGPUTexture,
    #[no_trace]
    texture_view_id: WebGPUTextureView,
    expired: Cell<bool>,
    #[no_trace]
    size: Size2D<u32>,
}

impl PlanarTexture {
    pub(crate) fn new(channel: WebGPU, device: &GPUDevice, snapshot: Snapshot) -> Self {
        let device_id = device.id();
        let queue_id = device.queue_id();
        let texture_id = WebGPUTexture(device.global().wgpu_id_hub().create_texture_id());
        let texture_view_id =
            WebGPUTextureView(device.global().wgpu_id_hub().create_texture_view_id());
        let size = snapshot.size();
        if let Err(error) = channel.0.send(WebGPURequest::CreatePlanarTexture {
            device_id: device_id.0,
            texture_id: texture_id.0,
            texture_view_id: texture_view_id.0,
            size,
            format: snapshot.format(),
        }) {
            warn!("Failed to send CreatePlanarTexture ({error})");
        }
        let self_ = Self {
            channel,
            device_id,
            queue_id,
            texture_id,
            texture_view_id,
            size,
            expired: Cell::new(true),
        };
        self_.update(snapshot);
        self_
    }

    pub(crate) fn size(&self) -> Size2D<u32> {
        self.size
    }

    pub(crate) fn update(&self, snapshot: Snapshot) {
        if !self.expired.get() {
            return;
        }
        if let Err(error) = self.channel.0.send(WebGPURequest::UpdatePlanarTexture {
            device_id: self.device_id.0,
            queue_id: self.queue_id.0,
            texture_id: self.texture_id.0,
            snapshot: snapshot.to_shared(),
        }) {
            warn!("Failed to send UpdatePlanarTexture ({error})");
        }
        self.expired.set(false);
    }

    pub(crate) fn expire(&self) {
        self.expired.set(true);
    }

    pub(crate) fn is_expired(&self) -> bool {
        self.expired.get()
    }
}

impl Drop for PlanarTexture {
    fn drop(&mut self) {
        if let Err(error) = self.channel.0.send(WebGPURequest::DropPlanarTexture(
            self.texture_id.0,
            self.texture_view_id.0,
        )) {
            warn!("Failed to send DropPlanarTexture ({error})");
        }
    }
}

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableGPUExternalTexture {
    #[ignore_malloc_size_of = "defined in webgpu"]
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
    external_texture: WebGPUExternalTexture,
}

impl Drop for DroppableGPUExternalTexture {
    fn drop(&mut self) {
        if let Err(error) = self
            .channel
            .0
            .send(WebGPURequest::DropExternalTexture(self.external_texture.0))
        {
            warn!(
                "Failed to send DropExternalTexture ({:?}) ({error})",
                self.external_texture.0
            );
        }
    }
}

#[dom_struct]
pub(crate) struct GPUExternalTexture {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    #[ignore_malloc_size_of = "rc"]
    planar_texture: Option<Rc<PlanarTexture>>,
    droppable: DroppableGPUExternalTexture,
}

impl GPUExternalTexture {
    fn new_inherited(
        channel: WebGPU,
        external_texture: WebGPUExternalTexture,
        label: USVString,
        planar_texture: Option<Rc<PlanarTexture>>,
    ) -> GPUExternalTexture {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            droppable: DroppableGPUExternalTexture {
                channel,
                external_texture,
            },
            planar_texture,
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        channel: WebGPU,
        external_texture: WebGPUExternalTexture,
        label: USVString,
        planar_texture: Option<Rc<PlanarTexture>>,
    ) -> DomRoot<GPUExternalTexture> {
        reflect_dom_object_with_cx(
            Box::new(GPUExternalTexture::new_inherited(
                channel,
                external_texture,
                label,
                planar_texture,
            )),
            global,
            cx,
        )
    }

    pub(crate) fn expire(&self) {
        if let Some(planar_texture) = &self.planar_texture {
            planar_texture.expire();
        }
        if let Err(error) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::DestroyExternalTexture(
                self.droppable.external_texture.0,
            ))
        {
            warn!(
                "Failed to send DestroyExternalTexture ({:?}) ({error})",
                self.droppable.external_texture.0
            );
        }
    }

    /// <https://www.w3.org/TR/webgpu/#dom-gpudevice-importexternaltexture>
    pub(crate) fn create(
        cx: &mut JSContext,
        device: &super::gpudevice::GPUDevice,
        descriptor: &GPUExternalTextureDescriptor,
    ) -> Fallible<DomRoot<GPUExternalTexture>> {
        let (size, planar_texture) = if device
            .Features()
            .wgpu_features()
            .contains(Features::EXTERNAL_TEXTURE)
        {
            descriptor.source.planar_video_for_webgpu(device)?
        } else {
            // spec assumes that this is always supported, but that is not the case in wgpu
            return Err(Error::NotSupported(Some(
                "ExternalTexture is not supported on this device".to_string(),
            )));
        };
        // 5. Let result be a new GPUExternalTexture object wrapping data.
        let device_id = device.id().0;
        let channel = device.channel();
        let external_texture_id = device.global().wgpu_id_hub().create_external_texture_id();

        if let Err(error) = channel.0.send(WebGPURequest::ImportExternalTexture {
            device_id,
            external_texture_id,
            size,
            label: descriptor.parent.label.to_string(),
            plane0: planar_texture
                .as_ref()
                .map(|planar_texture| planar_texture.texture_view_id.0),
        }) {
            warn!("Failed to send ImportExternalTexture ({error})");
        };
        let result = Self::new(
            cx,
            &device.global(),
            channel,
            WebGPUExternalTexture(external_texture_id),
            // Set result.label to descriptor.label.
            descriptor.parent.label.clone(),
            planar_texture,
        );
        // If source is an HTMLVideoElement, queue an automatic expiry task with device this and the following steps
        let this = Trusted::new(&*result);
        device
            .global()
            .task_manager()
            .webgpu_task_source()
            .queue(task!(expire: move || {
                this.root().expire();
            }));

        // Return result.
        Ok(result)
    }
}

impl GPUExternalTexture {
    pub(crate) fn id(&self) -> WebGPUExternalTexture {
        self.droppable.external_texture
    }
}

impl GPUExternalTextureMethods<crate::DomTypeHolder> for GPUExternalTexture {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }
}
