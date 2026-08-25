// META: title=Scheduler: Signal and Priority Combination
// META: global=window,worker
'use strict';

promise_test(async t => {
  const task1Result = scheduler.postTask(() => 'task1', {priority: 'user-visible'});

  const controller = new TaskController({priority: 'background'});
  const signal = controller.signal
  const task2Result = scheduler.postTask(() => 'task2', {priority: 'user-blocking', signal});

  const result = await Promise.race([task1Result, task2Result]);
  assert_equals('task2', result);
}, 'Test when scheduler.postTask() is passed both a signal and a priority');
