// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some - Number object can be used as thisArg
---*/

var objNumber = new Number();

function callbackfn(val, idx, obj) {
  return this === objNumber;
}

assert([11].some(callbackfn, objNumber), '[11].some(callbackfn, objNumber) !== true');
