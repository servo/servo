/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpurenderencoderbase
[Exposed=(Window, DedicatedWorker)]
interface mixin GPURenderEncoderBase {
    void setPipeline(GPURenderPipeline pipeline);

    void setIndexBuffer(GPUBuffer buffer, optional GPUSize64 offset = 0, optional GPUSize64 size = 0);
    void setVertexBuffer(GPUIndex32 slot, GPUBuffer buffer, optional GPUSize64 offset = 0, optional GPUSize64 size = 0);

    void draw(GPUSize32 vertexCount, optional GPUSize32 instanceCount = 1,
              optional GPUSize32 firstVertex = 0, optional GPUSize32 firstInstance = 0);
    void drawIndexed(GPUSize32 indexCount, optional GPUSize32 instanceCount = 1,
                     optional GPUSize32 firstIndex = 0,
                     optional GPUSignedOffset32 baseVertex = 0,
                     optional GPUSize32 firstInstance = 0);

    void drawIndirect(GPUBuffer indirectBuffer, GPUSize64 indirectOffset);
    void drawIndexedIndirect(GPUBuffer indirectBuffer, GPUSize64 indirectOffset);
};

typedef [EnforceRange] long GPUSignedOffset32;
