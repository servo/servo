// META: script=resources/maybe-garbage-collect.js
// ├──> maybeGarbageCollectAndCleanupAsync
// ├──> maybeGarbageCollectAsync
// └──> resolveGarbageCollection
/*---
esid: sec-finalization-registry-target
info: |
  FinalizationRegistry ( cleanupCallback )

  FinalizationRegistry.prototype.cleanupSome ( [ callback ] )

  ...
  4. If callback is not undefined and IsCallable(callback) is false, throw a TypeError exception.
  5. Perform ? CleanupFinalizationRegistry(finalizationRegistry, callback).
  6. Return undefined.

  Execution

  At any time, if an object obj is not live, an ECMAScript implementation may perform the following steps atomically:

  1. For each WeakRef ref such that ref.[[Target]] is obj,
    a. Set ref.[[Target]] to empty.
  2. For each FinalizationRegistry finalizationRegistry such that finalizationRegistry.[[Cells]] contains cell, such that cell.[[Target]] is obj,
    a. Set cell.[[Target]] to empty.
    b. Optionally, perform ! HostCleanupFinalizationRegistry(finalizationRegistry).
---*/


let cleanupCallback = 0;
let holdings = [];
function cb(holding) {
  holdings.push(holding);
}

let finalizationRegistry = new FinalizationRegistry(function() {
  cleanupCallback += 1;
});

function emptyCells() {
  let target = {};
  finalizationRegistry.register(target, 'a');

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

    let ticks = 0;
    await emptyCells();
    await ticks++;

    finalizationRegistry.cleanupSome(cb);

    // cleanupSome will be invoked if there are empty cells left. If the
    // cleanupCallback already ran, then cb won't be called.
    let expectedCalled = cleanupCallback === 1 ? 0 : 1;
    // This asserts the registered object was emptied in the previous GC.
    assert_equals(holdings.length, expectedCalled, 'cleanupSome callback for the first time');

    // At this point, we can't assert if cleanupCallback was called, because it's
    // optional. Although, we can finally assert it's not gonna be called anymore
    // for the other executions of the Garbage Collector.
    // The chance of having it called only happens right after the
    // cell.[[Target]] is set to empty.
    assert_true(cleanupCallback >= 0, 'cleanupCallback might be 0');
    assert_true(cleanupCallback <= 1, 'cleanupCallback might be 1');

    // Restoring the cleanupCallback variable to 0 will help us asserting the
    // finalizationRegistry callback is not called again.
    cleanupCallback = 0;

    await maybeGarbageCollectAsync();
    await ticks++;

    finalizationRegistry.cleanupSome(cb);

    assert_equals(holdings.length, expectedCalled, 'cleanupSome callback is not called anymore, no empty cells');
    assert_equals(cleanupCallback, 0, 'cleanupCallback is not called again #1');

    await maybeGarbageCollectAsync();
    await ticks++;

    finalizationRegistry.cleanupSome(cb);

    assert_equals(holdings.length, expectedCalled, 'cleanupSome callback is not called again #2');
    assert_equals(cleanupCallback, 0, 'cleanupCallback is not called again #2');
    assert_equals(ticks, 3, 'ticks is 3');

    if (holdings.length) {
      assert_array_equals(holdings, ['a']);
    }

    await maybeGarbageCollectAsync();
  })().catch(resolveGarbageCollection);
}, 'cleanupCallback has only one optional chance to be called for a GC that cleans up a registered target.');
