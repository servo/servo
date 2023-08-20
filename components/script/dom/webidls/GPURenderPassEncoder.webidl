/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpurenderpassencoder
[Exposed=(Window, DedicatedWorker), Pref="dom.webgpu.enabled"]
interface GPURenderPassEncoder {
    undefined setViewport(float x, float y,
                          float width, float height,
                          float minDepth, float maxDepth);

    undefined setScissorRect(GPUIntegerCoordinate x, GPUIntegerCoordinate y,
                             GPUIntegerCoordinate width, GPUIntegerCoordinate height);

    undefined setBlendConstant(GPUColor color);
    undefined setStencilReference(GPUStencilValue reference);

    //undefined beginOcclusionQuery(GPUSize32 queryIndex);
    //undefined endOcclusionQuery();

    //undefined beginPipelineStatisticsQuery(GPUQuerySet querySet, GPUSize32 queryIndex);
    //undefined endPipelineStatisticsQuery();

    //undefined writeTimestamp(GPUQuerySet querySet, GPUSize32 queryIndex);

    undefined executeBundles(sequence<GPURenderBundle> bundles);

    [Throws]
    undefined end();
};
GPURenderPassEncoder includes GPUObjectBase;
GPURenderPassEncoder includes GPUProgrammablePassEncoder;
GPURenderPassEncoder includes GPURenderEncoderBase;
