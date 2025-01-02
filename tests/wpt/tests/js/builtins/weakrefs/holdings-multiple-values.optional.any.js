// META: script=/common/gc.js
// META: script=resources/maybe-garbage-collect.js
// ├──> maybeGarbageCollectAndCleanupAsync
// └──> resolveGarbageCollection
/*---
esid: sec-properties-of-the-finalization-registry-constructor
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

---*/

function check(value, expectedName) {
  let holdings = [];
  let called = 0;
  let finalizationRegistry = new FinalizationRegistry(function() {});

  function callback(holding) {
    called += 1;
    holdings.push(holding);
  }

  // This is internal to avoid conflicts
  function emptyCells(value) {
    let target = {};
    finalizationRegistry.register(target, value);

    let prom = maybeGarbageCollectAndCleanupAsync(target);
    target = null;

    return prom;
  }

  return emptyCells(value).then(function() {
    finalizationRegistry.cleanupSome(callback);
    assert_equals(called, 1, expectedName);
    assert_equals(holdings.length, 1, expectedName);
    assert_equals(holdings[0], value, expectedName);
  });
}

test(() =>
  assert_implements(
  typeof FinalizationRegistry.prototype.cleanupSome === 'function',
  'FinalizationRegistry.prototype.cleanupSome is not implemented.'
), 'Requires FinalizationRegistry.prototype.cleanupSome');
promise_test(() => check(undefined, 'undefined'), '`undefined` as registered holding value');
promise_test(() => check(null, 'null'), '`null` as registered holding value');
promise_test(() => check('', 'the empty string'), '`""` as registered holding value');
promise_test(() => check({}, 'object'), '`{}` as registered holding value');
promise_test(() => check(42, 'number'), '`42` as registered holding value');
promise_test(() => check(true, 'true'), '`true` as registered holding value');
promise_test(() => check(false, 'false'), '`false` as registered holding value');
promise_test(() => check(Symbol(1), 'symbol'), '`Symbol(1)` as registered holding value');
