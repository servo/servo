/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpucomputepassencoder
[Exposed=(Window, DedicatedWorker), Serializable, Pref="dom.webgpu.enabled"]
interface GPUComputePassEncoder {
    undefined setPipeline(GPUComputePipeline pipeline);
    undefined dispatchWorkgroups(GPUSize32 x, optional GPUSize32 y = 1, optional GPUSize32 z = 1);
    //[Pref="dom.webgpu.indirect-dispatch.enabled"]
    undefined dispatchWorkgroupsIndirect(GPUBuffer indirectBuffer, GPUSize64 indirectOffset);

    [Throws]
    undefined end();
};
GPUComputePassEncoder includes GPUObjectBase;
GPUComputePassEncoder includes GPUProgrammablePassEncoder;

typedef [EnforceRange] unsigned long GPUSize32;
