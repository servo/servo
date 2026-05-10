// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettled-resolve-element-functions
description: >
  Cannot change result value of resolved Promise.allSettled elements.
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

function Constructor(executor) {
  function resolve(values) {
    callCount += 1;
    checkSettledPromises(values, [
      {
        status: 'fulfilled',
        value: 'expectedValue-p1'
      },
      {
        status: 'fulfilled',
        value: 'expectedValue-p2'
      }
    ], 'values');
  }
  executor(resolve, Test262Error.thrower);
}
Constructor.resolve = function(v) {
  return v;
};

var p1 = {
  then(onFulfilled, onRejected) {
    onFulfilled('expectedValue-p1');
    onFulfilled('unexpectedValue-p1');
  }
};
var p2 = {
  then(onFulfilled, onRejected) {
    onFulfilled('expectedValue-p2');
    onFulfilled('unexpectedValue-p2');
  }
};

assert.sameValue(callCount, 0, 'callCount before call to all()');

Promise.allSettled.call(Constructor, [p1, p2]);

assert.sameValue(callCount, 1, 'callCount after call to all()');
