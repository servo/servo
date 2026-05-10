// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: Array.prototype.reduce applied to Number object
---*/

function callbackfn(prevVal, curVal, idx, obj) {
  return obj instanceof Number;
}

var obj = new Number(-128);
obj.length = 2;
obj[0] = 11;
obj[1] = 12;

assert(Array.prototype.reduce.call(obj, callbackfn, 1), 'Array.prototype.reduce.call(obj, callbackfn, 1) !== true');
