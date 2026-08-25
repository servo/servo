// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - 'length' is a string containing a positive
    number
---*/

function callbackfn(val, idx, obj) {
  return val > 10;
}

var obj = {
  0: 10,
  1: 12,
  2: 9,
  length: 2
};

var newArr = Array.prototype.map.call(obj, callbackfn);

assert.sameValue(newArr.length, 2, 'newArr.length');
