/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

/// <reference types="@webgpu/types" />
let impl = undefined;
export function getGPU() {
  if (impl) {
    return impl;
  }

  if (typeof navigator === 'undefined' || navigator.gpu === undefined) {
    throw new Error('No WebGPU implementation found');
  }

  impl = navigator.gpu;
  return impl;
}
//# sourceMappingURL=implementation.js.map