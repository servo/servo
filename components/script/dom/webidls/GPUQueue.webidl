/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpuqueue
[Exposed=(Window, DedicatedWorker), Serializable, Pref="dom.webgpu.enabled"]
interface GPUQueue {
    undefined submit(sequence<GPUCommandBuffer> buffers);

    //TODO:
    //Promise<undefined> onSubmittedWorkDone();

    [Throws]
    undefined writeBuffer(
        GPUBuffer buffer,
        GPUSize64 bufferOffset,
        BufferSource data,
        optional GPUSize64 dataOffset = 0,
        optional GPUSize64 size);

    [Throws]
    undefined writeTexture(
      GPUImageCopyTexture destination,
      BufferSource data,
      GPUImageDataLayout dataLayout,
      GPUExtent3D size);

    //[Throws]
    //undefined copyExternalImageToTexture(
    //  GPUImageCopyExternalImage source,
    //  GPUImageCopyTextureTagged destination,
    //  GPUExtent3D copySize);
};
GPUQueue includes GPUObjectBase;
