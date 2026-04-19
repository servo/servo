// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%asyncfromsynciteratorprototype%.next
description: next() will unwrap a Promise value return by the sync iterator
info: |
  %AsyncFromSyncIteratorPrototype%.next ( value )
  ...
  2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
  ...
  5. Let nextResult be IteratorNext(syncIteratorRecord, value).
  6. IfAbruptRejectPromise(nextResult, promiseCapability).
  7. Let nextDone be IteratorComplete(nextResult).
  8. If AbruptRejectPromise(nextDone, promiseCapability).
  9. Let nextValue be IteratorValue(nextResult).
  10. IfAbruptRejectPromise(nextValue, promiseCapability).
  ...
  14. Let steps be the algorithm steps defined in Async-from-Sync Iterator Value Unwrap Functions.

  Async-from-Sync Iterator Value Unwrap Functions
  An async-from-sync iterator value unwrap function is an anonymous built-in
  function that is used by methods of %AsyncFromSyncIteratorPrototype% when
  processing the value field of an IteratorResult object, in order to wait for
  its value if it is a promise and re-package the result in a new "unwrapped"
  IteratorResult object. Each async iterator value unwrap function has a
  [[Done]] internal slot.

flags: [async]
features: [async-iteration]
---*/

function* g() {
  yield Promise.resolve(1);
}

async function* asyncg() {
  yield* g();
}

asyncg().next().then(function (result) {
  assert.sameValue(result.value, 1, "result.value should be unwrapped promise, got: " + result.value)
}).then($DONE, $DONE);
