/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpudevice
[Exposed=(Window, DedicatedWorker)/*, Serializable */, Pref="dom.webgpu.enabled"]
interface GPUDevice : EventTarget {
    readonly attribute GPUAdapter adapter;
    readonly attribute object extensions;
    readonly attribute object limits;

    GPUBuffer createBuffer(GPUBufferDescriptor descriptor);
    GPUMappedBuffer createBufferMapped(GPUBufferDescriptor descriptor);
    //Promise<GPUMappedBuffer> createBufferMappedAsync(GPUBufferDescriptor descriptor);
    //GPUTexture createTexture(GPUTextureDescriptor descriptor);
    //GPUSampler createSampler(optional GPUSamplerDescriptor descriptor = {});

    GPUBindGroupLayout createBindGroupLayout(GPUBindGroupLayoutDescriptor descriptor);
    GPUPipelineLayout createPipelineLayout(GPUPipelineLayoutDescriptor descriptor);
    /*GPUBindGroup createBindGroup(GPUBindGroupDescriptor descriptor);

    GPUShaderModule createShaderModule(GPUShaderModuleDescriptor descriptor);
    GPUComputePipeline createComputePipeline(GPUComputePipelineDescriptor descriptor);
    GPURenderPipeline createRenderPipeline(GPURenderPipelineDescriptor descriptor);

    GPUCommandEncoder createCommandEncoder(optional GPUCommandEncoderDescriptor descriptor = {});
    GPURenderBundleEncoder createRenderBundleEncoder(GPURenderBundleEncoderDescriptor descriptor);

    GPUQueue getQueue();*/
};
GPUDevice includes GPUObjectBase;
