// META: title=Scheduler: postTask with a signal and abort the signal when running the callback
// META: global=window,worker
'use strict';

promise_test(t => {
  const controller = new TaskController();
  const signal = controller.signal;
  return promise_rejects_dom(t, 'AbortError', scheduler.postTask(() => {
    controller.abort();
  }, { signal }));
}, 'Posting a task with a signal and abort the signal when running the sync callback');

promise_test(t => {
  const controller = new TaskController();
  const signal = controller.signal;
  return scheduler.postTask(async () => {
    await new Promise(resolve => t.step_timeout(resolve, 0));
    controller.abort();
  }, { signal });
}, 'Posting a task with a signal and abort the signal when running the async callback');
