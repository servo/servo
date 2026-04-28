// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.4.3.1
description: >
  Each Promise.race element is called with the same resolve function.
info: |
  Runtime Semantics: PerformPromiseRace ( iteratorRecord, promiseCapability, C )

  ...
  j. Let result be Invoke(nextPromise, "then", «promiseCapability.[[Resolve]], promiseCapability.[[Reject]]»).
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

var p1 = {
  then: function(onFulfilled, onRejected) {
    callCount1 += 1;
    assert.sameValue(onFulfilled, resolveFunction, "p1.then");
  }
};
var p2 = {
  then: function(onFulfilled, onRejected) {
    callCount2 += 1;
    assert.sameValue(onFulfilled, resolveFunction, "p2.then");
  }
};

Promise.race.call(Constructor, [p1, p2]);

assert.sameValue(callCount1, 1, "p1.then call count");
assert.sameValue(callCount2, 1, "p2.then call count");
