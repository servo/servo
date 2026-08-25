// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: Array.prototype.filter applied to Boolean Object
---*/

function callbackfn(val, idx, obj) {
  return obj instanceof Boolean;
}

var obj = new Boolean(true);
obj.length = 2;
obj[0] = 11;
obj[1] = 12;

var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr[0], 11, 'newArr[0]');
assert.sameValue(newArr[1], 12, 'newArr[1]');
