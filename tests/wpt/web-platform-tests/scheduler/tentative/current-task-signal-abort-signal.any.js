// META: title=Scheduler: Signal inheritance
// META: global=window,worker
'use strict';

promise_test(async t => {
  const controller = new AbortController();
  let innerTask;
  await scheduler.postTask(() => {
    innerTask = scheduler.postTask(() => {}, {signal: scheduler.currentTaskSignal});
  }, {signal: controller.signal});
  controller.abort();
  return promise_rejects_dom(t, 'AbortError', innerTask);
}, 'Test that currentTaskSignal wraps and follows an AbortSignal');
