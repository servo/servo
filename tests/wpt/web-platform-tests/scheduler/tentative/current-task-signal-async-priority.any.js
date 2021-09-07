// META: title=Scheduler: Signal inheritance
// META: global=window,worker
'use strict';

promise_test(t => {
  const controller = new TaskController({priority: 'user-blocking'});
  return scheduler.postTask(async () => {
    await new Promise(resolve => setTimeout(resolve, 0));
    assert_equals(scheduler.currentTaskSignal.priority, 'user-blocking');
  }, {signal: controller.signal});
}, 'Test that currentTaskSignal works through async promise resolution');
