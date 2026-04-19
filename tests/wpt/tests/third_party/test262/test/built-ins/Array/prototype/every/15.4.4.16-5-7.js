// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every - built-in functions can be used as thisArg
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return this === eval;
}

assert([11].every(callbackfn, eval), '[11].every(callbackfn, eval) !== true');
assert(accessed, 'accessed !== true');
