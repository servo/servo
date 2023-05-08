/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpu-interface
[Exposed=(Window, DedicatedWorker), Pref="dom.webgpu.enabled"]
interface GPU {
    // May reject with DOMException
    [NewObject]
    Promise<GPUAdapter?> requestAdapter(optional GPURequestAdapterOptions options = {});
    GPUTextureFormat getPreferredCanvasFormat();
};

dictionary GPURequestAdapterOptions {
    GPUPowerPreference powerPreference;
    boolean forceFallbackAdapter = false;
};

enum GPUPowerPreference {
    "low-power",
    "high-performance"
};
