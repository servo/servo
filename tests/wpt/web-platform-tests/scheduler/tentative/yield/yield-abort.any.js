'use strict';

promise_test(t => {
  const signal = AbortSignal.abort();
  return scheduler.postTask(async () => {
    const p = scheduler.yield({signal});
    await promise_rejects_dom(t, 'AbortError', p);
  });
}, 'yield() with an aborted signal');

promise_test(t => {
  const controller = new TaskController();
  const signal = controller.signal;
  return scheduler.postTask(async () => {
    scheduler.postTask(async () => {controller.abort();}, {priority: 'user-blocking'});
    assert_false(signal.aborted);
    const p = scheduler.yield({signal});
    await promise_rejects_dom(t, 'AbortError', p);
  });
}, 'yield() aborted in a separate task');
