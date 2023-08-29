/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpushadermodule
[Exposed=(Window, DedicatedWorker), Serializable, Pref="dom.webgpu.enabled"]
interface GPUShaderModule {
    [Throws]
    Promise<GPUCompilationInfo> compilationInfo();
};
GPUShaderModule includes GPUObjectBase;

dictionary GPUShaderModuleDescriptor : GPUObjectDescriptorBase {
    // UTF8String is not observably different from USVString
    required USVString code;
    object sourceMap;
};
