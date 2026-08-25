// META: script=/common/gc.js
// META: script=resources/maybe-garbage-collect.js
// ├──> maybeGarbageCollectAndCleanupAsync
// └──> resolveGarbageCollection
/*---
esid: sec-properties-of-the-finalization-registry-constructor
---*/

let called = 0;
let endOfCall = 0;
let finalizationRegistry = new FinalizationRegistry(function() {});

function callback(holding) {
  called += 1;

  if (called === 1) {
    // Atempt to re-enter the callback.
    let nestedCallbackRan = false;
    finalizationRegistry.cleanupSome(() => { nestedCallbackRan = true });
    assert_equals(nestedCallbackRan, true);
  }

  endOfCall += 1;
}

function emptyCells() {
  let o1 = {};
  let o2 = {};
  // Register more than one objects to test reentrancy.
  finalizationRegistry.register(o1, 'holdings 1');
  finalizationRegistry.register(o2, 'holdings 2');

  let prom = maybeGarbageCollectAndCleanupAsync(o1);
  o1 = null;

  return prom;
}

promise_test(() => {
  return (async () => {
    assert_implements(
      typeof FinalizationRegistry.prototype.cleanupSome === 'function',
      'FinalizationRegistry.prototype.cleanupSome is not implemented.'
    );
    await emptyCells();
    finalizationRegistry.cleanupSome(callback);

    assert_equals(called, 1, 'callback was called');
    assert_equals(endOfCall, 1, 'callback finished');
  })().catch(resolveGarbageCollection);
}, 'cleanupCallback has only one optional chance to be called for a GC that cleans up a registered target.');
