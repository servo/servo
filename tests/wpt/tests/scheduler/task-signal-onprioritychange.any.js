// META: title=Scheduler: TaskSignal onprioritychange
// META: global=window,worker
'use strict';

async_test(t => {
  const controller = new TaskController({priority: 'user-visible'});
  controller.signal.onprioritychange = t.step_func_done((event) => {
    assert_equals(controller.signal.priority, 'background');
    assert_equals(event.type, 'prioritychange');
    assert_equals(event.target.priority, 'background');
    assert_equals(event.previousPriority, 'user-visible');
  });
  controller.setPriority('background');
}, 'Test that TaskSignal.onprioritychange listens for prioritychange events');
