/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{ErrorScopeId, WebGPUDevice, WebGPURequest};
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::PipelineId;
use serde::{Deserialize, Serialize};
use wgpu::{
    hub::{GlobalIdentityHandlerFactory, IdentityHandler, IdentityHandlerFactory},
    id::{
        AdapterId, BindGroupId, BindGroupLayoutId, BufferId, CommandBufferId, ComputePipelineId,
        DeviceId, PipelineLayoutId, RenderBundleId, RenderPipelineId, SamplerId, ShaderModuleId,
        SurfaceId, SwapChainId, TextureId, TextureViewId, TypedId,
    },
};
use wgt::Backend;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebGPUOpResult {
    ValidationError(String),
    OutOfMemoryError,
    Success,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebGPUMsg {
    FreeAdapter(AdapterId),
    FreeDevice(DeviceId),
    FreeBuffer(BufferId),
    FreeSwapChain(SwapChainId),
    FreePipelineLayout(PipelineLayoutId),
    FreeComputePipeline(ComputePipelineId),
    FreeRenderPipeline(RenderPipelineId),
    FreeBindGroup(BindGroupId),
    FreeBindGroupLayout(BindGroupLayoutId),
    FreeCommandBuffer(CommandBufferId),
    FreeTexture(TextureId),
    FreeTextureView(TextureViewId),
    FreeSampler(SamplerId),
    FreeSurface(SurfaceId),
    FreeShaderModule(ShaderModuleId),
    FreeRenderBundle(RenderBundleId),
    WebGPUOpResult {
        device: WebGPUDevice,
        scope_id: Option<ErrorScopeId>,
        pipeline_id: PipelineId,
        result: WebGPUOpResult,
    },
    CleanDevice {
        device: WebGPUDevice,
        pipeline_id: PipelineId,
    },
    Exit,
}

#[derive(Debug)]
pub struct IdentityRecycler {
    sender: IpcSender<WebGPUMsg>,
    self_sender: IpcSender<(Option<ErrorScopeId>, WebGPURequest)>,
}

pub struct IdentityRecyclerFactory {
    pub sender: IpcSender<WebGPUMsg>,
    pub self_sender: IpcSender<(Option<ErrorScopeId>, WebGPURequest)>,
}

macro_rules! impl_identity_handler {
    ($id:ty, $st:tt, $($var:tt)*) => {
        impl IdentityHandler<$id> for IdentityRecycler {
            type Input = $id;
            fn process(&self, id: $id, _backend: Backend) -> Self::Input {
                log::debug!("process {} {:?}", $st, id);
                //debug_assert_eq!(id.unzip().2, backend);
                id
            }
            fn free(&self, id: $id) {
                log::debug!("free {} {:?}", $st, id);
                let msg = $($var)*(id);
                if self.sender.send(msg.clone()).is_err() {
                    log::error!("Failed to send {:?}", msg);
                }
            }
        }
    };
}

impl_identity_handler!(AdapterId, "adapter", WebGPUMsg::FreeAdapter);
impl_identity_handler!(SurfaceId, "surface", WebGPUMsg::FreeSurface);
impl_identity_handler!(SamplerId, "sampler", WebGPUMsg::FreeSampler);
impl_identity_handler!(TextureId, "texture", WebGPUMsg::FreeTexture);
impl_identity_handler!(TextureViewId, "texture_view", WebGPUMsg::FreeTextureView);
impl_identity_handler!(BufferId, "buffer", WebGPUMsg::FreeBuffer);
impl_identity_handler!(BindGroupId, "bind_group", WebGPUMsg::FreeBindGroup);
impl_identity_handler!(SwapChainId, "swap_chain", WebGPUMsg::FreeSwapChain);
impl_identity_handler!(ShaderModuleId, "shader_module", WebGPUMsg::FreeShaderModule);
impl_identity_handler!(RenderBundleId, "render_bundle", WebGPUMsg::FreeRenderBundle);
impl_identity_handler!(
    RenderPipelineId,
    "render_pipeline",
    WebGPUMsg::FreeRenderPipeline
);
impl_identity_handler!(
    ComputePipelineId,
    "compute_pipeline",
    WebGPUMsg::FreeComputePipeline
);
impl_identity_handler!(
    CommandBufferId,
    "command_buffer",
    WebGPUMsg::FreeCommandBuffer
);
impl_identity_handler!(
    BindGroupLayoutId,
    "bind_group_layout",
    WebGPUMsg::FreeBindGroupLayout
);
impl_identity_handler!(
    PipelineLayoutId,
    "pipeline_layout",
    WebGPUMsg::FreePipelineLayout
);

impl IdentityHandler<DeviceId> for IdentityRecycler {
    type Input = DeviceId;
    fn process(&self, id: DeviceId, _backend: Backend) -> Self::Input {
        log::debug!("process device {:?}", id);
        //debug_assert_eq!(id.unzip().2, backend);
        id
    }
    fn free(&self, id: DeviceId) {
        log::debug!("free device {:?}", id);
        if self.sender.send(WebGPUMsg::FreeDevice(id)).is_err() {
            log::error!("Failed to send FreeDevice({:?}) to script", id);
        }
        if self
            .self_sender
            .send((None, WebGPURequest::FreeDevice(id)))
            .is_err()
        {
            log::error!("Failed to send FreeDevice({:?}) to server", id);
        }
    }
}

impl<I: TypedId + Clone + std::fmt::Debug> IdentityHandlerFactory<I> for IdentityRecyclerFactory
where
    I: TypedId + Clone + std::fmt::Debug,
    IdentityRecycler: IdentityHandler<I>,
{
    type Filter = IdentityRecycler;
    fn spawn(&self, _min_index: u32) -> Self::Filter {
        IdentityRecycler {
            sender: self.sender.clone(),
            self_sender: self.self_sender.clone(),
        }
    }
}

impl GlobalIdentityHandlerFactory for IdentityRecyclerFactory {}
