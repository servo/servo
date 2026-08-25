// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every applied to the Math object
---*/

function callbackfn(val, idx, obj) {
  return ('[object Math]' !== Object.prototype.toString.call(obj));
}

Math.length = 1;
Math[0] = 1;

assert.sameValue(Array.prototype.every.call(Math, callbackfn), false, 'Array.prototype.every.call(Math, callbackfn)');
