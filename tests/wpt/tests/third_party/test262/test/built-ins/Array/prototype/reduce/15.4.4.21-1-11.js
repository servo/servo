// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: Array.prototype.reduce applied to Date object
---*/

function callbackfn(prevVal, curVal, idx, obj) {
  return obj instanceof Date;
}

var obj = new Date(0);
obj.length = 1;
obj[0] = 1;

assert(Array.prototype.reduce.call(obj, callbackfn, 1), 'Array.prototype.reduce.call(obj, callbackfn, 1) !== true');
