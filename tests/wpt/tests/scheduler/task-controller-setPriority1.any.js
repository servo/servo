// META: title=Scheduler: TaskController.setPriority()
// META: global=window,worker
'use strict';

promise_test(async t => {
  const controller = new TaskController();
  const signal = controller.signal;

  const tasks = [];
  const runOrder = [];
  const callback = id => { runOrder.push(id); };

  for (let i = 0; i < 5; i++)
    tasks.push(scheduler.postTask(() => callback(i), {signal}));
  tasks.push(scheduler.postTask(() => callback(5), {priority: 'user-blocking'}));
  tasks.push(scheduler.postTask(() => callback(6), {priority: 'user-visible' }));

  controller.setPriority('background');
  assert_equals(signal.priority, 'background');

  await Promise.all(tasks);

  assert_equals(runOrder.toString(), '5,6,0,1,2,3,4');
}, 'Test that TaskController.setPriority() changes the priority of all associated tasks');
