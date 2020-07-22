/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpurenderpassencoder
[Exposed=(Window, DedicatedWorker), Pref="dom.webgpu.enabled"]
interface GPURenderPassEncoder {
    void setViewport(float x, float y,
                     float width, float height,
                     float minDepth, float maxDepth);

    void setScissorRect(GPUIntegerCoordinate x, GPUIntegerCoordinate y,
                        GPUIntegerCoordinate width, GPUIntegerCoordinate height);

    void setBlendColor(GPUColor color);
    void setStencilReference(GPUStencilValue reference);

    //void beginOcclusionQuery(GPUSize32 queryIndex);
    //void endOcclusionQuery();

    //void beginPipelineStatisticsQuery(GPUQuerySet querySet, GPUSize32 queryIndex);
    //void endPipelineStatisticsQuery();

    //void writeTimestamp(GPUQuerySet querySet, GPUSize32 queryIndex);

    void executeBundles(sequence<GPURenderBundle> bundles);
    void endPass();
};
GPURenderPassEncoder includes GPUObjectBase;
GPURenderPassEncoder includes GPUProgrammablePassEncoder;
GPURenderPassEncoder includes GPURenderEncoderBase;
