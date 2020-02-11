/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpubuffer
[Exposed=(Window, DedicatedWorker), Serializable, Pref="dom.webgpu.enabled"]
interface GPUBuffer {
    Promise<ArrayBuffer> mapReadAsync();
    // Promise<ArrayBuffer> mapWriteAsync();
    void unmap();

    void destroy();
};
GPUBuffer includes GPUObjectBase;

dictionary GPUBufferDescriptor : GPUObjectDescriptorBase {
    required GPUBufferSize size;
    required GPUBufferUsageFlags usage;
};

typedef unsigned long long GPUBufferSize;

typedef unsigned long GPUBufferUsageFlags;

typedef sequence<any> GPUMappedBuffer;
