// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: Array.prototype.filter applied to Number object
---*/

function callbackfn(val, idx, obj) {
  return obj instanceof Number;
}

var obj = new Number(-128);
obj.length = 2;
obj[0] = 11;
obj[1] = 12;

var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr[0], 11, 'newArr[0]');
assert.sameValue(newArr[1], 12, 'newArr[1]');
