/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpurendercommandsmixin
[Exposed=(Window, DedicatedWorker)]
interface mixin GPURenderEncoderBase {
    undefined setPipeline(GPURenderPipeline pipeline);

    undefined setIndexBuffer(GPUBuffer buffer,
                             GPUIndexFormat indexFormat,
                             optional GPUSize64 offset = 0,
                             optional GPUSize64 size = 0);
    undefined setVertexBuffer(GPUIndex32 slot,
                             GPUBuffer buffer,
                             optional GPUSize64 offset = 0,
                             optional GPUSize64 size = 0);

    undefined draw(GPUSize32 vertexCount,
                   optional GPUSize32 instanceCount = 1,
                   optional GPUSize32 firstVertex = 0,
                   optional GPUSize32 firstInstance = 0);
    undefined drawIndexed(GPUSize32 indexCount,
                          optional GPUSize32 instanceCount = 1,
                          optional GPUSize32 firstIndex = 0,
                          optional GPUSignedOffset32 baseVertex = 0,
                          optional GPUSize32 firstInstance = 0);

    //[Pref="dom.webgpu.indirect-dispatch.enabled"]
    undefined drawIndirect(GPUBuffer indirectBuffer, GPUSize64 indirectOffset);
    //[Pref="dom.webgpu.indirect-dispatch.enabled"]
    undefined drawIndexedIndirect(GPUBuffer indirectBuffer, GPUSize64 indirectOffset);
};

typedef [EnforceRange] long GPUSignedOffset32;
