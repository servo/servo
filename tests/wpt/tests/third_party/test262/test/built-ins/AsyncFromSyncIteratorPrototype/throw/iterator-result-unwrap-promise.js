// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%asyncfromsynciteratorprototype%.throw
description: throw() will unwrap a Promise value return by the sync iterator
info: |
  %AsyncFromSyncIteratorPrototype%.throw ( value )
  ...
  2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
  ...
  5. Let throw be GetMethod(syncIterator, "throw").
  ...
  17. Let steps be the algorithm steps defined in Async-from-Sync Iterator Value Unwrap Functions.
  ...
  20. Return promiseCapability.[[Promise]].

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

var thrownError = new Error("Don't catch me.")

var obj = {
  [Symbol.iterator]() {
    return {
      next() {
        return { value: 1, done: false };
      },
      throw() {
        return {
          value: Promise.resolve(42),
          done: true
        };
      }
    };
  }
};

async function* asyncg() {
  return yield* obj;
}

let iter = asyncg();

iter.next().then(function (result) {
  iter.throw(thrownError).then(
    function (result) {
      assert.sameValue(result.value, 42, "Result.value should be unwrapped, got: " + result.value);

      iter.next().then(({ done, value }) => {
        assert.sameValue(done, true, 'the iterator is completed');
        assert.sameValue(value, undefined, 'value is undefined');
      }).then($DONE, $DONE);
    }
  ).catch($DONE);
}).catch($DONE);
