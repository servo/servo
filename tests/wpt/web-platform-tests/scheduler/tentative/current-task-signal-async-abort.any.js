// META: title=Scheduler: Signal inheritance
// META: global=window,worker
'use strict';

promise_test(t => {
  const controller = new TaskController({priority: 'user-blocking'});
  return scheduler.postTask(async () => {
    await new Promise(resolve => setTimeout(resolve, 0));
    const task = scheduler.postTask(() => {}, {signal: scheduler.currentTaskSignal});
    controller.abort();
    await promise_rejects_dom(t, 'AbortError', task);
  }, {signal: controller.signal});
}, 'Test that currentTaskSignal works through promise resolution when aborting tasks');
