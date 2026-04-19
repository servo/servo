// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: Array.prototype.reduce applied to the Math object
---*/

function callbackfn(prevVal, curVal, idx, obj) {
  return ('[object Math]' === Object.prototype.toString.call(obj));
}

Math.length = 1;
Math[0] = 1;

assert(Array.prototype.reduce.call(Math, callbackfn, 1), 'Array.prototype.reduce.call(Math, callbackfn, 1) !== true');
