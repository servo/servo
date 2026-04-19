// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some - the global object can be used as thisArg
---*/

var global = this;

function callbackfn(val, idx, obj) {
  return this === global;
}

assert([11].some(callbackfn, this), '[11].some(callbackfn, global) !== true');
