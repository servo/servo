// Copyright (C) 2022 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncgenerator-prototype-return
description: >
  A broken promise should reject the returned promise of
  AsyncGenerator.prototype.return when the generator is completed.
info: |
  AsyncGenerator.prototype.return ( value )
  ...
  8. If state is either suspendedStart or completed, then
    a. Set generator.[[AsyncGeneratorState]] to awaiting-return.
    b. Perform ! AsyncGeneratorAwaitReturn(generator).
  ...

  AsyncGeneratorAwaitReturn ( generator )
  ...
  6. Let promise be Completion(PromiseResolve(%Promise%, completion.[[Value]])).
  7. If promiseCompletion is an abrupt completion, then
    a. Set generator.[[AsyncGeneratorState]] to completed.
    b. Perform AsyncGeneratorCompleteStep(generator, promiseCompletion, true).
    c. Perform AsyncGeneratorDrainQueue(generator).
    d. Return unused.
  8. Assert: promiseCompletion.[[Type]] is normal.
  9. Let promise be promiseCompletion.[[Value]].
  ...

flags: [async]
features: [async-iteration]
---*/

let unblock;
let blocking = new Promise(resolve => { unblock = resolve; });
let unblocked = false;
var g = async function*() {
  await blocking;
  unblocked = true;
};

var it = g();
let brokenPromise = Promise.resolve(42);
Object.defineProperty(brokenPromise, 'constructor', {
  get: function () {
    throw new Error('broken promise');
  }
});

it.next().then(function(result) {
  assert.sameValue(result.value, undefined);
  assert.sameValue(result.done, true);

  it.return(brokenPromise)
    .then(
      () => {
        throw new Test262Error("Expected rejection");
      },
      err => {
        assert(unblocked, 'return should be rejected when the generator is completed');
        assert.sameValue(err.message, 'broken promise');
      }
    )
    .then($DONE, $DONE);
});

unblock();
