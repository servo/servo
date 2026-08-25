// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - element to be retrieved is own accessor
    property without a get function that overrides an inherited
    accessor property on an Array-like object
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (prevVal === undefined);
  }
}

Object.prototype[0] = 0;

var obj = {
  1: 1,
  2: 2,
  length: 3
};

Object.defineProperty(obj, "0", {
  set: function() {},
  configurable: true
});

Array.prototype.reduce.call(obj, callbackfn);

assert(testResult, 'testResult !== true');
