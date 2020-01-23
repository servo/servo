/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpubindgrouplayout
[Exposed=(Window, DedicatedWorker), Serializable, Pref="dom.webgpu.enabled"]
interface GPUBindGroup {
};
GPUBindGroup includes GPUObjectBase;

dictionary GPUBindGroupDescriptor : GPUObjectDescriptorBase {
    required GPUBindGroupLayout layout;
    required sequence<GPUBindGroupBindings> bindings;
};

typedef /*(GPUSampler or GPUTextureView or*/ GPUBufferBindings/*)*/ GPUBindingResource;

// Note: Servo codegen doesn't like the name `GPUBindGroupBinding` because it's already occupied
// dictionary GPUBindGroupBinding {
dictionary GPUBindGroupBindings {
    required unsigned long binding;
    required GPUBindingResource resource;
};

// Note: Servo codegen doesn't like the name `GPUBufferBinding` because it's already occupied
// dictionary GPUBufferBinding {
dictionary GPUBufferBindings {
    required GPUBuffer buffer;
    GPUBufferSize offset = 0;
    GPUBufferSize size;
};
