// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: Array.prototype.reduce - loop is broken once 'kPresent' is true
---*/

var called = 0;
var testResult = false;
var firstCalled = 0;
var secondCalled = 0;

function callbackfn(prevVal, val, idx, obj) {
  if (called === 0) {
    testResult = (idx === 1);
  }
  called++;
}

var arr = [, , ];

Object.defineProperty(arr, "0", {
  get: function() {
    firstCalled++;
    return 11;
  },
  configurable: true
});

Object.defineProperty(arr, "1", {
  get: function() {
    secondCalled++;
    return 9;
  },
  configurable: true
});

arr.reduce(callbackfn);

assert(testResult, 'testResult !== true');
assert.sameValue(firstCalled, 1, 'firstCalled');
assert.sameValue(secondCalled, 1, 'secondCalled');
