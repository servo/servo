// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - value of 'length' is a number (value is
    +0)
---*/

var accessed = false;

function callbackfn(prevVal, curVal, idx, obj) {
  accessed = true;
  return 2;
}

var obj = {
  0: 11,
  length: +0
};

assert.sameValue(Array.prototype.reduce.call(obj, callbackfn, 1), 1, 'Array.prototype.reduce.call(obj, callbackfn, 1)');
assert.sameValue(accessed, false, 'accessed');
