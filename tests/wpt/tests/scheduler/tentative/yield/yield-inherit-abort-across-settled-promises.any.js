'use strict';

// Companion to yield-inherit-across-settled-promises.any.js, covering the
// other half of the scheduling state: the abort source.
//
// A WebTaskSchedulingState carries both a priority source and an abort source,
// so a lost state costs the continuation its abort link, not just its
// priority.
//
// The abort must come from a *separate, later* task. Aborting from inside the
// enclosing task would also reject the postTask() result independently of the
// scheduling state, masking a missing link on the continuation itself.
function abortFromSeparateTask(controller) {
  scheduler.postTask(() => { controller.abort(); },
                     {priority: 'user-blocking'});
}

// yield() is queued first, then aborted from a later task, so only an abort
// source inherited through the continuation can reject it.
function yieldThenAbort(t, controller) {
  abortFromSeparateTask(controller);
  t.step(() => assert_false(controller.signal.aborted));
  return promise_rejects_dom(t, 'AbortError', scheduler.yield());
}

// The enclosing task's promise may resolve normally: the abort lands after the
// callback has started, so postTask() resolves with whatever the callback
// returns. The assertion that matters is inside yieldThenAbort(), on the
// continuation itself, so the task's promise is awaited only to surface errors
// from the body.
function abortInheritanceTest(name, body) {
  promise_test(async t => {
    const controller = new TaskController();
    await scheduler.postTask(() => body(t, controller),
                             {signal: controller.signal});
  }, name);
}

abortInheritanceTest('yield() inherits abort with no intervening promise',
    (t, controller) => yieldThenAbort(t, controller));

abortInheritanceTest('yield() inherits abort across a pending promise',
    async (t, controller) => {
      await new Promise(resolve => queueMicrotask(resolve));
      await yieldThenAbort(t, controller);
    });

// Already-settled promises.
abortInheritanceTest('yield() inherits abort across an await of a settled promise',
    async (t, controller) => {
      await Promise.resolve();
      await yieldThenAbort(t, controller);
    });

abortInheritanceTest('yield() inherits abort across an await of a non-promise',
    async (t, controller) => {
      await null;
      await yieldThenAbort(t, controller);
    });

// yield() is called *inside* the reaction, since awaiting the result of then()
// would suspend on the pending promise then() returns and restore the state.
abortInheritanceTest('yield() inherits abort in then() on a settled promise',
    (t, controller) => Promise.resolve().then(
        () => yieldThenAbort(t, controller)));

abortInheritanceTest('yield() inherits abort in a chained then() on a settled promise',
    (t, controller) => Promise.resolve().then(() => {}).then(
        () => yieldThenAbort(t, controller)));

abortInheritanceTest('yield() inherits abort across an await of a thenable',
    async (t, controller) => {
      await {then(resolve) { resolve(); }};
      await yieldThenAbort(t, controller);
    });

abortInheritanceTest('yield() inherits abort in a thenable then()',
    (t, controller) => new Promise(resolve => resolve({
      then(res) { res(yieldThenAbort(t, controller)); }
    })));
