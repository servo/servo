// META: title=Scheduler: TaskController.abort() Aborts Correct Task
// META: global=window,worker
'use strict';

promise_test(async t => {
  const taskControllers = [];
  const taskResults = [];

  for (let i = 0; i < 5; i++) {
    const controller = new TaskController();
    taskControllers.push(controller);

    const signal = controller.signal;
    taskResults.push(scheduler.postTask(() => i, {signal}));
  }

  const abortedTask = taskResults.splice(2, 1)[0];
  taskControllers[2].abort();
  await promise_rejects_dom(t, 'AbortError', abortedTask);

  const result = await Promise.all(taskResults);
  assert_equals(result.toString(), '0,1,3,4');
}, 'Test aborting a task aborts the appropriate task');
