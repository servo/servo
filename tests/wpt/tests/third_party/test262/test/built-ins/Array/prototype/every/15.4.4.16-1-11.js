// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every applied to Date object
---*/

function callbackfn(val, idx, obj) {
  return !(obj instanceof Date);
}

var obj = new Date(0);
obj.length = 1;
obj[0] = 1;

assert.sameValue(Array.prototype.every.call(obj, callbackfn), false, 'Array.prototype.every.call(obj, callbackfn)');
