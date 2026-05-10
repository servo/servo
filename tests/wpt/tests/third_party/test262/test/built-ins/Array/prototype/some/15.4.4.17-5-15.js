// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some - Date object can be used as thisArg
---*/

var objDate = new Date(0);

function callbackfn(val, idx, obj) {
  return this === objDate;
}

assert([11].some(callbackfn, objDate), '[11].some(callbackfn, objDate) !== true');
