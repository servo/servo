// META: title=Scheduler: TaskController.setPriority() repeated calls
// META: global=window,worker
'use strict';

promise_test(async t => {
  const controller = new TaskController();
  const signal = controller.signal;

  const tasks = [];
  const runOrder = [];
  const callback = id => { runOrder.push(id); };

  tasks.push(scheduler.postTask(() => callback(0), {signal}));
  tasks.push(scheduler.postTask(() => callback(1), {priority: 'user-blocking'}));
  tasks.push(scheduler.postTask(() => callback(2), {priority: 'user-visible' }));

  controller.setPriority('background');
  assert_equals(signal.priority, 'background');

  await Promise.all(tasks);
  assert_equals(runOrder.toString(), '1,2,0');

  while (tasks.length) { tasks.pop(); }
  while (runOrder.length) { runOrder.pop(); }

  tasks.push(scheduler.postTask(() => callback(3), {signal}));
  tasks.push(scheduler.postTask(() => callback(4), {priority: 'user-blocking'}));
  tasks.push(scheduler.postTask(() => callback(5), {priority: 'user-visible' }));

  controller.setPriority('user-blocking');
  assert_equals(signal.priority, 'user-blocking');

  await Promise.all(tasks);
  assert_equals(runOrder.toString(), '3,4,5');
}, 'TaskController.setPriority() changes the priority of all associated tasks when called repeatedly');

promise_test(async t => {
  const controller = new TaskController();
  const signal = controller.signal;

  const tasks = [];
  const runOrder = [];
  const callback = id => { runOrder.push(id); };

  tasks.push(scheduler.postTask(() => callback(0), {signal}));
  tasks.push(scheduler.postTask(() => callback(1), {priority: 'user-blocking'}));
  tasks.push(scheduler.postTask(() => callback(2), {priority: 'user-visible' }));

  controller.setPriority('background');
  assert_equals(signal.priority, 'background');

  controller.setPriority('user-visible');
  assert_equals(signal.priority, 'user-visible');

  controller.setPriority('user-blocking');
  assert_equals(signal.priority, 'user-blocking');

  await Promise.all(tasks);
  assert_equals(runOrder.toString(), '0,1,2');
}, 'TaskController.setPriority() changes the priority of all associated tasks when called repeatedly before tasks run');
