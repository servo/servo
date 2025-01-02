// META: script=/common/gc.js
// META: script=resources/maybe-garbage-collect.js
// ├──> maybeGarbageCollectAndCleanupAsync
// └──> resolveGarbageCollection
/*---
esid: sec-finalization-registry.prototype.unregister
info: |
  FinalizationRegistry.prototype.cleanupSome ( [ callback ] )

  1. Let finalizationRegistry be the this value.
  ...
  5. Perform ! CleanupFinalizationRegistry(finalizationRegistry, callback).
  ...

  CleanupFinalizationRegistry ( finalizationRegistry [ , callback ] )

  ...
  3. While finalizationRegistry.[[Cells]] contains a Record cell such that cell.[[WeakRefTarget]] is ~empty~, then an implementation may perform the following steps,
    a. Choose any such cell.
    b. Remove cell from finalizationRegistry.[[Cells]].
    c. Perform ? Call(callback, undefined, << cell.[[HeldValue]] >>).
  ...

  FinalizationRegistry.prototype.unregister ( unregisterToken )

  1. Let removed be false.
  2. For each Record { [[Target]], [[Holdings]], [[UnregisterToken]] } cell that is an element of finalizationRegistry.[[Cells]], do
    a. If SameValue(cell.[[UnregisterToken]], unregisterToken) is true, then
      i. Remove cell from finalizationRegistry.[[Cells]].
      ii. Set removed to true.
  3. Return removed.

---*/

let value = 'target!';
let token = {};
let finalizationRegistry = new FinalizationRegistry(function() {});

function emptyCells() {
  let target = {};
  finalizationRegistry.register(target, value, token);

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
    let holdings = [];
    finalizationRegistry.cleanupSome((holding) => {
      called += 1;
      holdings.push(holding);
    });

    assert_equals(called, 1);
    assert_equals(holdings.length, 1);
    assert_equals(holdings[0], value);

    let res = finalizationRegistry.unregister(token);
    assert_equals(res, false, 'unregister after iterating over it in cleanup');

  })().catch(resolveGarbageCollection);
}, 'Cannot unregister a cell that has been cleaned up');

