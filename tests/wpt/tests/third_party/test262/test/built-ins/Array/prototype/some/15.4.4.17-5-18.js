// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some - Error object can be used as thisArg
---*/

var objError = new RangeError();

function callbackfn(val, idx, obj) {
  return this === objError;
}

assert([11].some(callbackfn, objError), '[11].some(callbackfn, objError) !== true');
