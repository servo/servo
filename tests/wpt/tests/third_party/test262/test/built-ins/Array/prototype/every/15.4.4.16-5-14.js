// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
es5id: 15.4.4.16-5-14
description: Array.prototype.every - the Math object can be used as thisArg
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return this === Math;
}

assert([11].every(callbackfn, Math), '[11].every(callbackfn, Math) !== true');
assert(accessed, 'accessed !== true');
