// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%asyncfromsynciteratorprototype%.return
description: return() will reject promise if getter `done` abrupt completes
info: |
  %AsyncFromSyncIteratorPrototype%.return ( value )
  ...
  2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
  ...
  5. Let return be GetMethod(syncIterator, "return").
  ...
  8. Let returnResult be Call(return, syncIterator, « value »).
  9. IfAbruptRejectPromise(returnResult, promiseCapability).
  ...
  11. Let returnDone be IteratorComplete(returnResult).
  12. IfAbruptRejectPromise(returnDone, promiseCapability).
  13. Let returnValue be IteratorValue(returnResult).
  14. IfAbruptRejectPromise(returnValue, promiseCapability).
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
      return() {
        return {
          get done() {
            throw thrownError;
          },
          value: 1
        }
      }
    };
  }
};

async function* asyncg() {
  yield* obj;
}

var iter = asyncg();

iter.next().then(function(result) {

  iter.return().then(
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
