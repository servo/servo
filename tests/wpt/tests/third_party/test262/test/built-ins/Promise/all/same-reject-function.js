// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.4.1.1
description: >
  Each Promise.all element is called with the same reject function.
info: |
  Runtime Semantics: PerformPromiseAll( iteratorRecord, constructor, resultCapability)

  ...
  r. Let result be Invoke(nextPromise, "then", «resolveElement, resultCapability.[[Reject]]»).
  ...
---*/

function rejectFunction() {}

function Constructor(executor) {
  executor(Test262Error.thrower, rejectFunction);
}
Constructor.resolve = function(v) {
  return v;
};

var callCount1 = 0,
  callCount2 = 0;

var p1 = {
  then: function(onFulfilled, onRejected) {
    callCount1 += 1;
    assert.sameValue(onRejected, rejectFunction, "p1.then");
  }
};
var p2 = {
  then: function(onFulfilled, onRejected) {
    callCount2 += 1;
    assert.sameValue(onRejected, rejectFunction, "p2.then");
  }
};

Promise.all.call(Constructor, [p1, p2]);

assert.sameValue(callCount1, 1, "p1.then call count");
assert.sameValue(callCount2, 1, "p2.then call count");
