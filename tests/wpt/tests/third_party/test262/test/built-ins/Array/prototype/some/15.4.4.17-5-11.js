// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some - String object can be used as thisArg
---*/

var objString = new String();

function callbackfn(val, idx, obj) {
  return this === objString;
}

assert([11].some(callbackfn, objString), '[11].some(callbackfn, objString) !== true');
