// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%asyncfromsynciteratorprototype%.throw
description: throw() will return rejected promise if getter of `throw` abrupt completes
info: |
  %AsyncFromSyncIteratorPrototype%.throw ( value )
  ...
  2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
  ...
  5. Let throw be GetMethod(syncIterator, "throw").
  6. IfAbruptRejectPromise(throw, promiseCapability).
  ...
  8. Let throwResult be Call(throw, syncIterator, « value »).
  9. IfAbruptRejectPromise(throwResult, promiseCapability).
  ...
  22. Return promiseCapability.[[Promise]].

flags: [async]
features: [async-iteration]
---*/

var thrownError = new Error("Catch me.");

var obj = {
  [Symbol.iterator]() {
    return {
      next() {
        return { value: 1, done: false };
      },
      throw() {
        throw thrownError;
      }
    };
  }
};

async function* asyncg() {
  yield* obj;
}

var iter = asyncg();

iter.next().then(function(result) {

  iter.throw().then(
    function (result) {
      throw new Test262Error("Promise should be rejected, got: " + result.value);
    },
    function (err) {
      assert.sameValue(err, thrownError, "Promise should be rejected with thrown error");

      iter.next().then(({ done, value }) => {
        assert.sameValue(done, true, 'the iterator is completed');
        assert.sameValue(value, undefined, 'value is undefined');
      }).then($DONE, $DONE);
    }
  ).catch($DONE);
}).catch($DONE);
