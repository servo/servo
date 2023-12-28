/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for GPUDevice.lost.
`;import { Fixture } from '../../../../common/framework/fixture.js';
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { attemptGarbageCollection } from '../../../../common/util/collect_garbage.js';
import { getGPU } from '../../../../common/util/navigator_gpu.js';
import {
  assert,
  assertNotSettledWithinTime,
  raceWithRejectOnTimeout } from
'../../../../common/util/util.js';

class DeviceLostTests extends Fixture {
  // Default timeout for waiting for device lost is 2 seconds.
  kDeviceLostTimeoutMS = 2000;

  getDeviceLostWithTimeout(lost) {
    return raceWithRejectOnTimeout(lost, this.kDeviceLostTimeoutMS, 'device was not lost');
  }

  expectDeviceDestroyed(device) {
    this.eventualAsyncExpectation(async (niceStack) => {
      try {
        const lost = await this.getDeviceLostWithTimeout(device.lost);
        this.expect(lost.reason === 'destroyed', 'device was lost from destroy');
      } catch (ex) {
        niceStack.message = 'device was not lost';
        this.rec.expectationFailed(niceStack);
      }
    });
  }
}

export const g = makeTestGroup(DeviceLostTests);

g.test('not_lost_on_gc').
desc(
  `'lost' is never resolved by GPUDevice being garbage collected (with attemptGarbageCollection).`
).
fn(async (t) => {
  // Wraps a lost promise object creation in a function scope so that the device has the best
  // chance of being gone and ready for GC before trying to resolve the lost promise.
  const { lost } = await (async () => {
    const adapter = await getGPU(t.rec).requestAdapter();
    assert(adapter !== null);
    const lost = (await adapter.requestDevice()).lost;
    return { lost };
  })();
  await assertNotSettledWithinTime(lost, t.kDeviceLostTimeoutMS, 'device was unexpectedly lost');

  await attemptGarbageCollection();
});

g.test('lost_on_destroy').
desc(`'lost' is resolved, with reason='destroyed', on GPUDevice.destroy().`).
fn(async (t) => {
  const adapter = await getGPU(t.rec).requestAdapter();
  assert(adapter !== null);
  const device = await adapter.requestDevice();
  t.expectDeviceDestroyed(device);
  device.destroy();
});

g.test('same_object').
desc(`'lost' provides the same Promise and GPUDeviceLostInfo objects each time it's accessed.`).
fn(async (t) => {
  const adapter = await getGPU(t.rec).requestAdapter();
  assert(adapter !== null);
  const device = await adapter.requestDevice();

  // The promises should be the same promise object.
  const lostPromise1 = device.lost;
  const lostPromise2 = device.lost;
  t.expect(lostPromise1 === lostPromise2);

  // Promise object should still be the same after destroy.
  device.destroy();
  const lostPromise3 = device.lost;
  t.expect(lostPromise1 === lostPromise3);

  // The results should also be the same result object.
  const lost1 = await t.getDeviceLostWithTimeout(lostPromise1);
  const lost2 = await t.getDeviceLostWithTimeout(lostPromise2);
  const lost3 = await t.getDeviceLostWithTimeout(lostPromise3);
  // Promise object should still be the same after we've been notified about device loss.
  const lostPromise4 = device.lost;
  t.expect(lostPromise1 === lostPromise4);
  const lost4 = await t.getDeviceLostWithTimeout(lostPromise4);
  t.expect(lost1 === lost2 && lost2 === lost3 && lost3 === lost4);
});