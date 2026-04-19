// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - return value of callbackfn is a number
    (value is negative number)
---*/

function callbackfn(val, idx, obj) {
  return -5;
}

assert([11].some(callbackfn), '[11].some(callbackfn) !== true');
