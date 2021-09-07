// META: title=Scheduler: Signal inheritance
// META: global=window,worker
'use strict';

promise_test(t => {
  return scheduler.postTask(() => {
    assert_equals('user-blocking', scheduler.currentTaskSignal.priority);
  }, {priority: 'user-blocking'});
}, 'Test that currentTaskSignal propagates priority even if an explicit signal was not given');
