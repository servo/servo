// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some - the JSON object can be used as thisArg
---*/

function callbackfn(val, idx, obj) {
  return this === JSON;
}

assert([11].some(callbackfn, JSON), '[11].some(callbackfn, JSON) !== true');
