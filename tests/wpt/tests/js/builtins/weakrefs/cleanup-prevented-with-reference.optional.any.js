// META: script=/common/gc.js
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
---*/

let holdingsList = [];
function cb(holding) {
  holdingsList.push(holding);
};
let finalizationRegistry = new FinalizationRegistry(function() {});

let referenced = {};

function emptyCells() {
  let target = {};
  finalizationRegistry.register(target, 'target!');
  finalizationRegistry.register(referenced, 'referenced');

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
    finalizationRegistry.cleanupSome(cb);

    assert_equals(holdingsList.length, 1);
    assert_equals(holdingsList[0], 'target!');
    assert_equals(typeof referenced, 'object', 'referenced preserved');
  })().catch(resolveGarbageCollection);
}, 'cleanupCallback has only one optional chance to be called for a GC that cleans up a registered target.');
