/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpuadapter
[Exposed=(Window, DedicatedWorker), Pref="dom.webgpu.enabled"]
interface GPUAdapter {
    readonly attribute DOMString name;
    readonly attribute object extensions;
    //readonly attribute GPULimits limits; Don’t expose higher limits for now.

    // May reject with DOMException  // TODO: DOMException("OperationError")?
    // Promise<GPUDevice> requestDevice(optional GPUDeviceDescriptor descriptor = {});
};
