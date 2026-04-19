// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallsettled
description: >
  Each Promise.allSettled element is called with a new Promise.allSettled Resolve Element function.
info: |
  Runtime Semantics: PerformPromiseAllSettled ( iteratorRecord, constructor, resultCapability )

  ...
  k Let resolveElement be ! CreateBuiltinFunction(steps, « [[AlreadyCalled]], [[Index]], [[Values]], [[Capability]], [[RemainingElements]] »).
  ...
  z. Perform ? Invoke(nextPromise, "then", « resolveElement, rejectElement »).
  ...
features: [Promise.allSettled]
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
  then(onFulfilled, onRejected) {
    callCount1 += 1;
    p1OnFulfilled = onFulfilled;
    assert.notSameValue(onFulfilled, resolveFunction, 'p1.then');
  }
};
var p2 = {
  then(onFulfilled, onRejected) {
    callCount2 += 1;
    assert.notSameValue(onFulfilled, resolveFunction, 'p2.then');
    assert.notSameValue(onFulfilled, p1OnFulfilled, 'p1.onFulfilled != p2.onFulfilled');
  }
};

Promise.allSettled.call(Constructor, [p1, p2]);

assert.sameValue(callCount1, 1, 'p1.then call count');
assert.sameValue(callCount2, 1, 'p2.then call count');
