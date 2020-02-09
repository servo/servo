/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
fences validation tests.
`;
import { TestGroup } from '../../../framework/index.js';
import { ValidationTest } from './validation_test.js';
export const g = new TestGroup(ValidationTest); // TODO: Remove if https://github.com/gpuweb/gpuweb/issues/377 is decided

g.test('wait on a fence without signaling the value is invalid', async t => {
  const fence = t.queue.createFence();
  t.expectValidationError(() => {
    const promise = fence.onCompletion(2);
    t.shouldReject('OperationError', promise);
  });
}); // TODO: Remove if https://github.com/gpuweb/gpuweb/issues/377 is decided

g.test('wait on a fence with a value greater than signaled value is invalid', async t => {
  const fence = t.queue.createFence();
  t.queue.signal(fence, 2);
  t.expectValidationError(() => {
    const promise = fence.onCompletion(3);
    t.shouldReject('OperationError', promise);
  });
});
g.test('signal a value lower than signaled value is invalid', async t => {
  const fence = t.queue.createFence({
    initialValue: 1
  });
  t.expectValidationError(() => {
    t.queue.signal(fence, 0);
  });
});
g.test('signal a value equal to signaled value is invalid', async t => {
  const fence = t.queue.createFence({
    initialValue: 1
  });
  t.expectValidationError(() => {
    t.queue.signal(fence, 1);
  });
});
g.test('increasing fence value by more than 1 succeeds', async t => {
  const fence = t.queue.createFence();
  t.queue.signal(fence, 2);
  await fence.onCompletion(2);
  t.queue.signal(fence, 6);
  await fence.onCompletion(6);
});
g.test('signal a fence on a different device than it was created on is invalid', async t => {
  const fence = t.queue.createFence();
  const anotherDevice = await t.device.adapter.requestDevice();
  const anotherQueue = anotherDevice.defaultQueue;
  t.expectValidationError(() => {
    anotherQueue.signal(fence, 2);
  });
});
g.test('signal a fence on a different device does not update fence signaled value', async t => {
  const fence = t.queue.createFence({
    initialValue: 1
  });
  const anotherDevice = await t.device.adapter.requestDevice();
  const anotherQueue = anotherDevice.defaultQueue;
  t.expectValidationError(() => {
    anotherQueue.signal(fence, 2);
  });
  t.expect(fence.getCompletedValue() === 1);
  t.queue.signal(fence, 2);
  await fence.onCompletion(2);
});
//# sourceMappingURL=fences.spec.js.map