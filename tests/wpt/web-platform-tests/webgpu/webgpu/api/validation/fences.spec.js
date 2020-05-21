/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
fences validation tests.
`;
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { ValidationTest } from './validation_test.js';
export const g = makeTestGroup(ValidationTest); // TODO: Remove if https://github.com/gpuweb/gpuweb/issues/377 is decided

g.test('wait_on_a_fence_without_signaling_the_value_is_invalid').fn(async t => {
  const fence = t.queue.createFence();
  t.expectValidationError(() => {
    const promise = fence.onCompletion(2);
    t.shouldReject('OperationError', promise);
  });
}); // TODO: Remove if https://github.com/gpuweb/gpuweb/issues/377 is decided

g.test('wait_on_a_fence_with_a_value_greater_than_signaled_value_is_invalid').fn(async t => {
  const fence = t.queue.createFence();
  t.queue.signal(fence, 2);
  t.expectValidationError(() => {
    const promise = fence.onCompletion(3);
    t.shouldReject('OperationError', promise);
  });
});
g.test('signal_a_value_lower_than_signaled_value_is_invalid').fn(async t => {
  const fence = t.queue.createFence({
    initialValue: 1
  });
  t.expectValidationError(() => {
    t.queue.signal(fence, 0);
  });
});
g.test('signal_a_value_equal_to_signaled_value_is_invalid').fn(async t => {
  const fence = t.queue.createFence({
    initialValue: 1
  });
  t.expectValidationError(() => {
    t.queue.signal(fence, 1);
  });
});
g.test('increasing_fence_value_by_more_than_1_succeeds').fn(async t => {
  const fence = t.queue.createFence();
  t.queue.signal(fence, 2);
  await fence.onCompletion(2);
  t.queue.signal(fence, 6);
  await fence.onCompletion(6);
});
g.test('signal_a_fence_on_a_different_device_than_it_was_created_on_is_invalid').fn(async t => {
  const fence = t.queue.createFence();
  const anotherDevice = await t.device.adapter.requestDevice();
  const anotherQueue = anotherDevice.defaultQueue;
  t.expectValidationError(() => {
    anotherQueue.signal(fence, 2);
  });
});
g.test('signal_a_fence_on_a_different_device_does_not_update_fence_signaled_value').fn(async t => {
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