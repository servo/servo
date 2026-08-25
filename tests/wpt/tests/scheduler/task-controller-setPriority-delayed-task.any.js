// META: title=Scheduler: Change Delayed Task Priority
// META: global=window,worker
'use strict';

promise_test(t => {
  let taskCount = 0;
  const start = performance.now();
  const controller = new TaskController({priority: 'background'});

  const task1 = scheduler.postTask(() => {
    assert_equals(++taskCount, 1);
    controller.setPriority('user-blocking');
  }, {priority: 'user-blocking', delay: 10});

  const task2 = scheduler.postTask(() => {
    assert_equals(++taskCount, 2);

    const elapsed = performance.now() - start;

    if(navigator.userAgent.toLowerCase().indexOf('firefox') > -1){
      // Firefox returns the timings with different precision,
      // so we put 19 here.
      assert_greater_than_equal(elapsed, 19);
    } else {
      assert_greater_than_equal(elapsed, 20);
    }
  }, {signal: controller.signal, delay: 20});

  return Promise.all([task1, task2]);

}, "Tests delay when changing a delayed task's priority");
