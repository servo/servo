/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

import { assert } from '../../../common/framework/util/util.js';
export async function runRefTest(fn) {
  assert(typeof navigator !== 'undefined' && navigator.gpu !== undefined, 'No WebGPU implementation found');
  const adapter = await navigator.gpu.requestAdapter();
  const device = await adapter.requestDevice();
  const queue = device.defaultQueue;
  await fn({
    device,
    queue
  });
  takeScreenshotDelayed(50);
}
//# sourceMappingURL=gpu_ref_test.js.map