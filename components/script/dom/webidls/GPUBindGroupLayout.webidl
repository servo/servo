/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpubindgrouplayout
[Exposed=(Window, DedicatedWorker), Serializable, Pref="dom.webgpu.enabled"]
interface GPUBindGroupLayout {
};
GPUBindGroupLayout includes GPUObjectBase;

dictionary GPUBindGroupLayoutDescriptor : GPUObjectDescriptorBase {
    required sequence<GPUBindGroupLayoutEntry> entries;
};

dictionary GPUBindGroupLayoutEntry {
    required GPUIndex32 binding;
    required GPUShaderStageFlags visibility;
    GPUBufferBindingLayout buffer;
    GPUSamplerBindingLayout sampler;
    GPUTextureBindingLayout texture;
    GPUStorageTextureBindingLayout storageTexture;
};

enum GPUBufferBindingType {
    "uniform",
    "storage",
    "read-only-storage",
};

dictionary GPUBufferBindingLayout {
    GPUBufferBindingType type = "uniform";
    boolean hasDynamicOffset = false;
    GPUSize64 minBindingSize = 0;
};

enum GPUSamplerBindingType {
    "filtering",
    "non-filtering",
    "comparison",
};

dictionary GPUSamplerBindingLayout {
    GPUSamplerBindingType type = "filtering";
};

enum GPUTextureSampleType {
  "float",
  "unfilterable-float",
  "depth",
  "sint",
  "uint",
};

dictionary GPUTextureBindingLayout {
    GPUTextureSampleType sampleType = "float";
    GPUTextureViewDimension viewDimension = "2d";
    boolean multisampled = false;
};

enum GPUStorageTextureAccess {
    "write-only",
};

dictionary GPUStorageTextureBindingLayout {
    GPUStorageTextureAccess access = "write-only";
    required GPUTextureFormat format;
    GPUTextureViewDimension viewDimension = "2d";
};
