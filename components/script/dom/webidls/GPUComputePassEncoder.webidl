/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpucomputepassencoder
[Exposed=(Window, DedicatedWorker), Serializable, Pref="dom.webgpu.enabled"]
interface GPUComputePassEncoder {
    // void setPipeline(GPUComputePipeline pipeline);
    // void dispatch(unsigned long x, optional unsigned long y = 1, optional unsigned long z = 1);
    // void dispatchIndirect(GPUBuffer indirectBuffer, GPUBufferSize indirectOffset);

    // void endPass();
};
GPUComputePassEncoder includes GPUObjectBase;
GPUComputePassEncoder includes GPUProgrammablePassEncoder;
