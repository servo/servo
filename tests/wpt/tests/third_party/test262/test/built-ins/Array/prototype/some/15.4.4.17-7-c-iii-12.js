// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - return value of callbackfn is a number
    (value is NaN)
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return NaN;
}

assert.sameValue([11].some(callbackfn), false, '[11].some(callbackfn)');
