// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter - 'this' of 'callbackfn' is an String
    object when T is not an object (T is a string)
---*/

function callbackfn(val, idx, obj) {
  return 'hello' === this.valueOf();
}

var obj = {
  0: 11,
  length: 2
};
var newArr = Array.prototype.filter.call(obj, callbackfn, "hello");

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.sameValue(newArr[0], 11, 'newArr[0]');
