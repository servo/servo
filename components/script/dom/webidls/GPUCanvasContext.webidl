/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpucanvascontext
[Exposed=(Window, DedicatedWorker), Pref="dom.webgpu.enabled"]
interface GPUCanvasContext {
    readonly attribute (HTMLCanvasElement or OffscreenCanvas) canvas;

    // Calling configure() a second time invalidates the previous one,
    // and all of the textures it's produced.
    undefined configure(GPUCanvasConfiguration descriptor);
    undefined unconfigure();

    [Throws]
    GPUTexture getCurrentTexture();
};

enum GPUCanvasAlphaMode {
    "opaque",
    "premultiplied",
};

dictionary GPUCanvasConfiguration {
    required GPUDevice device;
    required GPUTextureFormat format;
    GPUTextureUsageFlags usage = 0x10;  // GPUTextureUsage.RENDER_ATTACHMENT
    sequence<GPUTextureFormat> viewFormats = [];
    // PredefinedColorSpace colorSpace = "srgb"; // TODO
    GPUCanvasAlphaMode alphaMode = "opaque";
};
