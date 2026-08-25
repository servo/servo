// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every - arguments to callbackfn are self consistent
---*/

var accessed = false;
var thisArg = {};
var obj = {
  0: 11,
  length: 1
};

function callbackfn() {
  accessed = true;
  return this === thisArg &&
    arguments[0] === 11 &&
    arguments[1] === 0 &&
    arguments[2] === obj;
}

assert(Array.prototype.every.call(obj, callbackfn, thisArg), 'Array.prototype.every.call(obj, callbackfn, thisArg) !== true');
assert(accessed, 'accessed !== true');
