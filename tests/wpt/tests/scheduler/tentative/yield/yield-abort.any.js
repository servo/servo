'use strict';

promise_test(t => {
  const controller = new TaskController();
  const signal = controller.signal;
  const task = scheduler.postTask(async () => {
    controller.abort();
    const p = scheduler.yield();
    await promise_rejects_dom(t, 'AbortError', p);
  }, {signal});
  return promise_rejects_dom(t, 'AbortError', task);
}, 'yield() with an aborted signal');

promise_test(t => {
  const controller = new TaskController();
  const signal = controller.signal;
  return scheduler.postTask(async () => {
    scheduler.postTask(async () => {controller.abort();}, {priority: 'user-blocking'});
    t.step(() => assert_false(signal.aborted));
    const p = scheduler.yield();
    await promise_rejects_dom(t, 'AbortError', p);
  }, {signal});
}, 'yield() aborted by TaskController in a separate task');

promise_test(t => {
  const controller = new AbortController();
  const signal = controller.signal;
  return scheduler.postTask(async () => {
    scheduler.postTask(async () => {controller.abort();}, {priority: 'user-blocking'});
    t.step(() => assert_false(signal.aborted));
    const p = scheduler.yield();
    await promise_rejects_dom(t, 'AbortError', p);
  }, {signal});
}, 'yield() aborted by AbortController in a separate task');
