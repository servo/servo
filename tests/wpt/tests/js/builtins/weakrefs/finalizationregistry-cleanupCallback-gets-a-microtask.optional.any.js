// META: script=/common/gc.js
// META: script=resources/maybe-garbage-collect.js
// ├──> maybeGarbageCollectAsync
// └──> resolveGarbageCollection
/*---
esid: sec-finalization-registry-target
info: |
  FinalizationRegistry ( cleanupCallback )

  Execution
  At any time, if a set of objects S is not live, an ECMAScript implementation may perform the
  following steps atomically:

  For each obj of S, do
    For each WeakRef ref such that ref.[[WeakRefTarget]] is obj, do
      Set ref.[[WeakRefTarget]] to empty.
    For each FinalizationRegistry fg such that fg.[[Cells]] contains cell, such that
    cell.[[WeakRefTarget]] is obj,
      Set cell.[[WeakRefTarget]] to empty.
      Optionally, perform ! HostCleanupFinalizationRegistry(fg).

  HostCleanupFinalizationRegistry(finalizationRegistry)

  HostCleanupFinalizationRegistry is an implementation-defined abstract operation that is expected
  to call CleanupFinalizationRegistry(finalizationRegistry) at some point in the future, if
  possible. The host's responsibility is to make this call at a time which does not interrupt
  synchronous ECMAScript code execution.
---*/

let count = 1_000;
let calls = 0;
let registries = [];
let callback = function() {
  calls++;
};
for (let i = 0; i < count; i++) {
  registries.push(
    new FinalizationRegistry(callback)
  );
}
setup({ allow_uncaught_exception: true });

promise_test((test) => {
  assert_implements(
    typeof FinalizationRegistry.prototype.register === 'function',
    'FinalizationRegistry.prototype.register is not implemented.'
  );
  return (async () => {

    {
      let target = {};
      for (let registry of registries) {
        registry.register(target, 1);
      }
      target = null;
    }

    await maybeGarbageCollectAsync();
    await test.step_wait(() => calls === count, `Expected ${count} registry cleanups.`);
  })().catch(resolveGarbageCollection);
}, 'HostCleanupFinalizationRegistry is an implementation-defined abstract operation that is expected to call CleanupFinalizationRegistry(finalizationRegistry) at some point in the future, if possible.');
