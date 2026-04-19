// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%asyncfromsynciteratorprototype%.throw
description: return() will return value undefined if sync `return` is undefined
info: |
  %AsyncFromSyncIteratorPrototype%.return ( value )
  ...
  2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
  ...
  5. Let return be GetMethod(syncIterator, "return").
  6. IfAbruptRejectPromise(return, promiseCapability).
  7. If return is undefined, then
    a. Let iterResult be ! CreateIterResultObject(value, true).
    b. Perform ! Call(promiseCapability.[[Resolve]], undefined, « iterResult »).
    c. Return promiseCapability.[[Promise]].

flags: [async]
features: [async-iteration]
---*/


var obj = {
  [Symbol.iterator]() {
    return {
      next() {
        return { value: 1, done: false };
      }
    };
  }
};

async function* asyncg() {
  yield* obj;
}

var iter = asyncg();

iter.next().then(function(result) {

  iter.return().then(function(result) {

    assert.sameValue(result.done, true, 'the iterator is completed');
    assert.sameValue(result.value, undefined, 'expect value to be undefined');

    iter.next().then(({ done, value }) => {
      assert.sameValue(done, true, 'the iterator is completed');
      assert.sameValue(value, undefined, 'value is undefined');
    }).then($DONE, $DONE);

  }).catch($DONE);

}).catch($DONE);
