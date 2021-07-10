/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpumapmode
[Exposed=(Window, DedicatedWorker), Pref="dom.webgpu.enabled"]
interface GPUMapMode {
    const GPUMapModeFlags READ  = 0x0001;
    const GPUMapModeFlags WRITE = 0x0002;
};

typedef [EnforceRange] unsigned long GPUMapModeFlags;
