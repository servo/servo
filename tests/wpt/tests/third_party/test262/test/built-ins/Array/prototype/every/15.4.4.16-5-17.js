// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
es5id: 15.4.4.16-5-17
description: Array.prototype.every - the JSON object can be used as thisArg
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return this === JSON;
}

assert([11].every(callbackfn, JSON), '[11].every(callbackfn, JSON) !== true');
assert(accessed, 'accessed !== true');
