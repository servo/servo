// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
es5id: 15.4.4.16-5-23
description: Array.prototype.every - number primitive can be used as thisArg
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return this.valueOf() === 101;
}

assert([11].every(callbackfn, 101), '[11].every(callbackfn, 101) !== true');
assert(accessed, 'accessed !== true');
