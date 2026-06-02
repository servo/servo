// META: title=Scheduler: postTask and AbortSignal
// META: global=window,worker
'use strict';

promise_test(t => {
  const controller = new AbortController();
  const signal = controller.signal;
  const taskResult = scheduler.postTask(() => {}, {signal});
  controller.abort();
  return promise_rejects_dom(t, 'AbortError', taskResult);
}, 'Test that scheduler.postTask() accepts an AbortSignal that is not also a TaskSignal');
