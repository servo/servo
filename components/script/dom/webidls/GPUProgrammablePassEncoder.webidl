/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpuprogrammablepassencoder
[Exposed=(Window, DedicatedWorker)]
interface mixin GPUProgrammablePassEncoder {
    // void setBindGroup(unsigned long index, GPUBindGroup bindGroup,
    //                   optional sequence<unsigned long> dynamicOffsets = []);

    // void setBindGroup(unsigned long index, GPUBindGroup bindGroup,
    //                   Uint32Array dynamicOffsetsData,
    //                   unsigned long long dynamicOffsetsDataStart,
    //                   unsigned long long dynamicOffsetsDataLength);

    // void pushDebugGroup(DOMString groupLabel);
    // void popDebugGroup();
    // void insertDebugMarker(DOMString markerLabel);
};
