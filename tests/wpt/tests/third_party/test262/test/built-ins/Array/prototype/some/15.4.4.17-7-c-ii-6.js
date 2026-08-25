// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some - arguments to callbackfn are self consistent
---*/

var obj = {
  0: 11,
  length: 1
};
var thisArg = {};

function callbackfn() {
  return this === thisArg && arguments[0] === 11 && arguments[1] === 0 && arguments[2] === obj;
}

assert(Array.prototype.some.call(obj, callbackfn, thisArg), 'Array.prototype.some.call(obj, callbackfn, thisArg) !== true');
