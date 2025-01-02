// META: title=Scheduler: TaskController.abort() with Signal and Priority
// META: global=window,worker
'use strict';

promise_test(async t => {
  const controller = new TaskController();
  const signal = controller.signal;

  const task1 = scheduler.postTask(() => {}, {signal});
  const task2 = scheduler.postTask(() => {}, {priority: 'background', signal});

  controller.abort();

  await promise_rejects_dom(t, 'AbortError',  task1);
  return promise_rejects_dom(t, 'AbortError',  task2);
}, 'Test that when scheduler.postTask() is given both a signal and priority, the signal abort is honored');
