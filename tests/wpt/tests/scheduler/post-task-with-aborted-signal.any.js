// META: title=Scheduler: postTask with an Aborted Signal
// META: global=window,worker
'use strict';

promise_test(t => {
  const controller = new TaskController();
  const signal = controller.signal;
  controller.abort();
  return promise_rejects_dom(t, 'AbortError', scheduler.postTask(() => {}, {signal}));
}, 'Posting a task with an aborted signal rejects with an AbortError');
