/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpu-interface
[Exposed=(Window, DedicatedWorker), Pref="dom.webgpu.enabled"]
interface GPU {
    Promise<GPUAdapter?> requestAdapter(optional GPURequestAdapterOptions options = {});
};

// https://gpuweb.github.io/gpuweb/#dictdef-gpurequestadapteroptions
dictionary GPURequestAdapterOptions {
    GPUPowerPreference powerPreference;
};

// https://gpuweb.github.io/gpuweb/#enumdef-gpupowerpreference
enum GPUPowerPreference {
    "low-power",
    "high-performance"
};
