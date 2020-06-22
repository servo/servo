/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpuadapter
[Exposed=(Window, DedicatedWorker), Pref="dom.webgpu.enabled"]
interface GPUAdapter {
    readonly attribute DOMString name;
    readonly attribute object extensions;
    //readonly attribute GPULimits limits; Don’t expose higher limits for now.

    // May reject with DOMException  // TODO: DOMException("OperationError")?
    Promise<GPUDevice> requestDevice(optional GPUDeviceDescriptor descriptor = {});
};

dictionary GPUDeviceDescriptor : GPUObjectDescriptorBase {
    sequence<GPUExtensionName> extensions = [];
    GPULimits limits = {};
};

enum GPUExtensionName {
    "texture-compression-bc",
    "pipeline-statistics-query"
};

dictionary GPULimits {
    GPUSize32 maxBindGroups = 4;
    GPUSize32 maxDynamicUniformBuffersPerPipelineLayout = 8;
    GPUSize32 maxDynamicStorageBuffersPerPipelineLayout = 4;
    GPUSize32 maxSampledTexturesPerShaderStage = 16;
    GPUSize32 maxSamplersPerShaderStage = 16;
    GPUSize32 maxStorageBuffersPerShaderStage = 4;
    GPUSize32 maxStorageTexturesPerShaderStage = 4;
    GPUSize32 maxUniformBuffersPerShaderStage = 12;
    GPUSize32 maxUniformBufferBindingSize = 16384;
};
