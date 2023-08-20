/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#typedefdef-gputextureusageflags
[Exposed=(Window, DedicatedWorker), Pref="dom.webgpu.enabled"]
interface GPUTextureUsage {
    const GPUTextureUsageFlags COPY_SRC          = 0x01;
    const GPUTextureUsageFlags COPY_DST          = 0x02;
    const GPUTextureUsageFlags TEXTURE_BINDING   = 0x04;
    const GPUTextureUsageFlags STORAGE_BINDING   = 0x08;
    const GPUTextureUsageFlags RENDER_ATTACHMENT = 0x10;
};

typedef [EnforceRange] unsigned long GPUTextureUsageFlags;
