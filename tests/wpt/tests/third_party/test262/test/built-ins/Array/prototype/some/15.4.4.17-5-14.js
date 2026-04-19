// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some - the Math object can be used as thisArg
---*/

function callbackfn(val, idx, obj) {
  return this === Math;
}

assert([11].some(callbackfn, Math), '[11].some(callbackfn, Math) !== true');
