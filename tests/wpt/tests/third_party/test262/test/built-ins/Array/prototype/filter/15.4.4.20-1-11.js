// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: Array.prototype.filter applied to Date object
---*/

function callbackfn(val, idx, obj) {
  return obj instanceof Date;
}

var obj = new Date(0);
obj.length = 1;
obj[0] = 1;

var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr[0], 1, 'newArr[0]');
