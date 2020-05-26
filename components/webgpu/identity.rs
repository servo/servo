/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;
use serde::{Deserialize, Serialize};
use wgpu::{
    hub::{GlobalIdentityHandlerFactory, IdentityHandler, IdentityHandlerFactory},
    id::{
        AdapterId, BindGroupId, BindGroupLayoutId, BufferId, CommandBufferId, ComputePipelineId,
        DeviceId, PipelineLayoutId, RenderPipelineId, SamplerId, ShaderModuleId, SurfaceId,
        SwapChainId, TextureId, TextureViewId, TypedId,
    },
};
use wgt::Backend;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
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
    Exit,
}

#[derive(Debug)]
pub struct IdentityRecycler {
    sender: IpcSender<WebGPUMsg>,
}

pub struct IdentityRecyclerFactory {
    pub sender: IpcSender<WebGPUMsg>,
}

macro_rules! impl_identity_handler {
    ($id:ty, $st:tt, $($var:tt)*) => {
        impl IdentityHandler<$id> for IdentityRecycler {
            type Input = $id;
            fn process(&self, id: $id, _backend: Backend) -> $id {
                log::debug!("process {} {:?}", $st, id);
                //debug_assert_eq!(id.unzip().2, backend);
                id
            }
            fn free(&self, id: $id) {
                log::debug!("free {} {:?}", $st, id);
                let msg = $($var)*(id);
                if self.sender.send(msg).is_err() {
                    log::error!("Failed to send {:?}", msg);
                }
            }
        }
    };
}

impl_identity_handler!(AdapterId, "adapter", WebGPUMsg::FreeAdapter);
impl_identity_handler!(DeviceId, "device", WebGPUMsg::FreeDevice);
impl_identity_handler!(SurfaceId, "surface", WebGPUMsg::FreeSurface);
impl_identity_handler!(SamplerId, "sampler", WebGPUMsg::FreeSampler);
impl_identity_handler!(TextureId, "texture", WebGPUMsg::FreeTexture);
impl_identity_handler!(TextureViewId, "texture_view", WebGPUMsg::FreeTextureView);
impl_identity_handler!(BufferId, "buffer", WebGPUMsg::FreeBuffer);
impl_identity_handler!(BindGroupId, "bind_group", WebGPUMsg::FreeBindGroup);
impl_identity_handler!(SwapChainId, "swap_chain", WebGPUMsg::FreeSwapChain);
impl_identity_handler!(ShaderModuleId, "shader_module", WebGPUMsg::FreeShaderModule);
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

impl<I: TypedId + Clone + std::fmt::Debug> IdentityHandlerFactory<I> for IdentityRecyclerFactory
where
    I: TypedId + Clone + std::fmt::Debug,
    IdentityRecycler: IdentityHandler<I>,
{
    type Filter = IdentityRecycler;
    fn spawn(&self, _min_index: u32) -> Self::Filter {
        IdentityRecycler {
            sender: self.sender.clone(),
        }
    }
}

impl GlobalIdentityHandlerFactory for IdentityRecyclerFactory {}
