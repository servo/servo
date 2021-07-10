/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpuprogrammablepassencoder
[Exposed=(Window, DedicatedWorker)]
interface mixin GPUProgrammablePassEncoder {
    void setBindGroup(GPUIndex32 index, GPUBindGroup bindGroup,
                      optional sequence<GPUBufferDynamicOffset> dynamicOffsets = []);

    // void setBindGroup(GPUIndex32 index, GPUBindGroup bindGroup,
    //                   Uint32Array dynamicOffsetsData,
    //                   GPUSize64 dynamicOffsetsDataStart,
    //                   GPUSize64 dynamicOffsetsDataLength);

    // void pushDebugGroup(DOMString groupLabel);
    // void popDebugGroup();
    // void insertDebugMarker(DOMString markerLabel);
};

typedef [EnforceRange] unsigned long GPUBufferDynamicOffset;
typedef [EnforceRange] unsigned long GPUIndex32;
