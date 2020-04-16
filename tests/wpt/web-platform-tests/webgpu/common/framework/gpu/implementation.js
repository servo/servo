/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

/// <reference types="@webgpu/types" />
import { assert } from '../util/util.js';
let impl = undefined;
export function getGPU() {
  if (impl) {
    return impl;
  }

  assert(typeof navigator !== 'undefined' && navigator.gpu !== undefined, 'No WebGPU implementation found');
  impl = navigator.gpu;
  return impl;
}
//# sourceMappingURL=implementation.js.map