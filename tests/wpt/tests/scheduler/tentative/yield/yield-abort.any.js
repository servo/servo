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
  const task = scheduler.postTask(async () => {
    controller.abort();
    const p = scheduler.yield({signal: 'inherit'});
    await promise_rejects_dom(t, 'AbortError', p);
  }, {signal});
  return promise_rejects_dom(t, 'AbortError', task);
}, 'yield() with an aborted signal (inherit signal)');

promise_test(t => {
  const controller = new TaskController();
  const signal = controller.signal;
  const task = scheduler.postTask(async () => {
    controller.abort();
    const p = scheduler.yield({signal: 'inherit', priority: 'background'});
    await promise_rejects_dom(t, 'AbortError', p);
  }, {signal});
  return promise_rejects_dom(t, 'AbortError', task);
}, 'yield() with an aborted signal (inherit signal priority override)');

promise_test(t => {
  const controller = new TaskController();
  const signal = controller.signal;
  const task = scheduler.postTask(async () => {
    controller.abort();
    await scheduler.yield({priority: 'inherit'});
  }, {signal});
  return promise_rejects_dom(t, 'AbortError', task);
}, 'yield() with an aborted signal (inherit priority only)');

promise_test(t => {
  const controller = new TaskController();
  const signal = controller.signal;
  return scheduler.postTask(async () => {
    scheduler.postTask(async () => {controller.abort();}, {priority: 'user-blocking'});
    t.step(() => assert_false(signal.aborted));
    const p = scheduler.yield({signal});
    await promise_rejects_dom(t, 'AbortError', p);
  });
}, 'yield() aborted in a separate task');

promise_test(t => {
  const controller = new TaskController();
  const signal = controller.signal;
  return scheduler.postTask(async () => {
    scheduler.postTask(async () => {controller.abort();}, {priority: 'user-blocking'});
    t.step(() => assert_false(signal.aborted));
    const p = scheduler.yield({signal: 'inherit'});
    await promise_rejects_dom(t, 'AbortError', p);
  }, {signal});
}, 'yield() aborted in a separate task (inherit)');

promise_test(t => {
  const controller = new TaskController();
  const signal = controller.signal;
  return scheduler.postTask(async () => {
    scheduler.postTask(async () => {controller.abort();}, {priority: 'user-blocking'});
    t.step(() => assert_false(signal.aborted));
    const p = scheduler.yield({signal: 'inherit', priority: 'background'});
    await promise_rejects_dom(t, 'AbortError', p);
  }, {signal});
}, 'yield() aborted in a separate task (inherit signal priority override)');

promise_test(t => {
  const controller = new TaskController();
  const signal = controller.signal;
  return scheduler.postTask(async () => {
    scheduler.postTask(async () => {controller.abort();}, {priority: 'user-blocking'});
    t.step(() => assert_false(signal.aborted));
    await scheduler.yield({priority: 'inherit'});
    t.step(() => assert_true(signal.aborted));
  }, {signal});
}, 'yield() aborted in a separate task (inherit priority only)');
