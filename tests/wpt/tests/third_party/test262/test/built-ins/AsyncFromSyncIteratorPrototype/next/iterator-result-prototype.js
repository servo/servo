// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%asyncfromsynciteratorprototype%.next
description: next() returns a promise for an IteratorResult object
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
  1. Return ! CreateIterResultObject(value, F.[[Done]]).

flags: [async]
features: [async-iteration]
---*/

function* g() {}

async function* asyncg() {
  yield* g();
}

asyncg().next().then(function (result) {
  assert(
    Object.prototype.hasOwnProperty.call(result, 'value'), 'Has "own" property `value`'
  );
  assert(
    Object.prototype.hasOwnProperty.call(result, 'done'), 'Has "own" property `done`'
  );
  assert.sameValue(Object.getPrototypeOf(result), Object.prototype);
}).then($DONE, $DONE);
