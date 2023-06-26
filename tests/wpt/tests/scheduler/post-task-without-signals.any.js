// META: title=Scheduler: Basic Functionality without Signals
// META: global=window,worker
'use strict';

promise_test(async t => {
  for (const priority of ['user-blocking', 'user-visible', 'background']) {
    const result = await scheduler.postTask(() => priority, {priority});
    assert_equals(result, priority);
  }
}, 'Basic functionality for scheduler.postTask() without using TaskSignals');
