// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.4.1.2
description: >
  Cannot change result value of resolved Promise.all element.
info: |
  Promise.all Resolve Element Functions

  1. Let alreadyCalled be the value of F's [[AlreadyCalled]] internal slot.
  2. If alreadyCalled.[[value]] is true, return undefined.
  3. Set alreadyCalled.[[value]] to true.
  ...
---*/

var callCount = 0;

function Constructor(executor) {
  function resolve(values) {
    callCount += 1;
    assert(Array.isArray(values), "values is array");
    assert.sameValue(values.length, 1, "values length");
    assert.sameValue(values[0], "expectedValue", "values[0]");
  }
  executor(resolve, Test262Error.thrower);
}
Constructor.resolve = function(v) {
  return v;
};

var p1 = {
  then: function(onFulfilled, onRejected) {
    onFulfilled("expectedValue");
    onFulfilled("unexpectedValue");
  }
};

assert.sameValue(callCount, 0, "callCount before call to all()");

Promise.all.call(Constructor, [p1]);

assert.sameValue(callCount, 1, "callCount after call to all()");
