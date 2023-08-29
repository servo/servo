/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpudevicelostinfo
[Exposed=(Window, DedicatedWorker), Pref="dom.webgpu.enabled"]
interface GPUDeviceLostInfo {
    readonly attribute GPUDeviceLostReason reason;
    readonly attribute DOMString message;
};

enum GPUDeviceLostReason {
    "unknown",
    "destroyed",
};

partial interface GPUDevice {
    [Throws]
    readonly attribute Promise<GPUDeviceLostInfo> lost;
};
