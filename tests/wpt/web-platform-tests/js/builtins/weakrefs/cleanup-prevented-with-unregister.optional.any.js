// META: script=resources/maybe-garbage-collect.js
// ├──> maybeGarbageCollectAndCleanupAsync
// └──> resolveGarbageCollection
/*---
esid: sec-finalization-registry.prototype.cleanupSome
info: |
  FinalizationRegistry.prototype.cleanupSome ( [ callback ] )

  1. Let finalizationRegistry be the this value.
  2. If Type(finalizationRegistry) is not Object, throw a TypeError exception.
  3. If finalizationRegistry does not have a [[Cells]] internal slot, throw a TypeError exception.
  4. If callback is not undefined and IsCallable(callback) is false, throw a TypeError exception.
  5. Perform ? CleanupFinalizationRegistry(finalizationRegistry, callback).
  6. Return undefined.

  FinalizationRegistry.prototype.unregister ( unregisterToken )

  1. Let removed be false.
  2. For each Record { [[Target]], [[Holdings]], [[UnregisterToken]] } cell that is an element of finalizationRegistry.[[Cells]], do
    a. If SameValue(cell.[[UnregisterToken]], unregisterToken) is true, then
      i. Remove cell from finalizationRegistry.[[Cells]].
      ii. Set removed to true.
  3. Return removed.
---*/
let token = {};
let finalizationRegistry = new FinalizationRegistry(function() {});

function emptyCells() {
  let target = {};
  finalizationRegistry.register(target, 'target!', token);

  let prom = maybeGarbageCollectAndCleanupAsync(target);
  target = null;

  return prom;
}

promise_test(() => {
  return (async () => {
    assert_implements(
      typeof FinalizationRegistry.prototype.cleanupSome === 'function',
      'FinalizationRegistry.prototype.cleanupSome is not implemented.'
    );

    await emptyCells();
    let called = 0;

    let res = finalizationRegistry.unregister(token);
    assert_equals(res, true, 'unregister target before iterating over it in cleanup');

    finalizationRegistry.cleanupSome((holding) => {
      called += 1;
    });

    assert_equals(called, 0, 'callback was not called');
  })().catch(resolveGarbageCollection);
}, 'Cleanup might be prevented with an unregister usage');
