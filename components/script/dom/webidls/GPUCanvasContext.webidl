/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */


[Exposed=(Window, DedicatedWorker), Pref="dom.webgpu.enabled"]
interface GPUCanvasContext {
    readonly attribute (HTMLCanvasElement or OffscreenCanvas) canvas;
};

// [Exposed=(Window, DedicatedWorker), Pref="dom.webgpu.enabled"]
// interface GPUCanvasContext {
//     readonly attribute (HTMLCanvasElement or OffscreenCanvas) canvas;

//     // Calling configure() a second time invalidates the previous one,
//     // and all of the textures it's produced.
//     [Throws]
//     undefined configure(GPUCanvasConfiguration descriptor);
//     undefined unconfigure();

//     [Throws]
//     GPUTexture getCurrentTexture();
// };