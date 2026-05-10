// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter - arguments to callbackfn are self
    consistent
---*/

var obj = {
  0: 11,
  length: 1
};
var thisArg = {};

function callbackfn() {
  return this === thisArg &&
    arguments[0] === 11 &&
    arguments[1] === 0 &&
    arguments[2] === obj;
}

var newArr = Array.prototype.filter.call(obj, callbackfn, thisArg);

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.sameValue(newArr[0], 11, 'newArr[0]');
