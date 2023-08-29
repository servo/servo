/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpuadapterinfo
[Exposed=(Window, DedicatedWorker), Pref="dom.webgpu.enabled"]
interface GPUAdapterInfo {
    readonly attribute DOMString vendor;
    readonly attribute DOMString architecture;
    readonly attribute DOMString device;
    readonly attribute DOMString description;
};
