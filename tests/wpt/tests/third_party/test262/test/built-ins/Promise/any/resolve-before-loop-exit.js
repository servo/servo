// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any
description: >
  Cannot tamper remainingElementsCount when two Promise.any Reject Element Function are called in succession.
info: |
  Let result be PerformPromiseAny(iteratorRecord, C, promiseCapability).

  Runtime Semantics: PerformPromiseAny

  ...
  Let remainingElementsCount be a new Record { [[value]]: 1 }.
  ...
  8.d ...
    ii. Set remainingElementsCount.[[value]] to remainingElementsCount.[[value]] − 1.
    iii. If remainingElementsCount.[[value]] is 0,
      1. Let error be a newly created AggregateError object.
      2. Perform ! DefinePropertyOrThrow(error, "errors", Property Descriptor { [[Configurable]]: true, [[Enumerable]]: false, [[Writable]]: true, [[Value]]: errors }).
      3. Return ThrowCompletion(error).
  ...

  Promise.any Reject Element Functions
  ...
  Let alreadyCalled be the value of F's [[AlreadyCalled]] internal slot.
  If alreadyCalled.[[value]] is true, return undefined.
  Set alreadyCalled.[[value]] to true.
  ...

features: [Promise.any, arrow-function]
---*/

let callCount = 0;
let errorArray;

function Constructor(executor) {
  function reject(error) {
    callCount += 1;
    errorArray = error.errors;

    assert(Array.isArray(error.errors), "error is array");
    assert.sameValue(error.errors.length, 3, "error.length");
    assert.sameValue(error.errors[0], "p1-rejection", "error.errors[0] === 'p1-rejection'");
    assert.sameValue(error.errors[1], "p2-rejection", "error.errors[1] === 'p2-rejection'");
    assert.sameValue(error.errors[2], "p3-rejection", "error.errors[2] === 'p3-rejection'");
    assert(error instanceof AggregateError, "error instanceof AggregateError");
  }
  executor(Test262Error.thrower, reject);
}
Constructor.resolve = function(v) {
  return v;
};

let p1OnRejected;

let p1 = {
  then(onResolved, onRejected) {
    p1OnRejected = onRejected;
  }
};
let p2 = {
  then(onResolved, onRejected) {
    p1OnRejected("p1-rejection");
    onRejected("p2-rejection");
  }
};
let p3 = {
  then(onResolved, onRejected) {
    onRejected("p3-rejection");
  }
};

assert.sameValue(callCount, 0, "callCount before call to any()");

Promise.any.call(Constructor, [p1, p2, p3]);

assert.sameValue(callCount, 1, "callCount after call to any()");
assert.sameValue(errorArray[0], "p1-rejection", "errorArray[0] === 'p1-rejection'");
assert.sameValue(errorArray[1], "p2-rejection", "errorArray[1] === 'p2-rejection'");
assert.sameValue(errorArray[2], "p3-rejection", "errorArray[2] === 'p3-rejection'");

p1OnRejected("unexpectedonRejectedValue");

assert.sameValue(callCount, 1, "callCount after call to p1OnRejected()");
assert.sameValue(
  errorArray[0],
  "p1-rejection",
  "errorArray[0] === 'p1-rejection', after call to p1OnRejected(...)"
);
assert.sameValue(
  errorArray[1],
  "p2-rejection",
  "errorArray[1] === 'p2-rejection', after call to p1OnRejected(...)"
);
assert.sameValue(
  errorArray[2],
  "p3-rejection",
  "errorArray[2] === 'p3-rejection', after call to p1OnRejected(...)"
);
