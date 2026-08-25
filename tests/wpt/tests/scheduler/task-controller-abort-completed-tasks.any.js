// META: title=Scheduler: Aborting Completed Tasks is a No-op
// META: global=window,worker
'use strict';

promise_test(async t => {
  const controller1 = new TaskController();
  const controller2 = new TaskController();

  await scheduler.postTask(() => {}, {signal: controller1.signal});

  const task = scheduler.postTask(() => {}, {signal: controller2.signal});
  controller2.abort();
  await promise_rejects_dom(t, 'AbortError', task);

  // The tasks associated with these controllers have completed, so this should
  // not lead to any unhandled rejections.
  controller1.abort();
  controller2.abort();
}, 'Aborting completed tasks should be a no-op.');
