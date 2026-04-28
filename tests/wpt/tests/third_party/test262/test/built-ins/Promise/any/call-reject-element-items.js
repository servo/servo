// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any-reject-element-functions
description: >
  Cannot change result value of rejected Promise.any elements.
info: |
  Promise.any Reject Element Functions

  Let alreadyCalled be the value of F's [[AlreadyCalled]] internal slot.
  If alreadyCalled.[[value]] is true, return undefined.
  Set alreadyCalled.[[value]] to true.

features: [Promise.any]
---*/

let callCount = 0;

function Constructor(executor) {
  function reject(error) {
    callCount += 1;
    assert(Array.isArray(error.errors), "error.errors is array");
    assert.sameValue(error.errors.length, 2, "error.errors length");
    assert.sameValue(error.errors[0], "expectedValue-p1", "error.errors[0]");
    assert.sameValue(error.errors[1], "expectedValue-p2", "error.errors[1]");
  }
  executor(Test262Error.thrower, reject);
}
Constructor.resolve = function(v) {
  return v;
};

let p1 = {
  then(onFulfilled, onRejected) {
    onRejected("expectedValue-p1");
    onRejected("unexpectedValue-p1");
  }
};
let p2 = {
  then(onFulfilled, onRejected) {
    onRejected("expectedValue-p2");
    onRejected("unexpectedValue-p2");
  }
};

assert.sameValue(callCount, 0, "callCount before call to any()");

Promise.any.call(Constructor, [p1, p2]);

assert.sameValue(callCount, 1, "callCount after call to any()");
