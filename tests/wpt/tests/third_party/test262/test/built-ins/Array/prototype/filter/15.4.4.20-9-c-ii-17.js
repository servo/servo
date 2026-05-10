// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter -'this' of 'callbackfn' is a Number object
    when T is not an object (T is a number)
---*/

function callbackfn(val, idx, o) {
  return 5 === this.valueOf();
}

var obj = {
  0: 11,
  length: 2
};
var newArr = Array.prototype.filter.call(obj, callbackfn, 5);

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.sameValue(newArr[0], 11, 'newArr[0]');
