// META: title=Scheduler: postTask uses abort reason
// META: global=window,worker
'use strict';

promise_test(t => {
  const controller = new TaskController();
  const signal = controller.signal;
  const reason = new Error("Custom Abort Error");
  controller.abort(reason);
  return promise_rejects_exactly(t, reason, scheduler.postTask(() => {}, {signal}));
}, 'Calling postTask with an aborted TaskSignal rejects the promise with the abort reason');

promise_test(t => {
  const controller = new AbortController();
  const signal = controller.signal;
  const reason = new Error("Custom Abort Error");
  controller.abort(reason);
  return promise_rejects_exactly(t, reason, scheduler.postTask(() => {}, {signal}));
}, 'Calling postTask with an aborted AbortSignal rejects the promise with the abort reason');

promise_test(t => {
  const controller = new TaskController();
  const signal = controller.signal;
  const reason = new Error("Custom Abort Error");
  const result = scheduler.postTask(() => {}, {signal});
  controller.abort(reason);
  return promise_rejects_exactly(t, reason, result);
}, 'Aborting a TaskSignal rejects the promise of a scheduled task with the abort reason');

promise_test(t => {
  const reason = new Error("Custom Abort Error");
  const controller = new AbortController();
  const signal = controller.signal;
  const result = scheduler.postTask(() => {}, {signal});
  controller.abort(reason);
  return promise_rejects_exactly(t, reason, result);
}, 'Aborting an AbortSignal rejects the promise of a scheduled task with the abort reason');
