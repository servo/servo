// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseany
description: >
  Promise.any does not prevent resolve from being called multiple times.
features: [Promise.any, arrow-function]
includes: [promiseHelper.js]
---*/
let callCount = 0;
let sequence = [];

function Constructor(executor) {
  function resolve(value) {
    callCount += 1;
    sequence.push(value);
  }
  executor(resolve, Test262Error.thrower);
}
Constructor.resolve = function(v) {
  return v;
};

let pResolve;
let a = {
  then(resolver, rejector) {
    pResolve = resolver;
  }
};

assert.sameValue(callCount, 0, 'callCount before call to any()');

Promise.any.call(Constructor, [a]);

assert.sameValue(callCount, 0, 'callCount after call to any()');

pResolve(1);
pResolve(2);
pResolve(3);

assert.sameValue(callCount, 3, 'callCount after resolving a');
assert.sameValue(sequence.length, 3);
checkSequence(sequence);
