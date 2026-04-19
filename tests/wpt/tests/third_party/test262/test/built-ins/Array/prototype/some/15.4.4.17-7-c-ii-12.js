// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some - callbackfn is called with 3 formal parameter
---*/

function callbackfn(val, idx, obj) {
  return val > 10 && obj[idx] === val;
}

assert([9, 12].some(callbackfn), '[9, 12].some(callbackfn) !== true');
