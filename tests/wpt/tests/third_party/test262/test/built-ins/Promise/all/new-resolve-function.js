// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.4.1.1
description: >
  Each Promise.all element is called with a new Promise.all Resolve Element function.
info: |
  Runtime Semantics: PerformPromiseAll( iteratorRecord, constructor, resultCapability)

  ...
  k. Let resolveElement be a new built-in function object as defined in Promise.all Resolve Element Functions.
  ...
  r. Let result be Invoke(nextPromise, "then", «resolveElement, resultCapability.[[Reject]]»).
  ...
---*/

function resolveFunction() {}

function Constructor(executor) {
  executor(resolveFunction, Test262Error.thrower);
}
Constructor.resolve = function(v) {
  return v;
};

var callCount1 = 0,
  callCount2 = 0;
var p1OnFulfilled;

var p1 = {
  then: function(onFulfilled, onRejected) {
    callCount1 += 1;
    p1OnFulfilled = onFulfilled;
    assert.notSameValue(onFulfilled, resolveFunction, "p1.then");
  }
};
var p2 = {
  then: function(onFulfilled, onRejected) {
    callCount2 += 1;
    assert.notSameValue(onFulfilled, resolveFunction, "p2.then");
    assert.notSameValue(onFulfilled, p1OnFulfilled, "p1.onFulfilled != p2.onFulfilled");
  }
};

Promise.all.call(Constructor, [p1, p2]);

assert.sameValue(callCount1, 1, "p1.then call count");
assert.sameValue(callCount2, 1, "p2.then call count");
