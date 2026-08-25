// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettled-resolve-element-functions
description: >
  Cannot change result value of resolved Promise.allSettled element after Promise.allSettled() returned.
info: |
  Promise.allSettled Resolve Element Functions

  1. Let F be the active function object.
  2. Let alreadyCalled be F.[[AlreadyCalled]].
  3. If alreadyCalled.[[Value]] is true, return undefined.
  4. Set alreadyCalled.[[Value]] to true.
  ...
includes: [promiseHelper.js]
features: [Promise.allSettled]
---*/

var callCount = 0;
var valuesArray;
var expected = [{ status: 'fulfilled', value: 'expectedValue' }];

function Constructor(executor) {
  function resolve(values) {
    callCount += 1;
    valuesArray = values;
    checkSettledPromises(values, expected, 'values');
  }
  executor(resolve, Test262Error.thrower);
}
Constructor.resolve = function(v) {
  return v;
};

var p1OnFulfilled;

var p1 = {
  then(onFulfilled, onRejected) {
    p1OnFulfilled = onFulfilled;
    onFulfilled('expectedValue');
  }
};

assert.sameValue(callCount, 0, 'callCount before call to all()');

Promise.allSettled.call(Constructor, [p1]);

assert.sameValue(callCount, 1, 'callCount after call to all()');
checkSettledPromises(valuesArray, expected, 'valuesArray after call to all()');

p1OnFulfilled('unexpectedValue');

assert.sameValue(callCount, 1, 'callCount after call to onFulfilled()');
checkSettledPromises(valuesArray, expected, 'valuesArray after call to onFulfilled()');
