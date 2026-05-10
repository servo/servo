// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.4.1.2
description: >
  Cannot change result value of resolved Promise.all elements.
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
    assert.sameValue(values.length, 2, "values length");
    assert.sameValue(values[0], "expectedValue-p1", "values[0]");
    assert.sameValue(values[1], "expectedValue-p2", "values[1]");
  }
  executor(resolve, Test262Error.thrower);
}
Constructor.resolve = function(v) {
  return v;
};

var p1 = {
  then: function(onFulfilled, onRejected) {
    onFulfilled("expectedValue-p1");
    onFulfilled("unexpectedValue-p1");
  }
};
var p2 = {
  then: function(onFulfilled, onRejected) {
    onFulfilled("expectedValue-p2");
    onFulfilled("unexpectedValue-p2");
  }
};

assert.sameValue(callCount, 0, "callCount before call to all()");

Promise.all.call(Constructor, [p1, p2]);

assert.sameValue(callCount, 1, "callCount after call to all()");
