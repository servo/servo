/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpucommandencoder
[Exposed=(Window, DedicatedWorker), Serializable, Pref="dom.webgpu.enabled"]
interface GPUCommandEncoder {
    // GPURenderPassEncoder beginRenderPass(GPURenderPassDescriptor descriptor);
    GPUComputePassEncoder beginComputePass(optional GPUComputePassDescriptor descriptor = {});

    void copyBufferToBuffer(
        GPUBuffer source,
        GPUBufferSize sourceOffset,
        GPUBuffer destination,
        GPUBufferSize destinationOffset,
        GPUBufferSize size);

    // void copyBufferToTexture(
    //     GPUBufferCopyView source,
    //     GPUTextureCopyView destination,
    //     GPUExtent3D copySize);

    // void copyTextureToBuffer(
    //     GPUTextureCopyView source,
    //     GPUBufferCopyView destination,
    //     GPUExtent3D copySize);

    // void copyTextureToTexture(
    //     GPUTextureCopyView source,
    //     GPUTextureCopyView destination,
    //     GPUExtent3D copySize);

    // void pushDebugGroup(DOMString groupLabel);
    // void popDebugGroup();
    // void insertDebugMarker(DOMString markerLabel);

    GPUCommandBuffer finish(optional GPUCommandBufferDescriptor descriptor = {});
};
GPUCommandEncoder includes GPUObjectBase;

dictionary GPUComputePassDescriptor : GPUObjectDescriptorBase {
};

dictionary GPUCommandBufferDescriptor : GPUObjectDescriptorBase {
};
