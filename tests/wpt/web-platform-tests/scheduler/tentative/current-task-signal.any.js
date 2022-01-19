// META: title=Scheduler: Signal inheritance
// META: global=window,worker
'use strict';

promise_test(async t => {
  const controller = new TaskController({priority: 'user-blocking'});
  let innerTask;

  await scheduler.postTask(() => {
    assert_equals(scheduler.currentTaskSignal.priority, 'user-blocking');
    innerTask = scheduler.postTask(() => {
      assert_equals(scheduler.currentTaskSignal.priority, 'user-blocking');
    }, {signal: scheduler.currentTaskSignal});
  }, {signal: controller.signal});

  return innerTask;
}, 'Test that currentTaskSignal uses the incumbent priority');
