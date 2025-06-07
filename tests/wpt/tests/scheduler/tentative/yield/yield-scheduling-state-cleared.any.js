'use strict';

promise_test(async t => {
  const ids = [];
  // The timer task will run after the background task, and the scheduling state
  // set in the background task should not leak to the timer task.
  const {promise, resolve} = Promise.withResolvers();
  scheduler.postTask(async () => {
    setTimeout(async () => {
      let task = scheduler.postTask(() => {
        ids.push('task');
      }, {priority: 'user-visible'});
      await scheduler.yield();
      ids.push('continuation');
      await task;
      resolve();
    });
  }, {priority: 'background'});
  await promise;
  assert_equals(ids.toString(), 'continuation,task');
}, 'yield() does not leak priority across tasks');
