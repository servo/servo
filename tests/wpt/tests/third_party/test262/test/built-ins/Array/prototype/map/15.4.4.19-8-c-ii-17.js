// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - 'this' object when T is not an object (T is
    a number)
---*/

function callbackfn(val, idx, obj) {
  return this.valueOf() === 5;
}

var obj = {
  0: 11,
  length: 2
};

var testResult = Array.prototype.map.call(obj, callbackfn, 5);

assert.sameValue(testResult[0], true, 'testResult[0]');
