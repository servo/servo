// Copyright (C) 2019 Leo Balter, 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseany
description: >
  Each Promise.any element is called with a new Promise.any Reject Element function.
info: |
  Runtime Semantics: PerformPromiseAny ( iteratorRecord, constructor, resultCapability )

  ...
  k. Let rejectElement be ! CreateBuiltinFunction(steps, « [[AlreadyCalled]], [[Index]], [[Errors]], [[Capability]], [[RemainingElements]] »).
  ...
  r. Perform ? Invoke(nextPromise, "then", « resultCapability.[[Resolve]], rejectElement »).
  ...

features: [Promise.any]
---*/

function rejectFunction() {}

function Constructor(executor) {
  executor(rejectFunction, Test262Error.thrower);
}
Constructor.resolve = function(v) {
  return v;
};

var callCount1 = 0;
var callCount2 = 0;
var p1OnRejected;

var p1 = {
  then(_, onRejected) {
    callCount1 += 1;
    p1OnRejected = onRejected;
    assert.notSameValue(onRejected, rejectFunction, 'p1.then');
  }
};
var p2 = {
  then(_, onRejected) {
    callCount2 += 1;
    assert.notSameValue(onRejected, rejectFunction, 'p2.then');
    assert.notSameValue(onRejected, p1OnRejected, 'p1.onRejected != p2.onRejected');
  }
};

Promise.any.call(Constructor, [p1, p2]);
assert.sameValue(callCount1, 1, 'p1.then call count');
assert.sameValue(callCount2, 1, 'p2.then call count');

