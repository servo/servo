'use strict';

function postInheritPriorityTestTask(config) {
  const ids = [];
  const task = scheduler.postTask(async () => {
    await new Promise(resolve => setTimeout(resolve));
    await fetch('/common/blank.html');
    await new Promise(resolve => setTimeout(resolve));
    const subtask = scheduler.postTask(() => { ids.push('subtask'); }, {priority: config.subTaskPriority});
    await scheduler.yield();
    ids.push('yield');
    await subtask;
  }, config.taskOptions);
  return {task, ids}
}

for (let priority of ['user-blocking', 'background']) {
  const expected = priority == 'user-blocking' ? 'yield,subtask' : 'subtask,yield';
  promise_test(async t => {
    const config = {
      taskOptions: {priority},
      subTaskPriority: 'user-blocking',
    };
    const {task, ids} = postInheritPriorityTestTask(config);
    await task;
    assert_equals(ids.join(), expected);
  }, `yield() inherits priority (string) across promises (${priority})`);

  promise_test(async t => {
    const signal = (new TaskController({priority})).signal;
    const config = {
      taskOptions: {signal},
      subTaskPriority: 'user-blocking',
    };
    const {task, ids} = postInheritPriorityTestTask(config);
    await task;
    assert_equals(ids.join(), expected);
  }, `yield() inherits priority (signal) across promises (${priority})`);
}

promise_test(async t => {
  const controller = new TaskController();
  const signal = controller.signal;
  return scheduler.postTask(async () => {
    await new Promise(resolve => setTimeout(resolve));
    await fetch('/common/blank.html');
    await new Promise(resolve => setTimeout(resolve));
    controller.abort();
    const p = scheduler.yield();
    await promise_rejects_dom(t, 'AbortError', p);
  }, {signal});
}, `yield() inherits abort across promises`);

promise_test(async t => {
  const ids = [];
  let {promise: p1, resolve} = Promise.withResolvers();
  // For promises, the scheduling state is bound to the future microtask when
  // the promise is awaited or .then() is called on it. This tests that the
  // right scheduling state is used, i.e. not the "resolve time" state.
  //
  // First, create a pending continuation (.then(...)) bound to the current
  // (null) scheduling state. The continuation calls yield(), which should
  // inherit the null scheduling state.
  p1 = p1.then(async () => {
    await scheduler.yield();
    ids.push('continuation');
  });
  // Next, resolve `p1` in a user-blocking task. The user-blocking scheduling
  // state should not be propagated to the continuation above.
  await scheduler.postTask(resolve, {priority: 'user-blocking'});
  // Finally, to test this, race another user-blocking task with the `p1`
  // continuation above. The continuation should run after this task, since it
  // should not inherit the user-blocking priority.
  const p2 = scheduler.postTask(() => {
    ids.push('task');
  }, {priority: 'user-blocking'});

  const result = await Promise.all([p1, p2]);
  assert_equals(ids.toString(), 'task,continuation');
}, 'yield() inherits .then() context, not resolve context');

promise_test(async t => {
  // This tests that non-promise microtasks also inherit scheduling state by
  // checking that the scheduling state is propagated from queueMicrotask() to
  // the subsequent microtask.
  //
  // First, create a pending continuation (.then(...)) which will be bound to
  // the current (null) context. The yield() below should have default priority.
  const ids = [];
  let {promise: p1, resolve} = Promise.withResolvers();
  p1 = p1.then(async () => {
    ids.push('p1-start');
    await scheduler.yield();
    ids.push('p1-continuation');
  });

  // Next, schedule a task which resolves `p1` and then calls queueMicrotask().
  // This is done to interleave the microtasks in a way that we can ensure
  // queueMicrotask() actually propagates scheduling state, rather than using
  // the state set when the postTask() callback starts.
  //
  // The yield() below should inherit the user-blocking priority.
  const p2 = scheduler.postTask(async () => {
    resolve();
    queueMicrotask(async () => {
      ids.push('p2-start');
      await scheduler.yield();
      ids.push('p2-continuation');
    })
  }, {priority: 'user-blocking'});

  // Finally, schedule another task to race with the contents of the `p2` task
  // above. Both yield() calls above happen during the `p2` task microtask
  // checkpoint, so both continuations are scheduled when the `p3` task below
  // runs. The p2-continuation (user-blocking continuation) should run before
  // the `p3` task, and the p1-continuation (default prioriy continuation)
  // should run after.
  const p3 = scheduler.postTask(() => {
    ids.push('p3');
  }, {priority: 'user-blocking'});

  await Promise.all([p1, p2, p3]);
  assert_equals(
      ids.toString(), "p1-start,p2-start,p2-continuation,p3,p1-continuation");
}, 'yield() inherits priority in queueMicrotask()');
