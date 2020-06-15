/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpucanvascontext
[Exposed=(Window, DedicatedWorker), Pref="dom.webgpu.enabled"]
interface GPUCanvasContext {
    GPUSwapChain configureSwapChain(GPUSwapChainDescriptor descriptor);

    //Promise<GPUTextureFormat> getSwapChainPreferredFormat(GPUDevice device);
};

dictionary GPUSwapChainDescriptor : GPUObjectDescriptorBase {
    required GPUDevice device;
    required GPUTextureFormat format;
    GPUTextureUsageFlags usage = 0x10;  // GPUTextureUsage.OUTPUT_ATTACHMENT
};
