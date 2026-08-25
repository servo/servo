// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: Array.prototype.reduce - 'callbackfn' is a function
---*/

var accessed = false;

function callbackfn(prevVal, curVal, idx, obj) {
  accessed = true;
  return curVal > 10;
}

assert.sameValue([11, 9].reduce(callbackfn, 1), false, '[11, 9].reduce(callbackfn, 1)');
assert(accessed, 'accessed !== true');
