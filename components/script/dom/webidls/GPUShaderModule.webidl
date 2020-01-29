/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpushadermodule
[Exposed=(Window, DedicatedWorker), Serializable, Pref="dom.webgpu.enabled"]
interface GPUShaderModule {
};
GPUShaderModule includes GPUObjectBase;

typedef (Uint32Array or DOMString) GPUShaderCode;

dictionary GPUShaderModuleDescriptor : GPUObjectDescriptorBase {
    required GPUShaderCode code;
};
