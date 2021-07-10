/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpuqueue
[Exposed=(Window, DedicatedWorker), Serializable, Pref="dom.webgpu.enabled"]
interface GPUQueue {
    void submit(sequence<GPUCommandBuffer> commandBuffers);

    //GPUFence createFence(optional GPUFenceDescriptor descriptor = {});
    //void signal(GPUFence fence, GPUFenceValue signalValue);

    [Throws] void writeBuffer(
        GPUBuffer buffer,
        GPUSize64 bufferOffset,
        /*[AllowShared]*/ BufferSource data,
        optional GPUSize64 dataOffset = 0,
        optional GPUSize64 size);

    [Throws] void writeTexture(
      GPUTextureCopyView destination,
      /*[AllowShared]*/ BufferSource data,
      GPUTextureDataLayout dataLayout,
      GPUExtent3D size);

    //void copyImageBitmapToTexture(
    //    GPUImageBitmapCopyView source,
    //    GPUTextureCopyView destination,
    //    GPUExtent3D copySize);
};
GPUQueue includes GPUObjectBase;
