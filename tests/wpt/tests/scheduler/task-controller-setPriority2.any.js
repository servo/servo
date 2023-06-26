// META: title=Scheduler: TaskController.setPriority and Task Order
// META: global=window,worker
'use strict';

promise_test(async t => {
  const tasks = [];
  const runOrder = [];
  const taskControllers = [];

  for (let i = 0; i < 5; i++) {
    taskControllers.push(new TaskController({priority: 'background'}));
    const signal = taskControllers[i].signal;
    tasks.push(scheduler.postTask(() => { runOrder.push(i); }, {signal}));
  }

  taskControllers[2].setPriority('user-blocking');
  assert_equals(taskControllers[2].signal.priority, 'user-blocking');

  await Promise.all(tasks);

  assert_equals(runOrder.toString(), '2,0,1,3,4');
}, 'Test TaskController.setPriority() affects task order.');
