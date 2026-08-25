// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - value of 'length' is string that is able to
    convert to number primitive (value is a decimal number)
---*/

function callbackfn(val, idx, obj) {
  return val < 10;
}

var obj = {
  0: 11,
  1: 9,
  2: 12,
  length: "2.5"
};

var newArr = Array.prototype.map.call(obj, callbackfn);

assert.sameValue(newArr.length, 2, 'newArr.length');
