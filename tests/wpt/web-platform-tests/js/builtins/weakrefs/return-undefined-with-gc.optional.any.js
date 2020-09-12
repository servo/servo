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

let called;
let fn = function() {
  called += 1;
  return 39;
};
let cb = function() {
  called += 1;
  return 42;
};
let finalizationRegistry = new FinalizationRegistry(fn);

function emptyCells() {
  let target = {};
  finalizationRegistry.register(target);

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
    called = 0;
    assert_equals(finalizationRegistry.cleanupSome(cb), undefined, 'regular callback');
    assert_equals(called, 1);

    await emptyCells();
    called = 0;
    assert_equals(finalizationRegistry.cleanupSome(fn), undefined, 'regular callback, same FG cleanup function');
    assert_equals(called, 1);

    await emptyCells();
    called = 0;
    assert_equals(finalizationRegistry.cleanupSome(), undefined, 'undefined (implicit) callback, defer to FB callback');
    assert_equals(called, 1);

    await emptyCells();
    called = 0;
    assert_equals(finalizationRegistry.cleanupSome(undefined), undefined, 'undefined (explicit) callback, defer to FB callback');
    assert_equals(called, 1);

    await emptyCells();
    assert_equals(finalizationRegistry.cleanupSome(() => 1), undefined, 'arrow function');

    await emptyCells();
    assert_equals(finalizationRegistry.cleanupSome(async function() {}), undefined, 'async function');

    await emptyCells();
    assert_equals(finalizationRegistry.cleanupSome(function *() {}), undefined, 'generator');

    await emptyCells();
    assert_equals(finalizationRegistry.cleanupSome(async function *() {}), undefined, 'async generator');

  })().catch(resolveGarbageCollection);
}, 'Return undefined regardless the result of CleanupFinalizationRegistry');
