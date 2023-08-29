/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#typedefdef-gpushaderstageflags
[Exposed=(Window, DedicatedWorker), Serializable, Pref="dom.webgpu.enabled"]
interface GPUShaderStage {
    const GPUShaderStageFlags VERTEX = 1;
    const GPUShaderStageFlags FRAGMENT = 2;
    const GPUShaderStageFlags COMPUTE = 4;
};

typedef [EnforceRange] unsigned long GPUShaderStageFlags;
