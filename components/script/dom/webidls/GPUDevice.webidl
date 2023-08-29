/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpudevice
[Exposed=(Window, DedicatedWorker), /*Serializable,*/ Pref="dom.webgpu.enabled"]
interface GPUDevice: EventTarget {
    //[SameObject] readonly attribute GPUSupportedFeatures features;
    [SameObject] readonly attribute GPUSupportedLimits limits;

    // Overriding the name to avoid collision with `class Queue` in gcc
    [SameObject, BinaryName="getQueue"] readonly attribute GPUQueue queue;

    undefined destroy();

    [NewObject, Throws]
    GPUBuffer createBuffer(GPUBufferDescriptor descriptor);
    [NewObject]
    GPUTexture createTexture(GPUTextureDescriptor descriptor);
    [NewObject]
    GPUSampler createSampler(optional GPUSamplerDescriptor descriptor = {});

    GPUBindGroupLayout createBindGroupLayout(GPUBindGroupLayoutDescriptor descriptor);
    GPUPipelineLayout createPipelineLayout(GPUPipelineLayoutDescriptor descriptor);
    GPUBindGroup createBindGroup(GPUBindGroupDescriptor descriptor);

    GPUShaderModule createShaderModule(GPUShaderModuleDescriptor descriptor);
    GPUComputePipeline createComputePipeline(GPUComputePipelineDescriptor descriptor);
    GPURenderPipeline createRenderPipeline(GPURenderPipelineDescriptor descriptor);

    [NewObject]
    Promise<GPUComputePipeline> createComputePipelineAsync(GPUComputePipelineDescriptor descriptor);
    [NewObject]
    Promise<GPURenderPipeline> createRenderPipelineAsync(GPURenderPipelineDescriptor descriptor);

    [NewObject]
    GPUCommandEncoder createCommandEncoder(optional GPUCommandEncoderDescriptor descriptor = {});
    [NewObject]
    GPURenderBundleEncoder createRenderBundleEncoder(GPURenderBundleEncoderDescriptor descriptor);
    //[NewObject]
    //GPUQuerySet createQuerySet(GPUQuerySetDescriptor descriptor);
};
GPUDevice includes GPUObjectBase;

//TODO
dictionary GPUCommandEncoderDescriptor : GPUObjectDescriptorBase {
    boolean measureExecutionTime = false;
};
