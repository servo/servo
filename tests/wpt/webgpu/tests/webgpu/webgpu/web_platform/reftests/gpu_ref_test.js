/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert } from '../../../common/util/util.js';import { takeScreenshotDelayed } from '../../../common/util/wpt_reftest_wait.js';





export function runRefTest(fn) {
  void (async () => {
    assert(
      typeof navigator !== 'undefined' && navigator.gpu !== undefined,
      'No WebGPU implementation found'
    );

    const adapter = await navigator.gpu.requestAdapter();
    assert(adapter !== null);
    const device = await adapter.requestDevice();
    assert(device !== null);
    const queue = device.queue;

    await fn({ device, queue });

    takeScreenshotDelayed(50);
  })();
}