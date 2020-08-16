/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { assert } from '../../../common/framework/util/util.js';

export async function runRefTest(fn) {
  assert(
    typeof navigator !== 'undefined' && navigator.gpu !== undefined,
    'No WebGPU implementation found'
  );

  const adapter = await navigator.gpu.requestAdapter();
  assert(adapter !== null);
  const device = await adapter.requestDevice();
  assert(device !== null);
  const queue = device.defaultQueue;

  await fn({ device, queue });

  takeScreenshotDelayed(50);
}
