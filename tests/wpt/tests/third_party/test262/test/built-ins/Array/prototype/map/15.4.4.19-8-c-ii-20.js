// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - callbackfn called with correct parameters
    (thisArg is correct)
---*/

function callbackfn(val, idx, obj) {
  return this.threshold === 10;
}

var thisArg = {
  threshold: 10
};

var obj = {
  0: 11,
  1: 9,
  length: 2
};

var testResult = Array.prototype.map.call(obj, callbackfn, thisArg);

assert.sameValue(testResult[0], true, 'testResult[0]');
