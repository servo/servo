/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpuadapter
[Exposed=(Window, DedicatedWorker), Pref="dom.webgpu.enabled"]
interface GPUAdapter {
    [SameObject] readonly attribute GPUSupportedFeatures features;
    [SameObject] readonly attribute GPUSupportedLimits limits;
    readonly attribute boolean isFallbackAdapter;

    [NewObject]
    Promise<GPUDevice> requestDevice(optional GPUDeviceDescriptor descriptor = {});
    [NewObject]
    Promise<GPUAdapterInfo> requestAdapterInfo(optional sequence<DOMString> unmaskHints = []);
};

dictionary GPUDeviceDescriptor {
    sequence<GPUFeatureName> requiredFeatures = [];
    record<DOMString, GPUSize64> requiredLimits;
};

enum GPUFeatureName {
    "depth-clip-control",
    "depth24unorm-stencil8",
    "depth32float-stencil8",
    "pipeline-statistics-query",
    "texture-compression-bc",
    "texture-compression-etc2",
    "texture-compression-astc",
    "timestamp-query",
    "indirect-first-instance",
};
