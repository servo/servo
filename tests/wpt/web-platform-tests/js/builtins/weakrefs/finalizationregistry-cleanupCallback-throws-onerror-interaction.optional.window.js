// META: script=resources/maybe-garbage-collect.js
// ├──> maybeGarbageCollectAsync
// └──> resolveGarbageCollection
/*---
esid: sec-finalization-registry-target
info: |
  FinalizationRegistry ( cleanupCallback )

  CleanupFinalizationRegistry ( finalizationRegistry [ , callback ] )

  The following steps are performed:

  Assert: finalizationRegistry has [[Cells]] and [[CleanupCallback]] internal slots.
  If callback is not present or undefined, set callback to finalizationRegistry.[[CleanupCallback]].
  While finalizationRegistry.[[Cells]] contains a Record cell such that cell.[[WeakRefTarget]] is
  empty, then an implementation may perform the following steps,
    Choose any such cell.
    Remove cell from finalizationRegistry.[[Cells]].
    Perform ? Call(callback, undefined, « cell.[[HeldValue]] »).
  Return NormalCompletion(undefined).

  EDITOR'S NOTE
  When called from HostCleanupFinalizationRegistry, if calling the callback throws an error, this will be caught within the RunJobs algorithm and reported to the host. HTML does not apply the RunJobs algorithm, but will also report the error, which may call window.onerror.
---*/

let error = new Error('FinalizationRegistryError');

let finalizationRegistry = new FinalizationRegistry(function() {
  throw error;
});

setup({ allow_uncaught_exception: true });

promise_test((test) => {
  assert_implements(
    typeof FinalizationRegistry.prototype.register === 'function',
    'FinalizationRegistry.prototype.register is not implemented.'
  );

  return (async () => {

    let resolve;
    let reject;
    let deferred = new Promise((resolverFn, rejecterFn) => {
      resolve = resolverFn;
      reject = rejecterFn;
    });

    window.onerror = test.step_func((message, source, lineno, colno, exception) => {
      assert_equals(exception, error, 'window.onerror received the intended error object.');
      resolve();
    });

    {
      let target = {};
      let heldValue = 1;
      finalizationRegistry.register(target, heldValue);
      target = null;
    }

    await maybeGarbageCollectAsync();

    // Since the process of garbage collection is non-deterministic, we cannot know when
    // (if ever) it will actually occur.
    test.step_timeout(() => { reject(); }, 5000);

    return deferred;
  })().catch(resolveGarbageCollection);
}, 'When called from HostCleanupFinalizationRegistry, if calling the callback throws an error, this will be caught within the RunJobs algorithm and reported to the host. HTML does not apply the RunJobs algorithm, but will also report the error, which may call window.onerror.');
