'use strict';

// The scheduling state of the enclosing task must be inherited by
// continuations that descend from it through microtasks, regardless of how the
// microtask was created.
//
// Per https://wicg.github.io/scheduling-apis/#sec-patches-html-hostmakejobcallback
// HostMakeJobCallback captures the event loop's current continuation state
// unconditionally, and HostCallJobCallback restores it.
//
// Each test runs in a 'background' postTask task and runs a yield()
// continuation. As well, there is a 'user-visible' task posted just
// before the yield.
//
// If the scheduling state was inherited correctly, the continuation is
// 'background' and runs after the task. If the scheduling state is
// lost, the continuation gets the default 'user-visible' priority and
// runs before the 'background'.

function raceContinuationAgainstUserVisibleTask(ids) {
  const task = scheduler.postTask(() => {
    ids.push('task');
  }, {priority: 'user-visible'});
  return scheduler.yield().then(() => {
    ids.push('continuation');
    return task;
  });
}

function inheritanceTest(name, body) {
  promise_test(async t => {
    const ids = [];
    await scheduler.postTask(() => body(ids), {priority: 'background'});
    assert_equals(ids.join(), 'task,continuation');
  }, name);
}

inheritanceTest('yield() inherits priority with no intervening promise',
    (ids) => raceContinuationAgainstUserVisibleTask(ids));

inheritanceTest('yield() inherits priority across a pending promise',
    async (ids) => {
      await new Promise(resolve => queueMicrotask(resolve));
      await raceContinuationAgainstUserVisibleTask(ids);
    });

// Already-settled promises.
inheritanceTest('yield() inherits priority across an await of a settled promise',
    async (ids) => {
      await Promise.resolve();
      await raceContinuationAgainstUserVisibleTask(ids);
    });

inheritanceTest('yield() inherits priority across an await of a non-promise',
    async (ids) => {
      await null;
      await raceContinuationAgainstUserVisibleTask(ids);
    });

inheritanceTest('yield() inherits priority in then() on a settled promise',
    (ids) => Promise.resolve().then(
        () => raceContinuationAgainstUserVisibleTask(ids)));

inheritanceTest('yield() inherits priority in a chained then() on a settled promise',
    (ids) => Promise.resolve().then(() => {}).then(
        () => raceContinuationAgainstUserVisibleTask(ids)));

inheritanceTest('yield() inherits priority across an await of a thenable',
    async (ids) => {
      await {then(resolve) { resolve(); }};
      await raceContinuationAgainstUserVisibleTask(ids);
    });

inheritanceTest('yield() inherits priority in a thenable then()',
    (ids) => new Promise(resolve => resolve({
      then(res) { res(raceContinuationAgainstUserVisibleTask(ids)); }
    })));

inheritanceTest('yield() inherits priority across a promise resolved with a thenable',
    async (ids) => {
      await new Promise(resolve => resolve({then(r) { r(); }}));
      await raceContinuationAgainstUserVisibleTask(ids);
    });
