/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpucomputepassencoder
[Exposed=(Window, DedicatedWorker), Serializable, Pref="dom.webgpu.enabled"]
interface GPUComputePassEncoder {
    undefined setPipeline(GPUComputePipeline pipeline);
    undefined dispatch(GPUSize32 x, optional GPUSize32 y = 1, optional GPUSize32 z = 1);
    undefined dispatchIndirect(GPUBuffer indirectBuffer, GPUSize64 indirectOffset);

    //void beginPipelineStatisticsQuery(GPUQuerySet querySet, GPUSize32 queryIndex);
    //void endPipelineStatisticsQuery();

    //void writeTimestamp(GPUQuerySet querySet, GPUSize32 queryIndex);

    undefined endPass();
};
GPUComputePassEncoder includes GPUObjectBase;
GPUComputePassEncoder includes GPUProgrammablePassEncoder;

typedef [EnforceRange] unsigned long GPUSize32;
