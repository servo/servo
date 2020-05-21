/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = '';
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { attemptGarbageCollection } from '../../../common/framework/util/collect_garbage.js';
import { raceWithRejectOnTimeout } from '../../../common/framework/util/util.js';
import { GPUTest } from '../../gpu_test.js';
export const g = makeTestGroup(GPUTest);
g.test('initial,no_descriptor').fn(t => {
  const fence = t.queue.createFence();
  t.expect(fence.getCompletedValue() === 0);
});
g.test('initial,empty_descriptor').fn(t => {
  const fence = t.queue.createFence({});
  t.expect(fence.getCompletedValue() === 0);
});
g.test('initial,descriptor_with_initialValue').fn(t => {
  const fence = t.queue.createFence({
    initialValue: 2
  });
  t.expect(fence.getCompletedValue() === 2);
}); // Promise resolves when onCompletion value is less than signal value.

g.test('wait,less_than_signaled').fn(async t => {
  const fence = t.queue.createFence();
  t.queue.signal(fence, 2);
  await fence.onCompletion(1);
  t.expect(fence.getCompletedValue() === 2);
}); // Promise resolves when onCompletion value is equal to signal value.

g.test('wait,equal_to_signaled').fn(async t => {
  const fence = t.queue.createFence();
  t.queue.signal(fence, 2);
  await fence.onCompletion(2);
  t.expect(fence.getCompletedValue() === 2);
}); // All promises resolve when signal is called once.

g.test('wait,signaled_once').fn(async t => {
  const fence = t.queue.createFence();
  t.queue.signal(fence, 20);
  const promises = [];

  for (let i = 0; i <= 20; ++i) {
    promises.push(fence.onCompletion(i).then(() => {
      t.expect(fence.getCompletedValue() >= i);
    }));
  }

  await Promise.all(promises);
}); // Promise resolves when signal is called multiple times.

g.test('wait,signaled_multiple_times').fn(async t => {
  const fence = t.queue.createFence();
  t.queue.signal(fence, 1);
  t.queue.signal(fence, 2);
  await fence.onCompletion(2);
  t.expect(fence.getCompletedValue() === 2);
}); // Promise resolves if fence has already completed.

g.test('wait,already_completed').fn(async t => {
  const fence = t.queue.createFence();
  t.queue.signal(fence, 2); // Wait for value to update.

  while (fence.getCompletedValue() < 2) {
    await new Promise(resolve => {
      requestAnimationFrame(resolve);
    });
  }

  t.expect(fence.getCompletedValue() === 2);
  await fence.onCompletion(2);
  t.expect(fence.getCompletedValue() === 2);
}); // Test many calls to signal and wait on fence values one at a time.

g.test('wait,many,serially').fn(async t => {
  const fence = t.queue.createFence();

  for (let i = 1; i <= 20; ++i) {
    t.queue.signal(fence, i);
    await fence.onCompletion(i);
    t.expect(fence.getCompletedValue() === i);
  }
}); // Test many calls to signal and wait on all fence values.

g.test('wait,many,parallel').fn(async t => {
  const fence = t.queue.createFence();
  const promises = [];

  for (let i = 1; i <= 20; ++i) {
    t.queue.signal(fence, i);
    promises.push(fence.onCompletion(i).then(() => {
      t.expect(fence.getCompletedValue() >= i);
    }));
  }

  await Promise.all(promises);
  t.expect(fence.getCompletedValue() === 20);
}); // Test onCompletion promise resolves within a time limit.

g.test('wait,resolves_within_timeout').fn(t => {
  const fence = t.queue.createFence();
  t.queue.signal(fence, 2);
  return raceWithRejectOnTimeout((async () => {
    await fence.onCompletion(2);
    t.expect(fence.getCompletedValue() === 2);
  })(), 100, 'The fence has not been resolved within time limit.');
}); // Test dropping references to the fence and onCompletion promise does not crash.

g.test('drop,fence_and_promise').fn(async t => {
  {
    const fence = t.queue.createFence();
    t.queue.signal(fence, 2);
    fence.onCompletion(2);
  }
  await attemptGarbageCollection();
}); // Test dropping references to the fence and holding the promise does not crash.

g.test('drop,promise').fn(async t => {
  let promise;
  {
    const fence = t.queue.createFence();
    t.queue.signal(fence, 2);
    promise = fence.onCompletion(2);
  }
  await attemptGarbageCollection();
  await promise;
});
//# sourceMappingURL=fences.spec.js.map