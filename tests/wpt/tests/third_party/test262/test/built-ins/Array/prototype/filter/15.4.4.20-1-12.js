// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: Array.prototype.filter applied to RegExp object
---*/

function callbackfn(val, idx, obj) {
  return obj instanceof RegExp;
}

var obj = new RegExp();
obj.length = 2;
obj[1] = true;

var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr[0], true, 'newArr[0]');
